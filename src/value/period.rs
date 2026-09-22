//! 期间与频率。
//!
//! `Period` 表达「期间身份」，`Frequency` 表达「发布频率」，两者不互相顶替。
//! **禁止**用 `String` 顶替 `Period`。

use crate::error::{EcbError, EcbResult};
use crate::value::date::Date;

/// 频率（契约 `source-library-contract.md` §2.1 的 7 个取值）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// 日频。
    Daily,
    /// 周频。
    Weekly,
    /// 月频。
    Monthly,
    /// 季频。
    Quarterly,
    /// 年频。
    Annual,
    /// 事件驱动。
    Event,
    /// 不规则。
    Irregular,
}

impl Frequency {
    /// 全部取值。
    pub const ALL: [Self; 7] = [
        Self::Daily,
        Self::Weekly,
        Self::Monthly,
        Self::Quarterly,
        Self::Annual,
        Self::Event,
        Self::Irregular,
    ];

    /// 名称（本层自有拼写，非源侧 code list）。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Quarterly => "Quarterly",
            Self::Annual => "Annual",
            Self::Event => "Event",
            Self::Irregular => "Irregular",
        }
    }

    /// 按名称解析。
    pub fn from_name(name: &str) -> EcbResult<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.name() == name)
            .ok_or_else(|| EcbError::Invalid(format!("未知频率：{name}")))
    }
}

/// 业务期间。
///
/// 变体集合与契约 `source-library-contract.md` §2.2 一致；`Event` 用于事件驱动的
/// 发布物（如政策决定），其身份由事件日期承载。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    /// 某日。
    Day(Date),
    /// 某年某月。
    Month {
        /// 年。
        year: i16,
        /// 月（1–12）。
        month: u8,
    },
    /// 某年某季度。
    Quarter {
        /// 年。
        year: i16,
        /// 季（1–4）。
        quarter: u8,
    },
    /// 某年。
    Year(i16),
    /// 某事件日。
    Event {
        /// 事件日期。
        date: Date,
    },
}

impl Period {
    /// 复验可由调用方直接构造的期间，复用受检构造器的取值域。
    pub(crate) fn validate(&self) -> EcbResult<()> {
        match *self {
            Self::Day(date) | Self::Event { date } => {
                Date::new(date.year(), date.month(), date.day())?;
            }
            Self::Month { year, month } => {
                Self::month(year, month)?;
            }
            Self::Quarter { year, quarter } => {
                Self::quarter(year, quarter)?;
            }
            Self::Year(year) => {
                Self::year(year)?;
            }
        }
        Ok(())
    }

    /// 构造「某日」。
    #[must_use]
    pub fn day(date: Date) -> Self {
        Self::Day(date)
    }

    /// 构造「某年某月」；月份非法时返回 `Invalid`。
    pub fn month(year: i16, month: u8) -> EcbResult<Self> {
        // 用该月首日验证年 / 月的合法性，避免另写一套范围判断。
        Date::new(year, month, 1).map_err(|error| match error {
            EcbError::Invalid(message) => EcbError::Invalid(format!("月期间非法：{message}")),
            other => other,
        })?;
        Ok(Self::Month { year, month })
    }

    /// 构造「某年某季度」；季度非法时返回 `Invalid`。
    pub fn quarter(year: i16, quarter: u8) -> EcbResult<Self> {
        if !(1..=4).contains(&quarter) {
            return Err(EcbError::Invalid(format!("季度 {quarter} 超出 1–4")));
        }
        // 用该季度首日验证年份合法性。
        Date::new(year, (quarter - 1) * 3 + 1, 1)?;
        Ok(Self::Quarter { year, quarter })
    }

    /// 构造「某年」；年份非法时返回 `Invalid`。
    pub fn year(year: i16) -> EcbResult<Self> {
        Date::new(year, 1, 1)?;
        Ok(Self::Year(year))
    }

    /// 构造「某事件日」。
    #[must_use]
    pub fn event(date: Date) -> Self {
        Self::Event { date }
    }

    /// 规范键（可被 [`Period::parse`] 反向解析）。
    ///
    /// 形态：`YYYY-MM-DD`（日）/ `YYYY-MM`（月）/ `YYYY-Qn`（季）/ `YYYY`（年）/
    /// `event:YYYY-MM-DD`（事件）。
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Day(date) => date.to_iso(),
            Self::Month { year, month } => format!("{year:04}-{month:02}"),
            Self::Quarter { year, quarter } => format!("{year:04}-Q{quarter}"),
            Self::Year(year) => format!("{year:04}"),
            Self::Event { date } => format!("event:{}", date.to_iso()),
        }
    }

    /// 严格解析 [`Period::key`] 产生的形态。
    pub fn parse(input: &str) -> EcbResult<Self> {
        if let Some(rest) = input.strip_prefix("event:") {
            return Ok(Self::event(Date::parse(rest)?));
        }
        let bytes = input.as_bytes();
        match bytes.len() {
            4 => {
                let year = parse_year(input)?;
                Self::year(year)
            }
            7 if bytes[4] == b'-' && bytes[5] == b'Q' => {
                let year = parse_year(&input[0..4])?;
                let quarter = parse_u8(&input[6..7], "季度")?;
                Self::quarter(year, quarter)
            }
            7 if bytes[4] == b'-' => {
                let year = parse_year(&input[0..4])?;
                let month = parse_u8(&input[5..7], "月份")?;
                Self::month(year, month)
            }
            10 => Ok(Self::day(Date::parse(input)?)),
            _ => Err(EcbError::Invalid(format!(
                "期间形态非法：{input}（须为 YYYY-MM-DD / YYYY-MM / YYYY-Qn / YYYY / event:YYYY-MM-DD）"
            ))),
        }
    }

    /// 与该期间最贴合的频率。
    #[must_use]
    pub fn frequency(&self) -> Frequency {
        match self {
            Self::Day(_) => Frequency::Daily,
            Self::Month { .. } => Frequency::Monthly,
            Self::Quarter { .. } => Frequency::Quarterly,
            Self::Year(_) => Frequency::Annual,
            Self::Event { .. } => Frequency::Event,
        }
    }
}

