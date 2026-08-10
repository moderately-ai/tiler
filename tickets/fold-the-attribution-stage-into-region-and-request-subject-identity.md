---
id: fold-the-attribution-stage-into-region-and-request-subject-identity
title: Fold the attribution stage into region and request-subject identity
status: done
priority: p1
dependencies: []
related: [enumerate-region-candidates-over-realization-stages, implement-stage-level-cover-atoms-for-multi-region-occurrences, widen-the-staged-realization-law-to-the-registered-elementary-families, admit-the-registered-elementary-families-as-recognizable-program-stages, resolve-which-authority-mints-a-multi-stage-region-candidate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity-domain]
---
## Outcome

Discharged whole inside [`enumerate-region-candidates-over-realization-stages`](enumerate-region-candidates-over-realization-stages.md). On the tree:

- Stage folds into region content identity as a conditional trailer (`covers_staged_member` → `append_stage_trailer` inside `encode_content`). Presence is a function of the member population's stage counts, so pre-stage encodings stay byte-identical; domains remain `tiler.compiler.region-content.v1` / `tiler.compiler.region-occurrence.v1` (appends-only, not a version step). The derivation for appends-only over a version bump is recorded at those sites.
- `encode_occurrence` does **not** append an occurrence-side trailer; the stage distinction reaches occurrence bytes through the length-prefixed embedded content encoding (comment at `encode_occurrence` and the assemble path).
- `encode_output_subject` is unchanged for stage: no recognizer mints a staged partition into identity runs; member runs still write occurrence ordinals only. Request subject reaches stage structure through law-registry identity already folded.
- `verify_candidate` rebuilds from the candidate's exact atom set via `form_candidate`; the retired `unencoded-member-stage` wall is gone (name survives only as historical comment).
- Zero region identity pins moved. Adjacent cover-mask obligation (every stage covered once; `member_index` still occurrence-sized for duplication legality) landed with the same producer.

Closes-when met. No residual identity work on this ticket; do not re-open for request-subject stage folding unless a recognizer begins minting staged partitions into subject runs.

## What was missing (pre-delivery)

**Historical problem statement — not live tree state.** Before the discharge below, region content identity and region occurrence identity encoded *occurrences*, not attribution atoms. `encode_content` renumbered members to region-local canonical positions and `encode_occurrence` wrote those positions plus the derived boundary and retained-value sites (`crates/tiler-compiler/src/region.rs`). Neither wrote a [`StageOrdinal`].

At that pre-state, `assemble` was the only constructor of a `RegionCandidate` and minted `SemanticStage::first` for every member, so no two candidates of one program could differ in a stage ordinal and the positions separated every distinct site. The premise was enforced rather than assumed — `verify_candidate` refused a candidate carrying a non-first stage by name (`RegionError::Invalid { rule: "unencoded-member-stage" }`), watched failing in `region::tests::region_candidates_are_verified_against_their_own_recomputation`.

The same premise held one layer up, and was recorded at `crates/tiler-compiler/src/request.rs`'s `encode_output_subject`: every recognized partition's member run wrote the occurrence ordinal alone, which is injective because `plan_elementwise`, `RecognizedSerialSumMembers::new`, and the contraction recognizer each mint a first stage and nothing else does. That request-subject premise remains true after discharge (see Outcome).

## What fires this

The first authority that mints a region candidate — or a recognized partition — covering *one stage* of a multi-stage occurrence. That is the producer [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md) supplies and the recognition [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) derives from it. Until one exists there is no multi-stage content to validate an encoding against, which is why the encoding was not guessed here: whether two stages of one occurrence are one content atom or two is a question about content that does not yet exist.

## What this ticket must then do, whole

The identity step executes in one commit or not at all:

- fold the stage ordinal into `encode_content` (and into occurrence identity only by embedding that content encoding — not as a separate occurrence-side trailer), and into `encode_output_subject`'s member runs if a recognizer mints a staged partition;
- decide, and record at the site, whether the step is a version bump of `tiler.compiler.region-content.v1` / `tiler.compiler.region-occurrence.v1` / `request-subject.v2` or an appends-only extension carried by per-tag injectivity reasoning — derived, not assumed appends-only because it is cheaper;
- lift `verify_candidate`'s `unencoded-member-stage` refusal in the same commit, replacing it with a rebuild that reads the atoms;
- recompute every pinned identity on the merged tree from observed failing values and enumerate each moved pin with its ledger. Region occurrence identity is embedded in cover identity, so the blast radius is at least the cover identity, the explain request qualifier, and any golden that renders a region label.

## The adjacent obligation in the same wave

The cover search's coverage mask, candidate index, and duplication counting are sized by the graph's operation count and read a member covered twice as duplication (`cover::member_index`, `cover::verify_cover`). A cover placing two stages of one occurrence therefore *refuses* — fail-closed, but wrong for a legal split. The mask must require every stage covered once rather than every operation covered once, and that lands with the same producer.

