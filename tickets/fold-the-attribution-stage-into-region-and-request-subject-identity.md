---
id: fold-the-attribution-stage-into-region-and-request-subject-identity
title: Fold the attribution stage into region and request-subject identity
status: in-progress
priority: p1
dependencies: []
related: [implement-stage-level-cover-atoms-for-multi-region-occurrences, widen-the-staged-realization-law-to-the-registered-elementary-families, admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity-domain, deferred]
claimed_from: todo
assignee: agent-recognizer
lease_expires_at: 1786041779
---
## What is missing

Region content identity and region occurrence identity encode *occurrences*, not attribution atoms. `encode_content` renumbers members to region-local canonical positions and `encode_occurrence` writes those positions plus the derived boundary and retained-value sites (`crates/tiler-compiler/src/region.rs`). Neither writes a [`StageOrdinal`].

That is complete today and stated as such at both encoders: `assemble` is the only constructor of a `RegionCandidate` and mints `SemanticStage::first` for every member, so no two candidates of one program can differ in a stage ordinal and the positions separate every distinct site. The premise is enforced rather than assumed — `verify_candidate` refuses a candidate carrying a non-first stage by name (`RegionError::Invalid { rule: "unencoded-member-stage" }`), watched failing in `region::tests::region_candidates_are_verified_against_their_own_recomputation`.

The same premise holds one layer up, and is recorded at `crates/tiler-compiler/src/request.rs`'s `encode_output_subject`: every recognized partition's member run writes the occurrence ordinal alone, which is injective because `plan_elementwise`, `RecognizedSerialSumMembers::new`, and the contraction recognizer each mint a first stage and nothing else does.

## What fires this

The first authority that mints a region candidate — or a recognized partition — covering *one stage* of a multi-stage occurrence. That is the producer [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md) supplies and the recognition [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) derives from it. Until one exists there is no multi-stage content to validate an encoding against, which is why the encoding was not guessed here: whether two stages of one occurrence are one content atom or two is a question about content that does not yet exist.

## What this ticket must then do, whole

The identity step executes in one commit or not at all:

- fold the stage ordinal into `encode_content` and `encode_occurrence`, and into `encode_output_subject`'s member runs if a recognizer mints a staged partition;
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
