use std::io::{BufRead, Read};
use std::ptr;
use std::slice;

#[derive(Debug, Clone, Copy)]
pub struct IoByte {
    byte: u8,
    eof: bool,
}

impl IoByte {
    pub const EOF: IoByte = IoByte { byte: 0, eof: true };

    #[inline(always)]
    pub fn from_u8(byte: u8) -> Self {
        Self { byte, eof: false }
    }

    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self.byte
    }

    #[inline(always)]
    pub fn is_eof(self) -> bool {
        self.eof
    }
}

pub trait ByteRead: Read {
    fn next_byte(&mut self) -> IoByte;
}

impl ByteRead for &'_ [u8] {
    fn next_byte(&mut self) -> IoByte {
        match self {
            [] => IoByte::EOF,
            [byte, remain @ ..] => {
                *self = remain;
                IoByte::from_u8(*byte)
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

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_raw(self) -> (*mut [u8], R) {
        let buf = Box::into_raw(self.buf);
        let reader = self.inner;
        (buf, reader)
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
    fn next_byte(&mut self) -> IoByte {
        if self.head != self.tail {
            unsafe {
                let byte = *self.head;
                self.head = self.head.add(1);
                IoByte::from_u8(byte)
            }
        } else {
            match self.inner.read(&mut *self.buf) {
                Ok(nread) => {
                    if nread == 0 {
                        IoByte::EOF
                    } else {
                        unsafe {
                            let byte = *self.buf.as_ptr();
                            self.head = self.buf.as_ptr().add(1);
                            self.tail = self.buf.as_ptr().add(nread);
                            IoByte::from_u8(byte)
                        }
                    }
                }
                Err(e) => panic!(e),
            }
        }
    }
}

#[cfg(unix)]
pub mod unix {
    use std::fs::File;
    use std::io::{self, Read};
    use std::os::raw::c_void;
    use std::os::unix::io::AsRawFd;

    #[derive(Debug)]
    pub struct UnixFdReader {
        file: File,
    }

    impl UnixFdReader {
        pub fn from_file(file: File) -> Self {
            Self { file }
        }
    }

    impl Read for UnixFdReader {
        #[inline(always)]
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            unsafe {
                let buf_ptr: *mut c_void = buf.as_mut_ptr().cast();
                let fd = self.file.as_raw_fd();
                let ret: isize = libc::read(fd, buf_ptr, buf.len());
                if ret < 0 {
                    panic!(io::Error::last_os_error())
                }
                Ok(ret as usize)
            }
        }
    }
}
