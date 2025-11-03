//! GB26875 编解码器模块
//!
//! 提供数据包和数据单元的编解码功能，支持 TCP 流处理和异步操作

use crate::error::{ParseError, ParseResult, EncodeError, EncodeResult};
use crate::frame::Packet;
use crate::data_unit::GenericDataUnit;
use crate::protocol::DataUnitType;
use bytes::{Bytes, BytesMut, BufMut};

/// GB26875 编解码器 trait
/// 
/// 定义了编解码器的基本接口，支持将 Rust 对象转换为字节序列，
/// 以及从字节序列解析 Rust 对象。
pub trait Codec<T> {
    /// 编码对象为字节序列
    /// 
    /// # Arguments
    /// * `item` - 要编码的对象
    /// 
    /// # Returns
    /// * `Result<Bytes, EncodeError>` - 成功返回字节序列
    fn encode(&self, item: &T) -> EncodeResult<Bytes>;

    /// 从字节序列解码对象
    /// 
    /// # Arguments
    /// * `data` - 字节序列
    /// 
    /// # Returns
    /// * `Result<T, ParseError>` - 成功返回解码的对象
    fn decode(&self, data: &[u8]) -> ParseResult<T>;
}

/// 数据包编解码器
/// 
/// 用于处理完整的 GB26875 数据包的编解码
#[derive(Debug, Clone, Default)]
pub struct PacketCodec;

impl PacketCodec {
    /// 创建新的数据包编解码器
    pub fn new() -> Self {
        PacketCodec
    }
}

impl Codec<Packet> for PacketCodec {
    fn encode(&self, packet: &Packet) -> EncodeResult<Bytes> {
        packet.encode()
    }

    fn decode(&self, data: &[u8]) -> ParseResult<Packet> {
        Packet::parse(data)
    }
}

/// 数据单元编解码器
/// 
/// 用于处理应用数据单元的编解码，支持标准类型和用户自定义类型
#[derive(Debug, Clone, Default)]
pub struct DataUnitCodec;

impl DataUnitCodec {
    /// 创建新的数据单元编解码器
    pub fn new() -> Self {
        DataUnitCodec
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
    /// * `data` - 包含类型标识符和内容的字节序列
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

        let data_type = DataUnitType::from_u8(data[0]);
        let content = &data[1..];
        
        GenericDataUnit::from_raw(data_type, content)
    }
}

impl Codec<GenericDataUnit> for DataUnitCodec {
    fn encode(&self, data_unit: &GenericDataUnit) -> EncodeResult<Bytes> {
        self.encode_generic(data_unit)
    }

    fn decode(&self, data: &[u8]) -> ParseResult<GenericDataUnit> {
        self.decode_generic(data)
    }
}

/// 流式编解码器
/// 
/// 用于处理 TCP 流中的数据包，能够处理粘包和分包问题
#[derive(Debug)]
pub struct StreamCodec {
    /// 内部缓冲区
    buffer: BytesMut,
    /// 数据包编解码器
    packet_codec: PacketCodec,
}

impl StreamCodec {
    /// 创建新的流式编解码器
    pub fn new() -> Self {
        StreamCodec {
            buffer: BytesMut::new(),
            packet_codec: PacketCodec::new(),
        }
    }

    /// 创建带初始容量的流式编解码器
    /// 
    /// # Arguments
    /// * `capacity` - 初始缓冲区容量
    pub fn with_capacity(capacity: usize) -> Self {
        StreamCodec {
            buffer: BytesMut::with_capacity(capacity),
            packet_codec: PacketCodec::new(),
        }
    }

    /// 向缓冲区添加数据
    /// 
    /// # Arguments
    /// * `data` - 要添加的数据
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// 尝试从缓冲区解析一个完整的数据包
    /// 
    /// # Returns
    /// * `Ok(Some(packet))` - 成功解析到数据包
    /// * `Ok(None)` - 缓冲区数据不足，需要更多数据
    /// * `Err(error)` - 解析错误
    pub fn try_decode(&mut self) -> ParseResult<Option<Packet>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        match Packet::try_parse(&self.buffer) {
            Ok((packet, consumed)) => {
                // 从缓冲区移除已消费的字节
                self.buffer.advance(consumed);
                Ok(Some(packet))
            }
            Err(ParseError::InsufficientData { .. }) => {
                // 数据不足，需要等待更多数据
                Ok(None)
            }
            Err(e) => {
                // 其他解析错误
                Err(e)
            }
        }
    }

    /// 编码数据包为字节序列
    /// 
    /// # Arguments
    /// * `packet` - 要编码的数据包
    /// 
    /// # Returns
    /// * `Result<Bytes, EncodeError>` - 成功返回字节序列
    pub fn encode(&self, packet: &Packet) -> EncodeResult<Bytes> {
        self.packet_codec.encode(packet)
    }

    /// 获取缓冲区大小
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// 清空缓冲区
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// 检查是否有待处理的数据
    pub fn has_pending_data(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// 获取缓冲区容量
    pub fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// 压缩缓冲区（释放未使用的容量）
    pub fn shrink_buffer(&mut self) {
        self.buffer.shrink_to_fit();
    }
}

impl Default for StreamCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "async")]
mod async_codec {
    use super::*;
    use tokio_util::codec::{Decoder, Encoder};
    use bytes::BufMut;

