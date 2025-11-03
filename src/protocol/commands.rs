//! GB26875 协议命令定义

use crate::error::{ParseError, ParseResult};

/// 控制单元命令字节定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Command {
    /// 预留
    Reserved = 0,
    /// 控制命令（时间同步）
    Control = 1,
    /// 发送数据（发送火灾报警和建筑消防设施运行状态等信息）
    SendData = 2,
    /// 确认（对控制命令和发送信息的回答）
    Acknowledge = 3,
    /// 请求（查询火灾报警和建筑消防设施运行状态等信息）
    Request = 4,
    /// 应答（返回查询的信息）
    Response = 5,
    /// 否认（对控制命令和发送信息的否定回答）
    Reject = 6,
    /// 预留 (7-127)
    StandardReserved(u8),
    /// 用户自定义 (128-255)
    UserDefined(u8),
}

impl Command {
    /// 从字节值创建命令类型
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Reserved,
            1 => Self::Control,
            2 => Self::SendData,
            3 => Self::Acknowledge,
            4 => Self::Request,
            5 => Self::Response,
            6 => Self::Reject,
            7..=127 => Self::StandardReserved(value),
            128..=255 => Self::UserDefined(value),
        }
    }

    /// 转换为字节值
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Reserved => 0,
            Self::Control => 1,
            Self::SendData => 2,
            Self::Acknowledge => 3,
            Self::Request => 4,
            Self::Response => 5,
            Self::Reject => 6,
            Self::StandardReserved(v) => v,
            Self::UserDefined(v) => v,
        }
    }

    /// 是否为用户自定义命令
    pub fn is_user_defined(&self) -> bool {
        matches!(self, Self::UserDefined(_))
    }

    /// 获取命令描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Reserved => "预留",
            Self::Control => "控制命令",
            Self::SendData => "发送数据",
            Self::Acknowledge => "确认",
            Self::Request => "请求",
            Self::Response => "应答",
            Self::Reject => "否认",
            Self::StandardReserved(_) => "标准预留",
            Self::UserDefined(_) => "用户自定义",
        }
    }

    /// 是否需要应用数据单元
    /// 
    /// 确认和否认命令通常不需要应用数据单元
    pub fn requires_data_unit(&self) -> bool {
        !matches!(self, Self::Acknowledge | Self::Reject)
    }

    /// 是否为响应命令（确认、应答、否认）
    pub fn is_response(&self) -> bool {
        matches!(self, Self::Acknowledge | Self::Response | Self::Reject)
    }

    /// 是否为请求命令（控制、发送数据、请求）
    pub fn is_request(&self) -> bool {
        matches!(self, Self::Control | Self::SendData | Self::Request)
    }

    /// 获取对应的响应命令
    /// 
    /// - Control -> Acknowledge/Reject
    /// - SendData -> Acknowledge/Reject  
    /// - Request -> Response/Reject
    pub fn get_ack_command(&self) -> Self {
        match self {
            Self::Control | Self::SendData => Self::Acknowledge,
            Self::Request => Self::Response,
            _ => Self::Acknowledge, // 默认返回确认
        }
    }

    /// 获取对应的拒绝命令
    pub fn get_reject_command(&self) -> Self {
        Self::Reject
    }

    /// 心跳命令别名
    pub const Heartbeat: Self = Self::SendData;
    /// 状态查询别名  
    pub const StatusQuery: Self = Self::Request;
    /// 状态上传别名
    pub const StatusUpload: Self = Self::SendData;
    /// 确认别名
    pub const Acknowledgment: Self = Self::Acknowledge;
}

/// 协议版本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtocolVersion {
    /// 主版本号（固定为1）
    pub major: u8,
    /// 用户版本号（用户自定义）
    pub minor: u8,
}

impl ProtocolVersion {
    /// 创建新的协议版本
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// 创建标准版本（1.1）
    pub fn standard() -> Self {
        Self::new(1, 1)
    }

    /// 创建v1.0版本
    pub fn v1_0() -> Self {
        Self::new(1, 0)
    }

    /// 从字节数组创建
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::new(bytes[0], bytes[1])
    }

    /// 转换为字节数组
    pub fn to_bytes(self) -> [u8; 2] {
        [self.major, self.minor]
    }

    /// 验证主版本号是否有效
    pub fn is_valid_major(&self) -> bool {
        self.major == 1
    }

    /// 是否为标准版本
    pub fn is_standard(&self) -> bool {
        self.major == 1 && self.minor == 1
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::standard()
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
