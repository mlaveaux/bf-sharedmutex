
# benchmarking

Benchmarking various implementations of readers-writer locks, `std::sync::RwLock`, `parking_lot::RwLock`, `tokio::sync::RwLock`.

```
    cargo bench
```

# Concurrency testing

Using the crate `loom` it is actually possible to test all possible
interleavings defined by the C11 memory standard to test the shared lock.

```
    RUSTFLAGS="--cfg loom" cargo test
```