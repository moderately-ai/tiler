---
id: restore-a-planning-phase-refusal-to-the-identity-growth-harness
title: Restore a planning-phase refusal to the identity-growth harness
status: in-progress
priority: p3
dependencies: []
related: [rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets, derive-the-region-shape-budgets-from-the-declaration]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, program-planning, evidence]
claimed_from: todo
assignee: worker-restore-a-pl
lease_expires_at: 1786137974
---
## User-visible outcome

`spikes/program-planning/identity-growth`'s `--perturb=program` mode watches a compilation that **verified, planned, and reached no kernel program** abort the sweep, rather than only a compilation the request-verification gate refused before any target compiles.

## Why this exists

**Fact, 2026-08-07.** The mode takes its program from `WALLS` rather than writing one down, deliberately: its predecessor was a hand-written reverse-axis `tiler::reindex-f32@1` whose justification expired silently when all six `ReindexFormKind` arms became recognized, at which point the perturbation stopped perturbing while its exit code stayed 1.

~~Reading the point out of the wall table means the same run also probes it, so the mode cannot silently stop testing what it says it tests.~~ **Struck; see §1 below.** It was false when written — `probe_the_walls` ran only after a successful `summarize`, so under this mode the wall table was never reached in either world. Making it true was the first deliverable, and it now is: `main` accumulates a verdict and every stage runs.

`unplannable_program` therefore selected the first wall with `reaches_planning: true`. After [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) and [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md), **the table holds exactly one wall and it refuses before planning**: 63 operations on `semantic_operations = 62`, raised at request verification with no target-qualified trace. The selection was changed to `WALLS.first()` so the mode still runs, and the coverage it lost is recorded in the harness's own doc comment and in the spike README's boundary section rather than left to be discovered.

Measured on base `cee4fe1a`:

```text
$ cd spikes/program-planning/identity-growth && cargo run --release -- --perturb=program
REFUSED at operations=2: the compilation batch refused: CompileFailure { class: BudgetExhausted, explain: "absent (refused before a target-qualified trace)" }
```

`explain: "absent"` is the evidence: the abort is the request-verification one.

## What is still covered and what is not

Covered: a refused compilation stops the sweep instead of leaving a gap in the ladder. That arm is the same code path in `main` either way.

Not covered: the later abort. A compilation that verifies the request, enters the target loop, seals a per-target trace, and still reaches no verified kernel program exercises strictly more of `compile_once` — `into_targets`, `into_parts`, and the `outcome.map_err` arm that reads a target-slot refusal's class and trace. Nothing in the harness exercises those failure arms now.

## What this ticket owes

- A point this build refuses **after** planning, for the governed profile and a program the wall table can also probe, so the mode keeps its no-standing-claim property. Whether one exists is the first question: every region-shape bound is now a derivation over the declaration, and the search bounds cost alternatives rather than plans.
- If one exists: a `WALLS` entry with `reaches_planning: true`, `unplannable_program` reselecting on the phase, and a rerun.
- If none exists: say so with the enumeration behind it, and decide explicitly whether the mode should keep a written-down program with a stated expiry condition or whether the phase coverage is genuinely unavailable under the governed profile. A written-down program is what the current design rejects, so choosing it needs a reason and a trigger.

## Explicit non-goals

Not moving any budget to manufacture a wall. Not weakening the rule that the perturbed program is cross-checked by the same run.

## Closes when

Either the harness watches a planning-phase abort again with the wall table cross-checking it, or the record states which programs were enumerated, that none refuses after planning under the governed profile, and what would change that.

## Repaired before dispatch, 2026-08-07 — three defects, one of them in the harness itself

Verified by the coordinator reading `spikes/program-planning/identity-growth/src/main.rs` and `crates/tiler-compiler/src/request.rs` in full, not relayed.

### 1. This ticket's own premise is false, and so is the harness comment it was copied from

Struck, from this ticket's body: "**Reading the point out of the wall table means the same run also probes it, so the mode cannot silently stop testing what it says it tests.**" The same sentence is asserted at `main.rs:723-726`.

`main.rs:338-343` reads:

```rust
if !summarize(&rows) { return ExitCode::FAILURE; }
if !probe_the_walls(&declaration, perturbation) { return ExitCode::FAILURE; }
```

