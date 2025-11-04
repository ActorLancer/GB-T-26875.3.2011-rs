# GB26875 Rust 实现开发文档

## 📋 项目概述

### 设计目标
- 实现 GB26875 城市消防远程监控系统通信协议的完整 Rust 库
- 提供高性能、类型安全的数据帧解析和编码功能
- 支持用户自定义扩展机制，满足不同厂商的定制需求
- 提供统一的序列化/反序列化接口，便于与其他系统集成
- 支持异步网络通信（TCP/UDP）
- 具备良好的可扩展性，适合发布到 crates.io

### 核心功能
1. **协议解析**: 解析 GB26875 标准数据帧，提取报警、状态、操作等信息
2. **协议编码**: 将高层数据结构编码为符合 GB26875 标准的数据帧
3. **扩展机制**: 支持用户自定义类型（128-255 范围）的动态注册和解析
4. **通用序列化**: 支持 JSON、MessagePack 等通用格式的数据转换
5. **异步通信**: 集成 tokio 生态，支持 TCP/UDP 网络通信
6. **粘包处理**: 处理 TCP 传输中的数据粘包问题

## 🏗️ 设计思路

### 1. 协议层次结构
```
应用层数据 (Application Data)
    ↓
GB26875 数据包 (Packet)
    ↓
TCP/UDP 传输层 (Transport Layer)
```

### 2. 扩展机制设计
采用 **Trait Object + 全局注册表** 的方式实现插件化架构：

- **编译时扩展**: 使用宏自动生成扩展代码，性能更好
- **运行时扩展**: 使用 trait object 动态注册，灵活性更高
- **统一接口**: 所有扩展都实现相同的 trait，保证一致性

### 3. 用户自定义支持范围
根据 GB26875 协议文档，以下字段支持用户自定义（128-255 范围）：

| 字段类型 | 位置 | 范围 | 说明 |
|---------|------|------|------|
| 协议版本号 | 控制单元 | 用户版本号字段 | 直接解析，无需注册 |
| 命令字节 | 控制单元 | 128~255 | 用户自定义命令类型 |
| 数据单元类型标志 | 应用数据单元 | 128~254 | 用户自定义数据单元 |
| 系统类型标志 | 信息体 | 128~255 | 用户自定义系统类型 |
| 部件类型标志 | 信息体 | 128~255 | 用户自定义部件类型 |
| 模拟量类型 | 信息体 | 128~255 | 用户自定义模拟量类型 |

### 4. 日志系统设计
- 使用 `log` facade，不依赖具体实现
- 通过 `logging` feature 控制日志代码编译
- 外部应用负责初始化日志实现（env_logger、tracing 等）
- 使用特定的 target（`gb26875::*`）便于日志过滤

## 🔧 实现思路

### 1. 核心模块职责

| 模块 | 职责 | 依赖关系 |
|------|------|----------|
| `protocol` | 协议常量和类型定义 | 基础模块 |
| `frame` | 数据帧结构定义 | → protocol |
| `codec` | 编解码核心逻辑 | → frame, extension |
| `extension` | 扩展机制实现 | → protocol |
| `builder` | 友好的构建 API | → codec |
| `transport` | 网络传输层 | → codec |

### 2. 数据流向

**解码流程**:
```
原始字节 → FrameParser → Packet → DataUnitCodec → 标准/自定义数据 → JSON等格式
```

**编码流程**:
```
高层数据 → PacketBuilder → Packet → PacketCodec → 字节流 → 网络传输
```

### 3. 扩展注册流程
```rust
// 1. 定义自定义数据类型
struct MyCustomAlarm { ... }

// 2. 实现扩展 trait
impl CustomDataUnit for MyCustomAlarm { ... }
impl DataUnitExtension for MyCustomAlarmExtension { ... }

// 3. 注册到全局注册表
ExtensionRegistry::global().register(MyCustomAlarmExtension)?;

// 4. 解析时自动调用
let packet = gb26875::parse(&raw_data)?; // 自动处理自定义类型
```

## 📁 项目结构

