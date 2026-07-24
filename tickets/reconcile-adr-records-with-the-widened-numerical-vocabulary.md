---
id: reconcile-adr-records-with-the-widened-numerical-vocabulary
title: Reconcile the ADR records with the widened numerical vocabulary
status: todo
priority: p1
dependencies: []
related: [widen-numerical-vocabulary-and-complete-identity, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, identity]
---
`widen-numerical-vocabulary-and-complete-identity` implemented ADR 0076 items 1 and 6. `contracts/decisions` was held by a live sibling for the whole of that work, so the decision records still describe the code as it was before. None of these is a wrong decision; each is a statement of fact or status that the implementation has overtaken.

Four edits, all in `docs/decisions/`:

- **ADR 0074 convention 5b, "two sites do not yet meet this"** (`0074-use-explicit-public-api-conventions.md`). It names `push_numerical`'s omission of `input_subnormals`/`result_subnormals` and `fusion_legality::effect_tag`'s wildcard, and says `extend-canonical-identity-encodings-for-reserved-variants` owns closing both. The first half is closed: `crates/tiler-ir/src/schedule/model.rs` now encodes both subnormal dimensions through `push_subnormal` and both permissions through `push_permission`, each an exhaustive match. The `effect_tag` half is untouched and still owned by that ticket. Record which half closed and where, rather than deleting the Fact.
- **ADR 0074's description of the fusion explain encoder** (same file, convention 5b's list of three sites). It quotes `bytes.push(match permission { NumericalPermission::Forbidden => 1 })` in `FusionNumericalProof::canonical_explain_evidence_bytes`. That site now calls `crate::request::permission_tag`, a shared exhaustive tag helper, and the encoded value for `Forbidden` is unchanged at `1`. A fourth site the ADR did not name also existed and is now repaired: `VerifiedRequestSubject::canonical_explain_subject_bytes` encoded all four numerical fields with `as u8` **discriminant casts**, which is strictly worse than a wildcard — a cast reads ordinal position, so reordering variants would have silently changed every encoded request subject with no diagnostic at all. Convention 5b should name the `as`-cast form explicitly; it is the same hazard and the current text does not cover it.
- **ADR 0076's `implementation_status`** (`0076-declare-target-honourable-numerical-realizations.md`), currently `not-started`. Items 1 and 6 are implemented; items 2, 3, 4, and 5 are not. `partial` is the accurate value. The four ordered follow-up tickets under "Implementation boundary" should say which are done.
- **ADR 0076's Facts about the implemented subset.** The Facts at "what the implemented subset is" and "the target's declaration is one boolean" are anchored to `6555119` and stay as written — an anchored measurement is not falsified by later work. The Fact headed "the two sibling identity encodings disagree" carries no anchor and now reads as a present-tense claim that is no longer true. Either anchor it or record that the disagreement is resolved.

Do not restate the implementation in the ADRs. The durable contract already states it: `docs/ir.md` now says the declared numerical realization is inside `IndexRegion`'s canonical structural program, that its encoding is complete over every dimension and exhaustive per dimension, and that no layer may substitute the contract key or a derived predicate for the fields they stand for.

Closes when the four edits land and `uv run --locked python scripts/check_repository.py` passes.
