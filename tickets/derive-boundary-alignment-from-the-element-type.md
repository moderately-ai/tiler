---
id: derive-boundary-alignment-from-the-element-type
title: Derive boundary alignment from the element type rather than the profile
status: in-progress
priority: p1
dependencies: []
related: [spike-bf16-through-the-second-dtype-seams, admit-bf16-into-the-schedule-and-kernel-vocabulary]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, abi, boundary]
claimed_from: todo
assignee: worker-alignment
lease_expires_at: 1785570907
---
## User-visible outcome

A boundary value's alignment requirement comes from its own element type, so a two-byte dtype states two-byte alignment instead of inheriting `f32`'s four. Today every boundary in the compiler is aligned as though it were `f32`, and for BF16 that is silently wrong in the permissive direction.

## Why this is a prerequisite rather than a cleanup

**Fact, at `ef3c051`.** `ByteAlignment::F32_NATURAL` is a constant `4` in `crates/tiler-compiler/src/boundary.rs`, and its own doc comment names the gap:

> The bounded profile's boundary values are strict `f32` throughout under `StrictF32NumericalContract`, and `ScheduledRegion` carries no resolved element type of its own. A widened dtype vocabulary must derive this from the boundary value's element type rather than from the profile, and that derivation needs a field the scheduled-region IR does not have today.

Reproduce in one line:

```sh
rg -n -B6 'F32_NATURAL: Self' crates/tiler-compiler/src/boundary.rs
```

**Fact.** It is consumed at roughly twenty sites across `call_registry.rs`, `call_abi.rs`, `call_declaration.rs`, `selection.rs`, `frontier.rs`, and `boundary.rs` itself.

**Inference.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) classified this as a *missing typed extension point* rather than an F32-specific fact: alignment is a property of the element type in every ABI, and the constant is standing in for a derivation the IR cannot yet express. A four-byte alignment applied to a two-byte element over-aligns, which is not a wrong answer today only because there is one dtype — the moment there are two, one of them is being told a requirement that is not its own, and an ABI check that passes for the wrong reason is the kind of thing that stops passing when a real allocator gets involved.

## Implementation keys

- The scheduled-region IR needs the resolved element type at the boundary. Decide where it lives and state the elimination: a field on the boundary value, a derivation from the region's scalar program, or a lookup through the semantic value — these differ in whether a region with no scalar program can answer, and in whether the answer is part of canonical identity.
- Alignment then derives from the element type's byte width. `StorageScalar::byte_width` is the existing exhaustive derivation and is the natural authority; do not add a second width table.
- `ByteAlignment::new` already refuses non-powers-of-two and must keep doing so, for the reason its doc gives: divisibility is a partial order over powers of two and not over arbitrary integers.
- Alignment subsumption stays divisibility. A widened dtype must not weaken the relation.
- If the derivation turns out to belong in `tiler-ir` rather than `tiler-compiler`, that is a scope change to report, not to absorb silently.

## Required evidence

- An `f32` boundary still derives four-byte alignment, and every existing ABI fixture is unchanged.
- A two-byte element derives two-byte alignment, exercised through a real boundary rather than a unit call on the constant.
- A boundary whose declared alignment does not satisfy its derived requirement is refused, and the refusal is observed.
- Whether the derivation enters canonical identity is stated explicitly, and if it does, the moved identity is recorded.

## Closes when

Boundary alignment derives from the element type at every site that consumes `F32_NATURAL`, `f32` behaviour and fixtures are unchanged, a narrower element derives a narrower alignment through a real boundary, the refusal path is observed failing, and the doc comment naming this gap is replaced by one describing what the code now does.

## Graph maintenance

- Independent of the semantic and target children; it can land in parallel with either.
- Gates `admit-bf16-into-the-schedule-and-kernel-vocabulary`, which introduces the first element type whose width is not four.
- The comment quoted above is the specification for this ticket. When the derivation lands, that comment is stale and must be corrected in the same change — a doc comment describing an absent mechanism is a defect once the mechanism exists.

## Outcome — derived from the scalar program; no IR field, no identity move (2026-08-01)

**The elimination, which is the ticket's first implementation key.** The three candidates were tested against what is actually reachable at `frontier::derive_boundary_contract`, the one place a boundary contract is built.

*A field on the boundary value* — **eliminated**. It would go on `tiler_ir::schedule::Access` or `IndexRegion`, both inside `CanonicalScheduledRegionIdentity`. Encoded, it moves every pinned schedule digest and steps the schedule domain a second time; unencoded, it is an unvalidated second authority two identical-identity regions could disagree on. It also stores a value that is already exactly determined — `tiler-ir`'s `kernel/verify.rs` *derives* each buffer's type from the scalar program and checks the kernel against it, so a field would be a second copy of a derived value.

