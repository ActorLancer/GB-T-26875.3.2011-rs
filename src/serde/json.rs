//! GB26875 JSON 序列化支持
//!
//! 提供数据包和数据单元的 JSON 序列化功能

#[cfg(feature = "serde")]
use crate::{
    data_unit::GenericDataUnit,
    error::{EncodeError, ParseError},
    frame::Packet,
};

#[cfg(feature = "serde")]
use serde_json;

/// JSON 序列化器
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Default)]
pub struct JsonSerializer;

#[cfg(feature = "serde")]
impl JsonSerializer {
    /// 创建新的 JSON 序列化器
    pub fn new() -> Self {
        JsonSerializer
    }

    /// 将数据包序列化为 JSON
    ///
    /// # Arguments
    /// * `packet` - 要序列化的数据包
    ///
    /// # Returns
    /// * `Result<String, EncodeError>` - 成功返回 JSON 字符串
    pub fn serialize_packet(&self, packet: &Packet) -> Result<String, EncodeError> {
        serde_json::to_string(packet).map_err(|e| EncodeError::SerializationError {
            reason: e.to_string(),
        })
    }

    /// 将数据包序列化为格式化的 JSON
    ///
    /// # Arguments
    /// * `packet` - 要序列化的数据包
    ///
    /// # Returns
    /// * `Result<String, EncodeError>` - 成功返回格式化的 JSON 字符串
    pub fn serialize_packet_pretty(&self, packet: &Packet) -> Result<String, EncodeError> {
        serde_json::to_string_pretty(packet).map_err(|e| EncodeError::SerializationError {
            reason: e.to_string(),
        })
    }

    /// 从 JSON 反序列化数据包
    ///
    /// # Arguments
    /// * `json` - JSON 字符串
    ///
    /// # Returns
    /// * `Result<Packet, ParseError>` - 成功返回数据包
    pub fn deserialize_packet(&self, json: &str) -> Result<Packet, ParseError> {
        serde_json::from_str(json).map_err(|e| ParseError::DeserializationError {
            reason: e.to_string(),
        })
    }

    /// 将数据单元序列化为 JSON
    ///
    /// # Arguments
    /// * `data_unit` - 要序列化的数据单元
    ///
    /// # Returns
    /// * `Result<String, EncodeError>` - 成功返回 JSON 字符串
    pub fn serialize_data_unit(&self, data_unit: &GenericDataUnit) -> Result<String, EncodeError> {
        serde_json::to_string(data_unit).map_err(|e| EncodeError::SerializationError {
            reason: e.to_string(),
        })
    }

    /// 将数据单元序列化为格式化的 JSON
    ///
    /// # Arguments
    /// * `data_unit` - 要序列化的数据单元
    ///
    /// # Returns
    /// * `Result<String, EncodeError>` - 成功返回格式化的 JSON 字符串
    pub fn serialize_data_unit_pretty(
        &self,
        data_unit: &GenericDataUnit,
    ) -> Result<String, EncodeError> {
        serde_json::to_string_pretty(data_unit).map_err(|e| EncodeError::SerializationError {
            reason: e.to_string(),
        })
    }

    /// 从 JSON 反序列化数据单元
    ///
    /// # Arguments
    /// * `json` - JSON 字符串
    ///
    /// # Returns
    /// * `Result<GenericDataUnit, ParseError>` - 成功返回数据单元
    pub fn deserialize_data_unit(&self, json: &str) -> Result<GenericDataUnit, ParseError> {
        serde_json::from_str(json).map_err(|e| ParseError::DeserializationError {
            reason: e.to_string(),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use super::*;
    use crate::{
        data_unit::standard::SystemStatus,
        frame::{ControlUnit, Timestamp},
        protocol::SystemType,
        protocol::{Command, ProtocolVersion},
    };

    #[test]
    fn test_packet_json_serialization() {
        let control_unit = ControlUnit::new(
            1,
            ProtocolVersion::v1_0(),
            Timestamp::now(),
            0x123456,
            0x654321,
            0,
            Command::Heartbeat,
        );
        let packet = Packet::without_data(control_unit).unwrap();

        let serializer = JsonSerializer::new();
        let json = serializer.serialize_packet(&packet).unwrap();
        let deserialized = serializer.deserialize_packet(&json).unwrap();

        assert_eq!(packet, deserialized);
    }

    #[test]
    fn test_data_unit_json_serialization() {
        let status = SystemStatus::new(SystemType::FireAlarm, 0x123456).unwrap();
        let data_unit = GenericDataUnit::SystemStatus(status);

        let serializer = JsonSerializer::new();
        let json = serializer.serialize_data_unit(&data_unit).unwrap();
        let deserialized = serializer.deserialize_data_unit(&json).unwrap();

        assert_eq!(data_unit, deserialized);
    }
}
