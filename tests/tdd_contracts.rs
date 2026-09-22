#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! TDD 行为契约（特性 005）。
//!
//! 入口集合 = 本 crate 全部公开入口（含 `validate*` / 判定函数 / 解析器）。
//!
//! 下表为每个入口**声明**的语义变异与应红 / 应绿用例（用例名均在文件内真实存在）。
//! 其中 **24 条**已在 `/tmp` 隔离副本（独立 `CARGO_TARGET_DIR`）上实测：变异后指定用例
//! 观测为红、原树同名用例观测为绿；实测清单与复现命令见 crate 汇报与 PR 描述。
//! **未实测的行是待执行声明**，不得据其声称已观测。
//!
//! // TDD-PROBE: EcbError::kind | 变异：把 Missing 映射为 Invalid | 红=error_kind_maps_every_variant_distinctly | 绿=error_kind_maps_every_variant_distinctly
//! // TDD-PROBE: EcbError::is_retryable | 变异：恒返回 true | 红=only_invariant_is_retryable | 绿=only_invariant_is_retryable
//! // TDD-PROBE: EcbDataType::ALL | 变异：只列 6 类 | 红=seven_data_types_are_listed_verbatim | 绿=seven_data_types_are_listed_verbatim
//! // TDD-PROBE: EcbDataType::code | 变异：POLICY_RATE 写成 POLICY | 红=seven_data_types_are_listed_verbatim | 绿=seven_data_types_are_listed_verbatim
//! // TDD-PROBE: EcbDataType::description | 变异：中文说明返回空串 | 红=seven_data_types_are_listed_verbatim | 绿=seven_data_types_are_listed_verbatim
//! // TDD-PROBE: EcbDataType::from_code | 变异：接受任意字符串 | 红=unknown_type_codes_are_rejected | 绿=unknown_type_codes_are_rejected
//! // TDD-PROBE: EcbDataType::is_curve_product | 变异：只认 ZeroSpot | 红=only_zero_spot_and_forward_are_curve_products | 绿=only_zero_spot_and_forward_are_curve_products
//! // TDD-PROBE: EcbProtocol::ALL | 变异：只列 2 个协议 | 红=protocol_names_are_names_only | 绿=protocol_names_are_names_only
//! // TDD-PROBE: EcbProtocol::name | 变异：返回带端点的字符串 | 红=protocol_names_are_names_only | 绿=protocol_names_are_names_only
//! // TDD-PROBE: EcbProtocol::from_name | 变异：大小写不敏感匹配 | 红=protocol_names_are_names_only | 绿=protocol_names_are_names_only
//! // TDD-PROBE: EcbDimension::try_new | 变异：允许空维度键 | 红=dimension_empty_parts_are_rejected | 绿=dimension_empty_parts_are_rejected
//! // TDD-PROBE: EcbDimension::key | 变异：用 `:` 连接键与值 | 红=dimension_key_shape_is_fixed | 绿=dimension_key_shape_is_fixed
//! // TDD-PROBE: EcbDataflowId::try_new | 变异：允许空维度列表 | 红=dataflow_id_requires_dimension_key | 绿=dataflow_id_requires_dimension_key
//! // TDD-PROBE: EcbDataflowId::declared_dimension_key | 变异：先排序再拼接 | 红=declared_order_is_preserved | 绿=declared_order_is_preserved
//! // TDD-PROBE: validate_dataflow_id | 变异：不检查重复维度键 | 红=duplicate_dimension_ids_are_rejected | 绿=duplicate_dimension_ids_are_rejected
//! // TDD-PROBE: EcbSeriesIdentity::try_new | 变异：允许空 indicator | 红=series_identity_requires_indicator_and_subject | 绿=series_identity_requires_indicator_and_subject
//! // TDD-PROBE: EcbSeriesIdentity::series_id | 变异：用 `/` 连接各段 | 红=series_id_has_fixed_shape | 绿=series_id_has_fixed_shape
//! // TDD-PROBE: validate_series_identity | 变异：跳过 indicator 检查 | 红=series_identity_requires_indicator_and_subject | 绿=series_identity_requires_indicator_and_subject
//! // TDD-PROBE: Unit::try_new | 变异：允许纯空白单位 | 红=unit_keeps_source_text | 绿=unit_keeps_source_text
//! // TDD-PROBE: Unit::text | 变异：返回大写化文本 | 红=unit_keeps_source_text | 绿=unit_keeps_source_text
//! // TDD-PROBE: EcbMissingReason::describe | 变异：Unspecified 返回空串 | 红=missing_reason_describe_is_never_empty | 绿=missing_reason_describe_is_never_empty
//! // TDD-PROBE: EcbValue::value | 变异：缺失时返回 Some(0.0) | 红=missing_value_is_never_zero | 绿=missing_value_is_never_zero
//! // TDD-PROBE: EcbValue::is_missing | 变异：恒返回 false | 红=missing_value_is_never_zero | 绿=missing_value_is_never_zero
//! // TDD-PROBE: EcbValue::missing_reason | 变异：有值时也返回 Some | 红=missing_value_is_never_zero | 绿=missing_value_is_never_zero
//! // TDD-PROBE: EcbObservation::new | 变异：跳过 validate_observation | 红=observation_new_validates | 绿=observation_new_validates
//! // TDD-PROBE: EcbObservation::source_series_id | 变异：丢掉 period 段 | 红=source_series_id_contains_period_and_vintage | 绿=source_series_id_contains_period_and_vintage
//! // TDD-PROBE: validate_observation | 变异：忽略空的源侧状态原文 | 红=empty_source_status_is_rejected | 绿=empty_source_status_is_rejected
//! // TDD-PROBE: Date::new | 变异：不校验闰年 2 月 | 红=date_rejects_impossible_days | 绿=date_rejects_impossible_days
//! // TDD-PROBE: Date::parse | 变异：接受未补零形式 | 红=strict_iso_only | 绿=strict_iso_only
//! // TDD-PROBE: Date::to_iso | 变异：月日不补零 | 红=date_round_trips_through_iso | 绿=date_round_trips_through_iso
//! // TDD-PROBE: Date::year/month/day | 变异：交换 month 与 day | 红=date_round_trips_through_iso | 绿=date_round_trips_through_iso
//! // TDD-PROBE: Date::is_leap_year | 变异：忽略百年不闰规则 | 红=leap_year_century_rules | 绿=leap_year_century_rules
//! // TDD-PROBE: Date::days_in_month | 变异：2 月恒为 28 天 | 红=leap_year_century_rules | 绿=leap_year_century_rules
//! // TDD-PROBE: Period::day/month/quarter/year/event | 变异：构造期不校验范围 | 红=period_constructors_validate_ranges | 绿=period_constructors_validate_ranges
//! // TDD-PROBE: Period::parse | 变异：接受 2026-13 月期间 | 红=period_parse_rejects_malformed | 绿=period_parse_rejects_malformed
//! // TDD-PROBE: Period::key | 变异：季度输出 2026-3 | 红=period_key_shapes_are_fixed | 绿=period_key_shapes_are_fixed
//! // TDD-PROBE: Period::frequency | 变异：Event 映射为 Daily | 红=period_frequency_mapping | 绿=period_frequency_mapping
//! // TDD-PROBE: Frequency::ALL | 变异：只列 6 个取值 | 红=frequency_names_round_trip | 绿=frequency_names_round_trip
//! // TDD-PROBE: Frequency::name | 变异：返回中文名 | 红=frequency_names_round_trip | 绿=frequency_names_round_trip
//! // TDD-PROBE: Frequency::from_name | 变异：大小写不敏感 | 红=frequency_names_round_trip | 绿=frequency_names_round_trip
//! // TDD-PROBE: ensure_not_curve_product | 变异：曲线点返回 Ok | 红=curve_products_are_routed_elsewhere | 绿=curve_products_are_routed_elsewhere
//! // TDD-PROBE: reject_curve_promotion | 变异：曲线点返回 Ok | 红=curve_observation_is_not_promotable | 绿=curve_observation_is_not_promotable
//! // TDD-PROBE: attest_official_dimension_order | 变异：返回 Ok | 红=official_order_attestation_always_refused | 绿=official_order_attestation_always_refused
//! // TDD-PROBE: claim_fiscal_write_authority | 变异：返回 Ok | 红=fiscal_write_authority_is_refused | 绿=fiscal_write_authority_is_refused
//! // TDD-PROBE: EcbAuthorization::is_authorized | 变异：Denied 也返回 true | 红=denied_verdicts_are_never_authorized | 绿=denied_verdicts_are_never_authorized
//! // TDD-PROBE: EcbAuthorization::denial_reason | 变异：理由返回空串 | 红=denied_verdicts_are_never_authorized | 绿=denied_verdicts_are_never_authorized
//! // TDD-PROBE: EcbAuthorizationEvidence::new | 变异：丢弃 signer | 红=evidence_fields_are_preserved | 绿=evidence_fields_are_preserved
//! // TDD-PROBE: decide_authorization | 变异：证据缺失时放行 | 红=evidence_absence_is_denied | 绿=evidence_absence_is_denied
//! // TDD-PROBE: registered_evidence | 变异：返回合成证据 | 红=registered_evidence_is_absent | 绿=registered_evidence_is_absent
//! // TDD-PROBE: current_authorization | 变异：恒返回 Authorized | 红=current_authorization_is_denied | 绿=current_authorization_is_denied
//! // TDD-PROBE: ensure_authorized | 变异：Denied 返回 Ok | 红=ensure_authorized_maps_denial | 绿=ensure_authorized_maps_denial
//! // TDD-PROBE: publication_semantics | 变异：返回 Formal | 红=publication_triple_is_fixed | 绿=publication_triple_is_fixed
//! // TDD-PROBE: publication_for_period | 变异：改写传入的期间 | 红=publication_period_is_echoed | 绿=publication_period_is_echoed
//! // TDD-PROBE: parse_ecb_observations | 变异：忽略未知字段 | 红=parser_rejects_unknown_fields | 绿=parser_rejects_unknown_fields

