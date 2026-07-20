---
schema: "tiler-doc/v1"
id: "ADR-0053"
kind: "decision"
title: "Gate artifact delivery and failures by consumer family"
topics: ["proc-macros", "apple-targets", "fallback", "cross-compilation"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.frontend-integration", "tiler.contract.metal-backend"]
evidence: ["tiler.research.macro-environment.build-environment", "tiler.research.apple-targets.compatibility"]
ticket: "macro-build-environment"
---

# 0053: Gate artifact delivery and failures by consumer family

**Status:** accepted

## Context

Stable procedural macros do not reliably observe Cargo's consumer target.
Failing expansion immediately whenever a selected Apple compiler is unavailable
would break the same ordinary inline source on unrelated non-Apple targets.
Silently falling back on a matching Apple target would instead hide a broken or
missing requested acceleration path.

## Decision

`ArtifactFamilySelection` carries an explicit delivery policy. For each selected
family, expansion either embeds a completed payload or retains its
toolchain/compiler diagnostic. Generated Rust gates the payload or diagnostic by
the family's versioned consumer-target `#[cfg]` predicate. A matching target
requires the selected artifact and sees `compile_error!` on build failure; a
nonmatching target uses the semantic fallback.

An unselected family intentionally uses fallback. `FallbackOnly` is an explicit
valid policy and invokes no backend compiler. Target-neutral semantic,
optimizer, verifier, and envelope failures remain unconditional compile errors.

## Consequences

- One inline invocation remains portable without proc-macro target discovery.
- Non-Apple native builds do not require Metal solely because source also
  supports macOS.
- Cross-building a selected Apple family on an incapable host fails with a
  source-spanned actionable diagnostic.
- Missing selected acceleration cannot silently ship as fallback on its target.
- A capable macOS host may perform selected-family work while building an
  unrelated target; the content cache bounds repeated cost.
- Family-to-`cfg` mappings become versioned generated-code contracts.

## Alternatives considered

Host-target inference is incorrect for cross compilation. Unconditional family
failure breaks unrelated targets. Opportunistic fallback on a matching target
hides broken generated code. A required build script or prepare command would
violate the accepted inline workflow.

## Traceability

Applies to frontend delivery and the Metal backend. Macro-environment and Apple
target evidence define the measured family boundary.
