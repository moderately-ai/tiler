---
id: decide-whether-refinement-evidence-may-reach-a-physical-provider
title: Decide whether refinement evidence may reach a physical provider
status: todo
priority: p1
dependencies: []
related: [carry-the-gather-relation-through-the-compiler-vertical, decide-the-data-dependent-index-representation-public-surface, emit-the-indirect-gather-on-metal]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, gather, identity]
---
## User-visible outcome

Physical planning can either obtain a scheduled gather's bounds proof through an accepted seam, or the repository records that it cannot and says what a provider gets instead — so the gather frontier is blocked by a decision on record rather than by a wall nobody chose.

## Why this exists

Filed 2026-08-22 by the coordinator when `carry-the-gather-relation-through-the-compiler-vertical` **stopped at a typed wall rather than minting a boundary**. That was the right call: it landed recognition, normalization, and request identity — a gather request now advances past arithmetic recognition on a U32-capable profile — and refused to invent the seam the frontier half needs.

**Fact — the wall is typed and named, not a silent gap.** `crates/tiler-compiler/src/physical.rs` declares `RegionVocabularyWall::GatherProofUnavailable`, renders it as `"gather-proof-unavailable"`, and returns it from the region-vocabulary check. Verified by the coordinator at three sites in that file.

**Fact — the blocking property is an identity binding, not an access-control choice.** A scheduled `BoundsProofKind::GatherSource` carries a `GatherIndexBoundsProof` that only the index layer's verifier-private deriver mints, and that proof **binds a `CanonicalIndexRegionIdentity`**. So physical planning cannot re-derive one without binding a schedule to a region nothing has lowered. The value exists at the right time — the delivering lane reports `resolve_lowering` and `enumerate_frontier` in the same scope — but **no seam carries it**, and `ImplementationContext` documents its provider surface exhaustively without refinement on it.

**Fact — the accepted packet does not answer this, and two of its citations are stale.** It has the frontier call `physical::gather_region` without saying where the proof comes from. Separately, the coordinator verified two of its references do not resolve: `crates/tiler-ir/src/index/refinement.rs` **is a 12-file directory, not a file** — the module-split false-absence hazard, inside an accepted packet — and `NormalizedOutput::epilogue()` does not exist, so there was nothing to update.

## The two questions, which are not the same

1. **May refinement evidence reach a physical provider at all?** This is a public-boundary question. `ImplementationContext`'s provider surface is enumerated deliberately; adding refinement to it widens what a third-party provider sees, and the identity binding means what it would see is not a free-standing fact but a claim about a specific region.
2. **How does `pipeline/verify.rs` obtain such a proof?** It re-derives rather than borrowing planning's copy **on purpose** — that independence is the property making verification meaningful. A seam that lets it borrow would quietly retire that property, which is a different decision from (1) and must not ride along with it.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict before writing packet prose.
- Apply AGENTS.md's decision-packet readiness gate in full. **Enumerate at your base rather than treating the two questions as the option set** — include the status quo, which is honest today: the wall is typed, the refusal is named, and no gather reaches a schedule.
- For each survivor: exactly what public surface it adds, whether a provider could use it to assert a proof it did not earn, what it costs `pipeline/verify.rs`'s independence, and the identity consequence.
- **Eliminate before ranking** any option that lets a provider supply or borrow a proof it cannot independently justify — that inverts the property the verifier-private deriver exists to hold.
- If one option dominates, recommend it rather than manufacturing a choice. If a real trade-off survives, ask Tom exactly one concrete question.
- Repair the packet's two stale citations where you find them, and say so — do not restate them.

## Exact-base Fact audit — 2026-08-22, `5c74ed4afbcddd3fe8794bdaec314ae316a042cb`

Every verdict below comes from reading the complete file, not from the search that located it. Anchors are quoted from the source bytes and each was grepped against the file it names before this was written.

