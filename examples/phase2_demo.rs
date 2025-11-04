//! Phase 2 功能演示
//! 
//! 展示核心编解码功能，包括：
//! - 数据包解析和编码
//! - 流式处理
//! - 数据验证
//! - 异步支持（如果启用async feature）

use gb26875::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GB26875 Phase 2 功能演示 ===\n");

    // 1. 基础数据包编解码演示
    basic_packet_demo()?;

    // 2. 流式编解码演示
    stream_codec_demo()?;

    // 3. 数据验证演示
    data_validation_demo()?;

    // 4. 编解码器构建器演示
    codec_builder_demo()?;

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
    let encoded = codec.encode(&packet)?;
    println!("   编码后大小: {} 字节", encoded.len());

    // 解码
    let decoded = codec.decode(&encoded)?;
    println!("   解码成功: 序列号={}", decoded.control_unit.sequence);

    Ok(())
}

/// 流式编解码演示
fn stream_codec_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. 流式编解码演示");
    
    // 创建流式编解码器
    let mut stream_codec = StreamCodec::new();
    
    // 创建测试数据包
    let control_unit = ControlUnit::new(
        2,
        ProtocolVersion::new(1, 0),
        Timestamp::now(),
        0x111111,
        0x222222,
        0,
        Command::SendData,
    )?;
    
    let packet = Packet::empty(control_unit);
    let encoded = stream_codec.encode(&packet)?;
    
    // 模拟分片接收
    let mid = encoded.len() / 2;
    
    // 第一片数据
    stream_codec.feed(&encoded[..mid]);
    let result1 = stream_codec.try_decode()?;
    println!("   第一片数据: {:?}", result1.is_some());
    
    // 第二片数据
    stream_codec.feed(&encoded[mid..]);
    let result2 = stream_codec.try_decode()?;
    println!("   第二片数据: 解码成功={}", result2.is_some());

    Ok(())
}

/// 数据验证演示
fn data_validation_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. 数据验证演示");
    
    let validator = DataValidator::new();
    
    // 创建有效数据包
    let control_unit = ControlUnit::new(
        3,
        ProtocolVersion::new(1, 0),
        Timestamp::now(),
        0x333333,
        0x444444,
        0,
        Command::SendData,
    )?;
    
    let packet = Packet::empty(control_unit);
    let encoded = packet.encode()?;
    
    // 验证完整帧
    match validator.validate_complete_frame(&encoded) {
        Ok(()) => println!("   帧验证: 通过"),
        Err(e) => println!("   帧验证: 失败 - {}", e),
    }
    
    // 验证基本格式
    match validator.validate_frame_format(&encoded) {
        Ok(()) => println!("   格式验证: 通过"),
        Err(e) => println!("   格式验证: 失败 - {}", e),
    }

    Ok(())
}

/// 编解码器构建器演示
fn codec_builder_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. 编解码器构建器演示");
    
    // 使用构建器创建编解码器
    let builder = CodecBuilder::new()
        .with_capacity(8192)
        .with_validation(true);
    
    let _packet_codec = builder.clone().build_packet_codec();
    let _data_unit_codec = builder.clone().build_data_unit_codec();
    let stream_codec = builder.clone().build_stream_codec();
    
    println!("   构建器创建的流式编解码器缓冲区大小: {}", stream_codec.buffer_len());
    
    #[cfg(feature = "async")]
    {
        let _framed_codec = builder.build_framed_codec();
        println!("   异步Framed编解码器: 已创建");
    }
    
    #[cfg(not(feature = "async"))]
    {
        println!("   异步功能: 未启用（需要 --features async）");
    }

    Ok(())
}
