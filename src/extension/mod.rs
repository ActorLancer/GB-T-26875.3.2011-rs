//! GB26875 扩展机制模块
//!
//! 提供用户自定义数据单元类型的扩展框架，支持编译时和运行时注册

pub mod registry;
pub mod traits;

// 重新导出主要类型
pub use registry::*;
pub use traits::*;

use crate::data_unit::GenericDataUnit;
use crate::error::ParseError;
use crate::error::ParseResult;
use crate::protocol::DataUnitType;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 扩展注册表信息
#[derive(Debug, Clone)]
pub struct ExtensionRegistryInfo {
    /// 扩展标识符
    pub extension_id: ExtensionId,
    /// 扩展代码
    pub code: u8,
    /// 扩展类型
    pub extension_type: String,
    /// 描述信息
    pub description: String,
    /// 注册时间
    pub registered_at: std::time::SystemTime,
}

/// 扩展注册表统计信息
#[derive(Debug, Clone)]
pub struct ExtensionRegistryStats {
    /// 总扩展数量
    pub total_extensions: usize,
    /// 命令扩展数量
    pub command_extensions: usize,
    /// 数据单元扩展数量
    pub data_unit_extensions: usize,
    /// 系统类型扩展数量
    pub system_type_extensions: usize,
    /// 部件类型扩展数量
    pub component_type_extensions: usize,
    /// 模拟量类型扩展数量
    pub analog_type_extensions: usize,
    /// 已使用的代码数量
    pub used_codes: usize,
    /// 可用的代码数量
    pub available_codes: usize,
}

/// 过程宏扩展注册表
///
/// 支持编译时生成的扩展类型注册
pub struct MacroExtensionRegistry {
    /// 命令扩展注册表
    pub command_extensions: RwLock<HashMap<u8, Box<dyn CommandExtension>>>,
    /// 数据单元扩展注册表
    pub data_unit_extensions: RwLock<HashMap<u8, Box<dyn DataUnitExtension>>>,
    /// 系统类型扩展注册表
    pub system_type_extensions: RwLock<HashMap<u8, Box<dyn SystemTypeExtension>>>,
    /// 部件类型扩展注册表
    pub component_type_extensions: RwLock<HashMap<u8, Box<dyn ComponentTypeExtension>>>,
    /// 模拟量类型扩展注册表
    pub analog_type_extensions: RwLock<HashMap<u8, Box<dyn AnalogTypeExtension>>>,
    /// 扩展ID冲突解决映射
    pub extension_id_registry: RwLock<HashMap<ExtensionId, u8>>,
}


impl MacroExtensionRegistry {
    /// 创建新的过程宏扩展注册表
    pub fn new() -> Self {
        Self {
            command_extensions: RwLock::new(HashMap::new()),
            data_unit_extensions: RwLock::new(HashMap::new()),
            system_type_extensions: RwLock::new(HashMap::new()),
            component_type_extensions: RwLock::new(HashMap::new()),
            analog_type_extensions: RwLock::new(HashMap::new()),
            extension_id_registry: RwLock::new(HashMap::new()),
        }
    }

    /// 注册命令扩展
    pub fn register_command(&self, extension: Box<dyn CommandExtension>) -> ExtensionResult<()> {
        let code = extension.command_code();
        let extension_id = extension.extension_id();

        // 检查代码冲突
        {
            let commands = self.command_extensions.read()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if commands.contains_key(&code) {
                return Err(ExtensionError::ValidationError {
                    reason: format!("命令代码 {} 已被注册", code)
                });
            }
        }

        // 检查扩展ID冲突
        {
            let mut id_registry = self.extension_id_registry.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if let Some(&existing_code) = id_registry.get(&extension_id) {
                return Err(ExtensionError::ValidationError {
                    reason: format!(
                        "扩展ID {} 已被代码 {} 注册",
                        extension_id, existing_code
                    )
                });
            }
            id_registry.insert(extension_id, code);
        }

        // 注册扩展
        let mut commands = self.command_extensions.write()
            .map_err(|_| ExtensionError::RegistryLockError)?;
        commands.insert(code, extension);

        
        log::info!("注册命令扩展: 代码={}", code);

        Ok(())
    }

