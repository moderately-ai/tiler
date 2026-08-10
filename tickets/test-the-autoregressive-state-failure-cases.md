---
id: test-the-autoregressive-state-failure-cases
title: Test the autoregressive failure cases over caller-retained tensors
status: todo
priority: p1
dependencies: [integrate-the-autoregressive-decode-loop, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, prove-the-c1-stateful-attention-vertical, evaluate-retained-shape-relations-before-routing-commit, admit-a-position-selecting-slice-for-the-rotary-table, bind-repeated-invocations-over-caller-retained-tensors]
scopes: [implementation/runtime, implementation/candle, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, fail-closed, consumer-neutral, language-model, class-conformance-fixture]
---
## User-visible outcome

Every failure this decode loop's contract claims to catch is a test that fails
without the check — and the ones nothing catches are recorded as uncaught rather
than left looking covered.

## Case correction — 2026-08-04

Rewritten under
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).
Two of the seven cases required refusals from a Tiler-owned KV state and are
withdrawn with it. **A withdrawn case is not a case that passes**, and both are
listed below with their reason so that a reader who remembers them can see they
were removed deliberately. `implementation/runtime` and `implementation/artifact`
stay declared: case 4's device-scope refusal is an adapter obligation and case 5's
is an artifact-assembly refusal, and both are generic.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Case 2 is retitled and scoped to the L5 dual: it refuses a mutually inconsistent `S,C,T` triple before the routing commit; it does **not** detect content-stale allocations whose bound extents are consistent (consumer obligation, out of this suite's refusal set). Case 6's pin is the L5 invariant-2 load-bearing claim — eight decode steps at C1 share one artifact identity — not an unconditional nine-as-one over prefill+decode; prefill sharing that identity remains conditional on symbolic packaging / D-19. Closes when splits case 2 (full refusal once evaluate-retained has landed, expected before this ticket is ready) from case 7 (recorded uncaught / differential at close). Case 3's "names the step" is the driver step index composed with Tiler's stage-named failure.

## Required cases

Each is paired with an accepted baseline, so a refusal is evidence about the one
thing the case changed.

1. ~~**Capacity exhaustion.**~~ **Withdrawn.** It required `C + T > capacity` to
   refuse inside Tiler. Tiler is handed one tensor per invocation at one bound
   extent and has no capacity to compare against; a driver's bound on its own
   pool is a driver test, and the driver owns it under
   [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md).
2. **Inconsistent extent bindings.** An invocation whose bound extents violate
   `S == C + T` refuses before the routing commit. This case tests the generic
   refusal that a *mutually inconsistent* triple of bound extents never reaches
   program work; it does **not** detect content-stale allocations whose extents
   are consistent with each other (bytes last written shorter than the bound
   extent) — that remains a consumer obligation and is out of this suite's
   refusal set. The relation is representable today; consuming it at launch
   preflight is
   [`evaluate-retained-shape-relations-before-routing-commit`](evaluate-retained-shape-relations-before-routing-commit.md).
   Until that lands, this case is recorded as *not refusable* rather than as
   passing. Once evaluate-retained has landed (expected before this ticket is
   ready via the integrate → execute-decode → evaluate chain), the case is a
   full watched-failing refusal test, not a permanent limitation.
3. **Post-commit failure.** *Split on 2026-08-04 from the case that read "Partial
   update".* Its Tiler half survives and is the case: force a dispatch failure
   after the routing commit, then assert that every bound input tensor is
   bit-identical, that no output is observable, and that the typed failure names
   the step (driver step index composed with Tiler's stage-named failure; not a
   public execution-ordinal field). Its other half — "poisons the state; a
   subsequent bind of that state refuses" — is withdrawn, because no Tiler
   object survives the invocation to refuse a later bind. The consumer-side
   obligation not to continue from pre-failure tensors is the decode loop's and
   is asserted there, not here.
4. **Cross-device reuse.** A value allocated against one bound context, bound
   under another, refuses at the adapter. The loader cannot detect it —
   `ExecutionEnvironment` carries a target profile, a backend family, and a
   representation, and two devices of one family classify identically — so the
   test must exercise the adapter's own check and must fail if that check is
   removed. *Generalized 2026-08-04:* the subject is a bound value rather than a
   KV state, so the case covers any invocation and needs no state type.
5. **Specialization contamination.** A packaged program specializing a kernel on
   a per-invocation bound extent is refused at artifact assembly; the accepted
   neighbour binding that extent routes.
6. **One identity across decode steps.** Eight decode steps at C1 (`T = 1`,
   `S = 11…18`) produce exactly one artifact identity (the anti-specialization-on-
   `S`/`C` invariant). A change that compiled per step fails here. Prefill
   (`T = 10`) sharing that same identity is conditional on symbolic packaging /
   D-19 broadcast-result-extent work from the 2026-08-05 L5 correction — record
   the measured identity count rather than assert nine invocations as one
   absolute pin.
7. **Incorrect position, recorded as uncaught.** Binding the wrong rotary rows
   produces a plausible wrong result that every layer accepts. The test asserts
   *that* — a differential against the conformance oracle — rather than asserting
   a refusal that does not exist, and it names
   [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md)
   as the work that would narrow it.

## Closes when

Every surviving case above runs, each refusal has been watched to fail against a
deliberate perturbation and to pass against its accepted baseline, case 2 is a
full refusal test once
[`evaluate-retained-shape-relations-before-routing-commit`](evaluate-retained-shape-relations-before-routing-commit.md)
has landed (not left as a skipped permanent limitation), case 7 is recorded with
its exact residual (uncaught / differential) rather than skipped or claimed as a
refusal, and the withdrawn capacity case is absent from the harness rather than
present and green.
