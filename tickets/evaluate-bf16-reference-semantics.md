---
id: evaluate-bf16-reference-semantics
title: Evaluate BF16 reference semantics from an exact-rational oracle
status: in-progress
priority: p1
dependencies: [register-the-bf16-semantic-operation-signatures]
related: [spike-bf16-through-the-second-dtype-seams, preserve-primary-dtype-standards-evidence, correct-the-bf16-reference-evaluation-status-outside-the-dtype-ledger]
scopes: [implementation/reference]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, dtype, bf16, reference, numerics]
claimed_from: todo
assignee: worker-bf16-ref
lease_expires_at: 1785580761
---
## User-visible outcome

`tiler-reference` evaluates a pure-BF16 program to exact bits, so every later BF16 claim — a kernel, a lowering, a device result — has an independent oracle to be wrong against. Without it a BF16 backend would have nothing to compare to, which is how a silently wrong tensor survives.

## Why the oracle must be exact rational

**Measurement.** Finding 24 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) records that no single operation can separate `f32`-precision evaluation from native `bfloat` arithmetic, because `f32`'s 24-bit significand exceeds the 18 bits that would make a second rounding to BF16's 8-bit significand innocuous.

**Inference.** An oracle that computed in host `f32` and rounded to BF16 would therefore agree with a double-rounding implementation *because it shares the defect*. Host-native arithmetic is not normative evidence for this dtype.

**Fact.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) built exactly this oracle out of tree — exact rational arithmetic over `num-bigint`, one rounding at the observable materialization — and it agreed on all 65,536 encodings and on 24 hand-derived witnesses across six categories. The spike's `src/bf16.rs` and `src/corpus.rs` are the working draft this ticket productionizes; its exhaustive round-trip and its overflow-boundary check are the shape the tests should take.

## Implementation keys

- A reference evaluator registered for each of the three BF16 operation keys, refusing any operand whose resolved type is not `tiler::bf16@1`.
- A `ReferenceValueValidator` for `tiler::bf16@1` checking element width against the registered descriptor. `ReferenceElement` is already a width-generic byte carrier and needs no change — the spike confirmed it holds a 2-byte element and refuses an empty payload.
- Exact rational evaluation, rounded once, round-to-nearest-ties-to-even. Reuse `tiler_ir::semantic::accuracy::ExactRational` if a descriptor-parameterized ingress is added; otherwise state why a local exact type is kept. **Do not** route BF16 values through `ExactRational::from_f32` and a host `f32` without recording that the widening is exact and total for BF16 specifically and does not generalize to F64 or F128.
- Exceptional values decided, not inherited: both zeros and their signs, subnormals preserved, `inf * 0` and `inf - inf` as the canonical NaN, overflow at the midpoint above the largest finite value, and NaN canonicalization to the realization's stated payload.
- The arithmetic NaN canonicalization is shared with the crate's existing rule rather than redefined.

## Required evidence

- Every one of the 65,536 BF16 encodings round-trips decode-then-round unchanged, except NaNs, which canonicalize. This is `exhaustive-finite` evidence and should be stated as such.
- A hand-derived witness corpus covering zeros and signs, subnormals and underflow, ties, ordinary rounding, overflow, and infinities/NaN. Every expected value derived from the format's parameters, **not** by running the implementation and recording what it said.
- The overflow boundary checked on both sides: the midpoint below it rounds to the largest finite value, the threshold itself overflows.
- A perturbation that changes only the tie rule and is watched failing — the spike used ties-away-from-zero and saw 2 of 24 witnesses disagree while 0 disagreed under the normative rule. Without this the corpus could pass while measuring nothing.
- An F32 program still evaluates identically, pinned by the existing reference registry identity.

## Closes when

The three BF16 operations evaluate to exact bits, the exhaustive and witness populations both pass with their counts reported, the tie perturbation is observed failing, the reference registry identity's movement is recorded, and `docs/dtype-support.md`'s BF16 `Reference evaluation` cell moves with its boundary stated.

## Graph maintenance

