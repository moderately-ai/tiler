---
id: bind-shapeenv-sources-into-tensor-boundaries-and-coefficients
title: Extend sourced extents to tensor boundaries and semi-affine coefficients
status: done
priority: p1
dependencies: []
related: [implement-shapeenv-index-bindings, implement-index-domain-predicates]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, mature-product]
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

## Outcome

**Done for tensor boundaries. Semi-affine coefficients and divisors are split, and one half of the boundary work is split too — both for the same owner-reserved reason, both stated rather than implied.** `crates/tiler-ir/src/index/{sourced,model,builder}.rs` and `crates/tiler-ir/src/shape/env{,/constraint}.rs` changed.

**Gate status, stated exactly rather than claimed green.** `uv run --locked python scripts/check_rust.py` passes in full, all 768 workspace tests pass, `tkt lint` and `git diff --check` are clean. `uv run --locked python scripts/check_repository.py` exits 1, on two `scripts/docs.py validate` errors that this change did not introduce and cannot fix from `implementation/ir`: `accept-adr-0077-metal-aot-crate-admission` and `accept-adr-0078-public-extension-seams` each depend on a drafting ticket rather than an acceptance node. **Reproduced at base**: a detached worktree at `63b02ec` with a clean tree emits the same two errors, byte for byte. Those tickets are Tom's acceptance nodes and another worker's in-flight change, so they were left untouched rather than swept into this commit.

### Decided: a boundary is a `SourcedShape`, and `Shape` does not change

The ticket named this as Tom's, "`Shape` is public and widely used, so this is a public-boundary question". The resolution is that it did not have to become one. `TensorData` now holds a `pub(crate) SourcedShape`, which is `Static(Shape)` or `Sourced(Vec<SourcedExtent>)` with a normalizing constructor that collapses an all-literal vector into the first.

That shape was chosen because it is the only one that leaves the public API *byte-for-byte* unchanged. `TensorRef::static_shape()` returns `Option<&'a Shape>` — a borrow. A bare `Vec<SourcedExtent>` would have forced it to materialize a `Shape` and return it by value, changing a public signature to express something no public caller can reach. Holding the `Shape` also keeps one definition of a static shape rather than two: an all-literal boundary *is* a `Shape`, never a parallel vector that happens to mean the same thing. The three external consumers of `static_shape()` — two in `tiler-compiler/src/legality.rs`, three in `tiler-reference/src/oracle.rs` — are untouched and still see `Some` for every region a public caller can build.

**Rejected: making the enum's two arms encode differently.** They do not. `SourcedShape::encode` frames the rank and then writes extent by extent through `SourcedExtent::encode`, so a literal axis encodes identically whichever arm holds it, and the representation choice is unobservable in identity. That is what makes the normalization a convenience rather than a second authority.

### Decided: the phase ceiling is one constant, reached two different ways

`EXTENT_PHASE_CEILING` binds a boundary extent and a domain extent alike, but the module now distinguishes *why*, because the two claims are not equally strong. For a **boundary** it is a direct quotation: an output boundary's extent is an "initial output shape", and the accepted contract requires every one of those to be "evaluable on the host before any device work begins". For a **domain** it remains the inference `implement-shapeenv-index-bindings` recorded. One constant, two justifications, neither promoted to the other's status.

### The defect swept: three copies of one interval predicate

`interval_verdict`, `access_needs_exhaustive_proof`, and `remap_access` each spelled out the same "every coordinate lies inside its axis" test independently. Latent, since all three agreed — and exactly the hazard this ticket would have made live, because extending two of three is what produces a region whose retained evidence records an interval proof for an access the verifier enumerated. All three now read `interval_verdict`. Three copies of the per-extent environment lookup (`determined_extent`, `extent_upper_bound`, `domain_is_nonempty`) collapsed into one `extent_interval` for the same reason.

### What the index layer now does

**A boundary extent is a `SourcedExtent`,** admitted against the region's one environment through the same `admit` a domain extent uses, before any of the boundary is retained — so a refused symbol leaves the draft untouched rather than half-sourced.

**A symbolic axis is compared against the side of its own interval that makes each answer sound, and the two sides differ.** Proving a coordinate in bounds needs it below the axis in every model, so it is compared against the axis's **lower** bound. Refuting one needs it at or above the axis in every model, so that uses the **upper** bound. A static axis has a one-point interval and both collapse to the literal, which is why this reads as one rule rather than a symbolic special case. The payoff is real: a read into an axis the environment only bounds below is proved, with the retained evidence honestly saying `Interval`.

**`write_is_permutation` compares extents rather than values** — the site the ticket named. `ExtentSources::proves_equal` decides it two ways, and both are needed: the `ShapeEnv` **equality class**, which proves equality with no value known for either side, and a **common determined value**, which is the only route when one side is a literal. One-sided throughout: `false` means not proved, never proved-different.

