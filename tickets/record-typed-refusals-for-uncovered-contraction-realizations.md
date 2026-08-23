---
id: record-typed-refusals-for-uncovered-contraction-realizations
title: Record typed refusals for uncovered contraction realizations
status: done
priority: p2
dependencies: []
related: [realize-the-attention-contractions-on-metal, admit-reassociated-contraction-schedule-alternatives, qualify-the-simdgroup-matrix-contraction-realization]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, explainability, contraction]
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

<<<<<<< HEAD
## Coordinator correction, 2026-08-22 — two errors in my own re-audit above, both caught by `worker-decline`

**Error 1 — I supplied an anchor I never ran, and it fails as false absence.** My re-audit says to quote the enum doc *"at the anchor `it can also say what it deliberately withheld`"*. The retired wording is kept so its count cannot shrink. That anchor returns **0**: a `///` line break splits the sentence, with L1308 ending `…complete only if it` and L1309 beginning `can also say…`, and my fragment straddles the break. The shortest resolving fragment is `can also say what it deliberately withheld`, which returns **1**. This is the exact hazard AGENTS.md records under `An anchor copied from the rendered view fails as absence, which is the dangerous direction` — committed by the coordinator, in a brief, after briefing four separate workers about it this same session. The obligation the coordinator section states is unambiguous: *run the grep yourself before handing it to anyone*. I did not.

**Error 2 — the method is `reason()`, not `tag()`.** My re-audit says `tag()` spells the two causes. There is no `tag()` on `StrategyDeclineCause`; the method is `pub const fn reason(self) -> &'static str`. I read the match arms and named the wrong function around them.

Neither error changed the outcome, because the worker read the source instead of trusting the brief. That is the intended failure mode, not a reason to relax the obligation.

## Coordinator verification of the lane's scope finding, 2026-08-22

The lane recorded **three** declines rather than the ticket's four, on the reading that `opaque_mps` is structurally unreachable by the governed provider rather than a withheld dimension. Corroborated at `dbf1cd98`: `OpaqueCallRegistry` is declared `pub(crate)` in `crates/tiler-compiler/src/call_registry.rs` and appears nowhere in the frontier or physical-provider path, so that strategy is not one the governed provider can consider and decline. Recording a decline for it would have named a ground that does not apply — the outcome this ticket exists to prevent, inverted.

The lane also reports that this ticket's own User-visible outcome is imprecise: *"reassociation, permutation, or absent distributivity"*, preserved verbatim here. Distributivity grounds none of the three; it is consumed by a contraction-order rewrite this build never enumerates. The third ground is ADR 0015 contraction. **Left as written rather than repaired in place**, because it is the outcome sentence and rewriting it is a scope change; the accurate three-ground list is in the lane's own table and in the landed tests.
=======
## Worker audit and repair at `2b179263`, 2026-08-22 — `worker-decline`

Per-Fact verdicts, each from the file read in full at this base.

**Fact 1 — verified.** `crates/tiler-compiler/src/frontier.rs` spelled the contraction arm `crate::physical::RegionSpellingKind::Contraction => (` followed by `contraction_region(request, producer, subject.write()).0`, with the comment `splitting it would consume the reassociation this` and no `DeclinedStrategy` beside it. **That comment anchor is pre-change and this commit replaces it**, so a grep for it at the tip correctly returns 0; the arm is now found by `contraction_alternatives = contraction_declines(request)`. The two model sites the coordinator named are real and were copied: both spell `StrategyDeclineCause::UnspellableRegion` under `PUBLISHING_COPY_COMPOSITION_RULE`, reached through `GovernedSpelling`'s `parallel` field and folded into the offer by `.fold(ProviderOffer::proposing(proposals), ProviderOffer::decline)`.