- Depends on the operation keys existing; there is nothing to register an evaluator against otherwise.
- Gates `admit-bf16-into-the-schedule-and-kernel-vocabulary`, which needs an oracle to compare a lowered kernel against.
- The reference registry identity is a durable number that this ticket moves. Record the before and after; a spike or prototype citing the old one is drift, not a failure.
- Exhaustive enumeration is available here because the format is 16 bits. `docs/dtype-support.md`'s dtype-addition recipe records that F64 and F128 are not exhaustively enumerable and need a stated bounded profile instead; do not let this ticket's method be read as the general one.

## Outcome

`tiler-reference` evaluates a pure-BF16 program to exact bits. `crates/tiler-reference/src/bf16.rs` registers a `tiler::bf16@1` value validator and one evaluator per key, and `ReferenceEvaluator::standard()` answers where it previously returned `MissingCapability`.

### The oracle, and what parameterizes it

**Fact.** Nothing in the module restates a format parameter. `Bf16Format::from_declarations` reads the encoded width, precision, exponent range, exponent-bias override, and subnormal presence from the registered `tiler::bf16@1` descriptor, and the canonical arithmetic NaN payload from `arithmetic_bf16_facts()`'s `BF16_FACT_CANONICAL_NAN_BITS`. `governed()` is that function applied to the registered pair. There is no second width table: the validator's payload width is `width_bits / 8` from the same descriptor `tiler_ir`'s `registered_bf16_payload_bytes` reads.

**Inference — the exact type, and why `ExactRational::from_f32` is not on the path.** The ingress is the encoding itself: a subnormal decodes as `trailing * 2^(emin - 7)` and a normal as `(trailing | 2^7) * 2^(biased - 127 - 7)`, both built with `ExactRational::from_integer(..).scale_by_power_of_two(..)`. No host `f32` is constructed anywhere in the module, so the ticket's conditional disclosure about BF16→binary32 widening exactness does not apply — that route was not taken. It would have been sound (BF16 shares binary32's exponent width and bias, so every encoding is the high sixteen bits of an exactly equal binary32 one, including subnormals, both zeros, both infinities, and every NaN payload), and it does **not** generalize: F64 and F128 have no lossless in-tree widening carrier. The reason to decode from fields anyway is that the descriptor already states the fields, so the direct route needs no carrier claim at all. No new exact type was added and `tiler_ir`'s `ExactRational` needed no new ingress.

**Inference — the rounding is reused, not rewritten.** `UlpFormat::round_to_nearest_ties_even` is the exact format-parameterized RNE this workspace already owns, and its `bfloat` rule row cites the same RISC-V operand format the catalog row names. Overflow is decided *before* it, against `largest_finite + 2^(emax - 7 - 1)` — the midpoint above the largest finite value, `511 * 2^119` — so the boundary is a stated rule rather than an artifact of the rounding loop. `round_to_nearest_ties_even` refuses on exactly the same set (`result > largest_finite`), so its `Err` arm reaches the same answer rather than a second policy; a comment says so at the site.

**Fact — the canonicalization rule is shared, not redefined.** The crate root owns it (`lib.rs`, beside `canonicalize_arithmetic_f32`, now stating the BF16 case explicitly); the BF16 payload comes from the operation's declared fact, and `the_arithmetic_nan_payload_is_the_one_the_operation_declares` binds the evaluated result to that declaration. A constant does **not** canonicalize — `BF16_FACT_NAN_BEHAVIOUR` says a constant preserves its payload — and `a_constant_preserves_a_non_canonical_nan_payload_that_arithmetic_removes` checks both halves against each other.

### Populations, with their counts

**Exhaustive-finite.** All **65,536** encodings decode-then-round unchanged, except the NaNs, which canonicalize to `0x7fc0`. The class census is asserted before the verdict — 2 zeros, 254 subnormals, 65,024 normals, 2 infinities, 254 NaNs, summing to 65,536 — so a check that silently stopped enumerating cannot read as a pass. Available only because the format is sixteen bits; the module's test doc says so and points at the recipe's F64/F128 row.

**Executable model.** **30** hand-derived witnesses in six categories, every expectation derived from the format's parameters and carrying its derivation in a comment.

