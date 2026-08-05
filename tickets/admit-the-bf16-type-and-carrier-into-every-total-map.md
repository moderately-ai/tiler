---
id: admit-the-bf16-type-and-carrier-into-every-total-map
title: Admit the BF16 kernel type and storage carrier into every total map
status: review
priority: p1
dependencies: []
related: [spike-bf16-through-the-second-dtype-seams]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/frontend, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, kernel-ir, vocabulary]
claimed_from: todo
assignee: agent-bf16-carrier
lease_expires_at: 1785943617
---
## User-visible outcome

`KernelType::Bf16` and `StorageScalar::Bf16` exist, every total map over those two vocabularies states what BF16 means or refuses it by name, and the workspace compiles. Nothing produces either variant yet — this ticket admits the *types*, not a program, a kernel, or a target capability.

## Why this is one atomic cross-crate change rather than four tickets

**Fact, measured at `3990f9d` on 2026-08-02.** `KernelType` (`crates/tiler-ir/src/kernel/model.rs`) and `StorageScalar` (`crates/tiler-ir/src/program/model.rs`) are deliberately **not** `#[non_exhaustive]`, and both types' doc comments say why: they are cross-crate *total maps* into artifact identity, so ADR 0074 convention 5b requires widening them to be a build error at every encoder that must decide what the new variant means.

**Measurement — the exact reproduction.** Adding only the two variants (with `byte_width` 2, `natural_access_type` `KernelType::Bf16`, and appended tags) and running `CARGO_TARGET_DIR=./target cargo check --workspace --all-targets` enumerates every site the design intends to stop. Because `cargo` halts at the first failing crate, the enumeration takes four rounds, each patching the previous round's sites:

| Round | Site | Scope |
| --- | --- | --- |
| 1 | `crates/tiler-ir/src/program/model.rs:543` `element_bytes` | `implementation/ir` |
| 1 | `crates/tiler-ir/src/program/model.rs:1389` `push_element_type` | `implementation/ir` |
| 2 | `crates/tiler-artifact/src/program/model.rs:1737` `element_type_tag` | `implementation/artifact` |
| 2 | `crates/tiler-artifact/src/program/model.rs:1758` `storage_scalar_tag` | `implementation/artifact` |
| 2 | `crates/tiler-artifact/src/program/codec/validate.rs:369` `check_binding_access` | `implementation/artifact` |
| 2 | `crates/tiler-compiler/src/physical.rs:2085` `index_arithmetic_requirement` | `implementation/compiler` |
| 3 | `crates/tiler-metal/src/emit.rs:812` `msl_type` | `implementation/metal` |
| 3 | `crates/tiler-compiler/src/boundary.rs:2130` `every_storage_carrier_has_a_representable_alignment` (test) | `implementation/compiler` |

**Inference.** There is no ordering of these edits that leaves the workspace compiling in between, so they are one commit or none. That is the designed behaviour, not an accident: each site is a place that must *decide*, and a half-landed widening is a vocabulary whose meaning some encoder has not stated.

**Measurement — the enumeration re-run at `4ff657c5`, 2026-08-05.** The table above holds and grew by three; the eight sites are all still sites, at shifted line numbers. `cargo check --workspace --all-targets` took six rounds rather than four, and a seventh site class only appeared under `cargo nextest`, because a `trybuild` fixture is compiled at test *run* time and no `cargo check` reaches it.

| Round | Site | Scope | Status vs. the table above |
| --- | --- | --- | --- |
| 1 | `crates/tiler-ir/src/program/model.rs:627` `element_bytes` | `implementation/ir` | expected (was `:543`) |
| 1 | `crates/tiler-ir/src/program/model.rs:1539` `push_element_type` | `implementation/ir` | expected (was `:1389`) |
| 2 | `crates/tiler-artifact/src/program/model.rs:1784` `element_type_tag` | `implementation/artifact` | expected (was `:1737`) |
| 2 | `crates/tiler-artifact/src/program/model.rs:1805` `storage_scalar_tag` | `implementation/artifact` | expected (was `:1758`) |
| 2 | `crates/tiler-artifact/src/program/codec/validate.rs:369` `check_binding_access` | `implementation/artifact` | expected |
| 2 | `crates/tiler-compiler/src/physical.rs:2295` `index_arithmetic_requirement` | `implementation/compiler` | expected (was `:2085`) |
| 3 | `crates/tiler-metal/src/emit.rs:812` `msl_type` | `implementation/metal` | expected |
| 5 | `crates/tiler-compiler/src/boundary.rs:2130` `every_storage_carrier_has_a_representable_alignment` (test) | `implementation/compiler` | expected |
| 5 | `crates/tiler-macros/src/binding.rs:854` `storage_scalar_path` | `implementation/frontend` | **new** |
| 6 | `crates/tiler/src/route/tests.rs:72` `dense_len` (test fixture) | `implementation/frontend` | **new** |
| nextest | `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs:61` `Buffer::dense` (`trybuild` fixture) | `implementation/frontend` | **new** |

