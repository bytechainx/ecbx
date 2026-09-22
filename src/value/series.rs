//! 数据类型、协议名与 SDMX 身份形。
//!
//! 身份形为 `source_series_id = dataflow + DSD + 规范化维度键`（再叠加
//! `indicator + subject + period + vintage`，见 [`crate::EcbObservation`]）。
//!
//! **两处硬禁**都在本文件里落成守卫：
//!
//! 1. 手填维度顺序**不得**冒充官方 code list（[`attest_official_dimension_order`] 恒拒绝）
//! 2. YC dataflow 的曲线点**不在本库晋级**（[`EcbDataType::is_curve_product`] /
//!    [`ensure_not_curve_product`]）

use std::fmt;

use serde::Deserialize;

use crate::error::{EcbError, EcbResult};

/// 数据类型（`specs/adapter/ecb.md` §1.2 逐字列出的 7 类）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcbDataType {
    /// `ZERO_SPOT`：零息即期曲线。
    ZeroSpot,
    /// `FORWARD`：隐含远期。
    Forward,
    /// `POLICY_RATE`：MRO/DFR/MLF。
    PolicyRate,
    /// `RFR_OVERNIGHT`：€STR。
    RfrOvernight,
    /// `CB_BALANCE_SHEET`：周报资产负债表。
    CbBalanceSheet,
    /// `MONEY_STOCK`：M0/M1/M2（BSI）。
    MoneyStock,
    /// `DEPOSIT_FACILITY`：存贷便利。
    DepositFacility,
}

impl EcbDataType {
    /// 全部 7 类，顺序与清单一致。
    pub const ALL: [Self; 7] = [
        Self::ZeroSpot,
        Self::Forward,
        Self::PolicyRate,
        Self::RfrOvernight,
        Self::CbBalanceSheet,
        Self::MoneyStock,
        Self::DepositFacility,
    ];

    /// 清单里的类型码（逐字）。
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::ZeroSpot => "ZERO_SPOT",
            Self::Forward => "FORWARD",
            Self::PolicyRate => "POLICY_RATE",
            Self::RfrOvernight => "RFR_OVERNIGHT",
            Self::CbBalanceSheet => "CB_BALANCE_SHEET",
            Self::MoneyStock => "MONEY_STOCK",
            Self::DepositFacility => "DEPOSIT_FACILITY",
        }
    }

    /// 清单里的中文说明（逐字）。
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::ZeroSpot => "零息即期曲线",
            Self::Forward => "隐含远期",
            Self::PolicyRate => "MRO/DFR/MLF",
            Self::RfrOvernight => "€STR",
            Self::CbBalanceSheet => "周报资产负债表",
            Self::MoneyStock => "M0/M1/M2（BSI）",
            Self::DepositFacility => "存贷便利",
        }
    }

    /// 是否曲线产品。
    ///
    /// `ZERO_SPOT` 与 `FORWARD` 是 YC dataflow 的曲线点：它们必须经 `yieldx`
    /// 映射批准后才能进内核，**本库不得晋级**。
    #[must_use]
    pub fn is_curve_product(&self) -> bool {
        matches!(self, Self::ZeroSpot | Self::Forward)
    }

    /// 按类型码解析。
    pub fn from_code(code: &str) -> EcbResult<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.code() == code)
            .ok_or_else(|| EcbError::Invalid(format!("未知数据类型码：{code}")))
    }
}

impl fmt::Display for EcbDataType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

/// 协议**名**枚举。
///
/// 本库只登记协议名，**不含任何端点**：清单把这些端点的许可登记为 UNKNOWN，
/// 规划端点 ≠ 访问合同。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcbProtocol {
    /// SDMX-JSON。
    SdmxJson,
    /// SDMX-ML。
    SdmxMl,
    /// CSV。
    Csv,
}

impl EcbProtocol {
    /// 全部协议名。
    pub const ALL: [Self; 3] = [Self::SdmxJson, Self::SdmxMl, Self::Csv];

    /// 协议名（清单 §1.1 逐字）。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SdmxJson => "SDMX-JSON",
            Self::SdmxMl => "SDMX-ML",
            Self::Csv => "CSV",
        }
    }

    /// 按名称解析（大小写敏感）。
    pub fn from_name(name: &str) -> EcbResult<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.name() == name)
            .ok_or_else(|| EcbError::Invalid(format!("未知协议名：{name}")))
    }
}

/// 单个维度（键 + 值）。
///
/// `id` / `value` 都是**调用方提供**的字符串：本库不内置任何官方 code list，
/// 因此也不能声称某个 `id` 是官方维度。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EcbDimension {
    /// 维度键。
    pub id: String,
    /// 维度取值。
    pub value: String,
}