use ecbx::{
    attest_official_dimension_order, claim_fiscal_write_authority, current_authorization,
    decide_authorization, ensure_authorized, ensure_not_curve_product, parse_ecb_observations,
    publication_for_period, publication_semantics, registered_evidence, reject_curve_promotion,
    validate_dataflow_id, validate_observation, validate_series_identity, AvailabilityEvidence,
    Date, EcbAuthorization, EcbAuthorizationEvidence, EcbDataType, EcbDataflowId, EcbDimension,
    EcbError, EcbErrorKind, EcbMissingReason, EcbObservation, EcbProtocol, EcbSeriesIdentity,
    EcbValue, Frequency, Period, PitEligibility, TimePrecision, Unit,
};

fn unit() -> Unit {
    Unit::try_new("SYNTH_UNIT").expect("合法单位")
}

fn series() -> EcbSeriesIdentity {
    let dataflow = EcbDataflowId::try_new(
        "YC",
        "DSD_SYNTH",
        vec![
            EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
            EcbDimension::try_new("DIM_B", "VAL_B").expect("合法维度"),
        ],
    )
    .expect("合法身份");
    EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", "SUBJ_SYNTH").expect("合法序列身份")
}

fn day() -> Date {
    Date::new(2026, 9, 18).expect("合法日期")
}

