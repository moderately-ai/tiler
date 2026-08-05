---
id: admit-the-registered-unary-families-at-the-compiler-request-boundary
title: Admit the registered unary families at the compiler request boundary
status: done
priority: p1
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary]
related: [admit-the-silu-activation-family, admit-the-reindex-and-broadcast-operation-families, admit-the-rms-normalization-family, admit-the-softmax-family, land-the-elementary-family-projection-adr, admit-the-structural-families-into-the-scheduled-region-vocabulary, declare-elementary-realizations-on-a-target-profile, correct-the-optimizer-stage-generality-claims-for-the-admitted-activation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
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

#### Amendment 2026-08-04 — the frontmatter block completed to the decision schema, field by field

**What changed and what did not.** [`complete-the-elementary-projection-adr-frontmatter`](complete-the-elementary-projection-adr-frontmatter.md) amended the fenced block below in place, against the tree at `116f11ad`, after [the carrier](land-the-elementary-family-projection-adr.md) stopped rather than fork the draft. Only the frontmatter block and the H1's number prefix moved: `id` and the H1 gained the allocated number, and `catalog_group`, `implementation_status`, `applies_to`, `evidence`, and `ticket` were added. **Not one word of the body's prose changed**, so the carrier's byte-identical transfer is still a transfer. `decision_status` stays `proposed` — nothing in the deriving work relayed an acceptance and this amendment relays none. Each value's ground is recorded below rather than left for a reader to reconstruct, because a frontmatter field is an assertion in the corpus's typed graph and an unexplained one is indistinguishable from a guess.

- **`id` → `"ADR-0099"`, and the H1 → `# 0099: …`.** **Fact — 0099 is the next free number at this base.** `0098-state-an-inline-regions-delivery-policy-with-a-named-profile-or-a-family-list.md` is the highest allocated record (`ls docs/decisions/0*.md | tail -1`), and no other ticket holds a drafted body claiming a number: `grep -rn 'id: "ADR-0' tickets/` returns nothing at `116f11ad`. Both spellings moved together, which is the point the carrier's Required delivery added on 2026-08-04 — a record whose filename, `id`, and heading disagreed would spell two identities. The carrier's licence to adjust a number it finds taken is unspent and still stands; if 0099 is allocated elsewhere before the transfer, moving both spellings again is that licence's whole scope.

- **`catalog_group` → `"physical-planning-lowering"`.** The ticket offered `foundation-semantics-extensions` and `numerical-operations` as the two defensible candidates. Both are eliminated on the same test — what the record *decides*, not what it is about — and a third survives. **`numerical-operations` fails** because the record decides nothing numerical: it pins no formula, selects no rounding, states no tolerance, and admits no accuracy contract. The pinned composition is `tiler::silu-f32@1`'s registered definition, which this record consumes as a premise; filed beside ADRs 0023, 0024, 0042, 0080, and 0095 it would offer a compiler-structure decision to a reader browsing for numerical meanings. **`foundation-semantics-extensions` fails** because the record touches no semantic registration, mints no extension seam, and adds nothing to the semantic graph — its subject is the physical region vocabulary and the request boundary's projection into it. **What survives is where the corpus already files this record's nearest neighbours.** Of the seven decisions whose `applies_to` names `tiler.contract.optimizer`, four are `physical-planning-lowering` — [0007](../docs/decisions/0007-first-class-kernel-schedules.md), [0043](../docs/decisions/0043-use-typed-phased-target-feasibility.md), [0069](../docs/decisions/0069-use-a-general-compilation-boundary.md), and [0073](../docs/decisions/0073-own-typed-explain-in-tiler-compiler.md) — and 0069, the general compilation boundary, and 0007, the region model this record spells a body into, are the two it composes with directly. Reproduce: `for f in docs/decisions/0*.md; do grep -q 'applies_to:.*tiler.contract.optimizer' "$f" && { printf '%s ' "$f"; grep -h '^catalog_group:' "$f"; }; done`.

- **`implementation_status` → `"partial"`.** Confirmed against the field's own definition — "the highest implementation maturity the record's own decided behaviour has reached" — rather than inferred from the family count, which is the reading the ticket warned against. **Fact — the decided behaviour is a rule over a class**: "A family is admissible at the request boundary when its per-point body is expressible in `PointwiseF32Node`." **Fact — what is implemented is that rule for one named family, not as a predicate over the class.** `ElementwiseFamily` in `crates/tiler-compiler/src/request.rs:2779` is the closed enum `Add`, `Multiply`, `Silu`, and `silu_point_body` is the only body in the crate: `grep -rn 'fn .*_point_body' crates/tiler-compiler/src/` returns one line. A second elementary family needs a new variant *and* a new body function, so the generality the rule states has never been exercised. **Inference — that is exactly `partial` and it is not `spike-only`**: both realizations of the one body are implemented on the ordinary compile path and the shared statement was observed under refinement's authority (`compile.lowering.refinement-refused`, in the perturbation table below), so the decided behaviour is built and tested for part of its stated class. **One ground was deliberately not used.** The two structural families' absence is *not* evidence for `partial` here, because the record disclaims them — "A family whose *access relation* has no spelling … is not made reachable by this decision" — and citing work a record disclaims to lower its own status would misreport what the field measures.

- **`applies_to` → `["tiler.contract.optimizer", "tiler.contract.ir"]`.** Derived from the Decision's two conjuncts, each tested against a sentence the candidate contract actually carries.
  - **`tiler.contract.optimizer` — bound.** The second conjunct fixes what the request boundary admits and what may be projected into the physical expression vocabulary. [The optimizer contract](../docs/compiler/optimizer.md) states the projection's reachable set — "A one-input, one-output, three-leaf same-family `f32` add or multiply chain … can complete ordinary compilation through a bounded verified `PointwiseF32Expression` schedule projection … It is a closed physical vocabulary for the implemented `f32` profile, not a generic scalar IR and not a second semantic operation authority" — and the rule for everything outside it: "until then the applicable capability or verifier rejects it by name rather than projecting it into `PointwiseF32Expression`." This decision is what moves an elementary family across that line, and states the condition under which a family may cross.
  - **`tiler.contract.ir` — bound.** The first conjunct is about the region vocabulary, which is `tiler-ir`'s. [The IR contract](../docs/ir.md) carries the same closedness as its own fact — "It is a closed physical projection distinct from the index layer's registry-governed scalar SSA and introduces no generic dtype or operation authority" — and this decision governs that vocabulary's growth rule in both directions: an elementary family does not earn a node, and the alternative that would have embedded an open registry-driven `ScalarOpKey` space in a region body is rejected on precisely the property that sentence states.
  - **`tiler.contract.operation-extensions` — tested and rejected, with the check stated.** That contract states registration, resolution, refinement, provenance, contention, and seam classification; this decision changes no sentence about any of them. The reach it grants an out-of-crate elementary family is a downstream consequence of the two rules above, not a rule of the extension surface, and the record mints no seam — it *declines* one. Exact check: `grep -n 'PointwiseF32\|region vocabulary\|per-point\|expressible' docs/operation-extensions.md` returns nothing at `116f11ad`. `operation-extensions` stays in `topics`, which is where a faceted discovery term belongs; the metadata contract keeps `topics` free and `catalog_group` and `applies_to` governed for exactly this reason.

- **`evidence` → `["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]`.** This is the field the ticket flagged as possibly having no honest target. The carrier's elimination is **upheld for one record and refuted for the other**, and the refutation rests on a sentence in the metadata contract rather than on a preference.
  - **Upheld — the Metal record is not named.** [The Metal elementary-function accuracy guarantee](../docs/research/numerics/metal-elementary-function-accuracy.md) establishes Table 8.1's `exp <= 4 ulp` and correctly rounded `x + y` and `x / y` under the governed flags, the four gaps that stop those numbers being written into a contract, and which corpus expectations §8.5 reaches. Nothing this record decides rests on any of it: the decision makes no accuracy claim, and the machinery that consumes Table 8.1 is `crates/tiler-compiler/src/target/accuracy.rs`'s obligation assessment — a separate mechanism the deriving ticket implemented and this record neither states nor governs. Naming it would plant the false authority edge the ticket forbids.
  - **Refuted — the L3′ record is genuine evidence, because the carrier applied the wrong predicate.** The carrier's test was whether the record "reasons about projecting a body from one shared statement". That is the test for `adopted_by`, not for `evidence`. [The metadata contract](../docs/document-metadata.md#typed-relationships) states that "`evidence`, `informs`, and `adopted_by` are independent predicates: evidence may support a decision without that decision adopting the report's proposal", and the corpus uses the edge that way: ADRs 0055, 0080, and 0095 each name `tiler.research.numerics.reduction-semantics-and-legality` as `evidence`, while that record's `adopted_by` names only 0012, 0013, 0014, 0022, and 0025. Reproduce in two lines: `grep -l 'reduction-semantics-and-legality' docs/decisions/0*.md` against `grep '^adopted_by' docs/research/numerics/reduction-semantics-and-legality.md`. ADR 0055 is the closest analogue — a physical-realization decision resting on a numerics record that derives none of it.
  - **What the decision rests on, exactly.** The Decision names a sink "whose vocabulary is deliberately smaller than the node enum", and the Alternatives section rejects a per-family node because "the node vocabulary is deliberately rounding-explicit — it carries `Divide` rather than a reciprocal node precisely so that a two-rounding substitution is unstatable — and a single node standing for a four-rounding composition hides what the vocabulary exists to expose." Both clauses are load-bearing only if the two spellings are observably different binary32 functions; if `x / d` and `x * (1/d)` agreed, a `PointwiseF32Node::Silu` would hide nothing and the elimination would collapse. That is a measurement, not an axiom, and [the L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) is where it is measured *and* where its boundary is stated: the two SiLU spellings differ at three of thirteen finite arguments, by the mechanism the record names — "the product form rounds twice, at the reciprocal and at the multiply, where the division rounds once". The implementing code cites that measurement at the exact argument: `silu_point_body`'s doc in `crates/tiler-compiler/src/elementary.rs` reads "`x * (1.0 / d)` rounds twice and would be a different binary32 function — measurably so at `0xc2b00000`, where the two spellings differ by one ULP", and `0xc2b00000` is the `-88.0` the L3′ record isolates. The edge therefore points at the record grounding the decision's decisive premise, which is what `evidence` is for.
  - **The boundary, stated here because the body cannot carry it.** The L3′ record does not derive the projection, so a reader following the edge finds the pinned composition and its rounding structure rather than the one-statement argument. The body is fixed — this amendment touches frontmatter only and the carrier transfers byte-identically — so this note is the qualification of record, and it is reachable from the landed ADR through the `ticket:` field, which is what that field is for.
  - **Candidate (b) is not filed, and the reason is not cost.** Authoring a research record to fill a required field, after the decision it would justify has already landed, is ceremony rather than research: it would restate the deriving ticket's own derivation under a different kind and add an authority nobody consulted. It is also unnecessary once a genuine edge exists. If a later reader refutes the edge above — the refutation would be a demonstration that the rounding-explicitness ground is *not* load-bearing for the Decision — the correct repair is that ticket, not a placeholder.

- **`ticket` → `"admit-the-registered-unary-families-at-the-compiler-request-boundary"`.** The deriving ticket, as the amendment ticket directs. Optional by the schema and present on 96 of 98 records; nothing here makes this the third exception, and it is the route by which a reader reaches this note.

- **Deliberately not added: `depends_on`.** It is optional, 78 of 98 records carry none, and it is not among the schema gaps this amendment was filed to close. Inferring specific decision-to-decision edges from the body's prose would be design work, and re-deriving or extending the decision is the amendment ticket's stated non-goal.

**The schema checks, run against the amended draft rather than against the corpus.** Each is the carrier's own reproduce command narrowed to the fenced block, and each was run at `116f11ad` with the amendment applied. Extracting the block, written with a four-backtick span because its own pattern contains the three-backtick fence: ````awk '/^```markdown$/{f=1;next} /^```$/{f=0} f' tickets/admit-the-registered-unary-families-at-the-compiler-request-boundary.md > /tmp/adr-0099-draft.md````.

| Rule | Check | Result |
| --- | --- | --- |
| the ten required decision keys are present | `awk '/^---$/{n++; next} n==1{print $1}' /tmp/adr-0099-draft.md` | `schema: id: kind: title: topics: catalog_group: decision_status: implementation_status: applies_to: evidence: ticket:` — the corpus's ten plus `ticket` |
| `id` is the fixed uppercase form | `grep '^id:' /tmp/adr-0099-draft.md \| grep -cv '^id: "ADR-[0-9]\{4\}"$'` | `0` |
| the H1 carries the number prefix | `grep -m1 '^# ' /tmp/adr-0099-draft.md \| grep -cv '^# [0-9]\{4\}: '` | `0` |
| the title matches the H1 with the prefix removed | read side by side | both are *Project an elementary family's per-point body from one shared statement* |
| present arrays are nonempty, unique, homogeneous | read `topics`, `applies_to`, `evidence` | 3, 2, and 1 unique strings; no empty placeholder anywhere in the block |
| every `applies_to` and `evidence` target exists | `grep -rl 'id: "tiler.contract.optimizer"' docs/`, and the same for `tiler.contract.ir` and the L3′ id | `docs/compiler/optimizer.md`, `docs/ir.md`, `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md` |

**Each check was shown able to say no before it was believed.** The `id` check reports `1` when run against the pre-amendment `id: "tiler.decision.elementary-family-projection"`, the H1 check reports `1` against the unnumbered heading, and the key census omits five names against the pre-amendment block — which is the state the carrier's stop reported, reproduced deliberately from `git show 116f11ad:tickets/admit-the-registered-unary-families-at-the-compiler-request-boundary.md`.

---

```markdown
---
schema: "tiler-doc/v1"
id: "ADR-0099"
kind: "decision"
title: "Project an elementary family's per-point body from one shared statement"
topics: ["optimizer", "numerics", "operation-extensions"]
catalog_group: "physical-planning-lowering"
decision_status: "proposed"
implementation_status: "partial"
applies_to: ["tiler.contract.optimizer", "tiler.contract.ir"]
evidence: ["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]
ticket: "admit-the-registered-unary-families-at-the-compiler-request-boundary"
---

# 0099: Project an elementary family's per-point body from one shared statement

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
