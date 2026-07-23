---
id: prototype-metal-aot-slice
title: Produce and validate the serial Sum Metal AOT bundle
status: todo
priority: p0
dependencies: [prototype-metal-bundle-assembly, prototype-proof-case-sidecar]
related: []
scopes: [implementation/metal, implementation/artifact, research/apple-targets, research/artifacts, implementation/metal-aot]
shared_scopes: [project/tickets, contracts/artifacts, contracts/numerics, implementation/cargo-lock]
paths: []
tags: [implementation, prototype, metal, vertical-slice]
---
Integration gate only: wire the already-implemented component capabilities into
the non-published `serial-sum-compile` producer and prove the complete offline
path end to end. This ticket builds no component capability itself — MSL
emission is owned by `prototype-metal-kir-lowering`, the exact-math/NaN
realization by `prototype-metal-numerical-realization`, SDK/family/flag
selection and `xcrun metal`/`xcrun metallib` invocation by
`prototype-apple-aot-driver`, bundle packaging by
`prototype-metal-bundle-assembly`, sidecar generation by
`prototype-proof-case-sidecar`, and decode/validation by
`prototype-neutral-artifact-codec`. If integration exposes a gap in a
component, reopen or follow up that ticket rather than implementing the
capability here.

The integration must:

- drive the selected fused program and retained materialized reference program
  through the composed components into one self-validating bundle plus its
  versioned proof-case sidecar, with every output-affecting input represented
  in identity or provenance as specified by the artifact contract;
- prove generated MSL and canonical manifest bytes are deterministic across
  repeated producer runs;
- decode and validate the produced bundle without a live device, exercising the
  codec's negative paths against this real bundle (noncanonical encoding,
  truncation, trailing content, corruption, duplicate/missing references,
  identity mismatch, unsupported target facts or schema versions); and
- measure and record metallib byte reproducibility with complete host/toolchain
  provenance; reproducibility is evidence, not a correctness claim or gate
  unless the selected toolchain evidence proves it.

Library loading and pipeline creation belong to the runtime tickets. Do not
implement dispatch, a generalized cache, a proc macro, or production artifact
compatibility.
