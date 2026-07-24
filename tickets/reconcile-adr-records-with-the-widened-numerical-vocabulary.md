---
id: reconcile-adr-records-with-the-widened-numerical-vocabulary
title: Reconcile the ADR records with the widened numerical vocabulary
status: in-progress
priority: p1
dependencies: []
related: [widen-numerical-vocabulary-and-complete-identity, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, identity]
claimed_from: todo
assignee: agent-reconcile-adr-records-with-the-widened-numerical-vocabulary
lease_expires_at: 1784932574
---
`widen-numerical-vocabulary-and-complete-identity` implemented ADR 0076 items 1 and 6. `contracts/decisions` was held by a live sibling for the whole of that work, so the decision records still describe the code as it was before. None of these is a wrong decision; each is a statement of fact or status that the implementation has overtaken.

Four edits, all in `docs/decisions/`:

- **ADR 0074 convention 5b, "two sites do not yet meet this"** (`0074-use-explicit-public-api-conventions.md`). It names `push_numerical`'s omission of `input_subnormals`/`result_subnormals` and `fusion_legality::effect_tag`'s wildcard, and says `extend-canonical-identity-encodings-for-reserved-variants` owns closing both. The first half is closed: `crates/tiler-ir/src/schedule/model.rs` now encodes both subnormal dimensions through `push_subnormal` and both permissions through `push_permission`, each an exhaustive match. The `effect_tag` half is untouched and still owned by that ticket. Record which half closed and where, rather than deleting the Fact.
- **ADR 0074's description of the fusion explain encoder** (same file, convention 5b's list of three sites). It quotes `bytes.push(match permission { NumericalPermission::Forbidden => 1 })` in `FusionNumericalProof::canonical_explain_evidence_bytes`. That site now calls `crate::request::permission_tag`, a shared exhaustive tag helper, and the encoded value for `Forbidden` is unchanged at `1`. A fourth site the ADR did not name also existed and is now repaired: `VerifiedRequestSubject::canonical_explain_subject_bytes` encoded all four numerical fields with `as u8` **discriminant casts**, which is strictly worse than a wildcard — a cast reads ordinal position, so reordering variants would have silently changed every encoded request subject with no diagnostic at all. Convention 5b should name the `as`-cast form explicitly; it is the same hazard and the current text does not cover it.
- **ADR 0076's `implementation_status`** (`0076-declare-target-honourable-numerical-realizations.md`), currently `not-started`. Items 1 and 6 are implemented; items 2, 3, 4, and 5 are not. `partial` is the accurate value. The four ordered follow-up tickets under "Implementation boundary" should say which are done.
- **ADR 0076's Facts about the implemented subset.** The Facts at "what the implemented subset is" and "the target's declaration is one boolean" are anchored to `6555119` and stay as written — an anchored measurement is not falsified by later work. The Fact headed "the two sibling identity encodings disagree" carries no anchor and now reads as a present-tense claim that is no longer true. Either anchor it or record that the disagreement is resolved.

Do not restate the implementation in the ADRs. The durable contract already states it: `docs/ir.md` now says the declared numerical realization is inside `IndexRegion`'s canonical structural program, that its encoding is complete over every dimension and exhaustive per dimension, and that no layer may substitute the contract key or a derived predicate for the fields they stand for.

Closes when the four edits land and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

All four edits landed, none of them by deleting a Fact.

