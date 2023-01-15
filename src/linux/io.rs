use std::{
    io::{self, Read, Write},
    os::{
        raw::c_char,
        unix::io::{AsRawFd, FromRawFd, RawFd},
    },
};

use super::syscall;

pub struct TunIo(RawFd);

impl TunIo {
    pub fn from_path(path: &[u8]) -> Self {
        // SAFETY:
        // call to libc.
        unsafe {
            let fd = libc::open(
                path.as_ptr().cast::<c_char>(),
                libc::O_RDWR | libc::O_NONBLOCK,
            );

            Self(FromRawFd::from_raw_fd(fd))
        }
    }
}

impl FromRawFd for TunIo {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(fd)
    }
}

impl AsRawFd for TunIo {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Read for TunIo {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        syscall!(read(self.0, buf.as_ptr() as *mut _, buf.len() as _)).map(|n| n as _)
    }
}

impl Write for TunIo {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        syscall!(write(self.0, buf.as_ptr() as *const _, buf.len() as _)).map(|n| n as _)
    }

    fn flush(&mut self) -> io::Result<()> {
        syscall!(fsync(self.0)).map(|_| ())
    }
}

impl Drop for TunIo {
    fn drop(&mut self) {
        let _ = syscall!(close(self.0));
    }
}
