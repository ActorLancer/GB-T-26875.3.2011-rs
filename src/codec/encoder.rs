//! 通用编码器
//!
//! 提供统一的编码接口，支持将各种数据结构编码为GB26875协议字节流

use super::traits::Encoder as EncoderTrait;
use super::{DataUnitCodec, PacketCodec};
use crate::data_unit::GenericDataUnit;
use crate::error::{EncodeError, EncodeResult};
use crate::frame::Packet;
use bytes::{BufMut, Bytes, BytesMut};
use std::collections::HashMap;

/// 通用编码器
///
/// 提供统一的编码接口，支持多种数据类型的编码
#[derive(Debug, Clone)]
pub struct Encoder {
    /// 数据包编解码器
    packet_codec: PacketCodec,
    /// 数据单元编解码器
    data_unit_codec: DataUnitCodec,
    /// 编码配置
    config: EncoderConfig,
}

/// 编码器配置
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用扩展类型支持
    pub enable_extensions: bool,
    /// 默认字符编码
    pub default_encoding: String,
    /// 自定义编码器映射
    pub custom_encoders: HashMap<u8, String>,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        EncoderConfig {
            enable_compression: false,
            enable_extensions: false,
            default_encoding: "UTF-8".to_string(),
            custom_encoders: HashMap::new(),
        }
    }
}

impl Encoder {
    /// 创建新的编码器
    pub fn new() -> Self {
        Encoder {
            packet_codec: PacketCodec::new(),
            data_unit_codec: DataUnitCodec::new(),
            config: EncoderConfig::default(),
        }
    }

    /// 使用指定配置创建编码器
    pub fn with_config(config: EncoderConfig) -> Self {
        let data_unit_codec = if config.enable_extensions {
            DataUnitCodec::with_extensions()
        } else {
            DataUnitCodec::new()
        };

        Encoder {
            packet_codec: PacketCodec::new(),
            data_unit_codec,
            config,
        }
    }

    /// 获取编码器配置
    pub fn config(&self) -> &EncoderConfig {
        &self.config
    }

    /// 更新编码器配置
    pub fn set_config(&mut self, config: EncoderConfig) {
        self.config = config;

        // 重新创建数据单元编解码器
        self.data_unit_codec = if self.config.enable_extensions {
            DataUnitCodec::with_extensions()
        } else {
            DataUnitCodec::new()
        };
    }
    /// 编码数据包
    pub fn encode_packet(&self, packet: &Packet) -> EncodeResult<Bytes> {
        crate::codec::traits::Encoder::encode(&self.packet_codec, packet)
    }

    /// 编码通用数据单元
    pub fn encode_data_unit(&self, data_unit: &GenericDataUnit) -> EncodeResult<Bytes> {
        self.data_unit_codec.encode_generic(data_unit)
    }

    /// 批量编码多个数据单元
    pub fn encode_batch(&self, data_units: &[GenericDataUnit]) -> EncodeResult<Bytes> {
        self.data_unit_codec.encode_batch(data_units)
    }

    /// 编码字符串（支持不同字符编码）
    pub fn encode_string(&self, s: &str, encoding: Option<&str>) -> EncodeResult<Bytes> {
        let encoding = encoding.unwrap_or(&self.config.default_encoding);

        match encoding {
            "UTF-8" | "utf-8" => Ok(Bytes::from(s.as_bytes().to_vec())),
            _ => Ok(Bytes::from(s.as_bytes().to_vec())), // 简化处理，都用UTF-8
        }
    }

    /// 编码数值类型
    pub fn encode_number<T>(&self, value: T) -> EncodeResult<Bytes>
    where
        T: Into<f64> + Copy,
    {
        let mut buf = BytesMut::new();
        let num_value: f64 = value.into();

        // 根据数值大小选择合适的编码方式
        if num_value == num_value.trunc()
            && num_value >= i32::MIN as f64
            && num_value <= i32::MAX as f64
        {
            // 整数
            buf.put_i32_le(num_value as i32);
        } else {
            // 浮点数
            buf.put_f64_le(num_value);
        }

        Ok(buf.freeze())
    }

    /// 编码时间戳
    pub fn encode_timestamp(&self, timestamp: &[u8; 6]) -> EncodeResult<Bytes> {
        // 简单验证时间戳格式
        if timestamp.len() != 6 {
            return Err(EncodeError::InvalidFormat {
                reason: "时间戳必须为6字节".to_string(),
            });
        }

        Ok(Bytes::from(timestamp.to_vec()))
    }

    /// 编码地址信息
    pub fn encode_address(&self, address: u16) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();
        buf.put_u16_le(address);
        Ok(buf.freeze())
    }

    /// 编码布尔值
    pub fn encode_boolean(&self, value: bool) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();
        buf.put_u8(if value { 1 } else { 0 });
        Ok(buf.freeze())
    }

    /// 编码枚举值
    pub fn encode_enum<T>(&self, value: T) -> EncodeResult<Bytes>
    where
        T: Into<u8>,
    {
        let mut buf = BytesMut::new();
        buf.put_u8(value.into());
        Ok(buf.freeze())
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl EncoderTrait<Packet> for Encoder {
    fn encode(&self, item: &Packet) -> EncodeResult<Bytes> {
        self.encode_packet(item)
    }
}

impl EncoderTrait<GenericDataUnit> for Encoder {
    fn encode(&self, item: &GenericDataUnit) -> EncodeResult<Bytes> {
        self.encode_data_unit(item)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_encode_string_utf8() {
        let encoder = Encoder::new();
        let result = encoder.encode_string("测试", Some("UTF-8"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_encode_number_integer() {
        let encoder = Encoder::new();
        let result = encoder.encode_number(42i32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4); // i32 = 4 bytes
    }

    #[test]
    fn test_encode_number_float() {
        let encoder = Encoder::new();
        let result = encoder.encode_number(42.5f64);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 8); // f64 = 8 bytes
    }

    #[test]
    fn test_encode_timestamp_valid() {
        let encoder = Encoder::new();
        let timestamp = [25, 11, 4, 14, 30, 0]; // 2025-11-04 14:30:00
        let result = encoder.encode_timestamp(&timestamp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encode_address() {
        let encoder = Encoder::new();
        let result = encoder.encode_address(0x1234);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_encode_boolean() {
        let encoder = Encoder::new();

        let true_result = encoder.encode_boolean(true);
        assert!(true_result.is_ok());
        assert_eq!(true_result.unwrap()[0], 1);

        let false_result = encoder.encode_boolean(false);
        assert!(false_result.is_ok());
        assert_eq!(false_result.unwrap()[0], 0);
    }

    #[test]
    fn test_encoder_config() {
        let mut config = EncoderConfig::default();
        config.enable_extensions = true;
        config.default_encoding = "UTF-8".to_string();

        let encoder = Encoder::with_config(config);
        assert!(encoder.config().enable_extensions);
        assert_eq!(encoder.config().default_encoding, "UTF-8");
    }
}
