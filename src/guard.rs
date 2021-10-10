use crate::common::*;

/// The guard is used to recover the owning data from [Chunks].
#[derive(Debug)]
#[repr(transparent)]
pub struct Guard<S> {
    pub(super) data: Arc<S>,
}

impl<S> Guard<S>
where
    S: Send,
{
    /// Tries to recover the owning data.
    ///
    /// The method succeeds if the referencing chunk iterator and all chunks are dropped.
    /// Otherwise, it returns the guard intact.
    pub fn try_unwrap(self) -> Result<S, Self> {
        Arc::try_unwrap(self.data).map_err(|data| Guard { data })
    }

    pub fn unwrap(self) -> S
    where
        S: Debug,
    {
        self.try_unwrap().unwrap()
    }

    /// Gets the reference count on the owning data.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}
