---
id: resolve-or-retire-the-scalar-lowering-provider-seam
title: Resolve or retire the scalar-lowering provider seam
status: awaiting-decision
priority: p1
dependencies: []
related: [own-or-close-the-adr-internal-open-questions, drive-an-external-physical-implementation-provider-through-compilation, land-the-scalar-lowering-seam-retirement-adr]
scopes: [implementation/compiler, contracts/optimizer, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, extension-seams]
claimed_from: todo
assignee: agent-scalar-seam
lease_expires_at: 1786061768
---
## User-visible outcome

`ScalarLoweringProvider` is either an installation seam a compiled program actually reaches — with the same out-of-crate installation evidence the index-access family already carries — or it is gone. It stops being a registered public seam that nothing on the compile path exercises.

## Why this exists

**Fact — the family registers and resolves, and no compile stage resolves it.** `docs/compiler/optimizer.md:235` states it outright: "`ScalarLoweringProvider` remains only implemented-and-resolvable support because no compile stage resolves that family." `docs/correctness-and-testing.md:117` repeats it as a gap in the conformance gate's own evidence — "Scalar-lowering providers register and resolve but no compile stage resolves that family, so no installation evidence exists for it."

**Fact — every caller is a test.** `resolve_scalar_lowering` is declared at `crates/tiler-compiler/src/capability.rs:1150`. Its call sites are `crates/tiler-compiler/src/capability.rs:1939`, `:2165`, `:2213`, `:2229`, `:2339`, `:2369` and `crates/tiler-compiler/src/legality.rs:2143`. The `#[cfg(test)] mod tests` boundaries are `capability.rs:1688-1689` and `legality.rs:1432-1433`, so every one of those sites is inside a test module. Reproduce with `grep -rn "resolve_scalar_lowering" crates/` and compare each line against those two boundaries.

> **Every number above has drifted, corrected 2026-08-04 by the stale-claim sweep at base `c4b4bdb9`. The Fact is unchanged and was re-derived rather than assumed — this is the one citation in the ticket whose *argument* is a line-number comparison, so following the stale numbers would have made the comparison meaningless rather than merely inconvenient.** Current: `resolve_scalar_lowering` is declared at `crates/tiler-compiler/src/capability.rs:1096`. Its call sites are `capability.rs:1897`, `:2123`, `:2171`, `:2187`, `:2297`, `:2328` and `crates/tiler-compiler/src/legality.rs:1683` — seven, the same count. The `#[cfg(test)]` attributes are at `capability.rs:1637` and `legality.rs:1000`. **Every one of the seven sites is below its file's boundary**, so the conclusion holds: the declaration at `:1096` is production, and no production caller exists. Reproduce with `grep -n 'resolve_scalar_lowering' crates/tiler-compiler/src/capability.rs crates/tiler-compiler/src/legality.rs` and `grep -n '^#\[cfg(test)\]' crates/tiler-compiler/src/capability.rs crates/tiler-compiler/src/legality.rs`, then compare — the second command prints exactly one line per file, which is what makes the comparison total rather than a sample.

**Fact — an accepted ADR names the question and assigns nobody.** [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md):144 asks "Whether `ScalarLoweringProvider` should reach the compile path at all", observes that `lowering.rs` resolves only `IndexAccess` and that "an index-access provider emits its own per-point scalar work through the same context", and closes with "No owner is assigned." This ticket is that owner.

**Inference — a registered public seam nothing exercises is unvalidated extension surface.** AGENTS.md's contract requires extension mechanisms to preserve validation, feasibility, and versioned identity, and states that "extensible" does not mean unknown behaviour is optimizable. A seam whose only evidence is in-crate unit tests makes no such guarantee, and the asymmetry with the index-access family — which has out-of-crate installation evidence plus a negative test that fails closed, per `docs/compiler/optimizer.md:235` — is what makes the gap visible.

## Run the elimination and state which candidate survived

