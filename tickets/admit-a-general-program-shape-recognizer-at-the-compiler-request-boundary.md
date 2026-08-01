---
id: admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary
title: Admit a general program-shape recognizer at the compiler request boundary
status: in-progress
priority: p1
dependencies: []
related: [reach-a-verified-kernel-through-the-structural-families, carry-the-elementary-numerical-dimensions-in-the-region-realization]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
claimed_from: todo
assignee: worker-recognizer
lease_expires_at: 1785610282
---
## User-visible outcome

A semantic program whose exact shape the compiler has never been taught reaches the optimizer on the strength of its operations and values, instead of being refused because it matches none of three hardcoded whole-program templates.

## Why this exists, and why nothing owns it

**Fact — the boundary is a fixed three-way template match.** `select_supported_strategy` (`crates/tiler-compiler/src/request.rs:2138`) tries `normalize_serial_sum` (`:2573`), `normalize_contraction` (`:2180`), and `normalize_pointwise` (`:2373`) in that order and, when all three refuse, returns the collected refusal. Each demands an exact operation cover — a four- or five-operation scale-bias-then-strict-serial `Sum`, a well-formed binary contraction, or a four-operation pointwise expression over one input plus constants — so a program composing two of them, or containing an admitted family none of them spells, is refused before any target-qualified explain trace exists.

**Fact — the roadmap already names an owner, and that owner disclaims it.** `docs/roadmap.md:409` and `:410` each read "R6 needs the whole-program recognizer, which [`reach-a-verified-kernel-through-the-structural-families`] owns". That ticket's Non-goals (`tickets/reach-a-verified-kernel-through-the-structural-families.md:37`) say verbatim: "A `ScalarProgram` copy variant, a standalone materializing reindex kernel, **a general program-shape recognizer**, and anything about the contraction family — which is blocked by the same recognizer and has its own tickets." The job is attributed to a ticket that explicitly excludes it, so no node owns it.

**Inference — this is the highest-leverage compilation-infrastructure gap on the board.** Three registered families are held at R5 by this recognizer alone, each row saying so in its own words at `docs/roadmap.md:409-411` ("the ceiling is the recognizer, not the family"); the contraction reached R6 only because `normalize_contraction` was added as a *fourth template* rather than because the boundary generalized; and `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` depends on this directly.

## Boundaries

- Generalizing recognition is not licence to admit an unrecognized program silently. An unsupported program must still refuse with `UnsupportedCapability` and a rule naming the property that was not recognized, in the split-rule idiom (`input-arity`, `output-arity`, `operation-set`, `dtype-f32`) that `admit-multi-input-elementwise-programs-at-the-compiler-boundary` established and Tom accepted.
- A recognizer may only admit what the physical layer can express. Admitting a program at the boundary that then fails mid-pipeline is worse than the refusal — the failure mode `admit-multi-input-elementwise-programs-at-the-compiler-boundary` located and declined to ship. Where the wall is below the boundary, file that widening as its own ticket and depend on it, as that precedent did.
- This is pre-production software with no external consumers: a normalization the general path subsumes is removed, not preserved beside its replacement.

## Closes when

A semantic program that no current normalization matches — minimally, one composing two admitted families in a single program — compiles through `tiler_compiler::session` to an emitted region; every refusal path names the unrecognized property under its own rule and each was observed failing against an accepted neighbour; and the roadmap's attribution of the whole-program recognizer is repointed at what this ticket lands. The roadmap edit itself belongs to [`correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings`](correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings.md) under `contracts/navigation`, which this ticket does not hold.

## Outcome

**The three whole-program templates are gone.** `normalize_serial_sum` and `normalize_pointwise` are removed, and `select_supported_strategy` no longer tries a sequence of shapes. It checks the three properties every recognized program shares — at least one declared input, exactly one output, `f32` throughout, each under its own rule — then classifies *the occurrence producing the output* and walks outward through the occurrences feeding it. A program whose exact shape nothing was taught is admitted when its occurrences compose into a region chain the physical layer can assemble.

**What generalized.** The elementwise dimension is now the whole `PointwiseF32Expression` vocabulary rather than a leaf count and an association. `recognize_elementwise` walks an arbitrary DAG — any depth, any number of declared inputs, mixed `add`/`multiply` families, shared subexpressions minted once, and rank-zero constant subexpressions evaluated per point — and it is the *same* walk for a whole-program elementwise program and for a reduction's prologue. `NormalizedPointwise` carries the recognized expression itself and `NormalizedSerialSum` carries its recognized prologue, so `normalized_pointwise_expression`, `scale_bias_expression`, and `scale_bias_expression_matches` in `physical.rs` are removed with the templates that needed them: the region builders and the request-subject binding now use the recognizer's own expression rather than rebuilding a spelling.

**What survived, and exactly why.** `normalize_contraction` survives as its own recognized body: a binary tensor contraction is one occurrence whose realization is `ScalarProgram::StrictTensorContraction`, and no elementwise expression spells a sum over indices shared by two operands. Its program-wide arity and dtype gates moved up into the shared prologue; its exact-cover check stayed, and now carries the reason the cover is exact (an elementwise epilogue over a contraction result has no region to be assembled into). The *fused* single-region serial-sum alternative also survives, and is now conditional rather than unconditional: `ScalarProgram::FusedMultiplyAddSerialSum` applies one scale and one bias per contributor, so `fused_region` returns `None` unless `fused_prologue_constants` recovers exactly that affine form from the recognized prologue. A general prologue therefore loses *an alternative*, never the program — the materialized two-region plan realizes every recognized prologue. `fused_prologue_constants` is the single authority the region builder, the subject binding, and the whole-program numerical proof all reach, so "a fused alternative exists" and "the fused equivalence proof is claimed" cannot disagree.

