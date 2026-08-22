---
id: keep-a-module-size-and-complexity-census-with-a-split-queue
title: Keep a module size and complexity census with a split queue
status: done
priority: p3
dependencies: []
related: [split-the-schedule-builder-into-cohesive-submodules, split-the-compiler-target-module-into-cohesive-submodules, split-the-artifact-program-test-monoliths-into-focused-modules, split-the-compiler-request-module-before-the-contributor-source-carrier, split-the-index-refinement-module-into-cohesive-submodules, split-the-compiler-pipeline-test-monolith-by-orchestration-phase, split-the-kernel-test-monolith-into-focused-modules, split-the-schedule-builder-test-monolith-into-focused-modules, split-the-compiler-request-test-monolith-into-focused-modules, split-the-ir-program-test-monolith-into-focused-modules, lift-the-inline-test-modules-that-dominate-their-files]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [maintainability, census]
---
## User-visible outcome

Module size stops regressing silently: a recorded census names the workspace's oversized files, a prioritized split queue orders the repairs, and the closing state records either a mechanical floor check or the reasoned decision not to add one.

## Census — 2026-08-19, base `e53eb65f` working tree (historical; superseded by the 2026-08-22 block below)

Command: `find crates -name '*.rs' | xargs wc -l | sort -rn`. Population: **60 files over 2,000 lines** across 469,689 workspace lines. Top of the census (lines, file):

- 13,746 `tiler-compiler/src/request.rs` — split ticket filed (sequenced behind the contraction merge, before the contributor-source carrier)
- 10,554 `tiler-ir/src/schedule/builder.rs` — split ticket filed, dispatched
- 9,438 `tiler-compiler/src/pipeline/tests.rs` — queue below
- 8,633 `tiler-compiler/src/target.rs` — split ticket filed, dispatched
- 7,936 `tiler-artifact/src/program/tests.rs` — split ticket filed, dispatched (with codec/tests.rs 5,795)
- 7,434 `tiler-ir/src/kernel/tests.rs`; 7,363 `tiler-compiler/src/frontier.rs`; 7,101 `tiler-ir/src/index/refinement.rs` (ticket filed); 5,848 `tiler-compiler/src/physical.rs`; 5,781 `tiler-compiler/src/explain.rs`; 5,661 `tiler-compiler/src/target/feasibility.rs`; 5,392 `tiler-ir/src/semantic/registry.rs`; 5,315 `tiler-ir/src/index/law.rs`; 5,266 `tiler-ir/src/program/tests.rs`; 5,192 `tiler-compiler/src/session.rs`; 5,109 `tiler-compiler/src/region.rs`; 4,807 `tiler-compiler/src/governed.rs`.

