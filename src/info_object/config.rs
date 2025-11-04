//! GB26875 配置信息对象
//!
//! 根据 GB26875 协议第8.2.1节实现的配置信息对象，包括建筑消防设施系统配置、部件配置和用户信息传输装置配置。

use bytes::Bytes;
use crate::error::{ParseResult, EncodeResult};
use crate::frame::timestamp::Timestamp;
use crate::protocol::types::{SystemType, ComponentType};
use super::InfoObject;

/// 建筑消防设施系统配置情况 (变长信息体 + 6字节时间戳)
/// 
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施系统配置情况
/// 
/// ## 字段布局
/// 
/// | 字段名          | 字节数     | 说明                     |
/// |----------------|-----------|--------------------------|
/// | 系统类型标志     | 1         | 建筑消防设施系统类型        |
/// | 系统地址        | 1         | 建筑消防设施系统地址        |
/// | 配置说明长度     | 1         | 配置说明字节数(L)          |
/// | 配置说明        | L(最多255) | 配置说明(GB18030编码)      |
/// | 配置时间        | 6         | 时间戳                   |
#[derive(Debug, Clone, PartialEq)]
pub struct FireSystemConfig {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 系统配置说明 (最多255字节，GB18030编码)
    pub config_description: Vec<u8>,
    /// 配置时间 (6字节)
    pub timestamp: Timestamp,
}

impl FireSystemConfig {
    /// 创建新的建筑消防设施系统配置情况    
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        config_description: Vec<u8>,
        timestamp: Timestamp,
    ) -> Result<Self, crate::error::ParseError> {
        if config_description.len() > 255 {
            return Err(crate::error::ParseError::InvalidValue { 
                field: "config_description_length".to_string(),
                value: config_description.len().to_string()
            });
        }
        
        Ok(Self {
            system_type,
            system_address,
            config_description,
            timestamp,
        })
    }
    
    /// 从文本创建配置说明 (自动转换为字节)
    pub fn with_description_text(
        system_type: SystemType,
        system_address: u8,
        description_text: &str,
        timestamp: Timestamp,
    ) -> Result<Self, crate::error::ParseError> {
        let config_description = description_text.as_bytes().to_vec();
        Self::new(system_type, system_address, config_description, timestamp)
    }
    
    /// 获取配置说明的文本形式
    pub fn description_text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.config_description)
    }
}

impl InfoObject for FireSystemConfig {
    fn object_type(&self) -> u8 {
        6 // 上传建筑消防设施系统配置情况
    }
    
    fn description(&self) -> Option<&str> {
        Some("建筑消防设施系统配置情况")
    }
    
    fn encode(&self) -> EncodeResult<Bytes> {
        let info_body_len = 3 + self.config_description.len(); // 3字节固定 + 变长配置说明
        let mut buf = Vec::with_capacity(info_body_len + 6); // 信息体 + 6字节时间戳
        
        // 信息体 (3 + L字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.push(self.config_description.len() as u8); // 配置说明长度
        buf.extend_from_slice(&self.config_description); // 配置说明
        
        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());
        
        Ok(Bytes::from(buf))
    }
    
    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 9 { // 最小长度：3字节固定 + 0字节配置说明 + 6字节时间戳
            return Err(crate::error::ParseError::TooShort { 
                expected: 9, 
                actual: data.len() 
            });
        }
        
        let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];
        let config_len = data[2] as usize;
        
        if data.len() < 3 + config_len + 6 {
            return Err(crate::error::ParseError::TooShort { 
                expected: 3 + config_len + 6, 
                actual: data.len() 
            });
        }
        
        let config_description = data[3..3 + config_len].to_vec();
        let timestamp = Timestamp::from_bytes(&data[3 + config_len..3 + config_len + 6])?;
        
        Self::new(system_type, system_address, config_description, timestamp)
            .map_err(|_| crate::error::ParseError::InvalidDataLength { actual: config_len, expected: 255 })
    }
    
    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

