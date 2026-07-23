---
id: prototype-public-compiler-api
title: Implement the reviewed public compiler boundary
status: todo
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, dx]
---
Implement ADR 0069's consumer-agnostic CompilationRequest, session/provider
inputs, checked compilation result, stable diagnostics/explain, and ordinary
call-site ergonomics over the verified pipeline. Tom reviews consequential
public crate, trait, type, and call-site boundaries before acceptance. Frontends
consume this API; backend feasibility components need not depend on it.

## Inherited explain review agenda

The merged typed-explain implementation deliberately kept its module private and
raised eight public-surface questions. Tom settled the first on 2026-07-23:
explain stays a compiler-owned module, with the vocabulary moving into
`tiler-ir` only if a second crate must read traces (tracked by
`record-explain-ownership-decision`). The remaining seven are deferred to this
ticket because they all concern a public surface that only this boundary
introduces. Settle each explicitly here rather than letting an implementation
choose by default:

- how successful and failed compilations return partial or complete reports;
- whether canonical traces are serialized or embedded in artifacts, noting that
  docs/artifact-abi.md currently does not contemplate embedding them;
- which renderer guarantees, retention controls, and provider-detail/redaction
  policy form part of the public contract;
- whether public enums are non-exhaustive, versioned schema views, or both;
- which component may mint trusted evidence receipts for external providers;
- whether the public identity is canonical bytes, a specified digest, or both;
- how much of the request-qualified renderer header is stable versus redacted.

The merged draft's own handoff notes on `tickets/prototype-typed-explain-infrastructure.md`
record the reasoning behind each; read them before proposing answers.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
