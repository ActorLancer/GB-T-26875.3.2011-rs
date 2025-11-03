//! GB26875 协议常量定义

/// 数据包启动符 (@@)
pub const FRAME_START: [u8; 2] = [0x40, 0x40]; // @@ = [64, 64]

/// 数据包结束符 (##)
pub const FRAME_END: [u8; 2] = [0x23, 0x23]; // ## = [35, 35]

/// 控制单元长度（不包括启动符和结束符）
pub const CONTROL_UNIT_LENGTH: usize = 25;

/// 数据包最小长度（启动符 + 控制单元 + 校验和 + 结束符）
pub const MIN_PACKET_SIZE: usize = 2 + CONTROL_UNIT_LENGTH + 1 + 2; // 30 字节

/// 应用数据单元最大长度
pub const MAX_DATA_UNIT_SIZE: usize = 1024;

/// 数据包最大长度
pub const MAX_PACKET_SIZE: usize = MIN_PACKET_SIZE + MAX_DATA_UNIT_SIZE; // 1054 字节

/// 时间标签长度
pub const TIMESTAMP_LENGTH: usize = 6;

/// 协议主版本号（固定值）
pub const PROTOCOL_MAJOR_VERSION: u8 = 1;

/// 默认超时时间（秒）
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

/// 默认重试次数
pub const DEFAULT_RETRY_COUNT: u8 = 3;

/// 心跳间隔（秒）- 正常状态
pub const HEARTBEAT_INTERVAL_NORMAL: u64 = 25; // 20-30秒

/// 心跳间隔（秒）- 异常状态
pub const HEARTBEAT_INTERVAL_ERROR: u64 = 5;

/// 用户自定义类型范围开始
pub const USER_DEFINED_START: u8 = 128;

/// 用户自定义类型范围结束
pub const USER_DEFINED_END: u8 = 255;

/// 用户自定义数据单元类型范围结束（排除255）
pub const USER_DEFINED_DATA_UNIT_END: u8 = 254;

/// 部件说明字符串最大长度
pub const COMPONENT_DESCRIPTION_MAX_LEN: usize = 31;

/// 系统配置说明最大长度
pub const SYSTEM_CONFIG_MAX_LEN: usize = 255;

/// 用户信息传输装置配置说明最大长度
pub const DEVICE_CONFIG_MAX_LEN: usize = 255;
