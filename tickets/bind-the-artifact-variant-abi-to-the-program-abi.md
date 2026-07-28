---
id: bind-the-artifact-variant-abi-to-the-program-abi
title: Bind the artifact variant ABI to the program ABI
status: in-progress
priority: p1
dependencies: [complete-program-identity-with-abi-guards-and-routing]
related: [prototype-artifact-program-model]
scopes: [implementation/artifact, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi, identity]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785209046
---
**Fact — two ABIs now describe one program, and nothing binds them.** `complete-program-identity-with-abi-guards-and-routing` gave `tiler_ir::program::VerifiedKernelProgram` its own ABI expression arena, applicability guard, per-stage launch geometry, and per-access accessible byte range, and folded all four into `tiler.kernel-program.v2` identity. `tiler_artifact::program`'s `VariantSpec` still declares its own guard, its own `LaunchSpec`, and its own per-binding `accessible_bytes`, on its own arena, under the separately versioned `guard_and_routing` schema.

**Fact — they are checked against the same third thing, not against each other.** `crates/tiler-artifact/src/program/builder.rs::check_bindings` proves a variant's accessible-byte expression evaluates to `access.view().window().length`, and `check_launch` proves its workgroup width equals `stage.kernel().requirements().threads_per_workgroup`. `crates/tiler-ir/src/program/builder.rs::check_stage_accesses` and `::check_stage_launch` prove exactly the same two equalities for the program's own expressions. So the two agree on every *value* by construction and on no *expression*: an assembler may package a variant whose accessible range is `UnsignedLiteral(24)` over a program whose own range is `rows * columns * 4`, and both verify.

**Why that matters and why it is not urgent.** Under static shapes the two forms coincide at every admissible binding, so nothing observable diverges today. Under dynamic shapes they do not: the artifact's expression is the one a runtime evaluates, and the program's is the one identity folds, so a cache hit keyed on program identity could serve an artifact whose runtime ABI computes a different number. The exact check that establishes the current state is `grep -n "accessible_bytes" crates/tiler-artifact/src/program/builder.rs crates/tiler-ir/src/program/builder.rs`.

**The one existing consumer already does the right thing by hand.** `prototypes/serial-sum-compile/src/bundle.rs::assemble` transliterates the program's arena onto the artifact's and resolves each variant use site from the replayed handle map, so its variant ABI *is* the program's. That is a producer convention, not a checked one, and this ticket is about making the artifact layer require it.

## User-visible outcome

The runtime ABI packaged for a variant is the ABI of the verified program it
carries. Program-owned applicability, launch, and accessible-range expressions
are derived from the program; callers cannot independently restate them.
Artifact-owned routing priority, launch preconditions, and deferred feasibility
remain explicit.

Whatever is chosen, `tiler.artifact-program.v2` identity changes meaning if a variant stops carrying its own expressions, so its domain tag and the `guard_and_routing` schema version are both in scope.

## Closes when

An artifact variant derives its applicability guard, launch geometry, and
per-binding accessible ranges from its bound `VerifiedKernelProgram`; callers
cannot construct a disagreement; artifact-owned launch preconditions, deferred
predicates, and routing rank are unaffected; any changed identity domain is
bumped with its reason recorded at the site; and `make full` passes.

## Ready to implement 2026-07-27 — shape confirmed, ADR 0075 status named

**Verified, so the next session does not re-derive it:**

- `VariantSpec` is `pub` with `pub applicability_guard: AbiExprId` (`crates/tiler-artifact/src/program/builder.rs:172`), and `EntrySpec`'s launch and per-binding `accessible_bytes` are likewise caller-supplied. Removing those three is a **public API removal**, so ADR 0075 applies: it may be built as a tested concrete draft, but the interface is Tom's to accept. `AGENTS.md` states that split in terms.
- The divergence is real but latent, exactly as recorded: the two ABIs are each checked against the same third value and never against each other, so under static shapes they coincide at every admissible binding and under dynamic shapes they need not.

**The implementation shape, which the ticket already settles.** `prototypes/serial-sum-compile/src/bundle.rs::assemble` transliterates the program's arena onto the artifact's and resolves each variant use site from the replayed handle map — its variant ABI *is* the program's. That is the convention to make checked: the builder performs the transliteration itself, and the three fields come off `VariantSpec`.

**One thing that got easier while this ticket waited.** `flatten-artifact-expression-identity` landed the shared arena primitives — `canonical_arena_traversal` and `compare_expr_nodes` are `pub` in `tiler_ir::program::abi`, and the artifact identity already numbers its arena from a use-site root list. A transliteration that derives the variant's expressions from the program now has one numbering to agree with rather than two key encodings, so the binding is a smaller change than when this ticket was written.

**Both version bumps are in scope and neither is optional**, per the ticket: `ARTIFACT_DOMAIN` (now `v5`, so `v6`) because a variant that stops carrying its own expressions changes what the identity means, and the `guard_and_routing` component schema for the same reason.

## Progress 2026-07-27 — the replay moved into the artifact layer

