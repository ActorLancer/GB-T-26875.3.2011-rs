//! GB26875 扩展 trait 定义
//!
//! 定义了扩展数据单元需要实现的 trait 和相关类型

use bytes::Bytes;
use std::any::Any;
use std::fmt;

// 重新导出供外部使用
pub use crate::error::{ExtensionError, ExtensionResult};

/// 扩展数据单元 trait
///
/// 所有用户自定义数据单元都必须实现这个 trait
pub trait ExtensionDataUnit: fmt::Debug + Send + Sync {
    /// 获取数据单元类型ID（128-254）
    fn type_id(&self) -> u8;

    /// 编码为字节序列
    ///
    /// # Returns
    /// * `ExtensionResult<Bytes>` - 成功返回编码后的字节序列
    fn encode(&self) -> ExtensionResult<Bytes>;

    /// 验证数据单元的有效性
    ///
    /// # Returns
    /// * `ExtensionResult<()>` - 验证成功返回 ()
    fn validate(&self) -> ExtensionResult<()> {
        Ok(())
    }

    /// 获取数据单元的字节长度
    ///
    /// # Returns
    /// * `usize` - 字节长度
    fn byte_length(&self) -> usize {
        self.encode().map(|b| b.len()).unwrap_or(0)
    }

    /// 转换为 Any trait，用于运行时类型转换
    ///
    /// # Returns
    /// * `&dyn Any` - Any trait 对象引用
    fn as_any(&self) -> &dyn Any;

    /// 获取数据单元的描述信息（可选）
    ///
    /// # Returns
    /// * `Option<String>` - 描述信息
    fn description(&self) -> Option<String> {
        None
    }

    /// 获取数据单元的版本信息（可选）
    ///
    /// # Returns
    /// * `Option<String>` - 版本信息
    fn version(&self) -> Option<String> {
        None
    }
}

/// 可克隆的扩展数据单元 trait
///
/// 为需要克隆功能的扩展数据单元提供额外支持
pub trait CloneableExtensionDataUnit: ExtensionDataUnit {
    /// 克隆数据单元
    ///
    /// # Returns
    /// * `Box<dyn CloneableExtensionDataUnit>` - 克隆的数据单元
    fn clone_boxed(&self) -> Box<dyn CloneableExtensionDataUnit>;
}

/// 可序列化的扩展数据单元 trait
///
/// 为需要序列化功能的扩展数据单元提供额外支持

pub trait SerializableExtensionDataUnit: ExtensionDataUnit {
    /// 序列化为 JSON
    ///
    /// # Returns
    /// * `ExtensionResult<String>` - 成功返回 JSON 字符串
    fn to_json(&self) -> ExtensionResult<String>;

    /// 从 JSON 反序列化
    ///
    /// # Arguments
    /// * `json` - JSON 字符串
    ///
    /// # Returns
    /// * `ExtensionResult<Self>` - 成功返回反序列化的对象
    fn from_json(json: &str) -> ExtensionResult<Self>
    where
        Self: Sized;
}

/// 扩展数据单元构建器 trait
///
/// 为复杂的扩展数据单元提供构建器模式支持
pub trait ExtensionDataUnitBuilder<T>
where
    T: ExtensionDataUnit,
{
    /// 构建数据单元
    ///
    /// # Returns
    /// * `ExtensionResult<T>` - 成功返回构建的数据单元
    fn build(self) -> ExtensionResult<T>;

    /// 验证构建参数
    ///
    /// # Returns
    /// * `ExtensionResult<()>` - 验证成功返回 ()
    fn validate(&self) -> ExtensionResult<()> {
        Ok(())
    }
}

/// 扩展数据单元解析器 trait
///
/// 定义了扩展数据单元的解析接口
pub trait ExtensionDataUnitParser<T>
where
    T: ExtensionDataUnit,
{
    /// 从字节序列解析数据单元
    ///
    /// # Arguments
    /// * `data` - 字节序列
    ///
    /// # Returns
    /// * `ExtensionResult<T>` - 成功返回解析的数据单元
    fn parse(data: &[u8]) -> ExtensionResult<T>;

    /// 尝试解析数据单元（不会抛出错误）
    ///
    /// # Arguments
    /// * `data` - 字节序列
    ///
    /// # Returns
    /// * `Option<T>` - 成功返回解析的数据单元，失败返回 None
    fn try_parse(data: &[u8]) -> Option<T> {
        Self::parse(data).ok()
    }

    /// 检查数据是否可以被解析
    ///
    /// # Arguments
    /// * `data` - 字节序列
    ///
    /// # Returns
    /// * `bool` - 如果可以解析返回 true
    fn can_parse(data: &[u8]) -> bool {
        Self::try_parse(data).is_some()
    }
}

