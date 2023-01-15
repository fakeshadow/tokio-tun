mod linux {
    pub mod address;
    pub mod interface;
    pub mod io;
    pub mod params;
    pub mod request;
}

mod builder;
mod error;
mod tun;

pub use self::{builder::TunBuilder, error::Error, tun::Tun};

#[cfg(not(target_os = "linux"))]
compile_error!("tokio-tun only support linux OS");
