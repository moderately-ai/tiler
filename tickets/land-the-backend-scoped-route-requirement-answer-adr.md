---
id: land-the-backend-scoped-route-requirement-answer-adr
title: Land the backend-scoped route-requirement answer ADR as proposed
status: done
priority: p2
dependencies: [design-the-adapter-owned-route-requirement-answer-channel]
related: [dispatch-a-tiler-region-on-metal-hardware, close-the-metal-gpu-family-out-of-crate-total-map]
scopes: [contracts/decisions, contracts/navigation, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, runtime, backends, metal, public-boundary]
---
## User-visible outcome

The backend-scoped route-requirement answer design exists as a `proposed` ADR under `docs/decisions/`, listed in the decision catalog, and the research record behind it is listed in the research catalog — so a reader arriving at either index finds it instead of a record nothing points at. Today the design is complete and lives only in `docs/research/runtime/` because the ticket that produced it could not reach either path.

## Why this is a separate ticket and not an omission

**Fact — the scope map, checkable in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions`, and maps `docs/decisions/README.md` **and `docs/research/README.md`** to `contracts/navigation`:

```sh
rg -n 'contracts/decisions|contracts/navigation' -A 14 ticketsplease.toml
```

**Fact.** `design-the-adapter-owned-route-requirement-answer-channel` holds `research/runtime` and `research/extensions` exclusively and `project/tickets` shared, and holds neither of the two scopes above. Writing an ADR file or editing either catalog from that branch is a guard escape. This is the same split [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) records for the BF16 design, and the idiom is copied deliberately.

**Note — two catalog rows, not one.** The BF16 precedent carried only the decision catalog because its research record's row already existed. This design's research record is new, so `docs/research/README.md` needs a row too, and that file is `contracts/navigation` as well.

## What already exists and must be used rather than rewritten

[Backend-scoped route-requirement answers](../docs/research/runtime/backend-scoped-route-requirement-answers.md) carries a **Drafted ADR body** section written to be landed verbatim: context, nine numbered decisions, consequences, and six alternatives-considered entries each with the elimination stated. It also carries the b1/b2 derivation, both worked examples, the public-boundary list, the measurement boundary, and the deferrals. Do not re-derive any of it.

## Implementation keys

- Create `docs/decisions/00NN-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md` with the frontmatter the record's drafted body states — `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "runtime-integration-placement"`. Take the next free number by reading the directory, not by remembering one: `0090` was the highest at `6f7caf3` and a sibling may have landed since.
- Add the row to the **proposed** section of `docs/decisions/README.md`, and to the numeric index further down, in the same change. An ADR appears in both.
- Add the research record's row to `docs/research/README.md` under `### Runtime, integration, and placement`, in title order — it sorts after "Autoregressive state and KV-cache ownership" and before "Candle Metal post-wait error checking". Copy the format from the neighbouring rows rather than from here: a list item whose link text is the title and whose target is the path relative to `docs/research/`, then an em dash, the disposition, a semicolon, the comma-joined evidence classes, a semicolon, and `informs:` followed by links to each `informs` contract and each `adopted_by` ADR. Its disposition is `pending` and its evidence class is `primary-source-synthesis`; it cites no experiment, so it carries no `experiments:` clause.
- Add the traceability block the sibling ADRs carry, pointing at [System architecture](../docs/architecture.md), [Artifact envelope and Metal kernel ABI profile](../docs/artifact-abi.md), the research record, and this ticket.
- Set `adopted_by` on the research record only if and when the ADR is *accepted*. A proposed ADR is not an adoption, and the record's `disposition` stays `pending` until Tom accepts. **Do not add a `related` link to the research record's frontmatter** — `docs/document-metadata.md` does not license `related` for `kind: research`, and the BF16 carrier ticket's instruction to add one should not be copied.

**Scope addition — 2026-08-01, during the work.** `research/runtime` was added to this ticket's scopes, and the addition reaches *prose only*: the research record's frontmatter is correct as written and did not change, exactly as the two keys above require. The reach is needed because "Closes when" requires the record to link the ADR in its prose and the "Drafted ADR body" heading to stop reading as the pending authority, and `ticketsplease.toml` maps `docs/research/runtime/**` to `research/runtime`, which this ticket did not hold. This is the same addition [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) made for `research/numerics`, one motive weaker — there the disposition move was real and the frontmatter had to change; here nothing in the frontmatter may move while the ADR is `proposed`. It collides with no live work: `design-the-adapter-owned-route-requirement-answer-channel`, which holds `research/runtime` exclusively, is `done`.

