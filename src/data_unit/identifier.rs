//! GB26875 数据单元标识符
//!
//! 根据 GB26875 协议，数据单元标识符用于标识不同类型的应用数据单元。
//! 标识符范围：
//! - 1-127: 标准数据单元类型
//! - 128-254: 用户自定义数据单元类型  
//! - 255: 保留

use crate::error::{ParseError, ParseResult};
use crate::protocol::DataUnitType;

/// 数据单元标识符
///
/// 包含数据单元类型标识和相关的解析功能
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataUnitIdentifier {
    /// 数据单元类型
    pub data_unit_type: DataUnitType,
    /// 是否为用户自定义类型
    pub is_custom: bool,
}

impl DataUnitIdentifier {
    /// 从数据单元类型创建标识符
    pub fn new(data_unit_type: DataUnitType) -> Self {
        let type_value = data_unit_type.to_u8();
        let is_custom = type_value >= 128 && type_value <= 254;

        Self {
            data_unit_type,
            is_custom,
        }
    }

    /// 从字节值创建标识符
    pub fn from_u8(value: u8) -> ParseResult<Self> {
        if value == 0 || value == 255 {
            return Err(ParseError::InvalidValue {
                field: "data_unit_type".to_string(),
                value: value.to_string(),
                reason: "Data unit type 0 and 255 are reserved".to_string(),
            });
        }

        let data_unit_type = DataUnitType::from_u8(value);
        Ok(Self::new(data_unit_type))
    }

    /// 转换为字节值
    pub fn to_u8(&self) -> u8 {
        self.data_unit_type.to_u8()
    }

    /// 检查是否为标准数据单元类型
    pub fn is_standard(&self) -> bool {
        !self.is_custom
    }

    /// 检查是否为上行数据单元（类型1-28）
    pub fn is_upstream(&self) -> bool {
        let type_value = self.to_u8();
        type_value >= 1 && type_value <= 28
    }

    /// 检查是否为下行数据单元（类型61-91）
    pub fn is_downstream(&self) -> bool {
        let type_value = self.to_u8();
        type_value >= 61 && type_value <= 91
    }

    /// 获取数据单元类别描述
    pub fn category(&self) -> &'static str {
        if self.is_custom {
            "用户自定义"
        } else if self.is_upstream() {
            "上行数据单元"
        } else if self.is_downstream() {
            "下行数据单元"
        } else {
            "其他标准类型"
        }
    }

    /// 获取数据单元类型的详细描述
    pub fn description(&self) -> String {
        // match self.data_unit_type {
        //     // 预留
        //     DataUnitType::Reserved => "预留".to_string(),
        //     DataUnitType::FireSystemReserved(n) => format!("建筑消防设施预留类型 {}", n),

        //     // 上行数据单元 (1-28)
        //     DataUnitType::UploadSystemStatus => "上传建筑消防设施系统状态".to_string(),
        //     DataUnitType::UploadComponentStatus => "上传建筑消防设施部件运行状态".to_string(),
        //     DataUnitType::UploadAnalogValue => "上传建筑消防设施模拟量值".to_string(),
        //     DataUnitType::UploadOperationInfo => "上传建筑消防设施操作信息".to_string(),
        //     DataUnitType::UploadSoftwareVersion => "上传建筑消防设施软件版本".to_string(),
        //     DataUnitType::UploadSystemConfig => "上传建筑消防设施系统配置".to_string(),
        //     DataUnitType::UploadComponentConfig => "上传建筑消防设施部件配置".to_string(),
        //     DataUnitType::UploadSystemTime => "上传建筑消防设施系统时间".to_string(),
        //     DataUnitType::UploadDeviceStatus => "上传用户信息传输装置状态".to_string(),

        //     // 下行数据单元 (61-91)
        //     DataUnitType::ReadSystemStatus => "读建筑消防设施系统状态".to_string(),
        //     DataUnitType::ReadComponentStatus => "读建筑消防设施部件运行状态".to_string(),
        //     DataUnitType::ReadAnalogValue => "读建筑消防设施模拟量值".to_string(),
        //     DataUnitType::ReadOperationInfo => "读建筑消防设施操作信息".to_string(),
        //     DataUnitType::ReadSoftwareVersion => "读建筑消防设施软件版本".to_string(),
        //     DataUnitType::ReadSystemConfig => "读建筑消防设施系统配置".to_string(),
        //     DataUnitType::ReadComponentConfig => "读建筑消防设施部件配置".to_string(),
        //     DataUnitType::ReadSystemTime => "读建筑消防设施系统时间".to_string(),
        //     DataUnitType::InitializeDevice => "初始化用户信息传输装置".to_string(),
        //     DataUnitType::SyncDeviceClock => "同步用户信息传输装置时钟".to_string(),
        //     DataUnitType::PatrolCommand => "查岗命令".to_string(),
        //       // 其他类型 - 使用协议定义的描述或通配符处理
        //     _ => self.data_unit_type.description().to_string(),
        // }

        self.data_unit_type.description().to_string()
    }
}

