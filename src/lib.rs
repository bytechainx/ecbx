#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! ecbx —— ECB 源事实的类型层：数据类型、SDMX 身份形、离线解析与 fail-closed 授权判定。
//!
//! ## 能力
//!
//! | 能力 | 状态 |
//! | --- | --- |
//! | 7 类数据类型枚举（`ZERO_SPOT` … `DEPOSIT_FACILITY`） | 已实现 |
//! | SDMX 身份形 `dataflow + DSD + 规范化维度键` | 已实现（顺序由调用方声明） |
//! | 观测值对象（期间 / 单位 / 具名缺失 / 可选 vintage） | 已实现 |
//! | 离线 JSON 解析（未知字段原子失败、重复身份拒绝） | 已实现 |
//! | 授权判定（fail-closed） | 已实现；本源 `authorization = unknown`，判定恒 `Denied` |
//! | 曲线点晋级 / 官方 code list / live SDMX 客户端 | **未实现且禁止晋级** |
//!
//! ## 责任边界
//!
//! 本库做：把清单里的源事实落成可机械校验的类型；对曲线路由、维度顺序与授权
//! 三类越界行为 fail-closed 拒绝。
//! 本库不做：联网采集、认证、缓存、再分发、存储、单位换算、派生指标。
//!
//! ## 非目标
//!
//! - 不实现 HTTP / SDMX 客户端：清单把全部端点许可登记为 UNKNOWN，**规划端点 ≠ 访问合同**
//! - 不内置官方 DSD / code list：SDMX dataflow/DSD **非**官方表，属性键白名单 ≠ 元数据已实现
//! - 不把 YC dataflow 的曲线点晋级进内核：须 `yieldx` 映射批准
//! - 不做任何派生指标（净流动性、利差、Credit Impulse、z-score）
//!
//! ## 诚实边界
//!
//! `production_decision = NO-GO`；清单 COMPLETE ≠ ship；authorization ≠ Production Ready。
//! 本源无 Owner 签核文件，`authorization = unknown`，本库**不假装**有签核。
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), ecbx::EcbError> {
//! use ecbx::{
//!     EcbDataType, EcbDataflowId, EcbDimension, EcbObservation, EcbSeriesIdentity, EcbValue,
//!     Frequency, Period, Unit,
//! };
//!
//! let dataflow = EcbDataflowId::try_new(
//!     "YC",
//!     "DSD_SYNTH",
//!     vec![EcbDimension::try_new("DIM_A", "VAL_A")?],
//! )?;
//! let series = EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", "SUBJ_SYNTH")?;
//! let observation = EcbObservation::new(
//!     series,
//!     EcbDataType::PolicyRate,
//!     Frequency::Daily,
//!     Period::parse("2026-09-18")?,
//!     EcbValue::Value(3.25),
//!     Unit::try_new("SYNTH_UNIT")?,
//!     None,
//! )?;
//!
//! assert_eq!(observation.value.value(), Some(3.25));
//! // 授权判定 fail-closed：本源无签核文件，故恒为 Denied。
//! let verdict = ecbx::current_authorization(ecbx::Date::new(2026, 9, 22)?);
//! assert!(!verdict.is_authorized());
//! # Ok(())
//! # }
//! ```

pub mod authz;
pub mod error;
pub mod parse;
pub mod pit;
pub mod value;

pub use authz::{
    current_authorization, decide_authorization, ensure_authorized, registered_evidence,
    EcbAuthorization, EcbAuthorizationEvidence,
};
pub use error::{EcbError, EcbErrorKind, EcbResult};
pub use parse::parse_ecb_observations;
pub use pit::{
    publication_for_period, publication_semantics, AvailabilityEvidence, PitEligibility,
    TimePrecision,
};
pub use value::{
    attest_official_dimension_order, claim_fiscal_write_authority, ensure_not_curve_product,
    reject_curve_promotion, validate_dataflow_id, validate_observation, validate_series_identity,
    Date, EcbDataType, EcbDataflowId, EcbDimension, EcbMissingReason, EcbObservation, EcbProtocol,
    EcbSeriesIdentity, EcbValue, Frequency, Period, Unit,
};
