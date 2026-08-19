---
id: fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it
title: Fail a partial path whose leading component has vanished instead of skipping it as external
status: done
priority: p3
dependencies: []
related: [stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: [check-citations.sh]
tags: [gates, citations, correctness]
---
## User-visible outcome

Deleting a whole directory can no longer convert a Tiler citation into one the checker skips as belonging to another project.

## Why this exists — filed 2026-08-19 from the ledger work

**Fact.** `check-citations.sh` decides that an unresolvable partial path belongs to another repository by testing its leading component against every component of every tracked path: the branch is guarded by `if (sub(/\/.*$/, "", lead) && !(lead in component))` and reports `rooted outside this tree`. The rationale in the header is sound for `candle-core/src/metal_backend/device.rs` — demanding upstream paths resolve here is an unsatisfiable condition.

**Inference — the branch fails open on a directory deletion.** A live citation of `codec/encode.rs:443` fails loudly today if `encode.rs` goes away, because `codec` is still a tracked component. Delete the whole `codec` directory and the same citation is skipped as belonging to another project instead, so a genuinely broken citation reports nothing. This is a coverage hole rather than a wrong resolution — unlike the twin case the parent ticket fixed, it never points a reader at the wrong file — which is why it was left out of that change rather than folded into it.

**Measurement 2026-08-19 at `bda38064`.** ~~32 citations reach the branch, over 10 distinct paths~~, and reading them shows all are genuinely upstream: eight spans under `candle-core/` and `candle-metal-kernels/`, two under `MacOSX26.5.sdk/`. The hole is unoccupied today, so this is prevention rather than repair.

    ./check-citations.sh --verbose | sed -n 's/^SKIP  [^:]*: `\(.*\)` (rooted.*/\1/p' | sort -u

**Correction — 2026-08-19, worker Fact audit at `23eb1bf4`.** The count above is false and the enumeration above is right; they disagree because they came from different places. The enumeration command returns exactly the ten spans the sentence lists, and re-reading their carrying documents confirms every one is genuinely upstream. The `32` was read off the census line, which sums two branches under the name of one: `external++` is incremented both by the version-pinned skip (`external crate source`) and by the component skip (`rooted outside this tree`), while the census printed only `%d skipped as rooted outside this tree`. Both `external++` sites and that single census field are present at `bda38064` as well, so this is a misread of the script rather than tree drift.

At `23eb1bf4` the split is **16 citations reaching the component branch** and 16 reaching the version-pinned branch. The sixteen are ten distinct spans over **seven** distinct files, not ten distinct paths: `candle-core/src/metal_backend/device.rs` carries two spans and `candle-metal-kernels/src/kernel.rs` and `MTLDevice.h` one each in addition to their first. Reproduce both numbers:

    ./check-citations.sh --verbose | grep -c 'rooted in .*recorded upstream project'
    ./check-citations.sh --verbose | grep -c '(external crate source)'

The census now prints the two on separate fields, because a line that adds two exclusions under one name is the silence the header of that script argues against everywhere else. It is the direct cause of this false Fact.

**Fact — a sibling hole of the same shape, found during the audit and closed in the same change.** The version-pinned skip decided a path belonged to another project from its spelling alone, testing the path against `^[A-Za-z0-9_-]+-[0-9]+\.[0-9]+\.[0-9]+\/` and nothing else. Five tracked directories already carry a version-pinned name — `docs/research/numerics/sources/arrow-25.0.0`, `gcc-16.1.0`, `llvm-project-llvmorg-22.1.8`, `rust-reference-rust-1.97.1`, `tosa-1.0.1` — so a partial citation rooted in one of them was skipped while the file it named sat in the index. Demonstrated by planting a citation of `arrow-25.0.0/Schema.fbs` with a line pin of 99999 into this ticket: the pre-change script printed `SKIP ... (external crate source)` and exited 0, though that file is tracked and has 571 lines. Both skips now require the leading component to be absent from every tracked path.

## Required work

- Re-audit the 32 and the 10 at your own base before designing anything; an occupied hole changes what the fix may do.
- Decide how a Tiler-rooted partial path is told from an upstream one without leaning on a component that a deletion can remove. Candidates worth costing: an explicit list of upstream roots, recorded the way the retired-ambiguity ledger is; requiring an upstream citation to carry its provenance in the span; or testing the *trailing* components rather than the leading one. Derive the design from the script, and state what each option would do to the 32.
- Perturb the subject: delete a tracked directory whose name no other tracked path carries, with a live citation rooted in it, and quote the checker before and after.

## Closes when

A citation rooted in this tree fails when its directory is deleted, the upstream population still skips, and `make citations` is green.

## Design comparison, and what each candidate does to the population

Measured at `23eb1bf4` against the 16 citations that reach the component branch.

- **Explicit upstream-root list, recorded like the retired-ambiguity ledger — chosen.** Three entries cover all 16 (`candle-core` 6, `candle-metal-kernels` 5, `MacOSX26.5.sdk` 5), so it converts none of the skipped population into failures. It is provably never wider than the old test, because a listed root is required to be absent from every tracked path: the skip set is a subset of the old one, and the measured difference on this tree is empty. A root colliding with a tracked component aborts the run at exit 2 rather than being honoured. Unlike the ambiguity ledger it carries no entry floor, and the asymmetry is the argument: a truncated ledger removes failures that used to fire, whereas a truncated upstream list makes the citations resting on it start failing on the very next run, so truncation is already loud.
- **Requiring an upstream citation to carry its provenance inside the span — rejected.** Provenance for these citations lives in the prose beside the span by existing convention (the candle revision `31f35b14`, the SDK build `25F70`), so this means a new accepted span grammar plus an edit to all 16 sites across six documents. That is a change to what counts as a citation and to a documentation convention, which is wider than this ticket authorizes, and it buys nothing the list does not: both end at a recorded fact, one recorded once per project and one recorded once per citation.
- **Testing the trailing components rather than the leading one — rejected on correctness, not on cost.** It does not close the hole and it opens a larger one. Deleting a directory removes the file too, so neither the leading nor the trailing component survives and the citation still skips. Worse, the ordinary broken citation is one whose *file* is missing while its directories are fine, so a trailing-component test would silently skip exactly the population this check exists to catch.
- **Deriving upstream roots from `git log --diff-filter=D` — rejected for the reason the header already gives the ambiguity ledger.** It cannot tell a deletion from a rename, it makes the verdict depend on clone depth, and an upstream root that shares a name with a long-deleted local directory would fail forever.
- **Status quo — eliminated at the gate.** It fails open, silently, on the one event that breaks every citation under a directory at once.

## Outcome

`check-citations.sh` now decides that an unresolvable multi-segment path belongs to another project only from the recorded upstream-root list, and the version-pinned skip carries the same requirement that the leading component be absent from every tracked path. The two are counted on separate census fields.

Evidence, all at `23eb1bf4`:

- **Subject perturbation.** Deleting `crates/tiler-runtime/tests/adapter_route/` — a directory whose name no other tracked path carries, carrying the one live pinned citation rooted in it — broke six pinned citations. Before: five failed and the sixth printed `SKIP ... (rooted outside this tree: no tracked path has a adapter_route component)`. After: seven fail, the new one naming the path, the root, and what to do about it. Restored, and the tree verified clean.
- **The upstream population still skips.** The set of skipped spans is byte-identical before and after, all ten of them, as are the version-pinned set and the 332 ambiguous ones. The only census difference is the intended split of the shared counter.
- **The collision guard reaches its subject.** Adding `crates` to the list makes the run print `upstream root(s) recorded in check-citations.sh are also components of tracked paths` and exit 2, printing no census at all.

Not run here, and deliberately: this delta touches `check-citations.sh`, which is on the gate-carry list in `AGENTS.md`, so **the latest green gate does not carry**. `make full` is the integrator's.
