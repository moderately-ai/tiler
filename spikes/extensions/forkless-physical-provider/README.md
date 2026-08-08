# Forkless custom Metal physical provider

**Question.** Can a separately authored, statically linked crate outside this workspace contribute one specialized Metal physical implementation alongside Tiler's governed provider, without forking `tiler-compiler` and without replacing `tiler-metal`?

**Answer at `cb62784c` on 2026-08-08: yes, and the useful part is now the boundary rather than the blocker.** A provider crate in a different workspace, resolving `tiler-compiler` through its own lockfile, implements `PhysicalImplementationProvider` against the public surface, installs through `CompileRequest::with_physical_providers`, has each body re-verified by the host, and is retained as an additional plan alternative that names it as its authority. Its bodies emit through stock `tiler-metal` unchanged. Five subjects stay reserved, and the retained compile-fail fixtures pin those.

**This overturns the spike's own falsification, and the prior result is not withdrawn.** At `7b1e3a7`, re-verified at `63f9259` and `d5960e81`, the answer was **no** on two independent grounds: the provider vocabulary was behind a private module, and no method installed a physical provider. Both are gone. The [2026-07-31 record](results/2026-07-31-macos-arm64.json) remains a correct dated observation about the commit it names — and the two blockers it recorded are exactly the two the landing had to remove, which is the elimination [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) cites it for. The [2026-08-08 record](results/2026-08-08-macos-arm64.json) supersedes it as the account of the current tree.

## Run it

From this directory:

```sh
cargo nextest run --workspace
```

Eight tests, no host toolchain required beyond the repository's pinned Rust: nothing here invokes `xcrun`, allocates a device, or dispatches. `rustup` resolves the repository pin by directory ancestry, so no selector is needed and this spike deliberately carries no `rust-toolchain.toml` of its own.

`cargo nextest run -E 'test(reserved_provider_subject_diagnostics)'` runs only the compile-fail evidence, which is the slow half.

Spikes gate nothing. The `Makefile` has no target for this directory — its header says *"Spikes deliberately have no target. A spike is a recorded measurement, and it"* is re-run from its own directory by whoever is working on it — and this is a separate workspace that the root manifest's explicit member list does not reach, so `cargo check --workspace` at the repository root never builds it. Clear `target/` after a run; it is unanchored in [`.gitignore`](.gitignore) because `trybuild` writes `probe/target/` whatever `CARGO_TARGET_DIR` is set to.

## What is here

`acme-provider/` is the separately authored crate. It depends on `tiler-compiler` and `tiler-ir` exactly as an out-of-tree crate would — by path, no feature flag, no `#[path]` include, no private access — and it implements the whole of `PhysicalImplementationProvider`. Its specialization is the workgroup width: 32 threads per workgroup where the governed provider emits one.

`probe/` drives it through the ordinary compile path and reads the result back out of the public boundary. `tests/composition.rs` holds the seven runtime claims; `tests/ui/` holds the five compile-fail fixtures and their three compiling contrasts.

`results/` records the toolchain, the resolved blockers, the reserved subjects, and the falsification runs, one file per dated run.

## Why this workspace still exists once the seam has landed

`crates/tiler-compiler/tests/external_physical_provider.rs` reaches the same public surface from a separate compilation unit and measures more of the frontier's behaviour than this probe does. What it cannot say is what [`docs/operation-extensions.md`](../../../docs/operation-extensions.md) states as the bar — *"a surface has reached a tested guarantee only when a provider written outside the defining crate's own governed set has driven it through the ordinary compile path and the resulting plan names that provider as its authority"* — because it lives inside the defining package. That contract names this spike as the artifact that would upgrade the row, which is why the refresh is a re-run rather than a retirement.

The difference is a package and workspace boundary rather than a visibility one, and it should be claimed at exactly that width. An integration test already reaches only `pub` items. What it shares with the crate under test is the workspace: one lockfile, one dependency resolution, one `cargo` invocation, and `tiler-reference` as a dev-dependency of `tiler-compiler` whose features unify into the build that test links against. This spike shares none of that. `tiler-compiler` declares no Cargo features today, so the feature half of the difference is currently empty and is recorded as a seam rather than as a measured gap.

So the two are complements: the in-tree test measures the frontier's behaviour, and this measures that the surface it uses is genuinely the one a published crate would get.

## What the retained fixtures pin, and what changed about that

Until 2026-08-08 they pinned an *absence* — no installation seam, no reachable vocabulary — and that is exactly the job they did: both went red the day the seam landed. **A red run means the question has been reopened, not that a golden should be blessed.** Re-pointing them at whatever the compiler now says would have produced a check nobody chose, which cannot fail meaningfully again.

