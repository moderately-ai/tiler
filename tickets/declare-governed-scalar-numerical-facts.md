---
id: declare-governed-scalar-numerical-facts
title: Declare the governed scalar operations' numerical facts instead of leaving them empty
status: done
priority: p1
dependencies: []
related: [register-governed-scalar-reference-evaluation, reconcile-single-contributor-strict-sum-nan-canonicalization]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, ir, contracts, milestone-0b]
---
The three governed scalar definitions declare *no* numerical facts, so the rounding and NaN-canonicalization rules a scalar reference oracle and an external index-access lowering provider must implement exist nowhere in the scalar authority.

**Fact (inspected source).** `crates/tiler-ir/src/index/scalar.rs::standard_definition` builds every governed scalar contract with `CanonicalValue::record([])` for both the `facts` and the `conformance` argument of `ScalarOperationContract::new`. All three of `tiler.scalar::constant-f32@1`, `multiply-f32@1`, and `add-f32@1` therefore carry an empty facts record and an empty conformance record. Their only normative statement is the `NormativeDefinitionRef` string — for example `"IEEE 754-2019 binary32 addition; tiler.scalar::add-f32@1"` — which names the IEEE operation and says nothing about which NaN payload a result carries.

**Fact — the semantic layer one level up does declare them.** `crates/tiler-ir/src/semantic/registry.rs` gives `tiler::multiply-f32@1` and `tiler::add-f32@1` the `arithmetic_f32_facts()` record: `"binary32-round-to-nearest-ties-even"`, `CANONICAL_F32_ARITHMETIC_NAN_BITS`, and a contraction-permitted boolean of `false`. `tiler::strict-serial-sum-f32@1` declares `"strict-left-fold"`, `"binary32-each-step"`, and the same canonical NaN bits. Each also carries a `standard_conformance(...)` identity. The scalar operations the governed lowerings emit to realize those very families declare none of it.

**Inference — the requirement currently lives only in implementations.** `register-governed-scalar-reference-evaluation` shipped `FrozenScalarReferenceRegistry::standard()`, whose `add`/`multiply` canonicalize an arithmetic NaN and whose `constant` does not. That behaviour is correct — it is what the tensor-level oracle does, and its test suite pins it against explicit non-canonical NaN bit patterns — but it was derived from the *semantic* operation facts and from reading `tiler_reference::binary`, not from anything the scalar authority states. A second reference capability for `tiler.scalar::add-f32@1`, or a third-party index-access lowering provider emitting it, has nothing to conform to and would be within its declared contract to propagate the host payload instead. That is a divergence the registry would admit without complaint, because `legality::check_authority_conformance` checks *which* scalar operations a region reaches, never what they compute.

**What this ticket must produce.**

1. Populate `facts` on the three governed scalar definitions with the rounding and canonical-NaN statements their semantic counterparts already carry, so the scalar authority is self-contained. `constant-f32` must state the opposite rule explicitly — an exact payload is reproduced verbatim and is *not* canonicalized — because "no fact" and "no canonicalization" are currently indistinguishable and only one of them is true.
2. Give each a `conformance` identity, as `standard_conformance` does at the semantic layer, so a capability can name which conformance revision it implements.
3. Decide whether the scalar facts should be *derived* from the semantic operation facts rather than restated. Two independently written records that must agree are a drift hazard of exactly the kind this repository has already been bitten by; if they stay separate, add a test that the governed scalar facts and the semantic operation facts agree on the canonical NaN payload.

**Expect an identity rebaseline, and treat it as the evidence.** `encode_definition` encodes both `facts` and `conformance` into every scalar definition, so `CanonicalScalarRegistrySnapshotIdentity` changes when they stop being empty. That cascades into `FrozenScalarReferenceRegistry::standard()`'s canonical identity, into `CanonicalLoweringRegistryIdentity`, and into any fixture that pins them. Record the before and after rather than only asserting the new values: a snapshot identity that did *not* move would mean the facts were not encoded.

Blocked-adjacent to `reconcile-single-contributor-strict-sum-nan-canonicalization`, which decides whether a zero-arithmetic-step fold canonicalizes. Do not write a scalar fact that presumes that answer.

## Outcome

All four governed scalar definitions now declare numerical facts and a conformance identity, and the drift hazard is closed by a test rather than by a shared constructor alone.

**Correction — the ticket counted three governed scalars; there are four.** `tiler.scalar::canonicalize-nan-f32@1` was admitted after this ticket was written, and it carries the same empty records the other three did. It is not an incidental fourth: it is the operation that made the *derive-versus-restate* question in item 3 decidable, because it is the one governed scalar with no semantic counterpart at all.

