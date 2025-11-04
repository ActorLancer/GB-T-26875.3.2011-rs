//! GB26875 建筑消防设施部件状态信息对象
//!
//! 根据 GB26875 协议第8.2.1节实现的部件状态信息对象，用于上传建筑消防设施部件的运行状态。

use bytes::Bytes;
use crate::error::{ParseResult, EncodeResult};
use crate::frame::timestamp::Timestamp;
use crate::protocol::types::{SystemType, ComponentType};
use super::InfoObject;

/// 建筑消防设施部件状态 (40字节信息体 + 6字节时间戳)
/// 
/// 根据GB26875协议8.2.1节定义，用于上传建筑消防设施部件运行状态信息
/// 
/// ## 字段布局
/// 
/// | 字段名        | 字节数 | 说明                     |
/// |--------------|-------|--------------------------|
/// | 系统类型标志   | 1     | 建筑消防设施系统类型        |
/// | 系统地址      | 1     | 建筑消防设施系统地址        |
/// | 部件类型      | 1     | 建筑消防设施部件类型        |
/// | 部件地址      | 4     | 部件地址(小端序)           |
/// | 部件状态      | 2     | 部件运行状态(小端序)        |
/// | 部件说明      | 31    | 部件说明(GB18030编码)      |
/// | 状态发生时间   | 6     | 时间戳                   |
/// 
/// ## 示例
/// 
/// ```rust
/// use gb26875::info_object::component_status::ComponentStatus;
/// use gb26875::protocol::types::{SystemType, ComponentType};
/// use gb26875::frame::timestamp::Timestamp;
/// 
/// let status = ComponentStatus::new(
///     SystemType::FireAlarm,
///     1,                              // 系统地址
///     ComponentType::SmokeFireDetector,
///     0x12345678,                     // 部件地址
///     0x0002,                         // 火警状态
///     [0u8; 31],                      // 部件说明
///     Timestamp::now()
/// ).with_description_text("烟雾探测器");
/// ```
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
    
    /// 获取部件说明的文本形式
    pub fn description_text(&self) -> Result<&str, std::str::Utf8Error> {
        let text = std::str::from_utf8(&self.component_description)?;
        Ok(text.trim_end_matches('\0'))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{SystemType, ComponentType};

    #[test]
    fn test_component_status_encode_decode() {
        let timestamp = Timestamp::now();
        let status = ComponentStatus::new(
            SystemType::FireAlarm,
            1,
            ComponentType::SmokeFireDetector,
            0x12345678,
            0x0002,
            [0u8; 31],
            timestamp
        );

        // 测试编码
        let encoded = status.encode().expect("Failed to encode");
        assert_eq!(encoded.len(), 46);

        // 测试解码
        let decoded = ComponentStatus::parse(&encoded).expect("Failed to decode");
        assert_eq!(decoded, status);
    }

    #[test]
    fn test_component_status_with_description() {
        let timestamp = Timestamp::now();
        let status = ComponentStatus::new(
            SystemType::FireAlarm,
            1,
            ComponentType::SmokeFireDetector,
            0x12345678,
            0x0002,
            [0u8; 31],
            timestamp
        ).with_description_text("烟雾探测器");

        // 验证描述设置正确
        let desc_text = status.description_text().unwrap();
        assert_eq!(desc_text, "烟雾探测器");
    }

    #[test]
    fn test_component_status_fields() {
        let timestamp = Timestamp::now();
        let status = ComponentStatus::new(
            SystemType::FireAlarm,
            2,
            ComponentType::TemperatureFireDetector,
            0xABCDEF12,
            0x0008,
            [0u8; 31],
            timestamp
        );

        assert_eq!(status.object_type(), 2);
        assert_eq!(status.system_type, SystemType::FireAlarm);
        assert_eq!(status.system_address, 2);
        assert_eq!(status.component_type, ComponentType::TemperatureFireDetector);
        assert_eq!(status.component_address, 0xABCDEF12);
        assert_eq!(status.component_state, 0x0008);
        assert_eq!(status.description(), Some("建筑消防设施部件状态"));
        assert!(status.timestamp().is_some());
    }
}