fn observation(value: EcbValue) -> EcbObservation {
    EcbObservation::new(
        series(),
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::day(day()),
        value,
        unit(),
        None,
    )
    .expect("合法观测")
}

fn every_error() -> Vec<EcbError> {
    vec![
        EcbError::Invalid("x".to_owned()),
        EcbError::Missing("x".to_owned()),
        EcbError::AuthorizationDenied("x".to_owned()),
        EcbError::RoutedElsewhere("x".to_owned()),
        EcbError::WriteAuthorityDenied("x".to_owned()),
        EcbError::SemanticallyRejected("x".to_owned()),
        EcbError::NotApplicable("x".to_owned()),
        EcbError::Invariant("x".to_owned()),
    ]
}

#[test]
fn error_kind_maps_every_variant_distinctly() {
    let kinds: Vec<EcbErrorKind> = every_error().iter().map(EcbError::kind).collect();
    assert_eq!(
        kinds,
        [
            EcbErrorKind::Invalid,
            EcbErrorKind::Missing,
            EcbErrorKind::AuthorizationDenied,
            EcbErrorKind::RoutedElsewhere,
            EcbErrorKind::WriteAuthorityDenied,
            EcbErrorKind::SemanticallyRejected,
            EcbErrorKind::NotApplicable,
            EcbErrorKind::Invariant,
        ]
    );
}

