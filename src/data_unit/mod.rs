//! GB26875 应用数据单元模块
//!
//! 应用数据单元是 GB26875 协议中承载具体业务数据的部分，
//! 位于数据包的控制单元之后，结束符之前。

pub mod standard;

// 重新导出主要类型
pub use standard::*;

use crate::error::{ParseResult, EncodeResult};
use crate::protocol::DataUnitType;
use bytes::Bytes;

/// 应用数据单元 trait
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
/// 和动态分发。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GenericDataUnit {
    /// 系统状态（类型 1）
    SystemStatus(standard::SystemStatus),
    /// 部件类型（类型 2）
    ComponentType(standard::ComponentType),
    /// 部件状态（类型 3）
    ComponentStatus(standard::ComponentStatus),
    /// 模拟量值（类型 4）
    AnalogValue(standard::AnalogValue),
    /// 操作信息（类型 5）
    OperationInfo(standard::OperationInfo),
    /// 软件版本（类型 6）
    SoftwareVersion(standard::SoftwareVersion),
    /// 配置信息（类型 7）
    ConfigInfo(standard::ConfigInfo),
    /// 时间（类型 8）
    Time(standard::Time),
    /// 未知或用户自定义类型的原始数据
    Raw {
        /// 数据单元类型
        data_type: DataUnitType,
        /// 原始字节数据
        data: Bytes,
    },
}

impl GenericDataUnit {
    /// 从类型和原始数据创建通用数据单元    /// 
    /// # Arguments
    /// * `data_type` - 数据单元类型
    /// * `data` - 原始字节数据
    /// 
    /// # Returns
    /// * `Result<GenericDataUnit, ParseError>` - 成功返回解析的数据单元
    pub fn from_raw(data_type: DataUnitType, data: &[u8]) -> ParseResult<Self> {
        match data_type {
            DataUnitType::UploadSystemStatus => {
                let status = standard::SystemStatus::parse(data)?;
                Ok(GenericDataUnit::SystemStatus(status))
            }
            DataUnitType::UploadComponentStatus => {
                let status = standard::ComponentStatus::parse(data)?;
                Ok(GenericDataUnit::ComponentStatus(status))
            }
            DataUnitType::UploadAnalogValue => {
                let value = standard::AnalogValue::parse(data)?;
                Ok(GenericDataUnit::AnalogValue(value))
            }
            DataUnitType::UploadOperationInfo => {
                let info = standard::OperationInfo::parse(data)?;
                Ok(GenericDataUnit::OperationInfo(info))
            }
            DataUnitType::UploadSoftwareVersion => {
                let version = standard::SoftwareVersion::parse(data)?;
                Ok(GenericDataUnit::SoftwareVersion(version))
            }
            DataUnitType::UploadSystemConfig => {
                let config = standard::ConfigInfo::parse(data)?;
                Ok(GenericDataUnit::ConfigInfo(config))
            }
            DataUnitType::UploadSystemTime => {
                let time = standard::Time::parse(data)?;
                Ok(GenericDataUnit::Time(time))
            }
            _ => {
                // 对于未知类型，存储为原始数据
                Ok(GenericDataUnit::Raw {
                    data_type,
                    data: Bytes::copy_from_slice(data),
                })
            }
        }
    }    /// 获取数据单元类型
    pub fn data_unit_type(&self) -> DataUnitType {
        match self {
            GenericDataUnit::SystemStatus(_) => DataUnitType::UploadSystemStatus,
            GenericDataUnit::ComponentType(_) => DataUnitType::UploadComponentConfig,
            GenericDataUnit::ComponentStatus(_) => DataUnitType::UploadComponentStatus,
            GenericDataUnit::AnalogValue(_) => DataUnitType::UploadAnalogValue,
            GenericDataUnit::OperationInfo(_) => DataUnitType::UploadOperationInfo,
            GenericDataUnit::SoftwareVersion(_) => DataUnitType::UploadSoftwareVersion,
            GenericDataUnit::ConfigInfo(_) => DataUnitType::UploadSystemConfig,
            GenericDataUnit::Time(_) => DataUnitType::UploadSystemTime,
            GenericDataUnit::Raw { data_type, .. } => *data_type,
        }
    }