They pin the five subjects the host reserves instead. Four of the five are also `compile_fail` doctests carrying their exact error code on `crates/tiler-compiler/src/physical_provider.rs`, so those are stated from two independent places; the fifth is stated only here.

**Fact — the verified request is not reachable.** `ImplementationContext::request` is private, so a provider cannot re-derive the host's normalization and disagree with it. What it reads instead is stated positively — the assessed target profile, the resolved numerical realization, the region subject, and the host's own baseline spelling of it. [`verified_request_is_not_reachable.rs`](probe/tests/ui/fail/verified_request_is_not_reachable.rs).

**Fact — a cost estimate cannot name another model.** `PhysicalCostEstimate::new` is private and `::structural` is the only reachable constructor, so attributing an estimate to a provider's own model has no spelling at all. The key itself is readable as `GOVERNED_PHYSICAL_COST_MODEL_KEY`; reading and writing are different rights and only the first is granted. [`cost_estimate_cannot_name_another_model.rs`](probe/tests/ui/fail/cost_estimate_cannot_name_another_model.rs).

**Fact — a region subject's members are not readable.** A member is a graph-local *authoring* coordinate, so exporting the ordinals would put an authoring accident into a provider's decision and, through a decline cause, into the trace. `covered_occurrences` is the count and is public. [`region_subject_members_are_not_readable.rs`](probe/tests/ui/fail/region_subject_members_are_not_readable.rs).

**Fact — the enumeration itself is not reachable.** `mod frontier` stays private even though the vocabulary it defines is publicly re-exported, so this is a module gate rather than an item one. Installing a provider and running the frontier are different rights: the re-verification the seam rests on happens inside `enumerate_frontier`. [`frontier_enumeration_is_not_reachable.rs`](probe/tests/ui/fail/frontier_enumeration_is_not_reachable.rs).

**Fact — a scheduled kernel is the only proposable body, and nothing else checks it.** `ProposalBody` and `ImplementationProposal::new` are both private, so the three bodies a provider may not propose are refused by having no spelling rather than by a runtime rejection it could mistake for a target verdict. This is the one reserved subject with no in-tree doctest; the restriction is stated in prose on `ImplementationProposal::scheduled_kernel` and this fixture is the only check over it. [`scheduled_kernel_is_the_only_proposable_body.rs`](probe/tests/ui/fail/scheduled_kernel_is_the_only_proposable_body.rs).

Each has a compiling contrast under [`pass/`](probe/tests/ui/pass), because a diagnostic says what the compiler rejects and not what it accepts, and the finding here is that a provider is *installable and bounded* rather than blocked.

**One boundary is deliberately not pinned as a golden.** The governed provider cannot be displaced — `InstalledPhysicalProviders` offers `governed`, `installed`, and `identities`, and removal has no spelling. A fixture asserting `E0599` on some invented removal method would pin the absence of *that name* and nothing more, which is the weak kind of check this refresh exists to remove rather than add. The property is asserted where it is observable instead: the governed provider's identity is still named by a retained plan when only a third party's provider is installed.

## What is *not* in the way

**Measurement — stock Metal emission is reusable unchanged.** `acme-provider` does not depend on `tiler-metal` at all. The probe lowers the bodies that actually reached the frontier with `tiler_ir::kernel::lower_scheduled_region` and emits them with `tiler_metal::emit::emit_translation_unit`, both public, and the emitted units pass `require_declared_realization` against the measured Apple facts under the realization the *compiler* resolved. Emission consumes verified kernels and knows nothing about who proposed them, which is exactly the separation that makes partial composition work.

**Measurement — the specialization is real, identity-bearing, and collision-free.** `threads_per_workgroup` is free under the intrinsic verifier and is folded into the canonical scheduled-region identity, so the 32-thread and 1-thread implementations of one region are additive alternatives with distinct identities. They emit byte-identical kernel bodies under *distinct entry-point symbols*, because the symbol is derived from that identity: two alternatives of one region would not collide in a translation unit holding both, and launch geometry is carried by the dispatch.

**Measurement — a trusted provider is not a believed one.** Two structurally invalid bodies, perturbed separately — a zero-thread workgroup and a launch grid one thread short of the iteration domain — each fail the whole compilation with `CompileFailureClass::InvalidCompilerOutput`. Reporting a provider whose IR is wrong as "this provider had nothing to offer" would make a defect indistinguishable from silence.

