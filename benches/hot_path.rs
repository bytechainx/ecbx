#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! ecbx 热路径微基准：离线解析 + 身份串组合。
//!
//! 入口为 `fn main()`，依赖 `Cargo.toml` 的 `[[bench]] harness = false`。
//! 输入为**合成文档**，不含任何真实源数据。

use std::hint::black_box;
use std::time::Instant;

use ecbx::parse_ecb_observations;

/// 合成文档：2 条观测，字段名取自清单身份形。
const DOCUMENT: &str = r#"{
  "_synthetic": true,
  "_note": "本文件为特性 005 自拟的合成样本，不是真实源数据，不构成任何证据。",
  "observations": [
    {
      "dataflow": "YC",
      "dsd": "DSD_SYNTH",
      "dimensions": [
        { "id": "DIM_A", "value": "VAL_A" },
        { "id": "DIM_B", "value": "VAL_B" }
      ],
      "indicator": "IND_SYNTH",
      "subject": "SUBJ_SYNTH",
      "data_type": "POLICY_RATE",
      "freq": "Daily",
      "period": "2026-09-18",
      "value": 3.25,
      "unit": "SYNTH_UNIT",
      "vintage_date": null
    },
    {
      "dataflow": "BSI",
      "dsd": "DSD_SYNTH",
      "dimensions": [ { "id": "DIM_A", "value": "VAL_C" } ],
      "indicator": "IND_SYNTH",
      "subject": "SUBJ_SYNTH",
      "data_type": "MONEY_STOCK",
      "freq": "Monthly",
      "period": "2026-09",
      "value": null,
      "obs_status": "SYNTH_STATUS",
      "unit": "SYNTH_UNIT"
    }
  ]
}"#;

fn main() {
    let iterations: u32 = 2_000;
    // 预热，避免把首次分配算进计时。
    for _ in 0..10 {
        let _ = parse_ecb_observations(DOCUMENT).expect("合成文档应可解析");
    }

    let start = Instant::now();
    let mut identities = 0usize;
    for _ in 0..iterations {
        let observations = parse_ecb_observations(DOCUMENT).expect("合成文档应可解析");
        identities = identities.wrapping_add(
            observations
                .iter()
                .map(|observation| observation.source_series_id().len())
                .sum(),
        );
        black_box(&observations);
    }
    let elapsed = start.elapsed();

    println!(
        "bench_ecbx_parse_offline: iters={iterations} total={elapsed:?} per_iter={:?} identity_bytes={}",
        elapsed / iterations,
        black_box(identities)
    );
}
