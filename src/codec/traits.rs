//! GB26875 编解码器 Trait 定义
//!
//! 定义了编解码器的核心抽象接口

use crate::error::{EncodeResult, ParseResult};
use bytes::Bytes;

/// GB26875 编解码器核心 trait
///
/// 所有编解码器都应该实现这个 trait，提供统一的编解码接口
pub trait Codec<T> {
    /// 编码对象为字节序列
    ///
    /// # Arguments
    /// * `item` - 要编码的对象
    ///
    /// # Returns
    /// * `EncodeResult<Bytes>` - 成功返回字节序列
    fn encode(&self, item: &T) -> EncodeResult<Bytes>;

    /// 从字节序列解码对象
    ///
    /// # Arguments
    /// * `data` - 字节序列
    ///
    /// # Returns
    /// * `ParseResult<T>` - 成功返回解码的对象
    fn decode(&self, data: &[u8]) -> ParseResult<T>;
}

/// 编码器 trait
///
/// 专门用于编码操作的 trait
pub trait Encoder<T> {
    /// 编码对象为字节序列
    fn encode(&self, item: &T) -> EncodeResult<Bytes>;
}

/// 解码器 trait
///
/// 专门用于解码操作的 trait
pub trait Decoder<T> {
    /// 从字节序列解码对象
    fn decode(&self, data: &[u8]) -> ParseResult<T>;
}

/// 流式编解码器 trait
///
/// 用于处理流式数据的编解码器
pub trait StreamCodec<T> {
    /// 尝试从缓冲区解码下一个项目
    ///
    /// # Arguments
    /// * `buffer` - 输入缓冲区
    ///
    /// # Returns
    /// * `Ok(Some(T))` - 成功解码一个项目
    /// * `Ok(None)` - 需要更多数据
    /// * `Err(_)` - 解码错误
    fn try_decode(&mut self, buffer: &mut bytes::BytesMut) -> ParseResult<Option<T>>;

    /// 编码项目到缓冲区
    ///
    /// # Arguments
    /// * `item` - 要编码的项目
    /// * `buffer` - 输出缓冲区
    fn encode_to(&self, item: &T, buffer: &mut bytes::BytesMut) -> EncodeResult<()>;
}

/// 为实现了 Encoder 和 Decoder 的类型自动实现 Codec
impl<T, E> Codec<T> for E
where
    E: Encoder<T> + Decoder<T>,
{
    fn encode(&self, item: &T) -> EncodeResult<Bytes> {
        Encoder::encode(self, item)
    }

    fn decode(&self, data: &[u8]) -> ParseResult<T> {
        Decoder::decode(self, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    // 测试编解码器的基本功能
    struct TestCodec;

    impl Encoder<u32> for TestCodec {
        fn encode(&self, item: &u32) -> EncodeResult<Bytes> {
            Ok(Bytes::from(item.to_le_bytes().to_vec()))
        }
    }

    impl Decoder<u32> for TestCodec {
        fn decode(&self, data: &[u8]) -> ParseResult<u32> {
            if data.len() < 4 {
                return Err(crate::error::ParseError::InsufficientData {
                    expected: 4,
                    actual: data.len(),
                });
            }
            let bytes: [u8; 4] = data[0..4].try_into().unwrap();
            Ok(u32::from_le_bytes(bytes))
        }
    }

    #[test]
    fn test_codec_trait() {
        let codec = TestCodec;
        let value = 0x12345678u32;
        // 测试编码
        let encoded = Encoder::encode(&codec, &value).unwrap();
        assert_eq!(encoded.len(), 4);

        // 测试解码
        let decoded = Decoder::decode(&codec, &encoded).unwrap();
        assert_eq!(decoded, value);
    }
}
