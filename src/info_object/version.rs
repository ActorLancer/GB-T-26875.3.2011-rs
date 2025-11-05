//! GB26875 软件版本信息对象
//!
//! 根据 GB26875 协议第8.2.1节实现的软件版本信息对象，包括建筑消防设施软件版本和用户信息传输装置软件版本。

use super::InfoObject;
use crate::error::{EncodeResult, ParseResult};
use crate::frame::timestamp::Timestamp;
use crate::protocol::types::SystemType;
use bytes::Bytes;

/// 建筑消防设施软件版本 (4字节信息体 + 6字节时间戳)
///
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施软件版本信息
///
/// ## 字段布局
///
/// | 字段名        | 字节数 | 说明              |
/// |--------------|-------|-------------------|
/// | 系统类型标志   | 1     | 建筑消防设施系统类型 |
/// | 系统地址      | 1     | 建筑消防设施系统地址 |
/// | 主版本号      | 1     | 软件主版本号       |
/// | 次版本号      | 1     | 软件次版本号       |
/// | 版本信息时间   | 6     | 时间戳            |
#[derive(Debug, Clone, PartialEq)]
pub struct FireSystemVersion {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 主版本号 (1字节)
    pub major_version: u8,
    /// 次版本号 (1字节)
    pub minor_version: u8,
    /// 版本信息时间 (6字节)
    pub timestamp: Timestamp,
}

impl FireSystemVersion {
    /// 创建新的建筑消防设施软件版本
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        major_version: u8,
        minor_version: u8,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            major_version,
            minor_version,
            timestamp,
        }
    }

    /// 获取版本字符串表示
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.major_version, self.minor_version)
    }
}

impl InfoObject for FireSystemVersion {
    fn object_type(&self) -> u8 {
        5 // 上传建筑消防设施软件版本
    }

    fn description(&self) -> Option<&str> {
        Some("建筑消防设施软件版本")
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(10); // 4字节信息体 + 6字节时间戳

        // 信息体 (4字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.push(self.major_version);
        buf.push(self.minor_version);

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
        let major_version = data[2];
        let minor_version = data[3];
        let timestamp = Timestamp::from_bytes(&data[4..10])?;

        Ok(FireSystemVersion::new(
            system_type,
            system_address,
            major_version,
            minor_version,
            timestamp,
        ))
    }

    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

/// 用户信息传输装置软件版本 (2字节信息体 + 6字节时间戳)
///
/// 根据GB26875协议8.2.1节定义，用于上传用户信息传输装置软件版本信息
///
/// ## 字段布局
///
/// | 字段名        | 字节数 | 说明              |
/// |--------------|-------|-------------------|
/// | 主版本号      | 1     | 软件主版本号       |
/// | 次版本号      | 1     | 软件次版本号       |
/// | 版本信息时间   | 6     | 时间戳            |
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceVersion {
    /// 主版本号 (1字节)
    pub major_version: u8,
    /// 次版本号 (1字节)
    pub minor_version: u8,
    /// 版本信息时间 (6字节)
    pub timestamp: Timestamp,
}

impl DeviceVersion {
    /// 创建新的用户信息传输装置软件版本
    pub fn new(major_version: u8, minor_version: u8, timestamp: Timestamp) -> Self {
        Self {
            major_version,
            minor_version,
            timestamp,
        }
    }

    /// 获取版本字符串表示
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.major_version, self.minor_version)
    }
}

impl InfoObject for DeviceVersion {
    fn object_type(&self) -> u8 {
        25 // 上传用户信息传输装置软件版本
    }

    fn description(&self) -> Option<&str> {
        Some("用户信息传输装置软件版本")
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(8); // 2字节信息体 + 6字节时间戳

        // 信息体 (2字节)
        buf.push(self.major_version);
        buf.push(self.minor_version);

        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());

        Ok(Bytes::from(buf))
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 8 {
            return Err(crate::error::ParseError::TooShort {
                expected: 8,
                actual: data.len(),
            });
        }

        let major_version = data[0];
        let minor_version = data[1];
        let timestamp = Timestamp::from_bytes(&data[2..8])?;

        Ok(DeviceVersion::new(major_version, minor_version, timestamp))
    }

    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::SystemType;

    #[test]
    fn test_fire_system_version_encode_decode() {
        let timestamp = Timestamp::now();
        let version = FireSystemVersion::new(SystemType::FireAlarm, 1, 2, 15, timestamp);

        // 测试编码
        let encoded = version.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 10);

        // 测试解码
        let decoded = FireSystemVersion::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, version);
    }

    #[test]
    fn test_device_version_encode_decode() {
        let timestamp = Timestamp::now();
        let version = DeviceVersion::new(1, 3, timestamp);

        // 测试编码
        let encoded = version.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 8);

        // 测试解码
        let decoded = DeviceVersion::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, version);
    }

    #[test]
    fn test_version_string() {
        let timestamp = Timestamp::now();

        let fire_version = FireSystemVersion::new(SystemType::FireAlarm, 1, 2, 15, timestamp);
        assert_eq!(fire_version.version_string(), "2.15");

        let device_version = DeviceVersion::new(1, 3, timestamp);
        assert_eq!(device_version.version_string(), "1.3");
    }

    #[test]
    fn test_version_fields() {
        let timestamp = Timestamp::now();

        let fire_version = FireSystemVersion::new(SystemType::FireAlarm, 2, 3, 7, timestamp);

        assert_eq!(fire_version.object_type(), 5);
        assert_eq!(fire_version.system_type, SystemType::FireAlarm);
        assert_eq!(fire_version.system_address, 2);
        assert_eq!(fire_version.major_version, 3);
        assert_eq!(fire_version.minor_version, 7);
        assert_eq!(fire_version.description(), Some("建筑消防设施软件版本"));
        assert!(fire_version.timestamp().is_some());

        let device_version = DeviceVersion::new(4, 2, timestamp);

        assert_eq!(device_version.object_type(), 25);
        assert_eq!(device_version.major_version, 4);
        assert_eq!(device_version.minor_version, 2);
        assert_eq!(
            device_version.description(),
            Some("用户信息传输装置软件版本")
        );
        assert!(device_version.timestamp().is_some());
    }
}