    /// 注册数据单元扩展
    pub fn register_data_unit(&self, extension: Box<dyn DataUnitExtension>) -> ExtensionResult<()> {
        let type_flag = extension.type_flag();
        let extension_id = extension.extension_id();

        // 检查类型标志冲突
        {
            let data_units = self.data_unit_extensions.read()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if data_units.contains_key(&type_flag) {
                return Err(ExtensionError::ValidationError {
                    reason: format!("数据单元类型标志 {} 已被注册", type_flag)
                });
            }
        }

        // 检查扩展ID冲突
        {
            let mut id_registry = self.extension_id_registry.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if let Some(&existing_flag) = id_registry.get(&extension_id) {
                return Err(ExtensionError::ValidationError {
                    reason: format!(
                        "扩展ID {} 已被类型标志 {} 注册",
                        extension_id, existing_flag
                    )
                });
            }
            id_registry.insert(extension_id, type_flag);
        }

        // 注册扩展
        let mut data_units = self.data_unit_extensions.write()
            .map_err(|_| ExtensionError::RegistryLockError)?;
        data_units.insert(type_flag, extension);

        
        log::info!("注册数据单元扩展: 类型标志={}", type_flag);

        Ok(())
    }

    /// 注册系统类型扩展
    pub fn register_system_type(&self, extension: Box<dyn SystemTypeExtension>) -> ExtensionResult<()> {
        let code = extension.system_type_code();
        let extension_id = extension.extension_id();

        // 检查代码冲突
        {
            let systems = self.system_type_extensions.read()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if systems.contains_key(&code) {
                return Err(ExtensionError::ValidationError {
                    reason: format!("系统类型代码 {} 已被注册", code)
                });
            }
        }

        // 检查扩展ID冲突
        {
            let mut id_registry = self.extension_id_registry.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if let Some(&existing_code) = id_registry.get(&extension_id) {
                return Err(ExtensionError::ValidationError {
                    reason: format!(
                        "扩展ID {} 已被代码 {} 注册",
                        extension_id, existing_code
                    )
                });
            }
            id_registry.insert(extension_id, code);
        }

        // 注册扩展
        let mut systems = self.system_type_extensions.write()
            .map_err(|_| ExtensionError::RegistryLockError)?;
        systems.insert(code, extension);

        
        log::info!("注册系统类型扩展: 代码={}", code);

        Ok(())
    }

    /// 注册部件类型扩展
    pub fn register_component_type(&self, extension: Box<dyn ComponentTypeExtension>) -> ExtensionResult<()> {
        let code = extension.component_type_code();
        let extension_id = extension.extension_id();

        // 检查代码冲突
        {
            let components = self.component_type_extensions.read()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if components.contains_key(&code) {
                return Err(ExtensionError::ValidationError {
                    reason: format!("部件类型代码 {} 已被注册", code)
                });
            }
        }

        // 检查扩展ID冲突
        {
            let mut id_registry = self.extension_id_registry.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if let Some(&existing_code) = id_registry.get(&extension_id) {
                return Err(ExtensionError::ValidationError {
                    reason: format!(
                        "扩展ID {} 已被代码 {} 注册",
                        extension_id, existing_code
                    )
                });
            }
            id_registry.insert(extension_id, code);
        }

        // 注册扩展
        let mut components = self.component_type_extensions.write()
            .map_err(|_| ExtensionError::RegistryLockError)?;
        components.insert(code, extension);

        
        log::info!("注册部件类型扩展: 代码={}", code);

        Ok(())
    }

