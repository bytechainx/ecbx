//! 离线解析入口：**字符串 → 观测集合**。
//!
//! 参数只有输入文本：**不接受** URL、HTTP 客户端、认证信息或任何网络相关参数。
//!
//! 输入形态（由 `docs/标准.md` 声明）：JSON 文档
//! `{ "_synthetic": <bool>, "_note": <string>, "observations": [ … ] }`。
//!
//! 两条硬约束：
//!
//! - **未知字段原子失败**（含顶层与观测层）
//! - **重复身份一律拒绝**（不去重、不后者覆盖前者）

use serde::Deserialize;

use crate::error::{EcbError, EcbResult};
use crate::value::{
    Date, EcbDataType, EcbDataflowId, EcbDimension, EcbMissingReason, EcbObservation,
    EcbSeriesIdentity, EcbValue, Frequency, Period, Unit,
};

/// 顶层文档。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDocument {
    /// 合成样本标记。**必须显式存在且为 `true`**。
    #[serde(rename = "_synthetic")]
    synthetic: bool,
    /// 合成样本说明。必须非空。
    #[serde(rename = "_note")]
    note: String,
    /// 观测数组。
    observations: Vec<WireObservation>,
}

/// 单条观测的输入形态。
///
/// 字段名取自 `specs/adapter/ecb.md` §1.3（身份形）与
/// `specs/数据清单/ecb.md` §2.3（观测字段）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireObservation {
    /// dataflow 名。
    dataflow: String,
    /// DSD 名。
    dsd: String,
    /// 维度键（按声明顺序）。
    dimensions: Vec<EcbDimension>,
    /// 指标。
    indicator: String,
    /// 主体。
    subject: String,
    /// 数据类型码（7 类之一）。
    data_type: String,
    /// 频率名。
    freq: String,
    /// 期间。
    period: String,
    /// 数值；`null` 表示缺失。
    value: Option<f64>,
    /// 源侧观测状态原文（`value` 为 `null` 时参与表达缺失原因）。
    #[serde(default)]
    obs_status: Option<String>,
    /// 源侧单位原文。
    unit: String,
    /// 修订日期（无官方 vintage 面时不填）。
    #[serde(default)]
    vintage_date: Option<String>,
}

/// 解析一整个离线文档。
///
/// 返回的观测顺序与文档中一致；任一条失败即**整体失败**（不返回部分结果）。
///
/// 输入必须显式标注 `_synthetic = true` 且给出非空 `_note`：本层只处理合成样本，
/// 拒绝把未标注输入当作源数据（对应 `FR-057` 的证据纪律）。
///
/// # Examples
///
/// ```
/// use ecbx::parse_ecb_observations;
///
/// # fn main() -> Result<(), ecbx::EcbError> {
/// let document = r#"{ "_synthetic": true, "_note": "合成样本", "observations": [] }"#;
/// let observations = parse_ecb_observations(document)?;
/// assert!(observations.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn parse_ecb_observations(input: &str) -> EcbResult<Vec<EcbObservation>> {
    let WireDocument {
        synthetic,
        note,
        observations: wire_observations,
    } = serde_json::from_str(input).map_err(describe_json_error)?;

    if !synthetic {
        return Err(EcbError::SemanticallyRejected(
            "离线解析只接受显式标注 `_synthetic = true` 的合成样本：本层不得把未标注输入当作源数据"
                .to_owned(),
        ));
    }
    if note.trim().is_empty() {
        return Err(EcbError::Missing(
            "`_note` 为空：合成样本必须说明「不是真实源数据」".to_owned(),
        ));
    }

    let mut observations = Vec::with_capacity(wire_observations.len());
    let mut seen: Vec<String> = Vec::with_capacity(wire_observations.len());
    for (index, wire) in wire_observations.into_iter().enumerate() {
        let observation = convert(wire).map_err(|error| annotate(index, error))?;
        let identity = observation.source_series_id();
        if seen.contains(&identity) {
            return Err(EcbError::SemanticallyRejected(format!(
                "第 {index} 条观测身份重复：{identity}（本库选择拒绝而非去重）"
            )));
        }
        seen.push(identity);
        observations.push(observation);
    }
    Ok(observations)
}

/// 单条观测的形态转换。
fn convert(wire: WireObservation) -> EcbResult<EcbObservation> {
    let dataflow = EcbDataflowId::try_new(&wire.dataflow, &wire.dsd, wire.dimensions)?;
    let series = EcbSeriesIdentity::try_new(dataflow, &wire.indicator, &wire.subject)?;
    let data_type = EcbDataType::from_code(&wire.data_type)?;
    let frequency = Frequency::from_name(&wire.freq)?;
    let period = Period::parse(&wire.period)?;
    let unit = Unit::try_new(&wire.unit)?;
    let vintage = wire.vintage_date.as_deref().map(Date::parse).transpose()?;
    let value = match wire.value {
        Some(value) => EcbValue::Value(value),
        None => EcbValue::Missing(match wire.obs_status.as_deref() {
            Some(status) if !status.trim().is_empty() => {
                EcbMissingReason::SourceStatus(status.to_owned())
            }
            _ => EcbMissingReason::Unspecified,
        }),
    };
    EcbObservation::new(series, data_type, frequency, period, value, unit, vintage)
}

