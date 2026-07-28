---
id: declare-a-required-gpu-family-in-the-artifact
title: Decide which live-device requirements an artifact route must declare
status: awaiting-decision
priority: p2
dependencies: []
related: [prototype-metal-runtime-preflight, carry-the-stage-execution-order-in-the-envelope]
scopes: [contracts/artifacts, implementation/artifact, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, runtime, metal, correctness]
---
## Decision needed (2026-07-28)

**What, if anything, must an artifact declare about the device a route requires — a governed family key, numeric floors, or nothing beyond the recorded provenance?**

A user should receive an explainable preflight refusal before committing a route the selected device cannot execute. The decision is the smallest readable requirement set sufficient to make that refusal precise without putting Apple-specific semantics in the neutral artifact layer.

**Measurement — the deciding experiment is not producible on available hardware.** This ticket said the question would be decided by "a kernel that builds a pipeline successfully on one family and not on another". Both devices reachable from this workspace report the **same family**:

| host | device | highest Apple family | max threads/threadgroup | max buffer |
| --- | --- | --- | --- | --- |
| M4 Max (local) | Apple M4 Max | **Apple9** | 1024 | 22,613,000,192 |
| M3 Pro (Tailscale) | Apple M3 Pro | **Apple9** | 1024 | 10,726,686,720 |

Produced by `prototypes/serial-sum-run`'s own `device_facts` (`prototypes/serial-sum-run/src/proof.rs:1545`) on each host, **recorded at commit `c0f35b4`**. Attributed to that commit rather than to "`main`", because `main` moves and the producing source has moved with it: `git log --oneline c0f35b4..HEAD -- prototypes/serial-sum-run/src/proof.rs` returns `e400e37`, so a rerun today is a rerun of different source. The two devices differ only in buffer and working-set size, which track installed memory rather than family, and their threadgroup limits are identical. **So no kernel can be written here that builds on one and not the other on family grounds** — the experiment needs a device of a different Apple generation, which this workspace does not have.

That is new and it changes the shape of the question rather than answering it: the ticket recorded that nothing had produced the measurement, and this records *why*, so nobody re-attempts it on these two machines.

**The three candidates, merged into one list with the verdict the measurement supports.** Do not settle this by picking the cheapest.

- **A governed family key on the target profile.** Readable, comparable against `supportsFamily`, and it turns the host's check into a real one. It also puts an Apple vocabulary into a consumer-agnostic artifact layer, which `AGENTS.md` guards against, so it needs a neutral spelling or an explicit backend-scoped extension point — and that spelling is itself an ADR 0075 public-boundary decision. **Survives, with a cost.**
- **Numeric floors rather than a family.** A minimum threadgroup size and a minimum buffer length are backend-neutral and directly checkable. They under-describe a family — two devices with equal limits can differ in features a kernel used — and the measurement above shows that risk is live rather than theoretical: the two devices here have *identical* threadgroup limits, so floors would distinguish nothing between them. **Survives, but is measurably weaker than it looks.**
- **Leave it undeclared and keep the provenance.** Honest and cheap. A kernel needing an absent feature fails at pipeline creation rather than at a readable refusal — which the preflight now classifies as a route miss, so it is not silently wrong, only late and imprecisely explained. **Survives, and is the status quo.**

All three survive, which is why this is Tom's. The elimination that *would* have decided it needs hardware this workspace lacks.

**No recommendation is offered, deliberately.** The ticket says not to settle it by picking the cheapest, and with the deciding measurement unavailable, any recommendation from here would be exactly that — a cost argument wearing an evidence argument's clothes. What can be said is that option 2's neutrality is worth less than it appears, because the only two devices available to test it are indistinguishable by the floors it would declare.

## What is true today

Split from `prototype-metal-runtime-preflight`, which added the device-side preflight and found that two of the things it was asked to check have nothing on the artifact side to check against.

**Fact — no artifact field names a required GPU family.** `MetalTargetFacts` (`crates/tiler-metal/src/target.rs:615-640`) carries six fields — the MSL version, the platform, the deployment minimum, the launch-index realization, the per-arithmetic-type subnormal facts, and the buffer binding capacity — and no family among them. The target profile reaches an artifact as a `TargetProfileKey` and a `TargetProfileDescriptorDigest`, and a digest is comparable rather than readable. Verified by reading `crates/tiler-metal/src/target.rs` and `crates/tiler-artifact/src/program/` at `40c58f3`, re-checked against `target.rs:615-640` at `01264be`.

**Fact — the device preflight therefore records rather than checks.** `device_facts` in `prototypes/serial-sum-run/src/proof.rs` reports the device name, its highest supported Apple family, `max_threads_per_threadgroup`, `max_buffer_length`, and `recommended_max_working_set_size` as provenance. Only the two limits with an artifact-side counterpart are checked: the pipeline's threadgroup capacity against the declared launch, and the per-buffer bound against a declared accessible range. Checking a family requirement the artifact never made would be inventing one.

**Fact — the host's stated environment is not a statement about its device.** `host_environment` builds an `ExecutionEnvironment` from the compiler's own target authority. That is a defensible independent source for the profile descriptor — it is not read from the artifact, so the comparison is not a tautology — and it establishes nothing about the GPU actually present. Nothing checks that the device satisfies what the profile assumes.

## Closes when

Either the artifact declares what a route requires of a device and the runtime preflight checks it with a typed refusal in the class `prototype-metal-runtime-preflight` defines, or the question is closed with a recorded decision that it stays undeclared and why, with the provenance left as the durable answer. `make full` passes.

**Trigger to reopen:** access to an Apple device of a different GPU generation, or a kernel whose pipeline creation is observed to fail for a feature reason on any reachable device.
