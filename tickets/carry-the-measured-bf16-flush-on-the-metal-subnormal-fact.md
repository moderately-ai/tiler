---
id: carry-the-measured-bf16-flush-on-the-metal-subnormal-fact
title: Carry the measured bf16 flush on the Metal subnormal fact
status: todo
priority: p1
dependencies: []
related: [carry-the-dtype-on-the-metal-subnormal-flush-fact, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, decide-per-dtype-dispatchability-as-a-target-capability, express-metal-honourability-in-the-shared-form]
scopes: [implementation/metal, research/apple-targets]
shared_scopes: []
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
