---
id: close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward
title: Close the four licence readings Tom supplied and admit Graefe and Ward
status: in-progress
priority: p2
dependencies: []
related: [vendor-the-tuning-loop-primary-sources-after-reading-each-licence, acquire-the-three-unreachable-adaptive-execution-sources]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-licence
lease_expires_at: 1786143861
---
## What happened

Four documents this repository could not fetch were **supplied by Tom on 2026-08-07** from a normal browser session, after the standing rule stopped an agent from working around the hosts that refused it. They sit at `/Users/tsanterre/Downloads/`:

| File | Row | Why the host refused |
| --- | --- | --- |
| `p1228-reddy.pdf` | `plan-diagrams-vldb-2005` | `www.vldb.org` HTTP 403 to `curl`, host-wide |
| `P103.pdf` | `pqo-vldb-1992` | same host, same 403 |
| `qt5h71f534.pdf` | `halide-autoscheduler-2019` | `escholarship.org` HTTP 202, zero-byte body |
| `66926.66960.pdf` | `graefe-ward-sigmod-1989` | `dl.acm.org` Cloudflare bot wall |

**Digests verified by the coordinator before any terms were read**, which is the discipline the licence-reading pass established. The three with recorded digests reproduced **exactly**: `401c6f66…b7b8` / 499,633 bytes, `c8713911…03da` / 1,293,795, `e4dd35a0…79c0` / 3,845,934. So these are the same byte streams the design record's claims were checked against, not substitutes. Graefe & Ward has no recorded digest because it had never been retrieved; it is `c128847cda52926d78c7a176dfafc2a38ec498664701f1b65bcd7ab18a57e32e`, 1,146,699 bytes.

## The licence readings, already performed — verify them, do not redo them blind

The coordinator read each rights notice. **Re-read each yourself and report agreement or disagreement**; these are the coordinator's readings and `AGENTS.md` ranks those as needing confirmation.

- **`plan-diagrams-vldb-2005`**, page 1: "Permission to copy without fee all or part of this material is granted provided that the copies are not made or distributed for **direct commercial advantage**, the VLDB copyright notice and the title of the publication and its date appear… To copy otherwise, or to republish, requires a fee and/or special permission from the Endowment." **The record predicted exactly this condition would decide the row, and it does.** Verdict: **not vendored**, fail-closed, on the same non-commercial condition that resolved two rewrite-search rows.
- **`pqo-vldb-1992`**, page 1: the same VLDB notice verbatim (the extraction is OCR-damaged but the operative clause is unambiguous). Verdict: **not vendored**, same ground.
- **`halide-autoscheduler-2019`**: page 1 is an eScholarship cover sheet with no notice; the ACM block sits on the article's first page — "granted without fee provided that copies are not made or distributed for profit or **commercial advantage** … To copy otherwise, or republish, **to post on servers** or to redistribute to lists, requires prior specific permission and/or a fee", with "© 2019 Copyright held by the owner/author(s). Publication rights licensed to ACM." **Checking a file into this repository is posting it on a server**, which the notice names explicitly. Verdict: **not vendored**.
- **`graefe-ward-sigmod-1989`**, page 1: "Permission to copy without fee all or part of this material is granted provided that the copies are not made or distributed for **direct commercial advantage**, the ACM copyright notice and the title of the publication and its date appear… To copy otherwise, or to republish, requires a fee and/or specific permission. © 1989 ACM". Verdict: **not vendored**.

**All four fail closed. Zero bytes are to be checked in**, which keeps the record's uniform metadata-only classification intact — now on thirteen readings rather than nine, with the classification finally resting on evidence for every reachable row.

## The second half, which is the more interesting one

`graefe-ward-sigmod-1989` **has been retrieved and can now be read.** Move it out of `## Awaiting retrieval` into the reachable class with its digest, its retrieval provenance (supplied by Tom, not fetched by this host — say so), and its licence verdict.

Then **read it** and settle two characterisations the record currently carries at second hand, both explicitly flagged as "not a reading of Graefe and Ward":

- Cole & Graefe 1994 say it introduced choose-plan but "left two all-important questions unanswered, namely how to choose which optimization decisions to delay and how to engineer a query optimizer that efficiently creates dynamic plans for arbitrarily complex queries at compile-time", and attribute to it a minimal-decision-procedure proposal they reject as unrealistic.
- Markl et al. say that because optimizer cost functions "are also not smooth, not even always continuous", "simple binary-search techniques as in [GW89] will not work."

Report whether each holds against the primary. The one thing that would change a conclusion: **if its minimal decision procedure is cheaper or more general than its two successors describe**, it becomes a candidate for resolving a deferred cost row at a later availability phase *without carrying alternatives* — which is the only thing that would make the choose-plan line competitive with the `AvailabilityPhase` ladder rather than a superset of it. The record says nothing currently suggests that. Confirm or refute.

**Do not alter the design record's conclusions** if this turns into a substantive redesign — report it and let it be ticketed separately.

## Closes when

Four licence verdicts recorded with their operative clauses quoted; the awaiting-retrieval class emptied and **kept rather than deleted**, per its own stated reason that an emptied channel which stops being counted is one a re-opened request slips back into unnoticed; Graefe & Ward's two secondary characterisations each confirmed or refuted against the primary; and the vendoring ticket's population figures updated to match.