/// 建筑消防设施系统部件配置情况 (38字节信息体 + 6字节时间戳)
/// 
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施系统部件配置情况
/// 
/// ## 字段布局
/// 
/// | 字段名        | 字节数 | 说明                     |
/// |--------------|-------|--------------------------|
/// | 系统类型标志   | 1     | 建筑消防设施系统类型        |
/// | 系统地址      | 1     | 建筑消防设施系统地址        |
/// | 部件类型      | 1     | 建筑消防设施部件类型        |
/// | 部件地址      | 4     | 部件地址(小端序)           |
/// | 部件说明      | 31    | 部件说明(GB18030编码)      |
/// | 配置时间      | 6     | 时间戳                   |
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentConfig {
    /// 系统类型标志 (1字节)
    pub system_type: SystemType,
    /// 系统地址 (1字节)
    pub system_address: u8,
    /// 部件类型 (1字节)
    pub component_type: ComponentType,
    /// 部件地址 (4字节，小端序)
    pub component_address: u32,
    /// 部件说明 (31字节，GB18030编码)
    pub component_description: [u8; 31],
    /// 配置时间 (6字节)
    pub timestamp: Timestamp,
}

impl ComponentConfig {
    /// 创建新的建筑消防设施系统部件配置情况
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        component_type: ComponentType,
        component_address: u32,
        component_description: [u8; 31],
        timestamp: Timestamp,
    ) -> Self {
        Self {
            system_type,
            system_address,
            component_type,
            component_address,
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
    
    /// 获取部件说明的文本形式
    pub fn description_text(&self) -> Result<&str, std::str::Utf8Error> {
        let text = std::str::from_utf8(&self.component_description)?;
        Ok(text.trim_end_matches('\0'))
    }
}

impl InfoObject for ComponentConfig {
    fn object_type(&self) -> u8 {
        7 // 上传建筑消防设施系统部件配置情况
    }
    
    fn description(&self) -> Option<&str> {
        Some("建筑消防设施系统部件配置情况")
    }
    
    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = Vec::with_capacity(44); // 38字节信息体 + 6字节时间戳
        
        // 信息体 (38字节)
        buf.push(self.system_type.to_u8());
        buf.push(self.system_address);
        buf.push(self.component_type.to_u8());
        buf.extend_from_slice(&self.component_address.to_le_bytes()); // 4字节，小端序
        buf.extend_from_slice(&self.component_description); // 31字节
        
        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());
        
        Ok(Bytes::from(buf))
    }
    
    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 44 {
            return Err(crate::error::ParseError::TooShort { 
                expected: 44, 
                actual: data.len() 
            });
        }
        
        let system_type = SystemType::from_u8(data[0]);
        let system_address = data[1];
        let component_type = ComponentType::from_u8(data[2]);
        let component_address = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
        
        let mut component_description = [0u8; 31];
        component_description.copy_from_slice(&data[7..38]);
        
        let timestamp = Timestamp::from_bytes(&data[38..44])?;
        
        Ok(ComponentConfig::new(
            system_type,
            system_address,
            component_type,
            component_address,
            component_description,
            timestamp,
        ))
    }
    
    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

/// 用户信息传输装置配置情况 (变长信息体 + 6字节时间戳)
/// 
/// 根据GB26875协议8.2.1节定义，用于上传用户信息传输装置配置情况
/// 
/// ## 字段布局
/// 
/// | 字段名          | 字节数     | 说明                     |
/// |----------------|-----------|--------------------------|
/// | 设备说明长度     | 1         | 设备说明字节数(L)          |
/// | 设备说明        | L(最多255) | 设备说明(GB18030编码)      |
/// | 配置时间        | 6         | 时间戳                   |
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceConfig {
    /// 设备配置说明 (最多255字节，GB18030编码)
    pub device_description: Vec<u8>,
    /// 配置时间 (6字节)
    pub timestamp: Timestamp,
}

impl DeviceConfig {
    /// 创建新的用户信息传输装置配置情况
    pub fn new(
        device_description: Vec<u8>,
        timestamp: Timestamp,
    ) -> Result<Self, crate::error::ParseError> {
        if device_description.len() > 255 {
            return Err(crate::error::ParseError::InvalidDataLength { actual: device_description.len(), expected: 255 });
        }

        Ok(Self {
            device_description,
            timestamp,
        })
    }
    
    /// 从文本创建设备配置说明 (自动转换为字节)
    pub fn with_description_text(
        description_text: &str,
        timestamp: Timestamp,
    ) -> Result<Self, crate::error::ParseError> {
        let device_description = description_text.as_bytes().to_vec();
        Self::new(device_description, timestamp)
    }
    
