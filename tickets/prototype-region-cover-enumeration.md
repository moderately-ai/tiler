---
id: prototype-region-cover-enumeration
title: Enumerate legal complete region covers
status: done
priority: p0
dependencies: [prototype-fusion-legality-and-numerical-proof]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning]
---
Enumerate bounded legal covers before physical program selection. Cover every
operation and named output, preserve occurrence identity and boundaries,
conservatively materialize fan-out unless duplication is explicitly legal, and
retain fused and singleton/materialized covers. This stage does not choose
implementations or claim a complete executable program.

Cover identity binds semantic graph meaning, exact region occurrences,
coverage, deliberate duplication, and proposed materialization edges. Local
physical frontiers are independently enumerated without depending on a global
cover; complete physical-plan selection follows both authorities.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

**Fact.** Added `crates/tiler-compiler/src/cover.rs` and registered `mod cover;` in `crates/tiler-compiler/src/lib.rs`. The module is a self-contained `pub(crate)` draft authority (`#![allow(dead_code, …)]`, mirroring the sibling `fusion_legality`/`frontier` private drafts); it is not wired into `pipeline::compile`.

**Fact.** `enumerate_covers(program, budgets, contract, cover_budgets) -> Result<CoverEnumeration, CoverError>` re-derives region-formation candidates from the program (`form_region_candidates`) and enumerates the bounded legal complete covers. Producer duplication is disabled in this profile, so a legal cover is an exact operation partition. The fully-materialized (all-singleton) cover is emitted unconditionally and the fused (whole-program) cover whenever a whole-program candidate exists; the remaining exact partitions are enumerated by anchoring each region on the minimum uncovered operation, bounded by `CoverBudgets { covers, expansions }`. Both required covers survive any budget; lost partitions are reported as a typed `CoverBudgetStop`.

**Fact.** Coverage is enforced closed. Every operation is covered exactly once. A bare-input passthrough named output (an output value that is a program input) cannot be covered by any operation region and is reported as `CoverInfeasibility::UnrootedNamedOutput` with an empty cover set — a valid `Ok` result distinct from an error. `verify_cover(program, budgets, contract, cover)` re-derives the authoritative candidates and returns the typed rejection taxonomy: `UncoveredMember`, `IllegalDuplication`, `UncoveredNamedOutput` (class `coverage`), a broken occurrence identity as `Region(RegionError::Invalid)` (class `region`), and mismatched materialization/identity/ordering as `Structure` (class `structure`).

**Fact.** Conservative fan-out materialization is represented by `MaterializationEdge`: a value produced in one region and read by others is materialized once and carries every consuming region (fan-out is one edge with several consumers, never duplication). `CoverDuplication` records deliberate duplication and is empty in this profile.

**Fact.** `RegionCoverIdentity` folds, in a canonical length-prefixed byte encoding over content-derived coordinates: the semantic graph identity (`SemanticGraphIdentity`), the exact region occurrence identities (which bind per-region content and coverage), the deliberate-duplication section, and the materialization edges. It excludes transient graph-local ordinals and enumeration order and uses only `BTreeMap`/`BTreeSet`; covers are returned in canonical identity order.

**Inference.** Enumeration is complete and sound for the bounded profile: 13 module tests pass, including agreement — set-for-set, without budget pressure — with an independent exhaustive powerset oracle over the candidate set on four fixtures (serial-sum, shared-constant, diamond, shared-producer). The oracle re-derives exact partitions by brute force rather than reusing the anchored search, so agreement is evidence.

**Measurement.** `uv run --locked python scripts/check_repository.py` passed; `git diff --check` clean; `ticketsplease guard tkt/prototype-region-cover-enumeration` clean (affected scope `implementation/compiler`, shared `project/tickets`). Toolchain: nightly-2026-07-19, macOS arm64.

**Proposal / deferred.** `CoverBudgets` is local to `cover.rs`; when cover enumeration is wired into the pipeline it should be promoted into `DeterministicBudgets` (kept out now to avoid test-only-read fields in the live `request` module). Deferred to their own authorities/tickets: typed explain-trace records for covers, wiring into `compile()`, and the complete-physical-plan-selection join of a cover with compatible per-region frontiers. Status left `in-progress`; the `pub(crate)` cover surface is a draft pending Tom's review of the exact commit.
