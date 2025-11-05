//! 简化的通用解码器
//!
//! 提供基础的解码接口，支持从GB26875协议字节流解码为各种数据结构

use super::traits::Decoder as DecoderTrait;
use super::{DataUnitCodec, PacketCodec};
use crate::data_unit::GenericDataUnit;
use crate::error::{ParseError, ParseResult};
use crate::frame::Packet;
use bytes::Buf;

/// 通用解码器
///
/// 提供基础的解码接口，支持常见数据类型的解码
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
}

impl Default for DecoderConfig {
    fn default() -> Self {
        DecoderConfig {
            strict_mode: true,
            enable_extensions: false,
            default_encoding: "UTF-8".to_string(),
            max_buffer_size: 64 * 1024, // 64KB
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
    }
    /// 解码通用数据单元
    pub fn decode_data_unit(&self, data: &[u8]) -> ParseResult<GenericDataUnit> {
        self.data_unit_codec.decode_generic(data)
    }

    /// 解码字符串（支持UTF-8编码）
    pub fn decode_string(&self, data: &[u8]) -> ParseResult<String> {
        std::str::from_utf8(data)
            .map(|s| s.to_string())
            .map_err(|e| ParseError::InvalidEncoding {
                encoding: "UTF-8".to_string(),
                reason: e.to_string(),
            })
    }

    /// 解码32位整数
    pub fn decode_i32(&self, data: &[u8]) -> ParseResult<i32> {
        if data.len() < 4 {
            return Err(ParseError::InsufficientData {
                expected: 4,
                actual: data.len(),
            });
        }
        let mut buf = &data[..];
        Ok(buf.get_i32_le())
    }

    /// 解码64位浮点数
    pub fn decode_f64(&self, data: &[u8]) -> ParseResult<f64> {
        if data.len() < 8 {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }
        let mut buf = &data[..];
        Ok(buf.get_f64_le())
    }
    /// 解码时间戳
    pub fn decode_timestamp(&self, data: &[u8]) -> ParseResult<[u8; 6]> {
        if data.len() != 6 {
            return Err(ParseError::InvalidDataLength {
                expected: 6,
                actual: data.len(),
            });
        }

        let mut timestamp = [0u8; 6];
        timestamp.copy_from_slice(&data[..6]);

        // 严格模式下验证时间戳的合法性
        if self.config.strict_mode {
            let month = timestamp[1];
            let day = timestamp[2];
            let hour = timestamp[3];
            let minute = timestamp[4];
            let second = timestamp[5];

            if month == 0 || month > 12 {
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

    /// 尝试解码数据包（非消费性）
    ///
    /// 检查数据是否包含完整的数据包，但不移动缓冲区指针
    pub fn try_decode_packet(&self, data: &[u8]) -> ParseResult<Option<(Packet, usize)>> {
        match Packet::try_parse(data) {
            Ok((packet, consumed)) => Ok(Some((packet, consumed))),
            Err(ParseError::InsufficientData { .. }) | Err(ParseError::TooShort { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_string_utf8() {
        let decoder = Decoder::new();
        let data = "测试".as_bytes();
        let result = decoder.decode_string(data);
        assert_eq!(result.unwrap(), "测试");
    }

    #[test]
    fn test_decode_i32() {
        let decoder = Decoder::new();
        let data = [42, 0, 0, 0]; // 42 in little-endian
        let result = decoder.decode_i32(&data);
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
}
