---
schema: "tiler-doc/v1"
id: "ADR-0074"
kind: "decision"
title: "Use explicit conformance conventions for public Tiler APIs"
topics: ["api", "rust", "verification", "identity", "governance"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir", "tiler.contract.architecture"]
evidence: ["tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.extensions.semantic-foundation-api-v2"]
refines: ["ADR-0071", "ADR-0072"]
ticket: "draft-public-api-conventions-adr"
---

# 0074: Use explicit conformance conventions for public Tiler APIs

**Status:** accepted

## Context

Tiler's landed Rust surfaces already share a recognizable shape, but no document states it. Each new surface therefore rediscovers the shape by imitation and re-litigates it by per-case review. This record proposes writing the shape down so that a surface which conforms needs no bespoke design debate, and only a genuine deviation needs a decision.

The evidence is the merged code at commit `b642007`, not theory.

**Fact — the shape is already pervasive.** Every canonical *byte* identity in the workspace is an opaque newtype over `Vec<u8>` or `Box<[u8]>` whose storage is private and whose only reader is `as_bytes()`: `CanonicalIndexRegionIdentity` (`crates/tiler-ir/src/index/model.rs`), `CanonicalScheduledRegionIdentity` (`crates/tiler-ir/src/schedule/model.rs`), `SemanticGraphIdentity` and the four-subject `SemanticIdentity` bundle (`crates/tiler-ir/src/semantic/identity.rs`), `SemanticOccurrenceIdentity`, `NumericalContractIdentity`, `RefinementContentIdentity`, and `IndexRefinementIdentity` (`crates/tiler-compiler/src/legality.rs`), `CanonicalReferenceRegistryIdentity` and `CanonicalScalarReferenceRegistryIdentity` (`tiler-reference`), and the crate-internal `RegionContentIdentity` and `RegionOccurrenceIdentity` (`region.rs`), `FusionLegalityContentIdentity` and `FusionLegalityIdentity` (`fusion_legality.rs`), `RegionCoverIdentity` (`cover.rs`), `ImplementationProposalIdentity` (`frontier.rs`), and `SelectedPlanIdentity` and `SelectedPortfolioIdentity` (`selection.rs`). Each of the encoders that derives one opens with a versioned domain tag of the form `b"tiler.<subject>.v<N>\0"` — NUL-terminated at every site but one, noted below — and the canonical bytes carry `u64` length prefixes before variable-length runs.

**Fact — no error type in any library signature erases its cause.** The workspace depends on neither `anyhow` nor `thiserror`; `Box<dyn Error>` appears only inside tests and doctests. Failure kinds are hand-written enums with structured payloads, `#[non_exhaustive]`, and a `source()` that preserves the underlying typed error — `IndexBuildError::ScalarAuthority(Arc<ScalarRegistryError>)` in `crates/tiler-ir/src/index/error.rs` is representative. `CheckedBuildError<Admission, Verification>` (`crates/tiler-ir/src/convenience.rs`) is generic over each layer's own two concrete error types precisely so a shared convenience does not collapse them into one lossy error.

**Fact — construction discipline is enforced by the type system, not by review.** `IndexRegionBuilder::build(self)` consumes the draft and returns `VerifiedIndexRegion` or an `IndexRegionBuildError` carrying the intact builder and every deterministic diagnostic. `IndexRegionBuilder::build_with` delegates to the shared `build_checked` helper, which scopes the mutable draft to an `FnOnce(&mut Self)` closure so the closure cannot reach the consuming verifier. Two `trybuild` compile-fail fixtures pin the guarantee: `crates/tiler-ir/tests/index-region/fail/forge_verified.rs` fails with "cannot construct `VerifiedIndexRegion` with struct literal syntax due to private fields", and `crates/tiler-ir/tests/index-region/fail/closure_cannot_build.rs` fails with E0507 because `build` takes `self`.

**Fact — the known gap is a convention gap, and it is unevenly distributed.** `#[non_exhaustive]` is used 74 times across `tiler-ir`, `tiler-compiler`, and `tiler-reference` — on essentially every public error enum and view enum — and zero times in `crates/tiler-ir/src/schedule/model.rs`, `crates/tiler-ir/src/schedule/numerics.rs`, or anywhere in `tiler-metal-aot`, whose enums and output records are documented as bounded-profile placeholders that will grow. That is a rule applied consistently where it was seen and dropped where it was not. The remediation is owned by `harden-public-enums-non-exhaustive`. A companion gap — canonical encodings that discard a facet only because its enum has one inhabited variant today — is owned by `extend-canonical-identity-encodings-for-reserved-variants`.

**Claim from the originating ticket, not independently verified here.** The ticket states that over the same period per-case public-boundary review caught "essentially one substantive issue", the missing `#[non_exhaustive]`. The gap itself is verified above; the claim about what the whole review history did and did not surface is a summary of work outside this repository's recorded artifacts and is recorded as the ticket's, not as a finding of this ADR. Nothing here depends on it.

**Inference.** A rule that is discoverable only by reading sibling modules is caught systematically by a written convention and only by luck in human review. That asymmetry, not any single defect, is the argument for this record.

### A premise from the ticket that does not survive checking

The originating ticket says four independent agents converged on the private-draft authority shape "without being instructed to". The artifact is real: `crates/tiler-compiler/src/{explain,feasibility,fusion_legality,cover,frontier,selection}.rs` are each a crate-private `mod` whose items are `pub(crate)` and which opens with a module-level `#![allow(dead_code, reason = "…")]` naming why the surface is not yet reachable from `compile()`.

**Fact.** The convergence was not independent, and each agent said so in writing. The `prototype-fusion-legality-and-numerical-proof` outcome records the pattern as "matching `explain.rs`'s private-draft precedent"; `prototype-physical-implementation-frontier` records "matching the reviewed-draft posture of the sibling `feasibility` and `fusion_legality` authorities"; `prototype-region-cover-enumeration` records "mirroring the sibling `fusion_legality`/`frontier` private drafts"; and `prototype-complete-physical-plan-selection` records "mirroring the `cover`/`frontier`/`fusion_legality` convention".

**Inference.** The mechanism was imitation of an unwritten precedent found in the tree, not independent derivation. That is a weaker claim about agent judgement and a *stronger* argument for this ADR: a convention propagated only by imitation degrades silently the first time a worker reads a non-conforming sibling, and it gives a reviewer no citable rule. This record therefore does not rest on the "independent convergence" premise, and the premise should not be repeated elsewhere.

## Decision

**Proposal.** A public Tiler API — any item reachable from a `pub` module of a workspace crate — satisfies the following conventions. Each is stated so that conformance can be checked by reading the item, and a deviation is a decision to be argued rather than an oversight.

### 1. Errors are typed and non-erasing

Distinct failure kinds are distinct variants of a concrete enum. A public fallible function returns that concrete type, never `Box<dyn Error>`, an erased trait object, or a string. A variant carries the structured data a caller needs to react — the rejected entity, the exhausted resource with its attempted and permitted quantities, the expected and actual arity — not a preformatted message. When a failure wraps a lower-layer error, `Error::source()` returns that error with its own type preserved.

A helper that composes two layers is generic over both concrete error types rather than unifying them. `CheckedBuildError<Admission, Verification>` is the reference form: it keeps an insertion-time admission rejection and a whole-object verification failure structurally distinct for every layer that adopts it.

### 2. Identity types are opaque and expose canonical bytes; short digests are presentation-only

A canonical identity is a newtype whose byte storage is private and whose only public reader is `as_bytes()`. Equality, ordering, hashing, dedup, and cache keying use the canonical bytes.

An identity a crate *derives* has no public constructor: it is produced only by the encoder that establishes what it means, so no caller can assemble one that names a subject no verifier examined. An identity a crate *receives* at a boundary may have an explicit wrapping constructor, provided the constructor and the type document that the bytes are treated as opaque and are never re-derived locally. `SemanticOccurrenceIdentity::from_bytes` and `NumericalContractIdentity::from_key` in `crates/tiler-compiler/src/legality.rs` are the two present instances, and both say so: refinement "treats it as opaque bytes: it is the *selected semantic source* the region is bound to, never re-derived here". A wrapping constructor is a statement that this crate is not the authority for that subject, and it must not be used to shortcut an identity the crate is the authority for.

A surface may also offer a short bounded label for explain output and diagnostics. That label is presentation-only and is never an equality or dedup input. `RegionContentIdentity::key()` states the rule in its own doc comment: "The label is a digest of the canonical bytes and is presentation only. Equality decisions always use `as_bytes`."

**Correction to the ticket's shorthand.** The method name `key()` is overloaded in the tree and must not be used as the marker for this rule. In `tiler-compiler` (`region.rs`, `cover.rs`, `selection.rs`) `key()` returns a `String` digest label. In `tiler-ir` (`index/scalar.rs`, `index/model.rs`, `semantic/registry.rs`, `semantic/operation.rs`, `semantic/interface.rs`) `key()` returns a borrowed *stable semantic key* — `&ScalarOpKey`, `&OpKey`, `&OutputKey` — which is meaning, is compared, and is encoded into identity. The convention is about the role of the value, not the spelling of the accessor. A new surface that names a digest label `key()` inherits a real collision hazard with the semantic-key accessors, which the naming open question below leaves unsettled.

### 3. Canonical encodings are domain-separated, length-prefixed, ordinal-free, and exhaustively matched

An encoder writes a versioned domain tag before any content, so bytes produced for one subject can never be mistaken for another subject's. It writes a fixed-width length before every variable-length run, so no concatenation of fields is ambiguous. It excludes transient identifiers — arena indices, builder insertion order, graph-local ordinals, planning identifiers — wherever the represented semantics are equivalent without them; `encode_identity` in `crates/tiler-ir/src/schedule/model.rs` documents excluding the transient `RegionId` for exactly this reason.

It matches every encoded enum exhaustively, with no wildcard arm and no silently omitted field. A single-variant enum is destructured irrefutably (`let ContributorOrder::OriginalAxisLexicographic = order;`) rather than matched with a catch-all, so adding a variant is a compile error at the encoding site. This is the fail-closed property: a widened enum must stop the build, never produce two structurally distinct subjects that share identity bytes.

**Fact — two sites do not yet meet this.** `push_numerical` omits `input_subnormals` and `result_subnormals` from `CanonicalScheduledRegionIdentity`, and `fusion_legality::effect_tag` maps every non-`Pure` `OperationEffect` to `u8::MAX` through a wildcard arm. Both are unobservable only while the enums have one inhabited variant. `extend-canonical-identity-encodings-for-reserved-variants` owns closing them.

**Fact — one small deviation in tag form.** Every domain tag in the workspace is NUL-terminated except `b"tiler.schedule.v1"` in `encode_identity`, and `push_numerical` writes `profile_key` NUL-terminated rather than length-prefixed. Neither is ambiguous today, because the tag is a fixed constant followed by fixed-width fields and `profile_key` is a `&'static str` chosen by the crate. This ADR proposes the NUL-terminated versioned tag and the length prefix as the uniform form so the reasoning does not have to be redone per site.

### 4. Construction is a transactional builder plus a consuming `build()` yielding an unforgeable verified product

This is ADR 0071's lifecycle, restated here as one item of a conformance checklist rather than re-decided. A public builder owns private storage and checks local invariants on each insertion, leaving the draft unchanged when an insertion is rejected. `build(self)` consumes the builder, runs whole-object verification, and returns either an opaque verified product or a typed failure carrying the diagnostics and recoverable builder ownership. `build`, not `freeze`, is the terminal vocabulary.

The verified product cannot be forged: its fields are private, so struct-literal construction fails to compile, and it offers no mutation, thawing, unchecked constructor, or mutable access to its draft. A closure convenience delegates to that same builder and the same consuming verifier — it does not re-implement verification — and it scopes the draft by mutable borrow so the closure body cannot itself reach the consuming step.

### 5. `#[non_exhaustive]` on public enums and output records documented as growing

A public enum whose variant set is a bounded-profile placeholder, and a public output record that will gain fields, carries `#[non_exhaustive]` so the addition lands additively rather than breaking a downstream `match` or struct literal. The doc comment that says the type is bounded and the attribute that makes growth additive belong together; a comment alone does not fail closed.

The rule is deliberately asymmetric. It does not extend to caller-constructed *input* records: a caller must be able to write the literal, and growing such a type is a constructor-signature change regardless, so `#[non_exhaustive]` buys nothing and costs construction ergonomics. `harden-public-enums-non-exhaustive` records the same split for the concrete `tiler-metal-aot` inputs.

Marking a recognized enum non-exhaustive forces its consumers to grow an explicit reject-unknown arm. That is the intended fail-closed posture, not a regression.

### 6. Verified products expose no `pub` fields; leaf value-data descriptors may

A type whose invariants were established by a verifier exposes no public fields. It exposes borrowed accessors, iterators, and view types that yield meaning without yielding storage. `VerifiedIndexRegion` holds a single private `Arc<VerifiedIndexRegionData>` and reads through `*Ref<'_>` views; `VerifiedScheduledRegion` holds three private fields and reads through `region()`, `requirements()`, and `canonical_identity()`.

A *leaf value-data descriptor* — a plain record with no cross-field invariant that a producer legitimately assembles or reads field by field, and which becomes trustworthy only once a verifier binds it into a verified product — may expose `pub` fields. `crates/tiler-ir/src/schedule/model.rs` states this posture in its own module doc: "The descriptor structs are read-transparent value data; only `ScheduledRegionBuilder::build` can bind a region into an opaque `VerifiedScheduledRegion`."

The two rules compose safely only because opacity is enforced at the verified boundary. Which of the two forms `tiler-ir` should use for its descriptors is not settled; see the open question below.

### 7. A new authority is crate-private until its facade is reviewed

A landed authority that is not yet reachable from its crate's entry point is a private `mod` whose items are `pub(crate)`, with a module-level `#![allow(dead_code, reason = "…")]` whose reason names what the surface reserves and, where it is known, which slice will consume it. It becomes `pub` only when Tom accepts the exact facade; a module that is already `pub` while under review says so in its module documentation, as `tiler_compiler::capability` and `tiler_compiler::legality` do ("Every public item here is a reviewed *draft* boundary").

This is the staging rule that makes the other six affordable: a surface can be built, tested, and reviewed at full fidelity without spending public-API commitment before the boundary is accepted.

## Consequences

- A conforming surface is reviewable against a citable list instead of against a reviewer's memory of sibling modules. A deviation becomes an explicit argument in a ticket outcome or a superseding decision.
- Most violations are visible by reading the item, without running anything: `Box<dyn Error>` in a public signature (1), a public constructor on a derived identity (2), a missing domain tag or a wildcard arm in an encoder (3), a `freeze`-style non-consuming terminal or a forgeable verified product (4), a growing public enum without `#[non_exhaustive]` (5), a `pub` field on a verified product (6), and a `pub` module without an accepted facade (7). Three points still need judgement rather than inspection: whether a record is a leaf value-data descriptor or an invariant-bearing type (6), whether a type is documented as growing (5), and whether an identity is derived or received at a boundary (2).
- Writing the conventions down does not make the codebase conform. Two identity encodings and two crates' public enums are known non-conforming, each with a named owner.
- The rules constrain shape, not semantics. None of them decides what an operation means, which plan is cheapest, or which layer owns a concept; those remain with the IR, optimizer, and architecture contracts.

## Implementation boundary

**This ADR records conventions and retrofits nothing.** It changes no code, alters no existing public surface, and does not by itself make any current API conforming. The conforming work is owned by separate implementation tickets — `harden-public-enums-non-exhaustive`, `extend-canonical-identity-encodings-for-reserved-variants`, and `unify-schedule-index-region-with-verified-index-region` — which may in turn amend or close the open questions below.

`implementation_status: "partial"` reflects the state of the decided behaviour, not of this document: most landed surfaces already satisfy most conventions, and the exceptions above are named with owners.

Conventions 1 through 6 are realized in `tiler-ir` and, for identity and errors, in `tiler-compiler`'s draft authorities. The closure convenience of convention 4 exists only for `tiler_ir::index::IndexRegionBuilder`; `ScheduledRegionBuilder` and `SemanticProgramBuilder` expose the transactional builder and consuming `build()` without a `build_with`, and `CheckedBuildError` plus the toy-builder tests in `crates/tiler-ir/src/convenience.rs` are the evidence that the shared shape generalizes to a second layer. That is a reservation, not implemented support.

## Open questions

These are recorded unresolved on purpose. None is settled by this ADR.

- **Descriptor accessor style within `tiler-ir`.** `tiler_ir::schedule` exposes leaf descriptors (`IndexRegion`, `Access`, `KernelSchedule`, `BoundsProof`, `OwnershipProof`, `NumericalRealization`) with `pub` fields, while the sibling `tiler_ir::index` keeps its data types `pub(super)` and reads through `*Ref<'_>` view accessors. Both satisfy convention 6, and this is not a soundness gap: opacity is enforced at `VerifiedScheduledRegion` and the descriptors are reachable only through `&ScheduledRegion`. It is nonetheless two styles in one crate, and this ADR deliberately does not pick a winner. The choice is owned by `unify-schedule-index-region-with-verified-index-region`, which must decide whether the unified form adopts view accessors (preferred if it needs field-level invariants) or records the pub-field value-data form as intended for schedule descriptors.
- **Whether `applies_to` should also name `tiler.contract.optimizer`.** This record names `tiler.contract.ir` (whose "Shared IR construction lifecycle" section is where conventions 1 through 6 would become normative) and `tiler.contract.architecture` (whose component-boundary and packaging sections are where convention 7 would). Convention 7 also constrains `tiler-compiler` modules that `docs/compiler/optimizer.md` describes, but that contract owns search semantics rather than Rust surface shape, and the compiler's public facade does not exist yet. Naming it now would claim authority this ADR has not earned. Revisit when the reviewed compiler facade lands.
- **Naming for presentation-only digest labels.** `key()` currently names both a presentation-only digest (`tiler-compiler`) and a stable semantic key (`tiler-ir`). A distinct spelling — `label()`, `display_id()`, or similar — would make the distinction visible at the call site, but renaming touches explain-record construction and its fixtures. No owner is assigned.
- **Whether a conformance check should be mechanized.** Several conventions are grep-shaped (`Box<dyn` in a `pub fn` signature, a `pub` field on a `Verified*` type, an encoder without a domain tag). Whether that belongs in the repository gate, in a Clippy configuration, or in review only is unresolved; a check that fires on the leaf-descriptor exception would be worse than none.

## Alternatives considered

Leaving the conventions implicit is the status quo, and it is what produced the `#[non_exhaustive]` gap: the rules propagated by imitation, each worker citing the sibling it happened to read, with nothing to cite when a sibling was wrong.

Encoding the conventions as an accepted contract section instead of a proposed ADR would skip the acceptance step this repository reserves for public boundaries. Acceptance is Tom's, and propagation into `docs/ir.md` and `docs/architecture.md` is a deliberate follow-up rather than a side effect of drafting.

Enforcing the conventions with a lint or gate before writing them down would freeze the leaf-descriptor exception and the digest-label naming before either is settled, and would report a violation without a citable rule to explain it.

Retrofitting the non-conforming surfaces inside this record would mix a conventions decision with identity re-baselining across two crates. Those edits change canonical bytes and belong to tickets that can rebaseline fixtures deliberately and state that the change is intentional rather than drift.
