---
id: decide-whether-a-derived-budget-belongs-in-the-request-subject
title: Decide whether a derived budget belongs in the request subject
status: todo
priority: p2
dependencies: []
related: [derive-the-region-shape-budgets-from-the-declaration, state-the-rule-that-a-deterministic-budget-is-a-derivation]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [budgets, identity, research]
---
## The question

`VerifiedRequestSubject::canonical_explain_subject_bytes` writes **all fourteen** deterministic budgets into the canonical request subject, which is carried into artifact identity. That is why every budget widening moves every governed compilation's qualifier — "for programs nowhere near any of these bounds as much as for ones at them, because a budget is a property of the *request* rather than of the plan chosen for it", as the budget comment puts it.

**But if a budget is a pure function of the declaration, and the declaration is already in the subject, then encoding the budget too is redundant** — it cannot distinguish two requests, because any two requests with equal declarations have equal budgets by construction. If that holds, the budget bytes can leave the subject, and **every future envelope widening becomes identity-neutral.**

That is the structural fix underneath [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) and [`state-the-rule-that-a-deterministic-budget-is-a-derivation`](state-the-rule-that-a-deterministic-budget-is-a-derivation.md): those two stop the ceiling being raised by hand; this one would stop the raising from costing anything.

## What must be established before any of that is true

**It is stated as a hypothesis and must not be assumed.** The coordinator did not verify what else the subject carries when filing this, and the whole argument turns on that. Establish, by reading rather than inference:

- **What the subject actually encodes.** Read `canonical_explain_subject_bytes` in full. Does it carry the declaration — input count, output count, occurrence count, the program's own identity — in a form from which every derived budget is recoverable? If the derivation's inputs are *not* in the subject, the budgets are not redundant and this ticket ends there, which is a legitimate outcome.
- **Whether every budget is derivable.** The rule ticket's audit is the input here: a budget that stays a genuine constant is not a function of the declaration, so it cannot leave the subject on this argument. The answer may well be "the derived ones may leave, the constants may not", which is a partial and still-valuable result.
- **Whether the subject has non-identity readers.** The subject is also rendered in explain output. A budget removed from the encoding may still need to be *reported*, and those are different requirements — encoding is about distinguishing requests, reporting is about telling a caller what bounded their compilation. Do not conflate them.

## The counter-argument, which is strong and must be answered rather than noted

**A budget is a property of the request, and the current design says so deliberately.** Two callers who ask for the same program under different budgets are asking different questions, and the subject exists to distinguish requests. The redundancy argument only defeats that if the budgets are *not* independently settable — that is, if the governed profile's budgets are the only ones reachable. **If a caller can state its own budgets, they are an input and must be encoded**, and the derivation changes nothing.

So the first thing to settle is whether budgets are ever caller-supplied, or always a property of the profile. Read `CompileRequest`'s construction and every `DeterministicBudgets` construction site.

## If the answer is yes

It is an encoding step on `tiler.compiler.request-subject.v5` and therefore an identity-domain migration with its own evidence and its own acceptance — **do not land it under this node.** File it, with the enumerated pinned population, and state plainly that the migration's cost is paid once against a permanent reduction in future identity movement. That trade is the decision, and it is Tom's.

## Explicit non-goals

No value changes, no encoding change under this ticket, and no pre-empting the two tickets above. This is research that ends in a decision, not an implementation.

## Closes when

The subject's contents are established by reading; whether budgets are ever caller-supplied is answered; the redundancy claim is proved or refuted for each budget class; and either the encoding step is filed with its population and trade stated, or the ticket records why the budgets must stay and what that costs.