`probe_the_walls` runs **only after** a successful `summarize`, so under `--perturb=program` the wall table is never reached — in *either* world. If the wall still refuses, `measure` errs and `main.rs:327-333` returns early. If the wall stops refusing — the regression this mode exists to catch — every row returns `Ok` with the same operation count, `exact_quadratic`'s consecutive-integer precondition fails, and `summarize` returns `false`. Both paths exit non-zero without probing anything.

**So the mode's documented verdict cannot say no.** "Each exits non-zero" is true in every reachable state, including the state where the perturbation has died. That is verbatim the predecessor failure this harness already records at `main.rs:718-722` — "a perturbation that stops perturbing while its exit code stays 1 is worse than none" — reintroduced one layer up.

**Fix it first**: accumulate a `bool` instead of returning early, so `probe_the_walls` runs regardless of `summarize`'s verdict. That is what makes this ticket's premise *true* rather than asserted, and it is a precondition for the rest of the work rather than a nicety.

### 2. The stated deliverable has an empty solution set

Struck: "a `WALLS` entry with `reaches_planning: true`". `Wall` carries only an operation count (`main.rs:128-147`) and `probe_the_walls` builds its probe as `chain_program(wall.operations)` (`main.rs:375`), so the table's entire vocabulary is the multiply chain — within which every point either compiles (2..=62) or refuses at request verification on `semantic_operations` (≥63). **No chain program refuses after planning**, so the deliverable as written cannot be satisfied.

The body's dichotomy — a `WALLS` entry versus a written-down program with an expiry — is therefore false. **Generalize `Wall` to carry a program constructor beside its class and phase.** That admits a second family while *keeping* the cross-check property intact, since the same run still compiles the same program.

### 3. Where to look, against this ticket's own contrary premise

Struck: "every region-shape bound is now a derivation over the declaration, and the search bounds cost alternatives rather than plans" — offered as grounds that no bound can refuse a plan.

`crates/tiler-compiler/src/request.rs:1002-1007` says the opposite for two of the three: region formation's attribution atom is a realization **stage** rather than an occurrence, and its live values include intermediates a staged law hands between stages, so "**both bounds still bind** on a program whose families realize region sequences". The collapse holds only where each occurrence is realized by *one* region. A staged family can exceed `region_members = 62` while `semantic_operations` still admits the program — a refusal raised inside the target loop, which is exactly `reaches_planning: true`.

That lead is more available than when this ticket was written: `13cb0664` (staged family reading a materialized intermediate) and `0b0b4bed` (recognizer widened past the f32 wall) both landed after its cited base.

### 4. Corrections to smaller claims

- **Re-measure rather than carry the numbers.** The quoted output is pinned to base `cee4fe1a`; six commits have since touched `crates/tiler-compiler`, `tiler-ir` and `tiler-build`, including BF16 fusion legality and the widened recognizer. The output is honestly labelled with its base, so this is staleness rather than a false claim.
- **The unexercised-path list is imprecise.** `into_targets` and `into_parts` *do* run on every successful ladder point (`main.rs:604-609`). Genuinely unexercised are the `outcome.map_err` arm (`main.rs:610-616`) and the three harness `ok_or_else` refusals (`main.rs:607`, `:630`, `:634`).

### 5. Name the enumeration domain

The closing condition's negative branch — "none refuses after planning under the governed profile" — names no domain, making it an unbounded universal over semantic programs, which AGENTS.md forbids claiming: measurements bound claims but do not prove unmeasured universals. **State the domain explicitly** (for example: the chain family over 2..=63, plus staged-reduction programs of *k* folds for an enumerated set of *k*) and report the population count, so "nothing ran" cannot read as "nothing refuses".

### 6. Scope gap, fixed

`shared_scopes` was empty while the work must edit this ticket file. `tkt guard`'s under-declaration check is file-authoritative with no self-ticket exemption, so the branch would have exited 6. `project/tickets` added.

## Outcome, 2026-08-07 — the positive branch fired

Base `25e76d5d911e830ddebadd813dbeecf3e546eba0`. Files changed: `spikes/program-planning/identity-growth/src/main.rs`, its `README.md`, `results/README.md`, a retained `results/2026-08-07-post-restored-planning-wall-apple-m4-max-macos27.0-26A5388g/growth.tsv`, and this ticket. No `crates/` file was touched, read-only as scoped.

