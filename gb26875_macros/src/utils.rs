//! 过程宏通用工具函数

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Error, Lit, Result};

/// GB26875 扩展属性解析器
pub struct Gb26875Attributes {
    pub code: Option<u8>,
    pub type_flag: Option<u8>,
    pub range: Option<String>,
    pub unit: Option<String>,
    pub description: Option<String>,
}

impl Gb26875Attributes {
    /// 从属性列表中解析GB26875属性
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self {
            code: None,
            type_flag: None,
            range: None,
            unit: None,
            description: None,
        };

        for attr in attrs {
            if attr.path().is_ident("gb26875") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("code") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(int_lit) = lit {
                            result.code = Some(int_lit.base10_parse()?);
                        } else {
                            return Err(Error::new_spanned(lit, "code 必须是整数"));
                        }
                    } else if meta.path.is_ident("type_flag") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Int(int_lit) = lit {
                            result.type_flag = Some(int_lit.base10_parse()?);
                        } else {
                            return Err(Error::new_spanned(lit, "type_flag 必须是整数"));
                        }
                    } else if meta.path.is_ident("range") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(str_lit) = lit {
                            result.range = Some(str_lit.value());
                        } else {
                            return Err(Error::new_spanned(lit, "range 必须是字符串"));
                        }
                    } else if meta.path.is_ident("unit") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(str_lit) = lit {
                            result.unit = Some(str_lit.value());
                        } else {
                            return Err(Error::new_spanned(lit, "unit 必须是字符串"));
                        }
                    } else if meta.path.is_ident("description") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(str_lit) = lit {
                            result.description = Some(str_lit.value());
                        } else {
                            return Err(Error::new_spanned(lit, "description 必须是字符串"));
                        }
                    } else {
                        return Err(Error::new_spanned(meta.path, "不支持的属性"));
                    }
                    Ok(())
                })?;
            }
        }

        Ok(result)
    }
}

/// 验证代码是否在指定范围内
pub fn validate_code_range(code: u8, min: u8, max: u8, type_name: &str) -> Result<()> {
    if code < min || code > max {
        return Err(Error::new(
            Span::call_site(),
            format!(
                "{} 代码必须在 {}-{} 范围内，实际值: {}",
                type_name, min, max, code
            ),
        ));
    }
    Ok(())
}

/// 生成扩展ID（基于模块路径和结构体名称）
pub fn generate_extension_id(struct_name: &Ident) -> TokenStream {
    quote! {
        gb26875::extension::ExtensionId {
            module_path: module_path!(),
            type_name: stringify!(#struct_name),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

/// 生成基础的扩展注册代码
pub fn generate_registration_code(
    struct_name: &Ident,
    trait_name: &Ident,
    extension_type: &str,
) -> TokenStream {
    let extension_id = generate_extension_id(struct_name);
    let register_fn_name = Ident::new(
        &format!("__register_{}_extension", struct_name),
        struct_name.span(),
    );

    quote! {
        impl gb26875::extension::ExtensionTrait for #struct_name {
            fn extension_id(&self) -> gb26875::extension::ExtensionId {
                #extension_id
            }

            fn extension_type(&self) -> &'static str {
                #extension_type
            }
        }

        // 自动注册到全局注册表（惰性注册）
        #[doc(hidden)]
        pub fn #register_fn_name() -> Result<(), gb26875::extension::ExtensionError> {
            use once_cell::sync::Lazy;
            use std::sync::Arc;

            static ONCE: Lazy<Result<(), gb26875::extension::ExtensionError>> = Lazy::new(|| {
                // 注册逻辑暂时留空，等注册表实现完成后再填充
                Ok(())
            });

            ONCE.clone()
        }
    }
}