| Fact as filed | Verdict | Evidence |
|---|---|---|
| The wall is typed and named at three sites in `crates/tiler-compiler/src/physical.rs` | **verified** | Declaration `GatherProofUnavailable,`; rendering `Self::GatherProofUnavailable => "gather-proof-unavailable",`; return `.then_some(Err(RegionVocabularyWall::GatherProofUnavailable)),` in `spell_output`'s `NormalizedOutput::Gather` arm. Its own doc comment already states the two questions this ticket exists to answer. |
| A scheduled `BoundsProofKind::GatherSource` carries a `GatherIndexBoundsProof` | **verified** | `crates/tiler-ir/src/schedule/model.rs`, anchor `proof: Box<crate::index::GatherIndexBoundsProof>,`. |
| That proof binds a `CanonicalIndexRegionIdentity` | **verified** | `crates/tiler-ir/src/index/model.rs`, anchor `pub const fn region(&self) -> &CanonicalIndexRegionIdentity {` on `impl GatherIndexBoundsProof`; the identity encoder writes it first, `push_slice(&mut out, subject.region.as_bytes());`. |
| The proof is minted only by a **verifier**-private deriver | **imprecise** | The deriver is `pub(super) fn derive_gather_index_bounds(` in `crates/tiler-ir/src/index/builder/gather.rs` — private to the index **builder**, and its live caller is `crates/tiler-ir/src/index/builder/compact.rs`. It runs during region construction and compaction, not in `refinement/verify.rs`. The load-bearing half of the claim holds and is stronger than "verifier-private" suggests: neither record has a public constructor or byte conversion, so the type is unforgeable outside `tiler_ir::index` whatever the module is called. |
| Physical planning cannot re-derive one without binding a schedule to a region nothing lowered | **verified** | The proof identity folds the region identity, so a throwaway region minted here yields different bytes. |
| The value exists at the right time — `resolve_lowering` and `enumerate_frontier` in one scope | **verified** | `crates/tiler-compiler/src/pipeline/planning.rs` binds `let lowering = match resolve_lowering(semantic, verified) {` and later calls `enumerate_frontier(verified, &subject, physical.providers(), physical.calls())` in the same function. |
| `ImplementationContext` documents its provider surface exhaustively without refinement on it | **verified but materially incomplete, and the omission inverts the ticket's premise** | The struct's own doc says the provider reads "the assessed target profile, the resolved numerical realization, the region subject, and this host's own baseline spelling". But `pub fn baseline(&self) -> Option<&BaselineImplementation>` hands back `BaselineImplementation`, whose `pub const fn region(&self) -> &ScheduledRegion {` exposes `pub index: IndexRegion`, whose `pub bounds_proofs: Vec<BoundsProof>` carries `pub kind: BoundsProofKind` — the boxed `GatherIndexBoundsProof` itself. All four are `pub`, `GatherIndexBoundsProof` is `Clone`, and `crates/tiler-compiler/src/physical_provider.rs` re-exports `BaselineImplementation` and `ImplementationContext` to out-of-crate providers. See the reframe below. |
| The accepted packet has the frontier call `physical::gather_region` without saying where the proof comes from | **verified**; `physical::gather_region` does not exist yet (`grep -rn "fn gather_region" crates/tiler-compiler/` returns nothing) |
| Two of the accepted packet's citations are stale | **verified, and repaired** | See "Citation repairs" below. |

## The reframe: question (1) was already answered, and not by this ticket

**Fact.** Under the surface Tom accepted on 2026-08-18, refinement evidence **already reaches a physical provider**. `BoundsProofKind::GatherSource` is a public variant whose `proof` is a public field; a `ScheduledRegion` carries it; `ImplementationContext::baseline()` returns one; and `physical_provider.rs` publishes both types to installed providers. The moment any gather baseline exists, every installed provider can read and `.clone()` a `GatherIndexBoundsProof`. No new public surface is required to make that true, and none can now make it false without reopening an accepted decision.

So the ticket's question 1 — *may* refinement evidence reach a provider — is not open. What is open is narrower and entirely crate-internal: `spell_region` receives `&VerifiedTargetRequest` and nothing else, while the proof lives in `ResolvedLowering` in the same `plan_target` scope. The wall is a missing **argument**, not a missing boundary.

**Fact — and this is the finding that decides the packet.** Nothing anywhere checks the proof's own subject.

- Schedule rule 8 (`crates/tiler-ir/src/schedule/builder/elementwise.rs`, anchor `GatherAddressReadRule::ProofMismatch`) compares four accessors — `proof.source_shape()`, `proof.result_shape()`, `proof.index_shape()`, `proof.axis()` — and none of `region()`, `access()`, `source()`, `index()`, `source_type()`, `index_type()`, `source_extent()`, `domain()`, `kind()`, `facts()`. It **cannot** compare `region()`: `tiler_ir::schedule::IndexRegion` carries no `CanonicalIndexRegionIdentity` counterpart to compare against.
- The compiler's request binding (`crates/tiler-compiler/src/physical.rs`, anchor `fn gather_accesses_match(`) matches `BoundsProofKind::GatherSource { source_shape, result_shape, axis, index_access, index_shape, .. }` — the `proof` field is elided by `..` and never inspected at all.

