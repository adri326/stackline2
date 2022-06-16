use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::rc::Weak;

pub mod utils;
use utils::*;

pub mod tile;
use tile::*;

mod signal;
pub use signal::*;

pub struct World {
    tiles: Vec<RefCell<FullTile>>,
    width: NonZeroUsize,
    height: NonZeroUsize,

    signals: Vec<Weak<Signal>>,
}

impl World {
    pub fn empty(width: usize, height: usize) -> Option<Self> {
        // TODO: check that width * height is a valid usize

        Some(Self {
            width: width.try_into().ok()?,
            height: height.try_into().ok()?,
            tiles: vec![RefCell::new(FullTile::default()); width * height],

            signals: vec![],
        })
    }

    pub fn send_signal(
        &mut self,
        position: (usize, usize),
        signal: Signal,
    ) -> Option<Weak<Signal>> {
        let tile = self.get(position)?;
        let weak = {
            let mut guard = tile.try_borrow_mut().ok()?;
            guard.set_signal(signal)?
        };

        self.signals.push(weak.clone());

        Some(weak)
    }

    /// Returns `Some((x + Δx, y + Δy))` iff `(x + Δx, y + Δy)` is inside the world
    #[inline]
    pub fn offset(&self, position: (usize, usize), offset: (i8, i8)) -> Option<(usize, usize)> {
        if offset.0 < 0 && (-offset.0) as usize > position.0
            || offset.1 < 0 && (-offset.1) as usize > position.1
        {
            return None;
        }

        // TODO: check that position and position + offset are valid isize values
        let new_pos = (
            (position.0 as isize + offset.0 as isize) as usize,
            (position.1 as isize + offset.1 as isize) as usize,
        );

        if new_pos.0 < self.width.get() && new_pos.1 < self.height.get() {
            Some(new_pos)
        } else {
            None
        }
    }

    #[inline]
    pub fn get<'b>(&'b self, position: (usize, usize)) -> Option<&'b RefCell<FullTile>> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles.get(position.1 * self.width.get() + position.0)
    }

    #[inline]
    pub fn in_bounds(&self, position: (usize, usize)) -> bool {
        position.0 < self.width.get() && position.1 < self.height.get()
    }
}
