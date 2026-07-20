---
schema: "tiler-doc/v1"
id: "ADR-0045"
kind: "decision"
title: "Bound inline proc-macro providers to host dependencies"
topics: ["extensions", "proc-macro", "rust"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "spike-only"
applies_to: ["tiler.contract.operation-extensions"]
evidence: ["tiler.research.extensions.proc-macro-extension-visibility"]
ticket: "proc-macro-extension-visibility"
---

# 0045: Bound inline proc-macro providers to host dependencies

**Status:** accepted

## Context

Tiler exposes a public operation registry and also requires independent inline
proc-macro AOT compilation. A stable proc macro is compiled as a host crate
before expanding tokens in its consumer. It cannot reflect an arbitrary
consumer-local type into its own process or add a reverse dependency on the
consumer without creating a Cargo cycle.

Deferring provider behavior until generated target code runs is too late for
expansion-time optimization and artifact generation.

## Decision

An inline Tiler proc macro constructs its frozen operation registry from
providers statically linked into the proc-macro host dependency graph and from
complete canonical semantic declarations visible in the invocation tokens.

Tiler built-ins and officially bundled providers use the public registration
API through that dependency graph. Cargo features may select optional providers
only when the macro package already declares those provider dependencies.

Arbitrary consumer-local Rust provider callbacks remain supported through the
ordinary compiler API but are not automatically visible to a separately
compiled inline macro. A future provider-specific macro wrapper may link the
provider while preserving the same inline AOT and artifact contracts. Source
scanning, consumer build scripts, registries, prepare commands, and runtime JIT
are not introduced as workarounds.

## Consequences

- The inline workflow remains ordinary and self-contained for supported
  operations.
- One macro binary has a build-time-bounded provider universe, but semantic IR
  and the public compiler API remain extensible.
- Official operations exercise the same public registry path as third-party
  compiler-API operations.
- Missing macro-side providers fail during semantic admission with explicit
  provider-set context.
- Cross-target environment measurement remains separate from this dependency
  result.

## Alternatives considered

Consumer linker inventories do not populate the host macro process. Passing a
type path gives the macro tokens, not executable consumer metadata. Reverse
dependencies are cyclic. Source scanning and auxiliary preparation violate the
accepted DX, and runtime invocation misses the AOT planning phase.
