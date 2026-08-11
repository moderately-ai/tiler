---
schema: "tiler-doc/v1"
id: "tiler.portal.experiments"
kind: "portal"
title: "Experiment catalog"
topics: ["experiments", "evidence"]
---

# Experiment catalog

Spikes are preserved executable evidence for bounded questions. They are not a
production implementation or a universal guarantee. `Reproducible` means the
checked-in procedure is complete under its recorded prerequisites; it does not
mean dependency-free, hermetic, or portable to every host.

## Running a spike

Nothing runs these automatically. The repository's `make` targets cover
`crates/` and `prototypes/` only, so a spike is exercised by whoever is working
on it, from its own directory.

A spike that is a Cargo workspace uses plain `cargo`, or the entrypoint beside
it — `spikes/shapes/nightly-dependent-static-shapes/check.sh` recompiles that
spike's retained `trybuild` goldens. These directories sit under the repository
root, so rustup resolves the same `rust-toolchain.toml` pin without a selector.

Two Cargo-workspace Kani spikes sit outside that last sentence, and their READMEs record why rather than leaving a reader to discover it: [Kani bounded verification of inexhaustible identity encoders](verification/kani-encoder-injectivity/README.md) and [Kani bounded verification of `push_slice` framing](verification/kani-push-slice-framing/README.md) are driven by `cargo kani`, which selects the nightly its own release bundles and ignores `rust-toolchain.toml` — measured from inside this repository, where the failure diagnostic reads `this compiler was built on 2025-11-20`, Kani 0.67.0's bundle rather than the pin. They are therefore not run by a clone alone: `cargo install --locked kani-verifier` plus `cargo kani setup` is a host toolchain addition, which the predecessor README records as authorized by pointing at the ticket's trigger check log rather than at anything checkable from this tree; the `push_slice` spike reuses that installation and says not to install or update anything. Three of the predecessor's nine harnesses are checked in as the record of what was attempted rather than as a suite to run, so a bare `cargo kani` there does not terminate.

A spike with a Python harness has no repository-managed interpreter; there is no
project virtual environment and no locked development dependency set. Run one
with `uv`, which fetches what it needs per invocation:

```sh
uv run --with pytest pytest spikes/embedding
uv run --with mpmath python spikes/numerics/check_witnesses.py
```

Most harnesses are standard library plus `pytest`. Five are not, and each says so in its own README: `spikes/numerics/check_witnesses.py` and `spikes/numerics/region_accuracy_probe.py` need `mpmath`; `spikes/program-planning/qwen3-conformance-fixture` and `spikes/program-planning/qwen3-corpus-reachability` each pin their own locked `torch` and `transformers` environment, because there the dependency *is* the evidence and a floating resolution would silently re-baseline the retained digests; and `spikes/numerics/qwen3-weight-quantization-profiles` needs `numpy`, `torch`, and `transformers` from the host interpreter and deliberately pins none of them, because every reading it takes is a difference against a baseline it recomputes in the same process.

The rows below are **maintained by hand**, like the [research catalog](../docs/research/README.md)'s and for the same reason: the renderer that once produced this block from frontmatter is gone, and the [metadata contract](../docs/document-metadata.md#validation-and-catalog-updates) makes editing the affected row part of the change that edits the record behind it.

<!-- BEGIN EXPERIMENT CATALOG -->
### Foundation, semantics, and extensions