## Explicit non-goals

- **Do not accept the ADR.** Acceptance of a public boundary is Tom's, and the record enumerates seven boundary items including reclassifying `tiler-metal` as a crate a consumer may name. Landing it `proposed` is the whole of this ticket.
- **Do not amend `docs/architecture.md`.** Decision item 6 restates "a consumer names `tiler` alone" as a property of the non-dispatching consumer, and that sentence lives in `contracts/foundation`, which this ticket does not hold. It is part of the acceptance sweep, not of landing a proposal.
- **Do not implement anything.** No crate gains an item, no test changes, and `spikes/runtime/inline-dispatch` stays fail-closed.

## Closes when

The ADR file exists with `decision_status: proposed`, both decision-catalog views list it, the research catalog lists the research record, the record links the ADR in its prose, and `make full` is green.

## Graph maintenance

- Depends on the design ticket, whose Outcome states exactly what the ADR must say.
- Gates nothing that exists. Implementation is a separate phase decision under the implementation boundary and has no ticket, deliberately: research completion does not authorize scaffolding.
- If Tom accepts the ADR in the same session, the acceptance sweep — catalog views, the architecture-contract sentence, and any released work — is that acceptance's own change and not this one.

## Outcome — 2026-08-01

**[ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md) — "Answer backend-scoped route requirements in the owning backend's vocabulary", `decision_status: proposed`.** Frontmatter is exactly what the record's drafted body specifies: `implementation_status: not-started`, `catalog_group: "runtime-integration-placement"`, `applies_to: ["tiler.contract.architecture", "tiler.contract.artifact-abi", "tiler.contract.metal-backend"]`, `evidence: ["tiler.research.runtime.backend-scoped-route-requirement-answers"]`, `depends_on: ["ADR-0074", "ADR-0075", "ADR-0081", "ADR-0086", "ADR-0090"]`, `ticket: "land-the-backend-scoped-route-requirement-answer-adr"`. The number was taken by reading the directory, and the implementation key's warning earned its keep: `0090` was highest at `6f7caf3`, `0091` had landed since, and `0092` was free.

**It landed proposed and nothing reads as accepted, which is where this differs from the BF16 carrier rather than merely copying it.** The status paragraph states that Tom's recorded acts are the design-ticket route with a b2 lean and the eliminations of candidate (a) and of fail-closed-forever-as-terminus, all on 2026-08-01, and that a lean toward a candidate is not an acceptance of the model the candidate produced. The **seven** boundary items are restated in the ADR by name and marked unaccepted, and the record's proposal-era disclosures were preserved rather than swept — the **Proposal** labels, the `disposition: pending`, and the "not self-accepted" framing all stand untouched. `adopted_by` was not set and no `related` edge was added, per the two implementation keys; the record's frontmatter is byte-unchanged. Nothing gated on this was released: no ticket was unblocked, `docs/architecture.md` was not amended, and no crate, test, or spike was touched. *Note for the next carrier:* the coordinator's brief said "six-item public-boundary list"; the record and this ticket both say seven, and seven is right — one item fires under ADR 0075's mechanical categories and six under AGENTS.md's broader clause, which is likely where the six came from.

**The body transferred verbatim, and it was checked rather than asserted.** Context, the nine numbered decisions, consequences, the six alternatives-considered entries, and the traceability paragraph were lifted from the record's span between the two horizontal rules. `diff` between that source range and the ADR's `## Context`-through-traceability span reports **no differences** once `### ` is mapped to `## `, and the raw `diff` reports **exactly five changed lines, all headings** — `Context`, `Decision`, `Consequences`, `Alternatives considered`, `Traceability` — because the record nests them one level under its own `##` heading and the ADR carries them at `##` under its title. That is the one deliberate deviation and it is a nesting level, not content. The check was proved able to fail before being trusted: substituting `Unsupported` for `Unrecognized` in the ADR's span made the normalized `diff` report three differing lines. Everything the ADR says beyond the span is *about* the decision: the status paragraph, the seven-item boundary restatement, the repeated measurement boundary (nothing compiled, nothing measured, every shape a type-system reservation), an implementation boundary naming item 5's anti-vacuity-guard obstacle, the five deferrals adopted by reference, and three traceability links the drafted paragraph omits — the `tiler-metal` backend contract, which is an `applies_to` target the prose would otherwise never name, and both work-record tickets.

