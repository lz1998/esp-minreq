use alloc::boxed::Box;
use core::cmp;
use embedded_io_async::{BufRead, ErrorType, Read};

/// The `BufReader` struct adds buffering to any reader.
///
/// It can be excessively inefficient to work directly with a [`AsyncRead`]
/// instance. A `BufReader` performs large, infrequent reads on the underlying
/// [`AsyncRead`] and maintains an in-memory buffer of the results.
///
/// `BufReader` can improve the speed of programs that make *small* and
/// *repeated* read calls to the same file or network socket. It does not
/// help when reading very large amounts at once, or reading just one or a few
/// times. It also provides no advantage when reading from a source that is
/// already in memory, like a `Vec<u8>`.
///
/// When the `BufReader` is dropped, the contents of its buffer will be
/// discarded. Creating multiple instances of a `BufReader` on the same
/// stream can cause data loss.
///
/// [`AsyncRead`]: futures_io::AsyncRead
///
// TODO: Examples
pub struct BufReader<R> {
    inner: R,
    buffer: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: Read> BufReader<R> {
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        let buffer = alloc::vec::from_elem(0, capacity);
        Self {
            inner,
            buffer: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,
        }
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// Unlike `fill_buf`, this will not attempt to fill the buffer if it is empty.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[self.pos..self.cap]
    }

    /// Invalidates all data in the internal buffer.
    #[inline]
    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.cap = 0;
    }
}

impl<R: Read> ErrorType for BufReader<R> {
    type Error = R::Error;
}

impl<R: Read> Read for BufReader<R> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.pos == self.cap && buf.len() >= self.buffer.len() {
            let res = self.inner.read(buf).await;
            self.discard_buffer();
            return res;
        }
        let mut rem = self.fill_buf().await?;
        let nread = rem.read(buf).await.unwrap();
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read> BufRead for BufReader<R> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buffer).await?;
            self.pos = 0;
        }
        Ok(&self.buffer[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}
