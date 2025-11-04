//! GB26875 数据单元构建器
//!
//! 提供友好的数据单元构建 API

use crate::builder::{Builder, ResettableBuilder};
use crate::error::{EncodeError, EncodeResult};
use crate::data_unit::standard::*;
use crate::protocol::{SystemType, ComponentType, AnalogType};

/// 系统状态构建器
#[derive(Debug, Clone, Default)]
pub struct SystemStatusBuilder {
    system_type: Option<SystemType>,
    system_address: Option<u32>,
}

impl SystemStatusBuilder {
    /// 创建新的系统状态构建器
    pub fn new() -> Self {
        SystemStatusBuilder {
            system_type: None,
            system_address: None,
        }
    }

    /// 设置系统类型
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn system_type(mut self, system_type: SystemType) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// 设置系统地址
    /// 
    /// # Arguments
    /// * `address` - 系统地址（3字节）
    /// 
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn system_address(mut self, address: u32) -> EncodeResult<Self> {
        if address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: format!("0x{:X}", address),
                reason: "系统地址必须在 3 字节范围内".to_string(),
            });
        }
        self.system_address = Some(address);
        Ok(self)
    }

    /// 使用火灾报警系统类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn fire_alarm_system(self) -> Self {
        self.system_type(SystemType::FireAlarm)
    }

    /// 使用自动喷水灭火系统类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn auto_sprinkler_system(self) -> Self {
        self.system_type(SystemType::AutoSprinkler)
    }
}

impl Builder<SystemStatus> for SystemStatusBuilder {
    fn build(self) -> EncodeResult<SystemStatus> {
        self.validate()?;
        SystemStatus::new(self.system_type.unwrap(), self.system_address.unwrap())
    }

    fn validate(&self) -> EncodeResult<()> {
        if self.system_type.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "system_type".to_string(),
                value: "None".to_string(),
                reason: "系统类型不能为空".to_string(),
            });
        }

        if self.system_address.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: "None".to_string(),
                reason: "系统地址不能为空".to_string(),
            });
        }

        Ok(())
    }
}

impl ResettableBuilder<SystemStatus> for SystemStatusBuilder {
    fn reset(&mut self) {
        self.system_type = None;
        self.system_address = None;
    }
}

/// 部件状态构建器
#[derive(Debug, Clone, Default)]
pub struct ComponentStatusBuilder {
    system_type: Option<SystemType>,
    system_address: Option<u8>, // 改为u8
    component_type: Option<ComponentType>,
    component_address: Option<u32>,
    status: Option<u16>, // 改为u16
    description: Option<Vec<u8>>,
}

impl ComponentStatusBuilder {
    /// 创建新的部件状态构建器
    pub fn new() -> Self {
        ComponentStatusBuilder {
            system_type: None,
            system_address: None,
            component_type: None,
            component_address: None,
            status: None,
            description: None,
        }
    }

    /// 设置系统类型
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn system_type(mut self, system_type: SystemType) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// 设置系统地址
    /// 
    /// # Arguments
    /// * `address` - 系统地址（1字节）
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn system_address(mut self, address: u8) -> Self {
        self.system_address = Some(address);
        self
    }

