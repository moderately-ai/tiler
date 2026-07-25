---
id: record-delivered-numerical-realization
title: Record the delivered numerical realization in the artifact
status: done
priority: p1
dependencies: [select-numerical-contract-and-compose-feasibility, declare-metal-numerical-honourability]
related: [draft-target-honourable-numerical-contract-adr, prototype-artifact-program-model, accept-the-delivered-realization-artifact-surface, wire-the-delivered-realization-record-into-the-artifact, carry-the-honourability-fact-provenance-into-the-artifact-record]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, numerics, needs-tom]
---
ADR 0076 item 4. A produced artifact carries a first-class, **readable** record of the numerical realization actually delivered: the resolved contract complete over every dimension, each dimension's means of honouring it, the target facts relied on, and the identity of the profile that declared them.

A consumer comparing generated output against a CPU reference reads this record. It does not reconstruct it from the request, from the selected compiler flags, or from the target's name.

## Why flags cannot substitute — this is measured, not assumed

Under `-fmetal-math-mode=relaxed` the emitted module records `!"air.compile.fast_math_disable"` while every floating-point operation in it carries `reassoc nsz arcp contract afn`. The module-level flag is therefore not a faithful summary of the licences actually applied, and an artifact-side reader that inferred the delivered realization from it would read **the opposite of the truth**. That single measurement is the whole argument for a first-class record.

## Why identity is not enough either

`docs/artifact-abi.md` already puts the numerical contract and the exact flags into artifact *identity*, which is what makes two artifacts distinguishable. That is a different job. A digest is comparable and not readable: it lets a consumer detect that two artifacts differ, and tells it nothing about what either one means. This ticket adds the readable statement alongside the existing digest; it does not replace or duplicate the identity encoding, and it must not become a second authority over what identity commits to.

## What the record's content is fixed by

Because ADR 0076 item 5 forbids delivering anything other than the declared contract, the delivered realization always equals the declared one for any artifact that exists. So the record is **not** a channel for reporting a downgrade — there are none by construction. It is the evidence that no downgrade occurred, plus the means by which each dimension was honoured. The means is the part a caller cannot derive, and it changes what a reference comparison should expect from a dimension honoured by emulation rather than natively.

Do not design an "actual versus requested" shape. There is no divergence to report, and a schema that admits one would invite a future implementation to fill it in.

## Boundary — this needs Tom

`tiler-artifact` gained its first public content in `prototype-artifact-program-model`. This ticket adds a public numerical surface to it, which under ADR 0075 is Tom's to approve before it is accepted or merged. Build a tested implementation as a concrete draft, present the boundary as an atomic decision with alternatives, and pause — a tested implementation is not implicit approval of its public interface.

Apply the ADR 0074 conventions: typed non-erasing errors, opaque identities with `as_bytes()` and presentation-only `label()` accessors, domain-tagged and length-prefixed encodings with no ordinal dependence, a transactional builder with a consuming `build()`, no `pub` fields on the verified product. On `#[non_exhaustive]`, apply the amended convention 5 rather than the blanket rule: an enum an out-of-crate consumer maps *totally* must stay exhaustive, because a wildcard there makes a missed variant silently wrong instead of a build error. `tiler-artifact` already encodes `KernelType`, `AddressSpace`, and `BufferAccess` into identity cross-crate, which is the worked precedent for that judgement.

## Blocked 2026-07-25 — the declared dependency was satisfied; the real one is not

Attempted from `implementation/artifact`. The frontmatter listed only `select-numerical-contract-and-compose-feasibility`, which is `done`, so this ticket presented as ready. It is not, and ADR 0076 says so in its own words.

**Fact — ADR 0076's implementation boundary, item 4, evidence refresh 2026-07-24** (`docs/decisions/0076-declare-target-honourable-numerical-realizations.md:160`): "`tiler-artifact` now carries an artifact-program model and a bounded neutral envelope codec, and that codec already encodes the resolved contract complete over its dimensions — `NumericalFacts` and `ResourceRequirements` both write all four dimensions through exhaustive tag maps. What item 4 still owns is everything the contract's *values* do not supply: each dimension's means of being honoured, the target facts relied on, and the declaring profile's identity. **None of that exists, because none of it can before ticket 3 declares it.**"

