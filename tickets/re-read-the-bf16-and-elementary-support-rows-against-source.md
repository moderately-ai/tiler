---
id: re-read-the-bf16-and-elementary-support-rows-against-source
title: Re-read the BF16 and elementary support rows against source
status: review
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-navigation-3
lease_expires_at: 1786031910
---
## The work (maturity audit 2026-08-06, key claims coordinator-verified: KernelConstant::Bf16Bits exists; the four named tickets are done)

BF16 row (`roadmap.md:477`): correct the three vocabulary facts the source refutes (KernelConstant/BinaryOp carry BF16; a BF16 VerifiedKernel exists and bit-agrees; a producer-built artifact round-trips), recount the live tickets from six to two (establish-bf16-optimizer-legality, conform-the-bf16-vertical-end-to-end), keep R4 while stating the non-monotone evidence above it. SiLU row (`:469`): the named wall (admit-the-registered-unary-families...) is done and ADR 0099 is accepted uncited — decide and state the evidence bar (R6 with unit-level emission evidence, or R5 naming the missing compiled golden) rather than leaving a closed ticket as the blocker. RMS-norm/softmax rows (`:470/:471`): replace the closed two-region ticket with the true blocker — no IndexRealizationLaw registered for either family (registry.rs:2391-2434) — and note the sequence surface acceptance. `roadmap.md:555`'s wall list names three done tickets; annotate or replace each.

## Closes when

Each row's claims reproduce against source, the evidence-bar decision is stated not silent, and no closed ticket is named as a live wall.

## Outcome — 2026-08-06

**Every claim below was read at source at base `afdac9c9`, and the row numbers in the section above had drifted by two navigation merges — each row was located by content instead.** The four support-matrix rows are at `docs/roadmap.md:469` (activation), `:470` (normalization), `:471` (softmax), and `:477` (reduced-precision floats), and the structural-limits paragraph is `:555`; the two catalog items and the one added mid-dispatch are in `docs/research/README.md`. No gate input was touched: the branch changes `docs/roadmap.md`, `docs/research/README.md`, and three files under `tickets/`.

### The BF16 row — three vocabulary facts flipped, the count recut, R4 held

All three of the row's stated absences are refuted at their construction sites, and each is corrected in tense because the *distinction* the sentence drew — a tag no value can be constructed under is a vocabulary, not a rung — is what the landings since have to be read against.

| Claim in the row | Check run | Verdict |
| --- | --- | --- |
| "`KernelConstant` and `BinaryOp` still carry no BF16 variant" | read `crates/tiler-ir/src/kernel/model.rs:312`–`:359`, `:420`–`:495`, `:1715`–`:1737` | **False.** `KernelConstant::Bf16Bits(u16)` at `:336`, encoded at tag `0x04` (`:1734`) where `F32Bits` is `0x03`; `BinaryOp::Bf16Add`/`Bf16Multiply` at `:428`/`:435`, tags `0x0a`/`0x0b` |
| "no `VerifiedKernel` can carry a BF16 buffer at all, because `verify.rs` derives every buffer's expected element type from the region's `ScalarProgram`, every arm of which is F32" | read `crates/tiler-ir/src/kernel/model.rs:172`–`:183` and `verify.rs:225`–`:267` | **False, and the stated mechanism is intact.** `region_element_type` answers `KernelType::Bf16` for `ScalarProgram::PointwiseBf16`; the derivation is the same single authority the row credits. `bf16_pointwise_kernel` (`crates/tiler-metal/src/tests.rs:496`) builds one |
| the BF16 value carried is unchecked | read `crates/tiler-compiler/src/pipeline/tests.rs:3279`–`:3340` | `a_bf16_kernel_agrees_with_the_reference_oracle_bit_for_bit` bit-compares ten witnesses against `ReferenceEvaluator::standard()` through an interpreter sharing no implementation with it |
| implied: no BF16 backend spelling | read `crates/tiler-metal/src/emit.rs:993`, `:1280`–`:1281`, `:1589`, `:1842`–`:1853`; `ls crates/tiler-metal/goldens/` | `msl_type` spells `bfloat`; `pointwise_scale_bias_bf16.metal` is one of nine goldens that compile and link |
| the producer wall recorded by the encoding ticket | read `crates/tiler-ir/src/semantic/registry.rs:2390`–`:2437`; `crates/tiler-artifact/src/program/codec/tests.rs:2584` | **Closed.** The three BF16 laws are registered, and `a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity` decodes, re-derives, and re-encodes |
| "six live tickets" | `grep -m1 '^status:' tickets/<id>.md` for all eight | **Two.** Four closed: `admit-bf16-into-the-schedule-and-kernel-vocabulary`, `lower-bf16-to-metal`, `carry-bf16-through-the-artifact-encoding-and-identity`, `validate-bf16-at-the-runtime-routing-boundary`. Left: `establish-bf16-optimizer-legality` (`todo`) and `conform-the-bf16-vertical-end-to-end` (**`in-progress` at this base**, so the row says in flight rather than queued) |

