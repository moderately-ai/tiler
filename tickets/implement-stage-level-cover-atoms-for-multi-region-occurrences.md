---
id: implement-stage-level-cover-atoms-for-multi-region-occurrences
title: Implement stage-level cover atoms for multi-region occurrences
status: in-progress
priority: p1
dependencies: []
related: [resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage, admit-the-registered-elementary-families-as-recognizable-program-stages, widen-the-staged-realization-law-to-the-registered-elementary-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, cover, identity-domain, p1-spine]
claimed_from: todo
assignee: agent-stage-atoms
lease_expires_at: 1786036406
---

## The decision this executes

**Tom decided on 2026-08-06, at the live session, relayed and executed by the coordinator:** the planner's attribution atom becomes a *(member, stage)* pair — Option A of [`resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`](resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md), whose derivation is the authority for what follows. The grounds, in the priority order Tom stated (correctness, performance, long-term maintainability, code quality): correctness is equal between the options; stage-level atoms are what let a family's internal pass fuse into a neighbouring region (the flash-shaped plan the project exists to reach); one identity migration instead of a guaranteed two; and stages are real domain objects the model should say exist. Option B (a multi-stage region spelling) is *rejected*, not deferred — its internal-boundary opacity would have to be undone by exactly this change later.

## The surface (from the fork ticket's derivation, verified at its cited sites)

`SemanticMemberId` (`crates/tiler-compiler/src/region.rs:123`, `pub(crate)`) is the attribution key throughout: `NormalizedOutput::owns_region_members` (`request.rs`), `physical::spell_output` (first-match by `members ==`), `cover::derive_duplication` (repeated member = duplication, never a split), the region graph, program assembly's stage coverage, and the identity encodings — cover identity, region-occurrence identity, and explain records all encode member positions. The change threads a stage ordinal beside the member everywhere attribution is decided, with single-stage occurrences carrying stage zero so every existing program's semantics are unchanged.

## The identity step, executed whole

Cover identity, region-occurrence identity, and any explain record encoding member positions move together: the encoding change lands with its version step at the owning layer (or per-tag injectivity reasoning if genuinely appends-only — derive which honestly, do not assume appends-only because it is cheaper), the ledger comments move in the same commit, and every pinned identity is recomputed on the tree from observed failing values with each moved pin enumerated in the report. The explain request qualifier (currently `ce6f9106c1c5933b`) will move if the subject bytes reach any changed encoding — verify rather than assume in either direction.

## What this ticket does and does not deliver

It delivers the attribution model: multi-region occurrences become representable, `derive_duplication` distinguishes a split from a duplication, and `spell_output` resolves per stage. It does NOT register any elementary family's law (that is [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md), which supplies the producers of multi-region realizations) — the two compose in the parent keystone. A test proving the new atom is load-bearing must exist without the law widening: the split-reduction's existing partial/final pair, or a hand-built two-stage subject, exercised so the stage ordinal is observed distinguishing what member sets could not.

## Closes when

The atom is `(member, stage)` at every attribution site, single-stage behaviour is byte-identical (existing pins unmoved or recomputed with ledgers per the step), the distinguishing test exists and was watched failing under the old key shape, and the parent keystone's wall 2 derivation is updated to point here as discharged.

## Outcome 2026-08-06 — the atom is a pair, and no pinned identity moved

### The atom

`SemanticStage { member: SemanticMemberId, stage: StageOrdinal }` in `crates/tiler-compiler/src/region.rs`, `Ord` member-major so a set of first-stage atoms sorts exactly as its member ordinals did and an occurrence's stages stay adjacent. `SemanticMemberId` survives as the occurrence ordinal, because the authorities whose subject *is* the occurrence still need it: graph lookups, canonical positions, duplication legality, lowering resolution, and `tiler_ir::program::CoveredOccurrence`.

Sites converted, each because attribution is decided there by comparing whole sets:

- **region graph** — `RegionCandidate::members`; `assemble` mints one `SemanticStage::first` per covered operation.
- **cover** — `CoverRegion::members`; `derive_duplication` and `derive_cover_cost`'s `recomputed_elements` key their `seen` sets on the atom, so a repeated *atom* is a duplication and a repeated *member* at distinct stages is a split that costs nothing. The per-occurrence vectors (coverage mask, candidate index, duplication legality) stay occurrence-indexed through `SemanticStage::member`, because "every operation is computed" and "recomputing this operation is legal" are facts about the operation.
- **recognizer** — `RecognizedSerialSumMembers`, `NormalizedPointwise/Contraction/Epilogue::members`, `owns_region_members`, `output_for_region`, `check_output_cover`. `NormalizedProgram::all_occurrences` is the new projection lowering asks, since an occurrence resolves one capability and mints one receipt however many regions realize it.
- **physical** — every `Vec<SemanticMemberId>` builder return, `VerifiedScheduledRegion::semantic_members`, `spell_region`/`spell_output`, and the subject bindings. **`final_reduction_region` now returns the reduction occurrence's second stage instead of `Vec::new()`**, and `verify_multi_pass_subject_binding`'s Final arm requires exactly that atom instead of `semantic_members.is_empty()`.
- **frontier** — `SubprogramStage`, `FrontierRegionSubject`, `AdmittedImplementation`.
- **program assembly** — `AssemblyStage::coverage` is now read from each *pass's* verified region rather than from the cover region for the first pass and `Vec::new()` for the rest; `covered()` projects it onto `CoveredOccurrence` by keeping first-stage atoms only.
- **selection / component cost / pipeline** — `member_key` is the `(member, stage)` pair, the repeated-work set keys on the atom, and `region_role` reads atoms.

