use crate::{chunk::Chunk, chunks::Chunks, common::*, iter::Iter, windows::Windows};

/// The trait adds methods for concurrent processing on any type that can be borrowed as a slice.
pub trait ConcurrentSlice<T> {
    /// Splits the slice-like data into two sub-slices, divided at specified index.
    ///
    /// # Panics
    /// The method panics if the index is out of bound.
    fn concurrent_split_at(self, index: usize) -> (Chunk<Self, T>, Chunk<Self, T>)
    where
        Self: 'static + AsMut<[T]> + Sized + Send,
        T: 'static + Send,
    {
        unsafe {
            let data = Arc::new(self);
            let ptr = Arc::as_ptr(&data) as *mut Self;
            let slice: &mut [T] = ptr.as_mut().unwrap().as_mut();
            let lslice = NonNull::new_unchecked(&mut slice[0..index] as *mut [T]);
            let rslice = NonNull::new_unchecked(&mut slice[index..] as *mut [T]);

            (
                Chunk {
                    data: data.clone(),
                    slice: lslice,
                },
                Chunk {
                    data,
                    slice: rslice,
                },
            )
        }
    }

    /// Returns an iterator of roughly fixed-sized chunks of the slice.
    ///
    /// Each chunk has `chunk_size` elements, expect the last chunk maybe shorter
    /// if there aren't enough elements.
    ///
    /// The yielded chunks maintain a global reference count. Each chunk refers to
    /// a mutable and exclusive sub-slice, enabling concurrent processing on input data.
    ///
    /// # Panics
    /// The method panics if `chunk_size` is zero and slice length is not zero.
    fn concurrent_chunks(mut self, chunk_size: usize) -> Chunks<Self, T>
    where
        Self: 'static + AsMut<[T]> + Sized + Send,
        T: 'static + Send,
    {
        let len = self.as_mut().len();
        assert!(
            len == 0 || chunk_size > 0,
            "chunk_size must be positive for non-empty slice"
        );

        Chunks {
            index: 0,
            chunk_size,
            end: len,
            data: Arc::new(self),
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator with roughly `division` length of roughly fixed-sized chunks of the slice.
    ///
    /// The chunk size is determined by `division`. The last chunk maybe shorter if
    /// there aren't enough elements. If `division` is `None`, it defaults to
    /// the number of system processors.
    ///
    /// # Panics
    /// The method panics if `division` is zero and slice length is not zero.
    fn concurrent_chunks_by_division(
        mut self,
        division: impl Into<Option<usize>>,
    ) -> Chunks<Self, T>
    where
        Self: 'static + AsMut<[T]> + Sized + Send,
        T: 'static + Send,
    {
        let len = self.as_mut().len();
        let division = division.into().unwrap_or_else(num_cpus::get);

        let chunk_size = if len == 0 {
            0
        } else {
            assert!(
                division > 0,
                "division must be positive for non-empty slice, but get zero"
            );
            (len + division - 1) / division
        };

        Chunks {
            index: 0,
            chunk_size,
            end: len,
            data: Arc::new(self),
            _phantom: PhantomData,
        }
    }

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
    /// The windows are contiguous and overlap. If the slice is shorter than size,
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
