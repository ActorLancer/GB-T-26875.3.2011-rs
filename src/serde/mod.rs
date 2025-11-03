//! GB26875 序列化支持模块
//!
//! 提供 Serde 序列化和反序列化功能

#[cfg(feature = "serde")]
pub mod json;

#[cfg(feature = "serde")]
pub use json::*;

// 当未启用 serde 功能时的占位符
#[cfg(not(feature = "serde"))]
pub struct SerdeNotEnabled;

#[cfg(not(feature = "serde"))]
impl SerdeNotEnabled {
    pub fn new() -> Self {
        SerdeNotEnabled
    }
}
