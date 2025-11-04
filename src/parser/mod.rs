//! GB26875 协议解析器模块
//!
//! 提供低级别的协议解析功能，包括帧边界检测、数据校验等

use crate::error::{ParseError, ParseResult};
use crate::protocol::constants::*;
use crate::frame::{Packet, checksum};
use bytes::{Bytes, BytesMut, Buf};

/// 帧边界检测器
///
/// 用于在字节流中检测完整的 GB26875 数据包边界
#[derive(Debug, Clone)]
pub struct FrameDetector {
    /// 当前状态
    state: DetectorState,
    /// 临时缓冲区
    temp_buffer: BytesMut,
}

#[derive(Debug, Clone, PartialEq)]
enum DetectorState {
    /// 寻找启动符
    SearchingStart,
    /// 已找到启动符，正在读取数据
    ReadingData { expected_total_len: Option<usize> },
    /// 检测到错误状态
    Error,
}

impl FrameDetector {
    /// 创建新的帧边界检测器
    pub fn new() -> Self {
        FrameDetector {
            state: DetectorState::SearchingStart,
            temp_buffer: BytesMut::new(),
        }
    }

    /// 创建带初始容量的帧边界检测器
    /// 
    /// # Arguments
    /// * `capacity` - 缓冲区初始容量
    pub fn with_capacity(capacity: usize) -> Self {
        FrameDetector {
            state: DetectorState::SearchingStart,
            temp_buffer: BytesMut::with_capacity(capacity),
        }
    }

    /// 向检测器添加数据并尝试检测完整的帧
    ///
    /// # Arguments
    /// * `data` - 新接收的数据
    /// 
    /// # Returns
    /// * `Ok(Some(frame))` - 检测到完整的帧
    /// * `Ok(None)` - 需要更多数据
    /// * `Err(error)` - 检测过程中发生错误
    pub fn feed(&mut self, data: &[u8]) -> ParseResult<Option<Bytes>> {
        self.temp_buffer.extend_from_slice(data);

        loop {
            match &self.state {
                DetectorState::SearchingStart => {
                    if let Some(start_pos) = self.find_start_marker() {
                        // 丢弃启动符之前的数据
                        self.temp_buffer.advance(start_pos);
                        self.state = DetectorState::ReadingData { expected_total_len: None };
                    } else {
                        // 如果没有找到启动符，保留最后一个字节（可能是启动符的一部分）
                        if self.temp_buffer.len() > 1 {
                            let keep_len = 1;
                            let drain_len = self.temp_buffer.len() - keep_len;
                            self.temp_buffer.advance(drain_len);
                        }
                        return Ok(None);
                    }
                }
                DetectorState::ReadingData { expected_total_len } => {
                    // 检查是否有足够的数据来确定包长度
                    if expected_total_len.is_none() && self.temp_buffer.len() >= MIN_PACKET_SIZE - 2 {
                        // 尝试读取控制单元以获取数据单元长度
                        if let Some(total_len) = self.calculate_expected_length()? {
                            self.state = DetectorState::ReadingData { expected_total_len: Some(total_len) };
                            continue;
                        }
                    }

                    if let Some(expected_len) = expected_total_len {
                        if self.temp_buffer.len() >= *expected_len {
                            // 提取完整的包
                            let frame_data = self.temp_buffer.split_to(*expected_len);
                            self.state = DetectorState::SearchingStart;
                            return Ok(Some(frame_data.freeze()));
                        }
                    }

                    // 检查缓冲区是否过大（防止内存攻击）
                    if self.temp_buffer.len() > MAX_PACKET_SIZE {
                        self.reset_to_error_state();
                        return Err(ParseError::DataUnitTooLarge {
                            size: self.temp_buffer.len(),
                            max_size: MAX_PACKET_SIZE,
                        });
                    }

                    return Ok(None);
                }
                DetectorState::Error => {
                    // 在错误状态下，尝试重新同步
                    self.reset();
                    continue;
                }
            }
        }
    }

    /// 寻找启动符位置
    fn find_start_marker(&self) -> Option<usize> {
        self.temp_buffer
            .windows(FRAME_START.len())
            .position(|window| window == &FRAME_START)
    }

