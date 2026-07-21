---
schema: "tiler-doc/v1"
id: "tiler.portal.decisions"
kind: "portal"
title: "Architecture decision records"
topics: ["decisions", "architecture"]
---

# Architecture decision records

ADRs record choices that constrain several components or would be expensive to
reverse. Proposed ADRs and design text remain non-decisions until explicitly
accepted. Unresolved questions are collected in
[open questions](../open-questions.md).

## Browse by theme

<!-- BEGIN GENERATED ADR TOPICS -->
### Foundation, semantics, and extensions

- [0062: Bind Rust markers to complete resolved value types](0062-bind-markers-to-resolved-value-types.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Numerical semantics](../numerical-semantics.md), [Operation extension contract](../operation-extensions.md); evidence: [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md), [Quantized value and transformation contract](../research/numerics/quantized-value-and-transform-contract.md), [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md)
- [0060: Bind Rust type markers through the explicit registry](0060-bind-rust-type-markers-through-the-registry.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Operation extension contract](../operation-extensions.md), [Numerical semantics](../numerical-semantics.md); evidence: [Operation-extension surface research](../research/extensions/operation-extension-surface.md), [Experimental operation API sketch](../research/extensions/operation-extension-api.md), [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md)
- [0045: Bound inline proc-macro providers to host dependencies](0045-bound-proc-macro-providers-to-host-dependencies.md) — accepted; contracts: [Operation extension contract](../operation-extensions.md); evidence: [Proc-macro visibility of operation extensions](../research/extensions/proc-macro-extension-visibility.md)
- [0063: Check value graph ownership at semantic admission](0063-check-graph-ownership-at-admission.md) — accepted; contracts: [IR stack and invariants](../ir.md); evidence: [Rust semantic-program construction lifecycle](../research/semantic-graph/rust-construction-lifecycle.md)
- [0064: Compact semantic programs at commitment](0064-compact-at-semantic-program-commit.md) — accepted; contracts: [System architecture](../architecture.md), [IR stack and invariants](../ir.md); evidence: [Rust semantic-program construction lifecycle](../research/semantic-graph/rust-construction-lifecycle.md), [Semantic tensor graph contract research memo](../research/semantic-graph/contract-memo.md)
- [0005: Expose a public semantic graph and extension boundary](0005-public-semantic-tensor-graph.md) — accepted; contracts: [System architecture](../architecture.md), [Vision and scope](../vision.md), [Operation extension contract](../operation-extensions.md); evidence: [Semantic tensor graph contract research memo](../research/semantic-graph/contract-memo.md), [Operation-extension surface research](../research/extensions/operation-extension-surface.md)
- [0065: Extract reference evaluation from the IR crate](0065-extract-reference-evaluation-from-ir.md) — accepted; contracts: [System architecture](../architecture.md), [IR stack and invariants](../ir.md); evidence: [Corrected semantic foundation API](../research/extensions/semantic-foundation-api-v2.md), [Normative reference evaluator slice](../research/reference/normative-reference-slice.md)
- [0061: Layer checked shape evidence over canonical typed values](0061-layer-checked-shape-evidence-over-values.md) — accepted; contracts: [IR stack and invariants](../ir.md); evidence: [Shape environment contract research memo](../research/shapes/shape-environment-contract.md), [Shape constraint prover boundary](../research/shapes/constraint-prover-boundary.md), [Stable-Rust shape-evidence feasibility](../research/shapes/stable-rust-shape-evidence.md), [Public static-shape evidence spelling](../research/shapes/public-static-shape-spelling.md), [Rust semantic-program construction lifecycle](../research/semantic-graph/rust-construction-lifecycle.md)
- [0006: Model semantic programs as operation/value graphs](0006-operation-value-graph.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Optimizer model](../compiler/optimizer.md); evidence: [Semantic tensor graph contract research memo](../research/semantic-graph/contract-memo.md)
- [0008: Separate extent symbols from typed root bindings](0008-typed-root-bindings.md) — accepted; contracts: [IR stack and invariants](../ir.md); evidence: [Shape environment contract research memo](../research/shapes/shape-environment-contract.md), [Shape constraint prover boundary](../research/shapes/constraint-prover-boundary.md)
- [0001: Separate semantic planning from physical scheduling](0001-separate-semantic-and-physical-plans.md) — accepted; contracts: [System architecture](../architecture.md), [IR stack and invariants](../ir.md), [Optimizer model](../compiler/optimizer.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md), [Vision and scope](../vision.md); evidence: [First-class scheduled-region model](../research/scheduling/scheduled-region-model.md)
- [0066: Separate semantic type authority from Rust marker bindings](0066-separate-semantic-authority-from-rust-markers.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Operation extension contract](../operation-extensions.md), [Numerical semantics](../numerical-semantics.md); evidence: [Corrected semantic foundation API](../research/extensions/semantic-foundation-api-v2.md), [Experimental operation API sketch](../research/extensions/operation-extension-api.md)
- [0058: Use a recoverable consuming semantic-program build](0058-use-a-recoverable-consuming-semantic-program-build.md) — accepted; contracts: [System architecture](../architecture.md), [IR stack and invariants](../ir.md); evidence: [Rust semantic-program construction lifecycle](../research/semantic-graph/rust-construction-lifecycle.md), [Shape environment contract research memo](../research/shapes/shape-environment-contract.md)
- [0044: Use an explicit capability-based operation registry](0044-use-explicit-capability-operation-registry.md) — accepted; contracts: [Operation extension contract](../operation-extensions.md); evidence: [Operation-extension surface research](../research/extensions/operation-extension-surface.md), [Experimental operation API sketch](../research/extensions/operation-extension-api.md)
- [0059: Use exact typed authoring handles over runtime-typed semantic values](0059-use-exact-typed-authoring-handles.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Numerical semantics](../numerical-semantics.md), [Operation extension contract](../operation-extensions.md); evidence: [Dtype resolution and mixed-precision precedent](../research/numerics/dtype-resolution-precedents.md), [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md), [Operation-extension surface research](../research/extensions/operation-extension-surface.md)
- [0056: Use four libraries and two proof executables](0056-use-four-libraries-and-two-proof-executables.md) — superseded; contracts: [System architecture](../architecture.md); evidence: [Prototype crate layout and Rust MSRV](../research/workspace/prototype-crate-layout-and-msrv.md)
- [0052: Use stable interface keys and host-canonical attributes](0052-use-stable-interface-keys-and-host-canonical-attributes.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Operation extension contract](../operation-extensions.md); evidence: [Operation-extension surface research](../research/extensions/operation-extension-surface.md)

