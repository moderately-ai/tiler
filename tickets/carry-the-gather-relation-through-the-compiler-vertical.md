---
id: carry-the-gather-relation-through-the-compiler-vertical
title: Carry the gather relation through the compiler vertical
status: in-progress
priority: p1
dependencies: [admit-the-selected-data-dependent-index-representation]
related: [decide-the-data-dependent-index-representation-public-surface]
scopes: [implementation/compiler, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, gather, identity, normalized-output]
claimed_from: todo
assignee: worker-gathervert
lease_expires_at: 1787438923
---
## User-visible outcome

A compiler request carrying a data-dependent gather reaches the scheduled-region relation the IR now admits, instead of being recorded as absent at three separate sites while the layer beneath it is fully built.

## Why this exists

Split out 2026-08-22 by the coordinator. The gather lane landed the schedule vocabulary and the kernel wall and **stopped at a coherent boundary rather than pushing through** — the relation is admitted and verified, nothing above can reach it, and the kernel wall refuses it by name. Its stopping point is the right one; this is the vertical above it.

**Read the accepted public-surface ticket before this one.** [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md) carries the exact accepted spellings, and the delivering lane reports that the parent ticket's remainder list **names the pieces without them** — it drafted a two-way resolution at schedule level from that list alone, found it wrong against the accepted surface, and discarded it. Anyone working from the remainder list alone will re-make that error.

## Facts

**Fact — `NormalizedOutput` is `pub(crate)` with five variants and roughly twenty exhaustive matches**, each of which needs a real gather answer rather than a stub; `spell_output` would have to build a gather region. Reported by the delivering lane; **re-derive the count at your base and say which unit you report.**

**Fact — nothing named `PendingInvocation*` exists anywhere in the tree.** The invocation-validation vocabulary is to be created, not extended. Re-verify by searching the tree rather than a named path — `AGENTS.md` records that a file-path citation fails as false absence after a module split because the named file usually still exists.

**Fact — three sites record gather's absence and must flip together.** The delivering lane named them and noted the earlier remainder listed none of them: `UNPLANNED_OPERATIONS` in `crates/tiler-compiler/src/policy.rs`, `gather_is_absent_from_the_governed_fusion_roles`, and `gather_is_absent_from_the_real_request_recognition_operation_set`. **A change that flips the capability without flipping these leaves two tests asserting the opposite of the tree.**

**Fact — the tags this lane consumes were verified free by the coordinator at `754b63fb`.** Compiler access-relation tag `0x06` and the governed lowering capability row 21→22. Tag spaces here are **per-frame, not global** — `TAG_LINEAR_IDENTITY` and `TAG_COVERAGE_PADDED` are both `0x01` deliberately — so a value appearing in another frame is not a collision. A *reserved* value is: in the schedule frame, `0x09` is retired-and-never-reused and `0x36` in the reduction frame is reserved for `CooperativeContractionSplit`. Re-derive whatever you take.

## Required work

- Re-audit every Fact at your base with a per-Fact verdict before editing.
- Deliver `NormalizedOutput::Gather` / `NormalizedOutputSubject::Gather`, the `gather-f32.v1` output subtag, the access-relation tag, the invocation-validation vocabulary, and the governed capability row — to the **accepted** spellings, not the remainder list's paraphrase.
- Flip all three absence-recording sites in the same change as the capability.
- Land the ADR 0108 schedule-clause amendment with its catalog and contract sweep. AGENTS.md: applying an ADR means aligning status, catalogs, contracts, terminology, and released graph edges — read the affected documents in full before declaring the sweep complete.
- State every identity domain that steps and every one that does not, with the derivation, and **recompute pins on the merged tree**, not on your base. The layer beneath stepped nothing; do not assume that carries.

## Evidence

- Perturb the subject separately for each new refusal and quote the failure text. The lane below this one found a rule that its own read-count gate made **unreachable by pigeonhole** — it caught that by asking what it would take for each rule to say *no*, which is the check to run here too.
- Before trusting any check, state what it would take for it to say *no*, and confirm that case is reachable.
- Size enumerations from the type. `core::mem::variant_count` makes a widened vocabulary a build error at the enumeration rather than a census that silently shrinks; the lane below replaced a five-of-twelve tag sample with exactly that, because a sample could not have shown the tag it needed was free.

## Non-goals

The oracle's independent proof-identity check — blocked on a public-boundary decision and owned by [`decide-how-the-oracle-independently-checks-a-gather-proof-identity`](decide-how-the-oracle-independently-checks-a-gather-proof-identity.md). Any artifact, manifest, cache, or Metal surface. Re-opening the accepted public surface.