**The rung is held at R4 and the reason is stated rather than assumed.** All three of R6's conjuncts now hold independently for the three BF16 keys — vocabulary, backend emission, and a flush-honouring declared realization accepted where the strict one is refused by name (`a_strict_bf16_contract_is_refused_on_the_measured_macos_row` asserts both halves). R5 is unmet: `crates/tiler-compiler/src/policy.rs:815`'s `UNPLANNED_OPERATIONS` still lists all three keys with no capability row. The ladder is monotone, so this is tested non-monotone physical evidence above an unmet rung, recorded in the rung cell the way the U4 dequantization row records its own.

### The activation row — the evidence bar decided as R5, with the derivation

**Decision: R5 stands, and the row now names what R6 is missing rather than a closed ticket.** The wall the row named is gone — `admit-the-registered-unary-families-at-the-compiler-request-boundary` is `done`, `ElementwiseFamily` is `Add | Multiply | Silu` (`crates/tiler-compiler/src/request.rs:4291`), and `the_activation_compiles_and_matches_the_reference_bit_for_bit` (`crates/tiler-compiler/src/pipeline/conformance.rs:1445`) compiles `silu(x)` through `compile()` and bit-compares three rows against the reference. ADR 0099 is `decision_status: accepted`, `implementation_status: partial`, and the row already cites it.

The derivation, stated so it can be refuted rather than only read. R6 is a conjunction of three: the vocabulary carries the construct, a backend emits it, and the target's declared numerical realization does not reject it. The first holds (`PointwiseF32Node::{Exp, Divide}`, `UnaryOp::F32Exp`, `BinaryOp::F32Divide`). The second holds at unit level — `the_silu_kernel_emits_the_precise_exponential_and_a_division` (`crates/tiler-metal/src/tests.rs:1810`) drives a real emission and asserts one `precise::exp(`, one division, and four forbidden spellings absent. **The third is contradicted by the only observation of it**: `the_silu_kernel_records_the_f32_subnormal_gap` (`:1872`) asserts a non-empty gap set and `require_declared_realization().expect_err()`, and no test observes a SiLU unit accepted by a declared realization — the flush-honouring half the BF16 fixture pairs with its refusal has no activation counterpart. Independently, **no golden compiles an elementary-function unit**: `grep -rn 'precise::' crates/tiler-metal/goldens/` returns nothing and `golden_compilation.rs` names nine goldens, none carrying an exponential or a division, so the emitted MSL's validity is a claim about its text. That is precisely the class of claim only compilation catches — `lower-bf16-to-metal` found `as_type<bfloat>(0x4000u)` rejected at the `metal` stage, a defect no string assertion would have surfaced.

**Why this is not a fork for Tom.** Promoting on the first two conjuncts would assert the third against its own recorded negation, which is a maturity overclaim rather than a priority difference; and the structural row reached R6 one day earlier on a translate-and-link measurement, so a lower bar here would make two rows in one table mean different things by R6. There is no surviving candidate to escalate. The bounded residual is filed as `compile-an-elementary-function-golden-through-the-metal-toolchain` (`todo`, `implementation/metal`), which owns the golden **and** the acceptance observation, and the row and its trigger cell both name it.

### The normalization and softmax rows — the closed ticket replaced with the real blocker

`lower-a-two-region-occurrence-through-one-index-access-capability` is `done`, `admit-a-multi-region-index-realization-law` is `done`, and **the sequence surface was accepted**: `accept-the-multi-region-index-realization-surface` is `done`, carrying `VerifiedIndexRegionSequence`, `StagedInputSource`, `StagedIntermediate`, `CanonicalIndexRegionSequenceIdentity`, and `IndexRegionSequenceError` to Tom. `IndexAccessLoweringProvider` now has a defaulted `lower_sequence`, and the counter the parked measurement recorded as `0` reads `1`.

The blocker is one layer down and is now what both rows name: `crates/tiler-ir/src/semantic/registry.rs:2390`–`:2437` registers `IndexRealizationLaw` rows for **twelve** operations and neither family is among them — the three `f32` pointwise keys, the strict serial sum, the activation, reindex, broadcast, the strict tensor contraction, strict-affine U4 dequantization, and the three `bf16` keys. The list's own comment says absence "fails closed later", and it does: `refine_index_region` resolves the law before `emit_region`, so a lawless family returns `MissingRealizationLaw` with a driven-provider count of exactly `0`. The request boundary is a second, higher wall for both, since `elementwise_family` classifies neither key. `admit-the-registered-elementary-families-as-recognizable-program-stages` (`todo`) owns both halves generally.

**A correction landed in that ticket's own body.** Its sequencing bullet claimed `crates/tiler-ir/src/index/scalar.rs` "registers ten governed keys and neither" `exp` nor `rsqrt`. Half of that is false: `exp_f32_scalar_op` is `:65`, landed with the activation. The genuinely missing keys are `rsqrt` for the normalization and a **maximum** for the softmax's shifting fold — a second key the bullet did not name at all — so the two families need different keys rather than one shared gap. Corrected in tense in that ticket rather than reported only here, because a future worker would have sequenced against it.

