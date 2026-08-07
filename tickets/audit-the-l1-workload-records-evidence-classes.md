---
id: audit-the-l1-workload-records-evidence-classes
title: Audit the L1 workload record's evidence classes against its relayed measurements
status: in-progress
priority: p3
dependencies: []
related: [audit-the-ingestion-records-no-measurements-header-claim]
scopes: [research/program-planning]
shared_scopes: []
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-l1-audit
lease_expires_at: 1786076416
---
## User-visible outcome

`docs/research/program-planning/first-metal-lm-workload.md`'s `evidence_classes` names the evidence it actually rests on, verified by a full read.

## The finding, from the L6 header audit

**Fact.** [`audit-the-ingestion-records-no-measurements-header-claim`](audit-the-ingestion-records-no-measurements-header-claim.md) swept every `docs/research/program-planning/` record for the same falsity it was correcting and found exactly one other: L1 carries `evidence_classes: ["primary-source-synthesis"]` while its body relays three **Measurement**-labelled paragraphs — the retained C1 fixture at line 196, the eighteen-position arithmetic at line 202, and the two-way F32 sensitivity envelope at line 241, the last measured on a named host row. Its own label legend at line 28 already lists **Measurement** as a class it uses, so the frontmatter and the record's stated vocabulary disagree.

**Fact.** `flash-class-capability-set.md` was checked and is clean: its single `**Measurement` hit is the label legend, not a measurement. No other `program-planning` record has the gap — L4 and L8 already carry `bounded-measurement`.

## The work

Read L1 in full — the same precondition the L6 audit worked under, because a class list can only be restated honestly against the whole record. Decide whether `bounded-measurement` (an observation holding only for its recorded inputs, environment, and procedure, per [document metadata](../docs/document-metadata.md)) is the honest addition, or whether the three paragraphs are relays that belong to their owning spike. Do not delete a relayed measurement to keep the field as it stands.

If the field moves, `docs/research/README.md`'s catalog line for the record moves with it, and that path is `contracts/navigation` rather than this scope — file or fold it into [`carry-the-l6-bounded-measurement-class-into-the-research-catalog`](carry-the-l6-bounded-measurement-class-into-the-research-catalog.md), which already owns the same drift for L6.

## Closes when

L1's `evidence_classes` and its body agree, verified by a full read, with any catalog consequence routed.