**Why `implementation/frontend` is declared above.** The three new sites are in `crates/tiler-macros/**` and `crates/tiler/**`, which `ticketsplease.toml` maps to that scope. The declaration is required rather than optional: the atomicity argument in this section applies to them exactly as to the other eight — leaving any one unpatched leaves the workspace red — so they cannot be split into a follow-up ticket without publishing a non-compiling commit. Declaring the scope is scheduling metadata and authorizes no new outcome; the underlying edits are one tag spelling and two fixture width tables. No other ticket held `implementation/frontend` when this was declared.

**Fact — there are five tag encoders for these two vocabularies, not four.** The Implementation-keys bullet below names four; `StorageScalar::tag` at `crates/tiler-ir/src/program/model.rs:354`, which `push_storage_scalar` calls, is the fifth. All five append.

**Fact — the `msl_type` bullet's stated justification is stale, and the decision it reaches is not.** That bullet rests on `declare-the-bf16-rows-on-the-authoritative-metal-profile` being `blocked`; it is `done` at this base, and `FIRST_MACOS_APPLE9` now carries `bf16_dispatchability: Dispatchable`, a BF16 subnormal row, and the profile key `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` under MSL 4.0 / macOS 26.0. The refusal is still correct, for the reason the Graph-maintenance section already gives: `lower-bf16-to-metal` owns the `bfloat` spelling *together with* the constant reinterpretation, the BF16 NaN canonicalization helper, the dispatch route, and the simulator refusal. A spelling landed here without that machinery would let a BF16 kernel emit source that compiles while the numerics it depends on are absent — a stronger reason to refuse than the unmeasured-capability one, reaching the same arm.

**Inference — this is why the BF16 graph edges could not be satisfied as drawn.** `carry-bf16-through-the-artifact-encoding-and-identity` and `lower-bf16-to-metal` both depend on `admit-bf16-into-the-schedule-and-kernel-vocabulary`, and that ticket declares `scopes: [implementation/ir]`. The dependency direction is right for *behaviour* and wrong for *compilation*: the IR ticket cannot produce a green commit without the arms those two tickets own. This ticket is the compile-forced minimum extracted from all three, so each of them keeps its real work and none of them is compile repair for another.

## Implementation keys