**Three catalog rows, not two — the ticket's own note was right and understated by one.** `docs/decisions/README.md` gained a theme row under *Runtime, integration, and placement*, alphabetically after ADR 0081 ("Admit" before "Answer") and before ADR 0003, and a chronology row after 0091; both read `proposed`, and **this is the first `proposed` row in either view** — `rg -l 'decision_status: "proposed"' docs/decisions/` returned nothing at `cb5d86a`, so the form was derived from the row grammar rather than copied, and the index preamble already anticipates it ("Proposed ADRs and design text remain non-decisions until explicitly accepted"). `docs/research/README.md` gained the record's row under the same theme, between "Autoregressive state and KV-cache ownership" and "Candle Metal post-wait error checking" as predicted: `— pending; primary-source-synthesis; informs:` the three contracts its frontmatter names, with **no `experiments:` clause** and **no ADR link**. Both omissions were checked rather than assumed. No experiment record `supports` this research record — `tiler.spike.runtime` names three others and `tiler.spike.runtime.inline-dispatch` names one, and none is this one, though the inline-dispatch README does cite it in prose. And the row lists no ADR because the row mirrors `informs` plus `adopted_by`, which is confirmed by the neighbouring `backend-provider-composition` row: it is `pending`, and it does **not** list ADR 0090 even though 0090 is `accepted` and cites it as evidence.

**The record's prose was demoted to provenance, which needed a scope this ticket did not hold.** `research/runtime` was added under the dated note above, reaching prose only — the frontmatter did not move and must not while the ADR is `proposed`. The "Drafted ADR body, written to be landed verbatim" heading became "landed as ADR 0092 on 2026-08-01, `decision_status: proposed`" with the span retained and explicitly demoted from authority to provenance; the paragraph explaining why the ADR could not be written from the design branch is preserved as the reusable part, and it now also records that this record's *own* catalog row was absent from `docs/research/README.md` entirely — the same scope split found one document further out, exactly as the BF16 landing found it. The "Public-boundary items" preamble gained the sentence that stops the misreading running the other way: the ADR is proposed, so it accepts neither the model nor any of the seven items.

**A pre-existing defect was found, bounded, and deliberately not repaired.** Eight local links inside the record's drafted-body span do not resolve from the record's own location, because the span was written with paths relative to `docs/decisions/`: `0074`, `0075`, `0081`, `0086`, and `0090` as bare siblings, `../architecture.md` and `../artifact-abi.md` one level short, and the traceability paragraph's self-citation pointing at `../research/runtime/…`. **All eight were already broken at `cb5d86a`** — verified by writing the base blob into the record's real directory and resolving from there, after a first attempt that checked it in an isolated scratch directory reported all fourteen broken and therefore proved nothing. All eight resolve correctly inside ADR 0092, which is where a reader should follow them from. Repointing them in the record would trade a reader's inconvenience for the byte-identity that makes the span quotable, so the condition is documented in the demotion paragraph instead. The BF16 record does not have this problem because its drafted blockquote carries no traceability section; **a drafted ADR body that includes one will hit this every time**, and the cheap convention — write the drafted traceability with record-relative paths and adjust on transfer, or keep it out of the drafted span — is a coordinator/Tom call rather than a defect ticket, so it is reported rather than filed.

### Verification

`tkt lint` (ok: no problems found), `git diff --check` (clean, exit 0), `tkt guard --base cb5d86a`, and `make full`.

Local links were resolved against the filesystem with counted populations, because a checker that silently matched nothing would report the same clean result as one that checked everything: 16 links in ADR 0092, 504 in `docs/decisions/README.md`, 325 in `docs/research/README.md`, and 6 in this ticket — **851 links across four files, none broken.** The research record was counted separately for the reason above: 12 links outside the retained span, none broken; 8 inside it, all pre-existing. Both link checks were made to fail before being believed — corrupting one `../../decisions/0075` target to `9075` produced exactly one `BROKEN` line and exit 1.

Documentation-only change: no crate, manifest, fixture, or spike was touched, so `make full` is evidence that the tree this change did not enter is still green, and is *not* evidence about the documents — nothing in the gate reads a catalog, resolves a doc link, or validates frontmatter, which is why the link, ordering, and identity checks above were run by hand.
