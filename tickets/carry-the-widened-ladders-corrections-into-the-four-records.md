---
id: carry-the-widened-ladders-corrections-into-the-four-records
title: Carry the widened ladder's corrections into the four records it could not edit
status: done
priority: p2
dependencies: []
related: [widen-the-identity-growth-ladder-to-the-governed-operation-budget]
scopes: [research/artifacts, contracts/decisions, contracts/artifacts, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, measurement]
---
## User-visible outcome

The four records the widened identity-growth ladder found stale or weakened say what the 2026-08-06 measurement established, so a reader of any of them reaches the measured curve rather than the pre-fold one.

## The corrections, owed verbatim by the ladder's Outcome

[`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](widen-the-identity-growth-ladder-to-the-governed-operation-budget.md) § "Owed corrections" enumerates them; that section is the authority and carries the exact replacement text. In summary:

- **`docs/research/artifacts/manifest-fixed-content-growth.md` Section 5** — every figure in its last four paragraphs is pre-fold and now wrong rather than stale: the curve (quadratic → `3525n + 727`), the embedding-ceiling crossing (32/33 → 148/149), the refusal point (695 → 19,038, ~21× → ~128×), the governed-maximum share (2.83× over → 41.8% of), and the "present risk" inference, which inverts. Its out-of-domain-probe sentence is no longer true.
- **`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`** — no number moves; three statements superseded: the fitted domain (2..=8 + probe → 2..=10 with four class-checked walls), the "still refuses on its own wall probe" staleness note (discharged), and the "Bounds on the evidence" out-of-domain confirmation, which **no longer exists** — record that as a weakening, not a silent drop.
- **`docs/artifact-abi.md:247`** and **`docs/ir.md:1138`** — the shared "2..=8 … nine-operation probe outside the fitted domain" phrasing; numbers confirmed, domain statement superseded to 2..=10 with no out-of-domain probe available.

## Closes when

All four records carry the corrections, each read in full around the edit so no adjacent sentence still asserts the superseded state.

## Outcome — 2026-08-06, all four records corrected

**`docs/research/artifacts/manifest-fixed-content-growth.md`** — the deepest sweep, because the whole record was written against the quadratic encoding. The headline inference, Section 5's last four paragraphs (curve, crossing 148/149, refusal 19,038 with the ~128× ordering, governed-maximum 41.8%/34.4%), and Section 6's "Landed" and "What that does not license" paragraphs now state the post-fold measurement; the "present risk" inference is inverted to "a future risk again, measured"; the "Why that conclusion survives the fit being wrong" paragraph is deleted as moot (the quadratic term is zero); the out-of-domain-probe claims are replaced everywhere with the explicit weakening (no out-of-domain confirmation exists; the path refuses above ten operations for non-size reasons, both walls filed as defects and linked). Section 7's counterpoint and Section 8's two boundary paragraphs re-tensed to name the then-quadratic encoding; Section 9 outcome 5 records delivery. The dated `f38813da` re-run paragraph is retained as evidence about the quadratic encoding and says so.

**`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`** — no number moved, per the ladder (all four quoted figures confirmed). The header measurement paragraph states the widened domain (2..=10, four class-checked walls) and the delivered re-run; "Bounds on the evidence" records the weakening explicitly — the 9-operation out-of-domain confirmation no longer exists — alongside what replaced it (residual zero at all nine points including two the record never had), and its re-measurement obligation is discharged. The Context/derivation body keeps its accepted-tense pre-fold figures, as an ADR's derivation must.

**`docs/artifact-abi.md:247`** and **`docs/ir.md:1138`** — the shared "2..=8 … nine-operation probe outside the fitted domain" phrasing replaced in both with the widened ladder and the no-out-of-domain-probe caveat; every numeric claim was already the confirmed post-fold figure and none moved.

Remaining pre-fold figures in all four files are inside explicitly dated or accepted-tense historical paragraphs framed by a corrected current statement; verified by grepping the four files for `2..=8`, `nine-operation`, `695`, `×125`, `32 and 33`, `2.83×` and reading each surviving hit in place.
