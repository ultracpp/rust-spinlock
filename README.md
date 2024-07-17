# SpinLock Implementation in Rust

This repository contains a custom implementation of a spinlock in Rust, which includes a backoff strategy for lock acquisition. The spinlock is designed to replace traditional mutexes, providing an alternative synchronization primitive with different performance characteristics.

## Files

- **spin_lock.rs**: Contains the `SpinLock` struct and its implementation.
- **main.rs**: Contains the test code for the `SpinLock` implementation.

## Features

- **Basic SpinLock**: A simple spinlock using an atomic boolean to manage the lock state.
- **Backoff Strategy**: Incorporates a backoff strategy that includes yielding and optional sleeping to reduce CPU usage during contention.
- **Max attempts Lock**: Adds a timeout feature to the lock acquisition, returning an error if the lock cannot be obtained after a specified number of attempts.

## Performance

In general, this custom spinlock implementation has shown to be more than 4 times faster compared to traditional mutexes, though actual performance gains can vary depending on the specific computer hardware and workload. Spinlocks are particularly useful for short, quick operations where the overhead of a mutex would be significant.

## Usage

### SpinLock

The `SpinLock` struct provides the following methods:

- `new(data: T) -> SpinLock<T>`: Creates a new `SpinLock` with the given data.
- `lock(&self)`: Acquires the lock, blocking until it is available.
- `unlock(&self)`: Releases the lock.
- `lock_with_max_attempts(&self) -> Result<(), &'static str>`: Attempts to acquire the lock, returning an error if the lock cannot be obtained after a maximum number of attempts.
- `with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R`: Acquires the lock, executes the given closure, and releases the lock.
- `with_lock_max_attempts<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, &'static str>`: Attempts to acquire the lock, executes the given closure, and releases the lock, returning an error if the lock cannot be obtained after a maximum number of attempts.

### Main

The `main.rs` file demonstrates the usage of `SpinLock` in a multithreaded context. The example creates multiple threads that increment a shared counter protected by the spinlock.

## Example

Here is a brief example of how to use the `SpinLock`:

```rust
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use spin_lock::SpinLock;

const THREAD_COUNT: usize = 10;
const JOB_COUNT: usize = 1000;

fn unix_timestamp() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

fn main() {
    let sum = Arc::new(SpinLock::new(0));
    let start = unix_timestamp();
    let mut vec = Vec::new();

    for _ in 0..THREAD_COUNT {
        let sum = Arc::clone(&sum);

        let thread = thread::spawn(move || {
            for _ in 0..JOB_COUNT * 10 {
                for a in 0..2 {
                    if let Err(e) = sum.lock_with_timeout() {
                        println!("Error: {}", e);
                        return;
                    }
                    unsafe {
                        *sum.data.get() += a;
                    }
                    sum.unlock();
                }
            }
        });

        vec.push(thread);
    }

    for thread in vec.drain(..) {
        thread.join().unwrap();
    }

    println!(
        "SpinLock: {} {}",
        unsafe { *sum.data.get() },
        unix_timestamp() - start);
}
```

## Running the Tests

To run the tests, simply compile and execute the `main.rs` file:

```sh
rustc main.rs
./main