Per-crate concentration: tiler-ir 150k lines / 155 files; tiler-compiler 132k / 65 files (the highest lines-per-file ratio — the split queue's centre of gravity).

**Re-verification — 2026-08-22.** Every number in this block was reconstructed from the tree at `e53eb65f` rather than trusted, and all of them hold: 469,689 lines across 451 files, 60 over 2,000, and each of the eighteen named file sizes byte-for-byte. Per-crate likewise — tiler-ir 150,165 / 155, tiler-compiler 131,846 / 65, whose 2,028 lines-per-file is the highest (tiler-artifact is next at 1,244). Reproduce with `git ls-tree -r e53eb65f --name-only -- crates | grep '\.rs$'` piped through `git show e53eb65f:"$p" | wc -l`. The block is retained unchanged as dated history; it describes a tree five days and many landings behind, and nothing in it should be read as current.

## Required work

- After the five filed splits land, re-run the census, extend the queue over the remaining hotspots above (test monoliths first — they carry the read-in-full burden with the least seam risk; then `frontier.rs`, `physical.rs`, `explain.rs`, `session.rs`, `region.rs`, `feasibility.rs`, `registry.rs`, `law.rs`), and file the next tranche as bounded tickets with the same pure-code-motion discipline.
- Decide, with a recorded rationale, whether a mechanical check is worth its maintenance: a size floor can only make regressions loud (e.g. a census script asserting no file exceeds a stated ceiling without a recorded exemption), and per repository rules a check that cannot fail is worse than none. If added, perturb it and quote the failure text; if declined, record why.
- Complexity between modules is part of the census, not just line counts: where a split reveals a dependency tangle (two modules that cannot separate without a shared-internals module larger than either), record it here as a restructuring candidate rather than forcing a bad seam.

**Status — 2026-08-22, base `ba46f2b2`.** All three items are discharged in the sections below. Item 1: the census is re-derived and the queue extended over every hotspot it names, with six tickets filed as tranche 2 and nine seams named for tranche 3. The queue's order differs from the one this item gives, on evidence — `region.rs` and `semantic/registry.rs` are the fourth and third largest source files and took one commit each in eight days, while `physical.rs` took ten and grew 990 lines, so ranking by size would have put the two nobody is touching ahead of the one everybody is. Item 2: declined, with the measurement and a reconsideration trigger. Item 3: no tangle found at this base; the negative result and its limits are recorded with the queue.

**Scope note.** `implementation/workspace` was added to this ticket at dispatch in case the floor check needed a `Makefile` line. The check was declined, so this lane never touched a file in that scope and the delta is tickets-only. The scope can be released.

## First tranche landed — 2026-08-19, batch at `3477a693`

All four splits merged green in one batch (builder `56d95195`, target `e723da0f`+`15ce3924`, request `012b1ea4`+`18677074`+`3477a693`, artifact tests `da32ddf8`) with the citation re-anchoring `573cfbe2` and the retired-key refusal pin `77459e19`. Workspace's largest source file is now `crates/tiler-compiler/src/pipeline/tests.rs` (9,438).

**Correction — 2026-08-22, base `ba46f2b2`.** All ten hashes above are ancestors of this base and each names the commit this paragraph claims. Two things about it are no longer current. The fifth split — [`split-the-index-refinement-module-into-cohesive-submodules`](split-the-index-refinement-module-into-cohesive-submodules.md), which the census block above records only as "ticket filed" — has since landed too, so all five of this ticket's `related` splits are `done`; the heading's "four" counts the first batch, not the tranche. And `pipeline/tests.rs` is still the workspace's largest file but is no longer 9,438 lines: it is **9,632**, having gained 194 lines across eleven commits in the eight days to this base. The figure 9,438 was correct at `3477a693`, which is what makes it a fair illustration of why a recorded census is a dated snapshot and not a live measurement.

**Lessons the batch pinned for every future split ticket's gate list:** (1) `make citations` on the post-split tree — every file move rots line-only citations, and only that gate sees it (14 rotted across the request/target splits; several had rotted long before, resolving by accident); (2) `cargo nextest run -p tiler` — the workspace-invariant scanners (`workspace_unsafe_sites`, which admits only `super::*` globs, and `cited_names_resolve`) live outside every split's package; (3) `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --document-private-items` — the no-private form is blind to intra-doc links in private modules (12 broken links invisible to the weaker gate; 27 more surfaced when the request spine went explicit).

**Correction — 2026-08-22, base `ba46f2b2`. Lesson (2) is false as written, and following it would skip one of the two scanners it names.** The claim that both scanners "live outside every split's package" holds for `workspace_unsafe_sites`, which is a test target of package `tiler` (`crates/tiler/tests/workspace_unsafe_sites.rs`). It does not hold for `cited_names_resolve`, which is a test target of package **`tiler-compiler`** — `crates/tiler-compiler/tests/cited_names_resolve.rs`, the only path it has ever had (`git log --follow` shows four commits, none a rename). `cargo nextest run -p tiler` therefore does not run it at all. The confusion is understandable and the correct instruction is narrower rather than wider: the scanner reads comments under `crates/tiler-compiler/` only, but resolves the names it finds against the *whole* workspace, so a `tiler-ir` or `tiler-artifact` rename can break a compiler citation while every package gate the renaming lane runs stays green. The gate list for a future split is `cargo nextest run -p tiler -p tiler-compiler` when the split is anywhere else, and `--workspace` when in doubt. Anchor for the resolution universe: `crates/tiler-compiler/tests/cited_names_resolve.rs "run appearing on a comment line under"`.

**Defects the batch surfaced and fixed:** the `cited_names_resolve` walk skipped any directory named `target`, hiding 9,513 lines of `src/target/` source (fixed with both-direction perturbation evidence); the request spine's globs silently carried two `#[cfg(test)]` items, 26 dead imports, and 27 accidental doc-link resolutions (now all explicit).

**Correction — 2026-08-22, base `ba46f2b2`. Three of the four figures in the paragraph above are wrong, and one is unsourced.**

- "hiding 9,513 lines of `src/target/` source" **understates the blind spot by a third and mislabels what 9,513 counts.** The fixing commit `15ce3924` states it: `collect_rust_files` put "13 source files and 14,613 lines outside both halves of the check at once", and adds that "Nine thousand of those lines predate the target-module split". Summed at `15ce3924^` the thirteen files are 14,600 by `wc -l`; the four that predate the split are 9,509. The commit's figures run one higher per file because it counts with `lines()`, not newlines — the same one-per-file offset this census's own commands carry. So 9,513 is the *pre-split subset*, not the hidden total, and the total is 14,613.
- "two `#[cfg(test)]` items" is **false: the block enumerates seventeen**, across six children. Read the `#[cfg(test)]` half of the spine added by `18677074`: `contract` 3, `recognize` 3, `subject` 7, `verified` 1, `verify` 3.
- "26 dead imports" has **no source anywhere in the repository.** It appears in this ticket and nowhere else: not in `18677074` (78 insertions, 22 deletions, one file — the deletions are the thirteen glob lines and their comment), not in `3477a693`, not in the request split's own ticket, and no commit message between `46bf1319` and this base contains it. No arrangement of the spine's import counts produces 26 either. Treat it as unverified and do not restate it.
- "27 accidental doc-link resolutions" is **verified**: `3477a693` records "27 intra-doc links in eight request/ children" that had been resolving through the pruned spine imports.

**Also verified in that paragraph, and in the one below it:** "14 rotted across the request/target splits" is exactly `7ec27a24`, "Repair the fourteen citations the request and target splits rotted" — four fresh anchors naming the deleted `builder.rs` path plus ten pre-existing line-only pins into the monolithic `request.rs` across six documents. It is a *different* population from the fifteen builder citations `573cfbe2` re-anchored, and the two must not be merged. "13 artifact test-fixture exports with no external consumer" and "two malformed section banners" both reproduce against [`split-the-artifact-program-test-monoliths-into-focused-modules`](split-the-artifact-program-test-monoliths-into-focused-modules.md), defects 2 and 4. The stale "two crate-private children" comment is still present and still stale: `crates/tiler-compiler/src/target.rs "two crate-private children of this cluster"` sits above three `pub(crate) mod` declarations — `accuracy`, `feasibility`, `honourability` — while the sentence after it describes only the latter two.

**Recorded follow-ups for the second tranche:** move `target.rs`'s 3,484-line inline test module to `target/tests.rs` (declined in-tranche under the zero-test-edit constraint, correctly); the stale "two crate-private children" comment in `target.rs`; 13 artifact test-fixture exports with no external consumer (recorded by the artifact lane); two malformed section banners preserved verbatim in the artifact tests; snapshot de-pin convention divergence (backtick-boundary move vs historical restatement — pick one when next touched); [`point-the-bare-builder-path-mentions-at-the-split-modules`](point-the-bare-builder-path-mentions-at-the-split-modules.md) (filed, and `done` as of this base).

**Correction — 2026-08-22, base `ba46f2b2`.** The inline test module in `target.rs` is **3,498** lines, not 3,484: the file is 3,817 lines and its `#[cfg(test)]` opens at 320. 3,484 was correct at `3477a693` (3,800 lines, module at 317) and drifted by fourteen in three days — small, but it is the figure a worker would quote back into a split ticket. The follow-up itself is unchanged and is now tranche 2 work below, widened from `target.rs` alone to the four files whose inline test text is 80% or more of them.

## Census — 2026-08-22, base `ba46f2b2`

Every figure below was derived at this base. Reproduce the size census with:

```sh
find crates -name '*.rs' | xargs wc -l | sort -rn
find crates -name '*.rs' | xargs wc -l | grep -c 'total$'   # must print 1: xargs batching would add totals
```

Two measurement notes, both of which have already cost this ticket a wrong number. `wc -l` counts newlines, so a file with no trailing newline reads one low; a figure produced by Rust's `lines()` runs one higher per file, which is the whole of the 14,600-against-14,613 discrepancy corrected above. And `grep -c` counts matching *lines*, not occurrences — every count here that could be affected was taken with `grep -o … | wc -l` or from a parser.

**Fact — 484,886 lines across 551 `.rs` files under `crates/`, of which 61 exceed 2,000 lines.** Against the 2026-08-19 block (469,689 / 451 / 60): **+15,197 lines, +100 files, and the over-2,000 population went *up* by one — in the same window that landed five splits.**

**Fact — a third of the census is test text, so raw size ranks the wrong thing.** 158,598 lines sit in 227 pure test files (a `tests.rs`, or any file under a `tests/` directory); a further 63,331 sit in 94 inline `#[cfg(test)] mod … { … }` blocks; 262,957 lines are production code. Four members of the over-2,000 population are 80% or more inline test text, so `wc -l` reports them as large source files when their source halves are small — `tiler-compiler/src/target.rs` 3,817 with 3,498 inline (92%), `tiler-ir/src/index/sourced.rs` 2,558 with 2,262 (88%), `tiler-build/src/metal_plan.rs` 2,300 with 1,862 (81%), `tiler-ir/src/semantic/accuracy/domain.rs` 1,034 with 933 (90%). Fifty-three files already declare `mod tests;` against a sibling file, so both conventions are live at once and the census instrument reads them differently.

**The two rankings the queue is built from.** The first is by production code — total lines minus the trailing inline `#[cfg(test)]` module — and it is the read-in-full tax on anyone editing behaviour:

| code | inline tests | total | file |
| ---: | ---: | ---: | --- |
| 5,098 | 1,178 | 6,276 | `crates/tiler-compiler/src/physical.rs` |
| 4,163 | 3,228 | 7,391 | `crates/tiler-compiler/src/frontier.rs` |
| 3,795 | 2,007 | 5,802 | `crates/tiler-compiler/src/explain.rs` |
| 3,786 | 161 | 3,947 | `crates/tiler-ir/src/schedule/model.rs` |
| 3,400 | 0 | 3,400 | `crates/tiler-artifact/src/program/model.rs` |
| 3,256 | 2,090 | 5,346 | `crates/tiler-ir/src/index/law.rs` |
| 3,201 | 2,278 | 5,479 | `crates/tiler-ir/src/semantic/registry.rs` |
| 3,081 | 2,111 | 5,192 | `crates/tiler-compiler/src/session.rs` |
| 2,948 | 0 | 2,948 | `crates/tiler-compiler/src/program.rs` |
| 2,946 | 2,163 | 5,109 | `crates/tiler-compiler/src/region.rs` |
| 2,933 | 0 | 2,933 | `crates/tiler-ir/src/index/builder.rs` |
| 2,929 | 0 | 2,929 | `crates/tiler-reference/src/oracle.rs` |
| 2,832 | 0 | 2,832 | `crates/tiler-ir/src/kernel/lower.rs` |
| 2,747 | 2,925 | 5,672 | `crates/tiler-compiler/src/target/feasibility.rs` |
| 2,715 | 176 | 2,891 | `crates/tiler-ir/src/kernel/model.rs` |

The second is the pure test files, which carry the same read-in-full obligation with none of the seam risk:

| lines | file | production siblings to mirror |
| ---: | --- | --- |
| 9,632 | `crates/tiler-compiler/src/pipeline/tests.rs` | `conformance`, `planning`, `trace`, `verify` |
| 7,434 | `crates/tiler-ir/src/kernel/tests.rs` | `builder`, `determinism`, `error`, `lower`, `model`, `verify` |
| 7,142 | `crates/tiler-ir/src/schedule/builder/tests.rs` | ten children incl. `contraction`, `copy`, `coverage`, `reduction`, `tile` |
| 5,829 | `crates/tiler-compiler/src/request/tests.rs` | thirteen children incl. `recognize`, `subject`, `verify`, `refusal` |
| 5,494 | `crates/tiler-ir/src/program/tests.rs` | `abi`, `builder`, `model`, `verify`, `contraction_witness` |
| 4,193 | `crates/tiler-metal/src/tests.rs` | `applicability`, `emit`, `record`, `target` (three `*_tests.rs` siblings already exist) |
| 3,715 | `crates/tiler-cache/src/expansion/tests.rs` | fifteen children incl. `store`, `collect`, `harness`, `retention` |
| 3,080 | `crates/tiler-ir/src/index/refinement/tests.rs` | eleven children incl. `verify`, `proof`, `registry`, `binding` |

**Fact — the first tranche created three of the eight test monoliths above.** `schedule/builder/tests.rs` (7,142), `request/tests.rs` (5,829), and `index/refinement/tests.rs` (3,080) did not exist at `e53eb65f`; the splits moved each parent's test text wholesale into one new file, which was the correct call under a zero-test-edit constraint and is why two of them now sit third and fourth in this list. Those three are also three of the six files that crossed 2,000 lines in this window, so half the crossings are the tranche's own output. Any future ceiling rule has to expect that, and any reader inferring "the splits made things worse" from the population count is reading the wrong number.

**Per-crate concentration.** `tiler-compiler` remains the centre of gravity at 1,493 lines per file across 91 files / 135,867 lines; `tiler-ir` is the largest at 184 files / 157,007 lines but averages 853. Next by ratio are `tiler-build` at 971 and `tiler-runtime` at 912; every other crate is under 950.

**Measurement — the trend, and why a snapshot alone cannot hold it.** Sizes at five bases, each reconstructed with `git ls-tree` rather than remembered:

| base | date | lines | files | over 2,000 | newly over 2,000 since previous |
| --- | --- | ---: | ---: | ---: | ---: |
| `d00a0b50` | 2026-07-25 | 96,213 | 130 | 11 | — |
| `dcda0042` | 2026-08-08 | 408,985 | 407 | 54 | 44 |
| `1ab21ef7` | 2026-08-14 | 450,903 | 442 | 59 | 5 |
| `e53eb65f` | 2026-08-19 | 469,689 | 451 | 60 | 1 |
| `ba46f2b2` | 2026-08-22 | 484,886 | 551 | 61 | 6 |

The 07-25 → 08-08 window is not representative: the workspace went from six crates to thirteen and 304 `.rs` files were added against 2 deleted across 2,865 commits. The three later windows are, and they put the crossing rate at roughly **one to two files per day**. That number decides the floor-check question below.

## Split queue — tranche 2 and tranche 3

Ordered by what a worker pays, which is coupling and distinct concerns weighted by how often anyone has to open the file — not by `wc -l`. Churn is commits touching the file between `1ab21ef7` (2026-08-14) and this base:

```sh
git rev-list --count 1ab21ef7..HEAD -- <path>
```

That reordering is not cosmetic. `region.rs` (5,109) and `semantic/registry.rs` (5,479) are the fourth and third largest source files in the workspace and took **one commit each** in eight days; `physical.rs` at 6,276 took ten and gained 990 lines. Ranking those three by size would put the two nobody is touching ahead of the one everybody is.

### Tranche 2 — filed now

Test text first, as the previous Required-work item ordered, and for once the coupling evidence agrees with the risk evidence: a test monolith has one concern per test and essentially no internal coupling, so its whole cost is the read-in-full obligation.

Churn across the whole over-2,000 population, by `git rev-list --count 1ab21ef7..HEAD -- <path>`, with the top of it:

| commits | file |
| ---: | --- |
| 13 | `crates/tiler-build/src/metal_plan.rs` |
| 11 | `crates/tiler-compiler/src/pipeline/tests.rs` |
| 10 | `crates/tiler-compiler/src/physical.rs` |
| 9 | `crates/tiler-runtime/tests/adapter_route/fixture.rs` |
| 8 | `crates/tiler-ir/src/schedule/model.rs`, `pipeline/trace.rs`, `frontier.rs`, `explain.rs`, `artifact/program/model.rs` |
| 7 | `crates/tiler-ir/src/kernel/tests.rs`, `session.rs`, `compiler/program.rs` |

The most-touched file over 2,000 lines is **not** `pipeline/tests.rs` but `metal_plan.rs`, which is 81% inline test text and therefore in the lift ticket rather than in a split of its own — a result worth stating because ranking by size alone would never have surfaced it. `pipeline/tests.rs` is second on churn and first on size, which is what puts it first here.

1. [`split-the-compiler-pipeline-test-monolith-by-orchestration-phase`](split-the-compiler-pipeline-test-monolith-by-orchestration-phase.md) — 9,632 lines, 11 commits.
2. [`split-the-kernel-test-monolith-into-focused-modules`](split-the-kernel-test-monolith-into-focused-modules.md) — 7,434 lines, 7 commits.
3. [`split-the-schedule-builder-test-monolith-into-focused-modules`](split-the-schedule-builder-test-monolith-into-focused-modules.md) — 7,142 lines, created by the first tranche.
4. [`split-the-compiler-request-test-monolith-into-focused-modules`](split-the-compiler-request-test-monolith-into-focused-modules.md) — 5,829 lines, created by the first tranche.
5. [`split-the-ir-program-test-monolith-into-focused-modules`](split-the-ir-program-test-monolith-into-focused-modules.md) — 5,494 lines, 6 commits.
6. [`lift-the-inline-test-modules-that-dominate-their-files`](lift-the-inline-test-modules-that-dominate-their-files.md) — the four ≥80% files, discharging this ticket's recorded `target.rs` follow-up and widening it to its siblings.

`tiler-metal/src/tests.rs` (4,193) and `tiler-cache/src/expansion/tests.rs` (3,715) are the same class and belong in the same tranche by subject; they are held to tranche 3 only so that this tranche stays the size the first one was, and because `tiler-metal` already carries three `*_tests.rs` siblings whose naming convention the metal lane should pick rather than have chosen for it.

### Tranche 3 — seams named here, not yet filed

Recorded at this level of detail so filing them is transcription rather than research. Each entry names the seam, the shared-internals risk, and what must not move.

**1. `crates/tiler-compiler/src/physical.rs` — 5,098 code, 10 commits, +990 lines in eight days. The worst file in the workspace on every axis at once.** Seven concerns: staged-plan and prologue folding (`staged_plan`, `affine_prologue`, `root_mean_square_scale_plan`); the region *spelling* vocabulary (`RegionWrite`, `RegionSpellingKind`, `RegionSpelling`, `RegionVocabularyWall`, `spell_region`, `spell_output`, `spell_staged`); `PhysicalError`; per-family region construction (`pointwise_region`, `elementwise_region`, `epilogue_region`, `staged_fold_region`, `contraction_region`, `reduction_region`, `fused_region`, `publishing_copy_region`); reduction partitioning (`governed_partition`, `capped_tree_partition`, `split_reduction_regions`, `WorkgroupTreeUnavailable`, `SplitUnavailable`); schedule verification and binding checks (`verify_schedule`, the `*_matches` family); and resource assessment (`AdmissionEvidence`, `ResourceVerdict`, `assess_resources`, `assess_region`, `assess_contract`, `region_proposal`). Seam: `physical/{plan, spelling, error, region, partition, verify, assess}`. No shared-internals tangle — the families communicate through `VerifiedScheduledRegion` and `StagedPlan`, both of which have an owner. Must not move: this file owns no encoder and no identity of its own — `VerifiedScheduledRegion::canonical_identity` forwards `tiler_ir::schedule`'s, so the split cannot change bytes as long as it moves no `tiler_ir` call. Separately, `take_capped_tree_above_cap_candidate_checks_for_test` is a `pub(crate)` test hook in production code and should be recorded as a separate defect rather than quietly relocated.

**2. `crates/tiler-compiler/src/explain.rs` — 3,795 code, 8 commits, +681.** Six concerns: the event and fact vocabulary (roughly a thousand lines of enums from `KeyKind` to `ExplainEvent`); the writer and record machinery; a test-only detail-capacity apparatus living in production code (`DetailCapacityWriterOpeningForTest`, `DetailCapacityAttemptForTest`, `with_detail_capacity_limits_for_test`, and three private `*ForTest` structs); the verified trace and `VerifiedCompilationExplain`; the `render_*` family; and the `encode_*` family. Seam: `explain/{vocabulary, writer, trace, render, encode, validate}`. Must not move: the encoders are identity-bearing and the explain qualifier golden pins their bytes. The `*ForTest` apparatus is a real finding, not a splitting detail — file it separately rather than deciding its home inside a code-motion lane.

**3. `crates/tiler-ir/src/index/law.rs` — 3,256 code, 5 commits.** One 760-line `IndexRealizationLaw` enum followed by one `realize_*` function per family. Seam: `law/{mod, context, pointwise, fold, broadcast, slice, concatenate, contraction, reindex}`, one file per family. The shared internals are `LawContext<'a>` and a handful of emit helpers — about 120 lines, far smaller than any of the families, so this is the *good* case the third Required-work item asks us to distinguish from a tangle. Must not move: `encode_scalar` and the law-row encoder feed `RegisteredIndexRealizationLaw`; no tag or byte may change.

**4. `crates/tiler-ir/src/schedule/model.rs` — 3,786 code, 8 commits, +606.** Cleanest seam of any file here, and the only large one whose inline test module (161 lines) is not the problem. Three concerns with almost no interleaving: the schedule vocabulary (`TensorRole` through `VerifiedScheduledRegion`, about 2,050 lines of types); the derivations (`element_count`, `partial_reduction_*`, `cooperative_*`, `reindex_decodes_are_bijective`, `contributor_count`, `derive_requirements`); and the `push_*` encoders. Seam: `model/{vocabulary, derive, encode}`. Must not move: the `push_*` family is the canonical scheduled-region identity. `push_logical_access_for_test` is the same test-hook smell as `physical.rs`'s and should be recorded with it.

**5. `crates/tiler-artifact/src/program/model.rs` — 3,400 code, 8 commits, +561, and zero tests of its own.** Four concerns: the schema and policy vocabulary; the stored `*Data` rows; the borrowed read-view `*Ref` family (`ArtifactInputRef`, `ArtifactOutputRef`, `InterfaceComponentRef`, `VariantRef`, `DeferredPredicateRef`, `EntryRef`, `BindingRef`, `AbiExprRef` — roughly 640 lines of pure projection); and the identity types with their encoders. Seam: `model/{schema, data, refs, identity, encode}`. Must not move: `CanonicalArtifactProgramIdentity`, `RecordedArtifactProgramIdentity`, both envelope digests, and every `push_*`.

**6. `crates/tiler-compiler/src/frontier.rs` — 4,163 code, 8 commits, +550. Higher risk than its size suggests.** Concerns: the proposal vocabulary; boundary-contract derivation; the *public* provider surface (`PhysicalImplementationProvider`, `ProviderOffer`, `DeclinedStrategy`, `StrategyDeclineCause`, `ImplementationProposal`, `ImplementationContext`, `BaselineImplementation`, `TargetApplicability`, `PhysicalCostEstimate` — twelve `pub` items including a trait); the admission and rejection vocabulary; the enumeration walk; the opaque-call encoders; and `GovernedPhysicalProvider`. Seam: `frontier/{proposal, boundary, provider, admission, enumerate, opaque, governed}`. Must not move: the public surface is Tom's, so the split must not add, remove, rename, or re-signature anything `pub`, and the proposal-identity encoders are byte-pinned. Sequence it after the lower-risk lanes.

**7. `crates/tiler-compiler/src/session.rs` — 3,081 code, 7 commits, +634. Highest risk in the queue: this is the crate's public facade, with forty `pub` items.** Concerns: the refusal and failure vocabulary (about 760 lines of `Target*Refusal` / `Target*Failure`); the borrowed report views (`PlanAlternative`, `SelectedCapability`, `SelectedImplementation`, `AbiConstruction`, `AbiEntry`, `ExplainReport`); the numerical contract and its builder; and the entry points (`compile`, `compile_governed`, and the `public_*` mapping functions). Seam: `session/{refusal, views, contract, compile}` behind an unchanged facade. Must not move: every public path must resolve exactly as before, which makes this pure re-export motion and nothing else. A tested public boundary stays a labelled draft until Tom accepts its surface, so this lane may not take the opportunity to tidy one.

**8. `crates/tiler-ir/src/semantic/registry.rs` — 3,201 code, 1 commit. Cold, so low priority despite being the third-largest source file.** Its obvious first cut is `StandardSemantics` and the built-in operation catalog (`ConstantF32`, `BinaryF32`, `StrictSerialSumF32`, `arithmetic_f32_facts`, `elementwise_binary_shape`), which is a *consumer* of the registry rather than part of it. Seam: `registry/{model, builder, frozen, standard, error, identity}`. Must not move: `SemanticRegistrySnapshotIdentity`, `SemanticDefinitionProjectionIdentity`, `SemanticAdmissionProvenanceIdentity`.

**9. `crates/tiler-compiler/src/region.rs` — 2,946 code, 1 commit, no growth. Cold; queue it last of the named nine.** Four concerns: the candidate and identity vocabulary; the internal `RegionGraph` (about 950 lines with `GraphOperation`, `ValueProducer`, `GraphValue`, `StageTopology`, `SyntheticIntermediate`); the formation walk (`Formation`, `form_candidate`, `classify`, `region_shape`); and the canonical encoders. Seam: `region/{model, graph, form, encode}`. Must not move: `RegionContentIdentity`, `RegionOccurrenceIdentity`, and every `encode_*`.

**Also in tranche 3, same class as tranche 2:** `tiler-metal/src/tests.rs` (4,193) and `tiler-cache/src/expansion/tests.rs` (3,715). **Not queued, and deliberately:** `crates/tiler-compiler/src/target/feasibility.rs` is 5,672 lines but only 2,747 of code, so the general inline-test rule reaches it and a concern split does not yet earn its own lane; the same holds for `semantic/program.rs` (1,760 code under 2,739 of tests) and `fusion_legality.rs` (1,948 under 2,319). The previous Required-work item named `feasibility.rs` in the production list; that was a `wc -l` ranking, and on the code ranking it sits fourteenth.

**No restructuring candidates found.** The third Required-work item asks for dependency tangles — two modules that cannot separate without a shared-internals module larger than either — to be recorded here rather than forced apart. Reading the nine subjects above, none has one. The closest is `index/law.rs`, whose families all reach `LawContext`, and that context is roughly 120 lines against families of 300 to 700, so it is an ordinary shared type rather than a tangle. This is a negative result at this base and not a general one; re-check it per lane, because the file that cannot be split cleanly will only announce itself to someone holding the whole of it.

## Mechanical floor check — declined, 2026-08-22, with a reconsideration trigger

**Decision: no size-ceiling check is added, and the reasoning is a measurement rather than a preference.**

**What it would have caught.** A ceiling check has exactly one honest shape here, and the repository already has two precedents for it: `crates/tiler/tests/workspace_population.rs`, which names an expected population, derives the actual one, and fails in both directions; and the `Makefile`'s fixture-count floors, whose comment states the principle — "a glob that has stopped matching produces no complaints, which is indistinguishable from a population that is clean". Applied to size, that shape is a named list of the 61 files now over 2,000 lines, failing when a file *not* on the list crosses, and requiring an entry to be deleted when a listed file drops below. It would have caught the three files that crossed by ordinary growth in the last window — `index/model.rs` 1,536 → 2,146, `program/realization/tests.rs` 1,828 → 2,028, `semantic/contraction/tests.rs` 1,317 → 2,014 — none of which anyone noticed.

**What it would cost, measured.** The crossing rate at this base is one to two files per day: 5 crossings in the six days to 2026-08-14, 1 in the five days to 2026-08-19, 6 in the three days to this base. The only action that restores green is adding a line to the exemption list. That is the difference between this and both precedents: a crate is admitted to the workspace a few times a year and a `trybuild` fixture is added rarely, so their "intended failure, not an obstacle" is a real review moment. A check that fires most days, and whose repair is editing its own expectation, trains reviewers to bump the number — and a gate whose green is routinely restored by revising what it expects is worse than no gate, because it launders growth as reviewed. AGENTS.md's own rule applies to the exemption list as much as to the assertion: state what it would take for the check to say *no* and mean it.

**And it would be blind to the dominant term.** Growth inside files already over the ceiling was +19,760 lines in the 08-08 → 08-14 window, +11,542 in the next, +3,787 in the last. A crossing check permits all of it. Making it *not* permit that — a per-file ceiling that fails on any growth — reintroduces the laundering problem at every landing instead of at every crossing.

**And its first firings would be against the repair work.** Three of the six crossings in the last window (`schedule/builder/tests.rs`, `request/tests.rs`, `index/refinement/tests.rs`, 16,051 lines together) are files the split tranche *created*. A ceiling would have reddened the lanes that were fixing the problem and been satisfied by three exemptions.

**The counterargument, stated.** Without a check this census is a dated snapshot, and this ticket has now demonstrated twice over that snapshots rot within days — the 2026-08-19 block was stale by five landings and one of its gate instructions was wrong when written. That is a real cost and it is why the decision carries a trigger rather than closing the question.

**Reconsideration trigger.** Re-evaluate when the crossing rate falls below one file per week sustained over two weeks, which is the point at which the exemption-list edit becomes a review moment instead of a chore. Check it with the reconstruction the trend table above uses:

```sh
for r in <base-two-weeks-ago> HEAD; do
  git ls-tree -r "$r" --name-only -- crates | grep '\.rs$' \
    | while read -r p; do echo "$(git show "$r:$p" | wc -l) $p"; done > "/tmp/census_$r.txt"
done
```

then compare the two files for paths over 2,000 in the second and not the first. Record the verdict here with its date and the numbers, `fired` or `not fired`, whichever it is.

**What is added instead, and it is not a check.** The commands above are recorded so that re-deriving this census is a two-minute job rather than the half-day of reconstruction it took here. That is the honest mitigation: the census stays manual, like the rest of the documentation, and AGENTS.md already carries the reading obligation that a mechanical check could not discharge anyway.

## Closes when

The first tranche of splits is landed and recorded here (done above), the second tranche is filed, and the mechanical-check decision is recorded with its rationale (and failure text, if adopted).

**All three are now satisfied at base `ba46f2b2`,** and the coordinator owns the transition. The first tranche is landed and recorded, and its five splits — including the index-refinement one this ticket recorded only as "ticket filed" — are all `done`. The second tranche is filed as the six tickets listed above. The mechanical-check decision is recorded as a decline, with the crossing-rate measurement behind it and a reconsideration trigger; the Closes-when admits "either a mechanical floor check or the reasoned decision not to add one", and this is the second. Tranche 3 is named but not filed, which is what "extend the queue ... and file the next tranche" asks for; if the intent was to file all of it, the nine entries carry their seams and constraints and are transcription work rather than research.