    /// 设置部件类型
    /// 
    /// # Arguments
    /// * `component_type` - 部件类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn component_type(mut self, component_type: ComponentType) -> Self {
        self.component_type = Some(component_type);
        self
    }

    /// 设置部件地址
    /// 
    /// # Arguments
    /// * `address` - 部件地址（4字节）
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn component_address(mut self, address: u32) -> Self {
        self.component_address = Some(address);
        self
    }

    /// 设置部件状态
    /// 
    /// # Arguments
    /// * `status` - 部件状态（2字节）
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    /// 设置部件描述（字符串）
    /// 
    /// # Arguments
    /// * `description` - 部件描述字符串
    /// 
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn description_str(mut self, description: &str) -> EncodeResult<Self> {
        #[cfg(feature = "encoding")]
        let encoded = {
            use encoding_rs::GB18030;
            let (encoded, _, _) = GB18030.encode(description);
            encoded.into_owned()
        };
        
        #[cfg(not(feature = "encoding"))]
        let encoded = description.as_bytes().to_vec();

        if encoded.len() > 31 {
            return Err(EncodeError::InvalidValue {
                field: "description".to_string(),
                value: format!("{} bytes", encoded.len()),
                reason: "部件描述长度不能超过 31 字节".to_string(),
            });
        }

        self.description = Some(encoded);
        Ok(self)
    }

    /// 设置部件描述（原始字节）
    /// 
    /// # Arguments
    /// * `description` - 部件描述字节数据
    /// 
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn description_bytes(mut self, description: Vec<u8>) -> EncodeResult<Self> {
        if description.len() > 31 {
            return Err(EncodeError::InvalidValue {
                field: "description".to_string(),
                value: format!("{} bytes", description.len()),
                reason: "部件描述长度不能超过 31 字节".to_string(),
            });
        }
        self.description = Some(description);
        Ok(self)
    }

    /// 设置为正常状态
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn normal_status(self) -> Self {
        self.status(0x00)
    }

    /// 设置为报警状态
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn alarm_status(self) -> Self {
        self.status(0x01)
    }

    /// 设置为故障状态
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn fault_status(self) -> Self {
        self.status(0x02)
    }

    /// 设置为烟雾探测器类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn smoke_detector(self) -> Self {
        self.component_type(ComponentType::SmokeFireDetector)
    }

    /// 设置为温度探测器类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn temperature_detector(self) -> Self {
        self.component_type(ComponentType::TemperatureFireDetector)
    }
}

impl Builder<ComponentStatus> for ComponentStatusBuilder {
    fn build(self) -> EncodeResult<ComponentStatus> {
        self.validate()?;
        ComponentStatus::new(
            self.system_type.unwrap(),
            self.system_address.unwrap(),
            self.component_type.unwrap(),
            self.component_address.unwrap(),
            self.status.unwrap(),
            self.description.unwrap_or_default(),
        )
    }

    fn validate(&self) -> EncodeResult<()> {
        if self.system_type.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "system_type".to_string(),
                value: "None".to_string(),
                reason: "系统类型不能为空".to_string(),
            });
        }

        if self.system_address.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: "None".to_string(),
                reason: "系统地址不能为空".to_string(),
            });
        }

        if self.component_type.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "component_type".to_string(),
                value: "None".to_string(),
                reason: "部件类型不能为空".to_string(),
            });
        }

        if self.component_address.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "component_address".to_string(),
                value: "None".to_string(),
                reason: "部件地址不能为空".to_string(),
            });
        }

        if self.status.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "status".to_string(),
                value: "None".to_string(),
                reason: "部件状态不能为空".to_string(),
            });
        }

        Ok(())
    }
}

impl ResettableBuilder<ComponentStatus> for ComponentStatusBuilder {
    fn reset(&mut self) {
        self.system_type = None;
        self.system_address = None;
        self.component_type = None;
        self.component_address = None;
        self.status = None;
        self.description = None;
    }
}

/// 模拟量值构建器
#[derive(Debug, Clone, Default)]
pub struct AnalogValueBuilder {
    system_type: Option<SystemType>,
    system_address: Option<u32>,
    component_type: Option<ComponentType>,
    component_address: Option<u32>,
    analog_type: Option<AnalogType>,
    value: Option<f32>,
}

impl AnalogValueBuilder {
    /// 创建新的模拟量值构建器
    pub fn new() -> Self {
        AnalogValueBuilder {
            system_type: None,
            system_address: None,
            component_type: None,
            component_address: None,
            analog_type: None,
            value: None,
        }
    }

