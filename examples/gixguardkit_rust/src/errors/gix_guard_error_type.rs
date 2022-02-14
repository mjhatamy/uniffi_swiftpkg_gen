use std::error::Error;
use std::fmt;

#[derive(Debug)]
#[repr(u32)]
pub enum GixTunnelErrorKind {
    #[allow(unused)]
    Ok = 0x0,
    Failed = 0x1000,
    NullInput = 0x1001,
    InvalidInput = 0x1002,
    InvalidInputLength = 0x1003,
}

impl GixTunnelErrorKind {
    pub(crate) fn errno(&self) -> i32 {
        use GixTunnelErrorKind::*;
        match *self {
            Ok => 0x0,
            Failed => 0x1000,
            NullInput => 0x1001,
            InvalidInput => 0x1002,
            InvalidInputLength => 0x1003,
        }
    }
}
impl fmt::Display for GixTunnelErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConfigError(errno = {})", self.errno())
    }
}

impl Error for GixTunnelErrorKind {
    fn description(&self) -> &str {
        ""
    }
}
