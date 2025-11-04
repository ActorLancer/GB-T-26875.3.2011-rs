use gb26875::frame::{ControlUnit, Packet};
use gb26875::frame::timestamp::Timestamp;
use gb26875::protocol::{Command, ProtocolVersion};

fn main() {
    let control_unit = ControlUnit::new(
        1,
        ProtocolVersion::v1_0(),
        Timestamp::now(),
        0x123456,
        0x654321,
        0,
        Command::Control,
    ).unwrap();
    
    let packet = Packet::empty(control_unit);
    let encoded = packet.encode().unwrap();
    
    println!("Empty packet size: {} bytes", encoded.len());
    println!("Expected size: 30 bytes");
    println!("Packet.len(): {} bytes", packet.len());
    println!("Bytes: {:02x?}", encoded);
}
