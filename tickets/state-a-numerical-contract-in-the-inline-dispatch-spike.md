---
id: state-a-numerical-contract-in-the-inline-dispatch-spike
title: State a numerical contract in the inline-dispatch spike's two regions
status: review
priority: p2
dependencies: []
related: [state-the-numerical-contract-in-the-region-grammar]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [spike, frontend, numerics, runtime]
claimed_from: todo
assignee: agent-inline-numerics
lease_expires_at: 1785685152
---
## User-visible outcome

`spikes/runtime/inline-dispatch` builds and runs again: both of its `tiler::tensor!` regions state the numerical contract the frontend now requires.

## Why this is a separate ticket

**Fact.** `state-the-numerical-contract-in-the-region-grammar` made a region that states no numerical contract a refusal at expansion. The spike's two regions — `dispatch_region` and `fallback_only_region` in `spikes/runtime/inline-dispatch/src/main.rs` — state none, so they no longer expand.

**Fact — no gate caught it, and none would.** `spikes/runtime/inline-dispatch` declares its own `[workspace]`, so `make check` and `make full` never reach it; AGENTS.md states that no `make` target reaches `spikes/`. The breakage is real and silent until someone runs the spike by hand from its own directory.

**Fact — why the landing ticket did not fix it.** `spikes/runtime/**` maps to the `research/runtime` scope, which that ticket did not hold and which live spike workers did.

## Implementation keys

- `dispatch_region` states `deliver macos;` and compiles ahead of time, so its contract must be one the bound macOS declaration honours: `flush_subnormals_to_zero_f32` preserves the artifact identity the spike's recorded run was taken under, and `flush_and_reassociate_f32` is the other admissible one. Anything else is a feasibility refusal at the `deliver` keyword.
- `fallback_only_region` states no `deliver`, so nothing checks its contract today; state the same one, and say in the comment beside it that the statement is inert on the fallback path — `check-the-stated-contract-on-the-semantic-fallback-path` owns that gap.
- Re-run the spike by hand per its README and confirm the recorded output still holds, including the artifact identity if the README pins one.

## Closes when

Both regions state a contract, the spike builds and runs from its own directory, and any recorded output or pinned identity it carries is re-verified rather than assumed.

## Outcome, 2026-08-02

**Measurement — the breakage was real, and it was observed rather than inferred.** `cargo run --release` from `spikes/runtime/inline-dispatch` at base `93e253d` exited `101` with two errors, one per region, at `src/main.rs:129` and `src/main.rs:149`: "this region states no numerical contract, so what its arithmetic means is undecided". Nothing was relying on a fallback; the spike simply did not build.

**Fact.** Both regions now state `contract flush_subnormals_to_zero_f32;`, placed after `deliver` and before `out`, which is the placement every fixture `state-the-numerical-contract-in-the-region-grammar` landed uses.

**Fact — the elimination has one survivor, so this was not a decision to escalate.** `strict_f32`, `reassociate_f32`, and `relaxed_f32` each require preserved input subnormals and are refused at the `deliver` target, which `crates/tiler-macros/src/aot/tests.rs`'s `the_bound_declaration_admits_the_two_flushing_contracts` pins. Of the admissible pair, `flush_and_reassociate_f32` authorizes ordered regrouping of a same-operation operand sequence and `(a * b) + c` is a pointwise chain containing none, so it would claim a freedom the program cannot exercise. Contraction stays forbidden under both, so the oracle's "not `mul_add`" argument does not turn on the choice.

**Measurement — the stated contract is an input to the kernel identity.** Stating `flush_and_reassociate_f32` on the delivering region produced entry symbol `tiler_kernel_f4013709b41a2116`; `flush_subnormals_to_zero_f32` produces `tiler_kernel_ae031ce7240f7495`, which is the symbol the README's transcript already pinned. Object length, binding count, launch, and every value were identical under both. So the identity preservation this ticket asked for is measured rather than assumed, and it is non-vacuous.

**Measurement — the transcript re-verified unchanged.** `cargo run --release` and `cargo run --release -- --halt-after-commit` both exit `0` on Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19`, and every recorded line is reproduced byte for byte. Boundary: one host, one OS build, one device; no performance claim.

**Fact — two new watched-failing rows.** The unstated-contract refusal above, and `contract strict_f32;` on the delivering region, which refuses at the `deliver` keyword with the compiler's target-feasibility diagnostic quoting `strict_f32` back. Both are recorded in the spike's README, which is what makes the statement load-bearing rather than decorative.

**Fact — the fallback region's statement is inert, and it says so.** `crate::expand` resolves the contract and then takes the branch that never calls `aot::deliver`. The comment beside the region and the README both name `check-the-stated-contract-on-the-semantic-fallback-path` as the owner, and that ticket is correctly `deferred` — none of its activation triggers fired here.

No defect was found in the landed region grammar, and no edit was made outside `research/runtime` and this ticket file.
