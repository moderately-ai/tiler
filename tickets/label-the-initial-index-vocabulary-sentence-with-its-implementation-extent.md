---
id: label-the-initial-index-vocabulary-sentence-with-its-implementation-extent
title: Label the initial-index vocabulary sentence with its implementation extent
status: done
priority: p3
dependencies: []
related: [refresh-the-l2-derivation-s-symbolic-index-profile-source-claims, admit-symbolic-index-expression-coefficients]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

A reader of `docs/ir.md`'s bounded initial index vocabulary can tell the admitted contract from the implemented subset, so the L2-refresh class of staleness — a contract sentence read as a source fact — has one fewer site to arise at.

## The finding, from the L2 symbolic-profile refresh

**Fact.** `docs/ir.md:1037` states the bounded initial index vocabulary admits "addition/negation, multiplication by a parameter-only expression, and Euclidean floor division/modulo by a proven-positive parameter-only expression". The implemented index layer admits a `ShapeSymbol` only as a floor-division or modulo divisor (`IndexRegionBuilder::linear_combination` takes `IndexInteger` coefficients; no constructor accepts a symbolic addend or coefficient), so two of the three admitted forms are contract-only today. No ticket owns that divergence directly; the nearest is `admit-live-extent-operands-to-payload-indexing` (todo), which owns the payload-consumable half.

