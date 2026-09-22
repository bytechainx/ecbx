//! ecbx 值对象门面。
//!
//! 子模块按职责拆分（日期 / 期间 / 身份形 / 观测），依赖方向单向：
//! `error ← value.*`，各子模块之间不构成环。

mod date;
mod observation;
mod period;
mod series;

pub use date::Date;
pub use observation::{
    reject_curve_promotion, validate_observation, EcbMissingReason, EcbObservation, EcbValue, Unit,
};
pub use period::{Frequency, Period};
pub use series::{
    attest_official_dimension_order, claim_fiscal_write_authority, ensure_not_curve_product,
    validate_dataflow_id, validate_series_identity, EcbDataType, EcbDataflowId, EcbDimension,
    EcbProtocol, EcbSeriesIdentity,
};

#[cfg(test)]
mod tests {
    use super::{EcbDataType, EcbDataflowId, EcbDimension, EcbProtocol, Frequency, Period, Unit};
    use crate::error::EcbErrorKind;

    #[test]
    fn facade_reexports_are_usable_together() {
        let dataflow = EcbDataflowId::try_new(
            "BSI",
            "DSD_SYNTH",
            vec![EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度")],
        )
        .expect("合法身份");
        assert_eq!(dataflow.declared_dimension_key(), "DIM_A=VAL_A");
        assert_eq!(EcbProtocol::ALL.len(), 3);
        assert_eq!(EcbDataType::ALL.len(), 7);
        assert_eq!(
            Frequency::from_name("Monthly").expect("合法频率"),
            Frequency::Monthly
        );
        assert_eq!(Period::parse("2026-09").expect("合法期间").key(), "2026-09");
        assert!(Unit::try_new("SYNTH_UNIT").is_ok());
        assert_eq!(
            EcbDataType::from_code("NOPE").expect_err("未知类型").kind(),
            EcbErrorKind::Invalid
        );
    }
}
