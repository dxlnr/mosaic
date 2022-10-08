#![allow(dead_code)]

use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    stream::{FuturesUnordered, Stream},
    Future,
};
use tokio::task::{JoinError, JoinHandle};

/// `ConcurrentFutures` can keep a capped number of futures running concurrently, and yield their
/// result as they finish. When the max number of concurrent futures is reached, new tasks are
/// queued until some in-flight futures finish.
pub struct ConcurrentFutures<T>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    /// In-flight futures.
    running: FuturesUnordered<JoinHandle<T::Output>>,
    /// Buffered tasks.
    queued: VecDeque<T>,
    /// Max number of concurrent futures.
    max_in_flight: usize,
}

impl<T> ConcurrentFutures<T>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    pub fn new(max_in_flight: usize) -> Self {
        Self {
            running: FuturesUnordered::new(),
            queued: VecDeque::new(),
            max_in_flight,
        }
    }

    pub fn push(&mut self, task: T) {
        self.queued.push_back(task)
    }
}

impl<T> Unpin for ConcurrentFutures<T>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
}

impl<T> Stream for ConcurrentFutures<T>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    type Item = Result<T::Output, JoinError>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        while this.running.len() < this.max_in_flight {
            if let Some(queued) = this.queued.pop_front() {
                let handle = tokio::spawn(queued);
                this.running.push(handle);
            } else {
                break;
            }
        }
        Pin::new(&mut this.running).poll_next(cx)
    }
}
