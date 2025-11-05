//! GB26875 操作信息对象
//!
//! 根据 GB26875 协议第8.2.1节实现的操作信息对象，包括建筑消防设施操作信息和用户信息传输装置操作信息。

use super::InfoObject;
use crate::error::{EncodeResult, ParseResult};
use crate::frame::timestamp::Timestamp;
use crate::protocol::types::SystemType;
use bytes::Bytes;

/// 建筑消防设施操作信息 (4字节信息体 + 6字节时间戳)
///
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施操作信息
///
/// ## 字段布局
///
/// | 字段名        | 字节数 | 说明              |
/// |--------------|-------|-------------------|
/// | 系统类型标志   | 1     | 建筑消防设施系统类型 |
/// | 系统地址      | 1     | 建筑消防设施系统地址 |
/// | 操作标志      | 1     | 操作类型位标志     |
/// | 操作员编号    | 1     | 操作员编号        |
/// | 操作发生时间   | 6     | 时间戳            |
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FireSystemOperation {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 操作标志 (1字节)
    pub operation_flags: u8,
    /// 操作员编号 (1字节)
    pub operator_id: u8,
    /// 操作发生时间 (6字节)
    pub timestamp: Timestamp,
}

impl FireSystemOperation {
    /// 创建新的建筑消防设施操作信息
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        operation_flags: u8,
        operator_id: u8,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            operation_flags,
            operator_id,
            timestamp,
        }
    }

    /// 设置复位操作
    pub fn with_reset(mut self) -> Self {
        self.operation_flags |= 0x01;
        self
    }

    /// 设置消音操作
    pub fn with_silence(mut self) -> Self {
        self.operation_flags |= 0x02;
        self
    }

    /// 设置手动报警操作
    pub fn with_manual_alarm(mut self) -> Self {
        self.operation_flags |= 0x04;
        self
    }

    /// 设置报警消除操作
    pub fn with_alarm_clear(mut self) -> Self {
        self.operation_flags |= 0x08;
        self
    }

    /// 设置自检操作
    pub fn with_self_test(mut self) -> Self {
        self.operation_flags |= 0x10;
        self
    }

    /// 设置确定操作
    pub fn with_confirm(mut self) -> Self {
        self.operation_flags |= 0x20;
        self
    }

    /// 设置测试操作
    pub fn with_test(mut self) -> Self {
        self.operation_flags |= 0x40;
        self
    }
}

impl InfoObject for FireSystemOperation {
    fn object_type(&self) -> u8 {
        4 // 上传建筑消防设施操作信息
    }

    fn description(&self) -> Option<&str> {
        Some("建筑消防设施操作信息")
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(10); // 4字节信息体 + 6字节时间戳

        // 信息体 (4字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.push(self.operation_flags);
        buf.push(self.operator_id);

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
        let operation_flags = data[2];
        let operator_id = data[3];
        let timestamp = Timestamp::from_bytes(&data[4..10])?;

        Ok(FireSystemOperation::new(
            system_type,
            system_address,
            operation_flags,
            operator_id,
            timestamp,
        ))
    }

    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

/// 用户信息传输装置操作信息 (2字节信息体 + 6字节时间戳)
///
/// 根据GB26875协议8.2.1节定义，用于上传用户信息传输装置操作信息
///
/// ## 字段布局
///
/// | 字段名        | 字节数 | 说明              |
/// |--------------|-------|-------------------|
/// | 操作标志      | 1     | 操作类型位标志     |
/// | 操作员编号    | 1     | 操作员编号        |
/// | 操作发生时间   | 6     | 时间戳            |
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceOperation {
    /// 操作标志 (1字节)
    pub operation_flags: u8,
    /// 操作员编号 (1字节)
    pub operator_id: u8,
    /// 操作发生时间 (6字节)
    pub timestamp: Timestamp,
}

impl DeviceOperation {
    /// 创建新的用户信息传输装置操作信息
    pub fn new(operation_flags: u8, operator_id: u8, timestamp: Timestamp) -> Self {
        Self {
            operation_flags,
            operator_id,
            timestamp,
        }
    }

    /// 设置复位操作
    pub fn with_reset(mut self) -> Self {
        self.operation_flags |= 0x01;
        self
    }

    /// 设置消音操作
    pub fn with_silence(mut self) -> Self {
        self.operation_flags |= 0x02;
        self
    }

    /// 设置手动报警操作
    pub fn with_manual_alarm(mut self) -> Self {
        self.operation_flags |= 0x04;
        self
    }

    /// 设置警情消除操作
    pub fn with_alarm_elimination(mut self) -> Self {
        self.operation_flags |= 0x08;
        self
    }

    /// 设置自检操作
    pub fn with_self_test(mut self) -> Self {
        self.operation_flags |= 0x10;
        self
    }

    /// 设置查岗应答操作
    pub fn with_patrol_response(mut self) -> Self {
        self.operation_flags |= 0x20;
        self
    }

    /// 设置测试操作
    pub fn with_test(mut self) -> Self {
        self.operation_flags |= 0x40;
        self
    }
}

impl InfoObject for DeviceOperation {
    fn object_type(&self) -> u8 {
        24 // 上传用户信息传输装置操作信息
    }

    fn description(&self) -> Option<&str> {
        Some("用户信息传输装置操作信息")
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(8); // 2字节信息体 + 6字节时间戳

        // 信息体 (2字节)
        buf.push(self.operation_flags);
        buf.push(self.operator_id);

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

        let operation_flags = data[0];
        let operator_id = data[1];
        let timestamp = Timestamp::from_bytes(&data[2..8])?;

        Ok(DeviceOperation::new(
            operation_flags,
            operator_id,
            timestamp,
        ))
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
    fn test_fire_system_operation_encode_decode() {
        let timestamp = Timestamp::now();
        let operation = FireSystemOperation::new(SystemType::FireAlarm, 1, 0x05, 100, timestamp);

        // 测试编码
        let encoded = operation.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 10);

        // 测试解码
        let decoded = FireSystemOperation::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, operation);
    }

    #[test]
    fn test_device_operation_encode_decode() {
        let timestamp = Timestamp::now();
        let operation = DeviceOperation::new(0x22, 50, timestamp);

        // 测试编码
        let encoded = operation.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 8);

        // 测试解码
        let decoded = DeviceOperation::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, operation);
    }

    #[test]
    fn test_fire_system_operation_builder() {
        let timestamp = Timestamp::now();
        let operation = FireSystemOperation::new(SystemType::FireAlarm, 1, 0x00, 100, timestamp)
            .with_reset()
            .with_silence()
            .with_manual_alarm();

        assert_eq!(operation.operation_flags, 0x07); // 0x01 | 0x02 | 0x04
    }

    #[test]
    fn test_device_operation_builder() {
        let timestamp = Timestamp::now();
        let operation = DeviceOperation::new(0x00, 50, timestamp)
            .with_reset()
            .with_self_test()
            .with_patrol_response();

        assert_eq!(operation.operation_flags, 0x31); // 0x01 | 0x10 | 0x20
    }
}
