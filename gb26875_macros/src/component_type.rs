//! ComponentType 扩展宏实现

use crate::utils::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Ident, Result};

/// 展开 ComponentType 派生宏
pub fn expand_component_type(input: &DeriveInput) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let attrs = Gb26875Attributes::parse(&input.attrs)?;

    // 验证必需属性
    let code = attrs
        .code
        .ok_or_else(|| Error::new_spanned(input, "ComponentType 扩展必须指定 code 属性"))?;

    // 验证代码范围（128-255）
    validate_code_range(code, 128, 255, "ComponentType")?;

    let description = attrs
        .description
        .unwrap_or_else(|| format!("自定义部件类型 {}", struct_name));

    // 生成实现代码
    let registration_code = generate_registration_code(
        struct_name,
        &Ident::new("ComponentTypeExtension", struct_name.span()),
        "component_type",
    );

    let expanded = quote! {
        #registration_code        impl gb26875::extension::ComponentTypeExtension for #struct_name {
            fn component_type_code(&self) -> u8 {
                #code
            }

            fn description(&self) -> &'static str {
                #description
            }

            fn encode_component_status(&self, _status: &gb26875::info_object::ComponentStatus)
                -> gb26875::extension::ExtensionResult<bytes::Bytes> {
                // 暂时不实现编解码功能
                Err(gb26875::extension::ExtensionError::ValidationError {
                    reason: "Component status encoding not implemented yet".to_string(),
                })
            }

            fn decode_component_status(&self, _data: &[u8])
                -> gb26875::extension::ExtensionResult<gb26875::info_object::ComponentStatus> {
                // 暂时不实现编解码功能
                Err(gb26875::extension::ExtensionError::ValidationError {
                    reason: "Component status decoding not implemented yet".to_string(),
                })
            }

            fn clone_boxed(&self) -> Box<dyn gb26875::extension::ComponentTypeExtension> {
                Box::new(*self)
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
                    .field("description", &#description)
                    .finish()
            }
        }
    };

    Ok(expanded)
}
