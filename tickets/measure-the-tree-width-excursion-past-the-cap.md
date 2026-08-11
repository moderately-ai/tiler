---
id: measure-the-tree-width-excursion-past-the-cap
title: Measure the tree width excursion past the cap
status: done
priority: p3
dependencies: []
related: [bound-the-tree-cap-s-unmeasured-downward-direction, cap-the-tree-reduction-participants-at-the-measured-256, calibrate-a-shape-aware-tree-width-cost-row]
scopes: [research/program-planning, implementation/compiler, contracts/navigation, contracts/optimizer, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, measurement]
---
## Per-Fact audit — 2026-08-11, exact base `946e0328`

Every claim below was re-read before the harness or result population was changed.

| Claim | Verdict | Source and reproduction |
| --- | --- | --- |
| The production tree takes the exact admissible participant count nearest 256, ties narrower | **verified** | `crates/tiler-compiler/src/physical.rs`, anchors `pub(crate) fn capped_tree_partition` and `let partition = capped_tree_partition(contributors)`. The lower scan, the strict `candidate < 2 * cap - below` upper scan, and `single_workgroup_tree_region`'s call site were read in full. |
| The rule's arithmetic maximum is 509, an excursion of 253 | **verified** | In `capped_tree_partition`, an above-cap `s` wins only while `s < 2 * 256 - l`; every admissible lower `l` is at least 2. An independent enumeration of `0..4_096` reproduced the production test's 3,530 admitting counts, 1,061 widened counts, and maximum chosen width 509. |
| Below 20,000 contributors, 1,133 counts take two; four is the first, and 1,042 is the first that declines another admissible width | **verified** | An independent exhaustive implementation of the production rule printed `take2=1133 first_take2_with_wider=1042`; direct divisor enumeration gives 514 `{2,257}` with 257 selected and 1,042 `{2,521}` with 2 selected. |
| Width 521 is representable and inside the qualified feasibility rows | **verified, with the authority separated** | `tiler_ir::schedule::MAX_COOPERATIVE_PARTICIPANTS` is 4,096; `workgroup_tree_tile` stages one `f32` per participant, so 521 needs 2,084 bytes; `FIRST_MACOS_APPLE9.local_memory_bytes` declares 32,768. Workgroup capacity is not a compile-profile fact: the declaration carries a `PreparedKernelPreflight` query, and the retained calibration observed its prepared entries admitting 1,024. |
| The retained calibration used only power-of-two contributor counts | **verified** | `spikes/program-planning/reduction-partition-calibration/README.md`, anchor `no shape here has a contributor count that is not a power of two`, agrees with the seven-shape table and retained TSV. Nearest-to-cap and truncate-from-below select the same width at each. |
| No retained row bears on a width above the cap | **false; repaired below** | The retained `sweep.tsv` has ten measured tree rows above 256, including 4 × 8,192 at 512 (10.7718 µs) and 1,024 (16.6812 µs). What is absent is a non-power-of-two count where the production rule selects an above-cap width. Reproduce with `awk -F '\t' '!/^#/ && $1!="rows" && $4=="single-workgroup-tree" && $5>256 && $24=="measured" {n++} END {print n}' spikes/program-planning/reduction-partition-calibration/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/sweep.tsv`. |
| Nothing measured says the excursion should stop at any width | **imprecise; repaired below** | The measured 512- and 1,024-participant rows are bounded evidence about costs beyond the cap, and at 4 × 8,192 both are slower than 256. They do not locate a reusable boundary on a sparse non-power-of-two divisor lattice. |
| The steepest retained span is 9.53 µs at 256 against 48.15 µs at two, 5.05× | **verified** | Exact retained rows at 4 × 8,192 are 9.5344 and 48.1541 µs; the README anchor `Between the best and worst *admissible* partition of one shape the span reaches` reports the rounded claim. |
| The retained harness can be reused unchanged | **imprecise; repaired below without changing the experiment** | Its regions, source/ABI reconstruction, oracle, preparation, timing, and noise controls remain the right machinery. Its `anchor` currently passes the balanced `governed_partition` to both strategies, while production now uses `governed_partition` for the split and `capped_tree_partition` for the tree. Reuse therefore requires distinct byte-identical production anchors before any off-production width is evidence. |

## The question

`capped_tree_partition` now takes the admissible participant count **nearest** `MEASURED_TREE_PARTICIPANT_CAP` rather than truncating at it, which fixed the direction but not its extent. The rule will widen past the cap by at most 253 participants, and that ceiling is arithmetic — a count `s` above the cap beats an admissible `l` at or below it only when `s - 256 < 256 - l`, and `l >= 2` forces `s <= 509` — rather than measured. Retained rows at 512 and 1,024 participants bound two costs beyond the cap, but **nothing measured locates a reusable excursion boundary on a sparse non-power-of-two divisor lattice**.

