use crate::{chunk_mut::ChunkMut, common::*};

pub use sized_chunks_mut::*;
mod sized_chunks_mut {
    use super::*;

    /// An iterator that yields [chunks](Chunk).
    #[derive(Debug)]
    pub struct SizedChunksMut<'a, S, T>
    where
        S: AsMut<[T]> + Send + Sync + 'a,
        T: Send + Sync,
    {
        pub(crate) index: usize,
        pub(crate) chunk_size: usize,
        pub(crate) end: usize,
        pub(crate) owner: Arc<S>,
        pub(crate) _phantom: PhantomData<&'a T>,
    }

    impl<'a, S, T> SizedChunksMut<'a, S, T>
    where
        S: AsMut<[T]> + Send + Sync + 'a,
        T: Send + Sync,
    {
        pub fn into_arc_owner(self) -> Arc<S> {
            self.owner
        }

        /// Tries to recover the owning data.
        ///
        /// The method succeeds if the referencing chunk iterator and all chunks are dropped.
        /// Otherwise, it returns the guard intact.
        pub fn try_unwrap_owner(self) -> Result<S, Self> {
            let Self {
                index,
                chunk_size,
                end,
                owner,
                ..
            } = self;

            Arc::try_unwrap(owner).map_err(|owner| Self {
                index,
                chunk_size,
                end,
                owner,
                _phantom: PhantomData,
            })
        }

        /// Gets the reference count on the owning data.
        pub fn ref_count(&self) -> usize {
            Arc::strong_count(&self.owner)
        }
    }

    impl<'a, S, T> Iterator for SizedChunksMut<'a, S, T>
    where
        S: AsMut<[T]> + Send + Sync + 'a,
        T: Send + Sync,
    {
        type Item = ChunkMut<'a, S, T>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.end {
                return None;
            }

            let start = self.index;
            let end = cmp::min(start + self.chunk_size, self.end);
            self.index = end;

            let owner = self.owner.clone();

            let slice = unsafe {
                let ptr = Arc::as_ptr(&owner) as *mut S;
                let slice: &mut [T] = ptr.as_mut().unwrap().as_mut();
                NonNull::new_unchecked(&mut slice[start..end] as *mut [T])
            };

            Some(ChunkMut {
                owner,
                slice,
                _phantom: PhantomData,
            })
        }
    }
}

pub use even_chunks_mut::*;
mod even_chunks_mut {
    use super::*;

    /// An iterator that yields [chunks](Chunk).
    #[derive(Debug)]
    pub struct EvenChunksMut<'a, S, T>
    where
        S: AsMut<[T]> + Send + Sync + 'a,
        T: Send + Sync,
    {
        pub(crate) base_chunk_size: usize,
        pub(crate) index: usize,
        pub(crate) long_end: usize,
        pub(crate) short_end: usize,
        pub(crate) owner: Arc<S>,
        pub(crate) _phantom: PhantomData<&'a T>,
    }

    impl<'a, S, T> EvenChunksMut<'a, S, T>
    where
        S: AsMut<[T]> + Send + Sync + 'a,
        T: Send + Sync,
    {
        pub fn into_arc_owner(self) -> Arc<S> {
            self.owner
        }

        /// Tries to recover the owning data.
        ///
        /// The method succeeds if the referencing chunk iterator and all chunks are dropped.
        /// Otherwise, it returns the guard intact.
        pub fn try_unwrap_owner(self) -> Result<S, Self> {
            let Self {
                index,
                base_chunk_size,
                long_end,
                short_end,
                owner,
                ..
            } = self;

            Arc::try_unwrap(owner).map_err(|owner| Self {
                index,
                base_chunk_size,
                long_end,
                short_end,
                owner,
                _phantom: PhantomData,
            })
        }

        /// Gets the reference count on the owning data.
        pub fn ref_count(&self) -> usize {
            Arc::strong_count(&self.owner)
        }
    }

    impl<'a, S, T> Iterator for EvenChunksMut<'a, S, T>
    where
        S: AsMut<[T]> + Send + Sync + 'a,
        T: Send + Sync,
    {
        type Item = ChunkMut<'a, S, T>;

        fn next(&mut self) -> Option<Self::Item> {
            debug_assert!(self.long_end <= self.short_end);

            let chunk_size = if self.index < self.long_end {
                self.base_chunk_size + 1
            } else if self.index < self.short_end {
                self.base_chunk_size
            } else {
                debug_assert!(self.index == self.short_end);
                return None;
            };

            let start = self.index;
            let end = start + chunk_size;
            self.index = end;
            debug_assert!(
                (start < self.long_end && end <= self.long_end)
                    || (start < self.short_end && end <= self.short_end)
            );

            let owner = self.owner.clone();
            let slice = unsafe {
                let ptr = Arc::as_ptr(&owner) as *mut S;
                let slice: &mut [T] = ptr.as_mut().unwrap().as_mut();
                NonNull::new_unchecked(&mut slice[start..end] as *mut [T])
            };

            Some(ChunkMut {
                owner,
                slice,
                _phantom: PhantomData,
            })
        }
    }
}
