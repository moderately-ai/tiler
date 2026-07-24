---
id: resolve-non-exhaustive-recognizer-hole
title: Resolve the non-exhaustive recognizer hole before applying the convention
status: done
priority: p1
dependencies: []
related: [harden-public-enums-non-exhaustive, extend-canonical-identity-encodings-for-reserved-variants, draft-public-api-conventions-adr, draft-public-boundary-approval-policy-adr]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, public-api, correctness]
---
Accepted ADR 0074's convention 5 says to mark public enums documented as growing
`#[non_exhaustive]` so a later variant "lands additively rather than breaking a
downstream `match`". It also says that forcing consumers to grow an explicit
reject-unknown arm is "the intended fail-closed posture, not a regression". Those
two statements are in tension for **recognizer** enums, and the tension is a real
hole rather than a wording nit.

## The hole

For an enum a *recognizer* matches on — `tiler_ir::schedule::ScalarProgram` is the
concrete case, matched by `tiler-compiler` to decide which programs it supports —
`#[non_exhaustive]` forces every cross-crate consumer to carry a wildcard arm.
Adding a variant then **compiles cleanly at every consumer** while silently
routing the new variant into `_ => reject`. Nothing breaks, nothing warns, and a
program the profile now supports is quietly refused.

The compile error was the feature. It is what forced each consumer to consciously
decide how to handle a newly supported case. `#[non_exhaustive]` converts a loud,
mechanical prompt into a silent behavioural change from accept to reject. It is
still fail-*closed* — it refuses rather than mishandles — but it is silently
**incomplete**, which is a different and harder failure to notice: the capability
exists in the IR and never reaches the consumer.

## Why it must be resolved before `harden-public-enums-non-exhaustive` runs

That ticket applies `#[non_exhaustive]` to exactly these enums, including
`ScalarProgram`. Running it against an unresolved rule would *install* the hole.
Add the dependency rather than trusting sequencing by luck.

It also interacts with `extend-canonical-identity-encodings-for-reserved-variants`,
which requires encoders to match **exhaustively** so a new variant is a compile
error at the encoding site. Note the interaction is not a contradiction: those
encoders live in the same crate as the enums, and `#[non_exhaustive]` only
constrains *other* crates. Verify that holds for every encoder before relying on
it — a same-crate assumption that later moves crates would break silently.

## What to decide

Amend ADR 0074's convention 5 to distinguish the cases rather than stating one
rule for all growing enums. Candidate distinction to evaluate, not to adopt
unexamined: `#[non_exhaustive]` suits enums consumers *produce or read* (errors,
provenance, descriptors), and is actively harmful for enums a consumer
*exhaustively recognizes* to decide support, where the compile break is the
mechanism that keeps recognizers complete. If that distinction survives scrutiny,
state how a recognizer enum announces growth instead — a versioned capability
list, a conformance test that fails when a variant lacks a recognizer arm, or an
explicit decision that such enums stay exhaustive and their growth is a
deliberate breaking change we accept while pre-alpha.

Whatever is chosen, amend the accepted ADR explicitly rather than leaving the
convention stated in a form we now know is wrong for one of its cases.

## Outcome

Accepted ADR 0074 amended in place; `decision_status` stays `accepted`. Two
conventions changed, each with an entry in a new `## Amendments` section that
quotes the superseded wording, gives the evidence, states why an amendment
rather than a superseding ADR, and says what a reader of the earlier text should
un-learn. The original reasoning is preserved rather than rewritten.

**Convention 5 is now three clauses, split by what the consumer's match has to
do.** 5a is the accepted rule verbatim, and remains the whole rule for a type no
crate outside the defining crate matches completely. 5b forbids
`#[non_exhaustive]` on a vocabulary an out-of-crate consumer maps *totally*, and
decides the collision with convention 3 in convention 3's favour. 5c keeps a
cross-crate *recognizer* enum exhaustive while Tiler is pre-alpha: its growth
announcement is the compile error, and that source-breaking change is accepted in
full on ADR 0075's recorded facts (`publish = false`, `version = "0.0.0"`, Tom's
2026-07-24 rejection of the compatibility premise). The convention's closing
sentence — reject-unknown as "the intended fail-closed posture, not a
regression" — is explicitly withdrawn as stated and the reason it is still true
for 5a is recorded.

Two mechanisms that would keep the attribute *and* the compile error are recorded
with their cost and a reconsideration trigger, not adopted: a declared capability
list checked by a conformance test, and the `non_exhaustive_omitted_patterns`
lint. The lint was measured, not assumed.

