---
id: retire-the-spikes-are-ungated-claim-across-the-repository
title: Retire the spikes-are-ungated claim across the repository
status: in-progress
priority: p2
dependencies: []
related: [correct-the-spike-portals-false-claim-that-no-make-target-reaches-spikes, decide-whether-the-citation-checker-should-reach-spike-records]
scopes: [implementation/workspace, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, spikes, gates, misattribution]
claimed_from: todo
assignee: worker-ungated
lease_expires_at: 1787450840
---
## User-visible outcome

No live record claims `spikes/` is reached by nothing, and no record attributes that claim to `AGENTS.md`, which says the opposite — so a reader cannot conclude from any portal, ADR, or ticket that a rotted spike link is theirs alone to catch.

## Why this exists

Filed 2026-08-22 after `worker-portal` repaired four portals and found the conflation is **a class of roughly 40 sites, not the two instances its ticket expected**. It repaired the three further sites inside its own scope and reported the rest rather than widening.

**Fact — the worst variant is misattributed authority.** Tickets attribute the ungated claim to `AGENTS.md`. **`AGENTS.md` makes no such claim anywhere**, and says the opposite at its line 263: `make citations` resolves every local markdown link in *"an open ticket, a live document, **or a retained spike record**"*. Verified by the coordinator at `56119040`: `grep -c "Nothing gates" AGENTS.md` returns **0**, and six tickets referencing `AGENTS.md` carry the ungated claim. A false claim is a repair; a false claim wearing canonical authority is what stops the next reader checking.

**Fact — `Makefile:7` is the upstream quote and is literally true.** It reads *"Spikes deliberately have no target."* Pin-quoted by **16 tracked files** at base `3e6cc78e` — the "**13 files** — verified by the coordinator — of which the delivering lane reports 7 are live, including ADRs 0074 and 0076 and four tickets, the rest dated audit transcripts" this read is corrected in the Fact audit below, which gives the full breakdown. **The sentence is correct and must not be retired**: no `make` target builds or runs a spike, deliberately, because AGENTS.md keeps exploratory dependencies out of the build gate. What is wrong is downstream consumption of it as *"nothing reaches spikes"*.

**Fact — the distinction to restore is threefold, and the delivering lane reproduced each half.** `make citations` **checks** spike markdown links — a broken one fails with `no tracked file or directory at …`. Spike **pinned citations** are **declined by decision**, raising a declined count while the run stays green; its negative control put the same citation in root `README.md` and got exit 2, proving declination rather than a matcher that cannot parse the form. And **nothing builds or runs** a spike: `cargo metadata` reports 16 workspace packages, none under `spikes/`, which appear only in `Cargo.toml`'s `exclude`.

**Reported by the sweep, unverified by the coordinator:** `docs/decisions/0090` says spikes "gate nothing"; roughly six further `docs/research/**` sites; about twenty more tickets; and one site marked **"Verified"** in a verification table.

## Fact audit at base `3e6cc78e`, 2026-08-22, by `worker-ungated`

Every verdict below is from reading the named file at this base and from four reproductions run in this worktree and reverted.

**Verified — the misattribution Fact, including its count.** `grep -c "Nothing gates" AGENTS.md` returns `0` at this base. The file says the opposite, under the "Documentation and durable records" heading, anchor `or a retained spike record`. Exactly **six** ticket files name `AGENTS.md` as the authority for a claim that nothing reaches, gates, or checks `spikes/`, so the coordinator's count is right on the nose: `fix-the-red-compatibility-evidence-mutation-test`, `restore-the-two-path-dependent-spikes-to-a-running-state`, `state-a-numerical-contract-in-the-inline-dispatch-spike`, `refresh-the-forkless-physical-provider-spike-against-the-landed-seam`, `keep-the-ungated-spikes-compiling-against-the-workspace-api`, and `state-the-spike-currency-convention-where-readers-look`. All six are repaired.

