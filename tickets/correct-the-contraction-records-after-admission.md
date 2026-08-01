---
id: correct-the-contraction-records-after-admission
title: Correct the contraction records now that the key is admitted
status: todo
priority: p2
dependencies: []
related: [admit-the-contraction-semantic-profile]
scopes: [contracts/decisions, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contraction, identity]
---
The support-matrix contraction row moved from R1 to R3 when `admit-the-contraction-semantic-profile` registered `tiler::strict-tensor-contraction-f32@1`. Two records outside that ticket's scopes still describe the earlier state, and a reader acts on them as fact.

**Fact.** [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) closes with "The [support matrix](../docs/roadmap.md#operation-family-support-matrix) contraction row stays at R1 — identity is decided, nothing is admitted." That sentence was true when the decision was accepted and is now false; the record's `implementation_status` is also still `not-started`. Correct the status field and the traceability sentence without rewriting the decision or its rationale: an ADR records what was decided, and what changed is the implementation state it reports.

**Fact.** The [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md) states **D-8** — whether the per-combine canonical-NaN obligation is realizable in a matrix instruction — as closing "when the contraction key's numerical signature is admitted and states whether per-combine canonicalization is required of a contraction or only at its result boundary". The registered signature now states it: `CONTRACTION_F32_FACT_NAN_CANONICALIZATION` is `after-every-combine-and-at-the-result-boundary`, matching the registered strict serial sum. Record D-8's closure with the ground for it — the stronger obligation was chosen so a later relaxation is a deliberate widening rather than a silent tightening, and so a materialized split-reduction partial carries a canonical payload — rather than only its outcome. The same record's "nothing here registers a contraction key" framing should also say what has since registered one.

**Non-goals.** Reopening ADR 0087, changing the numerical signature, or moving any other support-matrix row.

## Closes when

ADR 0087's implementation status and traceability sentence describe the admitted state, and the L3 record's D-8 is resolved against the registered signature with its derivation stated rather than asserted.
