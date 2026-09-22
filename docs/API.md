# ecbx 公开 API

> 本文件列出 `ecbx` 的公开面：类型 / 方法 + 一行语义。
> 公开面**不**声称任何能力等级、SLA、新鲜度保证或生产就绪。

## 错误模型

| 项 | 语义 |
| --- | --- |
| `EcbErrorKind` | 错误分类：`Invalid` / `Missing` / `AuthorizationDenied` / `RoutedElsewhere` / `WriteAuthorityDenied` / `SemanticallyRejected` / `NotApplicable` / `Invariant` |
| `EcbError` | 类型化错误，`#[non_exhaustive]`，消息为简体中文且不回显原文 |
| `EcbError::kind()` | 分类，供调用方按「如何反应」分流 |
| `EcbError::is_retryable()` | 本层无网络，除 `Invariant` 外一律 `false` |
| `EcbResult<T>` | `Result<T, EcbError>` |

## 数据类型与协议

| 项 | 语义 |
| --- | --- |
| `EcbDataType` | 清单 §1.2 的 7 类数据类型 |
| `EcbDataType::ALL` / `code` / `description` / `from_code` | 全集、类型码、中文说明、按码解析 |
| `EcbDataType::is_curve_product` | `ZERO_SPOT` / `FORWARD` 为曲线点 |
| `EcbProtocol` | SDMX-JSON / SDMX-ML / CSV **协议名**（不含端点） |
| `EcbProtocol::ALL` / `name` / `from_name` | 全集与名称互转 |

## 身份形

| 项 | 语义 |
| --- | --- |
| `EcbDimension` | 单个维度：`id` + `value` |
| `EcbDimension::try_new` / `key` | 构造校验 / `id=value` 片段 |
| `EcbDataflowId` | `dataflow` + `dsd` + 有序列出 `dimensions` |
| `EcbDataflowId::try_new` | 构造校验（非空、至少一维、键不重复） |
| `EcbDataflowId::declared_dimension_key` | 规范化维度键（保持声明顺序） |
| `EcbSeriesIdentity` | `dataflow` + `indicator` + `subject` |
| `EcbSeriesIdentity::try_new` / `series_id` | 构造校验 / 规范身份串 |
| `validate_dataflow_id` | 校验 dataflow 身份形 |
| `validate_series_identity` | 校验序列身份 |

## 观测值对象

| 项 | 语义 |
| --- | --- |
| `Unit` | 源侧单位原文（保留源单位，不做换算） |
| `Unit::try_new` / `text` | 构造校验 / 原文 |
| `EcbMissingReason` | 缺失原因：`SourceStatus(原文)` / `Unspecified` |
| `EcbValue` | `Value(f64)` / `Missing(EcbMissingReason)` |
| `EcbValue::value` / `is_missing` / `missing_reason` | 取值 / 是否缺失 / 缺失原因 |
| `EcbObservation` | 一条源事实观测（身份 + 类型 + 频率 + 期间 + 值 + 单位 + 可选 vintage） |
| `EcbObservation::new` | 构造并校验 |
| `EcbObservation::source_series_id` | 完整源身份串（含期间与 vintage） |
| `validate_observation` | 校验观测完整性与内部一致性 |
| `Date` | 自有日期（严格 `YYYY-MM-DD`，校验闰年） |
| `Date::new` / `parse` / `to_iso` / `year` / `month` / `day` / `is_leap_year` / `days_in_month` | 日期构造与解析 |
| `Period` | `Day(Date)` / `Month` / `Quarter` / `Year` / `Event` |
| `Period::day` / `month` / `quarter` / `year` / `event` / `parse` / `key` / `frequency` | 期间构造、解析、规范键与频率映射 |
| `Frequency` | `Daily` / `Weekly` / `Monthly` / `Quarterly` / `Annual` / `Event` / `Irregular` |
| `Frequency::ALL` / `name` / `from_name` | 全集与名称互转 |

## 守卫（fail-closed）

| 项 | 语义 |
| --- | --- |
| `ensure_not_curve_product` | 曲线点 → `RoutedElsewhere` |
| `reject_curve_promotion` | 曲线点观测不得晋级 |
| `attest_official_dimension_order` | 恒 `NotApplicable`：不得把声明顺序断言为官方顺序 |
| `claim_fiscal_write_authority` | 恒 `WriteAuthorityDenied`：主权 `pending` |

## 授权判定

| 项 | 语义 |
| --- | --- |
| `EcbAuthorization` | `Authorized { scope }` / `Denied { reason }` |
| `EcbAuthorization::is_authorized` / `denial_reason` | 是否放行 / 拒绝理由 |
| `EcbAuthorizationEvidence` | 证据的离线登记形态（签署者 / 范围 / 有效期可「不明」） |
| `EcbAuthorizationEvidence::new` | 构造一条证据登记（不做放行判断） |
| `decide_authorization` | fail-closed 判定 |
| `registered_evidence` | 本源的证据登记：恒 `None` |
| `current_authorization` | 本源当前判定：恒 `Denied` |
| `ensure_authorized` | `Denied` → `AuthorizationDenied` 错误 |

## publication 语义

| 项 | 语义 |
| --- | --- |
| `TimePrecision` | `Date` / `Instant` |
| `AvailabilityEvidence` | `Official` / `Calendar` / `Inferred` |
| `PitEligibility` | `Formal` / `NotEligible` |
| `publication_semantics` | 恒 `(Date, Inferred, NotEligible)` |
| `publication_for_period` | 三元组随期间一并返回（期间不参与判定） |

## 离线解析

| 项 | 语义 |
| --- | --- |
| `parse_ecb_observations` | JSON 文本 → 观测集合；未知字段原子失败、重复身份拒绝 |
