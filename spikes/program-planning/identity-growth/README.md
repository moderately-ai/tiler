---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.identity-growth"
kind: "experiment"
title: "How kernel-program identity grows against its 64 MiB bound"
topics: ["program-planning", "identity", "coverage", "index-refinement", "limits"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.program-planning.complete-model-ingestion-and-execution"]
entrypoints: ["spikes/program-planning/identity-growth/src/main.rs"]
last_verified: "2026-08-08"
ticket: "measure-executable-coverage-identity-growth-against-the-program-identity-bound"
---

# How kernel-program identity grows against its 64 MiB bound

[`measure-executable-coverage-identity-growth-against-the-program-identity-bound`](../../../tickets/measure-executable-coverage-identity-growth-against-the-program-identity-bound.md) inherited a structural inference with exactly one measured point behind it: because `CanonicalKernelProgramIdentity` embeds one whole reached-only executable-coverage identity per covered occurrence, one record per graph operation, and because each of those records embedded the complete `SemanticGraphIdentity` of the bound graph, program identity should be **Θ(operations × graph-encoding size)** — quadratic in graph size — against a hard `MAX_PROGRAM_IDENTITY_BYTES` of 64 MiB that fails closed with a typed refusal. The one measurement was a five-occurrence stage key at 21,366 bytes. What was unknown was how far from realistic program sizes the refusal sits.

This spike replaced that single point with a curve. On 2026-08-05 the curve was the quadratic the inference predicted. [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) then folded the per-record graph restatement to a fixed-width digest, and the run below measures what that did: **the curve is linear, and the quadratic coefficient is exactly zero rather than small.** Every result is retained at its own path, because each is evidence about the tree it measured, and [`results/README.md`](results/README.md) says which tree that was for each of the five.

**Six runs are retained and only the last describes the compilation path as it stands.** Four bounds have stopped this family in turn — `semantic_operations = 8`, the explain authority's detail ceiling, `region_expansions`, and `region_members = 32` — and each was removed by a ticket the previous run's wall table reported. The fifth run was the first whose ladder ends on `semantic_operations` itself, which is what the domain was supposed to be all along, and the sixth reproduces it byte for byte six commits later while adding the wall this spike had lost. The path qualifiers rather than new dates are because four of the six share a date with a predecessor and differ only in the compiler tree.

**Rows reproduced their predecessor's structural columns byte for byte through the first four runs, then once did not, and now do again.** Through 2026-08-07's `post-coverage-extremes` run the columns were identical over every shared domain — which was the measurement that neither the explain-ceiling fix nor the coverage-extremes fix moved any identity, since explain records are diagnostics and widening which regions a search reaches does not change the identity of the plan it selects. The `post-derived-region-budgets` run's `graph_bytes` still reproduced exactly, but its `program_bytes` and `coverage_bytes` were each larger by exactly **`5n − 4`** at every shared point; that is not the budget derivation — budgets enter the canonical *request subject*, not the kernel-program identity — and [`results/README.md`](results/README.md) records the attribution and its limits. The current `post-restored-planning-wall` run reproduces **all nine structural columns at all sixty-one points**, so the fit below is now a statement about two trees rather than one.

## What it drives

For each operation count in the reachable domain it builds a semantic program, compiles it through the **ordinary** path — the public `tiler_compiler::session::compile` boundary, whose lowering mints real index-refinement receipts, derives `CoveredOccurrence` records from them, and drives `KernelProgramBuilder` — and reads the byte length of the canonical identity off the verified program the compilation produced. Nothing here constructs an identity, a receipt, or a coverage record; a synthetic one would measure the harness rather than the compiler.

The generator emits one input, one hoisted constant, and a chain of `F32Multiply` steps, so the operation count is exactly `1 + multiplies` and every integer in the domain is reachable. It is a pure multiply chain rather than a mixed multiply/add body because a region holding a multiply adjacent to an add is refused under the one contract that permits arithmetic contraction, and a generator whose admissibility depended on the contract would put a second variable into a one-variable sweep.

## Running it

```sh
cd spikes/program-planning/identity-growth
cargo run --release
```

Four perturbations exist so that the harness's refusals are watched rather than trusted, and each exits non-zero:

```sh
cargo run --release -- --perturb=program    # a program this build reaches no kernel program for
cargo run --release -- --perturb=coverage   # a corrupted coverage expectation
cargo run --release -- --perturb=fit        # one byte moved in one measured row
cargo run --release -- --perturb=wall       # the wrong class expected at one wall
```

**Read the diagnosis, not the exit code.** A non-zero exit says only that *something* refused, and for `--perturb=program` that is not enough: its program is a wall-table entry, so a perturbation whose program has quietly started compiling would leave sixty-one identical rows whose operation counts are not consecutive, the exact fit would refuse them, and the run would exit 1 having tested nothing. Each mode names the check that refused it — three on stderr and `--perturb=fit` in the summary block, because the fit's verdict belongs beside the fit. Every stage runs regardless of what the ones before it decided, and the exit code is their conjunction; the wall table is the only stage that can attribute a dead perturbation, so it must not be skipped by an earlier refusal.

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

A run is retained by redirecting it to a new directory rather than over an existing one — `cargo run --release > results/<date>-<host>/growth.tsv` — and [`results/README.md`](results/README.md) is then extended with the tree it measured. Overwriting an existing file destroys the only record of the encoding it read.

## The domain is sixty-one points, and it is finally the governed budget itself

**Fact.** `DeterministicBudgets::governed` (`crates/tiler-compiler/src/request.rs`) caps `semantic_operations` at **62**, raised from 8 on 2026-08-05 and sized "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". `DeterministicBudgets` is `pub(crate)` and `CompileRequest` binds `InstalledCapabilities::governed`, so no public caller can state a wider budget.

**Measurement, 2026-08-07 — that budget is now the wall, and it is the first time it has been.** The run compiles at each point below and requires the stated outcome with its class *and its phase*, so which bound binds is measured rather than read off a constant:

| Program | Outcome | Class | The bound that produces it |
| --- | --- | --- | --- |
| chain, 2..=62 operations, extent 4 | verifies | — | — |
| chain, 63 operations, extent 4 | refuses | `BudgetExhausted` | `semantic_operations = 62`, raised *before* any target-qualified trace exists. It is the only wall the operation-count axis has. |
| chain, 2 operations, extent 268,435,456 | verifies | — | the control for the row below |
| chain, 2 operations, extent 268,435,457 | refuses | `NoFeasiblePlan` | `target.grid-axis`: the whole-program region needs one thread per element and the declaration measures `max_threads_per_grid_axis` at 268,435,456. Raised *after* planning, inside the target loop, carrying a 26-record sealed trace. |

**So the reachable domain widened from thirty-one points to sixty-one, and the governed budget of 62 stopped being unreachable: it is the ladder's own widest measured point.** The phase is compared rather than only the class because it is an independent property of each wall — a `semantic_operations` refusal that started arriving after planning would mean the program-size gate had moved behind the target loop, and a target rejection arriving before it would mean a per-target refusal had moved in front of the trace boundary. Neither is a finding the class alone can report.

**The last two rows are the ladder's *other* fixed parameter, and they are what restore the planning-phase arm.** The generator has two free parameters and the ladder sweeps one; the extent it holds fixed at 4 has a domain of its own, and the bound at its top is the only refusal this family has that arrives after a trace is opened. The wall table brackets it rather than sweeping it, because no measured column varies with extent — a second ladder would add rows and no information. The bracket is what removes the standing claim: 268,435,456 is written down in the harness because no public accessor exposes the declared maximum, and a measured row that moved in *either* direction fires one of the two entries.

**It cannot dissolve the way its predecessors did.** The two walls that reached planning before 2026-08-07 — `region_expansions` at twelve operations and `region_members` at thirty-three — were compiler-internal ceilings, and each was removed by re-deriving a bound. This one is a **measured hardware row**; widening it means measuring a wider Apple family, not deriving a number differently.

### What was here yesterday, and why it is not a number swap

**The thirty-three-operation wall this table carried this morning is gone, and it took twenty-nine more with it.** 33..=62 refused `BudgetExhausted` because `region_members` was the bare constant **32**: a pointwise family's recognized partition is its whole program and nothing smaller is implementable, so the whole-program region was the only cover with a plan and above thirty-two operations it was refused as a *region* although every bound on the program's own *size* admitted it. [`derive-the-region-shape-budgets-from-the-declaration`](../../../tickets/derive-the-region-shape-budgets-from-the-declaration.md) made all three region-shape bounds derivations over the declaration rather than constants — `region_members` from `semantic_operations` (**62**), `region_live_values` from `semantic_values` (**80**), and `region_boundary_outputs` from the declared output count (**3**, *narrower* than the 8 it replaced) — on the ground that a region is a subset of the program it covers, so the stated admission envelope and the actual planning envelope became the same formulas over one declaration rather than two disagreeing ceilings. The narrowing does not bind here: this family declares one output.

That means **every ladder and wall table this spike retained before 2026-08-07's second run described a domain truncated by a bound that no longer exists**, and the four earlier `results/` files each stop on a different such bound. [`results/README.md`](results/README.md) names them, so a reader reconciling a retained file against a rerun can tell which regime it belongs to instead of reading a smaller ladder as a smaller domain.

**The twelve-operation wall this table carried on 2026-08-06 went the same way, in a different class.** 12..=62 refused `NoFeasiblePlan` because `region_expansions` (10,000) stopped candidate growth before the whole-program region was formed — candidate count falling from 66 at eleven operations to 20 at twelve — so every surviving cover named an unimplemented region and the portfolio was empty. That contradicted `DeterministicBudgets::governed`'s own claim that every `region_*` bound "bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive": growth reaches the whole-program set *last*, so the one candidate the profile could implement was the first thing an exhausted search lost. [`region-expansion-exhaustion-loses-the-only-feasible-plan`](../../../tickets/region-expansion-exhaustion-loses-the-only-feasible-plan.md) made region formation retain **both** coverage extremes before growth starts — the singletons and the whole-program region, symmetric with the cover authority, which already retains the fully-materialized and fused covers unconditionally and whose guarantee was empty without this one. It also corrected the claim: two of the five `region_*` bounds bound a search and three bound a region's admissible *shape*, and only the second kind can refuse a program. **All three of that second kind are now derivations, and none of them refuses this family.**

**The eleven-operation wall this table carried before that is gone the same way.** It refused `InvalidCompilerOutput` — documented as "always a defect in Tiler rather than in the caller's program" — with 3,478 records in the sealed trace, of which about 2,300 were one `selection.region-coverage.v1` record per *rejected cover* against a single unimplemented singleton region. The rule now emits one record per unimplemented *region*, carrying a `blocked-covers` count, so the record population is bounded by the region count rather than by the cover count. Filed and closed as [`refuse-nothing-legal-on-the-explain-detail-ceiling`](../../../tickets/refuse-nothing-legal-on-the-explain-detail-ceiling.md).

The predecessor of this section asserted a single nine-operation probe and one budget. It **fired on 2026-08-06** — the budget had moved from 8 to 62 and the probe compiled — which is the refusal it existed to report and the reason the table replaced it. The table has now fired three more times: the compiled-where-a-refusal-was-expected arm at eleven, again at twelve, and again this afternoon at thirty-three *and* sixty-two at once; and the class comparison at sixty-two, where one fix moved a refusal from `NoFeasiblePlan` to `BudgetExhausted` without removing it. That is why each wall is probed *with its class and phase*: a wall that moves in kind fails as loudly as one that disappears, and a wall that disappears because the bound behind it dissolved is the outcome this table exists to produce.

**What the table lost on 2026-08-07 morning, and how it was recovered that afternoon.** No point on the operation-count axis refuses *after* planning any more, so `--perturb=program` — which takes its program from the wall table precisely so it carries no standing claim of its own — stopped watching the planning-phase abort specifically, and `unplannable_program` silently selected a refusal one phase earlier than the one it documents. [`restore-a-planning-phase-refusal-to-the-identity-growth-harness`](../../../tickets/restore-a-planning-phase-refusal-to-the-identity-growth-harness.md) restored it by generalizing the wall entry from an operation count to a program constructor, which is what made the extent axis statable at all, and by fixing the ordering defect described below.

### The enumeration behind that, with its population named

**Measurement, 2026-08-07, base `25e76d5d`.** "No program refuses after planning" is not a claim this spike can make about semantic programs in general, so what was enumerated is stated instead. **Sixty-six programs**, every one compiled through the ordinary path against the authoritative macOS Apple9 declaration under `FLUSH_SUBNORMALS_TO_ZERO_F32`:

| Population | Count | Outcome |
| --- | --- | --- |
| chain, 2..=63 operations at extent 4 | 62 | 2..=62 compile; 63 refuses `BudgetExhausted` **before** planning |
| chain, 2 operations at extents 1,024 / 65,536 / 16,777,216 / 268,435,455 / 268,435,456 | 5 | all compile |
| chain, 2 operations at extents 268,435,457 / 1,073,741,824 / 4,294,967,296 | 3 | all refuse `NoFeasiblePlan` **after** planning, each naming `target.grid-axis` with `threads=<extent>:268435456` |
| `rms_norm(matmul(a, b), a)` over two `[2, 2]` inputs, and its two-declared-input control | 2 | both refuse `UnsupportedCapability { rule: "accuracy.elementary.no-installed-realization" }` **before** planning |

Sixty-one of the sixty-six run on every ordinary sweep; four more run in every wall section, including the one retained as the planning-phase entry and its control. The last row is the one that ran once and is recorded rather than retained, and it is worth stating because it is where this ticket was told to look: the staged normalization *does* refuse `NoFeasiblePlan` after planning under `TargetProfile::governed()`, which `crates/tiler-compiler/tests/staged_family_over_a_materialized_intermediate.rs` asserts, and it does **not** under this spike's declaration — that profile installs no elementary realization for the normalization at all, so the refusal arrives two phases earlier and the family says nothing here. **A wall this spike could not probe under its own profile is not a wall this spike may record.**

**What this population does and does not bound.** It bounds one program family over two parameters and one two-occurrence staged family, under one contract and one target profile. It is not a statement about semantic programs, about other families, or about other declarations. What it supports is narrower and sufficient: at least one program this declaration admits refuses after planning, it is inside the wall table, and the mode that needs one selects it by phase rather than by position.

## Result

**Measurement, 2026-08-08**, retained at [`results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv`](results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv). Host: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max — a coordination host running other agents' builds, which the byte columns are immune to by construction and which the `compile_ms` column is not. **Load averages were not captured at this run**, unlike its five predecessors; they were heavy but unrecorded, so this file's `compile_ms` column carries even less than the usual reachability information and nothing else here depends on them. Toolchain: `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), the repository pin resolved by ancestry. Repository base `cc667626`; the branch that ran it touched no `crates/` file. Compile-only: nothing is emitted, linked, or dispatched.

**The graph column moved and the coverage column did not, and that pairing is the finding.** The predecessor was pinned to base `25e76d5d`, 268 commits earlier, 41 of which touch `crates/`. Compared column by column rather than through the fit, over all sixty-one shared points: `requested`, `operations`, `coverage_records`, `stages` and `alternatives` are identical; `graph_bytes` is larger by exactly **`n`**; `program_bytes` and `widest_alternative_bytes` are each larger by exactly **`n + 1`**; and `coverage_bytes` is **identical at every point**. **Inference — the graph column moved because [`carry-a-sourced-shape-on-semantic-values`](../../../tickets/carry-a-sourced-shape-on-semantic-values.md) stepped `tiler.semantic-graph.v2` to `v3` on 2026-08-07**, writing every extent through `SourcedShape::encode`, which prepends a source tag where the retired `encode_shape` wrote eight untagged big-endian bytes: one tagged extent per operation result over this rank-1 family is one byte per operation. Exactly one of the forty-one commits touches `crates/tiler-ir/src/semantic/identity.rs` or `crates/tiler-ir/src/shape/`, and it is that one — which narrows the attribution without being a bisection. The residual `+1` on `program_bytes` is a *constant* and is therefore not the per-extent tag; it is left unattributed rather than guessed at. **That `coverage_bytes` did not move at all is [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)'s fold read off the data**: a coverage record names the graph by fixed-width digest instead of restating it, so a graph that grew by `n` bytes propagated none of that growth into the `n` records referencing it, where the pre-fold encoding would have paid `n` bytes in each. [`results/README.md`](results/README.md) carries the comparison and its bound.

The predecessor's own re-measurement note is retained as history: it was pinned to base `cee4fe1a` and reproduced all nine structural columns identically at base `25e76d5d`, so `3530n + 723` described two trees rather than one for as long as it stood.

| Operations | Coverage records | Graph identity (bytes) | Program identity (bytes) | Coverage bytes | Mean record (bytes) |
| --- | --- | --- | --- | --- | --- |
| 2 | 2 | 419 | 7,786 | 5,980 | 2,990.0 |
| 3 | 3 | 554 | 11,317 | 9,281 | 3,093.7 |
| 4 | 4 | 689 | 14,848 | 12,582 | 3,145.5 |
| 5 | 5 | 824 | 18,379 | 15,883 | 3,176.6 |
| 6 | 6 | 959 | 21,910 | 19,184 | 3,197.3 |
| 7 | 7 | 1,094 | 25,441 | 22,485 | 3,212.1 |
| 8 | 8 | 1,229 | 28,972 | 25,786 | 3,223.2 |
| 9 | 9 | 1,364 | 32,503 | 29,087 | 3,231.9 |
| 10 | 10 | 1,499 | 36,034 | 32,388 | 3,238.8 |
| 11 | 11 | 1,634 | 39,565 | 35,689 | 3,244.5 |
| 12 | 12 | 1,769 | 43,096 | 38,990 | 3,249.2 |
| 13 | 13 | 1,904 | 46,627 | 42,291 | 3,253.2 |
| 14 | 14 | 2,039 | 50,158 | 45,592 | 3,256.6 |
| 15 | 15 | 2,174 | 53,689 | 48,893 | 3,259.5 |
| 16 | 16 | 2,309 | 57,220 | 52,194 | 3,262.1 |
| 17 | 17 | 2,444 | 60,751 | 55,495 | 3,264.4 |
| 18 | 18 | 2,579 | 64,282 | 58,796 | 3,266.4 |
| 19 | 19 | 2,714 | 67,813 | 62,097 | 3,268.3 |
| 20 | 20 | 2,849 | 71,344 | 65,398 | 3,269.9 |
| 21 | 21 | 2,984 | 74,875 | 68,699 | 3,271.4 |
| 22 | 22 | 3,119 | 78,406 | 72,000 | 3,272.7 |
| 23 | 23 | 3,254 | 81,937 | 75,301 | 3,274.0 |
| 24 | 24 | 3,389 | 85,468 | 78,602 | 3,275.1 |
| 25 | 25 | 3,524 | 88,999 | 81,903 | 3,276.1 |
| 26 | 26 | 3,659 | 92,530 | 85,204 | 3,277.1 |
| 27 | 27 | 3,794 | 96,061 | 88,505 | 3,278.0 |
| 28 | 28 | 3,929 | 99,592 | 91,806 | 3,278.8 |
| 29 | 29 | 4,064 | 103,123 | 95,107 | 3,279.6 |
| 30 | 30 | 4,199 | 106,654 | 98,408 | 3,280.3 |
| 31 | 31 | 4,334 | 110,185 | 101,709 | 3,280.9 |
| 32 | 32 | 4,469 | 113,716 | 105,010 | 3,281.6 |
| 33 | 33 | 4,604 | 117,247 | 108,311 | 3,282.2 |
| 34 | 34 | 4,739 | 120,778 | 111,612 | 3,282.7 |
| 35 | 35 | 4,874 | 124,309 | 114,913 | 3,283.2 |
| 36 | 36 | 5,009 | 127,840 | 118,214 | 3,283.7 |
| 37 | 37 | 5,144 | 131,371 | 121,515 | 3,284.2 |
| 38 | 38 | 5,279 | 134,902 | 124,816 | 3,284.6 |
| 39 | 39 | 5,414 | 138,433 | 128,117 | 3,285.1 |
| 40 | 40 | 5,549 | 141,964 | 131,418 | 3,285.4 |
| 41 | 41 | 5,684 | 145,495 | 134,719 | 3,285.8 |
| 42 | 42 | 5,819 | 149,026 | 138,020 | 3,286.2 |
| 43 | 43 | 5,954 | 152,557 | 141,321 | 3,286.5 |
| 44 | 44 | 6,089 | 156,088 | 144,622 | 3,286.9 |
| 45 | 45 | 6,224 | 159,619 | 147,923 | 3,287.2 |
| 46 | 46 | 6,359 | 163,150 | 151,224 | 3,287.5 |
| 47 | 47 | 6,494 | 166,681 | 154,525 | 3,287.8 |
| 48 | 48 | 6,629 | 170,212 | 157,826 | 3,288.0 |
| 49 | 49 | 6,764 | 173,743 | 161,127 | 3,288.3 |
| 50 | 50 | 6,899 | 177,274 | 164,428 | 3,288.6 |
| 51 | 51 | 7,034 | 180,805 | 167,729 | 3,288.8 |
| 52 | 52 | 7,169 | 184,336 | 171,030 | 3,289.0 |
| 53 | 53 | 7,304 | 187,867 | 174,331 | 3,289.3 |
| 54 | 54 | 7,439 | 191,398 | 177,632 | 3,289.5 |
| 55 | 55 | 7,574 | 194,929 | 180,933 | 3,289.7 |
| 56 | 56 | 7,709 | 198,460 | 184,234 | 3,289.9 |
| 57 | 57 | 7,844 | 201,991 | 187,535 | 3,290.1 |
| 58 | 58 | 7,979 | 205,522 | 190,836 | 3,290.3 |
| 59 | 59 | 8,114 | 209,053 | 194,137 | 3,290.5 |
| 60 | 60 | 8,249 | 212,584 | 197,438 | 3,290.6 |
| 61 | 61 | 8,384 | 216,115 | 200,739 | 3,290.8 |
| 62 | 62 | 8,519 | 219,646 | 204,040 | 3,291.0 |

Coverage records equal the semantic operation count at every point, and the run refuses if they ever do not.

### The curve is exactly linear, and the fit is an equality rather than a resemblance

The first difference of the program-identity column is **3,531 at every step** and the second difference is **0**, so the general quadratic the harness fits comes back with a zero leading coefficient, and it is reported only after reproducing every measured point to the byte:

```
program_bytes(n) = 3531n + 724        residual 0 at all sixty-one points
graph_bytes(n)   =  135n + 149        residual 0 at all sixty-one points
```

| n | 2 | 3 | 4 | 32 | 33 | 51 | 60 | 61 | 62 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| measured | 7,786 | 11,317 | 14,848 | 113,716 | 117,247 | 180,805 | 212,584 | 216,115 | 219,646 |
| `3531n + 724` | 7,786 | 11,317 | 14,848 | 113,716 | 117,247 | 180,805 | 212,584 | 216,115 | 219,646 |
| residual | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

**The linear form has now survived a doubling of its domain and two coefficient moves, and the three facts have different causes.** Thirty new consecutive operation counts, 33..=62, landed on one straight line with a second difference of exactly zero — that is a check the form could have failed and did not, and it is the only kind of confirmation available now that no wall stands below `semantic_operations`. What moves is the line itself, twice: `3525n + 727` was displaced by exactly `5n − 4` under an index-refinement encoding step, giving `3530n + 723`; that in turn was displaced by exactly `n + 1` under the `tiler.semantic-graph.v2 → v3` extent tagging, giving `3531n + 724`. Each older ladder is recovered from the newer one by subtraction rather than contradicted by it. [`results/README.md`](results/README.md) carries both attributions and the bound on each.

**That correction retires a claim this section used to make.** It said the fit "has been confirmed twenty-one steps outside the domain it was derived on", meaning `3525n + 727` was fitted on 2..=10 and then met the eleventh point and all of 12..=32 exactly as two defects stopped refusing them. Those confirmations were real on the trees that produced them and they do not carry to this one: the same points now measure `6n − 3` bytes higher. **A fit confirmed out of domain is a claim about one encoding, and it expires when the encoding moves.** What carries across every tree is the *form* — every run since ADR 0104 has read a quadratic coefficient of exactly zero.

**The mechanism, stated as the relation a reader can check: the program curve's quadratic coefficient is 0 while the graph curve still grows at 135 bytes per operation.** On 2026-08-05 those two numbers were both 134, and their equality *was* the quadratic — one whole graph identity per coverage record, one record per operation, the product of a linear thing with a linear count. ADR 0104 replaced the per-record restatement with a fixed-width digest, so the per-record reference stopped scaling with the graph and the product is gone. The third column of the harness's structural decomposition shows it directly: the bytes one added operation adds to the whole coverage section are **3,301 at every step**, flat over all sixty consecutive steps, where under the restatement they climbed with `n`. **The 2026-08-08 run is the sharpest available demonstration of that separation**, because it moved the graph curve and left the coverage curve exactly where it was: `graph_bytes` gained `n` bytes under the `v2 → v3` extent tagging and the coverage section's per-step 3,301 did not move by a byte, which under the pre-fold encoding it could not have done.

ADR 0104 predicted this curve before the fold landed and the prediction was met exactly on the tree it was written for: it derived `3525n + 719`, the 2026-08-06 run measured `3525n + 727`, and the eight-byte constant gap is the `tiler.kernel-program.v11` staged-realization step added after that arithmetic was written. The **linear coefficient** was predicted to the unit and stayed there through three trees; it reads 3,531 here only because of the later `5n − 4` and `n + 1` steps.

### Why the observed exponent reads 0.98 and says nothing either way

A log-log least-squares slope over all sixty-one points is **0.9824**; over the thirty-one-point ladder it read **0.9745** and over the ten-point one **0.9559**. Over the *quadratic* encoding, on the same generator and a narrower ladder still, it read **1.0863**. Two encodings whose curves differ in degree produce exponents about 0.1 apart, all near one, because the exponent reports where the domain is rather than what the curve is: under the quadratic the linear term dominated everywhere a program could reach, and under the linear one the constant term still shifts the slope below one — **the three linear ladders creep 0.9559 → 0.9745 → 0.9824 toward one as the domain widens, without arriving.** Only the exact fit distinguishes them, which is why the harness fits the polynomial and prints the exponent beside it rather than reporting an exponent alone.

### The refusal point

**Extrapolation, labelled.** Solving the fitted curve against the bound, identity first exceeds `MAX_PROGRAM_IDENTITY_BYTES` at **n = 19,006 operations** (67,110,910 bytes at 19,006; 67,107,379 bytes at 19,005). The widest measured point — 62 operations, 219,646 bytes — is **0.327% of the bound**. It has moved twice and only because the slope moved: 19,038 → 19,011 under the `5n − 4` index-refinement step, then 19,011 → 19,006 under the `n + 1` extent tagging. The widening of the ladder itself moved it not at all, because every new point landed on the fitted line rather than beside it.

**The extrapolation has no out-of-domain confirmation, and the run that widened the ladder to sixty-one points removed the last one rather than adding any.** Under the quadratic encoding the retained 2026-08-05 file recorded the nine-operation point as a *confirmed wall* — `semantic_operations` was 8 there — so that fit was never checked outside 2..=8 at all. Under the linear encoding the checks were the eleventh point and then all of 12..=32, and both sets are now inside the domain; the thirty points that run added were not predictions of the fitted line, because the line moved under them. **There is no program this compilation path admits at this extent that the ladder does not already contain**, so no further confirmation is obtainable along this axis without moving `semantic_operations`. The refusal point is an extrapolation across nearly three orders of magnitude with nothing outside the ladder to check it against.

**Measurement — the other axis is not a confirmation either, and it says why.** The wall table's control compiles a two-operation chain at extent 268,435,456 and its program identity is **7,796 bytes** against the ladder's 7,786 at the same operation count: ten bytes wider for an extent nine orders of magnitude larger, the same ten as under the previous encoding. That is the `EXTENT` note read off a measurement rather than asserted — extent enters the identity as a handful of bytes per value, so it moves the fitted **constant** and not the slope. It therefore cannot confirm `3531n + 724`, because it is a point of a different one-parameter family; what it bounds is how little the held-fixed parameter matters to the curve this spike reports.

The fit is exact on its domain, and the domain is 2..=62 operations. Every coefficient is a property of this one program family: the per-operation slope depends on operation-key length, arity, result rank, attribute width, the region identity, the reached definitions, and the admission provenance. A richer family moves both coefficients. **The direction of that error is not neutral**: transformer families carry longer keys, wider attributes, and higher-rank results than a unary `f32` multiply, all of which *raise* the per-operation slope and therefore *lower* the refusal point. 19,006 is an upper-ish estimate of where the bound binds, not a floor — and the `5n − 4` and `n + 1` steps this ladder has now measured are two small worked examples of the slope moving for reasons nothing about the program family predicts. **The second is a worked example of the higher-rank clause specifically**: the extent tag is one byte per extent, so a rank-`r` result pays `r` where this family pays 1.

## Verdict: the margin holds by a wider margin, and the ceiling it holds against has changed

The ticket asked for one of two answers — a margin, or a follow-up decision ticket for a digest form. It got both, in that order: the 2026-08-05 quadratic gave a margin of ×125 and raised the digest question, that question became [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md), and the fold it decided is what this run measures.

**Fact — the roadmap's contemplated program sizes are per-layer, not per-model.** [Complete model ingestion and execution](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md#whole-model-composition-three-programs-thirty-executions-three-identities) proposes one forward pass as **three semantic programs executed thirty times**: P1, the embedding gather, at 1 operation; P2, the decoder layer, at **≥ 51 operations**, executed 28 times against one artifact identity; and P3, the final norm and vocabulary projection, at 2 operations.

Evaluating the curve at those sizes, against both consumers:

| Program | Operations | Identity | Share of the 64 MiB program bound | Margin | Fixed content at ×2 | Share of the 1 MiB embedding ceiling |
| --- | --- | --- | --- | --- | --- | --- |
| P1 embedding gather | 1 | 4,255 B (fitted) | 0.006% | ×15,772 | 8,510 B | 0.81% |
| P3 norm and vocabulary projection | 2 | 7,786 B (**measured**) | 0.012% | ×8,619 | 15,572 B | 1.49% |
| **P2 decoder layer** | **≥ 51** | **180,805 B (measured at 51)** | **0.27%** | **×371** | **361,610 B** | **34.5%** |

P1 sits one operation *below* the ladder's floor — the chain needs a multiply to make its constant output-reachable — so its figure is an extrapolation in the other direction. P3's is the ladder's own two-operation row.

**P2's row stopped being an extrapolation.** Fifty-one operations sat nineteen steps above the ladder's top this morning and its figure was solved from the fit; it is now a measured row, and the fitted and measured values agree because the fit reproduces every point to the byte. What that measures is *this* family at fifty-one occurrences, not the decoder layer: the chain is a unary `f32` multiply chain and the layer is not, so the coefficient caveat below is untouched and the comparison remains an inference about a different program with the same occurrence count. What changed is that the ladder no longer has to reach across a wall to state it.

**So the 64 MiB bound is unreachable for the program sizes this roadmap contemplates, with a margin of about ×371 in bytes and ×373 in operation count** (19,006 fitted refusal against P2's 51), where before the fold it was ×125 and ×13.6. The margin is robust to the coefficient being wrong in the unfavourable direction: for the bound to bind at 51 operations the per-operation slope would have to be **1,315,845 bytes rather than the measured 3,531**, a 373× increase that no plausible widening of operation-key length, arity, or attribute width produces.

### What the fold changed about the contingency, and it is the interesting half

**Inference — a whole-model program now fits the program-identity bound, and the argument against fusing across layers has moved to a different ceiling.** [The transformer operation and shape surface derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md#occurrence-inventory-for-one-forward-pass) inventories one Qwen3-0.6B forward pass at **≥ 1,068 semantic occurrences**. Compiled as a single semantic program that is **3,771,832 bytes — 3.60 MiB, 5.6% of the 64 MiB bound**, where the pre-fold curve put it at ≈ 149 MiB and a hard typed refusal. It remains an extrapolation seventeen times the ladder's top, which the widening did not change.

That is a reversal of this spike's own previous verdict and it must not be read as a licence. **The per-layer partition is still load-bearing on size**, and the ceiling that says so is the per-invocation embedding one: at the post-[ADR 0103](../../../docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) envelope multiplicity of two, the same whole-model program's fixed content is **7,543,664 bytes against a 1,048,576-byte ceiling, 7.2× over** — and that ceiling has no typed refusal at the artifact layer at all, so its failure mode is an artifact that compiles and cannot be embedded. The same curve crosses it between **148 and 149 operations** (`2 × (3531·148 + 724) = 1,046,624`; `2 × (3531·149 + 724) = 1,053,686`), where before the fold it crossed between 50 and 51 — that is, *at* the roadmap's own decoder layer. **Neither slope move shifted that crossing**, which is the one place new coefficients could plausibly have changed a conclusion and twice did not: the crossing solves to 148.3 operations under `3530n + 723` and to 148.3 again under `3531n + 724`, along a line whose slope has moved by 0.17% in total since `3525n + 727`.

What survives unchanged from the 2026-08-05 verdict is its shape: the per-layer cut, grounded in that record on artifact-identity reuse and layer-count independence, has a second and independent size ground its own derivation never mentions. What changed is which ceiling supplies it, and how much room is left — P2 at 34.5% of the embedding ceiling rather than the 102% the pre-fold curve gave it at the same multiplicity of two (`2 × (134·51² + 3650·51 + 727) = 1,070,822`).

## Boundary

- **One program family** — a unary `f32` multiply chain over a rank-1 extent-4 tensor — one contract (`FLUSH_SUBNORMALS_TO_ZERO_F32`), one target profile (the authoritative macOS Apple9 declaration), `f32` only. Both fitted coefficients are that family's.
- **Sixty-one points, 2..=62 operations.** Not a sampling choice, and it *is* the governed budget: the ordinary compilation path refuses at sixty-three on `semantic_operations`, and the run proves that wall by compiling at it and requiring its class and its phase. Widening further is a budget decision, not a harness one.
- **The extent is bracketed, not swept.** The wall table compiles the two-operation chain at 268,435,456 and at 268,435,457 and requires the first to succeed and the second to refuse; nothing between 4 and the bound is measured, and no ladder column varies with extent. The bound itself is written down in the harness because no public accessor exposes the declared maximum, which is why the bracket exists — a measured row that moved in either direction fires one of the two probes.
- **The coefficients are pinned to a compiler tree, and the ladder has now watched them move twice.** `3531n + 724` describes base `cc667626` and nothing else. `3530n + 723` described bases `cee4fe1a` and `25e76d5d` identically, which is why reproducing across two trees bounds nothing about a third: base `cc667626` is that third tree and it moved. Before those, `3525n + 727` was measured on `d050f10a` plus a branch. The two displacements are exactly `5n − 4` and `n + 1`, so every ladder is recoverable from its successor by subtraction — a retained result remains a statement about the tree it names. [`results/README.md`](results/README.md) is the index.
- **The refusal point is an extrapolation across nearly three orders of magnitude, and it now has no out-of-domain confirmation and no way to obtain one.** Every wall that once supplied one fell, each confirmation became a ladder row, and the ladder now covers every program the path admits. It is the order of magnitude at which the bound becomes binding, not a number a caller may rely on.
- **The P2 and whole-model comparisons are inferences over a second document's inferences**, not measurements: nothing here compiled a transformer. Fifty-one occurrences is now a measured *row of this family*, which is not the same statement as measuring the decoder layer. Both source counts are explicit lower bounds (`≥ 51`, `≥ 1,068`), the 1,068-occurrence figure remains an extrapolation seventeen times the ladder's top, and the per-layer partition they rest on is a `Proposal` with `disposition: pending` rather than an accepted decision. The claim is that the numbers are separated by orders of magnitude, not that either program was observed.
- **The envelope multiplicity of two is imported, not measured here.** It comes from [the manifest-growth attribution](../../../docs/research/artifacts/manifest-fixed-content-growth.md), measured on one fixture for one landing's coverage increment; every embedding-ceiling figure above inherits that bound.
- **Compile-only.** No kernel was emitted, linked, or dispatched, so this spike makes no performance claim. The `compile_ms` column is reachability information — the minimum of the two runs behind each row, on a coordination host running several other agents' builds at load average 18 — and not a benchmark. It runs 1, 1, 1, 2, 2, 3, 6, 11, 22, 45 ms up to eleven operations, roughly doubling per step, then **falls to 6 ms at twelve and climbs smoothly to 20 ms at sixty-two**. The doubling is the connected-set enumeration, which is exponential in this family's fan shape; `region_expansions` (10,000) is what stops it, and it first binds at twelve. So the discontinuity is the budget engaging, and everything above it is bounded work growing gently with program size.
- **`CompileFailureClass::BudgetExhausted` carries no resource, limit, or actual value.** The sixty-three-operation wall's refusal also arrives before a target-qualified explain trace, so it can report *that* a budget refused but not *which*; attribution to `semantic_operations` is read from the source cited above rather than from the refusal. That gap is [`carry-the-exhausted-resource-through-the-budget-refusal`](../../../tickets/carry-the-exhausted-resource-through-the-budget-refusal.md). It no longer covers every wall here: the launch-geometry entry seals a trace whose `target.grid-axis` record carries `threads=268435457:268435456`, so that one attributes itself.
- **The enumeration behind "which programs refuse after planning" is sixty-six programs**, listed above with its outcomes. It is not a statement about semantic programs in general, about other families, or about other target profiles — the staged normalization refuses in a different phase under a different profile, which is the worked example of why.
