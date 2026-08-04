---
id: derive-a-reached-only-executable-coverage-identity
title: Derive a reached-only executable coverage identity
status: done
priority: p1
dependencies: [place-index-refinement-evidence-under-an-ir-owned-verifier, canonicalize-index-refinement-occurrence-ordinals]
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/decisions, contracts/artifacts, research/program-planning, contracts/foundation]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [design, implementation, identity]
---
## User-visible outcome

Executable stage coverage is minted only from a completed IR refinement receipt and identifies the selected, reached proof needed to replay that coverage, without making an unused registry/provider row invalidate an otherwise identical executable artifact.

## Facts and stop evidence

`IndexRefinementReceiptIdentity` currently nests complete semantic and scalar snapshot provenance through three routes: `encode_subject_identity` writes `SemanticCapabilityAuthority::registry_snapshot`; `encode_authority_identity` writes the same semantic snapshot plus `CanonicalScalarRegistrySnapshotIdentity`; and `encode_receipt_identity` writes `ScalarAuthorityEvidence::{semantic_snapshot, scalar_snapshot}`. The preserved `bind-stage-coverage-to-index-refinement-identity` draft folds that opaque identity into both independent stage encoders.

ADR 0072 requires selected plan/artifact provenance to exclude unused providers. `crates/tiler-artifact/src/program/tests.rs::an_unused_semantic_provider_revision_does_not_change_identity` is the existing executable invariant. The preserved draft broad test cannot honestly satisfy both statements: its compiler fixture first refuses the nonstandard semantic/scalar authority pairing, and any correctly minted receipt over those exact registries would then change when the unused provider revision moves.

## Candidate projections and elimination

1. **Fold the existing complete receipt identity.** Non-forgeable and replay-resistant, but eliminated: complete snapshots make unused semantic/scalar rows change program and artifact identity, directly violating ADR 0072 and the existing unused-provider invariant.
2. **Let program/artifact callers assemble a reached tuple from receipt accessors.** Can exclude snapshots, but eliminated: a caller can cross an occurrence, region, numerical contract, law row, scalar definition projection, or provider projection from different receipts. That recreates the substitution gap the coverage binding exists to close.
3. **Mint a second opaque executable-coverage projection only from a completed receipt.** Survives. IR owns construction and retains graph plus canonical occurrence binding, governed numerical contract, exact region, operation-specific law/provider row, reached semantic definition and admission projections, reached scalar definition and admission projections, reached type definition and admission projections, exact operand/result bindings, and residual proof identities. It excludes the complete semantic registry, scalar registry, and law-registry snapshots: those remain request/verifier authority, not selected executable provenance. A proof gap cannot mint either receipt or coverage identity.
4. **Change the receipt identity itself to the reached-only projection.** Eliminated. It reduces duplication only by conflating two equality questions that ADR 0072 separates: whether a receipt was minted under the exact frozen verifier/request authority, and whether selected executable evidence is unchanged by unused authority. It also silently changes the already-landed receipt v1 subject and its `PartialEq` semantics. Keeping a strict receipt plus a reached executable projection preserves both questions explicitly and lets future registry extensions remain cache-stable without weakening replay validation.

## Derived recommendation

Candidate 3 is the current sole safe recommendation: it keeps receipt verification strict while giving executable identity the ADR 0072 subject it actually owns. This is a consequential public IR/program boundary and remains a tested draft for Tom before acceptance.

## Implementation keys

- Add an opaque named executable-coverage identity/projection whose only public constructor is proof-derived from a completed `IndexRefinementReceipt`; do not expose a raw-byte or independently-fielded constructor.
- Define and document the exact retained reached subjects and why each prevents replay/substitution. Preserve canonical graph/occurrence stability across equivalent authoring orders.
- Keep complete snapshots in verifier/request authority only; never encode unused registry/provider rows in kernel-program or artifact identity.
- Advance every owning IR receipt/projection, kernel-program stage/program, and artifact stage/program domain exactly once on the merged tree; enumerate and recompute every pin.
- Preserve the two independent program/artifact stage encoders.

