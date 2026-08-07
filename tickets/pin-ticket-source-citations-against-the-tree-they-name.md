---
id: pin-ticket-source-citations-against-the-tree-they-name
title: Pin ticket source citations against the tree they name
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Why this exists

Every ticket audited on 2026-08-07 carried at least one false Fact. The most damaging class was **line citations that had drifted** — `admit-a-fusion-role-for-the-sub-tensor-selection-slice` had *every* citation stale by 200–400 lines, so a worker following them would have landed in unrelated code and edited the wrong thing. Others named a test that no longer exists, or a count that had since changed.

`AGENTS.md` now carries the reading obligation that actually controls this: **a ticket's stated Facts are stale until re-read at your own base**, and a worker's first deliverable is a per-Fact verdict. This ticket adds the cheap mechanical layer *underneath* that — not in place of it.

## The boundary, stated first because it is the point

**This check cannot verify a claim, and must not be presented as doing so.** A citation can resolve perfectly and still support a statement the code no longer makes — which is exactly what happened to the reassociation-obligation claim, where the file and symbol were right and the described behaviour was wrong. What a checker can catch is the cheapest subset: a path that does not exist, a line past end-of-file, a quoted anchor that appears nowhere.

So the deliverable is a **loud floor**, and its documentation must say plainly that a green result means "the citations point somewhere", never "the ticket's Facts are true".

## What to build

A check over `tickets/**` that, for each source citation:

- resolves the path against the working tree, failing when it does not exist;
- where a line number is given, fails if the file has fewer lines;
- where a **quoted anchor** is given, fails if the quoted text appears nowhere in the named file — this is the half with real signal, and it is why `AGENTS.md` now asks for anchors rather than bare line numbers.

Requirements that are not optional, drawn from checks in this repository that could not fail:

- **Name and count the population.** Report how many tickets and how many citations were examined. A run that parses zero citations and reports no problems must **fail**, not pass.
- **Be multi-line aware.** Citations wrap across lines in ticket prose, and a line-oriented matcher will silently miss them — the same defect that made a `grep` for `allow(unsafe_code` return one doc comment and none of four real attributes.
- **Watch it fail, per failure mode.** Plant a bad path, a past-EOF line, and a missing anchor, separately, and quote each failure. Then plant a citation that resolves but is *semantically* wrong and show the check **passes** — documenting the boundary above by demonstration rather than assertion.
- Exclude closed and superseded tickets, or dated-correction blocks that deliberately quote retired citations; decide which and say why. A condition that demands the repository forget what it corrected is unsatisfiable — that mistake has already been made here once.

## Where it runs

Decide and justify: `tkt lint`-adjacent, a `make` target, or a test. Note that `tickets/**` is not in the delta rule's gated set, so a ticket-only change currently carries the previous green gate — a check that only runs under `make full` would not see most ticket edits. That is an argument for the lighter gate, not an afterthought.

## Non-goals

Verifying that a citation supports its claim. Rewriting existing citations in bulk — repair them as tickets are dispatched, under the reading obligation. Editing `AGENTS.md`, which already carries the rule.

## Outcome

**Fact — the check exists and the tree passes it.** `check-citations.sh` resolves every pinned citation in an open ticket against the working tree, and `make citations` runs it in 0.46 s. On this branch it reports **265 pinned citations across 236 open ticket and comment files**, out of 1248 files read, 1012 of them skipped as terminal.

**Fact — the boundary is demonstrated, not asserted, and the fixture is live.** The paragraph below is deliberately false and the check passes it, because the citation resolves perfectly:

> **Deliberately false, retained as a fixture — do not "fix" it.** `make check` runs the citation check **last**, after the test target (`Makefile "check: citations fmt build lint test"`).

The anchor is the literal text of that line, so the path resolves, the anchor is found, and the check is silent. The claim is nevertheless wrong: `citations` is the *first* prerequisite, so it runs before `fmt`, not after `test`. That is the whole point — green means the citations point somewhere, never that the ticket is true. Anyone repairing this fixture removes the only standing demonstration of what the check cannot do; the wrong sentence is the evidence.