impl EcbDimension {
    /// 构造并校验（键与值均不得为空）。
    pub fn try_new(id: &str, value: &str) -> EcbResult<Self> {
        if id.trim().is_empty() {
            return Err(EcbError::Missing("维度键为空".to_owned()));
        }
        if value.trim().is_empty() {
            return Err(EcbError::Missing(format!("维度 {} 的取值为空", id.trim())));
        }
        Ok(Self {
            id: id.to_owned(),
            value: value.to_owned(),
        })
    }

    /// 单维度的规范化片段 `id=value`。
    #[must_use]
    pub fn key(&self) -> String {
        format!(
            "{}={}",
            escape_identity(&self.id),
            escape_identity(&self.value)
        )
    }
}

/// `dataflow + DSD + 有序列出` 的维度键。
///
/// **顺序是调用方声明的顺序**，不是官方 code list 顺序——本库既不持有也不推断
/// 官方顺序，见 [`attest_official_dimension_order`]。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcbDataflowId {
    /// dataflow 名（调用方提供）。
    pub dataflow: String,
    /// DSD 名（调用方提供；**非**官方表名，清单登记为 absent）。
    pub dsd: String,
    /// 维度键，按调用方声明的顺序排列。
    pub dimensions: Vec<EcbDimension>,
}

impl EcbDataflowId {
    /// 构造并校验。
    ///
    /// 要求 `dataflow` 与 `dsd` 非空、至少一个维度、维度键非空且互不重复。
    /// 校验通过**不代表**维度顺序是官方顺序——本库无从判断，故不声称。
    pub fn try_new(dataflow: &str, dsd: &str, dimensions: Vec<EcbDimension>) -> EcbResult<Self> {
        let candidate = Self {
            dataflow: dataflow.to_owned(),
            dsd: dsd.to_owned(),
            dimensions,
        };
        validate_dataflow_id(&candidate)?;
        Ok(candidate)
    }

    /// 规范化维度键（`id=value` 以 `.` 连接，保持声明顺序）。
    #[must_use]
    pub fn declared_dimension_key(&self) -> String {
        self.dimensions
            .iter()
            .map(EcbDimension::key)
            .collect::<Vec<String>>()
            .join(".")
    }
}

/// 校验 dataflow 身份形。
pub fn validate_dataflow_id(id: &EcbDataflowId) -> EcbResult<()> {
    if id.dataflow.trim().is_empty() {
        return Err(EcbError::Missing("dataflow 名为空".to_owned()));
    }
    if id.dsd.trim().is_empty() {
        return Err(EcbError::Missing("DSD 名为空".to_owned()));
    }
    if id.dimensions.is_empty() {
        return Err(EcbError::Missing(
            "维度键为空：身份形要求 dataflow + DSD + 规范化维度键".to_owned(),
        ));
    }
    for (index, dimension) in id.dimensions.iter().enumerate() {
        if dimension.id.trim().is_empty() {
            return Err(EcbError::Missing(format!("第 {index} 个维度键为空")));
        }
        if dimension.value.trim().is_empty() {
            return Err(EcbError::Missing(format!(
                "第 {index} 个维度（{}）的取值为空",
                dimension.id.trim()
            )));
        }
        if id.dimensions[index + 1..]
            .iter()
            .any(|other| other.id.trim() == dimension.id.trim())
        {
            return Err(EcbError::SemanticallyRejected(format!(
                "维度键 {} 重复：同一维度不得出现两次",
                dimension.id.trim()
            )));
        }
    }
    Ok(())
}

/// 拒绝把调用方手填的维度顺序**冒充**官方 code list。
///
/// 清单把官方 DSD / code list 登记为 `absent`，并明写「SDMX dataflow/DSD **非**官方表；
/// 离线属性键白名单 ≠ 元数据已实现」。因此本函数在设计上**恒拒绝**：
/// 它接受任何声明顺序，只为证明「本库没有、也不接受官方顺序的断言」。
pub fn attest_official_dimension_order(_dimensions: &[EcbDimension]) -> EcbResult<()> {
    Err(EcbError::NotApplicable(
        "官方 DSD / code list 未核验（absent）：本库不得把手填顺序断言为官方顺序".to_owned(),
    ))
}

/// 曲线点不得作为普通观测在本库晋级。
///
/// 曲线产品须指向 `yieldx`：清单 §2.2 把「YC 曲线点进 kernel」登记为 `planned` 且
/// 「须 yield_curve 映射批准」。
pub fn ensure_not_curve_product(data_type: &EcbDataType) -> EcbResult<()> {
    if data_type.is_curve_product() {
        return Err(EcbError::RoutedElsewhere(format!(
            "{} 是曲线点：须经 yieldx 映射批准后方可进内核，本库不晋级",
            data_type.code()
        )));
    }
    Ok(())
}

