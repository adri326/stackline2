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
    state: State,
}

// SAFETY: should not implement Tile
impl FullTile {
    pub fn new(cell: Option<AnyTile>) -> Self {
        Self {
            cell,
            signal: None,
            state: State::default()
        }
    }

    // SAFETY: must not access `self.signal`
    pub fn accepts_signal(&self, direction: Direction) -> bool {
        match self.cell {
            Some(ref tile) => self.state.accepts_signal() && tile.accepts_signal(direction),
            None => false,
        }
    }

    /// Returns Some(signal) iff self.cell.is_some()
    pub(crate) fn set_signal(&mut self, signal: Signal) -> Option<Weak<Signal>> {
        if self.cell.is_some() {
            let rc = Rc::new(signal);
            let weak = Rc::downgrade(&rc);
            self.signal = Some(rc);
            Some(weak)
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
    pub fn signal<'b>(&'b self) -> Option<&'b Rc<Signal>> {
        self.signal.as_ref()
    }

    #[inline]
    pub fn take_signal(&mut self) -> Option<Rc<Signal>> {
        std::mem::take(&mut self.signal)
    }

    // SAFETY: may only access `self.state`
    #[inline]
    pub fn state(&self) -> State {
        self.state
    }

    // SAFETY: may only access `self.state`
    #[inline]
    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    #[inline]
    pub fn next_state(&mut self) {
        self.state = self.state.next();
    }

    pub fn into_raw_mut<'b>(&'b mut self) -> (&'b mut Option<AnyTile>, &'b mut Option<Rc<Signal>>, &'b mut State) {
        (&mut self.cell, &mut self.signal, &mut self.state)
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

pub trait Tile: DynClone + std::fmt::Debug {
    /// Function to be called when the tile needs to update its internal state.
    /// During the "update" phase, the tile may access its signal and the other tiles immutably.
    #[inline]
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        context.next_state();
    }

    /// Function that will be called if the tile has a signal.
    fn transmit<'b>(&'b self, signal: Rc<Signal>, context: TransmitContext<'b>);

    /// Should return true iff the tile accepts a signal travelling in `Direction`
    #[inline]
    fn accepts_signal(&self, _direction: Direction) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct AnyTile(Box<dyn Tile>);

impl AnyTile {
    #[inline]
    pub fn new<T: Tile + 'static>(tile: T) -> Self {
        Self(Box::new(tile))
    }

    #[inline]
    pub fn update<'b>(&'b mut self, ctx: UpdateContext<'b>) {
        self.0.update(ctx)
    }

    #[inline]
    pub fn transmit<'b>(&'b self, signal: Rc<Signal>, context: TransmitContext<'b>) {
        self.0.transmit(signal, context)
    }

    #[inline]
    pub fn accepts_signal(&self, direction: Direction) -> bool {
        self.0.accepts_signal(direction)
    }
}

impl Clone for AnyTile {
    #[inline]
    fn clone(&self) -> Self {
        Self(clone_box(self.0.as_ref()))
    }
}

#[cfg(test)]
mod crate_macros {
    #[macro_export]
    macro_rules! test_tile_setup {
        ( $width:expr, $height:expr, [ $( $x:expr ),* ] ) => {{
            assert!($width > 0);
            assert!($height > 0);
            let mut pane = Pane::empty($width, $height).unwrap();
            let mut index = 0;

            $(
                {
                    let x = index % $width;
                    let y = index / $width;
                    *pane.get_mut((x, y)).unwrap() = FullTile::from($x);
                    index += 1;
                }
            )*

            assert!(index == $width * $height);

            pane
        }}
    }

    #[macro_export]
    macro_rules! test_set_signal {
        ( $pane:expr, $pos:expr, $dir:expr ) => {
            $pane.set_signal($pos, Signal::empty($pos, $dir)).unwrap();
        }
    }

    #[macro_export]
    macro_rules! assert_signal {
        ( $pane:expr, $pos:expr ) => {{
            let signal = $pane.get($pos).expect(&format!("Couldn't get tile at {:?}", $pos)).signal();
            assert!(signal.is_some());
            signal
        }};

        ( $pane:expr, $pos:expr, [ $( $data:expr ),* ] ) => {{
            let signal = assert_signal!($pane, $pos);
            // TODO: check that signal.data == data
        }}
    }

    #[macro_export]
    macro_rules! assert_no_signal {
        ( $pane:expr, $pos:expr) => {{
            let signal = $pane.get($pos).expect(&format!("Couldn't get tile at {:?}", $pos)).signal();
            assert!(signal.is_none());
        }}
    }
}
