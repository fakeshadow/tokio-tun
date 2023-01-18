use core::{
    pin::Pin,
    task::{ready, Context, Poll},
};

use std::{
    io::{self, Read, Write},
    net::Ipv4Addr,
    os::unix::io::{AsRawFd, RawFd},
    sync::Arc,
};

use tokio::io::{unix::AsyncFd, AsyncRead, AsyncWrite, ReadBuf};

use crate::error::Error;
use crate::linux::interface::Interface;
use crate::linux::io::TunIo;
use crate::linux::params::Params;

/// Represents a Tun/Tap device. Use [`Builder`] to create a new instance of Self.
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
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        loop {
            let mut guard = ready!(this.io.poll_read_ready_mut(cx))?;
            // SAFETY:
            // work around as stable std lack read_buf feature.
            let b = unsafe { &mut *(buf.unfilled_mut() as *mut [u8]) };
            if let Ok(res) = guard.try_io(|inner| inner.get_mut().read(b)) {
                return Poll::Ready(res.map(|n| {
                    // SAFETY:
                    // TunIo is trusted to return correct count of bytes written to buffer.
                    unsafe {
                        buf.assume_init(n);
                    }
                    buf.advance(n);
                }));
            }
        }
    }
}

impl AsyncWrite for Tun {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        loop {
            let mut guard = ready!(this.io.poll_write_ready_mut(cx))?;
            if let Ok(res) = guard.try_io(|inner| inner.get_mut().write(buf)) {
                return Poll::Ready(res);
            };
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        loop {
            let mut guard = ready!(this.io.poll_write_ready_mut(cx))?;
            if let Ok(res) = guard.try_io(|inner| inner.get_mut().flush()) {
                return Poll::Ready(res);
            };
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}

impl Tun {
    /// Creates a new instance of Tun/Tap device.
    pub(crate) fn new(params: Params) -> Result<Self, Error> {
        let (iface, mut tuns) = Self::allocate(params, 1)?;
        let tun = tuns.pop().unwrap();
        Ok(Self {
            iface: Arc::new(iface),
            io: AsyncFd::new(tun)?,
        })
    }

    fn allocate(params: Params, queues: usize) -> Result<(Interface, Vec<TunIo>), Error> {
        let tuns = (0..queues)
            .map(|_| TunIo::try_from_path(b"/dev/net/tun\0"))
            .collect::<io::Result<Vec<_>>>()?;

        let iface = Interface::new(
            &tuns,
            params.name.as_deref().unwrap_or_default(),
            params.flags,
        )?;
        iface.init(params, &tuns)?;
        Ok((iface, tuns))
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