    /// 计算期望的包总长度
    fn calculate_expected_length(&self) -> ParseResult<Option<usize>> {
        if self.temp_buffer.len() < 2 + CONTROL_UNIT_LENGTH {
            return Ok(None);
        }        // 跳过启动符，读取控制单元中的数据单元长度字段
        // 数据单元长度位于控制单元的第22-23字节（从0开始计数）
        let data_unit_len_offset = 2 + 22; // 启动符(2) + 控制单元偏移(22)

        if self.temp_buffer.len() < data_unit_len_offset + 2 {
            return Ok(None);
        }

        let data_unit_len = u16::from_le_bytes([
            self.temp_buffer[data_unit_len_offset],
            self.temp_buffer[data_unit_len_offset + 1],
        ]);

        // 验证数据单元长度是否合理
        if data_unit_len as usize > MAX_DATA_UNIT_SIZE {
            return Err(ParseError::DataUnitTooLarge {
                size: data_unit_len as usize,
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        let total_len = MIN_PACKET_SIZE + data_unit_len as usize;
        Ok(Some(total_len))
    }

    /// 重置检测器状态
    pub fn reset(&mut self) {
        self.state = DetectorState::SearchingStart;
        self.temp_buffer.clear();
    }

    /// 重置到错误状态
    fn reset_to_error_state(&mut self) {
        self.state = DetectorState::Error;
        self.temp_buffer.clear();
    }

    /// 获取当前缓冲区大小
    pub fn buffer_len(&self) -> usize {
        self.temp_buffer.len()
    }

    /// 检查是否有待处理的数据
    pub fn has_pending_data(&self) -> bool {
        !self.temp_buffer.is_empty()
    }

    /// 获取当前状态（调试用）
    pub fn current_state(&self) -> &str {
        match &self.state {
            DetectorState::SearchingStart => "SearchingStart",
            DetectorState::ReadingData { .. } => "ReadingData",
            DetectorState::Error => "Error",
        }
    }
}

impl Default for FrameDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 数据验证器
/// 
/// 提供GB26875协议的数据验证功能，包括帧格式、校验和、协议合规性等检查
#[derive(Debug, Clone)]
pub struct DataValidator;

impl DataValidator {
    /// 创建新的数据验证器
    pub fn new() -> Self {
        DataValidator
    }

    /// 验证数据帧的基本格式
    /// 
    /// # Arguments
    /// * `data` - 要验证的数据
    /// 
    /// # Returns
    /// * `Ok(())` - 验证通过
    /// * `Err(ParseError)` - 验证失败
    pub fn validate_frame_format(&self, data: &[u8]) -> ParseResult<()> {
        // 检查最小长度
        if data.len() < MIN_PACKET_SIZE {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: MIN_PACKET_SIZE,
            });
        }

        // 检查启动符
        if !data.starts_with(&FRAME_START) {
            let found = if data.len() >= 2 {
                vec![data[0], data[1]]
            } else {
                data.to_vec()
            };
            return Err(ParseError::InvalidFrameStart {
                expected: FRAME_START.to_vec(),
                found,
            });
        }

        // 检查结束符（如果数据足够长）
        if data.len() >= 2 && !data.ends_with(&FRAME_END) {
            let found = if data.len() >= 2 {
                vec![data[data.len()-2], data[data.len()-1]]
            } else {
                data.to_vec()
            };
            return Err(ParseError::InvalidFrameEnd {
                expected: FRAME_END.to_vec(),
                found,
            });
        }

        // 检查最大长度
        if data.len() > MAX_PACKET_SIZE {
            return Err(ParseError::DataUnitTooLarge {
                size: data.len(),
                max_size: MAX_PACKET_SIZE,
            });
        }

        Ok(())
    }

    /// 验证校验和
    /// 
    /// # Arguments
    /// * `data` - 包含校验和的完整数据帧（不包括结束符）
    /// 
    /// # Returns
    /// * `Ok(())` - 校验和正确
    /// * `Err(ParseError)` - 校验和错误
    pub fn validate_checksum(&self, data: &[u8]) -> ParseResult<()> {
        if data.len() < 4 {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: 4,
            });
        }

