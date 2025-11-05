//! GB26875 数据单元构建器
//!
//! 提供友好的数据单元构建 API，支持链式调用和类型安全的构建过程。
//!
//! ## 示例
//!
//! ```rust
//! use gb26875::builder::data_unit::DataUnitBuilder;
//! use gb26875::protocol::types::{SystemType, ComponentType, DataUnitType};
//! use gb26875::frame::Timestamp;
//!
//! // 构建上传系统状态数据单元
//! let data_unit = DataUnitBuilder::new(DataUnitType::UploadSystemStatus)
//!     .system_status()
//!     .system_type(SystemType::FireAlarm)
//!     .system_address(1)
//!     .system_state(0x0002)
//!     .timestamp(Timestamp::now())
//!     .build()?;
//!
//! // 构建上传部件状态数据单元
//! let data_unit = DataUnitBuilder::new(DataUnitType::UploadComponentStatus)
//!     .component_status()
//!     .system_type(SystemType::FireAlarm)
//!     .system_address(1)
//!     .component_type(ComponentType::SmokeFireDetector)
//!     .component_address(0x12345678)
//!     .component_state(0x0002)
//!     .description("一层大厅烟雾探测器")
//!     .timestamp(Timestamp::now())
//!     .build()?;
//! ```

use crate::data_unit::standard::upstream;
use crate::data_unit::GenericDataUnit;
use crate::error::{ParseError, ParseResult};
use crate::frame::Timestamp;
use crate::info_object::{
    analog_value::AnalogType, AnalogValue as InfoAnalogValue,
    ComponentStatus as InfoComponentStatus, SystemStatus as InfoSystemStatus,
};
use crate::protocol::types::{ComponentType, DataUnitType, SystemType};

/// 数据单元构建器入口
pub struct DataUnitBuilder {
    data_unit_type: DataUnitType,
}

impl DataUnitBuilder {
    /// 创建新的数据单元构建器
    pub fn new(data_unit_type: DataUnitType) -> Self {
        Self { data_unit_type }
    }

    /// 开始构建系统状态数据单元
    pub fn system_status(self) -> SystemStatusBuilder {
        SystemStatusBuilder::new(self.data_unit_type)
    }

    /// 开始构建部件状态数据单元
    pub fn component_status(self) -> ComponentStatusBuilder {
        ComponentStatusBuilder::new(self.data_unit_type)
    }

    /// 开始构建模拟量值数据单元
    pub fn analog_value(self) -> AnalogValueBuilder {
        AnalogValueBuilder::new(self.data_unit_type)
    }

    /// 开始构建操作信息数据单元
    pub fn operation_info(self) -> OperationInfoBuilder {
        OperationInfoBuilder::new(self.data_unit_type)
    }

    /// 开始构建版本信息数据单元
    pub fn version_info(self) -> VersionInfoBuilder {
        VersionInfoBuilder::new(self.data_unit_type)
    }

    /// 开始构建配置信息数据单元
    pub fn config_info(self) -> ConfigInfoBuilder {
        ConfigInfoBuilder::new(self.data_unit_type)
    }

    /// 开始构建时间信息数据单元
    pub fn time_info(self) -> TimeInfoBuilder {
        TimeInfoBuilder::new(self.data_unit_type)
    }
}

/// 系统状态构建器
pub struct SystemStatusBuilder {
    data_unit_type: DataUnitType,
    system_type: Option<SystemType>,
    system_address: Option<u8>,
    system_state: Option<u16>,
    timestamp: Option<Timestamp>,
}

impl SystemStatusBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self {
            data_unit_type,
            system_type: None,
            system_address: None,
            system_state: None,
            timestamp: None,
        }
    }

    /// 设置系统类型
    pub fn system_type(mut self, system_type: SystemType) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// 设置系统地址
    pub fn system_address(mut self, address: u8) -> Self {
        self.system_address = Some(address);
        self
    }

    /// 设置系统状态
    pub fn system_state(mut self, state: u16) -> Self {
        self.system_state = Some(state);
        self
    }

    /// 设置时间戳
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// 构建数据单元
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        let system_type = self.system_type.ok_or_else(|| ParseError::InvalidValue {
            field: "system_type".to_string(),
            value: "None".to_string(),
            reason: "System type is required".to_string(),
        })?;

        let system_address = self
            .system_address
            .ok_or_else(|| ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: "None".to_string(),
                reason: "System address is required".to_string(),
            })?;

        let system_state = self.system_state.ok_or_else(|| ParseError::InvalidValue {
            field: "system_state".to_string(),
            value: "None".to_string(),
            reason: "System state is required".to_string(),
        })?;

        let timestamp = self.timestamp.ok_or_else(|| ParseError::InvalidValue {
            field: "timestamp".to_string(),
            value: "None".to_string(),
            reason: "Timestamp is required".to_string(),
        })?;
        let system_status =
            InfoSystemStatus::new(system_type, system_address, system_state, timestamp);

        match self.data_unit_type {
            DataUnitType::UploadSystemStatus => Ok(GenericDataUnit::UploadSystemStatus(
                upstream::UploadSystemStatus::new(system_status, timestamp),
            )),
            _ => Err(ParseError::InvalidValue {
                field: "data_unit_type".to_string(),
                value: format!("{:?}", self.data_unit_type),
                reason: "Incompatible data unit type for system status".to_string(),
            }),
        }
    }
}

