//! GB26875 下行数据单元实现 (类型 61-91)
//!
//! 包含从监控中心到用户信息传输装置的数据传输单元，
//! 按照GB26875协议8.3.2节的规定实现。

use crate::error::{ParseError, ParseResult, EncodeError, EncodeResult};
use crate::data_unit::DataUnit;
use crate::protocol::{DataUnitType, SystemType};
use crate::frame::Timestamp;
use bytes::{Bytes, BufMut, BytesMut};

/// 读建筑消防设施系统状态 (类型61)
/// 
/// 根据GB26875协议8.3.2.1，数据格式为：
/// - 系统标志符（1字节）= 61
/// - 信息对象数目（1字节）= n（n不大于102）
/// - 系统类型1（1字节）+ 系统地址1（1字节）
/// - ...
/// - 系统类型n（1字节）+ 系统地址n（1字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadSystemStatus {
    /// 查询的系统信息列表（系统类型+系统地址）
    pub systems: Vec<(SystemType, u8)>, // 系统地址为1字节
}

impl ReadSystemStatus {
    /// 创建新的读系统状态命令
    pub fn new(systems: Vec<(SystemType, u8)>) -> EncodeResult<Self> {
        if systems.is_empty() {
            return Err(EncodeError::InvalidValue {
                field: "systems".to_string(),
                value: "empty".to_string(),
                reason: "At least one system must be specified".to_string(),
            });
        }
        
        if systems.len() > 102 {
            return Err(EncodeError::InvalidValue {
                field: "systems".to_string(),
                value: systems.len().to_string(),
                reason: "Maximum 102 systems allowed per protocol".to_string(),
            });
        }

        Ok(Self { systems })
    }

    /// 获取系统列表
    pub fn systems(&self) -> &[(SystemType, u8)] {
        &self.systems
    }
}