**Fact — the contract-values half is indeed already carried.** `crates/tiler-artifact/src/program/codec/model.rs:176-183`: `NumericalFacts` carries `profile_key`, `canonical_arithmetic_nan_bits`, `input_subnormals`, `result_subnormals`, `contraction`, and `reassociation`. `EntryRef::numerical()` reads the same realization off a verified artifact. So of the four content items this ticket owes, the resolved contract is done and the declaring profile's identity is expressible today (`TargetProfileRef`, already public in `program::keys`).

**Fact — the means vocabulary does not exist.** Exact check: `grep -rn "SupportedExactly\|SupportedWithExactEmulation\|SupportedOnlyUnderDeclaredRelaxation\|Honourab\|Honorab" crates/` returns no match. The four outcomes are named in `docs/numerical-semantics.md` and in ADR 0076 item 3; no crate implements them.

**Inference — building the draft now would invent the vocabulary ticket 3 owns.** ADR 0076 item 3 fixes the means declaration as "a stated, versioned profile fact with the same provenance discipline `CapabilityFact` already carries — an availability phase, a validity scope, an authority, and the declaring profile's identity", and assigns it to `tiler-metal` (`declare-metal-numerical-honourability`, p0, todo, scopes `implementation/metal` + `contracts/artifacts`) composed by `tiler-compiler` (`compose-numerical-honourability-and-retire-the-strict-boolean`, p1, todo). Defining a parallel means enum in `tiler-artifact` first would create a second authority over the same terms — which ADR 0076 itself forbids at line 58: "This record must therefore not invent a vocabulary. Doing so would create a second authority over the same terms, which the documentation contract forbids."

The same applies with more force to "the target facts relied on": no target-neutral representation of a relied-upon target fact exists, and inventing one so the record has a field to fill would be exactly the producer-less placeholder this repository has repeatedly had to retract.

**What changed here.** The missing dependency edge on `declare-metal-numerical-honourability` was added, so the work graph stops advertising this ticket as ready. Nothing was implemented; no public surface was added to `tiler-artifact`.

**Trigger for reconsideration.** When `declare-metal-numerical-honourability` lands the means vocabulary and the target-fact shape, this ticket becomes a projection of them into the artifact record plus its identity encoding, and its `needs-tom` public-surface question becomes answerable with a concrete draft rather than an invented one.

## Re-checked 2026-07-25 from `implementation/artifact` — the trigger fired and the ticket is still blocked, on a different thing

`declare-metal-numerical-honourability` is `done`, so the stated trigger has fired. Re-running the blocked note's own check against `2305c4a`:

**Fact — the means vocabulary now exists.** `grep -rn "SupportedExactly\|SupportedWithExactEmulation\|SupportedOnlyUnderDeclaredRelaxation\|Honourab\|Honorab" crates/` is no longer empty. `crates/tiler-compiler/src/honourability.rs:178-198` defines all four means, `NumericalHonourabilityFact` carries them with the provenance discipline ADR 0076 item 3 required, and `feasibility.rs` composes them into the target-profile descriptor.

**Fact — it is not reachable from `tiler-artifact`, twice over.** `HonouringMeans` is `pub(crate)` (`honourability.rs:179`), so nothing outside `tiler-compiler` can name it; and `grep -n tiler-artifact crates/tiler-compiler/Cargo.toml` is empty while `tiler-artifact` does not and must not depend on `tiler-compiler`, so the dependency direction forbids reaching it even if it were public. The vocabulary landed in a crate the artifact layer cannot see.

**Inference — the blocked note's reasoning survives with a new subject.** Projecting the record into `tiler-artifact` today would still mean *restating* the four means there, which is the second authority ADR 0076 line 58 forbids, only now the first authority demonstrably exists rather than being pending. What changed is that the question is answerable: either the vocabulary is promoted out of `tiler-compiler` into a crate both depend on (`tiler-ir`), or the artifact layer receives it as an opaque governed key the way it receives every other identity it is not the authority for — and `HonouringMeans::key` already mints exactly such a key (`"supported-exactly"`, `"supported-with-exact-emulation"`, …). That is a real atomic decision with two live options and a dependency direction that decides it, rather than a gap.

## Decision — auto-resolved, because the first option does not survive

Recorded rather than escalated: exactly one of the two options above survives the architectural guardrails, so there is no choice left to put to Tom. The elimination is stated so it can be refuted rather than only the conclusion.

