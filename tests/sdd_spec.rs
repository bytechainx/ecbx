#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! SDD 规格对照（特性 005）：把 `docs/标准.md` 的每个 `##` 章节转成可执行断言。
//!
//! 章节与断言函数须与 `docs/标准.md` 的 `##` 章节 1:1（检查器按标题逐字比对）。
//!
//! // SPEC-MAP: S-1 | 1. 数据类型标准 | assert_data_type_standard
//! // SPEC-MAP: S-2 | 2. 身份形与维度顺序标准 | assert_identity_and_declared_order_standard
//! // SPEC-MAP: S-3 | 3. 曲线路由标准 | assert_curve_routing_standard
//! // SPEC-MAP: S-4 | 4. 授权与 publication 标准 | assert_authorization_and_publication_standard
//! // SPEC-MAP: S-5 | 5. 输入形态与解析边界 | assert_input_form_and_parse_boundary
//! // SPEC-MAP: S-6 | 6. 合成夹具声明 | assert_synthetic_fixture_declaration
//! // SPEC-MAP: S-7 | 7. 非目标与门禁 | assert_non_goals_and_gates

use std::path::{Path, PathBuf};

use ecbx::{
    attest_official_dimension_order, claim_fiscal_write_authority, current_authorization,
    ensure_not_curve_product, parse_ecb_observations, publication_semantics,
    reject_curve_promotion, validate_dataflow_id, validate_observation, validate_series_identity,
    AvailabilityEvidence, Date, EcbDataType, EcbDataflowId, EcbDimension, EcbObservation,
    EcbProtocol, EcbSeriesIdentity, EcbValue, Frequency, Period, PitEligibility, TimePrecision,
    Unit,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("目录可读") {
        let path = entry.expect("目录项可读").path();
        if path.is_dir() {
            collect_rust_sources(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn sample_series() -> EcbSeriesIdentity {
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

/// S-1：数据类型只有清单 §1.2 的 7 类；协议只有名、不含端点。
#[test]
fn assert_data_type_standard() {
    assert_eq!(
        EcbDataType::ALL
            .iter()
            .map(EcbDataType::code)
            .collect::<Vec<&str>>(),
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
    assert!(EcbDataType::from_code("M3").is_err(), "不得自行扩集");
    assert_eq!(
        EcbProtocol::ALL
            .iter()
            .map(EcbProtocol::name)
            .collect::<Vec<&str>>(),
        ["SDMX-JSON", "SDMX-ML", "CSV"]
    );
}

/// S-2：身份形 = dataflow + DSD + 规范化维度键；顺序由调用方声明，不得冒充官方 code list。
#[test]
fn assert_identity_and_declared_order_standard() {
    let identity = sample_series();
    assert_eq!(
        identity.series_id(),
        "YC:DSD_SYNTH:DIM_A=VAL_A.DIM_B=VAL_B:IND_SYNTH:SUBJ_SYNTH"
    );
    assert!(validate_series_identity(&identity).is_ok());
    assert!(validate_dataflow_id(&identity.dataflow).is_ok());
    assert!(EcbDataflowId::try_new("YC", "DSD_SYNTH", Vec::new()).is_err());
    assert_eq!(
        attest_official_dimension_order(&identity.dataflow.dimensions)
            .expect_err("不得冒充官方顺序")
            .kind(),
        ecbx::EcbErrorKind::NotApplicable
    );
}

/// S-3：曲线点可被持有，但不得在本库晋级。
#[test]
fn assert_curve_routing_standard() {
    assert!(EcbDataType::ZeroSpot.is_curve_product());
    assert!(EcbDataType::Forward.is_curve_product());
    assert!(!EcbDataType::MoneyStock.is_curve_product());
    assert_eq!(
        ensure_not_curve_product(&EcbDataType::ZeroSpot)
            .expect_err("曲线点必须路由")
            .kind(),
        ecbx::EcbErrorKind::RoutedElsewhere
    );

    let curve = EcbObservation::new(
        sample_series(),
        EcbDataType::Forward,
        Frequency::Daily,
        Period::day(Date::new(2026, 9, 18).expect("合法日期")),
        EcbValue::Value(3.0),
        Unit::try_new("SYNTH_UNIT").expect("合法单位"),
        None,
    )
    .expect("曲线点可以被持有");
    assert!(validate_observation(&curve).is_ok());
    assert!(reject_curve_promotion(&curve).is_err());
}

/// S-4：授权 fail-closed；publication 恒 `Date` + `Inferred` + `NotEligible`；财政类主权 `pending`。
#[test]
fn assert_authorization_and_publication_standard() {
    assert!(!current_authorization(Date::new(2026, 9, 22).expect("合法日期")).is_authorized());
    assert_eq!(
        publication_semantics(),
        (
            TimePrecision::Date,
            AvailabilityEvidence::Inferred,
            PitEligibility::NotEligible
        )
    );
    assert_eq!(
        claim_fiscal_write_authority()
            .expect_err("pending 主权不得主张")
            .kind(),
        ecbx::EcbErrorKind::WriteAuthorityDenied
    );
}

/// S-5：输入形态为合成标注 JSON 文档；未知字段原子失败、重复身份拒绝、缺失具名。
#[test]
fn assert_input_form_and_parse_boundary() {
    let good = include_str!("fixtures/ecb_policy_observations.json");
    let observations = parse_ecb_observations(good).expect("合成夹具必须可解析");
    assert_eq!(observations.len(), 3);
    assert_eq!(observations[0].value.value(), Some(3.25));
    assert!(observations[1].value.is_missing(), "缺失必须具名而非 0");
    assert_eq!(
        observations[2].vintage,
        Some(Date::new(2026, 9, 20).expect("合法日期"))
    );

    let unknown = good.replace("\"freq\": \"Daily\",", "\"freq\": \"Daily\", \"q\": 1,");
    assert!(
        parse_ecb_observations(&unknown).is_err(),
        "未知字段必须失败"
    );

    let duplicated = r#"{ "_synthetic": true, "_note": "合成样本",
        "observations": [
          { "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ { "id": "A", "value": "B" } ],
            "indicator": "I", "subject": "S", "data_type": "POLICY_RATE", "freq": "Daily",
            "period": "2026-09-18", "value": 1.0, "unit": "U" },
          { "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ { "id": "A", "value": "B" } ],
            "indicator": "I", "subject": "S", "data_type": "POLICY_RATE", "freq": "Daily",
            "period": "2026-09-18", "value": 2.0, "unit": "U" }
        ] }"#;
    assert_eq!(
        parse_ecb_observations(duplicated)
            .expect_err("重复身份必须拒绝")
            .kind(),
        ecbx::EcbErrorKind::SemanticallyRejected
    );

    for bad in ["2026-2-3", "2026/09/18", "2026-02-30"] {
        assert!(Date::parse(bad).is_err(), "{bad}");
    }
    assert!(parse_ecb_observations(r#"{ "observations": [] }"#).is_err());
}

/// S-6：全部夹具为合成样本，不是真实源数据，不构成证据。
#[test]
fn assert_synthetic_fixture_declaration() {
    let fixtures = manifest_dir().join("tests").join("fixtures");
    let mut files = Vec::new();
    collect_rust_sources(&fixtures, &mut files);
    let mut json_files = Vec::new();
    for entry in std::fs::read_dir(&fixtures).expect("夹具目录可读") {
        let path = entry.expect("目录项可读").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            json_files.push(path);
        }
    }
    assert!(!json_files.is_empty(), "夹具目录不得为空");
    for path in json_files {
        let text = std::fs::read_to_string(&path).expect("夹具可读");
        assert!(text.contains("\"_synthetic\": true"), "{path:?}");
        assert!(text.contains("\"_note\""), "{path:?}");
        for forbidden in ["实测", "核验 PASS", "证据等级"] {
            assert!(!text.contains(forbidden), "{path:?} 含禁用表述 {forbidden}");
        }
    }
    assert!(files.is_empty(), "夹具目录不得混入 Rust 源文件");
}

/// S-7：非目标（零端点 / 零 HTTP 依赖 / 零凭据 / 零跨仓依赖）与门禁面。
#[test]
fn assert_non_goals_and_gates() {
    let mut sources = Vec::new();
    collect_rust_sources(&manifest_dir().join("src"), &mut sources);
    assert!(!sources.is_empty(), "src 不得为空");
    for path in &sources {
        let text = std::fs::read_to_string(path).expect("源文件可读");
        for forbidden in ["http://", "https://", "env::var", "from_env"] {
            assert!(!text.contains(forbidden), "{path:?} 含禁用片段 {forbidden}");
        }
    }

    // 零 HTTP 客户端 / 零跨仓依赖：`[dependencies]` 段只允许已登记的最小依赖集。
    let manifest =
        std::fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("Cargo.toml 可读");
    assert!(!manifest.contains("path = \"../"), "零跨仓依赖");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split('[').next())
        .expect("存在 [dependencies] 段");
    let allowed = ["serde", "serde_json", "thiserror", "csv", "toml"];
    for line in dependencies.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line
            .split(|c: char| c == '=' || c.is_whitespace())
            .next()
            .expect("依赖名");
        assert!(allowed.contains(&name), "未登记的依赖：{name}");
    }

    assert!(manifest.contains("[[bench]]"));
    assert!(manifest.contains("harness = false"));

    // 门禁的测试面：三类测试文件存在。
    for name in ["tdd_contracts.rs", "sdd_spec.rs", "aidd_boundary.rs"] {
        assert!(
            manifest_dir().join("tests").join(name).is_file(),
            "缺少 tests/{name}"
        );
    }
}