impl std::fmt::Display for DataUnitIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DataUnit(type={}, category={}, description={})",
            self.to_u8(),
            self.category(),
            self.description()
        )
    }
}

/// 数据单元标识符解析器
///
/// 用于从字节流中解析数据单元标识符
pub struct DataUnitIdentifierParser;

impl DataUnitIdentifierParser {
    /// 从字节流中解析标识符
    ///
    /// # Arguments
    /// * `data` - 包含标识符的字节数据
    ///
    /// # Returns
    /// * `ParseResult<(DataUnitIdentifier, usize)>` - 解析结果和消耗的字节数
    pub fn parse(data: &[u8]) -> ParseResult<(DataUnitIdentifier, usize)> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let identifier = DataUnitIdentifier::from_u8(data[0])?;
        Ok((identifier, 1))
    }

    /// 编码标识符为字节
    pub fn encode(identifier: &DataUnitIdentifier) -> Vec<u8> {
        vec![identifier.to_u8()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_creation() {
        // 测试标准类型
        let id1 = DataUnitIdentifier::from_u8(1).unwrap();
        assert_eq!(id1.to_u8(), 1);
        assert!(id1.is_standard());
        assert!(id1.is_upstream());
        assert!(!id1.is_downstream());
        assert!(!id1.is_custom);

        // 测试下行类型
        let id61 = DataUnitIdentifier::from_u8(61).unwrap();
        assert_eq!(id61.to_u8(), 61);
        assert!(id61.is_standard());
        assert!(!id61.is_upstream());
        assert!(id61.is_downstream());
        assert!(!id61.is_custom);

        // 测试自定义类型
        let id128 = DataUnitIdentifier::from_u8(128).unwrap();
        assert_eq!(id128.to_u8(), 128);
        assert!(!id128.is_standard());
        assert!(id128.is_custom);
    }

    #[test]
    fn test_invalid_identifiers() {
        // 测试保留值
        assert!(DataUnitIdentifier::from_u8(0).is_err());
        assert!(DataUnitIdentifier::from_u8(255).is_err());
    }

    #[test]
    fn test_parser() {
        let data = [1, 2, 3];
        let (id, consumed) = DataUnitIdentifierParser::parse(&data).unwrap();
        assert_eq!(id.to_u8(), 1);
        assert_eq!(consumed, 1);

        // 测试编码
        let encoded = DataUnitIdentifierParser::encode(&id);
        assert_eq!(encoded, vec![1]);
    }

    #[test]
    fn test_descriptions() {
        let id1 = DataUnitIdentifier::from_u8(1).unwrap();
        assert_eq!(id1.description(), "上传建筑消防设施系统状态");
        assert_eq!(id1.category(), "上行数据单元");

        let id61 = DataUnitIdentifier::from_u8(61).unwrap();
        assert_eq!(id61.description(), "读建筑消防设施系统状态");
        assert_eq!(id61.category(), "下行数据单元");

        let id128 = DataUnitIdentifier::from_u8(128).unwrap();
        assert_eq!(id128.category(), "用户自定义");
    }
}
