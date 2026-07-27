---
id: declare-a-required-gpu-family-in-the-artifact
title: Decide which live-device requirements an artifact route must declare
status: todo
priority: p2
dependencies: []
related: [prototype-metal-runtime-preflight, carry-the-stage-execution-order-in-the-envelope]
scopes: [contracts/artifacts, implementation/artifact, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, runtime, metal, correctness]
---
Split from `prototype-metal-runtime-preflight`, which added the device-side preflight and found that two of the things it was asked to check have nothing on the artifact side to check against.

## What is true today

**Fact — no artifact field names a required GPU family.** `MetalTargetFacts` carries the platform, the deployment minimum, the MSL version, the launch-index realization, the subnormal arithmetic facts, and the buffer binding capacity. The target profile reaches an artifact as a `TargetProfileKey` and a `TargetProfileDescriptorDigest`, and a digest is comparable rather than readable. Verified by reading `crates/tiler-metal/src/target.rs` and `crates/tiler-artifact/src/program/` at `40c58f3`.

**Fact — the device preflight therefore records rather than checks.** `device_facts` in `prototypes/serial-sum-run/src/proof.rs` reports the device name, its highest supported Apple family, `max_threads_per_threadgroup`, `max_buffer_length`, and `recommended_max_working_set_size` as provenance. Only the two limits with an artifact-side counterpart are checked: the pipeline's threadgroup capacity against the declared launch, and the per-buffer bound against a declared accessible range. Checking a family requirement the artifact never made would be inventing one.

**Fact — the host's stated environment is not a statement about its device.** `host_environment` builds an `ExecutionEnvironment` from the compiler's own target authority. That is a defensible independent source for the profile descriptor — it is not read from the artifact, so the comparison is not a tautology — and it establishes nothing about the GPU actually present. Nothing checks that the device satisfies what the profile assumes.

## The question this ticket has to answer

A user should receive an explainable preflight refusal before committing a
route the selected device cannot execute. Determine the smallest readable
requirement set sufficient to make that refusal precise without putting
Apple-specific semantics in the neutral artifact layer.

The candidates are not equivalent:

- **A governed family key on the target profile.** Readable, comparable against `supportsFamily`, and it makes the host's check a real one. It also puts an Apple vocabulary into a consumer-agnostic artifact layer, which `AGENTS.md` guards against, so it needs a neutral spelling or an explicit backend-scoped extension point.
- **Numeric floors rather than a family.** A minimum threadgroup size and a minimum buffer length are backend-neutral and directly checkable, and they under-describe a family: two devices with equal limits can differ in features a kernel used.
- **Leave it undeclared and keep the provenance.** Honest, cheap, and it means a kernel that needs a feature the device lacks fails at pipeline creation rather than at a readable refusal — which the preflight now classifies as a route miss, so it is not silently wrong, only late and imprecisely explained.

Do not settle this by picking the cheapest. The measurement that would decide it is a kernel that builds a pipeline successfully on one family and not on another; nothing in this repository has produced one.

## Closes when

Either the artifact declares what a route requires of a device and the runtime preflight checks it with a typed refusal in the class `prototype-metal-runtime-preflight` defines, or the question is closed with a recorded decision that it stays undeclared and why, with the provenance left as the durable answer. `make full` passes.