```
gb26875/
├── Cargo.toml                  # 项目配置
├── README.md                   # 使用说明
├── LICENSE-MIT                 # MIT 许可证
├── LICENSE-APACHE              # Apache 许可证
├── CHANGELOG.md                # 版本更新日志
├── DEVELOPMENT.md              # 开发文档（本文件）
├── TODO.md                     # 开发任务清单
│
├── examples/                   # 使用示例
│   ├── parse_packet.rs        # 解析数据包示例
│   ├── build_packet.rs        # 构建数据包示例
│   ├── tcp_server.rs          # TCP 服务器示例
│   ├── tcp_client.rs          # TCP 客户端示例
│   ├── custom_extension.rs    # 自定义扩展示例
│   ├── batch_processing.rs    # 批量处理示例
│   └── logging_demo.rs        # 日志使用示例
│
├── benches/                    # 性能基准测试
│   ├── parse_bench.rs         # 解析性能测试
│   ├── encode_bench.rs        # 编码性能测试
│   └── codec_bench.rs         # 编解码综合测试
│
├── tests/                      # 集成测试
│   ├── protocol_tests.rs      # 协议标准测试
│   ├── extension_tests.rs     # 扩展机制测试
│   ├── codec_tests.rs         # 编解码测试
│   ├── real_world_cases.rs    # 真实案例测试
│   └── fixtures/              # 测试数据
│       ├── valid_frames.bin   # 有效数据帧
│       └── test_cases.json    # 测试用例定义
│
└── src/                        # 源代码
    ├── lib.rs                 # 库入口，导出公共 API
    ├── prelude.rs             # 常用类型和 trait 的预导入
    │
    ├── protocol/              # 协议基础定义 📋
    │   ├── mod.rs            # 模块入口
    │   ├── constants.rs      # 协议常量（启动符、结束符等）
    │   ├── types.rs          # 枚举类型（系统类型、部件类型等）
    │   └── commands.rs       # 命令字节定义（1-6, 128-255）
    │
    ├── frame/                 # 数据帧结构 🔗
    │   ├── mod.rs            # 模块入口
    │   ├── header.rs         # 控制单元（25字节）
    │   ├── packet.rs         # 完整数据包结构
    │   ├── checksum.rs       # 校验和计算
    │   └── timestamp.rs      # 时间标签（6字节）
    │
    ├── data_unit/             # 应用数据单元 📊
    │   ├── mod.rs            # 模块入口
    │   ├── identifier.rs     # 数据单元标识符
    │   ├── standard/         # 标准数据单元（1-127）
    │   │   ├── mod.rs        # 标准类型入口
    │   │   ├── upstream.rs   # 上行数据（1-28）
    │   │   └── downstream.rs # 下行数据（61-91）
    │   └── custom.rs         # 自定义数据单元抽象
    │
    ├── info_object/           # 信息对象（信息体）📝
    │   ├── mod.rs            # 模块入口
    │   ├── system_status.rs  # 建筑消防设施系统状态
    │   ├── component_status.rs # 部件状态（40字节）
    │   ├── analog_value.rs   # 模拟量值（10字节）
    │   ├── operation.rs      # 操作信息（2-4字节）
    │   ├── version.rs        # 版本信息
    │   └── config.rs         # 配置信息（变长）
    │
    ├── codec/                 # 编解码器 ⚙️ 核心模块
    │   ├── mod.rs            # 模块入口
    │   ├── traits.rs         # Codec trait 定义
    │   ├── packet_codec.rs   # 数据包级编解码
    │   ├── data_unit_codec.rs # 数据单元级编解码
    │   ├── encoder.rs        # 通用编码器
    │   ├── decoder.rs        # 通用解码器
    │   └── framed.rs         # Tokio Framed Codec 集成
    │
    ├── parser/                # 底层解析器 🔍
    │   ├── mod.rs            # 模块入口
    │   ├── frame_parser.rs   # 帧边界解析
    │   ├── buffer.rs         # 缓冲区管理（处理粘包）
    │   └── validator.rs      # 数据校验逻辑
    │
    ├── builder/               # 构建器 🔨 友好 API
    │   ├── mod.rs            # 模块入口
    │   ├── packet_builder.rs # 数据包构建器
    │   ├── data_unit_builder.rs # 数据单元构建器
    │   └── fluent.rs         # 流式 API 封装
    │
    ├── extension/             # 扩展机制 🔌 核心设计
    │   ├── mod.rs            # 模块入口
    │   ├── traits.rs         # 扩展相关 trait 定义
    │   ├── registry.rs       # 全局扩展注册表
    │   ├── macros.rs         # 扩展定义辅助宏
    │   ├── unknown.rs        # 未知类型的默认处理
    │   └── examples/         # 内置扩展示例
    │       ├── mod.rs        # 示例模块入口
    │       └── custom_alarm.rs # 自定义报警类型示例
    │
    ├── serde/                 # 序列化支持 💾
    │   ├── mod.rs            # 模块入口
    │   ├── json.rs           # JSON 序列化
    │   └── custom.rs         # 自定义序列化格式
    │
    ├── transport/             # 传输层 🌐（可选）
    │   ├── mod.rs            # 模块入口
    │   ├── tcp.rs            # TCP 客户端/服务器
    │   └── udp.rs            # UDP 支持
    │
    ├── error.rs               # 错误类型定义 ❌
    ├── utils.rs               # 工具函数（编码转换等）🔧
    └── logging.rs             # 日志辅助宏（仅当 logging feature 启用）📄
```