/// 部件状态构建器
pub struct ComponentStatusBuilder {
    data_unit_type: DataUnitType,
    system_type: Option<SystemType>,
    system_address: Option<u8>,
    component_type: Option<ComponentType>,
    component_address: Option<u32>,
    component_state: Option<u16>,
    component_description: [u8; 31],
    timestamp: Option<Timestamp>,
}

impl ComponentStatusBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self {
            data_unit_type,
            system_type: None,
            system_address: None,
            component_type: None,
            component_address: None,
            component_state: None,
            component_description: [0u8; 31],
            timestamp: None,
        }
    }

    /// 设置系统类型
    pub fn system_type(mut self, system_type: SystemType) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// 设置系统地址
    pub fn system_address(mut self, address: u8) -> Self {
        self.system_address = Some(address);
        self
    }

    /// 设置部件类型
    pub fn component_type(mut self, component_type: ComponentType) -> Self {
        self.component_type = Some(component_type);
        self
    }

    /// 设置部件地址
    pub fn component_address(mut self, address: u32) -> Self {
        self.component_address = Some(address);
        self
    }

    /// 设置部件状态
    pub fn component_state(mut self, state: u16) -> Self {
        self.component_state = Some(state);
        self
    }

    /// 设置部件描述文本
    pub fn description(mut self, text: &str) -> Self {
        let bytes = text.as_bytes();
        let copy_len = bytes.len().min(31);
        self.component_description[..copy_len].copy_from_slice(&bytes[..copy_len]);
        self
    }

    /// 设置部件描述字节数组
    pub fn description_bytes(mut self, desc: [u8; 31]) -> Self {
        self.component_description = desc;
        self
    }

    /// 设置时间戳
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// 构建数据单元
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        let system_type = self.system_type.ok_or_else(|| ParseError::InvalidValue {
            field: "system_type".to_string(),
            value: "None".to_string(),
            reason: "System type is required".to_string(),
        })?;

        let system_address = self
            .system_address
            .ok_or_else(|| ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: "None".to_string(),
                reason: "System address is required".to_string(),
            })?;

        let component_type = self
            .component_type
            .ok_or_else(|| ParseError::InvalidValue {
                field: "component_type".to_string(),
                value: "None".to_string(),
                reason: "Component type is required".to_string(),
            })?;

        let component_address = self
            .component_address
            .ok_or_else(|| ParseError::InvalidValue {
                field: "component_address".to_string(),
                value: "None".to_string(),
                reason: "Component address is required".to_string(),
            })?;

        let component_state = self
            .component_state
            .ok_or_else(|| ParseError::InvalidValue {
                field: "component_state".to_string(),
                value: "None".to_string(),
                reason: "Component state is required".to_string(),
            })?;

        let timestamp = self.timestamp.ok_or_else(|| ParseError::InvalidValue {
            field: "timestamp".to_string(),
            value: "None".to_string(),
            reason: "Timestamp is required".to_string(),
        })?;

        let component_status = InfoComponentStatus::new(
            system_type,
            system_address,
            component_type,
            component_address,
            component_state,
            self.component_description,
            timestamp,
        );
        match self.data_unit_type {
            DataUnitType::UploadComponentStatus => Ok(GenericDataUnit::UploadComponentStatus(
                upstream::UploadComponentStatus::new(component_status, timestamp),
            )),
            _ => Err(ParseError::InvalidValue {
                field: "data_unit_type".to_string(),
                value: format!("{:?}", self.data_unit_type),
                reason: "Incompatible data unit type for component status".to_string(),
            }),
        }
    }
}

/// 模拟量值构建器
pub struct AnalogValueBuilder {
    data_unit_type: DataUnitType,
    system_type: Option<SystemType>,
    system_address: Option<u8>,
    component_type: Option<ComponentType>,
    component_address: Option<u32>,
    analog_type: Option<AnalogType>,
    analog_value: Option<i16>,
    timestamp: Option<Timestamp>,
}

