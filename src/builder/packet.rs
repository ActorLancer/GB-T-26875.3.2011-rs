//! GB26875 数据包构建器
//!
//! 提供友好的数据包构建 API

use crate::builder::{Builder, ResettableBuilder};
use crate::data_unit::GenericDataUnit;
use crate::error::{EncodeError, EncodeResult};
use crate::frame::{ControlUnit, Packet, Timestamp};
use crate::protocol::{Command, ProtocolVersion};
use bytes::Bytes;

/// 数据包构建器
///
/// 提供链式调用接口来构建 GB26875 数据包
#[derive(Debug, Clone)]
pub struct PacketBuilder {
    /// 业务流水号
    sequence: Option<u16>,
    /// 协议版本号
    version: Option<ProtocolVersion>,
    /// 发送时间标签
    timestamp: Option<Timestamp>,
    /// 源地址
    source_addr: Option<u64>,
    /// 目的地址
    dest_addr: Option<u64>,
    /// 命令字节
    command: Option<Command>,
    /// 应用数据单元
    data_unit: Option<Bytes>,
}

impl PacketBuilder {
    /// 创建新的数据包构建器
    pub fn new() -> Self {
        PacketBuilder {
            sequence: None,
            version: None,
            timestamp: None,
            source_addr: None,
            dest_addr: None,
            command: None,
            data_unit: None,
        }
    }

    /// 设置业务流水号
    ///
    /// # Arguments
    /// * `sequence` - 业务流水号
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn sequence(mut self, sequence: u16) -> Self {
        self.sequence = Some(sequence);
        self
    }

    /// 设置协议版本号
    ///
    /// # Arguments
    /// * `version` - 协议版本号
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn version(mut self, version: ProtocolVersion) -> Self {
        self.version = Some(version);
        self
    }

    /// 使用默认协议版本号（1.0）
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn default_version(mut self) -> Self {
        self.version = Some(ProtocolVersion::v1_0());
        self
    }

    /// 设置发送时间标签
    ///
    /// # Arguments
    /// * `timestamp` - 时间标签
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// 使用当前时间作为时间标签
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn current_timestamp(mut self) -> Self {
        self.timestamp = Some(Timestamp::now());
        self
    }