**Inference, and stated narrowly.** A proof minted for index region A, attached to a schedule for a shape-compatible occurrence B, passes every check in the tree. It stays *bounds*-sound, because both closed kinds — `VacuousEmptyResultDomain` and `U32RangeContainedBySourceExtent` — are functions of `(source_shape, index_shape, axis)` alone, and those three are exactly what rule 8 compares. What it corrupts is **identity**: the schedule encodes `push_slice(bytes, proof.identity().as_bytes())`, and those bytes fold A's region identity, A's access ordinal, A's tensor ordinals, A's resolved types, and A's ordered domain. The result is a schedule bound to a region nothing here lowered — verbatim the failure the wall's own doc gives as its reason for existing. The gap is therefore not hypothetical *after* this ticket; it is latent in the accepted surface today and becomes reachable the moment the first gather baseline exists.

## Option enumeration at this base

Eliminated before ranking, on the ground named:

- **D. Re-derive the proof during physical planning.** Builds a throwaway index region whose canonical identity no lowering produced, then embeds its proof. Conflates identities; this is the wall's stated reason.
- **E. Give `GatherIndexBoundsProof` a public constructor, byte conversion, or `From` impl.** Lets a provider assert a proof it did not earn. Inverts the deriver-private property outright.
- **F. Drop `proof` from `BoundsProofKind::GatherSource` and re-derive the conclusion at the schedule layer from the relation fields.** Tempting, because the two current kinds *are* relation-derivable. Eliminated on two grounds: it claims a complete outcome while depending on the kind set never growing — the invocation-validation work is exactly a third resolution that is **not** relation-derivable — and it retires an accepted public surface, which is Tom's and not this ticket's.
- **G. Further bounded research.** Not applicable. The question was answerable by reading and has been read.
- **H. Deferral.** Not a separate option; represented truthfully by A.

Survivors:

| Option | Public surface added | Can a provider assert an unearned proof? | Cost to `pipeline/verify.rs` independence | Identity consequence |
|---|---|---|---|---|
| **A. Status quo — keep the typed wall** | none | no proof exists to assert | none | none; no gather reaches a schedule |
| **B1. Thread `&ResolvedLowering` into the governed spelling path only** | **none** — `enumerate_frontier`, `govern_spelling`, `spell_region`, `spell_output` and `ResolvedLowering` are all `pub(crate)` or private | not *assert* — the type stays unforgeable — but **yes, relay**: a provider that retains a proof from one subject's `baseline()` can attach it to another proposal, and `gather_accesses_match`'s `..` will not look | none | a third-party proposal can carry a schedule identity naming another occurrence's index region |
| **B2. B1, plus require the retained proof to belong to this occurrence** — thread the same `&ResolvedLowering` into `verify_schedule_with_feasibility` → `verify_region_output_binding` → `gather_accesses_match`, and require `proof.region()` to equal the identity of the occurrence's own realized region | **none** — all four are `pub(crate)` or private | **no** | **strengthens it.** `verify_portfolio` already calls `resolve_lowering` itself (anchor `let lowering = resolve_lowering(semantic, request).map_err(|_| {`); this makes the verifier compare the retained proof against **its own** re-derivation rather than borrowing planning's. Nothing borrows | the proof's region identity is checked against the occurrence that earned it, so schedule and program identity cannot name a region nothing lowered |
| **C. Add a refinement or proof accessor to `ImplementationContext`** | **yes** — a new public accessor on the provider surface | yes, and worse: a provider could build a gather region with no host baseline to be checked against | none directly | same as B1, plus a published seam that outlives the reason for it |

## Nondominated frontier, and the recommendation

**C is dominated by B2** on every dimension: it adds public surface, admits a strictly larger misuse population, and enables nothing B2 does not. It is not on the frontier.

**B1 is dominated by B2.** B2 is at least as good on correctness, maintainability, and host runtime — `resolve_lowering` already runs in both the planning and verification phases, so the comparison costs no additional proof work — and strictly better on fail-closed strictness. B2's whole cost is one crate-private parameter on two more functions.

**A is dominated by B2 given the direction already on record.** A's only advantage is smaller state, and the repository has already decided against paying that price: Tom accepted the literal-only gather surface on 2026-08-18, [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md) has since landed `done`, and [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) is `blocked` with this ticket named in its `dependencies`. A is honest as a description of today, not as a destination.

**Proposal — B2 dominates, and no question is manufactured to accompany it.** Thread the already-derived `ResolvedLowering` into the governed spelling path *and* the proposal-verification path; the governed builder clones the occurrence's own statically proved bounds resolution out of `refinement.single_region()`'s gather access view; `gather_accesses_match` stops eliding `proof` and requires its `region()` to be the occurrence's own. `ImplementationContext` gains nothing.

