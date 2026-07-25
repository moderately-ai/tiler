---
id: restore-adr-0080-verbatim-quotation
title: Restore ADR 0080's verbatim quotation of the wording it corrects
status: done
priority: p2
dependencies: [let-a-correcting-document-quote-the-text-it-corrects]
related: [let-a-correcting-document-quote-the-text-it-corrects]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, gate]
---
**Fact.** `ecbe12b` rewrote one sentence of [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) into indirect speech to unblock the repository gate, and said in its own message that this was a workaround rather than the answer. The sentence exists to record *which words* two contracts carried while they were stale, and indirect speech is strictly weaker evidence for that than the words themselves. `let-a-correcting-document-quote-the-text-it-corrects` built the mechanism that lets the quotation come back; this ticket is the one-line restoration, split out only because it lives in `contracts/decisions` and that dependency's editor held `contracts/navigation`.

## Scope

In the paragraph beginning `**Two citations of this fact were stale and are corrected alongside this record.**`, replace

```text
both stated that `StrictF32NumericalContract::governed` was at that time the sole numerical contract the compiler registered.
```

with

```text
both wrote that `StrictF32NumericalContract::governed` "remains the only numerical contract the compiler registers"<!-- superseded-quotation -->.
```

That is exactly the wording `ecbe12b` removed, plus the `superseded-quotation` marker. The marker must sit directly after the closing quotation mark with no space, and the rest of the paragraph — including the `aa7c4f0` citation that tells a reader when the wording stopped being true — is unchanged. `docs/document-metadata.md` states the rule: the marker inverts the quotation obligation rather than lifting it, so the gate will now require this span to appear in *none* of the documents the paragraph links, and will fail if a later edit puts the wording back.

**Measurement — already performed, on a copy rather than in `docs/decisions/`.** The dependency's editor applied this exact substitution to an export of its own working tree and ran `uv run --locked python scripts/docs.py validate --root <export>`: green, 183 records, and the quotation replay reported `mined=324 checked=25 marked=1 findings=0`. So the substitution is expected to land green as written; run the gate anyway rather than trusting that.

## Closes when

ADR 0080 quotes the stale wording verbatim again, `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` both pass, and the Outcome records the gate result.

## Outcome

The substitution landed exactly as specified. [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md)'s paragraph now reads that [Numerical semantics](../docs/numerical-semantics.md) and [the optimizer model](../docs/compiler/optimizer.md) both wrote that `StrictF32NumericalContract::governed` "remains the only numerical contract the compiler registers", with the `superseded-quotation` marker directly after the closing quotation mark and no space. The `aa7c4f0` citation that dates when the wording stopped being true is unchanged, as is the rest of the paragraph and every other sentence of the record.

**The obligation the marker acquires, stated so it is not mistaken for an exemption.** The gate now requires that span to appear in *neither* linked contract. It is absent from both, which is the point — the same change that corrected them removed it. If a later edit restores the wording in either document, or reverts the correction, the gate reports that this record's own claim has become false. That is stronger than the indirect speech `ecbe12b` substituted, which asserted nothing a predicate could check.

**Verified rather than assumed.** `uv run --locked python scripts/docs.py validate` passes at 183 records with the marker in place. The dependency's measurement on an export predicted this and was not trusted on its own.

## A stale copy the restoration surfaced, split out

ADR 0080's paragraph says it corrects two citations "rather than leaving a fourth stale copy". Checking the marker's absence obligation meant grepping the corpus for the wording, and that turned up a governed copy neither it nor this ticket owns.

**Fact.** `crates/tiler-compiler/src/request.rs`'s `StrictF32NumericalContract::governed_profile` returns `[Self::governed(), Self::governed_flush_to_zero()]` and `is_governed` tests membership in it, so the compiler registers two contracts. **Fact.** `docs/roadmap.md:326` still says the fused-multiply-add permission is `Forbidden` "in the only numerical contract the compiler registers today". Reproduce with `grep -rn "only numerical contract" docs/`, which returns exactly that one line. **Inference.** Only the premise's arithmetic is stale; both registered contracts set `contraction: NumericalPermission::Forbidden`, so the sentence's conclusion about a device or library GEMM stands.

That line is `contracts/navigation`, which this ticket does not hold, so it is filed as [`correct-the-surviving-stale-one-contract-claims`](correct-the-surviving-stale-one-contract-claims.md) rather than edited here. Two ticket bodies carrying the same premise are named there too. The new ticket does not touch the marker's obligation — the roadmap is not a document ADR 0080's paragraph links, and its wording differs from the marked span — and it says so explicitly rather than leaving the next worker to work it out.

## Gate

`uv run --locked python scripts/docs.py render` and the full `uv run --locked python scripts/check_repository.py` both pass; `git diff --check` is clean and `tkt lint` reports no problems.
