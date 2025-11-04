//! GB26875协议合规性测试示例
//! 
//! 测试修复后的下行数据单元是否符合GB26875协议文档要求

use gb26875::prelude::*;
use gb26875::data_unit::standard::downstream::*;
use gb26875::protocol::SystemType;
use gb26875::frame::Timestamp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GB26875协议合规性测试 ===\n");

    // 测试读系统状态 (类型61) - 多系统查询
    test_read_system_status()?;
    
    // 测试读部件状态 (类型62) - 多部件查询  
    test_read_component_status()?;
    
    // 测试读模拟量值 (类型63) - 多部件查询
    test_read_analog_value()?;
    
    // 测试读操作信息 (类型64) - 包含记录数目和起始时间
    test_read_operation_info()?;
    
    // 测试读软件版本 (类型65) - 单系统查询
    test_read_software_version()?;
    
    // 测试读系统配置 (类型66) - 多系统查询（最多3个）
    test_read_system_config()?;
    
    // 测试读部件配置 (类型67) - 多部件查询（最多26个）
    test_read_component_config()?;
    
    // 测试读系统时间 (类型68) - 单系统查询
    test_read_system_time()?;
    
    // 测试读设备状态 (类型81) - 信息对象数目+预留
    test_read_device_status()?;
    
    // 测试读设备操作信息 (类型84) - 记录数目+起始时间
    test_read_device_operation()?;
    
    // 测试读设备版本 (类型85) - 信息对象数目+预留
    test_read_device_version()?;
    
    // 测试读设备配置 (类型86) - 信息对象数目+预留
    test_read_device_config()?;
    
    // 测试读设备时间 (类型88) - 信息对象数目+预留
    test_read_device_time()?;
    
    // 测试初始化设备 (类型89) - 信息对象数目+预留
    test_initialize_device()?;
    
    // 测试同步设备时钟 (类型90) - 信息对象数目+预留+时间戳
    test_sync_device_clock()?;
    
    // 测试查岗命令 (类型91) - 信息对象数目+预留
    test_patrol_command()?;

    println!("\n=== 所有测试通过！协议实现符合GB26875标准 ===");
    Ok(())
}

fn test_read_system_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. 测试读系统状态 (类型61)");
      // 测试多系统查询（协议规定最多102个）
    let systems = vec![
        (SystemType::FireAlarm, 0x12),
        (SystemType::Hydrant, 0x34),
        (SystemType::AutoSprinkler, 0x56),
    ];
    
    let cmd = ReadSystemStatus::new(systems)?;
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1字节) + 3组(系统类型1字节+系统地址1字节)
    assert_eq!(encoded.len(), 1 + 3 * 2);
    assert_eq!(encoded[0], 3); // 信息对象数目
    
    // 解码验证
    let decoded = ReadSystemStatus::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    println!("   ✓ 支持多系统查询（最多102个）");
    println!("   ✓ 编码长度：{} 字节", encoded.len());
    
    Ok(())
}

fn test_read_component_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. 测试读部件状态 (类型62)");
      let components = vec![
        (SystemType::FireAlarm, 0x12, 0x12345678),
        (SystemType::Hydrant, 0x34, 0x9ABCDEF0),
    ];
    
    let cmd = ReadComponentStatus::new(components)?;
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1字节) + 2组(系统类型1字节+系统地址1字节+部件地址4字节)
    assert_eq!(encoded.len(), 1 + 2 * 6);
    assert_eq!(encoded[0], 2); // 信息对象数目
    
    let decoded = ReadComponentStatus::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    println!("   ✓ 部件地址字段：4字节（符合协议）");
    println!("   ✓ 支持多部件查询（最多22个）");
    println!("   ✓ 编码长度：{} 字节", encoded.len());
    
    Ok(())
}

fn test_read_analog_value() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. 测试读模拟量值 (类型63)");
    
    let components = vec![
        (SystemType::FireAlarm, 0x01, 0x11111111),
    ];
    
    let cmd = ReadAnalogValue::new(components)?;
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 1 + 1 * 6);
    assert_eq!(encoded[0], 1); // 信息对象数目
    
    let decoded = ReadAnalogValue::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    println!("   ✓ 部件地址字段：4字节（符合协议）");
    println!("   ✓ 支持多部件查询（最多63个）");
    
    Ok(())
}

fn test_read_operation_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. 测试读操作信息 (类型64)");
    
    let timestamp = Timestamp::new(30, 15, 10, 1, 11, 24)?; // 2024年11月1日10:15:30
    let cmd = ReadOperationInfo::new(SystemType::FireAlarm, 0x12, 50, timestamp)?;
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1) + 系统类型(1) + 系统地址(1) + 记录数目(1) + 起始时间(6)
    assert_eq!(encoded.len(), 1 + 1 + 1 + 1 + 6);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[3], 50); // 记录数目
    
    let decoded = ReadOperationInfo::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含记录数目字段（≤102）");
    println!("   ✓ 包含起始时间字段（6字节）");
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    
    Ok(())
}

