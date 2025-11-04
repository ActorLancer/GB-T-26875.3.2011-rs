//! 通用解码器
//!
//! 提供统一的解码接口，支持从GB26875协议字节流解码为各种数据结构

use crate::error::{ParseError, ParseResult};
use crate::frame::Packet;
use crate::data_unit::GenericDataUnit;
use crate::info_object::*;
use crate::protocol::{DataUnitType, CommandCode};
use super::traits::{Codec, Decoder as DecoderTrait};
use super::{PacketCodec, DataUnitCodec};
use bytes::{Bytes, Buf};
use std::collections::HashMap;

// 为了兼容性创建类型别名
pub type Operation = FireSystemOperation;

/// 通用解码器
/// 
/// 提供统一的解码接口，支持多种数据类型的解码
#[derive(Debug, Clone)]
pub struct Decoder {
    /// 数据包编解码器
    packet_codec: PacketCodec,
    /// 数据单元编解码器
    data_unit_codec: DataUnitCodec,
    /// 解码配置
    config: DecoderConfig,
}

/// 解码器配置
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// 是否启用严格模式（严格校验数据格式）
    pub strict_mode: bool,
    /// 是否启用扩展类型支持
    pub enable_extensions: bool,
    /// 默认字符编码
    pub default_encoding: String,
    /// 最大缓冲区大小
    pub max_buffer_size: usize,
    /// 自定义解码器映射
    pub custom_decoders: HashMap<u8, String>,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        DecoderConfig {
            strict_mode: true,
            enable_extensions: false,
            default_encoding: "GB18030".to_string(),
            max_buffer_size: 64 * 1024, // 64KB
            custom_decoders: HashMap::new(),
        }
    }
}

impl Decoder {
    /// 创建新的解码器
    pub fn new() -> Self {
        Decoder {
            packet_codec: PacketCodec::new(),
            data_unit_codec: DataUnitCodec::new(),
            config: DecoderConfig::default(),
        }
    }

    /// 使用指定配置创建解码器
    pub fn with_config(config: DecoderConfig) -> Self {
        let data_unit_codec = if config.enable_extensions {
            DataUnitCodec::with_extensions()
        } else {
            DataUnitCodec::new()
        };

        Decoder {
            packet_codec: PacketCodec::new(),
            data_unit_codec,
            config,
        }
    }

    /// 获取解码器配置
    pub fn config(&self) -> &DecoderConfig {
        &self.config
    }

    /// 更新解码器配置
    pub fn set_config(&mut self, config: DecoderConfig) {
        self.config = config;
        
        // 重新创建数据单元编解码器
        self.data_unit_codec = if self.config.enable_extensions {
            DataUnitCodec::with_extensions()
        } else {
            DataUnitCodec::new()
        };
    }

    /// 解码数据包
    pub fn decode_packet(&self, data: &[u8]) -> ParseResult<Packet> {
        // 检查缓冲区大小限制
        if data.len() > self.config.max_buffer_size {
            return Err(ParseError::BufferTooLarge {
                max_size: self.config.max_buffer_size,
                actual_size: data.len(),
            });
        }

        self.packet_codec.decode(data)
    }    /// 解码通用数据单元
    pub fn decode_data_unit(&self, data: &[u8]) -> ParseResult<GenericDataUnit> {
        self.data_unit_codec.decode(data)
    }

    /// 批量解码多个数据单元
    pub fn decode_batch(&self, data: &[u8]) -> ParseResult<Vec<GenericDataUnit>> {
        self.data_unit_codec.decode_batch(data)
    }

    /// 智能解码 - 根据数据内容自动识别类型
    pub fn decode_smart(&self, data: &[u8]) -> ParseResult<Box<dyn std::any::Any>> {
        self.data_unit_codec.decode_smart(data)
    }

