//! GB26875 建筑消防设施部件模拟量值信息对象
//!
//! 根据 GB26875 协议第8.2.1节实现的模拟量值信息对象，用于上传建筑消防设施部件的模拟量值。

use super::InfoObject;
use crate::error::{EncodeResult, ParseResult};
use crate::frame::timestamp::Timestamp;
use crate::protocol::types::{ComponentType, SystemType};
use bytes::Bytes;

/// 模拟量类型定义
///
/// 根据GB26875协议8.2.1节定义的标准模拟量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum AnalogType {
    /// 未用
    Unused = 0,
    /// 事件计数 (件)
    EventCount = 1,
    /// 高度 (m)
    Height = 2,
    /// 温度 (℃)
    Temperature = 3,
    /// 压力 (MPa)
    PressureMPa = 4,
    /// 压力 (kPa)
    PressureKPa = 5,
    /// 气体浓度 (%LEL)
    GasConcentration = 6,
    /// 时间 (s)
    Time = 7,
    /// 电压 (V)
    Voltage = 8,
    /// 电流 (A)
    Current = 9,
    /// 流量 (L/s)
    FlowRate = 10,
    /// 风量 (m³/min)
    AirVolume = 11,
    /// 风速 (m/s)
    WindSpeed = 12,
}

impl AnalogType {
    /// 从u8值创建AnalogType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Unused),
            1 => Some(Self::EventCount),
            2 => Some(Self::Height),
            3 => Some(Self::Temperature),
            4 => Some(Self::PressureMPa),
            5 => Some(Self::PressureKPa),
            6 => Some(Self::GasConcentration),
            7 => Some(Self::Time),
            8 => Some(Self::Voltage),
            9 => Some(Self::Current),
            10 => Some(Self::FlowRate),
            11 => Some(Self::AirVolume),
            12 => Some(Self::WindSpeed),
            _ => None,
        }
    }
}

/// 建筑消防设施部件模拟量值 (10字节信息体 + 6字节时间戳)
///
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施部件模拟量值信息
///
/// ## 字段布局
///
/// | 字段名        | 字节数 | 说明                     |
/// |--------------|-------|--------------------------|
/// | 系统类型标志   | 1     | 建筑消防设施系统类型        |
/// | 系统地址      | 1     | 建筑消防设施系统地址        |
/// | 部件类型      | 1     | 建筑消防设施部件类型        |
/// | 部件地址      | 4     | 部件地址(小端序)           |
/// | 模拟量类型    | 1     | 模拟量类型标识            |
/// | 模拟量值      | 2     | 模拟量值(有符号,小端序)     |
/// | 值变化时间    | 6     | 时间戳                   |
///
/// ## 示例
///
/// ```rust
/// use gb26875::info_object::analog_value::{AnalogValue, AnalogType};
/// use gb26875::protocol::types::{SystemType, ComponentType};
/// use gb26875::frame::timestamp::Timestamp;
///
/// let analog_value = AnalogValue::new(
///     SystemType::FireAlarm,
///     1,                              // 系统地址
///     ComponentType::TemperatureFireDetector,
///     0x12345678,                     // 部件地址
///     AnalogType::Temperature,
///     250,                            // 25.0℃ (0.1℃精度)
///     Timestamp::now()
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalogValue {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 部件类型 (1字节)
    pub component_type: ComponentType,
    /// 部件地址 (4字节，小端序)
    pub component_address: u32,
    /// 模拟量类型 (1字节)
    pub analog_type: AnalogType,
    /// 模拟量值 (2字节有符号整型，小端序)
    pub analog_value: i16,
    /// 值变化时间 (6字节)
    pub timestamp: Timestamp,
}

impl AnalogValue {
    /// 创建新的模拟量值
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        component_type: ComponentType,
        component_address: u32,
        analog_type: AnalogType,
        analog_value: i16,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            component_type,
            component_address,
            analog_type,
            analog_value,
            timestamp,
        }
    }
}

impl InfoObject for AnalogValue {
    fn object_type(&self) -> u8 {
        3 // 上传建筑消防设施部件模拟量值
    }

    fn description(&self) -> Option<&str> {
        Some("建筑消防设施部件模拟量值")
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(16); // 10字节信息体 + 6字节时间戳

        // 信息体 (10字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.push(self.component_type.to_u8());
        buf.extend_from_slice(&self.component_address.to_le_bytes()); // 4字节，小端序
        buf.push(self.analog_type as u8);
        buf.extend_from_slice(&self.analog_value.to_le_bytes()); // 2字节有符号，小端序

        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());

        Ok(Bytes::from(buf))
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 16 {
            return Err(crate::error::ParseError::TooShort {
                expected: 16,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];
        let component_type = ComponentType::from_u8(data[2]);
        let component_address = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
        let analog_type = AnalogType::from_u8(data[7])
            .ok_or(crate::error::ParseError::InvalidAnalogType(data[7]))?;
        let analog_value = i16::from_le_bytes([data[8], data[9]]);
        let timestamp = Timestamp::from_bytes(&data[10..16])?;

        Ok(AnalogValue::new(
            system_type,
            system_address,
            component_type,
            component_address,
            analog_type,
            analog_value,
            timestamp,
        ))
    }

    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

mod tests {
    use super::*;
    use crate::protocol::types::{ComponentType, SystemType};

    #[test]
    fn test_analog_value_encode_decode() {
        let timestamp = Timestamp::now();
        let analog_value = AnalogValue::new(
            SystemType::FireAlarm,
            1,
            ComponentType::TemperatureFireDetector,
            0x12345678,
            AnalogType::Temperature,
            250,
            timestamp,
        );

        // 测试编码
        let encoded = analog_value.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 16);

        // 测试解码
        let decoded = AnalogValue::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, analog_value);
    }

    #[test]
    fn test_analog_type_conversion() {
        // 测试标准模拟量类型
        assert_eq!(AnalogType::from_u8(3), Some(AnalogType::Temperature));
        assert_eq!(AnalogType::from_u8(8), Some(AnalogType::Voltage));
        assert_eq!(AnalogType::from_u8(255), None); // 超出范围

        // 测试转换为u8
        assert_eq!(AnalogType::Temperature as u8, 3);
        assert_eq!(AnalogType::Voltage as u8, 8);
    }
    #[test]
    fn test_analog_value_fields() {
        let timestamp = Timestamp::now();
        let analog_value = AnalogValue::new(
            SystemType::FireAlarm,
            2,
            ComponentType::TemperatureFireDetector,
            0xABCDEF12,
            AnalogType::PressureKPa,
            1500,
            timestamp,
        );

        assert_eq!(analog_value.object_type(), 3);
        assert_eq!(analog_value.system_type, SystemType::FireAlarm);
        assert_eq!(analog_value.system_address, 2);
        assert_eq!(
            analog_value.component_type,
            ComponentType::TemperatureFireDetector
        );
        assert_eq!(analog_value.component_address, 0xABCDEF12);
        assert_eq!(analog_value.analog_type, AnalogType::PressureKPa);
        assert_eq!(analog_value.analog_value, 1500);
        assert_eq!(analog_value.description(), Some("建筑消防设施部件模拟量值"));
        assert!(analog_value.timestamp().is_some());
    }
}
