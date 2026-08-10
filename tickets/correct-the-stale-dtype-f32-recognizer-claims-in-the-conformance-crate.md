---
id: correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate
title: Correct the stale dtype-f32 recognizer claims in the conformance crate
status: done
priority: p2
dependencies: []
related: [widen-the-strategy-recognizer-past-the-f32-wall, conform-the-bf16-vertical-end-to-end, correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed, correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, dtype, correction]
---
## What is false

> **"Two module comments" was an undercount and is struck. Corrected 2026-08-07.** There are **three** sites carrying a live `dtype-f32` claim, plus a fourth stale claim of a different kind. The two originally listed are accurate as to line and text; the list was incomplete, and a worker who satisfied it would have failed the closing condition.

**Fact, verified 2026-08-07 by reading each file in full.** Four stale claims in `crates/tiler-conformance`, three of them the `dtype-f32` rule:

- `crates/tiler-conformance/src/lib.rs:17` — "non-`f32` program under the rule `dtype-f32` before a subject is normalized".
- `crates/tiler-conformance/src/bf16_vertical.rs:20` — "refuses every program carrying a non-`f32` value under the rule `dtype-f32`".
- **`crates/tiler-conformance/src/serial_sum.rs:17`** — "the recognizer's `dtype-f32` rule admits this program, so the portfolio, the plan alternatives, and their ABI are all crossed". A live claim, unlisted until now. Note its *conclusion* is still true — the compiler is in this vertical's path — so this one needs its **reason** replaced, not its outcome.
- **`crates/tiler-conformance/src/bf16_vertical.rs:26-27`** — asserts that `crates/tiler-compiler/src/pipeline/tests.rs`'s BF16 vertical "records the same boundary in the same words". It no longer does: `pipeline/tests.rs:3981-3988` now records the dtype rule as **removed**, citing `a_flush_accepting_bf16_contract_reaches_a_selected_plan`. This is a cross-file agreement claim, so correcting only one side re-breaks it.

`select_supported_strategy` no longer carries a `dtype-f32` rule at all. It derives the program's one arithmetic type and admits the two widths this build spells a per-point body in, refusing a width it cannot spell under `dtype-recognized` and a mixed-width program under `dtype-uniform`.

**Fact.** `crates/tiler-conformance/src/bf16_vertical.rs:24` cites the compiler test `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall`, which was renamed to `a_flush_accepting_bf16_contract_reaches_a_selected_plan` and now asserts the opposite outcome.

## What is true now

> **This whole section was overtaken and is struck. Corrected 2026-08-07.** It read that a multi-occurrence BF16 region "stops one layer further on, at `fusion_legality`, whose capability table is keyed by the `f32` operation set", cited `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall`, and instructed the corrector to write that the vertical stays hand-assembled "for the *fusion* reason rather than the dtype one". **`establish-bf16-optimizer-legality` landed on 2026-08-07 and removed that wall**, so the section directed a worker to **replace one false claim with another** — the most dangerous defect found in this ticket. The cited test does not exist; it was renamed to `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs:543`).
>
> **A second error, independent of the landing:** the section attributes the fixture `(x * 3.0) + (-0.0)` to this vertical. That is the **compiler's** fixture (`crates/tiler-compiler/src/pipeline/tests.rs:3991`). The conformance vertical is **`(x * 1.5) + 0.0`** (`crates/tiler-conformance/src/bf16_vertical.rs:6`, `SCALE_BITS = 0x3fc0` at `:120`, `BIAS_BITS = 0x0000` at `:122`), and `-0.0` is **deliberately rejected** there — `bf16_vertical.rs:51-53` explains that `fadd y, -0.0` is the IEEE identity and folds away, which would leave the `add` leg vacuous. Writing `-0.0` into this crate would contradict the reasoning the same file already gives.

A single-occurrence BF16 program is recognized, planned, and reaches a selected `PlanAlternative`. A **multi-occurrence** BF16 region now **fuses**, under a proof carried at its own width. Two boundaries survive and must be named wherever the fusion is stated, or the correction overshoots in the opposite direction:

- **Reassociation is withheld as `Unknown`** — not proved, merely not required.
- **The four reduction obligations are discharged vacuously, over an empty population** — which is not evidence the reductions are correct, only that none were present.

The surviving wall test, for citation in place of the dangling name, is `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` (`bf16_numerical_contract.rs:691`).

## The first task: establish why the BF16 vertical stays hand-assembled

**This ticket no longer knows.** The dtype reason is gone and the fusion reason is gone with it, and nothing structural bars this crate from the request boundary — `crates/tiler-conformance/Cargo.toml` lists `tiler-compiler.workspace = true` as a **normal** dependency, and `serial_sum` already calls `compile()`.