    /// 设置源地址
    ///
    /// # Arguments
    /// * `addr` - 源地址（6字节）
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn source_address(mut self, addr: u64) -> EncodeResult<Self> {
        if addr > 0xFFFFFFFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "source_address".to_string(),
                value: format!("0x{:X}", addr),
                reason: "源地址必须在 6 字节范围内".to_string(),
            });
        }
        self.source_addr = Some(addr);
        Ok(self)
    }

    /// 设置目的地址
    ///
    /// # Arguments
    /// * `addr` - 目的地址（6字节）
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn destination_address(mut self, addr: u64) -> EncodeResult<Self> {
        if addr > 0xFFFFFFFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "destination_address".to_string(),
                value: format!("0x{:X}", addr),
                reason: "目的地址必须在 6 字节范围内".to_string(),
            });
        }
        self.dest_addr = Some(addr);
        Ok(self)
    }

    /// 设置命令字节
    ///
    /// # Arguments
    /// * `command` - 命令字节
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    /// 设置应用数据单元（原始字节）
    ///
    /// # Arguments
    /// * `data` - 应用数据单元字节数据
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn data_unit_bytes(mut self, data: Bytes) -> EncodeResult<Self> {
        if data.len() > crate::protocol::constants::MAX_DATA_UNIT_SIZE {
            return Err(EncodeError::DataTooLarge {
                size: data.len(),
                max_size: crate::protocol::constants::MAX_DATA_UNIT_SIZE,
            });
        }
        self.data_unit = Some(data);
        Ok(self)
    }

    /// 设置应用数据单元（从通用数据单元）
    ///
    /// # Arguments
    /// * `data_unit` - 通用数据单元
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn data_unit(self, data_unit: GenericDataUnit) -> EncodeResult<Self> {
        let encoded = data_unit.encode()?;
        self.data_unit_bytes(encoded)
    }

    /// 清除应用数据单元
    ///
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn no_data_unit(mut self) -> Self {
        self.data_unit = None;
        self
    }

    /// 创建心跳包构建器
    ///
    /// # Arguments
    /// * `sequence` - 业务流水号
    /// * `source_addr` - 源地址
    /// * `dest_addr` - 目的地址
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回配置好的构建器
    pub fn heartbeat(sequence: u16, source_addr: u64, dest_addr: u64) -> EncodeResult<Self> {
        Ok(PacketBuilder::new()
            .sequence(sequence)
            .default_version()
            .current_timestamp()
            .source_address(source_addr)?
            .destination_address(dest_addr)?
            .command(Command::SendData)
            .no_data_unit())
    }

    /// 创建状态查询包构建器
    ///
    /// # Arguments
    /// * `sequence` - 业务流水号
    /// * `source_addr` - 源地址
    /// * `dest_addr` - 目的地址
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回配置好的构建器
    pub fn status_query(sequence: u16, source_addr: u64, dest_addr: u64) -> EncodeResult<Self> {
        Ok(PacketBuilder::new()
            .sequence(sequence)
            .default_version()
            .current_timestamp()
            .source_address(source_addr)?
            .destination_address(dest_addr)?
            .command(Command::Request)
            .no_data_unit())
    }

    /// 创建状态上报包构建器
    ///
    /// # Arguments
    /// * `sequence` - 业务流水号
    /// * `source_addr` - 源地址
    /// * `dest_addr` - 目的地址
    /// * `data_unit` - 状态数据单元
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回配置好的构建器
    pub fn status_upload(
        sequence: u16,
        source_addr: u64,
        dest_addr: u64,
        data_unit: GenericDataUnit,
    ) -> EncodeResult<Self> {
        Ok(PacketBuilder::new()
            .sequence(sequence)
            .default_version()
            .current_timestamp()
            .source_address(source_addr)?
            .destination_address(dest_addr)?
            .command(Command::SendData)
            .data_unit(data_unit)?)
    }

    /// 创建确认包构建器
    ///
    /// # Arguments
    /// * `sequence` - 业务流水号
    /// * `source_addr` - 源地址
    /// * `dest_addr` - 目的地址
    ///
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回配置好的构建器
    pub fn acknowledgment(sequence: u16, source_addr: u64, dest_addr: u64) -> EncodeResult<Self> {
        Ok(PacketBuilder::new()
            .sequence(sequence)
            .default_version()
            .current_timestamp()
            .source_address(source_addr)?
            .destination_address(dest_addr)?
            .command(Command::Acknowledge)
            .no_data_unit())
    }

    /// 获取当前设置的业务流水号
    pub fn get_sequence(&self) -> Option<u16> {
        self.sequence
    }

    /// 获取当前设置的源地址
    pub fn get_source_address(&self) -> Option<u64> {
        self.source_addr
    }

    /// 获取当前设置的目的地址
    pub fn get_destination_address(&self) -> Option<u64> {
        self.dest_addr
    }

    /// 获取当前设置的命令
    pub fn get_command(&self) -> Option<Command> {
        self.command
    }

    /// 检查是否有数据单元
    pub fn has_data_unit(&self) -> bool {
        self.data_unit.is_some()
    }

    /// 获取数据单元长度
    pub fn data_unit_length(&self) -> usize {
        self.data_unit.as_ref().map(|d| d.len()).unwrap_or(0)
    }
}

impl Builder<Packet> for PacketBuilder {
    fn build(self) -> EncodeResult<Packet> {
        self.validate()?;
        let control_unit = ControlUnit::new(
            self.sequence.unwrap(),
            self.version.unwrap(),
            self.timestamp.unwrap(),
            self.source_addr.unwrap(),
            self.dest_addr.unwrap(),
            self.data_unit.as_ref().map(|d| d.len()).unwrap_or(0) as u16,
            self.command.unwrap(),
        )
        .map_err(|e| EncodeError::TypeConversion(format!("Control unit creation failed: {}", e)))?;

        Packet::new(control_unit, self.data_unit)
    }

    fn validate(&self) -> EncodeResult<()> {
        if self.sequence.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "sequence".to_string(),
                value: "None".to_string(),
                reason: "业务流水号不能为空".to_string(),
            });
        }

        if self.version.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "version".to_string(),
                value: "None".to_string(),
                reason: "协议版本号不能为空".to_string(),
            });
        }

        if self.timestamp.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "timestamp".to_string(),
                value: "None".to_string(),
                reason: "时间标签不能为空".to_string(),
            });
        }

        if self.source_addr.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "source_addr".to_string(),
                value: "None".to_string(),
                reason: "源地址不能为空".to_string(),
            });
        }

        if self.dest_addr.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "dest_addr".to_string(),
                value: "None".to_string(),
                reason: "目的地址不能为空".to_string(),
            });
        }

        if self.command.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "command".to_string(),
                value: "None".to_string(),
                reason: "命令字节不能为空".to_string(),
            });
        }

        Ok(())
    }
}