    /// 编码为字节序列
    pub fn encode(&self) -> EncodeResult<Bytes> {
        match self {
            GenericDataUnit::SystemStatus(s) => s.encode(),
            GenericDataUnit::ComponentType(c) => c.encode(),
            GenericDataUnit::ComponentStatus(s) => s.encode(),
            GenericDataUnit::AnalogValue(v) => v.encode(),
            GenericDataUnit::OperationInfo(o) => o.encode(),
            GenericDataUnit::SoftwareVersion(v) => v.encode(),
            GenericDataUnit::ConfigInfo(c) => c.encode(),
            GenericDataUnit::Time(t) => t.encode(),
            GenericDataUnit::Raw { data, .. } => Ok(data.clone()),
        }
    }

    /// 验证数据单元
    pub fn validate(&self) -> ParseResult<()> {
        match self {
            GenericDataUnit::SystemStatus(s) => s.validate(),
            GenericDataUnit::ComponentType(c) => c.validate(),
            GenericDataUnit::ComponentStatus(s) => s.validate(),
            GenericDataUnit::AnalogValue(v) => v.validate(),
            GenericDataUnit::OperationInfo(o) => o.validate(),
            GenericDataUnit::SoftwareVersion(v) => v.validate(),
            GenericDataUnit::ConfigInfo(c) => c.validate(),
            GenericDataUnit::Time(t) => t.validate(),
            GenericDataUnit::Raw { .. } => Ok(()), // 原始数据无需特殊验证
        }
    }

    /// 获取字节长度
    pub fn byte_length(&self) -> usize {
        match self {
            GenericDataUnit::Raw { data, .. } => data.len(),
            _ => self.encode().map(|b| b.len()).unwrap_or(0),
        }
    }

    /// 检查是否为原始数据
    pub fn is_raw(&self) -> bool {
        matches!(self, GenericDataUnit::Raw { .. })
    }

    /// 获取原始数据（如果是原始类型）
    pub fn as_raw(&self) -> Option<(DataUnitType, &Bytes)> {
        match self {
            GenericDataUnit::Raw { data_type, data } => Some((*data_type, data)),
            _ => None,
        }
    }
}

impl std::fmt::Display for GenericDataUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericDataUnit::SystemStatus(s) => write!(f, "SystemStatus({})", s),
            GenericDataUnit::ComponentType(c) => write!(f, "ComponentType({})", c),
            GenericDataUnit::ComponentStatus(s) => write!(f, "ComponentStatus({})", s),
            GenericDataUnit::AnalogValue(v) => write!(f, "AnalogValue({})", v),
            GenericDataUnit::OperationInfo(o) => write!(f, "OperationInfo({})", o),
            GenericDataUnit::SoftwareVersion(v) => write!(f, "SoftwareVersion({})", v),
            GenericDataUnit::ConfigInfo(c) => write!(f, "ConfigInfo({})", c),
            GenericDataUnit::Time(t) => write!(f, "Time({})", t),
            GenericDataUnit::Raw { data_type, data } => {
                write!(f, "Raw({:?}, {} bytes)", data_type, data.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_data_unit_raw() {
        let data_type = DataUnitType::UserDefined(200);
        let raw_data = b"test raw data";
        
        let unit = GenericDataUnit::from_raw(data_type, raw_data).unwrap();
        
        assert_eq!(unit.data_unit_type(), data_type);
        assert!(unit.is_raw());
        
        if let Some((dt, data)) = unit.as_raw() {
            assert_eq!(dt, data_type);
            assert_eq!(data.as_ref(), raw_data);
        } else {
            panic!("Expected raw data unit");
        }
    }

    #[test]
    fn test_generic_data_unit_encode_decode() {
        let data_type = DataUnitType::UserDefined(150);
        let raw_data = b"test data for encoding";
        
        let unit = GenericDataUnit::Raw {
            data_type,
            data: Bytes::from_static(raw_data),
        };
        
        let encoded = unit.encode().unwrap();
        assert_eq!(encoded.as_ref(), raw_data);
        assert_eq!(unit.byte_length(), raw_data.len());
    }
}
