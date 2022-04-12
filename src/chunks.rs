use crate::{chunk::Chunk, common::*};

/// An iterator that yields [chunks](Chunk).
#[derive(Debug)]
pub struct Chunks<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    pub(super) index: usize,
    pub(super) chunk_size: usize,
    pub(super) end: usize,
    pub(super) data: Arc<S>,
    pub(super) _phantom: PhantomData<&'a T>,
}

impl<'a, S, T> Chunks<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    pub fn into_arc_owner(self) -> Arc<S> {
        self.data
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
            data,
            ..
        } = self;

        Arc::try_unwrap(data).map_err(|data| Self {
            index,
            chunk_size,
            end,
            data,
            _phantom: PhantomData,
        })
    }

    /// Gets the reference count on the owning data.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

impl<'a, S, T> Iterator for Chunks<'a, S, T>
where
    S: AsMut<[T]> + Send + Sync + 'a,
    T: Send + Sync,
{
    type Item = Chunk<'a, S, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            return None;
        }

        let start = self.index;
        let end = cmp::min(start + self.chunk_size, self.end);
        self.index = end;

        let data = self.data.clone();

        let slice = unsafe {
            let ptr = Arc::as_ptr(&data) as *mut S;
            let slice: &mut [T] = ptr.as_mut().unwrap().as_mut();
            NonNull::new_unchecked(&mut slice[start..end] as *mut [T])
        };

        Some(Chunk {
            owner: data,
            slice,
            _phantom: PhantomData,
        })
    }
}
