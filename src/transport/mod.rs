//! GB26875 传输层模块
//!
//! 提供网络传输功能，包括 TCP/UDP 客户端和服务器

#[cfg(feature = "async")]
/// TCP 传输实现
pub mod tcp;

#[cfg(feature = "async")]
/// UDP 传输实现
pub mod udp;

#[cfg(feature = "async")]
pub use tcp::*;

#[cfg(feature = "async")]
pub use udp::*;

// 当未启用 async 功能时的占位符
#[cfg(not(feature = "async"))]
pub struct AsyncNotEnabled;

#[cfg(not(feature = "async"))]
impl AsyncNotEnabled {
    pub fn new() -> Self {
        AsyncNotEnabled
    }
}
