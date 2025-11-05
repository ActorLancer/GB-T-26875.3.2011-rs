//! GB26875 应用数据单元模块
//!
//! 应用数据单元是 GB26875 协议中承载具体业务数据的部分，
//! 位于数据包的控制单元之后，结束符之前。
//!
//! ## 模块结构
//!
//! - `identifier`: 数据单元标识符管理
//! - `standard`: 标准数据单元实现（类型 1-127）
//! - `custom`: 自定义数据单元抽象（类型 128-254）

// 子模块声明
pub mod custom;
pub mod identifier;
pub mod standard;

// 重新导出主要类型
pub use custom::{
    CustomDataUnit, CustomDataUnitFactory, CustomDataUnitRegistry, RawCustomDataUnit, RegistryError,
};
pub use identifier::{DataUnitIdentifier, DataUnitIdentifierParser};
pub use standard::{downstream, upstream};

use crate::error::{EncodeResult, ParseResult};
use crate::protocol::DataUnitType;
use bytes::Bytes;

/// 应用数据单元基础 trait
///
/// 所有应用数据单元都应该实现这个 trait，提供统一的
/// 编解码接口和类型识别能力。
pub trait DataUnit: std::fmt::Debug + Send + Sync {
    /// 获取数据单元类型
    fn data_unit_type(&self) -> DataUnitType;

    /// 编码为字节序列
    fn encode(&self) -> EncodeResult<Bytes>;

    /// 从字节序列解析
    fn parse(data: &[u8]) -> ParseResult<Self>
    where
        Self: Sized;

    /// 验证数据单元的有效性
    fn validate(&self) -> ParseResult<()> {
        Ok(())
    }

    /// 获取数据单元的字节长度
    fn byte_length(&self) -> usize {
        self.encode().map(|b| b.len()).unwrap_or(0)
    }
}

/// 通用数据单元包装器
///
/// 用于处理已知和未知类型的数据单元，支持运行时类型识别
/// 和动态分发。支持标准数据单元和自定义数据单元的统一处理。
#[derive(Debug)]
pub enum GenericDataUnit {
    /// 上行数据单元 - 上传建筑消防设施系统状态 (类型1)
    UploadSystemStatus(standard::upstream::UploadSystemStatus),

    /// 上行数据单元 - 上传建筑消防设施部件运行状态 (类型2)
    UploadComponentStatus(standard::upstream::UploadComponentStatus),

    /// 上行数据单元 - 上传建筑消防设施部件模拟量值 (类型3)
    UploadAnalogValue(standard::upstream::UploadAnalogValue),

    /// 上行数据单元 - 上传建筑消防设施操作信息 (类型4)
    UploadOperationInfo(standard::upstream::UploadOperationInfo),

    /// 上行数据单元 - 上传建筑消防设施软件版本 (类型5)
    UploadSoftwareVersion(standard::upstream::UploadSoftwareVersion),

    /// 上行数据单元 - 上传建筑消防设施系统配置情况 (类型6)
    UploadSystemConfig(standard::upstream::UploadSystemConfig),

    /// 上行数据单元 - 上传建筑消防设施部件配置情况 (类型7)
    UploadComponentConfig(standard::upstream::UploadComponentConfig),

    /// 上行数据单元 - 上传建筑消防设施系统时间 (类型8)
    UploadSystemTime(standard::upstream::UploadSystemTime),

    /// 上行数据单元 - 上传用户信息传输装置运行状态 (类型21)
    UploadDeviceStatus(standard::upstream::UploadDeviceStatus),

    /// 上行数据单元 - 上传用户信息传输装置操作信息 (类型24)
    UploadDeviceOperation(standard::upstream::UploadDeviceOperation),

    /// 上行数据单元 - 上传用户信息传输装置软件版本 (类型25)
    UploadDeviceVersion(standard::upstream::UploadDeviceVersion),

    /// 上行数据单元 - 上传用户信息传输装置配置情况 (类型26)
    UploadDeviceConfig(standard::upstream::UploadDeviceConfig),

    /// 上行数据单元 - 上传用户信息传输装置系统时间 (类型28)
    UploadDeviceTime(standard::upstream::UploadDeviceTime),

