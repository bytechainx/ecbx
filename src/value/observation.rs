//! 观测值对象：单位、缺失表达与主观测类型。
//!
//! **源侧单位原样保留**：单位换算与派生指标一律归下游，本层不做。

use std::fmt;

use crate::error::{EcbError, EcbResult};
use crate::value::date::Date;
use crate::value::period::{Frequency, Period};
use crate::value::series::{
    ensure_not_curve_product, validate_series_identity, EcbDataType, EcbSeriesIdentity,
};

/// 源侧单位。
///
/// 保留**源单位原文**（清单 §3、§4 的形态），本层不做任何换算，也不内置单位枚举：
/// 清单未给出单位全集，凭空虚造集合就是编造。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit(String);

impl Unit {
    /// 构造并校验（非空、去首尾空白后非空）。
    pub fn try_new(text: &str) -> EcbResult<Self> {
        if text.trim().is_empty() {
            return Err(EcbError::Missing("源侧单位为空".to_owned()));
        }
        Ok(Self(text.trim().to_owned()))
    }

    /// 源单位原文。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// 缺失原因。
///
/// 这是**本库自有分类**，不是源侧 `obs_status` code list 的实现：本库不知道也不声称
/// 掌握该 code list，故只把源侧状态原文原样带出。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EcbMissingReason {
    /// 源侧给出了状态标记（原文保留）。
    SourceStatus(String),
    /// 源侧没有给出可用的状态标记。
    Unspecified,
}

impl EcbMissingReason {
    /// 可读描述。
    #[must_use]
    pub fn describe(&self) -> &str {
        match self {
            Self::SourceStatus(status) => status,
            Self::Unspecified => "源侧未给出状态标记",
        }
    }
}

/// 观测值：有值或具名缺失。
///
/// 缺失**绝不**静默转 0，也不允许用「空字符串」之类形态顶替。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum EcbValue {
    /// 源给出的数值。
    Value(f64),
    /// 缺失，附具名原因。
    Missing(EcbMissingReason),
}

impl EcbValue {
    /// 有值时返回数值。
    #[must_use]
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Value(value) => Some(*value),
            Self::Missing(_) => None,
        }
    }

    /// 是否缺失。
    #[must_use]
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    /// 缺失原因；有值时为 `None`。
    #[must_use]
    pub fn missing_reason(&self) -> Option<&EcbMissingReason> {
        match self {
            Self::Value(_) => None,
            Self::Missing(reason) => Some(reason),
        }
    }
}

/// 一条 ECB 源事实观测。
///
/// 身份形：`source_series_id = dataflow + DSD + 规范化维度键` +
/// `indicator + subject + period + vintage`，由 [`EcbObservation::source_series_id`] 组合。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct EcbObservation {
    /// 序列身份（`dataflow + DSD + 维度键 + indicator + subject`）。
    pub series: EcbSeriesIdentity,
    /// 数据类型。
    pub data_type: EcbDataType,
    /// 频率。
    pub frequency: Frequency,
    /// 业务期间。
    pub period: Period,
    /// 观测值（有值或具名缺失）。
    pub value: EcbValue,
    /// 源侧单位（原样保留）。
    pub unit: Unit,
    /// 修订标识。无官方 vintage 面时**必须**为 `None`，不得伪造。
    pub vintage: Option<Date>,
}

impl EcbObservation {
    /// 构造并校验一条观测。
    pub fn new(
        series: EcbSeriesIdentity,
        data_type: EcbDataType,
        frequency: Frequency,
        period: Period,
        value: EcbValue,
        unit: Unit,
        vintage: Option<Date>,
    ) -> EcbResult<Self> {
        let observation = Self {
            series,
            data_type,
            frequency,
            period,
            value,
            unit,
            vintage,
        };
        validate_observation(&observation)?;
        Ok(observation)
    }

    /// 完整源身份串（含期间与修订标识）。
    #[must_use]
    pub fn source_series_id(&self) -> String {
        let vintage = self
            .vintage
            .map_or_else(|| "none".to_owned(), |date| date.to_iso());
        format!(
            "{}:{}:{}",
            self.series.series_id(),
            self.period.key(),
            vintage
        )
    }
}

