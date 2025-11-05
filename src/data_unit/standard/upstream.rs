//! GB26875 上行数据单元实现 (类型 1-28)
//!
//! 包含从用户信息传输装置到监控中心的数据传输单元，
//! 按照GB26875协议8.3.1节的规定实现。

use crate::data_unit::DataUnit;
use crate::error::{EncodeResult, ParseError, ParseResult};
use crate::frame::Timestamp;
use crate::info_object::{InfoObject, SystemStatus as InfoSystemStatus};
use crate::protocol::DataUnitType;
use bytes::{BufMut, Bytes, BytesMut};

/// 上传建筑消防设施系统状态 (类型1)
///
/// 包含系统状态信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadSystemStatus {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 系统状态信息对象
    pub system_status: InfoSystemStatus,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadSystemStatus {
    /// 创建新的上传系统状态数据单元
    pub fn new(system_status: InfoSystemStatus, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            system_status,
            timestamp,
        }
    }

    /// 获取系统状态信息对象的引用
    pub fn system_status(&self) -> &InfoSystemStatus {
        &self.system_status
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadSystemStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 系统状态信息对象
        let info_bytes = self.system_status.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 10 + 6 {
            // 至少需要1+10+6=17字节 (系统状态是10字节)
            return Err(ParseError::InsufficientData {
                expected: 17,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "System status should have exactly 1 object".to_string(),
            });
        }

        // 系统状态信息对象 (10字节)
        let system_status = InfoSystemStatus::parse(&data[offset..offset + 10])?;
        offset += 10;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            system_status,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "System status should have exactly 1 object".to_string(),
            });
        }

        // Note: InfoObject trait doesn't have validate method in current implementation
        // We rely on the individual field validation during construction
        Ok(())
    }
}