**False as to its count — `Makefile:7` is pin-quoted by 16 tracked files, not 13.** `git grep -l "Spikes deliberately have no target" | wc -l` returns `16` at this base. The breakdown, which is what the "7 are live … four tickets" clause got wrong: 2 ADRs (0074, 0076), 1 live research record (`docs/research/apple-targets/numerical-behaviour.md`), 6 dated audit transcripts under `docs/research/documentation/ticket-audit-2026-08-10/reports/`, 1 live spike record (`spikes/extensions/forkless-physical-provider/README.md`), the `Makefile` itself, and **5** tickets — of which only this one is non-terminal, the other four being `done`. **The sentence itself is verified and is not retired**: no `Makefile` recipe names `spikes/` (`grep -n "spike" Makefile` returns only line 7, the comment), and `cargo metadata` lists 16 workspace packages under `crates/` and `prototypes/` with none under `spikes/`.

**Verified — the threefold distinction, all four reproductions run at this base and reverted.** (1) A broken markdown link appended to `spikes/extensions/forkless-physical-provider/README.md` failed `make citations` at **exit 2** with `no tracked file or directory at spikes/extensions/forkless-physical-provider/no-such-file-perturbation.md`. (2) A false pinned citation — the path `crates/tiler-ir/src/lib.rs`, pinned to line 99999, with an anchor string nowhere in the tree — appended to the same record left the gate at **exit 0**, moving only the spike declined count from **61 to 62**. (3) The identical citation appended to the repository-root `README.md` failed at **exit 2** with `line 99999 is past end of file`, which is what makes (2) a declination rather than a form the matcher cannot parse. *(The pin is described here rather than quoted in a single code span on purpose: this ticket is open, so its own pinned citations are checked, and quoting the perturbation intact failed `make citations` at exit 2 when this audit was first written. That is a fourth demonstration that the checker reaches what it claims to, paid for by hand.)* (4) **New, and not in the ticket as filed:** renaming `spikes/extensions/non-exhaustive-visibility/consuming/tests/ui/fail/cross_crate_total_map.stderr` — a fixture ADR 0074 cites as a markdown link — failed at **exit 2**. So a *live document's* link into `spikes/` is gate-checked too, which narrows ADR 0074's custody account and is repaired there. Baseline census at this base: 61 declined, 68 live spike files of 68 read, 601 spike links checked.

**Verified, and the count was low — "about twenty more tickets".** Under the false-shape vocabulary stated below, **34** ticket sites outside the six above carry a claim-shaped construction that is not already inside a dated correction. Reading each: most are `no `make` target reaches it` inside a run-it-by-hand or compile-breakage context, where the conclusion survives on the true reading and the sibling ticket [`correct-the-spike-records-that-still-say-spikes-is-outside-every-gate`](correct-the-spike-records-that-still-say-spikes-is-outside-every-gate.md) has already ruled the analogous `spikes/**` population out of scope for sweeping. **Nine** were genuinely false as stated and are repaired here — the `**Verified**` row in `exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`, the section heading in `correct-the-adr-gate-claims-the-python-gate-deletion-falsified`, the withdrawn checker-scope sentence in `refresh-the-forkless-physical-provider-spike-against-the-landed-seam`, two in `state-the-spike-currency-convention-where-readers-look`, and one each in `write-the-frontier-calibration-s-unwritten-quiet-host-gate`, `record-the-landed-physical-provider-seam-in-adrs-0078-and-0090`, `accept-the-loader-variant-eligibility-vocabulary`, and `name-the-elementary-identity-rewrite-dimension`. The rest are imprecise-but-sound and are left, with the boundary stated rather than silently drawn: where a site says `no `make` target reaches it` inside a sentence whose conclusion is about compiling or running, the conclusion survives on the true reading and the wording is left as the sibling ticket ruled for the analogous `spikes/**` population.

**Verified — the "Verified" table row.** It is `tickets/exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md`, row `| `make full` does not reach `spikes/` | **Verified** | `Makefile`: `Spikes deliberately have no target.` |`. It is false: `full: check doc` and `check: citations fmt build lint test`, so `make full` runs `make citations`. Repaired, and the row's evidence retained for the narrower claim it does support.

**Verified — `docs/decisions/0090` says spikes "gate nothing"**, at the 2026-08-08 correction, anchor `so this is a stale artifact rather than a broken gate`. Repaired as implementation-status prose; no decision, elimination, or Measurement moved.

