---
id: prototype-metal-aot-slice
title: Produce and validate the serial Sum Metal AOT bundle
status: todo
priority: p0
dependencies: [prototype-target-neutral-fusion-slice]
related: []
scopes: [implementation/metal, implementation/artifact, research/apple-targets, research/artifacts]
shared_scopes: [project/tickets, contracts/artifacts, contracts/numerics]
paths: [Cargo.lock]
tags: [implementation, prototype, metal, vertical-slice]
---
Turn the selected fused program and retained materialized reference program into
one self-validating Metal AOT bundle:

- emit deterministic MSL only from verified structured kernel IR, with every
  entry point needed by the one-stage fused and two-stage materialized programs;
- make the selected Apple artifact family, SDK, deployment minimum, MSL version,
  exact-math realization, compiler/linker flags, and compiler provenance
  explicit rather than inheriting toolchain defaults;
- realize the prototype's canonical arithmetic-NaN contract explicitly in
  generated code, prohibit unlicensed contraction/reassociation, and test these
  semantics independently of compiler fast/safe-math flags;
- invoke `xcrun metal` and `xcrun metallib` from the non-published
  `serial-sum-compile` producer;
- package the exact MSL identity, target facts, manifest, section digests, and
  metallib as one canonical bounded immutable bundle;
- generate a separate versioned prototype proof-case sidecar containing stable
  case keys, bit-preserving input bytes, expected bytes produced by the normative
  reference evaluator, semantic-program identity, numerical-profile identity,
  reference-evaluator/profile version, section digests, and the exact envelope
  digest it accompanies; the
  sidecar is test evidence and must not become runtime artifact semantics; and
- decode and validate the produced bundle without a live device, including
  negative tests for noncanonical encoding, truncation, trailing content,
  corruption, duplicate/missing references, identity mismatch, and unsupported
  target facts or schema versions.

The proof succeeds when the offline compiler accepts the fixed program, the
device-free validator accepts the completed bundle, and every output-affecting
input is represented in identity or provenance as specified by the artifact
contract. Generated MSL and canonical manifest bytes must be deterministic.
Measure and record metallib byte reproducibility with complete host/toolchain
provenance; it is not a correctness claim
or gate unless the selected toolchain evidence proves it. Library loading and
pipeline creation belong to the runtime ticket. Do not implement dispatch, a
generalized cache, a proc macro, or production artifact compatibility.
