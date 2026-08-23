---
id: make-the-draft-time-index-traversals-outside-compact-rs-exhaustive
title: Make the draft-time index traversals outside compact.rs exhaustive
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing]
---
## User-visible outcome

The draft-time index traversals outside `compact.rs` visit every variant and field of the records they walk, so a widened vocabulary is a build error at the walk rather than a branch silently never taken.

## Why this exists

Filed 2026-08-23 by the coordinator as the reported remainder of [`make-the-index-layer-traversals-exhaustive-over-the-records-they-walk`](make-the-index-layer-traversals-exhaustive-over-the-records-they-walk.md), which landed as `f5f4cff1` and took `crates/tiler-ir/src/index/builder/compact.rs` from **9** rest patterns to **0** with no wildcard match arms left. That lane reported this population and correctly declined it as outside its file scope and materially larger.

**Fact — the population, measured by the coordinator at `a0fd5af2`.** `grep -c '\.\. *}'` under `crates/tiler-ir/src/index/builder/`: **`proof.rs` 18**, **`reduction.rs` 2**, **`gather.rs` 1**. The delivering lane reported "at least 5 sites" in `proof.rs`; that was a conservative floor and the measured figure is higher. **Re-derive it yourself** — a raw `grep -c` counts lines and does not distinguish a traversal from a deliberate single-field projection, which is the distinction this ticket turns on.

**Fact — repaired 2026-08-23 by `worker-drafttrav` at `e846b62e`; the count above is right for what it counts and undercounts the population three ways.** Verified per file. (1) `grep -c` counts *lines*: `proof.rs` holds **22** occurrences on those 18 lines, because four arms pair `FloorDiv` and `Modulo` on a single line. (2) The pattern `\.\. *}` requires the brace on the rest's own line, so it misses `proof.rs`'s multi-line `definitely_outside, ..` in `verify_accesses`, whose `}` is on the next line — 19 rest-pattern lines and **23** occurrences, not 18 and 22. (3) Anchoring on the `builder/` directory excludes the parent module file `crates/tiler-ir/src/index/builder.rs`, which carries three more lines of the same rest pattern — five occurrences — across two draft-time walks: `expression_reads_environment` — the draft-time twin of the `gather.rs` walk this ticket names, whose doc comment says it "Mirrors `IndexRegionBuilder::expression_reads_environment`" — and `check_index_node_integers`. Over-anchoring under-counted here in the direction that reads as clean, which is the hazard `AGENTS.md` states.

**Fact — the elision is demonstrably silent, and the delivering lane proved it on this very population.** Its perturbation added `probe: bool` to `IndexNode::FloorDiv`. With `compact.rs` repaired the compiler raised `error[E0027]` at two traversal spans; with `compact.rs` reverted to base those two spans **disappeared entirely** — 9 errors instead of 11 — and the field compiled straight through both walks.

**Correction — 2026-08-23 by `worker-drafttrav` at `e846b62e`.** The sentence that stood here, "It recorded that `crates/tiler-ir/src/index/builder/gather.rs:205` compiles clean through a widened `FloorDiv` today", is **false**, and it is false in the direction that would have sent a worker to the wrong arm. `gather.rs` is byte-identical at `a0fd5af2` and `e846b62e` (`git diff a0fd5af2 e846b62e -- crates/tiler-ir/src/index/builder/gather.rs` is empty), and at both commits line 205 reads `IndexNode::FloorDiv { dividend, divisor } | IndexNode::Modulo { dividend, divisor } =>`, which binds both fields with **no rest pattern**. Adding `probe: bool` to `FloorDiv` at the base tree raises `error[E0027]` at `gather.rs:205:9`, measured. The file's single rest pattern is on the *preceding* arm, `IndexNode::LinearCombination { terms, .. }` at line 201, and the perturbation that reaches it widens `LinearCombination`, not `FloorDiv`. Cite the arm by its pattern text, not by the line.

**Why a traversal matters differently from an encoder.** An encoder that elides a field silently narrows an *identity*; a traversal that elides one silently fails to *visit*, so a widened variant is never reached and whatever the walk collects is quietly incomplete. `proof.rs` is the sharpest case: its `verify` computes reachable accesses, reachable operations, and used reductions, and an unvisited branch there is an obligation that goes unchecked rather than a byte that goes unwritten.

## Required work

