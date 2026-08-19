---
id: stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin
title: Stop the citation checker's ambiguity skip resolving against a basename twin
status: in-progress
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue, split-the-index-refinement-module-into-cohesive-submodules]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: [check-citations.sh]
tags: [gates, citations, correctness]
claimed_from: todo
assignee: worker-citation-checker
lease_expires_at: 1787157651
---
## User-visible outcome

Deleting one of two files sharing a basename can no longer convert a skipped-as-ambiguous partial-path citation into one that silently resolves green against the surviving twin: the checker either keeps a record of retired ambiguity or resolves partial paths in a way a deletion cannot repoint.

## Why this exists — filed 2026-08-19 from the refinement split's delivery

**Fact (verified by the split lane at `a2e98b27`).** `check-citations.sh` skips partial paths matched by more than one file. The refinement split deleted `crates/tiler-ir/src/index/refinement.rs`, making the bare suffix `refinement.rs` — previously ambiguous with `crates/tiler-ir/src/semantic/accuracy/refinement.rs` and therefore skipped — a unique suffix. One live snapshot citation began resolving **green against the wrong file**; four siblings failed only because their line numbers exceeded the surviving file's 659 lines. A shorter survivor would have made all five silently green. The population at risk is every skipped-ambiguous partial path (287 reported by the checker at recent runs) whose basename family shrinks to one.

## Required work

- Read `check-citations.sh` in full, including its stated rationale for which link shapes it declines to resolve.
- Choose and implement the fail-safe: candidates include refusing to resolve a bare-basename partial path at all (require at least one directory component), or failing loudly when a partial path resolves uniquely but the citing document predates the uniqueness — derive the actual design from the script's existing structure rather than this list.
- Perturb the subject, not the assertion: reproduce the twin scenario in a fixture (two files sharing a basename, a citation on the suffix, delete one) and quote the checker's failure text before and after the fix. Where the script's built-in fixture population supports it, extend that fixture rather than a one-off.
- Census the currently-skipped ambiguous population for suffixes one deletion away from uniqueness; state the count.
- `make citations` must stay green on the current tree, or every newly failing citation must be repaired in the same change with the read-the-claim discipline.

## Closes when

The twin scenario has a quoted failing check, the fix's perturbation evidence is recorded, the near-unique census is stated, and `make citations` is green.