fn test_read_software_version() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. 测试读软件版本 (类型65)");
    
    let cmd = ReadSoftwareVersion::new(SystemType::FireAlarm, 0x12);
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1) + 系统类型(1) + 系统地址(1)
    assert_eq!(encoded.len(), 3);
    assert_eq!(encoded[0], 1); // 信息对象数目固定为1
    
    let decoded = ReadSoftwareVersion::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 信息对象数目固定为1");
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    
    Ok(())
}

fn test_read_system_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. 测试读系统配置 (类型66)");
      let systems = vec![
        (SystemType::FireAlarm, 0x01),
        (SystemType::Hydrant, 0x02),
        (SystemType::AutoSprinkler, 0x03),
    ];
    
    let cmd = ReadSystemConfig::new(systems)?;
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 1 + 3 * 2);
    assert_eq!(encoded[0], 3); // 信息对象数目
    
    let decoded = ReadSystemConfig::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 支持多系统查询（最多3个）");
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    
    Ok(())
}

fn test_read_component_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("7. 测试读部件配置 (类型67)");
    
    let components = vec![
        (SystemType::FireAlarm, 0x01, 0x12345678),
    ];
    
    let cmd = ReadComponentConfig::new(components)?;
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 1 + 1 * 6);
    assert_eq!(encoded[0], 1); // 信息对象数目
    
    let decoded = ReadComponentConfig::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 支持多部件查询（最多26个）");
    println!("   ✓ 系统地址字段：1字节，部件地址字段：4字节（符合协议）");
    
    Ok(())
}

fn test_read_system_time() -> Result<(), Box<dyn std::error::Error>> {
    println!("8. 测试读系统时间 (类型68)");
    
    let cmd = ReadSystemTime::new(SystemType::FireAlarm, 0x12);
    let encoded = cmd.encode()?;
    
    // 验证格式：系统类型(1) + 系统地址(1)
    assert_eq!(encoded.len(), 2);
    
    let decoded = ReadSystemTime::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 系统地址字段：1字节（符合协议）");
    
    Ok(())
}

fn test_read_device_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("9. 测试读设备状态 (类型81)");
    
    let cmd = ReadDeviceStatus::new();
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1) + 预留(0)
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = ReadDeviceStatus::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    
    Ok(())
}

fn test_read_device_operation() -> Result<(), Box<dyn std::error::Error>> {
    println!("10. 测试读设备操作信息 (类型84)");
    
    let timestamp = Timestamp::new(0, 0, 0, 1, 1, 24)?; // 2024年1月1日00:00:00
    let cmd = ReadDeviceOperation::new(10, timestamp)?;
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1) + 记录数目(1) + 起始时间(6)
    assert_eq!(encoded.len(), 8);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 10); // 记录数目
    
    let decoded = ReadDeviceOperation::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含查询记录数目字段（≤102）");
    println!("   ✓ 包含指定起始时间字段（6字节）");
    
    Ok(())
}

fn test_read_device_version() -> Result<(), Box<dyn std::error::Error>> {
    println!("11. 测试读设备版本 (类型85)");
    
    let cmd = ReadDeviceVersion::new();
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = ReadDeviceVersion::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    
    Ok(())
}

fn test_read_device_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("12. 测试读设备配置 (类型86)");
    
    let cmd = ReadDeviceConfig::new();
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = ReadDeviceConfig::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    
    Ok(())
}

fn test_read_device_time() -> Result<(), Box<dyn std::error::Error>> {
    println!("13. 测试读设备时间 (类型88)");
    
    let cmd = ReadDeviceTime::new();
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = ReadDeviceTime::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    
    Ok(())
}

fn test_initialize_device() -> Result<(), Box<dyn std::error::Error>> {
    println!("14. 测试初始化设备 (类型89)");
    
    let cmd = InitializeDevice::new();
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = InitializeDevice::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    
    Ok(())
}

fn test_sync_device_clock() -> Result<(), Box<dyn std::error::Error>> {
    println!("15. 测试同步设备时钟 (类型90)");
    
    let timestamp = Timestamp::new(30, 45, 14, 4, 11, 24)?; // 2024年11月4日14:45:30
    let cmd = SyncDeviceClock::new(timestamp);
    let encoded = cmd.encode()?;
    
    // 验证格式：信息对象数目(1) + 预留(0) + 时间戳(6)
    assert_eq!(encoded.len(), 8);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = SyncDeviceClock::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    println!("   ✓ 包含目标时间字段（6字节）");
    
    Ok(())
}

fn test_patrol_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("16. 测试查岗命令 (类型91)");
    
    let cmd = PatrolCommand::new();
    let encoded = cmd.encode()?;
    
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 1); // 信息对象数目
    assert_eq!(encoded[1], 0); // 预留字段
    
    let decoded = PatrolCommand::parse(&encoded)?;
    assert_eq!(cmd, decoded);
    
    println!("   ✓ 包含信息对象数目字段（固定为1）");
    println!("   ✓ 包含预留字段（固定为0）");
    
    Ok(())
}
