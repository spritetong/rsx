use futures::{stream::FusedStream, Sink, SinkExt, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait SinkRsx<Item>: SinkExt<Item> {
    /// Transforms the error returned by the sink.
    fn safe_sink_map_err<E, F>(self, f: F) -> SafeSinkMapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E,
        Self: Sized,
    {
        SafeSinkMapErr::new(self, f)
    }
}

/// Sink for the [`sink_map_err`](super::SinkExt::sink_map_err) method.
#[pin_project]
#[derive(Debug, Clone)]
pub struct SafeSinkMapErr<Si, F> {
    #[pin]
    sink: Si,
    f: F,
}

impl<Si, F> SafeSinkMapErr<Si, F> {
    pub fn new(sink: Si, f: F) -> Self {
        Self { sink, f }
    }

    crate::future_delegate_access_inner!(sink, Si, ());
}

// Forwarding impl of Sink from the underlying sink
impl<Si, F, E, Item> Sink<Item> for SafeSinkMapErr<Si, F>
where
    Si: Sink<Item>,
    F: Fn(Si::Error) -> E,
{
    type Error = E;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .sink
            .poll_ready(cx)
            .map_err(|e| (self.as_mut().f)(e))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        self.as_mut()
            .project()
            .sink
            .start_send(item)
            .map_err(|e| (self.as_mut().f)(e))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .sink
            .poll_flush(cx)
            .map_err(|e| (self.as_mut().f)(e))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .sink
            .poll_close(cx)
            .map_err(|e| (self.as_mut().f)(e))
    }
}

// Forwarding impl of Stream from the underlying sink
impl<S: Stream, F> Stream for SafeSinkMapErr<S, F> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().sink.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.sink.size_hint()
    }
}

// Forwarding impl of FusedStream from the underlying sink
impl<S: FusedStream, F> FusedStream for SafeSinkMapErr<S, F> {
    fn is_terminated(&self) -> bool {
        self.sink.is_terminated()
    }
}
