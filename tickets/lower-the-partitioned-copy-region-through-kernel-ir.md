---
id: lower-the-partitioned-copy-region-through-kernel-ir
title: Lower the partitioned-copy region through kernel IR
status: todo
priority: p1
dependencies: [admit-the-partitioned-copy-scheduled-region, admit-an-explicit-non-arithmetic-region-and-delivery-state]
related: [plan-concatenate-through-one-partitioned-copy-entry]
scopes: [implementation/ir, implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, kernel-ir, concatenate, ownership, verification]
---
## Outcome

The canonical partitioned-copy schedule lowers to one verified KIR with one entry and no arithmetic operations. Each output coordinate is supplied by exactly one ordered member, with bounds and ownership linked to the scheduled proof.

## Required delivery

Prefer existing index, comparison, predicate, load, and store operations where their exact semantics suffice. If the canonical body uses one predicated store per member, add a dedicated total verifier arm proving the predicates mutually exclusive and exhaustive and tying every store to the one joint ownership witness. Do not relax the generic `stores == 1` rule or accept an arbitrary multi-store program.

One buffer binding serves each distinct source plus the output; member records reference bindings, so `concat(x, x)` has one source binding and two members. No arithmetic node, unguarded store, extra store, missing member, reordered member, or unstated source-selection fallback is admitted.

## Closes when

Scheduled and KIR identities bind every ordered member and proof; canonical-body equality remains the final check; missing/extra/unguarded-store and wrong-member perturbations fail separately; and one verified KIR covers all admitted arities within structural bounds.

## Coordinator note — 2026-08-23: a fifth wall that opens by itself, and a name collision to avoid

Recorded here rather than only in the packet it was found in, because this is where the work happens and a dependency edge is not a guarantee that anyone reads across it. Found by `worker-nonarith` while re-deriving [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md), and **verified independently by the coordinator** at `e10f64a0`.

**`verify_entry` in `crates/tiler-compiler/src/program.rs` refuses a copy stage only as a side effect, and will start admitting one silently.** It computes

```rust
let numerical_matches = match scheduled.region().index.program.numerical() {
    Some(numerical) => stage.kernel().numerical() == *numerical,
    None => false,
};
```

and then fails the shared `entry-contract` rule when `!numerical_matches`. For a copy region `numerical()` answers `None`, so the refusal is real today — but it is incidental. `grep -c 'PartitionedCopy\|BitPreservingCopy' crates/tiler-compiler/src/program.rs` returns **0**: this file never names the copy, and nothing here records that refusing one is its job.

**The consequence for this ticket.** When a copy kernel carries a copy classification, whatever makes `numerical()` answer `Some` for the copy arm — or changes this comparison — turns `None => false` into a *match*, and this wall opens **without anyone editing this file or reviewing the change**. Turn that `None` arm into an explicit arm comparison as part of landing the KIR carrier, and perturb it: a copy kernel reaching `verify_entry` must be refused by a rule that names the copy, not by an arithmetic contract that happens to disagree.

**A name collision that a migration would plausibly trip on.** The compiler's *publishing copy* is deliberately a `Numerical` identity-expression region and **not** a `RegionProgram::PartitionedCopy`, because a one-member partitioned copy would be a second spelling of it. Two different things in this codebase are called "copy". Giving the publishing copy a bit-preserving classification would hand it a guarantee it never asked for.

**Also unresolved and yours to settle with the packet, not around it.** Three sites in `crates/` — `kernel/error.rs`, `kernel/model.rs`, `kernel/lower.rs` — name this ticket as the accepted owner of the copy's kernel carrier, while the ticket this one **depends on** claims that arm in its own text. The carrier is double-claimed across a dependency edge; the packet flags it for Tom, so do not resolve it unilaterally.
