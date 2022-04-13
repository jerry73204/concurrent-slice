use concurrent_slice::Chunk;
use itertools::izip;

#[test]
fn lifetime_test() {
    let orig = &mut [0, 1, 2, 3, 4];
    let chunk: Chunk<'_, _, _> = Chunk::new(orig);

    let (lslice, rslice) = chunk.split_at(3);
    assert_eq!(&*lslice, &[0, 1, 2]);
    assert_eq!(&*rslice, &[3, 4]);
}

#[test]
fn merge_chunks_test() {
    let orig: Vec<_> = (0..16).collect();

    let mut chunks = Chunk::new(orig).into_even_chunks(3);
    let _ = chunks.next().unwrap();
    let _ = chunks.next().unwrap();
    let _ = chunks.next().unwrap();

    let new = chunks.try_unwrap_owner().unwrap();

    assert!(izip!(new, 0..16).all(|(lhs, rhs)| lhs == rhs));
}

#[test]
fn concat_chunks_test() {
    let orig: Vec<_> = (0..25).collect();

    let mut chunks = Chunk::new(orig).into_even_chunks(4);
    let chunk1 = chunks.next().unwrap();
    let chunk2 = chunks.next().unwrap();
    let chunk3 = chunks.next().unwrap();
    let chunk4 = chunks.next().unwrap();
    drop(chunks); // decrease ref count

    assert!(izip!(&chunk1, 0..7).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunk2, 7..13).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunk3, 13..19).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunk4, 19..25).all(|(&lhs, rhs)| lhs == rhs));

    let chunk12 = Chunk::cat(vec![chunk1, chunk2]);
    assert!(izip!(&chunk12, 0..13).all(|(&lhs, rhs)| lhs == rhs));

    let chunk34 = Chunk::cat(vec![chunk3, chunk4]);
    assert!(izip!(&chunk34, 13..19).all(|(&lhs, rhs)| lhs == rhs));

    let chunk1234 = Chunk::cat(vec![chunk12, chunk34]);
    assert!(izip!(&chunk1234, 0..25).all(|(&lhs, rhs)| lhs == rhs));

    let new = chunk1234.try_unwrap_owner().unwrap();

    assert!(izip!(&new, 0..25).all(|(&lhs, rhs)| lhs == rhs));
}

#[test]
fn concurrent_chunks_test() {
    let vec: Vec<_> = (0..16).collect();
    let chunks: Vec<_> = Chunk::new(vec).into_even_chunks(3).collect();
    assert_eq!(chunks.len(), 3);
    assert!(izip!(&chunks[0], 0..6).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunks[1], 6..11).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&chunks[2], 11..16).all(|(&lhs, rhs)| lhs == rhs));
}

#[test]
fn empty_concurrent_chunks_test() {
    assert_eq!(Chunk::new([1u8; 0]).into_sized_chunks(2).count(), 0);
    assert_eq!(Chunk::new([1u8; 0]).into_even_chunks(1).count(), 0);
}

#[test]
fn iter_owned_test() {
    let owner: Vec<_> = (0..3).collect();
    let owner = Chunk::new(owner);
    let mut iter = owner.into_iter_owned();
    assert_eq!(*iter.next().unwrap(), 0);
    assert_eq!(*iter.next().unwrap(), 1);
    assert_eq!(*iter.next().unwrap(), 2);
    assert!(iter.next().is_none());
}

#[test]
fn owning_windows_test() {
    let owner: Vec<_> = (0..5).collect();
    let owner = Chunk::new(owner);
    let mut windows = owner.into_windows_owned(3);
    assert_eq!(&*windows.next().unwrap(), &[0, 1, 2]);
    assert_eq!(&*windows.next().unwrap(), &[1, 2, 3]);
    assert_eq!(&*windows.next().unwrap(), &[2, 3, 4]);
    assert!(windows.next().is_none());
}

#[test]
fn split_at_test() {
    let vec: Vec<_> = (0..16).collect();
    let (lslice, rslice) = Chunk::new(vec).split_at(5);
    assert!(izip!(&lslice, 0..5).all(|(&lhs, rhs)| lhs == rhs));
    assert!(izip!(&rslice, 5..16).all(|(&lhs, rhs)| lhs == rhs));
}

#[test]
fn chunks_of_chunk_test() {
    let owner: Vec<_> = (0..9).collect();
    let mut chunks = Chunk::new(owner).into_even_chunks(2);

    let chunk1 = chunks.next().unwrap();
    let chunk2 = chunks.next().unwrap();
    assert!(chunks.next().is_none());
    assert_eq!(&*chunk1, &[0, 1, 2, 3, 4]);
    assert_eq!(&*chunk2, &[5, 6, 7, 8]);
    drop(chunks);

    let mut chunks = chunk1.into_sized_chunks(3);
    let chunk3 = chunks.next().unwrap();
    let chunk4 = chunks.next().unwrap();
    assert!(chunks.next().is_none());
    assert_eq!(&*chunk3, &[0, 1, 2]);
    assert_eq!(&*chunk4, &[3, 4]);
    drop(chunks);

    let mut chunks = chunk2.into_even_chunks(3);
    let chunk5 = chunks.next().unwrap();
    let chunk6 = chunks.next().unwrap();
    let chunk7 = chunks.next().unwrap();
    assert!(chunks.next().is_none());
    assert_eq!(&*chunk5, &[5, 6]);
    assert_eq!(&*chunk6, &[7]);
    assert_eq!(&*chunk7, &[8]);

    let chunk8 = Chunk::cat(vec![chunk4, chunk5]);
    assert_eq!(&*chunk8, &[3, 4, 5, 6]);

    let (chunk9, chunk10) = chunk3.split_at(1);
    assert_eq!(&*chunk9, &[0]);
    assert_eq!(&*chunk10, &[1, 2]);

    // if the ref count is decreased to 1, the owner can be recovered.
    drop(chunks);
    drop(chunk7);
    drop(chunk8);
    drop(chunk9);
    drop(chunk10);

    let owner = chunk6.try_unwrap_owner().unwrap();
    assert_eq!(owner, (0..9).collect::<Vec<_>>());
}
