//! GB26875 下行数据单元实现 (类型 61-91)
//!
//! 包含从监控中心到用户信息传输装置的数据传输单元，
//! 按照GB26875协议8.3.2节的规定实现。

use crate::error::{ParseError, ParseResult, EncodeError, EncodeResult};
use crate::data_unit::DataUnit;
use crate::protocol::{DataUnitType, SystemType, ComponentType as ProtocolComponentType};
use crate::frame::Timestamp;
use bytes::{Bytes, BufMut, BytesMut};

/// 读建筑消防设施系统状态 (类型61)
/// 
/// 用于请求读取系统状态信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadSystemStatus {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（3字节，小端序）
    pub system_address: u32, // 实际只使用低3字节
}

impl ReadSystemStatus {
    /// 创建新的读系统状态命令
    pub fn new(system_type: SystemType, system_address: u32) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
                reason: "System address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        Ok(Self {
            system_type,
            system_address,
        })
    }

    /// 获取系统类型
    pub fn system_type(&self) -> SystemType {
        self.system_type
    }

    /// 获取系统地址
    pub fn system_address(&self) -> u32 {
        self.system_address
    }
}

impl DataUnit for ReadSystemStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadSystemStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(4);
        
        // 系统类型 (1字节)
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址 (3字节, 小端序)
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 4 {
            return Err(ParseError::InsufficientData {
                expected: 4,
                actual: data.len(),
            });
        }

        // 系统类型 (1字节)
        let system_type = SystemType::from_u8(data[0]);

        // 系统地址 (3字节, 小端序)
        let system_address = 
            (data[1] as u32) |
            ((data[2] as u32) << 8) |
            ((data[3] as u32) << 16);

        Ok(Self {
            system_type,
            system_address,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
                reason: "System address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }
        Ok(())
    }
}

/// 读建筑消防设施部件运行状态 (类型62)
/// 
/// 用于请求读取部件状态信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadComponentStatus {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（3字节，小端序）
    pub system_address: u32,
    /// 部件类型
    pub component_type: ProtocolComponentType,
    /// 部件地址（3字节，小端序）
    pub component_address: u32,
}

impl ReadComponentStatus {
    /// 创建新的读部件状态命令
    pub fn new(
        system_type: SystemType,
        system_address: u32,
        component_type: ProtocolComponentType,
        component_address: u32,
    ) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
                reason: "System address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        if component_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "component_address".to_string(),
                value: component_address.to_string(),
                reason: "Component address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        Ok(Self {
            system_type,
            system_address,
            component_type,
            component_address,
        })
    }

    /// 获取系统类型
    pub fn system_type(&self) -> SystemType {
        self.system_type
    }

    /// 获取系统地址
    pub fn system_address(&self) -> u32 {
        self.system_address
    }

    /// 获取部件类型
    pub fn component_type(&self) -> ProtocolComponentType {
        self.component_type
    }

    /// 获取部件地址
    pub fn component_address(&self) -> u32 {
        self.component_address
    }
}

impl DataUnit for ReadComponentStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadComponentStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(8);
        
        // 系统类型 (1字节)
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址 (3字节, 小端序)
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        // 部件类型 (1字节)
        buf.put_u8(self.component_type.to_u8());
        
        // 部件地址 (3字节, 小端序)
        buf.put_u8((self.component_address & 0xFF) as u8);
        buf.put_u8(((self.component_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.component_address >> 16) & 0xFF) as u8);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 系统类型 (1字节)
        let system_type = SystemType::from_u8(data[offset]);
        offset += 1;

        // 系统地址 (3字节, 小端序)
        let system_address = 
            (data[offset] as u32) |
            ((data[offset + 1] as u32) << 8) |
            ((data[offset + 2] as u32) << 16);
        offset += 3;

        // 部件类型 (1字节)
        let component_type = ProtocolComponentType::from_u8(data[offset]);
        offset += 1;

        // 部件地址 (3字节, 小端序)
        let component_address = 
            (data[offset] as u32) |
            ((data[offset + 1] as u32) << 8) |
            ((data[offset + 2] as u32) << 16);

        Ok(Self {
            system_type,
            system_address,
            component_type,
            component_address,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
                reason: "System address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        if self.component_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "component_address".to_string(),
                value: self.component_address.to_string(),
                reason: "Component address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        Ok(())
    }
}