### 1. The ordering defect is fixed, and the fix is demonstrated rather than asserted

`main` accumulates a verdict — `swept && summarized && walls_held` — and every stage runs whatever the ones before it decided. The ladder loop `break`s on a refused point instead of returning.

**Demonstrated by deliberate failure**, with the perturbation killed at its source (`chain_past_the_grid_axis_bound` built at the bound rather than one past it, so the mode's program compiles) and one variable changed between the two runs:

| Run | Ordering | Exit | stderr | Wall section |
| --- | --- | --- | --- | --- |
| A | accumulated (fixed) | 1 | `THE WALL MOVED at the launch geometry bound: 2 operations compiled to a 7793-byte identity where NoFeasiblePlan was required … If this is the entry --perturb=program reads, that mode has stopped perturbing and its non-zero exit means nothing until this is fixed.` | present, names the moved entry |
| B | early-return (pre-fix) | 1 | **empty, 0 bytes** | **absent** |

Both print 61 ladder rows and both exit 1. Under B the only signal is the fit refusing a degenerate ladder — indistinguishable from a genuine encoding change, and exactly the state in which the documented verdict "each exits non-zero" is true while the mode tests nothing. Under A the same run names the dead perturbation. Both demo edits were reverted; the committed tree carries neither, and a fresh sweep reproduces the retained file on every column but `compile_ms`.

### 2. `Wall` carries a program constructor, and the deliverable's solution set is no longer empty

`Wall` is now `{ subject: WallSubject, program: fn() -> SemanticProgram, control: Option<fn() -> SemanticProgram>, class, reaches_planning, why }`. The operation count is read off the built program instead of restated beside it. `unplannable_program` selects `WALLS.iter().find(|w| w.reaches_planning)` and panics if none exists, rather than falling back to `WALLS.first()`.

`control` is new and load-bearing: a probe that must **compile**, so a refusal recorded beside it cannot be a boundary refusing everything.

### 3. What refuses after planning — and it is not what §3 above predicted

**The `region_members`-on-a-staged-family lead does not reach a program refusal, and the source says why.** `region.rs:2104` reports an over-budget member count as a `RegionRejection::Budget`, which *drops the candidate*; `whole_program_candidate` is then absent and the program still compiles whenever any other cover is implementable. For the one family whose only implementable cover is the whole-program region — this chain — members equal occurrences, so `check_program_budgets` refuses at 63 before region formation runs. The route needs a program whose only implementable cover is a >62-stage region, and nothing in this build's vocabulary builds one.

**The staged normalization does refuse after planning, but not under this spike's profile.** `rms_norm(matmul(a, b), a)` refuses `NoFeasiblePlan` after planning against `TargetProfile::governed()`, which `crates/tiler-compiler/tests/staged_family_over_a_materialized_intermediate.rs` asserts. Against `BoundMetalCompileDeclaration::first_macos_apple9()` — the profile this spike compiles at — it refuses `UnsupportedCapability { rule: "accuracy.elementary.no-installed-realization" }` with **no trace**, two phases earlier, and so does its declared-input control. That profile installs no elementary realization for the normalization at all. A wall this spike cannot probe under its own profile is not a wall it may record.

**What does refuse after planning is the generator's own second parameter.** The ladder sweeps operation count and holds extent fixed at 4; the extent has a domain of its own, and its top is a measured hardware row:

```text
#   launch geometry, at 2 operations: CONFIRMED NoFeasiblePlan after planning — … [the target slot refused: TargetCompileFailure { failure: CompileFailure { class: NoFeasiblePlan, explain: "26 records" }, refusal: None }]
#   launch geometry, control: CONFIRMED the program one step inside the recorded bound compiles, to a 7793-byte identity, …
```

The sealed trace names it exactly: `rule=target.grid-axis@1 … event=feasibility:grid-axis:rejected:target-infeasible:threads=268435457:268435456`. This is the arm the ticket wanted — recognized, covered, offered to the physical provider, declined — and it is the only run that reaches `compile_once`'s `outcome.map_err` arm. **It cannot dissolve the way its two predecessors did**: `region_expansions` and `region_members` were compiler-internal ceilings removed by re-deriving a bound; this is a measured Apple row, and widening it means measuring a wider family.

### 4. Enumeration domain, population 66

| Population | Count | Outcome |
| --- | --- | --- |
| chain, 2..=63 operations at extent 4 | 62 | 2..=62 compile; 63 refuses `BudgetExhausted` before planning |
| chain, 2 operations at extents 1,024 / 65,536 / 16,777,216 / 268,435,455 / 268,435,456 | 5 | all compile |
| chain, 2 operations at extents 268,435,457 / 1,073,741,824 / 4,294,967,296 | 3 | all refuse `NoFeasiblePlan` after planning, each naming `target.grid-axis` with `threads=<extent>:268435456` |
| `rms_norm(matmul(a, b), a)` over two `[2, 2]` inputs, and its two-declared-input control | 2 | both refuse `UnsupportedCapability { rule: "accuracy.elementary.no-installed-realization" }` before planning |

All against the Apple9 declaration under `FLUSH_SUBNORMALS_TO_ZERO_F32`. Sixty-one run on every ordinary sweep and four more in every wall section; the last row ran once and is recorded rather than retained. **This bounds one program family over two parameters plus one two-occurrence staged family, under one contract and one profile.** It is not a statement about semantic programs, other families, or other declarations — the last row is the worked example of why the difference matters.

### 5. Re-measured, and nothing moved

Base `25e76d5d`, six commits past the quoted `cee4fe1a`. **All nine structural columns are identical at all sixty-one points**, compared column by column rather than through the fit; `program_bytes(n) = 3530n + 723` and `graph_bytes(n) = 134n + 149` both still reproduce every point to the byte. So the spike's verdict now describes two trees. Retained at `results/2026-08-07-post-restored-planning-wall-…/growth.tsv`; the run's own second full sweep reproduces every column but `compile_ms`.

One figure is new: the launch-geometry control compiles a two-operation chain at extent 268,435,456 to a **7,793**-byte identity against the ladder's 7,783 at the same operation count — ten bytes for an extent nine orders of magnitude larger, which measures the `EXTENT` note's claim that the held-fixed parameter moves the constant and not the slope.

### 6. Perturbation modes, each with the line that produced its exit code

| Mode | Exit | Diagnosis |
| --- | --- | --- |
| `--perturb=program` | 1 | stderr: `REFUSED at operations=2: the target slot refused: TargetCompileFailure { failure: CompileFailure { class: NoFeasiblePlan, explain: "26 records" }, refusal: None } | tiler-explain-v7 …` (whole 26-record trace) |
| `--perturb=coverage` | 1 | stderr: `REFUSED at operations=2: the selected alternative covers 2 semantic occurrences but the graph has 3 operations; …` |
| `--perturb=fit` | 1 | **stdout**: `# NO EXACT QUADRATIC FITS the measured program-identity curve, so no refusal point is stated. …` — stderr is empty by design; the fit's verdict belongs in the summary block beside the fit |
| `--perturb=wall` | 1 | stderr: `THE WALL CHANGED KIND at the program size bound, 63 operations: this table expects NoFeasiblePlan raised before planning, and the compiler refused with … BudgetExhausted …` |

All four now reach the wall table, which none did before under `--perturb=program`.

### 7. Corrections carried into the records

- The harness's unexercised-path list was imprecise as the repair section said. `into_targets` and `into_parts` run on every successful ladder point; the `outcome.map_err` arm is now exercised by the launch-geometry wall and by `--perturb=program`. The three `ok_or_else` refusals remain unexercised.
- `Refusal`'s claim that "the one surviving wall now refuses before any trace is sealed, so the split currently costs nothing" is corrected: a wall seals a 26-record trace again, and the two-sink split is load-bearing.
- The README's "there is no program this compilation path admits that the ladder does not already contain" is narrowed to "at this extent", because the control is such a program.
- `carry-the-exhausted-resource-through-the-budget-refusal` no longer covers every wall here: the launch-geometry entry attributes itself from its own trace.

### Not done, deliberately

No budget was moved. No `crates/` file was touched. The ticket is left `in-progress` for the coordinator to close.
