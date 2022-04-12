use crate::{chunks::SizedChunks, common::*, EvenChunks};

/// A mutable sub-slice reference-counted reference to a slice-like data.
#[derive(Debug)]
pub struct Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    pub(super) owner: Arc<S>,
    pub(super) slice: NonNull<[T]>,
    pub(super) _phantom: PhantomData<&'a S>,
}

impl<'a, S, T> Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    pub fn new(owner: S) -> Self {
        Self::from_arc(Arc::new(owner))
    }

    pub fn from_arc(owner: Arc<S>) -> Self {
        unsafe {
            let ptr = Arc::as_ptr(&owner) as *mut S;
            let slice: &mut [T] = ptr.as_mut().unwrap().as_mut();
            let slice = NonNull::new_unchecked(slice as *mut [T]);
            Self {
                owner,
                slice,
                _phantom: PhantomData,
            }
        }
    }

    /// Splits the chunk into two sub-chunks, divided at specified index.
    ///
    /// # Panics
    /// The method panics if the index is out of bound.
    pub fn split_at(mut self, index: usize) -> (Chunk<'a, S, T>, Chunk<'a, S, T>) {
        unsafe {
            let owner = self.owner;
            let slice: &mut [T] = self.slice.as_mut();
            let lslice = NonNull::new_unchecked(&mut slice[0..index] as *mut [T]);
            let rslice = NonNull::new_unchecked(&mut slice[index..] as *mut [T]);

            (
                Chunk {
                    owner: owner.clone(),
                    slice: lslice,
                    _phantom: PhantomData,
                },
                Chunk {
                    owner,
                    slice: rslice,
                    _phantom: PhantomData,
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
    pub fn into_sized_chunks(self, chunk_size: usize) -> SizedChunks<'a, S, T> {
        assert!(mem::size_of::<T>() > 0, "zero-sized type is not allowed");

        unsafe {
            let owner = self.owner;
            let owner_ptr = Arc::as_ptr(&owner) as *mut S;
            let owner_slice = owner_ptr.as_mut().unwrap().as_mut();

            let slice_len = self.slice.as_ref().len();
            let slice_ptr = self.slice.as_ref().as_ptr();
            let start = slice_ptr.offset_from(owner_slice.as_ptr()) as usize;

            assert!(
                slice_len == 0 || chunk_size > 0,
                "chunk_size must be positive for non-empty slice"
            );

            SizedChunks {
                chunk_size,
                index: start,
                end: start + slice_len,
                owner,
                _phantom: PhantomData,
            }
        }
    }

    /// Returns an iterator of evenly sized chunks of the referencing slice.
    ///
    /// It returns exactly `num_chunks` mostly evenly sized chunks.
    ///
    /// # Panics
    /// The method panics if `num_chunks` is zero.
    pub fn into_even_chunks(self, num_chunks: usize) -> EvenChunks<'a, S, T> {
        assert!(mem::size_of::<T>() > 0, "zero-sized type is not allowed");

        unsafe {
            let owner = self.owner;
            let owner_ptr = Arc::as_ptr(&owner) as *mut S;
            let owner_slice = owner_ptr.as_mut().unwrap().as_mut();

            let slice_len = self.slice.as_ref().len();
            let slice_ptr = self.slice.as_ref().as_ptr();
            let start = slice_ptr.offset_from(owner_slice.as_ptr()) as usize;

            assert!(num_chunks > 0, "num_chunks must be positive, but get zero");

            let base_chunk_size = slice_len / num_chunks;
            let long_end = start + (slice_len % num_chunks) * (base_chunk_size + 1);

            EvenChunks {
                index: start,
                base_chunk_size,
                long_end,
                short_end: start + slice_len,
                owner,
                _phantom: PhantomData,
            }
        }
    }

    /// Gets the reference count on the owning data.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.owner)
    }

    /// Concatenates contiguous chunks into one chunk.
    ///
    /// # Panics
    /// The method panics if the chunks are not contiguous, or
    /// the chunks belong to different owners.
    pub fn cat<I>(chunks: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        unsafe {
            let mut chunks = chunks.into_iter();

            // obtain inner pointer from the first chunk
            let first = chunks.next().expect("the chunks must be non-empty");
            let owner = first.owner.clone();

            let mut chunks: Vec<_> = iter::once(first)
                .chain(chunks.inspect(|chunk| {
                    // verify if all chunks points to the same owner
                    assert_eq!(
                        Arc::as_ptr(&chunk.owner),
                        Arc::as_ptr(&owner),
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

            Chunk {
                owner,
                slice,
                _phantom: PhantomData,
            }
        }
    }

    pub fn into_arc_owner(self) -> Arc<S> {
        self.owner
    }

    pub fn into_arc_ref(self) -> ArcRef<S, [T]> {
        unsafe {
            let Self { owner, slice, .. } = self;
            ArcRef::new(owner).map(|_| slice.as_ref())
        }
    }

    /// Tries to recover the owning data.
    ///
    /// The method succeeds if the referencing chunk iterator and all chunks are dropped.
    /// Otherwise, it returns the guard intact.
    pub fn try_unwrap_owner(self) -> Result<S, Self> {
        let Self { owner, slice, .. } = self;
        Arc::try_unwrap(owner).map_err(|owner| Self {
            owner,
            slice,
            _phantom: PhantomData,
        })
    }
}

unsafe impl<'a, S, T> Send for Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
}

unsafe impl<'a, S, T> Sync for Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
}

impl<'a, S, T> AsRef<[T]> for Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<'a, S, T> AsMut<[T]> for Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<'a, S, T> Deref for Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { self.slice.as_ref() }
    }
}

impl<'a, S, T> DerefMut for Chunk<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.slice.as_mut() }
    }
}

impl<'a, S, T> IntoIterator for &'a Chunk<'_, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter()
    }
}

impl<'a, S, T> IntoIterator for &'a mut Chunk<'_, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().iter_mut()
    }
}