    /// 注册模拟量类型扩展
    pub fn register_analog_type(&self, extension: Box<dyn AnalogTypeExtension>) -> ExtensionResult<()> {
        let code = extension.analog_type_code();
        let extension_id = extension.extension_id();

        // 检查代码冲突
        {
            let analogs = self.analog_type_extensions.read()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if analogs.contains_key(&code) {
                return Err(ExtensionError::ValidationError {
                    reason: format!("模拟量类型代码 {} 已被注册", code)
                });
            }
        }

        // 检查扩展ID冲突
        {
            let mut id_registry = self.extension_id_registry.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            if let Some(&existing_code) = id_registry.get(&extension_id) {
                return Err(ExtensionError::ValidationError {
                    reason: format!(
                        "扩展ID {} 已被代码 {} 注册",
                        extension_id, existing_code
                    )
                });
            }
            id_registry.insert(extension_id, code);
        }

        // 注册扩展
        let mut analogs = self.analog_type_extensions.write()
            .map_err(|_| ExtensionError::RegistryLockError)?;
        analogs.insert(code, extension);

        
        log::info!("注册模拟量类型扩展: 代码={}", code);

        Ok(())
    }

    /// 查找命令扩展
    pub fn find_command(&self, code: u8) -> Option<Box<dyn CommandExtension>> {
        self.command_extensions.read()
            .ok()?
            .get(&code)
            .map(|ext| ext.clone_boxed())
    }

    /// 查找数据单元扩展
    pub fn find_data_unit(&self, type_flag: u8) -> Option<Box<dyn DataUnitExtension>> {
        self.data_unit_extensions.read()
            .ok()?
            .get(&type_flag)
            .map(|ext| ext.clone_boxed())
    }

    /// 查找系统类型扩展
    pub fn find_system_type(&self, code: u8) -> Option<Box<dyn SystemTypeExtension>> {
        self.system_type_extensions.read()
            .ok()
            .and_then(|systems| systems.get(&code).map(|ext| ext.clone_boxed()))
    }

    /// 查找部件类型扩展
    pub fn find_component_type(&self, code: u8) -> Option<Box<dyn ComponentTypeExtension>> {
        self.component_type_extensions.read()
            .ok()
            .and_then(|components| components.get(&code).map(|ext| ext.clone_boxed()))
    }

    /// 查找模拟量类型扩展
    pub fn find_analog_type(&self, code: u8) -> Option<Box<dyn AnalogTypeExtension>> {
        self.analog_type_extensions.read()
            .ok()
            .and_then(|analogs| analogs.get(&code).map(|ext| ext.clone_boxed()))
    }

    /// 解决扩展ID冲突
    pub fn resolve_extension_id_conflict(&self, extension_id: &ExtensionId) -> Option<u8> {
        self.extension_id_registry.read()
            .ok()?
            .get(extension_id)
            .copied()
    }