    /// 下行数据单元 - 读建筑消防设施系统状态 (类型61)
    ReadSystemStatus(standard::downstream::ReadSystemStatus),

    /// 下行数据单元 - 读建筑消防设施部件运行状态 (类型62)
    ReadComponentStatus(standard::downstream::ReadComponentStatus),
    /// 下行数据单元 - 读建筑消防设施模拟量值 (类型63)
    ReadAnalogValue(standard::downstream::ReadAnalogValue),

    /// 下行数据单元 - 读建筑消防设施操作信息 (类型64)
    ReadOperationInfo(standard::downstream::ReadOperationInfo),

    /// 下行数据单元 - 读建筑消防设施软件版本 (类型65)
    ReadSoftwareVersion(standard::downstream::ReadSoftwareVersion),

    /// 下行数据单元 - 读建筑消防设施系统配置情况 (类型66)
    ReadSystemConfig(standard::downstream::ReadSystemConfig),

    /// 下行数据单元 - 读建筑消防设施部件配置情况 (类型67)
    ReadComponentConfig(standard::downstream::ReadComponentConfig),

    /// 下行数据单元 - 读建筑消防设施系统时间 (类型68)
    ReadSystemTime(standard::downstream::ReadSystemTime),

    /// 下行数据单元 - 读用户信息传输装置运行状态 (类型81)
    ReadDeviceStatus(standard::downstream::ReadDeviceStatus),

    /// 下行数据单元 - 读用户信息传输装置操作信息记录 (类型84)
    ReadDeviceOperation(standard::downstream::ReadDeviceOperation),

    /// 下行数据单元 - 读用户信息传输装置软件版本 (类型85)
    ReadDeviceVersion(standard::downstream::ReadDeviceVersion),

    /// 下行数据单元 - 读用户信息传输装置配置情况 (类型86)
    ReadDeviceConfig(standard::downstream::ReadDeviceConfig),

    /// 下行数据单元 - 读用户信息传输装置系统时间 (类型88)
    ReadDeviceTime(standard::downstream::ReadDeviceTime),

    /// 下行数据单元 - 初始化用户信息传输装置 (类型89)
    InitializeDevice(standard::downstream::InitializeDevice),

    /// 下行数据单元 - 同步用户信息传输装置时钟 (类型90)
    SyncDeviceClock(standard::downstream::SyncDeviceClock),

    /// 下行数据单元 - 查岗命令 (类型91)
    PatrolCommand(standard::downstream::PatrolCommand),

    /// 自定义数据单元 (类型128-254)
    Custom(Box<dyn CustomDataUnit>),

    /// 扩展数据单元 (类型128-254，使用过程宏注册)
    Extension {
        /// 类型代码
        type_code: u8,
        /// 扩展数据
        data: Bytes,
        /// 描述信息
        description: String,
    },

    /// 未知或无法解析的原始数据
    Raw {
        /// 数据单元类型
        data_type: DataUnitType,
        /// 原始字节数据
        data: Bytes,
    },

    /// 未知数据单元类型
    Unknown {
        /// 数据单元类型
        data_unit_type: DataUnitType,
        /// 原始字节数据
        raw_data: Vec<u8>,
    },
}