*A lookup through the semantic value* — **eliminated**. There is no linkage to follow: `Access` carries `TensorRole` and `Option<EncodedComponentRole>` and no value identity, the compiler's `VerifiedScheduledRegion` carries `SemanticMemberId` (operations, not values), and `region.rs`'s `GraphValue` keeps only `type_encoding: Box<[u8]>` rather than a `ResolvedValueType`. Creating the linkage *is* the first candidate. Independently, the semantic width is the wrong quantity: it is the *logical* width, and the catalog's `u4` row is `width_bits: 4`, which is zero bytes and no alignment at all — the packed-u4 case is exactly where logical width and storage alignment diverge, and `StorageScalar`'s own doc keeps the three type layers apart for that reason.

*A derivation from the region's scalar program* — **survives, and the ticket's stated worry about it does not apply.** `IndexRegion::scalar_program` is a required field, not an `Option`: `ScheduledRegionBuilder::assemble` fails with `incomplete(ScheduleComponent::ScalarProgram)` without one, so **there is no region with no scalar program**, and the candidate always answers.

**Identity consequence: none.** The carrier is a pure function of content already inside `CanonicalScheduledRegionIdentity`, so it adds no identity input, and every admitted region's derived alignment is unchanged at four. **No `ScheduledRegion` field, no schedule domain version step, and the artifact-abi ledger is not engaged.** All 1919 workspace tests pass unmodified — no pin was rebaselined, and none moved.

**Scope note, declared rather than silent.** `implementation/ir` was added. The second key names `StorageScalar::byte_width` as the width authority and forbids a second width table, and that method was private to `tiler-ir`'s `program::model` with one caller. Honouring the key therefore requires widening its visibility to `pub`. That is a visibility change on an existing exhaustive `const fn` — no new field, no new variant, no encoding, no tag, no identity — so it is not the identity-bearing IR change the dispatch brief flagged for a stop. Adding a compiler-local width table instead would have satisfied the scope at the cost of the key the ticket exists to protect.

**What moved.** 23 sites: two on the production path (`bounded_requirements`, `bounded_guarantees` in `frontier.rs`, now taking the carrier) and 21 test-fixture sites across `boundary.rs`, `call_abi.rs`, `call_declaration.rs`, `call_registry.rs`, `selection.rs`, and `frontier.rs`. The constant is gone, so the compiler enumerated the sites rather than a grep.

Two sibling constants in `crates/tiler-compiler/src/program.rs` carried the same defect at the artifact layer and are fixed with it: `ELEMENT_BYTES = 4` and `ELEMENT_ALIGNMENT = 4`, stated as profile facts beside `MaterializedValueSpec`'s own `storage_scalar` field. Both now derive from one named `BOUNDED_CARRIER`, and the artifact alignment routes through `ByteAlignment` so it meets the same power-of-two refusal instead of reaching `check_alignment` as a bare integer.

**The refusal that replaced an over-alignment.** `StrictAffineU4Dequantize` binds `[U8, F32, U8]` reads, so no single carrier describes its boundary. It is refused with `boundary-carrier-unmodelled` rather than served `F32`, which would over-align its two `U8` buffers — the "passes for the wrong reason" outcome. `physical::verify_region_subject_binding` already refuses it upstream, so nothing on the compile path changes.

**Watched failures, each observed.** (1) Production site reverted to a hard-coded 4 → `the_property_builders_state_the_carriers_alignment_rather_than_a_constant` fails (`left: Alignment(4)`, `right: Alignment(1)`). (2) Mixed-carrier refusal widened to `Some(F32)` → `a_program_whose_boundary_carriers_disagree_is_refused_rather_than_widened` fails. (3) `StorageScalar::U8`'s width falsified to 4 → `alignment_derives_from_the_element_width_rather_than_the_profile` fails. The power-of-two refusal keeps its existing test, which still passes.

**The two-byte case is not reachable in this ticket, and that is a scope boundary rather than an omission.** The required evidence asks for a two-byte element through a real boundary. `StorageScalar` is `{U8, F32}`; there is no two-byte carrier, no `KernelType::Bf16`, and `compiler::target.rs`'s `ScalarArithmetic::new` admits only `f32` as an arithmetic subject. `tiler::bf16@1` is registered as a *semantic identity* (`semantic/catalog.rs`, and `ArithmeticType::Bf16`), but registration is recognition only and creates no storage carrier — the catalog says so in its own header. Adding the carrier means a `StorageScalar` variant plus a tag in `tiler-artifact`'s `storage_scalar_tag`, which is the artifact ABI ledger and is `admit-bf16-into-the-schedule-and-kernel-vocabulary`'s third implementation key verbatim — the ticket this one *gates*. Doing it here would invert that dependency and edit `contracts/artifacts`, which this ticket does not hold.

What is delivered instead is the same property one carrier narrower: `StorageScalar::U8` derives one byte, driven through the production property builders rather than as a unit call on a constant, and that is the case that catches a reverted site — with a single-dtype profile an `f32`-only test cannot distinguish a derivation from the constant, because both answer four. When `admit-bf16` adds the two-byte carrier, `every_storage_carrier_has_a_representable_alignment` becomes a build error until its array is extended, and the two-byte assertion belongs there.