So establish the true reason before writing any replacement text, and if there is no longer a good one, **say so** — "this vertical could now go through `compile()` and should" is a legitimate and valuable outcome of this ticket, filed as its own ticket rather than done here.

One **unverified** candidate, offered as a lead and not as an answer: the run binds `FIRST_MACOS_APPLE9`, whose BF16 rows are subnormals-only per `docs/dtype-support.md:140`, so a real `compile()` may refuse at numerical resolution. Verify or refute it; do not repeat it as fact.

## Why it is filed rather than fixed

`crates/tiler-conformance/**` is `implementation/conformance`, which the recognizer branch did not hold and which was live-claimed by another worker at the time.

## Required evidence

- All four sites state the boundary that exists, naming the rule that refuses and its owner.
- Every test name cited anywhere under `crates/tiler-conformance/src/**` resolves against `cargo nextest list`.
- `cargo nextest run -p tiler-conformance` passes, and `cargo clippy -p tiler-conformance --all-targets -- -D warnings` is clean.

> **The evidence bullet below was unfireable and is replaced. Corrected 2026-08-07.** It read: "`cargo doc --no-deps` with warnings denied still passes for the crate." **That check cannot fail for three of the four sites.** Every module in this crate is `#[cfg(test)]` (`crates/tiler-conformance/src/lib.rs:155-172`), so rustdoc never compiles `bf16_vertical` or `serial_sum` and never reads their headers — verified empirically: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-conformance` emits only `index.html`, `all.html` and `sidebar-items.js`, with no module pages at all. It passes identically before and after any edit to those files. The crate's own header states the mechanism at `lib.rs:116-119`. This is the repository's recurring defect — a check that cannot say *no* — and it is why the `nextest`/`clippy` bullets above replace it.

- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-conformance` still passes — retained, but **only `lib.rs`'s own header is exercised by it**, and the report must say so rather than presenting it as covering the crate.

## Closes when

No comment in `crates/tiler-conformance` claims a `dtype-f32` rule **as current behaviour**, and each cited test name exists.

**Classify per hit, not by count.** This crate's own idiom is to record retired text inside a dated correction — `lib.rs:13-14` and `bf16_vertical.rs:19` ("**Fact, at this commit.**") both do it — so a surviving `dtype-f32` mention is legitimate when its enclosing paragraph is a dated correction describing the gate as retired, and is a defect otherwise. Report the classification for each hit with its evidence. A bare count cannot tell the two apart, which is exactly how this ticket's sibling `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents` acquired an unsatisfiable closing condition; that ticket was repaired with this same rule and the wording is deliberately shared.

Also closes on: the "first task" above answered — the true reason the BF16 vertical stays hand-assembled is established and cited, or its absence is reported and filed.

## Out of scope — already filed

`crates/tiler-compiler/src/session.rs:1729-1736` and `crates/tiler-compiler/src/pipeline/tests.rs:3990-4000` carry the same stale fusion wall, and `session.rs` cites the same dangling test name on a **public** constructor's doc comment. They are `implementation/compiler`, which this ticket does not hold. Filed as [`correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed`](correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed.md). **Do not touch them from this branch.**

## Outcome — done, 2026-08-07

Landed at merge **`a5138ebd`** (worker commit `4bc010d0`). Four files, all inside `crates/tiler-conformance/src/`; 222 insertions, 35 deletions. `make full` exit 0 on the merged tree.

### The hand-assembly reason was established, not guessed — and it is a different layer than every comment claimed

The worker ran a real `compile()` probe of the vertical's own `semantic_program` under its own `declared_contract()` against `BoundMetalCompileDeclaration::first_macos_apple9().profile()`, rather than reasoning from the ticket's lead:

```
class=NoFeasiblePlan  refusal=NumericalContract(TargetNumericalContractRefusal {
  target_profile: "tiler.metal.macos-apple9.msl4-0.f32-bf16.v1",
  rejections: [ { requirement: Contraction { subject: Bf16/tiler::bf16@1, required: Forbidden },
                  disposition: Unknown } ] })
```

**Verified independently by the coordinator** at `crates/tiler-build/src/metal_declaration.rs:781-871`: the ledger declares **seven** complete numerical rows — contraction, reassociation `Forbidden`, reassociation `Permitted`, permutation, signed zero, NaN, infinity — and **all seven are bound to `f32`** (`let f32 = ScalarArithmetic::f32();` at `:791`). BF16 gets only dispatchability and the two subnormal tables (`:782`). So BF16's contraction dimension is `Unknown` and the target profile refuses.

