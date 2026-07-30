---
id: pin-the-admitted-unsafe-sites-in-the-workspace-gate
title: Decide whether the workspace gate pins admitted unsafe sites
status: deferred
priority: p3
dependencies: []
related: [record-the-case-by-case-unsafe-boundary, prototype-metal-runtime-execution]
scopes: [implementation/workspace, contracts/navigation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, workspace, gate, rust-api, deferred]
---
ADR 0079 permits unsafe code only at individually admitted sites. The current
compiler lint enforces that an unsafe block needs a local allow, but
`AGENTS.md` correctly states that no check inventories those allows. Review is
the only control over a newly added, moved, removed, or re-justified site.

A former Python workspace gate pinned each admitted
`(package-relative path, item signature, reason)` and had negative mutation
tests. That gate and its tests were deleted when repository verification moved
to the root `Makefile`; no implementation is currently in review.

## Deferred boundary

Keep the current review-only enforcement while the complete admitted population is two sites in one non-published prototype. A mechanical source scanner would add a second parsing authority to the gate before the production population exists, and the obvious grep-shaped implementation demonstrably misses both multi-line attributes.

## The admitted population today (2026-07-28)

The ticket asks Tom to choose an enforcement posture without saying what is being enforced over. It is **two sites, both in one non-published prototype**, and that changes how both options read.

Reproduce with `grep -rn --include='*.rs' -B1 '^    unsafe_code,' crates prototypes`, which returns exactly two matches:

| Site | `#[allow(` opens | `unsafe_code,` | `unsafe` block | Item |
| --- | --- | --- | --- | --- |
| `prototypes/serial-sum-run/src/buffer.rs` | `:35` | `:36` | `:52` | `pub fn write_f32` (`:39`) |
| `prototypes/serial-sum-run/src/buffer.rs` | `:67` | `:68` | `:85` | `pub fn read_f32` (`:72`) |

**There is no admitted unsafe site anywhere under `crates/`.** Both sites are in the one member permitted to diverge from the workspace `forbid` — `prototypes/serial-sum-run/Cargo.toml:39-41` declares `[lints.rust] unsafe_code = "deny"` with the reason stated in the manifest — and both meet ADR 0079's four conditions: `Buffer::contents` is the only route to `MTLBuffer` storage, each `#[allow]` carries a `reason`, each block is preceded by an `assert!` against the buffer's own `length()`, and each carries a `SAFETY` comment naming the invariant.

**And here is the fact that bears directly on the mechanical option.** `grep -rn --include='*.rs' 'allow(unsafe_code' crates prototypes | wc -l` returns **0**. Both attributes wrap across lines, so the obvious grep-shaped inventory matches *none of the population* and reports that cleanly — zero hits, exit non-zero, no error. A check written that way would say "no unadmitted sites" and "no sites at all" in exactly the same way it would say "the check did not run".

That is the hazard `AGENTS.md` states as **"a verdict is only as good as the check's ability to say no"**, and it is the same shape as the worktree survey that reported forty-three clean checkouts because `head` was unresolvable inside the loop, and as a `trybuild` glob that stops matching and reports a passing test having compiled nothing. **So it is a requirement on the mechanical option, not a caution:** the check must declare its expected population and count it, so that an empty inventory is a *failure* rather than a pass. A check that only looks for violations cannot distinguish zero violations from zero observations.

## The two options, with the population known

- **Review-only enforcement.** Permitted by ADR 0079, keeps the gate simple, and costs nothing to maintain. *Enables:* a new site is admitted by the same judgement ADR 0079 asks for — a human reading the diff that adds it, which is what "case by case" means. *Prevents:* nothing mechanically. A new allow, a moved one, a removed assertion, or a silently reworded `reason` relies entirely on diff review. **Two sites in one non-published prototype is the strongest available argument for this option, and the ticket currently hides it** by asking the question against an unstated and implicitly larger population.
- **Mechanical inventory.** *Enables:* the admitted population becomes explicit and machine-checked; a moved-plus-added pair cannot net out; the check can be made to prove its own failure path. *Prevents:* nothing about correctness directly — it prevents an *unreviewed* change to the population. *Costs:* a source-scanning authority in the gate, whose parsing boundary must be documented (the zero-hit grep above is the proof that the boundary is not obvious), and whose pin must be updated in the same change as any site edit.

## Recommendation

Restore the exact path, item signature, and reason inventory — **but the recommendation is now weaker than it was, and the re-derivation should be visible.** Against an unbounded population the argument is straightforward. Against **two** sites in a prototype that AGENTS.md already says is "rewritten or deleted as the slice they prove moves", the maintenance cost is a larger fraction of the benefit, and the honest summary is that this is a close call Tom could reasonably decide either way.

What still carries it: the permission is case-by-case, so a count alone is insufficient — moving one site while adding another must not pass, and only a path/signature/reason triple catches that. And the population is two *today*; `prototype-metal-runtime-execution` is where a third would arrive, and the moment to install an inventory is before the population grows rather than after.

If mechanical enforcement is selected, a **negative mutation test must prove the check can fail** for each of addition, move, removal, and reason change — run each mutation and watch it fail, rather than asserting the check compiles. And per the zero-hit fact above, the check must name and count its expected population so that finding nothing fails.

## Activation trigger

Reactivate before admitting the first production unsafe site, or when the admitted population grows beyond the two current prototype functions. At activation, derive the inventory mechanism from Rust syntax rather than a zero-observation grep, name and count the expected population, and demonstrate failure for addition, move, removal, and reason change. Tom reviews any resulting workspace-gate or unsafe-policy boundary.
