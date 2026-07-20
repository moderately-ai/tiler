---
id: synthesize-artifact-contracts
title: Synthesize artifact, cache, Metal, and macro contracts
status: done
priority: p1
dependencies: [cache-crash-race-harness, apple-artifact-compatibility, macro-build-environment, embedded-artifact-costs, runtime-execution-contract]
related: [synthesize-core-contracts, synthesize-optimizer-contracts]
scopes: [contracts/artifacts, contracts/integrations, contracts/core, research/embedding]
shared_scopes: []
paths: []
tags: [tiler-research, synthesis, decision]
---
Update the artifact ABI, Metal backend, frontend integration, and runtime integration contracts using the completed artifact and environment evidence. Preserve the inline, self-contained AOT developer experience and state target/toolchain compatibility precisely.

Acceptance requires one cache and bundle contract, complete identity categories, rebuild semantics, artifact-family selection rules, compatibility failure stages, preflight/fallback safety, and explicit deferred questions such as Catalyst and binary archives.

Completed synthesis accepts the target-neutral envelope/typed backend-payload
boundary; immutable self-validating cache protocol; complete Metal toolchain,
target, numerical, ABI, and program identity categories; explicit inline
artifact-family selection and toolchain rebuild boundary; measured direct-byte
embedding gates; staged Metal compatibility/preparation failures; and a one-way
runtime `RoutingCommit` before allocation or encoding. ADRs 0001–0004 are
accepted and ADRs 0050–0051 record cache publication and runtime ownership.

Catalyst, binary archives/dynamic Metal libraries, a production serialization
codec, cache location/GC limits, cross-machine/old-runtime compatibility,
rust-analyzer performance, and the actual macro-to-dispatch vertical remain
explicit measured or implementation gates rather than assumed support.

## Outcome

Reconciled the [artifact ABI](../docs/artifact-abi.md),
[Metal backend](../docs/backends/metal.md), [frontend integration](../docs/integration/frontends.md),
and [Candle integration](../docs/integration/candle.md) with the completed
artifact, cache, macro, compatibility, embedding, and runtime evidence. The
accepted decisions are ADRs 0002–0004, 0049–0051, and 0053.
