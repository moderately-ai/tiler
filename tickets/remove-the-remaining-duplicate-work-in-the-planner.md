---
id: remove-the-remaining-duplicate-work-in-the-planner
title: Remove the remaining duplicate work in the planner
status: todo
priority: p2
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [performance, compiler]
---
The Phase 1 remainder, after region formation and the request subject are dealt with. Each is a pure function recomputed; none changes semantics.

## Facts

**`enumerate_frontier` runs per (cover, region) for ~3 distinct inputs.** `pipeline.rs:1035`, `C x R` calls. The code proves the redundancy itself: `pipeline.rs:1048` dedups the explain record by `region_role`, and `region_role` (`pipeline.rs:708`) returns one of four `&'static str` values. Memoize on the key already computed one line later.

**Every alternative is built, then rebuilt identically to compare.** `build_alternative` (`pipeline.rs:1228`) produces the scheduled regions, kernels, program, and artifact plan; `verify_alternative` (`pipeline.rs:2469`) recomputes all of it and compares. Exactly 2x per alternative with no shared intermediate.

**`verify_request` runs twice per compilation** (`pipeline.rs:365`, `:393`), redoing whole-program recognition and per-target contract resolution. `pipeline.rs:362-363` clones `target_profiles` and `capabilities` unconditionally before knowing whether normalisation committed a rewrite — dead in the common case, where `normalized == None` at `:381`.

**The Pareto scan is O(P^2) and runs twice** — `selection.rs:492` via `pipeline.rs:829` and again at `pipeline.rs:2448`. Each allocates a `String` per plan via `label()` for comparisons that could use raw identity bytes. Same pattern at `pipeline.rs:1090`/`:1103-1106`, which is O(refused x P) FNV digests and `String` allocations.

**`encode_record(&record).len()`** (`explain.rs:1078`) builds the full canonical encoding of every explain record solely to measure its length, then drops it. `explain.rs:934` does the same for the trace header.

**`RecognizedSerialSumMembers::all()`** (`request.rs:632`) allocates, chains, sorts and dedups on every call, from `pipeline.rs:718`, `frontier.rs:1352`, `lowering.rs:285` and `:320`.

**`assemble_cover` runs before the dedup check** (`cover.rs:787`), so every duplicate partition pays full assembly. **`derive_materializations`** (`cover.rs:905`) allocates two `Vec<u8>` inside the sort comparator. **`singleton_occurrence_identity`** (`lowering.rs:360`) linear-scans all candidates per member.

## Closes when

Each item above is removed or justified in place; compile time is measured before and after; work-count guards cover the memoisations; artifact identity is byte-identical; `make full` passes.