- **Appends only, and the claim is carried per tag site.** `KernelType::Bf16` takes `0x06` and `StorageScalar::Bf16` takes `0x03`; every existing tag keeps its value and every field keeps its position, at all four encoders that carry these tags (`kernel/model.rs`'s `KernelType::tag`, `program/model.rs`'s `push_element_type`, and `tiler-artifact`'s `element_type_tag` and `storage_scalar_tag`). No previously encodable kernel, program, or artifact contains either byte, so no retained identity moves and no identity domain steps. Verify this by recomputing every pinned identity on the merged tree rather than by the gate staying green.
- `element_bytes(KernelType::Bf16)` and `StorageScalar::Bf16::byte_width()` are both `2`, derived at the one width authority each, with no second table.
- The artifact's `*_from_tag` decoders gain the new tags in the same change. A tag added to the encoder and not the decoder produces an artifact that encodes and fails to decode.
- `check_binding_access` pairs `StorageScalar::Bf16` with `KernelType::Bf16` and continues to refuse every mismatched pair — the two-versus-four-byte misread is the failure this check exists to prevent.
- **`msl_type` must refuse BF16, not spell it.** This is the load-bearing decision in this ticket. Emitting `bfloat` would make an unmeasured target capability appear available: `declare-the-bf16-rows-on-the-authoritative-metal-profile` is `blocked` because the only BF16 measurement is `-std=metal3.1` against macOS 13.0 while the authoritative profile is MSL 4.0 / macOS 26.0. Make `msl_type` fallible and return the existing unsupported-type refusal for BF16, so the widened vocabulary rejects explicitly rather than silently approximating. `lower-bf16-to-metal` replaces that refusal with a spelling once its profile dependency is satisfied.
- `index_arithmetic_requirement` classifies BF16 as imposing no index-arithmetic requirement, beside the other non-index types.
- Check whether `docs/artifact-abi.md` states anything that a widened element-type or storage-carrier vocabulary makes wrong. It does not enumerate tag values today, so the expected answer is no; the scope is declared so the answer can be *recorded* rather than assumed. If the ledger must move, it moves in this commit.

  **Answer, checked at `4ff657c5`: no sentence becomes wrong, and the document needed no edit.** It enumerates no element-type or storage-carrier tag values (`grep -n '0x0' docs/artifact-abi.md` returns only the digest-algorithm tag `0x01` and the resource-record presence byte). Three sentences bear on the widening and all three stay true: line 249 — every encoded enumeration goes "through the one governed tag table its vocabulary owns, never through a Rust discriminant, so inserting a variant cannot silently renumber a value already on disk" — is the exact mechanism used here, and `grep -rn 'as u8' ` over these two enums confirms no discriminant-derived encoding exists; line 508's "no storage width is derived … the absolute byte width of a governed element type is a backend fact this crate does not own" is unaffected by a carrier gaining a width elsewhere; and line 217's identity ledger does not move, because nothing appends-only steps a domain. Line 249's claim that each table is "pinned by an exhaustive round-trip test" *was* about to become false — `every_governed_tag_table_round_trips` iterated hand-written arrays with no exhaustiveness forcing, so it would have kept passing while covering one variant fewer. That test now carries a wildcard-free `match` beside each array, which is what makes line 249 true rather than aspirational.

## Required evidence

- `make full` green on the completed batch, from the log's own terminal lines.
- Every pinned identity recomputed on this tree, with the result stated: which moved (expected: none) and which did not.
- The `msl_type` refusal observed failing — a BF16-typed value reaches emission and is refused by name, and an F32 neighbour on the same path still emits, so the refusal is about the type and not a dead path.

  **Measurement boundary — the emission half of this is not constructible at this commit, and the refusal was still watched failing.** No `VerifiedKernel` can carry a BF16 buffer here: `crates/tiler-ir/src/kernel/verify.rs:245` derives every buffer's expected element type from the region's `ScalarProgram`, every arm of which is F32, so such a kernel is refused as `BufferContract` before emission is reached. Making one constructible is `admit-bf16-into-the-schedule-and-kernel-vocabulary`, which is blocked on *this* ticket, so the ordering forbids the end-to-end observation here rather than it having been skipped. What was observed: `msl_type(KernelType::Bf16)` refused by name while `msl_type(KernelType::F32)` returned `Ok("float")` in the same test, and the refusal watched failing by perturbing the arm to `Ok("bfloat")` — `left: Ok("bfloat")`, `right: Err(UnsupportedValueType { value_type: Bf16 })`. Reachability from emission is structural: all five `msl_type` call sites propagate its `Err`.
- `check_binding_access` refusing a `StorageScalar::Bf16` paired with `KernelType::F32`, observed failing.
- An F32 artifact and an F32 kernel identity byte-identical to `3990f9d`, pinned by the existing goldens.

## Closes when

Both variants exist, all eight sites above state BF16's meaning or refuse it by name, no pinned identity moved, the two refusals are observed failing, and the gate is green on the exact commit.

## Graph maintenance

- Blocks `admit-bf16-into-the-schedule-and-kernel-vocabulary`, which cannot produce a green commit before this lands.
- Does **not** subsume `carry-bf16-through-the-artifact-encoding-and-identity`: that ticket still owns the encode/decode round trip, the dtype's participation in program identity, the unknown-tag decoder refusal, and the `canonical_arithmetic_nan_bits` width question. This ticket only adds the tags those tests will exercise.
- Does **not** subsume `lower-bf16-to-metal`: that ticket still owns the `bfloat` spelling, the constant reinterpretation, the BF16 NaN canonicalization helper, dispatch on the measured row, and the simulator refusal. This ticket only makes the unspelled case an explicit refusal.
- Nothing here declares, implies, or depends on a BF16 target fact. The Metal arm is a refusal precisely so that no unmeasured capability becomes reachable.
- This ticket declares four implementation scopes and must be dispatched when all four are free. That is a real scheduling cost and it is the cost the non-`#[non_exhaustive]` design deliberately buys.