/// 校验观测的完整性与内部一致性。
pub fn validate_observation(observation: &EcbObservation) -> EcbResult<()> {
    observation.period.validate()?;
    if let EcbValue::Value(value) = observation.value {
        if !value.is_finite() {
            return Err(EcbError::Invalid("观测值必须是有限数值".to_owned()));
        }
    }
    validate_series_identity(&observation.series)?;
    if let EcbValue::Missing(EcbMissingReason::SourceStatus(status)) = &observation.value {
        if status.trim().is_empty() {
            return Err(EcbError::SemanticallyRejected(
                "缺失原因声明为源侧状态，但状态原文为空".to_owned(),
            ));
        }
    }
    Ok(())
}

/// 拒绝把曲线点观测晋级为普通观测。
///
/// `ZERO_SPOT` / `FORWARD` 属 YC dataflow 的曲线点：点映射须 `yieldx` 批准，
/// 本库不得晋级（跨源路由契约 §2 / §7）。
pub fn reject_curve_promotion(observation: &EcbObservation) -> EcbResult<()> {
    ensure_not_curve_product(&observation.data_type)
}

#[cfg(test)]
mod tests {
    use super::{
        reject_curve_promotion, validate_observation, EcbDataType, EcbMissingReason,
        EcbObservation, EcbValue, Unit,
    };
    use crate::error::EcbErrorKind;
    use crate::value::date::Date;
    use crate::value::period::{Frequency, Period};
    use crate::value::series::{EcbDataflowId, EcbDimension, EcbSeriesIdentity};

    fn unit() -> Unit {
        Unit::try_new("SYNTH_UNIT").expect("合法单位")
    }

