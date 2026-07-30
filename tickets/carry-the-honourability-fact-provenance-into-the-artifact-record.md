---
id: carry-the-honourability-fact-provenance-into-the-artifact-record
title: Carry structured target-fact provenance through declaration and selection
status: done
priority: p1
dependencies: []
related: [record-delivered-numerical-realization, name-the-compiler-and-environment-in-adr-0076-target-facts, record-metal-runtime-compiler-provenance-gap, carry-structured-provenance-through-numerical-rejections]
scopes: [implementation/metal, implementation/compiler, implementation/build, contracts/numerics, project/tickets]
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

**Fact — no shared target-fact type expresses both compiler builds and their execution environments.** `tiler-metal-aot::ArtifactProvenance` has a `CompilerFingerprint` for offline artifact production, but it is produced after compiler selection and does not identify a runtime compiler or pair a build with the environment in which a numerical behaviour was measured. The exact compiler-side check remains `rg "CompilerBuild|ExecutionEnvironment" crates/tiler-compiler/src crates/tiler-metal/src`, which returns no declaration before this ticket. `name-the-compiler-and-environment-in-adr-0076-target-facts` is `done` and holds `contracts/decisions` only — it added the requirement to the ADR and implemented nothing, which is correct for its scope and leaves this open.

**Measurement the requirement rests on**, recorded on that ticket: one Apple host resolves three distinct Metal compiler builds at one instant — offline `xcrun metal` `metalfe-32023.883` from the Xcode MetalToolchain asset, the macOS host runtime compiler `metalfe-32023.921` from `GPUCompiler.framework`, and the booted iOS 26.0 Simulator runtime compiler `metalfe-32023.830.1` — on Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113. They version independently, so a fact naming only "Metal on Apple silicon" names no compiler.

## What closes this

The target/profile declarer produces one structured, versioned provenance value that identifies the measured-fact authority, validity scope, compiler build, and execution environment. The shared honourability declaration validates it; the compiler carries it unchanged with selected evidence and exposes an exact proposed borrowed view for the later public-boundary review. Opaque display keys may supplement identity but cannot replace the structured fields a reader must interpret.

A governed guarantee and a measurement are different evidence bases. The prototype target-neutral baseline is a normative compiler-independent guarantee: its source names the versioned governing authority and guarantee, and does not pretend that one observed compiler/environment row proved it. A measured source cannot take that arm; it supplies one or more nonempty compiler-build sets paired with exact execution environments. The later Metal adapter owns those real rows.

This ticket does not modify the artifact record or encode provenance into artifact identity. Its historical ID predates that split. `redesign-the-delivered-realization-record-from-typed-evidence` owns the total checked representation and review packet; `wire-the-delivered-realization-record-into-the-artifact` owns production translation, encoding, and identity after Tom accepts the boundary.

## Proposed borrowed boundary for review

The compiler-side carry stays private. The later public facade belongs on `PlanAlternative`, because provenance qualifies one selected plan rather than the compilation as a whole:

```rust
impl PlanAlternative<'_> {
    pub fn honoured_numerical_facts(
        &self,
    ) -> impl ExactSizeIterator<Item = HonouredNumericalFact<'_>>;
}
```

`HonouredNumericalFact<'a>` is a copyable borrowed view with private storage and typed accessors for `NumericalDimension`, `ArithmeticType`, `DimensionBehaviour`, `HonouringMeans`, `AvailabilityPhase`, `FactAuthority`, `FactValidityScope`, the declaring profile key, and `FactSourceView<'a>`. `FactSourceView` exposes the provenance schema version, a borrowed `VersionedIdentityView` for the fact authority, and exactly one `FactEvidenceView`: either `GovernedGuarantee(VersionedIdentityView)` or `Measurement(ExactSizeIterator<MeasurementContextView>)`. Each measurement context exposes an exact-size iterator of `CompilerBuildView` plus one `ExecutionEnvironmentView`; each compiler build exposes a borrowed `CompilerBuildRoleView`, implementation key, version, and optional build; the environment exposes platform key, platform version/build, architecture key, and hardware description. The role view keeps the provider-defined role identity borrowed too, so the facade's no-owned-record statement is literal.

The facade does not expose `Arc`, vectors, constructors, canonical encoders, checked-profile internals, or an owned record. It is an allocation-free inspection subview: a consumer can read typed selected evidence without decoding opaque bytes or forging a compiler-verified fact. Borrowing ties the view to the owning `Compilation` allocation, not uniquely to one `PlanAlternative`, so this iterator alone is deliberately not the artifact translation authority. The later total boundary must consume `PlanAlternative` directly or a whole plan-scoped view that cross-checks policy subjects, all-dimension `Required`/`NotRequired` coverage, obligation and execution-locus associations, and this evidence pool together. The governed common roles plus a versioned provider-defined role prevent a future toolchain from collapsing an unknown role into the nearest current one.

## Implementation outcome

Commits `a12f709`, `2515db7`, and `cad3e43` add the private structured source vocabulary, validate it at checked-profile admission, canonicalize and deduplicate source/declaration tables, carry the exact checked fact through `ProvenEvidence` and `SelectedPlan`, and bind the complete source plus declaring profile into selected-plan identity. Governed guarantees remain distinct from measurements; measured sources require nonempty compiler-build sets paired with exact environments; descriptor v4 and request/selection identity pins were rebaselined from the resulting canonical bytes.

The malformed-source and build/environment identity checks were each perturbed and observed failing before restoration. Targeted compiler tests pass 418/418 with one skipped; the final workspace gate passes 1,300 debug tests, all doc tests, 491 release tests, rustdoc, ticket lint, and shellcheck. Two clean detached reviews accepted the implementation and the cached canonical-key optimization without findings.

## Graph maintenance

- The structured fact is produced at the target/profile declaration boundary and carried by the compiler; do not make the compiler the measured fact's authority merely because it owns feasibility.
- This is a prerequisite of caller-authored target profiles: an external measured Metal fact cannot honestly use the current compiler-minted `GovernedProfile` and `PortableProfile` provenance.
- After this lands, `admit-a-caller-declared-target-profile` owns the immutable checked declaration boundary, and `express-metal-honourability-in-the-shared-form` owns the exhaustive `tiler-build` adapter from Metal facts into that boundary.
- If you find yourself adding a field the producer cannot fill yet, stop — this ticket's own closing text forbids producer-less placeholders. Record what the producer would need instead, on this ticket.
- When the compiler-build/environment fact type first exists, tell `record-metal-runtime-compiler-provenance-gap` (related) — its gap is the same fact from the runtime side.
- `carry-structured-provenance-through-numerical-rejections` owns the adjacent negative path discovered here: the current `UnhonouredDimension` reconstructs a scalar summary and drops the refusing fact before explain. This ticket closes only the selected-evidence path it states.
- This producer work precedes `redesign-the-delivered-realization-record-from-typed-evidence`. The former edge to `wire-the-delivered-realization-record-into-the-artifact` was backwards: wiring cannot precede the provenance the required record must carry.
