//! GB26875 编解码器模块
//!
//! 提供数据包和数据单元的编解码功能，支持 TCP 流处理和异步操作

// 子模块声明
pub mod traits;
pub mod packet_codec;
pub mod data_unit_codec;
pub mod encoder;
pub mod decoder;

#[cfg(feature = "async")]
pub mod framed;

// 重新导出主要类型
pub use traits::{Codec, Encoder as EncoderTrait, Decoder as DecoderTrait, StreamCodec};
pub use packet_codec::PacketCodec;
pub use data_unit_codec::DataUnitCodec;
pub use encoder::{Encoder, EncoderConfig};
pub use decoder::{Decoder, DecoderConfig};

#[cfg(feature = "async")]
pub use framed::{GB26875FramedCodec, LengthFieldCodec, StreamProcessor, StreamStats};
