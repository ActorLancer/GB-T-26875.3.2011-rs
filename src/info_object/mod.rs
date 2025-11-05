//! GB26875 信息对象模块
//!
//! 根据 GB26875 协议第8.2.1节定义的信息对象实现

use crate::error::{EncodeResult, ParseResult};
use crate::frame::timestamp::Timestamp;
use bytes::Bytes;

/// 信息对象基础 trait
pub trait InfoObject: std::fmt::Debug + Send + Sync {
    /// 获取信息对象类型标识
    fn object_type(&self) -> u8;

    /// 获取信息对象描述
    fn description(&self) -> Option<&str> {
        None
    }

    /// 编码信息对象为字节序列
    fn encode(&self) -> EncodeResult<Bytes>;

    /// 从字节序列解析信息对象
    fn parse(data: &[u8]) -> ParseResult<Self>
    where
        Self: Sized;

    /// 获取信息对象的时间戳
    fn timestamp(&self) -> Option<&Timestamp> {
        None
    }
}

// 子模块声明
pub mod analog_value;
pub mod component_status;
pub mod config;
pub mod operation;
pub mod system_status;
pub mod version;

// 重新导出主要类型
pub use analog_value::{AnalogType, AnalogValue};
pub use component_status::ComponentStatus;
pub use config::{ComponentConfig, DeviceConfig, FireSystemConfig};
pub use operation::{DeviceOperation, FireSystemOperation};
pub use system_status::SystemStatus;
pub use version::{DeviceVersion, FireSystemVersion};
