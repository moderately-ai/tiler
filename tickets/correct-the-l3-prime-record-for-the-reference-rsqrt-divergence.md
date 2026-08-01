---
id: correct-the-l3-prime-record-for-the-reference-rsqrt-divergence
title: Correct the L3-prime record for the reference rsqrt divergence
status: in-progress
priority: p1
dependencies: []
related: [admit-the-rms-normalization-family, implement-parallel-reduction-strategies, design-model-level-qualification-and-optimization]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, normalization, transcendental, correction]
claimed_from: todo
assignee: worker-l3-correct
lease_expires_at: 1785580761
---
## User-visible outcome

The L3′ derivation records what its own retained measurement of `torch.rsqrt` actually shows, and decision **D-3**'s entry records that it closed — so the next reader of that record is not told an open question is open, or that a measured value is the reference.

## The correction, and how to reproduce it

**Measurement — reproduced, not asserted.** The [reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md) records `rsqrt_of_eps_alone` as `0x4479ffff`, from `torch.rsqrt(torch.tensor(1e-6, dtype=float32))`. The argument is the binary32 rounding of `1e-06`, payload `0x358637bd`, whose exact value is `9.999999974752427e-07`. The exact reciprocal square root of that value is `1000.00000126237864845…`, so the two binary32 values bracketing it are `0x447a0000` (exactly `1000.0`) and `0x447a0001`, and the correctly rounded value is `0x447a0000`.

`0x4479ffff` is one step *below* that pair — about `1.02` ULP from the exact reference — so it is not correctly rounded and not faithful either. It is exactly what the two-rounding composition `f32(1 / f32(sqrt(t)))` delivers at this argument, which is the spelling the derivation's own "`rsqrt`, not `1 / sqrt`" sentence exists to exclude.

The reproduction is three lines of exact arithmetic and is checked in Rust at `crates/tiler-reference/src/rms_norm/tests.rs::the_certified_reciprocal_square_root_separates_rsqrt_from_one_over_sqrt`, which asserts both values and their difference.

**Consequence — it propagates to a second recorded row.** The probe's `rms_subnormal_vector` is `0x02081cb9`. The squares of `1e-40` underflow to exactly `+0.0`, so that row's reciprocal square root argument is `eps` alone; with the correctly rounded scale the row is `0x02081cba`. The one-step difference is entirely the `rsqrt`.

## What the record should say, and what it should not

The derivation's RMS normalization table currently reads the zero row's measurement as if it were the reference: "`rsqrt(0 + 1e-6)` is `0x4479ffff` (≈ 999.99994, not 1000)". That sentence is a correct *measurement of one implementation* and is being read as the normative value. It needs the boundary restated rather than the number changed — the measurement stands, its class does not.

**Non-goal — do not change the pinned formula.** Nothing here suggests Tiler should reproduce `torch.rsqrt` bit for bit. `tiler::rms-norm-f32@1` states a `Faithful` contract derived from Metal's Table 8.1 and §8.2, and the reference model's value falls outside it; that is a finding about the reference model, and the model-level bound it feeds is [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md)'s, not this ticket's.

## Also owed by this ticket

- **D-3 closed.** `admit-the-rms-normalization-family` settled it as *define, not refuse*, with the elimination stated and [`scope-a-value-domain-precondition-for-squaring-overflow`](scope-a-value-domain-precondition-for-squaring-overflow.md) carrying the deferred capability. The derivation's "Unresolved decisions" entry still says it closes when the key is admitted; the key is admitted.
- **D-4's `rsqrt` half.** The record says every Table 8.1 entry the three verticals need is unadoptable without a derivation. The reciprocal square root's derivation now exists: correctly rounded under either mode §8.2 admits is exactly the faithful pair, so the entry is adoptable as `AccuracyContractForm::Faithful` and needs no metric reconciliation at all. Gap 1 and Gap 4 bind disjoint halves and only Gap 1's half needed a registered implication.
- **The support-matrix cross-reference.** `docs/roadmap.md` already carries the normalization's own row; the derivation's "Consequences for the ladder" section still says nothing moved.

## Closes when

