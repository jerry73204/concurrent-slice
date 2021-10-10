use crate::common::*;

/// The iterator returned from [owning_iter()](crate::slice::ConcurrentSlice::owning_windows).
#[derive(Debug)]
pub struct Windows<S, T> {
    pub(super) owner: OwningRef<S, [T]>,
    pub(super) size: usize,
    pub(super) index: usize,
}

impl<S, T> Clone for Windows<S, T>
where
    S: CloneStableAddress,
{
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            ..*self
        }
    }
}

impl<S, T> Iterator for Windows<S, T>
where
    S: CloneStableAddress,
{
    type Item = OwningRef<S, [T]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.size > self.owner.len() {
            return None;
        }

        let window = self
            .owner
            .clone()
            .map(|slice| &slice[self.index..(self.index + self.size)]);
        self.index += 1;
        Some(window)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.owner.len() >= self.size {
            let len = self.owner.len() - self.size + 1;
            (len, Some(len))
        } else {
            (0, Some(0))
        }
    }
}

impl<S, T> ExactSizeIterator for Windows<S, T> where S: CloneStableAddress {}
