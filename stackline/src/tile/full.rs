use super::*;
use serde::{Deserialize, Serialize};

/** Represents a tile that may be empty and may have a signal. The tile may only have a signal if it isn't empty.
Cloning a `FullTile` results in a `FullTile` that does not have any signal.

## Invariants

- `self.cell.is_none() -> self.signal.is_none()`
- `self.accepts_signal() -> self.cell.is_some()`

**/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullTile {
    cell: Option<AnyTile>,
    signal: Option<Signal>,
    state: State,
    pub(crate) updated: bool,
}

// NOTE: should not implement Tile
impl FullTile {
    pub fn new(cell: Option<AnyTile>) -> Self {
        Self {
            cell,
            signal: None,
            state: State::default(),
            updated: false,
        }
    }

    pub fn accepts_signal(&self, direction: Direction) -> bool {
        match self.cell {
            Some(ref tile) => self.state.accepts_signal() && tile.accepts_signal(direction),
            None => false,
        }
    }

    /// Returns `Some` iff self.cell.is_some()
    pub fn set_signal(&mut self, signal: Option<Signal>) -> Option<()> {
        if self.cell.is_some() {
            self.signal = signal;
            Some(())
        } else {
            None
        }
    }

    /// Returns the internal state of this full tile
    #[inline]
    pub fn get(&self) -> Option<&AnyTile> {
        self.cell.as_ref()
    }

    /// Returns a mutable reference to the internal state of this tile
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut AnyTile> {
        self.cell.as_mut()
    }

    /// Returns the signal of this tile
    #[inline]
    pub fn signal(&self) -> Option<&Signal> {
        self.signal.as_ref()
    }

    #[inline]
    pub fn take_signal(&mut self) -> Option<Signal> {
        std::mem::take(&mut self.signal)
    }

    #[inline]
    pub fn state(&self) -> State {
        self.state
    }

    #[inline]
    pub fn set_state(&mut self, state: State) {
        if self.cell.is_some() {
            self.state = state
        }
    }

    #[inline]
    pub fn next_state(&mut self) {
        self.state = self.state.next();
    }

    /// Draws itself on a [`TextSurface`] at `(x, y)`.
    /// If the tile is empty, does nothing
    pub fn draw(&self, x: i32, y: i32, surface: &mut TextSurface) {
        if let Some(cell) = &self.cell {
            cell.draw(x, y, self.state, surface);
        }
    }
}

impl Default for FullTile {
    #[inline]
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T: Tile + 'static> From<T> for FullTile
where
    AnyTile: From<T>,
{
    #[inline]
    fn from(tile: T) -> Self {
        Self::new(Some(AnyTile::from(tile)))
    }
}

impl From<()> for FullTile {
    #[inline]
    fn from(_empty: ()) -> Self {
        Self::new(None)
    }
}

// TODO: enum for <AnyTile as TryInto<T: Tile>>
// TODO: local trait for conversion to inner tiles
