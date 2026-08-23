---
id: lower-a-recognized-gather-through-a-governed-capability
title: Lower a recognized gather through a governed index-access capability
status: in-progress
priority: p1
dependencies: []
related: [carry-the-gather-relation-through-the-compiler-vertical, thread-resolved-lowering-into-the-governed-spelling-path, decide-the-data-dependent-index-representation-public-surface, emit-the-indirect-gather-on-metal, correct-the-optimizer-contract-capability-count-and-gather-standing]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, gather, lowering, identity, public-boundary]
claimed_from: todo
assignee: worker-gathercap
lease_expires_at: 1787468243
---
## User-visible outcome

A recognized gather occurrence resolves an installed lowering capability and refines to a verified index region carrying its own statically proved `GatherIndexBoundsProof`, so the proof a scheduled gather must embed exists in `ResolvedLowering` at planning time.

## Why this exists

Filed 2026-08-23 by `worker-thread` while auditing [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md), which was dispatched on the premise that the proof "sits in `ResolvedLowering` in the same `plan_target` scope — the value exists at the right time and no seam carries it". **That premise is false at `9b61b563`: the value does not exist, for any gather, ever.** No installed capability lowers a gather, so `resolve_lowering` refuses the whole compilation before physical planning is reached.

This is remainder item 2 of [`carry-the-gather-relation-through-the-compiler-vertical`](carry-the-gather-relation-through-the-compiler-vertical.md) — "**The governed lowering capability row (21 to 22)** and its `GovernedGatherF32` provider. Independent of (1) and the correct next lane." — which was written down in that ticket's Remainder and **never filed as a ticket**. Every open gather lane above it inherited the gap.

## Facts, each read at `9b61b563` in the file it names