**Verified — six further live `docs/research/**` sites, plus six dated audit transcripts.** All are outside this lane's scopes and are reported to their owners rather than edited.

### Search vocabulary for the census, and its unit

The census unit is **matching lines**, counted with a Python `re.finditer` over `git ls-files '*.md'` (**3,087** files) rather than with `grep -c`, which counts lines and would undercount multiple constructions on one line. Candidate lines are those mentioning `spike` case-insensitively, intersected with a claim-shaped construction from ten named buckets: `ungated` (`un-?gated|un-?checked|un-?governed|unreached|un-?reachable|un-?verified|un-?tested`), `nothingV` (`nothing` within five tokens of a gate verb), `Vnothing` (gate verb directly before `nothing`), `noN` (`no` within four tokens of a gate noun — gate, target, check, checker, CI, build, test, command, recipe, population, suite, automation, members), `notV` (`not|never|neither|nor` within four tokens of a gated/checked/reached/built/run/compiled participle), `outside`, `excluded from`, `absent`, `no automated|no CI`, plus `manualonly` and `silently` as discovery aids for true statements. That yields **557** claim-shaped lines in **294** files; restricting to the eight buckets that can actually express "spikes are unreached" leaves **449** lines in **258** files. The narrower **false-shape** pattern — a claim that no gate or target *reaches, touches, checks, covers, sees, or reads* `spikes/`, plus the fixed phrases `gates nothing`, `outside every/the gate`, `unchecked by every gate`, `is/are ungated`, `cannot see this directory`, `every gate stays green`, `no gate compiles` — is what the repair set was drawn from.

**Why that set is complete, and the one way it failed first.** It is complete because it is built from the *subject* side rather than the claim side: every line mentioning a spike is a candidate, and the buckets only filter, so a novel phrasing is caught by `noN`/`notV`/`nothingV` unless it names no negation at all. The first version of this census was **not** complete, and the failure is worth recording because it is invisible: the pattern was passed through a double-quoted shell string containing a backtick, so the shell consumed it as command substitution and silently corrupted the regex. That version missed `tickets/fix-the-red-compatibility-evidence-mutation-test.md` and `tickets/restore-the-two-path-dependent-spikes-to-a-running-state.md` — two of the six misattributions this ticket exists for. It was caught only because those two files had been found independently through an `AGENTS.md` co-mention search. **A census cross-checked by a second, differently-shaped search is what caught it; the counts alone looked plausible.** The final census is run from a Python file with no shell quoting.

## Required work

- Re-audit every Fact at your base with a per-Fact verdict, and **re-derive the site census yourself** — the ~37 out-of-scope sites are agent-reported and the coordinator verified only the misattribution, the `Makefile` quote, and its 13 pin-quotes. **Say which spellings you searched for and why that set is complete**; a census is only as complete as its search vocabulary, which is how a sibling ticket closed green over live sites this week.
- Repair the misattributions **first**. They are the highest-severity subset because they borrow authority the source does not grant.
- Leave `Makefile:7` alone and repair its consumers. If a consumer's conclusion survives on the true reading, re-evidence it rather than withdrawing it.
- **Preserve retired wording in dated corrections**; grep counts cannot shrink, and expecting them to is a false progress signal.
- An **accepted ADR** carrying the claim is not a free edit: repair the implementation-status prose, never the decision, and say which you touched.

## A hazard this lane surfaced that applies to whoever takes it

The delivering lane's `AGENTS.md` copy **in its session context was stale** — it lacked the "or a retained spike record" clause the on-disk file carries. **Read the file, not your context.** That is exactly the failure mode this ticket exists to clean up, one level up.

## Non-goals

Changing the checker's declared scope, which is an accepted decision; adding any build or test target over `spikes/`, which AGENTS.md forbids; and re-repairing the four portals already corrected.

## Closes when

No live record claims spikes are unreached, no record attributes that claim to `AGENTS.md`, `Makefile:7` still stands with its consumers reading it correctly, every correction preserves what it replaced, and the census is re-derived with its search vocabulary stated.
