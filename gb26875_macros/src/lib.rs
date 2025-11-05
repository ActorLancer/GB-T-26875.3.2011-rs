//! GB26875 过程宏
//!
//! 这个crate提供了用于GB26875协议的过程宏，支持自定义扩展类型的定义。
//!
//! # 主要功能
//!
//! - `#[derive(Command)]` - 定义自定义命令类型（128-255范围）
//! - `#[derive(DataUnit)]` - 定义自定义数据单元类型（128-254范围）  
//! - `#[derive(SystemType)]` - 定义自定义系统类型（128-255范围）
//! - `#[derive(ComponentType)]` - 定义自定义部件类型（128-255范围）
//! - `#[derive(AnalogType)]` - 定义自定义模拟量类型（128-255范围）
//!
//! # 使用示例
//!
//! ```rust
//! use gb26875_macros::*;
//!
//! #[derive(Command)]
//! #[gb26875(code = 128, description = "自定义登录命令")]
//! pub struct CustomLogin;
//!
//! #[derive(DataUnit)]
//! #[gb26875(type_flag = 128, description = "自定义报警数据")]
//! pub struct CustomAlarm {
//!     pub alarm_type: u8,
//!     pub alarm_level: u8,
//! }
//!
//! #[derive(AnalogType)]
//! #[gb26875(code = 128, range = "-40..85", unit = "°C")]
//! pub struct Temperature;
//! ```

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod analog_type;
mod command;
mod component_type;
mod data_unit;
mod system_type;
mod utils;

/// 为结构体派生 Command 扩展trait
///
/// 用于定义自定义命令类型（范围128-255）
///
/// # 属性
///
/// - `code`: 命令代码（必须在128-255范围内）
/// - `description`: 命令描述（可选）
///
/// # 示例
///
/// ```rust
/// #[derive(Command)]
/// #[gb26875(code = 128, description = "自定义登录命令")]
/// pub struct CustomLogin;
/// ```
#[proc_macro_derive(Command, attributes(gb26875))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match command::expand_command(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// 为结构体派生 DataUnit 扩展trait
///
/// 用于定义自定义数据单元类型（范围128-254）
///
/// # 属性
///
/// - `type_flag`: 数据单元类型标志（必须在128-254范围内）
/// - `description`: 数据单元描述（可选）
///
/// # 示例
///
/// ```rust
/// #[derive(DataUnit)]
/// #[gb26875(type_flag = 128, description = "自定义报警数据")]
/// pub struct CustomAlarm {
///     pub alarm_type: u8,
///     pub alarm_level: u8,
/// }
/// ```
#[proc_macro_derive(DataUnit, attributes(gb26875))]
pub fn derive_data_unit(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match data_unit::expand_data_unit(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// 为结构体派生 SystemType 扩展trait
///
/// 用于定义自定义系统类型（范围128-255）
///
/// # 属性
///
/// - `code`: 系统类型代码（必须在128-255范围内）
/// - `description`: 系统类型描述（可选）
///
/// # 示例
///
/// ```rust
/// #[derive(SystemType)]
/// #[gb26875(code = 128, description = "自定义安防系统")]
/// pub struct CustomSecuritySystem;
/// ```
#[proc_macro_derive(SystemType, attributes(gb26875))]
pub fn derive_system_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match system_type::expand_system_type(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// 为结构体派生 ComponentType 扩展trait
///
/// 用于定义自定义部件类型（范围128-255）
///
/// # 属性
///
/// - `code`: 部件类型代码（必须在128-255范围内）
/// - `description`: 部件类型描述（可选）
///
/// # 示例
///
/// ```rust
/// #[derive(ComponentType)]
/// #[gb26875(code = 128, description = "自定义传感器")]
/// pub struct CustomSensor;
/// ```
#[proc_macro_derive(ComponentType, attributes(gb26875))]
pub fn derive_component_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match component_type::expand_component_type(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// 为结构体派生 AnalogType 扩展trait
///
/// 用于定义自定义模拟量类型（范围128-255）
///
/// # 属性
///
/// - `code`: 模拟量类型代码（必须在128-255范围内）
/// - `range`: 值范围，支持字符串格式 ("-40..85") 或元组格式 (-40.0, 85.0)
/// - `unit`: 单位（可选）
/// - `description`: 模拟量类型描述（可选）
///
/// # 示例
///
/// ```rust
/// #[derive(AnalogType)]
/// #[gb26875(code = 128, range = "-40..85", unit = "°C")]
/// pub struct Temperature;
///
/// #[derive(AnalogType)]
/// #[gb26875(code = 129, range = (0.0, 100.0), unit = "%")]
/// pub struct Humidity;
/// ```
#[proc_macro_derive(AnalogType, attributes(gb26875))]
pub fn derive_analog_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match analog_type::expand_analog_type(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
