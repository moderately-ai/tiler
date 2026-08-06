---
id: enumerate-region-candidates-over-realization-stages
title: Enumerate region candidates over realization stages
status: done
priority: p1
dependencies: []
related: [resolve-which-authority-mints-a-multi-stage-region-candidate, fold-the-attribution-stage-into-region-and-request-subject-identity, admit-the-registered-elementary-families-as-recognizable-program-stages, implement-stage-level-cover-atoms-for-multi-region-occurrences]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, identity-domain]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1786044344
---
## User-visible outcome

A program containing a staged family enumerates one region candidate per realization stage, so the cover search sees the family's internal boundary and a registered elementary middle stage becomes coverable — the outcome [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) is blocked on. This executes Tom's Option A′ decision on [`resolve-which-authority-mints-a-multi-stage-region-candidate`](resolve-which-authority-mints-a-multi-stage-region-candidate.md), whose derivation is the specification and is not re-litigated here.

## The surface, enumerated from the decision node

- **`region::assemble` (`crates/tiler-compiler/src/region.rs`)** takes the registered `IndexRealizationLaw` authority (and therefore the scalar authority) as an input and mints one candidate per stage for an occurrence whose law realizes a region sequence, with intra-occurrence producer/consumer edges. Every other occurrence's enumeration stays first-stage.
- **Synthetic values.** A staged occurrence's published intermediate is no program value; the region graph, `RetainedOutput`, `MaterializationEdge::value` (`cover.rs`), program assembly's internal values, and the assembled program's ABI must carry it explicitly. Fail closed anywhere the value cannot yet be represented.
- **The cover obligations.** `verify_cover`'s per-operation counting becomes per-stage counting (every stage covered exactly once — the mask obligation `cover::member_index`'s doc names); `Partitioner::refused_duplication` and the completeness test move with it, each watched failing.
- **Identity, whole in this change.** [`fold-the-attribution-stage-into-region-and-request-subject-identity`](fold-the-attribution-stage-into-region-and-request-subject-identity.md) fires with this ticket and must land in the same change: region content, region occurrence, and request-subject identity encode the stage atom injectively with the reasoning at the encoding sites, the `unencoded-member-stage` premise guard is replaced by the encoding rather than widened around, and every single-stage program's bytes are proved unchanged or every moved pin is recomputed on the landing tree and enumerated in the commit.

## Non-goals

The softmax's law (still wants the maximum key and the multi-reader sequence vocabulary); any recognizer widening beyond what stage-enumerated candidates make reachable; reference bit-agreement for a compiled middle-stage program if a wall outside these scopes appears — name it with an owner instead.

## Closes when

A program with a normalization middle stage enumerates staged candidates, the cover search covers every stage exactly once, the identity encoding lands whole with its blast radius enumerated, and the recognizer parent's blocked outcome is re-evaluated against what remains.

## Outcome — 2026-08-06, executed inline by the coordinator

**A program with a staged occurrence enumerates one region candidate per realization stage, the cover search covers every stage exactly once, and the identity encoding landed whole with a blast radius of zero moved pins.** Commits `07350438` (node space), `579ad6e6` (pipeline wiring + staged battery), `d3bd904d` (cover conversion), plus this close.

**The architecture is a node-space layer.** Formation enumerates dense node ids — one per stage atom, member-major — so a program with no staged member has node ids equal to member ordinals and every existing enumeration is byte-identical; the pre-existing 704-test suite is the equivalence proof and `an_unstaged_program_forms_identically_with_and_without_the_law_authority` pins it candidate-for-candidate. The stage topology is read off each resolved law's own realized sequence through two additive `ResolvedIndexRealization` accessors (`realizes_region_sequence`, `realize_sequence` — the ir half of this ticket's scopes; the realization refinement already performs internally, exposed with the refusal mapped to refinement's own vocabulary). One authority: nothing in the compiler re-derives law semantics.

**Identity: appends-only, derived rather than assumed.** Both encoders carry a stage trailer exactly when a candidate touches a staged member — keyed on the member's stage count, not on the atoms' spelling, because a first-stage-only candidate of a staged occurrence computes something different from a single-stage candidate over the same operations and must not share bytes (`the_stage_trailer_separates_what_shared_bytes_would_conflate` pins all four separations). No previously encodable candidate's bytes move, no domain version steps, and the predicted blast radius (cover identity, request qualifier, region-label goldens) came to **zero moved pins** — the full workspace (2861 tests) is green with every pin untouched. The request subject reaches stage structure through the law-registry identity it already folds, so `encode_output_subject` needed no change; no recognizer mints a staged partition (`rg 'SemanticStage::first' crates/tiler-compiler/src/request.rs` — all three recognizers unchanged).

**The cover obligations landed with the same producer.** Coverage mask, candidate index, singleton collection, and the anchored search are per atom; duplication legality stays per occurrence. The staged cover test caught formation's singleton loop still ranging over operations — the exact class of gap the test-with-the-change discipline exists for — and asserts the mask both ways: a missing later stage refuses `UncoveredMember`, a doubled stage refuses `IllegalDuplication`. A cover fusing the pass beside its downstream consumer without the fold enumerates and verifies — the flash-shaped split the A′ decision exists to admit.

**Walls named, with owners.** No whole program containing a staged family reaches formation through `compile()`: the request recognizer still refuses under `operation-set` upstream, which is [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md)'s recognizer half, and physical lowering of a *selected* split-stage cover (materializing the handed value through program assembly and the ABI) is unreachable behind it — both fail closed today and are that ticket's re-evaluated remainder.

**Checks.** `cargo fmt --all --check`; `cargo check`/`clippy --all-targets -- -D warnings` for tiler-ir and tiler-compiler; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` both; `cargo nextest run --workspace` 2861 passed; `cargo test --workspace --doc` green incl. the ADR 0051 compile-fail evidence; `tkt lint`; `git diff --check`; guard at integration. Seven new tests: five formation (topology/roundtrip/split-boundaries/trailer/rebuild), one ir (accessor shapes incl. the mapped refusal), one cover (the mask both ways plus the fusion shape).
