use std::num::NonZeroUsize;
use std::rc::Weak;

mod signal;
pub use signal::*;

mod pane;
pub use pane::*;

pub mod utils;
use utils::*;

pub mod tile;
use tile::*;

pub mod context;
use context::*;

pub struct World {
    panes: Vec<Pane>,
}
