//! `TcpStream` split support.
//!
//! A `TcpStream` can be split into a `ReadHalf` and a
//! `WriteHalf` with the `TcpStream::split` method. `ReadHalf`
//! implements `AsyncRead` while `WriteHalf` implements `AsyncWrite`.
//!
//! Compared to the generic split of `AsyncRead + AsyncWrite`, this specialized
//! split has no associated overhead and enforces all invariants at the type
//! level.

use super::TcpStream;

use tokio_io::{AsyncRead, AsyncWrite};

use bytes::{Buf, BufMut};
use std::io;
use std::net::Shutdown;
use std::pin::Pin;
use std::task::{Context, Poll};

use std::sync::Arc;

/// Read half of a `TcpStream`.
#[derive(Debug)]
pub struct ReadHalf(Arc<TcpStream>);

/// Write half of a `TcpStream`.
///
/// Note that in the `AsyncWrite` implemenation of `TcpStreamWriteHalf`,
/// `poll_shutdown` actually shuts down the TCP stream in the write direction.
#[derive(Debug)]
pub struct WriteHalf(Arc<TcpStream>);

pub(crate) fn split(stream: TcpStream) -> (ReadHalf, WriteHalf) {
    let stream = Arc::new(stream);
    let stream2 = Arc::clone(&stream);
    (ReadHalf(stream), WriteHalf(stream2))
}

impl AsyncRead for ReadHalf {
    unsafe fn prepare_uninitialized_buffer(&self, _: &mut [u8]) -> bool {
        false
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.0.poll_read_priv(cx, buf)
    }

    fn poll_read_buf<B: BufMut>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>> {
        self.0.poll_read_buf_priv(cx, buf)
    }
}

impl AsyncWrite for WriteHalf {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.poll_write_priv(cx, buf)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // tcp flush is a no-op
        Poll::Ready(Ok(()))
    }

    // `poll_shutdown` on a write half shutdowns the stream in the "write" direction.
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.shutdown(Shutdown::Write).into()
    }

    fn poll_write_buf<B: Buf>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>> {
        self.0.poll_write_buf_priv(cx, buf)
    }
}

impl AsRef<TcpStream> for ReadHalf {
    fn as_ref(&self) -> &TcpStream {
        self.0.as_ref()
    }
}

impl AsRef<TcpStream> for WriteHalf {
    fn as_ref(&self) -> &TcpStream {
        self.0.as_ref()
    }
}
