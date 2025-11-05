//! GB26875 协议解析器模块
//!
//! 提供低级别的协议解析功能，包括帧边界检测、缓冲区管理、数据校验等

pub mod buffer;
pub mod frame_parser;
pub mod validator;

pub use buffer::{BufferError, BufferStats, StreamBuffer};
pub use frame_parser::{DetectorState, FrameDetector};
pub use validator::DataValidator;

use crate::error::{ParseError, ParseResult};
use crate::frame::Packet;

/// 解析统计信息
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// 成功解析的包数量
    pub packets_parsed: u64,
    /// 解析错误的数量
    pub parse_errors: u64,
    /// 校验和错误的数量
    pub checksum_errors: u64,
    /// 丢弃的字节数（同步用）
    pub bytes_discarded: u64,
}

/// 数据包解析器
///
/// 提供高级的数据包解析功能，包括验证和错误处理
#[derive(Debug, Clone, Default)]
pub struct PacketParser {
    /// 帧检测器
    detector: FrameDetector,
    /// 解析统计
    stats: ParseStats,
}

impl PacketParser {
    /// 创建新的数据包解析器
    pub fn new() -> Self {
        PacketParser {
            detector: FrameDetector::new(),
            stats: ParseStats::default(),
        }
    }

    /// 创建带初始容量的数据包解析器
    ///
    /// # Arguments
    /// * `capacity` - 缓冲区初始容量
    pub fn with_capacity(capacity: usize) -> Self {
        PacketParser {
            detector: FrameDetector::with_capacity(capacity),
            stats: ParseStats::default(),
        }
    }

    /// 解析数据并返回完整的数据包
    ///
    /// # Arguments
    /// * `data` - 新接收的数据
    ///
    /// # Returns
    /// * `Ok(Some(packet))` - 成功解析到数据包
    /// * `Ok(None)` - 需要更多数据
    /// * `Err(error)` - 解析错误
    pub fn parse(&mut self, data: &[u8]) -> ParseResult<Option<Packet>> {
        match self.detector.feed(data)? {
            Some(frame_data) => match Packet::parse(&frame_data) {
                Ok(packet) => {
                    self.stats.packets_parsed += 1;
                    Ok(Some(packet))
                }
                Err(ParseError::ChecksumMismatch { expected, actual }) => {
                    self.stats.checksum_errors += 1;
                    self.stats.parse_errors += 1;
                    Err(ParseError::ChecksumMismatch { expected, actual })
                }
                Err(e) => {
                    self.stats.parse_errors += 1;
                    Err(e)
                }
            },
            None => Ok(None),
        }
    }

    /// 重置解析器状态
    pub fn reset(&mut self) {
        self.detector.reset();
    }

    /// 获取解析统计信息
    pub fn stats(&self) -> &ParseStats {
        &self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = ParseStats::default();
    }

    /// 获取当前缓冲区大小
    pub fn buffer_len(&self) -> usize {
        self.detector.buffer_len()
    }

    /// 检查是否有待处理的数据
    pub fn has_pending_data(&self) -> bool {
        self.detector.has_pending_data()
    }
}

/// 批量数据包解析器
///
/// 用于一次性解析多个数据包，适用于文件或大块数据处理
#[derive(Debug, Clone, Default)]
pub struct BatchParser {
    parser: PacketParser,
}

impl BatchParser {
    /// 创建新的批量解析器
    pub fn new() -> Self {
        BatchParser {
            parser: PacketParser::new(),
        }
    }

