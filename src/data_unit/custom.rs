//! GB26875 自定义数据单元抽象
//!
//! 提供用户自定义数据单元的基础框架和扩展机制，
//! 支持 128-254 范围内的自定义数据单元类型。

use crate::error::{ParseResult, EncodeResult};
use crate::protocol::DataUnitType;
use crate::data_unit::DataUnit;
use bytes::Bytes;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 自定义数据单元 trait
/// 
/// 用户自定义数据单元需要实现此 trait 以提供类型安全的
/// 编解码功能和动态类型转换能力。
pub trait CustomDataUnit: DataUnit + Send + Sync + 'static {
    /// 获取自定义数据单元的名称
    fn name(&self) -> &str;
    
    /// 获取自定义数据单元的版本
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    /// 获取自定义数据单元的描述
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// 转换为 Any trait object，用于动态类型转换
    fn as_any(&self) -> &dyn Any;
    
    /// 克隆自身为 Box<dyn CustomDataUnit>
    fn clone_box(&self) -> Box<dyn CustomDataUnit>;
}

/// 自定义数据单元工厂 trait
/// 
/// 用于注册和创建自定义数据单元实例
pub trait CustomDataUnitFactory: Send + Sync + 'static {
    /// 获取工厂支持的数据单元类型
    fn data_unit_type(&self) -> DataUnitType;
    
    /// 从字节数据创建自定义数据单元实例
    fn create_from_bytes(&self, data: &[u8]) -> ParseResult<Box<dyn CustomDataUnit>>;
    
    /// 获取工厂名称
    fn name(&self) -> &str;
    
    /// 获取工厂描述
    fn description(&self) -> Option<&str> {
        None
    }
}

/// 自定义数据单元注册表
/// 
/// 全局注册表，用于管理用户自定义数据单元类型的注册和查找
pub struct CustomDataUnitRegistry {
    factories: RwLock<HashMap<u8, Arc<dyn CustomDataUnitFactory>>>,
}

impl CustomDataUnitRegistry {
    /// 创建新的注册表实例
    fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
        }
    }

    /// 获取全局注册表实例
    pub fn global() -> &'static Self {
        static INSTANCE: once_cell::sync::Lazy<CustomDataUnitRegistry> = 
            once_cell::sync::Lazy::new(|| CustomDataUnitRegistry::new());
        &*INSTANCE
    }

    /// 注册自定义数据单元工厂
    /// 
    /// # Arguments
    /// * `factory` - 自定义数据单元工厂
    /// 
    /// # Returns
    /// * `Result<(), RegistryError>` - 注册结果
    pub fn register(&self, factory: Arc<dyn CustomDataUnitFactory>) -> Result<(), RegistryError> {
        let data_type = factory.data_unit_type();
        let type_value = data_type.to_u8();
        
        // 验证类型范围（128-254）
        if !(128..=254).contains(&type_value) {
            return Err(RegistryError::InvalidTypeRange {
                type_value,
                expected_range: "128-254".to_string(),
            });
        }
        
        let mut factories = self.factories.write().map_err(|_| RegistryError::LockError)?;
        
        // 检查是否已注册
        if factories.contains_key(&type_value) {
            return Err(RegistryError::TypeAlreadyRegistered {
                type_value,
                existing_name: factories[&type_value].name().to_string(),
                new_name: factory.name().to_string(),
            });
        }
        
        factories.insert(type_value, factory);
        Ok(())
    }

    /// 注销自定义数据单元工厂
    pub fn unregister(&self, data_type: DataUnitType) -> Result<(), RegistryError> {
        let type_value = data_type.to_u8();
        let mut factories = self.factories.write().map_err(|_| RegistryError::LockError)?;
        
        if factories.remove(&type_value).is_none() {
            return Err(RegistryError::TypeNotFound { type_value });
        }
        
        Ok(())
    }

    /// 查找自定义数据单元工厂
    pub fn get_factory(&self, data_type: DataUnitType) -> Result<Arc<dyn CustomDataUnitFactory>, RegistryError> {
        let type_value = data_type.to_u8();
        let factories = self.factories.read().map_err(|_| RegistryError::LockError)?;
        
        factories.get(&type_value)
            .cloned()
            .ok_or(RegistryError::TypeNotFound { type_value })
    }

    /// 解析自定义数据单元
    pub fn parse_custom(&self, data_type: DataUnitType, data: &[u8]) -> ParseResult<Box<dyn CustomDataUnit>> {
        let factory = self.get_factory(data_type)
            .map_err(|e| crate::error::ParseError::Custom(format!("Registry error: {}", e)))?;
        
        factory.create_from_bytes(data)
    }

    /// 列出所有已注册的类型
    pub fn list_registered_types(&self) -> Result<Vec<(u8, String)>, RegistryError> {
        let factories = self.factories.read().map_err(|_| RegistryError::LockError)?;
        
        Ok(factories.iter()
            .map(|(&type_value, factory)| (type_value, factory.name().to_string()))
            .collect())
    }

    /// 获取注册的工厂数量
    pub fn count(&self) -> Result<usize, RegistryError> {
        let factories = self.factories.read().map_err(|_| RegistryError::LockError)?;
        Ok(factories.len())
    }

    /// 清空所有注册的工厂（主要用于测试）
    pub fn clear(&self) -> Result<(), RegistryError> {
        let mut factories = self.factories.write().map_err(|_| RegistryError::LockError)?;
        factories.clear();
        Ok(())
    }
}

