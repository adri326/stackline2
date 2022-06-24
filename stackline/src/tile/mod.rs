use super::*;
use dyn_clone::{clone_box, DynClone};

mod full;
pub use full::*;

// Generated rust file containing the AnyTile enum, which combines the structs implementing Tile found in tiles/.
// Its definition looks like this:
// #[derive(Clone, Debug)]
// #[enum_dispatch]
// pub enum AnyTile {
//     Wire(Wire),
//     Diode(Diode),
//     /* snip */
// }
//
// Note that all the implementing types will be read from the files in tiles/ and re-exported.
include!(concat!(env!("OUT_DIR"), "/anytile.rs"));

#[enum_dispatch(AnyTile)]
pub trait Tile: DynClone + std::fmt::Debug {
    /// Function to be called when the tile needs to be updated.
    #[inline]
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        context.next_state();
    }

    /// Should return true iff the tile accepts a signal travelling in `Direction`
    #[inline]
    #[allow(unused_variables)]
    fn accepts_signal(&self, direction: Direction) -> bool {
        true
    }

    /// Should draw itself on a [`TextSurface`].
    /// The `Tile` is allowed to draw outside of its coordinates, although doing so might cause glitches.
    // TODO: Use a 2d slice type
    #[inline]
    #[allow(unused_variables)]
    fn draw(&self, x: usize, y: usize, state: State, surface: &mut TextSurface) {
        // noop
    }
}

// #[derive(Debug)]
// pub struct AnyTile(Box<dyn Tile>);

// impl AnyTile {
//     #[inline]
//     pub fn new<T: Tile + 'static>(tile: T) -> Self {
//         Self(Box::new(tile))
//     }

//     #[inline]
//     pub fn update<'b>(&'b mut self, ctx: UpdateContext<'b>) {
//         self.0.update(ctx)
//     }

//     #[inline]
//     pub fn accepts_signal(&self, direction: Direction) -> bool {
//         self.0.accepts_signal(direction)
//     }

//     #[inline]
//     pub fn draw(&self, x: usize, y: usize, state: State, surface: &mut TextSurface) {
//         self.0.draw(x, y, state, surface);
//     }
// }

// impl Clone for AnyTile {
//     #[inline]
//     fn clone(&self) -> Self {
//         Self(clone_box(self.0.as_ref()))
//     }
// }

// impl<T: Tile + 'static> From<T> for AnyTile {
//     fn from(tile: T) -> AnyTile {
//         AnyTile(Box::new(tile))
//     }
// }

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
        };
    }

    #[macro_export]
    macro_rules! assert_signal {
        ( $pane:expr, $pos:expr ) => {{
            let guard = $pane
                .get($pos)
                .expect(&format!("Couldn't get tile at {:?}", $pos));
            let signal = guard.signal();
            assert!(signal.is_some());
            signal
        }};

        ( $pane:expr, $pos:expr, [ $( $data:expr ),* ] ) => {{
            let signal = assert_signal!($pane, $pos);
            // TODO: check that signal.data == data
        }};
    }

    #[macro_export]
    macro_rules! assert_no_signal {
        ( $pane:expr, $pos:expr) => {{
            let guard = $pane
                .get($pos)
                .expect(&format!("Couldn't get tile at {:?}", $pos));
            let signal = guard.signal();
            assert!(signal.is_none());
        }};
    }
}