    /// 异步数据包编解码器
    /// 
    /// 实现了 tokio-util 的 Decoder 和 Encoder trait，
    /// 可以与 Framed 结合使用进行异步网络通信
    #[derive(Debug, Clone, Default)]
    pub struct AsyncPacketCodec {
        packet_codec: PacketCodec,
    }

    impl AsyncPacketCodec {
        /// 创建新的异步数据包编解码器
        pub fn new() -> Self {
            AsyncPacketCodec {
                packet_codec: PacketCodec::new(),
            }
        }
    }

    impl Decoder for AsyncPacketCodec {
        type Item = Packet;
        type Error = ParseError;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            if src.is_empty() {
                return Ok(None);
            }

            match Packet::try_parse(src) {
                Ok((packet, consumed)) => {
                    src.advance(consumed);
                    Ok(Some(packet))
                }
                Err(ParseError::InsufficientData { .. }) => {
                    // 需要更多数据
                    Ok(None)
                }
                Err(e) => Err(e),
            }
        }
    }

    impl Encoder<Packet> for AsyncPacketCodec {
        type Error = EncodeError;

        fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
            let encoded = self.packet_codec.encode(&item)?;
            dst.reserve(encoded.len());
            dst.put_slice(&encoded);
            Ok(())
        }
    }

    /// 异步数据单元编解码器
    /// 
    /// 实现了 tokio-util 的 Decoder 和 Encoder trait
    #[derive(Debug, Clone, Default)]
    pub struct AsyncDataUnitCodec {
        data_unit_codec: DataUnitCodec,
    }

    impl AsyncDataUnitCodec {
        /// 创建新的异步数据单元编解码器
        pub fn new() -> Self {
            AsyncDataUnitCodec {
                data_unit_codec: DataUnitCodec::new(),
            }
        }
    }

    impl Decoder for AsyncDataUnitCodec {
        type Item = GenericDataUnit;
        type Error = ParseError;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            if src.len() < 2 {
                // 至少需要类型标识符和一些数据
                return Ok(None);
            }

            // 这里需要更复杂的逻辑来确定数据单元的完整长度
            // 简化版本：假设所有数据都是一个完整的数据单元
            let data_unit = self.data_unit_codec.decode(src)?;
            src.clear(); // 简化处理：清空缓冲区
            Ok(Some(data_unit))
        }
    }

    impl Encoder<GenericDataUnit> for AsyncDataUnitCodec {
        type Error = EncodeError;

        fn encode(&mut self, item: GenericDataUnit, dst: &mut BytesMut) -> Result<(), Self::Error> {
            let encoded = self.data_unit_codec.encode(&item)?;
            dst.reserve(encoded.len());
            dst.put_slice(&encoded);
            Ok(())
        }
    }
}

#[cfg(feature = "async")]
pub use async_codec::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{ControlUnit, Timestamp};
    use crate::protocol::{Command, ProtocolVersion};
    use crate::data_unit::standard::SystemStatus;
    use crate::protocol::SystemType;

    fn create_test_packet() -> Packet {
        let control_unit = ControlUnit::new(
            1,
            ProtocolVersion::v1_0(),
            Timestamp::now(),
            0x123456,
            0x654321,
            0,
            Command::Heartbeat,
        );
        Packet::without_data(control_unit).unwrap()
    }

    #[test]
    fn test_packet_codec() {
        let codec = PacketCodec::new();
        let packet = create_test_packet();
        
        let encoded = codec.encode(&packet).unwrap();
        let decoded = codec.decode(&encoded).unwrap();
        
        assert_eq!(packet, decoded);
    }

    #[test]
    fn test_data_unit_codec() {
        let codec = DataUnitCodec::new();
        let status = SystemStatus::new(SystemType::FireAlarm, 0x123456).unwrap();
        let data_unit = GenericDataUnit::SystemStatus(status);
        
        let encoded = codec.encode(&data_unit).unwrap();
        let decoded = codec.decode(&encoded).unwrap();
        
        assert_eq!(data_unit, decoded);
    }

    #[test]
    fn test_stream_codec() {
        let mut codec = StreamCodec::new();
        let packet = create_test_packet();
        
        // 编码数据包
        let encoded = codec.encode(&packet).unwrap();
        
        // 模拟分批接收数据
        let mid = encoded.len() / 2;
        codec.feed(&encoded[..mid]);
        
        // 第一次解析应该返回 None（数据不足）
        assert!(codec.try_decode().unwrap().is_none());
        
        // 添加剩余数据
        codec.feed(&encoded[mid..]);
        
        // 现在应该能解析出完整的数据包
        let decoded = codec.try_decode().unwrap().unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn test_stream_codec_multiple_packets() {
        let mut codec = StreamCodec::new();
        let packet1 = create_test_packet();
        let packet2 = create_test_packet();
        
        // 编码两个数据包并连接
        let encoded1 = codec.encode(&packet1).unwrap();
        let encoded2 = codec.encode(&packet2).unwrap();
        let mut combined = encoded1.to_vec();
        combined.extend_from_slice(&encoded2);
        
        // 一次性添加所有数据
        codec.feed(&combined);
        
        // 应该能解析出两个数据包
        let decoded1 = codec.try_decode().unwrap().unwrap();
        let decoded2 = codec.try_decode().unwrap().unwrap();
        
        assert_eq!(packet1, decoded1);
        assert_eq!(packet2, decoded2);
        
        // 缓冲区应该为空
        assert!(!codec.has_pending_data());
    }
}