    /// 设置系统类型
    /// 
    /// # Arguments
    /// * `system_type` - 系统类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn system_type(mut self, system_type: SystemType) -> Self {
        self.system_type = Some(system_type);
        self
    }

    /// 设置系统地址
    /// 
    /// # Arguments
    /// * `address` - 系统地址（3字节）
    /// 
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn system_address(mut self, address: u32) -> EncodeResult<Self> {
        if address > 0xFFFFFF {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: format!("0x{:X}", address),
                reason: "系统地址必须在 3 字节范围内".to_string(),
            });
        }
        self.system_address = Some(address);        Ok(self)
    }    /// 设置部件类型
    /// 
    /// # Arguments
    /// * `component_type` - 部件类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn component_type(mut self, component_type: ComponentType) -> Self {
        self.component_type = Some(component_type);
        self
    }

    /// 设置部件地址
    /// 
    /// # Arguments
    /// * `address` - 部件地址（4字节）
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn component_address(mut self, address: u32) -> Self {
        self.component_address = Some(address);
        self
    }

    /// 设置模拟量类型
    /// 
    /// # Arguments
    /// * `analog_type` - 模拟量类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn analog_type(mut self, analog_type: AnalogType) -> Self {
        self.analog_type = Some(analog_type);
        self
    }    /// 设置模拟量值
    /// 
    /// # Arguments
    /// * `value` - 模拟量值
    /// 
    /// # Returns
    /// * `Result<Self, EncodeError>` - 成功返回构建器实例
    pub fn value(mut self, value: f32) -> EncodeResult<Self> {
        if !value.is_finite() {
            return Err(EncodeError::InvalidValue {
                field: "value".to_string(),
                value: value.to_string(),
                reason: "模拟量值必须是有限数值".to_string(),
            });
        }
        self.value = Some(value);
        Ok(self)
    }    /// 设置为温度类型
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn temperature(self) -> Self {
        self.analog_type(AnalogType::Temperature)
    }

    /// 设置为压力类型（MPa）
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn pressure_mpa(self) -> Self {
        self.analog_type(AnalogType::PressureMPa)
    }

    /// 设置为压力类型（kPa）
    /// 
    /// # Returns
    /// * `Self` - 构建器实例
    pub fn pressure_kpa(self) -> Self {
        self.analog_type(AnalogType::PressureKPa)
    }
}

impl Builder<AnalogValue> for AnalogValueBuilder {    fn build(self) -> EncodeResult<AnalogValue> {
        self.validate()?;
        AnalogValue::new(
            self.system_type.unwrap(),
            self.system_address.unwrap() as u8,
            self.component_type.unwrap(),
            self.component_address.unwrap(),
            self.analog_type.unwrap(),
            self.value.unwrap().round() as i16,
        )
    }

    fn validate(&self) -> EncodeResult<()> {
        if self.system_type.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "system_type".to_string(),
                value: "None".to_string(),
                reason: "系统类型不能为空".to_string(),
            });
        }

        if self.system_address.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "system_address".to_string(),
                value: "None".to_string(),
                reason: "系统地址不能为空".to_string(),
            });
        }        if self.analog_type.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "analog_type".to_string(),
                value: "None".to_string(),
                reason: "模拟量类型不能为空".to_string(),
            });
        }

        if self.component_type.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "component_type".to_string(),
                value: "None".to_string(),
                reason: "部件类型不能为空".to_string(),
            });
        }

        if self.component_address.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "component_address".to_string(),
                value: "None".to_string(),
                reason: "部件地址不能为空".to_string(),
            });
        }

        if self.value.is_none() {
            return Err(EncodeError::InvalidValue {
                field: "value".to_string(),
                value: "None".to_string(),
                reason: "模拟量值不能为空".to_string(),
            });
        }

        Ok(())
    }
}

impl ResettableBuilder<AnalogValue> for AnalogValueBuilder {    fn reset(&mut self) {
        self.system_type = None;
        self.system_address = None;
        self.component_type = None;
        self.component_address = None;
        self.analog_type = None;
        self.value = None;
    }
}

/// 通用数据单元构建器
#[derive(Debug, Clone)]
pub enum DataUnitBuilder {
    /// 系统状态构建器
    SystemStatus(SystemStatusBuilder),
    /// 部件状态构建器
    ComponentStatus(ComponentStatusBuilder),
    /// 模拟量值构建器
    AnalogValue(AnalogValueBuilder),
}

