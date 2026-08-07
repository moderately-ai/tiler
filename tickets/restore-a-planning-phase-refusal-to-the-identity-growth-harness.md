---
id: restore-a-planning-phase-refusal-to-the-identity-growth-harness
title: Restore a planning-phase refusal to the identity-growth harness
status: todo
priority: p3
dependencies: []
related: [rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets, derive-the-region-shape-budgets-from-the-declaration]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, program-planning, evidence]
---
## User-visible outcome

`spikes/program-planning/identity-growth`'s `--perturb=program` mode watches a compilation that **verified, planned, and reached no kernel program** abort the sweep, rather than only a compilation the request-verification gate refused before any target compiles.

## Why this exists

**Fact, 2026-08-07.** The mode takes its program from `WALLS` rather than writing one down, deliberately: its predecessor was a hand-written reverse-axis `tiler::reindex-f32@1` whose justification expired silently when all six `ReindexFormKind` arms became recognized, at which point the perturbation stopped perturbing while its exit code stayed 1. Reading the point out of the wall table means the same run also probes it, so the mode cannot silently stop testing what it says it tests.

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
