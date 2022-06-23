use super::*;
use std::ptr::NonNull;

/** Provides an interface between a [`Tile`] and its parent [`Pane`] during [`Tile::update`].
    All actions performed through `UpdateContext` will be executed *after* all the tiles have updated.

    # Design

    There are several factors that came into the design of `UpdateContext`:

    - All of its methods are considered hot-path code, which means that allocations must be kept at a minimum
    - All of the actions must be performed after all the tiles were updated
    - We need mutable access to the current tile, so that it can update its internal state

    As a result, there are a few oddities to take note of:

    - If a [`Signal`] was in the updated tile, then it will be moved into the `UpdateContext`.
      If you wish to put the signal back into its tile,
      then you will need to call [`keep`](UpdateContext::keep) or [`send`](UpdateContext::send).
      See [`take_signal`](UpdateContext::take_signal) for more information.
    - Most methods that

    # Example

    Here is how you would implement a simple "counter" tile:

    ```
    # use stackline::{*, tile::*, context::*};
    #
    #[derive(Clone, Debug)]
    pub struct CounterTile(usize);

    impl CounterTile {
        pub fn new() -> Self {
            Self(0)
        }
    }

    impl Tile for CounterTile {
        fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
            if let Some(signal) = ctx.take_signal() {
                // Update the internal state
                self.0 += 1;

                // Send the signal along: first, get the offset (Δx, Δy) associated with its direction and the tile at (x+Δx,y+Δy).
                // Note that the next three lines can be shortened to `ctx.accepts_direction(signal.direction())`
                if let Some((pos, tile)) = ctx.get_offset(signal.direction().into_offset()) {
                    // Then, check that `tile` accepts signals
                    if tile.accepts_signal(signal.direction()) {
                        // Finally, send the signal
                        ctx.send(pos, signal).unwrap_or_else(|| unreachable!());
                    }
                }
            }
        }
    }

    ```

    # Safety

    Because [`Tile::update`] requires a `&mut self` reference, the current [`Tile`] cannot be accessed through [`UpdateContext::get`].
    This structure stores the [`State`] and [`Signal`] of the [`FullTile`] containing the current tile, so these can be accessed nonetheless, and it is still possible and safe to call [`UpdateContext::send`] on the current position.
**/
pub struct UpdateContext<'a> {
    position: (usize, usize),
    pane: NonNull<Pane>,
    state: State,
    signal: Option<Signal>,
    commit: &'a mut UpdateCommit,
}

// SAFETY: self.pane.tiles[self.position].cell may not be accessed by any method of UpdateContext
impl<'a> UpdateContext<'a> {
    /// Creates a new UpdateContext
    /// Returns `None` if the tile was already updated or is empty
    pub(crate) fn new(
        pane: &'a mut Pane,
        position: (usize, usize),
        commit: &'a mut UpdateCommit,
    ) -> Option<(UpdateContext<'a>, &'a mut AnyTile)> {
        let mut tile = pane.get_mut(position)?;
        if tile.updated {
            return None;
        }
        tile.updated = true; // prevent duplicate updates
        commit.updates.push(position);

        let ptr: *mut AnyTile = &mut **(tile.get_mut().as_mut()?);

        let res = Self {
            position,
            state: tile.state(),
            signal: tile.take_signal(),
            pane: unsafe {
                NonNull::new_unchecked(&mut *pane)
            },
            commit,
        };

        // SAFETY: ptr is a valid pointer
        // SAFETY: aliasing is prevented by the invariants of UpdateContext
        Some((res, unsafe { &mut *ptr }))
    }

