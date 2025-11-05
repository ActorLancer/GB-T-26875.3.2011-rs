//! Tokio Framed Codec 集成
//!
//! 为GB26875协议提供tokio-util的Framed支持，用于异步网络通信

use crate::error::{EncodeError, EncodeResult, ParseError, ParseResult};
use crate::frame::Packet;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;

#[cfg(feature = "async")]
use tokio_util::codec::{Decoder, Encoder};

/// GB26875 Framed Codec
///
/// 实现 tokio-util 的 Codec trait，用于在异步流中处理 GB26875 数据包
#[derive(Debug, Clone)]
pub struct GB26875FramedCodec {
    /// 最大帧长度限制
    max_frame_length: usize,
    /// 是否启用长度校验
    validate_length: bool,
}

impl GB26875FramedCodec {
    /// 创建新的 Framed Codec
    pub fn new() -> Self {
        GB26875FramedCodec {
            max_frame_length: 8192, // 8KB 默认最大帧长度
            validate_length: true,
        }
    }

    /// 创建带自定义最大长度的 Framed Codec
    pub fn with_max_length(max_length: usize) -> Self {
        GB26875FramedCodec {
            max_frame_length: max_length,
            validate_length: true,
        }
    }

    /// 设置长度校验
    pub fn set_validate_length(&mut self, validate: bool) {
        self.validate_length = validate;
    }

    /// 获取最大帧长度
    pub fn max_frame_length(&self) -> usize {
        self.max_frame_length
    }

    /// 尝试从缓冲区中找到完整的数据包
    fn find_frame(&self, buf: &BytesMut) -> ParseResult<Option<usize>> {
        if buf.len() < 25 {
            // 最小数据包长度检查
            return Ok(None);
        }

        // 检查启动符
        if buf[0] != 0x40 || buf[1] != 0x40 {
            return Err(ParseError::InvalidStartMarker {
                expected: [0x40, 0x40],
                actual: [buf[0], buf[1]],
            });
        }

        // 解析数据长度字段（控制单元第23-24字节）
        if buf.len() < 25 {
            return Ok(None);
        }

        let data_length = u16::from_le_bytes([buf[22], buf[23]]) as usize;

        // 计算完整数据包长度：控制单元(25字节) + 应用数据单元(data_length字节) + 校验和(1字节) + 结束符(1字节)
        let total_length = 25 + data_length + 1 + 1;

        // 检查最大长度限制
        if self.validate_length && total_length > self.max_frame_length {
            return Err(ParseError::FrameTooLarge {
                max_size: self.max_frame_length,
                actual_size: total_length,
            });
        }

        // 检查是否有足够的数据
        if buf.len() < total_length {
            return Ok(None);
        }

        // 检查结束符
        let end_marker_pos = total_length - 1;
        if buf[end_marker_pos] != 0x23 {
            return Err(ParseError::InvalidEndMarker {
                expected: 0x23,
                actual: buf[end_marker_pos],
            });
        }

        Ok(Some(total_length))
    }
}

impl Default for GB26875FramedCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "async")]
impl Decoder for GB26875FramedCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 查找完整的帧
        match self.find_frame(src) {
            Ok(Some(frame_length)) => {
                // 提取完整的帧数据
                let frame_data = src.split_to(frame_length);

                // 解析数据包
                match Packet::parse(&frame_data) {
                    Ok(packet) => Ok(Some(packet)),
                    Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                }
            }
            Ok(None) => {
                // 需要更多数据
                Ok(None)
            }
            Err(e) => {
                // 解析错误，清空缓冲区并返回错误
                src.clear();
                Err(io::Error::new(io::ErrorKind::InvalidData, e))
            }
        }
    }
}

#[cfg(feature = "async")]
impl Encoder<Packet> for GB26875FramedCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item.encode() {
            Ok(encoded) => {
                // 检查编码后的长度
                if self.validate_length && encoded.len() > self.max_frame_length {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("数据包过大: {} > {}", encoded.len(), self.max_frame_length),
                    ));
                }

                dst.reserve(encoded.len());
                dst.put_slice(&encoded);
                Ok(())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}

