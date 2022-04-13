//! The crate extends slice-type types with methods for concurrent processing.

mod chunk;
mod chunk_mut;
mod chunks;
mod chunks_mut;
mod common;

pub use chunk::*;
pub use chunk_mut::*;
pub use chunks::*;
pub use chunks_mut::*;
