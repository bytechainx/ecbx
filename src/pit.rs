//! publication 语义：时间精度 + 可得性证据层 + 正式 PIT 资格。
//!
//! 本源的三个取值**固定**为 `Date` / `Inferred` / `NotEligible`，且不由调用方决定。
//! 该三元组的判据见 `specs/features/005-macro-data-source-crates/contracts/source-library-contract.md` §5。

use crate::value::Date;

/// 时间精度：源只给日期还是给出时刻。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePrecision {
    /// 只有日期。
    Date,
    /// 有明确时刻。
    Instant,
}

/// 可得性证据层：官方字段 > 日历 > 推断。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvailabilityEvidence {
    /// 官方字段。
    Official,
    /// 发布日历。
    Calendar,
    /// 推断。
    Inferred,
}

/// 正式 PIT 资格。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PitEligibility {
    /// 可进正式 PIT。
    Formal,
    /// 不可进正式 PIT。
    NotEligible,
}

/// 本源的 publication 语义三元组。
///
/// 返回恒为 `(Date, Inferred, NotEligible)`：
/// 本层不认识官方 `vintage` 面、也不持有发布日历，故只能是推断层且不具正式 PIT 资格。
/// **禁止**补造 `00:00 UTC` 之类的时刻把 `Date` 伪装成 `Instant`。
#[must_use]
pub fn publication_semantics() -> (TimePrecision, AvailabilityEvidence, PitEligibility) {
    (
        TimePrecision::Date,
        AvailabilityEvidence::Inferred,
        PitEligibility::NotEligible,
    )
}

/// 与该源事实条目的观测期间一同返回的可得性三元组。
///
/// `period` 只用于证明「本判定与期间无关」，不参与判定；返回值恒为
/// `(period, Date, Inferred, NotEligible)`。
#[must_use]
pub fn publication_for_period(
    period: Date,
) -> (Date, TimePrecision, AvailabilityEvidence, PitEligibility) {
    let (precision, evidence, eligibility) = publication_semantics();
    (period, precision, evidence, eligibility)
}

#[cfg(test)]
mod tests {
    use super::{
        publication_for_period, publication_semantics, AvailabilityEvidence, PitEligibility,
        TimePrecision,
    };
    use crate::value::Date;

    #[test]
    fn semantics_triple_is_fixed() {
        assert_eq!(
            publication_semantics(),
            (
                TimePrecision::Date,
                AvailabilityEvidence::Inferred,
                PitEligibility::NotEligible
            )
        );
    }

    #[test]
    fn formal_pit_is_never_eligible() {
        assert_ne!(
            publication_semantics().2,
            PitEligibility::Formal,
            "正式 PIT 资格必须为 NotEligible"
        );
    }

    #[test]
    fn period_is_passed_through_without_promotion() {
        let period = Date::new(2026, 9, 18).expect("合法日期");
        let (echoed, precision, evidence, eligibility) = publication_for_period(period);
        assert_eq!(echoed, period);
        assert_eq!(precision, TimePrecision::Date);
        assert_eq!(evidence, AvailabilityEvidence::Inferred);
        assert_eq!(eligibility, PitEligibility::NotEligible);
    }
}
