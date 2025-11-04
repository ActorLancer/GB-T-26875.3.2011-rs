//! Phase 2 功能演示
//! 
//! 展示核心编解码功能

use gb26875::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GB26875 Phase 2 功能演示 ===\n");

    // 基础数据包编解码演示
    basic_packet_demo()?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 基础数据包编解码演示
fn basic_packet_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. 基础数据包编解码演示");
    
    // 创建控制单元
    let control_unit = ControlUnit::new(
        1,                   // 序列号
        ProtocolVersion::new(1, 0), // 协议版本
        Timestamp::now(),           // 时间戳
        0x123456,                   // 源地址
        0x654321,                   // 目的地址
        0,                          // 数据单元长度（会自动设置）
        Command::SendData,          // 命令
    )?;

    // 创建数据包
    let packet = Packet::empty(control_unit);
    
    // 编码
    let codec = PacketCodec::new();
    let encoded = gb26875::codec::traits::Encoder::encode(&codec, &packet)?;
    println!("   编码后大小: {} 字节", encoded.len());

    // 解码
    let decoded = gb26875::codec::traits::Decoder::decode(&codec, &encoded)?;
    println!("   解码成功: 序列号={}", decoded.control_unit.sequence);

    Ok(())
}