One rule, written once: `region::chain_realizes_subject` decides whether an ordered chain of dispatch claims realizes one subject. Both `frontier::admit_subprogram` and `pipeline::verify::verify_whole_program_schedule_coverage` call it; the sorted-concatenation comparison they each carried cannot express a chain whose later passes name their own stage.

### The identity derivation — nothing moved, and why that is a derivation rather than luck

**Fact.** `cargo nextest run --workspace` is **2841 passed, 7 skipped** with no pin edited. The explain request qualifier is still `ce6f9106c1c5933b`.

The stage ordinal reaches no encoding, because the only non-first atom in the tree is the split's combine claim, and no identity encoder is downstream of it:

- **Region content and occurrence identity** encode occurrences, and every candidate is single-stage — `assemble` is the sole constructor of a `RegionCandidate` and mints `SemanticStage::first`. Stated at `encode_occurrence`; enforced by `verify_candidate`, which refuses a non-first stage as `unencoded-member-stage` rather than rebuilding it against another candidate's bytes.
- **Cover identity** folds region occurrence identities, duplication canonical positions, and materialization edges. Every existing cover's atoms are first-stage, so `derive_duplication` returns what it returned.
- **Request-subject identity** (`encode_output_subject`) writes each member run's occurrence ordinal alone; `plan_elementwise`, `RecognizedSerialSumMembers::new`, and the contraction recognizer each mint a first stage, so the ordinals are injective over what any recognizer can produce. The premise is recorded at the encoder.
- **Proposal and subprogram identity** encode canonical scheduled-region identities, not members.
- **Kernel-program and artifact identity** are reached through `CoveredOccurrence`, and the first-stage projection means the combine contributes no record — byte-identical to the empty claim it replaced.

So the honest answer is neither a version step nor an appends-only extension: it is *no encoding change at all*, with the injectivity premise enforced at one site and the whole obligation filed as [`fold-the-attribution-stage-into-region-and-request-subject-identity`](fold-the-attribution-stage-into-region-and-request-subject-identity.md), deferred behind the ticket that mints the first multi-stage candidate. Guessing the encoding now would decide — with no content to validate against — whether two stages of one occurrence are one content atom or two.

### The distinguishing test, watched failing

`physical::tests::a_splits_two_passes_are_distinguished_by_stage_rather_than_member_set`. It takes the governed split of a four-contributor relaxed request, asserts the two passes claim one member and differ only in the ordinal, binds each pass under its own claim, and requires every other claim the member set alone could express to be refused — including the empty one.

Watched failing with `final_reduction_region` and the Final binding arm reverted to the old key shape, twice:

- the shape assertion, `assertion left == right failed / left: 0 / right: 1` — the combine claimed nothing, so no second stage existed to name;
- and with the shape assertions bypassed, the refusal assertion, printing `OLD-SHAPE PROBE combine_claim=[] empty_claim_binds=true` before failing on `the empty claim must be refused`. Under the old key shape the empty claim *bound*, which is exactly the ambiguity the pair removes.

Two supporting checks whose arms nothing else drives: `region::tests::a_chain_realizes_its_subject_only_when_every_stage_is_accounted_for` (four accepting chains, six refusing ones — unclaimed occurrence, foreign occurrence, first stage twice, later stage twice, a continuation of a stage nothing computed, a skipped stage) and the `unencoded-member-stage` arm added to `region_candidates_are_verified_against_their_own_recomputation`.

### What did not change, and the one thing that must change with the producer

The cover search's counting stays occurrence-keyed: its obligation today is that every *operation* is covered, and duplication legality is a property of the operation and the contract. That is exact while every candidate is single-stage, and it is not a model of a split — two candidates covering two stages of one occurrence would raise its count to two, and `verify_cover` reads a count above one as duplication. The search would therefore *refuse* a legal multi-region realization rather than admit it wrongly, which is the fail-closed direction; the mask has to become per-stage in the same wave that mints the first multi-stage candidate, and the note is at `cover::member_index`.

The IR is untouched — `tiler_ir::program`'s coverage is keyed on `SemanticOccurrence` and refuses an occurrence twice, and the first-stage projection is what keeps that whole-program proof intact without the stage ordinal reaching a shared-IR type. No `crates/tiler-ir/**` file is in the diff.
