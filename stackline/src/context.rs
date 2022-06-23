use super::*;

/** Provides an interface between a [`Tile`] and its parent [`Pane`] during [`Tile::update`].
    All actions performed through `UpdateContext` will be executed *after* all the tiles have updated.

    ## Design

    There are several factors that came into the design of [`UpdateContext`]:

    - all of its methods are considered hot-path code, which means that allocations must be kept at a minimum
    - all of the actions must be performed after all the tiles were updated
    - we need mutable access to the current tile, so that it can update its internal state

    ## Example

    Here is how you would implement a simple "counter" tile:

    ```
    # use stackline::{*, tile::*, context::*};

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

                // Send the signal along: first, get the offset (Δx, Δy) associated with its direction and the tile at (x+Δx,y+Δy)
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

    ## Safety

    Because [`Tile::update`] requires a `&mut self` reference, the current [`Tile`] cannot be accessed through [`UpdateContext::get`].
    This structure stores the [`State`] and [`Signal`] of the [`FullTile`] containing the current tile, so these can be accessed nonetheless, and it is still possible and safe to call [`UpdateContext::send`] on the current position.
**/
pub struct UpdateContext<'a> {
    position: (usize, usize),
    pane: &'a Pane,
    state: State,
    signal: Option<Signal>,
    commit: &'a mut UpdateCommit,
}

// SAFETY: self.pane.tiles[self.position] may not be accessed by any method of UpdateContext
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
            pane,
            commit,
        };

        // SAFETY: ptr is a valid pointer
        // SAFETY: aliasing is prevented by the invariants of UpdateContext
        Some((res, unsafe { &mut *ptr }))
    }

    /// Returns the position of the currently updated tile.
    ///
    /// ## Example
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

    /// Performs [`std::mem::take`] on the signal of the currently updated tile.
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
    #[inline]
    pub fn set_state(&mut self, state: State) {
        self.commit.set_state(self.position, state);
    }

    /// Sets the state of the current tile to `state.next()`
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
            self.pane.get(pos)
        }
    }

    /// Returns `Some((position.x + Δx, position.y + Δy))` iff `(x + Δx, y + Δy)` is inside the pane
    #[inline]
    pub fn offset(&self, offset: (i8, i8)) -> Option<(usize, usize)> {
        self.pane.offset(self.position, offset)
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

    /// Returns `Some(pos)` iff `pos = (x + Δx, y + Δy)` is a valid position and `self.get(pos).accepts_signal(direction)`
    #[inline]
    pub fn accepts_direction(&self, direction: Direction) -> Option<(usize, usize)> {
        let (pos, tile) = self.get_offset(direction.into_offset())?;
        if tile.accepts_signal(direction) {
            Some(pos)
        } else {
            None
        }
    }

    /// Sends a signal to be stored in a cell (may be the current one), the signal overrides that of the other cell
    /// Returns true if the signal was stored in a cell, false otherwise.
    /// The target cell's state will be set to `Active` if it received the signal.
    /// The signal's `position` will be set to `pos`.
    pub fn send(&mut self, pos: (usize, usize), mut signal: Signal) -> Option<()> {
        signal.set_position(pos);

        if !self.pane.in_bounds(pos) {
            return None;
        }

        self.commit.send(pos, signal);

        Some(())
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
