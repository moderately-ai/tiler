---
id: prove-the-exhaustible-encoder-injectivity-claims-natively
title: Prove the exhaustible encoder-injectivity claims natively
status: done
priority: p2
dependencies: []
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [verification, identity, injectivity, evidence-upgrade]
---

## User-visible outcome

Every canonical-encoding injectivity claim whose input domain is small enough to enumerate is backed by an exhaustive test rather than by prose reasoning — the claim's evidence class moves from a comment's argument to exhaustive-finite — and the encoders whose domains defeat exhaustion are enumerated by name with their domain sizes, which is the input the bounded-verification spike needs to pick its target.

## Why this exists (claims-ledger design work with Tom, 2026-08-06)

**Fact.** The identity discipline's per-tag injectivity claims are carried today by reasoning recorded at encoding sites plus mutation tests that each check one collision, not all of them. **Fact — some of these domains are tiny.** `push_synchronization_subject` (`crates/tiler-ir/src/schedule/model.rs:2399` at base `43e9b9af`; the ticket originally said line 2265, which was stale — corrected 2026-08-07 by the implementing worker) writes six bytes from small tag enums; its full input domain is 648 values, and exhaustive injectivity (all pairs, encode-equal implies value-equal) is a cheap native test under the repo's existing exhaustive-finite evidence class. **Inference.** Every such encoder can have its injectivity *proved* today with no new toolchain; leaving those claims on prose while the domain is enumerable is unspent evidence.

## The work

1. Enumerate every encoder participating in a canonical identity (schedule/kernel/program/semantic/index encodings in `tiler-ir`, the artifact codec in `tiler-artifact`; the compiler's explain-subject encoding is out of this ticket's scopes — enumerate it in the report, land nothing there). For each: the exact input domain and whether it is exhaustible at test-time cost (rule of thumb: full pair-set comparable in well under a second; state the count).
2. For each exhaustible encoder: land one exhaustive injectivity test beside it, in the existing test idiom, deterministic, with the population counted in the test so a shrunk domain fails rather than silently passing. Watch each fail under a deliberately introduced collision before trusting it.
3. For each inexhaustible encoder: record name, domain character (which fields blow the domain: u32/u64 ordinals, data-dependent loops), and the per-tag reasoning that currently carries it — this list is the spike's target menu and gets recorded in the ticket Outcome.
4. Do not weaken or replace the existing mutation tests; the exhaustive tests sit beside them.

## Closes when

The enumeration is complete with each encoder classified and counted, every exhaustible encoder has a passing exhaustive-injectivity test that was watched failing on a planted collision, and the inexhaustible list with domain characterizations is in the Outcome.

## Outcome (2026-08-07, worker on `tkt/prove-the-exhaustible-encoder-injectivity-claims-natively`)

### Ticket Fact repaired before building on it

The `push_synchronization_subject` line citation was stale (2265; the function is at `crates/tiler-ir/src/schedule/model.rs:2399` at base `43e9b9af`). The substantive claims held: the encoder does write six bytes from small tag enums, and its domain is 648 values — the "few hundred" estimate was right. Corrected in place above.

### What landed: 19 exhaustive-injectivity tests, each watched failing on a planted encoder defect

**Evidence tier asserted for every one of these: exhaustive finite evidence.** Not `SoundProof` — nothing here reasons about the encoder's text, it walks the domain — and not empirical, because nothing is sampled. The domains below really are finite and really are counted.

