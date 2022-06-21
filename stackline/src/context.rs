use super::*;
use std::cell::{Ref, RefMut};

/** Provides an interface between a [`Tile`] and its parent [`Pane`] during [`Tile::update`].
    All actions performed through `UpdateContext` will be executed *after* all the tiles have updated.
**/
pub struct UpdateContext<'a> {
    position: (usize, usize),
    pane: &'a Pane,
    state: State,
    signal: Option<Signal>,
    commit: &'a mut UpdateCommit
}

impl<'a> UpdateContext<'a> {
    /// Returns `None` if the tile was already updated or is empty
    pub(crate) fn new(pane: &'a Pane, position: (usize, usize), commit: &'a mut UpdateCommit) -> Option<(Self, RefMut<'a, AnyTile>)> {
        let mut guard = pane.get_mut(position)?;
        if guard.updated {
            return None
        }
        guard.updated = true; // prevent duplicate updates
        commit.updates.push(position);

        let (tile, signal, state) = guard.get_raw_mut();

        if tile.is_none() {
            return None
        }

        let res = Self {
            position,
            pane,
            state: *state,
            signal: std::mem::take(signal),
            commit
        };

        Some((res, RefMut::map(guard, |tile| tile.get_mut().unwrap_or_else(|| unreachable!()))))
    }

    /// Returns the position of the currently updated tile.
    #[inline]
    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    /// Returns the [signal](crate::FullTile::signal) of the currently updated tile.
    #[inline]
    pub fn signal<'b>(&'b self) -> Option<&'b Signal> where 'a: 'b {
        self.signal.as_ref()
    }

    /// Performs [`std::mem::take`] on the signal of the currently updated tile.
    #[inline]
    pub fn take_signal(&mut self) -> Option<Signal> {
        std::mem::take(&mut self.signal)
    }

    // #[inline]
    // pub fn set_signal(&mut self, signal: Option<Signal>) {
    //     *self.signal = signal;
    // }

    /// Returns the state of the current tile.
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
    /// Returns `None` if the tile is borrowed mutably or if it does not exist.
    #[inline]
    pub fn get<'b>(&'b self, pos: (usize, usize)) -> Option<Ref<'b, FullTile>> where 'a: 'b {
        self.pane.get(pos)
    }

    /// Returns `Some((position.x + Δx, position.y + Δy))` iff `(x + Δx, y + Δy)` is inside the pane
    #[inline]
    pub fn offset(&self, offset: (i8, i8)) -> Option<(usize, usize)> {
        self.pane.offset(self.position, offset)
    }

    /// Shortcut for calling both `ctx.offset(offset)` and `ctx.get(pos)`
    #[inline]
    pub fn get_offset<'b>(&'b self, offset: (i8, i8)) -> Option<((usize, usize), Ref<'b, FullTile>)> where 'a: 'b {
        self.offset(offset).and_then(|pos| self.get(pos).map(|tile| (pos, tile)))
    }

    /// Returns whether or not the tile at `pos` accepts a signal coming from `direction`.
    /// If the tile does not exist, then this function will return `false`.
    #[inline]
    pub fn accepts_signal(&self, pos: (usize, usize), direction: Direction) -> bool {
        match self.pane.get(pos) {
            Some(tile) => tile.accepts_signal(direction),
            None => false
        }
    }

    /// Sends a signal to be stored in a cell (may be the current one), the signal overrides that of the other cell
    /// Returns true if the signal was stored in a cell, false otherwise.
    /// The target cell's state will be set to `Active` if it received the signal.
    /// The signal's `position` will be set to `pos`.
    pub fn send(&mut self, pos: (usize, usize), mut signal: Signal) -> Option<()> {
        signal.set_position(pos);

        if !self.pane.in_bounds(pos) {
            return None
        }

        self.commit.send(pos, signal);

        Some(())
    }
}

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
            if let Some(mut tile) = pane.get_mut((x, y)) {
                tile.updated = false;
            }
        }

        for (x, y, state) in self.states {
            if let Some(mut tile) = pane.get_mut((x, y)) {
                tile.set_state(state);
            }
        }

        for (x, y, signal) in self.signals {
            let push_signal = if let Some(mut tile) = pane.get_mut((x, y)) {
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
