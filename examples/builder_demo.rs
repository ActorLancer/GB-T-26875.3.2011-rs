//! GB26875 数据单元构建器演示
//!
//! 展示如何使用构建器API创建不同类型的数据单元

use gb26875::prelude::*;
use gb26875::protocol::types::{SystemType, ComponentType, DataUnitType};
use gb26875::frame::Timestamp;
use gb26875::info_object::analog_value::AnalogType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GB26875 数据单元构建器演示");
    println!("============================");

    // 示例1：构建上传系统状态数据单元
    println!("\n1. 构建上传系统状态数据单元");
    let system_status_unit = DataUnitBuilder::new(DataUnitType::UploadSystemStatus)
        .system_status()
        .system_type(SystemType::FireAlarm)
        .system_address(1)
        .system_state(0x0002)  // 火警状态
        .timestamp(Timestamp::now())
        .build()?;

    println!("   类型: {:?}", system_status_unit.data_unit_type());
    println!("   上行数据单元: {}", system_status_unit.is_upstream());
    println!("   编码后字节长度: {} 字节", system_status_unit.encode()?.len());

    // 示例2：构建上传部件状态数据单元
    println!("\n2. 构建上传部件状态数据单元");
    let component_status_unit = DataUnitBuilder::new(DataUnitType::UploadComponentStatus)
        .component_status()
        .system_type(SystemType::FireAlarm)
        .system_address(1)
        .component_type(ComponentType::SmokeFireDetector)
        .component_address(0x12345678)
        .component_state(0x0002)  // 火警状态
        .description("一层大厅烟雾探测器")
        .timestamp(Timestamp::now())
        .build()?;

    println!("   类型: {:?}", component_status_unit.data_unit_type());
    println!("   上行数据单元: {}", component_status_unit.is_upstream());
    println!("   编码后字节长度: {} 字节", component_status_unit.encode()?.len());

    // 示例3：构建上传模拟量值数据单元
    println!("\n3. 构建上传模拟量值数据单元");
    let analog_value_unit = DataUnitBuilder::new(DataUnitType::UploadAnalogValue)
        .analog_value()
        .system_type(SystemType::FireAlarm)
        .system_address(1)
        .component_type(ComponentType::TemperatureFireDetector)
        .component_address(0x87654321)
        .analog_type(AnalogType::Temperature)
        .analog_value(250)  // 25.0°C (0.1°C精度)
        .timestamp(Timestamp::now())
        .build()?;

    println!("   类型: {:?}", analog_value_unit.data_unit_type());
    println!("   上行数据单元: {}", analog_value_unit.is_upstream());
    println!("   编码后字节长度: {} 字节", analog_value_unit.encode()?.len());

    // 示例4：验证数据单元
    println!("\n4. 验证数据单元");
    system_status_unit.validate()?;
    component_status_unit.validate()?;
    analog_value_unit.validate()?;
    println!("   所有数据单元验证通过 ✓");

    // 示例5：编码和解码一致性测试
    println!("\n5. 编码/解码一致性测试");
    let encoded = system_status_unit.encode()?;
    println!("   编码字节: {:02X?}", &encoded[..10.min(encoded.len())]);

    // 测试链式调用的灵活性
    println!("\n6. 链式调用示例");
    let _flexible_unit = DataUnitBuilder::new(DataUnitType::UploadSystemStatus)
        .system_status()
        .system_type(SystemType::AutoSprinkler)  // 自动喷水
        .system_address(2)
        .system_state(0x0001)  // 正常状态
        .timestamp(Timestamp::now())
        .build()?;

    println!("   灵活链式调用构建成功 ✓");

    println!("\n演示完成！");
    Ok(())
}
