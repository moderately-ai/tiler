---
id: move-the-runtime-semantic-validation-cells-for-f32-and-bf16
title: Move the runtime semantic validation cells for f32 and BF16
status: review
priority: p3
dependencies: [validate-bf16-at-the-runtime-routing-boundary]
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, dtype, bf16, runtime, maturity-matrix]
claimed_from: todo
assignee: agent-navigation-5
lease_expires_at: 1786039087
---
## User-visible outcome

`docs/dtype-support.md`'s `Runtime semantic validation` column stops reading `absent/unsupported` for the two dtypes that now have a runtime refusal, and says exactly how far that refusal goes.

## Why this is a separate ticket

**Fact.** `validate-bf16-at-the-runtime-routing-boundary` holds `implementation/runtime` and could not edit `docs/dtype-support.md`, which `ticketsplease.toml` maps to `contracts/navigation`.

**Fact.** The mechanism that ticket added is dtype-neutral: `ExecutionEnvironment::classify_dtype` is keyed by `ArithmeticType` and the eligibility filter resolves whatever arithmetic an entry records. Its suite exercises `f32` on all three declared families as well as BF16, so both rows are supported by the same evidence rather than one being inferred from the other. Its own graph-maintenance note required the `f32` row be filed separately rather than claimed, which is this ticket.

## Implementation keys

- The cell states a **bounded** guarantee. What is tested is refusal at the routing boundary — an undispatchable or unmeasured dtype filters its variant before ADR 0051's commit, and the two refusing resolutions are distinguishable. It is not evidence about execution, and BF16's `Backend execution` cell stays `absent/unsupported`.
- Whatever wording the cell takes, the same wording covers both rows or the difference between them is stated.
- Check the prose section the cells link to for sentences that the change makes false, and correct them in the same edit. Nothing validates this corpus.

## Closes when

Both cells state what is now true with their boundary named, the linked prose agrees with them, and no sentence elsewhere in `docs/dtype-support.md` still describes runtime dtype validation as absent.

## Outcome

**Both cells now read `tested guarantee, dtype refusal at the routing boundary only`**, identical wording because the mechanism is one dtype-neutral filter, with the evidence difference between the two rows stated at the `f32` anchor rather than smoothed over.

**Fact — the mechanism, read at source rather than from the source ticket's summary.** `ExecutionEnvironment::classify_dtype` (`crates/tiler-runtime/src/load/host.rs:178`) is keyed by `ArithmeticType` and branches on no width; `variant_eligibility` (`crates/tiler-runtime/src/load.rs:674`, the dtype arm at `:728`–`:737`) resolves every entry of every packaged variant through it inside `select_variant` (`:605`), so an undispatchable width is filtered before any applicability guard and before ADR 0051's commit.

**Correction to this ticket's own premise, which overstated the `f32` evidence.** The "Why this is a separate ticket" section says the suite "exercises `f32` on all three declared families as well as BF16, so both rows are supported by the same evidence". Exercising `f32` on all three families is `an_f32_route_is_unaffected_by_every_bf16_verdict` — a **neutrality control**, in which `f32` always routes. The route-level *refusal* cases are BF16's only, because every fixture family in that suite declares `f32` dispatchable. What `f32` carries in the refusal direction is the classification over `ArithmeticType::ALL` (`an_undeclared_dtype_is_unknown_and_refuses` at `crates/tiler-runtime/src/load/host.rs:325`, `a_silent_host_dispatches_nothing` at `:354`), the second of which does include `f32` on a silent host. The cells still take the same wording — the cell claims what the filter guarantees, and the filter is one total function — but the anchor states the asymmetry, which is the implementation key's second clause taken rather than its first.

**Falsified-prose sweep in `docs/dtype-support.md`, six sites, each read in full before editing.**

- The non-monotone example under the dtype-addition recipe read "F32 has a tested backend and no runtime semantic validation". Replaced with BF16's tested routing refusal above an absent backend execution, which is a non-monotone pair this landing makes true.
- The `Other IEEE binary floats and BF16` opening said no "runtime-validation vertical exists for any of them"; narrowed to `f16`/`f64`/`f128` with the BF16 exception dated.
- The 2026-08-02 dispatchability paragraph's trailing "no … execution, or runtime-validation support" and the 2026-08-01 reference paragraph's "What did not move" list both named runtime validation as absent; each carries a dated in-tense correction rather than a rewrite.
- Two sentences named `validate-bf16-at-the-runtime-routing-boundary` as a live owner of device execution and the execution witness. It is `done` and dispatched nothing, so both now name `conform-the-bf16-vertical-end-to-end` alone and record that it is `blocked` — a closed ticket named as a live wall is the defect an earlier navigation audit already had to correct once.
- The `Strict-affine U4/F32` row ended "and no dtype dispatchability axis exists". False, and false before this landing. Corrected with the reason it moves no cell of that row: `ArithmeticType` is `{F16, Bf16, F32, F64}` (`crates/tiler-ir/src/schedule/numerics.rs:354`–`:367`), so no quantized scheme is declarable through the axis, and the row's actual gap — semantic value precondition enforcement — is untouched.

**Fact — one reproducible negative check was broken in two ways and is repaired.** `# No dtype-family dispatchability axis exists.` searched `crates/tiler-compiler/src/{feasibility.rs,request.rs} crates/tiler-runtime/src`. `crates/tiler-compiler/src/feasibility.rs` has not existed since `5e0193c8` gathered the target-description authorities under `target/`; `rg` reports the missing path on stderr and searches the rest, so the check read as empty for the wrong reason before it read as non-empty for the right one — the exact "a check that cannot say no" shape AGENTS.md names. The path is corrected to `target.rs`/`target/` and the check is relabelled as a reporting check naming what it now finds. The summary sentence below the block said "the four expected-empty checks"; two are expected-empty today and both were re-run empty (`StrictAffineU8|U8::resolved_type` and `semantic_precondition|SemanticPrecondition`, each exit 1).

**Commands run** (from the worktree root, docs-only branch, no cargo):

```sh
rg -n 'DType|dtype.*dispatch|dispatch.*dtype' crates/tiler-compiler/src/target.rs crates/tiler-compiler/src/target crates/tiler-compiler/src/request.rs crates/tiler-runtime/src   # non-empty, 5 files
rg -n 'semantic_precondition|SemanticPrecondition' crates/tiler-runtime/src crates/tiler-artifact/src/program                                                                        # empty, exit 1
rg -n 'StrictAffineU8|U8::resolved_type' crates/tiler-ir/src/{schedule,kernel,program} crates/tiler-artifact/src/program crates/tiler-metal/src crates/tiler-compiler/src            # empty, exit 1
```

**Boundary.** No cargo check was run and none is required: the branch touches `docs/` and `tickets/` only, neither of which is a gate input. Every claim written into the ledger was read at its construction site; no claim rests on the source ticket's summary alone.
