//! GB26875 控制单元定义
//!
//! 控制单元包含业务流水号、协议版本号、发送时间标签、源地址、目的地址、
//! 应用数据单元长度、命令字节，总共25字节

use crate::error::{ParseError, ParseResult};
use crate::protocol::{Command, ProtocolVersion};
use crate::frame::Timestamp;

/// GB26875 控制单元（25字节）
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlUnit {
    /// 业务流水号（2字节，小端序）
    pub sequence: u16,
    /// 协议版本号（2字节）
    pub version: ProtocolVersion,
    /// 发送时间标签（6字节）
    pub timestamp: Timestamp,
    /// 源地址（6字节，小端序）
    pub source_addr: u64,
    /// 目的地址（6字节，小端序）
    pub dest_addr: u64,
    /// 应用数据单元长度（2字节，小端序）
    pub data_unit_len: u16,
    /// 命令字节（1字节）
    pub command: Command,
}

impl ControlUnit {
    /// 创建新的控制单元
    pub fn new(
        sequence: u16,
        version: ProtocolVersion,
        timestamp: Timestamp,
        source_addr: u64,
        dest_addr: u64,
        data_unit_len: u16,
        command: Command,
    ) -> ParseResult<Self> {        // 验证数据单元长度
        if data_unit_len > 1024 {
            return Err(ParseError::DataUnitTooLarge {
                size: data_unit_len as usize,
                max_size: 1024,
            });
        }

        // 验证地址范围（6字节最大值）
        const MAX_ADDR: u64 = 0xFFFF_FFFF_FFFF;
        if source_addr > MAX_ADDR {
            return Err(ParseError::ValidAddress(source_addr, "Address length of source exceeds maximum".to_string()));
        }
        if dest_addr > MAX_ADDR {
            return Err(ParseError::ValidAddress(dest_addr, "Address length of destination exceeds maximum".to_string()));
        }

        Ok(Self {
            sequence,
            version,
            timestamp,
            source_addr,
            dest_addr,
            data_unit_len,
            command,
        })
    }

