---
id: re-audit-adr-implementation-status-after-the-runtime-and-metal-landings
title: Re-audit ADR implementation status after the runtime and Metal landings
status: review
priority: p2
dependencies: []
related: [close-remaining-adr-status-drift, re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening, raise-the-adopted-research-records-to-their-landed-implementation-status]
scopes: [contracts/decisions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, status-drift, graph-repair]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785611640
---
## User-visible outcome

No accepted ADR reads `not-started` beside implemented, gate-covered production code, and no exclusion from the 2026-07 status audit rests on a premise the tree has since falsified.

## Why this exists

**Fact — ADR 0051 reads `not-started` and its decision is implemented.** `docs/decisions/0051-make-runtime-routing-commit-one-way.md:9` reads `implementation_status: "not-started"`. The one-way commit is production code: `Preflight` at `crates/tiler-runtime/src/load/route.rs:579`, declared neither `Clone` nor `Copy` with the comment "A route that could be duplicated could be committed twice, and 'committed once' is the property ADR 0051 asks for", and the infallible `Preflight::commit` at `:740`. The module header at `:7` states the invariant directly, and compile-fail doc-tests at `:671` and `:711` pin double-commit and mint-after-commit.

**Fact — the exclusion's stated premise is falsified.** [`close-remaining-adr-status-drift`](close-remaining-adr-status-drift.md):26 excluded ADR 0051 with the reason "a checked one-way routing policy exists in the compiler-produced plan at `crates/tiler-compiler/src/program.rs:511`, but every clause of the decision is a runtime-launcher behaviour and **no runtime exists**". `crates/tiler-runtime` exists. Its research record, `docs/research/runtime/runtime-execution-contract.md`, still reads `disposition: "adopted"` with `implementation_status: "spike-only"`.

**Fact — ADR 0086 reads `not-started` and calls a `done` ticket blocked.** `docs/decisions/0086-require-attributable-or-attested-native-translation.md:9` reads `not-started`, and `:57` reads "`construct-and-bind-the-first-authoritative-metal-compile-profile` remains blocked." That ticket is `done`, and the refusal the record decides is production code: `MetalHostPredicate::NativeTranslationAuthority` at `crates/tiler-metal/src/applicability.rs:397`, the `MetalHostEligibility` receipt holding the uninhabited authority at `:695-703`, the `UnknownNativeTranslationAuthority` refusal at `:800` reporting the rule `metal.host-applicability.unknown-translation-authority` at `:840`, and `native_translation_authority()` at `:1061` which the comment at `:1056` states can never return a value because the type is uninhabited.

