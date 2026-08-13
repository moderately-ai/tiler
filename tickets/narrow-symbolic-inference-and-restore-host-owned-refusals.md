---
id: narrow-symbolic-inference-and-restore-host-owned-refusals
title: Narrow symbolic inference and restore host-owned refusals
status: done
priority: p1
dependencies: [resolve-semantic-shape-inference-over-symbolic-extents, seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, retain-one-derived-proof-summary-per-shape-environment]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [implementation/ir, contracts/foundation, implementation/compiler, implementation/artifact, implementation/build]
shared_scopes: [project/tickets]
paths: [docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md]
tags: [implementation, shapes, semantic-graph, extensions, correctness, public-boundary]
---

## User-visible outcome

The first symbolic-inference release is intentionally narrow: built-in governed elementwise operations may use the program's shape environment, while public external providers remain static-only and fail closed. No provider can mint a host-authoritative environment refusal, and a literal-only operation reports its own capability limit rather than blaming the environment.

## Source-first Fact audit — 2026-08-12, exact base `611fefee15d8878b9458bd860d09490ec736a17f`

The 2026-08-11 Facts were written against `2f244dc7`. `seal-and-validate-sourced-shapes-at-semantic-inference-boundaries` and `retain-one-derived-proof-summary` have landed since. Each Fact is re-read at this base.

**Verified (still false) — a provider can stamp a host environment verdict.** `OperationInferenceError::from_extent_source` is still `pub` and still accepts any publicly constructible `ExtentSourceError`. `extent_aware_registry_error` still reads `rejection.source_error().extent_source()` and returns `BuildError::ExtentSource` without re-deriving the claimed undeclared, too-late, or not-proved fact against the builder's environment. `docs/operation-extensions.md` still states the admission test at anchor `A seam is a propose-then-re-verify boundary`: a provider cannot stamp its own provenance and the host re-derives every asserted fact.

**Imprecise — public no-environment inference now refuses symbols, but as the wrong class, and the broader public surface is still open.** `FrozenSemanticRegistry::infer_operation` still only forwards `None`. After seal-and-validate, `admit_value_fact_extents` runs before the callback and `SourcedShape::admit_against(None)` reports every symbol as `UndeclaredSymbol`, so the public no-environment entry no longer reaches `elementwise_binary_shape`'s rank-zero shortcut or an echo callback. That refusal is still an environment verdict, not a capability limit, and the documentation at anchor `Every symbolic operand is refused` still describes environment absence rather than a host-owned family limit. `infer_operation_with_extent_sources`, `OperationInferenceRequest::extent_sources`, and `ValueFact::new(..., impl Into<SourcedShape>)` remain public. `ExtentSources::new` is crate-private, but a caller that already holds a program's `ExtentSources` can still offer sourced facts to a public inferencer.

**Verified (still imprecise) — `SymbolicExtentUnsupported` is not an environment failure.** `ExtentSourceError` still says every variant is a refusal by the source environment. `SymbolicExtentUnsupported` still says the environment was not asked. `BuildError::ExtentSource` still tells callers to declare or constrain a symbol. The softmax registry fixture now observes `UndeclaredSymbol` on the no-environment path; the program-construction neighbour still asserts `BuildError::ExtentSource(ExtentSourceError::SymbolicExtentUnsupported { .. })`.

**Verified (still imprecise) — `SourcedExtent` and `ExtentSourceError` still omit `#[non_exhaustive]`.** `SourcedExtent` still documents a third source kind. Neither type carries the attribute. In-crate matches, including `SourcedExtent::tag` and the identity injectivity table, stay exhaustive. No out-of-crate total recognizer of either vocabulary was found; construction of known variants in other workspace crates does not require exhaustiveness. ADR 0074 convention 5a still applies.

**Verified — the built-in elementwise rule itself is still correct, and proof queries no longer re-solve.** F32 and BF16 still share `elementwise_binary_shape`. Symbol-involving axis equality is still admitted only through `ExtentSources::proves_equal`. Rank and literal disagreement remain the family diagnostic. Scalar broadcast is still decided on rank. The result still retains the left operand. After `retain-one-derived-proof-summary`, `ShapeEnv::proves_equal` reads the retained summary (`summary.same_class`) and does not re-solve the constraint system; `ExtentSources::proves_equal` still delegates there. This ticket still narrows the participation boundary without replacing that rule.

## Work

- Add one required private `ShapeInferenceParticipation::{LiteralOnly, GovernedEnvironmentAware}` value to every `OperationDefinition`. Replace the ambiguous public constructor with an explicitly literal-only constructor and keep governed environment-aware construction crate-private. Encode the fixed tag in both operation-definition identity populations; step `tiler.semantic-registry.v7` to `v8` and `tiler.semantic-definition-projection.v5` to `v6`, updating domain pins and every transitive golden.
- Restore the public provider-facing value constructor to a static `Shape` input. Keep a crate-private sourced constructor for governed semantic inference; preserve a total, non-panicking read view over retained facts.
- Make `FrozenSemanticRegistry::infer_operation_with_extent_sources`, `OperationInferenceRequest::extent_sources`, the sourced static-shape helper, and host extent-error construction/inspection crate-private for the narrow release. The ordinary public `infer_operation` mechanically rejects any symbolic operand before invoking a callback.
- Move `SymbolicExtentUnsupported` out of `ExtentSourceError` into a typed semantic operation-capability refusal. Preserve operation key/operand or axis/symbol detail sufficient for remediation; do not report it as an environment failure or silently substitute a provider message.
- Make the builder derive every `BuildError::ExtentSource` from its own environment validation or comparison. Provider output may propose facts and provider-attributed diagnostics, never a host environment verdict.
- Apply ADR 0074's `#[non_exhaustive]` posture to `SourcedExtent` and `ExtentSourceError`, updating exact in-workspace matches without wildcarding canonical encoders or weakening identity injectivity.
- Correct public documentation in `docs/operation-extensions.md` and the Rust API so static-only external participation, governed internal symbolic inference, preflight ordering, and refusal ownership agree exactly.

