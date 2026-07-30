---
id: prototype-metal-runtime-preflight
title: Implement Metal runtime preflight
status: done
priority: p0
dependencies: [prototype-runtime-artifact-validation, prototype-metal-aot-slice]
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, metal, correctness]
---
Preflight device, family, library, every selected function/pipeline, resources, bindings, launch expressions, and scratch before routing commit or program work. Distinguish route misses from corrupt artifacts and systemic failures with typed phases and injected failures.

## Outcome

Every device-decidable obligation is now discharged before `Preflight::commit`, at `40c58f3`. Where the ticket asks for something the artifact cannot express, the gap is recorded rather than approximated.

### The defect, which the source had already conceded

`Preflight::commit` is infallible and documents why: every decidable obligation is supposed to have been discharged before it, so a failure afterwards would mean an obligation was checked in the wrong stage. Three were checked afterwards, inside `dispatch_routed` — the library and pipeline creation, the allocation length, and the comparison of the declared workgroup against the capacity that pipeline reports. That function's own doc comment named them and called them "not decidable here".

They are decidable. They are not device-*free*, and the device is in hand before the commit.

**The workgroup check is what makes this correctness rather than tidiness.** A workgroup this pipeline cannot run is a fact about *this route on this device*, so a differently-declared variant might satisfy it, and ADR 0051 permits that fallback only while the commit has not been taken. Reporting it after the commit converts a fallback the host was still owed into a failure it merely reports.

### What the stage discharges, in order

`device_preflight` builds the library from the payload's object, resolves the entry symbol, creates the pipeline, compares the declared launch against `max_total_threads_per_threadgroup`, and allocates and fills every buffer including entry-internal scratch — all before the commit. `dispatch_prepared` then encodes and submits: it looks nothing up, allocates nothing, and has no refusal of its own. `submit` keeps the one thing that can still fail, a command buffer that does not reach `Completed`, checked before any read-back, and the buffers stay alive through their final device use.

Buffers are sized from the route's own accessible byte ranges rather than from the host's operand slice. Deriving a length from the host's data would re-answer a question the artifact already answered.

### The three classes, and why one of them is a derivation

A phase per stage — `library`, `function`, `pipeline`, `launch-geometry`, `resources` — and every refusal classified into exactly one of:

- **`corrupt-artifact`** — the library will not load, or the symbol is absent. Distinct from an integrity failure, which the codec already refused before any of this ran: the digest matched, so the object *is* what the producer published and it is content that will not execute. A caller re-fetches; retrying another variant of the same bytes is not indicated.
- **`route-miss`** — the workgroup exceeds the pipeline's capacity, or a binding exceeds the per-buffer limit. Another variant might fit, and a fallback is the indicated response.
- **`systemic`** — an allocation shorter than the length it was requested at. This is an assertion against the device's own report, so reaching it means the allocator did not honour a request it accepted.

**`PipelineRejected` is classified a route miss by derivation, not preference.** Metal reports pipeline-creation failure as a message string that does not separate "this function exceeds a device limit" from "the device is out of resources". Of the two ways to be wrong, calling a systemic failure a route miss costs a retry that then fails; calling a route miss systemic abandons an artifact that had a working variant. Only the second forfeits a fallback still held, so the classification takes the recoverable direction. The reasoning is on `PreflightRefusal::class` so it can be refuted rather than only read.

### Injected failures

Device-free, in the existing unit suite: `each_device_preflight_refusal_carries_its_phase_and_class` lists every variant with its expected phase and class explicitly rather than deriving them from the functions under test, so a variant that silently changed class fails rather than agreeing with itself; and `the_device_comparisons_refuse_exactly_at_their_boundary` pins each comparison at the largest accepted value and the smallest refused one, because an off-by-one either rejects a route the device would have run or admits one it cannot.

Device-dependent, in `probe_device_preflight` beside the existing `probe_fail_closed`, because `make full` reaches no device: an object that is not a `metallib`, a symbol the library does not publish, a workgroup one thread past the capacity *this device reported*, and a binding one byte past its buffer limit. Each perturbs one input; the unperturbed route prepares afterwards, which is what makes each refusal evidence about its own perturbation. A perturbation the device *accepts* is reported as `ProofError::ProbeAccepted` rather than passing quietly.

**Both new assertion families were confirmed able to fail before being trusted** — one expected class was flipped to a wrong variant and one boundary was moved one past capacity, and each was observed failing before the perturbation was reverted.

### Measurement — the hardware run

`cargo run -p tiler-prototype-compile -- --out <path>` then `cargo run -p tiler-prototype-run -- --artifact <path>`, Apple M4 Max, macOS 27.0 (26A5388g), Xcode 26.6 (17F113):

```text
device preflight: Apple M4 Max (Apple9), 1024 thread(s) per threadgroup, buffers to 22613000192 byte(s), working set 30150672384 byte(s)
  an object that is not a metallib: library/corrupt-artifact: the carried object did not load: Invalid library file
  an entry symbol the object does not publish: function/corrupt-artifact: the library publishes no "tiler_kernel_this_object_does_not_publish": Function '…' does not exist
  a workgroup one thread past this pipeline: launch-geometry/route-miss: "tiler_kernel_d8260aa9a85f7c45" admits 1024 thread(s) per threadgroup and the artifact declares 1025
  a binding one byte past the buffer limit: resources/route-miss: slot 0 must reach 22613000193 byte(s) and one buffer holds at most 22613000192
  the unperturbed route prepares: every stage cleared before the commit
bit-for-bit agreement: direct on 4 element(s), envelope on 4 element(s)
```

### The surface change

`Preflight` gains `object()` and `entry_symbol()`, mirroring the pair `RoutedDispatch` already carried over the same fields. Reaching those bytes only through `commit()` is precisely what forced the library and the pipeline to be built too late. Additive — not a new namespace, not a new trait, not a `pub(crate)`→`pub` promotion, not a breaking signature change — so it is outside ADR 0075's always-ask categories, and named here so a reviewer sees it rather than finds it. `tiler-runtime` remains device-free under ADR 0081: every device-touching line is in the prototype, which `implementation/runtime` also maps to.

### What is recorded rather than checked, and why

**"Family" has no artifact-side counterpart.** No artifact field names a required GPU family, a threadgroup floor, or a buffer-length floor — the target facts carry platform, deployment minimum, MSL version, subnormal arithmetic, and binding capacity, while the selected launch declaration is carried separately as an emission realization and the profile descriptor is a digest. So the device's name, its highest reported Apple family, and its limits are **provenance**: they say which device produced a measurement and they are what an artifact-side family declaration would later be checked against. Checking a requirement the artifact never made would be inventing one. The two limits that *do* have a counterpart — the pipeline's threadgroup capacity and the per-buffer bound — are checked rather than recorded.

**Historical boundary, since superseded.** This ticket landed when `accept_entry` selected exactly one entry. Multi-stage envelopes now decode and route; `preflight-every-entry-of-a-multi-stage-route` owns the remaining requirement that runtime preflight prepare every selected stage before route commit. The old single-entry implementation boundary is not evidence that the current reader refuses a multi-stage program.

**The host still does not verify the device against the profile it states.** `host_environment` derives the host's facts from the compiler's target authority, which is a defensible independent source for the profile descriptor and is not a statement about the device in front of it. The provenance above is the first half of closing that; the second half needs an artifact-side family declaration.

`declare-a-required-gpu-family-in-the-artifact` owns both remaining gaps and is filed as the follow-up.
