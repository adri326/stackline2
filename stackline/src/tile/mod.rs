use super::*;
use dyn_clone::{clone_box, DynClone};
use std::rc::Rc;

mod wire;
pub use wire::*;

/** Represents a tile that may be empty and may have a signal. The tile may only have a signal if it isn't empty.
Cloning a `FullTile` results in a `FullTile` that does not have any signal.

## Invariants

- `self.cell.is_none() -> self.signal.is_none()`
- `self.accepts_signal() -> self.cell.is_some()`

**/
#[derive(Clone, Debug)]
pub struct FullTile {
    cell: Option<AnyTile>,
    signal: Option<Rc<Signal>>,
}

impl FullTile {
    pub fn new(cell: Option<AnyTile>) -> Self {
        Self { cell, signal: None }
    }

    pub fn update(&mut self, world: &mut World, pos: (usize, usize)) {
        match self.cell {
            Some(ref mut tile) => {
                tile.update(world, &mut self.signal, pos);
            }
            None => {}
        }
    }

    pub fn accepts_signal(&self, direction: Direction) -> bool {
        match self.cell {
            Some(ref tile) => tile.accepts_signal(direction),
            None => false,
        }
    }

    /// Returns Some(signal) iff self.cell.is_some()
    pub fn set_signal(&mut self, signal: Signal) -> Option<Weak<Signal>> {
        if self.cell.is_some() {
            let rc = Rc::new(signal);
            let weak = Rc::downgrade(&rc);
            self.signal = Some(rc);
            Some(weak)
        } else {
            None
        }
    }
}

impl Default for FullTile {
    fn default() -> Self {
        Self::new(None)
    }
}

pub trait Tile: DynClone + std::fmt::Debug {
    fn update(&mut self, world: &mut World, signal: &mut Option<Rc<Signal>>, pos: (usize, usize));

    /// Should return true iff the tile accepts a signal travelling in `Direction`
    fn accepts_signal(&self, _direction: Direction) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct AnyTile(Box<dyn Tile>);

impl AnyTile {
    fn update(&mut self, world: &mut World, signal: &mut Option<Rc<Signal>>, pos: (usize, usize)) {
        self.0.update(world, signal, pos)
    }

    fn accepts_signal(&self, direction: Direction) -> bool {
        self.0.accepts_signal(direction)
    }
}

impl Clone for AnyTile {
    fn clone(&self) -> Self {
        Self(clone_box(self.0.as_ref()))
    }
}
