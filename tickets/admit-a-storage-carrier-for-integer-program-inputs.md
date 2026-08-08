---
id: admit-a-storage-carrier-for-integer-program-inputs
title: Admit a storage carrier for integer program inputs
status: blocked
priority: p1
dependencies: [admit-an-indirect-gather-family-for-tied-embedding-lookup, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, enumerate-the-mature-tensor-dtype-taxonomy, route-an-embedded-artifact-through-a-consumer-storage-seam, admit-the-bf16-type-and-carrier-into-every-total-map]
scopes: [implementation/ir, implementation/artifact, implementation/frontend, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, abi, frontend, gather, language-model, class-generic-capability]
claimed_from: todo
assignee: w-admit-a-s
lease_expires_at: 1786158471
---
## User-visible outcome

A `[T]` token-ID operand reaches a program as an integer, so the one operation between a model's inputs and its logits is fed by a value whose type says what it is.

## Why this exists

**Fact, re-measured 2026-08-07 at base `68f1ced6`.** The semantic layer already registers integer identities: `crates/tiler-ir/src/semantic/catalog.rs` registers `tiler::u8@1`, `tiler::u16@1`, `tiler::u32@1`, `tiler::i32@1`, and `tiler::i64@1` under ADR 0028, so a `[T]` index operand at `tiler::u32@1` has an admitted identity today. The registry is wider than that list: `bool`, `i2`, `i4`, `i8`, `i16`, `u2`, `u4`, and `u64` are registered beside them. Reproduce with `grep -n 'ADR 0028; tiler::' crates/tiler-ir/src/semantic/catalog.rs`.

**Fact, corrected 2026-08-07 — the previous wording was false in both its count and its citation.** It read: "`StorageScalar` at `crates/tiler-ir/src/program/model.rs:264` has exactly two variants, `U8` and `F32`." At `68f1ced6` the enum is at `model.rs:342` and has **three** variants — `U8`, `F32`, and `Bf16` — the third landed by [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md). Line 264 is inside `StorageEncoding`, an unrelated type. Reproduce with `grep -n 'pub enum StorageScalar' -A 16 crates/tiler-ir/src/program/model.rs`. The claim that no carrier exists for an integer *wider than a byte* still stands, which is why this ticket survives the correction.

**Inference.** The pinned workload's vocabulary is 151,936, which needs eighteen bits, so `U8` cannot carry a token ID. `F32` represents every integer below 2^24 exactly and would therefore work, which is why this is a decision rather than a gap.

## The decision — the ABI-cost half of the fork is measured and false

**Measurement, 2026-08-07 at `68f1ced6`, and it collapses the fork the previous body posed.** The "widen the carrier" arm was costed as "an artifact-ABI change: `StorageScalar::tag` participates in canonical encoding, and `natural_access_type` maps into the structured-kernel vocabulary, so both move." **Both encoders are append-only, so nothing already encodable moves.** `StorageScalar::tag` at `model.rs` carries the precedent verbatim for `Bf16 => 0x03`: "`U8` and `F32` keep their tags and every field keeps its position, so no previously encodable program's bytes move and the program identity domain does not step."

Measured rather than predicted: with `StorageScalar::U32` appended at tag `0x04`, `cargo nextest run -p tiler-ir -p tiler-artifact -p tiler-macros -p tiler -p tiler-compiler` ran **2213 tests, 2211 passed**. **No golden, pin, or identity test moved.** The two failures are both deliberate widening tripwires that a landing must retarget, not evidence of drift:

- `tiler-artifact program::codec::tests::an_unassigned_carrier_or_access_tag_is_refused_before_its_width_is_used` — `const UNASSIGNED_CARRIER: u8 = 0x04` is the tag the test forges to prove an unrecognized carrier is refused by name, and `0x04` is exactly the tag an appended `U32` claims. Fails with "the carrier vocabulary must not have grown into the tag this case perturbs to". A landing moves it to `0x05`. `const UNASSIGNED_ACCESS: u8 = 0x07` collides the same way with an appended `KernelType::U32`.
- `tiler-ir program::model::injectivity_tests::the_storage_scalar_encoding_is_injective_over_its_whole_domain` — `assert_eq!(STORAGE_SCALARS.len(), 3)`; `left: 4, right: 3`. The array is sized by `variant_count::<StorageScalar>()`, so the widening is a build error at the array and an assertion failure at the count, exactly as designed.