| Category | Cases | What each contributes |
| --- | --- | --- |
| zeros and signs | 6 | both zero sums, the opposite-signed sum, the sign exclusive-or through a multiply, and cancellation to `+0` |
| subnormals and underflow | 6 | preserved doubling, underflow to `+0` and to `-0` at the tie, the greatest subnormal plus the least reaching the least normal exactly, an exact halving into the subnormal range, and a subnormal negation |
| ties | 4 | ties-to-even rounding down at `128.5` quanta and up at `129.5`, the negative side, and a subnormal-spacing tie |
| ordinary rounding | 2 | `3 * (171 * 2^-9) = 513/512` rounding to one at `128.25` quanta, paired with the neighbour whose product is exactly representable |
| overflow | 5 | both signs by multiply and add, plus the boundary reached by arithmetic: a quarter of the top quantum above the greatest finite value stays finite, exactly half of it overflows |
| infinities and NaN | 7 | `inf * 0`, `inf - inf`, `inf + finite`, `inf * -finite`, `(-inf)^2`, and payload canonicalization through both arithmetics |

**Overflow boundary, both sides**, checked against the threshold directly rather than through an encoding: the midpoint between the largest finite value and the threshold rounds to `0x7f7f`; the threshold itself gives `0x7f80` and its negation `0xff80`; the largest finite value is still itself in both signs.

**Tie perturbation, observed failing.** `round_ties_away` in the test module changes *only* the halfway decision — same decode, same arithmetic, same threshold, same binade selection, same encode. The normative rule disagrees with the corpus at **0** of 30 witnesses; ties-away-from-zero disagrees at exactly **4**, asserted by name: half the least positive subnormal underflowing to `+0`, half the least negative subnormal underflowing to `-0`, the tie half a quantum above one, and the negative tie half a quantum below negative one. (The spike saw 2 of 24; this corpus adds the negative tie and the signed underflow pair.)

### Registry identity movement

**Measurement**, this worktree, pinned nightly, macOS arm64. `FrozenReferenceRegistry::standard().canonical_identity().as_bytes().len()`: **651,710 bytes at `26266d9` → 774,495 bytes** after this change, `+122,785` for one value validator and three capabilities. The measurement used a throwaway test that was removed before committing; it is reproducible in one line from that expression.

**Drift found, not a failure.** `spikes/target-profiles/scalar-cpu-vertical/README.md` and three ticket records cite 80,104 / 438,805 / 446,768 bytes from earlier runs. All three were already stale at this ticket's base — the base value is 651,710 — so the drift predates this change and is the kind the ticket's graph-maintenance section anticipates. Nothing in tree asserts a literal reference-registry identity, so no golden was rebaselined.

The `tiler.standard-reference` provider identity and capability revision were **not** bumped, following the precedent `admit-the-contraction-normative-reference` and `admit-the-reindex-and-broadcast-operation-families` record for the dtype catalog, the contraction, and the structural families: adding capabilities moves the registry identity on its own and changes no existing output.

### F32 unchanged

`an_f32_program_evaluates_identically_through_the_widened_registry` evaluates `1.5 * 2.0 + (-0.5)` to exactly `2.5`'s bits through `ReferenceEvaluator::standard()`, then evaluates a BF16 program through the **same** evaluator, so the F32 answer is known to come from the widened oracle rather than a narrower one built beside it. The whole workspace suite (2,089 tests) and the release-profile numerical run (717) are green.

### Deliberate failure perturbations

Each was made, run, observed, and reverted.

| Perturbation | Observed |
| --- | --- |
| overflow threshold set to `largest_finite` (saturate a quantum early) | 5 tests fail: the round trip names `0x7f7f`/`0xff7f`, the corpus names the quarter-quantum witness, the boundary test fails its ordering assertion, the tie test reports the normative rule disagreeing, and the parameter pin reports the moved threshold |
| signed-zero sum negative when *either* operand is negative | 2 tests fail, both naming `opposite-signed zeros sum to positive zero` |
| canonical arithmetic NaN payload changed to `0x7fc1` | 6 tests fail, including all 254 NaN round trips, the four NaN witnesses, the constant/arithmetic pair, and the declaration binding |
| subnormal/normal encode boundary moved one quantum (`<` to `<=`) | 6 tests fail, including 508 round-trip encodings and three witnesses across two categories |

