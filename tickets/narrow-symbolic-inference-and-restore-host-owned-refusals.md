---
id: narrow-symbolic-inference-and-restore-host-owned-refusals
title: Narrow symbolic inference and restore host-owned refusals
status: todo
priority: p1
dependencies: [resolve-semantic-shape-inference-over-symbolic-extents, seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, retain-one-derived-proof-summary-per-shape-environment]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, semantic-graph, extensions, correctness, public-boundary]
---

## User-visible outcome

The first symbolic-inference release is intentionally narrow: built-in governed elementwise operations may use the program's shape environment, while public external providers remain static-only and fail closed. No provider can mint a host-authoritative environment refusal, and a literal-only operation reports its own capability limit rather than blaming the environment.

## Source-first Fact audit — 2026-08-11, exact base `2f244dc7ff3a759d9688a482c27b48da70f37227`

**False — the provider cannot stamp host authority.** `OperationInferenceError::from_extent_source` is public and accepts any publicly constructible `ExtentSourceError`; `extent_aware_registry_error` trusts that payload and upgrades it to `BuildError::ExtentSource` without re-derivation. This contradicts `docs/operation-extensions.md`, anchor `A seam is a propose-then-re-verify boundary`, whose admission test says a provider cannot stamp its own provenance and the host re-derives every asserted fact.

**False — public no-environment inference refuses every symbolic operand.** `FrozenSemanticRegistry::infer_operation` only forwards `None`. The standard scalar-broadcast path may return the other operand before asking the environment, and an external provider may structurally accept or echo a symbolic value. The public documentation at anchor `Every symbolic operand is refused` overstates behavior.

**Imprecise — `SymbolicExtentUnsupported` is not an environment failure.** `ExtentSourceError` says every variant is a refusal by the source environment, while `SymbolicExtentUnsupported` says the environment was not asked and nothing about it failed. `BuildError::ExtentSource` then tells callers to declare or constrain a symbol, which cannot make a literal-only operation family support symbolic shapes. The failure belongs to semantic operation capability.

**Imprecise — two public growth vocabularies omit the accepted compatibility posture.** `SourcedExtent` anticipates a third source kind and `ExtentSourceError` has already grown, yet neither is `#[non_exhaustive]`. The complete workspace census finds no out-of-crate total recognizer that requires exhaustiveness, so ADR 0074 convention 5a applies.

**Verified — the built-in elementwise rule itself is correct.** F32 and BF16 share `elementwise_binary_shape`; symbol-involving axis equality is admitted only through `ExtentSources::proves_equal`, rank and literal disagreement remain the family diagnostic, scalar broadcast is explicit, and result spelling retains the left operand. This ticket narrows the surrounding participation boundary without replacing that rule.

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

## Closes when

The public provider path is mechanically static-only, the governed symbolic path is environment-bound and preflighted, host errors cannot be forged, capability and environment refusals are correctly layered, the ADR conventions are restored, and the decision ticket can enumerate one exact safe included/excluded surface.