/// 解析固定 4 位年份。
fn parse_year(segment: &str) -> EcbResult<i16> {
    if segment.len() != 4 || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(EcbError::Invalid(format!("年份形态非法：{segment}")));
    }
    segment
        .parse::<i16>()
        .map_err(|error| EcbError::Invalid(format!("年份无法解析：{error}")))
}

/// 解析固定数字段。
fn parse_u8(segment: &str, label: &str) -> EcbResult<u8> {
    if segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(EcbError::Invalid(format!("{label}形态非法：{segment}")));
    }
    segment
        .parse::<u8>()
        .map_err(|error| EcbError::Invalid(format!("{label}无法解析：{error}")))
}

#[cfg(test)]
mod tests {
    use super::{Frequency, Period};
    use crate::error::EcbErrorKind;
    use crate::value::date::Date;

    fn date(year: i16, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("合法日期")
    }

    #[test]
    fn frequency_names_round_trip() {
        for frequency in Frequency::ALL {
            assert_eq!(
                Frequency::from_name(frequency.name()).expect("可回读"),
                frequency
            );
        }
        assert!(Frequency::from_name("Fortnightly").is_err());
    }

    #[test]
    fn period_keys_round_trip() {
        let cases = [
            Period::day(date(2026, 9, 18)),
            Period::month(2026, 9).expect("合法月期间"),
            Period::quarter(2026, 3).expect("合法季期间"),
            Period::year(2026).expect("合法年期间"),
            Period::event(date(2026, 9, 18)),
        ];
        for period in cases {
            let key = period.key();
            assert_eq!(Period::parse(&key).expect("可回读"), period, "key={key}");
        }
    }

    #[test]
    fn period_keys_have_fixed_shapes() {
        assert_eq!(Period::day(date(2026, 9, 18)).key(), "2026-09-18");
        assert_eq!(Period::month(2026, 9).expect("合法").key(), "2026-09");
        assert_eq!(Period::quarter(2026, 3).expect("合法").key(), "2026-Q3");
        assert_eq!(Period::year(2026).expect("合法").key(), "2026");
        assert_eq!(Period::event(date(2026, 9, 18)).key(), "event:2026-09-18");
    }

    #[test]
    fn period_frequency_never_conflates_granularity() {
        assert_eq!(Period::day(date(2026, 9, 18)).frequency(), Frequency::Daily);
        assert_eq!(
            Period::month(2026, 9).expect("合法").frequency(),
            Frequency::Monthly
        );
        assert_eq!(
            Period::quarter(2026, 3).expect("合法").frequency(),
            Frequency::Quarterly
        );
        assert_eq!(
            Period::year(2026).expect("合法").frequency(),
            Frequency::Annual
        );
        assert_eq!(
            Period::event(date(2026, 9, 18)).frequency(),
            Frequency::Event
        );
    }

    #[test]
    fn period_parse_rejects_malformed_input() {
        let cases = [
            "2026-2-3",
            "2026/09/18",
            "2026-13",
            "2026-Q5",
            "2026-Q0",
            "26",
            "2026-09-18T00:00:00Z",
            "",
        ];
        for input in cases {
            let error = Period::parse(input).expect_err(input);
            assert_eq!(error.kind(), EcbErrorKind::Invalid, "{input}");
        }
    }

    #[test]
    fn period_constructors_validate_ranges() {
        assert!(Period::month(2026, 0).is_err());
        assert!(Period::month(2026, 13).is_err());
        assert!(Period::quarter(2026, 0).is_err());
        assert!(Period::quarter(2026, 5).is_err());
        assert!(Period::year(0).is_err());
        assert!(Period::year(10000).is_err());
    }

    #[test]
    fn validation_covers_valid_period_variants() {
        for key in [
            "2024-02-29",
            "2026-09",
            "2026-Q3",
            "2026",
            "event:2026-09-23",
        ] {
            assert!(Period::parse(key).unwrap().validate().is_ok());
        }
        for period in [
            Period::Month { year: 0, month: 1 },
            Period::Month {
                year: 2026,
                month: 0,
            },
            Period::Quarter {
                year: 0,
                quarter: 1,
            },
            Period::Quarter {
                year: 2026,
                quarter: 5,
            },
            Period::Year(10000),
        ] {
            assert!(period.validate().is_err());
        }
    }
}
