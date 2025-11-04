//! GB26875 信息对象模块
//!
//! 提供标准信息对象的实现，包括系统状态、部件状态、模拟量值等

use bytes::Bytes;
use crate::error::{ParseResult, EncodeResult};
use crate::protocol::types::{SystemType, ComponentType};
use crate::frame::timestamp::Timestamp;

/// 信息对象基础 trait
pub trait InfoObject: std::fmt::Debug + Send + Sync {
    /// 获取信息对象类型标识
    fn object_type(&self) -> u8;
    
    /// 获取信息对象描述
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// 编码信息对象为字节序列
    fn encode(&self) -> EncodeResult<Bytes>;
    
    /// 从字节序列解析信息对象
    fn parse(data: &[u8]) -> ParseResult<Self> where Self: Sized;
    
    /// 获取信息对象的时间戳
    fn timestamp(&self) -> Option<&Timestamp> {
        None
    }
}

/// 建筑消防设施系统状态 (4字节信息体 + 6字节时间戳)
#[derive(Debug, Clone, PartialEq)]
pub struct SystemStatus {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 系统状态 (2字节，小端序)
    pub system_state: u16,
    /// 状态发生时间 (6字节)
    pub timestamp: Timestamp,
}

impl SystemStatus {
    /// 创建新的系统状态
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        system_state: u16,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            system_state,
            timestamp,
        }
    }
}

impl InfoObject for SystemStatus {
    fn object_type(&self) -> u8 {
        1 // 上传建筑消防设施系统状态
    }
    
    fn description(&self) -> Option<&str> {
        Some("建筑消防设施系统状态")
    }
    
    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(10); // 4字节信息体 + 6字节时间戳
          // 信息体 (4字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);        buf.extend_from_slice(&self.system_state.to_le_bytes()); // 小端序
        
        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());
        
        Ok(Bytes::from(buf))
    }
    
    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 10 {
            return Err(crate::error::ParseError::TooShort { 
                expected: 10, 
                actual: data.len() 
            });
        }
          let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];
        let system_state = u16::from_le_bytes([data[2], data[3]]);
        let timestamp = Timestamp::from_bytes(&data[4..10])?;
        
        Ok(SystemStatus::new(system_type, system_address, system_state, timestamp))
    }
    
    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

/// 建筑消防设施部件状态 (40字节信息体 + 6字节时间戳)
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentStatus {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 部件类型 (1字节)
    pub component_type: ComponentType,
    /// 部件地址 (4字节，小端序)
    pub component_address: u32,
    /// 部件状态 (2字节，小端序)
    pub component_state: u16,
    /// 部件说明 (31字节，GB18030编码)
    pub component_description: [u8; 31],
    /// 状态发生时间 (6字节)
    pub timestamp: Timestamp,
}

impl ComponentStatus {
    /// 创建新的部件状态
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        component_type: ComponentType,
        component_address: u32,
        component_state: u16,
        component_description: [u8; 31],
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            component_type,
            component_address,
            component_state,
            component_description,
            timestamp,
        }
    }
    
    /// 设置部件说明文本 (自动转换为GB18030编码)
    pub fn with_description_text(mut self, text: &str) -> Self {
        let mut desc = [0u8; 31];
        let bytes = text.as_bytes();
        let copy_len = bytes.len().min(31);
        desc[..copy_len].copy_from_slice(&bytes[..copy_len]);
        self.component_description = desc;
        self
    }
}

impl InfoObject for ComponentStatus {
    fn object_type(&self) -> u8 {
        2 // 上传建筑消防设施部件运行状态
    }
    
    fn description(&self) -> Option<&str> {
        Some("建筑消防设施部件状态")
    }
    
    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(46); // 40字节信息体 + 6字节时间戳
          // 信息体 (40字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.push(self.component_type.to_u8());
        buf.extend_from_slice(&self.component_address.to_le_bytes()); // 4字节，小端序
        buf.extend_from_slice(&self.component_state.to_le_bytes()); // 2字节，小端序
        buf.extend_from_slice(&self.component_description); // 31字节
        
        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());
        
        Ok(Bytes::from(buf))
    }
    
    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 46 {
            return Err(crate::error::ParseError::TooShort { 
                expected: 46, 
                actual: data.len() 
            });
        }
          let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];
        let component_type = ComponentType::from_u8(data[2]);
        let component_address = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
        let component_state = u16::from_le_bytes([data[7], data[8]]);
        
        let mut component_description = [0u8; 31];
        component_description.copy_from_slice(&data[9..40]);
        
        let timestamp = Timestamp::from_bytes(&data[40..46])?;
        
        Ok(ComponentStatus::new(
            system_type,
            system_address,
            component_type,
            component_address,
            component_state,
            component_description,
            timestamp,
        ))
    }
    
    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

/// 模拟量类型定义
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq)]
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
                actual: data.len() 
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{SystemType, ComponentType};
    use crate::frame::timestamp::Timestamp;

    #[test]
    fn test_system_status_encode_decode() {
        let timestamp = Timestamp::now();
        let system_status = SystemStatus::new(
            SystemType::FireAlarm,
            1,
            0x0002, // 火警状态
            timestamp
        );

        // 测试编码
        let encoded = system_status.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 10); // 4字节信息体 + 6字节时间戳

        // 测试解码
        let decoded = SystemStatus::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, system_status);
    }

    #[test]
    fn test_component_status_encode_decode() {
        let timestamp = Timestamp::now();
        let desc = [0u8; 31]; // 空描述
        let component_status = ComponentStatus::new(
            SystemType::FireAlarm,
            1,
            ComponentType::SmokeFireDetector,
            0x12345678,
            0x0002, // 火警状态
            desc,
            timestamp
        );

        // 测试编码
        let encoded = component_status.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 46); // 40字节信息体 + 6字节时间戳

        // 测试解码
        let decoded = ComponentStatus::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, component_status);
    }

    #[test]
    fn test_analog_value_encode_decode() {
        let timestamp = Timestamp::now();
        let analog_value = AnalogValue::new(
            SystemType::FireAlarm,
            1,
            ComponentType::TemperatureFireDetector,
            0x12345678,
            AnalogType::Temperature,
            250, // 25.0℃ (0.1℃精度)
            timestamp
        );

        // 测试编码
        let encoded = analog_value.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 16); // 10字节信息体 + 6字节时间戳

        // 测试解码
        let decoded = AnalogValue::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, analog_value);
    }

    #[test]
    fn test_component_status_with_description() {
        let timestamp = Timestamp::now();
        let component_status = ComponentStatus::new(
            SystemType::FireAlarm,
            1,
            ComponentType::SmokeFireDetector,
            0x12345678,
            0x0002,
            [0u8; 31],
            timestamp
        ).with_description_text("烟雾探测器");

        // 验证描述设置正确
        let desc_text = std::str::from_utf8(&component_status.component_description)
            .unwrap()
            .trim_end_matches('\0');
        assert_eq!(desc_text, "烟雾探测器");
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
    fn test_info_object_trait() {
        let timestamp = Timestamp::now();
        
        let system_status = SystemStatus::new(
            SystemType::FireAlarm,
            1,
            0x0002,
            timestamp
        );

        // 测试InfoObject trait方法
        assert_eq!(system_status.object_type(), 1);
        assert_eq!(system_status.description(), Some("建筑消防设施系统状态"));
        assert!(system_status.timestamp().is_some());
    }
}