This ticket must not land a shrug. Test both candidates against correctness, performance, and long-term maintainability, and state the derivation so a reader can refute the elimination rather than only the conclusion.

- **Wire it.** The scalar family becomes a second seam a program reaches, with out-of-crate installation evidence and a companion negative test omitting one family and failing closed — the exact pair `docs/compiler/optimizer.md:235` describes for index-access, so a passing installation test cannot be explained by an installer ignoring its argument. What this must answer is what a scalar provider decides that an index-access provider delegating per-point work does not.
- **Retire it.** ADR 0078's own reading — that a scalar decomposition is an affordance an index-access provider may delegate to rather than a seam a program reaches — is accepted, and the registration, resolution, and their tests are removed. This supersedes an accepted ADR 0078 seam and is therefore **Tom's**: draft the superseding record, do not self-accept, and do not delete the seam before he accepts it.

## Closes when

One candidate is landed with its elimination written out; `docs/compiler/optimizer.md:235`, `docs/correctness-and-testing.md:117`, and ADR 0078:144's unowned question are each corrected or closed in the same change; and, if wiring, an out-of-crate caller installs a scalar-lowering provider, compiles through `session::compile`, and observes the artifact plan recording that provider as the lowering authority, with the companion omission observed failing closed.

**Revised 2026-08-06.** Retirement survived the elimination, and retirement is Tom's, so the clause "one candidate is landed" cannot be met from this branch — landing it would delete an accepted ADR 0078 seam before he accepted the record superseding it, which the ticket's own second bullet forbids. The two contract corrections are discharged here; ADR 0078:144 and the ADR 0078 item-2 row are `contracts/decisions`, which this ticket does not hold, so their exact owed text is recorded verbatim below for the carrier. This ticket therefore closes on Tom's decision, not on a landed removal, and its status is `awaiting-decision`.

## Outcome (2026-08-06, at `eee734cf`)

### The Fact was re-derived on this base and holds

`resolve_scalar_lowering` is declared at `crates/tiler-compiler/src/capability.rs:1372`. Its call sites are `capability.rs:2173`, `:2399`, `:2447`, `:2463`, `:2573`, `:2604` and `crates/tiler-compiler/src/legality.rs:1875` — seven, the same count the 2026-08-04 correction found. The `#[cfg(test)]` attributes are at `capability.rs:1913` and `legality.rs:1187`, exactly one per file. Every one of the seven sites is below its file's boundary and the declaration at `:1372` is above it, so the declaration is production and no production caller exists. Reproduced with the ticket's own two commands. `grep -rn 'resolve_scalar_lowering' crates/ prototypes/ spikes/` adds nothing outside those two files.

### The survivor is **retirement**, and the elimination turns on one question the wiring candidate cannot answer

**What a scalar provider would decide that an index-access provider delegating per-point work does not: nothing, and the reason is structural rather than incidental.**

**Fact — a realization law, not a provider, decides what a lowering emits.** `crates/tiler-ir/src/index/law.rs`'s header states it as the module's governing rule: "Verification requires the candidate region's exact canonical identity to equal the region this law constructs. A semantically equivalent alternate logical index form is deliberately refused." `ResolvedIndexRealization::verify_sequence` (`crates/tiler-ir/src/index/refinement.rs:1710-1737`) builds `expected = self.law.realize_sequence(subject, scalars)` and refuses unless `expected.identity() == realization.identity()`. An occurrence whose operation registers no law refuses as `MissingRealizationLaw` (`refinement.rs:776`) before any provider is driven, so every occurrence reaching the compile path has one.

**Fact — the per-point scalar work is inside the compared bytes, and the law names it exactly.** `IndexRealizationLaw::PointwiseBinary` carries the applied `ScalarOpKey`; `ConstantFromFloatBits` carries the attribute and the scalar; `StagedRootMeanSquareScaleF32` fixes its entire fold epilogue — divide by the folded contributor count, add the exact `eps` payload, apply the reciprocal square root — and `law.rs:120-131` states why it is fixed rather than parameterized: "carrying the epilogue and the pass as *data* would need a scalar-program language inside a law, which is the universal IR this module's header refuses".

