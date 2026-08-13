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
entrypoints: ["spikes/runtime/runtime_execution_contract.rs", "spikes/runtime/semantic_validation_enforcement.rs", "spikes/runtime/measure_semantic_validation.py", "spikes/runtime/candle_metal_post_wait.rs", "spikes/runtime/check_candle_post_wait_source.py", "spikes/runtime/inline-dispatch/README.md", "spikes/runtime/dynamic-kv-layout/README.md", "spikes/runtime/backend-provider-portfolio/README.md"]
last_verified: "2026-08-04"
ticket: "runtime-execution-contract"
---

# Runtime execution and validation spikes

## Standard Metal, custom Metal, and CPU in one portfolio

[`backend-provider-portfolio/`](backend-provider-portfolio/README.md) compiles one semantic program with `CompileRequest::with_physical_providers`, assembles Metal through `accept_or_publish_metal_plan` and CPU through `assemble_plan_artifact`, packages both under one variant-level `TargetProfileRef`, and routes separate explicit Metal and CPU attempts through `route_with_adapter`. There is no `BackendProvider` bundle and no family fallback. Cross-family preflight presents each family's own artifact under the other environment and refuses as `UnsupportedRepresentation`.

```sh
cd spikes/runtime/backend-provider-portfolio
CARGO_TARGET_DIR=./target cargo run -- results/2026-08-12-macos-arm64.json
```

## Dynamic KV physical layouts on Metal

[`dynamic-kv-layout/`](dynamic-kv-layout/README.md) compares exact-live
head-major, capacity-strided head-major, and sequence-major storage at the exact
C1 and B1 extents. It separates payload address-walk time from physical-buffer
allocation policy and retains independently failing address oracles for all
three representations.

## Inline region dispatch on Metal hardware

[`inline-dispatch/`](inline-dispatch/README.md) is the one experiment in this
directory that binds a real device. An out-of-tree consumer crate writes
`tiler::tensor! { … deliver macos; … }` invocations, implements
`tiler::value::DispatchAdapter` and `tiler::runtime::adapter::RuntimeAdapter`
against the facade alone, and receives what a Metal kernel wrote — compared
bit for bit against the consumer's own `f32` arithmetic. It holds two consumers
sharing one adapter: a pointwise region reaching `1/1 entry(ies) encoded`, and
a reduction whose *selected* plan needs two entries, which counts them from the
consumer's side and watches a back-to-front reordering return a wrong answer
before it trusts the ordered run. Its README carries the exact invocations, the
host and toolchain per run, both transcripts, every perturbation watched
failing, and the ADR 0079 admission for its single `unsafe` site. It is a
separate Cargo workspace and no `make` target reaches it; run it by hand from
its own directory.

## Dependency-free control and accounting models

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
uv run python spikes/runtime/measure_semantic_validation.py
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
uv run python spikes/runtime/check_candle_post_wait_source.py \
  /path/to/candle
```

The audit itself rejects any checkout that is not exactly at the pinned commit
or has tracked/untracked changes, before inspecting the expected source path.
The workspace helper may reuse an existing checkout, but provenance is not left
to a manual confirmation. This remains source evidence, not real-GPU fault
injection.