**Landed:** `ArtifactProgramBuilder::adopt_abi(arena, roots)` replays a verified program's ABI arena onto the builder's own, returning one slot per source position, `Some` exactly for the positions reached from `roots`. Plus `ArtifactBuildError::ExpressionOutOfRange` for a root or operand naming a position outside the arena.

**This is the mechanism the ticket needs, moved from a producer convention into the layer that should own it.** `prototypes/serial-sum-compile/src/bundle.rs::assemble` was the only consumer doing the right thing, by hand — the ticket says so in terms. Its `replay`/`reachable_from`/`resolve` are now the artifact builder's, so an assembler no longer has to know to transliterate.

**Two tests, one of them the load-bearing one.** `adopting_a_program_abi_replays_every_reached_position` asserts every named position is replayed **and that a second replay of the same arena mints nothing new** — the builder keys by content, so an arena naming one expression from two positions must resolve to one handle, or a variant would carry two spellings of one formula and identity would distinguish them. `adopting_an_abi_with_an_out_of_range_root_is_rejected` proves the bound is a typed rejection rather than a panic, and was verified to bite: deleting the range check makes it fail.

**Not done, and it is the half ADR 0075 reserves.** `VariantSpec.applicability_guard`, `EntrySpec`'s launch, and per-binding `accessible_bytes` are still caller-supplied, so a disagreement is still constructible. Removing them is a **public API removal** — `VariantSpec` is `pub` with `pub` fields — and `AGENTS.md` is explicit that a tested implementation is a concrete draft and not approval of its interface. What remains is wiring, not design: `check_bindings` and `check_launch` derive from `adopt_abi`'s map instead of validating a caller's expression, the three fields come off the specs, and `ARTIFACT_DOMAIN` (now `v5`) plus the `guard_and_routing` component schema both step, because a variant that stops carrying its own expressions changes what the identity means.

**The value of doing this half now** is that it is the part with no interface question, so the remaining step is a mechanical change against a tested primitive rather than a design and an API decision at once.

### Measurement 2026-07-27 — the disagreement is not hypothetical; it is what every fixture does

**Attempted and reverted:** requiring each variant expression to be structurally the program's own, via `compare_expr_nodes` over the map `adopt_abi` returns. It compiles and is a small, contained change — and turning it on fails **262 of `tiler-artifact`'s tests**, every one with `VariantAbiDisagreesWithProgram { use_site: LaunchThreads }` or an equivalent.

**That is the ticket's premise measured rather than argued.** The two ABIs really are checked against the same third value and never against each other, and the consequence is that *the entire fixture corpus declares its own launch and accessible-range expressions* rather than the program's. Only `prototypes/serial-sum-compile` does it the right way, exactly as this ticket says.

**So the cost is not the check — it is the corpus.** Whichever route is taken, deriving the expressions or requiring agreement, every fixture that builds a variant has to stop restating the program's formulas. That is mechanical but it is 262 call sites, and it is the reason this ticket is larger than its description suggests. Anyone estimating it from the "two functions to change" reading will be wrong by two orders of magnitude.

**What that implies for the route.** Deriving is now *cheaper* than checking, not more expensive: if the builder takes the expressions from the program, the fixtures simply stop supplying them and the churn is deletion. If it checks, every fixture has to be rewritten to supply the right thing. That inverts the ordering I recorded above, and it is worth stating plainly — the public API removal is the smaller change, not the larger one.

The check itself is reverted; `adopt_abi` and its tests remain, because they are the primitive either route needs.

### Measurement 2026-07-27, second attempt — the derive route, and where it actually stops

**Attempted and abandoned:** removing `VariantSpec::applicability_guard`, `LaunchSpec`'s `grid_threads` and `threads_per_workgroup`, and `BindingSpec::accessible_bytes`, and deriving all four from the bound program through `adopt_abi`. `ARTIFACT_DOMAIN` stepped to `v6` with its reason. `main` is untouched; the branch was deleted.

**What it established, in order:**

1. **The compile-side churn is 13 sites and every one is a deletion**, confirming the route inversion recorded above. The fixtures stop supplying the fields; nothing has to be rewritten to supply something different. That part of the estimate was right.
2. **But it then fails 266 tests at *verification*, not at compilation** — 126 distinct `ArtifactVerificationError`s, plus `ExpressionType` and `NonInterfaceRoot`. The build succeeds and the artifact is then refused.

**That second point is the real finding and it is not mechanical.** If the two ABIs were the same formulas differently spelled, substituting the program's would verify. They are not: the program's launch and accessible-range expressions **do not satisfy the obligations the artifact builder checks** — the phase and interface-root requirements `check_use` imposes, and the static-evaluation contract behind `ExpressionType`. So the divergence this ticket describes is deeper than restatement. The two layers require different *shapes* of expression for the same quantity, and binding them means reconciling those requirements, not just choosing which side supplies the bytes.

**What that changes for whoever takes this next.** The remaining work is not "delete three fields and thread a map". It is: establish why a program-owned expression fails the artifact's use-site obligations, decide whether the artifact's requirements are too strict or the program's expressions are under-constrained, and only then bind. That is a design question about the two layers' contracts, and it should probably be its own ticket ahead of this one.

