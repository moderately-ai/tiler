---
id: record-the-compilation-selection-in-target-measurement-provenance
title: Record the compilation selection in target measurement provenance
status: done
priority: p2
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, declare-the-bf16-rows-on-the-authoritative-metal-profile, measure-macos-apple9-bf16-under-unified-msl4-profile, refuse-unknown-fact-source-provenance-schemas-in-artifact-decode, carry-required-compilation-selection-identity-on-compile-profile-contexts, split-metal-profile-measurement-sources-by-compilation-selection]
scopes: [implementation/compiler, implementation/ir, implementation/artifact, implementation/build, implementation/metal-aot, contracts/decisions, contracts/numerics, contracts/artifacts, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, provenance, identity, numerics, decision, public-boundary]
---
## User-visible outcome

Measured target facts cannot silently claim the authority of a different compilation selection. A compile-profile measurement context will require one exact backend-owned selection identity; there is no absent, default, inferred, or fallback selection.

## Source-first audit — 2026-08-11

**Fact — verified.** `TargetCompileProfileMeasurementSource::new` in `crates/tiler-compiler/src/target.rs` accepts a producer and `TargetMeasurementContext` values. The underlying `MeasurementContext` in `crates/tiler-ir/src/numerics.rs` canonically carries compiler builds and the execution environment only. It cannot represent the requested SDK/platform/triple, language standard, optimization and numerical flags, or linker flags.

**Measurement — corrected.** The two retained 2026-07-31 records do not differ *exactly* in `-std` and requested target as this ticket previously claimed. They also differ in schema, date, population, and emitted AIR triple. The narrower evidence is load-bearing: every field the present provenance vocabulary can represent can be identical while the compilation selections differ. Given the same producer identity, those selections therefore collapse to the same provenance bytes. The record files themselves do not construct provenance, so the old unconditional descriptor-equality sentence was also imprecise.

**Fact — current production relevance narrowed.** The authoritative declaration now uses the unified 2026-08-02 MSL 4/macOS 26 F32+BF16 record, so the old MSL 3.1-versus-MSL 4 mixing hazard is not an active defect in that declaration. The public provenance schema remains unable to express the distinction.

**Fact — verified.** `tiler-metal-aot::CompileRequest` is the backend owner of the source, SDK/platform target, optimization policy, numerical policy, and the exact ordered compiler and linker flags. `CompilationIdentity` already demonstrates that the SDK selector and invocation flags are identity-bearing, but it also includes source and resolved toolchain facts that belong to different provenance fields.

**Fact — hidden delivery defect.** The artifact `decode_provenance` path reads an incoming provenance schema and discards it before reconstructing a current `FactSourceProvenance`. That can normalize a foreign schema instead of refusing it. The schema decoder must be healed before the new grammar is admitted.

**Fact — source population must split.** `tiler-build` currently shares one measured source across grid, cost, dispatchability, and numerical rows. The retained grid invocation did not select the same explicit O2/safe/precise/contract-off policy used by the projected cost and numerical evidence. Equal compiler builds and execution environments do not make those compilation selections equal. Rows must cite only contexts whose exact selection produced them.

## Accepted decision — 2026-08-11

Tom accepted in this conversation a required, backend-opaque, exact compilation-selection identity on every compile-profile measurement context.

- Compile-profile and runtime/device measurement contexts remain distinct typed routes. A runtime context does not invent compile flags.
- The compile-profile constructor requires exactly one nonempty selection. There is no `Option`, empty sentinel, default, inference from the target profile, or governed fallback.
- The producing backend owns the canonical bytes. The compiler and IR only validate the generic envelope, length-frame, compare, retain, and expose them; they never learn Metal vocabulary.
- Metal derives the value beside `CompileRequest::compile_flags()` and `link_flags()`. It includes the SDK selector, requested platform/target, and exact ordered compile/link selection. It excludes source text and resolved toolchain facts because those have separate authorities.
- Retain exact canonical bytes rather than only a digest. This preserves mathematical injectivity and readable evidence without creating a second hash authority. The host cost is linear in a small profile-construction record, outside kernel execution and compilation search.
- Do not invent a narrow independent limit. Empty input is refused early; the existing complete-profile descriptor envelope is the coarse alpha ceiling and cumulative authority. Any later tighter cap requires a measured consumer/resource need.
- A fact supported by multiple different selections uses separate measurement contexts/sources. A row never silently claims a set of selections.
- The identity is provenance only. It does not choose a backend, retry compilation, or provide any fallback.

## Ranked alternatives

1. **Accepted: required exact backend-owned canonical bytes per compile-profile context.** Best correctness and strictness; negligible bounded profile-construction cost; preserves backend neutrality and remains extensible.
2. **A required fixed digest of those complete bytes.** Fixed-size and fast, but introduces collision and digest-governance questions while making evidence less readable.
3. **A generic compiler-owned flag map.** Readable but inevitably incomplete and assigns backend vocabulary to the wrong layer.
4. **Attach selection to each compiler build or whole source.** Wrong multiplicity: one build can serve several selections, and one source population may require several contexts.
5. **Reuse full AOT compilation identity.** Conflates source and resolved toolchain evidence with selection.
6. **Leave the profile silent or use declaration prose only.** Rejected: two different authorities can remain canonically indistinguishable.

## Identity and delivery boundary

The implementation must rederive the exact domain-step population from source. The expected minimum is `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` 3 to 4 and `DELIVERED_REALIZATION_DOMAIN` v2 to v3, because delivered-realization decoding owns an unframed provenance grammar. Outer descriptors that length-frame a self-versioned provenance blob should not be stepped without a separate grammar change, although their values and downstream request, artifact, envelope, and cache pins may move transitively.

Delivery is split deliberately:

- `refuse-unknown-fact-source-provenance-schemas-in-artifact-decode` heals the decoder first.
- `carry-required-compilation-selection-identity-on-compile-profile-contexts` owns the generic and Metal-derived public carrier plus identity migration.
- `split-metal-profile-measurement-sources-by-compilation-selection` repairs the current authoritative source population and its ledger/pins.

## Outcome

Decision recorded and implementation separated from the decision. No production type, identity, profile, or artifact byte changed in this ticket.
