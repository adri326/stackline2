use super::*;
use veccell::{VecRef, VecRefMut};

/** Provides an interface between a [`Tile`] and its parent [`Pane`] during [`Tile::update`].
    All actions performed through `UpdateContext` will be executed *after* all the tiles have updated.

    # Design

    There are several factors that came into the design of `UpdateContext`:

    - All of its methods are considered hot-path code, which means that allocations must be kept at a minimum
    - All of the actions must be performed after all the tiles were updated
    - We need mutable access to the current tile, so that it can update its internal state

    As a result, there are a few oddities to take note of:

    - If a [`Signal`] was in the updated tile, then it will be *moved* into the `UpdateContext`.
      If you wish to put the signal back into its tile,
      then you will need to call [`keep`](UpdateContext::keep) or [`send`](UpdateContext::send).
      See [`take_signal`](UpdateContext::take_signal) for more information.
    - Most methods that modify the state of a tile will instead store the modified state in a temporary buffer
      and apply the modifications after every tile was updated. The only exception to this is [`UpdateContext::keep`].

    # Example

    Here is how you would implement a simple "counter" tile:

    ```
    # use stackline::tile::prelude::*;
    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

                // Send the signal along: first, get the offset (Δx, Δy) associated with its direction and the position at (x+Δx,y+Δy).
                if let Some(pos) = ctx.offset(signal.direction().into_offset()) {
                    // Finally, send the signal
                    let _ = ctx.send(pos, signal.direction(), signal);
                }
            }
        }
    }

    ```
**/
pub struct UpdateContext<'a> {
    position: (usize, usize),
    pane: &'a Pane,
    state: State,
    signal: Option<Signal>,
    commit: &'a mut UpdateCommit,
}

// SAFETY: self.pane.tiles[self.position].cell may not be accessed by any method of UpdateContext
impl<'a> UpdateContext<'a> {
    /// Creates a new UpdateContext
    /// Returns `None` if the tile was already updated, is empty or does not exist.
    pub(crate) fn new<'b>(
        pane: &'b Pane,
        position: (usize, usize),
        commit: &'a mut UpdateCommit,
    ) -> Option<(UpdateContext<'a>, VecRefMut<'b, FullTile>)>
    where
        'b: 'a, // 'b ⊇ 'a
    {
        let mut tile = pane.borrow_mut(position)?;
        if tile.updated {
            return None;
        }
        tile.updated = true; // prevent duplicate updates
        commit.updates.push(position);

        let res = Self {
            position,
            state: tile.state(),
            signal: tile.take_signal(),
            pane,
            commit,
        };

        Some((res, tile))
    }

    /// Returns the position of the currently updated tile.
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::tile::prelude::*;
    /// # #[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

    /// Returns the [`width`](Pane::width) of the current [`Pane`].
    #[inline]
    pub fn width(&self) -> NonZeroUsize {
        self.pane.width()
    }

    /// Returns the [`height`](Pane::height) of the current [`Pane`].
    #[inline]
    pub fn height(&self) -> NonZeroUsize {
        self.pane.height()
    }

    /// Returns a reference to the [signal](crate::FullTile::signal) of the currently updated tile.
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
    /// # use stackline::tile::prelude::*;
    /// #[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
        self.state = state;
        self.commit.set_state(self.position, self.state);
    }

    /// Sets the state of the current tile to `state.next()`
    ///
    /// # Note
    ///
    /// The actions of this function will only be executed *after* all the tiles of the [`Pane`] were [`updated`](Pane::step).
    #[inline]
    pub fn next_state(&mut self) {
        self.state = self.state.next();
        self.commit.set_state(self.position, self.state);
    }

    /// Returns an immutable reference to the [FullTile] at `pos` in the current [Pane].
    /// Returns `None` if the tile is borrowed mutably, if it is the current tile or if it does not exist.
    #[inline]
    pub fn get<'b>(&'b self, pos: (usize, usize)) -> Option<VecRef<'b, FullTile>>
    where
        'a: 'b,
    {
        if self.position == pos {
            None
        } else {
            self.pane.get(pos)
        }
    }

    /// Returns `Some((position.x + Δx, position.y + Δy))` iff `(x + Δx, y + Δy)` is inside the current pane.
    #[inline]
    pub fn offset(&self, offset: (i8, i8)) -> Option<(usize, usize)> {
        self.pane.offset(self.position, offset)
    }

    /// Returns `true` iff `(x, y)` is within the bounds of the current pane.
    #[inline]
    pub fn in_bounds(&self, position: (usize, usize)) -> bool {
        self.pane.in_bounds(position)
    }

    /// Shortcut for calling both `ctx.offset(offset)` and `ctx.get(pos)`
    #[inline]
    pub fn get_offset<'b>(
        &'b self,
        offset: (i8, i8),
    ) -> Option<((usize, usize), VecRef<'b, FullTile>)>
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
    /// # use stackline::tile::prelude::*;
    /// # #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    /// # pub struct MyTile;
    /// # impl Tile for MyTile {
    /// fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
    ///     if let Some(signal) = ctx.take_signal() {
    ///         if let Some(pos) = ctx.offset(Direction::Down.into_offset()) {
    ///             let _ = ctx.send(pos, Direction::Down, signal);
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

    // TODO: return Result
    /// Sends a signal to be stored in a cell (may be the current one), overriding any signal that was in that cell.
    ///
    /// Returns Some(()) if the signal was stored in a cell, None otherwise.
    /// The target cell's state will be set to `Active` if it received the signal.
    /// The signal's `position` will be set to `position`.
    ///
    /// You should change the `direction` of the signal before calling this function.
    ///
    /// # Note
    ///
    /// The actions of this function will only be executed *after* all the tiles of the [`Pane`] were [`updated`](Pane::step).
    /// See [`keep`](UpdateContext::keep) for a variant of this method that takes effect immediately.
    pub fn force_send(
        &mut self,
        position: (usize, usize),
        mut signal: Signal,
    ) -> Result<(), SendError> {
        if !self.in_bounds(position) {
            return Err(SendError(signal));
        }

        signal.set_position(position);

        self.commit.send(position, signal);

        Ok(())
    }

    /// Sends a signal to `position` if there is a tile at `position` that will accept our signal.
    /// Sets the signal direction to `direction` and its position to `position`.
    ///
    /// # Note
    ///
    /// The actions of this function will only be executed *after* all the tiles of the [`Pane`] were [`updated`](Pane::step).
    /// See [`keep`](UpdateContext::keep) for a variant of this method that takes effect immediately.
    pub fn send(
        &mut self,
        position: (usize, usize),
        direction: Direction,
        signal: Signal,
    ) -> Result<(), SendError> {
        if self.accepts_signal(position, direction) {
            let original_direction = signal.direction();
            self.force_send(position, signal.moved(direction))
                .map_err(|e| SendError(e.0.moved(original_direction)))
        } else {
            Err(SendError(signal))
        }
    }

    /// Sends a signal to another [`Pane`] in the world.
    /// If the tile at `coordinates = (pane, x, y)` does not exist, then the signal will be lost.
    pub fn send_outbound(&mut self, coordinates: (String, usize, usize), signal: Signal) {
        self.commit.send_outbound(coordinates, signal);
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
    /// If `take_signal` or `keep` are called before this functions, then it will do nothing.
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::tile::prelude::*;
    /// #[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
        if let Some(signal) = std::mem::take(&mut self.signal) {
            self.commit.set_self_signal(Some(signal));
        }
    }

    /// Executes an arbitrary function after all the updates.
    pub fn callback<F: for<'c> FnOnce(&'c mut Pane) + 'static>(&mut self, callback: F) {
        self.commit.set_callback(Box::new(callback));
    }
}

