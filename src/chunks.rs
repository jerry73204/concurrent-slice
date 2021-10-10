use crate::{chunk::Chunk, common::*, guard::Guard};

/// An iterator that yields [chunks](Chunk).
#[derive(Debug)]
pub struct Chunks<S, T>
where
    S: 'static + Send,
    T: 'static + Send,
{
    pub(super) index: usize,
    pub(super) chunk_size: usize,
    pub(super) end: usize,
    pub(super) data: Arc<S>,
    pub(super) _phantom: PhantomData<T>,
}

impl<S, T> Chunks<S, T>
where
    S: 'static + Send,
    T: 'static + Send,
{
    /// Obtains the guard that is used to recover the owning data.
    pub fn guard(&self) -> Guard<S> {
        Guard {
            data: self.data.clone(),
        }
    }

    /// Gets the reference count on the owning data.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

impl<S, T> Iterator for Chunks<S, T>
where
    S: 'static + AsMut<[T]> + Send,
    T: 'static + Send,
{
    type Item = Chunk<S, T>;

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

        Some(Chunk { data, slice })
    }
}
