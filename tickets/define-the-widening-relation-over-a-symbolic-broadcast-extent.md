---
id: define-the-widening-relation-over-a-symbolic-broadcast-extent
title: Define the widening relation when a broadcast result extent is a symbol
status: done
priority: p2
dependencies: [relocate-the-sourced-extent-vocabulary-to-the-shape-module, carry-a-sourced-shape-on-semantic-values]
related: [decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode, resolve-semantic-shape-inference-over-symbolic-extents, assemble-the-decoder-layer-program, assemble-the-causal-self-attention-block-program, design-model-ingestion-and-complete-execution, design-autoregressive-state-and-kv-cache, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [research/shapes, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, shapes, semantics, broadcast, identity, language-model]
---
## User-visible outcome

One decoder-layer program serves every admissible new-position count, including one, so a prefill row and a decode row are one artifact identity rather than two — or the impossibility is recorded as a durable refusal with the count corrected everywhere it is asserted.

## Why this exists

**Fact — the family refuses a degenerate widening, and the refusal has no escape.** `BroadcastAxisMapping::new` rejects any `Replicate` or `StretchUnit` entry whose declared result extent is below two under `broadcast.mapping.relation-does-not-widen`, and rejects a mapping stating only one-to-one correspondences under `broadcast.mapping.no-many-to-one-relation` (`crates/tiler-ir/src/semantic/broadcast.rs`, source anchors `pub fn new` and those two rule keys). A mapping must state one source per declared result axis and consume every operand axis exactly once in ascending order, so a rank-one operand widened onto a rank-two result must spell exactly one axis as `Replicate` — and at a result extent of one that spelling is refused. `[1024] -> [1, 1024]` therefore has **no broadcast spelling at all**.

**Measurement, 2026-08-05, at `crates/tiler-reference/tests/decoder_layer.rs`.** The assembled decoder layer carries fifty-eight occurrences and seventy-six values at the C1 prefill row (`T = 10`) and sixty-two and eighty at the C1 decode row (`T = 1`): six position-axis widenings change spelling, two losing their broadcast entirely and four becoming a narrower broadcast plus a unit-axis insertion, so broadcasts fall from eleven to nine and reindexes rise from sixteen to twenty-two with every other key unmoved (`a_single_new_position_changes_six_widenings`). At a fixed `T` a deeper cache moves no occurrence at all (`a_nonempty_cache_changes_no_occurrence`), so the divergence is the new-position count and not the cache.

**Inference — relocating the existing check does not repair it, and both alternatives are worse than the divergence.** The layer declares `T` over `[1, 32768]`. Deciding the widening at construction against the environment's proved lower bound refuses the mapping outright, because `T >= 2` is not proved — the layer becomes unbuildable at *both* rows. Deciding it where the extent is bound refuses at `LiveDevicePreflight` on the decode binding, which turns a construction-time graph divergence into a run-time refusal at exactly the row that needed it. What is left is to *define* the degenerate binding rather than refuse it.

**Inference — the family's own ground for the refusal inverts under a symbol rather than merely weakening, which is why this is a new question and not a relaxation request.** The stated reason is that one relation must have one spelling. At a **literal** extent of one, a `Replicate` and a `Reindex::insert_unit_axis` denote the same function, so two spellings genuinely exist for one relation and refusing one of them keeps canonical identity injective over meanings. At a **symbol**, the two denote different functions over the interval and coincide at exactly one point of it — so the rule that protects injectivity over literal mappings is the rule that makes a symbolic mapping unspellable. The two domains need different rules, and a blanket relaxation of the literal case is separately wrong: it would mint two artifact identities for one program while leaving the row-dependent declared extents in place, costing injectivity to buy an occurrence-count cosmetic.

**Fact — the second, weaker prerequisite is unrecorded anywhere.** A mapping holds `result_extents: Vec<Extent>` and folds them into the occurrence's canonical attribute bytes, which [`assemble-the-causal-self-attention-block-program`](assemble-the-causal-self-attention-block-program.md) already pinned as checks for the attention half; so a mapping stays a per-row constant even after every value shape becomes symbolic. [Symbolic semantic extents](../docs/research/shapes/symbolic-semantic-extents.md) does not reach shape-declaring attributes: it never defines sourced mapping extents or a symbolic widening predicate. Mechanical check at repair (2026-08-10): `grep -cin "broadcast"` returns `2` (incidental post-2026-08-08 family-list and check-path tokens, not a design of mapping extents), `grep -cin "mapping"` returns `0`, and control `grep -cin "extent"` returns `65`.

**What this bears on.** [L6's complete-model record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) states "270 executions over exactly three artifact identities" and now carries its condition and D-19; [L5's autoregressive-state record](../docs/research/runtime/autoregressive-state-and-kv-cache.md) states that the decode step is not a second program design and now carries the half of that which the measurement refutes; [L8's qualification record](../docs/research/program-planning/model-level-qualification.md) pins the identity count and the cold pipeline-creation count of three, both now conditional.

## What this must decide

- Whether `BroadcastAxisMapping`'s declared result extents become sourced (constant-or-symbol) alongside value shapes, and what that does to the mapping's canonical encoding, its domain separator, and the `tiler::broadcast-f32@1` key's version.
- What a many-to-one relation **means** when its result extent binds to one: an identity that is defined rather than refused, or a typed refusal that stands and makes one graph over both rows impossible.
- Whether the literal-extent refusals `relation-does-not-widen` and `no-many-to-one-relation` stay exactly as they are for a literal extent — the position this ticket recommends, because their injectivity ground is sound over literals and only over literals.
- What the index-access lowering derives for a degenerate widening, since `BroadcastAxisSource` is deliberately not `#[non_exhaustive]` precisely so that a relation the lowering has not seen is a build error.

## Accepted decision — 2026-08-12

**Provenance.** Tom accepted this decision directly in the interactive orchestration session after reviewing the current-source Fact audit, identity consequence, physical-consumer contradiction, host proof cost, first implementation tranche, and strongest counterargument. His response was `okay agreeed, next decision`.

Replace the governed built-in `tiler::broadcast-f32@1` completely with `tiler::broadcast-f32@2`; do not retain parallel v1 and v2 built-in paths in this pre-production repository. V2 carries sourced result extents under a new mapping encoding and semantic definition. ADR 0052 requires the semantic-version change because the admitted attribute grammar and meaning change and no separately governed compatibility rule exists.

A literal `Replicate` or `StretchUnit` result extent must remain at least two, and a wholly one-to-one literal mapping remains refused. Those refusals preserve the canonical reindex spelling. A symbolic many-to-one result extent is admitted only when the program's exact `ExtentSources` proves it positive. It may bind to one, including when the environment determines it is always one; at that binding the same authored parametric relation degenerates to a bijection without refusal, fallback, graph rewrite, or alternative semantic operation. A symbol that may be zero is refused during semantic construction.

`FromOperand` requires proved equality between the operand and declared result extent. `StretchUnit` additionally requires its operand extent to be proved equal to literal one. Construction is split between context-free canonical syntax and environment-dependent semantic application; no second environment authority and no public unchecked sourced-shape route is admitted.

The existing concrete `LogicalAccess::BroadcastReplication` and `LogicalAccess::ReindexBijection` remain exact and MECE over their current subjects. A new tagged parametric broadcast access relation carries the sourced relation through index and schedule IR. It may degenerate at one; replication-only reasoning is permitted only when the environment proves actual widening. The compiler must not specialize the semantic graph or schedule identity at the request boundary.

Validation must solve the environment once per mapping and then check all axes, rather than invoking the current independently solving proof queries once per axis. The intended bound is one environment solve plus `O(rank)` mapping work and transient storage, with no derived solver cache entering canonical identity.

**Explicit exclusions.** No literal relaxation, zero binding, implicit unit-axis insertion, binding-time graph rewrite, fallback, parallel retained v1 implementation, second caller-supplied environment, or misclassification as one of the existing concrete access variants. Empty-domain broadcasting remains a separate future semantic decision.

## Current-base Fact audit — 2026-08-12, exact base `196f6ccedd34dbf8876dfed26f32cf28dd93f99a`

- **Verified:** `BroadcastAxisMapping::new` enforces the two literal canonicality refusals, carries `Vec<Extent>`, and owns the `tiler.broadcast-axis-mapping.v1` encoding; `BroadcastF32::infer` accepts only static operand shapes.
- **Verified:** `a_single_new_position_changes_six_widenings` still pins 58 operations at `T = 10`, 62 at `T = 1`, broadcast 11 to 9, and reindex 16 to 22. Targeted nextest reran this check successfully.
- **Verified:** `BroadcastReplication` requires actual widening by element count and explicitly refuses an extent-one rank pad. Consequently the old packet's phrase “what the index-access lowering derives” understated a required new physical relation and could not be closed in `semantic/broadcast.rs` alone.
- **Verified:** ADR 0052, anchor `Schema evolution changes an operation's semantic version`, makes an operation-key step mandatory absent a separate compatibility rule.
- **Imprecise:** the former dependency on sourced-shape sealing was an implementation-readiness gate, not a prerequisite to deciding semantics. It now blocks the first implementation child instead of this completed decision.
- **False as a complete graph:** no ticket depended on this decision, while the one-artifact delivery claim required its physical and artifact consequences. The accepted tranche below repairs that omission.

## Do not

- Do not relax the literal-extent refusals as a shortcut to equal occurrence counts. It leaves the declared extents row-dependent, so it does not deliver one identity, and it costs the injectivity the refusals exist for.
- Do not close this by asserting a count of four artifact identities for the C1 row. Four is the count under a vocabulary that supplies symbolic value shapes and symbolic mapping extents but not this decision, and nothing proposes that vocabulary; under today's fixed extents the row is thirteen and under a sufficient one it is three.
- Do not admit chunked decode (`T >= 2` always, one-token chunk padded) as an answer. It equalizes the occurrence count and delivers no identity, because the mapping still declares a literal `[2, 1024]` against `[10, 1024]`; it appends a position to the retained `K` and `V` that no admitted family can remove; and truncating a `[8, C+2, 128]` row-major payload is a copy rather than a re-declared extent.
- Do not repair this by widening a consuming family's signature — for instance `tiler::rms-norm-f32@1` accepting a weight of the reduced axis's extent. It removes four of the six widenings and leaves fifty-four against fifty-six, and it reintroduces the implicit broadcasting the broadcast family's own definition forecloses.

## Implementation tranche

- [`replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics`](replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics.md) owns the semantic replacement and is blocked by sourced-shape sealing.
- [`carry-the-parametric-broadcast-relation-through-index-and-schedule-ir`](carry-the-parametric-broadcast-relation-through-index-and-schedule-ir.md) owns the non-concrete access carrier and its total verifier/lowering consequences.
- [`admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary`](admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary.md) owns unspecialized compiler admission.
- [`prove-one-decoder-artifact-across-symbolic-broadcast-bindings`](prove-one-decoder-artifact-across-symbolic-broadcast-bindings.md) owns the end-to-end identity proof and record updates.

## Closes when

Tom has decided and the exact accepted semantics, identity rule, physical carrier, exclusions, and hard-linked implementation tranche are durable. Implementation evidence and the decoder-layer count collapse belong to the child tickets rather than being falsely claimed by this decision record.

## Graph maintenance

- Filed 2026-08-05 by [`decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode`](decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md), which ran the elimination and corrected the three records rather than taking the vocabulary change.
- Depends on [`relocate-the-sourced-extent-vocabulary-to-the-shape-module`](relocate-the-sourced-extent-vocabulary-to-the-shape-module.md) and [`carry-a-sourced-shape-on-semantic-values`](carry-a-sourced-shape-on-semantic-values.md): a predicate over a sourced extent on a mapping was not designable before the sourced vocabulary lived in `tiler_ir::shape` and a semantic value could carry one. Both are now `done`; their completion is what ripens this ticket into a decision rather than silently answering it.
- Related to [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md) rather than dependent on it: that ticket routes the *elementwise* rule through `proves_equal`, and this one is the same question for a shape-declaring attribute, which its record does not reach.
- **Dependency correction — 2026-08-11, refined at acceptance 2026-08-12.** Independent review proved the shared public `SourcedShape` representation does not enforce normalization and can produce a safe-Rust panic or duplicate canonical spelling. That is a hard implementation gate but did not prevent deciding the semantics after the exact unsafe representation and accepted repair were known. The dependency therefore moved to [`replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics`](replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics.md), while this decision retains only its completed vocabulary prerequisites. The provider-specific narrowing ticket remains unrelated to this governed-family decision.
- Declared `research/shapes` because the addendum belongs beside the symbolic-extent record, and `implementation/ir` because `crates/tiler-ir/src/semantic/broadcast.rs` is where the predicate and the normative definition live.

## Trigger check log

- 2026-08-05 — **not fired.** The mapping's declared extents are still literal, so no symbolic widening predicate has a subject: `BroadcastAxisMapping` holds `result_extents: Vec<Extent>` with `Extent(u64)`, and both dependency tickets are `todo`. Recheck: `grep -n 'result_extents: Vec<Extent>' crates/tiler-ir/src/semantic/broadcast.rs` — while that line exists, the trigger has not fired.
- 2026-08-09 — **fired; moved to `awaiting-decision`.** Both prerequisite tickets are `done`: the one sourced-extent vocabulary now lives in `tiler_ir::shape`, semantic values carry `SourcedShape`, and symbolic elementwise results are constructible. `BroadcastAxisMapping` nevertheless still stores `result_extents: Vec<Extent>`, so the unresolved boundary is now exactly this ticket's public/identity question rather than missing prerequisite machinery. **Recommendation:** keep the literal-extent-one refusals unchanged, admit a sourced mapping extent whose many-to-one relation is defined over the symbol's whole domain and remains canonical when one binding is `1`, and derive the exact key/domain steps from the grammar that lands. The counterpoint is that this widens a public attribute and its canonical encoding to make one graph cover a degenerate binding; refusing it instead preserves the smaller surface but durably requires separate prefill/decode graph identities.
