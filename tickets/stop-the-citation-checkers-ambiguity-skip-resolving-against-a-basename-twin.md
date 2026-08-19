---
id: stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin
title: Stop the citation checker's ambiguity skip resolving against a basename twin
status: done
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue, split-the-index-refinement-module-into-cohesive-submodules]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: [check-citations.sh]
tags: [gates, citations, correctness]
---
## User-visible outcome

Deleting one of two files sharing a basename can no longer convert a skipped-as-ambiguous partial-path citation into one that silently resolves green against the surviving twin: the checker either keeps a record of retired ambiguity or resolves partial paths in a way a deletion cannot repoint.

## Why this exists — filed 2026-08-19 from the refinement split's delivery

**Fact (verified by the split lane at `a2e98b27`).** `check-citations.sh` skips partial paths matched by more than one file. The refinement split deleted `crates/tiler-ir/src/index/refinement.rs`, making the bare suffix `refinement.rs` — previously ambiguous with `crates/tiler-ir/src/semantic/accuracy/refinement.rs` and therefore skipped — a unique suffix. One live snapshot citation began resolving **green against the wrong file**; four siblings failed only because their line numbers exceeded the surviving file's 659 lines. A shorter survivor would have made all five silently green. The population at risk is every skipped-ambiguous partial path whose basename family shrinks to one.

**Fact audit — 2026-08-19 at base `bda38064`, before any edit.** Re-read at this base, not carried from the text above.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| `crates/tiler-ir/src/index/refinement.rs` was deleted at `a2e98b27` | **verified** | `git log --oneline --diff-filter=D -1 -- crates/tiler-ir/src/index/refinement.rs` returns `a2e98b27 Split the index refinement module into cohesive submodules` |
| The surviving twin is `crates/tiler-ir/src/semantic/accuracy/refinement.rs` and is 659 lines | **verified** | `git ls-files \| grep '\(^\|/\)refinement\.rs$'` returns that one path; `wc -l` returns 659 |
| The checker skips partial paths matched by more than one file | **verified** | the `suffix_count[path] > 1` branch of `classify()`, comment `guessing a path would invent a failure or hide one` |
| "287 reported by the checker at recent runs" | **false at this base** | `./check-citations.sh` reports `330 skipped as ambiguous` after the fixture addition below and reported **329** before it. The number is corpus-dependent and drifts every time a ticket opens or closes, so it is removed from the sentence above rather than replaced with today's value; the measurements that matter are pinned with their date in the Outcome |
| The five affected snapshot citations are still live | **false at this base** | no live citation carries the bare suffix `refinement.rs`. All seven tickets that still name it — `restate-the-single-region-realization-docs-after-the-sequence-widening`, `resolve-or-retire-the-scalar-lowering-provider-seam`, `realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity`, `execute-the-doc-drift-sweep-the-audit-enumerated`, `name-the-index-receipt-final-stage-accessors-for-what-they-return`, `widen-the-staged-realization-law-to-the-registered-elementary-families`, `stop-recomputing-pure-derivations-in-the-codec` — are `status: done` and therefore skipped as terminal. The demonstration had to be reproduced on a different twin pair; see the Outcome |

## Required work

- Read `check-citations.sh` in full, including its stated rationale for which link shapes it declines to resolve.
- Choose and implement the fail-safe: candidates include refusing to resolve a bare-basename partial path at all (require at least one directory component), or failing loudly when a partial path resolves uniquely but the citing document predates the uniqueness — derive the actual design from the script's existing structure rather than this list.
- Perturb the subject, not the assertion: reproduce the twin scenario in a fixture (two files sharing a basename, a citation on the suffix, delete one) and quote the checker's failure text before and after the fix. Where the script's built-in fixture population supports it, extend that fixture rather than a one-off.
- Census the currently-skipped ambiguous population for suffixes one deletion away from uniqueness; state the count.
- `make citations` must stay green on the current tree, or every newly failing citation must be repaired in the same change with the read-the-claim discipline.

## Closes when

The twin scenario has a quoted failing check, the fix's perturbation evidence is recorded, the near-unique census is stated, and `make citations` is green.

## Outcome — 2026-08-19, worker-citation-checker

### What was built: a retired-ambiguity ledger inside the checker

