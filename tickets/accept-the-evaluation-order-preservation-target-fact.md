---
id: accept-the-evaluation-order-preservation-target-fact
title: Accept the evaluation-order-preservation target fact
status: awaiting-decision
priority: p2
dependencies: []
related: [declare-evaluation-order-preservation-in-the-target-profile, measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order, admit-a-refutation-only-derived-bound-conformance-oracle]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [public-boundary, targets, numerics, needs-tom]
---
## The decision

Tom accepts or amends the exact public surface `declare-evaluation-order-preservation-in-the-target-profile` landed as a labelled draft in `tiler_compiler::target`. Filed at `awaiting-decision`: only Tom closes an acceptance ticket. ADR 0075 reserves a new public declared-fact family for him, and `crates/tiler-compiler/src/target.rs`'s module header already says in its own words that this family is *not* covered by the 2026-07-30 acceptance at `4ad5a2e`.

## The exact included surface

- `BackendArithmeticLicence` — `Withheld` / `Granted`, plus `key()`.
- `EvaluationOrderPreservation` — `Preserved` / `NotPreserved`, plus `key()`.
- `EvaluationOrderResolution` — `Preserved` / `NotPreserved` / `Deferred { available_at }` / `Unknown`.
- `TargetProfileBuilder::declare_evaluation_order_preservation` and `::declare_measured_evaluation_order_preservation`.
- `TargetProfile::evaluation_order_preservation(&subject, licence, available_phase)`.
- `TargetProfileBuildError::DuplicateEvaluationOrderPreservation { licence, phase }`.

## The exact excluded surface, and why each exclusion is deliberate

- **No math-mode spelling.** `safe`, `relaxed`, and `fast` are one backend driver's option tokens, and a consumer-agnostic profile that named them would have learnt a Metal flag. The key is the *licence* instead, which is what [finding 34](../docs/research/apple-targets/numerical-behaviour.md) attributes the behaviour to: the reordering fires exactly where the emitted operations carry LLVM's `reassoc`, and `relaxed` and `fast` differ only in `nnan`/`ninf`, which no measured cell attributes an order change to. **The consequence to weigh:** the two licence values cover all three modes today and a measurement separating `relaxed` from `fast` would need a third value here — a build error at every match rather than a silent inheritance, but a source change nonetheless.
- **No twelfth numerical dimension.** This is not a caller-contract dimension. `CANONICAL_DIMENSIONS` states what a caller's contract may grant *Tiler*; this states what Tiler's emission grants the *backend translator*, and the two are different subjects that ADR 0011's non-implication rule keeps apart. Adding it to the dimension list would have made it grantable by a contract, which it is not.
- **No `Unknown` variant on `EvaluationOrderPreservation`.** Absence is the `Unknown`, exactly as it is for `DTypeDispatchability` and for the synchronization family. A profile that never declared the property and one that declared "I don't know" would otherwise be two spellings of one state.
- **No feasibility consumer.** The fact is declared and resolvable; nothing admits or refuses on it yet. Its consumer is the [oracle derivation](../docs/research/reference/permitted-divergence-oracle.md)'s refusal class 3, whose derived oracle is filed `deferred` at `admit-a-refutation-only-derived-bound-conformance-oracle`.

## What a reader should check before deciding

- The identity consequence is **nil for every profile that exists**: the row family writes its bytes only when it holds a row, so the governed baseline, the bound macOS Metal declaration, and every test profile encode byte for byte what they encoded before. `complete_descriptor`'s header carries the derivation and `the_declared_profile_states_one_barrier_realization`'s 1,999-byte pin is the check. `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` stays at `v11`.
- The first production row is **not** declared. The macOS compile profile answers `Unknown`, because finding 34 was measured on a neighbouring offline compiler build (`metalfe-32023.921`) and every row that profile carries was measured under `metalfe-32023.883`. The authority ledger's "Evaluation-order preservation" section states the deferral and the two closing measurements, both of which move a host toolchain component.

## What closes this ticket

Either accept the surface as landed, or record the requested amendment here. Amendment costs a rename or a reshape in `crates/tiler-compiler/src/target.rs` and its two consumers' tests; no wire bytes and no artifact identity move either way, because no profile declares a row yet.
