---
id: restore-adr-0080-verbatim-quotation
title: Restore ADR 0080's verbatim quotation of the wording it corrects
status: todo
priority: p2
dependencies: [let-a-correcting-document-quote-the-text-it-corrects]
related: [let-a-correcting-document-quote-the-text-it-corrects]
scopes: [contracts/decisions]
shared_scopes: []
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
