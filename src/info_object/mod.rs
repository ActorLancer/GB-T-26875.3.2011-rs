//! GB26875 信息对象模块
//!
//! 提供标准信息对象的实现

// 这个模块目前为占位符，将在后续阶段实现
// 信息对象是更高级别的数据结构，基于数据单元构建

/// 信息对象基础 trait
pub trait InfoObject: std::fmt::Debug + Send + Sync {
    /// 获取信息对象类型
    fn object_type(&self) -> &str;
    
    /// 获取信息对象描述
    fn description(&self) -> Option<&str> {
        None
    }
}

/// 占位符结构，将来实现具体的信息对象
#[derive(Debug)]
pub struct InfoObjectPlaceholder;

impl InfoObject for InfoObjectPlaceholder {
    fn object_type(&self) -> &str {
        "placeholder"
    }
}
