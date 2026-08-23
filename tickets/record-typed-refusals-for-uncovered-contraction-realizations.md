---
id: record-typed-refusals-for-uncovered-contraction-realizations
title: Record typed refusals for uncovered contraction realizations
status: in-progress
priority: p2
dependencies: []
related: [realize-the-attention-contractions-on-metal, admit-reassociated-contraction-schedule-alternatives, qualify-the-simdgroup-matrix-contraction-realization]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, explainability, contraction]
claimed_from: todo
assignee: worker-decline
lease_expires_at: 1787454809
---
## User-visible outcome

A caller who asks why a contraction was realized by the direct fold and not by a split, a matrix instruction, or an opaque provider gets a typed decline naming *which* explanation applies — reassociation, permutation, or absent distributivity — rather than an absence.

## Why this exists

Filed 2026-08-22 by `worker-attention` as the enumerated remainder of [`realize-the-attention-contractions-on-metal`](realize-the-attention-contractions-on-metal.md), whose Required delivery asks for *"a refusal for every realization whose reduction topology is unstated or uncovered, naming reassociation, permutation, or the absent distributivity separately, because those are three different explanations."*

**Fact — the contraction arm records no decline at all today.** `govern_spelling`'s contraction case in `crates/tiler-compiler/src/frontier.rs` offers `contraction_region` and adds no parallel strategy, with the comment that splitting would consume the reassociation this family declares forbidden. That reasoning is correct and is exactly what should be *reported* rather than left implicit. Re-derive at your base.

**Fact — the vocabulary to say it already exists and needs no widening.** `StrategyDeclineCause` on the public `#[non_exhaustive]` enum already carries `NumericalPermissionRefused { dimension }` and `AlgebraicCapabilityUnsupported { dimension }`, which is the three-way split the ticket asks for without a new variant. Verify before adding anything public.

**Fact — the four uncovered realizations and their distinct grounds**, from the [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md): `ksplit_contiguous` needs reassociation; `ksplit_strided` needs reassociation *and* permutation, and is the measured demonstration that the two are different plans and not one; `simdgroup` delivers a fused multiply-add where ADR 0015's contraction permission is Forbidden *and* seeds its accumulator at `+0.0` where the profile declares no seed; `opaque_mps` is refuted against all twenty-two named topologies and cannot state its accumulation order at all.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict.
- Record one decline per uncovered realization, each naming its own ground. **Do not collapse them**: a caller told only "numerical permission refused" cannot tell a split from a matrix instruction.
- Prefer the existing decline vocabulary. If a new variant is genuinely required, that is a public-surface change and needs its own justification.
- One negative control: under a contract that *does* grant reassociation, the contiguous split's decline must change or disappear, so the decline is a function of the contract rather than a constant.
- Perturb each decline separately, subject not assertion, with quoted failure text.

## Non-goals

Offering any of these realizations — `admit-reassociated-contraction-schedule-alternatives` and `qualify-the-simdgroup-matrix-contraction-realization` own those; and the tiled alternative's own decline, which belongs to its offer ticket.

## Closes when

Each uncovered contraction realization records a typed decline naming its own ground, the three explanations stay separately named, the contract-sensitivity control holds, and the workspace gate is green.

## Coordinator re-audit at `d19c3b40`, 2026-08-22 — both checkable Facts verified, with the model sites named

Run and read by the coordinator at this base before dispatch. Contradict any of it with evidence rather than deferring.

**Fact 1 — verified.** `govern_spelling`'s contraction arm is `crate::physical::RegionSpellingKind::Contraction => (crate::physical::contraction_region(request, producer, subject.write()).0, …)` in `crates/tiler-compiler/src/frontier.rs`. It offers the region and records no decline beside it.

**Fact 2 — verified, and no widening is needed.** `pub enum StrategyDeclineCause` carries `#[non_exhaustive]` and already declares `NumericalPermissionRefused { dimension: &'static str }` and `AlgebraicCapabilityUnsupported { dimension }`, with `tag()` spelling them `"numerical-permission-refused"` and `"algebraic-capability-unsupported"`. The enum doc states the design intent this ticket is executing, at the anchor `it can also say what it deliberately withheld` — quote it in your reasoning rather than re-deriving it.

**Two model sites, which the ticket does not name.** A third variant, `UnspellableRegion`, is already recorded from this same function at two places in `frontier.rs`, and the recording mechanism is `.decline(DeclinedStrategy::new(strategy, cause))` with `DeclinedStrategy::new` a `const fn` taking a `&'static str` strategy name and a cause. **Copy that shape.** Reading those two sites first will show you how a decline reaches the caller and what the strategy-name convention is, which is cheaper than inferring it.

**On the public boundary.** `StrategyDeclineCause` is `pub` and `#[non_exhaustive]`, so adding a variant is not a breaking change for downstream matches — but it is still a public-surface addition and AGENTS.md reserves those to Tom. The Facts above say you should not need one. If your reading says otherwise, **stop and report** with the evidence rather than adding it; a fourth ground that the existing two cannot express is a genuine finding worth escalating, not a detail.

**Fact 3 is not verifiable from source and is not the coordinator's.** The four realizations and their grounds come from the L3 record `docs/research/scheduling/first-metal-contraction-realizations.md`. I did not re-derive them. Read that record in full and give it its own per-Fact verdict — in particular check that `ksplit_strided` really is the measured demonstration that reassociation and permutation are two plans rather than one, because that claim is what makes collapsing the declines wrong.

**On the negative control.** The Required work asks that under a contract granting reassociation, the contiguous split's decline changes or disappears. Before trusting it, state what would make it say *no* and confirm that case is reachable — a control that cannot fail is the recurring defect on this repository. Perturb the subject (the contract, the permission read), never the assertion, and quote the failure text.