### Numerical operations

- [0018: Canonicalize arithmetic NaNs for portable bitwise results](0018-portable-bitwise-nans.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md)
- [0022: Define reduction identities and initial values](0022-reduction-identities-and-initial-values.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md)
- [0015: Distinguish required FMA from optional contraction](0015-fma-vs-contraction.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md)
- [0012: Keep reduction topology in physical plans](0012-physical-reduction-topology.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md)
- [0010: Make conversion behavior a typed semantic contract](0010-typed-conversion-contracts.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Dtype resolution and mixed-precision precedent](../research/numerics/dtype-resolution-precedents.md), [Floating-point to integer conversion precedents](../research/numerics/float-to-integer-conversion-precedents.md)
- [0039: Make integer overflow explicit in operation identity](0039-explicit-integer-overflow-operations.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Integer arithmetic overflow precedents](../research/numerics/integer-overflow-precedents.md)
- [0021: Require proof or runtime validation for value assumptions](0021-validated-value-assumptions.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md), [Correctness and testing](../correctness-and-testing.md); evidence: [Region accuracy contracts and analyzable error budgets](../research/numerics/region-accuracy-contract.md)
- [0011: Resolve numerical permissions per operation](0011-per-operation-numerical-permissions.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md)
- [0009: Resolve numerical typing before semantic optimization](0009-resolved-numerical-typing.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Dtype resolution and mixed-precision precedent](../research/numerics/dtype-resolution-precedents.md)
- [0016: Resolve transcendental accuracy per operation](0016-transcendental-accuracy-contracts.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Transcendental accuracy precedents](../research/numerics/transcendental-accuracy-precedents.md)
- [0013: Scope deterministic numerical guarantees](0013-scoped-determinism.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md)
- [0041: Separate float-to-integer conversion families](0041-separate-float-to-integer-conversion-families.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Floating-point to integer conversion precedents](../research/numerics/float-to-integer-conversion-precedents.md)
- [0017: Separate local semantics from region accuracy goals](0017-local-vs-region-accuracy.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md), [Correctness and testing](../correctness-and-testing.md); evidence: [Region accuracy contracts and analyzable error budgets](../research/numerics/region-accuracy-contract.md)
- [0023: Separate propagating and number-preferring extrema](0023-floating-point-extrema-semantics.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Floating-point extrema precedents](../research/numerics/floating-point-extrema-precedents.md)
- [0014: Separate reassociation from operand permutation](0014-reassociation-vs-permutation.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md)
- [0025: Separate reduction empty results from physical padding](0025-reduction-empty-results-and-padding.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md)
- [0019: Separate subnormal input and result handling](0019-split-subnormal-handling.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md)
- [0040: Specialize integer division and remainder families](0040-specialize-integer-division-families.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Integer division and remainder precedents](../research/numerics/integer-division-precedents.md)
- [0024: Use round-to-nearest ties-to-even for initial arithmetic](0024-initial-arithmetic-rounding.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md)
- [0042: Use typed transcendental accuracy contracts](0042-use-typed-transcendental-accuracy-contracts.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Transcendental accuracy precedents](../research/numerics/transcendental-accuracy-precedents.md)
- [0020: Use value-only floating-point exceptions initially](0020-value-only-floating-point-exceptions.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md)

