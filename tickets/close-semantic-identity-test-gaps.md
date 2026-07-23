---
id: close-semantic-identity-test-gaps
title: Close semantic identity and registry hardening residuals
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, hardening]
---
Four adversarially verified residuals from the 2026-07-23 audit that the completed hardening tickets did not cover:

- add negative tests for the governed inferencers' untested rejection paths: `sum.axes.range` (axis at or beyond operand rank) and the f32 constant payload rejection (non-`FloatBits` format or wrong byte width); only `binary.shape`, `sum.axes.empty`, and `sum.axes.canonical` are currently tested;
- add a canonical-identity construction-order regression test that builds the same live DAG in two different valid topological insertion orders and asserts identical identity bytes; the existing test varies only dead-operation insertion position;
- `CanonicalValue::value_type`, `TypeDefinitionFacts::new`, `OperationDefinitionFacts::new`, and `OperationConformance::new` remain unvalidated `const` constructors that can carry over-bound values past the structural depth/node limits into facts wrappers and durable registry identity. Close this by **bounding at registration**, not by making the constructors fallible: they have live cross-crate call sites in `crates/tiler-compiler/src/request.rs` and `crates/tiler-reference/src/lib.rs`, so a fallible signature would change a public boundary and require scopes this ticket does not declare; and
- `ValueTypeDefinitionKey`'s derived `Ord` makes durable registry-identity byte order depend on Rust enum declaration order; replace it with an explicit `Ord` implementation. Note `ValueTypeDefinitionKey::encode` already emits stable discriminants 1/2/3, so the encoding is not at risk — only the BTreeMap iteration order is. The explicit order **must preserve the current derived order** (Nominal < Parameterized < EncodedNumeric) so existing registry-identity bytes do not change; add a regression fixture asserting byte stability.

Under the bound-at-registration option all changes stay inside tiler-ir and its tests; no public-boundary changes are expected, but any that emerge remain drafts until Tom reviews them. Sequence after the typed-explain branch merges — both hold `implementation/ir`, and the explain work touches the same compiler-side call sites this hardening ripples through.

## Outcome

All four residuals are closed inside `crates/tiler-ir/`. Every change is additive; no existing check, test, or identity byte changed.

**Governed-inferencer rejection paths.** **Fact:** through `FrozenSemanticRegistry::infer_operation`, `StrictSerialSumF32` rejects an axis with `sum.axes.range` only after the `sum.axes.type` and `sum.axes.width` checks, and the condition is `axis >= operand rank` (the companion `usize::try_from` arm is unreachable on the supported 64-bit profiles). `ConstantF32` carries two distinct `constant.bits` rejections: a category check on the attribute view, and a combined format/width check requiring exactly `tiler::f32@1` and four payload bytes. The category check is unreachable through the registered schema because `OperationDefinition::infer` always preflights `CanonicalValueKind::FloatBits` first, and a non-`tiler::f32@1` format is only reachable when that format is itself registered. `semantic::program::tests::all_rejected_operation_edits_preserve_arena_lengths` now covers `sum.axes.range`; `semantic::registry::tests::governed_constant_rejects_every_non_binary32_payload` covers the foreign-format and both wrong-width payloads, and `governed_constant_rechecks_the_payload_category_after_schema_preflight` re-registers the same inferencer behind a permissive `Bytes` schema to prove it fails closed independently of preflight.

**Construction-order identity.** **Fact:** `canonical_traversal` assigns canonical IDs by output-driven DFS over operand edges, so live arena indices are already excluded; the prior test varied only dead-operation insertion position. `semantic::program::tests::identity_ignores_live_topological_insertion_order` builds one live DAG in two valid topological insertion orders, asserts the arenas genuinely differ, and asserts identical graph identity bytes. **Measurement:** emitting operation records in live arena order instead of traversal order fails the new test and leaves the pre-existing dead-order test passing.

**Unvalidated `const` constructors.** **Fact:** every collection constructor re-measures its items, so only `CanonicalValue::value_type` can produce an over-bound canonical value — it wraps an already-admitted resolved type without re-measuring, adding one structural level — and `TypeDefinitionFacts::new`, `OperationDefinitionFacts::new`, and `OperationConformance::new` then carry it verbatim. **Measurement:** with the bound removed, a definition whose facts wrap a maximal-depth resolved type registers successfully and reaches a frozen 2311-byte snapshot identity. The bound now lives at registration: `SemanticRegistryRegistrar::admit_definition_value` re-measures the complete value through the new `types::validate_canonical_value` before any resource accounting, covering type-definition facts, operation facts, operation conformance, and schema attribute defaults — the fourth path reaches durable identity the same way and had the same hole. All four constructors keep their infallible `const` signatures, so the live call sites in `crates/tiler-compiler/src/request.rs` and `crates/tiler-reference/src/lib.rs` are untouched and both crates compile unchanged.

**`ValueTypeDefinitionKey` ordering.** **Fact:** the derived `Ord` made durable registry-identity iteration order depend on Rust variant declaration order. It is replaced by an explicit `Ord`/`PartialOrd` pair keyed on `family_discriminant`, which `encode` now also emits, so the ordering rank and the serialized discriminant cannot drift apart. The order is unchanged (Nominal < Parameterized < EncodedNumeric). **Measurement:** the governed standard-registry snapshot identity (1496 bytes) and the new fixture snapshot (307 bytes) are byte-identical before and after the change. `semantic::registry::tests::definition_family_order_is_explicit_and_byte_stable` pins the fixture bytes as a golden and fails both for a reordered-variant derived `Ord` and for an unrelated encoding change; the pre-existing registry-identity tests pass under the reorder, confirming the gap.

**Public surface.** One additive `RegistryError::InvalidDefinitionValue { subject, source }` variant on a `#[non_exhaustive]` enum, plus the new `DefinitionValueSubject` enum, both re-exported from `tiler_ir::semantic`. These are drafts pending Tom's boundary review.

**Out of scope, recorded:** `ResolvedValueType`'s derived `Ord` also follows variant declaration order. It does not reach any identity encoding — closure sets of concrete instances are never serialized — but it does determine which missing-authority error a `freeze` reports first. That is a diagnostic-determinism question for a separate ticket, not a durable-identity one.
