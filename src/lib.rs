//! The crate extends slice-type types with methods for concurrent processing.

mod chunk;
mod chunk_mut;
mod chunks;
mod chunks_mut;
mod common;
mod iter;
mod slice;
mod windows;

pub use chunk::*;
pub use chunk_mut::*;
pub use chunks::*;
pub use chunks_mut::*;
pub use iter::*;
pub use slice::*;
pub use windows::*;
