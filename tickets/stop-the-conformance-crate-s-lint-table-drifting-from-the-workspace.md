---
id: stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace
title: Stop the conformance crate's lint table drifting from the workspace
status: todo
priority: p2
dependencies: []
related: [carry-the-device-executed-value-proof-into-the-conformance-crate, decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [lints, maintainability]
---
## The risk this creates

`crates/tiler-conformance/Cargo.toml` **restates** the workspace lint table rather than inheriting it, because it needs `unsafe_code = "deny"` where the workspace sets `forbid` and a member cannot inherit a table and then relax one entry. Its own comment says so: "Mirrors the workspace, which this crate cannot inherit; see the note there."

That is the correct mechanism for the decision Tom made on 2026-08-07 — `deny` with named per-site allows, never a crate-level allow — and it has a cost the decision did not name: **this crate's lints can now drift from the workspace's silently.** A lint added, tightened, or removed workspace-wide reaches every other member and not this one, and nothing fails when they diverge.

`AGENTS.md` already names the general hazard — "crates should inherit workspace Rust and Clippy lints; inspect `[lints]` changes because inheritance is not enforced" — which is guidance to a reviewer rather than a check. This crate is the first member that *cannot* inherit, so it is the first place that guidance has no fallback.

## What this owes

A mechanism, or a recorded decision that none is worth its cost. Candidates, none settled:

- **A test that compares the two tables** and fails when they diverge on anything except the one entry deliberately different. It would have to read both manifests, which is a text-parsing test of the kind `dependency_direction.rs` and `workspace_population.rs` already establish precedent for — both hand-parse rather than take a dependency, and both exist because "review catches it only for as long as someone reviews."
- **Invert the exception**: set the workspace to `deny` and have every *other* member forbid it. Larger blast radius, and it weakens the default for members that should never contain unsafe — probably wrong, but it is the alternative that removes the divergence rather than watching it.
- **Record the divergence and accept it**, with the one intended difference named at both sites so a reader of either finds the other. Cheapest, and legitimate — but it should be a decision rather than the default that happens by not choosing.

Whichever lands, the **one intended difference must be stated at both ends** — the workspace table and the crate's — so neither reads as an oversight.

## Explicit non-goals

Do not change the crate's `unsafe_code = "deny"`; that is Tom's decision and this ticket implements nothing about unsafe. Do not relax any other lint to make the tables match — matching by weakening is the failure this exists to prevent.

## Closes when

Either a divergence between the two tables fails a check, or the divergence is recorded as accepted with the reason and the single intended difference named at both ends.