pub struct SendError(pub Signal);

impl std::fmt::Debug for SendError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("SendError")
            .field("signal", &"Signal {{...}}")
            .finish()
    }
}

impl std::fmt::Display for SendError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Couldn't send signal!")
    }
}

pub type UpdateCommitCallback = Box<dyn for<'c> FnOnce(&'c mut Pane) + 'static>;

/// Temporarily holds a list of actions to be made on a given Pane, which should be [applied](UpdateCommit::apply)
/// after every tile was updated.
pub(crate) struct UpdateCommit {
    states: Vec<(usize, usize, State)>,
    signals: Vec<(usize, usize, Option<Signal>)>,
    updates: Vec<(usize, usize)>,
    callbacks: Vec<UpdateCommitCallback>,
    outbound_signals: Vec<((String, usize, usize), Signal)>,

    self_signal: Option<Signal>,
}

impl UpdateCommit {
    pub(crate) fn new() -> Self {
        Self {
            states: Vec::new(),
            signals: Vec::new(),
            updates: Vec::new(),
            callbacks: Vec::new(),
            outbound_signals: Vec::new(),

            self_signal: None,
        }
    }

    fn send(&mut self, pos: (usize, usize), signal: Signal) {
        self.signals.push((pos.0, pos.1, Some(signal)));
    }

    fn set_state(&mut self, pos: (usize, usize), state: State) {
        self.states.push((pos.0, pos.1, state));
    }

    fn set_self_signal(&mut self, signal: Option<Signal>) {
        self.self_signal = signal;
    }

    fn set_callback(&mut self, callback: UpdateCommitCallback) {
        self.callbacks.push(callback);
    }

    fn send_outbound(&mut self, coordinates: (String, usize, usize), signal: Signal) {
        self.outbound_signals.push((coordinates, signal));
    }

    pub(crate) fn apply(self, pane: &mut Pane) -> PaneResult {
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

        for callback in self.callbacks {
            (callback)(pane);
        }

        PaneResult {
            outbound_signals: self.outbound_signals,
        }
    }

    /// Applies transformations on a FullTile before the end of the update phase
    #[inline]
    pub(crate) fn apply_immediate(&mut self, tile: &mut FullTile) {
        if let Some(signal) = std::mem::take(&mut self.self_signal) {
            tile.set_signal(Some(signal));
        }
    }
}