**Measurement — the body a provider proposes is the host's own spelling, specialized.** `ImplementationContext::baseline` hands back this compiler's single-dispatch region for each subject; the provider clones it and moves one field. This is why the crate is now short. The request-subject binding compares a proposed region's identity, iteration shape, scalar program, semantic members, and access map, so a hand-built body has to reproduce all five — and the two earlier revisions of `acme-provider` did, and both stopped compiling when `tiler-ir` moved underneath them. That maintenance burden is gone, and it was never evidence about the seam.

## What this spike still cannot answer

*Two providers' cost estimates were never compared.* The specialization declares the baseline's own structural estimate, so neither dominates and both survive. Whether two providers' independently computed estimates rank sensibly against each other is unmeasured; the seam makes it constructible and nothing here exercises it.

*The offered provider set is still lowering-only.* `Compilation::offered_providers` names neither the installed physical provider nor the governed one, while the same compilation's `PlanAlternative::selected_physical_providers` names both. So a caller reading only the offered half cannot tell a registration that failed to take effect from a provider that lost on cost. `InstalledPhysicalProviders::identities` closes half the gap; the compilation's own account of the environment it ran under has no reading. Recorded as a measured absence — whether that half should grow a physical row is ADR 0090 item 5's subject, not this spike's.

*Determinism and hard-feasibility refusal are measured in tree, not here.* `crates/tiler-compiler/tests/external_physical_provider.rs` covers both. The second needs a profile whose workgroup capacity is a compile-time declared fact, and this probe deliberately compiles against Tiler's own governed profile, which answers that capacity through a prepared-entry query — so the same specialization resolves as a deferred predicate here and produces no hard rejection to observe. Restating either out of tree would duplicate rather than strengthen.

## Measurement boundary

These are facts about one compiler, one host, and one commit, recorded in [`results/`](results). The `.stderr` goldens are the *rendering* of a diagnostic, not a stability guarantee: a later rustc could reword `E0603` or `E0624` with nothing in Tiler having changed, and the `E0624` goldens additionally quote the private item's own signature, so a signature edit in `tiler-compiler` moves them. They are retained anyway, because their job is to go red when the boundary moves. **A red run means the recorded boundary has moved and must be re-decided, never that a golden should be blessed to make it green.** Refresh one with `TRYBUILD=overwrite` only after deciding the recorded claim still holds, and re-record the toolchain in the same commit.

The claim each golden pins was predicted before the run and checked against the generated file: the error code and the exact item named. The *rendering* was generated rather than hand-written, which is a weaker discipline than the 2026-08-05 run's single hand-predicted golden and is stated as such — what carries the goldens is the falsification runs below rather than the prediction.

Nothing here measures runtime behaviour, cost-model comparability across providers, device execution, or numerical results.

## Proving the checks can say no

Six runs on 2026-08-08, reconstructible from the sentences below and not retained. Each copies `crates/` and this spike outside the repository, rewrites the three `tiler-*` path dependencies to absolute paths under the copy, and supplies a root manifest with the prototype members removed.

- **Control, unperturbed:** `8 tests run: 8 passed, 0 skipped`. The goldens normalize the dependency path to `$TILER_COMPILER`, so they are not tied to this checkout — which is what the explicit `path` keys in [`probe/Cargo.toml`](probe/Cargo.toml) exist for, and it is the assumption every perturbation below rests on.
- Publishing `mod request` and `ImplementationContext::request` turned `verified_request_is_not_reachable.rs` into `mismatch`, the `E0624` replaced by `error: type request::VerifiedTargetRequest is private` at a different span. One of eight; nothing else moved.
- Publishing `PhysicalCostEstimate::new` turned `cost_estimate_cannot_name_another_model.rs` into `Expected test case to fail to compile, but it succeeded.` One of eight; nothing else moved.
- Publishing `mod region` and `FrontierRegionSubject::semantic_members` turned `region_subject_members_are_not_readable.rs` into `mismatch`, the `E0624` replaced by `error: type region::SemanticStage is private`. One of eight; nothing else moved.
- Publishing `mod frontier` turned `frontier_enumeration_is_not_reachable.rs` into `mismatch`, the module gate becoming `error[E0603]: function enumerate_frontier is private` — the item gate behind it. **Two** of eight, because it also opens the `E0603` half of `scheduled_kernel_is_the_only_proposable_body.rs`; one boundary, read by two fixtures.
- Publishing `ImplementationProposal::new` alone, with `mod frontier` left private, turned `scheduled_kernel_is_the_only_proposable_body.rs` into `mismatch` with the `E0624` gone from the actual output and the `E0603` still there. One of eight, so that fixture's two properties are separately load-bearing rather than jointly satisfied by one perturbation.

In every run the three `pass/` fixtures stayed `ok`, which is what says a perturbation opened one boundary rather than breaking the build.