    /// 从字节数组解析控制单元
    pub fn from_bytes(bytes: &[u8]) -> ParseResult<Self> {
        if bytes.len() < 25 {            return Err(ParseError::TooShort {
                actual: bytes.len(),
                expected: 25,
            });
        }

        // 解析各个字段
        let sequence = u16::from_le_bytes([bytes[0], bytes[1]]);
        let version = ProtocolVersion::from_bytes([bytes[2], bytes[3]]);
        let timestamp = Timestamp::from_bytes(&bytes[4..10])?;
        
        // 解析6字节地址（小端序）
        let source_addr = u64::from_le_bytes([
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], 0, 0,
        ]);
        let dest_addr = u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], 0, 0,
        ]);
        
        let data_unit_len = u16::from_le_bytes([bytes[22], bytes[23]]);
        let command = Command::from_u8(bytes[24]);

        Self::new(
            sequence,
            version,
            timestamp,
            source_addr,
            dest_addr,
            data_unit_len,
            command,
        )
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> [u8; 25] {
        let mut bytes = [0u8; 25];

        // 业务流水号（小端序）
        let seq_bytes = self.sequence.to_le_bytes();
        bytes[0] = seq_bytes[0];
        bytes[1] = seq_bytes[1];

        // 协议版本号
        let version_bytes = self.version.to_bytes();
        bytes[2] = version_bytes[0];
        bytes[3] = version_bytes[1];

        // 时间标签
        let timestamp_bytes = self.timestamp.to_bytes();
        bytes[4..10].copy_from_slice(&timestamp_bytes);

        // 源地址（6字节，小端序）
        let source_bytes = self.source_addr.to_le_bytes();
        bytes[10..16].copy_from_slice(&source_bytes[0..6]);

        // 目的地址（6字节，小端序）
        let dest_bytes = self.dest_addr.to_le_bytes();
        bytes[16..22].copy_from_slice(&dest_bytes[0..6]);

        // 应用数据单元长度（小端序）
        let len_bytes = self.data_unit_len.to_le_bytes();
        bytes[22] = len_bytes[0];
        bytes[23] = len_bytes[1];

        // 命令字节
        bytes[24] = self.command.to_u8();

        bytes
    }

    /// 获取源地址的6字节表示
    pub fn source_addr_bytes(&self) -> [u8; 6] {
        let bytes = self.source_addr.to_le_bytes();
        [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
    }

    /// 获取目的地址的6字节表示
    pub fn dest_addr_bytes(&self) -> [u8; 6] {
        let bytes = self.dest_addr.to_le_bytes();
        [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
    }

    /// 设置源地址（从6字节数组）
    pub fn set_source_addr_from_bytes(&mut self, addr_bytes: [u8; 6]) {
        self.source_addr = u64::from_le_bytes([
            addr_bytes[0], addr_bytes[1], addr_bytes[2],
            addr_bytes[3], addr_bytes[4], addr_bytes[5],
            0, 0,
        ]);
    }

    /// 设置目的地址（从6字节数组）
    pub fn set_dest_addr_from_bytes(&mut self, addr_bytes: [u8; 6]) {
        self.dest_addr = u64::from_le_bytes([
            addr_bytes[0], addr_bytes[1], addr_bytes[2],
            addr_bytes[3], addr_bytes[4], addr_bytes[5],
            0, 0,
        ]);
    }

    /// 验证控制单元是否有效
    pub fn is_valid(&self) -> bool {
        self.data_unit_len <= 1024
            && self.timestamp.is_valid()
            && self.version.is_valid_major()
            && self.source_addr <= 0xFFFF_FFFF_FFFF
            && self.dest_addr <= 0xFFFF_FFFF_FFFF
    }

    /// 创建应答控制单元
    /// 
    /// 交换源地址和目的地址，设置为确认命令
    pub fn create_ack(&self, ack_command: Command) -> Self {
        Self {
            sequence: self.sequence, // 保持相同的流水号
            version: self.version,
            timestamp: Timestamp::now(), // 使用当前时间
            source_addr: self.dest_addr, // 交换地址
            dest_addr: self.source_addr,
            data_unit_len: 0, // 确认包通常没有数据单元
            command: ack_command,
        }
    }

    /// 创建确认控制单元
    pub fn create_acknowledge(&self) -> Self {
        self.create_ack(Command::Acknowledge)
    }

    /// 创建否认控制单元
    pub fn create_reject(&self) -> Self {
        self.create_ack(Command::Reject)
    }

    /// 创建应答控制单元（用于请求/应答模式）
    pub fn create_response(&self, data_unit_len: u16) -> Self {
        Self {
            sequence: self.sequence,
            version: self.version,
            timestamp: Timestamp::now(),
            source_addr: self.dest_addr,
            dest_addr: self.source_addr,
            data_unit_len,            command: Command::Response,
        }
    }

    /// 编码为字节序列（用于DataUnit trait）
    pub fn encode(&self) -> crate::error::EncodeResult<bytes::Bytes> {
        Ok(bytes::Bytes::from(self.to_bytes().to_vec()))
    }

    /// 从字节序列解析（用于DataUnit trait）
    pub fn parse(data: &[u8]) -> crate::error::ParseResult<Self> {
        Self::from_bytes(data)
    }

    /// 验证控制单元（用于DataUnit trait）
    pub fn validate(&self) -> crate::error::ParseResult<()> {
        if !self.is_valid() {
            return Err(crate::error::ParseError::InvalidValue {
                field: "control_unit".to_string(),
                value: "validation failed".to_string(),
                reason: "控制单元验证失败".to_string(),
            });
        }
        Ok(())
    }
}

impl Default for ControlUnit {
    fn default() -> Self {
        Self {
            sequence: 0,
            version: ProtocolVersion::default(),
            timestamp: Timestamp::default(),
            source_addr: 0,
            dest_addr: 0,
            data_unit_len: 0,
            command: Command::SendData,
        }
    }
}

impl std::fmt::Display for ControlUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ControlUnit {{ seq: {}, version: {}, time: {}, src: 0x{:012X}, dst: 0x{:012X}, len: {}, cmd: {:?} }}",
            self.sequence,
            self.version,
            self.timestamp,
            self.source_addr,
            self.dest_addr,
            self.data_unit_len,
            self.command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_unit_creation() {
        let timestamp = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        let version = ProtocolVersion::new(1, 1);
        
        let control_unit = ControlUnit::new(
            0x0001,
            version,
            timestamp,
            0x000379,
            0x01385B,
            0x0030,
            Command::SendData,
        ).unwrap();

        assert_eq!(control_unit.sequence, 0x0001);
        assert_eq!(control_unit.source_addr, 0x000379);
        assert_eq!(control_unit.dest_addr, 0x01385B);
        assert_eq!(control_unit.data_unit_len, 0x0030);
        assert_eq!(control_unit.command, Command::SendData);
    }

    #[test]
    fn test_control_unit_bytes() {
        let timestamp = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        let version = ProtocolVersion::new(1, 1);
        
        let control_unit = ControlUnit::new(
            0x0001,
            version,
            timestamp,
            0x000379,
            0x01385B,
            0x0030,
            Command::SendData,
        ).unwrap();

        let bytes = control_unit.to_bytes();
        let parsed = ControlUnit::from_bytes(&bytes).unwrap();
        
        assert_eq!(control_unit, parsed);
    }

    #[test]
    fn test_create_acknowledge() {
        let timestamp = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        let version = ProtocolVersion::new(1, 1);
        
        let original = ControlUnit::new(
            0x0001,
            version,
            timestamp,
            0x000379,
            0x01385B,
            0x0030,
            Command::SendData,
        ).unwrap();

        let ack = original.create_acknowledge();
        
        assert_eq!(ack.sequence, original.sequence);
        assert_eq!(ack.source_addr, original.dest_addr); // 地址交换
        assert_eq!(ack.dest_addr, original.source_addr);
        assert_eq!(ack.command, Command::Acknowledge);
        assert_eq!(ack.data_unit_len, 0); // 确认包无数据
    }

    #[test]
    fn test_invalid_data_unit_length() {
        let timestamp = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        let version = ProtocolVersion::new(1, 1);
        
        let result = ControlUnit::new(
            0x0001,
            version,
            timestamp,
            0x000379,
            0x01385B,
            1025, // 超过最大长度
            Command::SendData,
        );

        assert!(result.is_err());
    }
}
