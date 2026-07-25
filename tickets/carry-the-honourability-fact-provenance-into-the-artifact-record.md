---
id: carry-the-honourability-fact-provenance-into-the-artifact-record
title: Carry the honourability fact provenance into the artifact record
status: todo
priority: p2
dependencies: [wire-the-delivered-realization-record-into-the-artifact]
related: [record-delivered-numerical-realization, name-the-compiler-and-environment-in-adr-0076-target-facts, record-metal-runtime-compiler-provenance-gap]
scopes: [implementation/artifact, implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, artifact, compiler, numerics, provenance]
---
ADR 0076 item 3 fixes a numerical honourability declaration as "a stated, versioned profile fact with the same provenance discipline `CapabilityFact` already carries — an availability phase, a validity scope, an authority, and the declaring profile's identity", and adds that the validity scope "must identify which compiler build and which execution environment the declared behaviour was measured on". Item 4 states that the artifact record "inherits that requirement rather than adding one": a record naming a delivered realization without naming the compiler that produced it is not readable in the sense item 4 requires.

`record-delivered-numerical-realization`'s draft carries two of the four. `HonouredDimensionFact` holds the means and the availability phase; `DeliveredNumericalRealization` holds the declaring profile identity. The authority and the validity scope are absent, and so is the compiler-and-environment identification the scope must supply.

## Facts, so the gap is not mistaken for an oversight

**Fact — the authority and validity scope are unreachable, not merely unused.** `crates/tiler-compiler/src/feasibility.rs:291` and `:321` declare `FactAuthority` and `FactValidityScope` `pub(crate)`, and each offers only a `tag()` for the profile descriptor's own encoding. `tiler-artifact` and `tiler-compiler` are siblings that each depend only on `tiler-ir`, so no visibility change alone reaches them; a key-minting API on the compiler side is what a sibling can consume, the way `HonouringMeans::key` already serves the means.

**Fact — no type in the workspace expresses a compiler build or an execution environment as a target fact.** Exact check: `grep -rn "compiler build\|CompilerBuild\|ExecutionEnvironment\|execution environment" crates/tiler-compiler/src/ crates/tiler-metal/src/` returns nothing. `name-the-compiler-and-environment-in-adr-0076-target-facts` is `done` and holds `contracts/decisions` only — it added the requirement to the ADR and implemented nothing, which is correct for its scope and leaves this open.

**Measurement the requirement rests on**, recorded on that ticket: one Apple host resolves three distinct Metal compiler builds at one instant — offline `xcrun metal` `metalfe-32023.883` from the Xcode MetalToolchain asset, the macOS host runtime compiler `metalfe-32023.921` from `GPUCompiler.framework`, and the booted iOS 26.0 Simulator runtime compiler `metalfe-32023.830.1` — on Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113. They version independently, so a fact naming only "Metal on Apple silicon" names no compiler.

## What closes this

The declaring authority mints an opaque key for the fact's authority and validity scope, and the validity scope identifies the compiler build and execution environment measured. The artifact record carries them per dimension beside the means it already carries, and encodes them into the record's canonical bytes. No field is reserved before its producer exists: a field a producer cannot fill is the producer-less placeholder this repository has repeatedly had to retract, which is exactly why the draft omits them rather than defaulting them.
