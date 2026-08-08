---
id: decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span
title: Decide how the link check reads a retained byte-identical drafted ADR span
status: todo
priority: p1
dependencies: []
related: [repair-the-two-dangling-adr-links-in-the-conversion-pair-record, repair-the-eight-dangling-links-in-the-runtime-route-answer-record, resolve-the-markdown-links-the-citation-check-cannot-see]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Ten of the fourteen link failures are one deliberate construct, not fourteen wrong prefixes

The first run of the markdown-link resolution in `check-citations.sh` reported 14 dangling local links and filed five repair tickets. Ten of the 14 are inside a **retained byte-identical drafted ADR body** — a span a research record drafted for `docs/decisions/`, whose links are spelled relative to that destination, which a carrier ticket then transferred byte-identically into the ADR, and which the record keeps as provenance for the transfer.

- `docs/research/runtime/backend-scoped-route-requirement-answers.md` — 8 links, span drafted for [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md).
- `docs/research/numerics/conversion-family-decomposition-across-pairs.md` — 2 links, span drafted for [ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md).

Both records state the condition beside their span and refuse to repoint, in those words. The runtime record: `"Repointing them here is still refused, and now for two reasons rather than one: it would trade a reader's inconvenience for the byte-identity that makes this span quotable at all"`. The numerics record: `"This is stated here rather than repointed, because repointing would break the byte-identity the transfer depends on and a transfer that edits is a fork"`.

**Both destination ADRs are accepted and record the same refusal as their own rationale.** ADR 0102's Work record asserts as present-tense fact that `"the byte-identity the landing established still holds after acceptance"`. Repointing either record therefore falsifies an accepted ADR, and cannot be done from a research scope alone.

AGENTS.md's documentation section supports the refusal without stating it as a convention: `"When a research ticket cannot edit \`docs/decisions/\`, preserve a verbatim-landable ADR body and file a carrier ticket; editing during transfer creates a fork."` The runtime record's claim that AGENTS.md `"has since made \"state the condition beside the span rather than repoint it\" the standing convention"` overstates what AGENTS.md carries at `db3f4d077bf8bd680cacd7a36986f39fec6294f8`, and is a separate repair under `research/runtime`.

## Why this needs one decision rather than two repairs

`repair-the-eight-dangling-links-in-the-runtime-route-answer-record` and `repair-the-two-dangling-adr-links-in-the-conversion-pair-record` were both filed on the premise that the failures are `"a wrong relative path rather than a missing file"`. The filing verified that each target file exists — true, and it does not decide the question, because the path is wrong *for this file* on purpose. Two workers repairing them independently would either fork two accepted ADRs' byte-identity in two separate diffs, or adopt two different workarounds for the same construct.

The construct is also not closed: any future research record that drafts an ADR body it cannot land will produce more of these. The decision should say what the convention is, not only what these two files do.

## Options

1. **Fence the retained span in each record.** `check-citations.sh` already skips fenced blocks, and its header's stated reason is exactly this case — `"content proposed for somewhere else … relative to that directory and not to the ticket that quotes them"`. Preserves every byte of the span, so all four documents' byte-identity claims stay true, and needs no change to the checker. **Cost:** the span renders as literal text — headings, bold, and numbered lists stop rendering — and both records present the span as prose a reader may read. Also removes the span from pinned-citation checking; measured at this base, neither span contains a pinned citation, so the immediate loss is zero.
2. **Teach `check-citations.sh` a documented exclusion,** as it already carries for vendored upstream sources under `docs/research/*/sources/` on the identical ground that `"repairing them would mean editing evidence that is supposed to be a verbatim copy"`. Preserves the rendering. **Cost:** needs a delimiter the script can see, which is a new convention of its own, and it is a carve-out in a checker whose header argues each carve-out individually. Needs `implementation/workspace`; the script is in the delta rule's gated set, so it cannot carry the gate.
3. **Repoint, and fork the byte-identity deliberately.** One change correcting both records, ADR 0092, and ADR 0102, replacing each byte-identity claim with an account of what was spent and why. **Cost:** spends the property four documents were written to protect, and makes the transfer no longer re-verifiable by diff.

## Closes when

One option is chosen and recorded, and the two repair tickets above are rewritten to apply it. `make citations` reports no link failure in either record.

The applying change needs `research/runtime` and `research/numerics` under option 1, additionally `implementation/workspace` under option 2, and additionally `contracts/decisions` under option 3.
