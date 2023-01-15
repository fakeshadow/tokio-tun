pub(crate) mod addr_ext;
pub(crate) mod interface;
pub(crate) mod io;
pub(crate) mod params;
pub(crate) mod request;

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

use syscall;
