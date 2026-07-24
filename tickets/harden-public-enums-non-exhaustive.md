---
id: harden-public-enums-non-exhaustive
title: Mark growth-expecting public enums and output records non-exhaustive
status: todo
priority: p2
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [prototype-apple-aot-driver, prototype-scheduled-region-ir, resolve-non-exhaustive-recognizer-hole, harden-kernel-vocabulary-recognizer-completeness]
scopes: [implementation/ir, implementation/metal-aot, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening]
---
The newly merged `tiler_ir::schedule` and `tiler-metal-aot` public surfaces expose
enums that are documented single-variant or bounded-profile placeholders for the
wide-operation future, plus output records that will gain identity dimensions. As
written, every future operation/dtype/family variant and the deferred artifact
content digest is a breaking change to a downstream `match` or struct literal.

**Revised 2026-07-24 after `resolve-non-exhaustive-recognizer-hole`.** ADR 0074's
amended convention 5 splits the rule by what a consumer's match has to do, and
four of the ten `tiler_ir::schedule` types this ticket originally listed must
**not** be marked. The original list and its reasoning are preserved below the
revised one so the change is visible rather than silent.

Mark the growth-expecting types `#[non_exhaustive]` so those additions land
additively:

- `tiler_ir::schedule`, convention 5a only — no out-of-crate consumer matches
  these completely; `tiler-compiler` and `tiler-metal` construct them:
  `ReductionTopology`, `BoundsProofKind`, `OwnershipProofKind`,
  `ExecutionBinding`, `TailPolicy`, `ContributorOrder`.
- `tiler-metal-aot`: `AppleSdk` (reserves Mac Catalyst), `OptimizationLevel`, and
  the output records `ArtifactProvenance` and `CompiledArtifact` (which gain the
  deferred content digest). Leave the input structs (`CompileRequest`,
  `MetalTarget`, `NumericalRealization`) exhaustive: callers construct them and
  their growth is a `new()`-signature change regardless.

**Revised again 2026-07-24 after `choose-one-owner-for-apple-target-vocabulary`.**
The premise that `tiler-metal-aot`'s types "have no consumer outside
`tiler-metal-aot` at all" did not survive checking, and `MslVersion` is removed
from the bullet above because of it. `compile-golden-msl-through-the-aot-driver-in-the-gate`
gave `tiler-metal` a `[dev-dependencies]` edge to the driver, and
`#[non_exhaustive]` binds every out-of-crate consumer regardless of dependency
kind. Several driver types now have out-of-crate consumers; three of them must
stay exhaustive:

- `MslVersion` and `ApplePlatform` — convention 5b.
  `crates/tiler-metal/src/target_correspondence.rs` maps both onto
  `tiler_metal::target::{MslLanguageVersion, MetalPlatform}` *totally*, so that
  neither crate can gain a language standard or an artifact family the other
  lacks. Marking either forces a wildcard into a map whose only honest arm is
  the counterpart the variant itself determines; a wildcard could only invent
  one. `ApplePlatform` was never on the mark list — this records why it must
  stay off it.
- `DriverError` — convention 5c, and not previously considered here.
  `crates/tiler-metal/src/golden_compilation.rs::resolved_toolchain` recognizes
  it out of crate to separate an absent Apple toolchain (self-skip) from a
  defect (report). A wildcard there is correct today and silently wrong the
  moment a variant lands that must not read as an absent toolchain, which would
  convert a defect into a skipped test. The type now says so in its own doc
  comment.

`AppleSdk` and `OptimizationLevel` keep their place on the mark list: `tiler-metal`
constructs both and matches neither, which is 5a.

**Do not mark these four `tiler_ir::schedule` types; leaving them exhaustive is
the decided rule, not an omission.** Record in the Outcome that they were
deliberately excluded, alongside the three `tiler-metal-aot` types above.

- `SubnormalMode` and `NumericalPermission` — convention 5b. Two crates map them
  *totally*: `tiler_compiler::fusion::FusionNumericalProof::canonical_explain_evidence_bytes`
  encodes `NumericalPermission` into canonical evidence bytes, and
  `tiler_metal::emit::realization_requirements` derives the Metal compiler
  requirements a kernel's declared numerics impose from both. Marking either would
  force a wildcard into both sites: an identity collision at the first and, at the
  second, a compiled artifact whose flags and whose reported requirements disagree.
  `emit.rs` already documents that it depends on the attribute's absence.