**Measurement — that refusal is already observed on the ordinary compile path with an externally installed provider.** `pipeline::conformance::a_lowering_cannot_replace_the_semantic_providers_realization_law` (`crates/tiler-compiler/src/pipeline/conformance.rs:1591`) installs an out-of-crate-shaped multiply lowering that emits a structurally valid *alternate* realization and observes `RequestError::UnsupportedCapability { phase: "lowering", rule: "refinement-refused" }` with the mismatch explained before planning.

**Inference — a scalar provider driven anywhere on this path has exactly one admissible output.** It is the sub-expression the law already builds, decided by an authority the provider does not own. Any other output changes the realization identity and refuses the whole occurrence. A seam whose admissible output set is a singleton fixed elsewhere carries no decision, and wiring it would be decoration.

**Fact — and it has nowhere checked to return that output.** `legality::refine_index_region` refuses the family by name (`legality.rs:760-764`, `RefinementError::WrongFamily`), and the module header says why: it "refines only the `LoweringFamily::IndexAccess` family, because only that family emits a standalone region; a scalar-lowering capability is rejected explicitly". No `refine_scalar_*` authority exists anywhere in the tree — `grep -rn 'refine_scalar' crates/` returns nothing. ADR 0078 item 1's first necessary condition is that "the provider's output re-enters the ordinary checked path", and the record states plainly that "a surface that cannot is not a seam". **By ADR 0078's own governing rule the scalar family is not a seam today**, which is what makes its item-2 row a classification error rather than an unfilled evidence hole.

**The staged landing was checked against this and sharpens it rather than softening it.** Before the staged families, a per-point body of a pointwise law was the one construct that looked like a separable unit a scalar provider might own. The staged work delivered the opposite: a genuine multi-operation scalar expression — the root-mean-square epilogue — arrived as *law* data owned by the registering semantic provider, deliberately fixed, with every stage's reached scalar authority now folded into refinement content and occurrence identity (`legality.rs:1090-1098` and `:1145-1151`, under their own domain tags). The one thing that most resembled a scalar-provider decision arrived owned by the law and compared byte for byte. The direction of travel is away from a scalar seam.

### The symmetric question, so the elimination does not prove too much

An index-access provider's *emitted content* is dictated by the law in exactly the same way, so the argument above would retire that seam too if content latitude were the whole test. It is not, and three things distinguish the families:

- **It is mandatory.** `lowering::resolve_occurrence` fails closed for every recognized occurrence with no resolvable index-access capability. No program compiles without one. Nothing resolves the scalar family, so nothing is incomplete without it.
- **It is the named authority.** Its `{provider identity, capability revision}` pair is recorded in the artifact construction plan and re-derived from the installed registry when the plan is built and again when the portfolio is re-verified (`crates/tiler-compiler/src/program.rs`), so a receipt cannot name a provider the registry never resolved. Nothing records a scalar provider anywhere.
- **It completes an external semantic registration.** A third party defining a new operation registers its own law and must supply the code that emits that law's realization; Tiler ships no emitter for a family it does not define. That obligation is real even though the content is dictated — someone has to write the emitter. There is no corresponding obligation a scalar provider discharges.

### Tested against correctness, performance, and long-term maintainability

**Correctness.** Wiring loses outright. To reach the scalar family the host would have to own a decomposition of a realization into an index/access skeleton plus a splice point for a per-point body, and re-derive that the composition realizes the occurrence. That decomposition is what the law already owns whole, and `law.rs` refuses to carry scalar programs as data precisely so that a second scalar-expression vocabulary does not appear. Wiring would create exactly that second authority over one meaning, in a place with no verifier, to run a provider whose only admissible answer is the one the law already produced — the duplicated-authority failure mode AGENTS.md and ADR 0078 item 3 both name. Retirement removes a public surface that today makes an unbacked participation promise, which is the guardrail AGENTS.md states as "extensible does not mean unknown behaviour is optimizable".