**Fact 2 — verified in substance, imprecise in one name.** `pub enum StrategyDeclineCause` does carry `#[non_exhaustive]`, `NumericalPermissionRefused { dimension: &'static str }`, and `AlgebraicCapabilityUnsupported`, and the design-intent doc is there. **The coordinator's anchor for it does not resolve, and the cause is the one AGENTS.md documents.** `it can also say what it deliberately withheld` returns 0 against `crates/tiler-compiler/src/frontier.rs` because a `///` line break splits it: line 1308 ends `the enumeration is complete only if it` and line 1309 begins `can also say what it deliberately withheld`. The shortest fragment that resolves is `can also say what it deliberately withheld`, which returns 1. This is a supplied command that had never been run — the failure reads as *the text was removed* when the text is plainly there. **The method is `reason()`, not `tag()`.** The ticket and the coordinator re-audit both say "`tag()` spelling them"; there is no `tag()` on this type. `StrategyDeclineCause::reason()` returns `"numerical-permission-refused"` and `"algebraic-capability-unsupported"`; the only `tag`-like thing is the private `encode`'s leading byte, `0x01` and `0x06`. No widening was needed and none was made.

**Fact 3 — verified for three of the four realizations, and *wrong in scope* for the fourth.** The L3 record supports the grounds it states for `ksplit_contiguous`, `ksplit_strided`, and `simdgroup`. The `ksplit_strided` claim in particular holds and is load-bearing exactly as the coordinator said: the record's attribution table gives `ksplit_contiguous` → `contiguous_split+ftz` uniquely and `ksplit_strided` → `strided_split+ftz` uniquely, each with the other's twenty-one topologies refuted, and states `this candidate is the measured demonstration that the two are different plans and not one`. Collapsing those two declines would therefore be wrong, and the negative control below is built on that.

**The repair — `opaque_mps` is not a strategy this provider can decline, and the record already routed it elsewhere.** Three independent readings agree, and none of them is a preference:

- `ImplementationContext` carries `request`, `subject`, and a `baseline` cell and **no** `OpaqueCallRegistry`, so `GovernedPhysicalProvider` cannot name an opaque call. `StrategyDeclineCause`'s own doc defines a decline as `Why a provider considered a strategy for this subject and did not offer it`; a decline naming an opaque provider would report considering a strategy this provider has no way to consider.
- The refusal is a different mechanism. The record's ground is that a provider `cannot state its accumulation order at all cannot refine any contract` — a missing *numerical guarantee*, checked where an opaque proposal is admitted, not a withheld dimension decided from the request.
- The L3 record already assigned it. Its delivery table states `nothing is filed for `opaque_mps` as a realization`, and [`exercise-opaque-admissions-downstream-of-the-frontier`](exercise-opaque-admissions-downstream-of-the-frontier.md) records the evidence with `The L3 elimination is an instance of that existing rule and needs no new mechanism`, handing the remaining test gap to [`implement-opaque-physical-call-providers`](implement-opaque-physical-call-providers.md).

**So this ticket delivers three declines, not four, and no public surface was added.** This is the coordinator's "stop and report" case reached from the other direction: the fourth ground is not inexpressible in a way that needs a new `StrategyDeclineCause` variant — it is expressible, by the existing opaque-admission path, in a ticket that owns it.

**A second Fact-3 imprecision, in the outcome's own wording.** The User-visible outcome enumerates the explanations as "reassociation, permutation, or absent distributivity", but distributivity grounds **none** of these realizations. `crates/tiler-compiler/src/policy.rs` states the rule: `Distributivity, which a contraction-order rewrite would consume, is` absent rather than withheld. A contraction-order rewrite is the consumer, and this build enumerates none. The three grounds actually delivered are reassociation, permutation, and **ADR 0015 contraction** — the last being a fourth explanation the outcome's list omits. `the_algebraic_maxima_these_declines_assume_are_the_registered_ones` asserts `!descriptor.distributivity_supported()` so that admitting a contraction-order rewrite later finds this note rather than reading the absent decline as an oversight.

### What landed

Three declines from `contraction_declines` in `frontier.rs`, each naming its own ground, and each carrying the cause variant that says *which* of ADR 0014's two facts is missing rather than collapsing them:

| Strategy | Cause | Dimension | Source of the missing fact |
| --- | --- | --- | --- |
| `tiler.contraction.contiguous-k-split` | `NumericalPermissionRefused` | `numerics.reassociation` | the caller's resolved ceiling |
| `tiler.contraction.strided-k-split` | `AlgebraicCapabilityUnsupported` | `numerics.permutation` | the operation's declared maximum |
| `tiler.contraction.simdgroup-matrix` | `AlgebraicCapabilityUnsupported` | `numerics.contraction` | the operation's declared maximum |