**Option A — move the vocabulary into `tiler-ir` — is eliminated by what `tiler-ir` is for.** `AGENTS.md` fixes it as the crate describing *what tensor operations mean, not how a device executes them*, and requires semantic/logical IR to stay distinct from physical schedules and target-aware choices. `HonouringMeans` says how a *target* delivers a numerical behaviour — whether a dimension is supported exactly, by exact emulation, and so on. It is target-honourability, which the same document places in "typed target profiles, physical properties, schedule alternatives, feasibility predicates, and cost models". Relocating it into the semantic IR to solve a visibility problem would densify a physical choice into the layer that must not carry one, and it would do so for the convenience of a sibling crate rather than for any semantic reason.

**Option B is what this workspace already does everywhere else, and the precedent is not an analogy — it is the same mechanism.** `crates/tiler-artifact/src/program/keys.rs:147-176` gives every opaque identity a `from_bytes` over the doc "the bytes are treated as opaque: this crate compares and encodes them, and never re-derives them locally." That ignorance is exactly what keeps the artifact layer consumer-agnostic, and it is why `TargetProfileDescriptorDigest`, the capability key, and the feasibility rule set key all arrive as bytes minted elsewhere. A delivered numerical realization is one more identity the artifact layer is not the authority for.

**It also satisfies the constraint that motivated the block.** ADR 0076 forbids a second authority restating the four means. An opaque key restates nothing: `tiler-artifact` can compare two keys for equality — which is the whole of what identity validation needs — without being able to interpret either. The means stay in the one crate that decides them.

**What this does not license.** The key must be minted by `HonouringMeans::key` and carried, never reconstructed, defaulted, or inferred from a neighbouring field. An artifact holding no realization key is `Unknown` and must reject rather than assume — the same third-class treatment `carry-the-dtype-on-the-metal-subnormal-flush-fact` established for an unstated dtype, and for the same reason: an absent fact that reads as a permissive one is how a wrong tensor gets delivered quietly.

**What would reopen this.** A consumer of the artifact that must *reason over* the means rather than compare them — for example, choosing between two artifacts by how each honours a dimension. That consumer would need the vocabulary, and the right response would be to give the artifact layer a typed view of a key it still does not mint, not to relocate the authority.

**Not attempted.** This ticket adds a public numerical surface to `tiler-artifact`, which its own "Boundary — this needs Tom" section reserves under ADR 0075, and no such approval exists. The homing decision above should be presented as the atomic question when it is picked up.

## Outcome

**The draft is built and staged crate-private; the surface is Tom's.** The record is implemented and tested in `crates/tiler-artifact/src/program/realization.rs`, as a private `mod realization;` whose items are all `pub(crate)` and none re-exported, under ADR 0074 convention 7 with the required `#![allow(dead_code, reason = …)]`. **No public API was merged.** `accept-the-delivered-realization-artifact-surface` is the review gate, and `wire-the-delivered-realization-record-into-the-artifact` depends on it.

### Fact — the record's shape, and what each field is evidence of

`DeliveredNumericalRealization` holds one `TargetProfileRef` and four `HonouredDimensionFact`s, one per behaviour dimension of `tiler_ir::schedule::NumericalRealization`. Each fact holds an opaque `HonouringMeansKey` and the `AvailabilityPhase` the declaration was readable from.

- The **means key** is the part a caller cannot derive. It is bytes `HonouringMeans::key` mints, never re-derived or interpreted here, so the four terms stay in the one crate that decides them.
- The **availability phase** is `tiler_ir::program::abi::AvailabilityPhase`, the same type `tiler-compiler` imports, so it is one shared vocabulary rather than a restatement.
- The **profile reference** names the authority the means came from. It is the artifact's evidence that the declaration was made rather than assumed.

**One record per artifact, not per variant**, and that is derived rather than chosen: `ArtifactProgramBuilder::check_subject` already rejects a variant whose numerical contract or target profile differs from its siblings (`NumericalContractMismatch`, `TargetProfileMismatch`), so both are artifact-wide facts and a per-variant record would be a copy that has to be kept in agreement with three others.

**The record restates no behaviour.** ADR 0076's own evidence refresh says the contract's values are already carried and what item 4 still owns is the means, the target facts, and the declaring profile. So the behaviour each means was declared for *is* the artifact's resolved contract on that dimension, read from the artifact. There is no second copy that can disagree with the first, no "actual versus requested" shape, and nothing here is a second authority over what identity commits to.

