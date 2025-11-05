//! GB26875 扩展注册表实现
//!
//! 提供扩展数据单元类型的注册和管理功能

use crate::extension::traits::{ExtensionDataUnit, ExtensionError, ExtensionResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 扩展类型信息
#[derive(Debug, Clone)]
pub struct ExtensionTypeInfo {
    /// 类型ID
    pub type_id: u8,
    /// 类型名称
    pub name: String,
    /// 描述信息
    pub description: Option<String>,
    /// 版本信息
    pub version: Option<String>,
    /// 注册时间戳
    pub registered_at: std::time::SystemTime,
}

impl ExtensionTypeInfo {
    /// 创建新的类型信息
    pub fn new(type_id: u8, name: String) -> Self {
        ExtensionTypeInfo {
            type_id,
            name,
            description: None,
            version: None,
            registered_at: std::time::SystemTime::now(),
        }
    }

    /// 设置描述信息
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置版本信息
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}

/// 扩展工厂函数类型
pub type ExtensionFactory =
    Arc<dyn Fn(&[u8]) -> ExtensionResult<Box<dyn ExtensionDataUnit>> + Send + Sync>;

/// 线程安全的扩展注册表
#[derive(Debug)]
pub struct ThreadSafeExtensionRegistry {
    /// 内部注册表
    inner: Arc<RwLock<ExtensionRegistryInner>>,
}

struct ExtensionRegistryInner {
    /// 工厂函数映射
    factories: HashMap<u8, ExtensionFactory>,
    /// 类型信息映射
    type_infos: HashMap<u8, ExtensionTypeInfo>,
}

impl std::fmt::Debug for ExtensionRegistryInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionRegistryInner")
            .field(
                "factories",
                &format!("<{} factories>", self.factories.len()),
            )
            .field("type_infos", &self.type_infos)
            .finish()
    }
}

impl ThreadSafeExtensionRegistry {
    /// 创建新的线程安全扩展注册表
    pub fn new() -> Self {
        ThreadSafeExtensionRegistry {
            inner: Arc::new(RwLock::new(ExtensionRegistryInner {
                factories: HashMap::new(),
                type_infos: HashMap::new(),
            })),
        }
    }