**Fact — what counts as a citation is narrower than "a path", and deliberately so.** A citation is a code span carrying a path *plus a pin*: `path:LINE`, `path:START-END`, `path "anchor"`, or `path:LINE "anchor"`. A bare path with no pin is not checked, and 383 such mentions were skipped and counted. Two legitimate populations live there: files a ticket asks someone to create, and files whose deletion a ticket is recording — `scripts/check_workspace.py`, deleted at `e197176f` when the Python gate became the Makefile, is still named in ten tickets that are accurately describing history. Demanding those resolve is the unsatisfiable condition this ticket was told to avoid.

**Fact — the retired-citation convention is satisfiable, and was exercised while writing this.** A dated correction quotes a retired line number in prose, or as the bare `:789-810` suffix the house style already uses, rather than pinning it to a path. The first draft of the correction in `audit-dead-code-admissions-after-public-boundary-promotions` re-pinned the dead path and the check failed it; rewriting the line number into prose satisfied the check without weakening it or losing the record.

**Fact — terminal tickets are skipped, and the states are read rather than remembered.** `done` and `closed` come from the `category = "terminal"` entries in `ticketsplease.toml`, not a hardcoded pair. Their citations describe a tree at merge time and rot by design. Comment files carry no status of their own and inherit their parent ticket's, because a comment is part of the ticket a worker is told to read in full.

**Fact — one genuine stale citation was found on the real tree, and it was a live `todo`.** `audit-dead-code-admissions-after-public-boundary-promotions` cited `prototypes/serial-sum-compile/src/target.rs` at line 29 as carrying a file-scope `dead_code` admission. That file was added at `8dbffb93` and deleted at `2d2a7bd7`. Re-reading also refuted two counts in the same paragraph: the ticket claimed twelve production admissions and its own 2026-08-04 log entry claimed eight, while the tree has **seven** — `realization.rs` lost its file-scope admission at `8bfcd432`. All three were repaired in place with a dated correction.

**Measurement — five perturbations of the subject, each run separately, each quoted.** A nonexistent path: `no file in the tree is or ends with crates/tiler-compiler/src/no-such-file.rs`. A past-EOF line: `line 9999 is past end of file: Makefile has 66 lines`. An absent anchor: `anchor occurs nowhere in Makefile`. A code span wrapped across two lines of ticket prose: still parsed and still failed, where `grep` for the same span on one line returns `0`. A citation-free corpus: `parsed ZERO citations` and exit 1, so a matcher that stops reaching its subject cannot look clean.

**Fact — the matcher is multi-line aware on both sides.** Spans are assembled across line breaks as they close, so a citation wrapping in ticket prose is not lost; 67 non-fence lines under `tickets/` currently end mid-span. An anchor that fails a literal match is retried with whitespace collapsed, so it still matches a construct that wraps in the *source* — `crates/tiler-conformance/src/device_buffer.rs "#[allow(unsafe_code,"` resolves although `grep -rn 'allow(unsafe_code' --include='*.rs' crates prototypes` returns exactly one hit, a doc comment, and misses all four real attributes.

**Fact — it runs in the light gate, which is the placement the ticket argued for.** `citations` is the first prerequisite of `check` (`Makefile "check: citations fmt build lint test"`). `tickets/` is not in the delta rule's gated set, so a ticket-only change carries the previous green gate and reruns `tkt lint` alone; a check reachable only from `full` would never see most ticket edits. `make full` also shellchecks the script (`Makefile "shellcheck --severity style deps.sh check-citations.sh"`).

**Known gaps, stated rather than hidden.** Seven pinned citations use a short form whose basename is ambiguous (`lib.rs:34` has 42 candidates) and are counted, not resolved — guessing a path would invent a failure or hide one. Three cite dependency sources pinned by version (`objc2-metal-0.3.2/src/generated/MTLDevice.rs:238`) which no working-tree check can decide. Markdown link targets are not checked at all; AGENTS.md states that documentation links have no automated validator, and changing that is a separate decision.

