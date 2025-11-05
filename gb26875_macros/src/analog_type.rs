//! AnalogType 扩展宏实现

use crate::utils::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Ident, Result};

/// 展开 AnalogType 派生宏
pub fn expand_analog_type(input: &DeriveInput) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let attrs = Gb26875Attributes::parse(&input.attrs)?;

    // 验证必需属性
    let code = attrs
        .code
        .ok_or_else(|| Error::new_spanned(input, "AnalogType 扩展必须指定 code 属性"))?;

    // 验证代码范围（128-255）
    validate_code_range(code, 128, 255, "AnalogType")?;

    let range_str = attrs
        .range
        .ok_or_else(|| Error::new_spanned(input, "AnalogType 扩展必须指定 range 属性"))?;

    // 解析范围字符串，支持两种格式：
    // 1. 字符串格式: "-40..85"
    // 2. 元组格式: "(-40.0, 85.0)" （虽然这里是字符串，但内容是元组格式）
    let (min_value, max_value) = parse_range(&range_str)?;

    let unit = attrs.unit.unwrap_or_else(|| "".to_string());
    let description = attrs
        .description
        .unwrap_or_else(|| format!("自定义模拟量类型 {}", struct_name));

    // 生成实现代码
    let registration_code = generate_registration_code(
        struct_name,
        &Ident::new("AnalogTypeExtension", struct_name.span()),
        "analog_type",
    );

    let expanded = quote! {
        #registration_code        impl gb26875::extension::AnalogTypeExtension for #struct_name {
            fn analog_type_code(&self) -> u8 {
                #code
            }

            fn min_value(&self) -> f64 {
                #min_value
            }

            fn max_value(&self) -> f64 {
                #max_value
            }

            fn unit(&self) -> &'static str {
                #unit
            }

            fn description(&self) -> &'static str {
                #description
            }

            fn validate_value(&self, value: f64) -> bool {
                value >= self.min_value() && value <= self.max_value()
            }

            fn encode_analog_value(&self, _value: &gb26875::info_object::AnalogValue)
                -> gb26875::extension::ExtensionResult<bytes::Bytes> {
                // 暂时不实现编解码功能
                Err(gb26875::extension::ExtensionError::ValidationError {
                    reason: "Analog value encoding not implemented yet".to_string(),
                })
            }

            fn decode_analog_value(&self, _data: &[u8])
                -> gb26875::extension::ExtensionResult<gb26875::info_object::AnalogValue> {
                // 暂时不实现编解码功能
                Err(gb26875::extension::ExtensionError::ValidationError {
                    reason: "Analog value decoding not implemented yet".to_string(),
                })
            }
        }

        // 实现 Clone trait（如果还没有）
        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                *self
            }
        }

        // 实现 Copy trait（如果还没有）
        impl Copy for #struct_name {}

        // 实现 Debug trait（如果还没有）
        impl std::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#struct_name))
                    .field("code", &#code)
                    .field("range", &(#min_value, #max_value))
                    .field("unit", &#unit)
                    .field("description", &#description)
                    .finish()
            }
        }
    };

    Ok(expanded)
}

/// 解析范围字符串，支持两种格式
/// 1. 字符串格式: "-40..85"
/// 2. 元组格式: "(-40.0, 85.0)"
fn parse_range(range_str: &str) -> Result<(f64, f64)> {
    let range_str = range_str.trim();

    // 尝试解析元组格式: "(-40.0, 85.0)"
    if range_str.starts_with('(') && range_str.ends_with(')') {
        let inner = &range_str[1..range_str.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 2 {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "元组格式范围必须包含恰好两个值，例如: (-40.0, 85.0)",
            ));
        }

        let min: f64 = parts[0].trim().parse().map_err(|_| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("无法解析最小值: {}", parts[0].trim()),
            )
        })?;

        let max: f64 = parts[1].trim().parse().map_err(|_| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("无法解析最大值: {}", parts[1].trim()),
            )
        })?;

        return Ok((min, max));
    }

    // 尝试解析字符串格式: "-40..85"
    if let Some(pos) = range_str.find("..") {
        let min_str = &range_str[..pos];
        let max_str = &range_str[pos + 2..];

        let min: f64 = min_str.trim().parse().map_err(|_| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("无法解析最小值: {}", min_str.trim()),
            )
        })?;

        let max: f64 = max_str.trim().parse().map_err(|_| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("无法解析最大值: {}", max_str.trim()),
            )
        })?;

        return Ok((min, max));
    }

    Err(Error::new(
        proc_macro2::Span::call_site(),
        format!(
            "不支持的范围格式: {}，支持的格式: \"-40..85\" 或 \"(-40.0, 85.0)\"",
            range_str
        ),
    ))
}