/// 长度字段编解码器
///
/// 使用数据包中的长度字段来确定帧边界，提供更高效的帧检测
#[derive(Debug, Clone)]
pub struct LengthFieldCodec {
    /// 长度字段偏移量（从包开始的字节数）
    length_field_offset: usize,
    /// 长度字段长度（字节数）
    length_field_length: usize,
    /// 长度调整值（长度字段值需要加上的值来得到实际帧长度）
    length_adjustment: isize,
    /// 最大帧长度
    max_frame_length: usize,
}

impl LengthFieldCodec {
    /// 创建GB26875专用的长度字段编解码器
    pub fn new_gb26875() -> Self {
        LengthFieldCodec {
            length_field_offset: 22, // 长度字段在第22-23字节
            length_field_length: 2,  // 长度字段为2字节
            length_adjustment: 27,   // 控制单元(25) + 校验和(1) + 结束符(1) = 27
            max_frame_length: 8192,
        }
    }

    /// 创建自定义配置的长度字段编解码器
    pub fn new(
        length_field_offset: usize,
        length_field_length: usize,
        length_adjustment: isize,
        max_frame_length: usize,
    ) -> Self {
        LengthFieldCodec {
            length_field_offset,
            length_field_length,
            length_adjustment,
            max_frame_length,
        }
    }

    /// 尝试解码帧长度
    fn decode_frame_length(&self, buf: &BytesMut) -> ParseResult<Option<usize>> {
        let required_bytes = self.length_field_offset + self.length_field_length;

        if buf.len() < required_bytes {
            return Ok(None);
        }

        let length_bytes =
            &buf[self.length_field_offset..self.length_field_offset + self.length_field_length];

        let field_value = match self.length_field_length {
            1 => length_bytes[0] as u64,
            2 => u16::from_le_bytes([length_bytes[0], length_bytes[1]]) as u64,
            4 => u32::from_le_bytes([
                length_bytes[0],
                length_bytes[1],
                length_bytes[2],
                length_bytes[3],
            ]) as u64,
            8 => u64::from_le_bytes([
                length_bytes[0],
                length_bytes[1],
                length_bytes[2],
                length_bytes[3],
                length_bytes[4],
                length_bytes[5],
                length_bytes[6],
                length_bytes[7],
            ]),
            _ => {
                return Err(ParseError::InvalidLengthFieldSize {
                    size: self.length_field_length,
                })
            }
        };

        let frame_length = (field_value as isize + self.length_adjustment) as usize;

        if frame_length > self.max_frame_length {
            return Err(ParseError::FrameTooLarge {
                max_size: self.max_frame_length,
                actual_size: frame_length,
            });
        }

        Ok(Some(frame_length))
    }
}

#[cfg(feature = "async")]
impl Decoder for LengthFieldCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.decode_frame_length(src) {
            Ok(Some(frame_length)) => {
                if src.len() < frame_length {
                    // 需要更多数据
                    Ok(None)
                } else {
                    // 提取完整帧
                    let frame = src.split_to(frame_length);
                    Ok(Some(frame.freeze()))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                src.clear();
                Err(io::Error::new(io::ErrorKind::InvalidData, e))
            }
        }
    }
}

/// 流式数据包处理器
///
/// 提供流式处理多个数据包的功能，处理TCP流中的粘包问题
#[derive(Debug)]
pub struct StreamProcessor {
    /// 内部缓冲区
    buffer: BytesMut,
    /// Framed编解码器
    codec: GB26875FramedCodec,
    /// 处理的数据包数量
    packet_count: usize,
    /// 处理的字节数
    bytes_processed: usize,
}

impl StreamProcessor {
    /// 创建新的流处理器
    pub fn new() -> Self {
        StreamProcessor {
            buffer: BytesMut::new(),
            codec: GB26875FramedCodec::new(),
            packet_count: 0,
            bytes_processed: 0,
        }
    }

    /// 创建带自定义容量的流处理器
    pub fn with_capacity(capacity: usize) -> Self {
        StreamProcessor {
            buffer: BytesMut::with_capacity(capacity),
            codec: GB26875FramedCodec::new(),
            packet_count: 0,
            bytes_processed: 0,
        }
    }

