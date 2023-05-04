use std::{
    io,
    pin::{pin, Pin},
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Stream {
    is_client: bool,
    server_read_pos: usize,
    client_read_pos: usize,
    server: Vec<u8>,
    client: Vec<u8>,
}

impl Stream {
    pub fn role_server(&mut self) {
        self.is_client = false
    }
    pub fn role_client(&mut self) {
        self.is_client = true
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.is_client {
            let pos = self.client_read_pos;
            let res = pin!(&self.client[pos..]).poll_read(cx, buf);
            self.client_read_pos += buf.filled().len();
            res
        } else {
            let pos = self.server_read_pos;
            let res = pin!(&self.server[pos..]).poll_read(cx, buf);
            self.server_read_pos += buf.filled().len();
            res
        }
    }
}

impl AsyncWrite for Stream {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.is_client {
            self.server.extend_from_slice(buf)
        } else {
            self.client.extend_from_slice(buf)
        }
        Poll::Ready(Ok(buf.len()))
    }

    #[inline]
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Write::write_vectored(
            if self.is_client {
                &mut self.server
            } else {
                &mut self.client
            },
            bufs,
        ))
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl Stream {
    pub fn new(capacity: usize) -> Self {
        Self {
            server: Vec::with_capacity(capacity),
            client: Vec::with_capacity(capacity),
            server_read_pos: 0,
            client_read_pos: 0,
            is_client: true,
        }
    }
}

// -------------------------

use std::task::*;

const DATA: () = ();
const VTABLE: RawWakerVTable =
    RawWakerVTable::new(|_| RawWaker::new(&DATA, &VTABLE), no_op, no_op, no_op);

fn no_op(_: *const ()) {}

pub fn block_on<Fut>(mut fut: Fut) -> Fut::Output
where
    Fut: std::future::Future,
{
    let waker = unsafe { Waker::from_raw(RawWaker::new(&DATA, &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(output) => break output,
            Poll::Pending => {}
        }
    }
}