/// 扩展数据单元工厂 trait
///
/// 提供创建扩展数据单元实例的工厂方法
pub trait ExtensionDataUnitFactory {
    /// 支持的数据单元类型
    type DataUnit: ExtensionDataUnit;

    /// 获取支持的类型ID
    ///
    /// # Returns
    /// * `u8` - 类型ID
    fn type_id(&self) -> u8;

    /// 获取类型名称
    ///
    /// # Returns
    /// * `&str` - 类型名称
    fn type_name(&self) -> &str;

    /// 创建数据单元实例
    ///
    /// # Arguments
    /// * `data` - 原始数据
    ///
    /// # Returns
    /// * `ExtensionResult<Self::DataUnit>` - 成功返回数据单元实例
    fn create(&self, data: &[u8]) -> ExtensionResult<Self::DataUnit>;

    /// 创建默认实例
    ///
    /// # Returns
    /// * `ExtensionResult<Self::DataUnit>` - 成功返回默认数据单元实例
    fn create_default(&self) -> ExtensionResult<Self::DataUnit>;
}

/// 实现了自动 Clone 的扩展数据单元包装器
#[derive(Debug)]
pub struct CloneableExtensionWrapper<T>
where
    T: ExtensionDataUnit + Clone,
{
    inner: T,
}

impl<T> CloneableExtensionWrapper<T>
where
    T: ExtensionDataUnit + Clone,
{
    /// 创建新的包装器
    ///
    /// # Arguments
    /// * `inner` - 内部数据单元
    pub fn new(inner: T) -> Self {
        CloneableExtensionWrapper { inner }
    }

    /// 获取内部数据单元的引用
    ///
    /// # Returns
    /// * `&T` - 内部数据单元引用
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// 获取内部数据单元的可变引用
    ///
    /// # Returns
    /// * `&mut T` - 内部数据单元可变引用
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// 解包获取内部数据单元
    ///
    /// # Returns
    /// * `T` - 内部数据单元
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> ExtensionDataUnit for CloneableExtensionWrapper<T>
where
    T: ExtensionDataUnit + Clone + 'static,
{
    fn type_id(&self) -> u8 {
        self.inner.type_id()
    }

    fn encode(&self) -> ExtensionResult<Bytes> {
        self.inner.encode()
    }

    fn validate(&self) -> ExtensionResult<()> {
        self.inner.validate()
    }

    fn byte_length(&self) -> usize {
        self.inner.byte_length()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn description(&self) -> Option<String> {
        self.inner.description()
    }

    fn version(&self) -> Option<String> {
        self.inner.version()
    }
}

impl<T> CloneableExtensionDataUnit for CloneableExtensionWrapper<T>
where
    T: ExtensionDataUnit + Clone + 'static,
{
    fn clone_boxed(&self) -> Box<dyn CloneableExtensionDataUnit> {
        Box::new(CloneableExtensionWrapper::new(self.inner.clone()))
    }
}

impl<T> Clone for CloneableExtensionWrapper<T>
where
    T: ExtensionDataUnit + Clone,
{
    fn clone(&self) -> Self {
        CloneableExtensionWrapper::new(self.inner.clone())
    }
}

/// 类型安全的扩展数据单元转换辅助函数
///
/// # Arguments
/// * `extension` - 扩展数据单元
///
/// # Returns
/// * `Option<&T>` - 如果类型匹配返回转换后的引用
pub fn downcast_extension<T: ExtensionDataUnit + 'static>(
    extension: &dyn ExtensionDataUnit,
) -> Option<&T> {
    extension.as_any().downcast_ref::<T>()
}

/// 类型安全的可克隆扩展数据单元转换辅助函数
///
/// # Arguments
/// * `extension` - 可克隆扩展数据单元
///
/// # Returns
/// * `Option<&T>` - 如果类型匹配返回转换后的引用
pub fn downcast_cloneable_extension<T: CloneableExtensionDataUnit + 'static>(
    extension: &dyn CloneableExtensionDataUnit,
) -> Option<&T> {
    extension.as_any().downcast_ref::<T>()
}

mod tests {
    use super::*;

    // 测试用的简单扩展数据单元
    #[derive(Debug, Clone, PartialEq)]
    struct TestExtension {
        value: u32,
        name: String,
    }

