//! GB26875 扩展机制模块
//!
//! 提供用户自定义数据单元类型的扩展框架，支持编译时和运行时注册

pub mod registry;
pub mod traits;

// 重新导出主要类型
pub use registry::*;
pub use traits::*;

#[cfg(test)]
use crate::error::ParseError;
use crate::error::ParseResult;
use crate::protocol::DataUnitType;
use crate::data_unit::GenericDataUnit;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 扩展数据单元工厂函数类型
/// 
/// 用于创建特定类型的扩展数据单元解析器
pub type ExtensionFactory = Box<dyn Fn(&[u8]) -> ParseResult<Box<dyn ExtensionDataUnit>> + Send + Sync>;

/// 全局扩展注册表
/// 
/// 线程安全的扩展类型注册表，支持运行时注册用户自定义数据单元类型
static GLOBAL_REGISTRY: once_cell::sync::Lazy<Arc<RwLock<ExtensionRegistry>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(ExtensionRegistry::new())));

/// 扩展注册表
/// 
/// 管理用户自定义数据单元类型的注册和解析
pub struct ExtensionRegistry {
    /// 工厂函数映射表
    factories: HashMap<u8, ExtensionFactory>,
    /// 类型名称映射表（用于调试）
    type_names: HashMap<u8, String>,
}

impl std::fmt::Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionRegistry")
            .field("factories", &format!("{} registered", self.factories.len()))
            .field("type_names", &self.type_names)
            .finish()
    }
}

