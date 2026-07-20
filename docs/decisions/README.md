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

- [0045: Bound inline proc-macro providers to host dependencies](0045-bound-proc-macro-providers-to-host-dependencies.md) — accepted
- [0005: Expose a public semantic graph and extension boundary](0005-public-semantic-tensor-graph.md) — accepted
- [0006: Model semantic programs as operation/value graphs](0006-operation-value-graph.md) — accepted
- [0008: Separate extent symbols from typed root bindings](0008-typed-root-bindings.md) — accepted
- [0001: Separate semantic planning from physical scheduling](0001-separate-semantic-and-physical-plans.md) — accepted
- [0044: Use an explicit capability-based operation registry](0044-use-explicit-capability-operation-registry.md) — accepted
- [0052: Use stable interface keys and host-canonical attributes](0052-use-stable-interface-keys-and-host-canonical-attributes.md) — accepted

### Numerical operations

- [0018: Canonicalize arithmetic NaNs for portable bitwise results](0018-portable-bitwise-nans.md) — accepted
- [0022: Define reduction identities and initial values](0022-reduction-identities-and-initial-values.md) — accepted
- [0015: Distinguish required FMA from optional contraction](0015-fma-vs-contraction.md) — accepted
- [0012: Keep reduction topology in physical plans](0012-physical-reduction-topology.md) — accepted
- [0010: Make conversion behavior a typed semantic contract](0010-typed-conversion-contracts.md) — accepted
- [0039: Make integer overflow explicit in operation identity](0039-explicit-integer-overflow-operations.md) — accepted
- [0021: Require proof or runtime validation for value assumptions](0021-validated-value-assumptions.md) — accepted
- [0011: Resolve numerical permissions per operation](0011-per-operation-numerical-permissions.md) — accepted
- [0009: Resolve numerical typing before semantic optimization](0009-resolved-numerical-typing.md) — accepted
- [0016: Resolve transcendental accuracy per operation](0016-transcendental-accuracy-contracts.md) — accepted
- [0013: Scope deterministic numerical guarantees](0013-scoped-determinism.md) — accepted
- [0041: Separate float-to-integer conversion families](0041-separate-float-to-integer-conversion-families.md) — accepted
- [0017: Separate local semantics from region accuracy goals](0017-local-vs-region-accuracy.md) — accepted
- [0023: Separate propagating and number-preferring extrema](0023-floating-point-extrema-semantics.md) — accepted
- [0014: Separate reassociation from operand permutation](0014-reassociation-vs-permutation.md) — accepted
- [0025: Separate reduction empty results from physical padding](0025-reduction-empty-results-and-padding.md) — accepted
- [0019: Separate subnormal input and result handling](0019-split-subnormal-handling.md) — accepted
- [0040: Specialize integer division and remainder families](0040-specialize-integer-division-families.md) — accepted
- [0024: Use round-to-nearest ties-to-even for initial arithmetic](0024-initial-arithmetic-rounding.md) — accepted
- [0042: Use typed transcendental accuracy contracts](0042-use-typed-transcendental-accuracy-contracts.md) — accepted
- [0020: Use value-only floating-point exceptions initially](0020-value-only-floating-point-exceptions.md) — accepted

### Dtypes and quantization

- [0032: Fix strict affine quantization evaluation](0032-strict-affine-quantization-evaluation.md) — accepted
- [0029: Generalize affine quantization granularity with parameter index maps](0029-affine-quantization-parameter-maps.md) — accepted
- [0034: Govern admitted built-in dtype keys in Tiler](0034-tiler-governed-built-in-dtype-keys.md) — accepted
- [0037: Parameterize complex dtype identity by component type](0037-parameterize-complex-dtype-identity.md) — accepted
- [0035: Recognize IEEE decimal floating-point formats](0035-recognize-ieee-decimal-floating-formats.md) — accepted
- [0038: Recognize OCP microscaling schemes as compound values](0038-recognize-ocp-mx-schemes.md) — accepted
- [0036: Recognize standard binary and microscaling scalar formats](0036-recognize-standard-binary-and-microscaling-formats.md) — accepted
- [0028: Recognize standardized sub-byte integer types](0028-recognize-sub-byte-integers.md) — accepted
- [0031: Reject NaN in strict affine quantization](0031-strict-affine-quantization-rejects-nan.md) — accepted
- [0030: Represent quantized tensors as first-class assembled values](0030-first-class-quantized-values.md) — accepted
- [0026: Separate dtype representability from operation support](0026-dtype-representability-vs-operation-support.md) — accepted
- [0033: Separate semantic validation from physical enforcement](0033-semantic-validation-enforcement.md) — accepted
- [0027: Use uniform nominal identities for built-in and extension dtypes](0027-uniform-nominal-dtype-identity.md) — accepted

### Physical planning and lowering

- [0007: Make normalized hardware schedules first-class IR](0007-first-class-kernel-schedules.md) — accepted
- [0047: Model placement as physical properties over capability graphs](0047-model-placement-as-physical-properties.md) — accepted
- [0046: Separate logical tensor access from storage addressing](0046-separate-logical-access-from-storage-addressing.md) — accepted
- [0043: Use typed phased target feasibility](0043-use-typed-phased-target-feasibility.md) — accepted
- [0048: Verify structured kernels as schedule refinements](0048-verify-structured-kernels-as-schedule-refinements.md) — accepted

### Artifacts, build, and toolchains

- [0053: Gate artifact delivery and failures by consumer family](0053-gate-artifact-delivery-by-consumer-family.md) — accepted
- [0002: Generate Metal artifacts ahead of time](0002-aot-metal-artifacts.md) — accepted
- [0049: Select artifact families explicitly at inline invocations](0049-explicit-artifact-family-selection.md) — accepted
- [0004: Treat each inline macro invocation as an AOT bundle](0004-inline-macro-aot-bundles.md) — accepted
- [0050: Use immutable self-validating expansion-cache entries](0050-use-immutable-self-validating-expansion-cache-entries.md) — accepted

### Runtime, integration, and placement

- [0003: Keep the compiler independent of Candle](0003-candle-is-an-integration.md) — accepted
- [0051: Make runtime routing commit one-way before program work](0051-make-runtime-routing-commit-one-way.md) — accepted

### Documentation governance

- [0054: Use typed documentation metadata and derived backlinks](0054-use-typed-documentation-metadata.md) — accepted
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

## Template

```markdown
# NNNN: Decision title

**Status:** proposed | accepted | superseded

## Context

## Decision

## Consequences

## Alternatives considered
```
