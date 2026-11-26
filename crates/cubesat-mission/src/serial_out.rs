//! The text the mission writes, held until the serial link takes it.
//!
//! Writes are all or nothing: a line that doesn't fit is refused whole. The link reports how
//! many bytes it really sent with [`SerialOut::sent`], and the rest stays at the front, so the
//! next send carries on from that exact byte on the same connection. A line is never cut and
//! started again, which would put two record prefixes on one line.

use core::fmt;

/// Up to `N` bytes of mission output waiting for the serial link.
pub struct SerialOut<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for SerialOut<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> SerialOut<N> {
    /// An empty buffer.
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    /// The bytes still waiting to go out, oldest first.
    pub fn pending(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Nothing is waiting to go out.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The link sent the first `n` pending bytes; keep the rest at the front.
    pub fn sent(&mut self, n: usize) {
        let n = n.min(self.len);
        self.buf.copy_within(n..self.len, 0);
        self.len -= n;
    }

    /// Drop everything, for example when the terminal closes.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> fmt::Write for SerialOut<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > N {
            return Err(fmt::Error);
        }
        self.buf[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}
