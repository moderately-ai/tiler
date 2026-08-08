---
id: repair-the-two-dangling-adr-links-in-the-conversion-pair-record
title: Repair the two dangling ADR links in the conversion pair record
status: blocked
priority: p2
dependencies: [decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span]
related: []
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786167947
---
## What is broken

`docs/research/numerics/conversion-family-decomposition-across-pairs.md` links to two ADRs as bare filenames, as if the record lived in `docs/decisions/`:

- `](0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)`
- `](0041-separate-float-to-integer-conversion-families.md)`

Both resolve against `docs/research/numerics/` and find nothing. Both files exist at `docs/decisions/<same-name>`.

Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`.

## Per-Fact audit, 2026-08-08 at `db3f4d077bf8bd680cacd7a36986f39fec6294f8`

- **Verified.** Both link targets are spelled exactly as quoted above. Both sit on the one line carrying `"decided that BF16/binary32 conversion is two directional families with disjoint field sets"`, in the `### Context` section of the record's `## Drafted ADR body` span, and they are the only two markdown links anywhere inside that span.
- **Verified.** Both resolve against `docs/research/numerics/` and find nothing. `./check-citations.sh` reports `no tracked file or directory at docs/research/numerics/0091-…` and the matching `0041-…` line.
- **Verified.** Both targets exist at `docs/decisions/<same-name>`, and each names the ADR its sentence's claim actually rests on: 0091 is cited for "two directional families with disjoint field sets", which is ADR 0091's item 2, and 0041 is cited for "four float-to-integer families that differ in three fields", which is ADR 0041's decision. Neither is transposed.
- **Verified.** The check was blind to links before `resolve-the-markdown-links-the-citation-check-cannot-see`, and that ticket's Outcome names this ticket as one of the five it filed.
- **FALSE — "so only the relative prefix is wrong", and "the fix is the relative path, not the check".** The spelling is deliberate, is documented twice, and one of those documents is an accepted ADR this ticket's scope cannot edit.
  - The record states it itself, four lines above the span: `"The two ADR links in the Context paragraph below are written \`docs/decisions/\`-relative and therefore do not resolve from this record."` It goes on: `"This is stated here rather than repointed, because repointing would break the byte-identity the transfer depends on and a transfer that edits is a fork"`, and directs a reader wanting those ADRs to the record's own Traceability section, where both are spelled correctly for this location.
  - [ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md), **accepted**, records the same refusal in its Work record: `"The two ADR links in the Context paragraph are the drafted body's own: they are spelled \`docs/decisions/\`-relative for this destination, which makes them broken in the record that carries them, and that record states the fact beside the span rather than repointing — repointing would fork the byte-identity the transfer depends on."` The same paragraph asserts as a present-tense fact that `"the byte-identity the landing established still holds after acceptance"`.
  - So repointing does not repair a mistake; it falsifies an accepted ADR's stated fact. Correcting ADR 0102 to match would need `contracts/decisions`, which this ticket does not hold. The repair is unavailable inside this ticket's scope as written.

**The same false premise was filed against a sibling.** `repair-the-eight-dangling-links-in-the-runtime-route-answer-record` covers `docs/research/runtime/backend-scoped-route-requirement-answers.md`, whose eight failures are all inside its own retained drafted-ADR span for ADR 0092, and whose prose already states `"Repointing them here is still refused"`. Ten of the fourteen links the first link-check run reported are this one construct, not fourteen independent wrong prefixes; the filing ticket's `"every one is a wrong relative path rather than a missing file"` verified only that the target file exists, which is true and does not decide the question.

**Unverified, and noted for whoever holds `research/runtime`.** That record claims `"AGENTS.md's docs-maintenance section has since made \"state the condition beside the span rather than repoint it\" the standing convention for a drafted ADR body"`. AGENTS.md's documentation section carries no such sentence at this base; what it says is `"When a research ticket cannot edit \`docs/decisions/\`, preserve a verbatim-landable ADR body and file a carrier ticket; editing during transfer creates a fork."` That supports the refusal but is thinner than the claim made of it.

## What actually has to be decided

`check-citations.sh` resolves link targets outside inline code spans and outside fenced blocks, and its header already argues one exclusion of exactly this shape — vendored upstream sources, `"repairing them would mean editing evidence that is supposed to be a verbatim copy"` — plus a fenced-block rule whose stated rationale is `"content proposed for somewhere else … relative to that directory and not to the ticket that quotes them"`. A retained byte-identical drafted ADR body is that construct, reached through a second spelling.

Three candidate resolutions, none of which this ticket's scope can settle alone because any of them must apply the same way to both records:

1. **Fence the retained span** in each record. Keeps every byte of the span, so both records' and both ADRs' byte-identity claims stay true, and uses the checker's own existing mechanism with no change to it. Costs the span's rendering: headings, bold, and numbered lists become literal text.
2. **Teach `check-citations.sh` a documented exclusion** for a retained drafted span, as it already carries for vendored sources. Keeps the rendering. Needs `implementation/workspace`, and needs a delimiter the script can see.
3. **Repoint, and fork the byte-identity deliberately**, correcting ADR 0092, ADR 0102, and both records in one change. Needs `contracts/decisions` and both research scopes, and spends the property three documents were written to protect.

Filed as `decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span`.

## Closes when

The convention question above is decided, and this record's two links are brought into line with it. `make citations` reports no link failure in this file.

**Not closable by adding `../../decisions/` to the two targets.** That is the one repair this ticket originally prescribed and it is refused by the record it edits and by an accepted ADR.
