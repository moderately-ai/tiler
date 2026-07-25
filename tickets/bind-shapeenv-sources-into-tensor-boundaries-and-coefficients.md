---
id: bind-shapeenv-sources-into-tensor-boundaries-and-coefficients
title: Extend sourced extents to tensor boundaries and semi-affine coefficients
status: in-progress
priority: p1
dependencies: []
related: [implement-shapeenv-index-bindings, implement-index-domain-predicates]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, mature-product]
claimed_from: todo
assignee: agent-ir3
lease_expires_at: 1785005316
---
`implement-shapeenv-index-bindings` landed sourced extents for index **domain dimensions**. Two of the four things `docs/ir.md` assigns to the symbolic index profile remain literal-only, and they were split rather than half-implemented.

**Fact — tensor boundaries are still static.** `crates/tiler-ir/src/index/model.rs` holds `TensorData { role, value_type, shape: Shape }`, and `Shape` is `Vec<Extent>` over `u64`. `TensorRef::static_shape()` therefore returns `Some` unconditionally, unlike `DomainDimensionRef::static_extent()`, which now returns `None` for a symbolic dimension. The reserved `None` in `docs/ir.md` — "static dimensions and tensor boundaries expose optional `static_extent()` and `static_shape()` facts" — is realized on one of the two accessors it names.

**What this costs.** A region whose *output* extent is symbolic cannot be expressed. The landed slice proves a symbolic **read** in bounds against a static axis, and proves a symbolic **write** only when the environment determines the extent exactly. A dynamically shaped output — the ordinary case for a caller-sized program — needs the boundary's extent to name the same symbol as the domain's, so the write-ownership argument can compare two symbols rather than a symbol and a literal. `write_is_permutation` in `crates/tiler-ir/src/index/builder.rs` is the exact site: it compares `self.determined_extent(d) != Some(extent.get())`, and the symbolic form of that comparison is symbol equality, which the `ShapeEnv` constraint environment already decides.

**Fact — coefficients and divisors are still literal.** `IndexNode::LinearCombination` carries `IndexInteger` coefficients and `FloorDiv`/`Modulo` carry `u64` divisors. ADR 0046 admits more: "the initial expression vocabulary admits affine, constant-divisor quasi-affine, and guarded semi-affine expressions with symbolic coefficients or proven-positive symbolic divisors". The research memo classifies these as `SemiAffine`, distinct from `QuasiAffine`, and the shape contract states the proof consequence: "a symbolic divisor crosses the affine boundary and may produce a structured `Unknown` during static proof".

## Scope

Extend `SourcedExtent` use to tensor boundary extents and to expression coefficients and divisors, keeping the properties the domain slice established: no index-local symbol authority, the same phase ceiling, mathematical-integer semantics, and identity that names the symbol rather than a resolved value.

Two decisions this ticket owns rather than inherits. First, whether a boundary shape becomes a vector of `SourcedExtent` or a distinct sourced-shape type — `Shape` is public and widely used, so this is a public-boundary question and is Tom's. Second, what a proven-positive symbolic divisor requires: the constraint environment can decide `d >= 1`, but a divisor is also a *guard* rather than a semantic constraint in some uses, and the two are explicitly not interchangeable.

## Closes when

A symbolic output boundary is expressible and its write-ownership proof succeeds exactly when the environment proves the domain and boundary extents equal; a semi-affine coefficient or divisor is either admitted with its positivity proved or refused explicitly; the fragment each proof relies on is stated; and `uv run --locked python scripts/check_repository.py` passes.
