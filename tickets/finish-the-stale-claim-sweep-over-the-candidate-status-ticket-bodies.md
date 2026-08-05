---
id: finish-the-stale-claim-sweep-over-the-candidate-status-ticket-bodies
title: Finish the stale-claim sweep over the candidate-status ticket bodies
status: done
priority: p3
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, decide-the-expansion-cache-collection-schedule, reach-a-verified-kernel-through-the-structural-families]
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [documentation, work-graph, hygiene]
---
## User-visible outcome

A reader who picks up a `todo`, `deferred`, `blocked`, or `awaiting-decision` ticket can trust the blockers its body records, because every such claim has been checked against the tree rather than inherited from the run that wrote it.

## Why this exists

**Fact — a partial sweep on 2026-08-02 found four refuted blocker claims out of forty-five examined, and two of the four were holding real work parked.** `package-a-multi-entry-bundle-from-one-expansion` was `blocked` with no unmet graph edge, against a contract limit and a grammar limit that had both been lifted. `decide-the-expansion-cache-collection-schedule` was `deferred` against "there is no proc-macro frontend" while `crates/tiler-macros` is one and opens the cache. `reach-a-verified-kernel-through-the-structural-families` scheduled delivery of a one-input limit that no longer exists. `deliver-several-artifact-families-from-one-expansion` scheduled reconciliation of a documentation sentence already withdrawn. Each is corrected in place with its refuting source line.

**Fact — the sweep's own coverage is the reason this ticket exists, and it is countable rather than gestured at.** 740 ticket files exist; 154 are in a candidate status (`todo` 118, `deferred` 32, `blocked` 2, `awaiting-decision` 2); 45 of those matched a blocker/absence phrase set and **all 45 were examined**. The remaining **109 were never read.**

**Inference — the unread 109 are not safe by omission.** They were excluded by a substring filter, and a stale blocker can be worded outside it: "the only survivor", "fails closed today", "no owner", "not yet reachable", "pending". AGENTS.md's own rule applies to the sweep that produced this ticket — a failed search is evidence the search was wrong, not that the thing is absent, until the file has been read.

## Required work

- Read the 109 candidate-status tickets the phrase filter excluded. Read them; do not re-run a widened grep and call the remainder clear. State the count read so coverage stays countable.
- For each blocker, deferral, or absence claim, extract the specific checkable referent — symbol, file, test, constant, or ticket status — and verify it by reading the source file, not by a search returning nothing.
- Correct each stale claim in place: strike or mark the superseded sentence, keep the original rationale rather than deleting it, and cite the refuting file and line plus a one-line reproducing command.
- Move a ticket's status only when **every** recorded blocker is refuted and no unmet graph edge remains. A ticket with one blocker refuted and another live stays parked, with the live one named.
- Repair the two structural defects the partial sweep found and did not fix, both on `package-a-multi-entry-bundle-from-one-expansion`: a `blocked` status with zero unmet dependency edges, and a coordinator correction naming a replacement dependency that was never written into frontmatter and that points at a `closed` ticket. Sweep for both shapes across the board — a `closed` dependency orphans its dependents rather than satisfying them, and `tkt rollup` surfaces orphans.

## Also fix: line-number citations that drifted

Three tickets cite line numbers that no longer resolve, with the argument still correct. Line drift, not a wrong claim — repair the citation, do not reopen the reasoning.

- `add-subgroup-memory-scope-when-collectives-land`: the 2026-08-01 addendum's "corrected" citations are themselves stale. `barrier_call` is at `crates/tiler-metal/src/emit.rs:1601`, the subgroup binding at `:1604`, the rejection at `:1622`.
- `implement-boundary-property-enforcers`: the original 2026-07-27 deferral cites `frontier.rs:557`/`:563` for `bounded_guarantees`/`bounded_requirements`, which now live only inside `crates/tiler-compiler/src/boundary.rs`'s `mod tests` (`:1995`, `:2010`, `:2025`). The ticket's own 2026-07-28 addendum already declares that claim false, so this is superseded rather than misleading — mark it as such.
- `bind-stage-coverage-to-index-refinement-identity`: substance holds, cited lines `builder.rs:891-892`, `:903`, `model.rs:329-330` are stale. Current: `crates/tiler-ir/src/program/builder.rs:1079` and `:1090`, `crates/tiler-ir/src/program/model.rs:549`.

