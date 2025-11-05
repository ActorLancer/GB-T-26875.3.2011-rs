//! GB26875 构建器模块
//!
//! 提供友好的 API 构建器，简化数据包和数据单元的创建

pub mod data_unit;
pub mod packet;

// 重新导出主要类型
pub use data_unit::*;
pub use packet::*;

use crate::error::EncodeResult;

/// 通用构建器 trait
///
/// 为所有构建器提供统一的接口
pub trait Builder<T> {
    /// 构建目标对象
    ///
    /// # Returns
    /// * `Result<T, EncodeError>` - 成功返回构建的对象
    fn build(self) -> EncodeResult<T>;

    /// 验证构建参数
    ///
    /// # Returns
    /// * `Result<(), EncodeError>` - 验证成功返回 ()
    fn validate(&self) -> EncodeResult<()> {
        Ok(())
    }
}

/// 可重置的构建器 trait
///
/// 为支持重用的构建器提供重置功能
pub trait ResettableBuilder<T>: Builder<T> + Clone {
    /// 重置构建器到初始状态
    fn reset(&mut self);

    /// 构建对象并重置构建器
    ///
    /// # Returns
    /// * `Result<T, EncodeError>` - 成功返回构建的对象
    fn build_and_reset(&mut self) -> EncodeResult<T> {
        let cloned = self.clone();
        let result = cloned.build();
        if result.is_ok() {
            self.reset();
        }
        result
    }
}

mod tests {
    use super::*;
    use crate::error::EncodeError;

    // 测试用的简单构建器
    #[derive(Clone)]
    struct TestBuilder {
        value: Option<u32>,
    }

    impl TestBuilder {
        fn new() -> Self {
            TestBuilder { value: None }
        }

        fn with_value(mut self, value: u32) -> Self {
            self.value = Some(value);
            self
        }
    }

    impl Builder<u32> for TestBuilder {
        fn build(self) -> EncodeResult<u32> {
            self.value.ok_or_else(|| EncodeError::InvalidValue {
                field: "value".to_string(),
                value: "None".to_string(),
                reason: "值不能为空".to_string(),
            })
        }

        fn validate(&self) -> EncodeResult<()> {
            if self.value.is_none() {
                return Err(EncodeError::InvalidValue {
                    field: "value".to_string(),
                    value: "None".to_string(),
                    reason: "值不能为空".to_string(),
                });
            }
            Ok(())
        }
    }

    impl ResettableBuilder<u32> for TestBuilder {
        fn reset(&mut self) {
            self.value = None;
        }
    }

    #[test]
    fn test_builder() {
        let builder = TestBuilder::new().with_value(42);
        assert!(builder.validate().is_ok());
        assert_eq!(builder.build().unwrap(), 42);
    }

    #[test]
    fn test_builder_validation_failure() {
        let builder = TestBuilder::new();
        assert!(builder.validate().is_err());
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_resettable_builder() {
        let mut builder = TestBuilder::new().with_value(42);
        assert_eq!(builder.build_and_reset().unwrap(), 42);
        assert!(builder.validate().is_err()); // 重置后应该验证失败
    }
}
