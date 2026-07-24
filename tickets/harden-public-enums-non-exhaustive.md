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
- `tiler-metal-aot`: `AppleSdk` (reserves Mac Catalyst), `MslVersion`,
  `OptimizationLevel`, and the output records `ArtifactProvenance` and
  `CompiledArtifact` (which gain the deferred content digest). Leave the input
  structs (`CompileRequest`, `MetalTarget`, `NumericalRealization`) exhaustive:
  callers construct them and their growth is a `new()`-signature change regardless.
  These types have no consumer outside `tiler-metal-aot` at all, so the amendment
  does not touch this bullet.

**Do not mark these four; leaving them exhaustive is the decided rule, not an
omission.** Record in the Outcome that they were deliberately excluded.

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