    /// Returns the position of the currently updated tile.
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// # #[derive(Clone, Debug)]
    /// # pub struct MyTile;
    /// # impl Tile for MyTile {
    /// fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
    ///     if let Some(mut signal) = ctx.take_signal() {
    ///         let (x, y) = ctx.position();
    ///         signal.push(Value::Number(y as f64));
    ///         signal.push(Value::Number(x as f64));
    ///     }
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    /// Returns the [signal](crate::FullTile::signal) of the currently updated tile.
    #[inline]
    pub fn signal<'b>(&'b self) -> Option<&'b Signal>
    where
        'a: 'b,
    {
        self.signal.as_ref()
    }

    /// Performs [`std::mem::take`] on the [signal](crate::FullTile::signal) of the currently updated tile.
    ///
    /// # Note
    ///
    /// Even if this function is not called, the current tile will still be stripped from its signal whenever it is updated.
    ///
    /// If you do want to keep the signal where it is, then you must either
    /// [`send`](UpdateContext::send) it to the current tile
    /// (which will only take effect at the end of the update phase),
    /// or [`keep`](UpdateContext::keep) it (which will take effect immediately but cannot be called together with `take_signal`).
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// #[derive(Clone, Debug)]
    /// pub struct PrintTile;
    ///
    /// impl Tile for PrintTile {
    ///     fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
    ///         if let Some(mut signal) = ctx.take_signal() {
    ///             println!("{:?}", signal.pop());
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn take_signal(&mut self) -> Option<Signal> {
        std::mem::take(&mut self.signal)
    }

    /// Returns the [`State`] of the current tile.
    #[inline]
    pub fn state(&self) -> State {
        self.state
    }

    /// Sets the state of the current tile to `state`.
    ///
    /// # Note
    ///
    /// The actions of this function will only be executed *after* all the tiles of the [`Pane`] were [`updated`](Pane::step).
    #[inline]
    pub fn set_state(&mut self, state: State) {
        self.commit.set_state(self.position, state);
    }

    /// Sets the state of the current tile to `state.next()`
    ///
    /// # Note
    ///
    /// The actions of this function will only be executed *after* all the tiles of the [`Pane`] were [`updated`](Pane::step).
    #[inline]
    pub fn next_state(&mut self) {
        self.commit.set_state(self.position, self.state.next());
    }

    /// Returns an immutable reference to the [FullTile] at `pos` in the current [Pane].
    /// Returns `None` if the tile is borrowed mutably, if it is the current tile or if it does not exist.
    #[inline]
    pub fn get<'b>(&'b self, pos: (usize, usize)) -> Option<&'b FullTile>
    where
        'a: 'b,
    {
        if self.position == pos {
            None
        } else {
            unsafe {
                // SAFETY: pos != self.position, thus self.pane[self.position].cell cannot be accessed
                self.pane.as_ref().get(pos)
            }
        }
    }

    /// Returns `Some((position.x + Δx, position.y + Δy))` iff `(x + Δx, y + Δy)` is inside the current pane.
    #[inline]
    pub fn offset(&self, offset: (i8, i8)) -> Option<(usize, usize)> {
        unsafe {
            // SAFETY: Pane::offset does not read `self.pane.cells`
            self.pane.as_ref().offset(self.position, offset)
        }
    }

    /// Returns `true` iff `(x, y)` is within the bounds of the current pane.
    #[inline]
    pub fn in_bounds(&self, pos: (usize, usize)) -> bool {
        unsafe {
            // SAFETY: Pane::in_bounds does not read `self.pane.cells`
            self.pane.as_ref().in_bounds(pos)
        }
    }

    /// Shortcut for calling both `ctx.offset(offset)` and `ctx.get(pos)`
    #[inline]
    pub fn get_offset<'b>(&'b self, offset: (i8, i8)) -> Option<((usize, usize), &'b FullTile)>
    where
        'a: 'b,
    {
        self.offset(offset)
            .and_then(|pos| self.get(pos).map(|tile| (pos, tile)))
    }

    /// Returns whether or not the tile at `pos` accepts a signal coming from `direction`.
    /// If the tile does not exist, then this function will return `false`.
    #[inline]
    pub fn accepts_signal(&self, pos: (usize, usize), direction: Direction) -> bool {
        match self.get(pos) {
            Some(tile) => tile.accepts_signal(direction),
            None => false,
        }
    }

    /// Returns `Some(pos)` iff `pos = (x + Δx, y + Δy)` is a valid position and `ctx.get(pos).accepts_signal(direction)`.
    ///
    /// This can be used as a shortcut to [`ctx.get_offset(direction.into_offset())`](UpdateContext::get_offset)
    /// paired with [`ctx.accepts_signal(new_pos, direction)`](UpdateContext::accepts_signal).
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// # #[derive(Clone, Debug)]
    /// # pub struct MyTile;
    /// # impl Tile for MyTile {
    /// fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
    ///     if let Some(signal) = ctx.take_signal() {
    ///         if let Some(pos) = ctx.accepts_direction(Direction::Down) {
    ///             ctx.send(pos, signal);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn accepts_direction(&self, direction: Direction) -> Option<(usize, usize)> {
        let (pos, tile) = self.get_offset(direction.into_offset())?;
        if tile.accepts_signal(direction) {
            Some(pos)
        } else {
            None
        }
    }

    /// Sends a signal to be stored in a cell (may be the current one), overriding any signal that was in that cell.
    ///
    /// Returns true if the signal was stored in a cell, false otherwise.
    /// The target cell's state will be set to `Active` if it received the signal.
    /// The signal's `position` will be set to `pos`.
    ///
    /// # Note
    ///
    /// The actions of this function will only be executed *after* all the tiles of the [`Pane`] were [`updated`](Pane::step).
    /// See [`keep`](UpdateContext::keep) for a variant of this method that takes effect immediately.
    pub fn send(&mut self, pos: (usize, usize), mut signal: Signal) -> Option<()> {
        signal.set_position(pos);

        if !self.in_bounds(pos) {
            return None;
        }

        self.commit.send(pos, signal);

        Some(())
    }

    /// Stores the current signal back in the current tile, guaranteeing that it will stay there for
    /// this update cycle. See [`take_signal`](UpdateContext::take_signal) for more information.
    ///
    /// This method differs from [`send`](UpdateContext::send), as it takes action immediately.
    /// The signal may also not be modified, as it would otherwise break the guarantees of [`Pane::step`].
    ///
    /// This function will [`std::mem::take`] the signal stored in `UpdateContext`, similar to [`take_signal`](UpdateContext::take_signal).
    /// If you wish to modify or send copies of the signal, then you will need to call [`signal`](UpdateContext::signal) beforehand and make
    /// clones of the signal before calling `keep`.
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// #[derive(Clone, Debug)]
    /// pub struct StorageTile {};
    ///
    /// impl Tile for StorageTile {
    ///     fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
    ///         if ctx.signal().is_some() {
    ///             ctx.keep();
    ///         }
    ///         // If we weren't to do this, then the signal would get dropped here
    ///     }
    /// }
    /// ```
    pub fn keep(&mut self) {
        unsafe {
            // SAFETY: we only access self.pane[self.position].signal, not self.pane[self.position].cell
            self.pane.as_mut().get_mut(self.position).unwrap_or_else(|| unreachable!()).set_signal(
                std::mem::take(&mut self.signal)
            );
        }
    }
}

