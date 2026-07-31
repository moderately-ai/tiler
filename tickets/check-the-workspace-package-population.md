---
id: check-the-workspace-package-population
title: Check the workspace package population beside the dependency-direction test
status: done
priority: p2
dependencies: []
related: [correct-stale-post-vertical-integration-inventories]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inventory, correctness]
---
## User-visible outcome

Adding or removing a workspace member fails a test that names the expected population, so the crate-count assertions scattered through `docs/` and `tickets/` have one authority that says no when they drift, instead of each going stale silently until a reader happens to recount.

## Why this ticket exists rather than the check

`correct-stale-post-vertical-integration-inventories` was asked to add this check and could not place it in scope. That ticket held `contracts/integrations` and `implementation/metal-aot`; the natural home is `implementation/frontend`, which it did not hold. It filed this rather than placing the check badly, and the reasoning is worth keeping because it decides where the check goes:

- **The home is `crates/tiler/tests/`, beside `dependency_direction.rs`.** That file already reads the workspace `Cargo.lock` for a workspace-wide structural property, and already states this ticket's discipline in its own words — "Without these two, 'no offending edge' would also be what an empty or misparsed lockfile reports, so the check has to name its population before it can be trusted to say no." A population check is the same idea applied to the population itself.
- **`tiler-metal-aot` was rejected as a home.** ADR 0077 item 2 pins that crate's dependency closure empty and its responsibility bounded to invoking `xcrun`; a workspace-wide failure site inside it would point the next worker who admits `tiler-candle` at the Metal AOT driver for an unrelated failure.

## Implementation keys

**Fact — the derived population on 2026-07-31 at `01363ef`.** `cargo metadata --no-deps` reports thirteen packages: eleven production crates — `tiler`, `tiler-artifact`, `tiler-build`, `tiler-cache`, `tiler-compiler`, `tiler-ir`, `tiler-macros`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime` — plus two prototype members, `tiler-prototype-compile` (at `prototypes/serial-sum-compile`) and `tiler-prototype-run` (at `prototypes/serial-sum-run`).

**Fact — `workspace_members` carries two ID forms, and a parse that handles one silently drops the other.** Verified by reading the output at the base above: a member whose package name equals its directory name renders as `path+file:///…/crates/tiler#0.0.0`, while one whose name differs renders as `path+file:///…/prototypes/serial-sum-compile#tiler-prototype-compile@0.0.0`. Both prototype members take the second form, so a parse that only reads the last path segment would report them as `serial-sum-compile`/`serial-sum-run` and a parse that only splits on `@` would drop the other eleven. Read the name from the `#` fragment when it is present and from the final path segment otherwise, and assert the count so a parse that yields nothing fails loudly.

**Do not add a JSON dependency to `tiler` for this.** `dependency_direction.rs` hand-parses the narrow grammar it needs rather than pulling a parser into the facade crate's graph, and the same applies here; `workspace_members` is a flat JSON array of strings, which is why it is the field to read rather than `packages[].name` (package objects also contain `targets[].name`, so a naive `"name"` scan over `packages` matches target names too).

**Assert both the names and the count**, not one of them: an expected list compared only by membership passes when the workspace grows, and a count compared alone passes when one member is swapped for another.

**Prove it can say no before landing it.** Remove one expected name, watch the check fail, restore. The filing ticket already ran this against the equivalent derivation and recorded the evidence; redo it against the real test rather than inheriting the claim.

## Closes when

A test in `crates/tiler/tests/` derives the workspace package names from `cargo metadata --no-deps`, asserts the exact expected set and its count, fails when an expected name is removed, and passes at head; `cargo nextest run -p tiler` and `make full` are green.

## Delivered (2026-07-31)

`crates/tiler/tests/workspace_population.rs` derives the member names from `cargo metadata --no-deps`'s `workspace_members` field, handles both ID forms, and asserts the exact thirteen-name set and its count with named missing/unexpected members in the failure message. The perturbation was run against the real test: dropping `tiler-macros` from the expected list failed naming the count and the derived population, and the restored list passes. Hand-parsed per the filing ticket's constraint — no JSON dependency enters the facade crate's graph.

## Graph maintenance

- Whoever admits `tiler-candle` under `prototype-candle-metal-adapter` updates the expected population in the same commit as the member — that is the intended failure, not an obstacle.