**Performance.** Wiring drives a provider a second time to produce bytes already produced. Retirement's effect is nil at runtime: the family is resolved by nothing, so no compile-path work is removed or added.

**Long-term maintainability.** Wiring adds a splice vocabulary, a scalar refinement authority, an out-of-crate fixture pair, and a second identity domain for scalar sub-expressions, all to carry no decision. Retirement's cost is real but bounded and is stated honestly below rather than as "just deletion".

**Fact — retirement is identity-preserving, which is the one thing that could have made it expensive and does not.** `encode_capability_key` (`capability.rs:1831-1841`) writes `key.family.tag()`, and `LoweringFamily::IndexAccess` is tag `1`. Removing the `ScalarLowering` variant leaves that tag unchanged, so every frozen registry that exists today encodes to the same `CanonicalLoweringRegistryIdentity` bytes before and after. No ledger, golden, or identity pin is recomputed. `LoweringFamily::key_token` likewise keeps `"index-access"`, so no governed capability key moves.

**Fact — the removal is not thirteen deleted tests, and reading it that way would be the mistake.** `capability.rs`'s test module holds fifteen `register_scalar_lowering` sites and six `resolve_scalar_lowering` sites, and the scalar family is the *vehicle* for ten tests of the registry's own mechanics rather than their subject — each would read identically against any family: `snapshot_identity_is_independent_of_registration_order`, `duplicate_registration_of_one_provider_is_a_collision`, `one_operation_admits_more_than_one_registrable_signature`, `a_second_signature_for_one_family_and_operation_is_refused`, `contradictory_providers_resolve_to_a_deterministic_ambiguity`, `two_revisions_of_one_provider_resolve_to_an_ambiguity`, `a_missing_capability_resolves_to_a_typed_diagnostic`, `registration_rejects_an_operation_without_semantic_authority`, `registration_is_transactional_and_leaves_no_partial_state`, and `capability_revision_participates_in_snapshot_identity`. Every one of those must be **ported** to `register_index_access`, not deleted; several are load-bearing for ADR 0072 (`two_revisions_of_one_provider_resolve_to_an_ambiguity` is cited in ADR 0078 item 3 with its own coverage measurement). Only three are genuinely about the scalar family and go with it: `registers_two_families_and_resolves_each_to_its_provider` narrows to one family, `a_resolved_scalar_provider_emits_through_the_canonical_builder` goes, and `legality::tests::a_scalar_lowering_capability_is_not_an_index_refinement` goes together with the `RefinementError::WrongFamily` variant it is the sole constructor of. A retirement that deleted the ported ten would silently drop the registry's collision, ambiguity, and transactionality coverage, and the implementation ticket must say so.

**What retirement removes, from a full read of `capability.rs` and `legality.rs`.** `LoweringFamily::ScalarLowering` and its `key_token`, `tag`, and `Display` arms; the `ScalarLoweringProvider` trait; `ScalarLoweringContext` and its five methods; `ScalarLoweringResults`; `LoweringImplementation::ScalarLowering`; `LoweringCapabilityRegistryBuilder::register_scalar_lowering`; `FrozenLoweringCapabilityRegistry::resolve_scalar_lowering`; `ResolvedLoweringCapability::scalar_provider`; and, in `legality.rs`, `RefinementError::WrongFamily` with the now-unsatisfiable family guard at `:760-764`. Two consequential shapes fall out and are the implementation ticket's to decide rather than this record's: `LoweringImplementation` becomes a single-variant enum that may as well be `Arc<dyn IndexAccessLoweringProvider>`, and `LoweringFamily` becomes a single-variant `#[non_exhaustive]` enum whose `key_token` must survive because the governed capability key spells it. Collapsing either is a public-boundary change and is reserved to Tom under ADR 0075, so the record proposes the family's removal and deliberately not the enum's.

