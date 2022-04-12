use crate::{chunks::Chunks, common::*};

/// A mutable sub-slice reference-counted reference to a slice-like data.
#[derive(Debug)]
pub struct Chunk<S, T>
where
    S: AsMut<[T]>,
{
    pub(super) data: Arc<S>,
    pub(super) slice: NonNull<[T]>,
}

impl<S, T> Chunk<S, T>
where
    S: AsMut<[T]>,
{
    pub fn new(owner: S) -> Self {
        unsafe {
            let data = Arc::new(owner);
            let ptr = Arc::as_ptr(&data) as *mut S;
            let slice: &mut [T] = ptr.as_mut().unwrap().as_mut();
            let slice = NonNull::new_unchecked(slice as *mut [T]);
            Self { data, slice }
        }
    }

    /// Splits the chunk into two sub-chunks, divided at specified index.
    ///
    /// # Panics
    /// The method panics if the index is out of bound.
    pub fn split_at(mut self, index: usize) -> (Chunk<S, T>, Chunk<S, T>)
    where
        T: 'static + Send,
    {
        unsafe {
            let data = self.data;
            let slice: &mut [T] = self.slice.as_mut();
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

    /// Returns an iterator of fixed-sized chunks of the refencing slice.
    ///
    /// Each chunk has `chunk_size` elements, expect the last chunk maybe shorter
    /// if there aren't enough elements.
    ///
    /// The yielded chunks maintain a global reference count on owning data. Each chunk refers to
    /// a mutable and exclusive sub-slice, enabling concurrent processing on input data.
    ///
    /// # Panics
    /// The method panics if `chunk_size` is zero and slice length is not zero.
    pub fn into_sized_chunks(self, chunk_size: usize) -> Chunks<S, T>
    where
        S: 'static + Sized + Send,
        T: 'static + Send,
    {
        assert!(mem::size_of::<T>() > 0, "zero-sized type is not allowed");

        unsafe {
            let owner = self.data;
            let owner_ptr = Arc::as_ptr(&owner) as *mut S;
            let owner_slice = owner_ptr.as_mut().unwrap().as_mut();

            let slice_len = self.slice.as_ref().len();
            let slice_ptr = self.slice.as_ref().as_ptr();
            let start = slice_ptr.offset_from(owner_slice.as_ptr()) as usize;

            assert!(
                slice_len == 0 || chunk_size > 0,
                "chunk_size must be positive for non-empty slice"
            );

            Chunks {
                chunk_size,
                index: start,
                end: start + slice_len,
                data: owner,
                _phantom: PhantomData,
            }
        }
    }

    /// Returns an iterator of evenly sized chunks of the referencing slice.
    ///
    /// It returns exactly `num_chunks` mostly evenly sized chunks.
    ///
    /// # Panics
    /// The method panics if `division` is zero and slice length is not zero.
    pub fn into_even_chunks(self, num_chunks: usize) -> Chunks<S, T>
    where
        S: 'static + Sized + Send,
        T: 'static + Send,
    {
        assert!(mem::size_of::<T>() > 0, "zero-sized type is not allowed");

        unsafe {
            let data = self.data;
            let data_ptr = Arc::as_ptr(&data) as *mut S;
            let data_slice = data_ptr.as_mut().unwrap().as_mut();

            let slice_len = self.slice.as_ref().len();
            let slice_ptr = self.slice.as_ref().as_ptr();
            let start = slice_ptr.offset_from(data_slice.as_ptr()) as usize;

            assert!(num_chunks > 0, "num_chunks must be positive, but get zero");

            let chunk_size = (slice_len + num_chunks - 1) / num_chunks;

            Chunks {
                index: start,
                chunk_size,
                end: start + slice_len,
                data,
                _phantom: PhantomData,
            }
        }
    }

    /// Gets the reference count on the owning data.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    /// Concatenates contiguous chunks into one chunk.
    ///
    /// # Panics
    /// The method panics if the chunks are not contiguous, or
    /// the chunks refer to inconsistent data.
    pub fn cat<I>(chunks: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        S: AsMut<[T]>,
    {
        unsafe {
            let mut chunks = chunks.into_iter();

            // obtain inner pointer from the first chunk
            let first = chunks.next().expect("the chunks must be non-empty");
            let data = first.data.clone();

            let mut chunks: Vec<_> = iter::once(first)
                .chain(chunks.inspect(|chunk| {
                    // verify if all chunks points to the same owner
                    assert_eq!(
                        Arc::as_ptr(&chunk.data),
                        Arc::as_ptr(&data),
                        "inconsistent owner of the chunks"
                    );
                }))
                .collect();

            // verify if chunks are contiguous
            chunks
                .iter()
                .zip(chunks.iter().skip(1))
                .for_each(|(prev, next)| {
                    let prev_end = prev.slice.as_ref().as_ptr_range().end;
                    let next_start = next.slice.as_ref().as_ptr_range().start;
                    assert!(prev_end == next_start, "the chunks are not contiguous");
                });

            // save slice range
            let len = chunks.iter().map(|chunk| chunk.slice.as_ref().len()).sum();
            let slice_ptr: *mut T = chunks.first_mut().unwrap().as_mut().as_mut_ptr();

            // free chunk references
            drop(chunks);

            // create returning chunk
            let slice = {
                let slice = slice::from_raw_parts_mut(slice_ptr, len);
                NonNull::new_unchecked(slice as *mut [T])
            };

            Chunk { data, slice }
        }
    }

    pub fn into_arc_owner(self) -> Arc<S> {
        self.data
    }

    pub fn into_arc_ref(self) -> ArcRef<S, [T]> {
        unsafe {
            let Self { data, slice } = self;
            ArcRef::new(data).map(|_| slice.as_ref())
        }
    }

    /// Tries to recover the owning data.
    ///
    /// The method succeeds if the referencing chunk iterator and all chunks are dropped.
    /// Otherwise, it returns the guard intact.
    pub fn try_unwrap_owner(self) -> Result<S, Self> {
        let Self { data, slice } = self;
        Arc::try_unwrap(data).map_err(|data| Self { data, slice })
    }
}

unsafe impl<S, T> Send for Chunk<S, T> where S: AsMut<[T]> {}
unsafe impl<S, T> Sync for Chunk<S, T> where S: AsMut<[T]> {}

impl<S, T> AsRef<[T]> for Chunk<S, T>
where
    S: AsMut<[T]>,
{
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<S, T> AsMut<[T]> for Chunk<S, T>
where
    S: AsMut<[T]>,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<S, T> Deref for Chunk<S, T>
where
    S: AsMut<[T]>,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { self.slice.as_ref() }
    }
}

impl<S, T> DerefMut for Chunk<S, T>
where
    S: AsMut<[T]>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.slice.as_mut() }
    }
}

impl<'a, S, T> IntoIterator for &'a Chunk<S, T>
where
    S: AsMut<[T]>,
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter()
    }
}

impl<'a, S, T> IntoIterator for &'a mut Chunk<S, T>
where
    S: AsMut<[T]>,
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().iter_mut()
    }
}
