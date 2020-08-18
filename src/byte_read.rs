use std::io::{BufRead, Read};
use std::ptr;
use std::slice;

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
    head: *const u8,
    tail: *const u8,
}

impl<R: Read> ByteReader<R> {
    pub fn with_capacity(capacity: usize, reader: R) -> Self {
        Self {
            inner: reader,
            buf: vec![0; capacity].into(),
            head: ptr::null(),
            tail: ptr::null(),
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw(buf: *mut [u8], reader: R) -> Self {
        Self {
            inner: reader,
            buf: Box::from_raw(buf),
            head: ptr::null(),
            tail: ptr::null(),
        }
    }
}

impl<R: Read> Read for ByteReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> BufRead for ByteReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.head != self.tail {
            let len = self.tail as usize - self.head as usize;
            Ok(unsafe { slice::from_raw_parts(self.head, len) })
        } else {
            let nread = self.inner.read(&mut *self.buf)?;
            if nread == 0 {
                self.head = ptr::null();
                self.tail = ptr::null();
                Ok(&[])
            } else {
                self.head = self.buf.as_ptr();
                self.tail = unsafe { self.head.add(nread) };
                Ok(unsafe { slice::from_raw_parts(self.head, nread) })
            }
        }
    }

    fn consume(&mut self, amt: usize) {
        self.head = (self.head as usize)
            .saturating_add(amt)
            .min(self.tail as usize) as *const u8;
    }
}

impl<R: Read> ByteRead for ByteReader<R> {
    #[inline(always)]
    fn next_byte(&mut self) -> Option<u8> {
        if self.head != self.tail {
            unsafe {
                let byte = *self.head;
                self.head = self.head.add(1);
                Some(byte)
            }
        } else {
            match self.inner.read(&mut *self.buf) {
                Ok(nread) => {
                    if nread == 0 {
                        None
                    } else {
                        unsafe {
                            let byte = *self.buf.as_ptr();
                            self.head = self.buf.as_ptr().add(1);
                            self.tail = self.buf.as_ptr().add(nread);
                            Some(byte)
                        }
                    }
                }
                Err(e) => panic!(e),
            }
        }
    }
}
