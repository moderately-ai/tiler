---
id: correct-the-l6-records-stale-storagescalar-variant-count
title: Correct the L6 record's stale StorageScalar variant count
status: done
priority: p2
dependencies: []
related: [admit-a-storage-carrier-for-integer-program-inputs]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, dtype]
---
## User-visible outcome

The L6 ingestion record stops asserting a carrier vocabulary that has been wrong since the BF16 landing, so the next reader of the "IN-A has a blocker at the ABI" paragraph is not sent to the wrong line and the wrong count.

## The defect

**Fact, found 2026-08-07 at base `68f1ced6` while auditing [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md).** `docs/research/program-planning/complete-model-ingestion-and-execution.md`, in the section "The input boundary: the gather stays inside", asserts:

> **Fact —** the runtime-value boundary has no carrier for it: `StorageScalar` in `crates/tiler-ir/src/program/model.rs:264` has exactly two variants, `U8` and `F32`.

Both halves are false at this base. The enum is at `model.rs:342`, not `:264` — line 264 is inside `StorageEncoding`, an unrelated type — and it has **three** variants: `U8`, `F32`, and `Bf16`. `Bf16` landed under [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md). Reproduce with `grep -n 'pub enum StorageScalar' -A 16 crates/tiler-ir/src/program/model.rs`.

**Fact — the same paragraph's ABI-cost conclusion is also falsified, and that is the load-bearing half.** It closes: "`StorageScalar::tag` participates in canonical encoding, so a third variant is an artifact-ABI change and therefore Tom's." A third variant *already happened* and was not an ABI break: `StorageScalar::tag` appends, and its own comment records that "`U8` and `F32` keep their tags and every field keeps its position, so no previously encodable program's bytes move and the program identity domain does not step."

Measured on the carrier ticket's branch: with a fourth carrier appended at tag `0x04`, `cargo nextest run -p tiler-ir -p tiler-artifact -p tiler-macros -p tiler -p tiler-compiler` ran 2213 tests with 2211 passing and **no golden, pin, or identity test moving**. The two failures were deliberate widening tripwires (a forged-tag test whose `UNASSIGNED_CARRIER` is `0x04`, and a `variant_count`-sized domain assertion), not identity drift.

## Why this is its own ticket

`research/program-planning` is not in the carrier ticket's scopes, and it was held by a live exclusive claim when the defect was found. The correction is a documentation edit with no code consequence, so it does not belong inside the carrier landing either.

## Closes when

The paragraph carries a dated correction in the record's own house style — the false line citation and variant count replaced with anchors, and the "therefore Tom's" ABI conclusion restated against the measured append-only behaviour — with the prior rationale preserved rather than silently overwritten.

## Worker audit — 2026-08-07, at base `6d1bd6e8`

**`project/tickets` was added to `shared_scopes` in this edit.** The ticket declared `scopes: [research/program-planning]` and no shared scope, but recording this audit is an edit to `tickets/**`. Adding a scope required by authorized work is scheduling metadata under `AGENTS.md`; it authorizes no new outcome, and `project/tickets` is a shared scope so it takes nothing from another worker.

### Per-Fact verdict on the ticket as filed

Every Fact was re-read at `6d1bd6e8` rather than carried from the filing base `68f1ced6`.

| Ticket Fact | Verdict | Evidence at this base |
| --- | --- | --- |
| The record quotes "`StorageScalar` in `crates/tiler-ir/src/program/model.rs:264` has exactly two variants, `U8` and `F32`" | **Verified** | The sentence is present verbatim in the "The input boundary: the gather stays inside" section |
| The enum is at `model.rs:342`, not `:264`, and `:264` is inside `StorageEncoding` | **Verified** | `grep -n 'pub enum StorageScalar' crates/tiler-ir/src/program/model.rs` returns `342`; `pub enum StorageEncoding` is at `263` and its body spans `264`ff |
| It has three variants — `U8`, `F32`, `Bf16` | **Verified** | `grep -n 'pub enum StorageScalar' -A 16 crates/tiler-ir/src/program/model.rs`; `Bf16` is documented "as a *carrier*, not as a target capability" |
| `Bf16` landed under `admit-the-bf16-type-and-carrier-into-every-total-map` | **Verified** | `git log -S 'Bf16,' -- crates/tiler-ir/src/program/model.rs` names `129d783b`, whose message is that ticket's outcome |
| The record's ABI conclusion is "`StorageScalar::tag` participates in canonical encoding, so a third variant is an artifact-ABI change and therefore Tom's" | **Verified** as a quotation, and **the same claim is restated a second time in D-17**, which the ticket does not name. Both were corrected. |
| `StorageScalar::tag` appends, and its comment records that no previously encodable program's bytes move and the identity domain does not step | **Verified** | `grep -n 'Bf16 => 0x03' -B 4 crates/tiler-ir/src/program/model.rs`; corroborated at `docs/artifact-abi.md` and by `129d783b`'s message |
| The measured fourth-carrier run: 2,213 tests, 2,211 passing, no golden/pin/identity test moving, two deliberate tripwires | **Unverifiable at this base, relayed** | The measurement lives on `admit-a-storage-carrier-for-integer-program-inputs`' own branch and needs a `crates/**` edit to reproduce, which is outside this ticket's scopes. It is relayed in the record with its source and is not restated as a fact of the record. |
| "The correction is a documentation edit with no code consequence" | **Verified** | The diff touches `docs/` and `tickets/` only |

