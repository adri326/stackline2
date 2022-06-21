use std::marker::PhantomData;
use super::*;

/** An `UpdateContext` is created for every tile update during the "update" phase,
and it contains the necessary data for a tile to update its internal state.

During the update phase, a tile may only access itself mutably, through the mutable
reference it was initially passed through its [`update`](Tile::update) method.
All accesses to other tiles and all signals must be done immutably.

It thus *cannot* access itself through this context structure, although it may read its
signal here.
It *can* access the other tiles and their signals immutably.

## Examples

This type is most commonly found when implementing [`Tile::update`]:

```
# use stackline::{*, tile::Tile, utils::State, context::*};
#
#[derive(Clone, Debug)]
struct MyTile(u8);

impl Tile for MyTile {
    fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
        // Counts the number of active neighbors
        let mut active_neighbors = 0;

        for dy in -1..1 {
            for dx in -1..1 {
                let offset = (dx, dy);
                if offset == (0, 0) {
                    continue;
                }

                // Get tile next to us:
                if let Some((_position, tile)) = ctx.get_offset(offset) {
                    if tile.state() == State::Active {
                        active_neighbors += 1;
                    }
                }
            }
        }

        self.0 = active_neighbors;
        ctx.next_state(); // Become dormant
    }
#
#    fn transmit<'b>(&'b self, signal: Signal, ctx: TransmitContext<'b>) {}
}
```

**/

// SAFETY: `pane[position].cell` is borrow mutably, while a pointer to the original Pane is kept;
// thus, no other reference to `pane[position].cell` may be done
pub struct UpdateContext<'a> {
    position: (usize, usize),
    pane: *const Pane,
    state: &'a mut State,

    phantom: PhantomData<&'a Pane>,
}

impl<'a> UpdateContext<'a> {
    /// Creates a new context, returning the only mutable reference to `pane[position].cell` and the `UpdateContext`.
    #[inline]
    pub(crate) fn new(pane: &'a mut Pane, position: (usize, usize)) -> Option<(Self, &'a mut AnyTile)> {

        let ptr: *const Pane = &*pane;
        let (tile, _signal, state) = pane.get_mut(position)?.into_raw_mut();

        let res = Self {
            position,
            pane: ptr,
            state,
            phantom: PhantomData
        };

        Some((res, tile.as_mut()?))
    }

    /// Returns the position of the currently updated tile.
    #[inline]
    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    /// Returns the [signal](crate::FullTile::signal) of the currently updated tile.
    #[inline]
    pub fn signal<'b>(&'b self) -> Option<&'b Signal> where 'a: 'b {
        let pane = unsafe { self.pane() };

        // SAFETY: `pane[position].signal` is not borrowed mutably
        pane.get(self.position).unwrap().signal()
    }

    /// Returns the state of the current tile.
    #[inline]
    pub fn state(&self) -> State {
        *self.state
    }

    /// Sets the state of the current tile to `state`.
    #[inline]
    pub fn set_state(&mut self, state: State) {
        *self.state = state;
    }

    /// Sets the state of the current tile to `state.next()`
    #[inline]
    pub fn next_state(&mut self) {
        *self.state = self.state.next();
    }

    /// Returns an immutable reference to the [FullTile] at `pos` in the current [Pane].
    /// Returns `None` if the tile is the current tile (see [UpdateContext]) or if it does not exist.
    #[inline]
    pub fn get<'b>(&'b self, pos: (usize, usize)) -> Option<&'b FullTile> where 'a: 'b {
        let pane = unsafe { self.pane() };

        // SAFETY: we only access `pane[pos]` if `position != pos`
        if pos != self.position {
            pane.get(pos)
        } else {
            None
        }
    }

    /// Returns `Some((position.x + Δx, position.y + Δy))` iff `(x + Δx, y + Δy)` is inside the pane
    #[inline]
    pub fn offset(&self, offset: (i8, i8)) -> Option<(usize, usize)> {
        let pane = unsafe { self.pane() };

        // SAFETY: pane.offset does not access pane[position].cell
        pane.offset(self.position, offset)
    }

    /// Shortcut for calling both `ctx.offset(offset)` and `ctx.get(pos)`
    #[inline]
    pub fn get_offset<'b>(&'b self, offset: (i8, i8)) -> Option<((usize, usize), &'b FullTile)> where 'a: 'b {
        self.offset(offset).and_then(|pos| self.get(pos).map(|tile| (pos, tile)))
    }

    // SAFETY: `self.pane` originates from a `&'a mut Pane`,
    // guaranteeing that no accesses may be done outside of ours.
    // No access to `pane[position].cell` may be done!
    #[inline]
    unsafe fn pane<'b>(&'b self) -> &'b Pane {
        &*self.pane
    }
}

/// An UpdateContext is created for every tile update during the "transmit" phase,
/// and it contains the necessary data for a tile to transmit its internal signal to other tiles.
///
/// During this phase, the tile may access itself through an immutable borrow and its signal through an owned reference.
/// It *can* access the other tiles immutably, but it *cannot* access the other signals.
/// It *can* read and modify any tile's state.

// SAFETY: this structures ensures that it has exlusive, mutable access to `∀x, pane[x].signal, pane[x].state` and `pane.signals`.
// Other parts of `pane` may be accessed and returned immutably.
pub struct TransmitContext<'a> {
    position: (usize, usize),
    pane: *mut Pane,

    phantom: PhantomData<&'a mut Pane>,
}

