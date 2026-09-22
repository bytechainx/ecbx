//! 授权判定（fail-closed）。
//!
//! **本源 `authorization = unknown`**：无 Owner 签核文件，全部端点许可 UNKNOWN。
//! 因此本模块的判定入口在**当前登记状态**下恒为 `Denied`；判定是**只读结论**，
//! 不改写清单或名册的任何登记值。
//!
//! 判定的是「该源的这个范围是否被授权访问」，**不是**「本库是否生产就绪」——
//! 本库的 `production_decision` 恒为 `NO-GO`，两者共存不矛盾。

use crate::error::{EcbError, EcbResult};
use crate::value::Date;

/// 授权判定结果。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcbAuthorization {
    /// 证据有效且覆盖本次请求的范围与有效期。
    Authorized {
        /// 被覆盖的源 / 端点 / 模式 / 用途 / 有效期。
        scope: String,
    },
    /// 证据缺失 / 过期 / 签署者不明 / 覆盖范围不明。
    Denied {
        /// 可读的拒绝理由。
        reason: String,
    },
}

impl EcbAuthorization {
    /// 是否放行。
    #[must_use]
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized { .. })
    }

    /// 拒绝理由；放行时为 `None`。
    #[must_use]
    pub fn denial_reason(&self) -> Option<&str> {
        match self {
            Self::Authorized { .. } => None,
            Self::Denied { reason } => Some(reason.as_str()),
        }
    }
}

/// 授权证据的离线登记形态。
///
/// 各字段都可能在未核验时为「不明」，此时**必须**表达为 `None` / 空串，
/// 由判定函数 fail-closed 拒绝——**禁止**用「清单里写了 approved」替代证据。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcbAuthorizationEvidence {
    /// 证据出处（签核文件名或登记号）。空串视为缺失。
    pub evidence_ref: String,
    /// 签署者。不明时为 `None`。
    pub signer: Option<String>,
    /// 被覆盖的源 / 端点 / 模式 / 用途。不明时为 `None`。
    pub scope: Option<String>,
    /// 有效期截止日（含）。不明时为 `None`。
    pub valid_until: Option<Date>,
}

impl EcbAuthorizationEvidence {
    /// 构造一条证据登记。不做放行判断——判断归 [`decide_authorization`]。
    #[must_use]
    pub fn new(
        evidence_ref: &str,
        signer: Option<&str>,
        scope: Option<&str>,
        valid_until: Option<Date>,
    ) -> Self {
        Self {
            evidence_ref: evidence_ref.to_owned(),
            signer: signer.map(str::to_owned),
            scope: scope.map(str::to_owned),
            valid_until,
        }
    }
}

/// fail-closed 授权判定。
///
/// 下列任一情形**一律**返回 [`EcbAuthorization::Denied`]：
/// 证据缺失、证据出处为空、签署者不明、覆盖范围不明、有效期不明、证据已过期。
/// **MUST NOT 默认放行。**
#[must_use]
pub fn decide_authorization(
    evidence: Option<&EcbAuthorizationEvidence>,
    today: Date,
) -> EcbAuthorization {
    let denial = |reason: &str| EcbAuthorization::Denied {
        reason: reason.to_owned(),
    };
    let Some(evidence) = evidence else {
        return denial("授权证据缺失：本层不得默认放行");
    };
    if evidence.evidence_ref.trim().is_empty() {
        return denial("授权证据出处为空：无法追溯，按缺失处理");
    }
    let Some(signer) = evidence.signer.as_deref() else {
        return denial("签署者不明：无 Owner 签核，不得按已授权处理");
    };
    if signer.trim().is_empty() {
        return denial("签署者为空串：按签署者不明处理");
    }
    let Some(scope) = evidence.scope.as_deref() else {
        return denial("覆盖范围不明：无法判定本次请求是否被覆盖");
    };
    if scope.trim().is_empty() {
        return denial("覆盖范围为空串：按范围不明处理");
    }
    let Some(valid_until) = evidence.valid_until else {
        return denial("有效期不明：无法判定证据是否仍然有效");
    };
    if valid_until < today {
        return denial("证据已过期：有效期截止日早于当前日期");
    }
    EcbAuthorization::Authorized {
        scope: scope.to_owned(),
    }
}

/// 本源的授权证据登记返回值。
///
/// 恒为 `None`。这**不是**「暂未填写」，而是名册登记的既成事实：
/// `authorization = unknown`、`authorization_evidence = 无 Owner 签核文件；全部端点许可 UNKNOWN`。
#[must_use]
pub fn registered_evidence() -> Option<EcbAuthorizationEvidence> {
    None
}