impl AnalogValueBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self {
            data_unit_type,
            system_type: None,
            system_address: None,
            component_type: None,
            component_address: None,
            analog_type: None,
            analog_value: None,
            timestamp: None,
        }
    }

    /// 设置系统类型
    pub fn system_type(mut self, system_type: SystemType) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// 设置系统地址
    pub fn system_address(mut self, address: u8) -> Self {
        self.system_address = Some(address);
        self
    }

    /// 设置部件类型
    pub fn component_type(mut self, component_type: ComponentType) -> Self {
        self.component_type = Some(component_type);
        self
    }

    /// 设置部件地址
    pub fn component_address(mut self, address: u32) -> Self {
        self.component_address = Some(address);
        self
    }

    /// 设置模拟量类型
    pub fn analog_type(mut self, analog_type: AnalogType) -> Self {
        self.analog_type = Some(analog_type);
        self
    }

    /// 设置模拟量值
    pub fn analog_value(mut self, value: i16) -> Self {
        self.analog_value = Some(value);
        self
    }

    /// 设置时间戳
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// 构建数据单元
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        let system_type = self.system_type.ok_or_else(|| ParseError::InvalidValue {
            field: "system_type".to_string(),
            value: "None".to_string(),
            reason: "System type is required".to_string(),
        })?;

        let system_address = self
            .system_address
            .ok_or_else(|| ParseError::InvalidValue {
                field: "system_address".to_string(),
                value: "None".to_string(),
                reason: "System address is required".to_string(),
            })?;

        let component_type = self
            .component_type
            .ok_or_else(|| ParseError::InvalidValue {
                field: "component_type".to_string(),
                value: "None".to_string(),
                reason: "Component type is required".to_string(),
            })?;

        let component_address = self
            .component_address
            .ok_or_else(|| ParseError::InvalidValue {
                field: "component_address".to_string(),
                value: "None".to_string(),
                reason: "Component address is required".to_string(),
            })?;

        let analog_type = self.analog_type.ok_or_else(|| ParseError::InvalidValue {
            field: "analog_type".to_string(),
            value: "None".to_string(),
            reason: "Analog type is required".to_string(),
        })?;

        let analog_value = self.analog_value.ok_or_else(|| ParseError::InvalidValue {
            field: "analog_value".to_string(),
            value: "None".to_string(),
            reason: "Analog value is required".to_string(),
        })?;

        let timestamp = self.timestamp.ok_or_else(|| ParseError::InvalidValue {
            field: "timestamp".to_string(),
            value: "None".to_string(),
            reason: "Timestamp is required".to_string(),
        })?;

        let analog_val = InfoAnalogValue::new(
            system_type,
            system_address,
            component_type,
            component_address,
            analog_type,
            analog_value,
            timestamp,
        );
        match self.data_unit_type {
            DataUnitType::UploadAnalogValue => Ok(GenericDataUnit::UploadAnalogValue(
                upstream::UploadAnalogValue::new(analog_val, timestamp),
            )),
            _ => Err(ParseError::InvalidValue {
                field: "data_unit_type".to_string(),
                value: format!("{:?}", self.data_unit_type),
                reason: "Incompatible data unit type for analog value".to_string(),
            }),
        }
    }
}

// 占位符构建器 - 这些需要根据实际的信息对象API进行实现

/// 操作信息构建器
pub struct OperationInfoBuilder {
    data_unit_type: DataUnitType,
}

impl OperationInfoBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self { data_unit_type }
    }

    /// 构建数据单元 (占位符实现)
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        Err(ParseError::InvalidValue {
            field: "operation_info".to_string(),
            value: "builder".to_string(),
            reason: "OperationInfo builder not yet implemented".to_string(),
        })
    }
}

/// 版本信息构建器
pub struct VersionInfoBuilder {
    data_unit_type: DataUnitType,
}

impl VersionInfoBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self { data_unit_type }
    }

    /// 构建数据单元 (占位符实现)
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        Err(ParseError::InvalidValue {
            field: "version_info".to_string(),
            value: "builder".to_string(),
            reason: "VersionInfo builder not yet implemented".to_string(),
        })
    }
}

/// 配置信息构建器
pub struct ConfigInfoBuilder {
    data_unit_type: DataUnitType,
}

impl ConfigInfoBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self { data_unit_type }
    }

    /// 构建数据单元 (占位符实现)
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        Err(ParseError::InvalidValue {
            field: "config_info".to_string(),
            value: "builder".to_string(),
            reason: "ConfigInfo builder not yet implemented".to_string(),
        })
    }
}

/// 时间信息构建器
pub struct TimeInfoBuilder {
    data_unit_type: DataUnitType,
}

impl TimeInfoBuilder {
    fn new(data_unit_type: DataUnitType) -> Self {
        Self { data_unit_type }
    }

    /// 构建数据单元 (占位符实现)
    pub fn build(self) -> ParseResult<GenericDataUnit> {
        Err(ParseError::InvalidValue {
            field: "time_info".to_string(),
            value: "builder".to_string(),
            reason: "TimeInfo builder not yet implemented".to_string(),
        })
    }
}