impl GenericDataUnit {
    /// 从数据单元类型和原始数据创建通用数据单元
    ///
    /// 这个方法会尝试解析标准数据单元类型，如果失败则尝试
    /// 自定义数据单元注册表，最后回退到原始数据包装。
    ///
    /// # Arguments
    /// * `data_type` - 数据单元类型
    /// * `data` - 原始字节数据
    ///
    /// # Returns
    /// * `Result<GenericDataUnit, ParseError>` - 成功返回解析的数据单元
    pub fn from_raw(data_type: DataUnitType, data: &[u8]) -> ParseResult<Self> {
        match data_type {
            // 上行数据单元
            DataUnitType::UploadSystemStatus => {
                let unit = standard::upstream::UploadSystemStatus::parse(data)?;
                Ok(GenericDataUnit::UploadSystemStatus(unit))
            }
            DataUnitType::UploadComponentStatus => {
                let unit = standard::upstream::UploadComponentStatus::parse(data)?;
                Ok(GenericDataUnit::UploadComponentStatus(unit))
            }
            DataUnitType::UploadAnalogValue => {
                let unit = standard::upstream::UploadAnalogValue::parse(data)?;
                Ok(GenericDataUnit::UploadAnalogValue(unit))
            }
            DataUnitType::UploadOperationInfo => {
                let unit = standard::upstream::UploadOperationInfo::parse(data)?;
                Ok(GenericDataUnit::UploadOperationInfo(unit))
            }
            DataUnitType::UploadSoftwareVersion => {
                let unit = standard::upstream::UploadSoftwareVersion::parse(data)?;
                Ok(GenericDataUnit::UploadSoftwareVersion(unit))
            }
            DataUnitType::UploadSystemConfig => {
                let unit = standard::upstream::UploadSystemConfig::parse(data)?;
                Ok(GenericDataUnit::UploadSystemConfig(unit))
            }
            DataUnitType::UploadComponentConfig => {
                let unit = standard::upstream::UploadComponentConfig::parse(data)?;
                Ok(GenericDataUnit::UploadComponentConfig(unit))
            }
            DataUnitType::UploadSystemTime => {
                let unit = standard::upstream::UploadSystemTime::parse(data)?;
                Ok(GenericDataUnit::UploadSystemTime(unit))
            }
            DataUnitType::UploadDeviceStatus => {
                let unit = standard::upstream::UploadDeviceStatus::parse(data)?;
                Ok(GenericDataUnit::UploadDeviceStatus(unit))
            }
            DataUnitType::UploadDeviceOperation => {
                let unit = standard::upstream::UploadDeviceOperation::parse(data)?;
                Ok(GenericDataUnit::UploadDeviceOperation(unit))
            }
            DataUnitType::UploadDeviceVersion => {
                let unit = standard::upstream::UploadDeviceVersion::parse(data)?;
                Ok(GenericDataUnit::UploadDeviceVersion(unit))
            }
            DataUnitType::UploadDeviceConfig => {
                let unit = standard::upstream::UploadDeviceConfig::parse(data)?;
                Ok(GenericDataUnit::UploadDeviceConfig(unit))
            }
            DataUnitType::UploadDeviceTime => {
                let unit = standard::upstream::UploadDeviceTime::parse(data)?;
                Ok(GenericDataUnit::UploadDeviceTime(unit))
            }

            // 下行数据单元
            DataUnitType::ReadSystemStatus => {
                let unit = standard::downstream::ReadSystemStatus::parse(data)?;
                Ok(GenericDataUnit::ReadSystemStatus(unit))
            }
            DataUnitType::ReadComponentStatus => {
                let unit = standard::downstream::ReadComponentStatus::parse(data)?;
                Ok(GenericDataUnit::ReadComponentStatus(unit))
            }
            DataUnitType::ReadAnalogValue => {
                let unit = standard::downstream::ReadAnalogValue::parse(data)?;
                Ok(GenericDataUnit::ReadAnalogValue(unit))
            }
            DataUnitType::ReadOperationInfo => {
                let unit = standard::downstream::ReadOperationInfo::parse(data)?;
                Ok(GenericDataUnit::ReadOperationInfo(unit))
            }
            DataUnitType::ReadSoftwareVersion => {
                let unit = standard::downstream::ReadSoftwareVersion::parse(data)?;
                Ok(GenericDataUnit::ReadSoftwareVersion(unit))
            }
            DataUnitType::ReadSystemConfig => {
                let unit = standard::downstream::ReadSystemConfig::parse(data)?;
                Ok(GenericDataUnit::ReadSystemConfig(unit))
            }
            DataUnitType::ReadComponentConfig => {
                let unit = standard::downstream::ReadComponentConfig::parse(data)?;
                Ok(GenericDataUnit::ReadComponentConfig(unit))
            }
            DataUnitType::ReadSystemTime => {
                let unit = standard::downstream::ReadSystemTime::parse(data)?;
                Ok(GenericDataUnit::ReadSystemTime(unit))
            }
            DataUnitType::ReadDeviceStatus => {
                let unit = standard::downstream::ReadDeviceStatus::parse(data)?;
                Ok(GenericDataUnit::ReadDeviceStatus(unit))
            }
            DataUnitType::ReadDeviceOperation => {
                let unit = standard::downstream::ReadDeviceOperation::parse(data)?;
                Ok(GenericDataUnit::ReadDeviceOperation(unit))
            }
            DataUnitType::ReadDeviceVersion => {
                let unit = standard::downstream::ReadDeviceVersion::parse(data)?;
                Ok(GenericDataUnit::ReadDeviceVersion(unit))
            }
            DataUnitType::ReadDeviceConfig => {
                let unit = standard::downstream::ReadDeviceConfig::parse(data)?;
                Ok(GenericDataUnit::ReadDeviceConfig(unit))
            }
            DataUnitType::ReadDeviceTime => {
                let unit = standard::downstream::ReadDeviceTime::parse(data)?;
                Ok(GenericDataUnit::ReadDeviceTime(unit))
            }
            DataUnitType::InitializeDevice => {
                let unit = standard::downstream::InitializeDevice::parse(data)?;
                Ok(GenericDataUnit::InitializeDevice(unit))
            }
            DataUnitType::SyncDeviceClock => {
                let unit = standard::downstream::SyncDeviceClock::parse(data)?;
                Ok(GenericDataUnit::SyncDeviceClock(unit))
            }
            DataUnitType::PatrolCommand => {
                let unit = standard::downstream::PatrolCommand::parse(data)?;
                Ok(GenericDataUnit::PatrolCommand(unit))
            }

            // 其他类型暂时保存为原始数据
            _ => Ok(GenericDataUnit::Raw {
                data_type,
                data: Bytes::copy_from_slice(data),
            }),
        }
    }

