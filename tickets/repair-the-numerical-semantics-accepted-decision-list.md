---
id: repair-the-numerical-semantics-accepted-decision-list
title: Repair the numerical-semantics accepted-decision list
status: in-progress
priority: p3
dependencies: []
related: [execute-the-adr-0102-acceptance-sweep]
scopes: [contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, numerics, documentation, adr]
claimed_from: todo
assignee: orchestrator
lease_expires_at: 1786048130
---
## User-visible outcome

[Numerical semantics](../docs/numerical-semantics.md)'s ownership-boundary paragraph names every accepted decision that governs it, or names none and points at the catalog instead. Today it names a stale subset and reads as exhaustive.

## The defect, with the check that finds it

The paragraph says: "The accepted decisions are [ADRs 0009–0042](decisions/README.md) together with ADRs 0055, 0059, 0060, 0062, and 0066, with primary support in the [numerical research corpus](research/numerics/)."

**Fact — six accepted decisions apply to this contract and are absent from that enumeration**, and the enumeration was already wrong before the most recent one landed. Reproduce from the repository root:

```sh
for f in docs/decisions/[0-9]*.md; do
  n=$(basename "$f" | cut -c1-4)
  if grep -q 'tiler.contract.numerical-semantics' "$f" && grep -q 'decision_status: "accepted"' "$f"; then echo "$n"; fi
done | sort
```

It prints 45 numbers over a population of 102 numbered decision files. Thirty-nine fall inside the stated ranges. The six that do not are **0076** (declare target-honourable numerical realizations), **0080** (distributivity as a third dimension), **0091** (BF16 conversion families and the accumulator), **0095** (decline a distributivity permission), **0101** (elementary-function identity as a fourth dimension), and **0102** (key conversion families by the ordered pair). Five of the six predate ADR 0102, so this is not that acceptance's damage — the list has been drifting since ADR 0076 landed, and each acceptance sweep since has correctly declined to extend a broken enumeration.

## Why it was not fixed inside the ADR 0102 sweep

[`execute-the-adr-0102-acceptance-sweep`](execute-the-adr-0102-acceptance-sweep.md) held `contracts/numerics` and could have edited the line. It did not, because the sentence's truth never depended on ADR 0102's status — appending one number to a list already missing five would have converted a visible defect into an invisible one, and the sweep's job was to apply an acceptance rather than to repair an unrelated enumeration. The finding is recorded in that ticket's Outcome.

## The decision this ticket has to make

A hand-maintained list of forty-five numbers is a second catalog that must be updated by every future acceptance and has no validator, which is exactly how it reached this state. **Recommendation: replace the enumeration with a pointer** — the paragraph already links [the decisions index](../docs/decisions/README.md), whose "Numerical operations" theme section is the authoritative list and is maintained as part of every acceptance sweep. Deleting the numbers removes a duplicate that can disagree with its source; keeping them adds a step to every future sweep. The counter-argument is that the inline numbers let a reader see the governing set without leaving the contract, and that is real but is bought at the cost the drift measures.

Whichever way it goes, state the choice in the paragraph so the next sweep knows whether it owes an edit here.

## Closes when

The paragraph is true, and it says whether a future acceptance must update it.
