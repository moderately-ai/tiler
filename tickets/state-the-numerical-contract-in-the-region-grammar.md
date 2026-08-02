---
id: state-the-numerical-contract-in-the-region-grammar
title: State the numerical contract in the region grammar, explicitly
status: review
priority: p1
dependencies: []
related: [decide-the-inline-frontend-numerical-contract, denote-a-reduction-region-in-the-inline-macro-grammar, calibrate-and-activate-parallel-reduction-selection, state-the-region-contract-statement-in-the-frontends-contract, state-a-numerical-contract-in-the-inline-dispatch-spike, check-the-stated-contract-on-the-semantic-fallback-path]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, numerics, public-boundary]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785634941
---
## User-visible outcome

A `tensor!` region states the numerical contract it compiles under, in its own text, and a region that states none is refused at expansion with a diagnostic naming what to write — no silent default chooses a program's numerical meaning.

## Decision provenance

Tom decided on 2026-08-01 at the live session, recorded on [`decide-the-inline-frontend-numerical-contract`](decide-the-inline-frontend-numerical-contract.md): avoid assuming defaults and be specific; sane defaults can return later when the project's shape is known. The bound declaration admits two contracts (`FLUSH_SUBNORMALS_TO_ZERO_F32` and `FLUSH_AND_REASSOCIATE_F32`), the two are different meanings rather than two settings, and the choice belongs to the consumer's text.

## Implementation keys

- The spelling is a **public syntax surface** and goes to Tom under ADR 0075 as a concrete draft, exactly as the `strict_serial_sum` trio did. Prefer a declaration-block statement beside `sym`/`in`/`deliver` naming the contract's constant-style name; the grammar denotes a *contract*, never a plan.
- Only contract names the composed vocabulary defines are admitted, refused at the token otherwise with the admissible names listed; whether the target honours the stated contract stays the compiler's feasibility question, refused downstream with its own typed diagnostic — the grammar does not pre-answer it.
- A region that states no contract is refused at expansion — after this lands there is no `CONTRACT` constant left, and the refusal names the statement to add. Update every in-tree consumer (tests, trybuild goldens, the facade pass/fail suites, `deliver_compiles_embeds_and_routes`) to state their contract explicitly; the goldens gain the refusal case watched failing.
- `tiler_macros::aot` module documentation records the decision, its date, and venue; `the_bound_declaration_admits_the_two_flushing_contracts` keeps pinning the admissible pair.

## Closes when

A region states its contract and compiles under it, a contract-less region is refused with the naming diagnostic (watched failing first), an unknown contract name is refused at the token, Tom has accepted the syntax, and targeted tests plus the batch gate pass.

## Landed 2026-08-01 — the drafted syntax, awaiting Tom's acceptance

**The statement.** A fifth declaration-block statement beside `sym`, `in`, and `deliver`, at most once per region, naming one contract with one ordinary identifier:

```text
in a: f32[4], b: f32[4], c: f32[4];
contract flush_subnormals_to_zero_f32;
out (a * b) + c
```

The admissible names are the five `tiler_compiler::session::NumericalContract` constants under the lowercase spelling of each constant's own name: `strict_f32`, `flush_subnormals_to_zero_f32`, `relaxed_f32`, `reassociate_f32`, `flush_and_reassociate_f32`. `crates/tiler-macros/src/numerics.rs` holds the table and nothing composes a contract of its own.

**A region stating none is refused at the invocation**, naming the statement to add and every name it may take — there is no `CONTRACT` constant left in `crate::aot`:

```text
error: this region states no numerical contract, so what its arithmetic means is undecided; add a
`contract` statement to the declaration block, as in `contract flush_subnormals_to_zero_f32;`, naming one of …
```

**An unpublished name is refused at the name**, and deliberately does not pre-answer whether the target can honour an admissible one:

```text
error: `fast_math` is not a numerical contract a region may state; this frontend names `strict_f32`, … .
Whether the target you deliver for can honour the contract you state is a separate question,
answered by the compiler when it plans the region
```

**The spelling elimination.** Keyword: `contract` over the runner-up `numerics`. A region does not state a field of study, it states one contract — the compiler's own type is `NumericalContract` — and every admissible name ends in the arithmetic type it speaks for, so nothing reads as unqualified. Both survive on consequences (one keyword token, one parse shape, one diagnostic set), so this is decided on what the word denotes rather than on a fork; `numerical_contract` was eliminated as un-keyword-like beside `sym`/`in`/`out`. Name casing: lowercase `snake_case` over the runner-up `SCREAMING_SNAKE_CASE`, which would have been byte-identical to the Rust constant. The ratified `strict_serial_sum` call already declines to mirror its Rust facade's casing (`StrictSerialF32Sum`), and every other name a region writes — `f32`, `macos`, `out` — is lowercase. `an_unpublished_contract_name_is_refused_at_the_name` pins the runner-up as a refusal rather than a fold, so a region admits one spelling per contract instead of two. A hyphenated spelling in the `deliver` style was eliminated on a consequence rather than a preference: `flush-and-reassociate-f32` lexes as nine tokens and `Span::join` is unstable, so an unknown name could only be refused at `flush`.

**Both layers are tested separately**, which is the split the grammar deliberately does not pre-answer. `strict_f32` resolves in `crate::numerics` (`every_published_contract_has_exactly_one_region_spelling`) and is refused downstream by the compiler's target feasibility check (`a_contract_this_declaration_cannot_honour_is_refused_at_the_target`, and the `_unhonourable` case in the `contract_statement_diagnostics` golden). `the_bound_declaration_admits_the_two_flushing_contracts` keeps pinning the admissible pair, and its second half now asserts every admitted contract is nameable from a region rather than that a frontend constant is one of them.

**Every in-tree consumer states a contract explicitly**: 64 regions across the `tiler` doc-test, six `pass` fixtures, and six `fail` fixtures, plus `contract: None` on the `region` module's syntax fixtures. All delivering fixtures state `flush_subnormals_to_zero_f32`, which is the contract every one of them already compiled under, so no artifact identity moved and no identity-domain step was taken.

**One diagnostic outside the statement changed, and had to.** `crate::aot`'s `NoFeasiblePlan` rendering named a declared extent as the usual cause. A stated contract the target cannot honour now arrives through that same class, so a consumer writing `contract strict_f32;` would have been told to shrink a shape that was never the problem; the arm now names both causes.

**Filed rather than absorbed**: [`state-the-region-contract-statement-in-the-frontends-contract`](state-the-region-contract-statement-in-the-frontends-contract.md) (`docs/integration/frontends.md` is a scope this ticket did not hold), [`state-a-numerical-contract-in-the-inline-dispatch-spike`](state-a-numerical-contract-in-the-inline-dispatch-spike.md) (its two regions no longer expand, and no `make` target reaches `spikes/` to say so), and [`check-the-stated-contract-on-the-semantic-fallback-path`](check-the-stated-contract-on-the-semantic-fallback-path.md), filed `deferred` because the fallback evaluates no arithmetic yet and so has no behaviour a stated contract could constrain.

**Awaiting Tom.** The statement is a public syntax surface under ADR 0075 and is a concrete draft, not an accepted boundary. `review` until he accepts the spelling.
