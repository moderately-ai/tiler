---
id: admit-a-storage-carrier-for-integer-program-inputs
title: Admit a storage carrier for integer program inputs
status: done
priority: p1
dependencies: [admit-an-indirect-gather-family-for-tied-embedding-lookup, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, enumerate-the-mature-tensor-dtype-taxonomy, route-an-embedded-artifact-through-a-consumer-storage-seam, admit-the-bf16-type-and-carrier-into-every-total-map, admit-an-invocation-scoped-gather-index-validation-receipt]
scopes: [implementation/ir, implementation/artifact, implementation/frontend, contracts/artifacts, implementation/compiler, implementation/metal, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, abi, frontend, gather, language-model, class-generic-capability, trigger-fired, decision, needs-tom, public-boundary]
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
| `IndexArithmetic::of` (KernelType → index-arithmetic total map) | `crates/tiler-ir/src/kernel/model.rs` | `implementation/ir` | yes |
| `storage_scalar_tag`, `storage_scalar_from_tag` | `crates/tiler-artifact/src/program/model.rs` | `implementation/artifact` | yes |
| `element_type_tag`, `element_type_from_tag` | `crates/tiler-artifact/src/program/model.rs` | `implementation/artifact` | yes |
| `check_binding_access` | `crates/tiler-artifact/src/program/codec/validate.rs` | `implementation/artifact` | yes |
| `every_governed_tag_table_round_trips` | `crates/tiler-artifact/src/program/codec/tests.rs` | `implementation/artifact` | yes |
| `storage_scalar_path` | `crates/tiler-macros/src/binding.rs` | `implementation/frontend` | yes |
| **`every_storage_carrier_has_a_representable_alignment`** (test) | **`crates/tiler-compiler/src/boundary.rs`** | **`implementation/compiler`** | **yes** |
| **`msl_type`** (lib) | **`crates/tiler-metal/src/emit.rs`** | **`implementation/metal`** | **yes** |

The Declared? column matches frontmatter after the 2026-08-09 scope repair (all six implementation/contract scopes plus `contracts/decisions`). Earlier board prose that marked compiler/metal rows `no` is historical. `crates/tiler-compiler/src/physical.rs`'s `index_arithmetic_requirement` matches only `IndexArithmetic` (today a single arm on `CompleteU64`); it is **not** a `KernelType` total map and is not broken by appending `KernelType::U32`. The KernelType→index-arithmetic classifier is `IndexArithmetic::of` in IR (row above).

**Fact (historical as of the pre–scope-repair body; scopes are now declared).** The original site census found compiler and metal total-map sites outside the then-declared scopes, and the reason those sites matter is structural rather than a matter of effort: both widened enums are deliberately not `#[non_exhaustive]`; every site above is a wildcard-free match that ADR 0074 convention 5b requires to be a build error. There is no ordering of these edits that leaves the workspace compiling in between, so they are one commit or none — the same argument the BF16 ticket made, which is why that ticket declared `implementation/compiler` and `implementation/metal` and this one now does too (frontmatter already lists them).

**Even the dishonest `KernelType::I32` variant still needs compiler scope**: it still needs `crates/tiler-compiler/src/boundary.rs`. The compiler scope is unavoidable either way.

**Historical scheduling blocker, cleared.** `implementation/compiler` was held by a live exclusive claim (`answer-input-element-counts-as-the-declared-tensors-own-count`) when this was audited on 2026-08-07. `implementation/metal` was unheld. The 2026-08-09 board repair found no live claims and added both required scopes. Dependency and scope holds from that period are cleared; the live board state is `awaiting-decision` on the public surface (see Public-boundary correction below), not `blocked` on scopes or dependencies.

## What a redispatch needs

