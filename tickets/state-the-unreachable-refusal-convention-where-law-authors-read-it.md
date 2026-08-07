---
id: state-the-unreachable-refusal-convention-where-law-authors-read-it
title: State the unreachable refusal convention where law authors read it
status: todo
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