### Dtypes and quantization

- [0032: Fix strict affine quantization evaluation](0032-strict-affine-quantization-evaluation.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Affine quantization numerical semantics](../research/numerics/affine-quantization-semantics.md)
- [0029: Generalize affine quantization granularity with parameter index maps](0029-affine-quantization-parameter-maps.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Quantization representation in tensor IRs](../research/numerics/quantization-ir-precedents.md), [Quantized value and transformation contract](../research/numerics/quantized-value-and-transform-contract.md)
- [0034: Govern admitted built-in dtype keys in Tiler](0034-tiler-governed-built-in-dtype-keys.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md)
- [0037: Parameterize complex dtype identity by component type](0037-parameterize-complex-dtype-identity.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md)
- [0035: Recognize IEEE decimal floating-point formats](0035-recognize-ieee-decimal-floating-formats.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md), [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md)
- [0038: Recognize OCP microscaling schemes as compound values](0038-recognize-ocp-mx-schemes.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md), [Quantized value and transformation contract](../research/numerics/quantized-value-and-transform-contract.md)
- [0036: Recognize standard binary and microscaling scalar formats](0036-recognize-standard-binary-and-microscaling-formats.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md), [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md)
- [0028: Recognize standardized sub-byte integer types](0028-recognize-sub-byte-integers.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md)
- [0031: Reject NaN in strict affine quantization](0031-strict-affine-quantization-rejects-nan.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Affine quantization numerical semantics](../research/numerics/affine-quantization-semantics.md)
- [0030: Represent quantized tensors as first-class assembled values](0030-first-class-quantized-values.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Quantized value and transformation contract](../research/numerics/quantized-value-and-transform-contract.md)
- [0026: Separate dtype representability from operation support](0026-dtype-representability-vs-operation-support.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md)
- [0033: Separate semantic validation from physical enforcement](0033-semantic-validation-enforcement.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md), [Correctness and testing](../correctness-and-testing.md); evidence: [Affine quantization numerical semantics](../research/numerics/affine-quantization-semantics.md)
- [0027: Use uniform nominal identities for built-in and extension dtypes](0027-uniform-nominal-dtype-identity.md) — accepted; contracts: [Numerical semantics](../numerical-semantics.md); evidence: [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md)

### Physical planning and lowering

