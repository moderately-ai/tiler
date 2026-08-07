---
id: prove-the-governed-tag-tables-injective
title: Prove the governed tag tables injective
status: todo
priority: p2
dependencies: []
related: [prove-the-exhaustible-encoder-injectivity-claims-natively]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [verification, identity, injectivity, evidence-upgrade]
---
## User-visible outcome

Every governed `tag()` table reached only by an *inexhaustible* identity encoder is backed by an exhaustive pairwise-distinctness test over its whole variant set, so a duplicated tag literal fails the build's gate instead of silently folding two operations, address spaces, or authorities onto one identity.

## Why this exists (found while proving the exhaustible encoders, 2026-08-07)

**Fact.** `prove-the-exhaustible-encoder-injectivity-claims-natively` classified every canonical-identity *encoder* in `tiler-ir` and `tiler-artifact` and landed exhaustive injectivity tests for the 19 whose whole input domain is enumerable. Tag tables reached by one of those encoders are covered by it; the seven artifact tag tables in `crates/tiler-artifact/src/program/codec/tests.rs:541` are covered too, because a total `from_tag` left inverse over a complete enumeration implies injectivity.

**Fact.** About 50 `const fn tag(self) -> u8` tables are reached *only* from encoders that ticket correctly classified inexhaustible — `push_operation`, `push_block`, `AbiArenaTraversal::encode`, `TargetEvidence::encode`, `encode_environment`, and their peers. Each such table is itself a finite total map from a closed enum into one byte, so its injectivity is exhaustible even though its caller's is not. Most have no `from_tag` inverse and no distinctness test, so two variants sharing a literal is silent today.

**Fact — the population.** In `tiler-ir`: `kernel/model.rs` 12 tables (`KernelType` 6, `AddressSpace` 4, `BufferAccess` 2, `Builtin` 2, `BinaryOp` 12, `CompareOp` 1, `UnaryOp` 2, `ConvertOp` 4, `PackedExtractOp` 1, `ExecutionScope` 2, `MemoryScope` 2, `BarrierOrdering` 1); `program/model.rs` 7; `program/abi.rs` 3 (`AvailabilityPhase` 5, `AbiUnaryOp` 3, `AbiBinaryOp` 13); `numerics.rs` 7 (`NumericalDimension` 11, `PolicyLocus` 6, `FactAuthority` 7, `FactValidityScope` 5, `CompilerBuildRole` 8, `HonouringMeans` 4, `FactEvidenceBasis` 3); `schedule/` 11; `shape/` 5; `semantic/` 8; `index/` 3. In `tiler-artifact`: `RoutingPolicy`, `ArtifactExecutionPolicy`, `BindingKind`, `StageDependencyReason`, `RouteResourceDimension`, `RouteRequirement`, `RecordFamily`, `AssessmentDisposition`, `SectionKind`, `SectionDisposition`.

**Inference.** `FactAuthority` and `FactValidityScope` deserve first attention: both assign tags deliberately *out of declaration order* (`numerics.rs:1218` and `:1286`), which is exactly the shape where a hand-checked literal table is easiest to get wrong and hardest to spot in review.

## The work

1. For each table, enumerate its variants with an array sized by `core::mem::variant_count` so a widened vocabulary is a build error at the list. `#![cfg_attr(test, feature(variant_count))]` is already declared in both crates.
2. Assert the tags are pairwise distinct and count the population walked, so a shrunk enumeration fails rather than passing vacuously.
3. Where a `from_tag` inverse exists, also assert the round trip and that every unclaimed byte refuses — several already do this and need only the population guard.
4. Watch each new check fail on a planted duplicate literal before trusting it.
5. Do not weaken the existing round-trip tests; these sit beside them.

## Closes when

Every table in the enumeration above has a passing exhaustive distinctness test with a `variant_count`-guarded population, each watched failing on a planted duplicate tag, and any table deliberately left out is named with its reason.
