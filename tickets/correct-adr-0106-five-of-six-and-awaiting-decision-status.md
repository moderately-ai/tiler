---
id: correct-adr-0106-five-of-six-and-awaiting-decision-status
title: Correct ADR 0106's five-of-six and awaiting-decision present-tense status
status: done
priority: p2
dependencies: []
related: [retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell, decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, correction, conformance]
---
## User-visible outcome

ADR 0106 no longer claims in the present tense that five of six retained L3 contraction cells route, or that [`decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights`](decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights.md) is `awaiting-decision`. The 2026-08-09 status paragraphs stay as dated history. The live reason `implementation_status` remains `partial` is the admission's withheld public surface and support-matrix authority, not an unpublished sixth cell.

## Why this exists

[`retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell`](retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell.md) landed the accepted retirement and routed `w_vocab_slice` as `l3_member(5)`. Its Outcome named this document as still stale and out of that ticket's `contracts/decisions` hold. A reader of the admission record still meets the retired five-of-six / awaiting-decision present tense.

## Fact audit — 2026-08-12 at `9a93a02da0745ddc6fa7838b6f0c0a583ce741ae`

- **Verified — ADR 0106's 2026-08-09 current-status paragraph still says five of six and names the decision ticket as `awaiting-decision`.** Search `five of the six retained L3 contraction cells` in `docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md`.
- **Verified — the same date's survey-status paragraph still names the proof-payload decision as the remaining incomplete conformance item.** Search `The remaining incomplete conformance item is the proof-payload decision named above`.
- **Verified — both named tickets are `done`.** `tkt show decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights` and `tkt show retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell`.
- **Verified — `MAX_PROOF_PAYLOAD_BYTES` is absent from `crates/`.** Search `MAX_PROOF_PAYLOAD_BYTES` under `crates/` returns no match.
- **Verified — `CONTRACTION_MEMBERS` is `[ContractionMember; 7]` and includes `l3_member(5)`.** Search `pub(crate) const CONTRACTION_MEMBERS` in `crates/tiler-conformance/src/envelope.rs`.
- **Verified — `w_vocab_slice` is a retained L3 cell id.** Search `id: "w_vocab_slice"` in the same file.
- **Verified — this stale present tense is unique among live `docs/` contracts.** Other `five of the six` hits in `docs/` are unrelated research records or historical audit reports, not this admission's L3 routing claim.
- **Verified — catalogs do not restate the five-of-six reason.** `docs/decisions/README.md` lists ADR 0106 as accepted only.

Repairing these present-tense clauses does not change the admission decision.

## What closes this

Add dated corrections immediately after the two 2026-08-09 present-tense paragraphs. Preserve those paragraphs byte-for-byte as history. Withdraw only the live claims that five of six cells route and that the payload-limit ticket is `awaiting-decision`. State that the independent payload limit is gone, `w_vocab_slice` is `l3_member(5)`, and both named tickets are `done`. Keep `implementation_status: partial` and give the remaining reason from item 5 of this record: the admission still withholds a public surface and support-matrix authority. Update no crate, pin, identity, schema, catalog row, or other document.

## Non-goals

Re-auditing every conformance population, flipping `implementation_status` to complete, rewriting architecture or research documents, or editing closed tickets that still quote the five-of-six landing snapshot.

## Outcome — 2026-08-12

Added two dated 2026-08-12 corrections to ADR 0106 immediately after the 2026-08-09 status paragraphs. The historical five-of-six / awaiting-decision sentences remain as dated history. The live status is that all six retained L3 cells now sit in `CONTRACTION_MEMBERS` through `l3_member(5)`, the independent proof-payload limit is gone, and both named tickets are `done`. `implementation_status` stays `partial` because item 5 still withholds a public surface and support-matrix authority.

No crate, pin, identity, schema, catalog, or other document changed. `make citations` and `tkt lint` are the carry gate for this ticket-and-decision-only delta.