### What is owed to scopes this ticket does not hold

`contracts/decisions` (`docs/decisions/[0-9]*.md`) and `contracts/foundation` (`docs/operation-extensions.md`, `docs/architecture.md`) are not this ticket's, so their edits are recorded here rather than made. `contracts/navigation` owns `docs/open-questions.md`, whose two references to this ticket (`Q-PLAN-009` and `Q-PKG-002`) remain accurate and need no change.

**Owed to ADR 0078:144, verbatim, replacing that bullet's last two sentences** — the ones reading "Whether the scalar family is a second seam a program reaches, or a decomposition affordance an index-access provider may delegate to, is not decided by its existence. Owned by [`resolve-or-retire-the-scalar-lowering-provider-seam`](../../tickets/resolve-or-retire-the-scalar-lowering-provider-seam.md), which must run that elimination and either supply out-of-crate compile-path evidence or draft the superseding decision required before retirement.":

> **Answered 2026-08-06 — the affordance reading, and it is now a proposed decision rather than an open question.** [`resolve-or-retire-the-scalar-lowering-provider-seam`](../../tickets/resolve-or-retire-the-scalar-lowering-provider-seam.md) ran the elimination and found that the registered `IndexRealizationLaw` fixes the whole realization including its per-point scalar applications — `verify_sequence` refuses any candidate whose canonical identity differs, and `pipeline::conformance::a_lowering_cannot_replace_the_semantic_providers_realization_law` observes that on the ordinary path — so a scalar provider would have exactly one admissible output, decided elsewhere. It would also have nowhere checked to return it: `legality::refine_index_region` refuses the family by name and no `refine_scalar_*` authority exists, which item 1 of this record makes disqualifying rather than incidental. **This question is therefore closed as answered and reopened as a decision**: ADR 0103 proposes retiring the family and superseding this record's item-2 row, and only Tom's acceptance of that record settles it. The elimination's own reservation is retained: the byte-for-byte law comparison is what makes the output set a singleton, so a future decision to admit semantically equivalent alternate realizations would reopen the question rather than leave it answered.

**Owed to ADR 0078 item 2's inventory table**, when ADR 0103 is accepted: the `tiler_compiler::capability::ScalarLoweringProvider` row is removed and the sentence "**Fact — the scalar-lowering row's absence claim, stated so it can be refuted in one line**" at ADR 0078:63 goes with it. Until then both stand and the acceptance sweep executes the supersession, per the ADR 0100 precedent of naming the superseded item rather than edging the whole record.

**Owed to `contracts/foundation`**, when ADR 0103 is accepted: `docs/operation-extensions.md` carries the scalar family at `:14` (status line), `:58` (the three-claims paragraph), `:77` (the seam table row), `:85`, `:87`, and `:139`. `docs/architecture.md` was checked and carries no scalar-lowering claim — `grep -n 'scalar-lowering\|ScalarLowering' docs/architecture.md` returns nothing.

### What landed here

`docs/compiler/optimizer.md`'s maturity-boundary section and `docs/correctness-and-testing.md`'s conformance-gate evidence both stated the gap as an unfilled evidence row. Both now state the finding: the family carries nothing to install, with the law-comparison fact, the absent-checked-path fact, and the routing of the removal to Tom. In `docs/correctness-and-testing.md` the sentence was also lifted out of the multi-output paragraph it had been lodged inside — it has nothing to do with multi-output — and given its own paragraph before the per-dtype numerical-contract paragraph. No crate file is touched: `git diff --stat` against `eee734cf` is two documents plus this ticket and the carrier ticket, so the delta is eligible for the latest green gate under AGENTS.md's carry rule, and `tkt lint` was rerun.