- `ScalarProgram` and `LogicalAccess` — convention 5c. Their out-of-crate
  recognizers are `tiler_compiler::physical::verify_region_subject_binding` and
  two matches in `tiler_compiler::program`. Marking them would make a future
  variant compile cleanly at each recognizer and silently reroute it into
  reject-unknown.

**A premise in the original text that did not survive checking.** It said marking
`ScalarProgram` forces "`tiler-compiler`'s recognizer (`verify_region_subject_binding`,
`verify_access_and_semantics`)" to grow a reject-unknown arm. Only the first is in
`tiler-compiler`. `verify_access_and_semantics` is in
`crates/tiler-ir/src/schedule/builder.rs`, the crate that defines `ScalarProgram`,
where `#[non_exhaustive]` has no effect, and it already carries a catch-all because
it matches a three-way product — so a fourth scalar program is silently rejected
there today, with no attribute involved. Do not "fix" that here; it is a match-shape
problem convention 5 explicitly cannot govern.

The original text and reasoning, superseded above and kept for the record:

> - `tiler_ir::schedule`: `ScalarProgram`, `LogicalAccess`, `ReductionTopology`,
>   `BoundsProofKind`, `OwnershipProofKind`, `SubnormalMode`, `NumericalPermission`,
>   `ExecutionBinding`, `TailPolicy`, `ContributorOrder`.
>
> Marking `ScalarProgram` non-exhaustive forces `tiler-compiler`'s recognizer
> (`verify_region_subject_binding`, `verify_access_and_semantics`) to grow an
> explicit reject-unknown arm — this is the intended fail-closed posture, not a
> regression. There are no external consumers of either surface yet, so this is
> timing-free now and expensive to retrofit after the first consumer lands.

The "no external consumers yet" observation survives, but it argues the opposite
way for 5b and 5c types: with nothing published, the additive growth the attribute
buys protects nobody, while the compile error it destroys is the only mechanism
that keeps a total map correct and a recognizer complete. ADR 0075 records the
same phase facts and the trigger for revisiting them.

Reviewed as a fast-follow to the two prototype merges (Tom saw the surfaces).
This ticket does not change any semantics; it only reserves additive growth.

While polishing these newly landed public surfaces, also replace the ` ```ignore `
rustdoc example on `tiler_ir::index::IndexRegionBuilder::build_with` with a
runnable doctest if it can be done without ~25 lines of scalar-registry setup
(that setup cost is why it was left `ignore` at review time; the manual/closure
equivalence is currently proven only by the integration test, so the example
itself is unverified). If a runnable example remains impractical, record that
explicitly rather than leaving it looking like an oversight.

## The list decays — re-derive it, do not execute it as written

**Revised again 2026-07-24, second time in one day.** This ticket's enumeration has now been invalidated twice by consumers landing *after* it was written, and both times the correction was found by the ticket that created the consumer rather than by this one.

- `choose-one-owner-for-apple-target-vocabulary` added `crates/tiler-metal/src/target_correspondence.rs`, an out-of-crate total map over `MslVersion` and `ApplePlatform`, and `golden_compilation.rs` recognizes `DriverError` out of crate. That removed `MslVersion` from the mark list and put `ApplePlatform` and `DriverError` permanently off it.
- `prototype-neutral-artifact-codec` added `value_role_tag`, `subnormal_tag`, and `permission_tag` in `crates/tiler-artifact/src/program/model.rs` — exhaustive matches with **no wildcard arm** over `tiler_ir`'s `ValueRole`, `SubnormalMode`, and `NumericalPermission`. All three are now convention 5b and must **not** be marked.

**Inference — the defect is the form, not the entries.** A list of "enums to mark" is a snapshot of which out-of-crate consumers existed on the day it was written, and ADR 0074 states the classification "is a property of the consumers that exist". Every new cross-crate total map silently invalidates a line of it, and executing a stale line is not a no-op: forcing a wildcard into a total map converts a would-be build error into a silently wrong encoding, which is the exact failure convention 5b exists to prevent.

**Whoever takes this must re-derive the classification at execution time rather than trusting the enumeration above.** For each candidate enum, find every out-of-crate match on it and decide by what those consumers do — total map (5b, stay exhaustive), support recognizer (5c, stay exhaustive while pre-alpha), or partial/forwarding classification (5a, mark). Record the consumer that justifies each verdict, so the next reader can tell a decayed entry from a decided one.

**One entry has a pending trigger rather than a verdict.** `widen-numerical-vocabulary-and-complete-identity` will grow `SubnormalMode` and `NumericalPermission`, which will break the codec's exhaustive tag maps by construction. That break is the guard working; it is not a reason to add a wildcard.
