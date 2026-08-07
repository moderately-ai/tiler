---
id: close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward
title: Close the four licence readings Tom supplied and admit Graefe and Ward
status: done
priority: p2
dependencies: []
related: [vendor-the-tuning-loop-primary-sources-after-reading-each-licence, acquire-the-three-unreachable-adaptive-execution-sources]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: []
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

## Outcome — done, 2026-08-07

Landed at merge `15342bdc`'s ancestor (worker commit `cb79e543`). 9 files, +554/−39, **zero binary blobs**. Delta is `docs/` and `tickets/` only, carrying the green gate from `56046f77`.

**Digests re-verified before any terms were read.** The three with recorded hashes reproduced exactly; Graefe & Ward is a first acquisition at `c128847c…`. **Coordinator-confirmed on the merged tree**: `verify-sources.sh` reports `14 records verified (0 vendored, 4 local-only, 10 metadata-only, 0 pending-acquisition)` with `4 present and digest-verified, 0 absent`. The gitignored-bytes arrangement works end to end.

### The audit corrected the coordinator twice on precision

- **`pqo-vldb-1992`** — I called the notice "the same VLDB notice **verbatim**". The *notice* is; the *extraction* is not, being OCR-damaged in three tokens (`gmnttdpmvicfed`, `Very Lurge`, a broken `no- tice`). Operative words undamaged.
- **`graefe-ward-sigmod-1989`** — the paper reads "Association **for** Computing Machinery" where I wrote "of", and its copyright line is OCR-damaged, extracting a bare `0` where `©` belongs. **My "© 1989 ACM" silently normalized damaged text**, which is the thing the record's own convention forbids; it is now recorded as a measurement rather than a quotation.
- **A false Fact in this ticket**: "thirteen readings rather than nine" — both numbers wrong. The record had **ten** read and three unread, and Graefe & Ward sat outside the thirteen-row reachable population. Correct is **fourteen, up from ten**.

All four verdicts otherwise agree: **not vendored**, fail-closed, on the non-commercial condition each notice carries.

### Both second-hand characterisations turned out partly wrong

- **"Left two all-important questions unanswered" — confirmed, and understated.** §7 lists both as open, defers the plan-selection criterion by name, and the experiment never exercised the decision procedure at all: *"we 'forced' the choose-plan operator to use a plan of our choice."*
- **The minimal-decision-procedure attribution — right about the goal, wrong about the mechanism.** "Inverse" and cognates appear **zero times** in the 1989 paper; its method is repeated *forward* evaluation to find the break-even point. So 1994's rejection of "building inverses for all cost functions" refuses a stronger requirement than its predecessor ever made.
- **Markl et al.'s "binary search will not work" — citation real, premise wrong.** The 1989 paper handles non-smoothness *before* searching, splitting the range at discontinuities. The real gap is that it never says how smoothness is determined — which is not what the quoted sentence names.

**The finding that would have mattered is refuted.** The 1989 procedure is cheaper at start-up, but does **not** avoid carrying alternatives — the access module must contain support functions for all possible plans — and carries **no optimality guarantee**, which is 1994's contribution. Same retention cost without the justification, so the `AvailabilityPhase` conclusion is **strengthened**, not weakened.

### The §4 finding: an under-citation, not an error

Reddy & Haritsa §4 tests the three assumptions the PQO literature rests on — convexity, uniqueness, homogeneity — and reports "**none of the three assumptions hold true, even approximately**". That is the missing empirical half of the design record's Rule 2: if the winning region is non-convex and non-contiguous, interpolating between two measured winners is unsound *in general* rather than merely unproven. The record cited this paper only for the 68→7 geometry. It reaches Rule 2 independently and changes nothing, and is bounded to three commercial optimizers on TPC-H with the authors self-describing as "perforce speculative" — **nothing is established about Tiler's cost space**.

### The manifest decision reversed itself, for a new reason

`expected-sources.tsv` was built after all, at 14 rows, reversing the earlier no-manifest call. The deciding reason did not exist then: **a gitignored `local/` is invisible to `git status`, so it is exactly where a stray licence-restricted PDF hides.** `verify-sources.sh` was made to fail six ways — stray file, mutated bytes, truncated manifest, unknown classification, path escaping `local/`, missing row — and its summary always prints present-vs-absent counts so absent bytes are legitimate but never silent. `local-only` is defined as a **preservation class and explicitly not a licence verdict**; all fourteen rows remain non-redistributable.

The four catalog rows this owes `docs/research/README.md` are filed separately.
