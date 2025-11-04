//! GB26875 上行数据单元实现 (类型 1-28)
//!
//! 包含从用户信息传输装置到监控中心的数据传输单元，
//! 按照GB26875协议8.3.1节的规定实现。

use crate::error::{ParseError, ParseResult, EncodeError, EncodeResult};
use crate::data_unit::DataUnit;
use crate::protocol::{DataUnitType, SystemType};
use crate::info_object::{SystemStatus as InfoSystemStatus, InfoObject};
use crate::frame::Timestamp;
use bytes::{Bytes, BufMut, BytesMut};

/// 上传建筑消防设施系统状态 (类型1)
/// 
/// 包含系统状态信息对象和时间标签
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UploadSystemStatus {
    /// 信息对象数量 (固定为1)
    pub object_count: u8,
    /// 系统状态信息对象
    pub system_status: InfoSystemStatus,
    /// 时间标签
    pub timestamp: Timestamp,
}

impl UploadSystemStatus {
    /// 创建新的上传系统状态数据单元
    pub fn new(system_status: InfoSystemStatus, timestamp: Timestamp) -> Self {
        Self {
            object_count: 1,
            system_status,
            timestamp,
        }
    }

    /// 获取系统状态信息对象的引用
    pub fn system_status(&self) -> &InfoSystemStatus {
        &self.system_status
    }

    /// 获取时间标签的引用
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl DataUnit for UploadSystemStatus {
    fn data_unit_type(&self) -> DataUnitType {
        DataUnitType::UploadSystemStatus
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // 信息对象数量 (1字节)
        buf.put_u8(self.object_count);
        
        // 系统状态信息对象
        let info_bytes = self.system_status.encode()?;
        buf.extend_from_slice(&info_bytes);
        
        // 时间标签
        let time_bytes = self.timestamp.encode()?;
        buf.extend_from_slice(&time_bytes);
        
        Ok(buf.freeze())
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 1 + 10 + 6 { // 至少需要1+10+6=17字节 (系统状态是10字节)
            return Err(ParseError::InsufficientData {
                expected: 17,
                actual: data.len(),
            });
        }

        let mut offset = 0;
        
        // 信息对象数量
        let object_count = data[offset];
        offset += 1;

        if object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: object_count.to_string(),
                reason: "System status should have exactly 1 object".to_string(),
            });
        }

        // 系统状态信息对象 (10字节)
        let system_status = InfoSystemStatus::parse(&data[offset..offset+10])?;
        offset += 10;

        // 时间标签 (6字节)
        let timestamp = Timestamp::parse(&data[offset..offset+6])?;

        Ok(Self {
            object_count,
            system_status,
            timestamp,
        })
    }

    fn validate(&self) -> ParseResult<()> {
        if self.object_count != 1 {
            return Err(ParseError::InvalidValue {
                field: "object_count".to_string(),
                value: self.object_count.to_string(),
                reason: "System status should have exactly 1 object".to_string(),
            });
        }

        // Note: InfoObject trait doesn't have validate method in current implementation
        // We rely on the individual field validation during construction
        Ok(())
    }
}

// TODO: 继续实现其余的上行数据单元类型 (2-28)
// 当前专注于建立可工作的基础架构

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_system_status_basic() {
        // 基本功能测试，先验证数据单元类型
        let system_status = InfoSystemStatus {
            system_type: SystemType::FireAlarm,
            system_address: 0x12,
            system_state: 0x1234,
            timestamp: Timestamp::new(2024, 11, 4, 15, 30, 45).unwrap(),
        };
        
        let timestamp = Timestamp::new(2024, 11, 4, 15, 30, 45).unwrap();
        
        let upload = UploadSystemStatus::new(system_status, timestamp);
        
        // 验证基本属性
        assert_eq!(upload.data_unit_type(), DataUnitType::UploadSystemStatus);
        assert_eq!(upload.object_count, 1);
        assert_eq!(upload.system_status.system_type, SystemType::FireAlarm);
        assert_eq!(upload.system_status.system_address, 0x12);
        assert_eq!(upload.system_status.system_state, 0x1234);
    }
}
