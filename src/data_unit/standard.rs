//! GB26875 标准应用数据单元实现
//!
//! 包含 GB26875 协议中定义的所有标准应用数据单元类型，
//! 如系统状态、部件状态、模拟量值等。

use crate::error::{ParseError, ParseResult, EncodeError, EncodeResult};
use crate::data_unit::DataUnit;
use crate::protocol::{DataUnitType, SystemType, ComponentType as ProtocolComponentType, AnalogType};
use crate::frame::Timestamp;
use bytes::{Bytes, Buf, BufMut, BytesMut};

/// 系统状态数据单元（类型 1，4字节）
/// 
/// 用于上报系统的整体运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemStatus {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（3字节，小端序）
    pub system_address: u32, // 实际只使用低3字节
}

impl SystemStatus {
    /// 创建新的系统状态
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// * `system_address` - 系统地址（3字节）
    /// 
    /// # Returns
    /// * `Result<SystemStatus, EncodeError>` - 成功返回系统状态
    pub fn new(system_type: SystemType, system_address: u32) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
            });
        }

        Ok(SystemStatus {
            system_type,
            system_address,
        })
    }
}

impl DataUnit for SystemStatus {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(4);
        
        // 系统类型（1字节）
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址（3字节，小端序）
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() != 4 {
            return Err(ParseError::InvalidDataLength {
                expected: 4,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = u32::from_le_bytes([data[1], data[2], data[3], 0]);

        Ok(SystemStatus {
            system_type,
            system_address,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
            });
        }
        Ok(())
    }
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SystemStatus {{ type: {:?}, addr: 0x{:06X} }}", 
               self.system_type, self.system_address)
    }
}

/// 部件类型数据单元（类型 2，变长）
/// 
/// 用于上报部件的类型信息
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentType {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（3字节）
    pub system_address: u32,
    /// 部件类型
    pub component_type: ProtocolComponentType,
    /// 部件地址（4字节）
    pub component_address: u32,
}

impl ComponentType {
    /// 创建新的部件类型
    pub fn new(
        system_type: SystemType,
        system_address: u32,
        component_type: ProtocolComponentType,
        component_address: u32,
    ) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
            });
        }

        Ok(ComponentType {
            system_type,
            system_address,
            component_type,
            component_address,
        })
    }
}

impl DataUnit for ComponentType {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadComponentStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(9);
        
        // 系统类型（1字节）
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址（3字节，小端序）
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        // 部件类型（1字节）
        buf.put_u8(self.component_type.to_u8());
        
        // 部件地址（4字节，小端序）
        buf.put_u32_le(self.component_address);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() != 9 {
            return Err(ParseError::InvalidDataLength {
                expected: 9,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = u32::from_le_bytes([data[1], data[2], data[3], 0]);
        let component_type = ProtocolComponentType::from_u8(data[4]);
        let component_address = u32::from_le_bytes([data[5], data[6], data[7], data[8]]);

        Ok(ComponentType {
            system_type,
            system_address,
            component_type,
            component_address,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
            });
        }
        Ok(())
    }
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ComponentType {{ sys: {:?}@0x{:06X}, comp: {:?}@0x{:08X} }}",
            self.system_type, self.system_address,
            self.component_type, self.component_address
        )
    }
}

/// 部件状态数据单元（类型 3，40字节）
/// 
/// 用于上报部件的详细状态信息
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentStatus {
    /// 系统类型
    pub system_type: SystemType,
    /// 系统地址（3字节）
    pub system_address: u32,
    /// 部件类型
    pub component_type: ProtocolComponentType,
    /// 部件地址（4字节）
    pub component_address: u32,
    /// 部件状态（1字节）
    pub status: u8,
    /// 部件说明（31字节，GB2312编码）
    pub description: Vec<u8>,
}

