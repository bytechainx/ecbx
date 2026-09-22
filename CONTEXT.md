# ecbx 上下文

本文件定义 `ecbx` 与其使用方共享的核心词汇与边界。它只记录领域含义与能力边界，
不记录具体实现、存储或部署决定。

## 角色与边界

**源事实类型层**：把 `specs/adapter/ecb.md` 已钉死的采集范围落成可机械校验的 Rust 类型，
并提供离线解析与 fail-closed 授权判定。
_Avoid_: ECB 客户端 / SDK（联网采集、认证、缓存、再分发都不在本层）

**离线**：只处理**字符串 → 值对象**，不持有任何网络能力、不读凭据与环境变量。
_Avoid_: 采集器（本层不发起任何请求）

**授权判定是只读结论**：判定「该源的这个范围是否被授权访问」，不改写清单或名册的任何登记值。
_Avoid_: 授权变更（本特性不改变任何授权状态）

**契约层与生产就绪无关**：`production_decision` 恒为 `NO-GO`；本层判定不为「生产就绪」背书。
_Avoid_: Production Ready

## 核心词汇

**数据类型**（`EcbDataType`）：清单 §1.2 的 7 类，逐字为 `ZERO_SPOT` / `FORWARD` /
`POLICY_RATE` / `RFR_OVERNIGHT` / `CB_BALANCE_SHEET` / `MONEY_STOCK` / `DEPOSIT_FACILITY`。
_Avoid_: 产品树 / 品类（清单只给这 7 类，不得自行扩集）

**身份形**：`dataflow + DSD + 规范化维度键`，再叠加 `indicator + subject + period + vintage`。
_Avoid_: series key / 主键（本层不用「主键」一词，避免与存储层概念混淆）

**声明顺序**（declared order）：`EcbDataflowId.dimensions` 的顺序由调用方提供，
**不是**官方 code list 顺序。清单把官方 DSD / code list 登记为 `absent`。
_Avoid_: 官方顺序（本层没有、也不接受「这是官方顺序」的断言）

**曲线点**：`ZERO_SPOT` / `FORWARD` 属 YC dataflow；本库可以持有它们，但不得晋级进内核。
_Avoid_: 曲线（曲线构建归 `yieldx`，本库只表达源事实）

**具名缺失**（`EcbMissingReason`）：缺失值必须带原因，绝不静默转 0。
_Avoid_: 空值 / 0 值（两者语义不同，混淆即静默替换）

**源单位**（`Unit`）：原样保留源侧单位文本，换算归下游。
_Avoid_: 归一化单位（本层不做换算）

## rust-version 推导

按「依赖图中所有依赖所声明 `rust-version` 的最大值」推导
（`cargo metadata --format-version 1` 的 `rust_version` 字段）。实测（2026-09-22）：

| 依赖 | 版本 | 其 `rust-version` |
| --- | --- | --- |
| `memchr` | 2.8.3 | 1.61 |
| `serde` / `serde_core` | 1.0.229 | 1.56 |
| `itoa` | 1.0.18 | 1.68 |
| `proc-macro2` / `quote` / `serde_derive` / `serde_json` / `syn` / `thiserror` / `thiserror-impl` / `unicode-ident` / `zmij` | 见 `cargo metadata` | 1.71 |

最大值 = **1.71**，故 `Cargo.toml` 声明 `rust-version = "1.71"`。
无依赖缺失 `rust_version` 的情形，无需保守取值。

## 已知缺口

1. **无 Owner 签核文件**：`authorization = unknown`，因此授权判定恒 `Denied`；
   本库不假装有证据，也不提供任何「放行」运行时路径。
2. **官方 DSD / code list 未核验**（清单登记 `absent`）：`attest_official_dimension_order`
   恒拒绝；本库不提供维度顺序权威。
3. **live SDMX 客户端未实现且被阻断**（`blocked: auth unknown`）：无 HTTP 依赖、
   无端点字面量、无 Header、无凭据、无限流数字。
4. **曲线点不晋级**：YC 曲线点进 kernel 为 `planned`，须 `yield_curve` 映射批准。
5. **名称占用**：`ecbx` 未发布到 crates.io，仅以 git / path 依赖使用。
