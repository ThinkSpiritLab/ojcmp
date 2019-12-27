use crate::errno::on_error;

use std::marker::PhantomData;
use std::os::unix::io::RawFd;
use std::ptr::null;
use std::ptr::NonNull;

pub const EOF: u16 = 0xffff;

#[derive(Clone, Copy)]
pub struct CharEx(u16);

impl CharEx {
    #[inline(always)]
    pub fn is_eof(self) -> bool {
        self.0 == EOF
    }

    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self.0 as _
    }
}

impl From<u16> for CharEx {
    fn from(x: u16) -> Self {
        Self(x)
    }
}

pub trait CharsLike {
    fn next_char(&mut self) -> CharEx;
}

/// Chars generates chars from fd.
/// It terminates the process with errno when an error occurs.
/// Chars depends on outer buffer.
pub struct Chars<'a> {
    s: *const u8,
    e: *const u8,
    buf: NonNull<u8>,
    buf_len: usize,
    fd: RawFd,
    _marker: PhantomData<&'a mut [u8]>,
}

impl Drop for Chars<'_> {
    fn drop(&mut self) {
        // ignore error
        unsafe {
            let _ = libc::close(self.fd);
        }
    }
}

impl<'a> Chars<'a> {
    pub fn with_buf(buf: &'a mut [u8], fd: RawFd) -> Self {
        assert!(!buf.is_empty());
        Self {
            s: null(),
            e: null(),
            buf: unsafe { NonNull::new_unchecked(buf.as_mut_ptr()) },
            buf_len: buf.len(),
            fd,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    unsafe fn fill_buf(&mut self) -> usize {
        use libc::c_void;
        let buf: *mut c_void = self.buf.as_ptr() as _;
        let ret = libc::read(self.fd, buf, self.buf_len);
        if ret < 0 {
            on_error()
        }
        ret as _
    }

    /// Read bytes until EOF
    pub fn drop_all(mut self) {
        unsafe { while self.fill_buf() > 0 {} }
    }
}

impl CharsLike for Chars<'_> {
    #[inline(always)]
    fn next_char(&mut self) -> CharEx {
        unsafe {
            if self.s < self.e {
                let c = u16::from(*self.s);
                self.s = self.s.add(1);
                return c.into();
            }
            let len = self.fill_buf();
            if len == 0 {
                return EOF.into();
            }
            let s: *const u8 = self.buf.as_ptr() as *mut u8 as _;
            self.s = s.add(1);
            self.e = s.add(len as usize);
            u16::from(*s).into()
        }
    }
}
