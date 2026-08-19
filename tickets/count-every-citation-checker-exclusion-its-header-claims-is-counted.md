---
id: count-every-citation-checker-exclusion-its-header-claims-is-counted
title: Count every citation-checker exclusion its header claims is counted
status: in-progress
priority: p2
dependencies: []
related: [fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it, ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded, stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: [check-citations.sh]
tags: [gates, citations, correctness]
claimed_from: todo
assignee: worker-census
lease_expires_at: 1787169555
---
## User-visible outcome

`check-citations.sh` honours its own stated invariant — that no exclusion is silent. Every branch that drops a citation or a link is counted in the census and named in the header, so a coverage hole cannot hide behind a green run.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit, with the load-bearing finding reproduced by the coordinator at `5f8eccf6` before filing. This is the same defect class the chain that just closed exists to remove: the chain fixed two *wrong resolutions* and one *fail-open skip*, and the audit found that the script's own accounting still has uncounted branches.

**Fact — `qualifies()` drops citations before any counter, and the drop is fail-open.** Anchor `if (!qualifies(path)) return` in `classify()` sits ahead of the form counters (`cit_line++` / `cit_anchor++` / `cit_both++`) and ahead of all partial-path handling, so a span dropped there appears in **no** census line — not `forms`, not `not checked`, not `partial path`, not `ambiguity`. The header's `WHAT COUNTS AS A CITATION` section never lists it as an exclusion. The audit measured 9 spans reaching it today (`` `ADR:85` ``, `` `path "X"` ``, `` `0:100` ``, and similar), which are correctly not citations.

**Fact — that drop currently suppresses a failure the tree can actually produce, verified by the coordinator.** `qualifies()` returns 0 for an extensionless, directory-less name that is not a file at the repository root. There is no `LICENSE` at the root, and `git ls-files | grep -cE '(^|/)LICENSE$'` returns **9**. So a citation of `` `LICENSE:5` `` — a plausible spelling, since the header itself names vendored licences like `docs/research/numerics/sources/jax-v0.11.0/LICENSE:5` — is dropped in silence where the branch below would have raised an **unledgered-ambiguity FAIL** naming 9 candidates. Reproduced at `5f8eccf6`: appending `` A citation of `LICENSE:5` `` to an open ticket left the run at **exit 0** with a byte-identical census line (`1307 pinned citation(s) … 1537 live … file(s)`) and no `FAIL`. The same holds for `NOTICE`, `LICENSE-APACHE`, and `LICENSE-MIT`.

**Fact — the header states a universal the code does not honour.** The header says of its four "deliberately not resolved" link bullets that *each of these is counted in the census* (anchors `Each of these is counted in the` and `census, so every exclusion is a number` — the sentence wraps in the source, so the rendered full sentence greps 0). Three are. The fourth, the empty/whitespace target at anchor `A target with whitespace in it`, code anchor `if (dest == "" || dest ~ /[ \t]/) return`, feeds **no** counter and appears on no census line. The audit measured 6 such targets, all in `docs/research/numerics/sources/onnx-v1.22.0/Operators.md`.

**Fact — that branch also shadows a counted one.** It runs **first** in `link()`, ahead of the external and vendored tests, so those same 6 never reach `link_vendored`. The census line `212 in vendored upstream sources under docs/research/*/sources/` therefore understates its own population by 6, and the header's `212 of them reaching this branch` inherits the undercount.

**Fact — the collision abort is the only fatal that prints to stdout.** Anchor `upstream root(s) recorded in check-citations.sh are also components`. Every other fatal redirects to stderr (`ticketsplease.toml not found`, `docs/ not found`, `no ticket files matched`, `no document files matched docs/**`, `no document files matched *.md at the repository root`, `unknown argument`). Under a stderr-only redirected gate the run would exit 2 with no message, which is exactly what AGENTS.md's "for redirected gates, inspect terminal log lines" exists to catch. `make citations` calls the script directly, so the message is visible today; the inconsistency is the risk, not a live blindness.

**Fact — the `docs/` no-status-facet header enumeration no longer partitions its population.** Anchor `Twenty-four files under`. The header enumerates that branch as exactly two populations (nine Tiler documents, fifteen vendored specifications); the census now prints **1092**. The extra ~1068 are `docs/research/documentation/ticket-audit-2026-08-10/**`, a third population the header does not name. The "Twenty-four" is date-qualified (`on 2026-08-07`) so it is **not false** — the files are correctly checked and only the account is stale — but a reader reconciling 24 against 1092 gets no explanation.

**Fact — the ticket population uses fixed-depth globs the header rejects for `docs/`.** `set -- tickets/*.md` plus `tickets/*.comments/*.md`, while `docs/` uses `find` precisely because "the fixed-depth chain ... drops a whole subtree in silence the day someone nests one level deeper". Verified complete **today**: 1614 tracked ticket markdown files, 1533 + 81 matched, sum exact, and `deferred` is a status rather than a `tickets/deferred/` directory. Latent only — but there is no file-count floor on that population either, so a nested subtree would be invisible.

**Observed, deliberately not a Fact to act on:** the ticket Outcome for `fail-a-partial-path-...` says "all ten of them" while the script header says "all 16 citations", same commit and base. Both are true and neither states its unit — 10 distinct spans over 16 skip lines. This is the same distinct-versus-occurrence conflation the chain elsewhere polices, and it wants one clause in each, not a repair.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict before editing; re-derive each count yourself. Use `grep -o … | wc -l` for occurrences and say which unit you report — `grep -c` counts lines and has already caused a 25% undercount in this repository.
- Count both uncounted branches, name them in the header's exclusion lists, and print them in the census. Decide deliberately whether the `qualifies()` drop should stay a drop at all for the extensionless case, or whether it should fall through to the ambiguity check — **state the reasoning either way**, because that choice is the difference between a documented exclusion and a closed fail-open.
- Fix the ordering so a whitespace/empty target does not shadow `link_vendored`, or state why the shadowing is correct and reconcile the 212 with its true population.
- Send the collision abort to stderr with the other fatals.
- Repair the stale header enumeration by naming the third population, keeping the date-qualified measurement legible rather than rewriting it.
- Give the ticket population a floor, or record the reasoned decision not to add one, on the same standard the `docs/` population is held to.
- Add the one clause each to the two unit-ambiguous sentences.
- **Perturb each newly counted branch separately and quote the failure or the census movement.** A branch you have made countable but never seen move is not demonstrated. For the extensionless case, the coordinator's reproduction above is the before-state; show the after.

## Non-goals

Widening the bare-path exclusion — that is deliberate, documented, counted, and out of scope. Any change to what counts as a *link*. Any source change outside `check-citations.sh`.

## Closes when

Every branch that drops a citation or link is counted and named, the extensionless-name decision is made and reasoned, the vendored count reconciles with its population, the collision abort prints to stderr, each newly counted branch has been perturbed with its output quoted, and `make citations` plus `shellcheck --severity style check-citations.sh` are green.