    /// 从JSON格式解码
    #[cfg(feature = "serde")]
    pub fn decode_from_json<T>(&self, json_str: &str) -> ParseResult<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_str(json_str)
            .map_err(|e| ParseError::DeserializationError(e.to_string()))
    }

    /// 从JSON字节流解码
    #[cfg(feature = "serde")]
    pub fn decode_from_json_bytes<T>(&self, data: &[u8]) -> ParseResult<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let json_str = std::str::from_utf8(data)
            .map_err(|e| ParseError::InvalidEncoding {
                encoding: "UTF-8".to_string(),
                reason: e.to_string(),
            })?;
        
        self.decode_from_json(json_str)
    }

    /// 解码字符串（支持不同字符编码）
    pub fn decode_string(&self, data: &[u8], encoding: Option<&str>) -> ParseResult<String> {
        let encoding = encoding.unwrap_or(&self.config.default_encoding);
        
        match encoding {
            "UTF-8" | "utf-8" => {
                std::str::from_utf8(data)
                    .map(|s| s.to_string())
                    .map_err(|e| ParseError::InvalidEncoding {
                        encoding: "UTF-8".to_string(),
                        reason: e.to_string(),
                    })
            }
            "GB18030" | "gb18030" => {
                let (decoded, _, had_errors) = encoding_rs::GB18030.decode(data);
                if had_errors && self.config.strict_mode {
                    Err(ParseError::InvalidEncoding {
                        encoding: "GB18030".to_string(),
                        reason: "数据包含无效的GB18030字符".to_string(),
                    })
                } else {
                    Ok(decoded.to_string())
                }
            }
            _ => Err(ParseError::UnsupportedEncoding {
                encoding: encoding.to_string(),
            }),
        }
    }

    /// 解码数值类型
    pub fn decode_number<T>(&self, data: &[u8]) -> ParseResult<T>
    where
        T: From<i32> + From<f64> + Copy,
    {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 4,
                actual: 0,
            });
        }

        match data.len() {
            4 => {
                if data.len() < 4 {
                    return Err(ParseError::InsufficientData {
                        expected: 4,
                        actual: data.len(),
                    });
                }
                let mut buf = &data[..];
                let value = buf.get_i32_le();
                Ok(T::from(value))
            }
            8 => {
                if data.len() < 8 {
                    return Err(ParseError::InsufficientData {
                        expected: 8,
                        actual: data.len(),
                    });
                }
                let mut buf = &data[..];
                let value = buf.get_f64_le();
                Ok(T::from(value))
            }
            _ => Err(ParseError::InvalidDataLength {
                expected: vec![4, 8],
                actual: data.len(),
            }),
        }
    }

    /// 解码时间戳
    pub fn decode_timestamp(&self, data: &[u8]) -> ParseResult<[u8; 6]> {
        if data.len() != 6 {
            return Err(ParseError::InvalidDataLength {
                expected: vec![6],
                actual: data.len(),
            });
        }

        let mut timestamp = [0u8; 6];
        timestamp.copy_from_slice(&data[..6]);

        // 严格模式下验证时间戳的合法性
        if self.config.strict_mode {
            let year = timestamp[0];
            let month = timestamp[1];
            let day = timestamp[2];
            let hour = timestamp[3];
            let minute = timestamp[4];
            let second = timestamp[5];            if month == 0 || month > 12 {
                return Err(ParseError::InvalidTimestampReason {
                    reason: format!("月份无效: {}", month),
                });
            }

            if day == 0 || day > 31 {
                return Err(ParseError::InvalidTimestampReason {
                    reason: format!("日期无效: {}", day),
                });
            }

            if hour > 23 {
                return Err(ParseError::InvalidTimestampReason {
                    reason: format!("小时无效: {}", hour),
                });
            }

            if minute > 59 {
                return Err(ParseError::InvalidTimestampReason {
                    reason: format!("分钟无效: {}", minute),
                });
            }

            if second > 59 {
                return Err(ParseError::InvalidTimestampReason {
                    reason: format!("秒无效: {}", second),
                });
            }
        }

        Ok(timestamp)
    }

    /// 解码地址信息
    pub fn decode_address(&self, data: &[u8]) -> ParseResult<u16> {
        if data.len() < 2 {
            return Err(ParseError::InsufficientData {
                expected: 2,
                actual: data.len(),
            });
        }

        let mut buf = &data[..];
        Ok(buf.get_u16_le())
    }

    /// 解码布尔值
    pub fn decode_boolean(&self, data: &[u8]) -> ParseResult<bool> {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        match data[0] {
            0 => Ok(false),
            1 => Ok(true),
            other => {
                if self.config.strict_mode {
                    Err(ParseError::InvalidBooleanValue { value: other })
                } else {
                    Ok(other != 0)
                }
            }
        }
    }

    /// 解码枚举值
    pub fn decode_enum<T>(&self, data: &[u8]) -> ParseResult<T>
    where
        T: TryFrom<u8>,
        T::Error: std::fmt::Display,
    {
        if data.is_empty() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        T::try_from(data[0])
            .map_err(|e| ParseError::InvalidEnumValue {
                value: data[0],
                type_name: std::any::type_name::<T>().to_string(),
                reason: e.to_string(),
            })
    }

    /// 尝试解码数据包（非消费性）
    /// 
    /// 检查数据是否包含完整的数据包，但不移动缓冲区指针
    pub fn try_decode_packet(&self, data: &[u8]) -> ParseResult<Option<(Packet, usize)>> {
        match Packet::try_parse(data) {
            Ok((packet, consumed)) => Ok(Some((packet, consumed))),
            Err(ParseError::InsufficientData { .. }) | 
            Err(ParseError::TooShort { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 分析数据内容类型
    pub fn analyze_data_type(&self, data: &[u8]) -> ParseResult<DataAnalysis> {
        if data.is_empty() {
            return Ok(DataAnalysis {
                data_type: AnalyzedDataType::Empty,
                confidence: 1.0,
                details: "数据为空".to_string(),
            });
        }

        // 检查是否为数据包
        if data.len() >= 25 && data[0] == 0x40 && data[1] == 0x40 {
            return Ok(DataAnalysis {
                data_type: AnalyzedDataType::Packet,
                confidence: 0.95,
                details: "检测到数据包启动符".to_string(),
            });
        }

        // 检查是否为数据单元
        if data.len() >= 1 {
            let data_unit_type = DataUnitType::from_u8(data[0]);
            if data_unit_type.is_standard() || (data_unit_type.is_custom() && self.config.enable_extensions) {
                return Ok(DataAnalysis {
                    data_type: AnalyzedDataType::DataUnit,
                    confidence: 0.8,
                    details: format!("检测到数据单元类型: {:?}", data_unit_type),
                });
            }
        }

        // 检查是否为JSON
        if let Ok(_) = std::str::from_utf8(data) {
            if data[0] == b'{' || data[0] == b'[' {
                return Ok(DataAnalysis {
                    data_type: AnalyzedDataType::Json,
                    confidence: 0.7,
                    details: "检测到JSON格式".to_string(),
                });
            }
        }

        Ok(DataAnalysis {
            data_type: AnalyzedDataType::Unknown,
            confidence: 0.0,
            details: "未知数据类型".to_string(),
        })
    }
}

/// 数据分析结果
#[derive(Debug, Clone)]
pub struct DataAnalysis {
    /// 分析出的数据类型
    pub data_type: AnalyzedDataType,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 详细信息
    pub details: String,
}

/// 分析出的数据类型
#[derive(Debug, Clone, PartialEq)]
pub enum AnalyzedDataType {
    /// 空数据
    Empty,
    /// GB26875数据包
    Packet,
    /// 数据单元
    DataUnit,
    /// JSON格式
    Json,
    /// 未知类型
    Unknown,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl DecoderTrait<Packet> for Decoder {
    fn decode(&self, data: &[u8]) -> ParseResult<Packet> {
        self.decode_packet(data)
    }
}

impl DecoderTrait<GenericDataUnit> for Decoder {
    fn decode(&self, data: &[u8]) -> ParseResult<GenericDataUnit> {
        self.decode_data_unit(data)
    }
}

impl DecoderTrait<SystemStatus> for Decoder {
    fn decode(&self, data: &[u8]) -> ParseResult<SystemStatus> {
        self.decode_system_status(data)
    }
}

impl DecoderTrait<ComponentStatus> for Decoder {
    fn decode(&self, data: &[u8]) -> ParseResult<ComponentStatus> {
        self.decode_component_status(data)
    }
}

impl DecoderTrait<AnalogValue> for Decoder {
    fn decode(&self, data: &[u8]) -> ParseResult<AnalogValue> {
        self.decode_analog_value(data)
    }
}

impl DecoderTrait<Operation> for Decoder {
    fn decode(&self, data: &[u8]) -> ParseResult<Operation> {
        self.decode_operation(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::*;

    #[test]
    fn test_decode_string_utf8() {
        let decoder = Decoder::new();
        let data = "测试".as_bytes();
        let result = decoder.decode_string(data, Some("UTF-8"));
        assert_eq!(result.unwrap(), "测试");
    }

    #[test]
    fn test_decode_number_i32() {
        let decoder = Decoder::new();
        let data = [42, 0, 0, 0]; // 42 in little-endian
        let result: ParseResult<i32> = decoder.decode_number(&data);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_decode_timestamp_valid() {
        let decoder = Decoder::new();
        let data = [25, 11, 4, 14, 30, 0]; // 2025-11-04 14:30:00
        let result = decoder.decode_timestamp(&data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_decode_timestamp_invalid() {
        let mut config = DecoderConfig::default();
        config.strict_mode = true;
        let decoder = Decoder::with_config(config);
        
        let data = [25, 13, 4, 14, 30, 0]; // 无效月份
        let result = decoder.decode_timestamp(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_address() {
        let decoder = Decoder::new();
        let data = [0x34, 0x12]; // 0x1234 in little-endian
        let result = decoder.decode_address(&data);
        assert_eq!(result.unwrap(), 0x1234);
    }

    #[test]
    fn test_decode_boolean() {
        let decoder = Decoder::new();
        
        let true_data = [1];
        let result = decoder.decode_boolean(&true_data);
        assert_eq!(result.unwrap(), true);
        
        let false_data = [0];
        let result = decoder.decode_boolean(&false_data);
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_decode_boolean_strict_mode() {
        let mut config = DecoderConfig::default();
        config.strict_mode = true;
        let decoder = Decoder::with_config(config);
        
        let invalid_data = [2]; // 非0非1值
        let result = decoder.decode_boolean(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_config() {
        let mut config = DecoderConfig::default();
        config.strict_mode = false;
        config.enable_extensions = true;
        config.default_encoding = "UTF-8".to_string();
        
        let decoder = Decoder::with_config(config);
        assert!(!decoder.config().strict_mode);
        assert!(decoder.config().enable_extensions);
        assert_eq!(decoder.config().default_encoding, "UTF-8");
    }

    #[test]
    fn test_analyze_data_type_packet() {
        let decoder = Decoder::new();
        let data = [0x40, 0x40]; // 数据包启动符
        let analysis = decoder.analyze_data_type(&data).unwrap();
        assert_eq!(analysis.data_type, AnalyzedDataType::Packet);
        assert!(analysis.confidence > 0.9);
    }

    #[test]
    fn test_analyze_data_type_empty() {
        let decoder = Decoder::new();
        let data = [];
        let analysis = decoder.analyze_data_type(&data).unwrap();
        assert_eq!(analysis.data_type, AnalyzedDataType::Empty);
        assert_eq!(analysis.confidence, 1.0);
    }
}