/// 上传建筑消防设施部件运行状态 (类型2)
///
/// 包含部件状态信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadComponentStatus {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 部件状态信息对象
    pub component_status: crate::info_object::ComponentStatus,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadComponentStatus {
    /// 创建新的上传部件状态数据单元
    pub fn new(
        component_status: crate::info_object::ComponentStatus,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            object_count: 1,
            component_status,
            timestamp,
        }
    }

    /// 获取部件状态信息对象的引用
    pub fn component_status(&self) -> &crate::info_object::ComponentStatus {
        &self.component_status
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadComponentStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadComponentStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 部件状态信息对象
        let info_bytes = self.component_status.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 40 + 6 {
            // 至少需要1+40+6=47字节 (部件状态是40字节)
            return Err(ParseError::InsufficientData {
                expected: 47,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Component status should have exactly 1 object".to_string(),
            });
        }

        // 部件状态信息对象 (40字节)
        let component_status =
            crate::info_object::ComponentStatus::parse(&data[offset..offset + 40])?;
        offset += 40;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            component_status,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Component status should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传建筑消防设施模拟量值 (类型3)
///
/// 包含模拟量值信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadAnalogValue {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 模拟量值信息对象
    pub analog_value: crate::info_object::AnalogValue,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadAnalogValue {
    /// 创建新的上传模拟量值数据单元
    pub fn new(analog_value: crate::info_object::AnalogValue, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            analog_value,
            timestamp,
        }
    }

    /// 获取模拟量值信息对象的引用
    pub fn analog_value(&self) -> &crate::info_object::AnalogValue {
        &self.analog_value
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadAnalogValue {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadAnalogValue
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 模拟量值信息对象
        let info_bytes = self.analog_value.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 10 + 6 {
            // 至少需要1+10+6=17字节 (模拟量值是10字节)
            return Err(ParseError::InsufficientData {
                expected: 17,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Analog value should have exactly 1 object".to_string(),
            });
        }

        // 模拟量值信息对象 (10字节)
        let analog_value = crate::info_object::AnalogValue::parse(&data[offset..offset + 10])?;
        offset += 10;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            analog_value,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Analog value should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传建筑消防设施操作信息 (类型4)
///
/// 包含操作信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadOperationInfo {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 操作信息对象
    pub operation_info: crate::info_object::FireSystemOperation,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadOperationInfo {
    /// 创建新的上传操作信息数据单元
    pub fn new(
        operation_info: crate::info_object::FireSystemOperation,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            object_count: 1,
            operation_info,
            timestamp,
        }
    }

    /// 获取操作信息对象的引用
    pub fn operation_info(&self) -> &crate::info_object::FireSystemOperation {
        &self.operation_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadOperationInfo {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadOperationInfo
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 操作信息对象
        let info_bytes = self.operation_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 4 + 6 {
            // 至少需要1+4+6=11字节 (操作信息是4字节)
            return Err(ParseError::InsufficientData {
                expected: 11,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Operation info should have exactly 1 object".to_string(),
            });
        }

        // 操作信息对象 (4字节)
        let operation_info =
            crate::info_object::FireSystemOperation::parse(&data[offset..offset + 4])?;
        offset += 4;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            operation_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Operation info should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传建筑消防设施软件版本 (类型5)
///
/// 包含软件版本信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadSoftwareVersion {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 软件版本信息对象
    pub version_info: crate::info_object::FireSystemVersion,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadSoftwareVersion {
    /// 创建新的上传软件版本数据单元
    pub fn new(version_info: crate::info_object::FireSystemVersion, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            version_info,
            timestamp,
        }
    }

    /// 获取软件版本信息对象的引用
    pub fn version_info(&self) -> &crate::info_object::FireSystemVersion {
        &self.version_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadSoftwareVersion {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSoftwareVersion
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 软件版本信息对象
        let info_bytes = self.version_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 4 + 6 {
            // 至少需要1+4+6=11字节 (版本信息是4字节)
            return Err(ParseError::InsufficientData {
                expected: 11,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Software version should have exactly 1 object".to_string(),
            });
        }

        // 软件版本信息对象 (4字节)
        let version_info = crate::info_object::FireSystemVersion::parse(&data[offset..offset + 4])?;
        offset += 4;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            version_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Software version should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传建筑消防设施系统配置情况 (类型6)
///
/// 包含系统配置信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadSystemConfig {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 系统配置信息对象
    pub config_info: crate::info_object::FireSystemConfig,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadSystemConfig {
    /// 创建新的上传系统配置数据单元
    pub fn new(config_info: crate::info_object::FireSystemConfig, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            config_info,
            timestamp,
        }
    }

    /// 获取系统配置信息对象的引用
    pub fn config_info(&self) -> &crate::info_object::FireSystemConfig {
        &self.config_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadSystemConfig {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 系统配置信息对象
        let info_bytes = self.config_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 6 + 6 {
            // 至少需要1+6+6=13字节 (系统配置最小6字节)
            return Err(ParseError::InsufficientData {
                expected: 13,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "System config should have exactly 1 object".to_string(),
            });
        }

        // 计算配置信息大小 (总长度 - 对象数量 - 时间戳)
        let config_size = data.len() - 1 - 6;
        let config_info =
            crate::info_object::FireSystemConfig::parse(&data[offset..offset + config_size])?;
        offset += config_size;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            config_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "System config should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传建筑消防设施部件配置情况 (类型7)
///
/// 包含部件配置信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadComponentConfig {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 部件配置信息对象
    pub config_info: crate::info_object::ComponentConfig,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadComponentConfig {
    /// 创建新的上传部件配置数据单元
    pub fn new(config_info: crate::info_object::ComponentConfig, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            config_info,
            timestamp,
        }
    }

    /// 获取部件配置信息对象的引用
    pub fn config_info(&self) -> &crate::info_object::ComponentConfig {
        &self.config_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadComponentConfig {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadComponentConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 部件配置信息对象
        let info_bytes = self.config_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 43 + 6 {
            // 至少需要1+43+6=50字节 (部件配置是43字节)
            return Err(ParseError::InsufficientData {
                expected: 50,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Component config should have exactly 1 object".to_string(),
            });
        }

        // 部件配置信息对象 (43字节)
        let config_info = crate::info_object::ComponentConfig::parse(&data[offset..offset + 43])?;
        offset += 43;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            config_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Component config should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传建筑消防设施系统时间 (类型8)
///
/// 包含系统时间信息
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadSystemTime {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 系统时间戳
    pub system_time: Timestamp,
    /// 时间标签（数据单元时间标签）
    pub timestamp: Timestamp,
}

impl UploadSystemTime {
    /// 创建新的上传系统时间数据单元
    pub fn new(system_time: Timestamp, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            system_time,
            timestamp,
        }
    }

    /// 获取系统时间的引用
    pub fn system_time(&self) -> &Timestamp {
        &self.system_time
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadSystemTime {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemTime
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 系统时间 (6字节)
        let system_time_bytes = self.system_time.encode()?;
        buf.extend_from_slice(&system_time_bytes);

        // 时间标签 (6字节)
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 6 + 6 {
            // 需要1+6+6=13字节
            return Err(ParseError::InsufficientData {
                expected: 13,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "System time should have exactly 1 object".to_string(),
            });
        }

        // 系统时间 (6字节)
        let system_time = Timestamp::parse(&data[offset..offset + 6])?;
        offset += 6;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            system_time,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "System time should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传用户信息传输装置运行状态 (类型21)
///
/// 包含设备运行状态信息
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadDeviceStatus {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 设备状态 (2字节，小端序)
    pub device_status: u16,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadDeviceStatus {
    /// 创建新的上传设备状态数据单元
    pub fn new(device_status: u16, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            device_status,
            timestamp,
        }
    }

    /// 获取设备状态
    pub fn device_status(&self) -> u16 {
        self.device_status
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadDeviceStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadDeviceStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 设备状态 (2字节，小端序)
        buf.put_u16_le(self.device_status);

        // 时间标签 (6字节)
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 2 + 6 {
            // 需要1+2+6=9字节
            return Err(ParseError::InsufficientData {
                expected: 9,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Device status should have exactly 1 object".to_string(),
            });
        }

        // 设备状态 (2字节，小端序)
        let device_status = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            device_status,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Device status should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传用户信息传输装置操作信息 (类型24)
///
/// 包含设备操作信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadDeviceOperation {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 设备操作信息对象
    pub operation_info: crate::info_object::DeviceOperation,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadDeviceOperation {
    /// 创建新的上传设备操作信息数据单元
    pub fn new(operation_info: crate::info_object::DeviceOperation, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            operation_info,
            timestamp,
        }
    }

    /// 获取设备操作信息对象的引用
    pub fn operation_info(&self) -> &crate::info_object::DeviceOperation {
        &self.operation_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadDeviceOperation {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadDeviceOperation
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 设备操作信息对象
        let info_bytes = self.operation_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 2 + 6 {
            // 至少需要1+2+6=9字节 (设备操作信息是2字节)
            return Err(ParseError::InsufficientData {
                expected: 9,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Device operation should have exactly 1 object".to_string(),
            });
        }

        // 设备操作信息对象 (2字节)
        let operation_info = crate::info_object::DeviceOperation::parse(&data[offset..offset + 2])?;
        offset += 2;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            operation_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Device operation should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传用户信息传输装置软件版本 (类型25)
///
/// 包含设备软件版本信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadDeviceVersion {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 设备版本信息对象
    pub version_info: crate::info_object::DeviceVersion,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadDeviceVersion {
    /// 创建新的上传设备版本数据单元
    pub fn new(version_info: crate::info_object::DeviceVersion, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            version_info,
            timestamp,
        }
    }

    /// 获取设备版本信息对象的引用
    pub fn version_info(&self) -> &crate::info_object::DeviceVersion {
        &self.version_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadDeviceVersion {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadDeviceVersion
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 设备版本信息对象
        let info_bytes = self.version_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 4 + 6 {
            // 至少需要1+4+6=11字节 (设备版本信息是4字节)
            return Err(ParseError::InsufficientData {
                expected: 11,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Device version should have exactly 1 object".to_string(),
            });
        }

        // 设备版本信息对象 (4字节)
        let version_info = crate::info_object::DeviceVersion::parse(&data[offset..offset + 4])?;
        offset += 4;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            version_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Device version should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传用户信息传输装置配置情况 (类型26)
///
/// 包含设备配置信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadDeviceConfig {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 设备配置信息对象
    pub config_info: crate::info_object::DeviceConfig,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadDeviceConfig {
    /// 创建新的上传设备配置数据单元
    pub fn new(config_info: crate::info_object::DeviceConfig, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            config_info,
            timestamp,
        }
    }

    /// 获取设备配置信息对象的引用
    pub fn config_info(&self) -> &crate::info_object::DeviceConfig {
        &self.config_info
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadDeviceConfig {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadDeviceConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 设备配置信息对象
        let info_bytes = self.config_info.encode()?;
        buf.extend_from_slice(&info_bytes);

        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 26 + 6 {
            // 至少需要1+26+6=33字节 (设备配置信息是26字节)
            return Err(ParseError::InsufficientData {
                expected: 33,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Device config should have exactly 1 object".to_string(),
            });
        }

        // 设备配置信息对象 (26字节)
        let config_info = crate::info_object::DeviceConfig::parse(&data[offset..offset + 26])?;
        offset += 26;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            config_info,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Device config should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

/// 上传用户信息传输装置系统时间 (类型28)
///
/// 包含设备系统时间信息
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadDeviceTime {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 设备系统时间戳
    pub device_time: Timestamp,
    /// 时间标签（数据单元时间标签）
    pub timestamp: Timestamp,
}

impl UploadDeviceTime {
    /// 创建新的上传设备时间数据单元
    pub fn new(device_time: Timestamp, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            device_time,
            timestamp,
        }
    }

    /// 获取设备时间的引用
    pub fn device_time(&self) -> &Timestamp {
        &self.device_time
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadDeviceTime {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadDeviceTime
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);

        // 设备时间 (6字节)
        let device_time_bytes = self.device_time.encode()?;
        buf.extend_from_slice(&device_time_bytes);

        // 时间标签 (6字节)
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);

        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 6 + 6 {
            // 需要1+6+6=13字节
            return Err(ParseError::InsufficientData {
                expected: 13,
                actual: data.len(),
            });
        }

        let mut offset = 0;

        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "Device time should have exactly 1 object".to_string(),
            });
        }

        // 设备时间 (6字节)
        let device_time = Timestamp::parse(&data[offset..offset + 6])?;
        offset += 6;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset + 6])?;

        Ok(Self {
            object_count,
            device_time,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "Device time should have exactly 1 object".to_string(),
            });
        }

        Ok(())
    }
}

// TODO: 继续实现其余的上行数据单元类型 (2-28)
// 当前专注于建立可工作的基础架构

mod tests {
    use super::*;
    use crate::protocol::types::SystemType;
    #[test]
    fn test_upload_system_status_basic() {
        // 基本功能测试，先验证数据单元类型
        let system_status = InfoSystemStatus::new(
            SystemType::FireAlarm,
            0x12,
            0x1234,
            Timestamp::new(30, 15, 4, 26, 11, 24).unwrap(), // 30秒,15分,4时,26日,11月,24年(2024)
        );

        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap(); // 45秒,30分,15时,4日,11月,24年(2024)

        let upload = UploadSystemStatus::new(system_status, timestamp);

        // 验证基本属性
        assert_eq!(upload.data_unit_type(), DataUnitType::UploadSystemStatus);
        assert_eq!(upload.object_count, 1);
        assert_eq!(upload.system_status.system_type, SystemType::FireAlarm);
        assert_eq!(upload.system_status.system_address, 0x12);
        assert_eq!(upload.system_status.system_state, 0x1234);
    }
    #[test]
    fn test_upload_component_status_basic() {
        use crate::protocol::ComponentType;

        let component_status = crate::info_object::ComponentStatus::new(
            SystemType::FireAlarm,
            0x12,
            ComponentType::SmokeFireDetector,
            0x12345678,
            0x0002,
            [0u8; 31],
            Timestamp::new(30, 15, 4, 26, 11, 24).unwrap(),
        );

        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap();
        let upload = UploadComponentStatus::new(component_status, timestamp);

        assert_eq!(upload.data_unit_type(), DataUnitType::UploadComponentStatus);
        assert_eq!(upload.object_count, 1);
    }

    #[test]
    fn test_upload_analog_value_basic() {
        use crate::info_object::{AnalogType, AnalogValue};
        use crate::protocol::ComponentType;

        let analog_value = AnalogValue::new(
            SystemType::FireAlarm,
            0x12,
            ComponentType::TemperatureFireDetector,
            0x123456,
            AnalogType::Temperature,
            0x1234,
            Timestamp::new(30, 15, 4, 26, 11, 24).unwrap(),
        );

        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap();
        let upload = UploadAnalogValue::new(analog_value, timestamp);

        assert_eq!(upload.data_unit_type(), DataUnitType::UploadAnalogValue);
        assert_eq!(upload.object_count, 1);
    }

    #[test]
    fn test_upload_device_status_encode_decode() {
        let upload = UploadDeviceStatus::new(
            0x0001, // 设备状态
            Timestamp::new(45, 30, 15, 4, 11, 24).unwrap(),
        );

        // 编码
        let encoded = upload.encode().unwrap();
        assert_eq!(encoded.len(), 9); // 1 + 2 + 6

        // 解码
        let decoded = UploadDeviceStatus::parse(&encoded).unwrap();
        assert_eq!(upload, decoded);
        assert_eq!(decoded.device_status, 0x0001);
    }

    #[test]
    fn test_upload_system_time_encode_decode() {
        let system_time = Timestamp::new(30, 15, 4, 26, 11, 24).unwrap();
        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap();

        let upload = UploadSystemTime::new(system_time, timestamp);

        // 编码
        let encoded = upload.encode().unwrap();
        assert_eq!(encoded.len(), 13); // 1 + 6 + 6

        // 解码
        let decoded = UploadSystemTime::parse(&encoded).unwrap();
        assert_eq!(upload, decoded);
        assert_eq!(decoded.system_time, system_time);
        assert_eq!(decoded.timestamp, timestamp);
    }

    #[test]
    fn test_upload_device_time_encode_decode() {
        let device_time = Timestamp::new(30, 15, 4, 26, 11, 24).unwrap();
        let timestamp = Timestamp::new(45, 30, 15, 4, 11, 24).unwrap();

        let upload = UploadDeviceTime::new(device_time, timestamp);

        // 编码
        let encoded = upload.encode().unwrap();
        assert_eq!(encoded.len(), 13); // 1 + 6 + 6

        // 解码
        let decoded = UploadDeviceTime::parse(&encoded).unwrap();
        assert_eq!(upload, decoded);
        assert_eq!(decoded.device_time, device_time);
        assert_eq!(decoded.timestamp, timestamp);
    }
}