/// 本源当前授权判定。
///
/// 由于 [`registered_evidence`] 恒为 `None`，本函数恒返回 `Denied`。
#[must_use]
pub fn current_authorization(today: Date) -> EcbAuthorization {
    decide_authorization(registered_evidence().as_ref(), today)
}

/// 把判定结果转成错误：未放行时返回 [`EcbError::AuthorizationDenied`]。
///
/// 供解析与校验路径在进入「需要授权」的能力前统一 fail-closed。
pub fn ensure_authorized(verdict: &EcbAuthorization) -> EcbResult<()> {
    match verdict {
        EcbAuthorization::Authorized { .. } => Ok(()),
        EcbAuthorization::Denied { reason } => {
            Err(EcbError::AuthorizationDenied(reason.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        current_authorization, decide_authorization, ensure_authorized, registered_evidence,
        EcbAuthorization, EcbAuthorizationEvidence,
    };
    use crate::error::EcbErrorKind;
    use crate::value::Date;

    fn today() -> Date {
        Date::new(2026, 9, 22).expect("合法日期")
    }

    /// 合成证据只用于验证判定形状，**不构成任何真实授权证据**。
    fn synthetic_valid_evidence() -> EcbAuthorizationEvidence {
        EcbAuthorizationEvidence::new(
            "synthetic-evidence-ref",
            Some("synthetic-signer"),
            Some("offline_parse_and_types"),
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        )
    }

    #[test]
    fn missing_evidence_is_denied() {
        let verdict = decide_authorization(None, today());
        assert!(!verdict.is_authorized());
        assert!(verdict
            .denial_reason()
            .is_some_and(|r| !r.trim().is_empty()));
    }

    #[test]
    fn registered_evidence_is_absent_for_this_source() {
        // 本源 authorization = unknown：不得凭空构造证据。
        assert!(registered_evidence().is_none());
        assert!(!current_authorization(today()).is_authorized());
    }

    #[test]
    fn unknown_scope_is_denied() {
        let evidence = EcbAuthorizationEvidence::new(
            "synthetic-evidence-ref",
            Some("synthetic-signer"),
            None,
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        );
        let verdict = decide_authorization(Some(&evidence), today());
        assert!(!verdict.is_authorized());
        assert_eq!(
            verdict.denial_reason(),
            Some("覆盖范围不明：无法判定本次请求是否被覆盖")
        );
    }

    #[test]
    fn unknown_signer_is_denied() {
        let evidence = EcbAuthorizationEvidence::new(
            "synthetic-evidence-ref",
            None,
            Some("offline_parse_and_types"),
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        );
        assert!(!decide_authorization(Some(&evidence), today()).is_authorized());
    }

    #[test]
    fn unknown_expiry_is_denied() {
        let evidence = EcbAuthorizationEvidence::new(
            "synthetic-evidence-ref",
            Some("synthetic-signer"),
            Some("offline_parse_and_types"),
            None,
        );
        assert!(!decide_authorization(Some(&evidence), today()).is_authorized());
    }

    #[test]
    fn expired_evidence_is_denied() {
        let evidence = EcbAuthorizationEvidence::new(
            "synthetic-evidence-ref",
            Some("synthetic-signer"),
            Some("offline_parse_and_types"),
            Some(Date::new(2026, 1, 1).expect("合法日期")),
        );
        let verdict = decide_authorization(Some(&evidence), today());
        assert!(!verdict.is_authorized());
        assert_eq!(
            verdict.denial_reason(),
            Some("证据已过期：有效期截止日早于当前日期")
        );
    }

    #[test]
    fn complete_evidence_is_authorized_and_scoped() {
        let evidence = synthetic_valid_evidence();
        let verdict = decide_authorization(Some(&evidence), today());
        assert!(verdict.is_authorized());
        assert_eq!(verdict.denial_reason(), None);
        match verdict {
            EcbAuthorization::Authorized { scope } => {
                assert_eq!(scope, "offline_parse_and_types");
            }
            other => unreachable!("已断言放行：{other:?}"),
        }
    }

    #[test]
    fn empty_scope_string_is_denied() {
        let evidence = EcbAuthorizationEvidence::new(
            "synthetic-evidence-ref",
            Some("synthetic-signer"),
            Some("   "),
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        );
        assert!(!decide_authorization(Some(&evidence), today()).is_authorized());
    }

    #[test]
    fn ensure_authorized_maps_denial_to_error_kind() {
        let denied = current_authorization(today());
        let error = ensure_authorized(&denied).expect_err("unknown 授权必须拒绝");
        assert_eq!(error.kind(), EcbErrorKind::AuthorizationDenied);
        assert!(!error.is_retryable());

        let allowed = decide_authorization(Some(&synthetic_valid_evidence()), today());
        assert!(ensure_authorized(&allowed).is_ok());
    }
}
