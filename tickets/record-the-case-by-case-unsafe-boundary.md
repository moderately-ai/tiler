---
id: record-the-case-by-case-unsafe-boundary
title: Record the case-by-case unsafe boundary as an accepted decision
status: in-progress
priority: p1
dependencies: []
related: [prototype-metal-runtime-execution]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, rust-api]
claimed_from: todo
assignee: agent-decisions
lease_expires_at: 1784996298
---
`AGENTS.md` states that unsafe code "remains forbidden unless an accepted decision changes that boundary". Tom changed it on 2026-07-25 and no accepted record says so, which is exactly the drift that sentence exists to prevent.

**Fact — what was decided.** Unsafe is permitted **case by case**, isolated to specific functions or modules, explicitly **not** whole crates and **not** a workspace relaxation.

**Fact — how it is realized at `a56bff8`.** `prototypes/serial-sum-run` does not inherit `[workspace.lints]` and declares `unsafe_code = "deny"` instead of the workspace's `forbid`, because `forbid` cannot be relaxed by an inner attribute at any scope. Two functions in `buffer.rs` carry `#[allow(unsafe_code, reason = ...)]`; they are the complete extent of unsafe code in the workspace. The necessity is structural rather than convenient: `MTLBuffer` storage is reachable only through the raw pointer `Buffer::contents` returns, and no Metal binding exposes it safely.

**Fact — the exception is pinned, not merely permitted.** `scripts/check_workspace.py` carries `UNINHERITED_LINT_MEMBERS`, naming the one member allowed to diverge and its exact table. A second crate dropping inheritance fails the gate, and so does widening that member's `deny` to `allow`.

## What this ticket produces

An accepted ADR recording the decision, its scope, and the mechanism, plus the `AGENTS.md` amendment its own sentence requires. The record should state the three properties that make the exception safe — narrowest-scope opt-in, `deny` rather than `allow` at crate level, and a pinned gate check — and the rule for future sites: an unsafe block is admitted only where a foreign API makes it structurally unavoidable, carries a `reason`, is preceded by an assertion bounding what it touches, and has a `SAFETY` comment naming the invariant it relies on.

**Do not let this become a general licence.** The decision is case-by-case, and the record must make a future reader ask again rather than cite this as precedent for a third site.