    impl ExtensionDataUnit for TestExtension {
        fn type_id(&self) -> u8 {
            200
        }

        fn encode(&self) -> ExtensionResult<Bytes> {
            let mut data = Vec::new();
            data.extend_from_slice(&self.value.to_le_bytes());
            data.extend_from_slice(self.name.as_bytes());
            Ok(Bytes::from(data))
        }

        fn validate(&self) -> ExtensionResult<()> {
            if self.name.is_empty() {
                return Err(ExtensionError::ValidationError {
                    reason: "名称不能为空".to_string(),
                });
            }
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn description(&self) -> Option<String> {
            Some(format!("测试扩展数据单元: {}", self.name))
        }

        fn version(&self) -> Option<String> {
            Some("1.0.0".to_string())
        }
    }

    impl ExtensionDataUnitParser<TestExtension> for TestExtension {
        fn parse(data: &[u8]) -> ExtensionResult<TestExtension> {
            if data.len() < 4 {
                return Err(ExtensionError::ParseError(
                    format!("期望数据长度至少为4字节，实际为{}字节", data.len())
                ));
            }

            let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let name = String::from_utf8_lossy(&data[4..]).into_owned();

            let extension = TestExtension { value, name };
            extension.validate()?;
            Ok(extension)
        }
    }

    #[test]
    fn test_extension_data_unit() {
        let extension = TestExtension {
            value: 0x12345678,
            name: "test".to_string(),
        };

        assert_eq!(ExtensionDataUnit::type_id(&extension), 200);
        assert!(extension.validate().is_ok());
        assert!(extension.description().is_some());
        assert!(extension.version().is_some());

        let encoded = extension.encode().unwrap();
        assert!(!encoded.is_empty());

        let parsed = TestExtension::parse(&encoded).unwrap();
        assert_eq!(extension, parsed);
    }

    #[test]
    fn test_cloneable_extension_wrapper() {
        let extension = TestExtension {
            value: 0x87654321,
            name: "cloneable_test".to_string(),
        };

        let wrapper = CloneableExtensionWrapper::new(extension.clone());
        let cloned = wrapper.clone();

        assert_eq!(wrapper.inner(), &extension);
        assert_eq!(cloned.inner(), &extension);
        assert_eq!(
            ExtensionDataUnit::type_id(&wrapper),
            ExtensionDataUnit::type_id(&extension)
        );
    }

    #[test]
    fn test_downcast_extension() {
        let extension = TestExtension {
            value: 0x11223344,
            name: "downcast_test".to_string(),
        };

        let boxed: Box<dyn ExtensionDataUnit> = Box::new(extension.clone());
        let downcasted = downcast_extension::<TestExtension>(&*boxed);

        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap(), &extension);

        // 测试错误的类型转换
        #[derive(Debug)]
        struct OtherExtension;
        impl ExtensionDataUnit for OtherExtension {
            fn type_id(&self) -> u8 {
                201
            }
            fn encode(&self) -> ExtensionResult<Bytes> {
                Ok(Bytes::new())
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let other_downcasted = downcast_extension::<OtherExtension>(&*boxed);
        assert!(other_downcasted.is_none());
    }

    #[test]
    fn test_extension_validation() {
        let valid_extension = TestExtension {
            value: 123,
            name: "valid".to_string(),
        };
        assert!(valid_extension.validate().is_ok());

        let invalid_extension = TestExtension {
            value: 456,
            name: String::new(), // 空名称应该验证失败
        };
        assert!(invalid_extension.validate().is_err());
    }
}

//
// 以下是过程宏系统所需的trait定义
//

/// 扩展标识符
///
/// 用于唯一标识扩展类型，包含模块路径、类型名称和版本信息
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtensionId {
    /// 模块路径
    pub module_path: &'static str,
    /// 类型名称
    pub type_name: &'static str,
    /// 版本信息
    pub version: &'static str,
}

impl ExtensionId {
    /// 创建新的扩展标识符
    pub fn new(module_path: &'static str, type_name: &'static str, version: &'static str) -> Self {
        Self {
            module_path,
            type_name,
            version,
        }
    }

    /// 获取完整的扩展名称（模块路径::类型名称）
    pub fn full_name(&self) -> String {
        format!("{}::{}", self.module_path, self.type_name)
    }
}

impl fmt::Display for ExtensionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}::{}@{}",
            self.module_path, self.type_name, self.version
        )
    }
}

/// 基础扩展trait
///
/// 所有扩展类型都需要实现这个trait
pub trait ExtensionTrait: fmt::Debug + Send + Sync {
    /// 获取扩展标识符
    fn extension_id(&self) -> ExtensionId;

