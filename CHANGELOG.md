# Changelog — ecbx

本文件记录 `ecbx` 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)
与 [Semantic Versioning](https://semver.org/)。

## [Unreleased]

## [0.1.1] - 2026-09-23

### 修正

- 修复宏观数据源对抗审查发现的数值与身份边界缺陷
- 公开构造器与校验入口拒绝 NaN、正无穷、负无穷；新增构造与字段修改后的回归用例。

## [0.1.0] - 2026-09-22

### Added

- 数据类型枚举 `EcbDataType`：清单 §1.2 的 7 类
  （`ZERO_SPOT` / `FORWARD` / `POLICY_RATE` / `RFR_OVERNIGHT` / `CB_BALANCE_SHEET` /
  `MONEY_STOCK` / `DEPOSIT_FACILITY`），含 `is_curve_product`。
- 协议名的枚举 `EcbProtocol`（SDMX-JSON / SDMX-ML / CSV）；**只有名，不含端点**。
- 身份形值对象：`EcbDimension` / `EcbDataflowId` / `EcbSeriesIdentity`，
  规范身份串为 `dataflow:DSD:维度键:indicator:subject:period:vintage`。
- 观测值对象：`EcbObservation` / `EcbValue` / `EcbMissingReason` / `Unit`；
  缺失值以具名原因表达，**不静默转 0**；`vintage` 无源事实时为 `None`。
- 自有期间类型 `Date` / `Period` / `Frequency`，严格 `YYYY-MM-DD` 解析，不引入日期库。
- 离线解析入口 `parse_ecb_observations`：未知字段原子失败、重复身份拒绝、
  要求显式合成样本标注。
- 校验入口 `validate_dataflow_id` / `validate_series_identity` / `validate_observation`。
- fail-closed 守卫：`ensure_not_curve_product` / `reject_curve_promotion`（曲线点不晋级）、
  `attest_official_dimension_order`（拒绝冒充官方 code list）、
  `claim_fiscal_write_authority`（ECB / MOF 财政类写入主权 `pending`）。
- 授权判定：`EcbAuthorization` / `decide_authorization` / `current_authorization` /
  `ensure_authorized`；本源 `authorization = unknown`，判定恒 `Denied`。
- publication 语义 `publication_semantics`：恒为 `Date` + `Inferred` + `NotEligible`。
- 三类测试（`tests/tdd_contracts.rs` / `tests/sdd_spec.rs` / `tests/aidd_boundary.rs`）
  与合成夹具、`benches/hot_path.rs` 微基准。

### Notes

- 本版本**不构成任何生产授权**：`production_decision = NO-GO`，无 Owner 签核文件。
- 夹具全部为合成样本，不是真实源数据，不构成证据。