    /// 注册扩展类型
    ///
    /// # Arguments
    /// * `type_info` - 类型信息
    /// * `factory` - 工厂函数
    ///
    /// # Returns
    /// * `ExtensionResult<()>` - 成功返回 ()
    pub fn register(
        &self,
        type_info: ExtensionTypeInfo,
        factory: ExtensionFactory,
    ) -> ExtensionResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表写锁".to_string(),
            })?;

        // 验证类型ID范围
        if !(128..=254).contains(&type_info.type_id) {
            return Err(ExtensionError::ValidationError {
                reason: format!("扩展类型ID必须在128-254范围内，实际: {}", type_info.type_id),
            });
        }

        // 检查是否已注册
        if inner.factories.contains_key(&type_info.type_id) {
            let existing_name = inner
                .type_infos
                .get(&type_info.type_id)
                .map(|info| info.name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            return Err(ExtensionError::ValidationError {
                reason: format!(
                    "类型ID {} 已被注册为 '{}', 无法重复注册为 '{}'",
                    type_info.type_id, existing_name, type_info.name
                ),
            });
        }

        let type_id = type_info.type_id;
        inner.factories.insert(type_id, factory);
        inner.type_infos.insert(type_id, type_info);

        #[cfg(feature = "logging")]
        log::info!(
            "注册扩展类型: ID={}, 名称={}",
            type_id,
            inner.type_infos[&type_id].name
        );

        Ok(())
    }

    /// 解注册扩展类型
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    ///
    /// # Returns
    /// * `ExtensionResult<()>` - 成功返回 ()
    pub fn unregister(&self, type_id: u8) -> ExtensionResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表写锁".to_string(),
            })?;

        let _type_info = inner.type_infos.remove(&type_id);
        let factory_removed = inner.factories.remove(&type_id).is_some();

        if !factory_removed {
            return Err(ExtensionError::ValidationError {
                reason: format!("类型ID {} 未注册", type_id),
            });
        }

        #[cfg(feature = "logging")]
        if let Some(info) = type_info {
            log::info!("解注册扩展类型: ID={}, 名称={}", type_id, info.name);
        }

        Ok(())
    }

    /// 解析扩展数据单元
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    /// * `data` - 原始数据
    ///
    /// # Returns
    /// * `ExtensionResult<Box<dyn ExtensionDataUnit>>` - 成功返回扩展数据单元
    pub fn parse(&self, type_id: u8, data: &[u8]) -> ExtensionResult<Box<dyn ExtensionDataUnit>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表读锁".to_string(),
            })?;

        match inner.factories.get(&type_id) {
            Some(factory) => factory(data).map_err(|e| ExtensionError::ValidationError {
                reason: format!("解析类型ID {} 失败: {}", type_id, e),
            }),
            None => Err(ExtensionError::ValidationError {
                reason: format!("类型ID {} 未注册", type_id),
            }),
        }
    }

    /// 检查类型是否已注册
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    ///
    /// # Returns
    /// * `ExtensionResult<bool>` - 成功返回是否已注册
    pub fn is_registered(&self, type_id: u8) -> ExtensionResult<bool> {
        let inner = self
            .inner
            .read()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表读锁".to_string(),
            })?;

        Ok(inner.factories.contains_key(&type_id))
    }

    /// 获取类型信息
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    ///
    /// # Returns
    /// * `ExtensionResult<Option<ExtensionTypeInfo>>` - 成功返回类型信息
    pub fn get_type_info(&self, type_id: u8) -> ExtensionResult<Option<ExtensionTypeInfo>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表读锁".to_string(),
            })?;

        Ok(inner.type_infos.get(&type_id).cloned())
    }

    /// 获取所有已注册类型的信息
    ///
    /// # Returns
    /// * `ExtensionResult<Vec<ExtensionTypeInfo>>` - 成功返回类型信息列表
    pub fn list_registered_types(&self) -> ExtensionResult<Vec<ExtensionTypeInfo>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表读锁".to_string(),
            })?;

        Ok(inner.type_infos.values().cloned().collect())
    }

    /// 获取已注册类型的数量
    ///
    /// # Returns
    /// * `ExtensionResult<usize>` - 成功返回类型数量
    pub fn len(&self) -> ExtensionResult<usize> {
        let inner = self
            .inner
            .read()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表读锁".to_string(),
            })?;

        Ok(inner.factories.len())
    }

    /// 检查注册表是否为空
    ///
    /// # Returns
    /// * `ExtensionResult<bool>` - 成功返回是否为空
    pub fn is_empty(&self) -> ExtensionResult<bool> {
        let inner = self
            .inner
            .read()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表读锁".to_string(),
            })?;

        Ok(inner.factories.is_empty())
    }

    /// 清空注册表
    ///
    /// # Returns
    /// * `ExtensionResult<()>` - 成功返回 ()
    pub fn clear(&self) -> ExtensionResult<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| ExtensionError::ValidationError {
                reason: "无法获取注册表写锁".to_string(),
            })?;
        #[cfg(feature = "logging")]
        let count = inner.factories.len();
        inner.factories.clear();
        inner.type_infos.clear();

        #[cfg(feature = "logging")]
        log::info!("清空扩展注册表，共移除 {} 个类型", count);

        Ok(())
    }

    /// 获取注册表的克隆
    pub fn clone(&self) -> Self {
        ThreadSafeExtensionRegistry {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for ThreadSafeExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 扩展注册表构建器
#[derive(Debug)]
pub struct ExtensionRegistryBuilder {
    registry: ThreadSafeExtensionRegistry,
}

impl ExtensionRegistryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        ExtensionRegistryBuilder {
            registry: ThreadSafeExtensionRegistry::new(),
        }
    }

    /// 注册扩展类型
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    /// * `name` - 类型名称
    /// * `factory` - 工厂函数
    ///
    /// # Returns
    /// * `ExtensionResult<Self>` - 成功返回构建器
    pub fn register<F>(self, type_id: u8, name: &str, factory: F) -> ExtensionResult<Self>
    where
        F: Fn(&[u8]) -> ExtensionResult<Box<dyn ExtensionDataUnit>> + Send + Sync + 'static,
    {
        let type_info = ExtensionTypeInfo::new(type_id, name.to_string());
        let factory = Arc::new(factory);

        self.registry.register(type_info, factory)?;
        Ok(self)
    }

    /// 注册带描述的扩展类型
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    /// * `name` - 类型名称
    /// * `description` - 描述信息
    /// * `factory` - 工厂函数
    ///
    /// # Returns
    /// * `ExtensionResult<Self>` - 成功返回构建器
    pub fn register_with_description<F>(
        self,
        type_id: u8,
        name: &str,
        description: &str,
        factory: F,
    ) -> ExtensionResult<Self>
    where
        F: Fn(&[u8]) -> ExtensionResult<Box<dyn ExtensionDataUnit>> + Send + Sync + 'static,
    {
        let type_info = ExtensionTypeInfo::new(type_id, name.to_string())
            .with_description(description.to_string());
        let factory = Arc::new(factory);

        self.registry.register(type_info, factory)?;
        Ok(self)
    }

    /// 注册带版本的扩展类型
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    /// * `name` - 类型名称
    /// * `version` - 版本信息
    /// * `factory` - 工厂函数
    ///
    /// # Returns
    /// * `ExtensionResult<Self>` - 成功返回构建器
    pub fn register_with_version<F>(
        self,
        type_id: u8,
        name: &str,
        version: &str,
        factory: F,
    ) -> ExtensionResult<Self>
    where
        F: Fn(&[u8]) -> ExtensionResult<Box<dyn ExtensionDataUnit>> + Send + Sync + 'static,
    {
        let type_info =
            ExtensionTypeInfo::new(type_id, name.to_string()).with_version(version.to_string());
        let factory = Arc::new(factory);

        self.registry.register(type_info, factory)?;
        Ok(self)
    }

    /// 注册完整信息的扩展类型
    ///
    /// # Arguments
    /// * `type_id` - 类型ID
    /// * `name` - 类型名称
    /// * `description` - 描述信息
    /// * `version` - 版本信息
    /// * `factory` - 工厂函数
    ///
    /// # Returns
    /// * `ExtensionResult<Self>` - 成功返回构建器
    pub fn register_full<F>(
        self,
        type_id: u8,
        name: &str,
        description: &str,
        version: &str,
        factory: F,
    ) -> ExtensionResult<Self>
    where
        F: Fn(&[u8]) -> ExtensionResult<Box<dyn ExtensionDataUnit>> + Send + Sync + 'static,
    {
        let type_info = ExtensionTypeInfo::new(type_id, name.to_string())
            .with_description(description.to_string())
            .with_version(version.to_string());
        let factory = Arc::new(factory);

        self.registry.register(type_info, factory)?;
        Ok(self)
    }

    /// 构建注册表
    ///
    /// # Returns
    /// * `ThreadSafeExtensionRegistry` - 构建的注册表
    pub fn build(self) -> ThreadSafeExtensionRegistry {
        self.registry
    }
}

