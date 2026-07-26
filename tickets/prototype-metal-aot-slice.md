---
id: prototype-metal-aot-slice
title: Produce and validate the serial Sum Metal AOT bundle
status: done
priority: p0
dependencies: [prototype-metal-bundle-assembly, prototype-proof-case-sidecar, promote-the-proof-sidecar-facade]
related: []
scopes: [implementation/metal, implementation/artifact, research/apple-targets, research/artifacts, implementation/metal-aot, implementation/runtime, contracts/decisions, research/cache, research/shapes]
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

## Outcome

All four requirements are met, at `b6a5fad`. The integration built no component capability; where a requirement was not satisfiable against the real bundle, the boundary is recorded rather than the requirement quietly narrowed.

### The composed path, and the sidecar

`d9d03f7` published the proof-case sidecar the producer had never written, replacing the bare `.identity` file. The producer drives the selected fused program and the retained materialized reference through emission, `xcrun` compilation, payload assembly, and bundle assembly into one envelope plus its versioned sidecar; `the_published_sidecar_binds_to_the_published_envelope` proves the pair is consistent, and `a_perturbed_envelope_no_longer_binds_its_sidecar` proves a mismatched pair is refused rather than tolerated.

The filename is the whole interface between the two prototypes and no code crosses it. The producer wrote `.proof` while the runner still opened `.identity` for a complete commit, and the gate stayed green over a slice that could not run end to end. Both halves now pin `SIDECAR_SUFFIX` in a test naming the other side, which is the only mechanical comparison of the two crates' idea of that name.

### Determinism is across processes, and was not

**The case this replaces could not fail for the reason it existed to detect.** It called one in-process helper twice while its doc comment claimed to catch a value varying per process — a timestamp, an address, a hash seed. Two calls in one process share an address space and a hash seed by construction. The concrete bug class this blindness hides is a canonical-ordering defect: a `HashMap` read in iteration order agrees with itself within a process and disagrees across two.

`prototypes/serial-sum-compile/tests/determinism.rs` now runs the producer binary twice, in two processes, into two directories, and compares the envelope bytes, the sidecar bytes, and the artifact identity decoded back out of each sidecar. It is an integration target because `CARGO_BIN_EXE_*` and `CARGO_TARGET_TMPDIR` are not set for a unit test inside a `[[bin]]`. It also asserts non-vacuity — two empty files are equal to each other, and every comparison in the file would pass over a producer that published nothing.

**Measurement boundary.** Agreement across two processes is evidence that the encoding is a pure function of the declared inputs, on this host, with this toolchain, for this program. It is not a portability claim.

### Negative paths: which class refused, and what byte surgery cannot reach

`the_produced_bundle_is_refused_by_the_class_each_damage_earns` asserts the `ArtifactCodecFailure` class per damage form rather than only that something was refused — a bare `expect_err` passes when a bundle is rejected for the wrong reason. Twelve forms against a bundle a real `xcrun` link produced: no bytes, half the envelope, one byte short, magic alone, one trailing byte, a damaged magic and declared total length (`malformed`); an envelope format, canonical encoding, and digest algorithm this reader does not implement (`unsupported` — the ticket's "unsupported schema versions"); a section count past the governed bound (`limit`); and a damaged payload section (`integrity`). The class helper's wildcard returns a name no case expects, because `ArtifactCodecFailure` is `#[non_exhaustive]` and a sixth class would otherwise be folded into one of the five.

**Three named forms are unreachable here, and this is measured rather than assumed.** Noncanonical encoding, duplicate/missing references, and identity mismatch all live inside the region the manifest digest covers, so byte surgery on a published envelope earns `IntegrityFailure` before any structural check runs. `a_structural_violation_is_unreachable_behind_the_manifest_digest` pins that precedence. Reaching `Invalid` needs a manifest *re-encoded* around the violation, which is a codec-internal construction this producer cannot perform and should not gain a way to; those cases are owned by `a_forged_identity_is_rejected`, `a_repeated_interface_key_is_rejected`, `an_unreferenced_section_is_rejected`, `a_repeated_expression_node_is_rejected`, and `an_expression_reference_outside_the_arena_is_rejected` in `crates/tiler-artifact/src/program/codec/tests.rs`. To refute the precedence claim rather than the conclusion, flip any byte at or past the manifest digest at offset 37 and observe the class.

Device-free is what makes this the right home for the check. The runner's `probe_fail_closed` covers identity mismatch, a foreign profile descriptor, and a foreign backend family against real bytes, but it needs a Metal device; the runner's device-free unit tests use a synthetic fixture, not the produced bundle.

**Both new assertion families were confirmed able to fail before being trusted** — one expected class was flipped to a wrong variant and the case observed failing, and the determinism comparison was perturbed and observed failing. A check whose failure path is unreachable reports success for a population it never examined.

### Measurement — metallib byte reproducibility

Recorded as evidence, never asserted. Reproducibility of the linker's bytes is a property of a toolchain this repository does not control; gating on it would make an unrelated Xcode update look like a Tiler defect.

- **Result.** Byte-identical across two links on this host, 3,843 bytes each.
- **Toolchain.** `metal` "Apple metal version 32023.883 (metalfe-32023.883)"; `metallib` "AIR-LLD 32023.883 (metalfe-32023.883) (compatible with legacy metallib linker)".
- **SDK.** `macosx` 26.5, build 25F70. Xcode 26.6, build 17F113.
- **Flags.** `-target air64-apple-macos13.0 -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`.
- **Host.** Apple M4 Max, arm64, macOS 27.0 build 26A5388g.
- **Procedure.** `cargo test -p tiler-prototype-compile -- --nocapture`, reading `metallib_byte_reproducibility_is_measured_and_recorded`.

This is one host and one toolchain row. It qualifies a bounded profile and proves no universal reproducibility claim.

### Defects corrected in the sidecar work before it was committed

`INPUT_KEY` and `input_bits` had each lost their doc comment to a neighbour inserted above them, and a test sat above its module's `use` block. Substantively: `decode_f32_bits` truncated a payload to whole elements instead of refusing a wrong length, and only the *input* payload was length-checked — so a short expected buffer reached the comparison and was reported as `ProofError::Mismatch`, a claim about the device's arithmetic made about a defect in the record. Both payloads are now checked against the element count the artifact declares, and a wrong length fails as `SidecarShapeMismatch`.

### Not done here

Library loading, pipeline creation, and dispatch remain the runtime tickets'. No generalized cache, proc macro, or production artifact compatibility was implemented.
