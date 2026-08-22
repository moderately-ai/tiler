---
id: bind-frozen-live-extent-bytes-at-declared-backend-transports
title: Bind frozen live-extent bytes at declared backend transports
status: todo
priority: p0
dependencies: [associate-live-extent-operands-with-symbolic-semantic-interface-axes, accept-the-live-extent-artifact-envelope-row, re-prove-the-live-extent-operand-association-at-decode]
related: [prove-one-live-extent-artifact-payload-and-pipeline-at-two-n]
scopes: [implementation/artifact, implementation/runtime, implementation/metal, implementation/metal-aot, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, metal, abi, correctness]
---
## User-visible outcome

Every backend that admits a live-extent payload binds the frozen little-endian parameter bytes at the exact transport declared by the artifact. A backend without that support refuses before routing commit.

## Exact gap and per-Fact audit at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`

- **Verified.** `RoutedExtentParameter::parameter_bytes` and `transport_slot` in `crates/tiler-runtime/src/load/route.rs` publish the frozen bytes and placement contract.
- **Verified.** `rg -n 'parameter_bytes\(' crates --glob '*.rs'` finds only the definition. No backend consumes those bytes.
- **Verified.** The scalar execution fixture reads `RoutedExtentParameter::value()` directly and its image has only read/write buffer transports. Metal tests compile the `eN` MSL signature but do not bind or dispatch that transport.
- **Verified.** Artifact transport-cardinality and ordering negatives validate the envelope. They are not a backend-misbound execution negative, so the parent ticket's frozen-byte/declared-transport evidence remains absent.

## Required work

- At backend preparation, copy `parameter_bytes()` to the resource/argument at `transport_slot()` for every declared extent operand. Preserve canonical order and read-only semantics.
- A backend that cannot bind the declared scalar transport must refuse during preparation. After commit, a transport/value mismatch is terminal; no fallback or second scalar list is permitted.
- Make the backend execution oracle read the bound transport, not `.value()` or fixture-owned dimensions. Remove the bypass once the declared transport is exercised.
- If implementing this contract requires a new public adapter method or changes the accepted envelope row, produce a labelled draft and return to Tom. Do not self-accept an adapter surface.

## Required evidence

- One backend executes two neighbouring bindings from identical artifact/payload/pipeline subjects and observes the exact bytes at the declared slot.
- Independently swap slots, reverse endian order, omit the transport, mutate after freeze, and read `.value()` directly. Each subject perturbation must fail the unchanged assertion with quoted text.
- A backend with no extent-transport support refuses before program work; a post-commit mismatch is terminal.
- Targeted artifact/runtime/Metal/Metal-AOT/build tests, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Unsupported cases

Only the accepted unsigned 64-bit input-extent operand is in scope. Arbitrary scalar parameters and a second caller-supplied list remain unsupported.

## Closes when

No admitted backend can execute a live-extent payload without consuming the frozen bytes at the artifact-declared transport, and every unsupported backend refuses before commit.