**The composed program.** `crates/tiler-compiler/tests/composed_family_recognition.rs` compiles `sum((a * b) + c, axis 1)` over three declared inputs through `tiler_compiler::session` to a complete verified plan, under every registered contract but the contraction-permitting one. It matched no superseded normalization: the serial-sum template demanded exactly one declared input and the exact `x * scale + bias` prologue, and the pointwise template refused any program containing a reduction. Its one-input controls travel with it, and both mixed multiply/add bodies — the composed program and the shape the old template spelled — are asserted to refuse *together* under the contraction-permitting contract, which is what makes that skip a statement about the adjacency rather than about the composition. `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` changed direction with it: `(a * b) + (c * c)`, this repository's pinned "still refuses" case, now compiles.

**The refusal table, and the neighbour each was watched against.**

| rule | program | accepted neighbour |
| --- | --- | --- |
| `input-arity` | an all-constant graph, whose unused declaration a frozen program drops | the same expression with one leaf as the declared tensor |
| `output-arity` | the composed program with a second value named as an output | the composed program itself |
| `dtype-f32` | unreachable through a built program; the semantic authority refuses a non-`f32` builtin operand first | — |
| `operation-set` | `silu(a) + c` folded — a registered family with a registered lowering capability and no expression node | the composed program, differing in exactly that one occurrence |
| `operation-set` | a contraction with an elementwise epilogue | the bare contraction |
| `operation-set` | a program whose output *is* a declared input | any program with one recognized occurrence |
| `reduction-prologue` | `sum(x)` over a declared input | the same fold with one elementwise occurrence between them |
| `elementwise-shape`, `elementwise-arity`, `elementwise-attributes`, `elementwise-reads`, `elementwise-node-limit`, `elementwise-result-arity`, `elementwise-operand` | reservations the semantic authority or the expression builder reaches first | stated at each site |

**Where a wall is genuinely below the boundary, and what owns it.** Three, each refused at the boundary with a named rule and filed with a dependency edge on this ticket: `admit-a-reduction-over-a-declared-input-tensor` (`tiler-ir`'s `verify_access_and_semantics` admits a `StrictSerialSum` region only when its contributor access reads `TensorRole::Intermediate`, so `sum(x)` has no region — found by reading the verifier after a compilation failed there, not inferred); `admit-elementwise-epilogues-over-a-materialized-intermediate` (no elementwise region this profile builds reads an intermediate, and no contraction region writes one — a `tiler-compiler` gap, *not* a schedule-IR one, because `TensorRole::Intermediate` is a per-region role); and `admit-the-registered-unary-families-at-the-compiler-request-boundary` (`silu`, `reindex`, and `broadcast` have capabilities and no `ScalarProgram` or `LogicalAccess` spelling, and decomposing one at the boundary would be re-deriving a provider's lowering). `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` already depended on this ticket; its stale citation of three per-template guards is corrected to the one program-wide guard that replaced them.

**Budgets became derived rather than spelled.** `verify_program`'s pre-strategy `host-expression-nodes` and `buffers` requirements were literals sized for one declared input; both are now derived from the declared arity (`2 * inputs + 7` and `inputs + 3`), and both still reach the old numbers at one input. The governed `buffers` budget moved from `4` to `6` — the split program over the widest prologue the governed target's four buffer bindings admit — which is inside `VerifiedRequestSubject::canonical_bytes` and therefore moves every request subject, as the previous widening of the same field recorded.

**Pins that moved, and why each had to.** The explain request digest moved twice and is rebaselined once, from `a532d35f0cfdd29a` to `701c39d4a41e1a22`: the request subject records *what program was recognized*, and this change replaced the serial-sum arm's two constant fields with the recognized prologue expression, the pointwise arm's fixed leaf triple with the general node run, gave the serial-sum arm its first sub-tag, stepped the enclosing domain to `tiler.compiler.request-subject.v3`, and moved the `buffers` budget. The domain stepped rather than only the sub-tags because a newly tagged arm cannot be proven unreadable as the untagged one it replaced. `STRICT_F32_REGION_IDENTITY_HEX` did *not* move and could not: it is a `tiler-ir` test fixture over a hand-constructed region, not over a recognized program. The `buffers: 4` assertion in `the_widened_budgets_admit_the_split_program_and_still_refuse_a_narrower_request` is rebaselined to `6` with the derivation stated beside it.

**A defect found and not absorbed.** `correct-the-declined-strategy-record-for-an-unsplittable-reduction` (p1): a reduction with fewer than four contributors fails the whole compilation with `InvalidCompilerOutput(Explain(InvalidStageEvent))` under the two reassociation-permitting contracts. Reproduced on `d0b8445` before any change here, with the exact program, shapes, and per-contract table recorded in the ticket.

**Public items.** None. Every type, function, and rule touched is `pub(crate)` or test-local; the two new integration tests drive only the existing public `tiler_compiler::session` and `tiler_compiler::target` boundaries.

**Not this ticket's.** The roadmap's attribution of the whole-program recognizer at `docs/roadmap.md:409-411` is `correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings`'s under `contracts/navigation`, and is handed to it unchanged.
