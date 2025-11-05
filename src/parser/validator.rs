//! GB26875 协议数据验证器模块
//!
//! 提供数据验证功能，包括帧格式、校验和、协议合规性等检查

use crate::error::{ParseError, ParseResult};
use crate::frame::checksum;
use crate::protocol::constants::*;

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
                vec![data[data.len() - 2], data[data.len() - 1]]
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
            let calculated =
                checksum::calculate_packet_checksum(packet_without_checksum).unwrap_or(0);
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
    pub fn validate_data_unit_length(
        &self,
        declared_length: u16,
        actual_data: &[u8],
    ) -> ParseResult<()> {
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

mod tests {
    use super::*;
    use crate::frame::{ControlUnit, Packet, Timestamp};
    use crate::protocol::{Command, ProtocolVersion};

    fn create_test_packet_data() -> Vec<u8> {
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
    fn test_frame_format_validation() {
        let validator = DataValidator::new();
        let packet_data = create_test_packet_data();

        // 正常数据应该通过验证
        assert!(validator.validate_frame_format(&packet_data).is_ok());

        // 太短的数据应该失败
        let short_data = vec![0x40, 0x40];
        assert!(validator.validate_frame_format(&short_data).is_err());

        // 错误的启动符应该失败
        let mut bad_start = packet_data.clone();
        bad_start[0] = 0xFF;
        assert!(validator.validate_frame_format(&bad_start).is_err());
    }

    #[test]
    fn test_checksum_validation() {
        let validator = DataValidator::new();
        let packet_data = create_test_packet_data();

        // 正常数据应该通过校验和验证
        assert!(validator.validate_checksum(&packet_data).is_ok());

        // 修改校验和应该失败
        let mut bad_checksum = packet_data.clone();
        let checksum_pos = bad_checksum.len() - 3;
        bad_checksum[checksum_pos] = bad_checksum[checksum_pos].wrapping_add(1);
        assert!(validator.validate_checksum(&bad_checksum).is_err());
    }

    #[test]
    fn test_complete_frame_validation() {
        let validator = DataValidator::new();
        let packet_data = create_test_packet_data();

        // 完整的正常数据包应该通过所有验证
        assert!(validator.validate_complete_frame(&packet_data).is_ok());
    }

    #[test]
    fn test_data_unit_length_validation() {
        let validator = DataValidator::new();
        let test_data = vec![0x01, 0x02, 0x03, 0x04];

        // 长度匹配应该通过
        assert!(validator.validate_data_unit_length(4, &test_data).is_ok());

        // 长度不匹配应该失败
        assert!(validator.validate_data_unit_length(5, &test_data).is_err());
        assert!(validator.validate_data_unit_length(3, &test_data).is_err());
    }
}
