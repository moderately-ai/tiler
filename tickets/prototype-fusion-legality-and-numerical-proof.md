---
id: prototype-fusion-legality-and-numerical-proof
title: Derive fusion legality and numerical evidence
status: in-progress
priority: p0
dependencies: [repair-numerical-witness-integrity, prototype-semantic-index-refinement, prototype-index-region-reference-oracle]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, numerics]
claimed_from: todo
assignee: agent-prototype-fusion-legality
lease_expires_at: 1784851462
---
Derive legality from operation capabilities, access/effect contracts, materialization boundaries, conversions, and numerical policy instead of graph-specific rule tables or asserted proof labels. Produce replayable evidence or typed Unknown/rejection and cover exceptional values, conversion rounding, contraction, empty domains, and reduction order.

The proof output must distinguish reusable refinement content from its checked
binding to one exact region occurrence, value/access mapping, reached semantic
definitions, selected providers, and evidence. It must not place provider or
whole-program identity into pure index structure.

## Outcome

**Fact:** Added `crates/tiler-compiler/src/fusion_legality.rs`, a private draft authority (`derive_fusion_legality`, `verify_fusion_legality`) that derives fusion legality for one `RegionCandidate` from per-operation fusion capabilities, the access/effect contract, materialization/conversion boundaries, and the numerical policy. Its output is the three-way `FusionLegality` = `Legal(Box<FusionLegalityProof>)` | `Rejected(FusionRejection)` | `Unknown(FusionUnknown)`. `crates/tiler-compiler/src/region.rs` gained a minimal read-only projection (`RegionGraph::member_operation_facts`, `RegionGraph::member_canonical_position`, `MemberOperationFacts`) so the derivation can inspect a member's operation key, proven purity, and canonical operand/result value-type encodings without a graph-shape recognizer.

**Fact:** Legality is derived per operation, not from a graph shape or a fixed proof label. Each member's `OpKey` resolves a `FusionOperationRole` (value source / elementwise arithmetic / ordered reduction) through the compiler-owned `FusionNumericalCapabilities`, seeded for the governed strict-`f32` families (`constant`, `multiply`, `add`, `strict-serial-sum`); any family without a registered role fails closed to `Unknown` (`unsupported-operation-capability`). Nine obligations are discharged against the resolved roles, the reached semantic definitions, and the `StrictF32NumericalContract`: capability resolution, referential transparency, conversion-boundary preservation, arithmetic contraction, exceptional values (NaN canonicalization / signed zero / subnormals), reduction identity-and-empty-domain, reduction contributor order, reduction reassociation, and reduction operand permutation. Reassociation and operand permutation are kept independent per ADR 0014: permutation legality is derived from the ordered left-fold role rather than a contract permission field, because the bounded contract carries no distinct permutation permission.

**Fact:** Every obligation carries a `FusionEvidenceClass` — `NormativeGuarantee`, `SoundProof`, `ExhaustiveFinite`, `Empirical`, or `Unknown` — kept distinct and never collapsed. The bounded profile constructs `NormativeGuarantee` (normative-definition-backed order/identity/rounding), `SoundProof` (structural derivations from the verified region and policy), and `Unknown`. `ExhaustiveFinite` and `Empirical` are retained as distinct reserved classes so a future finite-domain or measured qualification cannot masquerade as a proof.

**Fact:** The proof separates reusable content from occurrence binding. `FusionLegalityContent` (identity `FusionLegalityContentIdentity`) is site- and provider-free: the canonical region-content identity, the numerical-contract key, the structural counts, and the ordered discharged obligations with evidence classes. `FusionLegalityProof` (identity `FusionLegalityIdentity`) binds that content to one occurrence: the region-occurrence identity, the reached semantic definitions (operation key + normative definition + effect), the frozen-registry snapshot, the selected fusion-capability provider, and the ordered value/access bindings (`FusionValueBindings`). A test asserts the provider name and the occurrence bytes are absent from the content identity and present in the occurrence identity, and that the occurrence identity contains the content identity — so provider and whole-program/site identity never enter the pure content structure.

**Fact:** `verify_fusion_legality` re-derives and requires exact equality with the retained proof, giving replayable evidence; a forged proof (tampered provider revision) and a candidate re-derived against another program both fail closed with typed errors. Ten module tests cover the legal serial-sum, a pure pointwise square (no reduction), the missing-capability `Unknown`, the foreign-NaN-contract `Unknown`, the forged-proof and foreign-graph rejections, evidence-class distinctness, and the content/occurrence separation invariant.

**Inference:** In the governed bounded profile a hard `Rejected` outcome is not reachable by any valid program, because the numerical vocabulary admits only `OperationEffect::Pure`, `NumericalPermission::Forbidden`, and `SubnormalMode::Preserve`; no valid-but-illegal program is expressible, so every genuine failure is a typed `Unknown`. The `Rejected` disposition is retained as a fail-closed guard (e.g. an impure member) and exercised through its typed surface, ready for the first non-pure effect or contraction-permitting policy.

**Proposal (deferred):** Wiring this authority into the private `compile()` pipeline — retiring `fusion.rs`'s asserted `FusionNumericalProof` labels as the SoundProof receipt and emitting these derived obligations through the explain vocabulary — is a follow-up integration, consistent with the sibling `capability` and `legality` authorities also not yet being wired in. The module-level `#![allow(dead_code)]` (matching `explain.rs`'s private-draft precedent) is removed at that point. No `tiler-ir` or `tiler-reference` semantic contract changed; the `implementation/ir` and `implementation/reference` scopes were declared but not needed beyond reuse.

**Measurement:** `uv run --locked python scripts/check_repository.py` passed; `cargo clippy -p tiler-compiler --all-targets` clean; `git diff --check` clean; 10 `fusion_legality` tests pass under `cargo nextest run`.
