//! GB26875 帧边界解析器
//!
//! 提供帧边界检测功能，用于在字节流中识别完整的GB26875数据包

use crate::error::{ParseError, ParseResult};
use crate::protocol::constants::*;
use bytes::{Buf, Bytes, BytesMut};

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

/// 帧检测器状态
///
/// 表示帧边界检测器的当前状态
#[derive(Debug, Clone, PartialEq)]
pub enum DetectorState {
    /// 寻找启动符
    SearchingStart,
    /// 已找到启动符，正在读取数据
    ReadingData {
        /// 期望的数据包总长度
        expected_total_len: Option<usize>,
    },
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
                        self.state = DetectorState::ReadingData {
                            expected_total_len: None,
                        };
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
                    if expected_total_len.is_none() && self.temp_buffer.len() >= MIN_PACKET_SIZE - 2
                    {
                        // 尝试读取控制单元以获取数据单元长度
                        if let Some(total_len) = self.calculate_expected_length()? {
                            self.state = DetectorState::ReadingData {
                                expected_total_len: Some(total_len),
                            };
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
        }

        // 跳过启动符，读取控制单元中的数据单元长度字段
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

mod tests {
    use super::*;
    use crate::frame::{ControlUnit, Packet, Timestamp};
    use crate::protocol::{Command, ProtocolVersion};

    fn create_test_packet_data() -> Vec<u8> {
        let control_unit = ControlUnit::new(
            1,
            ProtocolVersion::new(1, 0),
            Timestamp::now(),
            0x123456,
            0x654321,
            0,
            Command::SendData,
        )
        .unwrap();

        let packet = Packet::empty(control_unit);
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
}
