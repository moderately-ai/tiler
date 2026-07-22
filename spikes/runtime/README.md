---
schema: "tiler-doc/v1"
id: "tiler.spike.runtime"
kind: "experiment"
title: "Runtime execution and validation spikes"
topics: ["runtime", "fallback", "validation", "candle"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.runtime.execution-contract", "tiler.research.runtime.semantic-validation", "tiler.research.runtime.candle-post-wait"]
entrypoints: ["spikes/runtime/runtime_execution_contract.rs", "spikes/runtime/semantic_validation_enforcement.rs", "spikes/runtime/measure_semantic_validation.py", "spikes/runtime/candle_metal_post_wait.rs", "spikes/runtime/check_candle_post_wait_source.py"]
last_verified: "2026-07-21"
ticket: "runtime-execution-contract"
---

# Runtime execution and validation spikes

These dependency-free models test one-way routing authority, exact completion,
resource retention, residual semantic validation, and Candle's post-wait error
transition. They are bounded control/accounting models, not GPU performance
measurements. Run from the repository root:

```sh
rustc --edition 2021 --test spikes/runtime/runtime_execution_contract.rs -o /tmp/tiler-runtime-tests && /tmp/tiler-runtime-tests
rustc --edition 2021 --test spikes/runtime/semantic_validation_enforcement.rs -o /tmp/tiler-validation-tests && /tmp/tiler-validation-tests
rustc --edition 2021 --test spikes/runtime/candle_metal_post_wait.rs -o /tmp/tiler-candle-post-wait && /tmp/tiler-candle-post-wait
```

Regenerate the retained semantic-validation CPU measurement through the locked
repository environment:

```sh
uv run --locked python spikes/runtime/measure_semantic_validation.py
```

[`measurements/semantic-validation.json`](measurements/semantic-validation.json)
retains every individual sample, derived medians, the compiler/host/source
fields recorded by the harness, and the 300-second subprocess-group deadline.
The host fields identify `arm64` macOS 27.0 but do not identify a hardware model
or core count. It measures only the
optimized dependency-free CPU model; Metal/CUDA coefficients remain unmeasured.

The source audit additionally checks the exact Candle revision used by the
research report:

```sh
zsh -ic 'gwc https://github.com/huggingface/candle.git'
git -C /path/to/candle checkout --detach 31f35b147389700ed2a178ee66a91c3cc25cc80d
uv run --locked python spikes/runtime/check_candle_post_wait_source.py \
  /path/to/candle
```

The audit itself rejects any checkout that is not exactly at the pinned commit
or has tracked/untracked changes, before inspecting the expected source path.
The workspace helper may reuse an existing checkout, but provenance is not left
to a manual confirmation. This remains source evidence, not real-GPU fault
injection.