Confirmed reaching a caller through the ordinary compile path: the rendered explain of `projection(2, 2, 3)` under `STRICT_F32` carries three `frontier.strategy-decline.v1` records and `rejected-count:count=3`, and the two sources land in **different explain stages** — `capability-resolution` for the algebraic pair, `numerical-legality` for the reassociation one.

**Identity does not move, rederived rather than assumed.** `StrategyDeclineCause::encode` is reached only through `FrontierRejection::encode`, whose only non-test caller is `encode_rejection`, whose only non-test use is `rejections.sort_by_key(encode_rejection)` — a deterministic sort key, not an identity. `encode_proposal_identity` never encodes a decline. The full workspace gate reproduced every pinned identity, golden, and conformance value unchanged.

**The simdgroup explain assertion in `crates/tiler-compiler/tests/contraction_direct_path.rs` was narrowed, deliberately.** It banned the substring `simdgroup` from the whole render on the premise, quoted here from the base and **replaced by this commit**, that `The explain and retained-plan surfaces therefore stay silent` about that construct — so that anchor too returns 0 at the tip by design. A banned substring cannot separate a *formed* candidate from a *withheld* one, and this ticket's outcome requires the withheld one to be named. The five construct spellings — `multiply_accumulate`, `fma(`, `metal::fma`, `precise::fma`, `mad(` — stay banned everywhere; the retained-alternative identity and selected-capability surfaces still refuse `simdgroup` outright; and the render may now spell it on exactly one line, which must be a `frontier.strategy-decline.v1` record naming `tiler.contraction.simdgroup-matrix` under `algebraic-capability-unsupported`. That is a stricter assertion than the substring ban it replaced: it pins where the word may occur, not merely that it does not.

`tiler.contraction.` was added to `ADMITTED_NON_DOMAIN_PREFIXES` in `crates/tiler-compiler/src/domains.rs`; a strategy name separates no canonical byte subjects, exactly as the neighbouring `tiler.reduction.` does. `every_tiler_spelled_literal_is_pinned_or_classified` failed first and named the requirement.

### The negative control, and every perturbation

The control is `granting_reassociation_retires_only_the_contiguous_split_decline`. It first asserts the contiguous decline **is** present under `STRICT_F32`, so it cannot pass vacuously, then asserts it is gone under `REASSOCIATE_F32` while the other two stand unchanged. Each decline was perturbed separately, subject and never assertion:

- **Contiguous decline deleted.** `granting_reassociation_retires_only_the_contiguous_split_decline` failed with `the strict contract did not decline the contiguous split at all` — the non-vacuity guard, firing. `each_uncovered_contraction_realization_is_declined_under_its_own_ground` also failed, its left value missing the `contiguous-k-split` row.
- **Strided decline gated on the reassociation ceiling** — that is, collapsed onto the contiguous split's ground. `granting_reassociation_retires_only_the_contiguous_split_decline` failed with `granting reassociation changed a decline it does not reach`. The census test *passed* under this perturbation, which is the separation worth recording: the control, not the census, is what carries the L3 record's two-plans claim.
- **Matrix decline reading the caller's ceiling instead of the operation maximum.** `a_contraction_permitting_ceiling_still_withholds_the_matrix_realization` failed with `a contraction-permitting ceiling changed which fact withholds the matrix realization`, and the narrowed explain assertion failed with `the one line naming simdgroup is not a decline under the algebraic cause — it is missing algebraic-capability-unsupported`.
- **Registered permutation maximum widened to `permission-gated`.** `the_algebraic_maxima_these_declines_assume_are_the_registered_ones` failed at the decode, `InvalidGovernedContractionDescriptor { source: UnsupportedValue { field: Reduction(AttributeFieldId(5)) } }`.
- **Decoder widened as well, to admit and return that row.** The same test then failed at the assertion itself, `left: PermissionGated, right: Unsupported`.

**One honest limit on that last check, stated in its own doc comment.** `ContractionF32ReductionDescriptor`'s decoder returns `Unsupported` for permutation and signed-zero as literals rather than from the decoded row, and `arithmetic_contraction_supported` and `distributivity_supported` are `const fn`s answering `false`. So three of that test's four assertions cannot fail on a change to the registered facts alone — the decode is the load-bearing guard today, and the assertions go live only once a key generation widens the decoder too, which is the moment they matter.
>>>>>>> 0f963451