impl DataUnitBuilder {
    /// 创建系统状态构建器
    pub fn system_status() -> SystemStatusBuilder {
        SystemStatusBuilder::new()
    }

    /// 创建部件状态构建器
    pub fn component_status() -> ComponentStatusBuilder {
        ComponentStatusBuilder::new()
    }

    /// 创建模拟量值构建器
    pub fn analog_value() -> AnalogValueBuilder {
        AnalogValueBuilder::new()
    }
}

/// 便捷宏：创建系统状态
/// 
/// # Example
/// ```rust
/// use gb26875::system_status;
/// use gb26875::protocol::SystemType;
/// 
/// let status = system_status!(FireAlarm, 0x123456);
/// ```
#[macro_export]
macro_rules! system_status {
    ($system_type:expr, $address:expr) => {
        $crate::builder::SystemStatusBuilder::new()
            .system_type($system_type)
            .system_address($address)
            .map(|b| b.build())
    };
}

/// 便捷宏：创建部件状态
/// 
/// # Example
/// ```rust
/// use gb26875::component_status;
/// use gb26875::protocol::{SystemType, ComponentType};
/// 
/// let status = component_status!(
///     FireAlarm, 0x123456,
///     SmokeDetector, 0x87654321,
///     0x01, "烟雾探测器"
/// );
/// ```
#[macro_export]
macro_rules! component_status {
    ($sys_type:expr, $sys_addr:expr, $comp_type:expr, $comp_addr:expr, $status:expr, $desc:expr) => {
        $crate::builder::ComponentStatusBuilder::new()
            .system_type($sys_type)
            .system_address($sys_addr)
            .and_then(|b| b.component_type($comp_type).component_address($comp_addr).status($status).description_str($desc))
            .map(|b| b.build())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_status_builder() {
        let status = SystemStatusBuilder::new()
            .fire_alarm_system()
            .system_address(0x123456).unwrap()
            .build()
            .unwrap();

        assert_eq!(status.system_type, SystemType::FireAlarm);
        assert_eq!(status.system_address, 0x123456);
    }

    #[test]
    fn test_component_status_builder() {
        let status = ComponentStatusBuilder::new()
            .system_type(SystemType::FireAlarm)
            .system_address(0x12)
            .smoke_detector()
            .component_address(0x87654321)
            .alarm_status()
            .description_str("烟雾探测器").unwrap()
            .build()
            .unwrap();        
        assert_eq!(status.system_type, SystemType::FireAlarm);
        assert_eq!(status.component_type, ComponentType::SmokeFireDetector);
        assert_eq!(status.status, 0x01);
    }    
    #[test]
    fn test_analog_value_builder() {
        let value = AnalogValueBuilder::new()
            .system_type(SystemType::AutoSprinkler)
            .system_address(0x123456).unwrap()
            .component_type(ComponentType::SmokeFireDetector)
            .component_address(0x87654321)
            .temperature()
            .value(25.5).unwrap()
            .build()
            .unwrap();

        assert_eq!(value.system_type, SystemType::AutoSprinkler);
        assert_eq!(value.component_type, ComponentType::SmokeFireDetector);
        assert_eq!(value.analog_type, AnalogType::Temperature);
        assert_eq!(value.value, 26); // i16 值，因为25.5会被转换为26
    }

    #[test]
    fn test_builder_validation() {
        let builder = SystemStatusBuilder::new();
        assert!(builder.validate().is_err());

        let builder = SystemStatusBuilder::new().fire_alarm_system();
        assert!(builder.validate().is_err()); // 缺少地址

        let result = SystemStatusBuilder::new()
            .fire_alarm_system()
            .system_address(0xFFFFFF + 1); // 地址过大

        assert!(result.is_err());
    }

    #[test]
    fn test_resettable_builder() {
        let mut builder = SystemStatusBuilder::new()
            .fire_alarm_system()
            .system_address(0x123456).unwrap();

        let status = builder.build_and_reset().unwrap();
        assert_eq!(status.system_type, SystemType::FireAlarm);

        // 构建器应该被重置
        assert!(builder.validate().is_err());
    }
}