**Item 3 decided: restate and check, do not derive.** `crates/tiler-ir/src/index/scalar.rs::the_canonical_nan_conversion_has_no_semantic_counterpart` records the fact behind the decision — `FrozenSemanticRegistry::standard()` has no `tiler::canonicalize-nan-f32@1`, reproducible in one line as `operation_definition(&OpKey::new("tiler", "canonicalize-nan-f32", 1)?).is_none()`. A rule that copied each scalar's facts from its semantic operation would have no source for it. The two layers also mean different things by a "fact": the semantic record governs a whole-tensor operation family, the scalar record governs one per-point application, and the semantic layer's own field numbering is already operation-local rather than global (`arithmetic_f32_facts` puts the canonical NaN payload at field 2, `strict-serial-sum-f32` puts it at field 3). Deriving one from the other would have forced a correspondence that does not hold.

What replaces derivation is narrower and stronger than a shared record. `canonical_f32_bits` moved from a private function in `semantic/registry.rs` to `pub(crate)`, re-exported as `crate::semantic::canonical_f32_bits`, so both layers build the payload through one constructor and cannot disagree on the format key or the big-endian byte order. On top of that, `scalar_and_semantic_facts_agree_on_the_canonical_payload` compares the payloads the two records actually declare, collecting them by canonical value category rather than by field ID — so it still fires if either layer renumbers a field or drops one, which the shared constructor alone would not catch.

**The scalar fact vocabulary is uniform where the semantic one is not.** Four field IDs, each with one meaning across all four definitions: rounding (1), NaN-result rule (2), canonical NaN payload (3), contraction permitted (4). Fields 1 and 2 are stated by every governed scalar; 3 and 4 are stated only where they are defined.

That split is the substance of item 1's requirement that `constant-f32` state the preserving rule *positively*. Field 2 is always present, so it — not the absence of field 3 — is what a consumer reads to learn the payload rule. `constant-f32` states `"declared-payload-preserved"`, matching `docs/numerical-semantics.md` ("Constants retain their declared bit pattern until an operation's semantics produce a new value"); the other three state `"tiler::canonical-arithmetic-nan-f32@1"`, the exact versioned profile that document names rather than a synonym. Field 3 is then omitted on `constant-f32` because it installs no payload, and omitting it is now unambiguous rather than load-bearing.

Field 4 is stated only on `multiply-f32` and `add-f32`. Contraction is defined over a pattern of arithmetic operations; a constant and a conversion are not participants, and declaring `false` for them would answer a question the numerical contract does not pose about them. `contraction_is_stated_exactly_where_it_is_defined` pins both directions.

**Item 2: conformance identities are domain-separated from the semantic layer's.** The prefix is `tiler.scalar.conformance.`, not the semantic layer's `tiler.conformance.`. The two layers govern different contracts over the same operation names, so a shared string would have given two subjects one identity — the defect class this repository has closed three times. `scalar_conformance_is_domain_separated_from_semantic_conformance` asserts the two differ and pins both spellings. The identity is derived inside `standard_definition` from the key it is registering, so a definition cannot be registered under one name while claiming conformance to another.

**Measurement — the identity rebaseline, before and after** (macOS arm64, pinned nightly `nightly-2026-07-19`, base `f286289`). `FrozenScalarRegistry::standard().snapshot_identity()`:

| | bytes | sha256 |
| --- | --- | --- |
| before | 1136 | `5f7be5b2c04e6b3fc6e58bbac6d832b589f5458701636eb50744fc0a616fc549` |
| after | 1875 | `30fad8af489a6e4f2e81d4cf067d4e790ac0df6552884b54dbaf44aeb5367c35` |

The snapshot moved, which is the evidence the ticket asked for: a value that had not moved would mean `encode_definition` was not reaching the new records.

**Scope note — `implementation/compiler` was added to this ticket, and why.** The rebaseline cascaded into exactly one pinned fixture across the workspace's 674 tests: `crates/tiler-compiler/src/explain.rs::deterministic_trace_is_sealed_and_rendered_separately`, whose rendered request digest moved from `315e14544407d942` to `eeb25d3a45eebfd4`. That literal exists to be rebaselined by this class of change and says so in a comment already at the site — the request subject covers the frozen scalar authority, so a digest that survived would mean the subject reached the operation keys without reaching their contracts. The cascade is not separable into a follow-up: any interim in which the facts are declared and the digest is not rebaselined leaves the repository gate red for every concurrent worker. No in-progress ticket held `implementation/compiler` when the edit was made. The change is the literal plus its comment; nothing else in `tiler-compiler` was touched.

**Not done here, split out.** `expose-the-governed-fact-field-vocabulary` records that `ScalarOperationDefinition::facts()` and `OperationDefinition::canonical_facts()` are both publicly readable while the field IDs needed to interpret them are private to their modules, at both layers. Publishing that vocabulary is a new public surface and therefore owner-reserved, so this ticket matched the established precedent — the semantic layer keeps its field IDs private too — rather than widening the boundary unilaterally.
