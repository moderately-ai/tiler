---
id: repair-the-stale-three-carried-subject-claims
title: Repair the stale three-carried-subject claims
status: in-progress
priority: p2
dependencies: []
related: [package-the-admitted-live-schedule-into-a-symbolic-kernel-program, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/ir, implementation/artifact, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifact, identity]
claimed_from: todo
assignee: worker-three-subjects
lease_expires_at: 1787160394
---
## User-visible outcome

Every claim that the artifact envelope carries only three semantic subjects, or that the shape-environment subject does not travel, is corrected to the tree's actual shape: since the v17 retained-environment landing the envelope carries the three reached semantic subjects plus the lossless retained shape environment, and all four are folded into artifact identity.

## Why this exists

The `package-the-admitted-live-schedule-into-a-symbolic-kernel-program` Fact audit at `741bc9633a0372338a588eb8470bada79c185a31` found five sites whose "three carried subjects" reasoning predates the retained-environment landing (`69e17234`, recorded as artifact v17 / manifest 17.0 in `docs/artifact-abi.md`, anchor `carrying the retained shape environment moved the artifact to`). Each was verified stale by grep and full read at that base:

1. `crates/tiler-ir/src/program/error.rs`, anchor `unrepresented in the artifact's three carried subjects` — the `SymbolicInterfaceExtent` doc's supporting claim about the artifact's shape; the refusal itself is correct and stays.
2. `crates/tiler-ir/src/semantic/program.rs`, anchor `project_semantic` — the `no_symbolic_program_reaches_a_verified_kernel_program` test comment names the artifact codec's `project_semantic` (`crates/tiler-artifact/src/program/codec/model.rs`, anchor `fn project_semantic`) and claims the shape-environment subject does not travel, which is false since v17: that function now travels the subject (`retained_shape: data.retained.clone()`). *(Corrected 2026-08-18 per the packet's independent review at `1bd8aa7c`; this site originally claimed the function "no longer exists in the tree", a failed-search-as-absence error — the correct repair is to point the comment at the travelling function, not to treat it as dangling.)* The test's assertion is correct and stays.
3. `crates/tiler-artifact/src/program/builder.rs`, anchor `the envelope's three carried subjects sufficient` — `read_semantic_interface`'s error doc.
4. `docs/ir.md`, anchor `three carried subjects stay sufficient` — the 2026-08-07 correction paragraph's derived-consequence sentence: its conclusion (no symbolic program reaches a packaged artifact) is still current, but its stated mechanism ("no two artifacts can differ by the shape-environment subject they omit") is not — artifacts carry the subject and can differ by it, by pinned test (`crates/tiler-artifact/src/program/retained.rs`, anchor `an unused retained environment is identity-bearing`).
5. `docs/research/shapes/symbolic-semantic-extents.md`, anchor `keeps the artifact at three subjects` — the totality-coupling sentence; the coupling test it cites (`a_symbolic_semantic_program_never_reaches_the_artifact_builder`) still exists and passes, but the "three subjects" consequence it protects is superseded.

## Required work

- Correct each site with a dated correction in the file's existing convention, preserving what remains true (the refusals, the test assertions, the still-current conclusions) and replacing only the stale mechanism claims. Do not delete history that documents why the earlier reasoning held at its time.
- Re-grep the whole tree for the phrase families `three carried subjects` and `three subjects` scoped to artifact-envelope claims before closing, so the census cannot silently be five-of-more. Known scoped-out hits, recorded 2026-08-18 by the packet's independent review so the closer does not rediscover them as a surprise: four dated closed-ticket records (`tickets/admit-symbolic-extents-through-schedule-formation.md`, and three in `tickets/carry-a-sourced-shape-on-semantic-values.md`) are history by repository convention and are not repaired.
- If `package-the-admitted-live-schedule-into-a-symbolic-kernel-program`'s decision lands first and its implementation rewrites any of these sites (the `SymbolicInterfaceExtent` doc in particular), re-audit the census at that base and repair only the remainder.

## Closes when

Each surviving stale claim is corrected at a stated base with anchors, the census grep is quoted, and no artifact-shape claim in the five files contradicts the v17 grammar record.
