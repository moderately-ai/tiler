---
id: remove-the-scalar-lowering-family-from-the-compiler
title: Remove the scalar-lowering family from the compiler
status: in-progress
priority: p2
dependencies: []
related: [accept-adr-0105-retire-the-scalar-lowering-seam, resolve-or-retire-the-scalar-lowering-provider-seam, land-the-scalar-lowering-seam-retirement-adr]
scopes: [implementation/compiler, contracts/optimizer, contracts/numerics, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, extension-seams]
claimed_from: todo
assignee: agent-scalar-removal
lease_expires_at: 1786068414
---
## User-visible outcome

`tiler_compiler::capability` registers exactly one lowering family, and it is the one the compile path resolves. The public seam that today registers, resolves, and is tested while nothing on the compile path reaches it is gone, and the registry's own mechanics keep every test they have.

## Authority

[ADR 0105](../docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md), accepted by Tom on 2026-08-06 (provenance in [`accept-adr-0105-retire-the-scalar-lowering-seam`](accept-adr-0105-retire-the-scalar-lowering-seam.md)), decides the retirement and supersedes [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md)'s item-2 row for `ScalarLoweringProvider`. That record's decision 3 is what this ticket executes; its decision 4 is what this ticket must **not** execute. The derivation — the two-candidate elimination, the law-comparison argument, and the symmetric question showing why the index-access seam is not retired by the same reasoning — is [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md)'s Outcome, and the three findings below are carried verbatim from it because each is normative rather than descriptive.

This ticket is unblocked: the acceptance node is `done`, not `awaiting-decision`, so nothing is pending.

## The normative findings, verbatim from the deriving ticket

**Fact — the removal is not thirteen deleted tests, and reading it that way would be the mistake.** `capability.rs`'s test module holds fifteen `register_scalar_lowering` sites and six `resolve_scalar_lowering` sites, and the scalar family is the *vehicle* for ten tests of the registry's own mechanics rather than their subject — each would read identically against any family: `snapshot_identity_is_independent_of_registration_order`, `duplicate_registration_of_one_provider_is_a_collision`, `one_operation_admits_more_than_one_registrable_signature`, `a_second_signature_for_one_family_and_operation_is_refused`, `contradictory_providers_resolve_to_a_deterministic_ambiguity`, `two_revisions_of_one_provider_resolve_to_an_ambiguity`, `a_missing_capability_resolves_to_a_typed_diagnostic`, `registration_rejects_an_operation_without_semantic_authority`, `registration_is_transactional_and_leaves_no_partial_state`, and `capability_revision_participates_in_snapshot_identity`. Every one of those must be **ported** to `register_index_access`, not deleted; several are load-bearing for ADR 0072 (`two_revisions_of_one_provider_resolve_to_an_ambiguity` is cited in ADR 0078 item 3 with its own coverage measurement). Only three are genuinely about the scalar family and go with it: `registers_two_families_and_resolves_each_to_its_provider` narrows to one family, `a_resolved_scalar_provider_emits_through_the_canonical_builder` goes, and `legality::tests::a_scalar_lowering_capability_is_not_an_index_refinement` goes together with the `RefinementError::WrongFamily` variant it is the sole constructor of. A retirement that deleted the ported ten would silently drop the registry's collision, ambiguity, and transactionality coverage, and the implementation ticket must say so.

**What retirement removes, from a full read of `capability.rs` and `legality.rs`.** `LoweringFamily::ScalarLowering` and its `key_token`, `tag`, and `Display` arms; the `ScalarLoweringProvider` trait; `ScalarLoweringContext` and its five methods; `ScalarLoweringResults`; `LoweringImplementation::ScalarLowering`; `LoweringCapabilityRegistryBuilder::register_scalar_lowering`; `FrozenLoweringCapabilityRegistry::resolve_scalar_lowering`; `ResolvedLoweringCapability::scalar_provider`; and, in `legality.rs`, `RefinementError::WrongFamily` with the now-unsatisfiable family guard at `:760-764`. Two consequential shapes fall out and are the implementation ticket's to decide rather than this record's: `LoweringImplementation` becomes a single-variant enum that may as well be `Arc<dyn IndexAccessLoweringProvider>`, and `LoweringFamily` becomes a single-variant `#[non_exhaustive]` enum whose `key_token` must survive because the governed capability key spells it. Collapsing either is a public-boundary change and is reserved to Tom under ADR 0075, so the record proposes the family's removal and deliberately not the enum's.

