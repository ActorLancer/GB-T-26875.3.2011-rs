//! GB26875 扩展机制使用示例
//! 
//! 展示如何使用过程宏定义自定义扩展类型

use gb26875_macros::*;

// 定义自定义命令类型

#[derive(Command)]
#[gb26875(code = 128, description = "自定义设备登录命令")]
pub struct CustomLogin;

// 定义自定义数据单元类型

#[derive(DataUnit)]
#[gb26875(type_flag = 128, description = "自定义报警数据单元")]
pub struct CustomAlarm {
    pub alarm_type: u8,
    pub alarm_level: u8, 
    pub message: String,
}


impl Default for CustomAlarm {
    fn default() -> Self {
        Self {
            alarm_type: 0,
            alarm_level: 0,
            message: String::new(),
        }
    }
}


impl Clone for CustomAlarm {
    fn clone(&self) -> Self {
        Self {
            alarm_type: self.alarm_type,
            alarm_level: self.alarm_level,
            message: self.message.clone(),
        }
    }
}


impl std::fmt::Debug for CustomAlarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomAlarm")
            .field("alarm_type", &self.alarm_type)
            .field("alarm_level", &self.alarm_level)
            .field("message", &self.message)
            .finish()
    }
}

// 定义自定义模拟量类型

#[derive(AnalogType)]
#[gb26875(code = 128, range = "-40..85", unit = "°C")]
pub struct Temperature;


fn main() {
    println!("GB26875 扩展机制示例");
    
    // 测试命令扩展
    {
        use gb26875::extension::CommandExtension;
        let login = CustomLogin;
        println!("自定义命令: 代码={}, 描述={}", login.command_code(), login.description());
    }
    
    // 测试数据单元扩展
    {
        use gb26875::extension::DataUnitExtension;
        let alarm = CustomAlarm {
            alarm_type: 1,
            alarm_level: 2,
            message: "测试报警".to_string(),
        };
        println!("自定义数据单元: 类型标志={}, 描述={}", alarm.type_flag(), alarm.description());
    }
    
    // 测试模拟量扩展
    {
        use gb26875::extension::AnalogTypeExtension;
        let temp = Temperature;
        println!("温度传感器: 代码={}, 范围={:.1}°C - {:.1}°C, 单位={}", 
                temp.analog_type_code(), temp.min_value(), temp.max_value(), temp.unit());
        
        // 测试值验证
        let test_values = [25.0, -50.0, 100.0];
        for value in test_values {
            let valid = temp.validate_value(value);
            println!("  温度值 {:.1}°C: {}", value, if valid { "有效" } else { "无效" });
        }
    }
    
    println!("示例执行完成");
}