impl ComponentStatus {
    /// 创建新的部件状态
    pub fn new(
        system_type: SystemType,
        system_address: u32,
        component_type: ProtocolComponentType,
        component_address: u32,
        status: u8,
        description: Vec<u8>,
    ) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
            });
        }

        if description.len() > 31 {
            return Err(EncodeError::InvalidValue {
                field: "description".to_string(),
                value: format!("{} bytes", description.len()),
            });
        }

        Ok(ComponentStatus {
            system_type,
            system_address,
            component_type,
            component_address,
            status,
            description,
        })
    }

    /// 使用字符串描述创建部件状态（自动GB2312编码）
    pub fn with_description_str(
        system_type: SystemType,
        system_address: u32,
        component_type: ProtocolComponentType,
        component_address: u32,
        status: u8,
        description: &str,
    ) -> EncodeResult<Self> {
        // 使用 GB2312 编码描述
        #[cfg(feature = "encoding")]
        let encoded = {
            use encoding_rs::GB18030; // GB18030 兼容 GB2312
            let (encoded, _, _) = GB18030.encode(description);
            encoded.into_owned()
        };
        
        #[cfg(not(feature = "encoding"))]
        let encoded = description.as_bytes().to_vec();

        Self::new(
            system_type,
            system_address,
            component_type,
            component_address,
            status,
            encoded,
        )
    }

    /// 获取描述字符串（自动GB2312解码）
    pub fn description_str(&self) -> Result<String, std::string::FromUtf8Error> {
        #[cfg(feature = "encoding")]
        {
            use encoding_rs::GB18030;
            let (decoded, _, _) = GB18030.decode(&self.description);
            Ok(decoded.into_owned())
        }
        
        #[cfg(not(feature = "encoding"))]
        String::from_utf8(self.description.clone())
    }
}

impl DataUnit for ComponentStatus {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadComponentStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(40);
        
        // 系统类型（1字节）
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址（3字节，小端序）
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        // 部件类型（1字节）
        buf.put_u8(self.component_type.to_u8());
        
        // 部件地址（4字节，小端序）
        buf.put_u32_le(self.component_address);
        
        // 部件状态（1字节）
        buf.put_u8(self.status);
        
        // 部件说明（31字节，填充0）
        let mut desc = self.description.clone();
        desc.resize(31, 0);
        buf.put_slice(&desc);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() != 40 {
            return Err(ParseError::InvalidDataLength {
                expected: 40,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = u32::from_le_bytes([data[1], data[2], data[3], 0]);
        let component_type = ProtocolComponentType::from_u8(data[4]);
        let component_address = u32::from_le_bytes([data[5], data[6], data[7], data[8]]);
        let status = data[9];
        
        // 读取描述并移除尾部的0
        let mut description = data[10..41].to_vec();
        while description.last() == Some(&0) {
            description.pop();
        }

        Ok(ComponentStatus {
            system_type,
            system_address,
            component_type,
            component_address,
            status,
            description,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
            });
        }
        
        if self.description.len() > 31 {
            return Err(ParseError::InvalidValue {
                field: "description".to_string(),
                value: format!("{} bytes", self.description.len()),
            });
        }
        
        Ok(())
    }
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = self.description_str().unwrap_or_else(|_| "<invalid encoding>".to_string());
        write!(
            f,
            "ComponentStatus {{ sys: {:?}@0x{:06X}, comp: {:?}@0x{:08X}, status: 0x{:02X}, desc: \"{}\" }}",
            self.system_type, self.system_address,
            self.component_type, self.component_address,
            self.status, desc
        )
    }
}

/// 模拟量值数据单元（类型 4，10字节）
/// 
/// 用于上报模拟量的当前值
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnalogValue {
    /// 系统类型（1字节）
    pub system_type: SystemType,
    /// 系统地址（1字节）
    pub system_address: u8,
    /// 部件类型（1字节）
    pub analog_type: AnalogType,
    /// 部件地址（4字节）
    pub unit_address: u32,
    /// 模拟量类型（1字节）
    pub analog_address: u8,
    /// 模拟量值（2字节）
    pub value: i16,
}

impl AnalogValue {
    /// 创建新的模拟量值
    pub fn new(
        system_type: SystemType,
        system_address: u8,
        analog_type: AnalogType,
        unit_address: u32,
        analog_address: u8,
        value: i16,
    ) -> EncodeResult<Self> {
        if system_address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: system_address.to_string(),
            });
        }

        Ok(AnalogValue {
            system_type,
            system_address,
            analog_type,
            unit_address,
            value,
            analog_address,
        })
    }
}

impl DataUnit for AnalogValue {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadAnalogValue
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(10);
        
        // 系统类型（1字节）
        buf.put_u8(self.system_type.to_u8());
        
        // 系统地址（3字节，小端序）
        buf.put_u8((self.system_address & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 8) & 0xFF) as u8);
        buf.put_u8(((self.system_address >> 16) & 0xFF) as u8);
        
        // 模拟量类型（1字节）
        buf.put_u8(self.analog_type.to_u8());
        
        // 模拟量地址（2字节，小端序）
        buf.put_u16_le(self.analog_address as u16);

        // 模拟量值（2字节，小端序整数）
        buf.put_i16_le(self.value);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() != 10 {
            return Err(ParseError::InvalidDataLength {
                expected: 10,
                actual: data.len(),
            });
        }

        let system_type = SystemType::from_u8(data[0]);
        let system_address = u32::from_le_bytes([data[1], data[2], data[3], 0]);
        let analog_type = AnalogType::from_u8(data[4]);
        let analog_address = u32::from_le_bytes([data[5], data[6], data[7], data[8]]);
        
        // 解析浮点值
        let value_bytes = [data[9], data[10], data[11], data[12]];
        let value = f32::from_le_bytes(value_bytes);

        Ok(AnalogValue {
            system_type,
            system_address,
            analog_type,
            analog_address,
            value,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.system_address > 0xFFFFFF {
            return Err(ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: self.system_address.to_string(),
            });
        }
        
        Ok(())
    }
}