#[test]
fn only_invariant_is_retryable() {
    for error in every_error() {
        assert_eq!(
            error.is_retryable(),
            error.kind() == EcbErrorKind::Invariant,
            "{error:?}"
        );
    }
}

#[test]
fn seven_data_types_are_listed_verbatim() {
    let codes: Vec<&str> = EcbDataType::ALL.iter().map(EcbDataType::code).collect();
    assert_eq!(
        codes,
        [
            "ZERO_SPOT",
            "FORWARD",
            "POLICY_RATE",
            "RFR_OVERNIGHT",
            "CB_BALANCE_SHEET",
            "MONEY_STOCK",
            "DEPOSIT_FACILITY"
        ]
    );
    for data_type in EcbDataType::ALL {
        assert!(!data_type.description().is_empty(), "{data_type}");
    }
}

#[test]
fn unknown_type_codes_are_rejected() {
    for code in ["YIELD_CURVE", "zero_spot", "", "POLICY"] {
        let error = EcbDataType::from_code(code).expect_err(code);
        assert_eq!(error.kind(), EcbErrorKind::Invalid, "{code}");
    }
}

#[test]
fn only_zero_spot_and_forward_are_curve_products() {
    let curves: Vec<&str> = EcbDataType::ALL
        .iter()
        .filter(|data_type| data_type.is_curve_product())
        .map(EcbDataType::code)
        .collect();
    assert_eq!(curves, ["ZERO_SPOT", "FORWARD"]);
}

#[test]
fn protocol_names_are_names_only() {
    let names: Vec<&str> = EcbProtocol::ALL.iter().map(EcbProtocol::name).collect();
    assert_eq!(names, ["SDMX-JSON", "SDMX-ML", "CSV"]);
    for protocol in EcbProtocol::ALL {
        assert_eq!(
            EcbProtocol::from_name(protocol.name()).expect("可回读"),
            protocol
        );
        assert!(!protocol.name().contains('/'), "{protocol:?}");
    }
    assert!(EcbProtocol::from_name("sdmx-json").is_err());
}

#[test]
fn dimension_empty_parts_are_rejected() {
    assert!(EcbDimension::try_new("", "VAL_A").is_err());
    assert!(EcbDimension::try_new("   ", "VAL_A").is_err());
    assert!(EcbDimension::try_new("DIM_A", "").is_err());
    assert!(EcbDimension::try_new("DIM_A", "  ").is_err());
}

#[test]
fn dimension_key_shape_is_fixed() {
    let dimension = EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度");
    assert_eq!(dimension.key(), "DIM_A=VAL_A");
}

#[test]
fn dataflow_id_requires_dimension_key() {
    assert!(EcbDataflowId::try_new("YC", "DSD_SYNTH", Vec::new()).is_err());
    assert!(EcbDataflowId::try_new(
        "",
        "DSD_SYNTH",
        vec![EcbDimension::try_new("D", "V").unwrap()]
    )
    .is_err());
    assert!(
        EcbDataflowId::try_new("YC", " ", vec![EcbDimension::try_new("D", "V").unwrap()]).is_err()
    );
}