| encoder | site | domain | population |
| --- | --- | --- | --- |
| `push_synchronization_subject` | `tiler-ir` `schedule/model.rs` | kind x arrival scope x publish scope x fence x ordering | 648 |
| `push_subnormal` | `tiler-ir` `schedule/model.rs` | `SubnormalMode` | 3 |
| `push_permission` | `tiler-ir` `schedule/model.rs` | `NumericalPermission` | 2 |
| `push_exceptional_assumption` | `tiler-ir` `schedule/model.rs` | `ExceptionalValueAssumption` | 4 |
| `push_order` | `tiler-ir` `schedule/model.rs` | `ContributorOrder` | 1 |
| `push_synchronization` | `tiler-ir` `kernel/model.rs` | `Option<SynchronizationSubject>` | 649 |
| `push_subnormal` | `tiler-ir` `kernel/model.rs` | `SubnormalMode` | 3 |
| `push_permission` | `tiler-ir` `kernel/model.rs` | `NumericalPermission` | 2 |
| `push_exceptional_assumption` | `tiler-ir` `kernel/model.rs` | `ExceptionalValueAssumption` | 4 |
| `push_element_type` | `tiler-ir` `program/model.rs` | `KernelType` | 6 |
| `push_storage_scalar` | `tiler-ir` `program/model.rs` | `StorageScalar` | 3 |
| `push_storage_encoding` | `tiler-ir` `program/model.rs` | `StorageEncoding`, constructible | 7 of 512 candidates |
| `DimensionBehaviour::encode` | `tiler-ir` `numerics.rs` | 5 disjoint spaces | 12 |
| `ComponentValueDomain::encode` | `tiler-ir` `semantic/conformance.rs` | code range `u8 x u8`, plus positive-normal | 65 537 |
| `EncodedComponentShape::encode` | `tiler-ir` `semantic/types.rs` | 2 variants over a 1-value payload | 2 |
| `ParameterIndexMap::encode` | `tiler-ir` `semantic/types.rs` | `ParameterIndexMapKind` | 1 |
| `StaticEvidenceAuthority::encode` | `tiler-ir` `semantic/registry.rs` | 2 variants | 2 |
| `push_synchronization` | `tiler-artifact` `program/model.rs` | `Option<SynchronizationSubject>` | 649 |
| `push_storage_encoding` | `tiler-artifact` `program/model.rs` | `StorageEncoding`, constructible | 7 of 512 candidates |

Whole added suite runs in well under 0.1 s; the 65 537-value sweep alone is ~30 ms.

**How "nothing ran" is prevented.** Every enumeration over a plain enum is sized by `core::mem::variant_count`, so a widened vocabulary is a build error at the list rather than a population that silently shrinks. Every test additionally asserts the population it walked against a stated literal, so a domain that changes size fails and must be restated deliberately. `#![cfg_attr(test, feature(variant_count))]` was added to `tiler-ir` and `tiler-artifact` for this, following the precedent and written rationale in `crates/tiler-metal/src/lib.rs`; it is `test`-gated because the inhabitant lists are test-local and an unconditional declaration warns as an unused feature.

**Two properties, not one.** Injectivity alone does not compose: a variable-width component written into the middle of a record with no length prefix can shift the following field. Each test therefore also pins the encoded width — fixed where the encoder is fixed-width, per-variant where it is not (`ExceptionalValueAssumption`, `Option<SynchronizationSubject>`, `StorageEncoding`, `EncodedComponentShape`).

**`push_storage_encoding` is proved over the *constructible* domain, which is smaller than the type's.** `BitPackedEncoding` has private fields and one constructor admitting only widths below eight that divide eight. Both tests sweep all 512 `(u8, PackedBitOrder, PackedTailRule)` candidates through `new` and enumerate the survivors, so the population is derived from the admission rule rather than asserted beside it. The 504 rejected candidates are not values of the type as constructed and are outside the claim.

**Deliberate-failure demonstration.** All 19 tests were watched failing on a defect planted in the *encoding*, never in the assertion — a tag reused, a field dropped, a scope written twice, a byte not written. Each planted defect produced a test failure (not a compile error), and each collision failure named the exact colliding pair. Driver retained at the worker's scratchpad; the perturbations are listed in the worker report.

### An existing enumeration hardened

`all_behaviours()` in `crates/tiler-ir/src/numerics/tests.rs` already backed an exhaustive injectivity check (sort + dedup over all 12 `DimensionBehaviour` values), so that claim was already exhaustive-finite. Its *population* was not guarded: a hand-written list of twelve entries asserted against a hand-written `12`, so widening `SubnormalMode` would have left both at twelve and quietly narrowed the covered domain. It is now derived from the `variant_count`-sized space enumerations.

### Inexhaustible encoders — the bounded-verification spike's target menu

None of the following can be enumerated; each is listed with what blows the domain. This is **not** a claim that any is non-injective — each rests today on the framing argument in `crates/tiler-ir/src/identity.rs` plus per-site reasoning, which is an unverified argument, not evidence.

**`u32`/`u64` ordinals (2^32 or 2^64 per field).** `push_tensor_role` (`InputOrdinal`), `push_component_role` (`EncodedComponentRole`), `push_contraction_axis_source`, `push_synchronization_placement` (two `PhaseId`), `push_synchronization_point` (`SyncPointId`), `push_participant_range` (two `u64`), `push_workgroup_staging`, `push_bounds_proof`, `push_abi_reference`, `EntryPolicyBinding::encode`, `NumericalObligationKey::encode` (2^32 x 6 x 2^32).

**Slices and vectors (unbounded length).** `push_shape`, `push_axes`, `push_axis_decodes`, `push_participant_space`, `push_staged_span`, `push_cooperative_phase`, `push_cooperative_tile`, `push_schedule`, `push_staging`, `push_indices`, `push_requirements`, `encode_route_requirements`, `push_sorted_keys`, `push_interface_components`.

