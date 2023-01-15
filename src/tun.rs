use core::{
    pin::Pin,
    task::{self, ready, Context, Poll},
};

use std::net::Ipv4Addr;
use std::os::raw::c_char;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    os::fd::FromRawFd,
};

use tokio::io::{unix::AsyncFd, AsyncRead, AsyncWrite, ReadBuf};

use crate::error::Error;
use crate::linux::interface::Interface;
use crate::linux::io::TunIo;
use crate::linux::params::Params;

/// Represents a Tun/Tap device. Use [`TunBuilder`](struct.TunBuilder.html) to create a new instance of [`Tun`](struct.Tun.html).
pub struct Tun {
    iface: Arc<Interface>,
    io: AsyncFd<TunIo>,
}

impl AsRawFd for Tun {
    fn as_raw_fd(&self) -> RawFd {
        self.io.as_raw_fd()
    }
}

impl AsyncRead for Tun {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        let this = self.get_mut();
        loop {
            let mut guard = ready!(this.io.poll_read_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().read(buf.initialize_unfilled())) {
                Ok(Ok(n)) => {
                    buf.set_filled(buf.filled().len() + n);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_) => continue,
            }
        }
    }
}

impl AsyncWrite for Tun {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> task::Poll<io::Result<usize>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.io.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> task::Poll<io::Result<()>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.io.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().flush()) {
                Ok(result) => return Poll::Ready(result),
                Err(_) => continue,
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> task::Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}

impl Tun {
    /// Creates a new instance of Tun/Tap device.
    pub(crate) fn new(params: Params) -> Result<Self, Error> {
        let iface = Self::allocate(params, 1)?;
        let fd = iface.files()[0];
        Ok(Self {
            iface: Arc::new(iface),
            // SAFETY:
            // TODO. currently this is not safe.
            io: AsyncFd::new(unsafe { FromRawFd::from_raw_fd(fd) })?,
        })
    }

    fn allocate(params: Params, queues: usize) -> Result<Interface, Error> {
        static TUN: &[u8] = b"/dev/net/tun\0";

        let fds = (0..queues)
            .map(|_| unsafe {
                libc::open(
                    TUN.as_ptr().cast::<c_char>(),
                    libc::O_RDWR | libc::O_NONBLOCK,
                )
            })
            .collect::<Vec<_>>();

        let iface = Interface::new(
            fds,
            params.name.as_deref().unwrap_or_default(),
            params.flags,
        )?;
        iface.init(params)?;
        Ok(iface)
    }

    /// Returns the name of Tun/Tap device.
    pub fn name(&self) -> &str {
        self.iface.name()
    }

    /// Returns the value of MTU.
    pub fn mtu(&self) -> Result<i32, Error> {
        self.iface.mtu(None)
    }

    /// Returns the IPv4 address of MTU.
    pub fn address(&self) -> Result<Ipv4Addr, Error> {
        self.iface.address(None)
    }

    /// Returns the IPv4 destination address of MTU.
    pub fn destination(&self) -> Result<Ipv4Addr, Error> {
        self.iface.destination(None)
    }

    /// Returns the IPv4 broadcast address of MTU.
    pub fn broadcast(&self) -> Result<Ipv4Addr, Error> {
        self.iface.broadcast(None)
    }

    /// Returns the IPv4 netmask address of MTU.
    pub fn netmask(&self) -> Result<Ipv4Addr, Error> {
        self.iface.netmask(None)
    }

    /// Returns the flags of MTU.
    pub fn flags(&self) -> Result<i16, Error> {
        self.iface.flags(None)
    }
}
