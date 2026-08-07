---
id: carry-a-bf16-subnormal-realization-the-reference-can-be-told
title: Carry a BF16 subnormal realization the reference can be told
status: done
priority: p2
dependencies: [accept-the-bf16-subnormal-resolution-carrier]
related: [apply-the-declared-numerical-conformance-on-every-reference-evaluation-path, derive-the-oracle-for-a-permitted-divergence-candidate, conform-the-bf16-vertical-end-to-end, declare-the-bf16-rows-on-the-authoritative-metal-profile, state-and-check-a-bf16-numerical-contract, accept-the-bf16-subnormal-resolution-carrier]
scopes: [implementation/reference, implementation/ir]
shared_scopes: [project/tickets]
tags: [numerics, reference, conformance, bf16]
---
## User-visible outcome

A BF16 candidate compiled for a target that flushes BF16 subnormals is qualifiable against a reference that was told so, instead of against one whose only subnormal vocabulary is binary32.

## Both triggers have fired (2026-08-06); the section below is the state that deferred it

The two Facts marked **superseded** below were true when written and are false now. They are struck rather than deleted because the reason this ticket was deferred is the reason its deliverables are shaped as they are, and a reader who cannot see the original ground cannot judge whether the shape still fits.

## Why this was deferred rather than open

**Fact — the conformance object's two dimensions are binary32 functions.** `ReferenceNumericalConformance::apply_to_operand` and `apply_to_result` (`crates/tiler-reference/src/conformance.rs`) take and return `f32`, and the BF16 family performs no binary32 arithmetic to apply them to: its operands are exact rationals decoded from BF16 encodings and its one rounding is over BF16's value set. So the object cannot reach this family, and threading it there would be applying a format's rule to values not in that format.

**Fact — the behaviour is declared rather than unstated, and it is declared as preservation.** `BF16_FACT_SUBNORMALS` resolves to `preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed` (`crates/tiler-ir/src/semantic/bf16.rs:252`) and the value contract to `preserved-every-subnormal-encoding-denotes-a-distinct-constant` (`:207`). The reference realizes exactly that, so nothing is silently resolved today.

~~**Fact — no target Tiler compiles for has been measured to flush BF16 subnormals, and one measured row preserves the narrower format.** `crates/tiler-reference/src/conformance.rs`'s header records the measured Apple behaviour as flushing in `f32` "while preserving them in `f16`". BF16 is not `f16` and that row is not evidence about it, which is the gap this ticket would close and the reason it is not a claim either way.~~ **Superseded 2026-08-06.** Finding 24 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) measures BF16 arithmetic flushing on the macOS row across all seven flush dimensions, and `declare_metal_bf16_subnormal_behaviour` (`crates/tiler-build/src/metal_declaration.rs:768`) projects that measurement into declared input and result subnormal rows against `ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())` on the authoritative profile. The target this reference is compared against is now measured *and* declared to flush.

~~**Inference — so the declared preservation is currently a fact about every reachable target, and a realization vocabulary for a case nobody can compile would be a type-system reservation dressed as support.**~~ **Superseded 2026-08-06 by the same two facts.** The declared preservation is now a fact about *no* reachable target's BF16 arithmetic, which inverts the inference: it is the reference, not the vocabulary, that is now the reservation.

## Trigger check log

