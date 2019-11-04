use crate::errno::on_error;

use std::os::unix::io::RawFd;
use std::ptr::null;
use std::ptr::NonNull;

pub const EOF: u16 = 0xffff;

pub trait CharsLike {
    fn next_char(&mut self) -> u16;
}

/// Chars generates chars from fd. It terminates the process with errno when an error occurs.
pub struct Chars {
    s: *const u8,
    e: *const u8,
    buf: NonNull<u8>,
    buf_len: usize,
    fd: RawFd,
}

impl Drop for Chars {
    fn drop(&mut self) {
        // ignore error
        unsafe {
            let _ = libc::close(self.fd);
        }
    }
}

impl Chars {
    pub unsafe fn with_buf(buf: &mut [u8], fd: RawFd) -> Self {
        assert!(buf.len() > 0);
        Self {
            s: null(),
            e: null(),
            buf: NonNull::new_unchecked(buf.as_mut_ptr()),
            buf_len: buf.len(),
            fd,
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

impl CharsLike for Chars {
    #[inline(always)]
    fn next_char(&mut self) -> u16 {
        unsafe {
            if self.s < self.e {
                let c = u16::from(*self.s);
                self.s = self.s.add(1);
                return c;
            }
            let len = self.fill_buf();
            if len == 0 {
                return EOF;
            }
            let s: *const u8 = self.buf.as_ptr() as *mut u8 as _;
            self.s = s.add(1);
            self.e = s.add(len as usize);
            u16::from(*s)
        }
    }
}