- [0007: Make normalized hardware schedules first-class IR](0007-first-class-kernel-schedules.md) — accepted; contracts: [System architecture](../architecture.md), [IR stack and invariants](../ir.md), [Optimizer model](../compiler/optimizer.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md); evidence: [First-class scheduled-region model](../research/scheduling/scheduled-region-model.md)
- [0047: Model placement as physical properties over capability graphs](0047-model-placement-as-physical-properties.md) — accepted; contracts: [System architecture](../architecture.md), [Proposed CPU/SIMD target profile](../backends/cpu.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md); evidence: [Device placement and memory-domain contract](../research/placement/device-placement-and-memory-domains.md)
- [0046: Separate logical tensor access from storage addressing](0046-separate-logical-access-from-storage-addressing.md) — accepted; contracts: [IR stack and invariants](../ir.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md); evidence: [Symbolic index and access model](../research/indexing/index-access-model.md)
- [0055: Use a serial Sum for the first Metal value proof](0055-use-a-serial-sum-for-the-first-metal-value-proof.md) — accepted; contracts: [System architecture](../architecture.md), [Numerical semantics](../numerical-semantics.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md), [Metal AOT backend](../backends/metal.md), [Correctness and testing](../correctness-and-testing.md); evidence: [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md), [First-class scheduled-region model](../research/scheduling/scheduled-region-model.md), [Structured kernel IR and verifier boundary](../research/kernel-ir/structured-kernel-ir-verifier.md)
- [0043: Use typed phased target feasibility](0043-use-typed-phased-target-feasibility.md) — accepted; contracts: [System architecture](../architecture.md), [Proposed CPU/SIMD target profile](../backends/cpu.md), [Optimizer model](../compiler/optimizer.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md), [Cost model](../compiler/cost-model.md); evidence: [Target profiles and phased physical feasibility](../research/target-profiles/physical-feasibility-model.md)
- [0048: Verify structured kernels as schedule refinements](0048-verify-structured-kernels-as-schedule-refinements.md) — accepted; contracts: [System architecture](../architecture.md), [IR stack and invariants](../ir.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md); evidence: [Structured kernel IR and verifier boundary](../research/kernel-ir/structured-kernel-ir-verifier.md)

### Artifacts, build, and toolchains

- [0053: Gate artifact delivery and failures by consumer family](0053-gate-artifact-delivery-by-consumer-family.md) — accepted; contracts: [Frontend and proc-macro integration](../integration/frontends.md), [Metal AOT backend](../backends/metal.md); evidence: [Proc-macro build environment and freshness](../research/macro-environment/proc-macro-build-environment.md), [Apple Metal artifact compatibility](../research/apple-targets/artifact-compatibility.md)
- [0002: Generate Metal artifacts ahead of time](0002-aot-metal-artifacts.md) — accepted; contracts: [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md), [Metal AOT backend](../backends/metal.md), [Frontend and proc-macro integration](../integration/frontends.md); evidence: [Apple Metal artifact compatibility](../research/apple-targets/artifact-compatibility.md)
- [0049: Select artifact families explicitly at inline invocations](0049-explicit-artifact-family-selection.md) — accepted; contracts: [Frontend and proc-macro integration](../integration/frontends.md), [Metal AOT backend](../backends/metal.md); evidence: [Proc-macro build environment and freshness](../research/macro-environment/proc-macro-build-environment.md), [Apple Metal artifact compatibility](../research/apple-targets/artifact-compatibility.md)
- [0057: Set the prototype MSRV to Rust 1.89](0057-set-the-prototype-msrv-to-rust-1-89.md) — accepted; contracts: [System architecture](../architecture.md), [Frontend and proc-macro integration](../integration/frontends.md); evidence: [Prototype crate layout and Rust MSRV](../research/workspace/prototype-crate-layout-and-msrv.md)
- [0004: Treat each inline macro invocation as an AOT bundle](0004-inline-macro-aot-bundles.md) — accepted; contracts: [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md), [Frontend and proc-macro integration](../integration/frontends.md); evidence: [Proc-macro build environment and freshness](../research/macro-environment/proc-macro-build-environment.md), [Direct embedded-artifact costs across Rust crates](../research/embedding/embedded-artifact-costs.md), [Expansion cache crash and race protocol](../research/cache/crash-and-race-protocol.md)
- [0050: Use immutable self-validating expansion-cache entries](0050-use-immutable-self-validating-expansion-cache-entries.md) — accepted; contracts: [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md), [Frontend and proc-macro integration](../integration/frontends.md), [Metal AOT backend](../backends/metal.md); evidence: [Expansion cache crash and race protocol](../research/cache/crash-and-race-protocol.md)

