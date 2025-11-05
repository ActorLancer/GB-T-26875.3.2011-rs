//! GB26875 传输层模块
//!
//! 提供网络传输功能，包括 TCP/UDP 客户端和服务器

/// TCP 传输实现
pub mod tcp;

/// UDP 传输实现
pub mod udp;

pub use tcp::*;
pub use udp::*;
