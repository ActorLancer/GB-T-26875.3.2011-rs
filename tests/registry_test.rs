//! 扩展注册表功能测试
//!
//! 测试 Phase 4.1.2 增强的注册表和命名空间冲突解决功能

#[cfg(feature = "macros")]
use gb26875::extension::{
    MacroExtensionManager, ExtensionId,
    CommandExtension, DataUnitExtension, SystemTypeExtension, ComponentTypeExtension, AnalogTypeExtension,
};

#[cfg(feature = "macros")]
use gb26875_macros::{Command, DataUnit, SystemType, ComponentType, AnalogType};

#[cfg(feature = "macros")]
#[derive(Command)]
#[gb26875(code = 150, description = "测试命令150")]
struct TestCommand150;

#[cfg(feature = "macros")]
#[derive(Command)]
#[gb26875(code = 151, description = "测试命令151")]
struct TestCommand151;

#[cfg(feature = "macros")]
#[derive(DataUnit, Debug, Clone)]
#[gb26875(type_flag = 200, description = "测试数据单元200")]
struct TestDataUnit200 {
    value: u32,
}

#[cfg(feature = "macros")]
#[derive(SystemType)]
#[gb26875(code = 200, description = "测试系统类型200")]
struct TestSystemType200;

#[cfg(feature = "macros")]
#[derive(ComponentType)]
#[gb26875(code = 220, description = "测试部件类型220")]
struct TestComponentType220;

#[cfg(feature = "macros")]
#[derive(AnalogType)]
#[gb26875(code = 230, description = "测试模拟量类型230", range = "-100.0..100.0", unit = "°C")]
struct TestAnalogType230;