impl Default for ExtensionRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::traits::ExtensionDataUnit;
    use bytes::Bytes;
    use std::any::Any;

    // 测试用扩展数据单元
    #[derive(Debug, Clone)]
    struct TestExtension {
        type_id: u8,
        value: u32,
    }

    impl ExtensionDataUnit for TestExtension {
        fn type_id(&self) -> u8 {
            self.type_id
        }

        fn encode(&self) -> ExtensionResult<Bytes> {
            Ok(Bytes::copy_from_slice(&self.value.to_le_bytes()))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl TestExtension {
        fn parse(type_id: u8, data: &[u8]) -> ExtensionResult<Self> {
            if data.len() != 4 {
                return Err(ExtensionError::ParseError(
                    crate::error::ParseError::InvalidDataLength {
                        expected: 4,
                        actual: data.len(),
                    },
                ));
            }

            let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            Ok(TestExtension { type_id, value })
        }
    }

    #[test]
    fn test_thread_safe_extension_registry() {
        let registry = ThreadSafeExtensionRegistry::new();

        // 注册测试扩展
        let type_info = ExtensionTypeInfo::new(200, "TestExtension".to_string())
            .with_description("测试扩展数据单元".to_string())
            .with_version("1.0.0".to_string());

        let factory = Arc::new(
            |data: &[u8]| -> ExtensionResult<Box<dyn ExtensionDataUnit>> {
                let ext = TestExtension::parse(200, data)?;
                Ok(Box::new(ext))
            },
        );

        assert!(registry.register(type_info, factory).is_ok());

        // 检查是否已注册
        assert!(registry.is_registered(200).unwrap());
        assert!(!registry.is_registered(201).unwrap());

        // 获取类型信息
        let info = registry.get_type_info(200).unwrap().unwrap();
        assert_eq!(info.name, "TestExtension");
        assert_eq!(info.description, Some("测试扩展数据单元".to_string()));
        assert_eq!(info.version, Some("1.0.0".to_string())); // 测试解析
        let test_data = [0x12, 0x34, 0x56, 0x78];
        let parsed = registry.parse(200, &test_data).unwrap();
        assert_eq!(
            parsed.as_any().type_id(),
            std::any::TypeId::of::<TestExtension>()
        );

        // 测试列出所有类型
        let types = registry.list_registered_types().unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "TestExtension");

        // 测试解注册
        assert!(registry.unregister(200).is_ok());
        assert!(!registry.is_registered(200).unwrap());
        assert!(registry.is_empty().unwrap());
    }

    #[test]
    fn test_extension_registry_builder() {
        let registry = ExtensionRegistryBuilder::new()
            .register(200, "TestExtension1", |data| {
                let ext = TestExtension::parse(200, data)?;
                Ok(Box::new(ext) as Box<dyn ExtensionDataUnit>)
            })
            .unwrap()
            .register_with_description(201, "TestExtension2", "第二个测试扩展", |data| {
                let ext = TestExtension::parse(201, data)?;
                Ok(Box::new(ext) as Box<dyn ExtensionDataUnit>)
            })
            .unwrap()
            .register_full(
                202,
                "TestExtension3",
                "第三个测试扩展",
                "2.0.0",
                |data| {
                    let ext = TestExtension::parse(202, data)?;
                    Ok(Box::new(ext) as Box<dyn ExtensionDataUnit>)
                },
            )
            .unwrap()
            .build();

        assert_eq!(registry.len().unwrap(), 3);
        assert!(registry.is_registered(200).unwrap());
        assert!(registry.is_registered(201).unwrap());
        assert!(registry.is_registered(202).unwrap());

        let types = registry.list_registered_types().unwrap();
        assert_eq!(types.len(), 3);

        // 检查第三个扩展的完整信息
        let info = registry.get_type_info(202).unwrap().unwrap();
        assert_eq!(info.name, "TestExtension3");
        assert_eq!(info.description, Some("第三个测试扩展".to_string()));
        assert_eq!(info.version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_invalid_type_id() {
        let registry = ThreadSafeExtensionRegistry::new();

        // 测试无效的类型ID
        let type_info = ExtensionTypeInfo::new(127, "InvalidExtension".to_string()); // 小于128
        let factory =
            Arc::new(|_: &[u8]| -> ExtensionResult<Box<dyn ExtensionDataUnit>> { unreachable!() });

        assert!(registry.register(type_info, factory).is_err());

        let type_info = ExtensionTypeInfo::new(255, "InvalidExtension".to_string()); // 大于254
        let factory =
            Arc::new(|_: &[u8]| -> ExtensionResult<Box<dyn ExtensionDataUnit>> { unreachable!() });

        assert!(registry.register(type_info, factory).is_err());
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = ThreadSafeExtensionRegistry::new();

        let type_info = ExtensionTypeInfo::new(200, "FirstExtension".to_string());
        let factory =
            Arc::new(|_: &[u8]| -> ExtensionResult<Box<dyn ExtensionDataUnit>> { unreachable!() });

        // 第一次注册应该成功
        assert!(registry.register(type_info, factory).is_ok());

        // 第二次注册同一个类型ID应该失败
        let type_info = ExtensionTypeInfo::new(200, "SecondExtension".to_string());
        let factory =
            Arc::new(|_: &[u8]| -> ExtensionResult<Box<dyn ExtensionDataUnit>> { unreachable!() });

        assert!(registry.register(type_info, factory).is_err());
    }
}