## Explicit non-goals

Do not do the work any corrected ticket unblocks — correcting the record and dispatching against it are separate acts, and conflating them is how a sweep turns into an unscoped implementation run. Do not touch `done` or `closed` tickets except where one is cited as live evidence elsewhere.

## Closes when

Every candidate-status ticket body has been read, each blocker claim in one either verified against a source line or corrected with its refutation, the count read is stated against the count that exists, the two structural defects above are repaired board-wide, and the three drifted citations are updated.

## Sweep executed 2026-08-04, base `c4b4bdb9`

**The population is 181, not the 154 this ticket recorded, and all 181 bodies were read.** `todo` 122, `deferred` 51, `blocked` 4, `awaiting-decision` 4, out of 821 ticket files. Counted twice by independently constructed checks that agree — a frontmatter-block parser (`awk` over the first `---` block) and `grep -lE '^status: (todo|deferred|blocked|awaiting-decision)$'` — both printing 181, with a negative control for a status no ticket carries printing 0 so a passing count is distinguishable from a check that did not run. The corpus is 8,737 lines and was read whole rather than filtered; the 109-unread figure this ticket was filed on is superseded by the larger population.

**The `deferred` half had already been swept and the coverage is countable rather than asserted: 51 of 51 carry a dated `Trigger check log`** (`grep -lc '^## Trigger check log' $(deferred files) | wc -l` → 51), every entry dated 2026-08-04 and most ending in a one-line recheck command. That sweep is the reason the residue below is concentrated in `todo` and `awaiting-decision` bodies rather than spread evenly.

**Structural defect 1 recurred on a different ticket and is repaired.** [`bind-stage-coverage-to-index-refinement-identity`](bind-stage-coverage-to-index-refinement-identity.md) was `blocked` with **zero** unmet dependency edges — all four `done`, including `derive-a-reached-only-executable-coverage-identity`, which its own "Active stop" names as "the sole active prerequisite". Moved to `todo` with the derivation recorded in place; Tom's public-boundary review is preserved as a stop inside the ticket, because that is not a graph edge.

**Structural defect 2 is moot and neither of its shapes survives board-wide.** [`package-a-multi-entry-bundle-from-one-expansion`](package-a-multi-entry-bundle-from-one-expansion.md) is `done`, so the coordinator correction naming an unwritten replacement dependency has no live reader; `done` and `closed` tickets are outside this sweep's non-goals. The board-wide check ran anyway and found **no candidate-status ticket with a dependency on a `closed` or missing ticket** — 0 over 241 dependency edges from candidate tickets. The check is live rather than vacuous: run over *all* statuses it finds two `closed → closed` edges (`define-the-model-execution-state-boundary`, `enforce-unwrapped-prose-in-the-docs-gate`), which are harmless because their sources are terminal. Five candidate tickets carry a `related:` edge to the `closed` `realize-the-strict-contraction-on-metal`; `related` is informational and satisfies nothing, so none is orphaned.

**Corrections made, each in place with the original preserved.** The three drifted citations this ticket names, plus four more found by reading. `bind-stage-coverage-to-index-refinement-identity` (`builder.rs:891-892`→`:1078-1079`, `:903`→`:1090`, `model.rs:329-330`→`:549-550`, encoders `:1108-1109`/`:1303-1304`→`:1535-1536`/`:1734-1735`); `add-subgroup-memory-scope-when-collectives-land` (the 2026-08-01 "corrected" numbers drifted again; `barrier_call` `:1601`, subgroup binding `:1604`, rejection `:1622`, `fence_flag` `:1659`, scopes `model.rs:595`/`:614`, and the tripwire model verified at `tests.rs:853`); `implement-boundary-property-enforcers`; `close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire`, whose own numbers had drifted while telling readers to trust them over the addenda's, and whose claim that the subgroup-scope trigger "is now `compose-the-two-level-subgroup-and-workgroup-reduction`" is refuted by that ticket's own second addendum; `implement-parallel-reduction-strategies` (`frontier.rs:2107`→`:3681`, and its `tickets/…md:23-50` span no longer covers the restart condition, which moved); `resolve-or-retire-the-scalar-lowering-provider-seam`, whose "every caller is a test" argument *is* a line-number comparison and whose numbers had all moved; and three self-refuting reproducing commands in `awaiting-decision` bodies — `accept-adr-0098-inline-delivery-statement`'s dependency grep, which matches its own frontmatter; `accept-the-public-compiler-facade-boundary`'s `ls tickets/accept-*.md`, which now lists 17 files including itself, plus its `region.rs:57` citation, where the vocabulary is actually consumed at `aot.rs:438-480`; and `accept-the-public-route-requirement-answer-boundary`'s `architecture.md:389`, below.

