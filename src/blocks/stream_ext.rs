use core::pin::Pin;
use futures_util::ready;
use futures_util::stream::Stream;
use futures_util::task::{Context, Poll};
use pin_project::pin_project;
use std::future::Future;

#[pin_project]
pub struct Periodise<St, F, Fut> {
	#[pin]
	stream: St,
	waiter_factory: F,
	#[pin]
	future: Option<Fut>,
}

impl<St, F, Fut> Periodise<St, F, Fut> {
	pub fn new(stream: St, waiter_factory: F) -> Self {
		Self {
			stream,
			waiter_factory,
			future: None,
		}
	}
}

impl<St, F, Fut> Stream for Periodise<St, F, Fut>
where
	St: Stream,
	F: Fn() -> Fut,
	Fut: Future<Output = ()>,
{
	type Item = St::Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let mut this = self.project();
		if let Some(fut) = this.future.as_mut().as_pin_mut() {
			ready!(fut.poll(cx));
		}
		this.future.set(Some((this.waiter_factory)()));
		this.stream.as_mut().poll_next(cx)
	}
}

// This stream extension is currently unused because the return types from the streams in this
// project are `Result<_, Error>` where `Error` does not implement `Clone`. If I can find a way
// to specify that the trait bound for `Clone` is only actually required on the `Ok` variant of
// the stream item, then this would be a nice alternative to implementing the "on changes"
// functionailty within `util::watch`.
#[pin_project]
pub struct OnChanges<St>
where
	St: Stream,
{
	#[pin]
	stream: St,
	prev: Option<<St as Stream>::Item>,
}

impl<St: Stream> OnChanges<St> {
	pub fn new(stream: St) -> Self {
		Self { stream, prev: None }
	}
}

impl<St> Stream for OnChanges<St>
where
	St: Stream,
	<St as Stream>::Item: PartialEq + Clone,
{
	type Item = St::Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let mut this = self.project();
		loop {
			if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
				if let Some(prev) = this.prev.replace(item.clone()) {
					if prev != item {
						return Poll::Ready(Some(item));
					}
				} else {
					return Poll::Ready(Some(item)); // Always return on first read
				}
			} else {
				return Poll::Ready(None);
			}
		}
	}
}

pub trait StreamExt2: Stream {
	fn with_period<F, Fut>(self, f: F) -> Periodise<Self, F, Fut>
	where
		Self: Sized,
		F: Fn() -> Fut,
		Fut: Future<Output = ()>,
	{
		Periodise::new(self, f)
	}

	fn on_changes(self) -> OnChanges<Self>
	where
		Self: Sized,
	{
		OnChanges::new(self)
	}
}

impl<T: ?Sized> StreamExt2 for T where T: Stream {}
