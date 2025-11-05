//! 数据单元编解码器
//!
//! 提供应用数据单元的编解码功能，支持标准类型和用户自定义类型

use crate::data_unit::GenericDataUnit;
use crate::error::{EncodeResult, ParseError, ParseResult};
use crate::protocol::DataUnitType;
use bytes::{BufMut, Bytes, BytesMut};

// 扩展机制支持

use crate::extension::MacroExtensionManager;

/// 数据单元编解码器
///
/// 用于处理应用数据单元的编解码，支持标准类型和用户自定义类型
#[derive(Debug, Clone, Default)]
pub struct DataUnitCodec {
    /// 是否启用扩展类型支持
    enable_extensions: bool,
}

impl DataUnitCodec {
    /// 创建新的数据单元编解码器
    pub fn new() -> Self {
        DataUnitCodec {
            enable_extensions: false,
        }
    }

    /// 创建支持扩展类型的数据单元编解码器
    pub fn with_extensions() -> Self {
        DataUnitCodec {
            enable_extensions: true,
        }
    }

    /// 编码通用数据单元为字节序列
    ///
    /// # Arguments
    /// * `data_unit` - 通用数据单元
    ///
    /// # Returns
    /// * `Result<Bytes, EncodeError>` - 成功返回字节序列
    pub fn encode_generic(&self, data_unit: &GenericDataUnit) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        // 写入数据单元类型标识符（1字节）
        buf.put_u8(data_unit.data_unit_type().to_u8());

        // 写入数据单元内容
        let content = data_unit.encode()?;
        buf.put_slice(&content);

