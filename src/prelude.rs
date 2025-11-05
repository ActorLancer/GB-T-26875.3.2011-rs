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
pub use crate::error::{EncodeError, EncodeResult, ExtensionError, ParseError, ParseResult};

// 协议相关
pub use crate::frame::{ControlUnit, Timestamp};
pub use crate::protocol::{
    AnalogType, Command, ComponentType, DataUnitType, ProtocolVersion, SystemType,
};

// 编解码
pub use crate::codec::{Codec, DataUnitCodec, PacketCodec, StreamCodec};
pub use crate::parser::{DataValidator, FrameDetector, PacketParser};

// 构建器
pub use crate::builder::data_unit::{
    AnalogValueBuilder, ComponentStatusBuilder, DataUnitBuilder, SystemStatusBuilder,
};
pub use crate::builder::packet::PacketBuilder;

// 数据单元
pub use crate::data_unit::standard::{downstream, upstream};
pub use crate::data_unit::{DataUnit, GenericDataUnit};

// 信息对象 (重新导出以便构建器使用)
pub use crate::info_object::{
    AnalogType as InfoAnalogType, AnalogValue, ComponentConfig, ComponentStatus, DeviceConfig,
    DeviceOperation, DeviceVersion, FireSystemConfig, FireSystemOperation, FireSystemVersion,
    InfoObject, SystemStatus,
};

// 扩展机制
pub use crate::extension::{ExtensionDataUnit, ExtensionManager, ExtensionRegistry};

// 条件导入
#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

#[cfg(feature = "async")]
pub use crate::codec::{AsyncDataUnitCodec, AsyncPacketCodec};
