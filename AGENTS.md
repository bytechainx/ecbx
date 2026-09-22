# ecbx Agent 指南

> 本文件为 AI Agent 在本仓库工作时的入口指南。

## 项目定位

ECB 源事实的**类型层**：7 类数据类型、SDMX 身份形、离线解析、校验与 fail-closed 授权判定。
不做联网采集、不做认证、不做缓存、不做再分发、不做存储、不做单位换算、不做派生指标。

## 技术栈

- Rust edition 2021，`rust-version = 1.71`（按依赖图 `rust_version` 最大值推导，见 `CONTEXT.md`）
- 关键依赖：`thiserror` 2、`serde` 1（derive）、`serde_json` 1
- **无 path 依赖**：不依赖 `instrumentationx` 或任何 `bytechainx/*` crate
- crate 级 lint：`unwrap_used` / `expect_used` / `panic` / `unreachable` / `todo` /
  `unimplemented` 全部 `deny`（测试代码经 `cfg_attr(test)` 豁免）

## 代码结构

```text
src/
├── lib.rs               # crate 文档 + 模块声明 + 门面 pub use
├── error.rs             # EcbError / EcbErrorKind / EcbResult
├── authz.rs             # EcbAuthorization / 证据登记 / fail-closed 判定
├── pit.rs               # publication 语义三元组（恒 Date + Inferred + NotEligible）
├── parse.rs             # 离线 JSON 解析（无网络参数）
└── value.rs             # 值对象门面
    ├── value/date.rs        # Date
    ├── value/period.rs      # Period / Frequency
    ├── value/series.rs      # EcbDataType / EcbProtocol / 身份形 / 守卫
    └── value/observation.rs # Unit / EcbValue / EcbObservation
```

- 依赖方向单向：`error ← value ← {authz, pit, parse}`，无环
- `#![forbid(unsafe_code)]`、`#![deny(missing_docs)]`、`#![deny(unreachable_pub)]`

## 硬约束（改代码前先读）

| 禁止 | 原因 |
| --- | --- |
| 写任何 `https://…` 端点字面量、Header、限流数字、凭据字段 | 清单把全部端点许可登记为 UNKNOWN；规划端点 ≠ 访问合同 |
| 引入 HTTP 客户端 / 异步运行时 / `chrono` / `time` / `rand` | `FR-037`、`MR-DATA-003` |
| 读环境变量或凭据 | 本层只做离线解析 |
| 内置官方 DSD / code list / 维度顺序 | SDMX dataflow/DSD **非**官方表；属性键白名单 ≠ 元数据已实现 |
| 把 `ZERO_SPOT` / `FORWARD` 晋级为普通观测 | 曲线路由归 `yieldx` |
| 在值对象里实现派生指标或单位换算 | 派生归 analytics，换算归下游 |
| 把合成夹具写成「实测」「核验 PASS」 | 伪造证据（`FR-057`） |
| 建 `mod.rs` 或名为 `utils`/`helpers`/`common`/`manager`/`base`/`global`/`misc` 的文件 | `MR-STRUCT-003` / `MR-STRUCT-005` |
| 改 `specs/`、`docs/` 之外的共享文件或其它 crate 目录 | 越界写入 |

## 门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

## 相关文档

- 组织 Rust 规范：`~/org-config/rulesets/rust/RULES.md`
- 公共形状契约：`specs/005-macro-data-source-crates/contracts/source-library-contract.md`
- 跨源语义：`specs/005-macro-data-source-crates/contracts/cross-source-routing.md`
- 采集范围权威：`specs/adapter/ecb.md`
- API 文档：`docs/API.md`
- 标准与验收：`docs/标准.md`
- 术语与边界：`CONTEXT.md`
- 变更记录：`CHANGELOG.md`