**So the artifact-ABI objection does not survive contact with the code, and the remaining decision is narrower than the one stated for Tom.** What is left to decide is not "widen versus carry as `F32`" on ABI cost — it is whether the carrier's kernel access type is honest, which is the next section.

## The real blocker: `natural_access_type` has no honest target, and fixing that leaves this ticket's scopes

**Fact.** `StorageScalar::natural_access_type` is a total map into `KernelType`, and every existing carrier pairs 1:1 and width-exactly: `U8 -> U8`, `F32 -> F32`, `Bf16 -> Bf16`. `crates/tiler-artifact/src/program/codec/validate.rs`'s `check_binding_access` restates the same pairing with no wildcard, because "a slot whose carrier is two bytes wide and whose access type is four would address twice the bytes the interface provides".

**Fact.** There is no `KernelType::U32`. The vocabulary at `crates/tiler-ir/src/kernel/model.rs` is `Bool`, `U8`, `Index`, `F32`, `I32`, `Bf16`. `Index` is unsigned but eight bytes wide, so pairing a four-byte carrier with it is the exact width misread `check_binding_access` exists to refuse.

**Inference — pairing `StorageScalar::U32` with `KernelType::I32` is the defect, not the cheap option.** The widths match, so every check in the stack passes. What it buys is a token-ID carrier read as signed at the one operation whose out-of-range behaviour reads out of bounds — which is the failure ADR 0107 and the L6 record both name as the thing the integer carrier exists to prevent. Under `AGENTS.md`, "a cheaper path that can silently return wrong results is a defect, not a trade-off". So an honest landing needs `KernelType::U32` beside `StorageScalar::U32`.

**Measurement — the complete site enumeration, by the method [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md) established.** Both variants appended, then `CARGO_TARGET_DIR=./target cargo check --workspace --all-targets` iterated to green over eight rounds. Cargo halts at the first failing crate, so the rounds are the enumeration.

| Site (symbol) | File | Scope | Declared? |
| --- | --- | --- | --- |
| `StorageScalar` variant, `tag`, `byte_width`, `natural_access_type` | `crates/tiler-ir/src/program/model.rs` | `implementation/ir` | yes |
| `STORAGE_SCALARS` (`variant_count`-sized) | `crates/tiler-ir/src/program/model.rs` | `implementation/ir` | yes |
| `KernelType` variant, `KernelType::tag` | `crates/tiler-ir/src/kernel/model.rs` | `implementation/ir` | yes |
| `element_bytes`, `push_element_type`, `ELEMENT_TYPES` | `crates/tiler-ir/src/program/model.rs` | `implementation/ir` | yes |
| `storage_scalar_tag`, `storage_scalar_from_tag` | `crates/tiler-artifact/src/program/model.rs` | `implementation/artifact` | yes |
| `element_type_tag`, `element_type_from_tag` | `crates/tiler-artifact/src/program/model.rs` | `implementation/artifact` | yes |
| `check_binding_access` | `crates/tiler-artifact/src/program/codec/validate.rs` | `implementation/artifact` | yes |
| `every_governed_tag_table_round_trips` | `crates/tiler-artifact/src/program/codec/tests.rs` | `implementation/artifact` | yes |
| `storage_scalar_path` | `crates/tiler-macros/src/binding.rs` | `implementation/frontend` | yes |
| **`every_storage_carrier_has_a_representable_alignment`** (test) | **`crates/tiler-compiler/src/boundary.rs`** | **`implementation/compiler`** | **no** |
| **`index_arithmetic_requirement`** (lib) | **`crates/tiler-compiler/src/physical.rs`** | **`implementation/compiler`** | **no** |
| **`msl_type`** (lib) | **`crates/tiler-metal/src/emit.rs`** | **`implementation/metal`** | **no** |

**Fact — so this ticket cannot be landed under its declared scopes, and the reason is structural rather than a matter of effort.** Both widened enums are deliberately not `#[non_exhaustive]`; every site above is a wildcard-free match that ADR 0074 convention 5b requires to be a build error. There is no ordering of these edits that leaves the workspace compiling in between, so they are one commit or none — the same argument the BF16 ticket made, which is why that ticket declared `implementation/compiler` and `implementation/metal` and this one must too.

**Even the dishonest `KernelType::I32` variant does not fit the declared scopes**: it still needs `crates/tiler-compiler/src/boundary.rs`. The compiler scope is unavoidable either way.

**Blocked, not deferred.** `implementation/compiler` was held by a live exclusive claim (`answer-input-element-counts-as-the-declared-tensors-own-count`) when this was audited on 2026-08-07. `implementation/metal` was unheld. Adding a scope required by authorized work is scheduling metadata under `AGENTS.md`, but taking one that another live worker holds exclusively is a coordination decision, not a worker's.

