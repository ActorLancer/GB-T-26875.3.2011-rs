//! GB26875 数据包结构定义
//!
//! 数据包是 GB26875 协议的完整通信单元，包含启动符、控制单元、
//! 应用数据单元（可选）、校验和、结束符

use crate::error::{EncodeError, EncodeResult, ParseError, ParseResult};
use crate::frame::{calculate_checksum, ControlUnit};
use crate::protocol::constants::*;
use bytes::{BufMut, Bytes, BytesMut};


mod bytes_serde {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &Option<Bytes>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match bytes {
            Some(b) => b.as_ref().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Bytes>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Option<Vec<u8>> = Option::deserialize(deserializer)?;
        Ok(vec.map(Bytes::from))
    }
}

/// GB26875 数据包
///
/// 完整的GB26875 协议数据包结构：
/// - 启动符（2字节）: 0x40 0x40
/// - 控制单元（25字节）
/// - 应用数据单元（可变长度，可选）
/// - 校验和（1字节）
/// - 结束符（2字节）: 0x23 0x23
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Packet {
    /// 控制单元（25字节）
    pub control_unit: ControlUnit,
    /// 应用数据单元（可选）
    #[serde(with = "bytes_serde", skip_serializing_if = "Option::is_none")]
    pub data_unit: Option<Bytes>,
}

