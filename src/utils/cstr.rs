//! Fixed-size, NUL-terminated UTF-8 buffers for the original's `char name[N]` fields, so names
//! truncate exactly as `CopyUtf8` does and save files keep their layout.

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CStr<const N: usize> {
    buf: [u8; N],
}

impl<const N: usize> Default for CStr<N> {
    fn default() -> Self {
        CStr { buf: [0; N] }
    }
}

impl<const N: usize> CStr<N> {
    pub fn new(s: &str) -> Self {
        let mut c = Self::default();
        c.set(s);
        c
    }

    /// `CopyUtf8(buf, s, N)`: copies at most N-1 bytes without splitting a code point.
    pub fn set(&mut self, s: &str) {
        self.buf = [0; N];
        let t = crate::utils::utf8::copy_utf8(s, N);
        let b = t.as_bytes();
        let n = b.len().min(N.saturating_sub(1));
        self.buf[..n].copy_from_slice(&b[..n]);
    }

    pub fn as_str(&self) -> &str {
        let end = self.buf.iter().position(|&b| b == 0).unwrap_or(N);
        match std::str::from_utf8(&self.buf[..end]) {
            Ok(s) => s,
            Err(e) => std::str::from_utf8(&self.buf[..e.valid_up_to()]).unwrap_or(""),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buf[0] == 0
    }

    pub fn clear(&mut self) {
        self.buf = [0; N];
    }

    pub fn bytes(&self) -> &[u8; N] {
        &self.buf
    }

    /// Raw bytes as stored in save files (may contain anything after the NUL).
    pub fn from_raw(raw: &[u8]) -> Self {
        let mut c = Self::default();
        let n = raw.len().min(N);
        c.buf[..n].copy_from_slice(&raw[..n]);
        c
    }
}

impl<const N: usize> fmt::Debug for CStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<const N: usize> fmt::Display for CStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