## Deliberate perturbations

- Same graph, same reached operations/providers, but an unused semantic provider revision changes: executable coverage, kernel program, and artifact identities remain equal.
- Same reached scalar set, but an unused scalar definition/provider changes: identities remain equal.
- Reached semantic or scalar provider revision changes: identity changes.
- Region, numerical contract, canonical occurrence, law row/provider, operand/result binding, or residual proof changes one at a time: identity changes.
- Cross two completed receipts with equal shapes/interfaces: no public construction can form the crossed coverage; builder rejects foreign graph/duplicate occurrence.
- Equivalent graphs authored in different valid insertion orders mint equal canonical occurrence-bound executable identities.
- Proof pending/refusal has no executable coverage spelling.

## Closes when

The reached-only proof-derived public draft is accepted; all perturbations pass; both independent encoders and identity ledgers agree; the blocked stage-coverage ticket can consume it without weakening ADR 0072; targeted affected-crate checks and the full gate pass.

## Scope and concurrency record — 2026-08-04

The existing `implementation/ir` and `project/tickets` declarations cover the implementation and graph record changed here. `contracts/foundation` was added autonomously because `ticketsplease.toml` maps the source-derived IR identity statement in `docs/ir.md` there; this declares already-authorized work and does not expand the product outcome. The pre-existing `contracts/artifacts` scope is retained for the ticket's artifact identity audit, although this branch does not change an artifact contract or encoder.

Before editing, the exact four-file planned population — `crates/tiler-ir/src/index/{mod,refinement}.rs`, `docs/ir.md`, and this ticket — was first intersected with the concurrent KV research branch's 30-file population from `b4e3478d42ce21ed68e23f772b643c6370d36498` through `fff3894cc0731ca7df3cb51cc9f18714cba44aa7`; the intersection count was zero. After the KV authority landed, the same population was intersected with the exact 34-file change from this ticket's original base `d3b659a05699113a2315a39867dc5ae5115c3967` through full-gated KV closure `56b7ab22526ff0d7e6714abc8060b6773dfad5bb`; that intersection was also zero. The branch then merged exact pushed claim base `288e7aab920e3a8bc6d9b242621d5a1cbeabfa2c` without conflict before final checks.

**Resumption — 2026-08-04.** The branch merged `ce62ad550c73287a43099aefbc9eff11b2bafc31` cleanly; `git diff --name-only 39025a6d ac97efe0` outside `tickets/` and `docs/` is `README.md` alone, all of it arriving from `main`. Resumption adds `crates/tiler-artifact/src/program/tests.rs` to the population, which `ticketsplease.toml` maps to the already-declared `implementation/artifact`. Concurrency at resumption: the only live claims were `decide-the-expansion-cache-collection-schedule` (`research/cache`, `implementation/cache` — disjoint) and `supersede-the-runtime-owned-kv-state-design`, which shares `contracts/foundation` and `research/program-planning`. `git diff --name-only ce62ad55...tkt/supersede-the-runtime-owned-kv-state-design` was empty, so that branch had committed nothing to intersect; file-level disjointness on `docs/ir.md` therefore rests on the coordinator's instruction to that worker rather than on a diff, and is recorded as such rather than as a verified intersection. No live ticket held `implementation/artifact`.

## Tested draft and identity audit — 2026-08-04

**Proposal — exact public inventory, not self-accepted:** `tiler_ir::index` adds the opaque `IndexRefinementExecutableCoverageIdentity` with only `as_bytes(&self) -> &[u8]`, and `IndexRefinementReceipt` adds `executable_coverage_identity(&self) -> &IndexRefinementExecutableCoverageIdentity`. The type has no public raw-byte or field constructor, and no byte-level conversion; two compile-fail doctests exercise its private storage and the absent `From<&[u8]>`. The only production minting site is `mint_receipt`, reached after immediate verification or successful closed residual-proof completion. Pending and refused states therefore cannot spell executable coverage, and a third compile-fail doctest on `PendingIndexRefinementReceipt` holds that.