impl Packet {
    /// 创建新的数据包
    ///
    /// # Arguments
    /// * `control_unit` - 控制单元
    /// * `data_unit` - 应用数据单元（可选）
    ///
    /// # Returns
    /// * `EncodeResult<Self>` - 创建的数据包或编码错误
    ///
    /// # Examples
    /// ```
    /// use gb26875::prelude::*;
    /// use bytes::Bytes;
    ///
    /// let control_unit = ControlUnit::new(
    ///     1,
    ///     ProtocolVersion::standard(),
    ///     Timestamp::now(),
    ///     0x123456,
    ///     0x654321,
    ///     10,
    ///     Command::SendData
    /// )?;
    /// let data = Some(Bytes::from_static(b"test data"));
    /// let packet = Packet::new(control_unit, data)?;
    /// # Ok::<(), gb26875::error::EncodeError>(())
    /// ```
    pub fn new(mut control_unit: ControlUnit, data_unit: Option<Bytes>) -> EncodeResult<Self> {
        // 验证数据单元长度
        let data_len = data_unit.as_ref().map(|d| d.len()).unwrap_or(0);
        if data_len > MAX_DATA_UNIT_SIZE {
            return Err(EncodeError::DataUnitTooLarge {
                size: data_len,
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        // 更新控制单元中的数据单元长度
        control_unit.data_unit_len = data_len as u16;

        Ok(Packet {
            control_unit,
            data_unit,
        })
    }

    /// 创建包含数据的数据包
    ///
    /// # Arguments
    /// * `control_unit` - 控制单元
    /// * `data` - 应用数据单元
    ///
    /// # Returns
    /// * `EncodeResult<Self>` - 创建的数据包或编码错误
    ///
    /// # Examples
    /// ```
    /// use gb26875::prelude::*;
    /// use bytes::Bytes;
    ///
    /// let control_unit = ControlUnit::new(
    ///     1,
    ///     ProtocolVersion::standard(),
    ///     Timestamp::now(),
    ///     0x123456,
    ///     0x654321,
    ///     0,
    ///     Command::SendData
    /// )?;
    ///
    /// let data = Bytes::from_static(b"test data");
    /// let packet = Packet::with_data(control_unit, data)?;
    /// # Ok::<(), gb26875::error::EncodeError>(())
    /// ```    
    pub fn with_data(mut control_unit: ControlUnit, data: Bytes) -> EncodeResult<Self> {
        if data.len() > MAX_DATA_UNIT_SIZE {
            return Err(EncodeError::DataUnitTooLarge {
                size: data.len(),
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        control_unit.data_unit_len = data.len() as u16;
        Ok(Packet {
            control_unit,
            data_unit: Some(data),
        })
    }

    /// 获取数据包的总长度（包括所有字段）
    ///
    /// # Returns
    /// 数据包的总字节数
    pub fn len(&self) -> usize {
        FRAME_START.len() +          // 启动符（2字节）
        25 +                         // 控制单元（25字节）
        self.control_unit.data_unit_len as usize + // 应用数据单元
        1 +                          // 校验和（1字节）
        FRAME_END.len() // 结束符（2字节）
    }

    /// 检查数据包是否为空（无数据单元）
    pub fn is_empty(&self) -> bool {
        self.data_unit.is_none() || self.control_unit.data_unit_len == 0
    }

    /// 获取数据单元的引用
    pub fn data_unit(&self) -> Option<&Bytes> {
        self.data_unit.as_ref()
    }

    /// 获取数据单元的可变引用
    pub fn data_unit_mut(&mut self) -> Option<&mut Bytes> {
        self.data_unit.as_mut()
    }

    /// 设置数据单元
    pub fn set_data_unit(&mut self, data_unit: Option<Bytes>) -> EncodeResult<()> {
        let data_len = data_unit.as_ref().map(|d| d.len()).unwrap_or(0);
        if data_len > MAX_DATA_UNIT_SIZE {
            return Err(EncodeError::DataUnitTooLarge {
                size: data_len,
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        self.control_unit.data_unit_len = data_len as u16;
        self.data_unit = data_unit;
        Ok(())
    }

    /// 创建空数据包（仅控制单元）
    ///
    /// # Arguments
    /// * `control_unit` - 控制单元
    ///
    /// # Returns
    /// * `Self` - 创建的空数据包
    pub fn empty(mut control_unit: ControlUnit) -> Self {
        control_unit.data_unit_len = 0;
        Packet {
            control_unit,
            data_unit: None,
        }
    }

    /// 编码数据包为字节序列
    ///
    /// # Returns
    /// * `EncodeResult<Bytes>` - 编码后的字节序列或编码错误
    ///
    /// # Examples
    /// ```
    /// use gb26875::prelude::*;
    /// use bytes::Bytes;
    ///
    /// let control_unit = ControlUnit::new(
    ///     1,
    ///     ProtocolVersion::standard(),
    ///     Timestamp::now(),
    ///     0x123456,
    ///     0x654321,
    ///     0,
    ///     Command::SendData
    /// )?;
    ///
    /// let packet = Packet::empty(control_unit);
    /// let encoded = packet.encode()?;
    /// # Ok::<(), gb26875::error::EncodeError>(())
    /// ```
    pub fn encode(&self) -> EncodeResult<Bytes> {
        // 验证数据单元长度
        if let Some(ref data) = self.data_unit {
            if data.len() > MAX_DATA_UNIT_SIZE {
                return Err(EncodeError::DataUnitTooLarge {
                    size: data.len(),
                    max_size: MAX_DATA_UNIT_SIZE,
                });
            }
        }

        let total_len = self.len();
        let mut buf = BytesMut::with_capacity(total_len);

        // 写入启动符
        buf.put_slice(&FRAME_START);

        // 写入控制单元
        let control_bytes = self.control_unit.to_bytes();
        buf.put_slice(&control_bytes);

        // 写入应用数据单元（如果有）
        if let Some(ref data) = self.data_unit {
            buf.put_slice(data);
        }

        // 计算校验和（不包括启动符、结束符和校验和本身）
        let checksum_data = &buf[FRAME_START.len()..];
        let control_unit_bytes = &checksum_data[0..25]; // 控制单元25字节
        let data_unit_bytes = &checksum_data[25..]; // 剩余为数据单元
        let checksum = calculate_checksum(control_unit_bytes, data_unit_bytes);
        buf.put_u8(checksum);

        // 写入结束符
        buf.put_slice(&FRAME_END);

        Ok(buf.freeze())
    }

    /// 从字节序列解析数据包
    ///
    /// # Arguments
    /// * `data` - 包含数据包的字节序列
    ///
    /// # Returns
    /// * `ParseResult<Self>` - 解析的数据包或解析错误
    ///
    /// # Examples
    /// ```
    /// use gb26875::prelude::*;
    /// use bytes::Bytes;
    ///
    /// let control_unit = ControlUnit::new(
    ///     1,
    ///     ProtocolVersion::standard(),
    ///     Timestamp::now(),
    ///     0x123456,
    ///     0x654321,
    ///     0,
    ///     Command::SendData
    /// )?;
    ///
    /// let original = Packet::empty(control_unit);
    /// let encoded = original.encode()?;
    /// let decoded = Packet::from_bytes(&encoded)?;
    /// assert_eq!(original, decoded);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes(data: &[u8]) -> ParseResult<Self> {
        // 检查最小长度：启动符(2) + 控制单元(25) + 校验和(1) + 结束符(2) = 30字节
        const MIN_PACKET_SIZE: usize = 30;
        if data.len() < MIN_PACKET_SIZE {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: MIN_PACKET_SIZE,
            });
        }

        let mut cursor = 0;

        // 验证启动符
        if &data[cursor..cursor + FRAME_START.len()] != FRAME_START {
            return Err(ParseError::InvalidStartMarker(
                data[cursor],
                data[cursor + 1],
            ));
        }
        cursor += FRAME_START.len();

        // 解析控制单元（25字节）
        let control_unit = ControlUnit::from_bytes(&data[cursor..cursor + 25])?;
        cursor += 25;

        // 解析应用数据单元
        let data_unit_len = control_unit.data_unit_len as usize;
        let data_unit = if data_unit_len > 0 {
            // 检查是否有足够的数据
            if cursor + data_unit_len + 3 > data.len() {
                // +3 for checksum(1) + end_marker(2)
                return Err(ParseError::TooShort {
                    actual: data.len(),
                    expected: cursor + data_unit_len + 3,
                });
            }

            // 验证数据单元长度
            if data_unit_len > MAX_DATA_UNIT_SIZE {
                return Err(ParseError::DataUnitTooLarge {
                    size: data_unit_len,
                    max_size: MAX_DATA_UNIT_SIZE,
                });
            }

            let data_bytes = &data[cursor..cursor + data_unit_len];
            cursor += data_unit_len;
            Some(Bytes::copy_from_slice(data_bytes))
        } else {
            None
        };

        // 验证校验和
        let expected_checksum = data[cursor];
        cursor += 1;

        // 计算校验和（控制单元 + 数据单元）
        let checksum_data = &data[FRAME_START.len()..cursor - 1];
        let control_unit_bytes = &checksum_data[0..25];
        let data_unit_bytes = &checksum_data[25..];
        let calculated_checksum = calculate_checksum(control_unit_bytes, data_unit_bytes);

        if expected_checksum != calculated_checksum {
            return Err(ParseError::ChecksumMismatch {
                expected: expected_checksum,
                actual: calculated_checksum,
            });
        }

        // 验证结束符
        if cursor + FRAME_END.len() > data.len() {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: cursor + FRAME_END.len(),
            });
        }

        if &data[cursor..cursor + FRAME_END.len()] != FRAME_END {
            return Err(ParseError::InvalidEndMarker(data[cursor], data[cursor + 1]));
        }

        Ok(Packet {
            control_unit,
            data_unit,
        })
    }

    /// 解析数据包（from_bytes的别名，为了兼容性）
    ///
    /// # Arguments
    /// * `data` - 包含数据包的字节序列
    ///
    /// # Returns
    /// * `ParseResult<Self>` - 解析的数据包或解析错误
    pub fn parse(data: &[u8]) -> ParseResult<Self> {
        Self::from_bytes(data)
    }

    /// 尝试解析数据包，返回解析结果和消耗的字节数
    ///
    /// # Arguments
    /// * `data` - 包含数据包的字节序列
    ///
    /// # Returns
    /// * `ParseResult<(Self, usize)>` - 解析的数据包和消耗的字节数，或解析错误
    pub fn try_parse(data: &[u8]) -> ParseResult<(Self, usize)> {
        // 检查最小长度
        const MIN_PACKET_SIZE: usize = 30;
        if data.len() < MIN_PACKET_SIZE {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: MIN_PACKET_SIZE,
            });
        }

        let mut cursor = 0;

        // 验证启动符
        if &data[cursor..cursor + FRAME_START.len()] != FRAME_START {
            return Err(ParseError::InvalidStartMarker(
                data[cursor],
                data[cursor + 1],
            ));
        }
        cursor += FRAME_START.len();

        // 解析控制单元（25字节）
        let control_unit = ControlUnit::from_bytes(&data[cursor..cursor + 25])?;
        cursor += 25;

        // 计算预期的数据包总长度
        let data_unit_len = control_unit.data_unit_len as usize;
        let expected_total_len = FRAME_START.len() + 25 + data_unit_len + 1 + FRAME_END.len();

        // 检查是否有足够的数据
        if data.len() < expected_total_len {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: expected_total_len,
            });
        }

        // 解析应用数据单元
        let data_unit = if data_unit_len > 0 {
            // 验证数据单元长度
            if data_unit_len > MAX_DATA_UNIT_SIZE {
                return Err(ParseError::DataUnitTooLarge {
                    size: data_unit_len,
                    max_size: MAX_DATA_UNIT_SIZE,
                });
            }

            let data_bytes = &data[cursor..cursor + data_unit_len];
            cursor += data_unit_len;
            Some(Bytes::copy_from_slice(data_bytes))
        } else {
            None
        };

        // 验证校验和
        let expected_checksum = data[cursor];
        cursor += 1;

        // 计算校验和（控制单元 + 数据单元）
        let checksum_data = &data[FRAME_START.len()..cursor - 1];
        let control_unit_bytes = &checksum_data[0..25];
        let data_unit_bytes = &checksum_data[25..];
        let calculated_checksum = calculate_checksum(control_unit_bytes, data_unit_bytes);

        if expected_checksum != calculated_checksum {
            return Err(ParseError::ChecksumMismatch {
                expected: expected_checksum,
                actual: calculated_checksum,
            });
        }

        // 验证结束符
        if cursor + FRAME_END.len() > data.len() {
            return Err(ParseError::TooShort {
                actual: data.len(),
                expected: cursor + FRAME_END.len(),
            });
        }

        if &data[cursor..cursor + FRAME_END.len()] != FRAME_END {
            return Err(ParseError::InvalidEndMarker(data[cursor], data[cursor + 1]));
        }
        cursor += FRAME_END.len();

        let packet = Packet {
            control_unit,
            data_unit,
        };

        Ok((packet, cursor))
    }