1. **ADR 0074 convention 5b's "two sites do not yet meet this"** keeps its original Fact and gains an evidence refresh: `push_numerical`'s half is closed in `1f78223` — both subnormal dimensions through `push_subnormal`, both permissions through `push_permission`, and the two derived `permits_*` booleans gone, because a projection cannot fail closed when its source grows. `fusion_legality::effect_tag` is confirmed untouched by reading it: `OperationEffect` is still `#[non_exhaustive]`, the wildcard is still mandatory across the crate boundary, and `extend-canonical-identity-encodings-for-reserved-variants` still owns it.
2. **The `as`-cast hazard is now a named convention rather than a repaired one-off.** Convention 3 gained an amendment, with a full entry in "Amendments", forbidding *any* tag read from ordinal position — `as`, `std::mem::discriminant`, a `#[repr(u8)]` value read as data, a helper returning an index. The argument recorded is that a discriminant cast is strictly worse than the wildcard the convention already covered: a wildcard is stable under a reorder and mis-encodes only variants that did not exist, while a cast re-encodes *every subject ever produced* with no missing arm and no diagnostic. Also recorded is how the four casts in `VerifiedRequestSubject::canonical_explain_subject_bytes` were actually caught — only because the added variant was a *struct* variant, `FlushToZero { zero_sign }`, which cannot be cast at all; a fieldless variant would have compiled, and one inserted before `Preserve` would have compiled and silently shifted every prior subject. Convention 5b gained the classification sentence so a cast is not read as falling outside the clause, plus the inference that `#[non_exhaustive]` would not have closed the hole (which is why the rule belongs to 3 and not 5). The fusion-encoder quotation was re-pointed at the shared `permission_tag`, encoded value for `Forbidden` unchanged at `1`, and five newly landed 5b sites were enumerated.
3. **ADR 0076's `implementation_status`** moved `not-started` → `partial`, with item 1 marked done in `1f78223`, an "As landed" record of what shipped, and an explicit note of the divergence it introduced — the zero sign became a field of the flush behaviour rather than a resolution against the signed-zero contract, which ADR 0019 has not recorded. Items 2, 3, 4, and 5 stay unstarted; item 4's remit is narrowed by an evidence refresh, since `tiler-artifact` is no longer the four-line shell the record describes.
4. **ADR 0076's Facts** are anchored rather than rewritten. The two already-anchored measurements stay verbatim; the two present-tense claims gained `at 6555119` anchors plus evidence refreshes naming what closed — the identity-encoding disagreement is resolved, and the widened Metal gap vocabulary now has three variants whose per-region (not per-dimension) limitation survives all three.

**A third resolution was found for a choice the record stated as binary.** ADR 0074 previously said closing `effect_tag` requires either removing `#[non_exhaustive]` or moving the encoder. `crates/tiler-artifact/src/program/model.rs` demonstrates a third: keep the mandatory wildcard but make it a *typed rejection* (`UnrecognizedForeignVariant`) rather than a derived value. It preserves convention 3's property and gives up 5b's build-time signal. The record now states all three with their costs, and dates the correction — `f6da4c4` amended the conventions at 12:28 and `d5b6381` added the rejecting encoders at 13:01, so the two-option enumeration was accurate when written and incomplete 33 minutes later.

Verified by reading rather than inferred: `grep -rn " as u8" crates/` matches nothing, so the amendment retrofits nothing and is purely prospective; `subnormal_tag`/`permission_tag` exist in `crates/tiler-compiler/src/request.rs`; `physical::requires_strict_f32` is the exhaustively matched four-dimension *disjunction* claimed; the three `UnrecognizedForeignVariant` rejection sites exist; the three Metal gap variants exist in `crates/tiler-metal/src/record.rs`; `effect_tag`'s wildcard is unchanged; the 192→194 identity byte count is recorded at `crates/tiler-ir/src/schedule/builder.rs:470`; and both commit timestamps are exact.

Three follow-ups filed: `probe-the-non-exhaustive-discriminant-cast-hole` (the one **Inference** among Measurements in that record — it records an out-of-repo reproduction explicitly as *not* citable evidence, and flags that a passing compile has no `.stderr`, so the fixture likely has to pin the contrast), `re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening` (the widening falsified `close-remaining-adr-status-drift`'s stated *reason* for excluding both, which is not the same as establishing the conclusion), and `reconcile-adr-0019-zero-sign-placement-with-the-landed-flush`.

`uv run --locked python scripts/docs.py render` and `tkt lint` pass.