**Fact — corrected 2026-08-09.** Below 20,000 contributors, 1,133 counts still take **two** participants. Four is the smallest member, where two is the only admissible count. The first member at which the rule takes two while declining a wider admissible choice is 1,042 (`2 * 521`), whose only two admissible counts are 2 and 521: 521 is representable (`MAX_COOPERATIVE_PARTICIPANTS` is 4,096), inside the qualified Apple9 entry's 1,024 threads per workgroup, and stages 2,084 `f32` bytes against a declared 32,768. The rule declines it only because 521 is 265 above the cap while 2 is 254 below.

**Correction — 2026-08-11.** The final sentence above was too broad: the retained calibration measured ten tree rows above 256, up through the prepared entry's admitted 1,024, and at four rows of 8,192 both 512 and 1,024 were slower than 256. That is bounded prior evidence about widths above the cap. What it does not measure is a non-power-of-two contributor count where the production nearest-cap rule selects an above-cap width, nor enough sparse divisor lattices to locate a reusable excursion boundary. The experiment below asks exactly that narrower question.

**Fact — the production excursion is inferred, not measured, at every count this ticket concerns.** [The retained partition calibration](../spikes/program-planning/reduction-partition-calibration/README.md) states its own bound: "no shape here has a contributor count that is not a power of two". On a power of two the largest admissible count at or below the cap *is* the widest the cap admits, so no measured cell separates "nearest the cap" from "largest not exceeding the cap", and no measured cell exercises a production-selected width above the cap. The steepest measured span — the tree at four rows of 8,192, 9.53 µs at 256 participants against 48.15 µs at two, 5.05x — and the retained 512/1,024 rows are the bounded prior evidence this measurement extends.

## The experiment

Sweep the tree's admissible participant counts at **non-power-of-two** contributor counts on the qualified Apple9 macOS host, reusing `spikes/program-planning/reduction-partition-calibration`'s region construction, variant declaration, per-element verification, and noise controls so the results are comparable cell by cell. Preserve byte-identical source and ABI anchoring, but bind each strategy to its current production owner: `governed_partition` for the split and `capped_tree_partition` for the tree. A live Apple9 assertion must pass before a result may be retained.

Shapes should include at least: 514 (`2 * 257`, admissible {2, 257}), 1,042 (`2 * 521`, {2, 521}), and one count with a dense sub-cap lattice and a sparse one just above it, so a cost curve either falls off past the cap or does not. Hold the row count at two of the crossover contour's separated values so a finding is not confined to one side of it.

## What would change

- A measured excursion boundary replaces the arithmetic 509 in `capped_tree_partition`, or confirms it.
- A measurement that the cost curve is flat between 2 and 521 at 1,042 contributors would close this as *no rule change needed* and would be worth recording, because the whole downward-direction argument rests on the curve being steep.
- A measurement disagreeing with the 5.05x direction at a non-power-of-two count would reopen `MEASURED_TREE_PARTICIPANT_CAP` itself.

## Non-goals

No change to `MEASURED_TREE_PARTICIPANT_CAP`'s value without evidence at its own power-of-two shapes. No selection change. No widening that reaches past `tiler_ir::schedule::MAX_COOPERATIVE_PARTICIPANTS` or past a declared workgroup width — a width preference that withdraws a legal alternative has decided feasibility, which is what ruled out `max(capped_tree_partition, governed_partition)` in [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md).

## Outcome — 2026-08-11: the current rule is supported at the minimal excursion and refuted at both the dense lattice and sparse cutoff

**The measurement is complete; production is deliberately unchanged.** The finite matrix was frozen before timing as `{4, 16,384}` rows × `{514, 780, 1,042}` contributors. It measures every exact participant count that divides a contributor sequence into at least two parts of at least two: 52 tree widths per run. Primary and repeat each attempted 52, measured 52, and declined zero. The current production widths — derived from the compiler-published ABI and marked `production`, never misattributed to `governed_partition` — are 257, 260, and 2.

### Result and repeated boundary verdicts

The retained analyzer validates the exact matrix, row schema, metadata, environment, source/result digests, resource fields, and one production mark per shape before comparing costs. A gap counts only when it exceeds twice the two medians' combined standard errors.