impl ResettableBuilder<Packet> for PacketBuilder {
    fn reset(&mut self) {
        self.sequence = None;
        self.version = None;
        self.timestamp = None;
        self.source_addr = None;
        self.dest_addr = None;
        self.command = None;
        self.data_unit = None;
    }
}

impl Default for PacketBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info_object::SystemStatus;
    use crate::protocol::SystemType;

    #[test]
    fn test_packet_builder_basic() {
        let packet = PacketBuilder::new()
            .sequence(1)
            .default_version()
            .current_timestamp()
            .source_address(0x123456)
            .unwrap()
            .destination_address(0x654321)
            .unwrap()
            .command(Command::Control)
            .build()
            .unwrap();

        assert_eq!(packet.control_unit.sequence, 1);
        assert_eq!(packet.control_unit.source_addr, 0x123456);
        assert_eq!(packet.control_unit.dest_addr, 0x654321);
        assert_eq!(packet.control_unit.command, Command::Control);
        assert!(packet.data_unit().is_none());
    }

    #[test]
    fn test_packet_builder_with_data_unit() {
        let status = SystemStatus::new(
            SystemType::FireAlarm,
            1,      // system_address
            0x0002, // system_state
            Timestamp::now(),
        );
        let data_unit = GenericDataUnit::UploadSystemStatus(
            crate::data_unit::standard::upstream::UploadSystemStatus::new(status, Timestamp::now()),
        );

        let packet = PacketBuilder::new()
            .sequence(2)
            .default_version()
            .current_timestamp()
            .source_address(0x123456)
            .unwrap()
            .destination_address(0x654321)
            .unwrap()
            .command(Command::SendData)
            .data_unit(data_unit)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(packet.control_unit.sequence, 2);
        assert_eq!(packet.control_unit.command, Command::SendData);
        assert!(packet.data_unit.is_some());
        assert_eq!(packet.control_unit.data_unit_len, 17); // UploadSystemStatus 是 17 字节 (1+10+6)
    }
    #[test]
    fn test_packet_builder_heartbeat() {
        let packet = PacketBuilder::heartbeat(1, 0x123456, 0x654321)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(packet.control_unit.sequence, 1);
        assert_eq!(packet.control_unit.source_addr, 0x123456);
        assert_eq!(packet.control_unit.dest_addr, 0x654321);
        assert_eq!(packet.control_unit.command, Command::SendData);
        assert!(packet.data_unit.is_none());
    }

    #[test]
    fn test_packet_builder_validation() {
        let builder = PacketBuilder::new().sequence(1);
        assert!(builder.validate().is_err()); // 缺少必要字段

        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_packet_builder_invalid_address() {
        let result = PacketBuilder::new().source_address(0x1000000000000); // 超过 6 字节

        assert!(result.is_err());
    }

    #[test]
    fn test_resettable_packet_builder() {
        let mut builder = PacketBuilder::new()
            .sequence(1)
            .default_version()
            .current_timestamp()
            .source_address(0x123456)
            .unwrap()
            .destination_address(0x654321)
            .unwrap()
            .command(Command::SendData);

        let packet = builder.build_and_reset().unwrap();
        assert_eq!(packet.control_unit.sequence, 1);

        // 构建器应该被重置
        assert!(builder.get_sequence().is_none());
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_packet_builder_getters() {
        let builder = PacketBuilder::new()
            .sequence(42)
            .source_address(0x111111)
            .unwrap()
            .destination_address(0x222222)
            .unwrap()
            .command(Command::Request);

        assert_eq!(builder.get_sequence(), Some(42));
        assert_eq!(builder.get_source_address(), Some(0x111111));
        assert_eq!(builder.get_destination_address(), Some(0x222222));
        assert_eq!(builder.get_command(), Some(Command::Request));
        assert!(!builder.has_data_unit());
        assert_eq!(builder.data_unit_length(), 0);
    }
}
