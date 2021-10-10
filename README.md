# concurrrent-slice

This crate extends slice-type types with methods for concurrent processing.

\[ [API doc](https://docs.rs/concurrent-slice/) | [crates.io](https://crates.io/crates/concurrent-slice) \]

## Example

The `slice.concurrent_chunks(chunk_size)` divides the any owned slice-like types, such as `Vec<T>`,
into roughly equally sized chunks. Each chunk is filled with a constant on separate threads.
The original slice is then recovered from the guard given by the chunks.

```rust
let data: Vec<_> = vec![0u32; 12];

// Divide the vec into three chunks, each has length 4.
let mut chunks = data.concurrent_chunks(4);
let mut chunk1 = chunks.next().unwrap();
let mut chunk2 = chunks.next().unwrap();
let mut chunk3 = chunks.next().unwrap();

// Keeps the guard that will be used to recover the data.
let guard = chunks.guard();

// Process each chunk concurrently.
let handle1 = std::thread::spawn(move || {
    chunk1.iter_mut().for_each(|elem| {
*elem = 1;
    });
});

let handle2 = std::thread::spawn(move || {
    chunk2.iter_mut().for_each(|elem| {
*elem = 2;
    });
});

let handle3 = std::thread::spawn(move || {
    chunk3.iter_mut().for_each(|elem| {
*elem = 3;
    });
});

handle1.join().unwrap();
handle2.join().unwrap();
handle3.join().unwrap();

// We drop the chunks iterator to make sure the guard is the only reference to data.
drop(chunks);

// Recover the data.
let data = guard.unwrap();
assert_eq!(&data, &[1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
```

## License

MIT License. See [LICENSE](LICENSE.txt) file.