## Closes when

A gather request reaches the admitted relation, no site records gather as absent while the capability admits it, the ADR sweep is complete against documents read in full, every identity consequence is derived on the merged tree, each refusal has been watched firing, and the repository gate is green.

## Exact-base Fact audit — 2026-08-22, `3ba89314b17d1eca2da5328f608aaf83e54b6826`

Read in full at this base before any edit: this ticket; the complete accepted packet `decide-the-data-dependent-index-representation-public-surface`; `admit-the-selected-data-dependent-index-representation`; root `AGENTS.md`; ADR 0108; and the complete `crates/tiler-compiler/src/request/{normal_form,subject,recognize}.rs`, plus the implicated regions of `physical.rs`, `pipeline.rs`, `pipeline/verify.rs`, `policy.rs`, `fusion_legality.rs`, `capability.rs`, and `crates/tiler-ir/src/semantic/gather.rs`.

| Fact | Verdict |
|---|---|
| `NormalizedOutput` is `pub(crate)` with five variants and roughly twenty exhaustive matches | **verified, and the count is now exact rather than approximate.** Declared `pub(crate) enum NormalizedOutput` in `request/normal_form.rs` with five variants. The match census was taken **from the type**, not by grep: a throwaway sixth variant was added and `cargo check -p tiler-compiler --all-targets` produced exactly **20** `E0004` sites, unit = distinct `file:line:col`. By file: `normal_form.rs` 14, `physical.rs` 2, `pipeline.rs` 1, `pipeline/verify.rs` 1, `request/subject.rs` 1, `request/tests.rs` 1. The same probe on `NormalizedOutputSubject` gives **4** (`physical.rs` 3, `subject.rs` 1) — a population the Fact does not mention and which must move with it. |
| `spell_output` would have to build a gather region | **false as stated, and the correction is this lane's central finding.** `spell_output` only *classifies*; the builder is `frontier::govern_spelling`'s call to a region builder. That builder cannot exist at this base — see the blocking discovery below. |
| Nothing named `PendingInvocation*` exists anywhere in the tree | **verified.** `grep -rn 'PendingInvocation' --include='*.rs' --include='*.md' .` returns five hits, **all in `tickets/*.md`** and zero in `crates/`. Searched over the tree rather than a named path, as instructed. |
| Three sites record gather's absence and must flip together | **imprecise in both directions, and repaired.** All three named sites exist: `UNPLANNED_OPERATIONS` at `crates/tiler-compiler/src/policy.rs`, `gather_is_absent_from_the_governed_fusion_roles` in `fusion_legality.rs`, `gather_is_absent_from_the_real_request_recognition_operation_set` in `request/tests.rs`. But **only one of the three moved**, and a **fourth the Fact does not name did**. Landing the capability moved `gather_is_absent_from_the_real_request_recognition_operation_set` (recognition now resolves a gather) and `a_governed_gather_refuses_at_dispatch_before_arithmetic_recognition` in the same file. The other two stayed true and were deliberately **not** touched: `UNPLANNED_OPERATIONS` is about the *target* capability table `operation_capabilities()`, which gains a gather row only when a target claims the family; the fusion-role test asserts `FusionNumericalCapabilities::governed().classify` answers `None`, and the accepted packet adds no fusion role — `policy.rs`'s own comment records that classifying one "would assert a discharge that nothing performs". Flipping either would have made the suite assert something false. |
| Compiler access-relation tag `0x06` is free | **verified at this base.** `encode_access_relation` in `request/subject.rs` writes `0x01`, `0x02`, `0x03`, `PARAMETRIC_BROADCAST_ACCESS_TAG = 0x05`, and the wildcard refusal `0x00`; `UNREAD_DECLARED_INPUT_TAG = 0x04` is reserved against exactly this encoder. `0x06` is the first free value above all of them. |
| Governed lowering capability row 21 to 22 | **verified as a count, not consumed.** `GOVERNED_INDEX_ACCESS_CAPABILITIES = 21` in `governed.rs`, `#[cfg(test)]`. This lane adds no row; see the remainder. |

Two further corrections to the **accepted packet**, both recorded rather than worked around:

- The packet states the refinement-bound types "live in `crates/tiler-ir/src/index/refinement.rs`". **No such file exists.** `refinement` is a module *directory* of twelve files totalling ~7,400 lines; `IndexRefinementVerificationOutcome` is in `refinement/receipt.rs`. Verified by `ls`, not by a failed grep. This is the file-path-citation hazard the brief named, in its live form.
- The packet lists `epilogue()` among the `NormalizedOutput` accessors that must return `None` for Gather. **No `epilogue()` accessor exists** on `NormalizedOutput` at this base, so there was nothing to update. The other five named accessors do exist and were updated.

## Blocking discovery — physical planning cannot obtain the proof a scheduled gather must carry

**This is why the lane stops where it does, and it is a gap in the accepted packet rather than in the implementation.**

The packet specifies `frontier::govern_spelling` handling `RegionSpellingKind::Gather(write)` by calling `physical::gather_region`. A scheduled gather region must carry `BoundsProofKind::GatherSource`, which the layer beneath landed as holding `proof: Box<tiler_ir::index::GatherIndexBoundsProof>`. That proof:

- is minted **only** by the index layer's verifier-private `derive_gather_index_bounds`, and
- binds a `CanonicalIndexRegionIdentity` (`GatherIndexBoundsProof::region()`).

So physical planning cannot re-derive one: a throwaway index region built during planning would carry a *different* region identity than the one lowering produced, and embedding its proof would bind a schedule to a region nothing lowered — forking the identity domain the module exists to solely own.

The value does exist at the right time. `resolve_lowering` runs at `crates/tiler-compiler/src/pipeline/planning.rs:242` and `enumerate_frontier` at `planning.rs:469`, in the same function with the `ResolvedLowering` live in scope; `IndexRefinement::single_region()` and `realization()` reach a `VerifiedIndexRegion`, whose gather access exposes `bounds_resolution().statically_proved()`. **No seam carries it across.** `ImplementationContext` holds exactly `request`, `subject`, and a lazily-derived `baseline`, and its documentation states positively and exhaustively what an installed provider may read.

Closing this needs two decisions the accepted packet does not make, and a worker must not mint either:

1. **May refinement evidence reach a physical provider at all?** Adding it to `ImplementationContext` contradicts that type's stated contract, which is a public provider surface and therefore Tom's under ADR 0075. A `pub(crate)` accessor for the governed provider alone is the narrower option and does not change the public surface.
2. **How does the independent verify stage obtain the same proof?** `pipeline/verify.rs` re-derives lowering from scratch deliberately — `pipeline.rs` records that it "may not reuse a planning intermediate, because a verifier handed the value it is checking compares that value to itself and can never say no". A gather proof flowing from planning's `ResolvedLowering` into a scheduled region must be re-derived and compared there, not borrowed, or that independence is silently lost.

Rather than invent either, this lane makes the stop typed and named: `RegionVocabularyWall::GatherProofUnavailable`, reason `gather-proof-unavailable`.

## Outcome — the request layer carries the gather; physical planning declines it by name

Landed and gated:

- `NormalizedGather` and `NormalizedOutput::Gather`, with a real answer at **all 20** exhaustive match sites — no stub arms. `input_elements_at` answers each operand's own declared count, `reads_declared_input` recognizes both ordinals including the address operand, `members`/`owns_region_members` are the one-occurrence singleton, `producer_shape_for` is self.
- `NormalizedGatherSubject`, `NormalizedOutputSubject::Gather`, and the `gather-f32.v1` output sub-tag with all 4 subject-side match sites. The association spelling tag is written although only option B exists, so the recorded reopening of source-side-versus-fieldless can add `0x02` without moving these bytes.
- `encode_access_relation`'s `LogicalAccess::GatherSource` arm at compiler tag `0x06`.
- `recognize_gather`, and a **narrow, position-based** exemption in `recognized_program_arithmetic`: only the value at operand position 1 of a `tiler::gather-f32@1` occurrence is exempt from the program's one arithmetic. `recognized_arithmetic` still names exactly two widths, so no general U32 admission occurs — which is what the packet requires.
- Gather arms in `physical::{published_shape, declared_input_for_verified_access, verify_region_output_binding}` plus `gather_accesses_match` and `expression_is_the_single_leaf_identity`, `pipeline::output_region_role`, and `pipeline::verify::merges_nothing`.
- `RegionVocabularyWall::GatherProofUnavailable`.

**Where a gather request stops now, and it is two layers further than before.** On the governed target it is still refused for its exact U32 index before recognition (`DTypeNotDispatchable`), unchanged. On the U32-capable test profile it advances past arithmetic recognition — which previously refused it under `dtype-recognized` — through recognition, normalization, and subject projection, and stops at `phase: "lowering", rule: "missing-capability"`. Behind that row sits `gather-proof-unavailable`. **No gather acquires a schedule, kernel, artifact, manifest, cache, or dispatch route.**