| shape | production | best | production / best | verdict |
| --- | --- | --- | --- | --- |
| 4 × 514 | P257 at 9.8717 µs | P257 at 9.8717 µs | 1.000× | within noise of best; P2 and P257 share the plateau |
| 16,384 × 514 | P257 at 444.3644 µs | P257 at 444.3644 µs | 1.000× | best; P257 alone is in the plateau |
| 4 × 780 | P260 at 6.1243 µs | P39 at 3.3274 µs | **1.841×** | production beaten |
| 16,384 × 780 | P260 at 653.7328 µs | P39 at 631.3783 µs | **1.035×** | production beaten |
| 4 × 1,042 | P2 at 9.1574 µs | P521 at 7.1700 µs | **1.277×** | production beaten |
| 16,384 × 1,042 | P2 at 906.9511 µs | P521 at 896.1025 µs | **1.012×** | production beaten |

**All eight predeclared boundary verdicts repeat, eight for eight.** At 514, P2 → P257 is inside noise at four rows and faster at 16,384. At 780, P195 → P260 is faster and P260 → P390 is sharply slower at both row counts. At 1,042, P2 → P521 is faster at both. The repeat produces those same eight classifications. Across the exact 52-row population, primary-to-repeat median relative `p50` difference is 0.24%; maximum is 12.71%, on the low-cost side where host round-trip noise is largest.

This answers the ticket's alternatives more narrowly than its original `What would change` predicted. The cost is **not flat** between 2 and 521, so the sparse residue is real. But the matrix does not yield one excursion boundary: 257 is supported at the minimal sparse excursion, while the dense lattice's optimum is 39, far below both the cap and production's 260. Nearest-to-cap distance is not a sufficient general width model. Six cells on one host refute a universal rule but do not support another one, so `capped_tree_partition` and `MEASURED_TREE_PARTICIPANT_CAP` are unchanged. [`calibrate-a-shape-aware-tree-width-cost-row`](calibrate-a-shape-aware-tree-width-cost-row.md) owns a wider, held-out, shape-aware study covering both the dense-lattice below-cap optimum and sparse-cutoff reversal.

### Retained record, exact row

[`results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-excursion/`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-excursion/) holds primary `sweep.tsv`, same-matrix `repeat.tsv`, pinned `environment.tsv`, and device-free `analysis.txt`. The harness and analyzer are [`src/main.rs`](../spikes/program-planning/reduction-partition-calibration/src/main.rs) and [`src/excursion.rs`](../spikes/program-planning/reduction-partition-calibration/src/excursion.rs).

Environment: macOS 27.0 build `26A5388g`, arm64, Apple M4 Max asserted live as Apple9; `/Applications/Xcode.app` Xcode 26.6 build `17F113`; SDK macosx 26.5 build `25F70`; offline Metal/AIR-LLD 32023.883; nightly-2026-07-19, rustc 1.99.0-nightly `eff8269f7`. The coordinator reserved a quiet window; primary then repeat ran sequentially with no concurrent Cargo or full gate. Primary load was `2.97 4.33 4.66` before and `3.03 4.15 4.57` after; repeat was `2.95 4.12 4.56` before and `2.86 3.96 4.48` after.

**Custody correction before commit.** The exact timed release executable remains locally observable at 7,807,056 bytes, mtime `2026-08-11T06:23:37Z`, SHA-256 `c9c3e5718a3a7aa3531179d735783f01c254b4907dda7f1a345ae96e670b571d`; it is a `target/release` build product and is not checked in. The final `hash.main` is explicitly the replay source digest, not an unsupported claim that those source bytes built the timed executable. Post-run changes rename only the result selection label from `governed` to `production` and repair documentation/Clippy reasons; the retained TSV labels were repaired in the same way. Kernel construction, anchors, input, oracle, preparation, warm-up, timing, and numeric fields did not change. The exact relation and executable observations are pinned in `environment.tsv` and checked by the analyzer.

The scheduled tree source requests exactly `4 × participants` bytes of staging. The retained `threadgroup_bytes` field is the prepared Metal function's reported static allocation, observed as that request rounded up to 16 bytes across all rows: P257 requests 1,028 and reports 1,040; P521 requests 2,084 and reports 2,096. The validator pins that distinction; it is an observation on this prepared-pipeline row, not a portable alignment guarantee.

### Six independent subject perturbations, assertions unchanged, all restored