    fn series() -> EcbSeriesIdentity {
        let dataflow = EcbDataflowId::try_new(
            "YC",
            "DSD_SYNTH",
            vec![EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度")],
        )
        .expect("合法身份");
        EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", "SUBJ_SYNTH").expect("合法序列身份")
    }

    fn observation(data_type: EcbDataType, value: EcbValue) -> EcbObservation {
        EcbObservation::new(
            series(),
            data_type,
            Frequency::Daily,
            Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            value,
            unit(),
            None,
        )
        .expect("合法观测")
    }

    #[test]
    fn non_finite_constructor_is_rejected() {
        let period = Period::day(Date::new(2026, 9, 18).expect("合法日期"));
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                EcbObservation::new(
                    series(),
                    EcbDataType::PolicyRate,
                    Frequency::Daily,
                    period,
                    EcbValue::Value(value),
                    unit(),
                    None
                )
                .expect_err("非有限值必须拒绝")
                .kind(),
                crate::EcbErrorKind::Invalid
            );
        }
    }

    #[test]
    fn non_finite_mutation_is_rejected() {
        let period = Period::day(Date::new(2026, 9, 18).expect("合法日期"));
        let value = 1.0;
        let mut sample = EcbObservation::new(
            series(),
            EcbDataType::PolicyRate,
            Frequency::Daily,
            period,
            EcbValue::Value(value),
            unit(),
            None,
        )
        .expect("有限值合法");
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            sample.value = EcbValue::Value(value);
            assert_eq!(
                validate_observation(&sample)
                    .expect_err("修改后的非有限值必须拒绝")
                    .kind(),
                crate::EcbErrorKind::Invalid
            );
        }
    }

    #[test]
    fn unit_preserves_source_text_without_conversion() {
        let unit = Unit::try_new(" 100 million yen ").expect("合法单位");
        assert_eq!(unit.text(), "100 million yen");
        assert_eq!(unit.to_string(), "100 million yen");
        assert!(Unit::try_new("   ").is_err());
    }

    #[test]
    fn missing_value_is_named_and_never_zero() {
        let observation = observation(
            EcbDataType::PolicyRate,
            EcbValue::Missing(EcbMissingReason::SourceStatus("SYNTH_STATUS".to_owned())),
        );
        assert!(observation.value.is_missing());
        assert_eq!(observation.value.value(), None, "缺失不得静默转 0");
        assert_eq!(
            observation.value.missing_reason(),
            Some(&EcbMissingReason::SourceStatus("SYNTH_STATUS".to_owned()))
        );
        assert_eq!(
            EcbMissingReason::Unspecified.describe(),
            "源侧未给出状态标记"
        );
        assert!(validate_observation(&observation).is_ok());
    }

    #[test]
    fn empty_source_status_is_semantically_rejected() {
        let constructed = EcbObservation::new(
            series(),
            EcbDataType::PolicyRate,
            Frequency::Daily,
            Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            EcbValue::Missing(EcbMissingReason::SourceStatus("  ".to_owned())),
            unit(),
            None,
        );
        let error = constructed.expect_err("空状态原文必须拒绝");
        assert_eq!(error.kind(), EcbErrorKind::SemanticallyRejected);

        // 绕过构造器直接交给校验入口，结论必须一致。
        let hand_built = EcbObservation {
            series: series(),
            data_type: EcbDataType::PolicyRate,
            frequency: Frequency::Daily,
            period: Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            value: EcbValue::Missing(EcbMissingReason::SourceStatus(String::new())),
            unit: unit(),
            vintage: None,
        };
        assert_eq!(
            validate_observation(&hand_built)
                .expect_err("空状态原文必须拒绝")
                .kind(),
            EcbErrorKind::SemanticallyRejected
        );
    }

    #[test]
    fn source_series_id_composes_period_and_vintage() {
        let plain = observation(EcbDataType::PolicyRate, EcbValue::Value(3.25));
        assert_eq!(
            plain.source_series_id(),
            "YC:DSD_SYNTH:DIM_A=VAL_A:IND_SYNTH:SUBJ_SYNTH:2026-09-18:none"
        );

        let vintaged = EcbObservation::new(
            series(),
            EcbDataType::PolicyRate,
            Frequency::Daily,
            Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            EcbValue::Value(3.25),
            unit(),
            Some(Date::new(2026, 9, 20).expect("合法日期")),
        )
        .expect("合法观测");
        assert!(
            vintaged.source_series_id().ends_with(":2026-09-20"),
            "{}",
            vintaged.source_series_id()
        );
        assert_ne!(plain.source_series_id(), vintaged.source_series_id());
    }

    #[test]
    fn vintage_is_absent_unless_the_source_provides_it() {
        let observation = observation(EcbDataType::MoneyStock, EcbValue::Value(1.0));
        assert_eq!(observation.vintage, None, "无官方 vintage 面时必须为 None");
    }

    #[test]
    fn curve_observations_are_held_but_never_promoted() {
        for data_type in [EcbDataType::ZeroSpot, EcbDataType::Forward] {
            let observation = observation(data_type, EcbValue::Value(3.0));
            // 曲线点可以被持有（类型层保留了它）…
            assert!(validate_observation(&observation).is_ok());
            // …但不得晋级。
            let error = reject_curve_promotion(&observation).expect_err("曲线点不得晋级");
            assert_eq!(error.kind(), EcbErrorKind::RoutedElsewhere);
        }
        let non_curve = observation(EcbDataType::PolicyRate, EcbValue::Value(3.0));
        assert!(reject_curve_promotion(&non_curve).is_ok());
    }

    #[test]
    fn observation_with_invalid_series_is_rejected_at_construction() {
        let dataflow = EcbDataflowId::try_new(
            "YC",
            "DSD_SYNTH",
            vec![EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度")],
        )
        .expect("合法身份");
        let broken = EcbDataflowId {
            dataflow: dataflow.dataflow.clone(),
            dsd: dataflow.dsd.clone(),
            dimensions: dataflow.dimensions.clone(),
        };
        let identity = EcbSeriesIdentity {
            dataflow: broken,
            indicator: String::new(),
            subject: "SUBJ_SYNTH".to_owned(),
        };
        let result = EcbObservation::new(
            identity,
            EcbDataType::PolicyRate,
            Frequency::Daily,
            Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            EcbValue::Value(1.0),
            unit(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn invalid_public_periods_are_rejected() {
        for period in [
            crate::Period::Month {
                year: 2026,
                month: 99,
            },
            crate::Period::Quarter {
                year: 2026,
                quarter: 0,
            },
            crate::Period::Year(0),
        ] {
            assert!(EcbObservation::new(
                series(),
                EcbDataType::PolicyRate,
                crate::Frequency::Monthly,
                period,
                EcbValue::Value(1.0),
                unit(),
                None
            )
            .is_err());
        }
    }
}