impl DataUnit for ReadSystemStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadSystemStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2 + self.systems.len() * 2);
        
        // 系统标志符（1字节）= 61 (由协议框架添加，这里不编码)
        // 信息对象数目（1字节）
        buf.put_u8(self.systems.len() as u8);
        
        // 系统列表
        for (system_type, system_address) in &self.systems {
            buf.put_u8(system_type.to_u8());
            buf.put_u8(*system_address);
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let object_count = data[0] as usize;
        let expected_len = 1 + object_count * 2;
        
        if data.len() < expected_len {
            return Err(ParseError::InsufficientData {
                expected: expected_len,
                actual: data.len(),
            });
        }

        let mut systems = Vec::with_capacity(object_count);
        let mut offset = 1;
        
        for _ in 0..object_count {
            let system_type = SystemType::from_u8(data[offset]);
            let system_address = data[offset + 1];
            systems.push((system_type, system_address));
            offset += 2;
        }

        Ok(Self { systems })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.systems.is_empty() {
            return Err(ParseError::InvalidValue {
                field: "systems".to_string(),
                value: "empty".to_string(),
                reason: "At least one system must be specified".to_string(),
            });
        }
        
        if self.systems.len() > 102 {
            return Err(ParseError::InvalidValue {
                field: "systems".to_string(),
                value: self.systems.len().to_string(),
                reason: "Maximum 102 systems allowed per protocol".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 读建筑消防设施部件运行状态 (类型62)
/// 
/// 根据GB26875协议8.3.2.2，数据格式为：
/// - 系统标志符（1字节）= 62
/// - 信息对象数目（1字节）= n（n不大于22）
/// - 系统类型1（1字节）+ 系统地址1（1字节）+ 部件地址1（4字节）
/// - ...
/// - 系统类型n（1字节）+ 系统地址n（1字节）+ 部件地址n（4字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadComponentStatus {
    /// 查询的部件信息列表（系统类型+系统地址+部件地址）
    /// 部件地址为4字节，按协议规定包含部件类型和3字节地址
    pub components: Vec<(SystemType, u8, u32)>, // 系统地址1字节，部件地址4字节
}

impl ReadComponentStatus {
    /// 创建新的读部件状态命令
    pub fn new(components: Vec<(SystemType, u8, u32)>) -> EncodeResult<Self> {
        if components.is_empty() {
            return Err(EncodeError::InvalidValue {
                field: "components".to_string(),
                value: "empty".to_string(),
                reason: "At least one component must be specified".to_string(),
            });
        }
        
        if components.len() > 22 {
            return Err(EncodeError::InvalidValue {
                field: "components".to_string(),
                value: components.len().to_string(),
                reason: "Maximum 22 components allowed per protocol".to_string(),
            });
        }

        Ok(Self { components })
    }

    /// 获取部件列表
    pub fn components(&self) -> &[(SystemType, u8, u32)] {
        &self.components
    }
}

impl DataUnit for ReadComponentStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadComponentStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2 + self.components.len() * 6);
        
        // 信息对象数目（1字节）
        buf.put_u8(self.components.len() as u8);
        
        // 部件列表
        for (system_type, system_address, component_address) in &self.components {
            buf.put_u8(system_type.to_u8());
            buf.put_u8(*system_address);
            buf.put_u32_le(*component_address); // 4字节部件地址，小端序
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let object_count = data[0] as usize;
        let expected_len = 1 + object_count * 6; // 1 + n * (1 + 1 + 4)
        
        if data.len() < expected_len {
            return Err(ParseError::InsufficientData {
                expected: expected_len,
                actual: data.len(),
            });
        }

        let mut components = Vec::with_capacity(object_count);
        let mut offset = 1;
        
        for _ in 0..object_count {
            let system_type = SystemType::from_u8(data[offset]);
            let system_address = data[offset + 1];
            let component_address = u32::from_le_bytes([
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
            ]);
            components.push((system_type, system_address, component_address));
            offset += 6;
        }

        Ok(Self { components })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.components.is_empty() {
            return Err(ParseError::InvalidValue {
                field: "components".to_string(),
                value: "empty".to_string(),
                reason: "At least one component must be specified".to_string(),
            });
        }
        
        if self.components.len() > 22 {
            return Err(ParseError::InvalidValue {
                field: "components".to_string(),
                value: self.components.len().to_string(),
                reason: "Maximum 22 components allowed per protocol".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 读建筑消防设施模拟量值 (类型63)
/// 
/// 根据GB26875协议8.3.2.3，数据格式为：
/// - 系统标志符（1字节）= 63
/// - 信息对象数目（1字节）= n（n不大于63）
/// - 系统类型1（1字节）+ 系统地址1（1字节）+ 部件地址1（4字节）
/// - ...
/// - 系统类型n（1字节）+ 系统地址n（1字节）+ 部件地址n（4字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadAnalogValue {
    /// 查询的模拟量信息列表（系统类型+系统地址+部件地址）
    /// 部件地址为4字节，按协议规定包含部件类型和3字节地址
    pub components: Vec<(SystemType, u8, u32)>, // 系统地址1字节，部件地址4字节
}

impl ReadAnalogValue {
    /// 创建新的读模拟量值命令
    pub fn new(components: Vec<(SystemType, u8, u32)>) -> EncodeResult<Self> {
        if components.is_empty() {
            return Err(EncodeError::InvalidValue {
                field: "components".to_string(),
                value: "empty".to_string(),
                reason: "At least one component must be specified".to_string(),
            });
        }
        
        if components.len() > 63 {
            return Err(EncodeError::InvalidValue {
                field: "components".to_string(),
                value: components.len().to_string(),
                reason: "Maximum 63 components allowed per protocol".to_string(),
            });
        }

        Ok(Self { components })
    }

    /// 获取部件列表
    pub fn components(&self) -> &[(SystemType, u8, u32)] {
        &self.components
    }
}

impl DataUnit for ReadAnalogValue {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadAnalogValue
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2 + self.components.len() * 6);
        
        // 信息对象数目（1字节）
        buf.put_u8(self.components.len() as u8);
        
        // 部件列表
        for (system_type, system_address, component_address) in &self.components {
            buf.put_u8(system_type.to_u8());
            buf.put_u8(*system_address);
            buf.put_u32_le(*component_address); // 4字节部件地址，小端序
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let object_count = data[0] as usize;
        let expected_len = 1 + object_count * 6; // 1 + n * (1 + 1 + 4)
        
        if data.len() < expected_len {
            return Err(ParseError::InsufficientData {
                expected: expected_len,
                actual: data.len(),
            });
        }

        let mut components = Vec::with_capacity(object_count);
        let mut offset = 1;
        
        for _ in 0..object_count {
            let system_type = SystemType::from_u8(data[offset]);
            let system_address = data[offset + 1];
            let component_address = u32::from_le_bytes([
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
            ]);
            components.push((system_type, system_address, component_address));
            offset += 6;
        }

        Ok(Self { components })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.components.is_empty() {
            return Err(ParseError::InvalidValue {
                field: "components".to_string(),
                value: "empty".to_string(),
                reason: "At least one component must be specified".to_string(),
            });
        }
        
        if self.components.len() > 63 {
            return Err(ParseError::InvalidValue {
                field: "components".to_string(),
                value: self.components.len().to_string(),
                reason: "Maximum 63 components allowed per protocol".to_string(),
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
    }    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(8);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        buf.extend_from_slice(&self.target_time.to_bytes());
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for SyncDeviceClock".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }

        let target_time = Timestamp::from_bytes(&data[2..8])?;
        Ok(Self { target_time })
    }fn validate(&self) -> ParseResult<()> {
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
pub struct PatrolCommand;

impl PatrolCommand {
    /// 创建新的查岗命令
    pub fn new() -> Self {
        Self
    }
}

impl Default for PatrolCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DataUnit for PatrolCommand {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::PatrolCommand
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for PatrolCommand".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }

        Ok(Self::new())
    }

    fn validate(&self) -> ParseResult<()> {
        // 无需特殊验证
        Ok(())
    }
}

/// 初始化用户信息传输装置 (类型89)
/// 
/// 用于初始化设备
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeDevice;

impl InitializeDevice {
    /// 创建新的初始化设备命令
    pub fn new() -> Self {
        Self
    }
}

impl Default for InitializeDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl DataUnit for InitializeDevice {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::InitializeDevice
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for InitializeDevice".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }

        Ok(Self::new())
    }

    fn validate(&self) -> ParseResult<()> {
        // 无需特殊验证
        Ok(())
    }
}

/// 读建筑消防设施操作信息记录 (类型64)
/// 
/// 根据GB26875协议8.3.2.4，数据格式为：
/// - 系统标志符（1字节）= 64
/// - 信息对象数目（1字节）= 1
/// - 系统类型1（1字节）+ 系统地址1（1字节）+ 查询操作信息记录数目（1字节）+ 查询记录的指定起始时间（6字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadOperationInfo {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（1字节）
    pub system_address: u8,
    /// 查询操作信息记录数目（不大于102）
    pub record_count: u8,
    /// 查询记录的指定起始时间
    pub start_time: Timestamp,
}

impl ReadOperationInfo {
    /// 创建新的读操作信息命令
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// * `system_address` - 系统地址（1字节）
    /// * `record_count` - 记录数目（≤ 102）
    /// * `start_time` - 起始时间
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的实例
    /// * `Err(EncodeError)` - 参数超出范围时的错误
    pub fn new(
        system_type: SystemType, 
        system_address: u8, 
        record_count: u8,
        start_time: Timestamp
    ) -> EncodeResult<Self> {
        if record_count > 102 {
            return Err(EncodeError::InvalidValue {
                field: "record_count".to_string(),
                value: record_count.to_string(),
                reason: "Record count must be <= 102".to_string(),
            });
        }

        Ok(Self { 
            system_type, 
            system_address,
            record_count,
            start_time,
        })
    }
}

impl DataUnit for ReadOperationInfo {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadOperationInfo
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(9); // 1+1+1+6字节
        
        // 信息对象数目（1字节）= 1
        buf.put_u8(1);
        
        // 系统类型 (1字节)
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址 (1字节)
        buf.put_u8(self.system_address);
        
        // 查询操作信息记录数目 (1字节)
        buf.put_u8(self.record_count);
        
        // 查询记录的指定起始时间 (6字节)
        let time_bytes = self.start_time.encode()?;
        buf.extend_from_slice(&time_bytes);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 9 {
            return Err(ParseError::InsufficientData {
                expected: 9,
                actual: data.len(),
            });
        }

        // 信息对象数目（1字节）应该为1
        let object_count = data[0];
        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Object count must be 1 for ReadOperationInfo".to_string(),
            });
        }

        // 系统类型 (1字节)
        let system_type = SystemType::from_u8(data[1]);
        
        // 系统地址 (1字节)
        let system_address = data[2];
        
        // 查询操作信息记录数目 (1字节)
        let record_count = data[3];
        
        // 查询记录的指定起始时间 (6字节)
        let start_time = Timestamp::parse(&data[4..10])?;

        Ok(Self { 
            system_type, 
            system_address,
            record_count,
            start_time,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.record_count > 102 {
            return Err(ParseError::InvalidValue {
                field: "record_count".to_string(),
                value: self.record_count.to_string(),
                reason: "Record count must be <= 102".to_string(),
            });
        }
        
        if !self.start_time.is_valid() {
            return Err(ParseError::InvalidValue {
                field: "start_time".to_string(),
                value: format!("{:?}", self.start_time),
                reason: "Invalid timestamp fields".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 读建筑消防设施软件版本 (类型65)
/// 
/// 根据GB26875协议8.3.2.5，数据格式为：
/// - 系统标志符（1字节）= 65
/// - 信息对象数目（1字节）= 1
/// - 系统类型1（1字节）+ 系统地址1（1字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadSoftwareVersion {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（1字节）
    pub system_address: u8,
}

impl ReadSoftwareVersion {
    /// 创建新的读软件版本命令
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// * `system_address` - 系统地址（1字节）
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的实例
    pub fn new(system_type: SystemType, system_address: u8) -> Self {
        Self { system_type, system_address }
    }
}

impl DataUnit for ReadSoftwareVersion {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadSoftwareVersion
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(3);
        
        // 信息对象数目（1字节）= 1
        buf.put_u8(1);
        
        // 系统类型 (1字节)
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址 (1字节)
        buf.put_u8(self.system_address);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 3 {
            return Err(ParseError::InsufficientData {
                expected: 3,
                actual: data.len(),
            });
        }

        // 信息对象数目（1字节）应该为1
        let object_count = data[0];
        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Object count must be 1 for ReadSoftwareVersion".to_string(),
            });
        }

        let system_type = SystemType::from_u8(data[1]);
        let system_address = data[2];

        Ok(Self { system_type, system_address })
    }

    fn validate(&self) -> ParseResult<()> {
        // 1字节系统地址无特殊验证要求
        Ok(())
    }
}

/// 读建筑消防设施系统配置情况 (类型66)
/// 
/// 根据GB26875协议8.3.2.6，数据格式为：
/// - 系统标志符（1字节）= 66
/// - 信息对象数目（1字节）= n（n不大于3）
/// - 系统类型1（1字节）+ 系统地址1（1字节）
/// - ...
/// - 系统类型n（1字节）+ 系统地址n（1字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadSystemConfig {
    /// 查询的系统信息列表（系统类型+系统地址）
    pub systems: Vec<(SystemType, u8)>, // 系统地址为1字节
}

impl ReadSystemConfig {
    /// 创建新的读系统配置命令
    /// 
    /// # Arguments
    /// * `systems` - 系统列表（不大于3个）
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的实例
    /// * `Err(EncodeError)` - 参数超出范围时的错误
    pub fn new(systems: Vec<(SystemType, u8)>) -> EncodeResult<Self> {
        if systems.is_empty() {
            return Err(EncodeError::InvalidValue {
                field: "systems".to_string(),
                value: "empty".to_string(),
                reason: "At least one system must be specified".to_string(),
            });
        }
        
        if systems.len() > 3 {
            return Err(EncodeError::InvalidValue {
                field: "systems".to_string(),
                value: systems.len().to_string(),
                reason: "Maximum 3 systems allowed per protocol".to_string(),
            });
        }

        Ok(Self { systems })
    }

    /// 获取系统列表
    pub fn systems(&self) -> &[(SystemType, u8)] {
        &self.systems
    }
}

impl DataUnit for ReadSystemConfig {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadSystemConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(1 + self.systems.len() * 2);
        
        // 信息对象数目（1字节）
        buf.put_u8(self.systems.len() as u8);
        
        // 系统列表
        for (system_type, system_address) in &self.systems {
            buf.put_u8(system_type.to_u8());
            buf.put_u8(*system_address);
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let object_count = data[0] as usize;
        let expected_len = 1 + object_count * 2;
        
        if data.len() < expected_len {
            return Err(ParseError::InsufficientData {
                expected: expected_len,
                actual: data.len(),
            });
        }

        let mut systems = Vec::with_capacity(object_count);
        let mut offset = 1;
        
        for _ in 0..object_count {
            let system_type = SystemType::from_u8(data[offset]);
            let system_address = data[offset + 1];
            systems.push((system_type, system_address));
            offset += 2;
        }

        Ok(Self { systems })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.systems.is_empty() {
            return Err(ParseError::InvalidValue {
                field: "systems".to_string(),
                value: "empty".to_string(),
                reason: "At least one system must be specified".to_string(),
            });
        }
        
        if self.systems.len() > 3 {
            return Err(ParseError::InvalidValue {
                field: "systems".to_string(),
                value: self.systems.len().to_string(),
                reason: "Maximum 3 systems allowed per protocol".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 读建筑消防设施部件配置情况 (类型67)
/// 
/// 根据GB26875协议8.3.2.7，数据格式为：
/// - 系统标志符（1字节）= 67
/// - 信息对象数目（1字节）= n（n不大于26）
/// - 系统类型1（1字节）+ 系统地址1（1字节）+ 部件地址1（4字节）
/// - ...
/// - 系统类型n（1字节）+ 系统地址n（1字节）+ 部件地址n（4字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadComponentConfig {
    /// 查询的部件信息列表（系统类型+系统地址+部件地址）
    /// 部件地址为4字节，按协议规定包含部件类型和3字节地址
    pub components: Vec<(SystemType, u8, u32)>, // 系统地址1字节，部件地址4字节
}

impl ReadComponentConfig {
    /// 创建新的读部件配置命令
    /// 
    /// # Arguments
    /// * `components` - 部件列表（不大于26个）
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的实例
    /// * `Err(EncodeError)` - 参数超出范围时的错误
    pub fn new(components: Vec<(SystemType, u8, u32)>) -> EncodeResult<Self> {
        if components.is_empty() {
            return Err(EncodeError::InvalidValue {
                field: "components".to_string(),
                value: "empty".to_string(),
                reason: "At least one component must be specified".to_string(),
            });
        }
        
        if components.len() > 26 {
            return Err(EncodeError::InvalidValue {
                field: "components".to_string(),
                value: components.len().to_string(),
                reason: "Maximum 26 components allowed per protocol".to_string(),
            });
        }

        Ok(Self { components })
    }

    /// 获取部件列表
    pub fn components(&self) -> &[(SystemType, u8, u32)] {
        &self.components
    }
}

impl DataUnit for ReadComponentConfig {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadComponentConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(1 + self.components.len() * 6);
        
        // 信息对象数目（1字节）
        buf.put_u8(self.components.len() as u8);
        
        // 部件列表
        for (system_type, system_address, component_address) in &self.components {
            buf.put_u8(system_type.to_u8());
            buf.put_u8(*system_address);
            buf.put_u32_le(*component_address); // 4字节部件地址，小端序
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let object_count = data[0] as usize;
        let expected_len = 1 + object_count * 6; // 1 + n * (1 + 1 + 4)
        
        if data.len() < expected_len {
            return Err(ParseError::InsufficientData {
                expected: expected_len,
                actual: data.len(),
            });
        }

        let mut components = Vec::with_capacity(object_count);
        let mut offset = 1;
        
        for _ in 0..object_count {
            let system_type = SystemType::from_u8(data[offset]);
            let system_address = data[offset + 1];
            let component_address = u32::from_le_bytes([
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
            ]);
            components.push((system_type, system_address, component_address));
            offset += 6;
        }

        Ok(Self { components })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.components.is_empty() {
            return Err(ParseError::InvalidValue {
                field: "components".to_string(),
                value: "empty".to_string(),
                reason: "At least one component must be specified".to_string(),
            });
        }
        
        if self.components.len() > 26 {
            return Err(ParseError::InvalidValue {
                field: "components".to_string(),
                value: self.components.len().to_string(),
                reason: "Maximum 26 components allowed per protocol".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 读建筑消防设施系统时间 (类型68)
/// 
/// 根据GB26875协议8.3.2.8，数据格式为：
/// - 系统标志符（1字节）= 68
/// - 信息对象数目（1字节）= 1
/// - 系统类型1（1字节）+ 系统地址1（1字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadSystemTime {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（1字节）
    pub system_address: u8,
}

impl ReadSystemTime {
    /// 创建新的读系统时间命令
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// * `system_address` - 系统地址（1字节）
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的实例
    pub fn new(system_type: SystemType, system_address: u8) -> Self {
        Self { system_type, system_address }
    }
}

impl DataUnit for ReadSystemTime {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadSystemTime
    }    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(self.system_type.to_u8());
        buf.put_u8(self.system_address);
        Ok(buf.freeze())
    }    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];

        Ok(Self { system_type, system_address })
    }    fn validate(&self) -> ParseResult<()> {
        // 系统地址为1字节，无需额外验证
        Ok(())
    }
}

/// 读用户信息传输装置运行状态 (类型81)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadDeviceStatus;

impl ReadDeviceStatus {
    /// 创建新的读设备状态命令
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadDeviceStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl DataUnit for ReadDeviceStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadDeviceStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for ReadDeviceStatus".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }

        Ok(Self::new())
    }
}

/// 读用户信息传输装置操作信息记录 (类型84)
/// 
/// 监控中心请求用户信息传输装置传送操作信息记录，并指定记录起始时间和信息数目
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadDeviceOperation {
    /// 查询操作信息记录数目（≤102）
    pub record_count: u8,
    /// 查询记录的指定起始时间
    pub start_time: Timestamp,
}

impl ReadDeviceOperation {
    /// 创建新的读设备操作信息命令
    /// 
    /// # Arguments
    /// * `record_count` - 查询操作信息记录数目（不大于102）
    /// * `start_time` - 查询记录的指定起始时间
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的实例
    /// * `Err(String)` - 参数验证失败
    pub fn new(record_count: u8, start_time: Timestamp) -> Result<Self, String> {
        if record_count == 0 {
            return Err("Record count must be greater than 0".to_string());
        }
        if record_count > 102 {
            return Err("Record count must not exceed 102".to_string());
        }
        Ok(Self { record_count, start_time })
    }
}

impl DataUnit for ReadDeviceOperation {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadDeviceOperation
    }    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(8);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(self.record_count);
        buf.extend_from_slice(&self.start_time.to_bytes());
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for ReadDeviceOperation".to_string(),
            });
        }        let record_count = data[1];
        let start_time = Timestamp::from_bytes(&data[2..8])?;

        Self::new(record_count, start_time)
            .map_err(|e| ParseError::InvalidValue {
                field: "record_count".to_string(),
                value: record_count.to_string(),
                reason: e,
            })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.record_count == 0 {
            return Err(ParseError::InvalidValue {
                field: "record_count".to_string(),
                value: self.record_count.to_string(),
                reason: "Record count must be greater than 0".to_string(),
            });
        }
        
        if self.record_count > 102 {
            return Err(ParseError::InvalidValue {
                field: "record_count".to_string(),
                value: self.record_count.to_string(),
                reason: "Record count must not exceed 102".to_string(),
            });
        }
        
        Ok(())
    }
}