**`ShapeEnv::proves_equal` is new,** and it answers a question `extent_interval` structurally cannot. An interval is a fact about one symbol in isolation; two symbols confined to one wide interval are not thereby equal, and two the environment does force together are not thereby confined to a point. It is recomputed like every other query, so no derived solver state reaches identity — the property `implement-shapeenv-index-bindings` established for `extent_interval` and this preserves.

**The finite fallback now requires a determined domain *and* a determined boundary.** An undetermined boundary is refused before anything is budgeted, deliberately not as `ProofResourceLimit`, which `docs/ir.md` defines as meaning an enumeration stopped. The pre-existing case of an element count too large for this host is untouched and stays on its own path: `boundary_extents` (undetermined) and `boundary_element_count` (unrepresentable) are two answers, kept apart precisely because one is a refusal and the other is a resource limit.

**Identity moves to `tiler.index-region.v6`.** A boundary now encodes tagged extent by tagged extent where `v5` wrote eight raw bytes per axis, so the bytes of a *wholly static* boundary changed even though its meaning did not. **This is a deliberate re-baseline, not drift.** It rippled nowhere: index-region identity reaches no verified product — `bind-stage-coverage-to-index-refinement-identity` records that its only consumer is an EXPLAIN label — and all 761 workspace tests passed unchanged, which is the measurement confirming it.

### Split, with the reason, and the reason is the same one both times

- **`name-the-proved-extent-equality-bounds-proof`** — the *bounds* half of the dynamically shaped case. Ownership is proved for a wholly undetermined `[n] -> [n]` copy; bounds are not, because proving `i < m` from `0 <= i < n` and `m == n` is an equality-class argument and no `BoundsProofView` variant names it. `Interval` would be the tempting lie — nothing about either interval closed the question. Recording a wrong proof kind is worse than refusing, so the rule was left out. `a_wholly_undetermined_dynamic_copy_is_refused_rather_than_approximated` **measures** that refusal (`BoundsNotProven` + `WriteOwnershipNotProven`, and not `ProofResourceLimit`) rather than asserting it from reasoning, and is the trigger that fails when the follow-up lands.
- **`admit-semi-affine-index-expression-class`** — the whole coefficient/divisor half. Blocked harder than expected: the public `IndexExprClass` carries **no `#[non_exhaustive]`**, so admitting `SemiAffine` is a *breaking* change rather than an additive one, and `IndexExprView`'s `divisor: u64` is public by value. Neither is expressible `pub(crate)`, because `IndexExprRef::class()` and `view()` are the public accessors that must return them.

**The decision that ticket said it owned is decided there rather than deferred with it.** A proven-positive symbolic divisor requires a **semantic input constraint, never a variant guard** — a guard's failure "selects another valid plan or fallback", and there is no other plan under which `x floordiv 0` has a meaning, so positivity is a condition of the expression being *defined*. `extent_interval` and `proves_equal` both already read `self.constraints` and never `self.guards`, so a positivity query written against either is correct by construction. The follow-up inherits an answer.

### What was checked and deliberately not changed

`docs/ir.md`'s reserved clause — static dimensions and boundaries "return `Some` throughout this bounded profile" — is still literally true. `static_shape()` can now return `None`, but only through a `pub(crate)` constructor no public caller can reach, exactly as `static_extent()` has been since the dependency ticket. No contract text needed editing, and the file is outside this ticket's scope, so nothing was edited under it. The one genuinely stale sentence there — `docs/ir.md` still tracking semi-affine coefficients to `implement-shapeenv-index-bindings` — is routed to `admit-semi-affine-index-expression-class`, which declares `contracts/foundation` for it.

### Draft status

Everything new is `pub(crate)` under ADR 0074 convention 7: `SourcedShape`, `sourced_tensor`, `sourced_shape`, `ExtentSources::proves_equal`, `ShapeEnv::proves_equal`. No `pub` item, public trait, `unsafe` block, or dependency was added, and no public signature changed.

### Evidence

Six new tests in `crate::index::sourced`, each naming a contract clause, each rejection paired with its accepted neighbour so the refusal is evidence about the input:

- the bounds pair differs only in the axis floor, `m >= 8` versus `m >= 3`, against one 4-point domain;
- the ownership pair differs only in whether the environment asserts `m == n`, with the neighbour's `4 <= m <= 5` *containing* `n`'s value so the refusal cannot be read as an arithmetic one — and both fixtures prove their bounds by interval, so the pair turns on ownership and nothing else;
- `two_undetermined_symbols_are_proved_equal_by_their_equality_class` isolates the class route by asserting neither symbol is determined, so no comparison of values could have decided it;
- the identity test shows an output written `[m]` and one written `[4]` under an environment pinning `m == 4` are different programs, which is the `graph identity` versus `specialized identity` distinction the contract keeps;
- the boundary-ceiling test is the quoted-clause counterpart of the domain one; and
- the undetermined-copy test records the measurement boundary above.

257 `tiler-ir` unit tests pass, up from 250; 768 across the workspace, up from 761. Measured on macOS arm64 at the pinned nightly. See the gate note above for the two pre-existing repository-validation errors this change neither caused nor fixed.
