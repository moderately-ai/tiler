---
id: decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim
title: Decide whether ADR 0103's eight-domain count is a dated record or a stale claim
status: in-progress
priority: p3
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786172922
---
`docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md` states, in its consequences:

> **A fourth governed envelope digest domain is admitted.** `tiler.artifact-envelope.identity-digest.v1` joins `manifest-digest`, `section-digest`, and `envelope-digest`. It is separate from `manifest-digest` because that domain covers the manifest bytes this digest is written into. The no-prefix obligation is now over the crate's **eight** domains and is checked over the union of both containers, as the ABI contract already requires normatively.

(In the ADR, "the ABI contract" links to `docs/artifact-abi.md`. The link is flattened to plain text in this quotation because its `../` target resolves relative to `docs/decisions/`, not to `tickets/`.)

**Fact — verified 2026-08-08 at base `6eabf97e`.** The final sentence no longer describes the repository. `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check` established the true population as **eighteen** — the envelope's seven, the sidecar's four, and the artifact program's seven — and the check is no longer "over the union of both containers" but over every domain the crate admits. The count was also never eight at the time: the envelope's manifest framing tag and both payload domains were already admitted and simply uncounted.

**The first three sentences are correct and dated**, and identity-digest genuinely was the fourth *digest argument*.

## The actual question

An ADR is a dated record of a decision, not a live description of the tree, so the default answer may be to leave it and let `docs/artifact-abi.md` carry current state. But this sentence reads in the present tense and cites the contract as agreeing with it, which it now does not. The options:

1. **Leave it.** Consistent with treating an ADR as a point-in-time record. Costs a reader who reaches 0103 first a false present-tense claim.
2. **Append a dated correction** noting the population moved and pointing at the contract, preserving the original text.
3. **Edit the sentence.** Cheapest to read, but rewrites the record of what was decided.

Option 2 matches how this repository has handled superseded ADR language elsewhere and is the recommendation, but the choice is a documentation-convention call rather than a correctness one.

## Why this is a separate ticket

Scope. `docs/decisions/[0-9]*.md` is `contracts/decisions`, which the originating ticket does not hold. AGENTS.md also requires that a ticket unable to edit `docs/decisions/` hand the change over rather than fork it during transfer.

## Closes when

The convention question is answered and 0103 reflects it.
