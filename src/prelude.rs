//! GB26875 预导入模块
//!
//! 这个模块重新导出了库中最常用的类型和 trait，方便用户使用。
//!
//! ```rust
//! use gb26875::prelude::*;
//! ```

// 核心类型
pub use crate::{Packet, VERSION};

// 错误类型
pub use crate::error::{ParseError, EncodeError, ExtensionError, ParseResult, EncodeResult};

// 协议相关
pub use crate::protocol::{Command, SystemType, ComponentType, AnalogType, DataUnitType, ProtocolVersion};
pub use crate::frame::{ControlUnit, Timestamp};

// 编解码
pub use crate::codec::{Codec, PacketCodec, DataUnitCodec, StreamCodec};
pub use crate::parser::{PacketParser, FrameDetector};

// 构建器
pub use crate::builder::packet::PacketBuilder;
pub use crate::builder::data_unit::{SystemStatusBuilder, ComponentStatusBuilder, AnalogValueBuilder};

// 数据单元
pub use crate::data_unit::{DataUnit, GenericDataUnit};
pub use crate::data_unit::standard::{SystemStatus, ComponentStatus, AnalogValue};

// 扩展机制
pub use crate::extension::{ExtensionDataUnit, ExtensionManager, ExtensionRegistry};

// 条件导入
#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

#[cfg(feature = "async")]
pub use crate::codec::{AsyncPacketCodec, AsyncDataUnitCodec};