### Fact — completeness is structural and absence rejects

Within a record, one field per dimension makes a partial record unrepresentable; `DeliveredNumericalRealization::honoured` is therefore total and there is no dimension a reader can ask about and get nothing for. `DeliveredRealizationBuilder::build` is what refuses to produce a partial one, naming the first undeclared dimension in canonical order, and `declare` refuses a restatement rather than taking it last-wins — restating is a dropped fact, not a correction.

For a whole record that is absent, `require_recorded` is the only reader of an optional one and returns `UnrecordedRealization`. There is no `Default`, no `From`, and no accessor that manufactures a means, and `UnrecordedRealization` is deliberately *not* a variant of `DeliveredRealizationError`, so a caller matching on a malformed record cannot absorb an absent one. That is the third class `carry-the-dtype-on-the-metal-subnormal-flush-fact` established, for the same reason.

### Inference — a means readable only after packaging is refused, and the boundary is not invented

`declare` rejects any `AvailabilityPhase` later than `ArtifactEvidence`. This is the exact complement of the line the artifact layer already draws from the other side: `ArtifactBuildError::NonDeferredPredicatePhase` rejects a *deferred* predicate below `LiveDevicePreflight`, because a predicate decided at packaging is not deferred. A means declared readable only from live preflight onward was not relied on to produce these bytes, so recording it as delivered would claim evidence that does not exist.

### Fact — the two claims in the dispatch brief that did not survive checking

- **The means is not received through `keys.rs`'s `opaque_identity!` macro.** Doing so requires a new `ArtifactKeyKind` variant, and that enum is `pub`, so the "no public API" instruction and the reuse instruction were in direct conflict. The module carries its own bounded wrapping constructor and its own typed key error instead. `MAX_HONOURING_MEANS_KEY_BYTES` is deliberately not `MAX_OPAQUE_IDENTITY_BYTES`, because that constant's own documentation warns against sharing one bound across identities that share only a shape.
- **No presentation `label()` exists, and adding one would be wrong.** ADR 0074 convention 2 offers a label so a wide digest can be read; a means key is already text a reader can render, so a label digesting it would make the record *less* readable. The convention is about the role of the value, and this value has the readable role already.

### Deliberately not done, each with its blocker

- **Nothing constructs or reads a record from outside the crate.** The constructor and the readers are public artifact surface. `accept-the-delivered-realization-artifact-surface` gates it, `wire-the-delivered-realization-record-into-the-artifact` does it.
- **`canonical_bytes` is not folded into `CanonicalArtifactProgramIdentity`, and no envelope section carries the record.** Same blocker, same ticket; the identity fold also moves every pinned artifact identity, which is work that must be done on a merged tree rather than on a branch.
- **The fact's authority and validity scope are not carried, and neither is the compiler build and execution environment ADR 0076 item 3 requires the scope to identify.** `FactAuthority` and `FactValidityScope` are `pub(crate)` in `tiler-compiler` with no minting API, and `grep -rn "compiler build\|CompilerBuild\|ExecutionEnvironment\|execution environment" crates/tiler-compiler/src/ crates/tiler-metal/src/` is empty, so nothing exists to be carried. No field is reserved for them. `carry-the-honourability-fact-provenance-into-the-artifact-record` owns it.
- **The record is keyed by dimension alone, with no arithmetic type.** `carry-the-dtype-on-the-metal-subnormal-flush-fact` recorded on ADR 0076 boundary item 3 that a per-dimension declaration cannot express a row where `InputSubnormals` is `SupportedExactly` for `f16` and `Unsupported` for `f32`. `tiler_compiler::honourability::NumericalDimension` is dtype-free today, so this record projects a dtype-free authority; when the shared form carries the arithmetic type, this record's key widens with it, and the widening is a build error at `NumericalDimension::tag` and `CANONICAL_DIMENSIONS` rather than a silent inheritance.
- **`NumericalDimension` now exists in two sibling crates.** Both are projections of one authority — `tiler_ir::schedule::NumericalRealization`'s four behaviour fields, which `tiler-artifact` already projects field by field in its envelope's `NumericalFacts` — rather than two authorities over the means, which stays opaque. The durable fix is one dimension vocabulary in `tiler-ir`, which is where the record they name lives; it was not done here because `implementation/ir` and `implementation/compiler` are outside this ticket's scope.
