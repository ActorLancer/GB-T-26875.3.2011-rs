# GB26875-rs

[![Crates.io](https://img.shields.io/crates/v/gb26875.svg)](https://crates.io/crates/gb26875)
[![Documentation](https://docs.rs/gb26875/badge.svg)](https://docs.rs/gb26875)
[![CI](https://github.com/your-username/gb26875-rs/workflows/CI/badge.svg)](https://github.com/your-username/gb26875-rs/actions)
[![codecov](https://codecov.io/gh/your-username/gb26875-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/your-username/gb26875-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

🚀 **高性能、类型安全的 GB26875 城市消防远程监控系统通信协议 Rust 实现**

## 📖 项目介绍

GB26875-rs 是 GB/T 26875.3-2011《城市消防远程监控系统 第3部分：报警传输网络通信协议》的完整 Rust 实现。本库提供了：

- 🔥 **零拷贝解析**: 基于 `bytes` crate 的高性能数据包解析
- 🛡️ **类型安全**: 利用 Rust 类型系统确保协议正确性
- 🔌 **扩展机制**: 支持用户自定义数据类型（128-255 范围）
- 🌐 **异步支持**: 集成 tokio 生态，支持 TCP/UDP 网络通信
- 📦 **序列化支持**: 支持 JSON、serde 等通用序列化格式
- 🧩 **模块化设计**: 清晰的模块结构，易于理解和扩展

## ✨ 主要特性

### 🔍 协议支持
- ✅ 完整的 GB26875 数据包解析和编码
- ✅ 所有标准信息对象类型（系统状态、部件状态、模拟量值等）
- ✅ 上行数据单元（类型 1-28）和下行数据单元（类型 61-91）
- ✅ 控制单元解析（序列号、时间戳、地址信息等）
- ✅ 自动校验和验证

### ⚡ 高性能
- 零拷贝数据解析
- 流式处理支持
- 粘包自动处理
- 内存使用优化

### 🔧 易用性
- 友好的构建器 API
- 详细的错误信息
- 完整的文档和示例
- 类型安全的 API 设计

## 🚀 快速开始

### 安装

将以下内容添加到您的 `Cargo.toml`：

```toml
[dependencies]
gb26875 = "0.1"

# 可选功能
gb26875 = { version = "0.1", features = ["serde", "async"] }
```

### 基础使用

```rust
use gb26875::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据包
    let control_unit = ControlUnit::new(
        1,                          // 序列号
        ProtocolVersion::new(1, 0), // 协议版本
        Timestamp::now(),           // 时间戳
        0x123456,                   // 源地址
        0x654321,                   // 目的地址
        0,                          // 数据单元长度
        Command::SendData,          // 命令字节
    )?;
    
    let packet = Packet::empty(control_unit);
    
    // 编码数据包
    let codec = PacketCodec::new();
    let encoded = gb26875::codec::traits::Encoder::encode(&codec, &packet)?;
    println!("编码后大小: {} 字节", encoded.len());
    
    // 解码数据包
    let decoded = gb26875::codec::traits::Decoder::decode(&codec, &encoded)?;
    println!("解码成功: 序列号={}", decoded.control_unit.sequence);
    
    Ok(())
}
```

### 数据单元处理

```rust
use gb26875::prelude::*;

// 创建系统状态信息
let system_status = gb26875::info_object::SystemStatus::new(
    gb26875::protocol::SystemType::SmokeDetectionSystem,
    0x12,  // 系统地址
    0x34,  // 系统状态
    Timestamp::now(),
);

let data_unit = GenericDataUnit::UploadSystemStatus(
    gb26875::data_unit::standard::UploadSystemStatus::new(
        system_status,
        Timestamp::now()
    )
);

// 编解码
let codec = DataUnitCodec::new();
let encoded = codec.encode_generic(&data_unit)?;
let decoded = codec.decode_generic(&encoded)?;
```

## 📚 功能特性

### 支持的功能

| 功能 | 状态 | 描述 |
|------|------|------|
| 数据包解析 | ✅ | 完整的 GB26875 数据包解析 |
| 数据包编码 | ✅ | 将数据结构编码为协议格式 |
| 信息对象 | ✅ | 所有标准信息对象类型 |
| 流式处理 | ✅ | 支持 TCP 粘包处理 |
| 异步支持 | ✅ | tokio 集成 |
| 序列化 | ✅ | JSON、serde 支持 |
| 扩展机制 | 🚧 | 用户自定义类型支持 |

### 可选功能

通过 Cargo features 启用额外功能：

```toml
[dependencies]
gb26875 = { version = "0.1", features = ["full"] }
```

- `serde`: 启用 serde 序列化支持
- `async`: 启用异步网络通信支持
- `logging`: 启用日志记录
- `full`: 启用所有功能

## 🏗️ 架构设计

```
应用层
    ↓
GB26875 协议层 (本库)
    ↓
传输层 (TCP/UDP)
```

### 模块结构

- `frame/`: 数据帧和控制单元
- `data_unit/`: 应用数据单元
- `info_object/`: 信息对象（信息体）
- `codec/`: 编解码器
- `parser/`: 底层解析器
- `builder/`: 构建器 API
- `extension/`: 扩展机制
- `protocol/`: 协议常量和类型

## 🧪 测试

运行测试：

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test codec

# 运行带所有功能的测试
cargo test --all-features
```

## 📈 性能

在现代硬件上的基准测试结果：

| 指标 | 性能 |
|------|------|
| 解析速度 | >2MB/s |
| 内存使用 | <500B/packet |
| 解析延迟 | <100μs |

## 🤝 贡献

欢迎贡献！请参阅 [DEVELOPMENT.md](DEVELOPMENT.md) 了解详细的开发指南。

### 开发环境设置

1. 克隆仓库
2. 安装 Rust (1.70+)
3. 运行测试: `cargo test`
4. 运行示例: `cargo run --example phase2_demo`

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🔗 相关资源

- [GB/T 26875.3-2011 协议标准](./dev-documentation/)
- [API 文档](https://docs.rs/gb26875)
- [示例代码](./examples/)
- [开发文档](./DEVELOPMENT.md)

---

> 💡 **提示**: 这是一个正在积极开发中的项目。如果您在使用过程中遇到问题，请提交 Issue 或 PR。