**The question, which is smaller than a correction.** The sentence states an *admitted* vocabulary, which is the corpus's normal ordering — contracts may lead implementation. What it lacks is a maturity label separating "admitted" from "implemented" per AGENTS.md's reserved-type/seam/implemented/tested discipline. Decide whether the sentence gains an explicit extent label (with the implemented subset named and the gap's owner cited), or whether the surrounding section already carries the distinction and only a cross-reference is owed. Read the whole section before deciding; do not weaken the contract to match the implementation.

## Closes when

The sentence and its section state the contract/implementation split explicitly, verified by a full section read, with the gap's owning ticket cited.

## Outcome — labelled, and the section's other two extent claims were stale in the same way (2026-08-07)

### Decision: the sentence gains a label, because the section carried the *pattern* and not the *fact*

Read whole, Layer 2 does have a contract/implementation seam: "Proposed first static index profile" opens by saying the slice implements "a deliberately smaller, fail-closed subset of this contract", and the paragraph that closes it says completing the profile "will not complete the symbolic contract above". So the section knows the distinction exists. What a reader cannot do is apply it to the vocabulary sentence, for two reasons that are both defects rather than stylistic gaps. First, the sentence sits ~55 lines above the profile subsection with no forward cue, so a reader who stops at it has no signal that it leads implementation — which is exactly how the L2 derivation read it as a source fact. Second, and worse, the profile subsection's own extent claims were themselves stale, so following the cross-reference would have produced a *wrong* answer rather than an incomplete one. A bare cross-reference was therefore not available; the sentence needed the implemented subset stated at the sentence, and the two stale claims needed correcting so the three agree.

The contract was not weakened. Every admitted clause stands verbatim; what was added is a labelled paragraph saying which clauses are implemented.

### The implemented subset, verified by source read rather than from the ticket

The ticket's finding said "two of the three admitted forms are contract-only today". **That is wrong, and the correction is the substance of the label.** Verified at `d5c02609` in `crates/tiler-ir/src/index/builder.rs`:

- `linear_combination` (`:1019`) takes `constant: IndexInteger` and `terms: &[(IndexInteger, IndexExprId)]`. Addition and negation are therefore fully implemented over exact integers, and multiplication by a parameter-only expression is implemented **only** in its exact-integer case. No symbolic addend, no symbolic coefficient.
- `floor_div` (`:1108`) and `modulo` (`:1120`) take `divisor: SourcedExtent` and are the only expression constructors that do. `admit_divisor` (`:1177`) returns `IndexExprClass::QuasiAffine` for a literal and `IndexExprClass::SemiAffine` for a symbol, after `ExtentSources::proves_positive` decides positivity from semantic input constraints alone. **The symbolic divisor is implemented and publicly accepted** — `promote-the-symbolic-index-profile-to-a-public-boundary`, accepted 2026-07-31.

So exactly one admitted form is contract-only: the symbolic coefficient. A second, narrower point the ticket did not raise: even the implemented divisor is narrower than "parameter-only *expression*" reads, because `SourcedExtent` is one literal or one declared symbol and "deliberately two cases and not an expression tree" (`index/sourced.rs:138`). Both narrowings are now stated.

### The gap's owner did not exist, so it was filed

The ticket named `admit-live-extent-operands-to-payload-indexing` as "the nearest" owner, and nearest is as far as it goes: that ticket owns a live extent reaching a **compiled payload's** address and loop arithmetic, not the IR's ability to represent the expression. Tracing the IR half: `represent-semi-affine-index-expressions-in-the-ir` carried coefficients **and** divisors and was closed `superseded` into `promote-the-symbolic-index-profile-to-a-public-boundary`; that ticket's user-visible outcome promises both, its Implementation outcome delivers only the divisor, and it was accepted on that surface. The coefficient half was neither delivered nor re-split, so it left the graph. Confirmed by sweep: no open ticket mentions a symbolic coefficient.

**Filed `admit-symbolic-index-expression-coefficients`** (p1, `implementation/ir` + `contracts/foundation`, public-boundary tag) with the source facts, the normalization decision the work must make deliberately (`accumulate_linear_term` folds by exact arithmetic and drops zero coefficients; neither is available for an unpinned symbol), the decline-don't-approximate interval rule, the identity-domain consequence, and the note that positivity is *not* the admission predicate for a coefficient because a coefficient may be any sign. `docs/ir.md` cites it, and cites the payload ticket separately as the distinct gap it is.

### Old → new in `docs/ir.md`

**1. The vocabulary paragraph (was `:1034`–`:1042`), unwrapped to the file's newer single-line convention.** Contract text unchanged; one clause added — "**This paragraph states an admitted vocabulary, and the implemented one is narrower; the paragraph after it says by how much.**" A new labelled paragraph follows it, opening "**Implemented extent, 2026-08-07 — a symbol reaches an index *expression* at exactly one position, and it is the divisor.**", naming `floor_div`/`modulo`/`proves_positive`/`SemiAffine`, then `linear_combination`'s integer coefficients as the reason no symbolic addend or coefficient is expressible, then the `SourcedExtent`-is-not-an-expression-tree narrowing, then the two non-expression positions a symbol does reach (domain extent, boundary axis), and closing on the two tickets.

**2. The reserved-accessor paragraph (was `:1120`–`:1124`) — stale, and superseded rather than pending.** It read: "Static dimensions and tensor boundaries expose optional `static_extent()` and `static_shape()` facts rather than unconditional universal extents/shapes. They return `Some` throughout this bounded profile. A future symbolic profile can return `None` and expose its `ShapeEnv` expression through an additive borrowed view instead of changing the meaning of an existing accessor." **Neither accessor exists.** `grep -rn "static_extent\|static_shape" crates/` finds them only in `shape/env.rs` (a different type) and in doc-comments recording their removal; `DomainDimensionRef::extent` returns `&'a SourcedExtent` (`index/model.rs:722`) and `TensorRef::shape` returns `&'a SourcedShape` (`:758`). The paragraph now opens "**Superseded 2026-07-31 by what the symbolic work actually landed, and the replacement is stronger than the reservation**" and records why the optional pair was rejected: its invariant — exactly one accessor answers `Some` — is unenforceable, since a third source kind makes both `None` and every consumer that encoded "not static, therefore symbolic" is silently wrong. This is a disclosed adjacent correction: same defect class, same section, same exclusively-held scope.

**3. The tracking paragraph (was `:1155`–`:1164`) — every pointer aimed at a `done` ticket.** It routed "semi-affine symbolic coefficients and divisors" to `admit-semi-affine-index-expression-class`, which is `done` and delivered only `ShapeEnv::proves_positive`. Rewritten to attribute each landed part to the ticket that landed it — root bindings, symbolic boundaries, the positivity query, then the divisor + `SemiAffine` + public vocabulary under the 2026-07-31 promotion, then index-domain predicates — and to name the coefficient as the one part that did not land, tracked by the new ticket.

### Coherence check across `contracts/foundation`

`grep -rn "parameter-only\|initial vocabulary\|semi-affine\|SemiAffine\|quasi-affine\|symbolic coefficient\|symbolic divisor"` over `docs/ir.md`, `docs/glossary.md`, `docs/architecture.md`, `docs/vision.md`, `docs/operation-extensions.md`: the vocabulary is stated **once** in the scope, at the sentence this ticket labelled. `ir.md:1067`'s predicate paragraph names the three classes but states no vocabulary and needs nothing. The scope has no second site to keep coherent.

**Three sites outside the scope state the same vocabulary and are named here for the coordinator, not edited.** `docs/roadmap.md:275` (`contracts/navigation`) restates it while arguing a contraction needs no new access class — the argument is unaffected by the implementation extent, so this is a candidate cross-reference rather than a defect. `docs/research/indexing/index-access-model.md:163`–`:164` (`research/indexing`) is the research memo the contract derives from and correctly states the admitted form. `docs/research/shapes/transformer-operation-and-shape-surface.md:128` (`research/shapes`) was already corrected on 2026-08-07 by the L2 refresh and states the implemented extent correctly; its "nearest owner" citation of `admit-live-extent-operands-to-payload-indexing` is now superseded by the ticket filed here and would be worth repointing under a `research/shapes` claim.

### Checks

Documentation and tickets only; **no `crates/` path is touched, so the workspace gate is untouched and no `cargo` check applies**. `tkt lint` clean, `git diff --check` clean, `tkt guard` reports no scope escape.
