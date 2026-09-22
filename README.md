# ecbx

`ecbx` 是 ECB（欧洲央行）**源事实的类型层**：把清单里已钉死的 ECB 采集范围落成可机械
校验的 Rust 类型，并提供离线解析与 fail-closed 授权判定。

- 7 类数据类型的枚举（`ZERO_SPOT` / `FORWARD` / `POLICY_RATE` / `RFR_OVERNIGHT` /
  `CB_BALANCE_SHEET` / `MONEY_STOCK` / `DEPOSIT_FACILITY`）
- SDMX 身份形 `dataflow + DSD + 规范化维度键`，再叠加 `indicator + subject + period + vintage`
- 自有 `Date` / `Period` / `Frequency` / `Unit` 值对象（不引入任何日期库）
- 离线 JSON 解析：**未知字段原子失败**、**重复身份拒绝**、缺失值必须具名
- 三条 fail-closed 守卫：曲线点不得在本库晋级、手填维度顺序不得冒充官方 code list、
  `unknown` 授权一律 `Denied`
- 零 HTTP 依赖、零端点字面量、零凭据读取、零跨仓依赖

## 安装

本 crate **不发布到 crates.io**，通过 git 依赖引入：

```toml
[dependencies]
ecbx = { git = "https://github.com/bytechainx/ecbx" }
```

## 用法示例

构造一条观测并做校验：

```rust,no_run
use ecbx::{
    EcbDataType, EcbDataflowId, EcbDimension, EcbObservation, EcbSeriesIdentity, EcbValue,
    Frequency, Period, Unit,
};

fn main() -> Result<(), ecbx::EcbError> {
    let dataflow = EcbDataflowId::try_new(
        "YC",
        "DSD_SYNTH",
        vec![EcbDimension::try_new("DIM_A", "VAL_A")?],
    )?;
    let series = EcbSeriesIdentity::try_new(dataflow, "IND_SYNTH", "SUBJ_SYNTH")?;
    let observation = EcbObservation::new(
        series,
        EcbDataType::PolicyRate,
        Frequency::Daily,
        Period::parse("2026-09-18")?,
        EcbValue::Value(3.25),
        Unit::try_new("SYNTH_UNIT")?,
        None,
    )?;
    assert_eq!(observation.value.value(), Some(3.25));
    Ok(())
}
```

离线解析（参数只有文本，**不接受** URL、客户端或认证信息）：

```rust,no_run
use ecbx::parse_ecb_observations;

fn main() -> Result<(), ecbx::EcbError> {
    let document = r#"{ "_synthetic": true, "_note": "合成样本", "observations": [] }"#;
    let observations = parse_ecb_observations(document)?;
    assert!(observations.is_empty());
    Ok(())
}
```

## 主要内容

| 类型 / 函数 | 作用 |
| --- | --- |
| `EcbDataType` | 清单 §1.2 的 7 类数据类型，含 `is_curve_product` |
| `EcbProtocol` | SDMX-JSON / SDMX-ML / CSV **协议名**（不含端点） |
| `EcbDimension` / `EcbDataflowId` / `EcbSeriesIdentity` | 身份形与维度键 |
| `EcbObservation` / `EcbValue` / `EcbMissingReason` / `Unit` | 观测值对象（缺失必须具名） |
| `Date` / `Period` / `Frequency` | 自有期间类型，严格 ISO 解析 |
| `parse_ecb_observations` | 离线 JSON → 观测集合 |
| `validate_dataflow_id` / `validate_series_identity` / `validate_observation` | 校验入口 |
| `ensure_not_curve_product` / `reject_curve_promotion` | 曲线点不得在本库晋级 |
| `attest_official_dimension_order` | 恒拒绝「手填顺序即官方顺序」的断言 |
| `claim_fiscal_write_authority` | 恒拒绝：ECB / MOF 财政类写入主权为 `pending` |
| `EcbAuthorization` / `decide_authorization` / `current_authorization` | fail-closed 授权判定 |
| `publication_semantics` | 恒为 `Date` + `Inferred` + `NotEligible` |

## 非目标

- **不实现联网采集**：不写 HTTP / SDMX 客户端，不含任何端点字面量、Header 或限流数字
- **不实现官方元数据**：SDMX dataflow/DSD **非**官方表；属性键白名单 ≠ 元数据已实现
- **不把曲线点晋级进内核**：`ZERO_SPOT` / `FORWARD` 须经 `yieldx` 映射批准
- **不做派生指标**：净流动性、利差、Credit Impulse、z-score 一律归 analytics
- **不做单位换算**：保留源单位，换算归下游
- **不读凭据 / 环境变量**：本层只做离线解析
- **不建共享 core crate**：公共形状由 `specs/005-*/contracts/` 冻结，各库各自实现一遍

## 诚实边界

`production_decision = NO-GO`。清单 COMPLETE ≠ ship；authorization ≠ Production Ready。
本源**无 Owner 签核文件**（`authorization = unknown`），因此 `current_authorization` 恒返回
`Denied`——本库不假装有签核，也不以「清单里写了 approving」替代证据。

## 门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

全部用例离线运行，不访问任何外部服务。

## 许可

MIT OR Apache-2.0