    /// 添加数据到缓冲区
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        self.bytes_processed += data.len();
    }

    /// 尝试解析一个数据包
    pub fn try_decode(&mut self) -> ParseResult<Option<Packet>> {
        match self.codec.decode(&mut self.buffer) {
            Ok(Some(packet)) => {
                self.packet_count += 1;
                Ok(Some(packet))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ParseError::IoError(e.to_string())),
        }
    }

    /// 解析所有可用的数据包
    pub fn decode_all(&mut self) -> ParseResult<Vec<Packet>> {
        let mut packets = Vec::new();

        while let Some(packet) = self.try_decode()? {
            packets.push(packet);
        }

        Ok(packets)
    }

    /// 编码数据包到缓冲区
    pub fn encode(&mut self, packet: &Packet) -> EncodeResult<()> {
        match packet.encode() {
            Ok(encoded) => {
                self.buffer.extend_from_slice(&encoded);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// 获取缓冲区中的数据
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 获取统计信息
    pub fn stats(&self) -> StreamStats {
        StreamStats {
            packet_count: self.packet_count,
            bytes_processed: self.bytes_processed,
            buffer_size: self.buffer.len(),
        }
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.packet_count = 0;
        self.bytes_processed = 0;
    }
}

impl Default for StreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 流处理统计信息
#[derive(Debug, Clone)]
pub struct StreamStats {
    /// 处理的数据包数量
    pub packet_count: usize,
    /// 处理的字节数
    pub bytes_processed: usize,
    /// 当前缓冲区大小
    pub buffer_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Header, TimeStamp};
    use crate::protocol::{constants::*, CommandCode};

    #[test]
    fn test_gb26875_framed_codec_creation() {
        let codec = GB26875FramedCodec::new();
        assert_eq!(codec.max_frame_length(), 8192);

        let codec = GB26875FramedCodec::with_max_length(4096);
        assert_eq!(codec.max_frame_length(), 4096);
    }

    #[test]
    fn test_length_field_codec_creation() {
        let codec = LengthFieldCodec::new_gb26875();
        assert_eq!(codec.length_field_offset, 22);
        assert_eq!(codec.length_field_length, 2);
        assert_eq!(codec.length_adjustment, 27);
    }

    #[test]
    fn test_stream_processor_creation() {
        let processor = StreamProcessor::new();
        assert_eq!(processor.buffer().len(), 0);
        assert_eq!(processor.stats().packet_count, 0);

        let processor = StreamProcessor::with_capacity(1024);
        assert!(processor.buffer.capacity() >= 1024);
    }

    #[test]
    fn test_stream_processor_feed() {
        let mut processor = StreamProcessor::new();
        let data = [0x40, 0x40, 0x01, 0x02];

        processor.feed(&data);
        assert_eq!(processor.buffer().len(), 4);
        assert_eq!(processor.stats().bytes_processed, 4);
    }

    #[test]
    fn test_stream_processor_clear() {
        let mut processor = StreamProcessor::new();
        processor.feed(&[1, 2, 3, 4]);

        processor.clear();
        assert_eq!(processor.buffer().len(), 0);
    }

    #[test]
    fn test_stream_processor_stats() {
        let mut processor = StreamProcessor::new();
        processor.feed(&[1, 2, 3]);
        processor.feed(&[4, 5]);

        let stats = processor.stats();
        assert_eq!(stats.bytes_processed, 5);
        assert_eq!(stats.buffer_size, 5);

        processor.reset_stats();
        let stats = processor.stats();
        assert_eq!(stats.packet_count, 0);
        assert_eq!(stats.bytes_processed, 0);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_framed_codec_encode() {
        use tokio_util::codec::Encoder;

        let mut codec = GB26875FramedCodec::new();
        let mut dst = BytesMut::new();

        // 创建一个测试数据包
        let header = Header {
            start_marker: START_MARKER,
            version: PROTOCOL_VERSION,
            timestamp: TimeStamp([25, 11, 4, 14, 30, 0]),
            source_address: 0x1234,
            destination_address: 0x5678,
            application_data_unit_length: 0,
            command_code: CommandCode::ConfirmTestCommand,
            serial_number: 1,
        };

        let packet = Packet {
            header,
            application_data_unit: vec![],
            checksum: 0,
            end_marker: END_MARKER,
        };

        let result = codec.encode(packet, &mut dst);
        assert!(result.is_ok());
        assert!(!dst.is_empty());
    }
}