## Closes when

A multi-stage candidate has an identity distinct from every single-stage candidate over the same occurrences, `verify_candidate` rebuilds it rather than refusing it, the version-versus-appends-only derivation is recorded at each changed encoder, and every moved pin is enumerated with its ledger comment in the same commit.

## Trigger check log

- 2026-08-06 — not fired. No authority mints a candidate or a recognized partition with a non-first stage; `assemble` and the three recognizers are the only constructors and each mints `SemanticStage::first`. Reproduce: `rg -n 'SemanticStage::first|SemanticStage::new' crates/tiler-compiler/src`.

- 2026-08-06 — **fired.** The recognizer dispatch (`admit-the-registered-elementary-families-as-recognizable-program-stages`) mints the first multi-stage candidate, so the encoding must land whole in the same change; promoted to `todo` and claimed alongside it. Reproduce: `grep -m1 '^status:' tickets/admit-the-registered-elementary-families-as-recognizable-program-stages.md` no longer answers `blocked`.

- 2026-08-06 (second evaluation, same day) — **not fired, and the earlier "fired" is withdrawn.** The recognizer dispatch landed the normalization's *lowering* (`GovernedRootMeanSquareScaleF32`) and did not mint a multi-stage candidate or a staged recognized partition. The earlier entry read the dispatch's claim rather than the tree: it assumed the recognizer half would land in the same change, and that half is blocked on [`resolve-which-authority-mints-a-multi-stage-region-candidate`](resolve-which-authority-mints-a-multi-stage-region-candidate.md) — the question of *which authority* mints the claim is itself a fork, and the two answers disagree about whether this encoding must change at all. Reproduce: `rg -n 'SemanticStage::first|next_stage' crates/tiler-compiler/src` still shows `region::assemble` minting first stages only and `physical::final_reduction_region` as the sole non-first claim, which is a dispatch of one placed region rather than a candidate.

- 2026-08-06 — **the adjacent obligation is unchanged and its site is recorded.** `verify_cover` still counts per operation (`cover.rs:1158`–`:1180`) and `cover::member_index`'s doc still names the widening; nothing in this wave touched either, because nothing can yet produce a cover that would exercise them.

- 2026-08-06 — **evaluation under the decided fork: fires with the implementation, not before.** Tom chose Option A′ on [`resolve-which-authority-mints-a-multi-stage-region-candidate`](resolve-which-authority-mints-a-multi-stage-region-candidate.md), so stage-enumerated candidates are region formation's to mint and this encoding lands whole inside [`enumerate-region-candidates-over-realization-stages`](enumerate-region-candidates-over-realization-stages.md). Stays `deferred` until that ticket starts; the earlier fired/withdrawn pair above records why a dispatch's claim is not a tree's. Reproduce: `grep -m1 '^status:' tickets/enumerate-region-candidates-over-realization-stages.md` answers `todo`.

- 2026-08-06 — **fired and discharged whole, inside [`enumerate-region-candidates-over-realization-stages`](enumerate-region-candidates-over-realization-stages.md).** Every obligation, against the tree: the stage folds into region content identity as a conditional trailer (`append_stage_trailer`, `covers_staged_member` inside `encode_content` — the derivation for appends-only over a version step is at those sites: presence is a function of the member population's stage counts, so no previously encodable candidate's bytes move and injectivity holds within and across programs); occurrence identity inherits the trailer only by embedding those content bytes and has no occurrence-side trailer of its own; `encode_output_subject` is unchanged because no recognizer mints a staged partition and the request subject reaches stage structure through the law-registry identity it already folds; `verify_candidate`'s `unencoded-member-stage` refusal is replaced by the rebuild reading atoms; **zero pins moved** — the predicted at-least set (cover identity, request qualifier, goldens) is untouched, proven by the full workspace suite rather than recomputed, which is what the appends-only choice bought; and the adjacent cover-mask obligation landed with the same producer, watched refusing in both directions. Closes-when met on every clause.

## Fact audit — 2026-08-10

- Removed stale `deferred` board tag; status remains `done`.
- Related graph now lists the carrier [`enumerate-region-candidates-over-realization-stages`](enumerate-region-candidates-over-realization-stages.md) and the Option A′ decision [`resolve-which-authority-mints-a-multi-stage-region-candidate`](resolve-which-authority-mints-a-multi-stage-region-candidate.md).
- Pre-delivery problem section retitled and framed as historical; live encoding and verify path stated in Outcome.
- Discharge trailer wording corrected: content carries the conditional trailer; occurrence inherits via embed only (no occurrence-side trailer). Matches `append_stage_trailer` single call site inside `encode_content`.
