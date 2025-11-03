//! GB26875 校验和计算
//!
//! 校验和计算方法：控制单元中各字节数据（第3-27字节）以及应用数据单元的算数校验和，
//! 舍去8位以上的进位位后形成的1字节二进制数

/// 计算 GB26875 校验和
/// 
/// 计算第3到第27字节（控制单元，不包括启动符）+ 应用数据单元所有字节的算数和，
/// 然后只保留低8位（1字节），高于8位的进位全部丢弃
/// 
/// # 参数
/// 
/// * `control_unit` - 控制单元数据（25字节，不包括启动符）
/// * `data_unit` - 应用数据单元（可为空）
/// 
/// # 返回
/// 
/// 校验和字节值
pub fn calculate_checksum(control_unit: &[u8], data_unit: &[u8]) -> u8 {
    let mut sum: u32 = 0;

    // 加上控制单元的所有字节
    for &byte in control_unit {
        sum += byte as u32;
    }

    // 加上应用数据单元的所有字节
    for &byte in data_unit {
        sum += byte as u32;
    }

    // 只保留低8位
    (sum & 0xFF) as u8
}

/// 计算完整数据包的校验和
/// 
/// 从完整数据包中提取控制单元和应用数据单元，然后计算校验和
/// 
/// # 参数
/// 
/// * `packet_data` - 完整数据包（包括启动符，但不包括校验和和结束符）
/// 
/// # 返回
/// 
/// 校验和字节值，如果数据包格式不正确则返回 None
pub fn calculate_packet_checksum(packet_data: &[u8]) -> Option<u8> {
    // 数据包最小长度：启动符(2) + 控制单元(25) = 27
    if packet_data.len() < 27 {
        return None;
    }

    // 提取控制单元（跳过启动符的2字节）
    let control_unit = &packet_data[2..27];
    
    // 提取应用数据单元（如果存在）
    let data_unit = if packet_data.len() > 27 {
        &packet_data[27..]
    } else {
        &[]
    };

    Some(calculate_checksum(control_unit, data_unit))
}

/// 验证数据包校验和
/// 
/// # 参数
/// 
/// * `packet_data` - 完整数据包（不包括结束符）
/// * `expected_checksum` - 期望的校验和值
/// 
/// # 返回
/// 
/// 如果校验和正确返回 true，否则返回 false
pub fn verify_checksum(packet_data: &[u8], expected_checksum: u8) -> bool {
    if let Some(calculated) = calculate_packet_checksum(packet_data) {
        calculated == expected_checksum
    } else {
        false
    }
}

/// 计算原始字节序列的简单校验和
/// 
/// 这个函数用于计算任意字节序列的校验和，主要用于内部计算
/// 
/// # 参数
/// 
/// * `bytes` - 字节序列
/// 
/// # 返回
/// 
/// 校验和字节值
pub fn simple_checksum(bytes: &[u8]) -> u8 {
    let mut sum: u32 = 0;
    for &byte in bytes {
        sum += byte as u32;
    }
    (sum & 0xFF) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_checksum() {
        let data = [1, 2, 3, 4, 5];
        let checksum = simple_checksum(&data);
        assert_eq!(checksum, 15); // 1+2+3+4+5 = 15
    }

    #[test]
    fn test_checksum_overflow() {
        let data = [255, 255, 255]; // 255*3 = 765 = 0x2FD
        let checksum = simple_checksum(&data);
        assert_eq!(checksum, 253); // 765 & 0xFF = 253
    }

    #[test]
    fn test_calculate_checksum() {
        let control_unit = [1; 25]; // 25个1
        let data_unit = [2, 3, 4]; // 应用数据单元
        let checksum = calculate_checksum(&control_unit, &data_unit);
        assert_eq!(checksum, 34); // 25*1 + 2+3+4 = 34
    }

    #[test]
    fn test_calculate_packet_checksum() {
        // 构造一个最小的数据包：启动符 + 控制单元
        let mut packet = vec![0x40, 0x40]; // 启动符
        packet.extend_from_slice(&[1; 25]); // 控制单元
        
        let checksum = calculate_packet_checksum(&packet).unwrap();
        assert_eq!(checksum, 25); // 25个1的和
    }

    #[test]
    fn test_verify_checksum() {
        let mut packet = vec![0x40, 0x40]; // 启动符
        packet.extend_from_slice(&[1; 25]); // 控制单元
        
        assert!(verify_checksum(&packet, 25));
        assert!(!verify_checksum(&packet, 24));
    }

    #[test]
    fn test_packet_too_short() {
        let short_packet = [0x40, 0x40, 1, 2]; // 太短的数据包
        assert!(calculate_packet_checksum(&short_packet).is_none());
    }
}