- [Index and access-model experiment](indexing/README.md) — reproducible; executable-model; supports: [Symbolic index and access model](../docs/research/indexing/index-access-model.md)
- [Nightly dependent-array static-shape conformance](shapes/nightly-dependent-static-shapes/README.md) — reproducible; executable-model, bounded-measurement; supports: [Nightly arbitrary-rank const shape parameters](../docs/research/shapes/nightly-const-shape-parameters.md)
- [Normative reference-evaluator experiment](reference/README.md) — reproducible; executable-model; supports: [Normative reference evaluator slice](../docs/research/reference/normative-reference-slice.md)
- [Operation-extension experiments](extensions/README.md) — reproducible; executable-model, bounded-measurement; supports: [Operation-extension surface research](../docs/research/extensions/operation-extension-surface.md), [Experimental operation API sketch](../docs/research/extensions/operation-extension-api.md), [Proc-macro visibility of operation extensions](../docs/research/extensions/proc-macro-extension-visibility.md), [Consumer-neutral backend-provider composition](../docs/research/extensions/backend-provider-composition.md)
- [Semantic foundation API v2 compile-checking spike](extensions/semantic-foundation-api-v2/README.md) — reproducible; executable-model; supports: [Corrected semantic foundation API](../docs/research/extensions/semantic-foundation-api-v2.md)
- [Stable-Rust shape-evidence feasibility spike](shapes/shape-evidence/README.md) — reproducible; executable-model, bounded-measurement; supports: [Stable-Rust shape-evidence feasibility](../docs/research/shapes/stable-rust-shape-evidence.md), [Public static-shape evidence spelling](../docs/research/shapes/public-static-shape-spelling.md)

### Numerical operations

- [Metal runtime precise exponential at signed zero](apple-targets/exp-at-zero-runtime-probe/README.md) — reproducible; bounded-measurement; supports: [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md), [The tree-fold form of the online-softmax rescaling bound](../docs/research/numerics/tree-fold-online-softmax-bound.md)
- [Metal emitted-evaluation-order probe](apple-targets/evaluation-order-probe/README.md) — reproducible; bounded-measurement; supports: [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md), [The oracle for a permitted-divergence candidate](../docs/research/reference/permitted-divergence-oracle.md)
- [BF16 through the second-dtype seams](numerics/bf16-second-dtype/README.md) — reproducible; executable-model, exhaustive-finite, bounded-measurement; supports: [Mature tensor dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md), [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md), [BF16 computation, accumulator, and conversion](../docs/research/numerics/bf16-computation-accumulator-and-conversion.md)
- [The delivered-realization record, redesigned from typed evidence](numerics/delivered-realization-record/README.md) — reproducible; executable-model, exhaustive-finite; supports: [The delivered numerical realization record](../docs/research/numerics/delivered-numerical-realization-record.md)
- [Elementary-identity folding probe](numerics/elementary_identity_folding/README.md) — reproducible; bounded-measurement; supports: [The elementary-identity rewrite dimension](../docs/research/numerics/elementary-identity-rewrite-dimension.md)
- [Metal transcendental emission probe](numerics/metal_transcendental_emission/README.md) — reproducible; bounded-measurement; supports: [Transformer non-linear, normalization, and reduction contracts](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md)
- [Online-softmax rescaling bound probe](numerics/online_softmax_bound/README.md) — reproducible; bounded-measurement; supports: [Certified rounding-error bounds as rewrite permissions](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md)
- [Tree-fold online-softmax bound probe](numerics/online_softmax_tree_bound/README.md) — reproducible; bounded-measurement; supports: [The tree-fold form of the online-softmax rescaling bound](../docs/research/numerics/tree-fold-online-softmax-bound.md)
- [Reduction contract probe](numerics/reduction_contract/README.md) — reproducible; executable-model, exhaustive-finite; supports: [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md)
- [Region accuracy observation probe](numerics/region_accuracy/README.md) — reproducible; bounded-measurement; supports: [Region accuracy contracts and analyzable error budgets](../docs/research/numerics/region-accuracy-contract.md)
- [Qwen3-0.6B-Base candidate quantization profile probe](numerics/qwen3-weight-quantization-profiles/README.md) — reproducible; bounded-measurement; supports: [First quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md)
- [Sound accuracy probe](numerics/sound_accuracy/README.md) — reproducible; executable-model, bounded-measurement; supports: [Sound region-accuracy analyzer integration spike](../docs/research/numerics/sound-region-analyzer-spike.md), [Region accuracy contracts and analyzable error budgets](../docs/research/numerics/region-accuracy-contract.md)
- [Transformer reference-semantics probe](numerics/transformer_reference_semantics/README.md) — reproducible; bounded-measurement; supports: [Transformer non-linear, normalization, and reduction contracts](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md)

### Physical planning and lowering

