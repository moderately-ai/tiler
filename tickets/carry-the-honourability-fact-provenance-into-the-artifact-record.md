---
id: carry-the-honourability-fact-provenance-into-the-artifact-record
title: Carry the honourability fact provenance into the artifact record
status: todo
priority: p1
dependencies: [express-metal-honourability-in-the-shared-form]
related: [record-delivered-numerical-realization, name-the-compiler-and-environment-in-adr-0076-target-facts, record-metal-runtime-compiler-provenance-gap]
scopes: [implementation/metal, implementation/compiler, contracts/numerics]
shared_scopes: []
paths: []
tags: [implementation, artifact, compiler, numerics, provenance]
---
## User-visible outcome

The target/profile authority can produce structured provenance identifying the authority, validity scope, compiler build, and execution environment for each numerical honourability fact, and the compiler can validate and carry that provenance with selected evidence. A later artifact translation can therefore remain readable without treating the compiler as the measured fact's authority or requiring a consumer to decode opaque scope bytes.

ADR 0076 item 3 fixes a numerical honourability declaration as "a stated, versioned profile fact with the same provenance discipline `CapabilityFact` already carries — an availability phase, a validity scope, an authority, and the declaring profile's identity", and adds that the validity scope "must identify which compiler build and which execution environment the declared behaviour was measured on". Item 4 states that the artifact record "inherits that requirement rather than adding one": a record naming a delivered realization without naming the compiler that produced it is not readable in the sense item 4 requires.

`record-delivered-numerical-realization`'s historical draft carries two of the four. `HonouredDimensionFact` holds the means and the availability phase; `DeliveredNumericalRealization` holds the declaring profile identity. The authority and the validity scope are absent, and so is the compiler-and-environment identification the scope must supply.

## Facts, so the gap is not mistaken for an oversight

**Fact — the authority and validity scope are unreachable, not merely unused.** `crates/tiler-compiler/src/feasibility.rs` declares `FactAuthority` and `FactValidityScope` `pub(crate)`, and each offers only a `tag()` for the profile descriptor's own encoding. The compiler can carry these facts but is not automatically their authority: the Metal/profile declarer is the source of the measured statement. A compiler-minted opaque key would both mis-site authority and leave a future artifact reader unable to identify the compiler build and environment without a second recognizer.

**Fact — no type in the workspace expresses a compiler build or an execution environment as a target fact.** Exact check: `grep -rn "compiler build\|CompilerBuild\|ExecutionEnvironment\|execution environment" crates/tiler-compiler/src/ crates/tiler-metal/src/` returns nothing. `name-the-compiler-and-environment-in-adr-0076-target-facts` is `done` and holds `contracts/decisions` only — it added the requirement to the ADR and implemented nothing, which is correct for its scope and leaves this open.

**Measurement the requirement rests on**, recorded on that ticket: one Apple host resolves three distinct Metal compiler builds at one instant — offline `xcrun metal` `metalfe-32023.883` from the Xcode MetalToolchain asset, the macOS host runtime compiler `metalfe-32023.921` from `GPUCompiler.framework`, and the booted iOS 26.0 Simulator runtime compiler `metalfe-32023.830.1` — on Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113. They version independently, so a fact naming only "Metal on Apple silicon" names no compiler.

## What closes this

The target/profile declarer produces one structured, versioned provenance value that identifies the measured-fact authority, validity scope, compiler build, and execution environment. The shared honourability declaration validates it; the compiler carries it unchanged with selected evidence and exposes an exact proposed borrowed view for the later public-boundary review. Opaque display keys may supplement identity but cannot replace the structured fields a reader must interpret.

This ticket does not modify the artifact record or encode provenance into artifact identity. `redesign-the-delivered-realization-record-from-typed-evidence` owns the total checked representation and review packet; `wire-the-delivered-realization-record-into-the-artifact` owns production translation, encoding, and identity after Tom accepts the boundary.

## Graph maintenance

- The structured fact is produced at the target/profile declaration boundary and carried by the compiler; do not make the compiler the measured fact's authority merely because it owns feasibility.
- If you find yourself adding a field the producer cannot fill yet, stop — this ticket's own closing text forbids producer-less placeholders. Record what the producer would need instead, on this ticket.
- When the compiler-build/environment fact type first exists, tell `record-metal-runtime-compiler-provenance-gap` (related) — its gap is the same fact from the runtime side.
- This producer work precedes `redesign-the-delivered-realization-record-from-typed-evidence`. The former edge to `wire-the-delivered-realization-record-into-the-artifact` was backwards: wiring cannot precede the provenance the required record must carry.
