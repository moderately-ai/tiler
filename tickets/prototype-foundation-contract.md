---
id: prototype-foundation-contract
title: Resolve prototype crate graph and MSRV
status: done
priority: p0
dependencies: [research-readiness-gate]
related: []
scopes: [contracts/foundation]
shared_scopes: [contracts/decisions, contracts/navigation, project/tickets, research/workspace]
paths: []
tags: [implementation, prototype, architecture]
---
Select and record the smallest workspace/crate dependency graph and Rust MSRV for the authorized serial-Sum Metal value proof. Preserve backend/frontend neutrality, identify what remains a module versus crate, make cache-locking implications explicit, update Q-PKG-001/Q-PKG-004, and create dependency-ordered implementation tickets only after these prerequisites are accepted.

## Outcome

- ADR 0056 selects four reusable libraries and two non-published proof
  executables, mechanically separating runtime from optimizer internals.
- ADR 0057 selects Rust 1.89 and edition 2024 so the accepted cache protocol can
  use stable standard-library advisory locking behind an internal adapter.
- The supporting research records dependency alternatives, official Rust
  stabilization evidence, compatibility costs, and future split seams.
- Q-PKG-001 and Q-PKG-004 are resolved; implementation tickets are ordered from
  workspace scaffolding through semantic, planning, Metal, and runtime proof.

## Supersession note

ADR 0067 subsequently superseded ADR 0057's stable-only toolchain choice while
preserving its advisory-locking evidence. The exact dated-nightly migration is
owned by `spike-nightly-arbitrary-rank-shape-evidence`.