**The recognizer admits the program and fusion legality admits it too — fusion legality is never reached.** The live boundary is numerical resolution against the ledger's undeclared BF16 contraction row. That the refusal names `Contraction` rather than a subnormal dimension is itself the evidence that the flush-accepting contract cleared the measured rows. So the ticket's alternative outcome — "this could now go through `compile()` and should" — is **refuted**: it could not.

### Five stale sites, not four

`bf16_vertical.rs:428` carried a live `dtype-f32` claim the ticket never listed — it spelled the rule without naming it, so it was invisible to any search for the rule's name. Found by full read. Verified present in the base at that line. Six surviving `dtype-f32` mentions after the change, **all inside dated 2026-08-07 corrections** describing the gate as retired, per the crate's own idiom.

### A judgement call worth recording

The cross-file agreement claim at `bf16_vertical.rs:26-27` was **removed rather than restated**. A claim that two files phrase one boundary alike cannot be checked from this branch and re-breaks whenever either side is edited — and the other side was being edited concurrently. What replaces it is test-name citations, which `cargo nextest list` resolves. Correct call.

### The new test, and its perturbation

`the_request_boundary_stops_at_the_ledgers_undeclared_bf16_contraction_row` makes the reason checkable. **`tiler-compiler` cannot host this check** — `FIRST_MACOS_APPLE9` lives in `tiler-build`, which depends on the compiler; this crate depends on both.

**Coordinator-verified deliberate failure, against the real regression rather than the assertion:** adding a BF16 contraction row to the ledger turns it red, and the panic names the *next* undeclared dimension — `Reassociation { subject: Bf16, required: Forbidden }` — which is exactly the right diagnostic. Not a vacuous guard.

53 tests pass, **0 skipped**, so the measured half genuinely ran rather than passing as an unavailable host. All 20 cited test names resolve against `cargo nextest list --workspace`.

`cargo doc` was run and its caveat stated as required: it exercises **only `lib.rs`'s header**, emitting no module pages because every module is `#[cfg(test)]`, so it cannot fail for four of the five corrected sites.


> **Correction, 2026-08-07 — the coordinator's "reassociation is withheld as `Unknown`" was over-general and is struck.** Found by the worker on [`correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed`](correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed.md), which declined to write the claim into the code rather than repeating it, and verified by the coordinator at `crates/tiler-compiler/src/fusion_legality.rs:1641-1653`.
>
> The obligation is discharged **`SoundProof`** when `!has_reduction || reassociation == Forbidden`. A multi-occurrence **pointwise** BF16 region has no reduction, so its `ReductionReassociation` records `SoundProof` **vacuously** — not `Unknown`. The `Unknown { "unproven-reassociation" }` branch requires a reduction **and** a permitting contract, which is precisely the surviving wall `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs:691`).
>
> **The substance stands and only the mechanism was wrong:** reassociation is *not proved* for these regions, merely *not required*, because the region carries no reduction order to preserve. Say that, grounded on `BF16_FACT_REASSOCIATION_PERMITTED` being `false` and no BF16 family declaring an algebraic capability. **Writing "the obligation records `Unknown`" would be a new false claim** — the exact defect these tickets exist to remove.

> **Correction — 2026-08-10.** Two residual Outcome defects (ticket prose only; code-side close conditions still hold; status stays `done`).
>
> **(1) Wrong wall named as the `unproven-reassociation` site.** The 2026-08-07 block above ends the discharge explanation with "which is precisely the surviving wall `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall`". **That identification is false and is struck.** That wall's contract is strict BF16 with contraction Permitted and reassociation Forbidden; both SoundProof disjuncts hold for `ReductionReassociation`, and the test asserts `unrealized-contraction` (`FusionObligation::ArithmeticContraction`), not `unproven-reassociation`. The true site of `Unknown { "unproven-reassociation" }` today is `fusion_legality::tests::a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` — an f32 serial-sum reduction under a permitting reassociation contract (`crates/tiler-compiler/src/fusion_legality.rs`, asserts `FusionObligation::ReductionReassociation` and reason `unproven-reassociation`). Sibling ticket [`correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical`](correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical.md) already repaired the *source* `bf16_vertical` header; this ticket's Outcome tail was the leftover false name.
>
> **(2) `dtype-f32` census overcount.** "Six surviving `dtype-f32` mentions after the change" under "Five stale sites, not four" is **false at this base and is struck.** Exact-string search under `crates/tiler-conformance` finds **four** hits (`lib.rs`, `bf16_vertical.rs`, `serial_sum.rs`, `bf16_vertical/tests.rs`), each inside dated 2026-08-07 correction / retired-gate framing — not live behaviour. Counting rule: literal `dtype-f32` under that crate root, classified by enclosure per Closes when (dated correction describing the gate as retired is legitimate; present-tense live claim is not). Whether six was true at merge `a5138ebd` is not re-checked here.
