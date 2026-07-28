---
id: share-the-standard-scalar-registry-across-consumers
title: Move ad-hoc scalar registries onto the standard scalar profile
status: done
priority: p2
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, milestone-0b]
---
`FrozenScalarRegistry::standard()` now exists and governed compiler and
reference fixtures already use it. Ad-hoc registries remain in index-region,
capability, legality, and reference-oracle tests. Several deliberately vary a
provider revision or exercise externally registered scalar vocabulary, while
others may still be historical setup.

Not all of them should move. Classify the remaining sites by the behavior under
test, move only fixtures whose subject is the governed vocabulary, and document
why each intentional custom registry cannot use the standard profile.

**Closing evidence.** Each remaining ad-hoc registry carries a one-line reason
naming what it tests that the standard profile does not, and every site whose
subject is the governed vocabulary composes
`FrozenScalarRegistry::standard()`.

## Outcome — all thirteen classified, none moved, each reason verified (2026-07-27)

**Every `ScalarRegistryBuilder::new` site in the workspace was enumerated and classified — thirteen, across six files.** All thirteen turn out to be deliberate, so nothing moved onto `FrozenScalarRegistry::standard()`. That is the answer rather than a shortcut, and each site now carries a one-line reason naming what it tests that the standard profile does not.

**The classification, by what makes the standard profile unusable:**

- **Varies a provider revision** — `index_region.rs:185` and `index_region_oracle.rs:106` both take the revision as a *parameter*, because their subject is that a revision change moves the registry identity. A frozen profile is one revision.
- **Needs vocabulary the governed profile does not carry** — `state_step`, a defaulted attribute schema, a multi-result scalar, a reducer body, an inferencer that overflows on purpose, a definition with an unclosed dependency.
- **Needs a value sized against a limit** — the reached-definition byte limit and the aggregate registry byte budget. The governed definitions sit deliberately well inside both, so the standard profile cannot reach the preflight under test.
- **Needs a collision a frozen profile cannot stage** — `scalar_registration_failures_are_atomic_and_validate_nested_types` requires two *different* providers registering one definition, which is what `DuplicateDefinition` rejects; a frozen registry has one provider per definition by construction.
- **Deliberately disagrees with the occurrence** — `legality.rs` pairs its scalars with a lowering provider whose extent is a registration-time constant, so a fixture can register a provider that disagrees with the occurrence it is resolved for. A governed provider reads its extents from the occurrence facts and cannot disagree.

**Two of the reasons are factual claims about the standard profile and were checked rather than asserted.** `standard_definition` hardcodes `ScalarArity::exact(1)` for results, so "every governed scalar has result arity 1" holds and the multi-result site genuinely cannot use it. And `ScalarRegistryBuilder::standard()` contains no reducer registration at all, so the reducer-body fixture has nothing to compose.

**The site scrutinized hardest was `capability.rs`, and its reason is a judgement rather than a hard constraint — stated as such.** Its fixtures register an `example` scalar namespace that mirrors the governed shape, so mechanically they *could* be rebuilt on the standard profile. They are kept because the subject there is capability resolution and provider identity, not the vocabulary: binding them to `tiler.scalar::*` would make a future change to the governed profile's arity or attributes break tests that are not about it. A reader who disagrees can see the reasoning at the site rather than having to reconstruct it.

The change is comments only; `make full` passes.
