---
id: declare-a-required-gpu-family-in-the-artifact
title: Declare backend-neutral live-device route requirements
status: todo
priority: p2
dependencies: [source-or-rephase-first-metal-launch-limits]
related: [prototype-metal-runtime-preflight, carry-the-stage-execution-order-in-the-envelope]
scopes: [contracts/artifacts, implementation/artifact, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, runtime, metal, correctness]
---
## User-visible outcome

A runtime refuses a route before commitment when the selected live device lacks a backend feature or numerical resource floor the artifact declares, and the refusal names the exact unmet requirement without putting Apple-specific vocabulary in the neutral artifact core.

## Implementation keys

Provenance alone is eliminated because it permits a known device mismatch to survive until pipeline creation after routing. Numeric floors and backend feature requirements are complementary rather than alternatives: floors express quantities such as threadgroup capacity and accessible buffer range, while a backend-scoped governed feature key expresses non-quantitative capabilities that equal floors cannot distinguish.

Define a backend-neutral, versioned route-requirement family. Core quantitative rows carry typed dimensions and minima. Backend-scoped qualitative rows carry an owner namespace, governed requirement key/version, and canonical payload validated by the owning runtime adapter. The neutral artifact layer does not interpret an Apple family enum; the Metal adapter maps its governed requirements to `supportsFamily` or a more exact supported-feature query and produces a typed preflight refusal.

Reuse the exact-entry `PreparedEntryTargetRequirement` and prepare/resolve route introduced by `source-or-rephase-first-metal-launch-limits` for properties answered after pipeline preparation. Do not create a parallel query, relation, phase, or entry-subject vocabulary. Structural quantities already derivable from the verified route—launch geometry and each binding's accessible byte window—remain derived requirements and are checked directly against host/device limits; copying them into extra artifact rows would create a second authority.

The artifact records only additional requirements consumed by the selected executable route and not already derivable from its verified program. It does not copy a whole target profile, infer support from provenance, or treat an unknown requirement as skippable. Unknown owner, requirement kind, version, or payload rejects fail-closed. Zero additional rows is correct when the selected route consumes no backend feature; “missing” is decidable only against a producer-owned exhaustive declaration of the features the selected payload actually uses.

**Measurement — the deciding experiment is not producible on available hardware.** This ticket said the question would be decided by "a kernel that builds a pipeline successfully on one family and not on another". Both devices reachable from this workspace report the **same family**:

| host | device | highest Apple family | max threads/threadgroup | max buffer |
| --- | --- | --- | --- | --- |
| M4 Max (local) | Apple M4 Max | **Apple9** | 1024 | 22,613,000,192 |
| M3 Pro (Tailscale) | Apple M3 Pro | **Apple9** | 1024 | 10,726,686,720 |

Produced by `prototypes/serial-sum-run`'s own `device_facts` (`prototypes/serial-sum-run/src/proof.rs:1545`) on each host, **recorded at commit `c0f35b4`**. Attributed to that commit rather than to "`main`", because `main` moves and the producing source has moved with it: `git log --oneline c0f35b4..HEAD -- prototypes/serial-sum-run/src/proof.rs` returns `e400e37`, so a rerun today is a rerun of different source. The two devices differ only in buffer and working-set size, which track installed memory rather than family, and their threadgroup limits are identical. **So no kernel can be written here that builds on one and not the other on family grounds** — the experiment needs a device of a different Apple generation, which this workspace does not have.

This measurement cannot qualify a particular Apple cross-generation requirement, but it does not block the architecture: the two hosts already demonstrate why floors cannot replace feature rows. Preserve a deferred qualification trigger for access to a different GPU generation or an observed feature-specific pipeline refusal.

## What is true today

Split from `prototype-metal-runtime-preflight`, which added the device-side preflight and found that two of the things it was asked to check have nothing on the artifact side to check against.

**Fact — no artifact field names a required GPU family.** `MetalTargetFacts` carries the MSL version, platform, deployment minimum, per-arithmetic-type subnormal facts, and buffer binding capacity, and no family among them. The compiler target profile separately carries operation-complete KIR unsigned-64 arithmetic support, while the selected Metal emission realization records the launch-index parameter type chosen for one translation unit; neither is a live-device family fact and neither can establish one. Device-address width remains absent because the current KIR has no device-address-width consumer. The target profile reaches an artifact as a `TargetProfileKey` and a `TargetProfileDescriptorDigest`, and a digest is comparable rather than readable. The in-progress launch-limit route now carries prepared-entry workgroup requirements; this ticket must reuse rather than duplicate that completed vocabulary.

**Fact — the device preflight therefore records rather than checks family.** `device_facts` in `prototypes/serial-sum-run/src/proof.rs` reports the device name, its highest supported Apple family, `max_threads_per_threadgroup`, `max_buffer_length`, and `recommended_max_working_set_size` as provenance. Route-derived accessible ranges and prepared-pipeline workgroup capacity are checked through their existing authorities. Checking a family requirement the selected payload never declared would be inventing one.

**Fact — the host's stated environment is not a statement about its device.** `host_environment` builds an `ExecutionEnvironment` from the compiler's own target authority. That is a defensible independent source for the profile descriptor — it is not read from the artifact, so the comparison is not a tautology — and it establishes nothing about the GPU actually present. Nothing checks that the device satisfies what the profile assumes.

## Closes when

The artifact carries canonical quantitative and backend-scoped qualitative route requirements; the Metal runtime checks every row before route commitment; missing, unknown, malformed, duplicate, and unmet rows reject with typed causes; neutral artifact code contains no Apple family enum; representative checks are perturbed and observed failing; the exact artifact vocabulary, codec, runtime preflight, and refusal boundary receive Tom's public review; and `make full` passes.

## Graph maintenance

- Split Apple cross-generation qualification into a bounded measurement when another GPU generation or a feature-specific pipeline refusal becomes available.
- Keep route requirements distinct from target-fact provenance: provenance explains why a route was produced, while requirements are executable preconditions.
- Advance artifact schema and identity once on the merged tree and recompute every affected pin there.
- Keep device-specific observation and `supportsFamily` calls out of device-free `tiler-runtime`. The serial-sum prototype may prove the first adapter; promotion to a reusable Metal runtime adapter is a separate public ownership boundary requiring Tom's review.
