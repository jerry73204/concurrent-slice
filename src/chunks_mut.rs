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

pub use iter_mut::*;
mod iter_mut {
    use super::*;

    /// The iterator returned from [owning_iter()](crate::slice::ConcurrentSlice::owning_iter).
    #[derive(Debug)]
    pub struct IterMut<'a, S, T>
    where
        S: Sync + Send + AsMut<[T]> + 'a,
    {
        pub(crate) owner: Arc<S>,
        pub(crate) index: usize,
        pub(crate) end: usize,
        pub(crate) _phantom: PhantomData<&'a T>,
    }

    impl<'a, S, T> IterMut<'a, S, T>
    where
        S: Sync + Send + AsMut<[T]> + 'a,
    {
        pub fn try_unwrap_owner(self) -> Result<S, Self> {
            let Self {
                owner, index, end, ..
            } = self;

            Arc::try_unwrap(owner).map_err(|owner| Self {
                owner,
                index,
                end,
                _phantom: PhantomData,
            })
        }
    }

    impl<'a, S, T> Iterator for IterMut<'a, S, T>
    where
        S: Sync + Send + AsMut<[T]> + 'a,
    {
        type Item = OwnedMut<S, T>;

        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
                if self.index == self.end {
                    return None;
                }

                let owner_ptr = Arc::as_ptr(&self.owner) as *mut S;
                let slice: &mut [T] = owner_ptr.as_mut().unwrap().as_mut();
                let value = &mut slice[self.index];
                let ptr = NonNull::new_unchecked(value as *mut T);
                self.index += 1;

                Some(OwnedMut {
                    owner: self.owner.clone(),
                    ptr,
                })
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = self.end - self.index;
            (len, Some(len))
        }
    }

    impl<'a, S, T> ExactSizeIterator for IterMut<'a, S, T> where S: Sync + Send + AsMut<[T]> + 'a {}
}

pub use owned_mut::*;
pub mod owned_mut {
    use super::*;

    pub struct OwnedMut<S, T> {
        pub(crate) owner: Arc<S>,
        pub(crate) ptr: NonNull<T>,
    }

    impl<S, T> OwnedMut<S, T> {
        pub fn try_unwrap_owner(self) -> Result<S, Self> {
            let Self { owner, ptr } = self;
            Arc::try_unwrap(owner).map_err(|owner| Self { owner, ptr })
        }
    }

    impl<S, T> AsRef<T> for OwnedMut<S, T> {
        fn as_ref(&self) -> &T {
            unsafe { self.ptr.as_ref() }
        }
    }

    impl<S, T> AsMut<T> for OwnedMut<S, T> {
        fn as_mut(&mut self) -> &mut T {
            unsafe { self.ptr.as_mut() }
        }
    }

    impl<S, T> Deref for OwnedMut<S, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { self.ptr.as_ref() }
        }
    }

    impl<S, T> DerefMut for OwnedMut<S, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { self.ptr.as_mut() }
        }
    }

    impl<S, T> Debug for OwnedMut<S, T>
    where
        T: Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.deref()).fmt(f)
        }
    }

    impl<S, T> PartialEq for OwnedMut<S, T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            *self.deref() == *other.deref()
        }
    }

    impl<S, T> Eq for OwnedMut<S, T> where T: Eq {}

    impl<S, T> PartialOrd for OwnedMut<S, T>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            unsafe { (self.deref()).partial_cmp(other.ptr.as_ref()) }
        }
    }

    impl<S, T> Ord for OwnedMut<S, T>
    where
        T: Ord,
    {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            unsafe { (self.deref()).cmp(other.ptr.as_ref()) }
        }
    }

    impl<S, T> Hash for OwnedMut<S, T>
    where
        T: Hash,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.deref().hash(state);
        }
    }
}
