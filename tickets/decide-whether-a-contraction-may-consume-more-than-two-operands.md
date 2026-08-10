---
id: decide-whether-a-contraction-may-consume-more-than-two-operands
title: Decide whether a semantic contraction may consume more than two operands
status: deferred
priority: p3
dependencies: []
related: [decide-whether-to-admit-a-distributivity-permission, admit-the-contraction-semantic-profile, decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys]
scopes: [contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, decision, needs-tom]
---
## User-visible outcome

The last of ADR 0087's three reserved contraction choices has an owner, so "may a contraction node consume more than two operands" stops being a question the corpus reserves for Tom in three places and schedules nowhere.

## Why this exists

**Fact — three choices were reserved and this is the one with nothing.** [`decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys`](decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys.md) states it: "Of the three reserved choices, only the distributivity one has a ticket, in [`decide-whether-to-admit-a-distributivity-permission`]." Choice 1 is settled — that ticket is `done` and [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) accepts one keyed family carrying a renaming-invariant index-structure attribute. Choice 3 was decided on 2026-08-01 (declined; see the distributivity ticket). Choice 2 had no node.

**Fact — the corpus reserves it in three places.** Under `#### Decisions reserved for Tom` in `docs/roadmap.md`, item 2 of the Milestone 6 framing: "**Whether a semantic contraction node may consume more than two operands.**" In Q-SEM-015's trigger bullet in `docs/open-questions.md`: "Still reserved from the framing, and the only one of the three left: whether a semantic contraction node may consume more than two operands." And the sibling ticket, quoted above.

**Fact — a live refusal is what the answer would move.** `crates/tiler-ir/src/semantic/contraction.rs` reports the rule under diagnostic code `contraction.rule.index-in-more-than-two-operands`, one of the five structural admission rules that refuse at construction under their own named provider diagnostics. The Milestone 6 structural-rejection list in `docs/roadmap.md` states the rule's content — `for a multi-operand form, an index appearing in more than two operands`. The refusal is correct under the current decision; this question is whether the decision should change, not whether the refusal is a defect.

**Inference — the two remaining reserved choices are independent, and the answer here does not follow from choice 3's.** [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) records that "the derivation holds under either answer to the multi-operand question", and `docs/open-questions.md` says the same from the other side: "The three are independent: the distributivity derivation, and therefore ADR 0095's decline, holds under either answer to the multi-operand choice." So choice 3 being decided settles nothing here.

## Why this is deferred rather than asked

Nothing in the tree presses it. The contraction reached R6 through `normalize_contraction` in `crates/tiler-compiler/src/request.rs`, which admits every well-formed **binary** structure (docs and guards: exactly two operands; structure operand count must be two), and the support-matrix contraction row records `R6 for a whole-program contraction occurrence`; all three of the pinned workload's index structures — `td,od->to`, `grtd,gsd->grts`, `grts,gsd->grtd` — are binary. Putting a question to Tom that no workload can illustrate would spend his time on a hypothetical, and AGENTS.md asks a decision packet to carry a small concrete tensor program.

**Reconsideration trigger:** a named workload or frontend lowering that requires an index shared by three or more operands — concretely, a contraction whose natural spelling is refused by `contraction.rule.index-in-more-than-two-operands` and whose binary decomposition is either more expensive or not order-equivalent under the workload's numerical contract. The second half matters: if every candidate decomposes into binary contractions with equivalent semantics and no cost penalty, the trigger has not fired and the refusal stands. Until then, the rule refuses explicitly and the corpus names the reservation.

## Closes when

The trigger fires and this becomes one atomic question to Tom with the workload attached; or an accepted ADR settles it and the three reservation sites above are updated to point at that record; or the trigger is confirmed unfireable with the derivation recorded. Closing this needs an ADR in `docs/decisions/`, which is why this ticket claims `contracts/decisions` alongside `contracts/numerics`.

## Trigger check log

- 2026-08-04 — **not fired.** No workload's natural spelling is refused by `contraction.rule.index-in-more-than-two-operands`; the pinned workload's three index structures remain binary, and nothing added since states a three-operand shared index. Recheck: `grep -rn 'index-in-more-than-two-operands' crates/tiler-ir/src/semantic/contraction.rs`.
- 2026-08-09 — **not fired.** The semantic verifier still emits `contraction.rule.index-in-more-than-two-operands`, and the current tests exercise it only as a refusal. No named workload records a natural three-operand shared-index contraction or shows binary decomposition to be more expensive or numerically inequivalent. Recheck at that exact rule anchor in `crates/tiler-ir/src/semantic/contraction.rs` and its refusal assertions in `crates/tiler-ir/src/semantic/contraction/tests.rs`.
- 2026-08-10 — **not fired.** Same evidence: diagnostic `contraction.rule.index-in-more-than-two-operands` remains a structural refusal only; `normalize_contraction` still requires binary structure and program input arity; no named workload or frontend requires an index shared by three or more operands. Recheck: the diagnostic code in `crates/tiler-ir/src/semantic/contraction.rs` and its refusal assertions in `crates/tiler-ir/src/semantic/contraction/tests.rs`; binary guards on `normalize_contraction` in `crates/tiler-compiler/src/request.rs`.
