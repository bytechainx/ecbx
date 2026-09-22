//! ecbx 的错误分类与错误类型。
//!
//! 分类按「调用方应如何反应」划分：授权拒绝、路由拒绝、越权写入与语义拒绝
//! 各有独立变体，调用方无需做字符串匹配即可分流。

/// 错误分类：按「调用方应如何反应」划分。
///
/// 禁止用字符串匹配替代对本枚举的匹配。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcbErrorKind {
    /// 输入形态或取值非法（调用方应修数据，重试无意义）。
    Invalid,
    /// 缺少必需项。
    Missing,
    /// 授权判定未通过（fail-closed 落点）。
    AuthorizationDenied,
    /// 该产品不属于本域，已被路由规则拒绝。
    RoutedElsewhere,
    /// 越权写入（权威写入归他域）。
    WriteAuthorityDenied,
    /// 结构可解析但语义不被接受（如近义非同 ID、手填维度顺序冒充官方表）。
    SemanticallyRejected,
    /// 尚未实现的规划能力（如官方 DSD / code list 元数据）。
    NotApplicable,
    /// 不变量被破坏（库内 bug 的信号）。
    Invariant,
}

/// ecbx 错误。保留可区分的分类与来源链。
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EcbError {
    /// 输入非法。
    #[error("输入非法：{0}")]
    Invalid(String),
    /// 缺少必需项。
    #[error("缺少必需项：{0}")]
    Missing(String),
    /// 授权判定未通过。
    #[error("授权判定未通过：{0}")]
    AuthorizationDenied(String),
    /// 产品不属于本域。
    #[error("不属于本域，已路由他处：{0}")]
    RoutedElsewhere(String),
    /// 越权写入。
    #[error("越权写入被拒绝：{0}")]
    WriteAuthorityDenied(String),
    /// 语义拒绝。
    #[error("语义不被接受：{0}")]
    SemanticallyRejected(String),
    /// 规划能力未实现。
    #[error("规划能力尚未实现：{0}")]
    NotApplicable(String),
    /// 不变量被破坏。
    #[error("不变量被破坏：{0}")]
    Invariant(String),
}

impl EcbError {
    /// 分类，供调用方按「如何反应」分流。
    #[must_use]
    pub fn kind(&self) -> EcbErrorKind {
        match self {
            Self::Invalid(_) => EcbErrorKind::Invalid,
            Self::Missing(_) => EcbErrorKind::Missing,
            Self::AuthorizationDenied(_) => EcbErrorKind::AuthorizationDenied,
            Self::RoutedElsewhere(_) => EcbErrorKind::RoutedElsewhere,
            Self::WriteAuthorityDenied(_) => EcbErrorKind::WriteAuthorityDenied,
            Self::SemanticallyRejected(_) => EcbErrorKind::SemanticallyRejected,
            Self::NotApplicable(_) => EcbErrorKind::NotApplicable,
            Self::Invariant(_) => EcbErrorKind::Invariant,
        }
    }

    /// 是否值得重试。本层无网络，除 `Invariant` 外一律 `false`。
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self.kind(), EcbErrorKind::Invariant)
    }
}

/// 本 crate 统一结果别名。
pub type EcbResult<T> = Result<T, EcbError>;

#[cfg(test)]
mod tests {
    use super::{EcbError, EcbErrorKind};

    fn samples() -> Vec<EcbError> {
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
    fn kind_maps_every_variant_distinctly() {
        let kinds: Vec<EcbErrorKind> = samples().iter().map(EcbError::kind).collect();
        let expected = [
            EcbErrorKind::Invalid,
            EcbErrorKind::Missing,
            EcbErrorKind::AuthorizationDenied,
            EcbErrorKind::RoutedElsewhere,
            EcbErrorKind::WriteAuthorityDenied,
            EcbErrorKind::SemanticallyRejected,
            EcbErrorKind::NotApplicable,
            EcbErrorKind::Invariant,
        ];
        assert_eq!(kinds, expected);
        // 四类关键拒绝必须彼此可区分（授权 / 路由 / 越权 / 语义）。
        let key = [
            EcbErrorKind::AuthorizationDenied,
            EcbErrorKind::RoutedElsewhere,
            EcbErrorKind::WriteAuthorityDenied,
            EcbErrorKind::SemanticallyRejected,
        ];
        for (i, a) in key.iter().enumerate() {
            for b in &key[i + 1..] {
                assert_ne!(a, b, "关键拒绝分类不得互相混同");
            }
        }
    }

    #[test]
    fn only_invariant_is_retryable() {
        for error in samples() {
            let expected = error.kind() == EcbErrorKind::Invariant;
            assert_eq!(error.is_retryable(), expected, "{error:?}");
        }
    }

    #[test]
    fn display_is_non_empty_for_every_variant() {
        for error in samples() {
            assert!(!error.to_string().is_empty(), "{error:?}");
            assert!(!format!("{error:?}").is_empty(), "{error:?}");
        }
    }

    #[test]
    fn messages_do_not_echo_credentials_or_source_lines() {
        // 错误消息只承载可读理由，不回显原始响应正文 / 凭据 / 整行配置源码。
        let error = EcbError::SemanticallyRejected("维度 DIM_A 重复".to_owned());
        let text = error.to_string();
        assert!(text.contains("维度 DIM_A 重复"));
        assert!(!text.contains("password"), "{text}");
        assert!(!text.contains("token"), "{text}");
        assert!(!text.contains('='), "错误消息不得回显配置源码行：{text}");
    }
}
