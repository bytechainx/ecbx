# 贡献指南 — ecbx

## 门禁四件套（提交前必须全绿）

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

> `cargo test` 不加 `--all-targets`：那会漏掉 doctest。

共享 target 目录偶发 `Blocking waiting for file lock`，属正常现象；**不要**删除 target，
**不要**杀掉正在编译的进程。

## 测试分层（固定文件名与头部标记）

| 文件 | 头部标记 | 要求 |
| --- | --- | --- |
| `tests/tdd_contracts.rs` | `// TDD-PROBE:` | 四列表（入口 / 变异 / 红 / 绿），覆盖全部公开入口，无空列；变异须在 `/tmp` 副本上实跑过红绿 |
| `tests/sdd_spec.rs` | `// SPEC-MAP:` | 与 `docs/标准.md` 的 `##` 章节 1:1，无孤儿、无遗漏 |
| `tests/aidd_boundary.rs` | `// AIDD:` | ≥5 条，五列（边界 / 来源 / 复核 / 依据 / 结论）全部非空 |

三类测试都不依赖外部服务。

## 单元测试与源码同文件

组织 P0 要求：单元测试与源码同文件（`#[cfg(test)] mod tests { … }`）。
**禁止**把测试段移出源文件。

## 提交约定

- 变更走 feature branch → PR → review → merge；**禁止**直接 push `main`
- 提交信息说明「改了什么、为什么」，并附门禁实跑结果
- 不引入 HTTP 客户端、异步运行时、日期库、随机数库或任何 `bytechainx/*` crate
- 不写端点字面量、Header、限流数字、凭据字段
- 夹具必须带 `_synthetic: true` 与 `_note`，且不得被表述为「实测」「核验 PASS」

## 代码约定

- 注释、文档、错误消息使用简体中文；标识符保持英文
- 错误统一为 `EcbError`（`#[non_exhaustive]`），经 `kind()` 分类，不靠字符串匹配
- 非测试代码禁止 `unwrap()` / `expect()` / `panic!` / `println!`（lint 已 deny）
- 生产段行数目标 < 450 行；单文件生产段 > 800 行判 ERROR
- 公开项必须有文档（`#![deny(missing_docs)]`），只公开必要接口