**Fact — retirement is identity-preserving, which is the one thing that could have made it expensive and does not.** `encode_capability_key` (`capability.rs:1831-1841`) writes `key.family.tag()`, and `LoweringFamily::IndexAccess` is tag `1`. Removing the `ScalarLowering` variant leaves that tag unchanged, so every frozen registry that exists today encodes to the same `CanonicalLoweringRegistryIdentity` bytes before and after. No ledger, golden, or identity pin is recomputed. `LoweringFamily::key_token` likewise keeps `"index-access"`, so no governed capability key moves.

## What this ticket must not do

**Do not collapse either type that becomes single-variant.** ADR 0105 decision 4 reserves both to Tom under ADR 0075: `LoweringImplementation` becomes a single-variant enum that *could* become a bare `Arc<dyn IndexAccessLoweringProvider>`, and `LoweringFamily` becomes a single-variant `#[non_exhaustive]` enum whose `key_token` must survive because the governed capability key spells it. Leave both standing, in their reduced form, and say so in the Outcome. A second lowering family would want both back.

**Do not recompute any identity pin, ledger, or golden.** The removal is identity-preserving by the Fact above. A pin that moves is evidence the removal was executed wrongly — stop and diagnose rather than regenerate. State the check that nothing moved.

**Do not touch the index-access seam, the semantic-registry seam, or the reference-capability rows.** ADR 0105 changes none of them, and its own alternatives section records why the same argument does not retire the index-access family: it is mandatory, it is the named authority in the artifact construction plan, and it completes an external semantic registration.

## Line numbers drift; re-derive rather than following these

Every line number in the findings above is read at `eee734cf` and the deriving ticket records that its numbers had already drifted once and were re-derived rather than assumed. Re-derive on your own base before editing, with the deriving ticket's own two commands:

```sh
grep -n 'resolve_scalar_lowering' crates/tiler-compiler/src/capability.rs crates/tiler-compiler/src/legality.rs
grep -n '^#\[cfg(test)\]' crates/tiler-compiler/src/capability.rs crates/tiler-compiler/src/legality.rs
```

The second prints exactly one line per file, which is what makes the comparison total rather than a sample. `grep -rn 'resolve_scalar_lowering\|register_scalar_lowering\|ScalarLoweringProvider\|ScalarLoweringContext\|ScalarLoweringResults\|WrongFamily' crates/ prototypes/ spikes/` is the population this ticket must empty, and counting it before and after is what makes "nothing left behind" a count rather than an assertion.

## Contract corrections owed in the same change

`contracts/optimizer` and `contracts/numerics` are held here because the deriving ticket's landed corrections describe a live gap that this ticket closes. `docs/compiler/optimizer.md`'s maturity boundary and `docs/correctness-and-testing.md`'s conformance-gate evidence each state the finding — that the family carries nothing to install and that the removal is routed to Tom. Once the family is gone, both must state the end state rather than a routed finding, and neither may leave a sentence claiming a family the crate no longer has. `contracts/foundation` is held for [the operation extension contract](../docs/operation-extensions.md), whose status line and registry-lifecycle `Fact` both say the family is retired **and still in the crate**; both become wrong the moment this lands and must be corrected here rather than left for a sweep.

## Closes when

The population grep above returns nothing outside this ticket's own records; the ported ten tests pass against `register_index_access` with their assertions intact rather than weakened; `two_revisions_of_one_provider_resolve_to_an_ambiguity` still measures what ADR 0078 item 3 cites it for; the two reserved types are reduced but not collapsed; no identity pin, ledger, or golden moved and the check that proves it is stated; the three contracts state the end state; and `make full` is green on the completed change.
