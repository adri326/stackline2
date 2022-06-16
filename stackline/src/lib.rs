use std::num::NonZeroUsize;
use std::rc::Weak;

pub mod utils;
use utils::*;

pub mod tile;
use tile::*;

mod signal;
pub use signal::*;

pub mod context;
use context::*;

pub struct World {
    panes: Vec<Pane>,
}

pub struct Pane {
    tiles: Vec<FullTile>,
    width: NonZeroUsize,
    height: NonZeroUsize,

    signals: Vec<Weak<Signal>>,
}

impl Pane {
    pub fn empty(width: usize, height: usize) -> Option<Self> {
        // TODO: check that width * height is a valid usize

        Some(Self {
            width: width.try_into().ok()?,
            height: height.try_into().ok()?,
            tiles: vec![FullTile::default(); width * height],

            signals: vec![],
        })
    }

    /// Returns `Some((x + Δx, y + Δy))` iff `(x + Δx, y + Δy)` is inside the world
    // SAFETY: this function may *not* access `self.signals`, `∀x, self.tiles[x].cell` or `∀x, self.tiles[x].signal`
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
    pub fn get<'b>(&'b self, position: (usize, usize)) -> Option<&'b FullTile> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles.get(position.1 * self.width.get() + position.0)
    }

    #[inline]
    pub fn get_mut<'b>(&'b mut self, position: (usize, usize)) -> Option<&'b mut FullTile> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles.get_mut(position.1 * self.width.get() + position.0)
    }

    #[inline]
    pub fn in_bounds(&self, position: (usize, usize)) -> bool {
        position.0 < self.width.get() && position.1 < self.height.get()
    }
}