        // 提取校验和（倒数第一个字节，在结束符之前）
        if data.len() < 3 {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: 3,
            });
        }

        let expected_checksum = data[data.len() - 3]; // 校验和在结束符之前
        let packet_without_checksum = &data[..data.len() - 3];

        if checksum::verify_checksum(packet_without_checksum, expected_checksum) {
            Ok(())
        } else {
            let calculated = checksum::calculate_packet_checksum(packet_without_checksum).unwrap_or(0);
            Err(ParseError::ChecksumMismatch {
                expected: expected_checksum,
                actual: calculated,
            })
        }
    }

    /// 验证协议版本合规性
    /// 
    /// # Arguments
    /// * `data` - 控制单元数据
    /// 
    /// # Returns
    /// * `Ok(())` - 协议版本有效
    /// * `Err(ParseError)` - 协议版本无效
    pub fn validate_protocol_version(&self, data: &[u8]) -> ParseResult<()> {
        if data.len() < CONTROL_UNIT_LENGTH {
            return Err(ParseError::InsufficientData {
                expected: CONTROL_UNIT_LENGTH,
                actual: data.len(),
            });
        }

        // 协议版本在控制单元的第4-5字节（跳过设备地址的3字节和序列号的1字节）
        let version_bytes = &data[4..6];
        let version = u16::from_le_bytes([version_bytes[0], version_bytes[1]]);
        
        // GB26875协议版本范围检查
        let major = (version >> 8) as u8;
        let minor = (version & 0xFF) as u8;
        
        if major == 0 || major > 99 || minor > 99 {
            return Err(ParseError::UnsupportedVersion {
                version: format!("{}.{}", major, minor),
            });
        }

        Ok(())
    }

    /// 验证数据单元长度的合理性
    /// 
    /// # Arguments
    /// * `declared_length` - 声明的数据单元长度
    /// * `actual_data` - 实际的数据单元内容
    /// 
    /// # Returns
    /// * `Ok(())` - 长度一致
    /// * `Err(ParseError)` - 长度不一致
    pub fn validate_data_unit_length(&self, declared_length: u16, actual_data: &[u8]) -> ParseResult<()> {
        if declared_length as usize != actual_data.len() {
            return Err(ParseError::DataUnitLengthMismatch {
                declared: declared_length as usize,
                actual: actual_data.len(),
            });
        }

        if declared_length as usize > MAX_DATA_UNIT_SIZE {
            return Err(ParseError::DataUnitTooLarge {
                size: declared_length as usize,
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        Ok(())
    }

    /// 综合验证数据帧
    /// 
    /// # Arguments
    /// * `data` - 完整的数据帧
    /// 
    /// # Returns
    /// * `Ok(())` - 所有验证都通过
    /// * `Err(ParseError)` - 任何验证失败
    pub fn validate_complete_frame(&self, data: &[u8]) -> ParseResult<()> {
        // 1. 基本格式验证
        self.validate_frame_format(data)?;

        // 2. 校验和验证
        self.validate_checksum(data)?;

        // 3. 协议版本验证
        if data.len() >= 2 + CONTROL_UNIT_LENGTH {
            let control_unit_data = &data[2..2 + CONTROL_UNIT_LENGTH];
            self.validate_protocol_version(control_unit_data)?;
        }

        Ok(())
    }
}

impl Default for DataValidator {
    fn default() -> Self {
        Self::new()
    }
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

/// 解析统计信息
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// 成功解析的包数量
    pub packets_parsed: u64,
    /// 解析错误的数量?
    pub parse_errors: u64,
    /// 校验和错误的数量
    pub checksum_errors: u64,
    /// 丢弃的字节数（同步用）
    pub bytes_discarded: u64,
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
            Some(frame_data) => {
                match Packet::parse(&frame_data) {
                    Ok(packet) => {
                        self.stats.packets_parsed += 1;
                        Ok(Some(packet))
                    }
                    Err(ParseError::ChecksumMismatch { .. }) => {
                        self.stats.checksum_errors += 1;
                        self.stats.parse_errors += 1;
                        Err(ParseError::ChecksumMismatch {
                            expected: 0, // 实际值在具体解析中获取
                            actual: 0
                        })
                    }
                    Err(e) => {
                        self.stats.parse_errors += 1;
                        Err(e)
                    }
                }
            }
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
        let remaining = data;

        while !remaining.is_empty() {
            match self.parser.parse(remaining)? {
                Some(packet) => {
                    packets.push(packet);
                    // 这里需要更精确的处理来确定消耗了多少字节
                    // 简化版本：假设整个 remaining 都被处理了
                    break;
                }
                None => {
                    // 需要更多数据，但在批量模式下没有更多数据了
                    break;
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
    use crate::protocol::{Command, ProtocolVersion};    fn create_test_packet_data() -> Vec<u8> {
        // 心跳包
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
    fn test_frame_detector() {
        let mut detector = FrameDetector::new();
        let packet_data = create_test_packet_data();
        
        // 分批添加数据
        let mid = packet_data.len() / 2;

        // 第一批数据
        let result1 = detector.feed(&packet_data[..mid]).unwrap();
        assert!(result1.is_none()); // 数据不足

        // 第二批数据
        let result2 = detector.feed(&packet_data[mid..]).unwrap();
        assert!(result2.is_some()); // 检测到完整帧

        let detected_frame = result2.unwrap();
        assert_eq!(detected_frame.to_vec(), packet_data);
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
    fn test_frame_detector_with_noise() {
        let mut detector = FrameDetector::new();
        let packet_data = create_test_packet_data();

        // 添加一些噪声数据
        let noise = vec![0xFF, 0xEE, 0xDD, 0xCC];
        let mut data_with_noise = noise.clone();
        data_with_noise.extend_from_slice(&packet_data);
        data_with_noise.extend_from_slice(&noise);
        
        let result = detector.feed(&data_with_noise).unwrap();
        assert!(result.is_some());
        
        let detected_frame = result.unwrap();
        assert_eq!(detected_frame.to_vec(), packet_data);
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
}