/// 读用户信息传输装置软件版本 (类型85)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadDeviceVersion;

impl ReadDeviceVersion {
    /// 创建新的读设备版本命令
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadDeviceVersion {
    fn default() -> Self {
        Self::new()
    }
}

impl DataUnit for ReadDeviceVersion {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadDeviceVersion
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for ReadDeviceVersion".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }        Ok(Self::new())
    }
}

/// 读用户信息传输装置配置情况 (类型86)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadDeviceConfig;

impl ReadDeviceConfig {
    /// 创建新的读设备配置命令
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadDeviceConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DataUnit for ReadDeviceConfig {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadDeviceConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for ReadDeviceConfig".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }

        Ok(Self::new())
    }
}

/// 读用户信息传输装置系统时间 (类型88)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadDeviceTime;

impl ReadDeviceTime {
    /// 创建新的读设备时间命令
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadDeviceTime {
    fn default() -> Self {
        Self::new()
    }
}

impl DataUnit for ReadDeviceTime {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::ReadDeviceTime
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(1); // 信息对象数目 = 1
        buf.put_u8(0); // 预留 = 0
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        // 验证信息对象数目应为1
        if data[0] != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: data[0].to_string(),
                reason: "Object count must be 1 for ReadDeviceTime".to_string(),
            });
        }

        // 验证预留字段应为0
        if data[1] != 0 {
            return Err(ParseError::InvalidValue {
                field: "reserved".to_string(),
                value: data[1].to_string(),
                reason: "Reserved field must be 0".to_string(),
            });
        }

        Ok(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::SystemType;    #[test]
    fn test_read_system_status_encode_decode() {
        let read_cmd = ReadSystemStatus::new(vec![
            (SystemType::FireAlarm, 0x12),
        ]).unwrap();
        
        // 编码
        let encoded = read_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 3); // 1字节数目 + 1组(2字节)
        
        // 解码
        let decoded = ReadSystemStatus::parse(&encoded).unwrap();
        
        assert_eq!(read_cmd, decoded);
        assert_eq!(decoded.systems.len(), 1);
        assert_eq!(decoded.systems[0].0, SystemType::FireAlarm);
        assert_eq!(decoded.systems[0].1, 0x12);
    }

    #[test]
    fn test_read_component_status_encode_decode() {
        let read_cmd = ReadComponentStatus::new(vec![
            (SystemType::FireAlarm, 0x12, 0x789ABC),
        ]).unwrap();
        
        // 编码
        let encoded = read_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 7); // 1字节数目 + 1组(6字节)
        
        // 解码
        let decoded = ReadComponentStatus::parse(&encoded).unwrap();
        
        assert_eq!(read_cmd, decoded);
        assert_eq!(decoded.components.len(), 1);
        assert_eq!(decoded.components[0].0, SystemType::FireAlarm);
        assert_eq!(decoded.components[0].1, 0x12);
        assert_eq!(decoded.components[0].2, 0x789ABC);
    }    #[test]
    fn test_sync_device_clock_encode_decode() {
        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap(); // 45秒,30分,15时,4日,11月,24年(2024)
        let sync_cmd = SyncDeviceClock::new(timestamp);
        
        // 编码
        let encoded = sync_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 8); // 信息对象数目(1) + 预留(0) + 时间戳(6)
        assert_eq!(encoded[0], 1); // 信息对象数目
        assert_eq!(encoded[1], 0); // 预留
        
        // 解码
        let decoded = SyncDeviceClock::parse(&encoded).unwrap();
        
        assert_eq!(sync_cmd, decoded);
        assert_eq!(decoded.target_time, timestamp);
    }#[test]
    fn test_patrol_command_encode_decode() {
        let patrol_cmd = PatrolCommand::new();
        
        // 编码
        let encoded = patrol_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 2); // 信息对象数目(1) + 预留(0)
        assert_eq!(encoded[0], 1); // 信息对象数目
        assert_eq!(encoded[1], 0); // 预留
        
        // 解码
        let decoded = PatrolCommand::parse(&encoded).unwrap();
        
        assert_eq!(patrol_cmd, decoded);
    }#[test]
    fn test_initialize_device_encode_decode() {
        let init_cmd = InitializeDevice::new();
        
        // 编码
        let encoded = init_cmd.encode().unwrap();
        assert_eq!(encoded.len(), 2); // 信息对象数目(1) + 预留(0)
        assert_eq!(encoded[0], 1); // 信息对象数目
        assert_eq!(encoded[1], 0); // 预留
        
        // 解码
        let decoded = InitializeDevice::parse(&encoded).unwrap();
        
        assert_eq!(init_cmd, decoded);
    }#[test]
    fn test_address_validation() {
        // 测试系统数量超出范围的情况
        let mut systems = Vec::new();
        for i in 0..103 { // 超过最大102个
            systems.push((SystemType::FireAlarm, i as u8));
        }
        assert!(ReadSystemStatus::new(systems).is_err());
        
        // 测试部件数量超出范围的情况
        let mut components = Vec::new();
        for i in 0..23 { // 超过最大22个
            components.push((SystemType::FireAlarm, i as u8, 0x123456));
        }
        assert!(ReadComponentStatus::new(components).is_err());
    }
}