## Strict narrow contract

- There is no default symbolic policy and no fallback from environment-aware inference to structural equality.
- External providers receive only static semantic facts in this first pass. Encountering a symbol is a typed preflight refusal before callback.
- Governed built-ins may receive sourced facts only through the builder's exact environment-bound path, after `seal-and-validate-sourced-shapes-at-semantic-inference-boundaries` holds normalization and admission.
- Unsupported symbolic families refuse where the capability is missing; they do not defer, coerce, specialize, or reinterpret a symbol as a literal.

## Evidence

- A public external provider cannot construct or receive a sourced result/operand through the ordinary inference API; the static neighbour still works.
- Standard no-environment scalar-plus-symbol and custom echo-provider probes fail before callback.
- A provider attempting to forge an undeclared/not-equal extent error cannot cause `BuildError::ExtentSource`; the host-derived counterpart remains typed.
- A literal-only governed family reports the semantic capability refusal, while a truly undeclared/late/not-proved environment fact remains `ExtentSourceError`.
- Out-of-crate trybuild fixtures prove the intended static provider surface and the non-exhaustive matching contract.
- Perturb preflight, provider-error authority, error classification, and each non-exhaustive census independently with assertions unchanged.

## Public boundary and acceptance

This deliberately revises the broader public surface proposed by `resolve-semantic-shape-inference-over-symbolic-extents`. Tom approved the narrow direction on 2026-08-11: built-in environment-aware elementwise behavior stays; the premature external symbolic-provider surface retreats until an explicit required policy and host proof protocol exist. The exact revised signatures remain a labelled draft until reviewed in the implementation diff.

**Final architecture accepted 2026-08-12.** Tom accepted the required two-mode internal policy, public literal-only construction, governed crate-private construction, fixed identity tag, host-owned semantic refusal, and no-default/no-fallback posture in the ChatGPT coordination thread. Implementation review still verifies the exact Rust spellings and exclusions, but does not reopen these semantics without contradictory source evidence.

## Implementation record — 2026-08-12

Exact Rust spellings remain a labelled draft.

**Scopes added after the identity step.** `implementation/compiler`, `implementation/artifact`, and `implementation/build` are reverse-deps of the registry/projection step: the explain request qualifier, out-of-crate static provider fixtures, and the standard Metal artifact/cache/fixed-content pins all moved. `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` is on `paths` because the Metal pin test requires the live-pin paragraph to name the same values.

**Identity.** `tiler.semantic-registry.v7` → `v8`. `tiler.semantic-definition-projection.v5` → `v6`. Participation tag is `0x01` literal-only, `0x02` governed. Standard provider revision stays 7. Live pins recomputed on this branch:

- explain request qualifier `4f6429492ac63d04` → `6e91a843fd9e69b8`
- Metal artifact `39e765637a7e014adac2b8a30788798758ca46584b558732c2bda41b7639ddda` → `9b739d215336de436ef334ded614ef4b43db9edfec170ee5032fee809975b3b7`
- Metal cache `7e00d9fa0ce90749e6f7d3d42e0f2aaabe5670e0359a0c20d1580a09bb967130` → `1a04d873fe54c3785d1770a7ee4537a607c2acc9a5ae67f328e8f49de53621e4`
- Metal fixed content `65_313` → `65_327` (+14: one participation byte per encoded operation definition, folded through the nested semantic subjects)
- slice-law semantic snapshot digest `72a5c44e73a9fb76…` → `15a35d501845fb22…`; law-registry digest `ddfb4dc459d7ca53…` → `7a7d1933feffa058…`

**Perturbations, assertions unchanged.** Dropping operand preflight: `an_echo_provider_does_not_receive_a_symbolic_operand_on_the_public_path` panics `assertion failed: !called.load(Ordering::SeqCst)`; `public_inference_refuses_a_scalar_plus_symbol_before_callback` panics `left: 0` / `right: 1`. Restoring builder promotion of a provider-stamped `extent_source`: `a_provider_cannot_forge_an_undeclared_extent_error_into_a_host_verdict` panics `a stamped undeclared symbol is not a host environment verdict: sourced-extent.undeclared-symbol: forge::ghost is not declared by this program's shape environment`; the not-equal neighbour names `program/0::n` and `program/0::m`. Mapping the capability refusal through `SemanticRegistry`: `a_literal_only_family_declines_a_symbolic_operand_by_name` panics `a literal-only family reports a capability refusal, not semantic.symbolic-operand-unsupported: …`. Shrinking the `ExtentSourceError` census to three variants: `expected an array with a size of 4, found one with a size of 3`. Shrinking the `SourcedExtent` tag table to one inhabitant: `expected an array with a size of 2, found one with a size of 1`.

## Closes when

The public provider path is mechanically static-only, the governed symbolic path is environment-bound and preflighted, host errors cannot be forged, capability and environment refusals are correctly layered, the ADR conventions are restored, and the decision ticket can enumerate one exact safe included/excluded surface.