**Measurements** (two-crate probe, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`
from the pinned `nightly-2026-07-19`, edition 2024, macOS arm64):

- A same-crate `match` over a `#[non_exhaustive]` enum compiles with no wildcard.
  The assumption `extend-canonical-identity-encodings-for-reserved-variants`
  rests on **holds** for the `schedule/model.rs` encoders, which are same-crate.
- A cross-crate `match` listing every variant without a wildcard fails `E0004`.
- `#![feature(non_exhaustive_omitted_patterns_lint)]` plus
  `#[deny(non_exhaustive_omitted_patterns)]` makes a cross-crate recognizer that
  keeps its wildcard but omits a known variant fail to compile. Without the
  feature gate the attribute is inert and emits only `unknown_lints`, which the
  warning-free gate converts into a failure rather than a silent hole.

**Findings that changed the decision, all verified by full reads at `37f1350`:**

- The same-crate assumption does **not** hold for every encoder.
  `tiler_compiler::fusion_legality::effect_tag` encodes `OperationEffect`, which
  is already `#[non_exhaustive]` in `tiler-ir` — so conventions 3 and 5 have been
  in direct contradiction at that site since both were accepted, and
  `extend-canonical-identity-encodings-for-reserved-variants` could not have
  closed it under the unamended rule. Recorded as an amended note in convention 3.
- Two further out-of-crate total maps exist:
  `tiler_compiler::fusion::FusionNumericalProof::canonical_explain_evidence_bytes`
  over `NumericalPermission`, and `tiler_metal::emit::realization_requirements`
  over `NumericalPermission` and `SubnormalMode`. The second already documents in
  code that it depends on the attribute's absence. Both enums were on
  `harden-public-enums-non-exhaustive`'s list; marking them would have produced an
  identity collision and a compiled artifact whose flags and reported
  requirements disagree — neither a rejection, so neither fails closed.
- The genuine cross-crate recognizers of the shared IR vocabulary are
  `tiler_compiler::physical::verify_region_subject_binding` over `ScalarProgram`
  and two matches in `tiler_compiler::program` over `Option<&LogicalAccess>`.
- The originating ticket's framing did not fully survive.
  `verify_access_and_semantics` is in `tiler-ir`, not `tiler-compiler`, so the
  attribute cannot affect it; and it already carries a catch-all because it
  matches a three-way product, which means a fourth `ScalarProgram` is silently
  rejected there **today** with no attribute involved. The hole is therefore not
  exclusively an attribute problem, and convention 5 now says so: a product match
  needs its own completeness argument that no attribute rule can supply.
- The kernel-IR vocabulary is the same class one layer down. Twelve
  `#[non_exhaustive]` enums in `tiler_ir::kernel` are recognized by
  `tiler-metal`'s emitter. Conforming is a net deletion: ten of the emitter's
  wildcards are unreachable today and exist only because the attribute forces
  them, three are live and each already names its rejected variants in a comment,
  and one is a structural tuple catch-all no attribute affects.

**Convention 4** (added mid-ticket by the coordinator after a parallel agent hit
it while propagating conventions into `docs/ir.md`; claims re-verified here
independently). "`build`, not `freeze`, is the terminal vocabulary" is withdrawn
as a workspace-wide rule. The convention now states the property — the terminal
consumes its builder and yields an unforgeable immutable product — and restores
the scope qualifier this record dropped when restating ADR 0071. Verified: all
five public `freeze` terminals take `self` and return a `Frozen*` newtype over a
private `Arc`; no terminal anywhere in the workspace takes `&self` or `&mut self`;
this record's own Consequences list already named the correct rule
("non-consuming"), so the Decision text contradicted it. **ADR 0071 needs no
change** — it scopes its sentence to "each shared target-neutral IR layer", where
no `freeze` exists and the vocabulary claim is exactly right; the defect was
ADR 0074 widening it to every public API. The unamended text would have forced
`propagate-accepted-api-conventions-into-governed-contracts` to reword a
`docs/ir.md` lifecycle that was already correct.

The registry family's second difference — every shared-IR `build` returns a
recoverable builder while three `freeze` terminals return a typed error without
one — is recorded as a new open question rather than decided, with the honest
split: the two infallible terminals raise no question, `EmptyRegistry` has
nothing worth recovering, and only `SemanticRegistryBuilder::freeze` fails in a
way a caller could plausibly correct and retry. Converging it changes three
public signatures across two crates, which is Tom's to review.

**Consistency work.** ADR 0075's open question "whether adding a variant to a
recognized `#[non_exhaustive]` enum is additive growth" is recorded as resolved:
under amended convention 5 a recognized enum is not a non-exhaustive one, so the
case cannot arise, and what remains marked has by construction no out-of-crate
consumer that maps it completely — making that no-approval category name
something checkable. `harden-public-enums-non-exhaustive` is revised so it cannot
install the hole it warned about: four types moved to an explicit do-not-mark
list, the original list and reasoning preserved as a quoted block, and the
`verify_access_and_semantics` premise corrected.
`harden-kernel-vocabulary-recognizer-completeness` is new and owns the kernel
vocabulary named in convention 5c.

No code was changed. `harden-public-enums-non-exhaustive` and the new ticket
apply the rule.