**Fact — the compiler's lowering facade cannot emit a gather.** `IndexAccessLoweringContext` (`crates/tiler-compiler/src/capability.rs`, anchor `pub struct IndexAccessLoweringContext<'a> {`) exposes `dimension`, `input_tensor`, `output_tensor`, `constant`, `dimension_expr`, `linear_combination`, `floor_div`, `modulo`, `read`, `write`, `apply`, `apply_in`, `reduce`, and `output`. There is **no `gather_read`**, and `grep -rn "fn gather_read" crates/` finds it only at `crates/tiler-ir/src/index/builder.rs` (the IR builder), `crates/tiler-ir/src/index/law.rs`, `crates/tiler-ir/src/index/model.rs`, and one reference-oracle test helper. `git log -S "gather_read" -- crates/tiler-compiler/src/capability.rs` is **empty**: the facade method was drafted and withdrawn inside the parent lane and never landed.

**Fact — the governed registry carries no gather row.** `grep -rn -i "gather" crates/tiler-compiler/src/governed.rs` returns exactly one hit, a comment at line 1868 about a reindex that is "not a gather". `grep -rn -i "gather" crates/tiler-compiler/src/capability.rs` returns **zero**. `GOVERNED_INDEX_ACCESS_CAPABILITIES = 21` is unchanged.

**Fact — so `resolve_lowering` refuses every gather, and the tree already asserts it.** `a_gather_occurrence_resolves_no_lowering_at_this_base` in `crates/tiler-compiler/src/request/tests.rs` requires `resolve_lowering` to answer `Err` with reason `missing-capability` for a gather program, and carries a negative control proving the same call answers `Ok` for an elementwise fixture. It passes at this base.

**Fact — the refusal is *before* physical planning, not inside it.** `enumerate_complete_plans` in `crates/tiler-compiler/src/pipeline/planning.rs` opens with `let lowering = match resolve_lowering(semantic, verified) {` and returns `Err(lowering_failure(&source, cause))` on the error arm — above `enumerate_covers`, above `enumerate_frontier`, above `govern_spelling` and `spell_region`. So **no gather member set can reach the region-vocabulary check through the pipeline at all**; the only live caller of `spell_region` with gather members is a unit test that hands it a request directly.

**Fact — the end-to-end test already names this ticket's work as the prior authority.** `a_governed_gather_refuses_at_dispatch_before_governed_lowering` (same file) pins the widened-profile compile at `("lowering", "missing-capability")` and its own doc names two authorities standing between that refusal and one that could grant a gather a schedule: the governed capability row itself, and `RegionVocabularyWall::GatherProofUnavailable`, *"which is what physical planning answers once a row exists"*. The row is first. That doc comment wraps across `///` lines, so the anchor is the single-line fragment `authorities stand between this refusal and one that could`; the full sentence greps to zero and would read as false absence.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict before editing. The counts and the empty greps are the load-bearing ones.
- Land `IndexAccessLoweringContext::gather_read` to the accepted spelling in [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md), **and the governed row that consumes it in the same change** — the parent lane withdrew the facade precisely because "a public surface with no consumer is not decision-ready", and re-landing it alone would repeat that.
- Land the `GovernedGatherF32` provider and its capability row, moving `GOVERNED_INDEX_ACCESS_CAPABILITIES` 21 to 22.
- Establish that the emitted region refines: `refine` must produce `OccurrenceEvidence::Refined` whose `IndexRefinement::single_region()` is `Some`, and whose gather access exposes `bounds_resolution().statically_proved()`. A gather whose bounds obligation is *not* statically discharged mints a validation requirement instead and must stay refused — that population belongs to [`admit-an-invocation-scoped-gather-index-validation-receipt`](admit-an-invocation-scoped-gather-index-validation-receipt.md), not here.
- Flip `a_gather_occurrence_resolves_no_lowering_at_this_base` and the widened half of `a_governed_gather_refuses_at_dispatch_before_governed_lowering` in the same change. Both currently assert the opposite of what this ticket delivers; leaving either would leave the suite asserting a falsehood. **Do not touch `UNPLANNED_OPERATIONS` or `gather_is_absent_from_the_governed_fusion_roles`** — the parent lane verified both remain true and recorded that flipping them asserts the opposite of the tree.

## Public boundary

**`IndexAccessLoweringContext::gather_read` is a `pub` method on a `pub` type**, re-exported to out-of-crate lowering providers. That meets ADR 0075's reservation bar, so its exact signature is Tom's unless it lands verbatim as the already-accepted packet spells it. State which, with the anchor, before writing it.

## Identity domains

**The request-subject domain steps for every program in the repository, not only for gathers.** `CanonicalLoweringRegistryIdentity` folds the capability list, and `crates/tiler-compiler/src/request/subject.rs` writes it into every request subject: `push_slice(&mut bytes, self.lowering_registry.as_bytes());`. Adding one row therefore moves every pinned request-subject golden and everything keyed on one. Derive the full consequence on the merged tree and recompute the pins there rather than on your base. This blast radius is the reason the row is its own lane rather than a detail of the frontier work.

## Non-goals

`physical::gather_region`, the `govern_spelling` gather arm, and retiring `RegionVocabularyWall::GatherProofUnavailable` — those are [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md), which this unblocks. The invocation-validation vocabulary. Any Metal, artifact, manifest, or cache surface. Re-opening the accepted data-dependent index surface.

## Closes when

A recognized gather resolves a governed capability, refines to a single verified index region whose gather access carries a statically proved bounds resolution, `resolve_lowering` answers `Ok` for a gather program, no test asserts the absence this lane removes, every identity consequence is derived on the merged tree with pins recomputed there, each new refusal has been watched firing on a perturbed subject, and the workspace gate is green.

## Coordinator pre-dispatch note — 2026-08-23 at `5b2e4414`: the accepted spelling exists, and where to find it

Every Fact in this ticket reproduced for the coordinator: `gather_read` is absent from `crates/tiler-compiler/src/capability.rs` (**0** hits), `git log -S "gather_read"` against that file is **empty**, `GOVERNED_INDEX_ACCESS_CAPABILITIES` is **21** at `crates/tiler-compiler/src/governed.rs`, and `enumerate_complete_plans` states the ordering itself at the anchor `Lowering-capability resolution precedes every cover`.

**The Public boundary section asks the worker to state whether the signature is Tom's or already accepted. It is already accepted, and this ticket did not name where.** [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md) is `done`, and its acceptance record states that Tom accepted **option B** with *"the three exact public `gather_read` refusals for each nonliteral dimension"*, fixing the accepted surface as *"exactly the reviewed packet at `a25f4268b768f1b0391db34798676f910d4f1660`"*. That ticket spells `pub fn gather_read(` in two places. **Land it verbatim from there.** Any deviation from that spelling is a new public boundary and therefore Tom's — stop and report rather than adjusting it.

**A related edge this ticket was missing, which matters for not re-deriving landed work.** The acceptance routes implementation through [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md), which is **`done`** — and the IR half already landed: `pub fn gather_read(` exists at `crates/tiler-ir/src/index/builder.rs`. So this ticket is **not** a duplicate of it, and the distinction is exact: the *IR builder's* `gather_read` is landed; the *compiler's* `IndexAccessLoweringContext::gather_read` facade method is the one that was drafted and withdrawn. Read the landed IR method before writing the facade — its refusals are the accepted three and the facade must not restate them differently.

**Unverified by the coordinator and left for the worker:** the refinement claim (`OccurrenceEvidence::Refined`, `single_region()`, `statically_proved()`), the two test flips, and the full identity blast radius. The identity consequence in particular is stated as reaching every request subject in the repository; derive it on the merged tree and recompute pins there, as this ticket already says.

## Worker exact-base Fact audit — 2026-08-23 at `a0fd5af21030e76245e48d2d2b3a9632caca77dc`

`worker-gathercap` re-ran every Fact in the file it names before editing. All five reproduce; none needed repair.

| Fact | Verdict | Evidence at this base |
| --- | --- | --- |
| The compiler's lowering facade cannot emit a gather | **verified** | `grep -c "gather_read" crates/tiler-compiler/src/capability.rs` → `0`. `grep -rn "fn gather_read" crates/` → four hits, all in `tiler-ir` (`index/law.rs`, `index/builder.rs`, `index/model.rs` twice) plus `crates/tiler-reference/tests/index_region_oracle.rs`; none in `tiler-compiler`. `git log -S "gather_read" -- crates/tiler-compiler/src/capability.rs` → empty. |
| The governed registry carries no gather row | **verified** | `grep -rn -i "gather" crates/tiler-compiler/src/governed.rs` → exactly one hit, line 1868, the reindex comment. The same grep over `capability.rs` → `0`. `GOVERNED_INDEX_ACCESS_CAPABILITIES: usize = 21` at `governed.rs:244`. |
| `resolve_lowering` refuses every gather, and the tree asserts it | **verified** | `a_gather_occurrence_resolves_no_lowering_at_this_base` at `crates/tiler-compiler/src/request/tests.rs:6658`, passing at this base with its elementwise negative control. |
| The refusal precedes physical planning | **verified** | `let lowering = match resolve_lowering(semantic, verified) {` at `pipeline/planning.rs:242`, under the anchor `Lowering-capability resolution precedes every cover` at line 238, above `enumerate_covers`. |
| The end-to-end test names this ticket's work as the prior authority | **verified** | `a_governed_gather_refuses_at_dispatch_before_governed_lowering` at `request/tests.rs:1295`; the wrapped anchor `authorities stand between this refusal and one that could` resolves at line 1287, and the full sentence greps to `0` exactly as the ticket warns. |

**Public boundary — landed verbatim, not re-decided.** The facade is the second `pub fn gather_read(` in [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md), under the anchor `The governed compiler lowering registry gains a revision-1 gather capability and this exact facade`. Its seven-line signature — `&mut self`, `source: TensorId`, `index: TensorId`, `domain: &[DimensionId]`, `source_coordinates: &[IndexExprId]`, `index_coordinates: &[IndexExprId]`, `axis: Axis`, returning `Result<ScalarValueId, LoweringEmitError>` — was copied from that record character for character. The facade states **no** refusal of its own: it delegates to `IndexRegionBuilder::gather_read` and converts through the existing `From<IndexBuildError>`, so the accepted three literal refusals are stated once, in the IR.

## Identity domains — derived on the tree this lane merges into

`main`, `origin/main`, and this branch's base were all `a0fd5af2` when the work was done (`git rev-list --left-right --count main...HEAD` → `0 0`), so the tree the pin was recomputed on *is* the merged tree. **If `main` moves before this merges, the coordinator must recompute the pin on the merged tree rather than trusting the value below.**

**Steps:**

- `CanonicalLoweringRegistryIdentity` — one more `LoweringCapabilityKey` and its four pooled authority identities enter `compute_identity`.
- The request subject, for **every** program in the repository, not only gathers — `push_slice(&mut bytes, self.lowering_registry.as_bytes())` in `crates/tiler-compiler/src/request/subject.rs`. The *encoding version* does not step: no previously encodable byte moved, only the value fed in, which is exactly the case `explain.rs`'s pin comment describes.
- Everything derived from the request subject: the explain trace's request qualifier, `crate::fusion`'s canonical explain subject, and the kernel-program and artifact identities that embed it. **Exactly one of these is pinned anywhere in the workspace**: `crates/tiler-compiler/src/explain.rs`, `tiler-explain-v10 request=8bdb7dd58e3aa485` → `e1ce290f22c582a1`, recomputed here. `grep -rnE 'request=[0-9a-f]{16}' crates/` returns that one line and nothing else; `grep -rnE '"[0-9a-f]{16}"' crates/` returns only hex-digit lookup tables.

**Does not step:**

- The semantic registry snapshot — no operation was registered; `tiler::gather-f32@1` was already in `FrozenSemanticRegistry::standard`. Every `tiler-ir` `semantic/registry.rs` pin is untouched.
- The scalar registry snapshot — the row declares `emitted: Vec::new()` and touches no scalar authority.
- The realization-law sidecar, and so the `realization_registry` bytes in the request subject — `IndexRealizationLaw::gather_f32()` was already registered by `admit-the-selected-data-dependent-index-representation`. This lane registers no law.
- `CanonicalIndexRegionIdentity` and every index-refinement content/occurrence tag — no existing region's identity moves; a gather merely now produces such values where none existed.
- Target-profile declaration bytes, the recognized-output subject encodings pinned as `DECLARED_INPUT`/`POINTWISE_PROLOGUE` in `request/tests.rs`, and every `result_sha256` numerical digest — all unaffected, and all green.

Derived by reading `subject.rs`'s encoder and `capability.rs`'s `compute_identity`, then confirmed empirically: `cargo nextest run --workspace --no-fail-fast` before recomputing the pin reported `4057 tests run: 4056 passed, 1 failed` with that one pin the sole failure, and removing the capability row again reverted it to `8bdb7dd58e3aa485`.

## Subject perturbations watched firing

Each perturbs the subject, never the assertion, and each was reverted.

| Perturbation | What it proves | Quoted failure |
| --- | --- | --- |
| Delete the `GovernedGatherF32` row from `governed_index_access_capabilities` | Every dependent check, including the identity pin, is caused by this row | `the governed gather capability lowers a recognized gather: Resolve { member: SemanticMemberId(0), source: MissingCapability { family: IndexAccess, operation: OpKey(… "gather-f32" …) … } }`, plus `left: ("lowering", "missing-capability")  right: ("planning", "region-vocabulary")` and the pin reverting to `left: "tiler-explain-v10 request=8bdb7dd58e3aa485"` |
| Declare the index operand before the source in the provider | The operand order is checked, not assumed | `source: OperandInterface { position: 0 }` |
| Exchange the source and index coordinate runs handed to `gather_read` | The composed-domain split is checked against the law's own realization | `source: IrVerifier(SemanticRealizationMismatch { expected: CanonicalIndexRegionIdentity([…]), actual: CanonicalIndexRegionIdentity([…]) })` |
| Leave `GOVERNED_INDEX_ACCESS_CAPABILITIES` at 21 | The census reaches its subject | `assertion left == right failed  left: 22  right: 21` at `governed.rs:3576` |
| Make `spell_output`'s gather arm raise `FusedPrologueUnspellable` instead | The new trace assertion names the wall rather than any vocabulary gap | `the trace must name the gather wall as the cause: tiler-explain-v10 request=…` followed by the whole rendered trace |

One perturbation **did not** redden and the negative result is recorded rather than dropped: interning the coordinate expressions in reverse axis order (then reversing the vector back) left the test green, so `CanonicalIndexRegionIdentity` is structural rather than dependent on expression-creation order. The provider therefore has to match the law's *structure*, not its emission sequence.

## Outcome

Landed on `tkt/lower-a-recognized-gather-through-a-governed-capability`.

- `IndexAccessLoweringContext::gather_read` in `crates/tiler-compiler/src/capability.rs`, verbatim from the accepted packet, delegating with no refusal of its own.
- `GovernedGatherF32` in `crates/tiler-compiler/src/governed.rs` and its `tiler::gather-f32@1` row with signature `[f32, u32] -> [f32]`, `emitted: Vec::new()`, provider `tiler::governed-index-access.gather-f32@1`, revision 1. `GOVERNED_INDEX_ACCESS_CAPABILITIES` 21 → 22.
- `a_gather_occurrence_resolves_no_lowering_at_this_base` → `a_gather_occurrence_resolves_a_governed_lowering_and_refines`: `resolve_lowering` answers `Ok`, the evidence is `OccurrenceEvidence::Refined`, `single_region()` is `Some`, its gather access exposes `bounds_resolution().statically_proved()` with kind `U32RangeContainedBySourceExtent`, and the proof's region equals the realized region's `canonical_identity()`. Its negative control substitutes the installed authority through `install_governed_index_access(&mut builder, &[gather_f32_op()])`.
- `a_governed_gather_refuses_at_dispatch_before_governed_lowering` → `a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary`: the widened half moved `("lowering", "missing-capability")` → `("planning", "region-vocabulary")`, with the named wall read out of the explain trace so the coarse capability class cannot stand in for it.
- `a_gather_proof_minted_for_another_region_is_refused` gained the **positive control** its own doc said belonged to this lane: the occurrence's own realized region's proof admits the identical scheduled region, while the transplant and an `unresolved_for_test()` lowering both refuse under `Intrinsic { rule: "request-binding" }`.

**Deliberately not touched.** `UNPLANNED_OPERATIONS` and `gather_is_absent_from_the_governed_fusion_roles` are both still true and unchanged — a gather still consumes no numerical freedom and still holds no fusion role. Only the *prose* above `UNPLANNED_OPERATIONS` was repaired, and it was already stale at this base in two of its three clauses: it claimed "no schedule access relation, no realization law row, and no lowering capability names this family", while `LogicalAccess::GatherSource` and `IndexRealizationLaw::gather_f32` both already existed.

**Population this lane does not distinguish.** A gather whose bounds obligation is *not* statically discharged — `gather_program()`'s `[4, 2]` source is one — now also resolves a lowering and mints a `GatherIndexValidationRequirement`. It stays refused, but at the same `RegionVocabularyWall::GatherProofUnavailable` a *proved* gather stops at, because the wall declines every gather member set unconditionally. Separating the two is [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md) and [`admit-an-invocation-scoped-gather-index-validation-receipt`](admit-an-invocation-scoped-gather-index-validation-receipt.md), not this lane.

**Out-of-scope repair filed rather than made.** `docs/compiler/optimizer.md` states the capability population as twenty-one and gives the gather family an account this change supersedes; that path is `contracts/optimizer`, which this lane does not hold. [`correct-the-optimizer-contract-capability-count-and-gather-standing`](correct-the-optimizer-contract-capability-count-and-gather-standing.md).
