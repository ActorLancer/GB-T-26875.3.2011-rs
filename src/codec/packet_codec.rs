//! GB26875 数据包编解码器
//!
//! 提供数据包级别的编解码功能，处理完整的GB26875数据包

use crate::codec::traits::{Decoder, Encoder};
use crate::error::{EncodeResult, ParseError, ParseResult};
use crate::frame::Packet;
use crate::protocol::constants::*;
use bytes::{BufMut, Bytes, BytesMut};

/// GB26875 数据包编解码器
///
/// 负责将 Packet 结构体编码为符合GB26875协议的字节流，
/// 以及从字节流解码为 Packet 结构体
#[derive(Debug, Clone, Default)]
pub struct PacketCodec;

impl PacketCodec {
    /// 创建新的数据包编解码器
    pub fn new() -> Self {
        Self
    }

    /// 验证数据包的完整性
    ///
    /// # Arguments
    /// * `data` - 待验证的字节数据
    ///
    /// # Returns
    /// * `ParseResult<()>` - 验证成功或错误
    fn validate_packet_integrity(data: &[u8]) -> ParseResult<()> {
        // 检查最小长度
        if data.len() < MIN_PACKET_SIZE {
            return Err(ParseError::InsufficientData {
                expected: MIN_PACKET_SIZE,
                actual: data.len(),
            });
        }

        // 检查启动符
        if &data[0..2] != &FRAME_START {
            return Err(ParseError::InvalidFrameStart {
                expected: FRAME_START.to_vec(),
                found: [data[0], data[1]].to_vec(),
            });
        }

        // 检查结束符
        let end_pos = data.len() - 2;
        if &data[end_pos..end_pos + 2] != &FRAME_END {
            return Err(ParseError::InvalidFrameEnd {
                expected: FRAME_END.to_vec(),
                found: [data[end_pos], data[end_pos + 1]].to_vec(),
            });
        } // 验证校验和
        let checksum_pos = data.len() - 3;
        // 控制单元从偏移2开始（跳过启动符），长度25
        let control_unit = &data[2..27];
        // 数据单元从偏移27开始到校验和位置结束
        let data_unit = &data[27..checksum_pos];
        let calculated_checksum =
            crate::frame::checksum::calculate_checksum(control_unit, data_unit);
        let packet_checksum = data[checksum_pos];

        if calculated_checksum != packet_checksum {
            return Err(ParseError::ChecksumMismatch {
                expected: calculated_checksum,
                actual: packet_checksum,
            });
        }

        Ok(())
    }

    /// 从原始字节数据中提取控制单元和数据单元
    ///
    /// # Arguments
    /// * `data` - 完整的数据包字节数据
    ///
    /// # Returns
    /// * `ParseResult<(ControlUnit, Option<Bytes>)>` - 解析出的控制单元和数据单元
    fn extract_packet_components(
        data: &[u8],
    ) -> ParseResult<(crate::frame::ControlUnit, Option<Bytes>)> {
        // 跳过启动符，提取控制单元（25字节）
        let control_data = &data[2..2 + CONTROL_UNIT_LENGTH];
        let control_unit = crate::frame::ControlUnit::from_bytes(control_data)?;

        // 提取数据单元（如果存在）
        let data_unit = if control_unit.data_unit_len > 0 {
            let data_start = 2 + CONTROL_UNIT_LENGTH;
            let data_end = data_start + control_unit.data_unit_len as usize;

            if data_end > data.len() - 3 {
                // 减去校验和(1字节) + 结束符(2字节)
                return Err(ParseError::DataUnitLengthMismatch {
                    declared: control_unit.data_unit_len as usize,
                    actual: data.len() - data_start - 3,
                });
            }

            Some(Bytes::copy_from_slice(&data[data_start..data_end]))
        } else {
            None
        };

        Ok((control_unit, data_unit))
    }
}

impl Encoder<Packet> for PacketCodec {
    fn encode(&self, packet: &Packet) -> EncodeResult<Bytes> {
        // 计算总长度
        let data_len = packet.data_unit.as_ref().map(|d| d.len()).unwrap_or(0);
        let total_len = 2 + CONTROL_UNIT_LENGTH + data_len + 1 + 2; // 启动符 + 控制单元 + 数据单元 + 校验和 + 结束符

        let mut buffer = BytesMut::with_capacity(total_len);

        // 写入启动符
        buffer.put_slice(&FRAME_START);
        // 写入控制单元
        let control_bytes = packet.control_unit.to_bytes();
        buffer.put_slice(&control_bytes);

        // 写入数据单元（如果存在）
        if let Some(ref data_unit) = packet.data_unit {
            buffer.put_slice(data_unit);
        }

        // 计算并写入校验和（控制单元 + 数据单元）
        let control_unit = &buffer[2..27]; // 控制单元部分
        let data_unit = &buffer[27..]; // 数据单元部分
        let checksum = crate::frame::checksum::calculate_checksum(control_unit, data_unit);
        buffer.put_u8(checksum);

        // 写入结束符
        buffer.put_slice(&FRAME_END);

        Ok(buffer.freeze())
    }
}

impl Decoder<Packet> for PacketCodec {
    fn decode(&self, data: &[u8]) -> ParseResult<Packet> {
        // 验证数据包完整性
        Self::validate_packet_integrity(data)?;

        // 提取数据包组件
        let (control_unit, data_unit) = Self::extract_packet_components(data)?;

        // 创建数据包
        Packet::new(control_unit, data_unit).map_err(|e| ParseError::InvalidPacket {
            reason: format!("Failed to create packet: {}", e),
        })
    }
}

