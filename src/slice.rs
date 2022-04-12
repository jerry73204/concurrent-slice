use crate::{chunk::Chunk, chunks::Chunks, common::*, iter::Iter, windows::Windows};

/// The trait adds methods for concurrent processing on any type that can be borrowed as a slice.
pub trait ConcurrentSlice<T> {
    /// Returns an iterator of owned references to each element of the slice.
    fn owning_iter(self) -> Iter<Self, T>
    where
        Self: 'static + Send + Deref + CloneStableAddress,
        Self::Target: AsRef<[T]>,
    {
        let owner = OwningRef::new(self).map(|me| me.as_ref());
        Iter { owner, index: 0 }
    }

    /// Returns an iterator of owned windows of length `size`.
    /// The windows are contiguous and overlapped. If the slice is shorter than size,
    /// the iterator returns no values.
    fn owning_windows(self, size: usize) -> Windows<Self, T>
    where
        Self: 'static + Send + Deref + CloneStableAddress,
        Self::Target: AsRef<[T]>,
    {
        assert!(size > 0, "size must be positive");
        let owner = OwningRef::new(self).map(|me| me.as_ref());

        Windows {
            owner,
            size,
            index: 0,
        }
    }
}

impl<S, T> ConcurrentSlice<T> for S {}
