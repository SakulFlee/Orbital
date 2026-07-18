use std::future::Future;
use std::task::{Context, Poll, Waker};
use std::thread;

/// Synchronously block on a future by busy-polling with `Waker::noop()`.
///
/// This is used to bridge wgpu's async API (`request_adapter`, `request_device`)
/// in contexts where no async runtime is available. The futures returned by wgpu
/// resolve on the first poll in practice, so the busy-wait is harmless.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut pinned = Box::pin(future);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => thread::yield_now(),
        }
    }
}