#[cfg(feature = "macros")]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_registry_basic_operations() {
        // 清空注册表以确保测试环境干净
        let _ = MacroExtensionManager::clear_all_global();

        // 注册各种类型的扩展
        let command = Box::new(TestCommand150) as Box<dyn CommandExtension>;
        let result = MacroExtensionManager::register_command_global(command);
        assert!(result.is_ok(), "命令扩展注册失败: {:?}", result);

        let data_unit = Box::new(TestDataUnit200 { value: 12345 }) as Box<dyn DataUnitExtension>;
        let result = MacroExtensionManager::register_data_unit_global(data_unit);
        assert!(result.is_ok(), "数据单元扩展注册失败: {:?}", result);

        let system_type = Box::new(TestSystemType200) as Box<dyn SystemTypeExtension>;
        let result = MacroExtensionManager::register_system_type_global(system_type);
        assert!(result.is_ok(), "系统类型扩展注册失败: {:?}", result);

        let component_type = Box::new(TestComponentType220) as Box<dyn ComponentTypeExtension>;
        let result = MacroExtensionManager::register_component_type_global(component_type);
        assert!(result.is_ok(), "部件类型扩展注册失败: {:?}", result);

        let analog_type = Box::new(TestAnalogType230) as Box<dyn AnalogTypeExtension>;
        let result = MacroExtensionManager::register_analog_type_global(analog_type);
        assert!(result.is_ok(), "模拟量类型扩展注册失败: {:?}", result);

        // 验证注册成功
        assert!(MacroExtensionManager::find_command_global(150).is_some());
        assert!(MacroExtensionManager::find_data_unit_global(200).is_some());
        assert!(MacroExtensionManager::find_system_type_global(200).is_some());
        assert!(MacroExtensionManager::find_component_type_global(220).is_some());
        assert!(MacroExtensionManager::find_analog_type_global(230).is_some());
    }

    #[test]
    fn test_extension_id_management() {
        // 清空注册表
        let _ = MacroExtensionManager::clear_all_global();

        // 注册一个扩展
        let command = Box::new(TestCommand151) as Box<dyn CommandExtension>;
        let _ = MacroExtensionManager::register_command_global(command);

        // 获取所有扩展ID
        let extension_ids = MacroExtensionManager::list_extension_ids_global();
        assert!(!extension_ids.is_empty(), "应该有至少一个扩展ID");

        // 检查特定扩展ID是否注册
        if let Some(first_id) = extension_ids.first() {
            assert!(MacroExtensionManager::is_extension_id_registered_global(first_id));
            
            // 获取扩展信息
            let info = MacroExtensionManager::get_extension_info_global(first_id);
            assert!(info.is_some(), "应该能获取扩展信息");
            
            if let Some(info) = info {
                assert_eq!(info.code, 151);
                assert_eq!(info.extension_type, "Command");
                assert!(!info.description.is_empty());
            }
        }
    }

    #[test]
    fn test_extension_search_capabilities() {
        // 清空注册表
        let _ = MacroExtensionManager::clear_all_global();

        // 注册测试扩展
        let command = Box::new(TestCommand150) as Box<dyn CommandExtension>;
        let _ = MacroExtensionManager::register_command_global(command);

        // 按名称查找扩展
        let extensions = MacroExtensionManager::find_extension_by_name_global(
            "registry_test", 
            "TestCommand150"
        );
        
        // 可能找不到，因为模块路径可能不同，这是正常的
        // 主要是测试方法是否正常工作
        println!("找到的扩展: {:?}", extensions);

        // 测试版本查找
        let code = MacroExtensionManager::find_extension_by_version_global(
            "registry_test",
            "TestCommand150", 
            "1.0.0"
        );
        
        // 可能为None，这是正常的，主要是测试方法不崩溃
        println!("找到的代码: {:?}", code);
    }

    #[test]
    fn test_conflict_detection_and_available_codes() {
        // 清空注册表
        let _ = MacroExtensionManager::clear_all_global();

        // 注册一个扩展
        let command = Box::new(TestCommand150) as Box<dyn CommandExtension>;
        let _ = MacroExtensionManager::register_command_global(command);

        // 检查代码冲突
        let conflicts = MacroExtensionManager::check_code_conflicts_global(150);
        assert!(!conflicts.is_empty(), "代码150应该有冲突");

        // 检查没有注册的代码
        let no_conflicts = MacroExtensionManager::check_code_conflicts_global(199);
        assert!(no_conflicts.is_empty(), "代码199应该没有冲突");

        // 建议可用代码
        let available_codes = MacroExtensionManager::suggest_available_codes_global(10);
        assert!(!available_codes.is_empty(), "应该有可用代码");
        assert!(available_codes.len() <= 10, "返回的代码数量不应超过请求数量");
        
        // 确保建议的代码都在有效范围内
        for code in &available_codes {
            assert!(*code >= 128 && *code <= 254, "建议的代码应该在128-254范围内");
        }
    }

    #[test]
    fn test_registry_statistics() {
        // 清空注册表
        let _ = MacroExtensionManager::clear_all_global();

        // 注册不同类型的扩展
        let command = Box::new(TestCommand150) as Box<dyn CommandExtension>;
        let _ = MacroExtensionManager::register_command_global(command);

        let data_unit = Box::new(TestDataUnit200 { value: 999 }) as Box<dyn DataUnitExtension>;
        let _ = MacroExtensionManager::register_data_unit_global(data_unit);

        // 获取统计信息
        let stats = MacroExtensionManager::get_registry_stats_global();
        
        assert!(stats.total_extensions >= 2, "总扩展数量应该至少为2");
        assert!(stats.command_extensions >= 1, "命令扩展数量应该至少为1");
        assert!(stats.data_unit_extensions >= 1, "数据单元扩展数量应该至少为1");
        assert!(stats.used_codes >= 2, "已使用的代码数量应该至少为2");
        assert!(stats.available_codes <= 125, "可用代码数量应该不超过125");
        
        println!("注册表统计信息: {:?}", stats);
    }

    #[test]
    fn test_extension_listing() {
        // 清空注册表
        let _ = MacroExtensionManager::clear_all_global();

        // 注册多个扩展
        let command = Box::new(TestCommand150) as Box<dyn CommandExtension>;
        let _ = MacroExtensionManager::register_command_global(command);

        let system_type = Box::new(TestSystemType200) as Box<dyn SystemTypeExtension>;
        let _ = MacroExtensionManager::register_system_type_global(system_type);

        // 列出所有扩展信息
        let all_extensions = MacroExtensionManager::list_all_extensions_global();
        assert!(all_extensions.len() >= 2, "应该有至少2个扩展");

        // 验证扩展信息的格式
        for extension_info in &all_extensions {
            assert!(!extension_info.extension_id.module_path.is_empty());
            assert!(!extension_info.extension_id.type_name.is_empty());
            assert!(!extension_info.extension_id.version.is_empty());
            assert!(extension_info.code >= 128 && extension_info.code <= 254);
            assert!(!extension_info.extension_type.is_empty());
            assert!(!extension_info.description.is_empty());
        }

        println!("所有扩展信息:");
        for info in &all_extensions {
            println!("  - {}: {} (代码: {})", 
                info.extension_id.full_name(), 
                info.description, 
                info.code
            );
        }
    }

    #[test]
    fn test_namespace_conflict_resolution() {
        // 测试版本冲突解决
        let extension_ids = vec![
            ExtensionId::new("test", "MyExtension", "1.0.0"),
            ExtensionId::new("test", "MyExtension", "2.0.0"),
            ExtensionId::new("test", "MyExtension", "1.5.0"),
        ];

        let best_version = MacroExtensionManager::resolve_namespace_conflict_by_version(&extension_ids);
        assert!(best_version.is_some());
        
        if let Some(best) = best_version {
            assert_eq!(best.version, "2.0.0", "应该选择最新版本");
        }

        // 测试优先级冲突解决
        let extension_ids = vec![
            ExtensionId::new("lib", "MyExtension", "1.0.0"),
            ExtensionId::new("lib::submodule", "MyExtension", "1.0.0"),
            ExtensionId::new("app::module", "MyExtension", "1.0.0"),
        ];

        let best_priority = MacroExtensionManager::resolve_namespace_conflict_by_priority(&extension_ids);
        assert!(best_priority.is_some());
        
        if let Some(best) = best_priority {
            // 应该选择模块路径更深的扩展
            assert!(best.module_path.contains("::"), "应该选择更深的模块路径");
        }
    }

    #[test]
    fn test_extension_compatibility_validation() {
        // 测试有效的扩展ID
        let valid_id = ExtensionId::new("test::module", "MyExtension", "1.0.0");
        let result = MacroExtensionManager::validate_extension_compatibility(&valid_id);
        assert!(result.is_ok(), "有效的扩展ID应该通过验证");

        // 测试无效的扩展ID（空字段）
        let invalid_id = ExtensionId::new("", "MyExtension", "1.0.0");
        let result = MacroExtensionManager::validate_extension_compatibility(&invalid_id);
        assert!(result.is_err(), "空模块路径应该验证失败");

        // 测试无效的版本格式
        let invalid_version_id = ExtensionId::new("test", "MyExtension", "");
        let result = MacroExtensionManager::validate_extension_compatibility(&invalid_version_id);
        assert!(result.is_err(), "空版本应该验证失败");
    }

    #[test]
    fn test_extension_dependencies() {
        // 测试扩展依赖功能（目前是占位符实现）
        let extension_id = ExtensionId::new("test", "MyExtension", "1.0.0");
        let dependencies = MacroExtensionManager::get_extension_dependencies(&extension_id);
        
        // 目前应该返回空列表
        assert!(dependencies.is_empty(), "目前的实现应该返回空依赖列表");
    }

    #[test]
    fn test_clear_all_functionality() {
        // 注册一些扩展
        let command = Box::new(TestCommand150) as Box<dyn CommandExtension>;
        let _ = MacroExtensionManager::register_command_global(command);

        let data_unit = Box::new(TestDataUnit200 { value: 777 }) as Box<dyn DataUnitExtension>;
        let _ = MacroExtensionManager::register_data_unit_global(data_unit);

        // 验证注册成功
        let stats_before = MacroExtensionManager::get_registry_stats_global();
        assert!(stats_before.total_extensions > 0, "清空前应该有扩展");

        // 清空所有注册
        let result = MacroExtensionManager::clear_all_global();
        assert!(result.is_ok(), "清空操作应该成功");

        // 验证清空成功
        let stats_after = MacroExtensionManager::get_registry_stats_global();
        assert_eq!(stats_after.total_extensions, 0, "清空后应该没有扩展");
        assert_eq!(stats_after.used_codes, 0, "清空后应该没有使用的代码");
    }
}
