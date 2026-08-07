---
id: state-the-unreachable-refusal-convention-where-law-authors-read-it
title: State the unreachable refusal convention where law authors read it
status: done
priority: p2
dependencies: []
related: [accept-the-softmax-realization-law, accept-the-partitioned-concatenate-realization-law]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [docs, conventions]
---
## What this owes

**Tom ruled on 2026-08-07** that a realization law may state a refusal rule that is unreachable from a verified occurrence. That ruling currently lives in two closed acceptance nodes, which is where the next law author will not look. Write it where they will: at `IndexRealizationLaw`'s own definition in `crates/tiler-ir/src/index/law.rs`, as the vocabulary's stated convention.

## The convention, with its fence — and the fence is the load-bearing half

**A law may state a refusal a re-read subject can reach, even though no construction path reaches it.** The ground: a law is interpreted against a *subject*, not against the inferencer that produced it. `IndexRefinementSubject::derive` builds a subject from the family's own inferencer, so the malformed cases are refused before a subject exists — but a hand-built or re-read subject is not so constrained, and the law is what answers it. That is a **reinterpretation boundary**.

**It does not extend to a construction-path refusal nothing can reach.** On the same day, a mixed-width refusal proposed for the BF16 reference was **rejected** on exactly the opposite ground: `region_arithmetic_type` is a total function from a `ScalarProgram` to one `ArithmeticType`, so no constructible program could ever fire it, and a check that can never be watched failing makes a maturity claim the evidence cannot support. Both rulings are the same principle applied to different facts — state a refusal something can reach; do not state one nothing can — and the convention is wrong if it is read as blanket permission for untested checks.

So the text must carry **both** halves and the test that separates them: *can a subject reach this rule by any route, including one no current producer takes?* If yes, state it. If no, it is not a check.

## Concretely

- The convention at `IndexRealizationLaw`'s definition, naming the currently-unreachable rules as its worked instances: `softmax-reduced-axis-rank`, and concatenate's `concatenate-result-shape`, `concatenate-operand-binding` and `concatenate-result-arity`.
- Each of those four rules' own doc says it is unreachable from a verified occurrence *and why it is stated anyway*, so the reason travels with the rule rather than only with the vocabulary.
- Do not weaken the standing requirement that a **reachable** refusal is watched failing before it is trusted. This convention is an exception for one class, not a relaxation of the discipline.

## Explicit non-goals

No behaviour change, no new rule, no rule removed, and no identity movement — this is documentation of an accepted convention. Do not extend it beyond realization laws; whether it generalizes to any other vocabulary is not decided and should not be implied by where the text is placed.

## Closes when

The convention and its fence are stated at `IndexRealizationLaw`, the four unreachable rules each carry their own reason, and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir` passes.

## Outcome — delivered 2026-08-07 at `5d6ce320`

The convention sits at `IndexRealizationLaw`'s own enum doc in `crates/tiler-ir/src/index/law.rs`, under **"Stating a refusal no construction path reaches"**, carrying all four parts the ticket demanded: the permission, the fence, the test that separates them, and the explicit non-relaxation. The closing sentence scopes it to the realization-law vocabulary and states that whether it reaches any other is undecided — so the placement cannot imply generalization.

**All four rules were confirmed unreachable by reading rather than by trusting the brief**, and the reasoning is better than the brief's in two places.

- `softmax-reduced-axis-rank` — the family's inferencer refuses absent, duplicated and wrong-rank axes before `derive` runs. The worker also checked the **cross-family route** the existing tests use, since they apply this law to rms-norm and serial-sum subjects: the only other family matching the required record shape is the strict serial sum, whose result always drops at least one axis, so `softmax-shape` refuses first. Reindex, broadcast, slice and contraction encode their attribute as a record, so a different rule answers.
- `concatenate-result-shape` — the inferencer derives the declared result with the *identical* call over the same ordered operand shapes, so the re-derivation agrees by construction. The worker read **all four sites** carrying the rule name rather than only the disagreement one, and documented that the last two are downstream of the derivation already having proved the axis within the shared rank.
- `concatenate-operand-binding` — **the brief's phrasing was slightly wrong and the worker said so.** The ground is structural to `derive`, which builds the operand list *as* indices into the boundary list it collects, rather than to any inferencer. The doc states the actual ground.
- `concatenate-result-arity` — every registered family declares exactly one result, but the graph admits up to `MAX_OPERATION_RESULTS = 1024`. **This becomes reachable the moment a multi-result family is registered**, which is the cleanest instance of the fence's "a route no current producer takes" and the best argument that the convention is a real permission rather than a licence.

**One correction beyond the brief.** `the_concatenate_law_refuses_occurrences_outside_its_form`'s doc said "the **two** remaining rules" and named two; there are three, omitting `concatenate-result-arity`. Corrected and pointed at the convention.

**No pin moved**, and the check was not a silent pass: the golden identity test was run explicitly on the finished commit (1 run, 1 passed) rather than inferred from a green suite, and the worker checked the one comment-reaches-identity hazard the ledger records — it is `tiler-metal`'s *emitted MSL* provenance header, which Rust doc comments in `tiler-ir` do not enter.

**Deliberately no test added**, with the right reason: watching an unreachable rule fire would require hand-building a subject no constructor admits, and would contradict the fence being documented.

`make full` exit 0 on the branch; re-gated on the merged tree at integration.
