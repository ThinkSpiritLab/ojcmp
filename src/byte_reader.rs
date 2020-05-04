use std::io::Cursor;
use std::io::Read;
use std::marker::PhantomData;
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr::{self, NonNull};

#[derive(Debug, Clone, Copy)]
pub struct Byte {
    byte: u8,
    is_eof: bool,
}

pub struct ByteReader<'buf, 'fd> {
    s: *const u8,
    e: *const u8,
    buf: NonNull<u8>,
    buf_len: usize,
    fd: RawFd,
    _marker: PhantomData<(&'buf mut [u8], &'fd mut ())>,
}

pub trait ByteReaderLike {
    fn next_byte(&mut self) -> Byte;

    fn next_buf(&mut self) -> Option<&[u8]>;
}

impl Byte {
    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self.byte
    }

    #[inline(always)]
    pub fn is_eof(self) -> bool {
        if self.is_eof {
            debug_assert_eq!(self.byte, 0);
        }
        self.is_eof
    }

    fn from_u8(byte: u8) -> Self {
        Self {
            byte,
            is_eof: false,
        }
    }

    fn from_eof() -> Self {
        Self {
            byte: 0,
            is_eof: true,
        }
    }
}

impl<'buf, 'fd> ByteReader<'buf, 'fd> {
    pub fn new<F: AsRawFd + ?Sized>(fd: &'fd mut F, buf: &'buf mut [u8]) -> Self {
        assert!(!buf.is_empty());

        Self {
            s: ptr::null(),
            e: ptr::null(),
            buf: unsafe { NonNull::new_unchecked(buf.as_mut_ptr()) },
            buf_len: buf.len(),
            fd: fd.as_raw_fd(),
            _marker: PhantomData,
        }
    }
}

impl ByteReader<'_, '_> {
    #[inline(always)]
    unsafe fn fill_buf(&mut self) -> usize {
        use libc::c_void;
        let buf_ptr: *mut c_void = self.buf.as_ptr().cast();
        let ret: isize = libc::read(self.fd, buf_ptr, self.buf_len);
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            let code = err.raw_os_error().unwrap();
            crate::error::exit(code, err)
        }
        ret as _
    }

    /// Reads bytes until EOF.
    pub fn wait_all(mut self) {
        unsafe { while self.fill_buf() > 0 {} }
    }
}

impl ByteReaderLike for ByteReader<'_, '_> {
    fn next_byte(&mut self) -> Byte {
        unsafe {
            debug_assert!(self.s <= self.e);

            {
                let s = self.s;
                let e = self.e;
                if s < e {
                    let byte = *s;
                    self.s = s.add(1);
                    return Byte::from_u8(byte);
                }
            }
            {
                let len = self.fill_buf();
                if len == 0 {
                    return Byte::from_eof();
                }
                let s: *const u8 = self.buf.as_ptr();
                let byte = *s;
                self.s = s.add(1);
                self.e = s.add(len as usize);
                Byte::from_u8(byte)
            }
        }
    }

    fn next_buf(&mut self) -> Option<&[u8]> {
        unsafe {
            if self.s < self.e {
                // let len = self.e.offset_from(self.s) as usize;
                let len = self.e as usize - self.s as usize;

                let ans: &[u8] = std::slice::from_raw_parts(self.s, len);

                self.s = ptr::null();
                self.e = ptr::null();
                return Some(ans);
            }

            let len = self.fill_buf();
            if len != 0 {
                let buf_ptr = self.buf.as_ptr();
                let ans: &[u8] = std::slice::from_raw_parts(buf_ptr, len);
                Some(ans)
            } else {
                None
            }
        }
    }
}

impl ByteReaderLike for Cursor<&'_ [u8]> {
    fn next_byte(&mut self) -> Byte {
        let mut buf = [0_u8; 1];
        match self.read_exact(&mut buf) {
            Ok(_) => Byte::from_u8(buf[0]),
            Err(_) => Byte::from_eof(),
        }
    }

    fn next_buf(&mut self) -> Option<&[u8]> {
        use std::io::BufRead;

        let pos = self.position() as usize;
        let &buf = self.get_ref();

        if pos < buf.len() {
            self.consume(buf.len() - pos);
            Some(&buf[pos..])
        } else {
            None
        }
    }
}