impl std::fmt::Display for AnalogValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AnalogValue {{ sys: {:?}@0x{:06X}, analog: {:?}@0x{:08X}, value: {} }}",
            self.system_type, self.system_address,
            self.analog_type, self.analog_address,
            self.value
        )
    }
}

// 为了保持文件不过长，我将剩余的数据单元类型放在单独的模块中
// 或者继续在这里实现其他标准类型...

/// 操作信息数据单元（类型 5，变长 2-4字节）
/// 
/// 用于上报操作相关信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OperationInfo {
    /// 操作类型（1字节）
    pub operation_type: u8,
    /// 操作员编号（1字节）
    pub operator_id: u8,
    /// 扩展信息（0-2字节，可选）
    pub extended_info: Option<u16>,
}

impl OperationInfo {
    /// 创建新的操作信息
    pub fn new(operation_type: u8, operator_id: u8, extended_info: Option<u16>) -> Self {
        OperationInfo {
            operation_type,
            operator_id,
            extended_info,
        }
    }
}

impl DataUnit for OperationInfo {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadOperationInfo
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(4);
        
        buf.put_u8(self.operation_type);
        buf.put_u8(self.operator_id);
        
        if let Some(ext) = self.extended_info {
            buf.put_u16_le(ext);
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 2 || data.len() > 4 {
            return Err(ParseError::InvalidDataLength {
                expected: 2, // 或2-4的范围
                actual: data.len(),
            });
        }

        let operation_type = data[0];
        let operator_id = data[1];
        let extended_info = if data.len() >= 4 {
            Some(u16::from_le_bytes([data[2], data[3]]))
        } else {
            None
        };

        Ok(OperationInfo {
            operation_type,
            operator_id,
            extended_info,
        })
    }
}

impl std::fmt::Display for OperationInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.extended_info {
            Some(ext) => write!(f, "OperationInfo {{ type: 0x{:02X}, operator: {}, ext: 0x{:04X} }}", 
                              self.operation_type, self.operator_id, ext),
            None => write!(f, "OperationInfo {{ type: 0x{:02X}, operator: {} }}", 
                          self.operation_type, self.operator_id),
        }
    }
}

/// 软件版本数据单元（类型 6，变长）
/// 
/// 用于上报软件版本信息
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SoftwareVersion {
    /// 主版本号
    pub major: u8,
    /// 次版本号
    pub minor: u8,
    /// 修订版本号
    pub patch: u8,
    /// 版本字符串（可选，GB2312编码）
    pub version_string: Option<Vec<u8>>,
}

impl SoftwareVersion {
    /// 创建新的软件版本
    pub fn new(major: u8, minor: u8, patch: u8, version_string: Option<Vec<u8>>) -> Self {
        SoftwareVersion {
            major,
            minor,
            patch,
            version_string,
        }
    }

    /// 使用字符串创建软件版本
    pub fn with_string(major: u8, minor: u8, patch: u8, version_str: Option<&str>) -> EncodeResult<Self> {
        let version_string = if let Some(s) = version_str {
            #[cfg(feature = "encoding")]
            {
                use encoding_rs::GB18030;
                let (encoded, _, _) = GB18030.encode(s);
                Some(encoded.into_owned())
            }
            
            #[cfg(not(feature = "encoding"))]
            Some(s.as_bytes().to_vec())
        } else {
            None
        };

        Ok(SoftwareVersion {
            major,
            minor,
            patch,
            version_string,
        })
    }

    /// 获取版本字符串
    pub fn version_str(&self) -> Option<Result<String, std::string::FromUtf8Error>> {
        self.version_string.as_ref().map(|bytes| {
            #[cfg(feature = "encoding")]
            {
                use encoding_rs::GB18030;
                let (decoded, _, _) = GB18030.decode(bytes);
                Ok(decoded.into_owned())
            }
            
            #[cfg(not(feature = "encoding"))]
            String::from_utf8(bytes.clone())
        })
    }
}

impl DataUnit for SoftwareVersion {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSoftwareVersion
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let base_len = 3;
        let string_len = self.version_string.as_ref().map(|s| s.len()).unwrap_or(0);
        let mut buf = BytesMut::with_capacity(base_len + string_len);
        
