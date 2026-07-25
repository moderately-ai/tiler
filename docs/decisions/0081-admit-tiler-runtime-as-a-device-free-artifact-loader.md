---
schema: "tiler-doc/v1"
id: "ADR-0081"
kind: "decision"
title: "Admit tiler-runtime as a device-free artifact loader"
topics: ["rust", "workspace", "dependencies", "runtime", "artifacts"]
catalog_group: "runtime-integration-placement"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.artifacts.target-neutral-envelope"]
depends_on: ["ADR-0051", "ADR-0043"]
refines: ["ADR-0077"]
ticket: "admit-the-device-free-runtime-validation-crate"
---

# 0081: Admit tiler-runtime as a device-free artifact loader

**Status:** accepted. Tom decided this on 2026-07-25 on the evidence below. It admits a seventh reusable library, `tiler-runtime`, whose whole content is the half of a runtime that can be decided without hardware.

## Context

**Fact — the artifact envelope round-trips and is not in the execution path.** `prototypes/serial-sum-compile` assembles a real compilation and a real 3,667-byte `metallib` into a 32,259-byte envelope with three sections and one variant, and proves encode, decode, and byte-identical re-encode. `prototypes/serial-sum-run` separately dispatches the same program on an Apple M4 Max and matches `tiler-reference` bit for bit. The runner reaches the device through `Device::new_library_with_data(&artifact.metallib)` — the bytes go straight from `CompiledArtifact` to the device and no envelope is encoded, decoded, or validated on the way. Two descriptions of one compilation therefore exist and only one is load-bearing.

**Fact — the missing component is not a Metal component.** Everything that would collapse the two is decidable from bytes: decoding, integrity and identity re-derivation, required-feature negotiation, declared-target-profile classification, program binding, carried-object resolution, and the one-way routing commit ADR 0051 requires. None of it needs a device, and none of it is Metal-specific.

**Fact — the path was reserved and its owner was explicitly temporary.** `ticketsplease.toml` mapped `"implementation/runtime"` to the glob `crates/tiler-runtime/**` while `[scope_crates]` pointed the scope at the prototype executable, under a comment stating that the mapping is "a temporary owner while its production crate is absent" and that "each crate-admission ticket must atomically add the real workspace package and add or move its mapping here". No document, ADR, or ticket named the crate; the string existed only in that glob.

**Fact — the accepted profile withholds a *reusable Metal-runtime* crate, and states the test for what that means.** [ADR 0056](0056-use-four-libraries-and-two-proof-executables.md)'s Decision says "No frontend, proc-macro, Candle, generalized cache, or reusable Metal-runtime crate is created for the first proof", and the [architecture contract](../architecture.md) repeats it as "until the proof reaches those boundaries". [ADR 0077](0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) then wrote the operative test into the same clause it was applying, so that the clause could be applied rather than argued about: a component that "never touches a live device, an `MTLDevice`, or a pipeline state … is not the reusable Metal-*runtime* crate that clause withholds", and — in the same paragraph — "A reader must not cite this admission as precedent for admitting one."

## Decision

### 1. `tiler-runtime` is the device-free half of a Tiler runtime

**Decided.** It decodes artifact bytes through `tiler-artifact`'s codec, classifies an artifact's declared target profile against a host's stated execution environment, binds a decoded artifact to the program a caller expects by comparing canonical identities, resolves which carried object realizes it, and performs the one-way routing commit. It creates no device, no library, no pipeline state, no buffer, and no command encoder.

**Why the line is "touches no device object" rather than "is not about execution".** The second is unfalsifiable — every part of a runtime is about execution. The first is checkable by reading a dependency list and a source file, which is what makes it usable as a boundary by the next person rather than a description of intent.

### 2. Its dependency closure is `[tiler-artifact]`, and that is decided rather than incidental

**Decided.** `scripts/check_workspace.py`'s `EXPECTED_DEPENDENCIES` pins `"tiler-runtime": [tiler-artifact]`, so the closure is mechanically checked in the same table that pins `"tiler-metal-aot": []`.

**Why each absent edge is absent.** `tiler-compiler` is absent because a loader that could reach the optimizer could rebuild a plan instead of validating the one it was handed, which is the boundary ADR 0056 created the crate split to enforce. A platform binding is absent because it would make a load undecidable without hardware, and the value of this crate is precisely that its whole contract is testable on a machine with no GPU. `tiler-ir` is absent as a *direct* edge because every type the loader names is an artifact-layer type; `tiler-artifact` links it transitively, and recording that as a direct edge would claim a dependency the source does not have.

### 3. ADR 0077's non-precedent clause is applied here, not waived

**Decided.** The clause withholds reusable Metal-*runtime* crates, and ADR 0077 states the test: a component touching no live device, no `MTLDevice`, and no pipeline state is not one. `tiler-runtime` meets that test — item 1 is the same predicate, and item 2 makes it structural by refusing the dependency that would let it fail. So this admission is licensed by the clause's own wording rather than being an exception to it, and the crate the clause withholds is still withheld: the component that creates devices, libraries, and pipeline states remains inside `prototypes/serial-sum-run`.

