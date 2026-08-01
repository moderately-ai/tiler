---
id: strengthen-the-family-cfg-evidence-with-the-installed-cross-targets
title: Strengthen the family-cfg evidence with the installed cross-targets
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-cross-target
lease_expires_at: 1785562660
---
## User-visible outcome

The five-target family-`cfg` matrix is proved by real cross-target compilation rather than by `rustc --print cfg` inference, so the claim "a nonmatching target compiles the semantic fallback" rests on a build that ran.

## Why

**Fact.** `generate-cfg-gated-artifact-family-delivery` recorded this as out of reach: "`rustup target list --installed` reports `aarch64-apple-darwin` alone, and installing a target is a host-toolchain change reserved to Tom." Tom authorized the change on 2026-07-31 and the coordinator installed exactly `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-apple-ios-macabi`, and `x86_64-unknown-linux-gnu` (rust-std components only, via rustup, removable with `rustup target remove`). The blocked evidence should now be re-run per AGENTS.md's rule that once authorized, the exact resulting component is recorded and the blocked measurement re-runs.

## Work

Compile the delivery emitter's matching, nonmatching, and retained-diagnostic fixture shapes with `cargo check --target <t>` (or the trybuild equivalent if it admits a target override) for each installed target, assert the expected pass/fail per the recorded five-target matrix, and record the evidence beside the existing `rustc --print cfg` derivation in the family_cfg tests or the delivery record — whichever the existing evidence idiom favours. Note the boundary: `cargo check`-level evidence needs no SDK or linker; a full link is out of scope.

## Closes when

Each of the five targets has a real compilation outcome recorded agreeing with the matrix, and the evidence states its check-only boundary.