/// 声明「ECB / 日本 MOF 财政类」的权威写入主权。
///
/// 跨源路由契约 §3 把该行的主权登记为 **`pending`（未决）**，并要求实现保持拒绝
/// （双方均不得主张权威写入）。本函数因此恒返回
/// [`EcbError::WriteAuthorityDenied`]。
pub fn claim_fiscal_write_authority() -> EcbResult<()> {
    Err(EcbError::WriteAuthorityDenied(
        "ECB / 日本 MOF 财政类的写入主权为 pending（未决）：本库不得主张权威写入".to_owned(),
    ))
}

/// `indicator + subject + dataflow 身份` 的序列身份。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcbSeriesIdentity {
    /// dataflow 身份（含 DSD 与声明顺序下的维度键）。
    pub dataflow: EcbDataflowId,
    /// 指标。
    pub indicator: String,
    /// 主体（如部门 / 国家 / 币种）。
    pub subject: String,
}

impl EcbSeriesIdentity {
    /// 构造并校验。
    pub fn try_new(dataflow: EcbDataflowId, indicator: &str, subject: &str) -> EcbResult<Self> {
        if indicator.trim().is_empty() {
            return Err(EcbError::Missing("indicator 为空".to_owned()));
        }
        if subject.trim().is_empty() {
            return Err(EcbError::Missing("subject 为空".to_owned()));
        }
        Ok(Self {
            dataflow,
            indicator: indicator.to_owned(),
            subject: subject.to_owned(),
        })
    }

    /// `dataflow + DSD + 规范化维度键 + indicator + subject` 的规范身份串。
    #[must_use]
    pub fn series_id(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            escape_identity(&self.dataflow.dataflow),
            escape_identity(&self.dataflow.dsd),
            self.dataflow.declared_dimension_key(),
            escape_identity(&self.indicator),
            escape_identity(&self.subject)
        )
    }
}

/// 校验序列身份。
pub fn validate_series_identity(identity: &EcbSeriesIdentity) -> EcbResult<()> {
    validate_dataflow_id(&identity.dataflow)?;
    if identity.indicator.trim().is_empty() {
        return Err(EcbError::Missing("indicator 为空".to_owned()));
    }
    if identity.subject.trim().is_empty() {
        return Err(EcbError::Missing("subject 为空".to_owned()));
    }
    Ok(())
}