### The structural-limits paragraph at `:555`

All four tickets in its wall list are `done` (`admit-the-registered-unary-families-at-the-compiler-request-boundary`, `admit-elementwise-epilogues-over-a-materialized-intermediate`, `admit-a-reduction-over-a-declared-input-tensor`, `admit-ordered-multi-output-programs-at-the-compiler-request-boundary`), so the list named no live wall; each is annotated with which landing removed which wall rather than dropped. The sentence above it was also stale in a way the brief did not name — its "one output" clause is false, since the cardinality guard was removed outright and `recognize_program_outputs` walks each declared output with `check_output_cover` requiring a partition — so the paragraph now states what the boundary *still* refuses, read at `crates/tiler-compiler/src/request.rs`: `input-arity`, `missing-output`, `dtype-f32`, `operation-set`, `output-partition-overlap`, `structural-operand`, and the strict-affine U4 scalar program.

### Two stale pins corrected along the way

The activation row recorded the explain request qualifier moving to `50c735514f5d51ca` and the concatenate row asserted it "stays `a95ad77532352d7f`". Both are historical values of a live pin that now reads `689c3aefc30f48d3` (`crates/tiler-compiler/src/explain.rs:4183`, whose ledger comment carries each intervening step). The first is the activation landing's own value and is kept with the date and the current reading beside it; the second's *observation* — that the concatenate fusion role did not move the digest — survives and is kept, while its value is corrected.

### Catalog items

- **The freedom-sites row** is added to `docs/research/README.md`, in the `numerical-operations` group its record's own `catalog_group` names, at the alphabetical position between the two `Floating-point …` rows and `Initial operation conformance matrix`. The row text is the one recorded verbatim in `enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle`'s Outcome, and it agrees with `docs/research/reference/plan-freedom-sites.md`'s frontmatter field for field: `disposition: pending`, `evidence_classes: [primary-source-synthesis, sound-proof]`, `informs: [correctness-and-testing, numerical-semantics]`. The record's path resolves at this base.
- **No ADR-catalog line is owed for the emitted-pragma landing**, verified rather than assumed. `docs/decisions/README.md` carries 0091 twice, at `:61` and `:235`, and both read `accepted`; the record's frontmatter is `decision_status: accepted`, `applies_to: [tiler.contract.numerical-semantics]`, `evidence: [tiler.research.numerics.bf16-computation-accumulator-and-conversion]`, and the `:61` row renders exactly those two edges with the matching title. The catalog renders no `implementation_status`, which is the only field the pragma landing could have touched, so the line describes the record accurately and nothing was changed.
- **The enforcer input-property exclusion row** is added to the `physical-planning-lowering` group, before `Exhaustive fusion-region oracle` and after `CPU vector realization facts`. Its frontmatter was read at `origin/main` (`git show origin/main:docs/research/region-search/enforcer-input-property-exclusion.md`), not from the dispatch message: `disposition: informational`, `evidence_classes: [primary-source-synthesis]`, `depends_on: [tiler.research.region-search.rewrite-search-formalism]`, no `informs`, no experiments. **`depends_on` is not rendered by this catalog** — no row in the file carries such a clause, and `plan-freedom-sites` declares one that its own drafted row omits — so the row carries disposition and evidence only. **Its link target does not exist at this branch's base**: the record merged to `origin/main` after `afdac9c9`, so the link resolves only on the integration tree.

### Scope, checks, and boundaries

Changed files: `docs/roadmap.md` and `docs/research/README.md` (`contracts/navigation`), and `tickets/re-read-the-bf16-and-elementary-support-rows-against-source.md`, `tickets/admit-the-registered-elementary-families-as-recognizable-program-stages.md`, and the new `tickets/compile-an-elementary-function-golden-through-the-metal-toolchain.md` (`project/tickets`). Nothing under `crates/`, `prototypes/`, `Cargo.*`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh`, so this delta touches no gate input.

**Measurement boundary — nothing was run.** Every claim is a source or metadata reading at `afdac9c9` (plus `origin/main` for the one record the coordinator added mid-dispatch), each with a file and a line, and each absence stated as a command that reproduces it. No cargo build, no test execution, no device.

### Found and not absorbed

`KernelType::Bf16`'s own doc comment (`crates/tiler-ir/src/kernel/model.rs:111`–`:115`) still reads "What is still absent is a *backend* that can emit one: `crates/tiler-metal` refuses this type by name rather than spelling `bfloat`, because it carries no BF16 constant reinterpretation, canonicalization helper, or dispatch route." All three named absences were landed by `lower-bf16-to-metal`; `msl_type` spells `bfloat` at `crates/tiler-metal/src/emit.rs:993` and `MetalHelper::CanonicalizeBf16Nan` exists. The comment is the defect and the source wins. It is `implementation/ir`, which this ticket does not hold, so it is reported to the coordinator rather than edited here.
