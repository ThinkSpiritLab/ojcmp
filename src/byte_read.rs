use std::io::{BufRead, Read};

pub trait ByteRead: Read {
    fn next_byte(&mut self) -> Option<u8>;
}

impl ByteRead for &'_ [u8] {
    fn next_byte(&mut self) -> Option<u8> {
        match self {
            [] => None,
            [byte, remain @ ..] => {
                *self = remain;
                Some(*byte)
            }
        }
    }
}

#[derive(Debug)]
pub struct ByteReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: Read> ByteReader<R> {
    pub fn with_capacity(capacity: usize, reader: R) -> Self {
        Self {
            inner: reader,
            buf: vec![0; capacity].into(),
            pos: 0,
            cap: 0,
        }
    }

    pub unsafe fn from_raw(buf: *mut [u8], reader: R) -> Self {
        Self {
            inner: reader,
            buf: Box::from_raw(buf),
            pos: 0,
            cap: 0,
        }
    }
}

impl<R: Read> Read for ByteReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut inner_buf = self.fill_buf()?;
        let nread = inner_buf.read(buf)?;
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read> BufRead for ByteReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.pos < self.cap {
            Ok(&self.buf[self.pos..self.cap])
        } else {
            let nread = self.inner.read(&mut *self.buf)?;
            if nread == 0 {
                Ok(&[])
            } else {
                self.pos = 0;
                self.cap = nread;
                Ok(&self.buf[..nread])
            }
        }
    }

    fn consume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_add(amt).min(self.cap);
    }
}

impl<R: Read> ByteRead for ByteReader<R> {
    #[inline(always)]
    fn next_byte(&mut self) -> Option<u8> {
        debug_assert!(self.cap <= self.buf.len());
        if self.pos < self.cap {
            let byte = self.buf[self.pos];
            self.pos += 1;
            Some(byte)
        } else {
            match self.inner.read(&mut *self.buf) {
                Ok(nread) => {
                    if nread == 0 {
                        None
                    } else {
                        let byte = self.buf[0];
                        self.cap = nread;
                        self.pos = 1;
                        Some(byte)
                    }
                }
                Err(e) => panic!(e),
            }
        }
    }
}