- Scopes `[implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/frontend, contracts/artifacts]` — the BF16 ticket's exact set, already declared on this ticket, and free simultaneously at dispatch time.
- A decision on `msl_type(KernelType::U32)`: spell `uint`, or refuse by name. **Resolved below: refuse by name** until a backend consumer produces a `U32`-typed kernel value; the refuse seam is `MetalEmitError::UnsupportedValueType`. Do **not** cite a live BF16 `msl_type` refusal — at the current tree `msl_type(KernelType::Bf16)` spells `Ok("bfloat")`. U32's refusal stands on maturity/support (no producer of a U32-typed kernel value yet), not on the BF16 numerics ground that once refused BF16 spelling.
- `cargo check --workspace --all-targets` does **not** reach `trybuild` fixtures, which compile at test *run* time. The BF16 landing found a site that only appeared under `cargo nextest`. The enumeration above is complete for `check` and unverified for `trybuild`.

## Closes when

The carrier question is answered with its consequence stated, the answer is implemented, an index operand's stored type is checked at the bind boundary, and a value of the wrong stored type refuses by name rather than being reinterpreted.

## Coordinator verification and the redispatch conditions, 2026-08-07

Merged at `435bd0d5`'s successor. **No code landed and that was correct** — the honest landing is one commit spanning six scopes or none, and three of them were undeclared.

### Everything material re-verified independently

- **`KernelType` has no `U32`.** `crates/tiler-ir/src/kernel/model.rs`, `enum KernelType`: `Bool`, `U8`, `Index`, `F32`, `I32`, `Bf16`. Confirmed by reading.
- **The two widening tripwires are real and sit where reported.** `crates/tiler-artifact/src/program/codec/tests.rs`, `UNASSIGNED_CARRIER = 0x04` and `UNASSIGNED_ACCESS = 0x07` — precisely the tags an appended `StorageScalar::U32` and `KernelType::U32` claim. A landing must retarget both; they are doing exactly the job they were written for.
- **Widening sites that still require compiler and metal scopes** (scopes now declared): `tiler-metal`'s `msl_type` and `boundary.rs`'s `every_storage_carrier_has_a_representable_alignment`. The KernelType index-arithmetic total map is `IndexArithmetic::of` in `crates/tiler-ir/src/kernel/model.rs` (`implementation/ir`), not `physical.rs`'s `index_arithmetic_requirement` (that function matches only `IndexArithmetic` and is not broken by `KernelType::U32`).
- **The pinned-identity measurement stands as historical evidence at `68f1ced6`**: appending `U32` at `0x04` moved **no golden, pin, or identity test** across 2,213 tests in five packages. Empirical, bounded to that base; redispatch must re-measure on the land base rather than treat the count as live.

### The blocker is real and not routable around

`natural_access_type` is a width-exact 1:1 map into `KernelType`. Pairing a token-ID carrier with `I32` would read it **signed** at the one operation whose out-of-range behaviour reads out of bounds — the defect ADR 0107 exists to prevent, since gather bounds are a semantic precondition never clamped and never wrapped. So `KernelType::U32` is required, and with it `implementation/compiler` and `implementation/metal`.

### The `msl_type` question is decided here, not escalated

The worker asked whether `msl_type(KernelType::U32)` should spell `uint` or refuse by name, noting the BF16 precedent may not transfer since BF16 refused for absent *numerics* and a `u32` index carrier has none to be absent.

**Refuse by name.** It eliminates under AGENTS.md rather than surviving as a genuine fork: no index-layer access class exists, so nothing can produce a kernel holding a `U32`-typed value, and emitting `uint` would be an unexercised path asserting a backend capability nothing demonstrates. AGENTS.md prefers typed, explainable failure over a silently wrong fast path, and requires maturity claims to track demonstrated support. Use the existing `MetalEmitError::UnsupportedValueType` seam; the refusal's `reason` must name **what lifts it** — the first backend consumer producing a `U32`-typed kernel value — so it reads as a stated boundary rather than an omission. Do not model this after a live BF16 refusal arm: `msl_type(KernelType::Bf16)` now returns `Ok("bfloat")`. U32's refuse-by-name decision stands on its own maturity ground, not on BF16's former numerics refusal.

### Release trigger for redispatch