1. Rebuilding the tree at P2 while production publishes P257 failed the source anchor: `4x514: the rebuilt single-workgroup tree at the governed participant count (257) does not emit the source the compiler emits`. The final message says `production participant count`; only that terminology changed after this observed failure.
2. Adding one to the rebuilt tree launch width failed the ABI anchor: `assertion left == right failed: 4x514: compiler publishes Launch { grid_threads: 1028, threads_per_workgroup: 257 } and spike derives Launch { grid_threads: 1028, threads_per_workgroup: 258 }`.
3. Changing every unit operand from `1.0` to `2.0` failed the unchanged per-element oracle: `4x514 single-workgroup-tree at 2 x 257: output[0] is 1028 (44808000), expected 514 (44008000)`.
4. Redirecting the result binding to the input buffer failed the unchanged per-element oracle: `4x514 single-workgroup-tree at 2 x 257: output[0] is 0 (00000000), expected 514 (44008000)`.
5. Removing the P257 production mark from the primary TSV, while updating only its custody digest so semantic validation was reached, failed: `assertion left == right failed: .../sweep.tsv: 4x514 production mark moved`, `left: []`, `right: [257]`.
6. Adding one source comment to `src/regions.rs` with the retained digest unchanged failed: `assertion left == right failed: src/regions.rs digest moved`, observing `a410f5cf671c...` against retained `1a7d4f492fef...`.

### Review correction — 2026-08-11: exact environment values, not presence

Independent review replaced `environment.toolchain` and `environment.rustc` with bogus 1970/deadbeef values, and separately replaced the quiet-window occupancy with a concurrent-build claim. The analyzer still passed because those rows, both dates, and all four loads were checked only for presence. That was a real evidence defect: the README and this Outcome claim their exact values.

The corrected validator equality-pins **all 39 non-digest environment values** in the retained record, including both UTC dates, logical-core count, selected and default developer directories, rustc and named toolchain, occupancy, all four load rows, measurement method, staging-allocation meanings, and timed-executable custody observations. It separately requires the exact **45-key** population and validates all six digest values against their subjects. Extra, missing, stale, or merely present environment facts can no longer support `# validation passed`.

Four independent post-fix perturbations, validator unchanged, were observed and restored:

1. `environment.toolchain = nightly-1970-01-01-deadbeef` failed with ``environment key `environment.toolchain` moved``; `left: Some("nightly-1970-01-01-deadbeef")`, `right: Some("nightly-2026-07-19-aarch64-apple-darwin")`.
2. `host.occupancy = concurrent Cargo build ran throughout both timed submissions` failed with ``environment key `host.occupancy` moved``; `left: Some("concurrent Cargo build ran throughout both timed submissions")`, `right: Some("coordinator-reserved quiet window; primary then repeat ran sequentially; no other Cargo or full gate ran during either timed submission")`.
3. `environment.date_utc.primary = 1970-01-01T00:00:00Z` failed with ``environment key `environment.date_utc.primary` moved``; `left: Some("1970-01-01T00:00:00Z")`, `right: Some("2026-08-11T06:23:36Z")`.
4. `host.load.repeat.after = 99.99 99.99 99.99` failed with ``environment key `host.load.repeat.after` moved``; `left: Some("99.99 99.99 99.99")`, `right: Some("2.86 3.96 4.48")`.

The clean retained environment again reproduces `analysis.txt` byte for byte after those restorations.

### Boundary

One profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract (`FLUSH_AND_REASSOCIATE_F32`), one multiply-add-prologue/trailing-axis-reduction family, `f32`, one host row, six shapes. Wall-clock commit-to-completed difference quotient, not GPU-busy time or a latency estimate. Sixty-four repeated encodes make it steady-state per-plan cost, not first-call latency. Unit inputs make every regrouping exactly equal, so this is correctness coverage for ownership/coverage/synchronization but no rounding-error measurement. No other Apple family, OS row, dtype, device, program family, row-count distribution, or irregular/ragged partition inherits the result.

## Graph maintenance

Filed 2026-08-08 from [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md), which landed the rule and bounded the direction arithmetically rather than by measurement. Every count above was verified by exhausting the range at that ticket's base rather than taken from a report.

Scopes `contracts/navigation`, `contracts/optimizer`, and `research/target-profiles` were added on 2026-08-11 because the completed measurement makes the two catalogs, fusion contract's blanket cost-`Unknown`, and authority ledger's consumer boundary imprecise; each changed record is named in the Outcome. They authorize evidence corrections only, not a production selection or public-boundary change.


## Integration — 2026-08-11

Integrated reviewed candidate a574248cd093ae5409275be4cd6fe904f60d503f into main at merge commit 12742df2032810545e22735453518fa01ab2c905. The production selector and cap remain unchanged; calibrate-a-shape-aware-tree-width-cost-row owns the wider held-out study.