The derivation states the measurement's class correctly, records D-3's answer and its elimination, records what D-4's `rsqrt` half now supports, and the `informs`/catalog metadata still agrees with the corpus.

## Outcome

All three obligations are met in `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md`, and the sweep the ticket required found six further sentences the SiLU and RMS landings falsified — one of them a wrong *mechanism* rather than a stale status, which is the reason the sweep was worth its cost.

### 1. The `rsqrt` measurement's class

The zero row and the subnormal row of the RMS exceptional-value table are re-labelled as measurements *of `torch.rsqrt`*, and a new subsection, **Correction — the reciprocal square root of `eps` is a measurement of one implementation, and this record read it as the reference**, carries the derivation. It quotes what the record previously said, states that the measurement stands and its class does not, and gives the exact arithmetic: the argument is `0x358637bd` (exactly `9.999999974752427e-07`), the exact reciprocal square root is `1000.00000126237864845…`, the correctly rounded value is `0x447a0000`, and the faithful pair is `(0x447a0000, 0x447a0001)`. `0x4479ffff` is one step below the pair, about `1.02` ULP out, and is exactly `f32(1 / f32(sqrt(t)))` — the composition the pinned formula's `rsqrt` choice exists to exclude.

Cited reproductions, both verified present before citing: `the_certified_reciprocal_square_root_separates_rsqrt_from_one_over_sqrt` and `the_registered_faithful_contract_admits_the_pair_and_refuses_the_third_value`, in `crates/tiler-reference/src/rms_norm/tests.rs`. The second is what makes this a finding rather than a rounding preference — `0x4479ffff` and `0x447a0002` both `Violate` while both pair members `Conform`.

The propagation to `rms_subnormal_vector` is recorded (`0x02081cb9` against the certified `0x02081cba`, the whole difference being the `rsqrt`, because the squares of `1e-40` underflow so both rows share `eps` as their argument). The two-wide worked example is stated as **unaffected** — its argument `0x41480001` is non-discriminating and the certified reference reproduces `0x3e90d0c2` and both output bits — so a reader does not assume every retained bit pattern moved.

The non-goal is stated explicitly: the pinned formula is unchanged, nothing suggests reproducing `torch.rsqrt` bit for bit, and the finding feeds `design-model-level-qualification-and-optimization`'s model-level bound rather than a per-operation tolerance.

### 2. D-3 closed

The D-3 entry is now **CLOSED 2026-08-01: define, not refuse**, carrying the three-route elimination rather than only the conclusion — construction sees no element values; no operand carries a proved bound; a runtime scan is a costed second pass over 144,384·`T` contributors needing either 113 host readbacks per forward pass or a device-side validation mechanism the bounded profile lacks. The "either way" corpus condition the entry itself set is recorded as met. `scope-a-value-domain-precondition-for-squaring-overflow` is named as owning the deferred capability, with the distinction that the deferral is a missing mechanism rather than a missing decision. The Large-row table cell that pointed at D-3 as open now points at the closure.

### 3. D-4's `rsqrt` half

The D-4 entry gains four paragraphs recording that both derivations were performed on disjoint halves, exactly as the accuracy record predicted. The `rsqrt` half closed by **deriving a form rather than adopting a number**: Table 8.1 states correctly rounded, §8.2 admits either rounding mode, a correctly rounded result under either mode is a member of the faithful pair and both members are reachable, so the promised set *is* `AccuracyContractForm::Faithful` — tight, not conservative. Both load-bearing consequences are recorded: `CorrectlyRounded { NearestTiesToEven }` would be a stronger claim than §8.2 supports and would be *admitted* rather than rejected because `refines` proves correctly-rounded-satisfies-faithful, and a faithful contract is metric-free so **Gap 1 does not bind this entry and no second `ScaledMetric` row was registered**. `the_normalization_needs_no_registered_implication_at_all` is cited as the executable form of the disjointness. The `Exp` half (Gap 1, factor 3, bound 12) is recorded alongside it, and what remains is stated: softmax needs `Exp` again, and Gap 3 is unmoved for every entry.

### The sweep — six further corrections, with old class → new class

