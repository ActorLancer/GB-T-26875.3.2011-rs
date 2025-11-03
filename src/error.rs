//! GB26875 错误类型定义

use thiserror::Error;

/// 解析错误
#[derive(Error, Debug)]
pub enum ParseError {
    /// 地址错误
    #[error("Address error: {0} - {1}")]
    ValidAddress(u64, String),    /// 数据包太短  
    #[error("Packet too short: got {got} bytes, need at least {need}")]
    TooShort { 
        /// 实际获得的字节数
        got: usize, 
        /// 需要的最少字节数
        need: usize 
    },

    /// 数据不足
    #[error("Insufficient data: expected {expected} bytes, got {actual}")]
    InsufficientData { 
        /// 期望的字节数
        expected: usize, 
        /// 实际获得的字节数
        actual: usize 
    },

    /// 无效的启动符
    #[error("Invalid start marker: expected [64, 64], got [{0}, {1}]")]
    InvalidStartMarker(u8, u8),    /// 无效的帧开始
    #[error("Invalid frame start: expected {expected:?}, got {found:?}")]
    InvalidFrameStart { 
        /// 期望的字节序列
        expected: Vec<u8>, 
        /// 实际发现的字节序列
        found: Vec<u8> 
    },

    /// 无效的结束符
    #[error("Invalid end marker: expected [35, 35], got [{0}, {1}]")]
    InvalidEndMarker(u8, u8),    /// 无效的帧结束
    #[error("Invalid frame end: expected {expected:?}, got {found:?}")]
    InvalidFrameEnd { 
        /// 期望的字节序列
        expected: Vec<u8>, 
        /// 实际发现的字节序列
        found: Vec<u8> 
    },

    /// 校验和错误
    #[error("Checksum error")]
    Checksum,    /// 校验和不匹配
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { 
        /// 期望的校验和
        expected: u8, 
        /// 实际的校验和
        actual: u8 
    },

    /// 数据单元长度超限
    #[error("Data unit too large: {size} bytes (max: {max_size})")]
    DataUnitTooLarge { 
        /// 实际大小
        size: usize, 
        /// 最大允许大小
        max_size: usize 
    },

    /// 数据长度不匹配
    #[error("Data length mismatch: expected {expected}, got {actual}")]
    DataLengthMismatch { 
        /// 期望的长度
        expected: usize, 
        /// 实际的长度
        actual: usize 
    },    /// 无效的数据长度
    #[error("Invalid data length: {actual} (expected: {expected})")]
    InvalidDataLength { 
        /// 实际长度
        actual: usize, 
        /// 期望长度
        expected: usize 
    },

    /// 无效的值
    #[error("Invalid value: {value} for field '{field}'")]
    InvalidValue { 
        /// 字段名称
        field: String, 
        /// 字段值
        value: String 
    },

    /// 无效的时间戳
    #[error("Invalid timestamp: {field} = {value}")]
    InvalidTimestamp { 
        /// 字段名称
        field: String, 
        /// 字段值
        value: u8 
    },

    /// 字符串编码错误
    #[error("String encoding error: {0}")]
    StringEncoding(String),    /// 反序列化错误
    #[error("Deserialization error: {message}")]
    DeserializationError { 
        /// 错误消息
        message: String 
    },

    /// IO错误（用于async支持）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 编码错误
#[derive(Error, Debug)]
pub enum EncodeError {
    /// 数据单元太大
    #[error("Data unit too large: {size} bytes (max: {max_size})")]
    DataUnitTooLarge { 
        /// 实际大小
        size: usize, 
        /// 最大允许大小
        max_size: usize 
    },

    /// 数据太大
    #[error("Data too large: {size} bytes (max: {max_size})")]
    DataTooLarge { 
        /// 实际大小
        size: usize, 
        /// 最大允许大小
        max_size: usize 
    },

    /// 无效的值
    #[error("Invalid value: {value} for field '{field}' - {reason}")]
    InvalidValue { 
        /// 字段名称
        field: String, 
        /// 字段值
        value: String, 
        /// 错误原因
        reason: String 
    },

    /// 字符串编码错误
    #[error("String encoding error: {0}")]
    StringEncoding(String),

    /// 字符串太长
    #[error("String too long: {len} bytes (max: {max})")]
    StringTooLong { 
        /// 实际长度
        len: usize, 
        /// 最大允许长度
        max: usize 
    },

    /// 序列化错误
    #[error("Serialization error: {message}")]
    Serialization { 
        /// 错误消息
        message: String 
    },

    /// 类型转换错误
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// IO错误（用于async支持）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 扩展错误
#[derive(Error, Debug)]
pub enum ExtensionError {
    /// 类型标志无效
    #[error("Invalid type flag: {0} (must be 128-255)")]
    InvalidTypeFlag(u8),    /// 扩展已注册
    #[error("Extension already registered for type {type_id}")]
    AlreadyRegistered { 
        /// 类型ID
        type_id: u8 
    },

    /// 扩展未找到
    #[error("Extension not found for type {type_id}")]
    NotFound { 
        /// 类型ID
        type_id: u8 
    },

    /// 解析失败
    #[error("Parse failed for extension type {type_id}: {error}")]
    ParseFailed { 
        /// 类型ID
        type_id: u8, 
        /// 错误信息
        error: String 
    },

    /// 注册表锁错误
    #[error("Registry lock error")]
    RegistryLockError,
}

/// 解析结果类型
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// 编码结果类型
pub type EncodeResult<T> = std::result::Result<T, EncodeError>;

/// 扩展结果类型
pub type ExtensionResult<T> = std::result::Result<T, ExtensionError>;