    /// 验证数据包是否有效
    ///
    /// # Returns
    /// * `ParseResult<()>` - 验证成功或验证错误
    pub fn validate(&self) -> ParseResult<()> {
        // 验证控制单元
        self.control_unit.validate()?;

        // 验证数据单元长度一致性
        let actual_len = self.data_unit.as_ref().map(|d| d.len()).unwrap_or(0);
        if actual_len != self.control_unit.data_unit_len as usize {
            return Err(ParseError::DataLengthMismatch {
                expected: self.control_unit.data_unit_len as usize,
                actual: actual_len,
            });
        }

        // 验证数据单元长度限制
        if actual_len > MAX_DATA_UNIT_SIZE {
            return Err(ParseError::DataUnitTooLarge {
                size: actual_len,
                max_size: MAX_DATA_UNIT_SIZE,
            });
        }

        Ok(())
    }

    /// 创建应答数据包
    ///
    /// # Arguments
    /// * `response_data` - 应答数据（可选）
    ///
    /// # Returns
    /// * `EncodeResult<Self>` - 应答数据包或编码错误
    pub fn create_response(&self, response_data: Option<Bytes>) -> EncodeResult<Self> {
        let data_len = response_data.as_ref().map(|d| d.len()).unwrap_or(0);
        let response_control_unit = self.control_unit.create_response(data_len as u16);

        Self::new(response_control_unit, response_data)
    }

