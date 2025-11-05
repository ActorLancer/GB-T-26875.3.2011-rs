//! 过程宏基本功能测试


use gb26875_macros::*;


mod tests {
    use super::*;
    #[derive(Command)]
    #[gb26875(code = 128, description = "测试登录命令")]
    pub struct TestLogin;

    #[derive(DataUnit)]
    #[gb26875(type_flag = 128, description = "测试报警数据")]
    pub struct TestAlarm {
        pub alarm_type: u8,
        pub alarm_level: u8,
    }

    impl Default for TestAlarm {
        fn default() -> Self {
            Self {
                alarm_type: 0,
                alarm_level: 0,
            }
        }
    }
    impl Clone for TestAlarm {
        fn clone(&self) -> Self {
            Self {
                alarm_type: self.alarm_type,
                alarm_level: self.alarm_level,
            }
        }
    }

    impl std::fmt::Debug for TestAlarm {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestAlarm")
                .field("alarm_type", &self.alarm_type)
                .field("alarm_level", &self.alarm_level)
                .finish()
        }
    }

    #[derive(SystemType)]
    #[gb26875(code = 128, description = "测试安防系统")]
    pub struct TestSecuritySystem;

    #[derive(ComponentType)]
    #[gb26875(code = 128, description = "测试传感器")]
    pub struct TestSensor;

    #[derive(AnalogType)]
    #[gb26875(code = 128, range = "-40..85", unit = "°C")]
    pub struct TestTemperature;

    #[test]
    fn test_extension_trait() {
        use gb26875::extension::ExtensionTrait;

        let login = TestLogin;
        let extension_id = login.extension_id();

        assert_eq!(extension_id.type_name, "TestLogin");
        assert_eq!(extension_id.version, env!("CARGO_PKG_VERSION"));
        assert!(extension_id.module_path.contains("macro_test"));
        assert_eq!(login.extension_type(), "command");
    }

    #[test]
    fn test_command_extension() {
        use gb26875::extension::CommandExtension;

        let login = TestLogin;
        assert_eq!(login.command_code(), 128);
        assert_eq!(login.description(), "测试登录命令");

        // 测试编码
        let encoded = login.encode();
        assert!(encoded.is_ok());

        // 测试克隆
        let cloned = login.clone_boxed();
        assert_eq!(cloned.command_code(), 128);
    }

    #[test]
    fn test_data_unit_extension() {
        use gb26875::extension::DataUnitExtension;

        let alarm = TestAlarm {
            alarm_type: 1,
            alarm_level: 2,
        };

        assert_eq!(alarm.type_flag(), 128);
        assert_eq!(alarm.description(), "测试报警数据");

        // 测试编码
        let encoded = alarm.encode();
        assert!(encoded.is_ok());

        // 测试信息对象
        let info_objects = alarm.get_info_objects();
        assert!(info_objects.is_empty()); // 默认实现返回空

        // 测试克隆
        let cloned = alarm.clone_boxed();
        assert_eq!(cloned.type_flag(), 128);
    }

    #[test]
    fn test_analog_type_extension() {
        use gb26875::extension::AnalogTypeExtension;

        let temp = TestTemperature;
        assert_eq!(temp.analog_type_code(), 128);
        assert_eq!(temp.min_value(), -40.0);
        assert_eq!(temp.max_value(), 85.0);
        assert_eq!(temp.unit(), "°C");

        // 测试值验证
        assert!(temp.validate_value(25.0));
        assert!(!temp.validate_value(-50.0));
        assert!(!temp.validate_value(100.0));
    }
}