impl ExtensionRegistry {
    /// 创建新的扩展注册表
    pub fn new() -> Self {
        ExtensionRegistry {
            factories: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    /// 注册扩展数据单元类型
    /// 
    /// # Arguments
    /// * `type_id` - 数据单元类型ID（128-254）
    /// * `name` - 类型名称（用于调试）
    /// * `factory` - 工厂函数
    /// 
    /// # Returns
    /// * `Result<(), ExtensionError>` - 成功返回 ()
    /// 
    /// # Example
    /// ```rust
    /// use gb26875::extension::{ExtensionRegistry, ExtensionDataUnit};
    /// use gb26875::error::ParseResult;
    /// 
    /// let mut registry = ExtensionRegistry::new();
    /// 
    /// registry.register(
    ///     200,
    ///     "CustomSensor".to_string(),
    ///     Box::new(|data| {
    ///         // 自定义解析逻辑
    ///         Ok(Box::new(CustomSensorData::parse(data)?))
    ///     })
    /// )?;
    /// # Ok::<(), gb26875::error::ExtensionError>(())
    /// ```
    pub fn register(
        &mut self,
        type_id: u8,
        name: String,
        factory: ExtensionFactory,
    ) -> Result<(), ExtensionError> {
        // 验证类型ID范围
        if !(128..=254).contains(&type_id) {
            return Err(ExtensionError::InvalidTypeFlag(type_id));
        }

        // 检查是否已注册
        if self.factories.contains_key(&type_id) {
            return Err(ExtensionError::AlreadyRegistered { type_id });
        }

        self.factories.insert(type_id, factory);
        self.type_names.insert(type_id, name);

        #[cfg(feature = "logging")]
        log::debug!("注册扩展类型: ID={}, 名称={}", type_id, self.type_names[&type_id]);

        Ok(())
    }

    /// 解注册扩展数据单元类型
    /// 
    /// # Arguments
    /// * `type_id` - 要解注册的类型ID
    /// 
    /// # Returns
    /// * `Result<(), ExtensionError>` - 成功返回 ()
    pub fn unregister(&mut self, type_id: u8) -> Result<(), ExtensionError> {
        if self.factories.remove(&type_id).is_none() {
            return Err(ExtensionError::NotFound { type_id });
        }        #[cfg(feature = "logging")]
        let type_name = self.type_names.remove(&type_id);

        #[cfg(feature = "logging")]
        log::debug!("解注册扩展类型: ID={}, 名称={:?}", type_id, type_name);

        Ok(())
    }

    /// 解析扩展数据单元
    /// 
    /// # Arguments
    /// * `type_id` - 数据单元类型ID
    /// * `data` - 原始数据
    /// 
    /// # Returns
    /// * `Result<Box<dyn ExtensionDataUnit>, ExtensionError>` - 成功返回扩展数据单元
    pub fn parse_extension(
        &self,
        type_id: u8,
        data: &[u8],
    ) -> Result<Box<dyn ExtensionDataUnit>, ExtensionError> {
        match self.factories.get(&type_id) {
            Some(factory) => {
                factory(data).map_err(|e| ExtensionError::ParseFailed {
                    type_id,
                    error: e.to_string(),
                })
            }
            None => Err(ExtensionError::NotFound { type_id }),
        }
    }

    /// 检查类型是否已注册
    /// 
    /// # Arguments
    /// * `type_id` - 类型ID
    /// 
    /// # Returns
    /// * `bool` - 如果已注册返回 true
    pub fn is_registered(&self, type_id: u8) -> bool {
        self.factories.contains_key(&type_id)
    }

    /// 获取已注册的类型列表
    /// 
    /// # Returns
    /// * `Vec<(u8, &str)>` - 类型ID和名称的列表
    pub fn registered_types(&self) -> Vec<(u8, &str)> {
        self.type_names
            .iter()
            .map(|(&id, name)| (id, name.as_str()))
            .collect()
    }

    /// 获取类型名称
    /// 
    /// # Arguments
    /// * `type_id` - 类型ID
    /// 
    /// # Returns
    /// * `Option<&str>` - 类型名称
    pub fn type_name(&self, type_id: u8) -> Option<&str> {
        self.type_names.get(&type_id).map(|s| s.as_str())
    }

    /// 清空所有注册的类型
    pub fn clear(&mut self) {
        self.factories.clear();
        self.type_names.clear();

        #[cfg(feature = "logging")]
        log::debug!("清空所有扩展类型注册");
    }

    /// 获取注册的类型数量
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    /// 检查注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局扩展管理器
/// 
/// 提供全局扩展注册表的访问接口
pub struct ExtensionManager;

impl ExtensionManager {
    /// 注册全局扩展类型
    /// 
    /// # Arguments
    /// * `type_id` - 数据单元类型ID（128-254）
    /// * `name` - 类型名称
    /// * `factory` - 工厂函数
    /// 
    /// # Returns
    /// * `Result<(), ExtensionError>` - 成功返回 ()
    pub fn register_global(
        type_id: u8,
        name: String,
        factory: ExtensionFactory,
    ) -> Result<(), ExtensionError> {
        GLOBAL_REGISTRY
            .write()
            .map_err(|_| ExtensionError::RegistryLockError)?
            .register(type_id, name, factory)
    }

    /// 解析全局扩展数据单元
    /// 
    /// # Arguments
    /// * `type_id` - 数据单元类型ID
    /// * `data` - 原始数据
    /// 
    /// # Returns
    /// * `Result<Box<dyn ExtensionDataUnit>, ExtensionError>` - 成功返回扩展数据单元
    pub fn parse_global(
        type_id: u8,
        data: &[u8],
    ) -> Result<Box<dyn ExtensionDataUnit>, ExtensionError> {
        GLOBAL_REGISTRY
            .read()
            .map_err(|_| ExtensionError::RegistryLockError)?
            .parse_extension(type_id, data)
    }

    /// 检查全局类型是否已注册
    /// 
    /// # Arguments
    /// * `type_id` - 类型ID
    /// 
    /// # Returns
    /// * `Result<bool, ExtensionError>` - 成功返回是否已注册
    pub fn is_registered_global(type_id: u8) -> Result<bool, ExtensionError> {
        Ok(GLOBAL_REGISTRY
            .read()
            .map_err(|_| ExtensionError::RegistryLockError)?
            .is_registered(type_id))
    }

    /// 获取全局已注册的类型列表
    /// 
    /// # Returns
    /// * `Result<Vec<(u8, String)>, ExtensionError>` - 成功返回类型列表
    pub fn registered_types_global() -> Result<Vec<(u8, String)>, ExtensionError> {
        Ok(GLOBAL_REGISTRY
            .read()
            .map_err(|_| ExtensionError::RegistryLockError)?
            .registered_types()
            .into_iter()
            .map(|(id, name)| (id, name.to_string()))
            .collect())
    }

    /// 解注册全局扩展类型
    /// 
    /// # Arguments
    /// * `type_id` - 要解注册的类型ID
    /// 
    /// # Returns
    /// * `Result<(), ExtensionError>` - 成功返回 ()
    pub fn unregister_global(type_id: u8) -> Result<(), ExtensionError> {
        GLOBAL_REGISTRY
            .write()
            .map_err(|_| ExtensionError::RegistryLockError)?
            .unregister(type_id)
    }

    /// 清空全局注册表
    /// 
    /// # Returns
    /// * `Result<(), ExtensionError>` - 成功返回 ()
    pub fn clear_global() -> Result<(), ExtensionError> {
        GLOBAL_REGISTRY
            .write()
            .map_err(|_| ExtensionError::RegistryLockError)?
            .clear();
        Ok(())
    }
}

/// 扩展数据单元解析辅助函数
/// 
/// 尝试使用全局注册表解析数据单元，如果失败则返回原始数据
pub fn parse_with_extensions(data_type: DataUnitType, data: &[u8]) -> GenericDataUnit {
    match data_type {
        DataUnitType::UserDefined(type_id) => {
            // 尝试使用扩展解析
            match ExtensionManager::parse_global(type_id, data) {                Ok(_extension) => {
                    // 成功解析为扩展类型，但需要适配为 GenericDataUnit
                    // 这里我们将其作为原始数据存储，但保留类型信息
                    GenericDataUnit::Raw {
                        data_type,
                        data: Bytes::copy_from_slice(data),
                    }
                }
                Err(_) => {
                    // 解析失败，返回原始数据
                    GenericDataUnit::Raw {
                        data_type,
                        data: Bytes::copy_from_slice(data),
                    }
                }
            }
        }
        _ => {
            // 标准类型，使用默认解析
            GenericDataUnit::from_raw(data_type, data)
                .unwrap_or_else(|_| GenericDataUnit::Raw {
                    data_type,
                    data: Bytes::copy_from_slice(data),
                })
        }
    }
}

/// 便捷宏：注册扩展类型
/// 
/// # Example
/// ```rust
/// use gb26875::register_extension;
/// 
/// register_extension!(200, "CustomSensor", |data| {
///     // 自定义解析逻辑
///     CustomSensorData::parse(data)
/// });
/// ```
#[macro_export]
macro_rules! register_extension {
    ($type_id:expr, $name:expr, $parser:expr) => {
        $crate::extension::ExtensionManager::register_global(
            $type_id,
            $name.to_string(),
            Box::new($parser),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::traits::{ExtensionDataUnit, ExtensionResult};

    // 测试用的扩展数据单元
    #[derive(Debug, Clone, PartialEq)]
    struct TestExtension {
        value: u32,
    }

    impl ExtensionDataUnit for TestExtension {
        fn type_id(&self) -> u8 {
            200
        }

        fn encode(&self) -> ExtensionResult<Bytes> {
            Ok(Bytes::copy_from_slice(&self.value.to_le_bytes()))
        }

        fn validate(&self) -> ExtensionResult<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl TestExtension {
        fn parse(data: &[u8]) -> ParseResult<Self> {            if data.len() != 4 {
                return Err(ParseError::InvalidDataLength {
                    expected: 4,
                    actual: data.len(),
                });
            }

            let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            Ok(TestExtension { value })
        }
    }

    #[test]
    fn test_extension_registry() {
        let mut registry = ExtensionRegistry::new();

        // 注册测试扩展
        let result = registry.register(
            200,
            "TestExtension".to_string(),
            Box::new(|data| {
                let ext = TestExtension::parse(data)?;
                Ok(Box::new(ext) as Box<dyn ExtensionDataUnit>)
            }),
        );
        assert!(result.is_ok());

        // 检查是否已注册
        assert!(registry.is_registered(200));
        assert_eq!(registry.type_name(200), Some("TestExtension"));

        // 测试解析
        let test_data = [0x12, 0x34, 0x56, 0x78];
        let parsed = registry.parse_extension(200, &test_data).unwrap();
        assert_eq!(parsed.type_id(), 200);
    }

    #[test]
    fn test_global_extension_manager() {
        // 清空全局注册表
        ExtensionManager::clear_global().unwrap();

        // 注册全局扩展
        let result = ExtensionManager::register_global(
            200,  // 使用200而不是201以匹配TestExtension的type_id
            "GlobalTestExtension".to_string(),
            Box::new(|data| {
                let ext = TestExtension::parse(data)?;
                Ok(Box::new(ext) as Box<dyn ExtensionDataUnit>)
            }),
        );
        assert!(result.is_ok());

        // 检查是否已注册
        assert!(ExtensionManager::is_registered_global(200).unwrap());

        // 测试解析
        let test_data = [0x12, 0x34, 0x56, 0x78];
        let parsed = ExtensionManager::parse_global(200, &test_data).unwrap();
        assert_eq!(parsed.type_id(), 200);
    }

    #[test]
    fn test_parse_with_extensions() {
        // 测试标准类型解析
        let standard_type = DataUnitType::UploadSystemStatus;
        let standard_data = [0x01, 0x12, 0x34, 0x56]; // 系统状态数据
        let result = parse_with_extensions(standard_type, &standard_data);
        
        match result {
            GenericDataUnit::SystemStatus(_) => {} // 期望的结果
            _ => panic!("期望解析为系统状态"),
        }

        // 测试用户自定义类型（作为原始数据）
        let user_type = DataUnitType::UserDefined(200);
        let user_data = [0xFF, 0xEE, 0xDD, 0xCC];
        let result = parse_with_extensions(user_type, &user_data);
        
        match result {
            GenericDataUnit::Raw { data_type, data } => {
                assert_eq!(data_type, user_type);
                assert_eq!(data.as_ref(), &user_data);
            }
            _ => panic!("期望解析为原始数据"),
        }
    }
}
