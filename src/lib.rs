//! # GB26875 - 城市消防远程监控系统通信协议 Rust 实现
//!
//! 这个库提供了 GB26875 城市消防远程监控系统通信协议的完整 Rust 实现，
//! 包括数据帧解析、编码、自定义扩展机制以及网络传输支持。
//!
//! ## 快速开始
//!
//! ```rust
//! use gb26875::prelude::*;
//! use gb26875::frame::{Packet, ControlUnit, Timestamp};
//! use gb26875::protocol::{Command, ProtocolVersion};
//!
//! // 创建数据包
//! let control_unit = ControlUnit::new(
//!     1,                          // sequence
//!     ProtocolVersion::v1_0(),    // version
//!     Timestamp::now(),           // timestamp
//!     0x123456,                   // source_addr
//!     0x654321,                   // dest_addr
//!     0,                          // data_unit_len
//!     Command::Heartbeat          // command
//! );
//! let packet = Packet::without_data(control_unit)?;
//!
//! // 编码为字节
//! let encoded = packet.encode()?;
//!
//! // 解析数据包
//! let parsed_packet = Packet::parse(&encoded)?;
//! # Ok::<(), gb26875::error::EncodeError>(())
//! ```
//!
//! ## 使用构建器 API
//!
//! ```rust
//! use gb26875::builder::PacketBuilder;
//! use gb26875::protocol::Command;
//!
//! // 使用构建器创建心跳包
//! let packet = PacketBuilder::heartbeat(1, 0x123456, 0x654321)?
//!     .build()?;
//! # Ok::<(), gb26875::error::EncodeError>(())
//! ```
//!
//! ## Features
//!
//! - `serde`: 启用 JSON 等格式的序列化支持
//! - `async`: 启用异步网络传输支持  
//! - `logging`: 启用日志输出
//! - `full`: 启用所有功能
//!
//! ## 扩展机制
//!
//! 支持用户自定义类型（128-255 范围）：
//!
//! ```rust
//! use gb26875::extension::*;
//! use bytes::Bytes;
//!
//! // 定义自定义数据单元
//! #[derive(Debug)]
//! struct MyCustomData {
//!     value: u32,
//! }
//!
//! impl ExtensionDataUnit for MyCustomData {
//!     fn type_id(&self) -> u8 { 200 }
//!     fn encode(&self) -> ExtensionResult<Bytes> {
//!         Ok(Bytes::copy_from_slice(&self.value.to_le_bytes()))
//!     }
//!     fn as_any(&self) -> &dyn std::any::Any { self }
//! }
//!
//! // 注册扩展
//! ExtensionManager::register_global(200, "MyCustomData".to_string(),
//!     Box::new(|data| Ok(Box::new(MyCustomData { value: 42 }))))?;
//! # Ok::<(), gb26875::extension::ExtensionError>(())
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// 公共模块导出
pub mod builder;
pub mod codec;
pub mod data_unit;
pub mod error;
pub mod extension;
pub mod frame;
pub mod info_object;
pub mod parser;
pub mod protocol;

// 可选模块
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod transport;

// 预导入模块
pub mod prelude;

// 重新导出核心类型
pub use error::{EncodeError, ExtensionError, ParseError};
pub use frame::Packet;

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 支持的协议版本
pub const PROTOCOL_VERSION: (u8, u8) = (1, 1); // 主版本号.用户版本号