**Retained from the attempts:** `adopt_abi` and its two tests, already on `main` — the replay primitive either route needs, and the only part of this that was ever mechanical.

### Split 2026-07-27 — this now depends on reconciling the two layers' obligations

The 266-test verification wall above is not this ticket's work. It is a question about what a verified program promises about its ABI, and it is filed as [`reconcile-the-artifact-and-program-abi-expression-obligations`](reconcile-the-artifact-and-program-abi-expression-obligations.md) with the reproduction, the failure classes, and the three candidate answers carried in — so none of it is re-derived.

Once that lands, what remains here is what the original estimate described: remove the three restated fields, resolve each use site through `adopt_abi`'s map, step `ARTIFACT_DOMAIN` and the `guard_and_routing` schema. The compile-side churn is 13 sites and every one is a deletion, which was measured rather than guessed.

### Correction 2026-07-27 — the split's premise is open

The 266-test wall was read as "the two layers require differently shaped expressions". That is not established: the captured failures include `ForeignHandle`, which is a handle resolved against the wrong builder — a wiring fault in the attempt, not a layer disagreement — and the 126 `ArtifactVerificationError`s could not be classified because their `Debug` dumps the builder and the cause is past the truncation.

The dependency on [`reconcile-the-artifact-and-program-abi-expression-obligations`](reconcile-the-artifact-and-program-abi-expression-obligations.md) stands, because that ticket's first job is now to separate the attempt's own bugs from genuine obligation failures. If there are none of the latter, it closes and this ticket reverts to the mechanical change its original estimate described.

### Correction 2026-07-27 (final) — there is no obligation conflict; the cause was `UnusedExpression`

The 266-test wall was `UnusedExpression`, unanimously: adopting the program's ABI while the fixtures still mint their own through `formulas(&mut draft)` leaves those unreachable from any use site, and the artifact correctly refuses an arena carrying nodes nothing uses. The artifact layer accepts program-owned expressions without complaint.

`reconcile-the-artifact-and-program-abi-expression-obligations` is closed as obsolete and the dependency is removed. This ticket is the mechanical change its original estimate described, with one addition now known:

1. Remove `VariantSpec::applicability_guard`, `LaunchSpec::{grid_threads, threads_per_workgroup}`, `BindingSpec::accessible_bytes` — 13 sites, all deletions.
2. Adopt once in `push_variant` via `adopt_abi`; resolve each use site from the map.
3. **Stop the fixtures minting what they no longer supply.** `formulas()` mints `rows`, `input_bytes`, `output_bytes`, `one`, and `always`; once the variant derives them, only what deferred predicates and launch preconditions still reference should be minted. This is the step the earlier attempts missed and it is deletion in one helper.
4. Step `ARTIFACT_DOMAIN` (`v5` → `v6`) and the `guard_and_routing` schema, with the reason at the site.

### Progress 2026-07-27 — the change works; six tests need a coverage judgement

**Built end to end and reverted at 6 failures from 132.** The derive route is correct and the earlier walls are gone. Sequence that got there, all four steps of the plan above:

1. Removed the three restated fields — 13 sites, all deletions, as measured.
2. Adopted once in `push_variant`; `check_launch` and `check_bindings` resolve each use site from the map.
3. **Trimmed `formulas()` to what a caller still supplies.** This is the step both earlier attempts missed. It minted `rows`, `input_bytes`, `output_bytes`, `one`, `always`; only the last two are still referenced, and minting the other three left them unreachable from any use site. That alone was **125 of the 132 failures** — `UnusedExpression`, exactly as diagnosed.
4. Stepped `ARTIFACT_DOMAIN` to `v6` with its reason at the site.

**The six that remain, and each one's disposition.** They are not defects; every one is a test whose subject the change makes unrepresentable, except the last:

| test | why it fails | disposition |
| --- | --- | --- |
| `rejects_a_guard_that_is_not_a_predicate` | the guard is derived; a caller cannot supply a non-predicate | delete — subject unconstructible |
| `rejects_a_guard_naming_a_fact_from_a_later_phase` | same | delete |
| `rejects_a_size_expression_naming_a_device_property` | accessible ranges are derived | delete |
| `rejects_an_expression_handle_from_another_builder` | reached the builder through a removed field | rewrite against a field that still exists — a launch precondition |
| `evaluation_reports_an_unbound_root_rather_than_guessing` | same shape | rewrite the same way |
| `artifact_identity_size_grows_linearly_with_the_abi_arena` | **real coverage.** It grew the *artifact's* guard to drive arena size; the guard is now the program's, so growth has to move to the program's ABI fixture | adapt — do not delete |

**Reverted rather than landed** because the last row is the one that matters: deleting three tests whose subject no longer exists is correct and already precedented in this change, but the linearity instrument is the evidence that artifact identity is linear in arena size, and adapting it means growing `fused_program`'s ABI instead. Getting that wrong silently removes the guarantee `flatten-artifact-expression-identity` established. That judgement wants a fresh session, not the end of one.

**Everything needed is on `main`:** `adopt_abi`, its tests, and the replay probe. The remaining work is the four steps above plus the six dispositions in that table.
