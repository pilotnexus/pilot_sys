use core::task::{RawWaker, RawWakerVTable};
use futures::{future::{Either, select}, Future };
use futures::future::FusedFuture;
use core::pin::Pin;

pub struct State<'a, T> {
    pub future: Pin<&'a mut dyn FusedFuture<Output = T>>,
}

/// Becomes ready as soon as one of the given futures becomes ready.
pub async fn either(fut1: impl Future<Output = ()>, fut2: impl Future<Output = ()>) {
    futures::pin_mut!(fut1);
    futures::pin_mut!(fut2);
    select(fut1, fut2).await;
}

/// Takes two futures, if the first future completes first, returns Ok(Output), 
/// if the second future returns first, an Err(Output) is returned
/// Can be used for timeouts:
pub async fn wait_or<A, B>(wait_for: impl Future<Output = A>, err: impl Future<Output = B>) -> Result<A, B> {
  futures::pin_mut!(wait_for);
  futures::pin_mut!(err);
  match select(wait_for, err).await {
      Either::Left((a, _)) => Ok(a),
      Either::Right((b, _)) => Err(b)
  }
}

/// Creates an RawWaker that does nothing.
pub fn raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        raw_waker()
    }
    fn wake(_: *const ()) {}
    fn drop(_: *const ()) {}

    let vtable = &RawWakerVTable::new(clone, wake, wake, drop);
    RawWaker::new(0 as *const (), vtable)
}
