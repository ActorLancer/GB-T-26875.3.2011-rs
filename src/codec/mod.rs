//! GB26875 编解码器模块
//!
//! 提供数据包和数据单元的编解码功能，支持 TCP 流处理和异步操作

// 子模块声明
pub mod data_unit_codec;
pub mod decoder;
pub mod encoder;
pub mod packet_codec;
pub mod traits;


pub mod framed;

// 重新导出主要类型
pub use data_unit_codec::DataUnitCodec;
pub use decoder::{Decoder, DecoderConfig};
pub use encoder::{Encoder, EncoderConfig};
pub use packet_codec::PacketCodec;
pub use traits::{Codec, Decoder as DecoderTrait, Encoder as EncoderTrait, StreamCodec};


pub use framed::{GB26875FramedCodec, LengthFieldCodec, StreamProcessor, StreamStats};
