//! GB26875 错误类型定义

use thiserror::Error;

/// 解析错误
#[derive(Error, Debug)]
pub enum ParseError {
    /// 地址错误
    #[error("Address error: {0} - {1}")]
    ValidAddress(u64, String),

    /// 数据包太短  
    #[error("Packet too short: got {got} bytes, need at least {need}")]
    TooShort { got: usize, need: usize },

    /// 数据不足
    #[error("Insufficient data: expected {expected} bytes, got {actual}")]
    InsufficientData { expected: usize, actual: usize },

    /// 无效的启动符
    #[error("Invalid start marker: expected [64, 64], got [{0}, {1}]")]
    InvalidStartMarker(u8, u8),    /// 无效的帧开始
    #[error("Invalid frame start: expected {expected:?}, got {found:?}")]
    InvalidFrameStart { expected: Vec<u8>, found: Vec<u8> },

    /// 无效的结束符
    #[error("Invalid end marker: expected [35, 35], got [{0}, {1}]")]
    InvalidEndMarker(u8, u8),    /// 无效的帧结束
    #[error("Invalid frame end: expected {expected:?}, got {found:?}")]
    InvalidFrameEnd { expected: Vec<u8>, found: Vec<u8> },

    /// 校验和错误
    #[error("Checksum error")]
    Checksum,    /// 校验和不匹配
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: u8, actual: u8 },

    /// 数据单元长度超限
    #[error("Data unit too large: {size} bytes (max: {max_size})")]
    DataUnitTooLarge { size: usize, max_size: usize },

    /// 数据长度不匹配
    #[error("Data length mismatch: expected {expected}, got {actual}")]
    DataLengthMismatch { expected: usize, actual: usize },    /// 无效的数据长度
    #[error("Invalid data length: {actual} (expected: {expected})")]
    InvalidDataLength { actual: usize, expected: usize },

    /// 无效的值
    #[error("Invalid value: {value} for field '{field}'")]
    InvalidValue { field: String, value: String },

    /// 无效的时间戳
    #[error("Invalid timestamp: {field} = {value}")]
    InvalidTimestamp { field: String, value: u8 },

    /// 字符串编码错误
    #[error("String encoding error: {0}")]
    StringEncoding(String),

    /// 反序列化错误
    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    /// IO错误（用于async支持）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 编码错误
#[derive(Error, Debug)]
pub enum EncodeError {
    /// 数据单元太大
    #[error("Data unit too large: {size} bytes (max: {max_size})")]
    DataUnitTooLarge { size: usize, max_size: usize },

    /// 数据太大
    #[error("Data too large: {size} bytes (max: {max_size})")]
    DataTooLarge { size: usize, max_size: usize },

    /// 无效的值
    #[error("Invalid value: {value} for field '{field}' - {reason}")]
    InvalidValue { field: String, value: String, reason: String },

    /// 字符串编码错误
    #[error("String encoding error: {0}")]
    StringEncoding(String),

    /// 字符串太长
    #[error("String too long: {len} bytes (max: {max})")]
    StringTooLong { len: usize, max: usize },

    /// 序列化错误
    #[error("Serialization error: {message}")]
    Serialization { message: String },

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
    InvalidTypeFlag(u8),

    /// 扩展已注册
    #[error("Extension already registered for type {type_id}")]
    AlreadyRegistered { type_id: u8 },

    /// 扩展未找到
    #[error("Extension not found for type {type_id}")]
    NotFound { type_id: u8 },

    /// 解析失败
    #[error("Parse failed for extension type {type_id}: {error}")]
    ParseFailed { type_id: u8, error: String },

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