**Fact — retained subject:** the new encoder length-frames the exact graph, canonical occurrence, numerical contract, canonical verified region, reached semantic definitions and admission, exact law row, resolved law provider and revision, reached scalar definitions and admission, reached semantic type definitions and admission, ordered operand/result binding records, and every residual proof identity. It does not encode the subject semantic snapshot, scalar snapshot, or frozen law-registry snapshot. Those remain in the existing strict receipt/request authority.

**Fact — omitted subjects are determined, not dropped.** The encoder does not restate the operation key, ordered signature, host-canonical attributes, or boundary shapes. `compute_graph_identity` writes each of them for every operation in canonical traversal order under `tiler.semantic-graph.v2` (`crates/tiler-ir/src/semantic/identity.rs`, via `encode_operation`), and `IndexRefinementSubject::derive` fixes the retained occurrence to that same canonical ordinal, so the retained `(graph, occurrence)` pair already determines them. Encoding them again would restate a determined value rather than close an open substitution; the reasoning is recorded on the type's doc comment so a later reader does not mistake the omission for an oversight.

**Fact — domain audit:** this introduces exactly one independently tagged domain, `tiler.ir.index-refinement-executable-coverage.v1`. It does not change the grammar or meaning of receipt v1, subject v2, authority v1, resolution v1, proof v1, kernel-program v7, artifact-stage v2, artifact-program v14, or the envelope. No existing literal pin consumes the new domain. **No pinned identity moved on this branch**: no version constant, golden, or literal pin is added, removed, or edited by the diff, which touches only `refinement.rs`, the `index/mod.rs` re-export list, the artifact program test, `docs/ir.md`, and this ticket. The dependent `bind-stage-coverage-to-index-refinement-identity` ticket still owns program and artifact consumption, their two independent encoders, and any resulting merged-tree version/pin recomputation.

**Fact — deliberate evidence, one named test per perturbation bullet.**

| Perturbation | Test | Location |
| --- | --- | --- |
| Unused semantic provider revision leaves coverage equal | `executable_coverage_excludes_unused_authority_but_retains_reached_scalar_provenance` | `crates/tiler-ir/src/index/refinement.rs` |
| …and leaves kernel-program and artifact identity equal | `an_unused_semantic_provider_revision_does_not_change_identity` | `crates/tiler-artifact/src/program/tests.rs` |
| Unused scalar definition/provider leaves coverage equal | `executable_coverage_excludes_unused_authority_but_retains_reached_scalar_provenance` | `crates/tiler-ir/src/index/refinement.rs` |
| Reached semantic or scalar provider revision moves coverage | `executable_coverage_excludes_unused_authority_but_retains_reached_scalar_provenance` | `crates/tiler-ir/src/index/refinement.rs` |
| Region, contract, occurrence, law row, law provider, law revision, operand binding, result binding, residual proof — one at a time | `executable_coverage_retains_each_replay_and_substitution_boundary` | `crates/tiler-ir/src/index/refinement.rs` |
| Crossing two completed receipts with equal shapes/interfaces | `completion_receipts_cannot_be_cross_wired_between_real_occurrences` | `crates/tiler-ir/src/index/refinement.rs` |
| No public construction can form a crossed coverage | two `compile_fail` doctests on `IndexRefinementExecutableCoverageIdentity` | `crates/tiler-ir/src/index/refinement.rs` |
| Equivalent authoring orders mint equal occurrence-bound identities | `equivalent_authoring_orders_retain_directional_canonical_occurrences` | `crates/tiler-ir/src/index/refinement.rs` |
| Proof pending/refusal has no executable coverage spelling | `pending_and_refused_proofs_have_no_executable_coverage_spelling` plus the `compile_fail` doctest on `PendingIndexRefinementReceipt` | `crates/tiler-ir/src/index/refinement.rs` |

