
# benchmarking

Benchmarking various implementations of readers-writer locks, `std::sync::RwLock`, `parking_lot::RwLock`, `tokio::sync::RwLock`.

```
cargo bench --message-format=json > output.json
```

We compare several different Rust implementations of readers-writer locks.

 - [pflock](https://crates.io/crates/pflock), based on "Reader-writer synchronization for shared-memory multiprocessor real-time systems"
 - [pairlock](https://crates.io/crates/pairlock)
 - [tokio](https://tokio.rs/)
 - [sharedmutex](https://crates.io/crates/shared-mutex)
 - [widerwlock](https://crates.io/crates/widerwlock)
 - [spin](https://crates.io/crates/spin)
 - [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html)

# Concurrency testing

Using the crate `loom` it is actually possible to test all possible
interleavings defined by the C11 memory standard to test the shared lock.

```
RUSTFLAGS="--cfg loom" cargo test
```

# Related work

[1] BRAVO – Biased Locking for Reader-Writer Locks https://www.usenix.org/system/files/atc19-dice.pdf
[2] NUMA-aware reader-writer locks https://dl.acm.org/doi/10.1145/2442516.2442532
[3] brlocks https://lwn.net/Articles/378911/
[4] Scalable reader-writer locks for parallel systems https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=222989
[5] Scalable read-mostly synchronization using passive reader-writerlocks https://www.usenix.org/conference/atc14/technical-sessions/presentation/liu
[6] Distributed Reader-Writer Mutex https://www.1024cores.net/home/lock-free-algorithms/reader-writer-problem/distributed-reader-writer-mutex
[7] Distributed Cache-Line Counter Scalable RW-Lock http://concurrencyfreaks.blogspot.com/2013/09/distributed-cache-line-counter-scalable.html
[8] Folly - https://github.com/facebook/folly