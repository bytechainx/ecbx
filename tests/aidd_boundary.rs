#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! AIDD 对抗 / 边界用例（特性 005）。
//!
//! 候选由 AI 生成，逐条人工复核后仅保留「结论=保留」项；丢弃项登记于 PR 描述。
//!
//! // AIDD: 空类型码 from_code("") | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §1 类型码只认 7 个字面值 | 结论=保留
//! // AIDD: 世纪闰年边界 1900-02-29 / 2000-02-29 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 严格日期解析 | 结论=保留
//! // AIDD: 季度取 0 与 5 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 期间形态一律拒绝 | 结论=保留
//! // AIDD: 超长维度键（10000 字符） | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 身份形不得 panic | 结论=保留
//! // AIDD: 大小写不同的维度键（DIM_A / dim_a） | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 重复判定按字面 | 结论=保留
//! // AIDD: 同身份不同 vintage 是否算重复 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 身份含 vintage | 结论=保留
//! // AIDD: 非 ASCII 主体与单位文本 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 单位原样保留 | 结论=保留
//! // AIDD: 极大与极小 f64（±1e308） | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 值原样保留 | 结论=保留
//! // AIDD: _synthetic=false 的输入 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 只接受合成标注输入 | 结论=保留

use ecbx::{
    parse_ecb_observations, EcbDataType, EcbDataflowId, EcbDimension, EcbErrorKind, EcbObservation,
    EcbSeriesIdentity, EcbValue, Frequency, Period, Unit,
};

fn unit() -> Unit {
    Unit::try_new("SYNTH_UNIT").expect("合法单位")
}

fn day() -> ecbx::Date {
    ecbx::Date::new(2026, 9, 18).expect("合法日期")
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

/// 边界：空类型码必须被拒绝，且分类为 `Invalid`（不是 panic）。
#[test]
fn aidd_empty_type_code_is_rejected() {
    for code in ["", " ", "  ZERO_SPOT"] {
        let error = EcbDataType::from_code(code).expect_err(code);
        assert_eq!(error.kind(), EcbErrorKind::Invalid, "{code:?}");
    }
}

/// 边界：世纪闰年规则——1900 不是闰年，2000 是。
#[test]
fn aidd_century_leap_year_boundary() {
    assert!(ecbx::Date::new(1900, 2, 29).is_err());
    assert!(ecbx::Date::new(2000, 2, 29).is_ok());
    assert!(ecbx::Date::new(2100, 2, 29).is_err());
    assert!(ecbx::Date::new(2024, 2, 29).is_ok());
}

/// 边界：季度 0 与 5 必须拒绝，且 `Period::parse` 不接受越界季度。
#[test]
fn aidd_quarter_bounds_are_rejected() {
    assert!(Period::quarter(2026, 0).is_err());
    assert!(Period::quarter(2026, 5).is_err());
    assert!(Period::parse("2026-Q0").is_err());
    assert!(Period::parse("2026-Q5").is_err());
    assert!(Period::parse("2026-Q4").is_ok());
}

/// 边界：超长维度键不得 panic，且仍是合法身份。
#[test]
fn aidd_extremely_long_dimension_key() {
    let long_id = "D".repeat(10_000);
    let long_value = "V".repeat(10_000);
    let dimension = EcbDimension::try_new(&long_id, &long_value).expect("超长维度仍合法");
    let dataflow = EcbDataflowId::try_new("YC", "DSD_SYNTH", vec![dimension]).expect("合法身份");
    let key = dataflow.declared_dimension_key();
    assert_eq!(key.len(), 20_001, "键 = 10000 + '=' + 10000");
    assert!(key.starts_with('D') && key.ends_with('V'));
}

/// 边界：重复判定按字面——大小写不同的维度键不是重复。
#[test]
fn aidd_dimension_id_case_is_significant() {
    let mixed = vec![
        EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
        EcbDimension::try_new("dim_a", "VAL_B").expect("合法维度"),
    ];
    let dataflow = EcbDataflowId::try_new("YC", "DSD_SYNTH", mixed).expect("大小写不同不算重复");
    assert_eq!(dataflow.declared_dimension_key(), "DIM_A=VAL_A.dim_a=VAL_B");
}

/// 边界：身份含 vintage，故「同序列同期间、不同 vintage」不是重复。
#[test]
fn aidd_same_series_different_vintage_is_not_duplicate() {
    let document = r#"{ "_synthetic": true, "_note": "合成样本",
        "observations": [
          { "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ { "id": "A", "value": "B" } ],
            "indicator": "I", "subject": "S", "data_type": "POLICY_RATE", "freq": "Daily",
            "period": "2026-09-18", "value": 1.0, "unit": "U" },
          { "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ { "id": "A", "value": "B" } ],
            "indicator": "I", "subject": "S", "data_type": "POLICY_RATE", "freq": "Daily",
            "period": "2026-09-18", "value": 1.0, "unit": "U", "vintage_date": "2026-09-20" }
        ] }"#;
    let observations = parse_ecb_observations(document).expect("不同 vintage 不算重复");
    assert_eq!(observations.len(), 2);
    assert_ne!(
        observations[0].source_series_id(),
        observations[1].source_series_id()
    );
}

/// 边界：非 ASCII 主体与单位文本原样保留，不被改写。
#[test]
fn aidd_non_ascii_subject_and_unit_are_preserved() {
    let unit = Unit::try_new("  億円 (100 million yen)  ").expect("合法单位");
    assert_eq!(unit.text(), "億円 (100 million yen)");

    let dataflow = EcbDataflowId::try_new(
        "YC",
        "DSD_SYNTH",
        vec![EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度")],
    )
    .expect("合法身份");
    let identity = EcbSeriesIdentity::try_new(dataflow, "指標", "主体").expect("非 ASCII 合法");
    let observation = EcbObservation::new(
        identity,
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::day(day()),
        EcbValue::Value(1.0),
        unit,
        None,
    )
    .expect("合法观测");
    assert!(observation.source_series_id().contains("指標"));
    assert_eq!(observation.unit.text(), "億円 (100 million yen)");
}

/// 边界：极大与极小 f64 原样保留，不被截断或改写。
#[test]
fn aidd_extreme_f64_values_are_preserved() {
    for value in [f64::MAX, f64::MIN, 0.0, -0.0] {
        let observation = EcbObservation::new(
            series(),
            EcbDataType::PolicyRate,
            Frequency::Daily,
            Period::day(day()),
            EcbValue::Value(value),
            unit(),
            None,
        )
        .expect("合法观测");
        assert_eq!(observation.value.value(), Some(value));
    }
}

/// 边界：显式声明非合成的输入必须被拒绝（不得当作源数据）。
#[test]
fn aidd_non_synthetic_input_is_refused() {
    let document = r#"{ "_synthetic": false, "_note": "x", "observations": [] }"#;
    assert_eq!(
        parse_ecb_observations(document)
            .expect_err("不得把非合成输入当源数据")
            .kind(),
        EcbErrorKind::SemanticallyRejected
    );
}