The kernel-program leg is asserted on `VerifiedKernelProgram::canonical_identity` separately from the artifact leg, because the artifact folds the program identity and an equal artifact alone cannot distinguish an unchanged program from two changes that cancelled.

**Fact — the completer's `Disproved` arm is not reachable from a small verified region, and that is a property of the layering rather than a gap in the test.** `IndexRegionBuilder` runs its own exhaustive fallback under `MAX_EXHAUSTIVE_PROOF_CELLS` (1,048,576) and refuses an out-of-bounds access as `CoordinateOutOfBounds` at build time, so a disprovable small region never becomes a `VerifiedIndexRegion`; it also discharges the residual it can walk, so a *provable* small region carries no residual either. Reaching a `Disproved` completion needs a region inside the cell window between that bound and `MAX_FINITE_DOMAIN_PROOF_CELLS` (16,777,216) whose per-point integer work still fits the shared 64 MiB byte bound, and the existing out-of-bounds fixture `residual_region(1, 5, 1)` exceeds that byte bound (see `exact_finite_evaluation_returns_the_first_counterexample`). Both refusal arms leave `complete` through the same `Err`, so the coverage claim is carried by the reachable `Unknown` refusal; the test records the reasoning at its definition.

**Fact — failure-path evidence, every new check run against a case that must fail.** Deleting the numerical-contract frame from the production encoder failed `executable_coverage_retains_each_replay_and_substitution_boundary` at its contract-only inequality (earlier session). Resumption added six more:

- deleting the residual-proof frame failed `pending_and_refused_proofs_have_no_executable_coverage_spelling` and `executable_coverage_retains_each_replay_and_substitution_boundary`;
- deleting the occurrence frame failed `completion_receipts_cannot_be_cross_wired_between_real_occurrences` and `executable_coverage_retains_each_replay_and_substitution_boundary`;
- adding `semantic_snapshot` and `scalar_snapshot` back into the coverage encoder failed `executable_coverage_excludes_unused_authority_but_retains_reached_scalar_provenance` — the ADR 0072 exclusion is what that test measures;
- building the two artifact fixtures from genuinely different kernel programs failed the new kernel-program leg of `an_unused_semantic_provider_revision_does_not_change_identity`, so that assertion is a live comparison rather than a value against itself;
- making the coverage field public, adding `impl From<&[u8]>`, and adding an `executable_coverage_identity` accessor to `PendingIndexRefinementReceipt` each made its corresponding `compile_fail` doctest compile and therefore fail — all three reported "Test compiled successfully, but it's marked `compile_fail`".

Every perturbation was reverted and the green run below was taken on the restored tree.

**Measurement — resumed branch checks** (pinned toolchain, `ac97efe0` merged tree): `cargo fmt -p tiler-ir -p tiler-artifact -- --check` clean; `cargo check --workspace --all-targets` clean; `cargo nextest run -p tiler-ir -p tiler-compiler -p tiler-artifact --no-fail-fast` passed 1,557/1,557 with three configured skips; `cargo test -p tiler-ir -p tiler-compiler -p tiler-artifact --doc` passed 23/23 with one ignored, including all three new compile-fail cases; `cargo clippy -p <pkg> --all-targets -- -D warnings` exited 0 for each of the three crates. The workspace gate remains red for an unrelated reason recorded in `restore-the-metal-toolchain-so-the-workspace-gate-can-run-green` (missing metallib, `tiler-macros` and `tiler-prototype-compile`); none of those packages is touched here.

## Acceptance — 2026-08-04

Tom accepted the `IndexRefinementExecutableCoverageIdentity` public boundary as drafted (no re-encoding of the graph-determined operation subjects) in a direct session message on 2026-08-04; the acceptance was relayed and executed by the orchestrator, which merged the branch and moved the `docs/ir.md` disclosure from proposal to accepted wording in the same integration. The dependent stage-coverage binding work is released.