- Re-audit the population at your base with a per-file verdict, and **census by record type reachable from `CompactedRegion`**, not by grep string. The delivering lane's spelling — `grep -n '\.\. *}'` — is the one that catches a rest pattern regardless of the binding name, because these loops bind single letters (`t.role`, `d.extent`, `v.definition`) that any `node.`-shaped search misses. State your vocabulary and why it is complete.
- **A rest pattern is not automatically a defect.** A walk that genuinely needs one field may be right to say so; where it is, record why at the site rather than widening it. The delivering lane left `alpha_dimension_order`, `visit_access_dimensions`, and the `AccessData` accessors unchanged for exactly that reason, and its reasoning is worth reading before you touch the analogous sites here.
- Where the walk should be exhaustive, remove the rest pattern and bind unused fields to `_`.
- Perturb by widening a variant and quote the build error, confirming the span lands **in the traversal**. Reproduce the negative control: with the file at base, that span must be absent — that asymmetry is the whole demonstration. **One perturbation does not reach every walk here**: widening `IndexNode::FloorDiv` never touches `verify`'s three filters, which walk `ScalarValueDefinition` and `ScalarOperationKindData`, nor `compact_reducer_body`, which walks `ReducerBodyValueSource`, nor the `IntervalVerdict` projection. Perturb each record separately, or the run that reddens everything will not show which walk is load-bearing.
- **State whether any identity value moves. Expected: none** — these are draft-time walks consumed before compaction, not encoders. Rederive rather than assume, and stop and report if one does.

## Non-goals

`compact.rs`, done by `f5f4cff1`. The identity encoders and their length twins, done by `a0659d05`. `remap_node` and `remap_operation`, which build through exhaustive struct literals and already error on a new field. Changing what any traversal collects.

## Closes when

Every draft-time traversal under `crates/tiler-ir/src/index/builder/` either visits its record exhaustively or records why a rest pattern is correct there, the census states its vocabulary and why it is complete, no identity value has moved, and a widening perturbation is watched failing at a traversal span with its base-tree negative control quoted.

## Coordinator accountability — 2026-08-23: the false Fact was mine, and my census undercounted three ways

`worker-drafttrav` repaired this ticket's Fact and was right. Recording the coordinator's side of it, because the ticket text was mine.

**The false Fact.** I wrote that *"`crates/tiler-ir/src/index/builder/gather.rs:205` compiles clean through a widened `FloorDiv` today."* Retired wording preserved. Verified at `a0fd5af2`, the base I filed against: line 205 reads `IndexNode::FloorDiv { dividend, divisor } | IndexNode::Modulo { dividend, divisor } =>` — **both fields bound, no rest pattern** — so widening `FloorDiv` raises `E0027` there rather than compiling through. The file's single rest pattern is the **preceding** arm, `LinearCombination { terms, .. }` at line **201**, and reaching it requires widening `LinearCombination`, a different perturbation. A worker trusting me would have opened an already-exhaustive arm, found nothing wrong, and concluded the site was fine.

**I relayed it rather than checking it.** It came from the delivering lane's out-of-scope report and I carried it into a ticket without opening the file — the same failure as three anchor citations earlier in this session, and the coordinator obligation is explicit: *"run it yourself first — a supplied command that has never been executed is a claim, not a check."*

**The census undercounted three ways, each in the direction that reads as clean.** All three verified at `a0fd5af2`:

- **`grep -c` counts lines, not occurrences.** `proof.rs` shows **18** lines but **22** occurrences, because four arms pair `FloorDiv` and `Modulo` on one line. This is the hazard AGENTS.md records at `grep -c` counts **lines**, not occurrences — committed by the coordinator while citing it to workers.
- **The pattern needed the brace on the rest's own line**, missing `IntervalVerdict { definitely_outside, .. }` whose `}` wraps. True figure 19 lines / 23 occurrences.
- **Anchoring on the `builder/` directory excluded the parent module file.** `crates/tiler-ir/src/index/builder.rs` carries **3** more lines / 5 occurrences, in two draft-time walks — one of which is the twin of the `gather.rs` walk this ticket named, with the two doc comments cross-referencing each other. Repairing one and leaving the other would have split a single question across two spellings.

The lane widened its own vocabulary to a bare `\.\.` scan classified by reading, which is what made the set complete. That is the correct response to a census whose anchor is the coordinator's guess.

**End state verified by the coordinator on the merged tree:** zero rest patterns remain in non-test code across `index/builder.rs`, `proof.rs`, `reduction.rs`, `gather.rs`, and `compact.rs`. The single residual is `proof.rs:1807`, inside `#[cfg(test)] mod tests` (which opens at line 1737) — deliberately left, and correctly.