    /// 创建确认数据包
    ///
    /// # Returns
    /// * `Self` - 确认数据包
    pub fn create_acknowledge(&self) -> Self {
        let ack_control_unit = self.control_unit.create_acknowledge();
        Packet::empty(ack_control_unit)
    }

    /// 创建否认数据包
    ///
    /// # Returns
    /// * `Self` - 否认数据包
    pub fn create_reject(&self) -> Self {
        let reject_control_unit = self.control_unit.create_reject();
        Packet::empty(reject_control_unit)
    }

    // TODO：监测心跳包的条件是传输装置发送一个命令字节为0x02的数据包，且应用单元长度为0，然后监控中心发送一个命令字节为0x03应用单元长度为0的数据包确认
    /// 检查是否为心跳包
    pub fn is_heartbeat(&self) -> bool {
        // matches!(self.control_unit.command, crate::protocol::Command::Heartbeat)
        todo!();
    }

    /// 检查是否为确认包
    pub fn is_acknowledge(&self) -> bool {
        matches!(
            self.control_unit.command,
            crate::protocol::Command::Acknowledge
        )
    }

    /// 检查是否为否认包
    pub fn is_reject(&self) -> bool {
        matches!(self.control_unit.command, crate::protocol::Command::Reject)
    }
}

impl Default for Packet {
    fn default() -> Self {
        Packet {
            control_unit: ControlUnit::default(),
            data_unit: None,
        }
    }
}