### The superseding record, verbatim for the carrier

[`land-the-scalar-lowering-seam-retirement-adr`](land-the-scalar-lowering-seam-retirement-adr.md) transfers the span below the rule unedited, takes the next free number, and writes traceability, implementation boundary, and open questions fresh at the destination. The span carries no markdown links by construction, so nothing needs repointing; the check is that the `](` count inside its line range is zero.

---

**Title:** Retire the scalar-lowering provider seam

**Frontmatter:**

```
schema: "tiler-doc/v1"
id: "ADR-0103"
kind: "decision"
title: "Retire the scalar-lowering provider seam"
topics: ["extensions", "api", "capability", "governance"]
catalog_group: "foundation-semantics-extensions"
decision_status: "proposed"
implementation_status: "not-started"
applies_to: ["tiler.contract.operation-extensions", "tiler.contract.optimizer", "tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.extensions.backend-provider-composition"]
depends_on: ["ADR-0044", "ADR-0072", "ADR-0078"]
ticket: "land-the-scalar-lowering-seam-retirement-adr"
```

## Context

ADR 0078 classified `tiler_compiler::capability::ScalarLoweringProvider` as an intended third-party seam at *implemented support*, and left an open question asking whether the family "should reach the compile path at all", observing that `lowering.rs` resolves only `IndexAccess` and that "an index-access provider emits its own per-point scalar work through the same context". It assigned no owner. `resolve-or-retire-the-scalar-lowering-provider-seam` was made that owner and ran the elimination between the two candidates: wire the family so a compiled program reaches it, with the out-of-crate installation evidence and the companion fail-closed negative test the index-access family carries; or accept the affordance reading and remove it.

**Fact — the family registers and resolves, and no production caller resolves it.** At `eee734cf`, `resolve_scalar_lowering` is declared at `crates/tiler-compiler/src/capability.rs:1372`, above that file's sole `#[cfg(test)]` at `:1913`. Its seven call sites — `capability.rs:2173`, `:2399`, `:2447`, `:2463`, `:2573`, `:2604`, and `crates/tiler-compiler/src/legality.rs:1875` — all lie below their file's sole `#[cfg(test)]` boundary, at `capability.rs:1913` and `legality.rs:1187`. Reproduce with `grep -n 'resolve_scalar_lowering'` and `grep -n '^#\[cfg(test)\]'` over those two files and compare; each command prints exactly one boundary line per file, which is what makes the comparison total rather than a sample.

**Fact — a registered realization law, not a provider, decides what a lowering emits.** `crates/tiler-ir/src/index/law.rs` states its own governing rule: verification requires the candidate region's exact canonical identity to equal the region the law constructs, and a semantically equivalent alternate logical index form is deliberately refused. `ResolvedIndexRealization::verify_sequence` builds the expected realization from the law and refuses unless the two identities are equal; an occurrence whose operation registers no law refuses as `MissingRealizationLaw` before a provider is driven at all.

**Fact — the per-point scalar computation is inside those compared bytes, and the law names it exactly.** `IndexRealizationLaw::PointwiseBinary` carries the applied scalar key. `ConstantFromFloatBits` carries the attribute and the scalar. `StagedRootMeanSquareScaleF32` fixes its whole fold epilogue — divide by the folded contributor count, add the exact bias payload, apply the reciprocal square root — and the law module states why it is fixed rather than parameterized: carrying the epilogue and the pass as data would need a scalar-program language inside a law, which is the universal IR that module refuses.

**Measurement — the refusal is observed on the ordinary compile path against an externally installed provider.** `pipeline::conformance::a_lowering_cannot_replace_the_semantic_providers_realization_law` installs a lowering that emits a structurally valid alternate multiply realization and observes an `UnsupportedCapability` refusal at `phase: "lowering", rule: "refinement-refused"`, explained before planning.

