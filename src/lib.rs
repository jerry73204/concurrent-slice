//! The crate extends slice-type types with methods for concurrent processing.

mod chunk;
mod chunks;
mod common;
mod iter;
mod slice;
mod windows;

pub use chunk::*;
pub use chunks::*;
pub use iter::*;
pub use slice::*;
pub use windows::*;