### Runtime, integration, and placement

- [0003: Keep the compiler independent of Candle](0003-candle-is-an-integration.md) — accepted; contracts: [Candle integration](../integration/candle.md); evidence: [Consumer-neutral runtime execution contract](../research/runtime/runtime-execution-contract.md)
- [0051: Make runtime routing commit one-way before program work](0051-make-runtime-routing-commit-one-way.md) — accepted; contracts: [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md), [Candle integration](../integration/candle.md); evidence: [Consumer-neutral runtime execution contract](../research/runtime/runtime-execution-contract.md), [Candle Metal post-wait error checking](../research/runtime/candle-metal-post-wait-error-checking.md), [Semantic validation enforcement](../research/runtime/semantic-validation-enforcement.md)

### Documentation governance

- [0054: Use typed documentation metadata and derived backlinks](0054-use-typed-documentation-metadata.md) — accepted; contracts: [Documentation metadata and traceability](../document-metadata.md); evidence: [Information architecture and provenance audit](../research/documentation/information-architecture-audit.md)
<!-- END GENERATED ADR TOPICS -->

## Chronological index

- [0001: Separate semantic planning from physical scheduling](0001-separate-semantic-and-physical-plans.md) — accepted
- [0002: Generate Metal artifacts ahead of time](0002-aot-metal-artifacts.md) — accepted
- [0003: Keep the compiler independent of Candle](0003-candle-is-an-integration.md) — accepted
- [0004: Treat each inline macro invocation as an AOT bundle](0004-inline-macro-aot-bundles.md) — accepted
- [0005: Expose a public semantic graph and extension boundary](0005-public-semantic-tensor-graph.md) — accepted
- [0006: Model semantic programs as operation/value graphs](0006-operation-value-graph.md) — accepted
- [0007: Make normalized hardware schedules first-class IR](0007-first-class-kernel-schedules.md) — accepted
- [0008: Separate extent symbols from typed root bindings](0008-typed-root-bindings.md) — accepted
- [0009: Resolve numerical typing before semantic optimization](0009-resolved-numerical-typing.md) — accepted
- [0010: Make conversion behavior a typed semantic contract](0010-typed-conversion-contracts.md) — accepted
- [0011: Resolve numerical permissions per operation](0011-per-operation-numerical-permissions.md) — accepted
- [0012: Keep reduction topology in physical plans](0012-physical-reduction-topology.md) — accepted
- [0013: Scope deterministic numerical guarantees](0013-scoped-determinism.md) — accepted
- [0014: Separate reassociation from operand permutation](0014-reassociation-vs-permutation.md) — accepted
- [0015: Distinguish required FMA from optional contraction](0015-fma-vs-contraction.md) — accepted
- [0016: Resolve transcendental accuracy per operation](0016-transcendental-accuracy-contracts.md) — accepted
- [0017: Separate local semantics from region accuracy goals](0017-local-vs-region-accuracy.md) — accepted
- [0018: Canonicalize arithmetic NaNs for portable bitwise results](0018-portable-bitwise-nans.md) — accepted
- [0019: Separate subnormal input and result handling](0019-split-subnormal-handling.md) — accepted
- [0020: Use value-only floating-point exceptions initially](0020-value-only-floating-point-exceptions.md) — accepted
- [0021: Require proof or runtime validation for value assumptions](0021-validated-value-assumptions.md) — accepted
- [0022: Define reduction identities and initial values](0022-reduction-identities-and-initial-values.md) — accepted
- [0023: Separate propagating and number-preferring extrema](0023-floating-point-extrema-semantics.md) — accepted
- [0024: Use round-to-nearest ties-to-even for initial arithmetic](0024-initial-arithmetic-rounding.md) — accepted
- [0025: Separate reduction empty results from physical padding](0025-reduction-empty-results-and-padding.md) — accepted
- [0026: Separate dtype representability from operation support](0026-dtype-representability-vs-operation-support.md) — accepted
- [0027: Use uniform nominal identities for built-in and extension dtypes](0027-uniform-nominal-dtype-identity.md) — accepted
- [0028: Recognize standardized sub-byte integer types](0028-recognize-sub-byte-integers.md) — accepted
- [0029: Generalize affine quantization granularity with parameter index maps](0029-affine-quantization-parameter-maps.md) — accepted
- [0030: Represent quantized tensors as first-class assembled values](0030-first-class-quantized-values.md) — accepted
- [0031: Reject NaN in strict affine quantization](0031-strict-affine-quantization-rejects-nan.md) — accepted
- [0032: Fix strict affine quantization evaluation](0032-strict-affine-quantization-evaluation.md) — accepted
- [0033: Separate semantic validation from physical enforcement](0033-semantic-validation-enforcement.md) — accepted
- [0034: Govern admitted built-in dtype keys in Tiler](0034-tiler-governed-built-in-dtype-keys.md) — accepted
- [0035: Recognize IEEE decimal floating-point formats](0035-recognize-ieee-decimal-floating-formats.md) — accepted
- [0036: Recognize standard binary and microscaling scalar formats](0036-recognize-standard-binary-and-microscaling-formats.md) — accepted
- [0037: Parameterize complex dtype identity by component type](0037-parameterize-complex-dtype-identity.md) — accepted
- [0038: Recognize OCP microscaling schemes as compound values](0038-recognize-ocp-mx-schemes.md) — accepted
- [0039: Make integer overflow explicit in operation identity](0039-explicit-integer-overflow-operations.md) — accepted
- [0040: Specialize integer division and remainder families](0040-specialize-integer-division-families.md) — accepted
- [0041: Separate float-to-integer conversion families](0041-separate-float-to-integer-conversion-families.md) — accepted
- [0042: Use typed transcendental accuracy contracts](0042-use-typed-transcendental-accuracy-contracts.md) — accepted
- [0043: Use typed phased target feasibility](0043-use-typed-phased-target-feasibility.md) — accepted
- [0044: Use an explicit capability-based operation registry](0044-use-explicit-capability-operation-registry.md) — accepted
- [0045: Bound inline proc-macro providers to host dependencies](0045-bound-proc-macro-providers-to-host-dependencies.md) — accepted
- [0046: Separate logical tensor access from storage addressing](0046-separate-logical-access-from-storage-addressing.md) — accepted
- [0047: Model placement as physical properties over capability graphs](0047-model-placement-as-physical-properties.md) — accepted
- [0048: Verify structured kernels as schedule refinements](0048-verify-structured-kernels-as-schedule-refinements.md) — accepted
- [0049: Select artifact families explicitly at inline invocations](0049-explicit-artifact-family-selection.md) — accepted
- [0050: Use immutable self-validating expansion-cache entries](0050-use-immutable-self-validating-expansion-cache-entries.md) — accepted
- [0051: Make runtime routing commit one-way before program work](0051-make-runtime-routing-commit-one-way.md) — accepted
- [0052: Use stable interface keys and host-canonical attributes](0052-use-stable-interface-keys-and-host-canonical-attributes.md) — accepted
- [0053: Gate artifact delivery and failures by consumer family](0053-gate-artifact-delivery-by-consumer-family.md) — accepted
- [0054: Use typed documentation metadata and derived backlinks](0054-use-typed-documentation-metadata.md) — accepted
- [0055: Use a serial Sum for the first Metal value proof](0055-use-a-serial-sum-for-the-first-metal-value-proof.md) — accepted
- [0056: Use four libraries and two proof executables](0056-use-four-libraries-and-two-proof-executables.md) — accepted
- [0057: Set the prototype MSRV to Rust 1.89](0057-set-the-prototype-msrv-to-rust-1-89.md) — accepted
- [0058: Use a recoverable consuming semantic-program build](0058-use-a-recoverable-consuming-semantic-program-build.md) — accepted
- [0059: Use exact typed authoring handles over runtime-typed semantic values](0059-use-exact-typed-authoring-handles.md) — accepted

## Template

```markdown
# NNNN: Decision title

**Status:** proposed | accepted | superseded

## Context

## Decision

## Consequences

## Alternatives considered
```
