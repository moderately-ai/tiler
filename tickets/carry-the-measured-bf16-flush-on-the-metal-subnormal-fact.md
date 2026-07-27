---
id: carry-the-measured-bf16-flush-on-the-metal-subnormal-fact
title: Carry the measured bf16 flush on the Metal subnormal fact
status: done
priority: p1
dependencies: []
related: [carry-the-dtype-on-the-metal-subnormal-flush-fact, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, decide-per-dtype-dispatchability-as-a-target-capability, express-metal-honourability-in-the-shared-form]
scopes: [implementation/metal, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, dtype]
---
Two tickets landed within an hour of each other from the same base and compose correctly but incompletely: the fact learned to carry a dtype, and a third dtype was measured, and nothing joined them.

**Fact — `crates/tiler-metal/src/target.rs:277-282`.** `MetalFloatArithmeticType` names `F32` and `F16` only.

**Fact — the measurement exists.** `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes` (done, commit `2c8f973`) establishes that `bf16` **flushes** on the Mac GPU, across all seven flush dimensions, under `safe`/`relaxed`/`fast`, at `-O0` and `-O2`, on both compilation paths, every verdict carrying an execution witness reporting `executed`.

**Inference — the current behaviour is correct, and that is why this is p1 rather than p0.** `bf16` is not a silently wrong answer today; it is `Unstated`, and `carry-the-dtype-on-the-metal-subnormal-flush-fact` made an unstated dtype a *third class* that fails closed with `MetalEmitError::UnstatedSubnormalArithmetic` before any gap is computed. So the defect is that measured evidence is sitting outside the type that exists to hold it, not that a consumer can be misled.

## What the row must record, beyond the behaviour

**The result is macOS-only and the record must not lose that.** The iOS Simulator compiled *and linked* every `bf16` module, then failed `newComputePipelineStateWithFunction:` with `XPC_ERROR_CONNECTION_INTERRUPTED` — both compilation paths, including the arithmetic-free kernel — while running every `f32` and `f16` kernel in the same invocation. `bf16` is therefore `Unknown` for both iOS families, with the cause unmeasured rather than guessed. A row that states "bf16 flushes" without its family bound would be a portable guarantee derived from one tested host, which `AGENTS.md` forbids in terms.

**Family scoping already exists at the enclosing authority.**
`MetalSubnormalArithmeticFacts` belongs to one `MetalTargetFacts`, and
`MetalTargetFacts` already names its `MetalPlatform`. Add `Bf16` to the
arithmetic-type vocabulary and state the measured flush behavior only in the
macOS target-fact rows whose evidence supports it. Leave iOS device and
simulator rows unstated (`Unknown`); do not duplicate the platform dimension
inside each dtype row.

**`decide-per-dtype-dispatchability-as-a-target-capability` (`contracts/decisions`) owns the adjacent contract question** raised by a device that refuses a dtype at pipeline creation. Do not decide it here; state what this row assumes and link it.

## What this ticket must not do

**Do not add `f64`, integer, or quantized rows.** They are explicitly `Unknown`, and the measurement's own finding 24 is that a neighbour predicts nothing about a dtype — `f32` flushing and `f16` preserving is precisely why. Adding an unmeasured row would convert `Unknown` into a claim, and `AGENTS.md` keeps `Unknown` a distinct class from empirical evidence and from a normative guarantee.

**Do not restate the mechanism as settled beyond what was shown.** The `bf16` result refutes native subnormal support in narrow formats and leaves one mechanism covering all three dtypes with no free parameter. It does **not** separate that mechanism from native `bfloat16` arithmetic flushing at its own boundary — those agree on every operand a single operation can supply, because 24 bits exceeds the 18 that would make a second rounding to `bfloat16`'s 8-bit significand innocuous. A two-operation `bf16` chain with a rounding-sensitive intermediate would separate them, and no such measurement exists.