**Fact — the scalar family has no checked path to re-enter.** `legality::refine_index_region` refuses a resolved scalar capability by name as `RefinementError::WrongFamily`, and its module header states that it refines only the index-access family because only that family emits a standalone region. No `refine_scalar_*` authority exists anywhere in the tree; `grep -rn 'refine_scalar' crates/` returns nothing.

**Fact — the staged families moved more scalar expression into the law, not less.** The root-mean-square scale is the first genuinely multi-operation scalar expression the vocabulary carries, and it arrived as law data owned by the registering semantic provider, with every stage's reached scalar authority folded into refinement content and occurrence identity under their own domain tags. The construct that most resembled something a scalar provider would decide arrived decided by the law.

## Decision

### 1. `ScalarLoweringProvider` is a decomposition affordance, not an extension seam

**Proposal.** ADR 0078's own reading is adopted. A scalar decomposition is something an index-access provider may factor its per-point work through — as ordinary code it writes, calls, and shares — and is not a boundary a program reaches through a registry.

**Why this follows rather than being a preference.** ADR 0078 item 1 makes four properties constitutive of a seam and states that a surface which cannot satisfy them is not a seam. The first is that the provider's output re-enters the ordinary checked path. The scalar family's output is a list of scalar value identifiers handed back into a caller-owned region builder, checked by nothing that binds it to a semantic occurrence, and the one authority that could check it refuses the family by name. The classification and the mechanism have simply disagreed since the family landed.

### 2. The family carries no decision, and that is the elimination's core

**Inference.** Because the law fixes the realization byte for byte and the scalar applications are inside the compared identity, a scalar provider resolved anywhere on the compile path would have exactly one admissible output: the sub-expression the law already builds. Any other output changes the realization identity and refuses the whole occurrence. Its answer is therefore determined by an authority it does not own, and installing it decides nothing.

**Why the index-access seam is not retired by the same argument.** An index-access provider's emitted content is dictated identically, and three things nevertheless make it a seam. It is mandatory: no program compiles without a resolvable index-access capability for every recognized occurrence. It is the named authority: its provider identity and capability revision are recorded in the artifact construction plan and re-derived from the installed registry when the plan is built and again when the portfolio is re-verified. And it completes an external semantic registration: a third party defining a new operation registers its own law and must supply the emitter for it, because the host ships none. The scalar family holds none of the three.

### 3. Removal, and what removal is not

**Proposal.** `LoweringFamily::ScalarLowering`, the `ScalarLoweringProvider` trait, `ScalarLoweringContext`, `ScalarLoweringResults`, `LoweringImplementation::ScalarLowering`, `register_scalar_lowering`, `resolve_scalar_lowering`, `ResolvedLoweringCapability::scalar_provider`, and `RefinementError::WrongFamily` with its now-unsatisfiable guard are removed.

**Fact — the removal is identity-preserving.** The capability key encoder writes the family's stable tag, and index access is tag one. Removing the second variant leaves every frozen registry that exists encoding to the same canonical identity bytes, and the governed capability key's family token is unchanged. No ledger, golden, or identity pin is recomputed.

**Proposal — the registry-mechanics tests are ported, not deleted, and this is normative rather than advisory.** Most of the capability module's scalar registrations belong to tests of the registry's own mechanics that merely happen to be written against the scalar family: registration-order identity independence, duplicate collision, the second-signature conflation guard and its per-provider and per-family probes, deterministic ambiguity ordering, the two-revisions ambiguity, the missing-capability diagnostic, missing semantic authority, transactional registration, and capability-revision identity participation. Each must be re-expressed against `register_index_access`. The two-revisions case in particular is cited by ADR 0078 item 3 with its own measured coverage claim, and deleting it would remove that record's evidence. Only three tests are genuinely about the scalar family and go with it: the two-family resolution test narrows to one family, the scalar-provider emission test goes, and the refinement test proving the family is rejected goes with the variant it is the sole constructor of.

