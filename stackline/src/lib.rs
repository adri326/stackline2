/*! # Stackline v2

`Stackline v2` is the successor of [stackline](https://github.com/adri326/stackline), an esoteric language inspired by [Wireworld](https://mathworld.wolfram.com/WireWorld.html) and [ORCA](https://github.com/hundredrabbits/Orca).

This library is the rust implementation of the core logic of the language.


*/

use std::num::NonZeroUsize;

pub mod signal;
use signal::*;

pub mod pane;
use pane::*;

pub mod utils;
use utils::*;

pub mod tile;
use tile::*;

pub mod context;
use context::*;

pub mod text;
use text::*;

pub struct World {
    panes: Vec<Pane>,
}

pub mod prelude {
    pub use crate::pane::Pane;
    pub use crate::World;

    pub use crate::text::{TextChar, TextSurface};

    pub use crate::context::UpdateContext;
    pub use crate::signal::{Signal, Value};
    pub use crate::tile::Tile;
    pub use crate::utils::*;
}