## 📋 开发任务清单 (TODO)

### 🟢 Phase 1: 基础框架（优先级：高）
- [ ] **1.1** 项目初始化
  - [ ] 配置 Cargo.toml（依赖、features、元数据）
  - [ ] 设置 CI/CD（GitHub Actions）
  - [ ] 编写基础 README.md
- [ ] **1.2** 协议基础定义 (`protocol/`)
  - [ ] 定义协议常量（启动符、结束符等）
  - [ ] 实现标准类型枚举（系统类型、部件类型等）
  - [ ] 定义命令字节枚举
- [ ] **1.3** 数据帧结构 (`frame/`)
  - [ ] 实现时间标签结构（6字节）
  - [ ] 实现控制单元结构（25字节）
  - [ ] 实现数据包结构
  - [ ] 实现校验和计算

### 🟡 Phase 2: 核心编解码（优先级：高）
- [ ] **2.1** 基础解析器 (`parser/`)
  - [ ] 实现帧边界检测
  - [ ] 实现粘包处理缓冲区
  - [ ] 实现数据校验逻辑
- [ ] **2.2** 编解码器核心 (`codec/`)
  - [ ] 定义 Codec trait
  - [ ] 实现数据包编解码器
  - [ ] 实现数据单元编解码器
  - [ ] 集成 tokio-util Framed

### 🟢 Phase 3: 信息对象实现（优先级：中）
- [x] **3.1** 标准信息对象 (`info_object/`)
  - [x] 系统状态（4字节）
  - [x] 部件状态（40字节）
  - [x] 模拟量值（10字节）
  - [x] 操作信息（2-4字节）
  - [x] 版本信息
  - [x] 配置信息（变长）
- [x] **3.2** 数据单元实现 (`data_unit/`)
  - [x] 上行数据单元（类型1-28）
  - [x] 下行数据单元（类型61-91）
  - [x] 数据单元标识符处理
  - [x] GenericDataUnit系统重构
  - [x] Builder系统重构
  - [ ] 下行数据单元（类型61-91）
  - [ ] 数据单元标识符处理

### 🔵 Phase 4: 扩展机制（优先级：高）
- [ ] **4.1** 扩展框架 (`extension/`)
  - [ ] 定义扩展 trait（CustomDataUnit、DataUnitExtension）
  - [ ] 实现全局注册表（线程安全）
  - [ ] 实现未知类型的默认处理
- [ ] **4.2** 扩展辅助工具
  - [ ] 编写扩展定义宏
  - [ ] 提供扩展示例代码
  - [ ] 编写扩展开发文档

