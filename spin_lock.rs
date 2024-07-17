/*
 * cskiplist - custom implementation of a spinlock in Rust
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const USE_SLEEP_SPIN_LOCK: bool = true;
const SPIN_LOCK_SLEEP_ONE_FREQUENCY: usize = 50;
const SPIN_LOCK_MAX_ATTEMPTS: usize = 500;

pub struct SpinLock<T> {
    lock_: AtomicBool,
    pub data: std::cell::UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> SpinLock<T> {
        SpinLock {
            lock_: AtomicBool::new(false),
            data: std::cell::UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) {
        let mut freq = 0;

        loop {
            if self
                .lock_
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }

            while self.lock_.load(Ordering::Relaxed) {
                thread::yield_now();

                if USE_SLEEP_SPIN_LOCK {
                    freq += 1;

                    if freq == SPIN_LOCK_SLEEP_ONE_FREQUENCY {
                        thread::sleep(Duration::from_millis(1));
                        freq = 0;
                    }
                }
            }
        }
    }

    pub fn lock_with_max_attempts(&self) -> Result<(), &'static str> {
        let mut freq = 0;
        let mut attempts = 0;

        loop {
            if self
                .lock_
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(());
            }

            while self.lock_.load(Ordering::Relaxed) {
                thread::yield_now();
                attempts += 1;

                if attempts >= SPIN_LOCK_MAX_ATTEMPTS {
                    return Err("Failed to acquire lock after maximum attempts");
                }

                if USE_SLEEP_SPIN_LOCK {
                    freq += 1;

                    if freq == SPIN_LOCK_SLEEP_ONE_FREQUENCY {
                        thread::sleep(Duration::from_millis(1));
                        freq = 0;
                    }
                }
            }
        }
    }

    pub fn unlock(&self) {
        self.lock_.store(false, Ordering::Release);
    }

    #[allow(dead_code)]
    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.lock();
        let result = unsafe { f(&mut *self.data.get()) };
        self.unlock();
        result
    }

    #[allow(dead_code)]
    pub fn with_lock_timeout<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, &'static str> {
        self.lock_with_max_attempts()?;
        let result = unsafe { f(&mut *self.data.get()) };
        self.unlock();
        Ok(result)
    }
}
