use super::*;

/** Represents a tile that may be empty and may have a signal. The tile may only have a signal if it isn't empty.
Cloning a `FullTile` results in a `FullTile` that does not have any signal.

## Invariants

- `self.cell.is_none() -> self.signal.is_none()`
- `self.accepts_signal() -> self.cell.is_some()`

**/
#[derive(Clone, Debug)]
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
    pub(crate) fn set_signal(&mut self, signal: Option<Signal>) -> Option<()> {
        if self.cell.is_some() {
            self.signal = signal;
            Some(())
        } else {
            None
        }
    }

    /// Returns the internal state of this full tile
    #[inline]
    pub fn get<'b>(&'b self) -> Option<&'b AnyTile> {
        self.cell.as_ref()
    }

    /// Returns a mutable reference to the internal state of this tile
    #[inline]
    pub fn get_mut<'b>(&'b mut self) -> Option<&'b mut AnyTile> {
        self.cell.as_mut()
    }

    /// Returns the signal of this tile
    #[inline]
    pub fn signal<'b>(&'b self) -> Option<&'b Signal> {
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
    pub fn draw(&self, x: usize, y: usize, surface: &mut TextSurface) {
        match self.cell {
            Some(ref cell) => cell.draw(x, y, self.state, surface),
            None => {}
        }
    }
}

impl Default for FullTile {
    #[inline]
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T: Tile + 'static> From<T> for FullTile {
    #[inline]
    fn from(tile: T) -> Self {
        Self::new(Some(AnyTile::new(tile)))
    }
}

impl From<()> for FullTile {
    #[inline]
    fn from(_empty: ()) -> Self {
        Self::new(None)
    }
}