**Expect call sites not to need changes, and verify rather than assume it.** The facts are built from `unmeasured()` via `stating(type, behaviour)`, so an existing constructor that never mentions `bf16` should keep compiling and leave it unstated. If adding the variant does force `MetalTargetFacts::new` sites in `prototypes/serial-sum-{compile,run}` to change, those are `implementation/metal-aot` and `implementation/runtime` — scopes this ticket does not hold. Split rather than escaping.

## Closes when

The measured `bf16` behaviour is expressible and expressed with its family bound intact, every unnamed dtype still rejects rather than defaulting, no unmeasured row was added, and the full gate passes.

## Outcome

Done. `MetalFloatArithmeticType::Bf16` exists, the measured macOS row is stated, and the two iOS families stay `Unknown`.

**The row, and why its zero is measured rather than assumed.** `FlushesToZero { zero_sign: PreservesSign }`. The sign is not inferred from the `f32` row beside it: finding 24's table records `8040` → `8000` on the multiply sign dimension and again on the division sign dimension, so the flush is measured to preserve sign at `bfloat16` width. Stating `PreservesSign` without those two rows would have been exactly the neighbour-reading this record exists to prevent.

**Family scoping was kept at the enclosing authority, as the ticket asked.** The row is stated in `crates/tiler-metal`'s two `MacOs` fixtures — `tests.rs::subnormal_facts` and `golden_compilation.rs::emitter_facts` — and nowhere else. No platform dimension was added inside a dtype row. `bf16` is `Unknown` for both iOS families for two different reasons, and the type's documentation now records both: the Simulator compiles and links every `bfloat` module and then refuses to create a pipeline for it, including for an arithmetic-free `materialize_bf16`, so the refusal is about the format rather than an operation; `IOsDevice` was never asked.

**Call sites were verified, not assumed.** The ticket flagged that forcing changes in `prototypes/serial-sum-{compile,run}` would be a scope escape. It does not: all four out-of-crate construction sites build facts through `unmeasured().stating(…)`, which is additive, and nothing outside `crates/tiler-metal/src/target.rs` matches the enum exhaustively. `cargo check --workspace --all-targets` passes with no edit outside this ticket's scope.

**What did change, inside scope, is the four goldens** — one line each. `emit.rs` prints one header line per member of `MetalFloatArithmeticType::ALL`, including `not stated`, so every emission gains `//   bf16: flushes-to-zero-preserving-sign`. They were regenerated by inserting exactly that line rather than by overwriting: `git diff --stat` shows `4 files changed, 4 insertions(+)`, and the byte-comparison tests then pass, which is what proves emission is otherwise unchanged. `golden_compilation` recompiles all four through the real driver and still does.

**No unmeasured row was added.** `f64` and every integer and quantized format remain absent, and the type's "what three dtypes establish" section says explicitly that a fourth dtype could agree with any measured one.

**The mechanism is recorded as narrowed, not settled.** The wider-internal-precision explanation now accounts for all three measured dtypes with no free parameter and the competing one survives only weakened to a per-format claim with no independent evidence. But it is *not* separated from native `bfloat16` arithmetic flushing at its own boundary, and no single operation can separate them — a value is `bf16`-subnormal exactly when it is `f32`-subnormal, and `f32`'s 24-bit significand exceeds the 18 bits that would make a second rounding to `bfloat16`'s 8-bit significand innocuous. A two-operation chain with a rounding-sensitive intermediate would, and none has been measured. The documentation says that rather than rounding the inference up to a rule.

**Evidence.** `bf16_is_unknown_until_it_is_stated_even_beside_an_identical_f32_fact` asserts the new slot inherits nothing. It is worth having precisely because the measured `bf16` and `f32` behaviours are the *same value*: a record answering `bf16` from the `f32` entry would look right on this row and be a guess, and would be wrong half the time on a vocabulary where `f16` preserves. `every_arithmetic_type_indexes_to_its_own_slot` already exercised the new variant automatically and proves the index map is still a bijection onto `0..COUNT`.

Gate: `make full` green (975 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).
