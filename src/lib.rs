mod linux;

mod builder;
mod error;
mod tun;

pub use self::{builder::Builder, error::Error, tun::Tun};

#[cfg(not(target_os = "linux"))]
compile_error!("tokio-tun only support linux OS");
