---
id: produce-a-custom-backend-payload-through-the-build-orchestrator
title: Produce a custom backend payload through the build orchestrator
status: todo
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary]
related: [drive-the-build-orchestrator-from-a-checked-compiler-plan, assemble-prepared-metal-artifacts-in-tiler-build]
scopes: [implementation/build, implementation/compiler, implementation/artifact, contracts/artifacts, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, build, artifacts]
---
## User-visible outcome

A statically linked custom backend producer can consume verified compiler output through the build orchestrator and publish one canonical backend payload without forging derived identity or coupling the compiler core to that backend.

## Implementation keys

- Define the smallest accepted emitter/artifact-producer facade; do not expose `tiler-build` internals as the generic model.
- Feed only verified structured KIR/program products and the accepted target/profile request into the producer.
- Separate pure emission from external AOT/tool invocation and artifact assembly even when one backend implements both.
- Derive backend, representation, entry mapping, compilation subject, payload digest, target obligations, and cache subject through their owning checked builders.
- Preserve complete source/toolchain/flag/provenance identity and reject mismatched profile or ABI before external compilation.
- Support a producer that is not Metal and a partial custom Metal provider that reuses standard Metal emission where the accepted composition permits it.
- Prove malformed entry mappings, unstable identity, duplicate producers, forged payload facts, and cache subject disagreement fail.
- Present the exact public trait/type/call-site boundary to Tom.

## Closes when

One external producer creates a decoded, self-validating payload through the ordinary build path, byte and identity determinism are demonstrated, mutation tests move every affected identity, targeted checks pass, and the standard Metal path remains behaviorally unchanged.

## Graph maintenance

- Feed the payload into runtime-adapter and cross-process join tickets.
- Keep dynamic plugin loading and provider discovery out of scope.
- If a new crate is required, split crate admission into its own Tom-reviewed ticket before implementation.