/// Temporarily holds a list of actions to be made on a given Pane, which should be [applied](UpdateCommit::apply)
/// after every tile was updated.
pub(crate) struct UpdateCommit {
    states: Vec<(usize, usize, State)>,
    signals: Vec<(usize, usize, Option<Signal>)>,
    updates: Vec<(usize, usize)>,
}

impl UpdateCommit {
    pub(crate) fn new() -> Self {
        Self {
            states: Vec::new(),
            signals: Vec::new(),
            updates: Vec::new(),
        }
    }

    fn send(&mut self, pos: (usize, usize), signal: Signal) {
        self.signals.push((pos.0, pos.1, Some(signal)));
    }

    fn set_state(&mut self, pos: (usize, usize), state: State) {
        self.states.push((pos.0, pos.1, state));
    }

    pub(crate) fn apply(self, pane: &mut Pane) {
        for (x, y) in self.updates {
            if let Some(tile) = pane.get_mut((x, y)) {
                tile.updated = false;
            }
        }

        for (x, y, state) in self.states {
            if let Some(tile) = pane.get_mut((x, y)) {
                tile.set_state(state);
            }
        }

        for (x, y, signal) in self.signals {
            let push_signal = if let Some(tile) = pane.get_mut((x, y)) {
                tile.set_signal(signal);
                tile.set_state(State::Active);
                // For some reason std::mem::drop(tile) isn't enough here
                true
            } else {
                false
            };

            if push_signal {
                pane.signals.push((x, y));
            }
        }
    }
}