    /// 从数据单元类型和内容解析通用数据单元
    ///
    /// 这是对`from_raw`方法的别名，提供与codec模块的兼容性
    ///
    /// # Arguments
    /// * `data_type` - 数据单元类型
    /// * `content` - 数据内容（不包含类型标识符）
    ///
    /// # Returns
    /// * `Result<GenericDataUnit, ParseError>` - 成功返回解析的数据单元
    pub fn parse_from_type_and_content(data_type: DataUnitType, content: &[u8]) -> ParseResult<Self> {
        Self::from_raw(data_type, content)
    }

    /// 获取数据单元类型
    pub fn data_unit_type(&self) -> DataUnitType {
        match self {
            GenericDataUnit::UploadSystemStatus(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadComponentStatus(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadAnalogValue(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadOperationInfo(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadSoftwareVersion(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadSystemConfig(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadComponentConfig(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadSystemTime(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadDeviceStatus(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadDeviceOperation(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadDeviceVersion(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadDeviceConfig(unit) => unit.data_unit_type(),
            GenericDataUnit::UploadDeviceTime(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadSystemStatus(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadComponentStatus(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadAnalogValue(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadOperationInfo(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadSoftwareVersion(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadSystemConfig(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadComponentConfig(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadSystemTime(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadDeviceStatus(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadDeviceOperation(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadDeviceVersion(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadDeviceConfig(unit) => unit.data_unit_type(),
            GenericDataUnit::ReadDeviceTime(unit) => unit.data_unit_type(),
            GenericDataUnit::InitializeDevice(unit) => unit.data_unit_type(),
            GenericDataUnit::SyncDeviceClock(unit) => unit.data_unit_type(),
            GenericDataUnit::PatrolCommand(unit) => unit.data_unit_type(),
            GenericDataUnit::Custom(unit) => unit.data_unit_type(),
            GenericDataUnit::Extension { type_code, .. } => DataUnitType::UserDefined(*type_code),
            GenericDataUnit::Raw { data_type, .. } => *data_type,
            GenericDataUnit::Unknown { data_unit_type, .. } => *data_unit_type,
        }
    }

    /// 编码为字节序列
    pub fn encode(&self) -> EncodeResult<Bytes> {
        match self {
            GenericDataUnit::UploadSystemStatus(unit) => unit.encode(),
            GenericDataUnit::UploadComponentStatus(unit) => unit.encode(),
            GenericDataUnit::UploadAnalogValue(unit) => unit.encode(),
            GenericDataUnit::UploadOperationInfo(unit) => unit.encode(),
            GenericDataUnit::UploadSoftwareVersion(unit) => unit.encode(),
            GenericDataUnit::UploadSystemConfig(unit) => unit.encode(),
            GenericDataUnit::UploadComponentConfig(unit) => unit.encode(),
            GenericDataUnit::UploadSystemTime(unit) => unit.encode(),
            GenericDataUnit::UploadDeviceStatus(unit) => unit.encode(),
            GenericDataUnit::UploadDeviceOperation(unit) => unit.encode(),
            GenericDataUnit::UploadDeviceVersion(unit) => unit.encode(),
            GenericDataUnit::UploadDeviceConfig(unit) => unit.encode(),
            GenericDataUnit::UploadDeviceTime(unit) => unit.encode(),
            GenericDataUnit::ReadSystemStatus(unit) => unit.encode(),
            GenericDataUnit::ReadComponentStatus(unit) => unit.encode(),
            GenericDataUnit::ReadAnalogValue(unit) => unit.encode(),
            GenericDataUnit::ReadOperationInfo(unit) => unit.encode(),
            GenericDataUnit::ReadSoftwareVersion(unit) => unit.encode(),
            GenericDataUnit::ReadSystemConfig(unit) => unit.encode(),
            GenericDataUnit::ReadComponentConfig(unit) => unit.encode(),
            GenericDataUnit::ReadSystemTime(unit) => unit.encode(),
            GenericDataUnit::ReadDeviceStatus(unit) => unit.encode(),
            GenericDataUnit::ReadDeviceOperation(unit) => unit.encode(),
            GenericDataUnit::ReadDeviceVersion(unit) => unit.encode(),
            GenericDataUnit::ReadDeviceConfig(unit) => unit.encode(),
            GenericDataUnit::ReadDeviceTime(unit) => unit.encode(),
            GenericDataUnit::InitializeDevice(unit) => unit.encode(),
            GenericDataUnit::SyncDeviceClock(unit) => unit.encode(),
            GenericDataUnit::PatrolCommand(unit) => unit.encode(),
            GenericDataUnit::Custom(unit) => unit.encode(),
            GenericDataUnit::Extension { data, .. } => Ok(data.clone()),
            GenericDataUnit::Raw { data, .. } => Ok(data.clone()),
            GenericDataUnit::Unknown { raw_data, .. } => Ok(Bytes::copy_from_slice(raw_data)),
        }
    }

    /// 验证数据单元
    pub fn validate(&self) -> ParseResult<()> {
        match self {
            GenericDataUnit::UploadSystemStatus(unit) => unit.validate(),
            GenericDataUnit::UploadComponentStatus(unit) => unit.validate(),
            GenericDataUnit::UploadAnalogValue(unit) => unit.validate(),
            GenericDataUnit::UploadOperationInfo(unit) => unit.validate(),
            GenericDataUnit::UploadSoftwareVersion(unit) => unit.validate(),
            GenericDataUnit::UploadSystemConfig(unit) => unit.validate(),
            GenericDataUnit::UploadComponentConfig(unit) => unit.validate(),
            GenericDataUnit::UploadSystemTime(unit) => unit.validate(),
            GenericDataUnit::UploadDeviceStatus(unit) => unit.validate(),
            GenericDataUnit::UploadDeviceOperation(unit) => unit.validate(),
            GenericDataUnit::UploadDeviceVersion(unit) => unit.validate(),
            GenericDataUnit::UploadDeviceConfig(unit) => unit.validate(),
            GenericDataUnit::UploadDeviceTime(unit) => unit.validate(),
            GenericDataUnit::ReadSystemStatus(unit) => unit.validate(),
            GenericDataUnit::ReadComponentStatus(unit) => unit.validate(),
            GenericDataUnit::ReadAnalogValue(unit) => unit.validate(),
            GenericDataUnit::ReadOperationInfo(unit) => unit.validate(),
            GenericDataUnit::ReadSoftwareVersion(unit) => unit.validate(),
            GenericDataUnit::ReadSystemConfig(unit) => unit.validate(),
            GenericDataUnit::ReadComponentConfig(unit) => unit.validate(),
            GenericDataUnit::ReadSystemTime(unit) => unit.validate(),
            GenericDataUnit::ReadDeviceStatus(unit) => unit.validate(),
            GenericDataUnit::ReadDeviceOperation(unit) => unit.validate(),
            GenericDataUnit::ReadDeviceVersion(unit) => unit.validate(),
            GenericDataUnit::ReadDeviceConfig(unit) => unit.validate(),
            GenericDataUnit::ReadDeviceTime(unit) => unit.validate(),
            GenericDataUnit::InitializeDevice(unit) => unit.validate(),
            GenericDataUnit::SyncDeviceClock(unit) => unit.validate(),
            GenericDataUnit::PatrolCommand(unit) => unit.validate(),
            GenericDataUnit::Custom(unit) => unit.validate(),
            GenericDataUnit::Extension { .. } => Ok(()), // 扩展数据单元在注册时已验证
            GenericDataUnit::Raw { .. } => Ok(()), // 原始数据无需特殊验证
            GenericDataUnit::Unknown { .. } => Ok(()), // 未知数据无需验证
        }
    }

    /// 检查是否为上行数据单元
    pub fn is_upstream(&self) -> bool {
        let identifier = DataUnitIdentifier::new(self.data_unit_type());
        identifier.is_upstream()
    }

    /// 检查是否为下行数据单元
    pub fn is_downstream(&self) -> bool {
        let identifier = DataUnitIdentifier::new(self.data_unit_type());
        identifier.is_downstream()
    }

    /// 检查是否为自定义数据单元
    pub fn is_custom(&self) -> bool {
        match self {
            GenericDataUnit::Custom(_) => true,
            GenericDataUnit::Raw { data_type, .. } => {
                let type_value = data_type.to_u8();
                (128..=254).contains(&type_value)
            }
            _ => false,
        }
    }

    /// 获取描述信息
    pub fn description(&self) -> String {
        let identifier = DataUnitIdentifier::new(self.data_unit_type());
        identifier.description()
    }
}

impl std::fmt::Display for GenericDataUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.description(),
            self.data_unit_type().to_u8()
        )
    }
}

impl Clone for GenericDataUnit {
    fn clone(&self) -> Self {
        match self {
            GenericDataUnit::UploadSystemStatus(unit) => {
                GenericDataUnit::UploadSystemStatus(unit.clone())
            }
            GenericDataUnit::UploadComponentStatus(unit) => {
                GenericDataUnit::UploadComponentStatus(unit.clone())
            }
            GenericDataUnit::UploadAnalogValue(unit) => {
                GenericDataUnit::UploadAnalogValue(unit.clone())
            }
            GenericDataUnit::UploadOperationInfo(unit) => {
                GenericDataUnit::UploadOperationInfo(unit.clone())
            }
            GenericDataUnit::UploadSoftwareVersion(unit) => {
                GenericDataUnit::UploadSoftwareVersion(unit.clone())
            }
            GenericDataUnit::UploadSystemConfig(unit) => {
                GenericDataUnit::UploadSystemConfig(unit.clone())
            }
            GenericDataUnit::UploadComponentConfig(unit) => {
                GenericDataUnit::UploadComponentConfig(unit.clone())
            }
            GenericDataUnit::UploadSystemTime(unit) => {
                GenericDataUnit::UploadSystemTime(unit.clone())
            }
            GenericDataUnit::UploadDeviceStatus(unit) => {
                GenericDataUnit::UploadDeviceStatus(unit.clone())
            }
            GenericDataUnit::UploadDeviceOperation(unit) => {
                GenericDataUnit::UploadDeviceOperation(unit.clone())
            }
            GenericDataUnit::UploadDeviceVersion(unit) => {
                GenericDataUnit::UploadDeviceVersion(unit.clone())
            }
            GenericDataUnit::UploadDeviceConfig(unit) => {
                GenericDataUnit::UploadDeviceConfig(unit.clone())
            }
            GenericDataUnit::UploadDeviceTime(unit) => {
                GenericDataUnit::UploadDeviceTime(unit.clone())
            }
            GenericDataUnit::ReadSystemStatus(unit) => {
                GenericDataUnit::ReadSystemStatus(unit.clone())
            }
            GenericDataUnit::ReadComponentStatus(unit) => {
                GenericDataUnit::ReadComponentStatus(unit.clone())
            }
            GenericDataUnit::ReadAnalogValue(unit) => {
                GenericDataUnit::ReadAnalogValue(unit.clone())
            }
            GenericDataUnit::ReadOperationInfo(unit) => {
                GenericDataUnit::ReadOperationInfo(unit.clone())
            }
            GenericDataUnit::ReadSoftwareVersion(unit) => {
                GenericDataUnit::ReadSoftwareVersion(*unit)
            }
            GenericDataUnit::ReadSystemConfig(unit) => {
                GenericDataUnit::ReadSystemConfig(unit.clone())
            }
            GenericDataUnit::ReadComponentConfig(unit) => {
                GenericDataUnit::ReadComponentConfig(unit.clone())
            }
            GenericDataUnit::ReadSystemTime(unit) => GenericDataUnit::ReadSystemTime(*unit),
            GenericDataUnit::ReadDeviceStatus(unit) => GenericDataUnit::ReadDeviceStatus(*unit),
            GenericDataUnit::ReadDeviceOperation(unit) => {
                GenericDataUnit::ReadDeviceOperation(unit.clone())
            }
            GenericDataUnit::ReadDeviceVersion(unit) => GenericDataUnit::ReadDeviceVersion(*unit),
            GenericDataUnit::ReadDeviceConfig(unit) => GenericDataUnit::ReadDeviceConfig(*unit),
            GenericDataUnit::ReadDeviceTime(unit) => GenericDataUnit::ReadDeviceTime(*unit),
            GenericDataUnit::InitializeDevice(unit) => GenericDataUnit::InitializeDevice(*unit),
            GenericDataUnit::SyncDeviceClock(unit) => {
                GenericDataUnit::SyncDeviceClock(unit.clone())
            }
            GenericDataUnit::PatrolCommand(unit) => GenericDataUnit::PatrolCommand(*unit),
            GenericDataUnit::Custom(unit) => GenericDataUnit::Custom(unit.clone_box()),
            GenericDataUnit::Extension { type_code, data, description } => GenericDataUnit::Extension {
                type_code: *type_code,
                data: data.clone(),
                description: description.clone(),
            },
            GenericDataUnit::Raw { data_type, data } => GenericDataUnit::Raw {
                data_type: *data_type,
                data: data.clone(),
            },
            GenericDataUnit::Unknown { data_unit_type, raw_data } => GenericDataUnit::Unknown {
                data_unit_type: *data_unit_type,
                raw_data: raw_data.clone(),
            },
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_generic_data_unit_creation() {
        // 测试从原始数据创建
        let data = vec![1, 2, 3, 4];

        // 测试未知类型
        let raw_unit = GenericDataUnit::Raw {
            data_type: DataUnitType::UploadSystemStatus,
            data: Bytes::from(data.clone()),
        };

        assert_eq!(raw_unit.data_unit_type(), DataUnitType::UploadSystemStatus);

        // 测试类型判断
        assert!(raw_unit.is_upstream());
        assert!(!raw_unit.is_downstream());
        assert!(!raw_unit.is_custom());
    }

    #[test]
    fn test_data_unit_identifier() {
        let id = DataUnitIdentifier::from_u8(1).unwrap();
        assert!(id.is_upstream());
        assert!(!id.is_downstream());
        assert!(!id.is_custom);
        assert_eq!(id.category(), "上行数据单元");

        let id61 = DataUnitIdentifier::from_u8(61).unwrap();
        assert!(!id61.is_upstream());
        assert!(id61.is_downstream());
        assert!(!id61.is_custom);

        let id128 = DataUnitIdentifier::from_u8(128).unwrap();
        assert!(!id128.is_upstream());
        assert!(!id128.is_downstream());
        assert!(id128.is_custom);
    }
}
