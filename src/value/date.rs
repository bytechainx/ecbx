//! 日期（自有类型；不引入任何第三方日期库）。
//!
//! 只承载「期间身份」，不承载时区、夏令时、算术或格式化。

use std::fmt;

use crate::error::{EcbError, EcbResult};

/// 日历日期。
///
/// 构造与解析都做完整校验：月 1–12、日按月份与闰年。**不接受**未补零形式
/// （`2026-2-3`）、非 ISO 分隔符（`2026/02/03`）与带时间部分的字符串。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// 年（1–9999）。
    year: i16,
    /// 月（1–12）。
    month: u8,
    /// 日（1–该月天数）。
    day: u8,
}

/// 年的取值上界：保证 ISO 形式固定为 4 位。
const MAX_YEAR: i16 = 9999;

impl Date {
    /// 构造并校验日期。
    pub fn new(year: i16, month: u8, day: u8) -> EcbResult<Self> {
        if !(1..=MAX_YEAR).contains(&year) {
            return Err(EcbError::Invalid(format!("年份 {year} 超出 1–{MAX_YEAR}")));
        }
        let limit = Self::days_in_month(year, month)?;
        if day == 0 || day > limit {
            return Err(EcbError::Invalid(format!(
                "{year}-{month:02} 的日 {day} 超出 1–{limit}"
            )));
        }
        Ok(Self { year, month, day })
    }

    /// 严格 ISO 解析：只接受 `YYYY-MM-DD`（月、日各两位补零）。
    pub fn parse(input: &str) -> EcbResult<Self> {
        let bytes = input.as_bytes();
        if bytes.len() != 10 {
            return Err(EcbError::Invalid(format!(
                "日期必须为 YYYY-MM-DD：长度 {} 非法",
                bytes.len()
            )));
        }
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(EcbError::Invalid(
                "日期必须为 YYYY-MM-DD：分隔符必须为 `-`".to_owned(),
            ));
        }
        let digits_at = |range: core::ops::Range<usize>| -> EcbResult<&str> {
            for index in range.clone() {
                if !bytes[index].is_ascii_digit() {
                    return Err(EcbError::Invalid(format!("日期第 {index} 位不是数字")));
                }
            }
            Ok(&input[range])
        };
        let year = digits_at(0..4)?
            .parse::<i16>()
            .map_err(|e| EcbError::Invalid(format!("年份无法解析：{e}")))?;
        let month = digits_at(5..7)?
            .parse::<u8>()
            .map_err(|e| EcbError::Invalid(format!("月份无法解析：{e}")))?;
        let day = digits_at(8..10)?
            .parse::<u8>()
            .map_err(|e| EcbError::Invalid(format!("日无法解析：{e}")))?;
        Self::new(year, month, day)
    }

    /// 年。
    #[must_use]
    pub fn year(&self) -> i16 {
        self.year
    }

    /// 月（1–12）。
    #[must_use]
    pub fn month(&self) -> u8 {
        self.month
    }

    /// 日（1–31）。
    #[must_use]
    pub fn day(&self) -> u8 {
        self.day
    }

    /// 是否闰年（格里高利历规则）。
    #[must_use]
    pub fn is_leap_year(year: i16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// 某年某月的天数；月份非法时返回 `Invalid`。
    pub fn days_in_month(year: i16, month: u8) -> EcbResult<u8> {
        let limit = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            other => {
                return Err(EcbError::Invalid(format!("月份 {other} 超出 1–12")));
            }
        };
        Ok(limit)
    }

    /// ISO 文本形式 `YYYY-MM-DD`。
    #[must_use]
    pub fn to_iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_iso())
    }
}

#[cfg(test)]
mod tests {
    use super::Date;
    use crate::error::EcbErrorKind;

    #[test]
    fn accepts_padded_iso_form() {
        let date = Date::parse("2026-09-18").expect("合法日期");
        assert_eq!((date.year(), date.month(), date.day()), (2026, 9, 18));
        assert_eq!(date.to_iso(), "2026-09-18");
        assert_eq!(date.to_string(), "2026-09-18");
    }

    #[test]
    fn rejects_non_padded_form() {
        let error = Date::parse("2026-2-3").expect_err("未补零必须拒绝");
        assert_eq!(error.kind(), EcbErrorKind::Invalid);
    }

    #[test]
    fn rejects_slash_separator() {
        assert!(Date::parse("2026/02/03").is_err());
    }

    #[test]
    fn rejects_time_component() {
        assert!(Date::parse("2026-02-03T00:00:00Z").is_err());
        assert!(Date::parse("2026-02-03 00:00:00").is_err());
    }

    #[test]
    fn rejects_out_of_range_month_and_day() {
        assert!(Date::new(2026, 0, 1).is_err());
        assert!(Date::new(2026, 13, 1).is_err());
        assert!(Date::new(2026, 2, 29).is_err());
        assert!(Date::new(2026, 4, 31).is_err());
        assert!(Date::new(2026, 1, 0).is_err());
        assert!(Date::new(0, 1, 1).is_err());
        assert!(Date::new(10000, 1, 1).is_err());
    }

    #[test]
    fn leap_year_rules_cover_century_cases() {
        assert!(Date::is_leap_year(2024));
        assert!(!Date::is_leap_year(2026));
        assert!(!Date::is_leap_year(1900));
        assert!(Date::is_leap_year(2000));
        assert!(Date::new(2024, 2, 29).is_ok());
        assert!(Date::new(2000, 2, 29).is_ok());
        assert!(Date::new(1900, 2, 29).is_err());
    }

    #[test]
    fn rejects_non_ascii_and_sign_characters() {
        assert!(Date::parse("2026-09-1８").is_err());
        assert!(Date::parse("+026-09-18").is_err());
        assert!(Date::parse("2026-09-1 ").is_err());
    }
}