/// 用于流式处理的数据包编解码器
///
/// 支持从不完整的数据流中提取完整的数据包
#[derive(Debug, Clone, Default)]
pub struct StreamingPacketCodec {
    /// 内部缓冲区
    buffer: BytesMut,
}

impl StreamingPacketCodec {
    /// 创建新的流式数据包编解码器
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    /// 向缓冲区添加数据
    ///
    /// # Arguments
    /// * `data` - 新的字节数据
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// 尝试从缓冲区提取下一个完整的数据包
    ///
    /// # Returns
    /// * `Ok(Some(Packet))` - 成功提取一个数据包
    /// * `Ok(None)` - 没有足够的数据构成完整数据包
    /// * `Err(_)` - 解析错误
    pub fn try_decode_next(&mut self) -> ParseResult<Option<Packet>> {
        loop {
            // 寻找启动符
            let start_pos = self.find_frame_start()?;
            if start_pos.is_none() {
                // 没有找到启动符，清空缓冲区
                self.buffer.clear();
                return Ok(None);
            }

            let start_pos = start_pos.unwrap();

            // 移除启动符之前的垃圾数据
            if start_pos > 0 {
                let _ = self.buffer.split_to(start_pos);
            }

            // 检查是否有足够的数据来确定包长度
            if self.buffer.len() < MIN_PACKET_SIZE {
                return Ok(None);
            }

            // 尝试解析控制单元以获取数据单元长度
            let packet_length = self.calculate_packet_length()?;
            if packet_length.is_none() {
                return Ok(None);
            }

            let packet_length = packet_length.unwrap();

            // 检查是否有完整的数据包
            if self.buffer.len() < packet_length {
                return Ok(None);
            }

            // 提取完整数据包
            let packet_data = self.buffer.split_to(packet_length); // 解码数据包
            let codec = PacketCodec::new();
            match crate::codec::traits::Decoder::decode(&codec, &packet_data) {
                Ok(packet) => return Ok(Some(packet)),
                Err(_) => {
                    // 解码失败，继续寻找下一个启动符
                    continue;
                }
            }
        }
    }

    /// 在缓冲区中寻找帧启动符位置
    fn find_frame_start(&self) -> ParseResult<Option<usize>> {
        for i in 0..self.buffer.len().saturating_sub(1) {
            if self.buffer[i] == FRAME_START[0] && self.buffer[i + 1] == FRAME_START[1] {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// 计算数据包的总长度
    fn calculate_packet_length(&self) -> ParseResult<Option<usize>> {
        if self.buffer.len() < 2 + CONTROL_UNIT_LENGTH {
            return Ok(None);
        }

        // 解析控制单元中的数据单元长度字段
        let data_unit_len_offset = 2 + 2 + 2 + 6 + 6 + 6; // 启动符 + 流水号 + 版本 + 时间戳 + 源地址 + 目的地址
        if self.buffer.len() < data_unit_len_offset + 2 {
            return Ok(None);
        }

        let data_unit_len = u16::from_le_bytes([
            self.buffer[data_unit_len_offset],
            self.buffer[data_unit_len_offset + 1],
        ]) as usize;

        // 计算总包长度：启动符(2) + 控制单元(25) + 数据单元 + 校验和(1) + 结束符(2)
        let total_length = 2 + CONTROL_UNIT_LENGTH + data_unit_len + 1 + 2;

        // 验证长度是否合理
        if total_length > MAX_PACKET_SIZE {
            return Err(ParseError::DataUnitTooLarge {
                size: data_unit_len,
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        Ok(Some(total_length))
    }

    /// 清空内部缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 获取缓冲区当前大小
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

mod tests {
    use super::*;
    use crate::frame::{ControlUnit, Timestamp};
    use crate::protocol::{Command, ProtocolVersion};

    #[test]
    fn test_packet_encode_decode() {
        let timestamp = Timestamp::new(30, 15, 10, 1, 11, 24).unwrap();
        let control_unit = ControlUnit::new(
            0x1234,
            ProtocolVersion::new(1, 1),
            timestamp,
            0x123456789ABC,
            0xDEF123456789,
            5,
            Command::SendData,
        )
        .unwrap();

        let data_unit = Some(Bytes::from_static(b"hello"));
        let packet = Packet::new(control_unit, data_unit).unwrap();

        let codec = PacketCodec::new(); // 测试编码
        let encoded = crate::codec::traits::Encoder::encode(&codec, &packet).unwrap();
        assert!(encoded.len() > MIN_PACKET_SIZE);

        // 测试解码
        let decoded = crate::codec::traits::Decoder::decode(&codec, &encoded).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn test_streaming_codec() {
        let timestamp = Timestamp::new(30, 15, 10, 1, 11, 24).unwrap();
        let control_unit = ControlUnit::new(
            0x1234,
            ProtocolVersion::new(1, 1),
            timestamp,
            0x123456,
            0x654321,
            0,
            Command::SendData,
        )
        .unwrap();
        let packet = Packet::new(control_unit, None).unwrap();
        let codec = PacketCodec::new();
        let encoded = crate::codec::traits::Encoder::encode(&codec, &packet).unwrap();

        let mut streaming_codec = StreamingPacketCodec::new();

        // 分块喂入数据
        let mid = encoded.len() / 2;
        streaming_codec.feed(&encoded[0..mid]);
        assert!(streaming_codec.try_decode_next().unwrap().is_none());

        streaming_codec.feed(&encoded[mid..]);
        let decoded = streaming_codec.try_decode_next().unwrap();
        assert!(decoded.is_some());
        assert_eq!(packet, decoded.unwrap());
    }
}
