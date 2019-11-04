use crate::errno::on_error;

use std::os::unix::io::RawFd;
use std::ptr::null;

pub const EOF: u16 = 0xffff;

pub trait CharsLike {
    fn next_char(&mut self) -> u16;
    fn next_char_strip_cr(&mut self) -> u16;
}

/// Chars generates chars from fd. It terminates the process with errno when an error occurs.
pub struct Chars {
    s: *const u8,
    e: *const u8,
    buf: Box<[u8]>,
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
    pub fn with_capacity(cap: usize, fd: RawFd) -> Self {
        assert!(cap > 0);
        let buf: Box<[u8]> = unsafe {
            let mut buf = Vec::with_capacity(cap);
            buf.set_len(cap);
            buf.into_boxed_slice()
        };
        Self {
            s: null(),
            e: null(),
            buf,
            fd,
        }
    }

    #[inline(always)]
    unsafe fn fill_buf(&mut self) -> usize {
        use libc::c_void;
        let buf: *mut c_void = self.buf.as_mut_ptr() as _;
        let ret = libc::read(self.fd, buf, self.buf.len());
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

impl Chars {
    /// precondition: buf has valid data
    /// precondition: self.s > self.buf.as_ptr()
    #[inline(always)]
    unsafe fn go_back(&mut self) {
        self.s = self.s.sub(1)
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

    #[inline(always)]
    fn next_char_strip_cr(&mut self) -> u16 {
        let c = self.next_char();
        if c == EOF || (c as u8) != b'\r' {
            return c;
        }
        let n = self.next_char();
        if n == EOF {
            return u16::from(b'\r');
        }
        if (n as u8) == b'\n' {
            return u16::from(b'\n');
        }
        unsafe { self.go_back() };
        u16::from(b'\r')
    }
}