**Why this is recorded rather than assumed.** ADR 0077 anticipated being misused as precedent and said so. A reader arriving at this record must be able to see that the clause was read, its stated test applied, and the result checked — not that a later record found the clause inconvenient. If a future component wants a device object, this record is not its precedent either, for the same reason.

### 4. The admission moves five things together or it moves nothing

**Decided.** A crate admission is complete when `Cargo.toml`'s `members` and `[workspace.dependencies]`, `scripts/check_workspace.py`'s four pinned tables, the [architecture contract](../architecture.md)'s accepted packaging profile, `ticketsplease.toml`'s `[scope_crates]` owner, and this record all agree. Landing a subset would leave the mechanically checked contract disagreeing with the accepted architecture text, which is the exact state ADR 0077 was written to end.

## Consequences

- The workspace carries seven reusable libraries and two non-published proof executables. As with ADR 0077, that count is an ordinal about the crate being admitted rather than a new cap.
- `prototypes/serial-sum-run` gains a route through the envelope. Keeping the direct-dispatch path beside it is deliberate and is recorded on [`route-the-runtime-proof-through-the-artifact-envelope`](../../tickets/route-the-runtime-proof-through-the-artifact-envelope.md): if the direct path still matches the reference and the envelope path does not, the envelope is at fault, and that is a diagnostic worth retaining.
- The profile still deliberately omits frontend, proc-macro, Candle, generalized cache, and reusable Metal-runtime crates.
- `implementation_status` is `partial` and states a real gap rather than rounding up. The loader implements what a decoded envelope publishes. It cannot evaluate an applicability guard, a binding's accessible byte range, or a launch formula, because those rows are held in the envelope and reachable only through a `VerifiedArtifactProgram` no decode produces; it cannot say which buffer a binding slot addresses, because `BindingData` carries no value reference; and it cannot read a payload's entry symbol, because the payload-metadata section has no public parser. Each of those is a projection or encoding gap owned by [`carry-reconstructable-kernel-programs-in-the-neutral-envelope`](../../tickets/carry-reconstructable-kernel-programs-in-the-neutral-envelope.md), not by this record, and the loader refuses the cases it cannot resolve rather than approximating them.

## Alternatives considered

**Wait for the runtime proof to reach the boundary first.** The architecture contract's "until the proof reaches those boundaries" is a real condition, and the runtime proof had not produced envelope evidence when this was decided. Rejected because the condition is circular in this instance: the proof cannot reach the envelope boundary without a component that decodes and validates envelopes, so waiting for the evidence indefinitely postpones producing it. What breaks the circle is that the missing component is the *device-free* one, whose correctness does not depend on the proof having run.

**Put the loader in `tiler-artifact`.** Rejected because the two answer different questions. `tiler-artifact` decides what an artifact *is* and owns one authority over its bytes; a loader decides whether *this host* may execute *this* artifact, which requires a host's stated environment as an input. Folding the second into the first would put a host-relative decision inside the crate that must be host-independent to be a wire-format authority.

**Put the loader in `prototypes/serial-sum-run`.** Rejected because it would tie a backend-independent contract to one prototype executable, make it untestable without a device, and reproduce for the runtime exactly the situation ADR 0077 corrected for the driver — a component whose boundary is real but whose home makes the boundary unenforceable.

**Admit a full Metal runtime crate now.** Rejected: that is precisely the crate ADR 0056 withholds and ADR 0077 refuses to be precedent for. The device half stays in the proof executable until a runtime proof produces evidence about what a reusable one owes its consumers.

## Traceability

The [prototype crate layout research](../research/workspace/prototype-crate-layout-and-msrv.md) is the evidence that the crate set mechanically enforces Tiler's layer separation rather than being a packaging convenience, which is what makes admitting one a decision. The [target-neutral artifact envelope research](../research/artifacts/target-neutral-artifact-envelope.md) is the evidence behind what a decoded envelope publishes and therefore what a device-free loader can and cannot decide. [ADR 0051](0051-make-runtime-routing-commit-one-way.md) owns the one-way routing commit this crate implements, and [ADR 0043](0043-use-typed-phased-target-feasibility.md) owns the requirement that a declared target profile carry both a governed key and an exact descriptor, which is what makes compatibility a classification rather than a key comparison. The [architecture contract](../architecture.md) owns the packaging profile this record amends. The work records are [`admit-the-device-free-runtime-validation-crate`](../../tickets/admit-the-device-free-runtime-validation-crate.md) for this record and the admission, [`prototype-runtime-artifact-validation`](../../tickets/prototype-runtime-artifact-validation.md) for the loader's implementation, and [`route-the-runtime-proof-through-the-artifact-envelope`](../../tickets/route-the-runtime-proof-through-the-artifact-envelope.md) for removing the bypass.