- 2026-08-05 — **not fired.** No BF16 target row is measured, and no registered numerical contract carries a per-format subnormal resolution. Reproduce the second half with `grep -rn 'BF16\|bf16' crates/tiler-ir/src/schedule/numerics.rs` (empty: `NumericalRealization`'s subnormal fields are format-agnostic and are read as binary32 by every consumer).
- 2026-08-06 — **fired, both conditions independently.** The command the line above records as empty now returns 49 matching lines, and what it returns is the second condition met rather than incidental mentions: `ArithmeticType` (`crates/tiler-ir/src/schedule/numerics.rs:354`, `Bf16` at `:358`) names `Bf16` as a subject a behaviour is declared *for*, `BF16_NUMERICAL_CONTRACT_KEY_DOMAIN` renders `bf16` contracts under their own closed grammar, and `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16` (`crates/tiler-compiler/src/session.rs:1522`) is a registered contract resolving the subnormal dimensions per format. The first condition is met separately by the authoritative profile declaration cited above. Reproduce the whole verdict in one line: `grep -c 'BF16\|bf16' crates/tiler-ir/src/schedule/numerics.rs && grep -n 'declare_metal_bf16_subnormal_behaviour' crates/tiler-build/src/metal_declaration.rs` — a nonzero count beside a declaring function is both conditions. Moved to `todo` and to p2 by the worker on `conform-the-bf16-vertical-end-to-end`, which is blocked on this ticket's deliverable and has been given the dependency edge.

## Trigger

Either of:

- a target profile declares BF16 arithmetic that flushes subnormals, or a measurement on a qualified row observes it; or
- a registered numerical contract resolves a subnormal dimension per format rather than once for the region.

## What this ticket must produce

- A declaration that names *which format* a subnormal resolution speaks about, so `NumericalRealization`'s two fields stop being implicitly binary32. This is a public boundary and is Tom's, not self-accepted. **The fork a dispatcher should brief explicitly**, since it is what makes this a decision rather than a mechanical widening: the format can be *derived* at the point of use — a BF16 capability knows its own format by construction, so it could read the format-agnostic `SubnormalMode` off the conformance and apply it at its own rounding boundary with no new field — or it can be *declared*, adding a subject to `NumericalRealization`'s two fields. The derived route needs no public boundary and no `implementation/ir` edit, and it is correct exactly while no program mixes widths; the declared route survives the first admitted BF16/binary32 conversion, which [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) already decides the shape of but nothing registers. Neither is dominant on correctness today, which is why it reaches Tom rather than being settled by a worker.
- A reference that can be told it, applied at the BF16 rounding boundary where the family's own arithmetic commits a value.
- The counterexample population: a BF16 operand and a BF16 product in the subnormal range whose preserving and flushing readings differ, with exact encodings.
- The declared fact updated from an unconditional `preserved` to whatever the realization vocabulary makes it, and the reference's `bf16` module header — which currently names this ticket as where the gap lives — updated with it.

## Explicit non-goals

Widening the binary32 conformance object to stand in for a BF16 one; approximating BF16 flushing with the binary32 modes; any change to the exact-rational arithmetic or its single rounding.

## Closes when

The trigger has fired, a BF16 subnormal realization is declared and accepted, and the reference applies it with a case watched failing.

## Outcome — core delivered 2026-08-07, fork parked

**The route-independent core is landed and the branch is integrable; the ticket stays `in-progress` because its close gates on the fork, which is Tom's.** The fork is filed as [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md), which carries the packet: both arms with what each enables and prevents, the strongest counterpoint to each, a recommendation with its ground, and the exact follow-up files and scope set under each.

### The machinery

`Bf16SubnormalRealization` (`crates/tiler-reference/src/bf16.rs`) carries ADR 0019's two independent dimensions over BF16's value set, in the schedule's own `SubnormalMode` and `FlushedZeroSign` vocabulary rather than a second spelling of it. It is applied at exactly two sites, both on **encodings**, so neither reaches the exact-rational arithmetic or the single rounding between them:

- `Bf16Format::accept_operand` replaces a subnormal operand encoding *before* it is decoded — after the decode an operand is an exact rational that no longer knows it came from a subnormal encoding;
- `Bf16Format::commit` performs the family's single `round` and then applies the result dimension to the **rounded** encoding, which is the produced result a target flushes. A value that rounds *up* to the least normal is therefore normal and is not flushed, and one that rounds *down* into the subnormal range is — the distinction a mode applied to the pre-rounding exact value would lose.

`Bf16BinaryReference::combine_under` takes the realization per evaluation rather than holding it as capability state, because a registry holds one `Arc` per key while the contract an evaluation is performed under is the caller's. `combine` delegates with `Bf16SubnormalRealization::preserving()`, and that is what all three registered keys reach, so **no registered value moved**: the exhaustive census re-checks that the preserving realization is the identity on all 65,536 encodings in both dimensions.

The type is crate-internal (`pub(crate)`). Making it public would be self-accepting a boundary this ticket says is Tom's, and arm A of the fork likely never needs it public at all.

### The counterexamples

Seven cases in `crates/tiler-reference/src/bf16/tests.rs`, each stating **all four** resolutions of the two dimensions — the coverage the operation conformance matrix requires and ADR 0076 cites — because a case stating only the preserving and both-flushing answers cannot distinguish a realization that resolved one dimension from one that resolved both. Four reproduce a measured finding 24 row on the macOS Apple9 row:

| case | preserved | input flushed | result flushed | both | measured row |
| --- | --- | --- | --- | --- | --- |
| `0040 * 4000` (subnormal operand doubled) | `0080` | `0000` | `0080` | `0000` | input flush, multiply: `0040 -> 0000` |
| `8040 * 4000` (the same, negated) | `8080` | `8000` | `8080` | `8000` | input flush, sign: `8040 -> 8000` |
| `0080 * 3f00` (normal halved into the subnormal range) | `0040` | `0040` | `0000` | `0000` | result flush, multiply: `0080 -> 0000` |
| `8080 * 3f00` (the same, negated) | `8040` | `8040` | `8000` | `8000` | — |
| `0080 * 3eab` (rounds *inexactly* into the range: 42.75 quanta to 43) | `002b` | `002b` | `0000` | `0000` | — |
| `8040 + 0080` (subnormal addend beside a normal) | `0040` | `0080` | `0000` | `0080` | input flush, additive path: `8040 -> 0080` |
| `0081 * 3f00` (a tie into the range: 64.5 quanta to the even 64) | `0040` | `0040` | `0000` | `0000` | — |

Two cases separate the input dimension alone, four the result dimension alone, and one separates both and answers three different values — its flushed answer is *normal*, so the flush is invisible as a returned zero, which is the shape ADR 0076's additive-path row records. The population count, the measured-row count, and the per-dimension separation counts are all asserted, so a population that emptied could not still look like a passing check.

**Watched failing, in both directions and on each dimension separately.** `the_reference_answers_the_realization_it_is_told_and_not_another` asserts each reading's answer *and* that the reference did not return any of the other three readings' differing answers. Two perturbations were run and reverted: dropping the result dimension from `commit` fails three tests with 26 named disagreements; dropping the input dimension from `accept_operand` fails the same three with 16. `the_flushed_zero_sign_is_read_on_both_dimensions` holds the sign resolution to `PreservesSign -> 8000` and `AlwaysPositive -> 0000` on both dimensions, so a flush that erased the sign fails — finding 24 measures the BF16 input flush returning `8040 -> 8000`, and the result dimension's sign is stated as what the declared mode requires rather than as a second measurement.

### The declared fact: unchanged, and that is a decision rather than a deferral

`BF16_FACT_SUBNORMALS`'s unconditional `preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed` **honestly survives this landing and is not edited.** The fact states what `tiler::multiply-bf16@1` and `tiler::add-bf16@1` *mean*; a flushing realization is a declared deviation a region's numerical contract carries, not a second opinion about the operation's semantics. Weakening it to match a target would be the authority substitution ADR 0076 forbids, and the repository already answers this question the same way one module over: `crates/tiler-ir/src/semantic/quantization.rs:182` records that `tiler::dequantize-strict-affine@1`'s `preserve-subnormals` "stays declared and unweakened: it is what the decode *means*, and substituting a flushing realization for it would be the authority substitution ADR 0076 forbids." `crates/tiler-ir/src/semantic/bf16.rs:35` says the same thing prospectively — "Subnormal preservation here is semantics, never a target claim."

Two consequences. The change is confined to `implementation/reference` and touches no `tiler-ir` file, which is the graph note's stated preference. And it steps no identity: operation definition facts move the registry snapshot and every identity derived from it (`crates/tiler-ir/src/semantic/bf16.rs:12-16`), so leaving the fact alone leaves every pin alone. This holds under **both** arms of the fork, so it is not a deferral to the decision.

### The `bf16` module header

Rewritten to the new state: the binary32 conformance object still cannot reach this family and widening it is still refused; the family can now be told a realization of its own, at the two named sites; nothing supplies a flushing one, and which route would is the parked fork; the declared facts stay unconditional for the reason above.

**The reproduction command in [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) still holds and was deliberately preserved.** That paragraph reproduces the gap with `grep -n conformance crates/tiler-reference/src/bf16.rs`, "whose only two hits are the module header explaining the gap". The rewritten header carries exactly two lowercase `conformance` occurrences, both in the header, and the gap it explains is still a gap — the reference can be told, and nothing tells it. `contracts/numerics` is outside this ticket's scopes, so the paragraph is not edited here; retiring it belongs to the change that closes the fork, per this ticket's own graph-maintenance note.

### Commit

On `tkt/carry-a-bf16-subnormal-realization-the-reference-can-be-told`, base `dedb95b6`. The machinery and its population landed at `a759c058`, this record and the decision node at `efa5bc68`, and a test-readability follow-up at `598989c9`; the branch tip is one commit further and carries this paragraph. Integrate the tip, which the gate below was run against. Four files: `crates/tiler-reference/src/bf16.rs` and `crates/tiler-reference/src/bf16/tests.rs` under `implementation/reference`, and the two tickets under `project/tickets`. `implementation/ir` is declared and unused, per the fact decision above.

### Checks

Run in the worktree. `make full` was green at the gated commit: 2921 workspace tests passed with 7 skipped, the release numerical run 1012 passed with 3 skipped, rustdoc under `-D warnings`, doc-tests, `tkt lint`, and shellcheck. The branch tip carries one further **ticket-only** markdown correction on top of that commit and touches nothing under `crates/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh`, so it carries the gate under the delta rule; `tkt lint` was rerun on it.

- `cargo fmt --all --check`
- `cargo clippy -p tiler-reference --all-targets -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-reference --no-deps`
- `cargo nextest run --workspace` — 2921 passed, 7 skipped
- `cargo test --workspace --doc`
- `make full` — exit 0
- `tkt lint` (clean), `git diff --check` (clean), and `tkt guard tkt/carry-a-bf16-subnormal-realization-the-reference-can-be-told --format json` — `"conflict": false`, `"under_declared": []`, severity `warn` from shared-scope overlaps only, changed files exactly the four above.

### What is not delivered

The wiring — nothing supplies a flushing realization — and the retirement of the correctness-and-testing exception paragraph, which stays true until the wiring lands. Both are the fork's, on `accept-the-bf16-subnormal-resolution-carrier`.

## Graph maintenance

Filed by [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md), which had to decide what the BF16 capabilities do with a conformance they cannot use. Filed `deferred` rather than `todo` because its triggers had not fired and the board must not offer non-work; moved to `todo` on 2026-08-06 when both fired, per the log above.

- `implementation/ir` is required only by the *declared* route in the fork above. A dispatcher choosing the derived route can run this on `implementation/reference` alone; the scope stays declared either way, because the fork is not resolved until Tom resolves it and a brief must not pre-commit the scope set to one arm.
- The exception this gap creates in [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) — that the declared-contract comparison rule has one family that cannot follow it — was recorded on 2026-08-06 by `conform-the-bf16-vertical-end-to-end`. Closing this ticket must retire that paragraph in the same change, or it becomes a stale disclosure of a gap that no longer exists.

## Closed 2026-08-07 — the fork is resolved and the remainder is split out

**Tom decided the fork on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator: **arm A**, the format derived at the point of use, with no mixed-width refusal and arm B closed against a trigger. The reasoning, including why this node's own recommendation of the staged shape was refuted by reading the source in full, is on [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md).

**This ticket's own deliverables are all landed and in `main`** — commits `a759c058`, `efa5bc68` and `598989c9` are ancestors of `HEAD`, confirmed by the coordinator rather than relayed. The machinery, the seven-case counterexample population, the watched-failing perturbations in both directions and on each dimension separately, and the module header are in the tree.

**The remainder is a narrow ticket rather than an open parent**, per AGENTS.md: split the bounded remainder, then close the revised parent so dependents can proceed. Two pieces:

- [`wire-the-bf16-reference-to-the-realization-it-is-told`](wire-the-bf16-reference-to-the-realization-it-is-told.md) — the one missing link, `ReferenceOperation::evaluate` reaching `combine_under` with the realization built from `request.conformance()`, plus retiring the [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) exception paragraph this gap created.
- [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md) — filed from a defect found while assessing the fork: `ReferenceNumericalConformance::from_realization` has **no caller anywhere**, so every evaluation in the workspace runs under the strict reading whatever a region declared. Neither arm of the fork would have delivered this ticket's user-visible outcome without it.

**The declared facts stayed unconditional and no identity moved**, which holds under the accepted arm exactly as this ticket's Outcome argued it would.