### Identity domains — every one derived, none stepped

- `tiler.compiler.request-subject.v6` **does not step.** `gather-f32.v1` is a fresh output sub-tag and `0x06` a fresh relation tag; both are populations the earlier vocabulary could not express, so every previously encodable subject encodes to exactly the bytes it did. Evidence: the module's existing pinned request-subject goldens and qualifiers pass **unedited**, and the whole workspace suite is green.
- **The lowering registry identity does not move**, because no capability row was added.
- **The realization-registry identity does not move**, because no law row was added.
- **The schedule identity domain does not move**, because no schedule bytes are written by this lane.
- **The semantic registry does not move**; `GatherF32` keeps `LiteralOnly` participation.
- **No artifact, manifest, or cache identity exists to move** — nothing gets that far.

Nothing expected to hold moved, so there was nothing to stop and report.

### Subject perturbations, each driven separately with its quoted failure

| Perturbation | Failure text |
|---|---|
| gather arm returns `PartialCoverage` instead of the named wall | `a gather's own member set is declined by name, not reported as partial coverage` / `left: Err(PartialCoverage)  right: Err(GatherProofUnavailable)` |
| gather relation tag moved onto the parametric carrier's `0x05` | `the gather source relation takes the request frame's next free tag` / `left: Some(5)  right: Some(6)` |
| `LinearIdentity` moved onto `0x06`, holding the gather pin green | `two encodable access relations share a request-subject tag: [6]` / `left: 1  right: 2` |
| the axis dropped from `encode_output_subject`'s gather arm | `the gathered axis must move the subject` |

**Two checks were found unable to fail and were rebuilt before being trusted.**

1. The relation test originally asserted the gather tag differed from each named constant *after* pinning it to `0x06`. Those assertions were **unreachable by pigeonhole** — any perturbation of the tag trips the pin first, so they could only ever prove the pin runs. The distinctness check now collects the encoder's own first bytes and requires them pairwise distinct, which fails independently; the third row above is that demonstration.
2. The subject test originally compared whole `canonical_explain_subject_bytes` for two gather programs. It **stayed green with the axis deleted from the encoder**, because a request subject opens with the semantic graph identity, which already separates the two programs — the assertion was measuring the graph identity, not the projection. It now encodes the arm directly via `encode_output_subject(&output_subject(..))`, following the module's existing forge idiom, and the fourth row above is the demonstration that it now catches the dropped field. Two of its ten perturbations — swapping the declared association, and moving the local address ordinal — are unreachable from any authored program and are the ADR 0108 amendment's central claim.

## Remainder — not landed here, and why

Ordered by what unblocks what.

1. **The proof seam**, and it gates everything below it. Both decisions in the blocking discovery above must be answered before `physical::gather_region` and the frontier offer can exist. This is the one item that is not a worker's to take.
2. **The governed lowering capability row (21 to 22)** and its `GovernedGatherF32` provider. Independent of (1) and the correct next lane. Note that `IndexAccessLoweringContext::gather_read` — the accepted facade — was written and then **deliberately removed** from this lane: it is a `pub` method on a `pub` type with no caller until the row exists, and a public surface with no consumer is not decision-ready.
3. **`RegionSpellingKind::Gather`, `physical::gather_region`, and the `frontier::govern_spelling` arm** with the packet's exact costs. Blocked on (1). `verify_region_output_binding`'s gather arm and `gather_accesses_match` are already written and correct, and are what that lane's region must satisfy.
4. **The invocation-validation vocabulary**: `InvocationGatherIndexValidationRequirement` in `tiler-ir`'s `index::refinement` module, the two `InvocationValidationRequired` outcomes, `tiler_compiler::legality::PendingInvocationIndexValidation`, and the `gather-invocation-validation-required` reason. Untouched here. `IndexRefinementVerificationOutcome` has two variants (`refinement/receipt.rs`) and `IndexRefinementOutcome` two (`legality.rs`); each gains a third.
5. **`NormalizedOutput::gather()` is `#[cfg(test)]` at this base** rather than `pub(crate)`, because its production consumers are (3). The lane that adds them widens it in the same change.
6. **`UNPLANNED_OPERATIONS` and the governed fusion-role test stay as they are**, truthfully, until a target claims the family and a fusion role is actually justified. Neither is a stale assertion to clear.
