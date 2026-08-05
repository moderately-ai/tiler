---
id: admit-the-registered-unary-families-at-the-compiler-request-boundary
title: Admit the registered unary families at the compiler request boundary
status: in-progress
priority: p1
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary]
related: [admit-the-silu-activation-family, admit-the-reindex-and-broadcast-operation-families, admit-the-rms-normalization-family, admit-the-softmax-family, land-the-elementary-family-projection-adr, admit-the-structural-families-into-the-scheduled-region-vocabulary, declare-elementary-realizations-on-a-target-profile, correct-the-optimizer-stage-generality-claims-for-the-admitted-activation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
claimed_from: todo
assignee: agent-unary-families
lease_expires_at: 1785893513
---
## User-visible outcome

A program stating `tiler::silu-f32@1` reaches the optimizer and compiles, instead of refusing at the request boundary under `operation-set` despite the family having registered semantics *and* a registered index-access lowering capability.

**Revised 2026-08-04, and the two families struck from it went somewhere.** This outcome named all three registered unary families. `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` are not deliverable from this ticket's scopes: what they lack is a `LogicalAccess` spelling of their *access relation*, which is a `tiler-ir` widening, and unlike an elementary body there is no projection into the existing region vocabulary that substitutes for it. [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) carries them with a dependency edge on this ticket. The narrowing is recorded rather than absorbed: the two families still refuse under `operation-set`, and the refusal is asserted beside the activation's admission so the rule reads which vocabulary is missing.

## Why this exists, and why it is not the recognizer's to fix alone

**Fact — the capability exists and the boundary cannot reach it.** `governed_index_access_capabilities` registers eight capabilities, and three of them — `silu-f32`, `reindex-f32`, `broadcast-f32` — name families no recognized program can contain. `crates/tiler-compiler/src/lowering.rs`'s `resolve_lowering` would resolve each of them for a member the recognizer produced; the recognizer never produces one.

**Fact — the vocabulary the recognizer targets is the region's, not the capability's.** `select_supported_strategy` builds a `ScalarProgram` and a `LogicalAccess` per region. `PointwiseF32Node` has no sigmoid-weighted linear unit and `LogicalAccess` has no reindex; the only broadcast is `ScalarBroadcast`, a rank-zero operand read once. So a recognizer that admitted `silu(x)` would have to decompose it into multiply, exp, add, and divide nodes — which is this boundary re-deriving what the registered provider's lowering already states, and exactly what occurrence refinement exists to prevent.

**Inference — the question is architectural before it is mechanical.** Either the region vocabulary grows a node per admitted elementary family (and the accuracy contract each carries has to reach the region's numerical realization), or a region gains a way to name an occurrence whose per-point body is the resolved capability's emitted index region. The first is additive and bounded; the second is the seam that makes an out-of-crate provider's family reachable without a `tiler-ir` change per family. Choosing between them is a design decision with an ADR-shaped consequence, and it is the first work item here.

## Boundaries

- Refinement stays the authority that proves a provider's region realizes its occurrence. Whichever route is chosen, the compiler must not restate a provider's per-point arithmetic as its own.
- Each family's registered accuracy contract must reach whatever the region records, or the compiled program would make a tolerance claim nothing carries.
- Until it lands, `operation-set` remains the refusal, and the `a_family_outside_the_expression_vocabulary_refuses_with_a_typed_reason` test in `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` is what keeps it observed. **That test now carries both halves**: the activation compiles and the reindex refuses, under the same request, so the rule stays attributable after the boundary moved.

## Closes when

The route is chosen and recorded as an accepted decision; at least one of the three families compiles through `tiler_compiler::session` to an emitted region whose numerical realization carries its accuracy obligation; and a family with no installed capability still refuses by name, observed failing.

## Outcome, 2026-08-04

**The route is chosen and derived below; `tiler::silu-f32@1` compiles through `tiler_compiler::session`, its registered accuracy contract is assessed against the target on every compile, and both structural families remain refused by name.** The route decision is drafted as an ADR body in the section after next, because this ticket's scopes cannot reach `docs/decisions/`; [`land-the-elementary-family-projection-adr`](land-the-elementary-family-projection-adr.md) carries it and its acceptance is Tom's.

### The route, and why the elimination leaves one candidate

**Fact — the physical vocabulary already spells the activation's body, so route 1's cost is already paid.** `PointwiseF32Node` carries `Exp`, `Divide`, and `Rsqrt` (`crates/tiler-ir/src/schedule/pointwise.rs`), and `crates/tiler-metal/src/tests.rs`'s `silu_kernel` already builds and lowers exactly the SiLU expression to a verified kernel. The ticket's framing — "the region vocabulary grows a node per admitted elementary family" — described a `PointwiseF32Node::Silu`, and that is *not* what route 1 needs: the vocabulary is deliberately rounding-explicit (`Divide`'s own doc states why a reciprocal-and-multiply is a separate node rather than a permission), so a single node for a four-rounding composition would hide exactly what the vocabulary exists to expose.

**Inference — route 2 is eliminated by the property it was proposed for.** "A region names an occurrence whose per-point body is the resolved capability's emitted index region" buys reachability for an out-of-crate family without a `tiler-ir` change. That is the same thing as admitting an *open*, registry-driven `ScalarOpKey` DAG into the region body — and `PointwiseF32Node`'s closedness is what forces every schedule-identity encoding, KIR lowering, and backend emission site to fail the build when a new physical meaning arrives. Restricting the embedded body to a closed subset restores the property and collapses route 2 into route 1 plus a projection. So the seam route 2 offers is the seam AGENTS.md's extension rule forbids: "'extensible' does not mean unknown behavior is optimizable."

**What survives is route 1 with the boundary's spelling made non-independent.** The ticket's real constraint is not "do not decompose" — the boundary already decomposes `tiler::add-f32@1` into `PointwiseF32Node::Add` — it is "do not *restate* a provider's per-point arithmetic as your own", i.e. do not create a second unchecked claim about one meaning. `crates/tiler-compiler/src/elementary.rs` is the answer: the composition is stated **once**, against an abstract per-point sink, and both realizations are driven from it — `GovernedSiluF32` emits it as index-region scalar applications, and `recognize_elementwise` projects it into the physical expression. Neither can drift without breaking the other's build.

**Measurement — the shared statement puts the boundary's projection under refinement's authority, and this was observed rather than argued.** Perturbing `silu_point_body` to divide the divisor by the argument was watched failing at `compile.lowering.refinement-refused`: the *index-region* half of the shared statement no longer realized the occurrence, and `legality::refine_index_region` caught it before any region was scheduled. That is exactly the composition the Boundaries section asked for — refinement remains the authority, and the boundary's projection inherits its proof because the two are one statement.

### What is implemented

- **`crates/tiler-compiler/src/elementary.rs`** (new). `ElementaryPointSink` is the smallest per-point vocabulary the admitted bodies reach — constant, add, multiply, divide, exp, and deliberately no reciprocal and no negate, so the two-rounding spellings the pinned reference forbids are unstatable. `silu_point_body` is the one statement of `x / (1 + Exp(-x))`. `PointwiseExpressionSink` realizes it into `PointwiseF32ExpressionBuilder`; `GovernedElementarySink` in `governed.rs` realizes it into `IndexAccessLoweringContext`.
- **`request.rs`.** `ElementwiseFamily` gains `Silu` with a per-family declared operand count, and `recognize_elementwise`'s walk is arity-general rather than binary-only. A silu occurrence is projected through the shared body; every other rule the walk enforces — shape, attributes, reads, node limit, cover — applies unchanged.
- **`target/accuracy.rs`.** `required_elementary_accuracy` is the requirement side of the existing three-row `installed_elementary_realizations` table, reading each family's contract from `tiler-ir`'s own registered constructor. `assess_program_elementary_accuracy` deduplicates the program's obligations by operation and requires each to be provably refined. `declared_elementary_realizations` gates the installed rows on the target being byte-identically the governed profile, because every row is attributed to `governed_profile_source()` and reading them onto another profile would attribute a quoted specification guarantee and a measured corpus to a declaration that never made either.
- **`request.rs`'s `require_elementary_accuracy`**, called per target in `verify_request` beside the dtype-dispatch check and *before* numerical-contract resolution — the obligation is the registered operation's and no statable contract widens or waives it — and again in `readmit_candidate`, so a rewrite cannot inherit an admission granted to a program that did not contain the family. A refusal is `RequestError::UnrealizedElementaryAccuracy`, target-local, classified `UnsupportedCapability` (install or declare a realization), carrying the refusing authority's own stable key.

This is the call site the `#![allow(dead_code)]` on `target/accuracy.rs` said was waiting on "a whole-program recognizer that admits an elementary operation". Its reason is rewritten to name what is still unconstructed: the *structured* reporting either outcome carries.

### What is not implemented, and where it went

- **`tiler::reindex-f32@1` and `tiler::broadcast-f32@1` remain refused under `operation-set`**, and the wall is a `tiler-ir` one this ticket's scopes cannot reach: `LogicalAccess` has no reindex map, and its only broadcast is `ScalarBroadcast`, a rank-zero operand read once. There is no *projection* available the way there is for an elementary body, because what is missing is the access relation rather than the per-point arithmetic. [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) owns the widening; this ticket's outcome is revised to the one family it delivered.
- **A caller-built target profile cannot declare an elementary realization**, so it fails closed on any program containing one. Adding the declaration is a `TargetProfileBuilder` public-boundary change — Tom's — and [`declare-elementary-realizations-on-a-target-profile`](declare-elementary-realizations-on-a-target-profile.md) carries it together with the structured public refusal it would explain.
- **`docs/compiler/optimizer.md` still lists all three families as refused** in its "What each stage is general over today" section. `docs/compiler/**` maps to `contracts/optimizer`, which this ticket does not hold and a live sibling does; [`correct-the-optimizer-stage-generality-claims-for-the-admitted-activation`](correct-the-optimizer-stage-generality-claims-for-the-admitted-activation.md) carries it.

### Drafted ADR body, to be landed byte-identically

The body below is written to be copied verbatim into `docs/decisions/` by its carrier ticket. Its status is **proposed**; nothing here records an acceptance.

---

```markdown
---
schema: "tiler-doc/v1"
id: "tiler.decision.elementary-family-projection"
kind: "decision"
title: "Project an elementary family's per-point body from one shared statement"
topics: ["optimizer", "numerics", "operation-extensions"]
decision_status: "proposed"
---

# Project an elementary family's per-point body from one shared statement

**Status:** proposed

## Context

A registered semantic family whose normative definition pins a *composition* —
`tiler::silu-f32@1`'s `x / (1 + Exp(-x))`, with the negation exact and the
addition and the division rounding once each — has to be realized twice inside
the compiler. The governed index-access lowering emits it as `tiler_ir::index`
scalar applications, which occurrence refinement then proves realizes the
occurrence. The request boundary projects it into the physical
`PointwiseF32Expression` the scheduled region carries to a backend.

Two routes were proposed for making such a family reachable at all:

1. the region vocabulary grows a node per admitted elementary family; or
2. a region gains a way to name an occurrence whose per-point body *is* the
   resolved capability's emitted index region.

## Decision

**The region vocabulary spells an elementary family's per-point body in its
existing primitive nodes, and the compiler states that body exactly once.**

A family is admissible at the request boundary when its per-point body is
expressible in `PointwiseF32Node`. The body is written in one place, against an
abstract per-point sink whose vocabulary is deliberately smaller than the node
enum, and every realization — the index-access lowering's and the request
boundary's projection — is driven from that one statement.

## Consequences

A family whose body is expressible needs no `tiler-ir` change to become
reachable, and gets none: `Exp`, `Divide`, and `Rsqrt` were already nodes, and
already emitted by the Metal backend, before any of them was reachable through
the request boundary.

The boundary's projection is not an independent claim. Because the index-access
lowering emits the same statement, occurrence refinement's proof that the
emitted region realizes the semantic occurrence is also evidence about the
projection: a change to the composition that made it stop realizing the
occurrence fails at refinement, before any region is scheduled.

A family whose *access relation* has no spelling — a reindex, a non-scalar
broadcast — is not made reachable by this decision and continues to refuse by
name. The missing vocabulary there is `LogicalAccess`, and no projection
substitutes for it.

## Alternatives considered

**One node per elementary family.** A `PointwiseF32Node::Silu` would preserve the
semantic family down to the backend. It was rejected because the node vocabulary
is deliberately rounding-explicit — it carries `Divide` rather than a reciprocal
node precisely so that a two-rounding substitution is unstatable — and a single
node standing for a four-rounding composition hides what the vocabulary exists
to expose. It would also relocate rather than remove the second authority, since
each backend would then re-derive the composition.

**Embedding the resolved capability's emitted index region as the region body.**
This was the more interesting route: it would make an out-of-crate provider's
family reachable without a `tiler-ir` change per family, and refinement's proof
would carry over by construction. It was rejected because the emitted region's
scalar vocabulary is an open, registry-driven `ScalarOpKey` space, while
`PointwiseF32Node` is closed — and that closedness is what makes a new physical
meaning a build error at every schedule-identity, KIR-lowering, and backend
emission site rather than a silently unlowerable body. Restricting the embedded
body to a closed subset restores the property and reduces the route to this
decision plus a projection.

**Decomposing the family at the boundary without a shared statement.** Rejected
because it creates two independent claims about one meaning, only one of which
any authority checks.
```

---

### Verification

`cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo nextest run -p tiler-compiler` (614 passed, 1 skipped); `cargo clippy -p tiler-compiler --all-targets -- -D warnings`.

Every new check was perturbed once and watched failing:

| check | perturbation | observed failure |
| --- | --- | --- |
| the recognizer admits the activation | *the pre-change tree* | three tests asserted `operation-set` for a silu program and failed with `Ok(())` |
| `elementary::tests::the_activation_body_projects_to_the_pinned_expression` | swap `silu_point_body`'s division operands | rendered composition mismatch |
| `conformance::the_activation_compiles_and_matches_the_reference_bit_for_bit` | swap the division operands in `PointwiseExpressionSink` alone, leaving the index-region half correct | bit mismatch against `tiler-reference` on the first row |
| the shared statement is under refinement's authority | swap `silu_point_body`'s division operands | `compile.lowering.refinement-refused` before any region was scheduled |
| `conformance::omitting_the_activation_capability_refuses_the_recognized_occurrence` | make `registry_without` omit nothing | the compile succeeded where `missing-capability` was asserted |
| `conformance::a_profile_declaring_no_elementary_realization_refuses_the_activation` | make `declared_elementary_realizations` ignore the target | `None` where `UnrealizedElementaryAccuracy` was asserted |

One check is preserved rather than added and cannot fire through a built program: `elementwise-arity`, which compares an occurrence's operand count against its registered family's. The semantic registry enforces arity at construction, so a program reaching the recognizer with a disagreeing count is invalid state. It was already unfirable in the same way before this change (`let [lhs, rhs] = … else`), and it is kept because the projection needs a total function over operand slices.