/// 读建筑消防设施模拟量值 (类型63)
/// 
/// 用于请求读取模拟量值信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadAnalogValue {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（3字节，小端序）
    pub system_address: u32,
    /// 部件类型
    pub component_type: ProtocolComponentType,
    /// 部件地址（3字节，小端序）
    pub component_address: u32,
}

impl ReadAnalogValue {
    /// 创建新的读模拟量值命令
    pub fn new(
        system_type: SystemType,
        system_address: u32,
        component_type: ProtocolComponentType,
        component_address: u32,
    ) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
                reason: "System address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        if component_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "component_address".to_string(),
                value: component_address.to_string(),
                reason: "Component address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        Ok(Self {
            system_type,
            system_address,
            component_type,
            component_address,
        })
    }
}

impl DataUnit for ReadAnalogValue {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadAnalogValue
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(8);
        
        // 系统类型 (1字节)
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址 (3字节, 小端序)
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        // 部件类型 (1字节)
        buf.put_u8(self.component_type.to_u8());
        
        // 部件地址 (3字节, 小端序)
        buf.put_u8((self.component_address & 0xFF) as u8);
        buf.put_u8(((self.component_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.component_address >> 16) & 0xFF) as u8);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 系统类型 (1字节)
        let system_type = SystemType::from_u8(data[offset]);
        offset += 1;

        // 系统地址 (3字节, 小端序)
        let system_address = 
            (data[offset] as u32) |
            ((data[offset + 1] as u32) << 8) |
            ((data[offset + 2] as u32) << 16);
        offset += 3;

        // 部件类型 (1字节)
        let component_type = ProtocolComponentType::from_u8(data[offset]);
        offset += 1;

        // 部件地址 (3字节, 小端序)
        let component_address = 
            (data[offset] as u32) |
            ((data[offset + 1] as u32) << 8) |
            ((data[offset + 2] as u32) << 16);

        Ok(Self {
            system_type,
            system_address,
            component_type,
            component_address,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
                reason: "System address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        if self.component_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "component_address".to_string(),
                value: self.component_address.to_string(),
                reason: "Component address must be <= 0xFFFFFF (3 bytes)".to_string(),
            });
        }

        Ok(())
    }
}

/// 同步用户信息传输装置时钟 (类型90)
/// 
/// 用于同步设备时钟
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncDeviceClock {
    /// 目标时间
    pub target_time: Timestamp,
}

impl SyncDeviceClock {
    /// 创建新的同步时钟命令
    pub fn new(target_time: Timestamp) -> Self {
        Self { target_time }
    }

    /// 获取目标时间
    pub fn target_time(&self) -> &Timestamp {
        &self.target_time
    }
}

impl DataUnit for SyncDeviceClock {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::SyncDeviceClock
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        // 直接编码时间标签
        self.target_time.encode()
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 6 {
            return Err(ParseError::InsufficientData {
                expected: 6,
                actual: data.len(),
            });
        }