- [Bootstrap cost-model experiment](cost-model/README.md) — reproducible; executable-model; supports: [Initial cost model and calibration plan](../docs/research/cost-model/bootstrap-cost-model.md)
- [C1 attention-block reference probe](program-planning/attention-block-reference/README.md) — reproducible; bounded-measurement; supports: [First attention program vertical](../docs/research/program-planning/first-attention-program-vertical.md)
- [Bounded scalar CPU backend vertical](target-profiles/scalar-cpu-vertical/README.md) — reproducible; executable-model, bounded-measurement; supports: [Target profiles and phased physical feasibility](../docs/research/target-profiles/physical-feasibility-model.md), [Consumer-neutral backend-provider composition](../docs/research/extensions/backend-provider-composition.md)
- [Region-search experiments](region-search/README.md) — reproducible; exhaustive-finite, executable-model; supports: [Exhaustive fusion-region oracle](../docs/research/region-search/exhaustive-region-oracle.md), [The rewrite-search formalism](../docs/research/region-search/rewrite-search-formalism.md)
- [How kernel-program identity grows against its 64 MiB bound](program-planning/identity-growth/README.md) — reproducible; bounded-measurement, executable-model; supports: [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md)
- [Kernel-program planning experiment](program-planning/README.md) — reproducible; executable-model; supports: [KernelProgram and conservative buffer planning](../docs/research/program-planning/kernel-program-buffer-plan.md)
- [Metal contraction realization probe](scheduling/metal_contraction_vertical/README.md) — reproducible; bounded-measurement, executable-model; supports: [First Metal contraction realizations](../docs/research/scheduling/first-metal-contraction-realizations.md)
- [Qwen3-0.6B-Base C1 conformance and attribution reference fixture](program-planning/qwen3-conformance-fixture/README.md) — reproducible; bounded-measurement, executable-model; supports: [First Metal language-model workload profile](../docs/research/program-planning/first-metal-lm-workload.md), [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md), [Model-level correctness and performance qualification](../docs/research/program-planning/model-level-qualification.md)
- [Qwen3-0.6B-Base conformance-corpus reachability probe](program-planning/qwen3-corpus-reachability/README.md) — reproducible; exhaustive-finite, bounded-measurement; supports: [Model-level correctness and performance qualification](../docs/research/program-planning/model-level-qualification.md), [First Metal language-model workload profile](../docs/research/program-planning/first-metal-lm-workload.md)
- [Scheduled-region model experiment](scheduling/README.md) — reproducible; executable-model; supports: [First-class scheduled-region model](../docs/research/scheduling/scheduled-region-model.md)
- [Structured kernel-IR verifier experiment](kernel-ir/README.md) — reproducible; executable-model; supports: [Structured kernel IR and verifier boundary](../docs/research/kernel-ir/structured-kernel-ir-verifier.md)
- [What grid-axis extent this Apple9 macOS row actually dispatches](target-profiles/metal-grid-axis-extent/README.md) — reproducible; bounded-measurement, exhaustive-finite; supports: [Authority ledger for the first macOS Metal compile profile](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)
- [Where a parallel-reduction crossover could be measured](program-planning/reduction-crossover/README.md) — reproducible; bounded-measurement; supports: [Authority ledger for the first macOS Metal compile profile](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)
- [Where the parallel-reduction crossover is, measured on the device](program-planning/reduction-dispatch-crossover/README.md) — reproducible; bounded-measurement; supports: [Authority ledger for the first macOS Metal compile profile](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)
- [Reduction partition calibration and the current tree-width excursion, measured on the device](program-planning/reduction-partition-calibration/README.md) — reproducible; bounded-measurement; supports: [Authority ledger for the first macOS Metal compile profile](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)

### Artifacts, build, and toolchains