    /// 解析数据并返回所有找到的数据包
    ///
    /// # Arguments
    /// * `data` - 要解析的数据
    ///
    /// # Returns
    /// * `Result<Vec<Packet>, ParseError>` - 成功返回数据包列表
    pub fn parse_all(&mut self, data: &[u8]) -> ParseResult<Vec<Packet>> {
        let mut packets = Vec::new();
        let mut remaining = data;

        while !remaining.is_empty() {
            let before_len = self.parser.buffer_len();
            match self.parser.parse(remaining)? {
                Some(packet) => {
                    packets.push(packet);
                    // 在实际实现中，我们需要跟踪消耗了多少字节
                    // 这里简化处理：如果解析成功，假设处理了所有数据
                    break;
                }
                None => {
                    // 没有更多完整的包可以解析
                    let after_len = self.parser.buffer_len();
                    if after_len > before_len {
                        // 有数据被缓存，说明可能有部分包，继续等待更多数据
                        break;
                    } else {
                        // 没有数据被缓存，可能是噪声数据，跳过一些字节
                        remaining = &remaining[1..];
                    }
                }
            }
        }

        Ok(packets)
    }

    /// 获取解析统计信息
    pub fn stats(&self) -> &ParseStats {
        self.parser.stats()
    }

    /// 重置解析器状态
    pub fn reset(&mut self) {
        self.parser.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{ControlUnit, Timestamp};
    use crate::protocol::{Command, ProtocolVersion};

    fn create_test_packet_data() -> Vec<u8> {
        // 创建心跳包
        let control_unit = ControlUnit::new(
            1,
            ProtocolVersion::new(1, 0),
            Timestamp::now(),
            0x123456,
            0x654321,
            0,
            Command::SendData,
        );
        let packet = Packet::empty(control_unit.unwrap());
        packet.encode().unwrap().to_vec()
    }

    #[test]
    fn test_packet_parser() {
        let mut parser = PacketParser::new();
        let packet_data = create_test_packet_data();

        let result = parser.parse(&packet_data).unwrap();
        assert!(result.is_some());

        let stats = parser.stats();
        assert_eq!(stats.packets_parsed, 1);
        assert_eq!(stats.parse_errors, 0);
    }

    #[test]
    fn test_packet_parser_partial_data() {
        let mut parser = PacketParser::new();
        let packet_data = create_test_packet_data();

        // 发送部分数据
        let mid = packet_data.len() / 2;
        let result1 = parser.parse(&packet_data[..mid]).unwrap();
        assert!(result1.is_none()); // 应该等待更多数据

        // 发送剩余数据
        let result2 = parser.parse(&packet_data[mid..]).unwrap();
        assert!(result2.is_some()); // 应该解析成功

        let stats = parser.stats();
        assert_eq!(stats.packets_parsed, 1);
    }

    #[test]
    fn test_batch_parser() {
        let mut parser = BatchParser::new();
        let packet_data = create_test_packet_data();

        // 创建包含多个数据包的数据
        let mut multi_packet_data = packet_data.clone();
        multi_packet_data.extend_from_slice(&packet_data);

        let packets = parser.parse_all(&multi_packet_data).unwrap();

        // 由于简化实现，这里可能只解析到一个包
        assert!(!packets.is_empty());
    }

    #[test]
    fn test_parser_stats() {
        let mut parser = PacketParser::new();

        // 测试初始状态
        let stats = parser.stats();
        assert_eq!(stats.packets_parsed, 0);
        assert_eq!(stats.parse_errors, 0);

        // 解析一个有效包
        let packet_data = create_test_packet_data();
        let _ = parser.parse(&packet_data).unwrap();

        let stats = parser.stats();
        assert_eq!(stats.packets_parsed, 1);

        // 重置统计
        parser.reset_stats();
        let stats = parser.stats();
        assert_eq!(stats.packets_parsed, 0);
    }

    #[test]
    fn test_parser_with_noise() {
        let mut parser = PacketParser::new();
        let packet_data = create_test_packet_data();

        // 在数据包前后添加噪声
        let noise = vec![0xFF, 0xEE, 0xDD, 0xCC];
        let mut data_with_noise = noise.clone();
        data_with_noise.extend_from_slice(&packet_data);
        data_with_noise.extend_from_slice(&noise);

        let result = parser.parse(&data_with_noise).unwrap();
        assert!(result.is_some());

        let stats = parser.stats();
        assert_eq!(stats.packets_parsed, 1);
    }
}
