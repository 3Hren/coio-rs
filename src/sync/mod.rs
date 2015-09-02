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

use coroutine::{Coroutine};

pub use self::mutex::Mutex;

pub mod mutex;
pub mod mpsc;

use runtime::processor::Processor;

struct WaitList {
    inner: VecDeque<*mut Coroutine>,
}

impl WaitList {
    fn new() -> WaitList {
        WaitList {
            inner: VecDeque::with_capacity(64),
        }
    }

    fn sleep(&mut self) {
        let current = unsafe {
            Processor::current().running()
        };

        match current {
            Some(coro) => {
                self.inner.push_back(coro);
            }
            None => {}
        }

        Processor::current().block();
    }

    fn wake(&mut self) {
        match self.inner.pop_front() {
            Some(coro) => {
                unsafe {
                    Processor::current().ready(coro);
                }
            }
            None => {}
        }
    }
}
