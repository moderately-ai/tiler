---
id: answer-per-ordinal-element-counts-only-for-ordinals-an-output-reads
title: Answer per-ordinal element counts only for ordinals an output reads
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`resolve_work_items` stops spuriously refusing an opaque call's `PerElementOf` scaling in a multi-output program whose outputs iterate different domains — a false negative today, because a non-reading output volunteers an element count for an ordinal it never loads and breaks the agreement fold.

## Why this exists (audited 2026-08-06, coordinator-verified: input_keys = program.inputs() at request.rs:5154; agreed fold at :1768; sole caller frontier.rs:2676)

`NormalizedOutput::input_elements_at` (`crates/tiler-compiler/src/request.rs:1514`) answers for every ordinal below the program's declared arity in its `SerialSum` and `Pointwise` arms, because `input_keys` is the whole program's declared list. Its two siblings on the same type — `max_input_elements` and `reads_declared_input` — were corrected to read the recognized read lists when subset reads landed; this one was not, and its comment ("Every declared input of a reduced program is read at the contributor domain") is the stale premise. The `Epilogue` arm guards its own half then recurses into the unguarded producer arm.

## The work

Both arms gate on `self.reads_declared_input(ordinal)` — the authority already on the type — before answering; the comment restates per-read truth. Failure perturbation: two outputs over disjoint inputs at different extents with an opaque call bound to ordinal 0 and `PerElementOf` — before: `UnknownParameter`; after: the reading output's count — and the genuine one-input-two-domains disagreement case still refusing, or the fix removed the check.

## Closes when

Both arms answer only for read ordinals, both perturbations are observed, and the stale comment is corrected.

## Outcome — delivered 2026-08-07 at `03c6e1bf`

Both single-shape arms of `NormalizedOutput::input_elements_at` now gate on `reads_declared_input` instead of the declared arity, and the stale premise is replaced rather than patched: "every declared input of a reduced program is read at the contributor domain" held only while every walk had to read every declared input, and the doc now says why it expired.

**One change beyond the ticket's literal text, and it was required by the stated outcome rather than added to it.** Gating the arms alone does not deliver the user-visible result — verified rather than assumed: with the gate applied and the fold untouched, the disjoint-inputs fixture still reported `UnknownParameter`, because `agreed` compares `Option`s and a silent output's `None` is a *value* that disagrees with every count rather than an abstention. So `agreed_input_elements_at` folds over the reading outputs only.

**The filter's predicate is the subtle part and it is right.** It asks `reads_declared_input` rather than "did this output produce a count". Filtering on the answer would silently drop a genuine refusal — an epilogue chain that reads one ordinal from *both* halves at two domains answers `None` because it has no single domain, and that disagreement must survive. Filtering on the answer would let a sibling's count stand for a chain that has none.

**A gap in the ticket's own stated evidence, found and closed.** The first perturbation — reverting the arms to the arity bound — is **not observable** on the disjoint-outputs fixture this ticket named, because the program-scoped filter already excludes the non-reading output there. A second fixture was added, an epilogue chain whose producer never folds the ordinal its epilogue reads, which is where the arm gate is genuinely load-bearing. Without it the "both arms answer only for read ordinals" claim would have been untested despite three passing perturbations.

**No pin moved** — `357f0676…`, `c626e43b…`, 65,242 bytes and descriptor 2,099 byte-identical at base and head, with the pin test and the descriptor assertion both run. No public surface added or widened. `make full` exit 0 on the branch and again on the merged tree: 2,976 workspace, 1,045 release.

**A correction to the coordinator's brief, made correctly.** The brief stated `project/tickets` was already declared shared; the ticket file said otherwise. The worker read the repository over the brief and added it — the right precedence, and the error was mine.

### Released — an out-of-scope defect found by reading, with a measured probe

[`answer-input-element-counts-as-the-declared-tensors-own-count`](answer-input-element-counts-as-the-declared-tensors-own-count.md). The five arms disagree about *what* they count: `Contraction` and `Staged` answer the operand tensor's own count, while the two single-shape arms answer the region's iteration domain. Those coincide for a dense read and diverge for a widening one — measured on `a * broadcast(w)` with `w: [2]`, where ordinal 0 answers **4** for a tensor holding **2** elements. Since a call binds `TensorRole::Input { ordinal }`, which names the ABI's buffer, that over-counts work: the confidently-wrong work count `WorkScaling` exists to prevent.

That ticket also carries a consequence for *this* one, recorded so it is not lost: settling it makes the two outputs of the shared fixture agree on ordinal 0, so this ticket's refusing-neighbour assertion must be **re-founded on a surviving fixture** rather than merely re-run.