/// 把 serde_json 的解析错误转成本层错误。
///
/// **只带出行号、列号与错误分类，不回显原文片段**——serde 的默认消息会把非法取值
/// 原样拼进字符串，那样会把凭据或整行配置源码带进错误消息。
fn describe_json_error(error: serde_json::Error) -> EcbError {
    EcbError::Invalid(format!(
        "JSON 文档不可解析（行 {} 列 {}，分类 {:?}）：请检查字段名、类型与取值形态",
        error.line(),
        error.column(),
        error.classify()
    ))
}

/// 给单条观测的错误补上位置信息，保持原分类不变。
fn annotate(index: usize, error: EcbError) -> EcbError {
    let prefix = format!("第 {index} 条观测：");
    match error {
        EcbError::Invalid(message) => EcbError::Invalid(format!("{prefix}{message}")),
        EcbError::Missing(message) => EcbError::Missing(format!("{prefix}{message}")),
        EcbError::AuthorizationDenied(message) => {
            EcbError::AuthorizationDenied(format!("{prefix}{message}"))
        }
        EcbError::RoutedElsewhere(message) => {
            EcbError::RoutedElsewhere(format!("{prefix}{message}"))
        }
        EcbError::WriteAuthorityDenied(message) => {
            EcbError::WriteAuthorityDenied(format!("{prefix}{message}"))
        }
        EcbError::SemanticallyRejected(message) => {
            EcbError::SemanticallyRejected(format!("{prefix}{message}"))
        }
        EcbError::NotApplicable(message) => EcbError::NotApplicable(format!("{prefix}{message}")),
        EcbError::Invariant(message) => EcbError::Invariant(format!("{prefix}{message}")),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_ecb_observations;
    use crate::error::EcbErrorKind;
    use crate::value::{reject_curve_promotion, EcbDataType, EcbMissingReason, EcbValue};

    const NOTE: &str = "本文件为特性 005 自拟的合成样本，不是真实源数据，不构成任何证据。";

    /// 把观测数组包成合法的合成文档。
    fn wrap(observations: &str) -> String {
        format!(
            r#"{{ "_synthetic": true, "_note": "{NOTE}", "observations": [ {observations} ] }}"#
        )
    }

    fn observation(data_type: &str, period: &str, extra: &str) -> String {
        format!(
            r#"{{ "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ {{ "id": "DIM_A", "value": "VAL_A" }} ], "indicator": "IND_SYNTH", "subject": "SUBJ_SYNTH", "data_type": "{data_type}", "freq": "Daily", "period": "{period}", "value": 1.0, "unit": "SYNTH_UNIT"{extra} }}"#
        )
    }

    fn synthetic_document() -> String {
        wrap(
            r#"{
      "dataflow": "YC",
      "dsd": "DSD_SYNTH",
      "dimensions": [
        { "id": "DIM_A", "value": "VAL_A" },
        { "id": "DIM_B", "value": "VAL_B" }
      ],
      "indicator": "IND_SYNTH",
      "subject": "SUBJ_SYNTH",
      "data_type": "POLICY_RATE",
      "freq": "Daily",
      "period": "2026-09-18",
      "value": 3.25,
      "unit": "SYNTH_UNIT",
      "vintage_date": null
    },
    {
      "dataflow": "BSI",
      "dsd": "DSD_SYNTH",
      "dimensions": [ { "id": "DIM_A", "value": "VAL_C" } ],
      "indicator": "IND_SYNTH",
      "subject": "SUBJ_SYNTH",
      "data_type": "MONEY_STOCK",
      "freq": "Monthly",
      "period": "2026-09",
      "value": null,
      "obs_status": "SYNTH_STATUS",
      "unit": "SYNTH_UNIT"
    }"#,
        )
    }

    #[test]
    fn parses_positions_values_and_named_missing() {
        let observations = parse_ecb_observations(&synthetic_document()).expect("合成文档应可解析");
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].data_type, EcbDataType::PolicyRate);
        assert_eq!(observations[0].value, EcbValue::Value(3.25));
        assert_eq!(observations[0].unit.text(), "SYNTH_UNIT");
        assert_eq!(observations[0].vintage, None);
        assert_eq!(observations[1].value.value(), None, "缺失不得转 0");
        assert_eq!(
            observations[1].value.missing_reason(),
            Some(&EcbMissingReason::SourceStatus("SYNTH_STATUS".to_owned()))
        );
        assert_ne!(
            observations[0].source_series_id(),
            observations[1].source_series_id()
        );
    }

    #[test]
    fn unmarked_input_is_refused() {
        // 缺 `_synthetic` / `_note` ⇒ 不是合法离线文档。
        let unmarked = r#"{ "observations": [] }"#;
        assert!(parse_ecb_observations(unmarked).is_err());

        let contradicted = r#"{ "_synthetic": false, "_note": "x", "observations": [] }"#;
        let error = parse_ecb_observations(contradicted).expect_err("不得把非合成输入当源数据");
        assert_eq!(error.kind(), EcbErrorKind::SemanticallyRejected);

        let blank_note = r#"{ "_synthetic": true, "_note": "  ", "observations": [] }"#;
        assert_eq!(
            parse_ecb_observations(blank_note)
                .expect_err("空说明必须拒绝")
                .kind(),
            EcbErrorKind::Missing
        );
    }

    #[test]
    fn missing_field_fails_atomically() {
        // `unit` 是必需字段：缺失即整体失败（`value` 允许缺省为「缺失」）。
        let document = wrap(
            r#"{ "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ { "id": "DIM_A", "value": "VAL_A" } ], "indicator": "IND_SYNTH", "subject": "SUBJ_SYNTH", "data_type": "POLICY_RATE", "freq": "Daily", "period": "2026-09-18", "value": 1.0 }"#,
        );
        let error = parse_ecb_observations(&document).expect_err("缺字段必须整体失败");
        assert_eq!(error.kind(), EcbErrorKind::Invalid);
    }

    #[test]
    fn absent_value_field_degrades_to_named_missing_not_zero() {
        let document = wrap(
            r#"{ "dataflow": "YC", "dsd": "DSD_SYNTH", "dimensions": [ { "id": "DIM_A", "value": "VAL_A" } ], "indicator": "IND_SYNTH", "subject": "SUBJ_SYNTH", "data_type": "POLICY_RATE", "freq": "Daily", "period": "2026-09-18", "unit": "SYNTH_UNIT" }"#,
        );
        let observations = parse_ecb_observations(&document).expect("缺省 value 视为缺失");
        assert_eq!(observations[0].value.value(), None, "缺失不得转 0");
        assert_eq!(
            observations[0].value.missing_reason(),
            Some(&EcbMissingReason::Unspecified)
        );
    }

    #[test]
    fn unknown_field_fails_atomically() {
        let observation_level = wrap(&observation(
            "POLICY_RATE",
            "2026-09-18",
            ", \"latest_release\": true",
        ));
        let error = parse_ecb_observations(&observation_level).expect_err("观测层未知字段必须失败");
        assert_eq!(error.kind(), EcbErrorKind::Invalid);

        let top_level = format!(
            r#"{{ "_synthetic": true, "_note": "{NOTE}", "observations": [], "endpoint": "nope" }}"#
        );
        assert!(parse_ecb_observations(&top_level).is_err());
    }

    #[test]
    fn json_error_message_does_not_echo_the_offending_value() {
        let document = format!(
            r#"{{ "_synthetic": true, "_note": "{NOTE}", "observations": [ {{ "dataflow": "synthetic-secret-value", "dsd": 1 }} ] }}"#
        );
        let error = parse_ecb_observations(&document).expect_err("非法类型必须失败");
        let message = error.to_string();
        assert!(!message.contains("synthetic-secret-value"), "{message}");
        assert_eq!(error.kind(), EcbErrorKind::Invalid);
    }

    #[test]
    fn illegal_date_is_rejected() {
        for period in [
            "2026-2-3",
            "2026/09/18",
            "2026-02-30",
            "2026-09-18T00:00:00Z",
        ] {
            let document = wrap(&observation("POLICY_RATE", period, ""));
            let error = parse_ecb_observations(&document).expect_err("非法日期必须拒绝");
            assert_eq!(error.kind(), EcbErrorKind::Invalid, "period={period}");
        }
    }

    #[test]
    fn duplicate_identity_is_rejected_not_deduplicated() {
        let one = observation("POLICY_RATE", "2026-09-18", "");
        let document = wrap(&format!("{one}, {one}"));
        let error = parse_ecb_observations(&document).expect_err("重复身份必须拒绝");
        assert_eq!(error.kind(), EcbErrorKind::SemanticallyRejected);
    }

    #[test]
    fn unsupported_data_type_code_is_rejected() {
        let document = wrap(&observation("YIELD_CURVE", "2026-09-18", ""));
        let error = parse_ecb_observations(&document).expect_err("未知类型码必须拒绝");
        assert_eq!(error.kind(), EcbErrorKind::Invalid);
        assert!(error.to_string().contains("第 0 条观测"), "{error}");
    }

    #[test]
    fn curve_observation_is_parsed_but_still_not_promotable() {
        let document = wrap(&observation("ZERO_SPOT", "2026-09-18", ""));
        let observations = parse_ecb_observations(&document).expect("曲线点本身可被解析");
        assert_eq!(observations[0].data_type, EcbDataType::ZeroSpot);
        assert!(reject_curve_promotion(&observations[0]).is_err());
    }

    #[test]
    fn empty_observation_list_is_accepted() {
        let observations = parse_ecb_observations(&wrap("")).expect("空数组是合法文档");
        assert!(observations.is_empty());
    }
}