A suffix matching exactly one tracked file is already stable under deletion — delete the file it names and the count falls to zero, which is the existing loud `no file in the tree is or ends with` failure, and deleting anything else cannot change what it names. The unstable case is the *ambiguous* one, and it is unstable in the wrong direction: an ambiguous suffix is skipped, so the day its family shrinks to one the citation stops being skipped and starts resolving against whichever twin survived. Nothing in a working tree separates "this suffix has always named one file" from "this suffix named two until last week", because the deleted twin leaves no trace in the index.

So the memory is written down. `check-citations.sh` now carries a ledger of 41 suffixes, seeded from the 40 observed ambiguous under a live citation on this tree plus `refinement.rs`, whose collapse this ticket records and which no live citation now carries. Two rules, both failures rather than skips:

- a ledgered suffix that now matches exactly one file **fails**, naming the survivor it would otherwise have resolved against;
- a cited suffix matching more than one file and **absent** from the ledger **fails**, because it turned ambiguous after the ledger was written and the citation has silently stopped being checked.

The second rule is what keeps the ledger from being a snapshot that rots, and it closes a second silence that was already there: before this change, adding a file that collided with a cited suffix dropped that citation out of the checked population with no signal but a moved census number.

### Alternatives rejected, with the measurement that rejected each

- **Require at least one directory component.** Six of the 22 near-unique suffixes already carry one — `program/verify.rs`, `program/model.rs`, `program/handles.rs`, `program/mod.rs`, `builder/proof.rs`, `bf16/tests.rs` — so it leaves the exposed population only partly covered, while failing 420 of the 470 partial-path citations that resolve correctly today. It fails the wrong ones and misses six of the right ones. The `program/verify.rs` perturbation below is the direct demonstration.
- **Fail on ambiguity outright.** Strictly the strongest rule, and the one the ledger converges on as citations are lengthened. It cannot land here: 329 live citations would fail, 323 of them in `docs/` across 120 files, and this branch declares `implementation/workspace` and `project/tickets` only. Repairing them means reading 323 claims, which is a separate ticket, not an implementation detail of this one.
- **Derive the memory from `git log --diff-filter=D`.** Needs no maintenance and directly models "a twin was deleted", but cannot tell a deletion from a rename or decide whether two paths ever coexisted, so a module move poisons a suffix forever. Measured over the 65 ever-deleted paths here: one live citation would newly fail, `payload.rs:289` in `docs/research/extensions/backend-provider-composition.md`, which reading confirms correctly names `check_provenance` in `crates/tiler-artifact/src/program/codec/payload.rs` — one invented failure, no caught defect. Split into [`ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded`](ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded.md), where the deleted set is a review list rather than the rule.

### The near-unique census

**Measurement — 2026-08-19 at `bda38064`.** 329 citations are skipped as ambiguous over 40 distinct suffixes; 470 resolve by unique suffix over 99. **22 of the 40 match exactly two tracked files and are therefore one deletion from resolving against a survivor**, carrying **117 citations** between them.

Derived twice and agreed. From the checker — both commands rerun clean against the landed script, whose verbose skip line now ends `, on the ledger)`:

```sh
./check-citations.sh --verbose | grep -c 'ambiguous, 2 candidates,'      # 117
./check-citations.sh --verbose |
  sed -n 's/^SKIP  [^:]*: `\(.*\)` (ambiguous.*/\1/p' |
  sed -E 's/[:[:space:]].*$//' | LC_ALL=C sort -u | wc -l                # 40
```

Independently from `git ls-files`, counting the tracked files ending at a `/` boundary with each of those 40 suffixes: 22 have exactly two. The two derivations name the same 22.

The 117 are not equally dangerous, and the split matters more than the count: replaying each of the two possible deletions against each citation, **77 of the 117 would resolve silently green** against the survivor, and only 40 would fail on a line number past its end. All 40 of those are `request.rs`, whose twins are 181 and 105 lines. Every other near-unique suffix would pass in silence.

### Perturbations — the subject broken four ways, each failure quoted

Each perturbation was applied to the real tree or to the real script and then reverted; `git status --porcelain` showed only `M check-citations.sh` afterwards. The pre-fix script is `git show HEAD:check-citations.sh` at this base.

