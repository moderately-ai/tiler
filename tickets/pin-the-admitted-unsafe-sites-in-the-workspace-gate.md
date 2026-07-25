---
id: pin-the-admitted-unsafe-sites-in-the-workspace-gate
title: Pin the admitted unsafe sites in the workspace gate
status: todo
priority: p2
dependencies: []
related: [record-the-case-by-case-unsafe-boundary, prototype-metal-runtime-execution]
scopes: [implementation/workspace]
shared_scopes: []
paths: []
tags: [implementation, workspace, gate, rust-api]
---
ADR 0079 admits unsafe code case by case at an individual function or module, and records that exactly one half of that boundary is mechanically checked. This ticket closes the other half.

**Fact — what the gate pins today.** `scripts/check_workspace.py` carries `UNINHERITED_LINT_MEMBERS`, a single-entry map from `tiler-prototype-run` to the exact `[lints]` table it may declare instead of inheriting `[workspace.lints]`. It is consulted twice: `expected_member_manifest` substitutes it for `{workspace = true}` in the full manifest comparison, and a second explicit comparison reports the lint table on its own. So a second member dropping inheritance fails the gate, and so does widening that member's `unsafe_code = "deny"` to `"allow"`, adding a lint to its table, or removing one.

**Fact — what nothing checks.** No check counts, locates, or constrains `#[allow(unsafe_code, reason = ...)]` attributes inside the crate permitted to have them. At `43f685f` there are two, both in `prototypes/serial-sum-run/src/buffer.rs` on `write_f32` and `read_f32`. A third added anywhere in that crate compiles and passes `uv run --locked python scripts/check_repository.py` unchanged. ADR 0079's item 2 third property is a claim about which *crates* may diverge; it is not a claim about sites, and the ADR says so in its Consequences rather than leaving a reader to assume the check exists.

**Why it matters more than the crate half.** ADR 0079 is deliberately a case-by-case permission: a third site is a new decision, not an application of the existing one. That rule is currently enforced by review alone, and review is exactly what a gate is for when the predicate is mechanical. The crate half — the part a reviewer would notice anyway, because it changes a manifest — is the half that is already checked.

## The design question this ticket must answer

What the check pins is not obvious and should be decided by writing it, not assumed here. At least three predicates are available and they fail differently:

- **A count.** Cheapest, and it fails on the wrong thing: moving a site from one function to another passes, and adding a site while deleting an unrelated one passes.
- **File-and-item pairs.** Names each admitted site as `(path, item)`. Fails closed on an addition and on a move, at the cost of a rename churning the pin. Needs a source scan rather than a manifest read, which is a new capability for this script.
- **The attribute text.** Strongest — it pins the `reason` string too, so weakening a justification is a gate failure. Most brittle to reformatting; `rustfmt` owns the wrapping of a long `reason`.

Whichever is chosen, decide and record whether the check reads Rust source textually or parses it, and what it does about `#[allow(unsafe_code)]` appearing inside a string, a comment, or a `#[cfg]`-disabled item. A textual scan that cannot distinguish those is not obviously wrong for this repository — the whole universe is one crate and twelve grep hits — but the limitation must be stated in the script rather than discovered later.

## Closes when

`scripts/check_workspace.py` fails when an `#[allow(unsafe_code)]` site is added, moved, or removed without updating its pin; a mutation test proves that failure the way the script's existing checks are proven; ADR 0079's Consequences bullet naming this gap is amended to record that it is closed (that edit is `contracts/decisions` and needs the scope added or a split); and the full gate passes.
