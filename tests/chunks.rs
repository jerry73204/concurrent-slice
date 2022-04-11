use concurrent_slice::{Chunk, ConcurrentSlice};
use itertools::izip;
use std::sync::Arc;

#[test]
fn merge_chunks_test() {
    let orig: Vec<_> = (0..16).collect();

    let mut chunks = orig.concurrent_chunks_by_division(3);
    let _ = chunks.next().unwrap();
    let _ = chunks.next().unwrap();
    let _ = chunks.next().unwrap();

    let guard = chunks.guard();
    drop(chunks);
    let new = guard.try_unwrap().unwrap();

    assert!(izip!(new, 0..16).all(|(lhs, rhs)| lhs == rhs));
}

#[test]
fn concat_chunks_test() {
    let orig: Vec<_> = (0..25).collect();

    let mut chunks = orig.concurrent_chunks_by_division(4);
    let chunk1 = chunks.next().unwrap();
    let chunk2 = chunks.next().unwrap();
    let chunk3 = chunks.next().unwrap();
    let chunk4 = chunks.next().unwrap();
    drop(chunks); // decrease ref count

    let chunk12 = Chunk::cat(vec![chunk1, chunk2]);
    assert!(izip!(&chunk12, 0..14).all(|(&lhs, rhs)| lhs == rhs));

    let chunk34 = Chunk::cat(vec![chunk3, chunk4]);
    assert!(izip!(&chunk34, 14..25).all(|(&lhs, rhs)| lhs == rhs));

    let chunk1234 = Chunk::cat(vec![chunk12, chunk34]);
    assert!(izip!(&chunk1234, 0..25).all(|(&lhs, rhs)| lhs == rhs));

    let guard = chunk1234.guard();
    drop(chunk1234);
    let new = guard.try_unwrap().unwrap();

    assert!(izip!(&new, 0..25).all(|(&lhs, rhs)| lhs == rhs));
}

#[test]
fn concurrent_chunks_test() {
    let vec: Vec<_> = (0..16).collect();
    let chunks: Vec<_> = vec.concurrent_chunks_by_division(3).collect();
    assert_eq!(chunks.len(), 3);
    assert!(izip!(&chunks[0], 0..6).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunks[1], 6..12).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunks[2], 12..16).all(|(&lhs, rhs)| lhs == rhs));
}

#[test]
fn empty_concurrent_chunks_test() {
    assert_eq!([(); 0].concurrent_chunks(2).count(), 0);
    assert_eq!([(); 0].concurrent_chunks_by_division(None).count(), 0);
}

#[test]
fn owning_iter_test() {
    let owner: Vec<_> = (0..3).collect();
    let owner = Arc::new(owner);
    let mut windows = owner.owning_iter();
    assert_eq!(*windows.next().unwrap(), 0);
    assert_eq!(*windows.next().unwrap(), 1);
    assert_eq!(*windows.next().unwrap(), 2);
    assert!(windows.next().is_none());
}

#[test]
fn owning_windows_test() {
    let owner: Vec<_> = (0..5).collect();
    let owner = Arc::new(owner);
    let mut windows = owner.owning_windows(3);
    assert_eq!(&*windows.next().unwrap(), &[0, 1, 2]);
    assert_eq!(&*windows.next().unwrap(), &[1, 2, 3]);
    assert_eq!(&*windows.next().unwrap(), &[2, 3, 4]);
    assert!(windows.next().is_none());
}

#[test]
fn split_at_test() {
    let vec: Vec<_> = (0..16).collect();
    let (lslice, rslice) = vec.concurrent_split_at(5);
    assert!(izip!(&lslice, 0..5).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&rslice, 5..16).all(|(&lhs, rhs)| lhs == rhs));
}

#[test]
fn chunks_of_chunk_test() {
    let owner: Vec<_> = (0..9).collect();
    let mut chunks = owner.concurrent_chunks_by_division(2);

    let chunk1 = chunks.next().unwrap();
    let chunk2 = chunks.next().unwrap();
    assert!(chunks.next().is_none());
    drop(chunks);

    let mut chunks = chunk1.chunks(3);
    let chunk3 = chunks.next().unwrap();
    let chunk4 = chunks.next().unwrap();
    assert!(chunks.next().is_none());
    assert_eq!(&*chunk3, &[0, 1, 2]);
    assert_eq!(&*chunk4, &[3, 4]);
    drop(chunks);

    let mut chunks = chunk2.chunks_by_division(3);
    let chunk5 = chunks.next().unwrap();
    let chunk6 = chunks.next().unwrap();
    assert!(chunks.next().is_none());
    assert_eq!(&*chunk5, &[5, 6]);
    assert_eq!(&*chunk6, &[7, 8]);

    let chunk7 = Chunk::cat(vec![chunk4, chunk5]);
    assert_eq!(&*chunk7, &[3, 4, 5, 6]);

    let (chunk8, chunk9) = chunk7.split_at(1);
    assert_eq!(&*chunk8, &[3]);
    assert_eq!(&*chunk9, &[4, 5, 6]);

    // if the ref count is correct, the data should be recovered
    let guard = chunk6.guard();

    drop(chunks);
    drop(chunk3);
    drop(chunk6);
    drop(chunk8);
    drop(chunk9);

    let owner = guard.try_unwrap().unwrap();
    assert_eq!(owner, (0..9).collect::<Vec<_>>());
}