impl<'a> TransmitContext<'a> {
    pub(crate) fn new(pane: &'a mut Pane, position: (usize, usize)) -> Option<(Self, &'a AnyTile, Signal)> {
        let ptr: *mut Pane = &mut *pane;
        // SAFETY: no mutable accesses to `∀x, pane[x].cell` are made by `TransmitContext`
        let tile: &AnyTile = unsafe {
            (*ptr).get(position).unwrap().get()?
        };
        let signal = pane.get_mut(position)?.take_signal()?;

        let res = Self {
            position,
            pane: ptr,

            phantom: PhantomData
        };

        Some((res, tile, signal))
    }

    /// Returns the position of the current tile
    #[inline]
    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    /// Returns an immutable reference to the [tile](AnyTile) at `pos` in the current [Pane].
    /// Returns `None` if that tile does not exist.
    #[inline]
    pub fn get<'b>(&'b self, pos: (usize, usize)) -> Option<&'b AnyTile> where 'a: 'b {
        let pane = unsafe { self.pane() };

        // SAFETY: we only return pane[pos].cell
        pane.get(pos)?.get()
    }

    /// Returns the state of the tile at `pos` in the current [Pane]
    #[inline]
    pub fn get_state(&self, pos: (usize, usize)) -> Option<State> {
        let pane = unsafe { self.pane() };

        // SAFETY: we only return a copy of pane[pos].state
        Some(pane.get(pos)?.state())
    }

    /// Sets the state of the tile at `pos` in the current [Pane]
    #[inline]
    pub fn set_state(&self, pos: (usize, usize), state: State) -> Option<()> {
        let pane = unsafe { self.pane_mut() };

        // SAFETY: there are no borrows of pane[pos].state
        pane.get_mut(pos)?.set_state(state);
        Some(())
    }

    /// Returns whether or not the tile at `pos` accepts a signal coming from `direction`.
    /// If the tile does not exist, then this function will return `false`.
    #[inline]
    pub fn accepts_signal(&self, pos: (usize, usize), direction: Direction) -> bool {
        let pane = unsafe { self.pane() };

        // SAFETY: does not access `pane[pos].signal`
        match pane.get(pos) {
            Some(tile) => tile.accepts_signal(direction),
            None => false
        }
    }

    /// Sends a signal to be stored in a cell (may be the current one), the signal overrides that of the other cell
    /// Returns true if the signal was stored in a cell, false otherwise.
    /// The target cell's state will be set to `Active` if it received the signal.
    /// The signal's `position` will be set to `pos`.
    pub fn send<'b>(&'b mut self, pos: (usize, usize), mut signal: Signal) -> Option<()> where 'a: 'b {
        // SAFETY: we do not return any reference to any data borrowed in this function
        // SAFETY: we only access `pane[pos].signal`, `pane[pos].state` and `pane.signals`
        let pane = unsafe { self.pane_mut() };

        signal.set_position(pos);

        pane.set_signal(pos, signal)?;
        // SAFETY: we only access `pane[pos].state`
        pane.get_mut(pos).unwrap_or_else(|| unreachable!()).set_state(State::Active);

        Some(())
    }

    /// Returns `Some((position.x + Δx, position.y + Δy))` iff `(x + Δx, y + Δy)` is inside the pane
    #[inline]
    pub fn offset(&self, offset: (i8, i8)) -> Option<(usize, usize)> {
        let pane = unsafe { self.pane() };

        // SAFETY: pane.offset does not access pane[position].signal or pane.signals
        pane.offset(self.position, offset)
    }

    // SAFETY: `self.pane` originates from a `&'a mut Pane`,
    // guaranteeing that no accesses may be done outside of ours.
    #[inline]
    unsafe fn pane<'b>(&'b self) -> &'b Pane {
        &*self.pane
    }

    // SAFETY: `self.pane` originates from a `&'a mut Pane`,
    // guaranteeing that no accesses may be done outside of ours.
    #[inline]
    unsafe fn pane_mut<'b>(&'b self) -> &'b mut Pane {
        &mut *self.pane
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_update_exclusivity() {
        // Check that UpdateContext does not allow any other reference to `tiles[position].cell` to be made
        let mut pane = Pane::empty(4, 4).unwrap();
        let mut tile = FullTile::from(Wire::new(Orientation::Any));
        tile.set_signal(Signal::empty((1, 2), Direction::Up)).unwrap();
        *pane.get_mut((1, 2)).unwrap() = tile;

        let (ctx, _tile) = UpdateContext::new(&mut pane, (1, 2)).unwrap();

        assert_eq!(ctx.position(), (1, 2));

        let tile_self: Option<&FullTile> = ctx.get((1, 2)); // The FullTile may not be read
        assert!(tile_self.is_none());

        assert!(ctx.signal().is_some()); // Our Signal may be read, though
    }

    #[test]
    fn test_transmit() {
        let mut pane = Pane::empty(4, 4).unwrap();
        let mut tile = FullTile::from(Wire::new(Orientation::Any));
        tile.set_signal(Signal::empty((1, 2), Direction::Up)).unwrap();
        *pane.get_mut((1, 2)).unwrap() = tile;

        let (ctx, _tile, _signal) = TransmitContext::new(&mut pane, (1, 2)).unwrap();

        assert_eq!(ctx.position(), (1, 2));

        let tile_self: Option<&AnyTile> = ctx.get((1, 2));
        assert!(tile_self.is_some()); // We may read our AnyTile, as they are under an immutable borrow

        // Check that the signal was dropped
        std::mem::drop(ctx);
        assert!(pane.get((1, 2)).unwrap().signal().is_none());
    }
}
