---
id: qualify-the-simdgroup-matrix-contraction-realization
title: Qualify or refuse the simdgroup-matrix contraction realization
status: done
priority: p2
dependencies: [realize-the-contraction-through-the-appendable-direct-path]
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, carry-the-dtype-on-the-metal-subnormal-flush-fact, declare-metal-numerical-honourability, exercise-opaque-admissions-downstream-of-the-frontier, pin-the-strict-contraction-simdgroup-refusal, research-an-explicit-seeded-fused-contraction-operation]
scopes: [implementation/metal, contracts/artifacts, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, contraction, target-facts, decision, needs-tom, public-boundary, identity]
---
## User-visible outcome

The fastest hand-written realization measured — `simdgroup_float8x8` — is about 1.6–1.7x faster than the tiled strict kernel on the four prefill cells whose `M` extent admits it, and about 3.6–4.5x slower than the library route on those cells. The 4.5x endpoint is the `t_prefill_mlp_512` cell; the `M = 10` C1 cell refuses the realization. The realization either needs a distinct declared numerical contract a caller can ask for, or must be refused for the current registered contraction with a reason a reader can act on. Today it is neither: it is measured to disagree with that registered operation, and no durable target or realization record says what the measurement permits.

## What was measured, and what it eliminates

**Measurement — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md), on an Apple M4 Max under macOS build `26A5388g`.** The source audit at dispatched base `ebceb731539597530ad015c22a209c9b23e24eac` found at least three independent incompatibilities with the registered `tiler::strict-tensor-contraction-f32@1` operation:

1. **It delivers a fused multiply-add.** On the `contraction_pair` case the scalar kernels return the separately rounded `0x3fc58f9e` and `contract_simdgroup`, compiled in the same module under `-fmetal-math-mode=safe -ffp-contract=off`, returns the fused `0x3fc58f9d`. This is [finding 16](../docs/research/apple-targets/numerical-behaviour.md) at a new construct: the flag is a defence against the compiler contracting a written pair and is no defence against a fused operation the source asked for. ADR 0015's contraction permission is Forbidden under both the strict and flush-to-zero contracts.
2. **It seeds its accumulator at `+0.0`.** On `negative_zero_seed` it returns `0x00000000` where the strict fold returns `0x80000000`. The spike source initializes the matrix accumulator with `0.0f`; the registered operation instead says `none-the-accumulator-starts-at-the-first-product`. No production contraction node can currently declare the proposed `initial = +0.0`, so this is a different operation rather than a target realization of the current one.
3. **It cannot establish the registered NaN boundary.** The current operation requires canonical NaN after every combine and at the result boundary. As the L3 record explains under `D-8`, `simdgroup_multiply_accumulate` does not expose a site at which the implementation can canonicalize after every contributor; an observed canonical result at the instruction boundary cannot prove that obligation was met internally.

A separate structural limitation: it refuses `M = 1` and `M = 10` on its own `M, N, K` multiple-of-8 precondition, which are the workload's decode and C1 shapes, so the decode path would need output-side `M`-padding.

**Measurement — the attribution, and its exact strength.** Over the eight-case corpus the kernel is consistent with exactly one of twenty-two named topologies, `fma_zero_seed_fold+ftz` — a fused left fold over a `+0.0`-seeded accumulator — with the other twenty-one refuted and the refuting case named for each. The retained inputs, environment, manifest digests, observations, and attribution table independently preserve that result for F32 at `K = 16` on an Apple M4 Max, macOS `27.0 26A5388g`, SDK `26.5 25F70`, Xcode `26.6 17F113`, and offline Metal compiler `32023.883`, using MSL 4.0 safe/precise mode with contraction off. It is exhaustive elimination over that finite candidate set, not a proof of the instruction's unpublished contributor order, intermediate precision, or per-combine NaN behaviour. Apple specifies the matrix operation's result only as `d = a * b + c`; it does not publish those internal properties. Under this repository's evidence classes, widening the corpus can strengthen the empirical record but cannot turn it into a universal hard guarantee.

## Source-audit correction and decision boundary — 2026-08-10

The former Qualification branch assumed a versioned target fact stating contraction permission and `+0.0` seed would be enough to make this realization caller-selectable. That is false at the current tree. Target honourability cannot substitute for the operation contract under [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md), and the sole registered keyed contraction family under [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) is unseeded and carries the per-combine canonicalization rule.

**Review correction — 2026-08-10.** The retained `t_prefill_mlp_512` minima are 3733.667 µs for simdgroup and 838.750 µs for the library route; `3733.667 / 838.750 = 4.451465872`, which rounds to the 4.5x endpoint above.