**Strings.** `push_numerical` (`profile_key`), `push_component_type` (`ResolvedValueType::canonical_encoding`), `push_origin` (`InputKey`), `TypeKey::encode`, `ProviderIdentity::encode`, `ShapeSymbol::encode`, `TargetPropertyQuery::canonical_bytes`, `ExecutionEnvironmentIdentity::encode` (5 strings), `CompilerBuildIdentity::encode`.

**Structural recursion.** `push_operation` / `push_block` (mutually recursive), `AbiArenaTraversal::encode` and `expr_key` (whole DAG), `CanonicalValue::encode` (`Sequence`/`Record` recurse), `ResolvedValueType::encode` (recurses through `TypeArguments`), `ExtentRelation::encode` (`Factorization` carries a `Vec<ExtentTerm>`), `encode_index_node`.

**Top-level identity encoders** (`schedule::encode_identity`, `kernel::encode_identity`, `program::encode_identity`, `artifact::encode_identity`, `compute_graph_identity`, `compute_identity`, `encode_environment`, `encode_sequence_identity`, `derive_identity`) are unbounded by construction.

**Two worth naming for the spike specifically, because they are *almost* exhaustible:**

- `push_resources` (`tiler-artifact` `program/model.rs`) — a `u32`/`u32`/`u64`/`bool` prefix, then a **finite tail of 1 495 296 values** (`Option<SynchronizationSubject>` 649 x `SubnormalMode`^2 x `NumericalPermission`^4 x `ExceptionalValueAssumption`^2). Holding the prefix fixed makes the tail enumerable, which is exactly the shape a bounded verifier wants: a small unbounded head over a large finite tail.
- `push_numerical` (both `tiler-ir` copies and the artifact copy) — a length-framed string and a `u32`, then a **finite tail of 2 304 values**. Same shape, smaller.

`push_tensor_role` and `push_component_role` are the cheapest genuinely unbounded targets: three shapes over one `u32` each, no recursion and no slices, so a bounded proof over the whole `u32` range is a single-variable problem.

### Out of scope, enumerated as the ticket asks

`tiler-compiler`'s explain-subject encoding is outside this ticket's scopes and nothing was landed there. Its finite-domain sites, for whoever picks it up: `push_tensor_role` and `access_mode_tag` in `selection.rs` and `frontier.rs`, and `push_tensor_role_name` / `tensor_role_name_len` in `call_registry.rs` — all cross-crate total maps over `TensorRole` (unbounded, `InputOrdinal`) and `AccessMode` (finite, 2).

### Deliberately not done, and why it is a separate ticket rather than a descope

About 50 `fn tag(self) -> u8` tables in `tiler-ir` and `tiler-artifact` are reached only by *inexhaustible* encoders — `BinaryOp` (12), `AbiBinaryOp` (13), `ConvertOp` (4), `FactAuthority` (7), `ExtentRelation` (6), and so on. Each is itself a finite total map whose injectivity is exhaustible and, for most of them, unproved: a duplicated tag literal is silent today. That is a real and cheap population, but it is a different unit from this ticket's — the ticket classifies *encoders*, and every one of these sits inside an encoder already classified inexhaustible above. Filed as `prove-the-governed-tag-tables-injective` rather than absorbed here, so the two claims stay separately auditable.

Tables reached by an exhaustible encoder are already covered by the tests above. Separately, the seven artifact tag tables in `crates/tiler-artifact/src/program/codec/tests.rs:541` are already proved injective — a total `from_tag` left inverse over a complete enumeration implies injectivity — so they need nothing.

**Correction 2026-08-08.** The left-inverse argument is valid, but two enumerations it quantified over were not shown complete. The `SubnormalMode` and `ExceptionalValueAssumption` lists carry payload products and have no type-derived sizes, so widening `FlushedZeroSign` or `ValueDomainProvenance` can leave the round trip short after the tag tables are repaired. [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md) owns those two populations and the parallel `FencedSpaces` census. The other five artifact tables and this ticket's encoder classifications are unchanged.

### Pinned identities

**None moved.** No encoding was changed; the whole diff is tests, test-local enumerations, two `cfg(test)` feature declarations, and doc comments. Verified by `cargo nextest run --workspace` on this branch's tree: 3085 tests run, 3085 passed, 7 skipped — including every checked-in identity hex, golden, and digest pin in `tiler-ir`, `tiler-artifact`, `tiler-cache`, `tiler-metal`, and `tiler-conformance`.
