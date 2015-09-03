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

//! Multi-producer, single-consumer FIFO queue communication primitives.

pub use std::sync::mpsc::{SendError, TryRecvError, RecvError};
use std::cell::UnsafeCell;
use std::sync::Arc;

use std::sync::mpsc;

use sync::WaitList;

#[derive(Clone)]
pub struct Sender<T> {
    inner: mpsc::Sender<T>,
    wait_list: Arc<UnsafeCell<WaitList>>,
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Send for Receiver<T> {}

impl<T> Sender<T> {
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        try!(self.inner.send(t));

        unsafe { &mut *self.wait_list.get() }.wake();

        Ok(())
    }
}

pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
    wait_list: Arc<UnsafeCell<WaitList>>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.inner.try_recv()
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        loop {
            match self.try_recv() {
                Ok(v) => return Ok(v),
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => return Err(RecvError),
            }

            unsafe { &mut *self.wait_list.get() }.push().block();
        }
    }
}

/// Create a channel pair
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel();

    let wait_list = Arc::new(UnsafeCell::new(WaitList::new()));

    let tx = Sender {
        inner: tx,
        wait_list: wait_list.clone()
    };

    let rx = Receiver {
        inner: rx,
        wait_list: wait_list
    };

    (tx, rx)
}

#[cfg(test)]
mod test {
    use super::*;

    use {spawn, run};

    #[test]
    fn test_channel_basic() {
        let (tx, rx) = channel();
        spawn(move|| {
            tx.send(1).unwrap();
        });

        run(1);

        assert_eq!(1, rx.recv().unwrap());
    }
}