#[test]
fn declared_order_is_preserved() {
    let forward = EcbDataflowId::try_new(
        "YC",
        "DSD_SYNTH",
        vec![
            EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
            EcbDimension::try_new("DIM_B", "VAL_B").expect("合法维度"),
        ],
    )
    .expect("合法身份");
    assert_eq!(forward.declared_dimension_key(), "DIM_A=VAL_A.DIM_B=VAL_B");

    let reversed = EcbDataflowId::try_new(
        "YC",
        "DSD_SYNTH",
        vec![
            EcbDimension::try_new("DIM_B", "VAL_B").expect("合法维度"),
            EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
        ],
    )
    .expect("合法身份");
    assert_eq!(reversed.declared_dimension_key(), "DIM_B=VAL_B.DIM_A=VAL_A");
}

#[test]
fn duplicate_dimension_ids_are_rejected() {
    let duplicated = vec![
        EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
        EcbDimension::try_new("DIM_A", "VAL_B").expect("合法维度"),
    ];
    let error = EcbDataflowId::try_new("YC", "DSD_SYNTH", duplicated).expect_err("重复维度键");
    assert_eq!(error.kind(), EcbErrorKind::SemanticallyRejected);
}

#[test]
fn series_identity_requires_indicator_and_subject() {
    let dataflow = EcbDataflowId::try_new(
        "YC",
        "DSD_SYNTH",
        vec![EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度")],
    )
    .expect("合法身份");
    assert!(EcbSeriesIdentity::try_new(dataflow.clone(), "", "SUBJ_SYNTH").is_err());
    assert!(EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", "  ").is_err());
    assert!(validate_series_identity(&series()).is_ok());
    assert!(validate_dataflow_id(&series().dataflow).is_ok());
}

#[test]
fn series_id_has_fixed_shape() {
    assert_eq!(
        series().series_id(),
        "YC:DSD_SYNTH:DIM_A=VAL_A.DIM_B=VAL_B:IND_SYNTH:SUBJ_SYNTH"
    );
}

#[test]
fn unit_keeps_source_text() {
    let unit = Unit::try_new(" 100 million yen ").expect("合法单位");
    assert_eq!(unit.text(), "100 million yen");
    assert_eq!(unit.to_string(), "100 million yen");
    assert!(Unit::try_new("").is_err());
    assert!(Unit::try_new("   ").is_err());
}

#[test]
fn missing_reason_describe_is_never_empty() {
    assert_eq!(
        EcbMissingReason::SourceStatus("SYNTH_STATUS".to_owned()).describe(),
        "SYNTH_STATUS"
    );
    assert!(!EcbMissingReason::Unspecified.describe().is_empty());
}

#[test]
fn missing_value_is_never_zero() {
    let present = observation(EcbValue::Value(3.25));
    assert_eq!(present.value.value(), Some(3.25));
    assert!(!present.value.is_missing());
    assert_eq!(present.value.missing_reason(), None);

    let absent = observation(EcbValue::Missing(EcbMissingReason::Unspecified));
    assert_eq!(absent.value.value(), None, "缺失不得转 0");
    assert!(absent.value.is_missing());
    assert!(absent.value.missing_reason().is_some());
}

#[test]
fn observation_new_validates() {
    let built = EcbObservation::new(
        series(),
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::day(day()),
        EcbValue::Value(1.0),
        unit(),
        None,
    );
    assert!(built.is_ok());

    let broken = EcbObservation::new(
        series(),
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::day(day()),
        EcbValue::Missing(EcbMissingReason::SourceStatus("  ".to_owned())),
        unit(),
        None,
    );
    assert!(broken.is_err(), "构造期必须校验");
}

#[test]
fn source_series_id_contains_period_and_vintage() {
    let plain = observation(EcbValue::Value(1.0));
    assert_eq!(
        plain.source_series_id(),
        "YC:DSD_SYNTH:DIM_A=VAL_A.DIM_B=VAL_B:IND_SYNTH:SUBJ_SYNTH:2026-09-18:none"
    );

    let vintaged = EcbObservation::new(
        series(),
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::day(day()),
        EcbValue::Value(1.0),
        unit(),
        Some(Date::new(2026, 9, 20).expect("合法日期")),
    )
    .expect("合法观测");
    assert!(vintaged.source_series_id().ends_with(":2026-09-20"));
    assert_ne!(plain.source_series_id(), vintaged.source_series_id());
}

#[test]
fn empty_source_status_is_rejected() {
    assert!(validate_observation(&observation(EcbValue::Value(1.0))).is_ok());
    // 具名缺失必须带非空原因：构造器与校验入口结论一致。
    let broken = EcbObservation::new(
        series(),
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::day(day()),
        EcbValue::Missing(EcbMissingReason::SourceStatus(String::new())),
        unit(),
        None,
    )
    .expect_err("空状态原文必须拒绝");
    assert_eq!(broken.kind(), EcbErrorKind::SemanticallyRejected);
}

#[test]
fn date_rejects_impossible_days() {
    assert!(Date::new(2026, 2, 29).is_err());
    assert!(Date::new(2026, 4, 31).is_err());
    assert!(Date::new(2026, 0, 1).is_err());
    assert!(Date::new(2026, 13, 1).is_err());
    assert!(Date::new(2026, 1, 0).is_err());
    assert!(Date::new(2024, 2, 29).is_ok());
}

#[test]
fn strict_iso_only() {
    for input in [
        "2026-2-3",
        "2026/02/03",
        "2026-09-18T00:00:00Z",
        "2026-09-18 00:00:00",
        "26-09-18",
        "",
    ] {
        assert!(Date::parse(input).is_err(), "{input}");
    }
    assert!(Date::parse("2026-09-18").is_ok());
}

#[test]
fn date_round_trips_through_iso() {
    let date = Date::parse("2026-09-08").expect("合法日期");
    assert_eq!((date.year(), date.month(), date.day()), (2026, 9, 8));
    assert_eq!(date.to_iso(), "2026-09-08", "月日必须补零");
    assert_eq!(Date::parse(&date.to_iso()).expect("可回读"), date);
}

#[test]
fn leap_year_century_rules() {
    assert!(Date::is_leap_year(2024));
    assert!(!Date::is_leap_year(1900));
    assert!(Date::is_leap_year(2000));
    assert_eq!(Date::days_in_month(2024, 2).expect("2 月"), 29);
    assert_eq!(Date::days_in_month(2026, 2).expect("2 月"), 28);
    assert_eq!(Date::days_in_month(2026, 4).expect("4 月"), 30);
    assert_eq!(Date::days_in_month(2026, 12).expect("12 月"), 31);
    assert!(Date::days_in_month(2026, 13).is_err());
}

#[test]
fn period_constructors_validate_ranges() {
    assert!(Period::month(2026, 0).is_err());
    assert!(Period::month(2026, 13).is_err());
    assert!(Period::quarter(2026, 0).is_err());
    assert!(Period::quarter(2026, 5).is_err());
    assert!(Period::year(0).is_err());
    assert!(Period::month(2026, 9).is_ok());
    assert!(Period::quarter(2026, 3).is_ok());
    assert!(Period::year(2026).is_ok());
}

#[test]
fn period_parse_rejects_malformed() {
    for input in [
        "2026-2-3",
        "2026/09/18",
        "2026-13",
        "2026-Q5",
        "2026-Q0",
        "26",
        "",
    ] {
        assert!(Period::parse(input).is_err(), "{input}");
    }
    for input in [
        "2026-09-18",
        "2026-09",
        "2026-Q3",
        "2026",
        "event:2026-09-18",
    ] {
        assert!(Period::parse(input).is_ok(), "{input}");
    }
}

#[test]
fn period_key_shapes_are_fixed() {
    assert_eq!(Period::day(day()).key(), "2026-09-18");
    assert_eq!(Period::month(2026, 9).expect("合法").key(), "2026-09");
    assert_eq!(Period::quarter(2026, 3).expect("合法").key(), "2026-Q3");
    assert_eq!(Period::year(2026).expect("合法").key(), "2026");
    assert_eq!(Period::event(day()).key(), "event:2026-09-18");
    for period in [
        Period::day(day()),
        Period::month(2026, 9).expect("合法"),
        Period::quarter(2026, 3).expect("合法"),
        Period::year(2026).expect("合法"),
        Period::event(day()),
    ] {
        assert_eq!(Period::parse(&period.key()).expect("可回读"), period);
    }
}

#[test]
fn period_frequency_mapping() {
    assert_eq!(Period::day(day()).frequency(), Frequency::Daily);
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
    assert_eq!(Period::event(day()).frequency(), Frequency::Event);
}

#[test]
fn frequency_names_round_trip() {
    let names: Vec<&str> = Frequency::ALL.iter().map(Frequency::name).collect();
    assert_eq!(
        names,
        [
            "Daily",
            "Weekly",
            "Monthly",
            "Quarterly",
            "Annual",
            "Event",
            "Irregular"
        ]
    );
    for frequency in Frequency::ALL {
        assert_eq!(
            Frequency::from_name(frequency.name()).expect("可回读"),
            frequency
        );
    }
    assert!(Frequency::from_name("daily").is_err());
}

#[test]
fn curve_products_are_routed_elsewhere() {
    for data_type in [EcbDataType::ZeroSpot, EcbDataType::Forward] {
        let error = ensure_not_curve_product(&data_type).expect_err("曲线点必须路由");
        assert_eq!(error.kind(), EcbErrorKind::RoutedElsewhere);
        assert!(!error.is_retryable());
    }
    for data_type in [
        EcbDataType::PolicyRate,
        EcbDataType::RfrOvernight,
        EcbDataType::CbBalanceSheet,
        EcbDataType::MoneyStock,
        EcbDataType::DepositFacility,
    ] {
        assert!(ensure_not_curve_product(&data_type).is_ok(), "{data_type}");
    }
}

#[test]
fn curve_observation_is_not_promotable() {
    let curve = EcbObservation::new(
        series(),
        EcbDataType::ZeroSpot,
        Frequency::Daily,
        Period::day(day()),
        EcbValue::Value(3.0),
        unit(),
        None,
    )
    .expect("曲线点可以被持有");
    assert!(validate_observation(&curve).is_ok());
    assert_eq!(
        reject_curve_promotion(&curve)
            .expect_err("曲线点不得晋级")
            .kind(),
        EcbErrorKind::RoutedElsewhere
    );
    assert!(reject_curve_promotion(&observation(EcbValue::Value(1.0))).is_ok());
}

#[test]
fn official_order_attestation_always_refused() {
    let dimensions = vec![EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度")];
    for input in [dimensions.as_slice(), &[]] {
        let error = attest_official_dimension_order(input).expect_err("不得冒充官方顺序");
        assert_eq!(error.kind(), EcbErrorKind::NotApplicable);
    }
}

#[test]
fn fiscal_write_authority_is_refused() {
    let error = claim_fiscal_write_authority().expect_err("pending 主权不得主张");
    assert_eq!(error.kind(), EcbErrorKind::WriteAuthorityDenied);
}

#[test]
fn denied_verdicts_are_never_authorized() {
    let verdict = EcbAuthorization::Denied {
        reason: "无 Owner 签核文件".to_owned(),
    };
    assert!(!verdict.is_authorized());
    assert!(verdict
        .denial_reason()
        .is_some_and(|reason| !reason.trim().is_empty()));
}

#[test]
fn evidence_fields_are_preserved() {
    // 合成证据只用于验证判定形状，不构成任何真实授权证据。
    let evidence = EcbAuthorizationEvidence::new(
        "synthetic-evidence-ref",
        Some("synthetic-signer"),
        Some("offline_parse_and_types"),
        Some(Date::new(2030, 1, 1).expect("合法日期")),
    );
    let verdict = decide_authorization(Some(&evidence), day());
    assert!(verdict.is_authorized());
    assert_eq!(verdict.denial_reason(), None);
    match verdict {
        EcbAuthorization::Authorized { scope } => assert_eq!(scope, "offline_parse_and_types"),
        other => unreachable!("已断言放行：{other:?}"),
    }
}

#[test]
fn evidence_absence_is_denied() {
    assert!(!decide_authorization(None, day()).is_authorized());

    let no_signer = EcbAuthorizationEvidence::new(
        "synthetic-evidence-ref",
        None,
        Some("offline_parse_and_types"),
        Some(Date::new(2030, 1, 1).expect("合法日期")),
    );
    assert!(!decide_authorization(Some(&no_signer), day()).is_authorized());

    let no_scope = EcbAuthorizationEvidence::new(
        "synthetic-evidence-ref",
        Some("synthetic-signer"),
        None,
        Some(Date::new(2030, 1, 1).expect("合法日期")),
    );
    assert!(!decide_authorization(Some(&no_scope), day()).is_authorized());

    let expired = EcbAuthorizationEvidence::new(
        "synthetic-evidence-ref",
        Some("synthetic-signer"),
        Some("offline_parse_and_types"),
        Some(Date::new(2026, 1, 1).expect("合法日期")),
    );
    assert!(!decide_authorization(Some(&expired), day()).is_authorized());
}

#[test]
fn registered_evidence_is_absent() {
    assert!(registered_evidence().is_none(), "本源不得凭空构造证据");
}

#[test]
fn current_authorization_is_denied() {
    let verdict = current_authorization(day());
    assert!(!verdict.is_authorized());
    assert!(verdict
        .denial_reason()
        .is_some_and(|reason| !reason.trim().is_empty()));
}

#[test]
fn ensure_authorized_maps_denial() {
    let error = ensure_authorized(&current_authorization(day())).expect_err("unknown 授权必须拒绝");
    assert_eq!(error.kind(), EcbErrorKind::AuthorizationDenied);
    assert!(!error.is_retryable());

    let allowed = EcbAuthorization::Authorized {
        scope: "synthetic-scope".to_owned(),
    };
    assert!(ensure_authorized(&allowed).is_ok());
}

#[test]
fn publication_triple_is_fixed() {
    assert_eq!(
        publication_semantics(),
        (
            TimePrecision::Date,
            AvailabilityEvidence::Inferred,
            PitEligibility::NotEligible
        )
    );
    assert_ne!(publication_semantics().2, PitEligibility::Formal);
}

#[test]
fn publication_period_is_echoed() {
    let (echoed, precision, evidence, eligibility) = publication_for_period(day());
    assert_eq!(echoed, day());
    assert_eq!(precision, TimePrecision::Date);
    assert_eq!(evidence, AvailabilityEvidence::Inferred);
    assert_eq!(eligibility, PitEligibility::NotEligible);
}

#[test]
fn parser_rejects_unknown_fields() {
    let good = include_str!("fixtures/ecb_policy_observations.json");
    assert_eq!(
        parse_ecb_observations(good)
            .expect("合成夹具必须可解析")
            .len(),
        3
    );

    let with_unknown = good.replace(
        "\"vintage_date\": null",
        "\"vintage_date\": null, \"endpoint\": \"nope\"",
    );
    let error = parse_ecb_observations(&with_unknown).expect_err("未知字段必须原子失败");
    assert_eq!(error.kind(), EcbErrorKind::Invalid);
}

#[test]
fn curve_fixture_is_parsed_but_never_promoted() {
    let document = include_str!("fixtures/ecb_curve_observations.json");
    let observations = parse_ecb_observations(document).expect("合成夹具必须可解析");
    assert_eq!(observations.len(), 2);
    for observation in &observations {
        assert!(observation.data_type.is_curve_product());
        assert!(reject_curve_promotion(observation).is_err());
    }
}
