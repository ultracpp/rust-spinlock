/*
 * SpinLock - custom implementation of a spinlock in Rust
 * Copyright (c) 2024 Eungsuk Jeon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use spin_lock::SpinLock;

const THREAD_COUNT: usize = 32;
const JOB_COUNT: usize = 1000000;

fn unix_timestamp() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

fn main() {
    let lock_ = Arc::new(SpinLock::new(0));
    let start = unix_timestamp();
    let mut vec = Vec::new();

    for _ in 0..THREAD_COUNT {
        let lock_ = Arc::clone(&lock_);

        let thread = thread::spawn(move || {
            for _ in 0..JOB_COUNT * 10 {
                for a in 0..2 {
                    /*if let Err(e) = lock_.lock_with_max_attempts() {
                        println!("Error: {}", e);
                        return;
                    }*/
                    lock_.lock();
                    unsafe {
                        *lock_.data.get() += a;
                    }
                    lock_.unlock();
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
        unsafe { *lock_.data.get() },
        unix_timestamp() - start
    );
}