/// 注册表操作错误
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// 数据单元类型值超出自定义范围（128-254）
    #[error("Invalid type range: {type_value} (expected: {expected_range})")]
    InvalidTypeRange {
        /// 尝试注册的类型值
        type_value: u8,
        /// 期望的有效范围
        expected_range: String,
    },
    
    /// 数据单元类型已被注册
    #[error("Type {type_value} already registered: existing='{existing_name}', new='{new_name}'")]
    TypeAlreadyRegistered {
        /// 已注册的类型值
        type_value: u8,
        /// 已存在的注册名称
        existing_name: String,
        /// 新尝试注册的名称
        new_name: String,
    },
    
    /// 数据单元类型未在注册表中找到
    #[error("Type {type_value} not found in registry")]
    TypeNotFound {
        /// 未找到的类型值
        type_value: u8,
    },
    
    /// 注册表锁错误
    #[error("Lock error in registry")]
    LockError,
}

/// 原始自定义数据单元
/// 
/// 用于包装未知或未注册的自定义数据单元
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCustomDataUnit {
    /// 数据单元类型
    pub data_type: DataUnitType,
    /// 原始字节数据
    pub data: Bytes,
    /// 可选的名称
    pub name: Option<String>,
}

impl RawCustomDataUnit {
    /// 创建新的原始自定义数据单元
    pub fn new(data_type: DataUnitType, data: Bytes) -> Self {
        Self {
            data_type,
            data,
            name: None,
        }
    }

    /// 设置名称
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// 获取数据长度
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl DataUnit for RawCustomDataUnit {
    fn data_unit_type(&self) -> DataUnitType {
        self.data_type
    }

    fn encode(&self) -> EncodeResult<Bytes> {
        Ok(self.data.clone())
    }    fn parse(_data: &[u8]) -> ParseResult<Self> {
        // 这个方法不应该被直接调用，因为我们需要数据类型信息
        Err(crate::error::ParseError::Custom(
            "RawCustomDataUnit::parse requires data type information".to_string()
        ))
    }

    fn validate(&self) -> ParseResult<()> {
        // 原始数据单元的基本验证
        let type_value = self.data_type.to_u8();
        if !(128..=254).contains(&type_value) {
            return Err(crate::error::ParseError::InvalidValue {
                field: "data_type".to_string(),
                value: type_value.to_string(),
                reason: "Custom data unit type must be in range 128-254".to_string(),
            });
        }
        Ok(())
    }
}

impl CustomDataUnit for RawCustomDataUnit {
    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("RawCustomDataUnit")
    }