### 4. What this record does not decide

**Proposal.** Two shapes fall out of the removal and are reserved rather than settled here, both being public-boundary changes routed to Tom under ADR 0075. `LoweringImplementation` becomes a single-variant enum that could collapse to a bare provider handle. `LoweringFamily` becomes a single-variant non-exhaustive enum whose family token must survive because the governed capability key spells it. This record proposes removing the family and deliberately not collapsing either type; a future second family would want both back.

Nothing here changes the index-access seam, the semantic registry seam, the reference-capability rows, or the physical-implementation provider's classification.

## Consequences

- ADR 0078's item-2 inventory loses a row, and the record's claim that every seam in item 2 realizes item 1's rule without exception becomes true where it was previously false for one row. The supersession is item-2 scoped and stated in prose on both records rather than as a whole-record edge.
- The conformance gate stops owing an installation-evidence row it could never produce. A gate that lists an unproducible row indefinitely trains readers to discount its owed list, which is the cost this removes.
- The affordance survives the seam. Nothing stops an index-access provider from factoring its per-point work into shared code; the host already does exactly that, stating one elementary per-point body once and implementing two sinks over it. What is removed is the claim that such a factoring is a *registered participation boundary*.
- The reservation this decision rests on is named so it can be checked. The output set is a singleton *because* the law comparison is byte-for-byte. Admitting semantically equivalent alternate realizations would give a lowering genuine latitude, and the question of whether per-point latitude should then be separately installable would reopen. That is a reconsideration trigger and not a hedge: it is the same comparison ADR 0078 item 3 warns is the most likely thing a future change erodes.
- Removing a public item from a pre-alpha crate with no external consumer costs nothing in compatibility, and this record claims no compatibility benefit for keeping it either. ADR 0075 already records that Tiler has no external consumer and that the compatibility framing was rejected.

## Alternatives considered

**Wire the family so a compiled program reaches it.** This was the elimination's other candidate and it was tested rather than dismissed. To resolve a scalar capability on the compile path the host would have to own a decomposition of a realization into an index/access skeleton and a splice point for a per-point body, drive the scalar provider at that point, and re-derive that the composition realizes the occurrence. That decomposition is exactly what the law owns whole, and the law module refuses to carry scalar programs as data specifically so a second scalar-expression vocabulary does not appear. Wiring would therefore introduce that second authority over one meaning, in a place with no verifier, to run a provider whose only admissible answer is the one already produced. It loses on correctness, on maintainability, and on performance simultaneously, which is unusual enough to state plainly.

**Wire it in a weaker form: resolve the scalar family but let the index-access provider decide whether to consult it.** Rejected because it makes the registry's answer optional, which contradicts the resolution discipline the optimizer contract states — resolution is unconditional and fails closed, with no default and no preference. A capability a provider may ignore is not a resolution.

**Keep the family registered and unresolved, with the gap documented.** This is the status quo, and it is what the two contracts recorded until this elimination. Rejected because a registered public seam nothing exercises makes a participation promise with no validation, feasibility, or explainability behind it, and because the honest documentation of the gap had already been written twice and read as owed work both times. Documenting a classification error does not correct it.

**Retire it by deleting the family and its tests together.** Rejected as the likeliest wrong execution of the right decision. Ten of the capability module's registry-mechanics tests are written against the scalar family for convenience and cover collision, ambiguity, conflation, transactionality, and identity participation, one of them cited as evidence by ADR 0078 item 3. Deleting them with the family would remove coverage of the surviving seam, so item 3 makes porting normative.

**Defer the question again with a trigger.** Rejected because the question is answerable by reading, which AGENTS.md classifies as research rather than escalation, and it has now been read. A trigger would defer a settled finding, and the one genuine unknown — whether the byte-for-byte law comparison stays — is already recorded as this record's reconsideration trigger.

---