/// 对身份组件中的分隔符与转义前缀编码，普通字符保持兼容。
fn escape_identity(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '%' => encoded.push_str("%25"),
            ':' => encoded.push_str("%3A"),
            '.' => encoded.push_str("%2E"),
            '=' => encoded.push_str("%3D"),
            other => encoded.push(other),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        attest_official_dimension_order, claim_fiscal_write_authority, ensure_not_curve_product,
        validate_dataflow_id, validate_series_identity, EcbDataType, EcbDataflowId, EcbDimension,
        EcbProtocol, EcbSeriesIdentity,
    };
    use crate::error::EcbErrorKind;

    fn dims() -> Vec<EcbDimension> {
        vec![
            EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
            EcbDimension::try_new("DIM_B", "VAL_B").expect("合法维度"),
        ]
    }

    fn identity() -> EcbSeriesIdentity {
        let dataflow = EcbDataflowId::try_new("YC", "DSD_SYNTH", dims()).expect("合法身份");
        EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", "SUBJ_SYNTH").expect("合法序列身份")
    }

    #[test]
    fn seven_data_types_are_exhaustive_and_round_trip() {
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
            assert_eq!(
                EcbDataType::from_code(data_type.code()).expect("可回读"),
                data_type
            );
            assert!(!data_type.description().is_empty());
            assert_eq!(data_type.to_string(), data_type.code());
        }
        assert!(EcbDataType::from_code("YIELD_CURVE").is_err());
    }

    #[test]
    fn only_zero_spot_and_forward_are_curve_products() {
        for data_type in EcbDataType::ALL {
            let expected = matches!(data_type, EcbDataType::ZeroSpot | EcbDataType::Forward);
            assert_eq!(data_type.is_curve_product(), expected, "{data_type}");
            assert_eq!(ensure_not_curve_product(&data_type).is_err(), expected);
        }
    }

    #[test]
    fn curve_products_are_routed_elsewhere_not_silently_accepted() {
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
    fn protocol_names_are_names_only_and_round_trip() {
        let names: Vec<&str> = EcbProtocol::ALL.iter().map(EcbProtocol::name).collect();
        assert_eq!(names, ["SDMX-JSON", "SDMX-ML", "CSV"]);
        for protocol in EcbProtocol::ALL {
            assert_eq!(
                EcbProtocol::from_name(protocol.name()).expect("可回读"),
                protocol
            );
        }
        assert!(EcbProtocol::from_name("sdmx-json").is_err());
    }

    #[test]
    fn dimension_order_is_preserved_but_never_claimed_official() {
        let dataflow = EcbDataflowId::try_new("YC", "DSD_SYNTH", dims()).expect("合法身份");
        assert_eq!(dataflow.declared_dimension_key(), "DIM_A=VAL_A.DIM_B=VAL_B");

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
        assert_ne!(
            dataflow.declared_dimension_key(),
            reversed.declared_dimension_key(),
            "顺序不同 ⇒ 维度键不同，不得互相冒充"
        );

        // 无论声明什么顺序，本库都拒绝「这是官方顺序」的断言。
        let error = attest_official_dimension_order(&dims()).expect_err("不得冒充官方顺序");
        assert_eq!(error.kind(), EcbErrorKind::NotApplicable);
        assert!(attest_official_dimension_order(&[]).is_err());
    }

    #[test]
    fn dataflow_id_rejects_missing_and_duplicate_parts() {
        assert!(EcbDataflowId::try_new("", "DSD_SYNTH", dims()).is_err());
        assert!(EcbDataflowId::try_new("YC", "  ", dims()).is_err());
        assert!(EcbDataflowId::try_new("YC", "DSD_SYNTH", vec![]).is_err());
        assert!(EcbDimension::try_new("", "VAL_A").is_err());
        assert!(EcbDimension::try_new("DIM_A", " ").is_err());
        let duplicated = vec![
            EcbDimension::try_new("DIM_A", "VAL_A").expect("合法维度"),
            EcbDimension::try_new("DIM_A", "VAL_B").expect("合法维度"),
        ];
        let error =
            EcbDataflowId::try_new("YC", "DSD_SYNTH", duplicated).expect_err("维度键重复必须拒绝");
        assert_eq!(error.kind(), EcbErrorKind::SemanticallyRejected);
    }

    #[test]
    fn series_identity_composes_the_documented_parts() {
        let identity = identity();
        assert_eq!(
            identity.series_id(),
            "YC:DSD_SYNTH:DIM_A=VAL_A.DIM_B=VAL_B:IND_SYNTH:SUBJ_SYNTH"
        );
        assert!(validate_series_identity(&identity).is_ok());
        assert!(validate_dataflow_id(&identity.dataflow).is_ok());
    }

    #[test]
    fn series_identity_rejects_empty_indicator_or_subject() {
        let dataflow = EcbDataflowId::try_new("YC", "DSD_SYNTH", dims()).expect("合法身份");
        assert!(EcbSeriesIdentity::try_new(dataflow.clone(), "", "SUBJ_SYNTH").is_err());
        assert!(EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", " ").is_err());
    }

    #[test]
    fn fiscal_write_authority_is_refused_as_pending() {
        let error = claim_fiscal_write_authority().expect_err("pending 主权不得主张");
        assert_eq!(error.kind(), EcbErrorKind::WriteAuthorityDenied);
    }

    #[test]
    fn identity_delimiters_do_not_collide() {
        let one = EcbDataflowId::try_new(
            "SYNTH",
            "DSD_SYNTH",
            vec![EcbDimension::try_new("A", "B.C=D").unwrap()],
        )
        .unwrap();
        let two = EcbDataflowId::try_new(
            "SYNTH",
            "DSD_SYNTH",
            vec![
                EcbDimension::try_new("A", "B").unwrap(),
                EcbDimension::try_new("C", "D").unwrap(),
            ],
        )
        .unwrap();
        assert_ne!(one.declared_dimension_key(), two.declared_dimension_key());
        let left = EcbSeriesIdentity::try_new(one.clone(), "I:S", "T").unwrap();
        let right = EcbSeriesIdentity::try_new(one, "I", "S:T").unwrap();
        assert_ne!(left.series_id(), right.series_id());
    }

    #[test]
    fn identity_escape_is_unambiguous_and_plain_keys_are_unchanged() {
        assert_eq!(super::escape_identity("DIM_A"), "DIM_A");
        assert_eq!(super::escape_identity("中文"), "中文");
        for (left, right) in [
            ("A.B", "A%2EB"),
            ("A=B", "A%3DB"),
            ("A:B", "A%3AB"),
            ("A%B", "A%25B"),
        ] {
            assert_ne!(super::escape_identity(left), super::escape_identity(right));
        }
        let a = EcbDataflowId::try_new("A:B", "C", vec![EcbDimension::try_new("D", "V").unwrap()])
            .unwrap();
        let b = EcbDataflowId::try_new("A", "B:C", vec![EcbDimension::try_new("D", "V").unwrap()])
            .unwrap();
        assert_ne!(
            EcbSeriesIdentity::try_new(a, "I", "S").unwrap().series_id(),
            EcbSeriesIdentity::try_new(b, "I", "S").unwrap().series_id()
        );
        let a = EcbDimension::try_new("A=B", "C").unwrap();
        let b = EcbDimension::try_new("A", "B=C").unwrap();
        assert_ne!(a.key(), b.key());
    }
}