    fn description(&self) -> Option<&str> {
        Some("未注册的自定义数据单元原始数据")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn CustomDataUnit> {
        Box::new(self.clone())
    }
}

/// 帮助宏，简化自定义数据单元工厂的实现
#[macro_export]
macro_rules! impl_custom_data_unit_factory {
    ($factory_name:ident, $data_unit_type:expr, $target_type:ty) => {
        pub struct $factory_name;

        impl CustomDataUnitFactory for $factory_name {
            fn data_unit_type(&self) -> DataUnitType {
                $data_unit_type
            }

            fn create_from_bytes(&self, data: &[u8]) -> ParseResult<Box<dyn CustomDataUnit>> {
                let instance = <$target_type>::parse(data)?;
                Ok(Box::new(instance))
            }

            fn name(&self) -> &str {
                stringify!($factory_name)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DataUnitType;

    // 测试用的自定义数据单元
    #[derive(Debug, Clone)]
    struct TestCustomDataUnit {
        data: Vec<u8>,
    }

    impl DataUnit for TestCustomDataUnit {
        fn data_unit_type(&self) -> DataUnitType {
            DataUnitType::Custom(128)
        }

        fn encode(&self) -> EncodeResult<Bytes> {
            Ok(Bytes::from(self.data.clone()))
        }

        fn parse(data: &[u8]) -> ParseResult<Self> {
            Ok(Self {
                data: data.to_vec(),
            })
        }
    }

    impl CustomDataUnit for TestCustomDataUnit {
        fn name(&self) -> &str {
            "TestCustomDataUnit"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn clone_box(&self) -> Box<dyn CustomDataUnit> {
            Box::new(self.clone())
        }
    }

    // 测试工厂
    struct TestFactory;

    impl CustomDataUnitFactory for TestFactory {
        fn data_unit_type(&self) -> DataUnitType {
            DataUnitType::Custom(128)
        }

        fn create_from_bytes(&self, data: &[u8]) -> ParseResult<Box<dyn CustomDataUnit>> {
            let instance = TestCustomDataUnit::parse(data)?;
            Ok(Box::new(instance))
        }

        fn name(&self) -> &str {
            "TestFactory"
        }
    }

    #[test]
    fn test_registry_basic_operations() {
        let registry = CustomDataUnitRegistry::new();
        
        // 测试注册
        let factory = Arc::new(TestFactory);
        assert!(registry.register(factory.clone()).is_ok());
        
        // 测试重复注册
        assert!(registry.register(factory).is_err());
        
        // 测试查找
        let found_factory = registry.get_factory(DataUnitType::Custom(128));
        assert!(found_factory.is_ok());
        
        // 测试解析
        let test_data = vec![1, 2, 3, 4];
        let parsed = registry.parse_custom(DataUnitType::Custom(128), &test_data);
        assert!(parsed.is_ok());
        
        // 测试列表
        let types = registry.list_registered_types().unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], (128, "TestFactory".to_string()));
        
        // 测试注销
        assert!(registry.unregister(DataUnitType::Custom(128)).is_ok());
        assert_eq!(registry.count().unwrap(), 0);
    }

    #[test]
    fn test_invalid_type_range() {
        let registry = CustomDataUnitRegistry::new();
        
        // 测试无效的类型范围
        struct InvalidFactory;
        impl CustomDataUnitFactory for InvalidFactory {
            fn data_unit_type(&self) -> DataUnitType {
                DataUnitType::UploadSystemStatus // 这是标准类型，不应该在自定义注册表中
            }
            fn create_from_bytes(&self, _data: &[u8]) -> ParseResult<Box<dyn CustomDataUnit>> {
                unreachable!()
            }
            fn name(&self) -> &str { "InvalidFactory" }
        }
        
        let factory = Arc::new(InvalidFactory);
        assert!(registry.register(factory).is_err());
    }

    #[test]
    fn test_raw_custom_data_unit() {
        let data = Bytes::from(vec![1, 2, 3, 4]);
        let raw = RawCustomDataUnit::new(DataUnitType::Custom(200), data.clone());
        
        assert_eq!(raw.data_unit_type(), DataUnitType::Custom(200));
        assert_eq!(raw.encode().unwrap(), data);
        assert_eq!(raw.len(), 4);
        assert!(!raw.is_empty());
        assert_eq!(raw.name(), "RawCustomDataUnit");
        
        // 测试带名称的版本
        let raw_with_name = raw.with_name("MyCustomUnit".to_string());
        assert_eq!(raw_with_name.name(), "MyCustomUnit");
    }
}
