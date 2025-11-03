//! GB26875 时间标签定义
//!
//! 时间标签为6字节，格式为：秒、分、时、日、月、年

use crate::error::{ParseError, ParseResult};

/// GB26875 时间标签（6字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamp {
    /// 秒 (0-59)
    pub second: u8,
    /// 分 (0-59)
    pub minute: u8,
    /// 时 (0-23)
    pub hour: u8,
    /// 日 (1-31)
    pub day: u8,
    /// 月 (1-12)
    pub month: u8,
    /// 年 (0-99, 表示2000-2099年)
    pub year: u8,
}

impl Timestamp {
    /// 创建新的时间标签
    pub fn new(second: u8, minute: u8, hour: u8, day: u8, month: u8, year: u8) -> ParseResult<Self> {
        // 验证时间字段的有效性
        if second > 59 {
            return Err(ParseError::InvalidTimestamp {
                field: "second".to_string(),
                value: second,
            });
        }
        if minute > 59 {
            return Err(ParseError::InvalidTimestamp {
                field: "minute".to_string(),
                value: minute,
            });
        }
        if hour > 23 {
            return Err(ParseError::InvalidTimestamp {
                field: "hour".to_string(),
                value: hour,
            });
        }
        if day == 0 || day > 31 {
            return Err(ParseError::InvalidTimestamp {
                field: "day".to_string(),
                value: day,
            });
        }
        if month == 0 || month > 12 {
            return Err(ParseError::InvalidTimestamp {
                field: "month".to_string(),
                value: month,
            });
        }
        if year > 99 {
            return Err(ParseError::InvalidTimestamp {
                field: "year".to_string(),
                value: year,
            });
        }

        Ok(Self {
            second,
            minute,
            hour,
            day,
            month,
            year,
        })
    }