        let target_time = Timestamp::parse(data)?;
        Ok(Self { target_time })
    }    fn validate(&self) -> ParseResult<()> {
        // 验证时间戳有效性
        if !self.target_time.is_valid() {
            return Err(ParseError::InvalidValue {
                field: "target_time".to_string(),
                value: format!("{:?}", self.target_time),
                reason: "Invalid timestamp fields".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 查岗命令 (类型91)
/// 
/// 用于查询设备状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatrolCommand {
    /// 查岗序号
    pub sequence: u8,
}

impl PatrolCommand {
    /// 创建新的查岗命令
    pub fn new(sequence: u8) -> Self {
        Self { sequence }
    }

    /// 获取查岗序号
    pub fn sequence(&self) -> u8 {
        self.sequence
    }
}

impl DataUnit for PatrolCommand {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::PatrolCommand
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(1);
        buf.put_u8(self.sequence);
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        Ok(Self {
            sequence: data[0],
        })
    }

    fn validate(&self) -> ParseResult<()> {
        // 查岗序号无特殊限制
        Ok(())
    }
}

/// 初始化用户信息传输装置 (类型89)
/// 
/// 用于初始化设备
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeDevice {
    /// 初始化类型
    pub init_type: u8,
}

impl InitializeDevice {
    /// 创建新的初始化设备命令
    pub fn new(init_type: u8) -> Self {
        Self { init_type }
    }

    /// 获取初始化类型
    pub fn init_type(&self) -> u8 {
        self.init_type
    }
}

impl DataUnit for InitializeDevice {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::InitializeDevice
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(1);
        buf.put_u8(self.init_type);
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        Ok(Self {
            init_type: data[0],
        })
    }

    fn validate(&self) -> ParseResult<()> {
        // 初始化类型无特殊限制
        Ok(())
    }
}

// TODO: 继续实现其余的下行数据单元类型
// - ReadOperationInfo (类型64) - 读建筑消防设施操作信息
// - ReadSoftwareVersion (类型65) - 读建筑消防设施软件版本
// - ReadSystemConfig (类型66) - 读建筑消防设施系统配置情况
// - ReadComponentConfig (类型67) - 读建筑消防设施部件配置情况
// - ReadSystemTime (类型68) - 读建筑消防设施系统时间
// - ReadDeviceStatus (类型81) - 读用户信息传输装置运行状态
// - ReadDeviceOperation (类型84) - 读用户信息传输装置操作信息记录
// - ReadDeviceVersion (类型85) - 读用户信息传输装置软件版本
// - ReadDeviceConfig (类型86) - 读用户信息传输装置配置情况
// - ReadDeviceTime (类型88) - 读用户信息传输装置系统时间

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{SystemType, ComponentType};

    #[test]
    fn test_read_system_status_encode_decode() {
        let read_cmd = ReadSystemStatus::new(
            SystemType::FireAlarm,
            0x123456,
        ).unwrap();
        
        // 编码
        let encoded = read_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 4);
        
        // 解码
        let decoded = ReadSystemStatus::parse(&encoded).unwrap();
        
        assert_eq!(read_cmd, decoded);
        assert_eq!(decoded.system_type, SystemType::FireAlarm);
        assert_eq!(decoded.system_address, 0x123456);
    }

    #[test]
    fn test_read_component_status_encode_decode() {
        let read_cmd = ReadComponentStatus::new(
            SystemType::FireAlarm,
            0x123456,
            ComponentType::FireAlarmController,
            0x789ABC,
        ).unwrap();
        
        // 编码
        let encoded = read_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 8);
        
        // 解码
        let decoded = ReadComponentStatus::parse(&encoded).unwrap();
        
        assert_eq!(read_cmd, decoded);
        assert_eq!(decoded.system_type, SystemType::FireAlarm);
        assert_eq!(decoded.system_address, 0x123456);
        assert_eq!(decoded.component_type, ComponentType::FireAlarmController);
        assert_eq!(decoded.component_address, 0x789ABC);
    }

    #[test]
    fn test_sync_device_clock_encode_decode() {
        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap(); // 45秒,30分,15时,4日,11月,24年(2024)
        let sync_cmd = SyncDeviceClock::new(timestamp);
        
        // 编码
        let encoded = sync_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 6);
        
        // 解码
        let decoded = SyncDeviceClock::parse(&encoded).unwrap();
        
        assert_eq!(sync_cmd, decoded);
        assert_eq!(decoded.target_time, timestamp);
    }

    #[test]
    fn test_patrol_command_encode_decode() {
        let patrol_cmd = PatrolCommand::new(123);
        
        // 编码
        let encoded = patrol_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 123);
        
        // 解码
        let decoded = PatrolCommand::parse(&encoded).unwrap();
        
        assert_eq!(patrol_cmd, decoded);
        assert_eq!(decoded.sequence, 123);
    }

    #[test]
    fn test_initialize_device_encode_decode() {
        let init_cmd = InitializeDevice::new(0xFF);
        
        // 编码
        let encoded = init_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 0xFF);
        
        // 解码
        let decoded = InitializeDevice::parse(&encoded).unwrap();
        
        assert_eq!(init_cmd, decoded);
        assert_eq!(decoded.init_type, 0xFF);
    }

    #[test]
    fn test_address_validation() {
        // 测试地址超出范围的情况
        assert!(ReadSystemStatus::new(SystemType::FireAlarm, 0x1000000).is_err());
        
        assert!(ReadComponentStatus::new(
            SystemType::FireAlarm,
            0x1000000, // 超出范围
            ComponentType::FireAlarmController,
            0x123456,
        ).is_err());

        assert!(ReadComponentStatus::new(
            SystemType::FireAlarm,
            0x123456,
            ComponentType::FireAlarmController,
            0x1000000, // 超出范围
        ).is_err());
    }
}
