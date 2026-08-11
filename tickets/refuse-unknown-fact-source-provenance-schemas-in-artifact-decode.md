---
id: refuse-unknown-fact-source-provenance-schemas-in-artifact-decode
title: Refuse unknown fact-source provenance schemas in artifact decode
status: todo
priority: p1
dependencies: []
related: [record-the-compilation-selection-in-target-measurement-provenance, carry-required-compilation-selection-identity-on-compile-profile-contexts]
scopes: [implementation/ir, implementation/artifact, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, provenance, identity, correctness, fail-closed]
---
## User-visible outcome

An artifact carrying an unsupported fact-source provenance schema is rejected with a typed error before any field is interpreted. It is never normalized into the current schema.

## Fact

`decode_provenance` in `crates/tiler-artifact/src/program/realization/codec.rs` reads the incoming schema into `_schema`, discards it, decodes the remaining bytes as the current grammar, and reconstructs the value through `FactSourceProvenance::new`. That constructor emits the current schema, so a foreign schema can be silently normalized if its remaining bytes happen to parse.

## Required delivery

- Make provenance-schema dispatch explicit and exhaustive before decoding the body.
- Support only schemas with an actual decoder. Unknown, newer, and retired schemas have typed, distinguishable refusal; there is no current-schema fallback.
- Preserve the incoming schema through any accepted version-specific decode so canonical re-encoding cannot silently change it.
- Re-read every artifact and compiler consumer before choosing whether an older schema is supported or refused.
- Perturb only the schema bytes of an otherwise valid delivered-realization record and quote the exact refusal. Also prove current-schema round trip and canonical-byte equality.

## Non-goals

This ticket does not add compilation-selection provenance, migrate a target profile, or reinterpret old bytes under a new grammar.

## Closes when

Every admitted provenance schema has an explicit decoder and every other schema fails closed before its body is consumed.