`each_unrealizable_declaration_is_refused_by_name` additionally drives eight of the nine `UnsupportedBf16Declaration` variants from perturbed fact records and asserts each refusal by value, with the governed pair accepted first so the refusals are about their own perturbation. `MissingDescriptor` is the one variant no perturbable input reaches — it needs a catalog that stopped registering `tiler::bf16@1` — and the test says so.

### `docs/dtype-support.md`

**Edited, not deferred.** `contracts/navigation` was added to this ticket's `shared_scopes` on 2026-08-01 for exactly this file. The concurrently live contraction work's navigation edits are in `docs/roadmap.md`, which this branch does not touch, so the two are disjoint. Changed here:

- BF16's `Reference evaluation` cell: `absent/unsupported` → `tested guarantee, constant/multiply/add only`.
- A new family-note `Fact` paragraph stating the boundary, the evidence, and what did **not** move.
- The prior `Fact` paragraph's sentence claiming the reference still refuses each key, corrected to point at the new one.
- The dtype-addition recipe's rung-4 dry-run cell, which said BF16's binary32 cross-check does not generalize. The landed reference has no widening cross-check at all, so the cell now states the real F64/F128 obstacle: they are not exhaustively enumerable and need a stated bounded profile.
- Two reproducible searches in the visibility-audit block, whose stated populations this change invalidated: the reference-construction search now spans `bf16.rs`, and the case-insensitive spelling search's "current matches" sentence now names the BF16 registration in `standard.rs`.

**Filed rather than edited.** `correct-the-bf16-reference-evaluation-status-outside-the-dtype-ledger` covers the two remaining stale assertions, both outside this branch's scopes: `docs/roadmap.md`'s R-rung row (held by live navigation work) and `docs/research/numerics/bf16-computation-accumulator-and-conversion.md`'s maturity table (a `research/numerics` scope this ticket does not hold). `tickets/register-the-bf16-semantic-operation-signatures.md` names the test this branch inverted; it is a historical outcome record about its own base commit and is left as written, superseded by this record.

### Public boundary, for Tom

Three additions, none self-accepted:

- `tiler_reference::UnsupportedBf16Declaration`, a `#[non_exhaustive]` enum with nine variants and a `rule()` diagnostic string, shaped after the accepted `UnsupportedContractionDeclaration`.
- `ReferenceRegistryError::UnsupportedBf16 { source }`, a variant on a `#[non_exhaustive]` enum. It deliberately carries no resolved type — the variant names it, and carrying one pushed the enum past `clippy::result_large_err`.
- No new public function or trait. The evaluators, the validator, the format type, and `register_standard_bf16` are all crate-private and follow the existing provider idiom; the family is reached through `ReferenceEvaluator::standard()`.

### Verification

`cargo fmt`; `cargo check -p tiler-reference --all-targets`; `cargo nextest run -p tiler-reference` (185 passed); `cargo clippy -p tiler-reference --all-targets -- -D warnings`; `make full` green end to end — fmt-check, workspace check, workspace clippy, 2,089 workspace tests, 32 doc-tests, rustdoc with warnings denied, 717 release-profile tests, `ticketsplease lint` clean, shellcheck clean. `git diff --check` clean. `tkt guard --base 26266d9 tkt/evaluate-bf16-reference-semantics`: verdict `ok`, no WARNs, declared scopes `contracts/navigation, implementation/reference, project/tickets`.

### Measurement boundary and unsupported cases

Everything above is host evidence about an oracle. No device executed anything, and no BF16 target, kernel, lowering, artifact, or runtime-validation path exists. The oracle covers exactly `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and `tiler::add-bf16@1` over the elementwise shapes those keys' own inferencer admits (equal shapes or one rank-zero operand); a mismatched pair is refused. No conversion between BF16 and F32 is evaluated in either direction, no fused multiply-add or accumulation exists to be exact about, and the declared BF16 accumulator type remains a fact rather than a measured target capability. `admit-bf16-into-the-schedule-and-kernel-vocabulary` now has the oracle it was gated on.