        Ok(buf.freeze())
    }

    /// 从字节序列解码通用数据单元
    ///
    /// # Arguments
    /// * `data` - 字节序列，第一个字节是类型标识符
    ///
    /// # Returns
    /// * `Result<GenericDataUnit, ParseError>` - 成功返回通用数据单元
    pub fn decode_generic(&self, data: &[u8]) -> ParseResult<GenericDataUnit> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        let data_unit_type = DataUnitType::from_u8(data[0]);
        let content = &data[1..];

        // 检查是否为扩展类型（128-254）
        if self.enable_extensions && data[0] >= 128 && data[0] <= 254 {
            return self.decode_extension_data_unit(data[0], content);
        }

        // 解析标准数据单元
        GenericDataUnit::parse_from_type_and_content(data_unit_type, content)
    }

    /// 解码扩展数据单元
    ///
    /// # Arguments
    /// * `type_code` - 数据单元类型代码（128-254）
    /// * `content` - 数据单元内容字节
    ///
    /// # Returns
    /// * `ParseResult<GenericDataUnit>` - 成功返回通用数据单元包装的扩展
    
    fn decode_extension_data_unit(&self, type_code: u8, content: &[u8]) -> ParseResult<GenericDataUnit> {
        // 从全局注册表查找数据单元扩展
        let manager = MacroExtensionManager::global();
        
        if let Some(extension) = manager.find_data_unit(type_code) {
            // 将扩展数据包装为 GenericDataUnit
            Ok(GenericDataUnit::Extension {
                type_code,
                data: bytes::Bytes::copy_from_slice(content),
                description: extension.description().to_string(),
            })
        } else {
            // 未找到对应的扩展，返回未知数据单元
            Ok(GenericDataUnit::Unknown {
                data_unit_type: DataUnitType::UserDefined(type_code),
                raw_data: content.to_vec(),
            })
        }
    }

    /// 编码扩展数据单元
    ///
    /// # Arguments  
    /// * `type_code` - 数据单元类型代码
    /// * `extension_data` - 扩展数据
    ///
    /// # Returns
    /// * `EncodeResult<Bytes>` - 编码结果
    
    pub fn encode_extension_data_unit(&self, type_code: u8, extension_data: &Bytes) -> EncodeResult<Bytes> {
        let manager = MacroExtensionManager::global();
        
        if let Some(_extension) = manager.find_data_unit(type_code) {
            // 对于扩展类型，直接返回数据（具体编码由扩展实现）
            Ok(extension_data.clone())
        } else {
            Err(crate::error::EncodeError::UnsupportedDataUnit {
                data_unit_type: type_code,
                reason: format!("No extension registered for code {}", type_code),
            })
        }
    }

    /// 批量编码多个数据单元
    pub fn encode_batch(&self, data_units: &[GenericDataUnit]) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();

        for data_unit in data_units {
            let encoded = self.encode_generic(data_unit)?;
            buf.put_slice(&encoded);
        }

        Ok(buf.freeze())
    }

    /// 批量解码多个数据单元
    ///
    /// 从连续的字节流中解析多个数据单元
    pub fn decode_batch(&self, mut data: &[u8]) -> ParseResult<Vec<GenericDataUnit>> {
        let mut results = Vec::new();

        while !data.is_empty() {
            // 先检查是否有足够的数据进行类型识别
            if data.len() < 1 {
                break;
            }

            // 尝试解析一个数据单元
            let data_unit = self.decode_generic(data)?; // 计算已消费的字节数 (类型标识符 + 内容长度)
            let content = data_unit.encode().map_err(|_| ParseError::InvalidValue {
                field: "data_unit_content".to_string(),
                value: "failed to encode".to_string(),
                reason: "encoding error during batch decode".to_string(),
            })?;
            let consumed = 1 + content.len();

            if consumed > data.len() {
                return Err(ParseError::InsufficientData {
                    expected: consumed,
                    actual: data.len(),
                });
            }

            results.push(data_unit);
            data = &data[consumed..];
        }

        Ok(results)
    }

    /// 根据数据单元类型智能解码
    ///
    /// 自动识别数据单元类型并调用相应的解码函数
    pub fn decode_smart(&self, data: &[u8]) -> ParseResult<Box<dyn std::any::Any>> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        // 对于智能解码，我们直接解码为通用数据单元
        let generic = self.decode_generic(data)?;
        Ok(Box::new(generic))
    }

    /// 检查是否支持指定的数据单元类型
    ///
    /// # Arguments
    /// * `type_code` - 数据单元类型代码
    ///
    /// # Returns
    /// * `bool` - 是否支持该类型
    pub fn supports_data_unit_type(&self, type_code: u8) -> bool {
        // 标准数据单元类型（1-127）总是支持的
        if type_code >= 1 && type_code <= 127 {
            return true;
        }

        // 检查是否为扩展类型（128-254）
        if self.enable_extensions && type_code >= 128 && type_code <= 254 {
            let manager = MacroExtensionManager::global();
            return manager.find_data_unit(type_code).is_some();
        }

        false
    }

    /// 获取所有支持的扩展数据单元类型
    ///
    /// # Returns
    /// * `Vec<u8>` - 支持的扩展类型代码列表
    
    pub fn list_extension_data_unit_types(&self) -> Vec<u8> {
        if !self.enable_extensions {
            return Vec::new();
        }

        let manager = MacroExtensionManager::global();
        // 获取数据单元扩展注册表中的所有代码
        if let Ok(registry) = manager.data_unit_extensions.read() {
            registry.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// 获取扩展数据单元的描述信息
    ///
    /// # Arguments
    /// * `type_code` - 数据单元类型代码
    ///
    /// # Returns
    /// * `Option<String>` - 描述信息，如果找不到则返回 None
    
    pub fn get_extension_description(&self, type_code: u8) -> Option<String> {
        if !self.enable_extensions {
            return None;
        }

        let manager = MacroExtensionManager::global();
        manager.find_data_unit(type_code)
            .map(|ext| ext.description().to_string())
    }
}

impl super::traits::Codec<GenericDataUnit> for DataUnitCodec {
    fn encode(&self, data_unit: &GenericDataUnit) -> EncodeResult<Bytes> {
        self.encode_generic(data_unit)
    }

    fn decode(&self, data: &[u8]) -> ParseResult<GenericDataUnit> {
        self.decode_generic(data)
    }
}

mod tests {
    use super::super::traits::Codec;
    use super::*;
    use crate::data_unit::standard::upstream::*;
    use crate::frame::timestamp::Timestamp;
    use crate::info_object::*;
    use crate::protocol::types::*;

    #[test]
    fn test_encode_decode_upload_system_status() {
        let codec = DataUnitCodec::new();

        let system_status = SystemStatus::new(
            SystemType::FireAlarm,
            1,      // system_address
            0x0002, // system_state
            Timestamp::now(),
        );

        let data_unit = GenericDataUnit::UploadSystemStatus(UploadSystemStatus::new(
            system_status,
            Timestamp::now(),
        ));

        let encoded = codec.encode(&data_unit).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        // 验证类型匹配
        assert_eq!(data_unit.data_unit_type(), decoded.data_unit_type());
    }

    #[test]
    fn test_batch_encode_decode() {
        let codec = DataUnitCodec::new();

        let system_status = SystemStatus::new(SystemType::FireAlarm, 1, 0x0002, Timestamp::now());

        let data_units = vec![GenericDataUnit::UploadSystemStatus(
            UploadSystemStatus::new(system_status, Timestamp::now()),
        )];

        let encoded = codec.encode_batch(&data_units).unwrap();
        let decoded = codec.decode_batch(&encoded).unwrap();

        assert_eq!(data_units.len(), decoded.len());
    }

    #[test]
    fn test_smart_decode() {
        let codec = DataUnitCodec::new();

        let system_status = SystemStatus::new(SystemType::FireAlarm, 1, 0x0002, Timestamp::now());

        let data_unit = GenericDataUnit::UploadSystemStatus(UploadSystemStatus::new(
            system_status,
            Timestamp::now(),
        ));

        let encoded = codec.encode(&data_unit).unwrap();
        let decoded = codec.decode_smart(&encoded).unwrap();

        // 验证可以向下转型为GenericDataUnit
        assert!(decoded.downcast_ref::<GenericDataUnit>().is_some());
    }

    #[test]
    fn test_codec_with_extensions() {
        let codec = DataUnitCodec::with_extensions();
        assert!(codec.enable_extensions);

        let codec_normal = DataUnitCodec::new();
        assert!(!codec_normal.enable_extensions);
    }
}
