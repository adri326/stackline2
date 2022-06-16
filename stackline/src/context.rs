use std::marker::PhantomData;
use std::rc::Rc;
use super::*;

/// An UpdateContext is created for every tile update during the "update" phase,
/// and it contains the necessary data for a tile to update its internal state.
///
/// During the update phase, a tile may only access itself mutably, through the mutable
/// reference it was initially passed through its [`update`](Tile::update) method.
/// All accesses to other tiles and all signals must be done immutably.
///
/// It thus *cannot* access itself through this context structure, although it may read its
/// signal here.
/// It *can* access the other tiles and their signals immutably.

// SAFETY: `pane[position].cell` is borrow mutably, while a pointer to the original Pane is kept;
// thus, no other reference to `pane[position].cell` may be done
pub struct UpdateContext<'a> {
    pub position: (usize, usize),
    pane: *const Pane,

    phantom: PhantomData<&'a Pane>,
}

impl<'a> UpdateContext<'a> {
    #[inline]
    pub(crate) fn new(pane: &'a mut Pane, position: (usize, usize)) -> Option<(Self, &'a mut AnyTile)> {

        let ptr: *const Pane = &*pane;
        let tile = pane.get_mut(position)?.get_mut()?;

        let res = Self {
            position,
            pane: ptr,
            phantom: PhantomData
        };

        Some((res, tile))
    }

    #[inline]
    pub fn signal<'b>(&'b self) -> Option<&'b Rc<Signal>> where 'a: 'b {
        let pane = unsafe { self.pane() };

        // SAFETY: `pane[position].signal` is not borrowed mutably
        pane.get(self.position).unwrap().signal()
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
/// It may access the other tiles immutably, but it cannot access the other signals.

// SAFETY: this structures ensures that it has exlusive, mutable access to `∀x, pane[x].signal` and `pane.signals`.
// Other parts of `pane` may be accessed and returned immutably.
pub struct TransmitContext<'a> {
    pub position: (usize, usize),
    pane: *mut Pane,

    phantom: PhantomData<&'a mut Pane>,
}

impl<'a> TransmitContext<'a> {
    pub(crate) fn new(pane: &'a mut Pane, position: (usize, usize)) -> Option<(Self, &'a AnyTile, Rc<Signal>)> {
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

    /// Returns an immutable reference to the [tile](AnyTile) at `pos` in the current [Pane].
    /// Returns `None` if that tile does not exist.
    #[inline]
    pub fn get<'b>(&'b self, pos: (usize, usize)) -> Option<&'b AnyTile> where 'a: 'b {
        let pane = unsafe { self.pane() };

        // SAFETY: we only return pane[pos].cell
        pane.get(pos)?.get()
    }

    /// Sends a signal to be stored in a cell (may be the current one), the signal overrides that of the other cell
    /// Returns true if the signal was stored in a cell, false otherwise
    pub fn send<'b>(&'b self, pos: (usize, usize), signal: Signal) -> bool where 'a: 'b {
        // SAFETY: we do not return any reference to any data borrowed in this function
        // SAFETY: we only access `pane[pos].signal` and `pane.signals`
        let pane = unsafe { self.pane_mut() };

        match pane.get_mut(pos) {
            Some(ref mut tile) => {
                if let Some(weak) = tile.set_signal(signal) {
                    pane.signals.push(weak);
                    true
                } else {
                    false
                }
            }
            _ => false
        }
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