    /// 获取扩展类型（用于调试和日志）
    fn extension_type(&self) -> &'static str;
}

/// 命令扩展trait
///
/// 用于定义自定义命令类型（128-255范围）
pub trait CommandExtension: ExtensionTrait {
    /// 获取命令代码
    fn command_code(&self) -> u8;

    /// 获取命令描述
    fn description(&self) -> &'static str;

    /// 编码命令数据
    fn encode(&self) -> ExtensionResult<Bytes>;

    /// 克隆为Box包装的trait对象
    fn clone_boxed(&self) -> Box<dyn CommandExtension>;
}

/// 命令扩展解析器trait
///
/// 用于解析命令数据（分离以保持dyn兼容性）
pub trait CommandExtensionParser {
    /// 解码命令数据
    fn decode(data: &[u8]) -> ExtensionResult<Box<dyn CommandExtension>>;
}

/// 数据单元扩展trait
///
/// 用于定义自定义数据单元类型（128-254范围）
pub trait DataUnitExtension: ExtensionTrait {
    /// 获取数据单元类型标志
    fn type_flag(&self) -> u8;

    /// 获取数据单元描述
    fn description(&self) -> &'static str;

    /// 编码数据单元
    fn encode(&self) -> ExtensionResult<Bytes>;

    /// 获取信息对象列表（返回原始字节数据）
    fn get_info_objects(&self) -> Vec<Bytes>;

    /// 克隆为Box包装的trait对象
    fn clone_boxed(&self) -> Box<dyn DataUnitExtension>;
}

/// 数据单元扩展解析器trait
///
/// 用于解析数据单元数据（分离以保持dyn兼容性）
pub trait DataUnitExtensionParser {
    /// 解码数据单元
    fn decode(data: &[u8]) -> ExtensionResult<Box<dyn DataUnitExtension>>;
}

/// 系统类型扩展trait
///
/// 用于定义自定义系统类型（128-255范围）
pub trait SystemTypeExtension: ExtensionTrait {
    /// 获取系统类型代码
    fn system_type_code(&self) -> u8;

    /// 获取系统类型描述
    fn description(&self) -> &'static str;

    /// 编码系统状态
    fn encode_system_status(
        &self,
        status: &crate::info_object::SystemStatus,
    ) -> ExtensionResult<Bytes>;

    /// 解码系统状态
    fn decode_system_status(
        &self,
        data: &[u8],
    ) -> ExtensionResult<crate::info_object::SystemStatus>;

    /// 克隆为Box包装的trait对象
    fn clone_boxed(&self) -> Box<dyn SystemTypeExtension>;
}

/// 部件类型扩展trait
///
/// 用于定义自定义部件类型（128-255范围）
pub trait ComponentTypeExtension: ExtensionTrait {
    /// 获取部件类型代码
    fn component_type_code(&self) -> u8;

    /// 获取部件类型描述
    fn description(&self) -> &'static str;

    /// 编码部件状态
    fn encode_component_status(
        &self,
        status: &crate::info_object::ComponentStatus,
    ) -> ExtensionResult<Bytes>;

    /// 解码部件状态
    fn decode_component_status(
        &self,
        data: &[u8],
    ) -> ExtensionResult<crate::info_object::ComponentStatus>;

    /// 克隆为Box包装的trait对象
    fn clone_boxed(&self) -> Box<dyn ComponentTypeExtension>;
}

/// 模拟量类型扩展trait
///
/// 用于定义自定义模拟量类型（128-255范围）
pub trait AnalogTypeExtension: ExtensionTrait {
    /// 获取模拟量类型代码
    fn analog_type_code(&self) -> u8;

    /// 获取最小值
    fn min_value(&self) -> f64;

    /// 获取最大值
    fn max_value(&self) -> f64;

    /// 获取单位
    fn unit(&self) -> &'static str;

    /// 获取模拟量类型描述
    fn description(&self) -> &'static str;

    /// 验证值是否在有效范围内
    fn validate_value(&self, value: f64) -> bool;

    /// 编码模拟量值
    fn encode_analog_value(
        &self,
        value: &crate::info_object::AnalogValue,
    ) -> ExtensionResult<Bytes>;

    /// 解码模拟量值
    fn decode_analog_value(&self, data: &[u8]) -> ExtensionResult<crate::info_object::AnalogValue>;

    /// 克隆为Box包装的trait对象
    fn clone_boxed(&self) -> Box<dyn AnalogTypeExtension>;
}
