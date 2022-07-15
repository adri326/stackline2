/*! This module contains the [`Tile`] trait, which defines common behavior for all tiles in stackline.
 * It also contains [`FullTile`], which holds the state and signal of the different tiles alongside a [`Tile`] wrapped in [`AnyTile`].
 *
 * Lastly, [`AnyTile`] is an enum containing all of the [`Tile`] instances that were discovered in the `tiles/` directory.
 * See [its documentation](AnyTile) for more information on the discovery process.
*/
use super::*;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

mod full;
pub use full::*;

#[cfg(test)]
#[macro_use]
mod macros;

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

/// The `Tile` trait defines shared behavior for every tile in the language.
///
/// Tiles are the building block of the language, and take the role of the instructions of the language.
/// With this trait, you may define the following behaviors of instructions:
/// - what it should do if it receives a [`Signal`], with [`Tile::update`]
/// - when [`Signal`s](Signal) can be sent to it, with [`Tile::accepts_signal`]
/// - how it should be displayed on the screen, with [`Tile::draw`]
///
/// # Example
///
/// Let's start by implementing a basic [`Tile`], which simply forwards any incomming [`Signal`] to its right.
/// Create a file in the `tiles/` folder containing the following
///
/// ```no_run
/// // First, import the needed types. Because we are writing files
/// // that will be part of the "stackline" crate, we have to import them using `crate`:
/// # /*
/// use crate::prelude::*;
/// use crate::tile::prelude::*;
/// # */
/// # // Doctests don't allow us to use `crate` to refer to the current crate
/// # use stackline::prelude::*;
/// # use stackline::tile::prelude::*;
///
/// // Tiles must implement Clone, Debug, Serialize and Deserialize
/// #[derive(Clone, Debug, Serialize, Deserialize)]
/// pub struct MyTile {
///     // This is where your tile can store its internal state.
///     // For this tile, we don't need any!
/// }
///
/// impl MyTile {
///     // It's a good idea to provide a constructor for writing tests
///     pub fn new() -> Self {
///         MyTile {}
///     }
/// }
///
/// impl Tile for MyTile {
///     // The update method is where we will put the logic of our simple tile
///     fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
///         // Check if we have a signal
///         if let Some(signal) = context.take_signal() {
///             // We do have a signal, so we will forward it to the tile on our right.
///
///             // First, get the coordinates of the tile to our right:
///             if let Some(right_position) = context.offset((1, 0)) {
///                 // Then, send the signal! We also need to tell `send`
///                 // that the signal is moving to the right.
///                 context.send(right_position, Direction::Right, signal);
///             }
///         }
///
///         // If we are Active, become Dormant
///         // If we are Dormant, become Idle
///         if context.state() != State::Idle {
///             context.next_state();
///         }
///     }
///
///     // The Tile trait provides a default implementations for the other methods,
///     // which satisfy our needs.
/// }
///
/// // Lastly, we should write unit tests for our code
/// #[cfg(test)]
/// mod test {
///     use super::*;
///
///     #[test]
///     fn test_my_tile() {
///         use crate::tile::Wire;
///         use crate::utils::Orientation::*;
///
///         // We create a Pane containing our tile, surrounded by wires:
///         //  |
///         // -X-
///         //  |
///         // With X our tile
///         let mut pane = test_tile_setup!(3, 3, [
///             (), Wire::new(Vertical), (),
///             Wire::new(Horizontal), MyTile::new(), Wire::new(Horizontal),
///             (), Wire::new(Vertical), ()
///         ]);
///
///         // Send signals coming from the top, bottom and left of our tile,
///         // And check that they are all forwarded to the right
///         let signals = [
///             (0, 1, Direction::Right),
///             (1, 0, Direction::Down),
///             (1, 2, Direction::Up),
///         ];
///
///         for (x, y, dir) in signals {
///             test_set_signal!(pane, (x, y), dir);
///             pane.step();
///
///             // Our tile should accept the signal
///             assert_signal!(pane, (1, 1));
///
///             pane.step();
///             // Our tile should have moved the signal to the right
///             assert_no_signal!(pane, (1, 1));
///             assert_signal!(pane, (2, 1));
///
///             pane.step();
///             pane.step();
///         }
///     }
/// }
/// ```
#[enum_dispatch(AnyTile)]
pub trait Tile: std::clone::Clone + std::fmt::Debug + Serialize + for<'d> Deserialize<'d> {
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
    ///
    /// By default, draws a single character at the tile's location, determined by [`Tile::draw_simple`]
    // TODO: Use a 2d slice type
    #[inline]
    #[allow(unused_variables)]
    fn draw(&self, x: i32, y: i32, state: State, surface: &mut TextSurface) {
        if let (Ok(x), Ok(y)) = (x.try_into(), y.try_into()) {
            surface.set(x, y, self.draw_simple(state));
        }
    }

    /// Used by the default implementation of `draw`,
    #[inline]
    #[allow(unused_variables)]
    fn draw_simple(&self, state: State) -> TextChar {
        TextChar::default()
    }
}

pub mod prelude {
    pub use crate::prelude::*;
    pub use crate::signal::Signal;
    pub use crate::text::*;
    pub use crate::tile::{AnyTile, FullTile};
    pub use crate::utils::State;

    pub use serde::{Deserialize, Serialize};
}
