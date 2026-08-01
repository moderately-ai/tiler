---
id: test-the-autoregressive-state-failure-cases
title: Test the autoregressive state failure cases
status: todo
priority: p1
dependencies: [integrate-the-autoregressive-decode-loop]
related: [design-autoregressive-state-and-kv-cache, prove-the-c1-stateful-attention-vertical]
scopes: [implementation/runtime, implementation/candle, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, kv-cache, fail-closed, language-model]
---
## User-visible outcome

Every failure the state contract claims to catch is a test that fails without the check — and the one it cannot catch is recorded as uncaught rather than left looking covered.

## Required cases

Each is paired with an accepted baseline, so a refusal is evidence about the one thing the case changed.

1. **Capacity exhaustion.** `C + T > capacity` refuses before any program work, naming both quantities.
2. **Stale state.** A bind whose `S` does not equal `C + T` refuses at the shape environment. Requires [`admit-an-additive-extent-relation`](admit-an-additive-extent-relation.md); until it lands, this case is recorded as *not refusable* rather than as passing.
3. **Partial update.** A dispatch failure after the routing commit leaves the input state bit-identical, publishes nothing, does not advance the cursor, and poisons the state; a subsequent bind of that state refuses.
4. **Cross-device reuse.** A state scoped to one bound context, bound under another, refuses at the adapter. The loader cannot detect this — `ExecutionEnvironment` carries a target profile, a backend family, and a representation, and two devices of one family classify identically — so the test must exercise the adapter's own check and must fail if that check is removed.
5. **Specialization contamination.** A packaged program specializing a kernel on `S` is refused at artifact assembly; the accepted neighbour binding `S` as an extent routes.
6. **One identity across the run.** C1's nine executions produce one artifact identity. A change that compiled per step fails here.
7. **Incorrect position, recorded as uncaught.** Binding the wrong rotary rows produces a plausible wrong result that every layer accepts. The test asserts *that* — a differential against the conformance oracle — rather than asserting a refusal that does not exist, and it names [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) as the work that would narrow it.

## Closes when

Every case above runs, each refusal has been watched to fail against a deliberate perturbation and to pass against its accepted baseline, and cases 2 and 7 are recorded with their exact current limitation rather than skipped.