        buf.put_u8(self.major);
        buf.put_u8(self.minor);
        buf.put_u8(self.patch);
        
        if let Some(ref s) = self.version_string {
            buf.put_slice(s);
        }
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 3 {
            return Err(ParseError::InvalidDataLength {
                expected: 3,
                actual: data.len(),
            });
        }

        let major = data[0];
        let minor = data[1];
        let patch = data[2];
        
        let version_string = if data.len() > 3 {
            Some(data[3..].to_vec())
        } else {
            None
        };

        Ok(SoftwareVersion {
            major,
            minor,
            patch,
            version_string,
        })
    }
}

impl std::fmt::Display for SoftwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.version_str() {
            Some(Ok(s)) => write!(f, "SoftwareVersion {{ {}.{}.{} \"{}\" }}", 
                                 self.major, self.minor, self.patch, s),
            Some(Err(_)) => write!(f, "SoftwareVersion {{ {}.{}.{} <invalid encoding> }}", 
                                  self.major, self.minor, self.patch),
            None => write!(f, "SoftwareVersion {{ {}.{}.{} }}", 
                          self.major, self.minor, self.patch),
        }
    }
}

/// 配置信息数据单元（类型 7，变长）
/// 
/// 用于上报或设置配置信息
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigInfo {
    /// 配置类型
    pub config_type: u8,
    /// 配置数据
    pub config_data: Vec<u8>,
}

impl ConfigInfo {
    /// 创建新的配置信息
    pub fn new(config_type: u8, config_data: Vec<u8>) -> Self {
        ConfigInfo {
            config_type,
            config_data,
        }
    }
}

impl DataUnit for ConfigInfo {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemConfig
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::with_capacity(1 + self.config_data.len());
        
        buf.put_u8(self.config_type);
        buf.put_slice(&self.config_data);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.is_empty() {
            return Err(ParseError::InvalidDataLength {
                expected: 1,
                actual: 0,
            });
        }

        let config_type = data[0];
        let config_data = data[1..].to_vec();

        Ok(ConfigInfo {
            config_type,
            config_data,
        })
    }
}

impl std::fmt::Display for ConfigInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigInfo {{ type: 0x{:02X}, data: {} bytes }}", 
               self.config_type, self.config_data.len())
    }
}

/// 时间数据单元（类型 8，6字节）
/// 
/// 用于时间同步
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Time {
    /// 时间标签
    pub timestamp: Timestamp,
}

impl Time {
    /// 创建新的时间数据单元
    pub fn new(timestamp: Timestamp) -> Self {
        Time { timestamp }
    }

    /// 使用当前时间创建
    pub fn now() -> Self {
        Time {
            timestamp: Timestamp::now(),
        }
    }
}

impl DataUnit for Time {    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemTime
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        Ok(self.timestamp.encode())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        let timestamp = Timestamp::parse(data)?;
        Ok(Time { timestamp })
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Time {{ {} }}", self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_status() {
        let status = SystemStatus::new(SystemType::FireAlarm, 0x123456).unwrap();
        let encoded = status.encode().unwrap();
        let decoded = SystemStatus::parse(&encoded).unwrap();
        
        assert_eq!(status, decoded);
        assert_eq!(encoded.len(), 4);
    }

    #[test]
    fn test_analog_value() {
        let value = AnalogValue::new(
            SystemType::AutoSprinkler,
            0x123456,
            AnalogType::Temperature,
            0x87654321,
            25.5,
        ).unwrap();
        
        let encoded = value.encode().unwrap();
        let decoded = AnalogValue::parse(&encoded).unwrap();
        
        assert_eq!(value.system_type, decoded.system_type);
        assert_eq!(value.system_address, decoded.system_address);
        assert_eq!(value.analog_type, decoded.analog_type);
        assert_eq!(value.analog_address, decoded.analog_address);
        assert!((value.value - decoded.value).abs() < f32::EPSILON);
    }

    #[test]
    fn test_component_status() {
        let status = ComponentStatus::with_description_str(
            SystemType::FireAlarm,
            0x123456,
            ProtocolComponentType::SmokeDetector,
            0x87654321,
            0x01,
            "测试部件",
        ).unwrap();
        
        let encoded = status.encode().unwrap();
        let decoded = ComponentStatus::parse(&encoded).unwrap();
        
        assert_eq!(status.system_type, decoded.system_type);
        assert_eq!(status.system_address, decoded.system_address);
        assert_eq!(status.component_type, decoded.component_type);
        assert_eq!(status.component_address, decoded.component_address);
        assert_eq!(status.status, decoded.status);
    }
}