    /// 从字节数组解析时间标签
    pub fn from_bytes(bytes: &[u8]) -> ParseResult<Self> {
        if bytes.len() < 6 {
            return Err(ParseError::TooShort {
                got: bytes.len(),
                need: 6,
            });
        }

        Self::new(
            bytes[0], // 秒
            bytes[1], // 分
            bytes[2], // 时
            bytes[3], // 日
            bytes[4], // 月
            bytes[5], // 年
        )
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> [u8; 6] {
        [
            self.second,
            self.minute,
            self.hour,
            self.day,
            self.month,
            self.year,
        ]
    }

    /// 获取当前时间的时间标签
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = duration.as_secs();

        // 转换为本地时间（简化实现，实际应该考虑时区）
        let days = timestamp / 86400; // 天数
        let seconds_in_day = timestamp % 86400;
        
        let hour = (seconds_in_day / 3600) as u8;
        let minute = ((seconds_in_day % 3600) / 60) as u8;
        let second = (seconds_in_day % 60) as u8;

        // 简化的日期计算（从1970年1月1日开始）
        let mut year = 1970;
        let mut remaining_days = days;

        // 跳到2000年
        if remaining_days >= 10957 { // 1970-2000的天数
            year = 2000;
            remaining_days -= 10957;
        }

        // 计算年份
        while remaining_days >= 365 {
            let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            
            if remaining_days >= days_in_year {
                remaining_days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }

        // 计算月份和日期
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_month = [31, if is_leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        let mut month = 1;
        for &days in &days_in_month {
            if remaining_days >= days as u64 {
                remaining_days -= days as u64;
                month += 1;
            } else {
                break;
            }
        }

        let day = (remaining_days + 1) as u8;
        let year_2digit = ((year % 100) as u8).min(99);

        // 使用 unwrap_or 来处理可能的验证错误，提供默认值
        Self::new(second, minute, hour, day, month, year_2digit)
            .unwrap_or_else(|_| Self {
                second: 0,
                minute: 0,
                hour: 0,
                day: 1,
                month: 1,
                year: 25, // 2025年
            })
    }

    /// 转换为完整年份（2000-2099）
    pub fn full_year(&self) -> u16 {
        2000 + self.year as u16
    }

    /// 验证时间标签是否有效
    pub fn is_valid(&self) -> bool {
        self.second <= 59
            && self.minute <= 59
            && self.hour <= 23
            && self.day >= 1
            && self.day <= 31
            && self.month >= 1
            && self.month <= 12
            && self.year <= 99
    }

    /// 转换为标准的 Unix 时间戳（秒）
    pub fn to_unix_timestamp(&self) -> Option<u64> {
        if !self.is_valid() {
            return None;
        }

        // 简化实现，实际应该使用更精确的日期库
        let year = self.full_year() as u64;
        let mut timestamp = 0u64;

        // 从1970年计算到目标年份的秒数
        for y in 1970..year {
            let is_leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
            timestamp += if is_leap { 366 } else { 365 };
        }

        // 添加月份的天数
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_month = [31, if is_leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        for month in 1..self.month {
            timestamp += days_in_month[(month - 1) as usize] as u64;
        }

        // 添加天数、小时、分钟、秒
        timestamp += (self.day - 1) as u64; // 天数（减1因为当天开始为0）
        timestamp = timestamp * 24 * 60 * 60; // 转换为秒
        timestamp += self.hour as u64 * 3600;
        timestamp += self.minute as u64 * 60;
        timestamp += self.second as u64;

        Some(timestamp)
    }

    /// 从 Unix 时间戳创建时间标签
    pub fn from_unix_timestamp(timestamp: u64) -> Self {
        // 这里使用简化实现，实际应该使用日期库
        let days = timestamp / 86400;
        let seconds_in_day = timestamp % 86400;
        
        let hour = (seconds_in_day / 3600) as u8;
        let minute = ((seconds_in_day % 3600) / 60) as u8;
        let second = (seconds_in_day % 60) as u8;

        // 简化的日期计算
        let mut year = 1970u64;
        let mut remaining_days = days;

        while remaining_days >= 365 {
            let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            
            if remaining_days >= days_in_year {
                remaining_days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }

        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_month = [31, if is_leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        let mut month = 1;
        for &days in &days_in_month {
            if remaining_days >= days as u64 {
                remaining_days -= days as u64;
                month += 1;
            } else {
                break;
            }
        }

        let day = (remaining_days + 1) as u8;
        let year_2digit = ((year % 100) as u8).min(99);        Self::new(second, minute, hour, day, month, year_2digit)
            .unwrap_or_else(|_| Self::now())
    }

    /// 编码为字节序列（用于DataUnit trait）
    pub fn encode(&self) -> crate::error::EncodeResult<bytes::Bytes> {
        Ok(bytes::Bytes::from(self.to_bytes().to_vec()))
    }

    /// 从字节序列解析（用于DataUnit trait）
    pub fn parse(data: &[u8]) -> crate::error::ParseResult<Self> {
        Self::from_bytes(data)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.full_year(),
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_creation() {
        let ts = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        assert_eq!(ts.second, 2);
        assert_eq!(ts.minute, 58);
        assert_eq!(ts.hour, 9);
        assert_eq!(ts.day, 26);
        assert_eq!(ts.month, 9);
        assert_eq!(ts.year, 12);
        assert_eq!(ts.full_year(), 2012);
    }

    #[test]
    fn test_timestamp_invalid() {
        assert!(Timestamp::new(60, 0, 0, 1, 1, 0).is_err()); // 秒无效
        assert!(Timestamp::new(0, 60, 0, 1, 1, 0).is_err()); // 分无效
        assert!(Timestamp::new(0, 0, 24, 1, 1, 0).is_err()); // 时无效
        assert!(Timestamp::new(0, 0, 0, 0, 1, 0).is_err());  // 日无效
        assert!(Timestamp::new(0, 0, 0, 1, 0, 0).is_err());  // 月无效
        assert!(Timestamp::new(0, 0, 0, 1, 1, 100).is_err()); // 年无效
    }

    #[test]
    fn test_timestamp_bytes() {
        let ts = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        let bytes = ts.to_bytes();
        assert_eq!(bytes, [2, 58, 9, 26, 9, 12]);

        let parsed = Timestamp::from_bytes(&bytes).unwrap();
        assert_eq!(ts, parsed);
    }

    #[test]
    fn test_timestamp_display() {
        let ts = Timestamp::new(2, 58, 9, 26, 9, 12).unwrap();
        assert_eq!(ts.to_string(), "2012-09-26 09:58:02");
    }
}
