---
schema: "tiler-doc/v1"
id: "tiler.spike.runtime"
kind: "experiment"
title: "Runtime execution and validation spikes"
topics: ["runtime", "fallback", "validation", "candle"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.runtime.execution-contract", "tiler.research.runtime.semantic-validation", "tiler.research.runtime.candle-post-wait"]
entrypoints: ["spikes/runtime/runtime_execution_contract.rs", "spikes/runtime/semantic_validation_enforcement.rs", "spikes/runtime/candle_metal_post_wait.rs", "spikes/runtime/check_candle_post_wait_source.py"]
last_verified: "2026-07-20"
ticket: "runtime-execution-contract"
---

# Runtime execution and validation spikes

These dependency-free models test one-way routing authority, exact completion,
resource retention, residual semantic validation, and Candle's post-wait error
transition. Run from the repository root:

```sh
rustc --edition 2021 --test spikes/runtime/runtime_execution_contract.rs -o /tmp/tiler-runtime-tests && /tmp/tiler-runtime-tests
rustc --edition 2021 --test spikes/runtime/semantic_validation_enforcement.rs -o /tmp/tiler-validation-tests && /tmp/tiler-validation-tests
rustc --edition 2021 --test spikes/runtime/candle_metal_post_wait.rs -o /tmp/tiler-candle-post-wait && /tmp/tiler-candle-post-wait
```

The source audit additionally checks the exact Candle revision used by the
research report:

```sh
zsh -ic 'gwc https://github.com/huggingface/candle.git'
git -C /path/to/candle checkout --detach 31f35b147389700ed2a178ee66a91c3cc25cc80d
python3 spikes/runtime/check_candle_post_wait_source.py \
  /path/to/candle/candle-metal-kernels/src/metal/commands.rs
```

The workspace helper may reuse an existing checkout. Confirm it is at the
pinned commit before running the audit. These are control models, not real-GPU
fault injection or performance measurements.