    /// 获取设备配置说明的文本形式
    pub fn description_text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.device_description)
    }
}

impl InfoObject for DeviceConfig {
    fn object_type(&self) -> u8 {
        26 // 上传用户信息传输装置配置情况
    }
    
    fn description(&self) -> Option<&str> {
        Some("用户信息传输装置配置情况")
    }
    
    fn encode(&self) -> EncodeResult<Bytes> {
        let info_body_len = 1 + self.device_description.len(); // 1字节长度 + 变长设备说明
        let mut buf = Vec::with_capacity(info_body_len + 6); // 信息体 + 6字节时间戳
        
        // 信息体 (1 + L字节)
        buf.push(self.device_description.len() as u8); // 设备说明长度
        buf.extend_from_slice(&self.device_description); // 设备说明
        
        // 时间戳 (6字节)
        buf.extend_from_slice(&self.timestamp.to_bytes());
        
        Ok(Bytes::from(buf))
    }
    
    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 7 { // 最小长度：1字节长度 + 0字节设备说明 + 6字节时间戳
            return Err(crate::error::ParseError::TooShort { 
                expected: 7, 
                actual: data.len() 
            });
        }
        
        let device_desc_len = data[0] as usize;
        
        if data.len() < 1 + device_desc_len + 6 {
            return Err(crate::error::ParseError::TooShort { 
                expected: 1 + device_desc_len + 6, 
                actual: data.len() 
            });
        }
        
        let device_description = data[1..1 + device_desc_len].to_vec();
        let timestamp = Timestamp::from_bytes(&data[1 + device_desc_len..1 + device_desc_len + 6])?;
        
        Self::new(device_description, timestamp)
            .map_err(|_| crate::error::ParseError::InvalidDataLength { actual: device_desc_len, expected: 255 })
    }

    fn timestamp(&self) -> Option<&Timestamp> {
        Some(&self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{SystemType, ComponentType};

    #[test]
    fn test_fire_system_config_encode_decode() {
        let timestamp = Timestamp::now();
        let config = FireSystemConfig::with_description_text(
            SystemType::FireAlarm,
            1,
            "火灾报警系统配置",
            timestamp
        ).expect("Failed to create config");

        // 测试编码
        let encoded = config.encode().expect("Failed to encode");
        assert!(encoded.len() >= 9); // 至少9字节

        // 测试解码
        let decoded = FireSystemConfig::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, config);
    }

    #[test]
    fn test_component_config_encode_decode() {
        let timestamp = Timestamp::now();
        let config = ComponentConfig::new(
            SystemType::FireAlarm,
            1,
            ComponentType::SmokeFireDetector,
            0x12345678,
            [0u8; 31],
            timestamp
        ).with_description_text("烟雾探测器配置");

        // 测试编码
        let encoded = config.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 44);

        // 测试解码
        let decoded = ComponentConfig::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, config);
    }

    #[test]
    fn test_device_config_encode_decode() {
        let timestamp = Timestamp::now();
        let config = DeviceConfig::with_description_text(
            "GB26875用户信息传输装置",
            timestamp
        ).expect("Failed to create config");

        // 测试编码
        let encoded = config.encode().expect("Failed to encode");
        assert!(encoded.len() >= 7); // 至少7字节

        // 测试解码
        let decoded = DeviceConfig::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, config);
    }

    #[test]
    fn test_config_description_text() {
        let timestamp = Timestamp::now();
        
        let fire_config = FireSystemConfig::with_description_text(
            SystemType::FireAlarm,
            1,
            "火灾报警系统配置",
            timestamp
        ).expect("Failed to create config");
        assert_eq!(fire_config.description_text().unwrap(), "火灾报警系统配置");
        
        let component_config = ComponentConfig::new(
            SystemType::FireAlarm,
            1,
            ComponentType::SmokeFireDetector,
            0x12345678,
            [0u8; 31],
            timestamp
        ).with_description_text("烟雾探测器配置");
        assert_eq!(component_config.description_text().unwrap(), "烟雾探测器配置");
        
        let device_config = DeviceConfig::with_description_text(
            "GB26875用户信息传输装置",
            timestamp
        ).expect("Failed to create config");
        assert_eq!(device_config.description_text().unwrap(), "GB26875用户信息传输装置");
    }
}