**Fact — five further exclusions were never re-examined, and the audit says so.** `close-remaining-adr-status-drift.md:31` records that after `re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening` superseded two of the eight exclusions, "the other six exclusions in that bullet and the `undetermined` ADR 0034 finding are untouched and were not re-examined." Excluding ADR 0051 (this ticket's headline), those five are **ADR 0012**, **ADR 0014**, **ADR 0020**, **ADR 0022**, and **ADR 0025**, plus the ADR 0034 `undetermined` finding at `:27`.

## Work

Read each record in full against the current tree and move `implementation_status` where the evidence supports it, applying the rule [`close-remaining-adr-status-drift`](close-remaining-adr-status-drift.md) established and [`re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening`](re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening.md) applied: a bump requires the decision's central mechanism to exist in `crates/`, not a type-system reservation or an architectural seam. Give each bumped record an `Implementation boundary` section naming the realized and unrealized clauses, as that precedent did. Correct ADR 0086:57's blocked claim in the same change, and raise `docs/research/runtime/runtime-execution-contract.md`'s `implementation_status` if the ADR bump supports it.

## Boundaries

- The field is a **retained high-water mark**, not a live mirror of the working tree (`docs/document-metadata.md:63`). Never lower one.
- Distinguish the four maturity claims AGENTS.md separates — a type-system reservation, an architectural seam, implemented support, and a tested guarantee. An exclusion re-examined and confirmed is a real outcome; record the confirmation with its current evidence rather than leaving the stale premise standing.
- Scope is `contracts/decisions` plus `research/runtime` for the one research record. The six adopted research records that read `spike-only` against `partial` ADRs are [`raise-the-adopted-research-records-to-their-landed-implementation-status`](raise-the-adopted-research-records-to-their-landed-implementation-status.md)'s; do not absorb them.
- `docs/decisions/README.md` is generated and `implementation_status` is not a catalog input, per `close-remaining-adr-status-drift.md:29`; confirm rather than assume.

## Closes when

ADR 0051 and ADR 0086 carry statuses their production code supports; ADR 0086:57's blocked claim is corrected; each of the five untouched exclusions and the ADR 0034 finding is either bumped or re-confirmed against current evidence with the stale premise replaced; and no accepted ADR is left reading `not-started` beside gate-covered code.

## Outcome

Every record named here was read in full against the tree at `2aa0824`, and every implementing file was read in full rather than sampled: `crates/tiler-runtime/src/{lib.rs,adapter.rs,load.rs,load/host.rs,load/route.rs}`, `crates/tiler-metal/src/applicability.rs`, `crates/tiler-ir/src/schedule/numerics.rs`, `crates/tiler-ir/src/semantic/catalog.rs`, and the reduction-topology, schedule-verifier, fusion-legality, normalization, and catalog spans of `crates/tiler-ir/src/schedule/{model.rs,builder.rs}`, `crates/tiler-compiler/src/{fusion_legality.rs,normalize.rs,physical.rs}`, and `crates/tiler-ir/src/semantic/{registry.rs,contraction.rs,operation.rs}`.

**Seven records moved, each with an `Implementation boundary` section naming realized and unrealized clauses with exact sites.** ADR 0051 and ADR 0086 (the headline bumps), ADR 0012, ADR 0014, ADR 0020, ADR 0022, and ADR 0034 all moved `not-started` to `partial`. No `decision_status` was touched and no decision text was changed except ADR 0086's corrected Consequence. `docs/research/runtime/runtime-execution-contract.md` moved `spike-only` to `partial`.

**The two headline citations were verified line by line and had not drifted.** ADR 0051: `Preflight` at `route.rs:579` declared neither `Clone` nor `Copy`, the infallible consuming `commit` at `:740`, the module header at `:7`, and six compile-fail doc-tests whose fences are at `:379`, `:496`, `:668`, `:683`, `:707`, and `:726` — this ticket's `:671` and `:711` name the `fn` inside two of those blocks, which is the same evidence read one line differently rather than drift. ADR 0086: `MetalHostPredicate::NativeTranslationAuthority` at `applicability.rs:397`, the receipt holding the uninhabited authority at `:695`–`:703`, the refusal at `:800`, its rule key at `:840`, and `native_translation_authority()` at `:1061` with the comment at `:1056`. The structural claim is compile-enforced by `structural_unreachability::every_outcome_is_a_refusal` at `:1082`, an exhaustive match with no `Ok` arm.

**ADR 0086:57 corrected.** The bullet claimed `construct-and-bind-the-first-authoritative-metal-compile-profile` "remains blocked"; that ticket is `done` as of 2026-07-31. What was blocked and stays blocked is the *runtime offer*, not the profile: `tiler_build::BoundMetalCompileDeclaration` binds `tiler.metal.macos-apple9.msl4-0.f32.v1` and `accept_or_publish_metal_plan` verifies against it before emission, while the production offer path observes the host and returns this decision's typed refusal. The correction states that split and cites the ticket's own closing note rather than restating the old sentence.

**ADR 0025 is the one exclusion re-confirmed rather than bumped, and the confirmation is recorded live.** Its own added content over ADR 0022 is proven-neutral *physical padding* as a separately gated capability; nothing pads. `TailPolicy` admits only `Exact` and `ContributorPartition::covers` requires an exact cover, so the "inject padding only with proof" rule is vacuously respected rather than implemented. The empty-domain half it shares with ADR 0022 is real and is recorded in ADR 0022's boundary section. The reopening trigger is the first schedule strategy that pads or masks inactive lanes. [`close-remaining-adr-status-drift`](close-remaining-adr-status-drift.md) now carries that confirmation and the five supersessions in place of the dead premise.

**ADR 0034's `undetermined` finding is resolved into a determination that keeps half of it standing.** The governed catalog at `crates/tiler-ir/src/semantic/catalog.rs` is a single construction site for thirty built-in identities, each with a mandatory normative reference naming authority, document edition, exact format, and preserved source, and an immutable canonical descriptor that alias spellings, lookalikes, unpublished versions, and owner-namespaced same-name identities all fail against. Still true from the 2026-07 finding: `NormativeDefinitionRef` is a `String` validated only for non-emptiness and length, so the four parts are an unparsed convention; and no same-format owner check runs before minting a key — the correctly-external OCP spellings are preserved by a test asserting non-registration rather than by an admission check.

**One divergence found, filed rather than absorbed.** `RuntimeAdapter::plan_dispatch` (`crates/tiler-runtime/src/adapter.rs:371`) takes a `&Preflight`, allocates program storage, and refuses recoverably (`fallback_permitted` reports `Plan` as recoverable at `:533`), while ADR 0051 places allocation after the commit and the adopted research record states that preparation "must not allocate a program output, program temporary, validation record, private transaction result". `spikes/runtime/inline-dispatch/src/adapter.rs:755` implements exactly that, and `crates/tiler-runtime/src/load.rs:11` states the opposite rule in the same crate. This is a contradiction rather than an unimplemented clause, so it is recorded as **Divergent** in ADR 0051's boundary section and owned by [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](reconcile-the-pre-commit-allocation-seam-with-adr-0051.md); the status field was not used to paper over it.

**Measurement — `docs/decisions/README.md` is untouched, confirmed by reading rather than assumed.** Its generated `BEGIN GENERATED ADR TOPICS` rows carry title, `decision_status`, `applies_to` contracts, and `evidence` research records, and no row carries `implementation_status`, so the catalog is not an input this work could invalidate. `contracts/navigation` was not entered.

**Not absorbed.** The six adopted research records reading `spike-only` against `partial` ADRs remain [`raise-the-adopted-research-records-to-their-landed-implementation-status`](raise-the-adopted-research-records-to-their-landed-implementation-status.md)'s. ADRs 0094 and 0095 read `not-started` correctly: their mechanisms genuinely do not exist. No record's `implementation_status` was lowered.