## What a redispatch needs

- Scopes `[implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/frontend, contracts/artifacts]` — the BF16 ticket's exact set, and free simultaneously.
- A decision on `msl_type(KernelType::U32)`: spell `uint`, or refuse by name as BF16 does. The BF16 precedent refused, because a spelling that compiles while the numerics behind it are absent is worse than an explicit refusal. A `u32` index carrier has no numerics behind it to be absent, so the two cases may differ; this is a real question and not a formality.
- `cargo check --workspace --all-targets` does **not** reach `trybuild` fixtures, which compile at test *run* time. The BF16 landing found a site that only appeared under `cargo nextest`. The enumeration above is complete for `check` and unverified for `trybuild`.

## Closes when

The carrier question is answered with its consequence stated, the answer is implemented, an index operand's stored type is checked at the bind boundary, and a value of the wrong stored type refuses by name rather than being reinterpreted.

## Coordinator verification and the redispatch conditions, 2026-08-07

Merged at `435bd0d5`'s successor. **No code landed and that was correct** — the honest landing is one commit spanning six scopes or none, and three of them were undeclared.

### Everything material re-verified independently

- **`KernelType` has no `U32`.** `crates/tiler-ir/src/kernel/model.rs`, `enum KernelType`: `Bool`, `U8`, `Index`, `F32`, `I32`, `Bf16`. Confirmed by reading.
- **The two widening tripwires are real and sit where reported.** `crates/tiler-artifact/src/program/codec/tests.rs`, `UNASSIGNED_CARRIER = 0x04` and `UNASSIGNED_ACCESS = 0x07` — precisely the tags an appended `StorageScalar::U32` and `KernelType::U32` claim. A landing must retarget both; they are doing exactly the job they were written for.
- **All three out-of-scope sites exist** at the named functions: `physical.rs`'s `index_arithmetic_requirement`, `tiler-metal`'s `msl_type`, and `boundary.rs`'s `every_storage_carrier_has_a_representable_alignment`.
- **The pinned-identity measurement stands**: appending `U32` at `0x04` moved **no golden, pin, or identity test** across 2,213 tests in five packages. Empirical, bounded to that base.

### The blocker is real and not routable around

`natural_access_type` is a width-exact 1:1 map into `KernelType`. Pairing a token-ID carrier with `I32` would read it **signed** at the one operation whose out-of-range behaviour reads out of bounds — the defect ADR 0107 exists to prevent, since gather bounds are a semantic precondition never clamped and never wrapped. So `KernelType::U32` is required, and with it `implementation/compiler` and `implementation/metal`.

### The `msl_type` question is decided here, not escalated

The worker asked whether `msl_type(KernelType::U32)` should spell `uint` or refuse by name, noting the BF16 precedent may not transfer since BF16 refused for absent *numerics* and a `u32` index carrier has none to be absent.

**Refuse by name.** It eliminates under AGENTS.md rather than surviving as a genuine fork: no index-layer access class exists, so nothing can produce a kernel holding a `U32`-typed value, and emitting `uint` would be an unexercised path asserting a backend capability nothing demonstrates. AGENTS.md prefers typed, explainable failure over a silently wrong fast path, and requires maturity claims to track demonstrated support. The refusal's `reason` must name **what lifts it** — the first backend consumer producing a `U32`-typed kernel value — so it reads as a stated boundary rather than an omission. This differs from BF16's ground and the comment should say so rather than citing the precedent as if it transferred.

### Release trigger for redispatch

**All six scopes free simultaneously**: `implementation/ir`, `implementation/artifact`, `implementation/frontend`, `contracts/artifacts`, `implementation/compiler`, `implementation/metal`. Add the last two to this ticket at redispatch — scheduling metadata under AGENTS.md, and explained here.

Recheck with `tkt claims` plus a scope scan of live `tkt/` branches. At the time of writing, `implementation/compiler` was held by an exclusive claim on `answer-input-element-counts-as-the-declared-tensors-own-count` and `implementation/metal` was free but undeclared.

The redispatch brief must carry: the corrected `StorageScalar` Fact (**three** variants at `enum StorageScalar`, not two — the ticket's own repro command was self-falsifying, printing `Bf16` inside its own `-A 6` window), the append-not-move finding for both tag encoders, the two tripwires to retarget, and the `msl_type` decision above.
