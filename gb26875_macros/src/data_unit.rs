//! DataUnit 扩展宏实现

use crate::utils::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Ident, Result};

/// 展开 DataUnit 派生宏
pub fn expand_data_unit(input: &DeriveInput) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let attrs = Gb26875Attributes::parse(&input.attrs)?;

    // 验证必需属性
    let type_flag = attrs
        .type_flag
        .ok_or_else(|| Error::new_spanned(input, "DataUnit 扩展必须指定 type_flag 属性"))?;

    // 验证类型标志范围（128-254）
    validate_code_range(type_flag, 128, 254, "DataUnit")?;

    let description = attrs
        .description
        .unwrap_or_else(|| format!("自定义数据单元类型 {}", struct_name));

    // 生成实现代码
    let registration_code = generate_registration_code(
        struct_name,
        &Ident::new("DataUnitExtension", struct_name.span()),
        "data_unit",
    );

    let expanded = quote! {
        #registration_code
        impl gb26875::extension::DataUnitExtension for #struct_name {
            fn type_flag(&self) -> u8 {
                #type_flag
            }

            fn description(&self) -> &'static str {
                #description
            }

            fn encode(&self) -> gb26875::extension::ExtensionResult<bytes::Bytes> {
                // 默认实现：序列化结构体字段
                // 这里可以根据字段类型生成具体的编码逻辑
                // 暂时返回空数据
                Ok(bytes::Bytes::new())
            }

            fn get_info_objects(&self) -> Vec<bytes::Bytes> {
                // 默认实现：空的信息对象列表
                // 子类可以重写此方法来提供具体的信息对象
                vec![]
            }

            fn clone_boxed(&self) -> Box<dyn gb26875::extension::DataUnitExtension> {
                Box::new(self.clone())
            }
        }

        impl gb26875::extension::DataUnitExtensionParser for #struct_name {
            fn decode(_data: &[u8]) -> gb26875::extension::ExtensionResult<Box<dyn gb26875::extension::DataUnitExtension>> {
                // 默认实现：返回验证错误，提示用户需要实现具体的解码逻辑
                Err(gb26875::extension::ExtensionError::ValidationError {
                    reason: format!("{}类型的解码功能尚未实现", stringify!(#struct_name))
                })
            }
        }
    };

    Ok(expanded)
}