impl std::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Packet {{ control: {}, data_len: {} }}",
            self.control_unit,
            self.data_unit.as_ref().map(|d| d.len()).unwrap_or(0)
        )
    }
}

mod tests {
    use super::*;
    use crate::frame::Timestamp;
    use crate::protocol::{Command, ProtocolVersion};

    fn create_test_control_unit() -> ControlUnit {
        ControlUnit::new(
            1,
            ProtocolVersion::standard(),
            Timestamp::now(),
            0x123456,
            0x654321,
            0,
            Command::SendData,
        )
        .unwrap()
    }

    #[test]
    fn test_packet_creation() {
        let control_unit = create_test_control_unit();
        let packet = Packet::empty(control_unit.clone());

        assert_eq!(packet.control_unit, control_unit);
        assert!(packet.data_unit.is_none());
        assert!(packet.is_empty());
    }

    #[test]
    fn test_packet_with_data() {
        let control_unit = create_test_control_unit();
        let data = Bytes::from_static(b"test data");
        let packet = Packet::with_data(control_unit, data.clone()).unwrap();

        assert_eq!(packet.data_unit(), Some(&data));
        assert!(!packet.is_empty());
        assert_eq!(packet.control_unit.data_unit_len, data.len() as u16);
    }

    #[test]
    fn test_packet_encode_decode() {
        let control_unit = create_test_control_unit();
        let original = Packet::empty(control_unit);

        let encoded = original.encode().unwrap();
        let decoded = Packet::from_bytes(&encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_data_too_large() {
        let control_unit = create_test_control_unit();
        let large_data = vec![0u8; MAX_DATA_UNIT_SIZE + 1];
        let data = Bytes::from(large_data);

        let result = Packet::with_data(control_unit, data);
        assert!(matches!(result, Err(EncodeError::DataUnitTooLarge { .. })));
    }

    #[test]
    fn test_packet_validation() {
        let control_unit = create_test_control_unit();
        let packet = Packet::empty(control_unit);

        assert!(packet.validate().is_ok());
    }

    #[test]
    fn test_packet_responses() {
        let control_unit = create_test_control_unit();
        let packet = Packet::empty(control_unit);

        let ack = packet.create_acknowledge();
        assert!(ack.is_acknowledge());

        let reject = packet.create_reject();
        assert!(reject.is_reject());
    }
}
