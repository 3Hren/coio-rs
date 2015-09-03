// The MIT License (MIT)

// Copyright (c) 2015 Y. T. Chung <zonyitoo@gmail.com>

//  Permission is hereby granted, free of charge, to any person obtaining a
//  copy of this software and associated documentation files (the "Software"),
//  to deal in the Software without restriction, including without limitation
//  the rights to use, copy, modify, merge, publish, distribute, sublicense,
//  and/or sell copies of the Software, and to permit persons to whom the
//  Software is furnished to do so, subject to the following conditions:
//
//  The above copyright notice and this permission notice shall be included in
//  all copies or substantial portions of the Software.
//
//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
//  OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//  FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//  DEALINGS IN THE SOFTWARE.

//! Coroutine synchronization

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use coroutine::{Coroutine};

pub use self::mutex::Mutex;

pub mod mutex;
pub mod mpsc;

use runtime::processor::Processor;

struct WaitList {
    /// Blocked coroutines queue.
    inner: VecDeque<*mut Coroutine>,

    /// Protects the queue from concurrent access.
    mutex: AtomicBool,
}

impl WaitList {
    fn new() -> WaitList {
        WaitList {
            inner: VecDeque::with_capacity(64),
            mutex: AtomicBool::new(false),
        }
    }

    fn push(&mut self) -> &'static mut Processor {
        let current = unsafe {
            Processor::current().running()
        };

        self.lock();

        match current {
            Some(coro) => self.inner.push_back(coro),
            None => {}
        }

        self.unlock();

        Processor::current()
    }

    fn wake(&mut self) {
        self.lock();

        match self.inner.pop_front() {
            Some(coro) => unsafe { Processor::current().ready(coro); },
            None => {}
        }

        self.unlock();
    }

    fn lock(&mut self) {
        loop {
            if self.mutex.compare_and_swap(false, true, Ordering::SeqCst) == false {
                break;
            }
        }
    }

    fn unlock(&mut self) {
        loop {
            if self.mutex.compare_and_swap(true, false, Ordering::SeqCst) == true {
                break;
            }
        }
    }
}
