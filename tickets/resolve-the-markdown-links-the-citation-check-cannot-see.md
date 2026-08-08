---
id: resolve-the-markdown-links-the-citation-check-cannot-see
title: Resolve the markdown links the citation check cannot see
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-resolve-t
lease_expires_at: 1786166309
---
## The catalog rows that navigate the whole corpus are unchecked

`check-citations.sh` validates **pinned** citations — a path with a line number or a quoted anchor. It does not validate **markdown links**, which are how every catalog, index and cross-reference in `docs/` is navigated.

**Coordinator-verified by planting the failure.** Replacing a live ADR link in `docs/decisions/README.md` with `](9999-no-such-adr-…)` leaves `make citations` at **exit 0**, reporting `every pinned citation resolves`, with the citation count unchanged. ~~The dangling link is counted among **`3255 bare path mention(s) carrying no line or anchor`** and never resolved.~~ **FALSE — see the correction below; the target is never parsed at all.**

Found by the worker repairing ADR 0107's catalog rows, which pointed *both* links in its new row at non-existent files and got byte-identical output. Its break test succeeded at showing the gate is blind here rather than at showing it works — and it said so plainly instead of recording a green run as evidence.

## Why this is p1 rather than housekeeping

~~The population is **3,255 references**~~ **(wrong population — the real link count is 5,945; see the correction below)**, and it includes the entry points `AGENTS.md` directs every reader to: *"For broad design work, start with `docs/README.md`; accepted decisions are indexed in `docs/decisions/README.md`."* A dangling row there is a reader sent nowhere from the document that exists to route them.

It is also the exact defect class this repository keeps finding: the checker reports a clean verdict over a population it never examined, and a reader reasonably takes "citations green" to mean the document's references resolve. `AGENTS.md` is explicit that these catalogs have **no automated validator** — this closes the half of that gap that is mechanical.

The worker resolved all **582 local links** with a one-off check that correctly reported the two dangling ones and exited 1, so the property is cheaply checkable. What is missing is that it runs.

## Requirements

- **Resolve every local markdown link** in the populations the script already reads, relative to its own file's directory, and **report the population separately** from pinned citations so neither can collapse into the other. The script already keeps `tickets` and `docs` apart; follow that shape.
- **Floor it.** A run that resolves zero links must fail — the failure this repository has hit repeatedly, and which the script's five existing per-form floors exist to prevent.
- **Do not check external links** (`http`, `mailto`) or anchors into other documents' headings without deciding that separately and saying why.
- **Watch it fail**, separately, for: a dangling relative link, a link to a file outside the tree, and an empty link population. Perturb the subject, quote each message.
- **Expect the first full run to fail** on real defects and **report them rather than weakening the check**. The docs extension found fourteen when it landed; this population is larger.

## Scope note

`check-citations.sh` and `Makefile` are both in the delta rule's gated set, so a change here **cannot carry the gate** — run `make full`.

## Closes when

`make citations` resolves local markdown links across both populations with counts reported separately, fails on an empty link population, has been watched failing in three ways, and every defect the first run surfaces is repaired or filed.

## Outcome

`check-citations.sh` now resolves local markdown links in the same population it already read, reported on their own `links:` block and floored three ways.

**Correction to the Facts above.** The claim that a dangling link "is counted among `3255 bare path mention(s) carrying no line or anchor`" is **false**, and the byte-identical output the ticket itself reports is the proof: `bare_paths` is incremented only inside `classify()`, which is reached only from a closed inline code span, and a link target is not one. Planting `](9999-no-such-adr-planted-by-the-worker.md)` in `docs/decisions/README.md` left every counter unchanged including that one — the target was not miscategorized, it was never parsed. The `3,255` figure is a count of bare paths in **code spans**, unrelated to links; it reads 3246 at `33d59cd0`, and the true link population is **5,945** resolved with 1,359 more classified away. The blindness the ticket describes is real and understated.

Reproduced independently before building: exit 0, `every pinned citation resolves`, all counters identical with and without the planted link.

**Decisions taken, each stated in the script header.** External targets (`scheme://`, `mailto:`, `tel:`) are not resolved — a claim about the network, not this tree, on a gate with no network. Heading anchors are not resolved, path-carrying and same-document alike, because the slug is renderer-produced and no renderer is pinned; both counts are printed and `decide-whether-heading-anchors-in-markdown-links-can-be-resolved` holds the question with a trigger. Links inside `docs/research/*/sources/` are not resolved: 507 targets, 212 reaching the local branch, 92 dangling, all unsatisfiable because one file of each upstream tree was vendored. Resolution is against the git index, not the filesystem — a link is a promise to a reader with a clone, awk cannot read a directory, and this host is case-insensitive.

**Fourteen defects surfaced, none repairable in this scope** (`docs/**` is not `implementation/workspace`). All fourteen verified by reading each site and confirming the intended target exists, so every one is a wrong relative path rather than a missing file. Filed:

- `repair-the-eight-dangling-links-in-the-runtime-route-answer-record` — `research/runtime`, 8
- `repair-the-two-dangling-adr-0075-links-in-adr-0107` — `contracts/decisions`, 2
- `repair-the-two-dangling-adr-links-in-the-conversion-pair-record` — `research/numerics`, 2
- `repair-the-dangling-ticket-link-in-the-frontends-contract` — `contracts/integrations`, 1
- `repair-the-self-referential-link-in-the-concatenate-fusion-record` — `research/indexing`, 1

**`make citations` is red until those five land.** That is the check working, not a defect in it, but it gates `make check` and `make full` for everyone, so those five should be integrated with or before this change.

**Gap found and not closed:** the repository-root documents (`AGENTS.md`, `README.md`, `CLAUDE.md`, 7 links) are in no population, so a dangling link planted in `AGENTS.md` produces no failure. Not fixed here because the requirements bounded the work to the populations already read; filed as `check-the-markdown-links-in-the-repository-root-documents`.

**Cross-checked against an independent resolver** written from scratch over a separately-parsed population. It reported the same 14 plus 6 more, each of which decomposes into a deliberate exclusion: 4 in fenced blocks in `tickets/catalog-the-kani-verification-research-and-spike.md` (catalog rows proposed for `docs/research/`, relative to there), 1 in a comment whose parent ticket is `done`, and 1 code span quoting the planted failure. The oracle also read 72 comment files this script correctly skips — it resolved a comment's parent status before the parent was parsed, which is the ordering the population assembly here is explicitly built to get right.

**Preserved:** the script-owned fixture and all five form floors, the `done`/`closed` terminal skip, and the `superseded`-only document rule with `implementation_status` still never consulted. Pinned-citation counts are unchanged by the link work (948 at the base; 950 after this ticket's own files land).

**Watched failing three ways**, each perturbing the subject and reverted:

- dangling relative link — `no tracked file or directory at docs/decisions/9999-no-such-adr-planted-by-the-worker.md`, 14 failures to 15
- outside the tree — `walks above the repository root` and `site-absolute target, but nothing in this tree is served from a web root`, 14 to 16
- empty link population — matcher blinded to `](`: `EMPTY the tickets/** markdown link population contributed 0 checked link(s)` and `UNEXERCISED local markdown link resolution: parsed 0 times`, naming the fixture link that should have fed it, exit 1

The nested backticks in an earlier draft of that last line put a literal link form into prose and the check failed on it, which is the intended behaviour reaching its own author.