Qualification therefore requires a consequential semantic expansion before any Metal implementation: a new keyed operation (or accepted revision) that defines the fused per-contributor step, explicit `+0.0` seed, permitted result/order and NaN behaviour; matching reference semantics and validation; a public dtype-keyed target-realization vocabulary; and coherent target-profile, delivered-artifact schema/domain/version, cache/artifact identity, rejection, and conformance population. The retained record can supply bounded empirical provenance for that work, but not the missing normative order or canonicalization guarantee. Tom retains that public, identity, and research-to-implementation decision.

## The question for Tom

Should this ticket refuse `simdgroup_multiply_accumulate` specifically as a realization of the current `tiler::strict-tensor-contraction-f32@1`, or authorize research toward a distinct seeded, fused contraction operation and its identity-bearing realization surface?

- **Refuse it for the current operation (recommended).** Record the three incompatibilities and the unpublished-order evidence boundary once, keep the production route unavailable, and reproduce the two retained observations in the owning check. This says nothing universal about a future operation with different semantics. The strongest counterpoint is that it leaves the fastest hand-written kernel unusable even for callers that might accept fusion and a `+0.0` seed, and a later operation-design ticket may revisit the same evidence.
- **Authorize a distinct operation and realization design.** First define its seed, contributor-order/result-set, precision, NaN, validation, reference, target-fact, artifact, identity, and rejection contracts; then decide whether bounded Apple9 evidence is sufficient to admit an empirical realization. The strongest counterpoint is that the vendor leaves the correctness-bearing order and internal precision unpublished, so this could create a large public and identity surface without a target guarantee strong enough to execute it safely.

**Recommendation:** refuse only the current registered operation. All three mismatches are source-established now, and the bounded empirical record cannot discharge its hard contract. This preserves the measurement and leaves a future, explicitly different operation open without making the broader and unsupported architectural claim that simdgroup matrix contraction can never be declared.

A widened corpus remains useful research: eight cases at `K = 16` on one GPU are the entire evidence base, and no case yet separates an intra-tile reordering from an ascending fused fold. It is not a route to a universal guarantee by itself.

## Non-goals

Making it the default. `M`-padding. Any claim about another Apple GPU family, another dtype, or the runtime-compilation path, none of which the spike exercised.

## Closes when

Tom chooses one branch. The route is then either designed as a distinct keyed operation with complete public, target, artifact, identity, reference, and conformance consequences, or refused specifically for the current registered operation with its reason recorded. The `contraction_pair` and `negative_zero_seed` observations are reproduced by whatever check the accepted outcome installs.

## Source-first corrections — 2026-08-12

- **False host attribution repaired.** The four-cell timing ratios were measured on the M3 Pro/macOS `26A5378n` timing host. The M4 Max/macOS `26A5388g` record owns the retained correctness observations and finite topology attribution. They are separate evidence populations.
- **Incomplete shape statement repaired.** The simdgroup path refuses both the `M = 1` decode shape and the `M = 10` C1 shape because its retained kernel requires `M`, `N`, and `K` to be multiples of eight.
- **Imprecise implementation state repaired.** Production already fails closed structurally: the strict contraction lowering writes a first-product seed and separate multiply/canonicalize/add/canonicalize steps, and its Metal regression forbids `simdgroup`, `multiply_accumulate`, `fma`, and `mad`. What remained undecided was the durable realization classification and its evidence record, not whether the current emitter silently selected the instruction.
- **Unsafe evolution wording repaired.** `tiler::strict-tensor-contraction-f32@1` may not be revised in place to acquire seeded fused semantics. Any such semantics require a distinct `OpKey`, reference meaning, validation, and identity.
- **Overbroad schema consequence repaired.** A future distinct semantic operation does not by itself require new fixed artifact realization fields. Its reached semantic and schedule identities already move. Artifact schema widening is owed only if that future design introduces a genuinely new delivered physical fact.

## Accepted decision — 2026-08-12

Tom accepted the recommended refusal in this thread. `simdgroup_multiply_accumulate` is not a realization of `tiler::strict-tensor-contraction-f32@1` because it is fused where the operation requires separate rounding, starts from `+0.0` where the operation starts from the first product, and cannot establish the operation's per-combine NaN boundary or unpublished internal order/precision. The finite eight-case, twenty-two-topology record remains bounded empirical evidence and cannot discharge those hard obligations.

The current operation and production path remain unchanged, so this decision itself moves no semantic, schedule, artifact, cache, schema, or identity domain. [Pin the strict refusal](pin-the-strict-contraction-simdgroup-refusal.md) owns the durable typed classification and retained-observation regressions. [Research a distinct seeded fused operation](research-an-explicit-seeded-fused-contraction-operation.md) is deferred until a caller need and evidence capable of defining portable reference semantics both exist; no target-specific opaque “whatever Apple does” operation is authorized.