**Not done, and it needs Tom.** AGENTS.md's delta-rule paragraph tells a ticket-only change to "rerun `tkt lint`" and does not name `make citations`. Editing AGENTS.md is this ticket's explicit non-goal, so the sentence is unchanged and the coupling currently lives only in the `Makefile` comment. That paragraph should name the target.

## Outcome — done, 2026-08-07

Landed at merge **`66db2178`** (worker commit `7e3a7367`). `check-citations.sh` at the repository root beside `deps.sh`, wired as `make citations` and as `make check`'s first prerequisite. `make full` exit 0 on the merged tree; 3,098 workspace tests, 1,080 release tests, shellcheck clean over both scripts. Runs in **0.46 s**.

**The tree passes**: 269 pinned citations across 236 open ticket and comment files, 1,248 files read, 1,012 skipped as terminal.

### The boundary is demonstrated by a permanent fixture, not asserted

This ticket now carries a citation that **resolves and is false**: it claims `make check` runs the citation check *last, after the test target*, anchored on `` `Makefile "check: citations fmt build lint test"` ``. The anchor is verbatim; the claim is wrong — `citations` is prerequisite **#1**, and `make -n check` prints it first. **The check passes it.** Both the script header and the ticket say not to "fix" the sentence: it is the only standing evidence of what the check cannot do, and repairing it would delete the demonstration.

### Design decisions worth knowing

**A citation is a path *plus a pin*.** Bare paths go unchecked — 394 of them — because two legitimate populations live there: files a ticket asks someone to create, and files whose deletion a ticket records. `scripts/check_workspace.py`, deleted at `e197176f`, is still named accurately in ten tickets. Checking bare paths would have produced exactly the unsatisfiable condition this ticket was written to avoid.

**Retired citations in dated corrections are handled by convention, not exemption** — write the retired line in prose, or as the bare `:789-810` suffix the house style already uses. Exercised the hard way: the worker's own first draft re-pinned a dead path, the check failed it, and rewriting the number into prose satisfied it without weakening anything.

**Terminal status is read from `ticketsplease.toml`'s `category = "terminal"`** rather than hardcoding `done`/`closed`, and comment files inherit their parent's status.

### Six failure modes, each perturbed separately

Bad path; line past EOF; absent anchor; descending range; a citation **wrapped across two lines of prose** (parsed and failed, where a `grep` for the same span returns 0); and a zero-citation corpus, which reports `parsed ZERO citations…the matcher has stopped reaching its subject` and exits 1.

### It found a live defect on its first real run

`audit-dead-code-admissions-after-public-boundary-promotions` cited a file **deleted at `2d2a7bd7`**. Re-reading refuted two further Facts in the same paragraph: it claimed twelve admissions, its own log claimed eight, the tree has **seven**. Its reproduce command was itself line-oriented — `grep -B2 dead_code | grep '#!\[allow('` only sees a `dead_code` within two lines of its opening attribute — so its count was right by luck.

### Two coordinator errors it corrected

The brief's "**200–400 lines**" drift range was wrong in both directions: four citations moved forward by **+71 to +371**, and a fifth moved **backwards by 171**. `AGENTS.md` now states that drift is not one-directional, because a reader assuming citations only slide downward still lands in the wrong place.

And `AGENTS.md`'s delta-rule paragraph named only `tkt lint` for a carried gate. Since `tickets/` is not in the gated set, the citation check would have been skipped by **exactly the deltas it exists to police**. It now names `make citations` with that reasoning stated.

### Left as the worker placed it

`check-citations.sh` sits at the repository root rather than under `scripts/`, deliberately: `e197176f` removed the Python gate and reintroducing that directory shape was avoided. Its `implementation/workspace` glob was added in the same commit, which is why `tkt guard` warned against the base config — resolved on merge.