**This ticket's own correction of `implement-boundary-property-enforcers` was wrong and is refuted there.** The "Also fix" section below states that `bounded_guarantees`/`bounded_requirements` "now live **only** inside `crates/tiler-compiler/src/boundary.rs`'s `mod tests`". The `boundary.rs` test helpers are real at `:2010`/`:2025` under `mod tests` at `:1995`, but the production functions the deferral's argument is about are live in `crates/tiler-compiler/src/frontier.rs` at `:885` and `:911`, now taking a `carrier: StorageScalar` and called from four sites (`:668`, `:674`, `:741`, `:760`) rather than two. Reproduce: `grep -rn 'fn bounded_guarantees\|fn bounded_requirements' crates/` prints four declarations across two files. Repointing that ticket at the test module would have moved its argument onto a fixture — two functions sharing a name are not the same function.

**One correction was left for Tom rather than made.** `accept-the-public-route-requirement-answer-boundary` quotes `docs/architecture.md:389` as "`tiler` is the one crate a consumer names…". That string no longer appears in the file at all (`grep -c` → 0); the sentence is at `:424` and already reads "the one crate an **inline-frontend** consumer names", with a following sentence disclaiming `tiler` as the general facade. So the flat overstatement the ticket describes is gone — but the narrowing separates inline-frontend from general-compiler consumers, while ADR 0092 item 2 is about a *dispatching* consumer naming `tiler-metal`, which it does not address. Whether item 2 is thereby discharged is an accepted-decision question and is recorded in that node for the packet rather than decided here. Two sites in `docs/research/runtime/backend-scoped-route-requirement-answers.md` (`:257`, `:317`) still carry the old quotation and citation; both are outside this ticket's scopes.

**What was verified and stands.** `accept-the-public-route-requirement-answer-boundary`'s ripeness check — `grep -rn "RouteRequirement" crates/tiler-build/src/` returns nothing, so the compiler still mints no route requirement (positive control: the string appears in 20 files under `crates/`, so the search works). `accept-the-public-compiler-facade-boundary`'s three `docs/correctness-and-testing.md` spans all still resolve at `:106-111`, `:113`, `:117`, and its structural unripeness holds: `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` is `todo`. The other three `blocked` tickets are correctly blocked — `admit-bf16-into-the-schedule-and-kernel-vocabulary` on a `todo` edge, `admit-lane-typed-values-and-masked-memory-into-the-kernel-ir` on a `todo` edge, and `declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles` on a `deferred` one, which satisfies no dependent.

## Graph maintenance

- File a separate ticket for any real defect this sweep discovers in code; this ticket corrects records and must not absorb an implementation fix. **None was found: every finding was a record defect, and no source file was edited.**
- A ticket this sweep moves out of `deferred` needs its activation triggers checked as actually fired, not merely plausible — a deferral filed dispatchable has been claimed and read before anyone noticed the triggers had not fired. **No ticket was moved out of `deferred`.** The one status move was `blocked` → `todo`, whose condition is a graph fact rather than a trigger.
- **Line-number citations drift faster than anything else in this corpus and repairing them one sweep at a time does not converge.** Three of the seven repaired here were themselves *previous* repairs of the same citation, and one explicitly instructed readers to trust it over an earlier correction. That is a corpus-wide shape rather than a residue of this sweep, and it wants its own decision — cite by symbol and reproducing command instead of by line, or accept the drift and stop repairing it. Filing that decision is out of this ticket's scope and is left as the named remainder.