### 🟢 Phase 5: 友好 API（优先级：中）
- [ ] **5.1** 构建器 (`builder/`)
  - [ ] 数据包构建器（PacketBuilder）
  - [ ] 数据单元构建器
  - [ ] 流式 API 封装
- [ ] **5.2** 序列化支持 (`serde/`)
  - [ ] JSON 序列化/反序列化
  - [ ] 自定义序列化格式支持

### 🔵 Phase 6: 网络传输（优先级：低）
- [ ] **6.1** 传输层实现 (`transport/`)
  - [ ] TCP 客户端/服务器
  - [ ] UDP 支持
  - [ ] 连接管理和重连机制
- [ ] **6.2** 高级功能
  - [ ] 心跳机制（20-30秒间隔）
  - [ ] 重发机制（超时3次）
  - [ ] 连接状态监控

### 🟣 Phase 7: 测试和文档（优先级：中）
- [ ] **7.1** 单元测试
  - [ ] 协议解析测试
  - [ ] 编码功能测试
  - [ ] 扩展机制测试
- [ ] **7.2** 集成测试
  - [ ] 真实数据案例测试
  - [ ] 性能基准测试
  - [ ] 错误处理测试
- [ ] **7.3** 文档完善
  - [ ] API 文档（rustdoc）
  - [ ] 使用指南
  - [ ] 最佳实践文档

### 🟢 Phase 8: 发布准备（优先级：低）
- [ ] **8.1** 代码质量
  - [ ] 代码审查和重构
  - [ ] 性能优化
  - [ ] 内存泄漏检查
- [ ] **8.2** 发布准备
  - [ ] 版本号规划
  - [ ] CHANGELOG 编写
  - [ ] crates.io 发布

## 🛠️ 技术栈

### 核心依赖
```toml
[dependencies]
# 核心
bytes = "1"                      # 零拷贝字节处理
thiserror = "1"                  # 错误处理
encoding_rs = "0.8"              # GB18030 编码支持

# 可选功能
serde = { version = "1", features = ["derive"], optional = true }
serde_json = { version = "1", optional = true }
tokio = { version = "1", features = ["net", "io-util"], optional = true }
tokio-util = { version = "0.7", features = ["codec"], optional = true }
log = { version = "0.4", optional = true }

[features]
default = []
serde = ["dep:serde", "dep:serde_json"]
async = ["dep:tokio", "dep:tokio-util"]
logging = ["dep:log"]
full = ["serde", "async", "logging"]
```

### 开发工具
- **测试**: 标准库 + criterion（基准测试）
- **文档**: rustdoc + mdbook（用户指南）
- **CI/CD**: GitHub Actions
- **代码质量**: clippy + rustfmt

## 📊 性能目标

| 指标 | 目标值 | 备注 |
|------|--------|------|
| 解析速度 | >1MB/s | 单线程解析性能 |
| 内存使用 | <1KB/packet | 小型数据包内存开销 |
| 延迟 | <1ms | 单个数据包处理延迟 |
| 吞吐量 | >10K packets/s | 高并发场景 |

## 🔄 版本规划

- **v0.1.0**: 基础解析和编码功能
- **v0.2.0**: 扩展机制和自定义类型支持
- **v0.3.0**: 异步网络支持
- **v0.4.0**: 性能优化和稳定性改进
- **v1.0.0**: 功能完整，生产就绪

## 📝 开发规范

### 代码风格
- 使用 `rustfmt` 进行代码格式化
- 遵循 Rust API 设计指南
- 所有公共 API 必须有文档注释
- 使用 `clippy` 进行代码检查

### 提交规范
- 使用常规提交格式（Conventional Commits）
- 每个 PR 必须通过所有测试
- 新功能必须包含测试用例
- 重大更改需要更新 CHANGELOG

### 文档要求
- 所有公共 API 都需要详细的 rustdoc 注释
- 提供使用示例和最佳实践
- 维护更新日志和迁移指南

---

**更新时间**: 2025年11月3日  
**版本**: v1.0  
**状态**: 设计阶段 → 开发阶段准备中