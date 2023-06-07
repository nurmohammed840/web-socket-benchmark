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
    pub fn new(capacity: usize) -> Self {
        Self {
            server: Vec::with_capacity(capacity),
            client: Vec::with_capacity(capacity),
            server_read_pos: 0,
            client_read_pos: 0,
            is_client: true,
        }
    }

    pub fn role_server(&mut self) {
        self.is_client = false
    }
    pub fn role_client(&mut self) {
        self.is_client = true
    }

    fn _poll_write(&mut self, _: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        if self.is_client {
            self.server.extend_from_slice(buf)
        } else {
            self.client.extend_from_slice(buf)
        }
        Poll::Ready(Ok(buf.len()))
    }

    fn _poll_write_vectored(
        &mut self,
        _: &mut Context,
        bufs: &[io::IoSlice],
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
}

impl futures_util::AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.is_client {
            let pos = self.client_read_pos;
            let res = futures_util::AsyncRead::poll_read(pin!(&self.client[pos..]), cx, buf);
            self.client_read_pos += buf.len();
            res
        } else {
            let pos = self.server_read_pos;
            let res = futures_util::AsyncRead::poll_read(pin!(&self.server[pos..]), cx, buf);
            self.server_read_pos += buf.len();
            res
        }
    }
}
impl futures_util::AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self._poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[io::IoSlice],
    ) -> Poll<io::Result<usize>> {
        self._poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        if self.is_client {
            let pos = self.client_read_pos;
            let res = AsyncRead::poll_read(pin!(&self.client[pos..]), cx, buf);
            self.client_read_pos += buf.filled().len();
            res
        } else {
            let pos = self.server_read_pos;
            let res = AsyncRead::poll_read(pin!(&self.server[pos..]), cx, buf);
            self.server_read_pos += buf.filled().len();
            res
        }
    }
}
impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self._poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[io::IoSlice],
    ) -> Poll<io::Result<usize>> {
        self._poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

// -------------------------

use std::task::*;

static DATA: () = ();
static VTABLE: RawWakerVTable = RawWakerVTable::new(|_| raw_waker(), no_op, no_op, no_op);
fn raw_waker() -> RawWaker {
    RawWaker::new(&DATA, &VTABLE)
}

fn no_op(_: *const ()) {}

pub fn block_on<Fut>(mut fut: Fut) -> Fut::Output
where
    Fut: std::future::Future,
{
    let waker = unsafe { Waker::from_raw(raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(output) => break output,
            Poll::Pending => {}
        }
    }
}