    /// 列出所有已注册的扩展ID
    pub fn list_extension_ids(&self) -> Vec<ExtensionId> {
        self.extension_id_registry.read()
            .map(|registry| registry.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 检查扩展ID是否已注册
    pub fn is_extension_id_registered(&self, extension_id: &ExtensionId) -> bool {
        self.extension_id_registry.read()
            .map(|registry| registry.contains_key(extension_id))
            .unwrap_or(false)
    }

    /// 根据模块路径和类型名称查找扩展ID
    pub fn find_extension_by_name(&self, module_path: &str, type_name: &str) -> Vec<ExtensionId> {
        self.extension_id_registry.read()
            .map(|registry| {
                registry.keys()
                    .filter(|id| id.module_path == module_path && id.type_name == type_name)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 根据版本获取特定扩展
    pub fn find_extension_by_version(&self, module_path: &str, type_name: &str, version: &str) -> Option<u8> {
        // 在已注册的扩展中查找匹配的扩展ID
        self.extension_id_registry.read()
            .ok()
            .and_then(|registry| {
                registry.iter()
                    .find(|(id, _)| {
                        id.module_path == module_path && 
                        id.type_name == type_name && 
                        id.version == version
                    })
                    .map(|(_, &code)| code)
            })
    }

    /// 获取扩展的详细信息
    pub fn get_extension_info(&self, extension_id: &ExtensionId) -> Option<ExtensionRegistryInfo> {
        let code = self.resolve_extension_id_conflict(extension_id)?;
        
        // 尝试从各个注册表中查找扩展信息
        if let Some(command) = self.command_extensions.read().ok()?.get(&code) {
            return Some(ExtensionRegistryInfo {
                extension_id: extension_id.clone(),
                code,
                extension_type: "Command".to_string(),
                description: command.description().to_string(),
                registered_at: std::time::SystemTime::now(), // 简化实现
            });
        }

        if let Some(data_unit) = self.data_unit_extensions.read().ok()?.get(&code) {
            return Some(ExtensionRegistryInfo {
                extension_id: extension_id.clone(),
                code,
                extension_type: "DataUnit".to_string(),
                description: data_unit.description().to_string(),
                registered_at: std::time::SystemTime::now(),
            });
        }

        if let Some(system_type) = self.system_type_extensions.read().ok()?.get(&code) {
            return Some(ExtensionRegistryInfo {
                extension_id: extension_id.clone(),
                code,
                extension_type: "SystemType".to_string(),
                description: system_type.description().to_string(),
                registered_at: std::time::SystemTime::now(),
            });
        }

        if let Some(component_type) = self.component_type_extensions.read().ok()?.get(&code) {
            return Some(ExtensionRegistryInfo {
                extension_id: extension_id.clone(),
                code,
                extension_type: "ComponentType".to_string(),
                description: component_type.description().to_string(),
                registered_at: std::time::SystemTime::now(),
            });
        }

        if let Some(analog_type) = self.analog_type_extensions.read().ok()?.get(&code) {
            return Some(ExtensionRegistryInfo {
                extension_id: extension_id.clone(),
                code,
                extension_type: "AnalogType".to_string(),
                description: analog_type.description().to_string(),
                registered_at: std::time::SystemTime::now(),
            });
        }

        None
    }

    /// 列出所有已注册扩展的信息
    pub fn list_all_extensions(&self) -> Vec<ExtensionRegistryInfo> {
        let mut extensions = Vec::new();
        
        for extension_id in self.list_extension_ids() {
            if let Some(info) = self.get_extension_info(&extension_id) {
                extensions.push(info);
            }
        }
        
        extensions.sort_by(|a, b| a.extension_id.full_name().cmp(&b.extension_id.full_name()));
        extensions
    }

    /// 检查代码冲突
    pub fn check_code_conflicts(&self, code: u8) -> Vec<ExtensionId> {
        let mut conflicts = Vec::new();
        
        if let Ok(id_registry) = self.extension_id_registry.read() {
            for (extension_id, registered_code) in id_registry.iter() {
                if *registered_code == code {
                    conflicts.push(extension_id.clone());
                }
            }
        }
        
        conflicts
    }

    /// 建议可用的代码范围
    pub fn suggest_available_codes(&self, count: usize) -> Vec<u8> {
        let mut available_codes = Vec::new();
        let used_codes: std::collections::HashSet<u8> = if let Ok(id_registry) = self.extension_id_registry.read() {
            id_registry.values().copied().collect()
        } else {
            std::collections::HashSet::new()
        };

        for code in 128..=254u8 {
            if !used_codes.contains(&code) {
                available_codes.push(code);
                if available_codes.len() >= count {
                    break;
                }
            }
        }

        available_codes
    }

    /// 清空所有注册
    pub fn clear_all(&self) -> ExtensionResult<()> {
        // 清空扩展ID注册表
        {
            let mut id_registry = self.extension_id_registry.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            id_registry.clear();
        }

        // 清空各个扩展注册表
        {
            let mut commands = self.command_extensions.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            commands.clear();
        }

        {
            let mut data_units = self.data_unit_extensions.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            data_units.clear();
        }

        {
            let mut systems = self.system_type_extensions.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            systems.clear();
        }

        {
            let mut components = self.component_type_extensions.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            components.clear();
        }

        {
            let mut analogs = self.analog_type_extensions.write()
                .map_err(|_| ExtensionError::RegistryLockError)?;
            analogs.clear();
        }

        
        log::info!("已清空所有过程宏扩展注册");

        Ok(())
    }

    /// 获取注册表统计信息
    pub fn get_registry_stats(&self) -> ExtensionRegistryStats {
        let command_count = self.command_extensions.read()
            .map(|c| c.len())
            .unwrap_or(0);
        
        let data_unit_count = self.data_unit_extensions.read()
            .map(|d| d.len())
            .unwrap_or(0);
        
        let system_type_count = self.system_type_extensions.read()
            .map(|s| s.len())
            .unwrap_or(0);
        
        let component_type_count = self.component_type_extensions.read()
            .map(|c| c.len())
            .unwrap_or(0);
        
        let analog_type_count = self.analog_type_extensions.read()
            .map(|a| a.len())
            .unwrap_or(0);

        ExtensionRegistryStats {
            total_extensions: command_count + data_unit_count + system_type_count + component_type_count + analog_type_count,
            command_extensions: command_count,
            data_unit_extensions: data_unit_count,
            system_type_extensions: system_type_count,
            component_type_extensions: component_type_count,
            analog_type_extensions: analog_type_count,
            used_codes: self.extension_id_registry.read()
                .map(|r| r.len())
                .unwrap_or(0),
            available_codes: 127 - self.extension_id_registry.read()
                .map(|r| r.len())
                .unwrap_or(0), // 128-254 = 127 total codes
        }
    }
}


impl Default for MacroExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局过程宏扩展注册表实例

static MACRO_EXTENSION_REGISTRY: once_cell::sync::Lazy<MacroExtensionRegistry> =
    once_cell::sync::Lazy::new(|| MacroExtensionRegistry::new());

/// 全局过程宏扩展管理器

pub struct MacroExtensionManager;


impl MacroExtensionManager {
    /// 获取全局扩展注册表的引用
    pub fn global() -> &'static MacroExtensionRegistry {
        &MACRO_EXTENSION_REGISTRY
    }

    /// 注册命令扩展到全局注册表
    pub fn register_command_global(extension: Box<dyn CommandExtension>) -> ExtensionResult<()> {
        MACRO_EXTENSION_REGISTRY.register_command(extension)
    }

    /// 注册数据单元扩展到全局注册表
    pub fn register_data_unit_global(extension: Box<dyn DataUnitExtension>) -> ExtensionResult<()> {
        MACRO_EXTENSION_REGISTRY.register_data_unit(extension)
    }

    /// 注册系统类型扩展到全局注册表
    pub fn register_system_type_global(extension: Box<dyn SystemTypeExtension>) -> ExtensionResult<()> {
        MACRO_EXTENSION_REGISTRY.register_system_type(extension)
    }

    /// 注册部件类型扩展到全局注册表
    pub fn register_component_type_global(extension: Box<dyn ComponentTypeExtension>) -> ExtensionResult<()> {
        MACRO_EXTENSION_REGISTRY.register_component_type(extension)
    }

    /// 注册模拟量类型扩展到全局注册表
    pub fn register_analog_type_global(extension: Box<dyn AnalogTypeExtension>) -> ExtensionResult<()> {
        MACRO_EXTENSION_REGISTRY.register_analog_type(extension)
    }

    /// 从全局注册表查找命令扩展
    pub fn find_command_global(code: u8) -> Option<Box<dyn CommandExtension>> {
        MACRO_EXTENSION_REGISTRY.find_command(code)
    }

    /// 从全局注册表查找数据单元扩展
    pub fn find_data_unit_global(type_flag: u8) -> Option<Box<dyn DataUnitExtension>> {
        MACRO_EXTENSION_REGISTRY.find_data_unit(type_flag)
    }

    /// 从全局注册表查找系统类型扩展
    pub fn find_system_type_global(code: u8) -> Option<Box<dyn SystemTypeExtension>> {
        MACRO_EXTENSION_REGISTRY.find_system_type(code)
    }

    /// 从全局注册表查找部件类型扩展
    pub fn find_component_type_global(code: u8) -> Option<Box<dyn ComponentTypeExtension>> {
        MACRO_EXTENSION_REGISTRY.find_component_type(code)
    }

    /// 从全局注册表查找模拟量类型扩展
    pub fn find_analog_type_global(code: u8) -> Option<Box<dyn AnalogTypeExtension>> {
        MACRO_EXTENSION_REGISTRY.find_analog_type(code)
    }

    /// 解决扩展ID冲突
    pub fn resolve_extension_id_conflict_global(extension_id: &ExtensionId) -> Option<u8> {
        MACRO_EXTENSION_REGISTRY.resolve_extension_id_conflict(extension_id)
    }

    /// 列出所有已注册的扩展ID
    pub fn list_extension_ids_global() -> Vec<ExtensionId> {
        MACRO_EXTENSION_REGISTRY.list_extension_ids()
    }

    /// 检查扩展ID是否已注册
    pub fn is_extension_id_registered_global(extension_id: &ExtensionId) -> bool {
        MACRO_EXTENSION_REGISTRY.is_extension_id_registered(extension_id)
    }

    /// 根据模块路径和类型名称查找扩展ID
    pub fn find_extension_by_name_global(module_path: &str, type_name: &str) -> Vec<ExtensionId> {
        MACRO_EXTENSION_REGISTRY.find_extension_by_name(module_path, type_name)
    }

    /// 根据版本获取特定扩展
    pub fn find_extension_by_version_global(module_path: &str, type_name: &str, version: &str) -> Option<u8> {
        MACRO_EXTENSION_REGISTRY.find_extension_by_version(module_path, type_name, version)
    }

    /// 获取扩展的详细信息
    pub fn get_extension_info_global(extension_id: &ExtensionId) -> Option<ExtensionRegistryInfo> {
        MACRO_EXTENSION_REGISTRY.get_extension_info(extension_id)
    }

    /// 列出所有已注册扩展的信息
    pub fn list_all_extensions_global() -> Vec<ExtensionRegistryInfo> {
        MACRO_EXTENSION_REGISTRY.list_all_extensions()
    }

    /// 检查代码冲突
    pub fn check_code_conflicts_global(code: u8) -> Vec<ExtensionId> {
        MACRO_EXTENSION_REGISTRY.check_code_conflicts(code)
    }

    /// 建议可用的代码范围
    pub fn suggest_available_codes_global(count: usize) -> Vec<u8> {
        MACRO_EXTENSION_REGISTRY.suggest_available_codes(count)
    }

    /// 清空所有注册
    pub fn clear_all_global() -> ExtensionResult<()> {
        MACRO_EXTENSION_REGISTRY.clear_all()
    }

    /// 获取注册表统计信息
    pub fn get_registry_stats_global() -> ExtensionRegistryStats {
        MACRO_EXTENSION_REGISTRY.get_registry_stats()
    }

    /// 高级命名空间冲突解决
    /// 
    /// 当同一个扩展ID有多个版本时，选择最新版本
    pub fn resolve_namespace_conflict_by_version(extension_ids: &[ExtensionId]) -> Option<ExtensionId> {
        if extension_ids.is_empty() {
            return None;
        }

        // 简单的版本比较：假设版本号是语义化版本（如 "1.0.0"）
        let mut best_extension = &extension_ids[0];
        let mut best_version = parse_version(best_extension.version);

        for extension_id in extension_ids.iter().skip(1) {
            let version = parse_version(extension_id.version);
            if compare_versions(&version, &best_version) > 0 {
                best_extension = extension_id;
                best_version = version;
            }
        }

        Some(best_extension.clone())
    }

    /// 根据优先级解决扩展冲突
    /// 
    /// 根据模块路径的深度和字典序选择最合适的扩展
    pub fn resolve_namespace_conflict_by_priority(extension_ids: &[ExtensionId]) -> Option<ExtensionId> {
        if extension_ids.is_empty() {
            return None;
        }

        // 优先选择模块路径更深的扩展（更具体的模块）
        let mut best_extension = &extension_ids[0];
        let mut best_depth = count_module_depth(best_extension.module_path);

        for extension_id in extension_ids.iter().skip(1) {
            let depth = count_module_depth(extension_id.module_path);
            
            // 如果深度更深，选择这个扩展
            if depth > best_depth {
                best_extension = extension_id;
                best_depth = depth;
            } 
            // 如果深度相同，按模块路径字典序选择
            else if depth == best_depth && extension_id.module_path < best_extension.module_path {
                best_extension = extension_id;
            }
        }

        Some(best_extension.clone())
    }

    /// 获取扩展依赖信息（未来扩展）
    pub fn get_extension_dependencies(extension_id: &ExtensionId) -> Vec<ExtensionId> {
        // 目前返回空列表，未来可以实现扩展依赖跟踪
        let _ = extension_id; // 避免未使用参数警告
        Vec::new()
    }

    /// 验证扩展兼容性（未来扩展）
    pub fn validate_extension_compatibility(extension_id: &ExtensionId) -> ExtensionResult<()> {
        // 目前只做基本验证
        if extension_id.module_path.is_empty() || extension_id.type_name.is_empty() || extension_id.version.is_empty() {
            return Err(ExtensionError::ValidationError {
                reason: "扩展ID字段不能为空".to_string(),
            });
        }

        // 检查版本格式（简单检查）
        if !is_valid_version_format(extension_id.version) {
            return Err(ExtensionError::ValidationError {
                reason: format!("无效的版本格式: {}", extension_id.version),
            });
        }

        Ok(())
    }
}

/// 解析版本号为可比较的元组
#[allow(dead_code)]
fn parse_version(version: &str) -> (u32, u32, u32) {
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

/// 比较两个版本号
/// 返回值：-1 表示 a < b，0 表示 a == b，1 表示 a > b
#[allow(dead_code)]
fn compare_versions(a: &(u32, u32, u32), b: &(u32, u32, u32)) -> i32 {
    if a.0 != b.0 {
        if a.0 > b.0 { 1 } else { -1 }
    } else if a.1 != b.1 {
        if a.1 > b.1 { 1 } else { -1 }
    } else if a.2 != b.2 {
        if a.2 > b.2 { 1 } else { -1 }
    } else {
        0
    }
}

/// 计算模块路径深度
#[allow(dead_code)]
fn count_module_depth(module_path: &str) -> usize {
    if module_path.is_empty() {
        0
    } else {
        module_path.matches("::").count() + 1
    }
}

/// 验证版本格式是否有效
#[allow(dead_code)]
fn is_valid_version_format(version: &str) -> bool {
    // 简单验证：至少包含一个数字和点
    version.chars().any(|c| c.is_ascii_digit()) && 
    (version.contains('.') || version.chars().all(|c| c.is_ascii_digit()))
}

/// 扩展数据单元工厂函数类型
///
/// 用于创建特定类型的扩展数据单元解析器
pub type ExtensionFactory =
    Box<dyn Fn(&[u8]) -> ParseResult<Box<dyn ExtensionDataUnit>> + Send + Sync>;

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

        
        log::debug!(
            "注册扩展类型: ID={}, 名称={}",
            type_id,
            self.type_names[&type_id]
        );

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
        }
        
        let type_name = self.type_names.remove(&type_id);

        
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
            Some(factory) => factory(data).map_err(|e| ExtensionError::ParseFailed {
                type_id,
                error: e.to_string(),
            }),
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
            match ExtensionManager::parse_global(type_id, data) {
                Ok(_extension) => {
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
            GenericDataUnit::from_raw(data_type, data).unwrap_or_else(|_| GenericDataUnit::Raw {
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
        fn parse(data: &[u8]) -> ParseResult<Self> {
            if data.len() != 4 {
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
            200, // 使用200而不是201以匹配TestExtension的type_id
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
        // UploadSystemStatus需要17字节: 1(对象数量) + 10(系统状态信息体) + 6(时间戳)
        let standard_data = [
            0x01, // 对象数量
            0x01, 0x01, 0x02, 0x00, // 系统状态信息体 (4字节)
            0x30, 0x15, 0x04, 0x1A, 0x0B, 0x18, // 系统状态时间戳 (6字节)
            0x2D, 0x1E, 0x0F, 0x04, 0x0B, 0x18, // 数据单元时间戳 (6字节)
        ];
        let result = parse_with_extensions(standard_type, &standard_data);

        match result {
            GenericDataUnit::UploadSystemStatus(_) => {} // 期望的结果
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
