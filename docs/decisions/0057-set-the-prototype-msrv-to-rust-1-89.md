---
schema: "tiler-doc/v1"
id: "ADR-0057"
kind: "decision"
title: "Set the prototype MSRV to Rust 1.89"
topics: ["rust", "msrv", "cache", "toolchains"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture", "tiler.contract.frontend-integration"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv"]
ticket: "prototype-foundation-contract"
---

# 0057: Set the prototype MSRV to Rust 1.89

**Status:** accepted

## Context

The expansion cache requires stable cross-process advisory file locking. Rust
1.89 is the first release with the complete standard `File` locking API. An
older MSRV would require another dependency or a platform adapter whose crash,
descriptor, and interoperability behavior would need separate auditing.

## Decision

All initial workspace packages use Rust 2024 and declare
`rust-version = "1.89"`. CI checks this floor. Cache locking remains behind an
internal adapter even though its first implementation uses `std::fs::File`.

Stable proc-macro APIs remain mandatory. Nightly tracked-input APIs are not a
reason to raise or weaken the toolchain contract; external Xcode changes retain
the documented rebuild requirement.

## Consequences

- The prototype uses a standard-library lock primitive matching the accepted
  cache protocol.
- Rust 1.88-and-older consumers are unsupported initially.
- A later audited lock adapter may lower the MSRV without changing semantic,
  artifact, or cache identities.
- A future MSRV increase remains an explicit compatibility decision.

## Alternatives considered

Supporting an older compiler with a locking crate offers wider compatibility
but adds correctness surface before the prototype has demonstrated demand.
Using the latest stable compiler would unnecessarily make unrelated new APIs
part of the compatibility floor.

## Traceability

The [workspace research](../research/workspace/prototype-crate-layout-and-msrv.md)
records the official Rust stabilization evidence and compatibility tradeoff.
