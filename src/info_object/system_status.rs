//! GB26875 建筑消防设施系统状态信息对象
//!
//! 根据 GB26875 协议第8.2.1节实现的系统状态信息对象，用于上传建筑消防设施系统的运行状态。

use super::InfoObject;
use crate::error::{EncodeResult, ParseResult};
use crate::frame::timestamp::Timestamp;
use crate::protocol::types::SystemType;
use bytes::Bytes;

/// 建筑消防设施系统状态 (4字节信息体 + 6字节时间戳)
///
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施系统状态信息
///
/// ## 字段布局
///
/// | 字段名        | 字节数 | 说明              |
/// |--------------|-------|-------------------|
/// | 系统类型标志   | 1     | 建筑消防设施系统类型 |
/// | 系统地址      | 1     | 建筑消防设施系统地址 |
/// | 系统状态      | 2     | 系统运行状态(小端序) |
/// | 状态发生时间   | 6     | 时间戳            |
///
/// ## 示例
///
/// ```rust
/// use gb26875::info_object::system_status::SystemStatus;
/// use gb26875::protocol::types::SystemType;
/// use gb26875::frame::timestamp::Timestamp;
///
/// let status = SystemStatus::new(
///     SystemType::FireAlarm,
///     1,         // 系统地址
///     0x0002,    // 火警状态
///     Timestamp::now()
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SystemStatus {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 系统状态 (2字节，小端序)
    pub system_state: u16,
    /// 状态发生时间 (6字节)
    pub timestamp: Timestamp,
}

impl SystemStatus {
    /// 创建新的系统状态
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        system_state: u16,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            system_state,
            timestamp,
        }
    }
}

impl InfoObject for SystemStatus {
    fn object_type(&self) -> u8 {
        1 // 上传建筑消防设施系统状态
    }

    fn description(&self) -> Option<&str> {
        Some("建筑消防设施系统状态")
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(10); // 4字节信息体 + 6字节时间戳

        // 信息体 (4字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.extend_from_slice(&self.system_state.to_le_bytes()); // 小端序

        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());

        Ok(Bytes::from(buf))
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 10 {
            return Err(crate::error::ParseError::TooShort {
                expected: 10,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];
        let system_state = u16::from_le_bytes([data[2], data[3]]);
        let timestamp = Timestamp::from_bytes(&data[4..10])?;

        Ok(SystemStatus::new(
            system_type,
            system_address,
            system_state,
            timestamp,
        ))
    }

    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

mod tests {
    use super::*;
    use crate::protocol::types::SystemType;

    #[test]
    fn test_system_status_encode_decode() {
        let timestamp = Timestamp::now();
        let status = SystemStatus::new(SystemType::FireAlarm, 1, 0x0002, timestamp);

        // 测试编码
        let encoded = status.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 10);

        // 测试解码
        let decoded = SystemStatus::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, status);
    }

    #[test]
    fn test_system_status_fields() {
        let timestamp = Timestamp::now();
        let status = SystemStatus::new(SystemType::FireAlarm, 5, 0x0004, timestamp);

        assert_eq!(status.object_type(), 1);
        assert_eq!(status.system_type, SystemType::FireAlarm);
        assert_eq!(status.system_address, 5);
        assert_eq!(status.system_state, 0x0004);
        assert_eq!(status.description(), Some("建筑消防设施系统状态"));
        assert!(status.timestamp().is_some());
    }
}