**1. The twin scenario, bare basename.** `git rm crates/tiler-conformance/src/applicability.rs`, leaving `crates/tiler-metal/src/applicability.rs` — a different crate — alone under the suffix. The pre-fix checker stays **green** and silently converts twelve citations from skipped to resolved:

```text
  partial path 482 resolved by unique suffix, 317 skipped as ambiguous, 32 skipped as rooted outside this tree
check-citations: every pinned citation and every local markdown link resolves. ...
```

Against 470 resolved and 329 ambiguous before the deletion. The fixed checker exits 1 with twelve failures:

```text
FAIL  docs/research/runtime/backend-scoped-route-requirement-answers.md
        citation: `applicability.rs:843`
        applicability.rs is on the retired-ambiguity ledger in check-citations.sh, and exactly one tracked file ends with it now: crates/tiler-metal/src/applicability.rs. A twin was deleted, so resolving would point this citation at the survivor rather than at the file it was written about. Re-read which file the claim is about and pin a path long enough to be unique on its own.

  ambiguity    41 ledger entry(s) against a floor of 41, 318 citation(s) matched one, 12 collapsed to a survivor, 0 ambiguous off the ledger
check-citations: 12 citation(s) do not resolve against this tree.
```

**2. The same scenario on a suffix that already carries a directory component**, which is the negative control for the rejected alternative. `git rm crates/tiler-artifact/src/program/verify.rs` leaves `crates/tiler-ir/src/program/verify.rs`. Pre-fix, green again, three more citations silently resolved (`473 resolved by unique suffix, 326 skipped as ambiguous`). Post-fix:

```text
FAIL  docs/research/indexing/concatenate-fusion-role-and-lowering.md
        citation: `program/verify.rs:203`
        program/verify.rs is on the retired-ambiguity ledger in check-citations.sh, and exactly one tracked file ends with it now: crates/tiler-ir/src/program/verify.rs. ...
check-citations: 3 citation(s) do not resolve against this tree.
```

**3. Ambiguity forming rather than collapsing.** Adding a tracked `spikes/explain.rs` beside `crates/tiler-compiler/src/explain.rs`. Pre-fix the checker stays green while 23 citations drop out of the checked population (`447 resolved by unique suffix, 352 skipped as ambiguous`). Post-fix, 23 failures:

```text
        citation: `explain.rs:4149`
        explain.rs matches 2 tracked files but is absent from the retired-ambiguity ledger in check-citations.sh, so it turned ambiguous after that ledger was written and this citation has silently stopped being checked. Pin a path long enough to be unique on its own, or add the suffix to the ledger so a later deletion cannot repoint it.
```

**4. The two floors, perturbed separately** so neither stands in for the other. Cutting three entries out of the ledger heredoc:

```text
SHORT  the retired-ambiguity ledger population reached 39 entry(s), below its floor of 41.
       Every entry records a suffix observed ambiguous while a live citation rested on it, and entries are added rather than pruned; ...
```

Removing the ledger consult from the ambiguous branch — restoring exactly the pre-fix behaviour there, and nothing else — leaves the whole report byte-identical to a green run except:

```text
  ambiguity    41 ledger entry(s) against a floor of 41, 0 citation(s) matched one, 0 collapsed to a survivor, 0 ambiguous off the ledger

UNEXERCISED  the retired-ambiguity ledger lookup: parsed 0 times, so nothing exercised that path.
             The built-in fixture carries `lib.rs:1`, a suffix 44 tracked files end with, which should have fed it.
```

That floor is fed by a new fixture citation rather than by the corpus: the built-in fixture now carries `lib.rs:1`, a suffix 44 tracked files end with, so a matcher that stops consulting the ledger fails even on an empty corpus.

### Not covered, and where it went

- Families that collapsed **before** the ledger was seeded leave no observation to seed from. Measured today: exactly one such suffix has a live citation, and reading it shows no defect. [`ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded`](ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded.md).
- A partial path whose **leading component** vanishes from the tree is skipped as belonging to another project instead of failing. That is a coverage hole rather than a wrong resolution, and it is unoccupied today — all 32 citations reaching that branch are genuinely upstream. [`fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it`](fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it.md).
- The ledger does not make an ambiguous citation *checked*; it makes its ambiguity durable. Lengthening the 329 into unique paths is the strictly better end state and is the rejected alternative above, wanting a scope this branch does not hold.