| Where | Was | Now |
| --- | --- | --- |
| **Status** paragraph | Fact: "every family it names sits at R2 … this record moves no row" | Corrected — SiLU and RMS normalization are at **R5**; softmax, masking's `Select`, and the reductions row are unmoved. The record still moves no row itself; its tickets did |
| **SiLU exceptional values** | Measurement/Inference: the `-88.73` band's value "is a subnormal, and the qualified Metal row flushes subnormals to zero anyway" | **False in both halves.** No subnormal is produced anywhere in F32 SiLU — `silu(-88.7228)` is `0x82b173cc`, a normal value, and `silu(-88.73)` is already `-0.0`. The route is finite-overflow to `+inf`, then a finite negative over an infinity, exact by IEEE sign rules. The target conclusion inverts: with no flush involved, a *preserving* realization agrees too. What the band actually depends on is §8.1's INF guarantee, which is conditioned on fast math being disabled — a stronger obligation than the flush reasoning it replaces |
| **SiLU spellings** | Measurement: the two spellings "differ at exactly one" input | Corrected — true of those twelve inputs, not a property of the spellings. The registered corpus finds **three** of thirteen finite arguments: `0xc2b00000` by one ULP, `0xc2b17213` and `0xc2b17217` by three ULPs each. The record's own stated boundary is what this vindicates |
| **Reductions** | Fact: `tiler::strict-serial-sum-f32@1` holds "the sole `OrderedReduction` fusion role", so a prologue-carrying sum resolves to "no fusion legality at all" | Corrected for the prologue-carrying half only — `PrologueCarryingOrderedReduction` now exists. The maximum half stands, the rung did not move, and the reason is preserved: it is not a new *reducer* |
| **Metal feasibility** | Fact: "what is missing is on Tiler's side of the boundary" | Corrected — `UnaryOp::{F32Exp,F32Rsqrt}`, `BinaryOp::F32Divide`, and `PointwiseF32Node::Rsqrt` all exist and emit `precise::exp`/`precise::rsqrt`. What remains missing is the *maximum* reduction |
| **Consequences for the ladder** | Inference: "Nothing moved" | Preserved verbatim as the honest claim at delivery, then corrected with a six-row before/after table. Adds that the four-claim vocabulary *does* now apply to what landed: implemented support with a bounded corpus, R6/R7 unclaimed for both |

Three further paragraphs gained **Landed** notes where a proposal was registered as written rather than superseded — the RMS dtype signature and `eps`-in-identity, the SiLU division form, and the accuracy-contract vocabulary consumed twice without gaining a form. D-5 is marked **consumed for one of the two sums** (the normalization declares `tiler::f32@1` and its verifier refuses narrower; softmax's denominator is untouched), and the typed-refusals section gained a status paragraph distinguishing the refusals now implemented and perturbed from softmax's, and noting the elementary-accuracy refusal is deliberately not folded into the canonical target descriptor.

### Metadata

`implementation_status` moved `not-started` → `partial`, which is the value the corpus now describes and which no catalog view mirrors (checked: `docs/research/README.md`, `docs/README.md`, `docs/roadmap.md`). `disposition` was **left at `pending` deliberately** — see the reported remainder below. `informs`, `evidence_classes`, `research_status`, and `depends_on` are unchanged and still agree with the corpus.

### Reported, not absorbed — one remainder outside this ticket's scopes

`docs/research/README.md:53` mirrors this record's `disposition` as "pending" in the research catalog. With two of the three verticals landed, `partially-adopted` is the accurate value and is already in use elsewhere in the corpus (`mature-dtype-taxonomy`, `reduction-semantics-and-legality`, `region-accuracy-contract`). Changing the frontmatter without the catalog line would break the mechanical agreement this ticket's own closure condition requires, and `docs/research/README.md` is reached by no scope declared here (`research/numerics` is `docs/research/numerics/**` and `spikes/numerics/**`). It is a one-word change in each of two files and is reported rather than filed as a ticket, because a ticket would cost more than the change.

### Verification

`tkt lint` clean; `git diff --check` clean; `tkt guard --base 26266d9` exit 0; `make full` green on the committed tree (docs-only change, run to prove the crates untouched). The cited test name was verified to exist before citing it.