**This changes what the ticket is for, so it is stated rather than acted on.** B2 adds no public crate, module, trait, type, or call-site boundary — verified item by item above — so it does not meet ADR 0075's reservation bar and is not Tom's under the escalation rule. The ticket's `needs-tom` tag and its "Closes when" clause both presuppose a Tom decision the evidence says is not required. The coordinator owns whether to route it anyway; this lane does not close it.

**If it is routed to Tom regardless, there is exactly one question worth his time, and it is not the one the ticket asks.** *The surface you accepted on 2026-08-18 already hands every installed physical provider a clonable `GatherIndexBoundsProof` through `baseline().region()`, and nothing in the tree checks which occurrence a retained proof belongs to. Should that check land as a prerequisite of the gather frontier work (recommended), or should `baseline()` additionally withhold a gather baseline from providers the way it already withholds a published-and-consumed one?* The second is genuinely fail-closed but costs third-party gather specialization permanently, and buys nothing once the check exists.

### Strongest counterargument, reversal evidence, and perturbations

The strongest counterargument to B2 is that it puts refinement into the request-binding verifier, coupling two layers that are currently independent. The answer is that it couples them in the *safe* direction: `pipeline/verify.rs` derives its own `ResolvedLowering` and would compare planning's retained proof against that, which is the opposite of borrowing. Evidence that would reverse B2: a demonstration that a gather occurrence's realized region identity is not available at `verify_region_output_binding` without re-driving a provider — that would make the check cost real work rather than a comparison, and would move the check to construction only.

Perturbations that must show failure text before B2 is believed, each perturbing the subject and not the assertion: attach a proof minted for a different occurrence whose relation fields agree, and the new binding rule must name the region mismatch while every existing gather control stays green; delete the `region()` comparison and the same case must go green, proving the rule is load-bearing; hold `enumerate_frontier`'s memo key fixed while varying the occurrence, since `crates/tiler-compiler/src/pipeline/planning.rs` documents that function as `a pure function of the request, the subject, and` its physical authorities — quoted to the line break, because the source comment wraps and the full sentence greps to zero and the memo `frontiers_by_subject` depends on it — the added argument is constant across that loop and `FrontierRegionSubject` carries `semantic_members`, so purity survives, but a check must say so rather than the comment.

## Citation repairs

Both were repaired in place in [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md), as dated notes beside the affected text following the house convention, so the retired strings stay greppable inside the notes rather than being silently rewritten.

1. `crates/tiler-ir/src/index/refinement.rs` — dead since the split at `a2e98b27`; the directory `crates/tiler-ir/src/index/refinement/` replaced it. Each named type was relocated at its definition. `InvocationGatherIndexValidationRequirement` is unbuilt at this base, which is correct rather than stale.
2. `NormalizedOutput::epilogue()` — never existed. `impl NormalizedOutput` carries exactly `serial_sum`, `try_serial_sum`, `pointwise`, `contraction`, `gather`, `staged`, `carries_parametric_broadcast`, and `producer_shape_for`.

**The general lesson, since it cost real time here: an accepted packet is not exempt from citation rot.** A file-path citation is the worst kind to leave in one, because a module split makes it fail as *absence* — the direction that reads as "the module was removed" and invites a worker to restore a claim that was never gone. Neither `docs/decisions/[0-9]*.md` file needs the same repair: `0104` and `0105` do still contain the retired path, but only inside their own dated notes, which quote it deliberately so a grep hit lands in the note. Both were read to confirm that.

## Follow-up tickets this packet requires

- **Bind a scheduled gather's retained bounds proof to its own occurrence.** Owns the `gather_accesses_match` change and the `&ResolvedLowering` threading through `verify_schedule_with_feasibility`. Prerequisite of the frontier work, not a detail of it; it closes a gap that is latent in the accepted surface independently of whether the frontier lands.
- **Thread resolved lowering into the governed spelling path.** Owns `enumerate_frontier` → `govern_spelling` → `spell_region`, `physical::gather_region`, and the retirement of `RegionVocabularyWall::GatherProofUnavailable`. Blocked on the ticket above.
- **Record that schedule rule 8 cannot check the proof's region identity.** A one-paragraph source note at `GatherAddressReadRule::ProofMismatch` stating that the schedule layer has no `CanonicalIndexRegionIdentity` counterpart and that the compiler owns that half — so a later reader does not mistake the omission for an oversight and "fix" it at the wrong layer.

## Non-goals

Implementing either seam; removing the typed wall; the Metal gather emission, which now depends on this and on the bounds-proof repair; and re-opening the accepted data-dependent index surface beyond the two citation repairs.

## Closes when

Tom has accepted one route by which physical planning obtains a gather bounds proof — or accepted that it does not, with what a provider receives instead recorded — the effect on `pipeline/verify.rs`'s independence is stated either way, and the packet's stale citations are repaired.