- [Apple Metal target compatibility and numerical spikes](apple-targets/README.md) — reproducible; bounded-measurement; supports: [Apple Metal artifact compatibility](../docs/research/apple-targets/artifact-compatibility.md), [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md), [Authority ledger for the first macOS Metal compile profile](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md), [The oracle for a permitted-divergence candidate](../docs/research/reference/permitted-divergence-oracle.md)
- [Artifact envelope spike](artifacts/README.md) — reproducible; executable-model; supports: [Target-neutral artifact and backend payload envelope](../docs/research/artifacts/target-neutral-artifact-envelope.md)
- [Embedded-artifact cost and self-containment probes](embedding/README.md) — reproducible; bounded-measurement; supports: [Direct embedded-artifact costs across Rust crates](../docs/research/embedding/embedded-artifact-costs.md), [Self-contained byte-literal embedding under Cargo and rust-analyzer](../docs/research/embedding/self-contained-embedding.md)
- [Expansion cache crash and race spike](cache/README.md) — reproducible; executable-model, bounded-measurement; supports: [Expansion cache crash and race protocol](../docs/research/cache/crash-and-race-protocol.md), [Bounded expansion cache collection and accounting](../docs/research/cache/bounded-collection.md), [Supported expansion cache filesystems](../docs/research/cache/supported-filesystems.md), [The expansion cache under Cargo and rust-analyzer](../docs/research/cache/build-tool-exercise.md)
- [Expansion cache envelope-section digest coverage probe](cache/envelope-digest-coverage/README.md) — reproducible; exhaustive-finite, executable-model; supports: [Expansion cache hot-path efficiency](../docs/research/cache/hot-path-efficiency.md)
- [Expansion cache hot-path efficiency probe](cache/hot-path-efficiency/README.md) — reproducible; bounded-measurement; supports: [Expansion cache hot-path efficiency](../docs/research/cache/hot-path-efficiency.md)
- [Kani bounded verification of inexhaustible identity encoders](verification/kani-encoder-injectivity/README.md) — reproducible; executable-model, bounded-measurement; supports: [Kani bounded verification of inexhaustible identity encoders](../docs/research/verification/kani-bounded-encoder-verification.md)
- [Kani bounded verification of `push_slice` framing](verification/kani-push-slice-framing/README.md) — reproducible; executable-model, bounded-measurement; supports: [Kani bounded verification of `push_slice` framing](../docs/research/verification/kani-push-slice-framing.md)
- [Proc-macro environment and artifact-family spikes](macro-environment/README.md) — reproducible; bounded-measurement; supports: [Proc-macro build environment and freshness](../docs/research/macro-environment/proc-macro-build-environment.md)
- [What validating one artifact envelope allocates](artifacts/decoder-allocation/README.md) — reproducible; bounded-measurement; supports: [Artifact decoder allocation amplification](../docs/research/artifacts/decoder-allocation-amplification.md)
- [Which landings moved the artifact envelope's fixed content](artifacts/manifest-growth-attribution/README.md) — reproducible; bounded-measurement; supports: [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md)

### Runtime, integration, and placement

- [Placement and memory-domain model](placement/README.md) — reproducible; executable-model; supports: [Device placement and memory-domain contract](../docs/research/placement/device-placement-and-memory-domains.md)
- [Runtime execution and validation spikes](runtime/README.md) — reproducible; executable-model, bounded-measurement; supports: [Consumer-neutral runtime execution contract](../docs/research/runtime/runtime-execution-contract.md), [Semantic validation enforcement](../docs/research/runtime/semantic-validation-enforcement.md), [Candle Metal post-wait error checking](../docs/research/runtime/candle-metal-post-wait-error-checking.md)
- [Dynamic KV physical-layout comparison](runtime/dynamic-kv-layout/README.md) — reproducible; bounded-measurement, executable-model; supports: [Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md)
- [Inline regions dispatched on Metal hardware](runtime/inline-dispatch/README.md) — reproducible; executable-model, bounded-measurement; supports: [Consumer-neutral runtime execution contract](../docs/research/runtime/runtime-execution-contract.md)
- [Transfer synchronization and lifetime model](transfers/README.md) — reproducible; executable-model; supports: [Transfer synchronization and resource-lifetime contract](../docs/research/transfers/transfer-synchronization-and-resource-lifetime.md)

<!-- END EXPERIMENT CATALOG -->

Each experiment entry identifies its supported research claim, exact entry
point, prerequisites, retained results, and measurement boundary. Generated
local caches remain ignored; cited fixtures remain tracked.
