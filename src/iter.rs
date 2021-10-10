use crate::common::*;

/// The iterator returned from [owning_iter()](crate::slice::ConcurrentSlice::owning_iter).
#[derive(Debug)]
pub struct Iter<S, T> {
    pub(super) owner: OwningRef<S, [T]>,
    pub(super) index: usize,
}

impl<S, T> Clone for Iter<S, T>
where
    S: CloneStableAddress,
{
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            index: self.index,
        }
    }
}

impl<S, T> Iterator for Iter<S, T>
where
    S: CloneStableAddress,
{
    type Item = OwningRef<S, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.owner.len() {
            return None;
        }

        let item = self.owner.clone().map(|slice| &slice[self.index]);
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.owner.len();
        (len, Some(len))
    }
}

impl<S, T> ExactSizeIterator for Iter<S, T> where S: CloneStableAddress {}