**Imprecise, and corrected in the landed text.** The ticket calls the false half "the ABI-cost conclusion … also falsified", which reads as though the conclusion is false. It is not: only its stated ground is. The verdict "therefore Tom's" survives, and the record now says so with the surviving grounds named rather than leaving the conclusion floating.

### Re-derivation of the conclusion built on the false premise

Splitting the retired sentence into its parts:

- **Premise — `StorageScalar::tag` participates in canonical encoding: true.** `push_storage_scalar` pushes `scalar.tag()` and is called from `canonical_keys`' `value_key` and from the `CanonicalKernelProgramIdentity` builder's `push_value` (`grep -n 'push_storage_scalar' crates/tiler-ir/src/program/model.rs` returns the definition and both call sites).
- **Step — "so a third variant is an artifact-ABI change": false.** The third variant already happened. `Bf16` took `0x03`, `U8` and `F32` kept theirs, and no identity domain stepped. Participating in canonical encoding is what makes the tag table append-only under a compile error, not what would make widening it a break. The residue that is true runs forward, not backward: an unknown tag is refused by name at decode rather than misread.
- **Conclusion — "therefore Tom's": survives, on other grounds.** (1) `StorageScalar` is a re-exported public type of `tiler-ir`, so widening it is a public-boundary change under `AGENTS.md` regardless of byte cost — the ground **D-17 already gave for itself**, which is why the record contained its own repair. (2) ADR 0074 convention 5b keeps the vocabulary out of `#[non_exhaustive]` so the widening is a build error at every out-of-crate total map, making it one atomic cross-crate commit. (3) The substantive undecided content is `natural_access_type`: `KernelType` is `Bool`, `U8`, `Index`, `F32`, `I32`, `Bf16` with no four-byte unsigned integer, and `check_binding_access` pairs carrier to access type 1:1 and width-exactly with no wildcard, so a `U32` carrier pinned to `I32` passes every check while reading a token ID as signed at the one operation whose out-of-range behaviour reads out of bounds. An honest carrier needs `KernelType::U32` and an `msl_type` answer, which is Tom's.

**Why the verdict survived while its ground did not**, stated so a reader does not have to re-derive it: none of the three carriers is an eighteen-bit integer. `U8` is one byte; `Bf16` is a two-byte float with an eight-bit significand, exact on integers only to 256; `F32` is exact below 2²⁴ but is a float identity, which is the trade ADR 0041 prices. The count moved from two to three and the blocker did not, which is the same reason the sibling row in [`model-level-qualification.md`](../docs/research/program-planning/model-level-qualification.md)'s refusal-site table kept its verdict under the same correction.

### Landed

- `docs/research/program-planning/complete-model-ingestion-and-execution.md` — the "The input boundary" paragraph and D-17, each carrying a dated correction that quotes its retired text, cites by runnable `grep` rather than by line number, and states the surviving grounds.
- This ticket — the scope declaration and this audit.

Checks run: `tkt lint`, `make citations`, `git diff --check`, `tkt guard --base 6d1bd6e8`. `make full` is not run: the diff touches only `docs/` and `tickets/`, none of the carry-blocking paths `AGENTS.md` lists, so it carries the latest green gate. Every `grep` written into the record was executed and returns a non-empty result.

## Outcome — done, 2026-08-07

Landed at merge `aa4a9573`'s successor (worker commit `1d556a1e`). `docs/` and `tickets/` only, carries the green gate.

### This ticket's own framing was wrong, and correcting it was the substance

It said "the ABI-cost conclusion is **also falsified**". **Only its stated ground was falsified, not the conclusion** — and a worker taking the framing at face value would have withdrawn a "therefore Tom's" that is genuinely true.

The re-derivation, split cleanly:

- **Premise true** — `push_storage_scalar` pushes `scalar.tag()`, reached from `canonical_keys`' `value_key` and the kernel-program identity builder.
- **Middle step false** — "a third variant is an artifact-ABI change". A third variant *already happened*: `Bf16 => 0x03`, with `U8`/`F32` tags unchanged and both the landing commit and `docs/artifact-abi.md` recording no retained identity moving and no domain stepping. Participating in canonical encoding is what makes the tag table **append-only under a compile error**, not what makes widening it a break.
- **Conclusion survives on three grounds, none of them the stated one**: `StorageScalar` is a re-exported public type of `tiler-ir` — which is the ground **D-17 four sections down already gave for itself**, so the record contained its own repair; ADR 0074 convention 5b keeps the vocabulary out of `#[non_exhaustive]`, making the widening one atomic cross-crate commit; and the substantive replacement, that `natural_access_type` is a total width-exact map into `KernelType` which has no four-byte unsigned integer, so pairing `U32` with `I32` passes every check while reading a token ID as **signed** at the one operation whose out-of-range behaviour reads out of bounds.
- **The blocker verdict survives** for the anticipated reason: none of the three variants is an 18-bit integer carrier. `Bf16` is a two-byte float with an 8-bit significand, exact on integers only to 256, so it carries a token ID no better than `U8`.

### The ticket also under-scoped its own defect

It named one paragraph. **D-17 in "Unresolved decisions" carried the same two-variant count and the same false ABI ground independently** — correcting only the named paragraph would have left the record self-contradicting. Found by reading the file rather than grepping the quoted string.

Every citation in the repaired record is now a runnable `grep` with **no line numbers introduced**, and all seven were executed and returned non-empty. The measured 2,213-test figure from the carrier branch was **relayed with attribution rather than restated as a fact of the record**, since it could not be verified at this base.