**The six required scopes are now declared**: `implementation/ir`, `implementation/artifact`, `implementation/frontend`, `contracts/artifacts`, `implementation/compiler`, `implementation/metal`. Redispatch waits for the two frontmatter dependencies to complete and for those scopes to be conflict-free at dispatch time; it no longer waits for someone to repair this ticket's own declaration.

Recheck with `tkt claims` plus a scope scan of live `tkt/` branches. At the original audit, `implementation/compiler` was held by an exclusive claim on `answer-input-element-counts-as-the-declared-tensors-own-count`; on 2026-08-09 `tkt claims --format json` reported no live claims after expired claim metadata was released.

The redispatch brief must carry: the corrected `StorageScalar` Fact (**three** variants at `enum StorageScalar`, not two — the ticket's own repro command was self-falsifying, printing `Bf16` inside its own `-A 6` window), the append-not-move finding for both tag encoders, the two tripwires to retarget, and the `msl_type` decision above.

## Unblocked 2026-08-09

Both declared dependencies are `done`: [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) and [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md). `tkt claims --format json` reports no live claim, all six required scopes are already declared, and the only implementation choice this record left open — `msl_type(KernelType::U32)` — was resolved above as a named refusal until a backend consumer exists. Moved `blocked` → `todo`; the widening remains one coherent cross-scope landing with the stated trybuild audit and perturbations, not a partial carrier edit.

## Public-boundary correction — 2026-08-09

Clearing dependencies did not accept the two public enum additions. Tom must accept the exact coherent surface before implementation: append `StorageScalar::U32` at tag `0x04`, append `KernelType::U32` at the next unclaimed tag, preserve width-exact binding, and make Metal refuse the kernel type by name until a real backend consumer exists. Recommendation: accept that exact honest carrier/access pair; `F32`, `I32`, and eight-byte `Index` are all semantically wrong substitutes. The ticket is therefore `awaiting-decision`, not dependency-blocked and not yet implementation-ready.

## Accepted — 2026-08-11

Tom accepted the recommended honest carrier/access pair in the Codex coordination thread by replying `sounds good, accept`. The relay source is Tom's direct response to the ranked decision packet; this acceptance moves the ticket to `todo` and does not claim implementation.

The exact accepted surface is append-only `StorageScalar::U32` at tag `0x04` and append-only `KernelType::U32` at the next unclaimed tag. Binding remains width- and type-exact: a U32 semantic input must be backed by U32 storage and read through a U32 kernel access type. `F32`, `I32`, `U8`, and the eight-byte `Index` type are not aliases or fallbacks, and no implicit conversion, reinterpretation, narrowing, or widening is admitted.

Metal must initially refuse `KernelType::U32` through the existing typed `UnsupportedValueType` path. The refusal names the missing maturity condition: an admitted backend consumer that produces a U32-typed kernel value and supplies direct emission evidence. No unexercised `uint` spelling lands merely because MSL can state one. A later gather-lowering vertical may lift this refusal only with its own tested producer, exact binding, and public-boundary consequences.

This storage decision does not discharge gather bounds. The invocation-scoped preflight ticket still validates every host-visible index against the gathered extent, seals the immutable snapshot, and refuses missing, stale, mismatched, or out-of-range evidence before routing commit. No carrier choice creates a clamp, wrap, unchecked execution, plan substitution, reference execution, or backend fallback.

The broader parameterized-integer carrier is deferred until a second concrete integer-storage use establishes a shared vocabulary. The smaller exact pair is preferred now because it is semantically honest, preserves all existing encoded bytes, matches the four-byte input representation, and keeps unsupported backend execution fail-closed.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Ticket-audit wave re-read the site census against the live tree and fixed three stale present-tense claims that would misroute a redispatch brief:

1. **`IndexArithmetic::of` is the KernelType → index-arithmetic total map**, at `crates/tiler-ir/src/kernel/model.rs` under `implementation/ir`. The former table row naming `index_arithmetic_requirement` in `crates/tiler-compiler/src/physical.rs` under `implementation/compiler` was wrong for a `KernelType::U32` widening: that function matches only `IndexArithmetic` (single arm `CompleteU64`) and is not broken by appending a kernel type. The site table above now lists `IndexArithmetic::of` instead.
2. **Declared? on the compiler and metal rows was stale.** Frontmatter already declares `implementation/compiler` and `implementation/metal` (post–2026-08-09 repair). The table now says **yes**; the earlier "cannot be landed under its declared scopes" sentence is historical of the pre-repair declaration, not of the current board state.
3. **BF16 `msl_type` spelling is live** (`Ok("bfloat")`); there is no present BF16 refuse arm to cite as precedent. U32 refuse-by-name remains this ticket's decision, discharged via `UnsupportedValueType` until a backend consumer produces a U32-typed kernel value. Status, dependencies, and scopes needed no metadata change; outcome still undelivered (no `StorageScalar::U32` / `KernelType::U32`).

## Exact-base Fact repair — 2026-08-17

**Correction at implementation base `b9d53e8d`.** The site table's claim to be complete became imprecise after the typed alignment authority was centralized. In addition to the compiler-side `every_storage_carrier_has_a_representable_alignment`, `crates/tiler-ir/src/program/alignment.rs` now owns a `variant_count::<StorageScalar>()`-sized `CARRIERS` census and a wildcard-free match in its own test of the same name. An honest widening must extend both censuses. Reproduce with `rg -n 'variant_count::<StorageScalar>|every_storage_carrier_has_a_representable_alignment' crates/tiler-ir/src/program/alignment.rs crates/tiler-compiler/src/boundary.rs`.

The earlier instruction that the Metal refusal's `reason` must name what lifts it was also imprecise about the accepted existing error shape. `MetalEmitError::UnsupportedValueType` carries only `value_type`; this ticket did not accept a new public error field. `msl_type(KernelType::U32)` therefore uses that existing typed variant unchanged, while the lifting condition remains stated in the `msl_type` arm documentation and its direct refusal test. Reproduce the current shape with `rg -n 'UnsupportedValueType|pub\(crate\) const fn msl_type' crates/tiler-metal/src/diagnostic.rs crates/tiler-metal/src/emit.rs`.

Neither repair changes this ticket's purpose, accepted public enum pair, identity requirement, or fail-closed Metal boundary. The 2026-08-07 append-only test count remains historical evidence at `68f1ced6`; identity nonmovement must still be re-measured against this implementation base.

**Correction from the coherent-pair audit.** The historical paragraph describing "two failures" covered the partial carrier-only experiment at `68f1ced6`; it is not a complete prediction for the accepted pair at this base. Appending both variants makes three deliberate widening checks say no before they are retargeted: the artifact unknown-tag control collides with both new tags in one test, while the program identity censuses independently report the element-type population growing 6 → 7 and the storage-scalar population growing 3 → 4. The landing retargets the unknown tags to carrier `0x05` and access `0x08`, and resizes both censuses from their types. Reproduce the three subjects with `cargo nextest run -p tiler-ir -p tiler-artifact -E 'test(the_program_element_type_encoding_is_injective_over_its_whole_domain) | test(the_storage_scalar_encoding_is_injective_over_its_whole_domain) | test(an_unassigned_carrier_or_access_tag_is_refused_before_its_width_is_used)'` after temporarily omitting each corresponding census/tag update.

## Implementation evidence — 2026-08-17

**Implementation subject.** Commit `bc0b7c0ea07c6f001da1f67ae0901d86015fe820`, with exact parent/base `b9d53e8de52960d4a5207c4bde53d7068951532b`, delivers the accepted coherent pair. It appends public `StorageScalar::U32` at tag `0x04`, byte width four, with exact natural access `KernelType::U32`; appends public `KernelType::U32` at tag `0x07`; and makes `IndexArithmetic::of(KernelType::U32)` return `None`. The program and artifact encoders, forward/inverse tables, variant-count-sized censuses, alignment authorities, frontend spelling, and exact bind boundary all carry the new pair. Unpacked U32 storage round-trips only through U32 access. Equal-width I32/F32 access, an F32 stored neighbour, and bit-packed U32 refuse rather than alias or reinterpret.

**Unsupported boundary.** This is a physical program-input carrier and exact access type only. It adds no integer arithmetic, conversion, reinterpretation, widening, narrowing, gather-bounds discharge, gather lowering, backend producer, target dispatchability, runtime execution, or generalized integer carrier. Metal directly returns `MetalEmitError::UnsupportedValueType { value_type: KernelType::U32 }`; it does not emit `uint`. That refusal can be lifted only by an admitted backend consumer producing a U32-typed kernel value with direct emission evidence.

**Identity and schema nonmovement.** Existing tags and canonical bytes remain pinned, while the new exact bytes are independently pinned in the IR and artifact tables. Unknown-tag controls move to carrier `0x05` and access `0x08`. Exact-base diff inspection changes no identity-domain or schema constant: the kernel, kernel-program, artifact, and manifest remain `tiler.kernel.v8`, `tiler.kernel-program.v12`, `tiler.artifact-program.v18`, and schema `18.0`. The complete affected test run retains all earlier identity and golden pins.

**Positive gates.** The focused U32 selection ran 16 tests and passed 16. The final affected-package run — `tiler-ir`, `tiler-artifact`, `tiler-macros`, `tiler`, `tiler-compiler`, and `tiler-metal`, including their UI/trybuild suites — ran 2,803 tests, passed 2,803, and skipped 3. `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, affected-package `cargo clippy --all-targets -- -D warnings`, workspace rustdoc with `RUSTDOCFLAGS='-D warnings'`, workspace doctests with warnings denied, `tkt lint --format json`, `make citations`, and `git diff --check` all passed.

The deliberately broader `cargo clippy --workspace --all-targets -- -D warnings` reached three pre-existing warnings in the untouched and crate-style-gate-excluded `prototypes/serial-sum-run/src/proof.rs`: one `redundant_closure_for_method_calls` and two `err_expect` findings. `git diff b9d53e8d..bc0b7c0e -- prototypes/serial-sum-run/src/proof.rs` is empty. The six affected packages pass Clippy across all targets with warnings denied.

**Independent negative controls, all restored without changing their checks.** Each subject perturbation made its intended check say no:

- Reusing the U32 storage tag as `0x03` failed with `left: [1, 2, 3, 3]`, `right: [1, 2, 3, 4]`.
- Reusing the U32 kernel tag as `0x06` failed with `KernelType::tag tag 0x06 is shared by U32 and Bf16`.
- Pairing U32 storage naturally with I32 failed with `left: I32`, `right: U32`.
- Making Metal spell U32 as `uint` failed with `U32 has no Metal producer yet: "uint"`.
- Omitting U32 from a type-sized carrier census failed to compile with `expected an array with a size of 4, found one with a size of 3`.
- Admitting U32 storage through I32 access made the refusal test report `left: Ok(ArtifactEnvelope { ... })`, `right: Err(BindingAccessTypeMismatch)`.
- Admitting an F32 stored neighbour for a U32 component made the refusal test report `left: Ok(ArtifactEnvelope { ... })`, `right: Err(BindingComponentMismatch)`.

**Scope and workspace state.** The exact-base branch guard exits zero with no under-declared or unattributed file across direct scopes `implementation/ir`, `implementation/artifact`, `implementation/frontend`, `implementation/compiler`, `implementation/metal`, `contracts/artifacts`, `contracts/navigation`, and shared `project/tickets`; `contracts/decisions` remains declared but has no implementation diff. Guard reports only non-gating live-claim collision warnings. The implementation worktree was clean after commit. Its generated `target/` occupied 16.2 GiB and was removed with `cargo clean`; the resulting worktree measured 137 MiB with 146 GiB available on the volume.
