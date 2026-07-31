---
schema: "tiler-doc/v1"
id: "ADR-0088"
kind: "decision"
title: "Admit tiler and tiler-macros as the consumer frontend pair"
topics: ["rust", "workspace", "dependencies", "frontends", "proc-macros"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.frontend-integration"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.macro-environment.build-environment", "tiler.research.embedding.artifact-costs", "tiler.research.cache.build-tool-exercise"]
depends_on: ["ADR-0045", "ADR-0049"]
refines: ["ADR-0077"]
ticket: "record-the-frontend-crate-admission-in-the-design-corpus"
---

# 0088: Admit tiler and tiler-macros as the consumer frontend pair

**Status:** accepted. Tom ratified the two-crate topology and the public `tiler::tensor!` path on 2026-07-30, accepted the exact facade surface on 2026-07-31 under [`admit-the-tiler-facade-and-proc-macro-crate-boundary`](../../tickets/admit-the-tiler-facade-and-proc-macro-crate-boundary.md), and accepted the artifact-family-selection boundary including the `tiler-macros` → `tiler-metal-aot` edge on 2026-07-31 under [`promote-artifact-family-selection-for-the-frontend`](../../tickets/promote-artifact-family-selection-for-the-frontend.md). It admits a tenth and eleventh reusable library and amends the packaging profile clause that withheld frontend and proc-macro crates.

Every other member carries an admission record — [ADR 0077](0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) for `tiler-metal-aot`, [ADR 0081](0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md) for `tiler-runtime`, [ADR 0082](0082-admit-tiler-cache-as-the-expansion-cache-owner.md) for `tiler-cache`, [ADR 0085](0085-admit-tiler-build-as-the-build-time-orchestrator.md) for `tiler-build`. These two carried none, because the admitting ticket held the `implementation/frontend` and `implementation/workspace` scopes and structurally could not write into `docs/`. This record closes that gap, in the same shape ADR 0077 closed it for the driver.

## Context

**Fact — the two crates exist and their edges are what a resolver reports.** At `5767204`, `Cargo.toml`'s `members` carries `crates/tiler` and `crates/tiler-macros`. `cargo metadata --no-deps` reports `tiler` with normal `[tiler-macros]` and development `[trybuild]`, and `tiler-macros` with normal `[tiler-metal-aot]` and no development dependency. `crates/tiler-macros/Cargo.toml` declares `proc-macro = true`; `crates/tiler/Cargo.toml` does not.

**Fact — Rust's proc-macro restriction is what makes this a pair rather than a crate.** A `proc-macro` crate may export nothing but macros, so the crate implementing `tensor!` can never also carry the runtime and frontend types a consumer needs. Making `tiler` the proc-macro crate would cap the facade at macros permanently; making `tiler-macros` the crate consumers import would either fix the public path as `tiler_macros::tensor!` or force generated tokens to name internal crates the consumer never declared.

**Fact — the accepted packaging profile withheld exactly these two crates.** [ADR 0056](0056-use-four-libraries-and-two-proof-executables.md)'s Decision says "No frontend, proc-macro, Candle, generalized cache, or reusable Metal-runtime crate is created for the first proof"; [`docs/architecture.md`](../architecture.md) repeats it as "until the proof reaches those boundaries"; and ADR 0077 item 5 restates it while warning that its own admission "must not" be cited as precedent for the crates the clause withholds. [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) separately puts "a new publicly reachable namespace — a new crate" in the always-ask category, which is why this is Tom's acceptance rather than a derivation.

**Fact — the frontend cannot state a delivery policy without reaching the offline driver.** [ADR 0049](0049-explicit-artifact-family-selection.md) requires every inline invocation to resolve a typed canonical `ArtifactFamilySelection`, and its one canonical encoder is `tiler_metal_aot::family`. `promote-artifact-family-selection-for-the-frontend` eliminated copying the vocabulary (two canonical encoders over one identity subject, which ADR 0074 convention 2 exists to prevent) and moving it beneath the driver (which spends the empty closure ADR 0077 item 2 decides). What survived is that the frontend depends on the driver, leaving only *which* frontend crate pays for the edge.

**Fact — a `proc-macro` crate's dependencies are host-built.** Cargo builds a `proc-macro` crate and everything it depends on for the host, and they never enter a consumer's target build graph. [ADR 0045](0045-bound-proc-macro-providers-to-host-dependencies.md) is the record that already reasons from that property, bounding an inline macro's operation-provider snapshot to "providers in the macro's host dependency graph".

**Fact — the corpus asserted the absence these crates falsify, and one assertion was executable.** `docs/status.md` carried `test ! -d crates/tiler-macros` and `! rg -n 'proc-macro\s*=\s*true' crates --glob Cargo.toml` under prose calling them reproducible; both inverted the moment the admission merged. `docs/architecture.md` said "nine reusable libraries", and `docs/research/cache/build-tool-exercise.md` gave "`crates/tiler-macros/**` is a mapped path with no crate behind it" as the reason the cache-root chooser had no owner.

## Decision

### 1. `tiler` is the one crate a consumer names

**Decided.** It owns the consumer-visible import path, the `tensor!` re-export, and the absolute paths generated tokens spell. A consumer writes `tiler` in its manifest and nothing else in the workspace is part of that contract; a consumer that had to name an internal crate to make generated code compile would be holding a dependency it never agreed to.

A procedural macro has no `$crate`, so its expansion must spell an absolute path, and that path has to terminate somewhere public. `#[doc(hidden)] pub mod __private` is where it terminates, and it terminates in the facade precisely so "generate only paths reachable through the consumer's declared `tiler` dependency" is true rather than intended. It carries no compatibility claim and is disclosed rather than hidden behind the attribute.

### 2. `tiler-macros` is the implementation half, and `tiler` → `tiler-macros` is the direction

**Decided.** `tiler-macros` owns token parsing, span mapping, expansion, and the frontend's statement of its artifact-family delivery policy. A normal facade re-exporting the macro is the standard direction and the only one that keeps both properties item 1 requires: the public path stays `tiler::tensor!`, and generated tokens resolve through a crate the consumer already named.

Neither crate creates a second semantic operation vocabulary, invokes runtime JIT, scans consumer source, requires a consumer `build.rs`, or hides a generated dependency the consumer did not receive. That is the accepted inline developer experience, not a property of the current placeholder.

### 3. `tiler-macros` → `tiler-metal-aot` is a normal edge, and its placement is the decision rather than the edge

**Decided.** The frontend must reach `tiler_metal_aot::family` to state a policy at all, and the edge sits on `tiler-macros` because a `proc-macro` crate and its dependencies are built for the host and never reach a consumer's target build graph. The macro crate therefore holds an edge to a process-spawning Apple toolchain driver at no cost to any consumer.

**Why the facade may not hold it.** The same edge on `tiler` would compile a driver that spawns `xcrun metal` and `xcrun metallib` into every consumer on every platform, and would publish Apple backend policy on a consumer-neutral boundary. That is the cost ADR 0077 item 4 already refused when it kept `tiler-metal`'s edge to the driver development-only, arriving here by a second route. Nothing a consumer writes needs the type: a delivery policy is stated in region syntax, and generated tokens name `#[cfg]` predicates and byte literals.

**The edge is normal rather than development** because the expansion itself states the policy, not only its tests. `crates/tiler-macros/src/delivery.rs` is reached from `tensor!`'s expansion path, and ADR 0053 forbids a selected-family build failure becoming silent fallback on the matching target — a refusal an expansion must be able to produce is not test machinery.

**`tiler-metal-aot`'s empty dependency closure is untouched.** ADR 0077 item 2 decides that closure, and it stays decided: this edge points *at* the driver, not out of it. The driver acquires no frontend, artifact, cache, or compiler knowledge, and its manifest still declares neither `[dependencies]` nor `[dev-dependencies]`.

### 4. Nothing in the workspace may depend on the frontend, and that half is checked

**Decided.** The frontend sits at the top of the workspace graph. An edge from any internal crate to `tiler` or `tiler-macros` would put a frontend's macro, grammar, and expansion machinery inside the compiler's dependency closure, which is the coupling the crate split exists to prevent. `tiler` is deliberately absent from `[workspace.dependencies]`, with the reason written at the point of absence, so the edge is not one an autocomplete can add.

`crates/tiler/tests/dependency_direction.rs` reads `Cargo.lock` — what Cargo actually resolved, merging normal, build, and development edges into one list per package — and fails if any non-frontend package holds a direct edge to a frontend package, or if `tiler` holds one to `tiler-metal-aot`. Both assertions name their population first: the test fails loudly if the two frontend packages are missing from the parse, if the facade's edge to `tiler-macros` is absent, or if `tiler-macros` has stopped depending on the driver, because otherwise "no offending edge" and "the check did not run" would be the same observation.

**This is the first checked slice of the packaging profile's edge table since `scripts/check_workspace.py` was deleted by `e197176`.** ADRs 0077 and 0081 each carry a correction recording that their closures became described rather than checked. One edge class is now checked again — nothing may depend on the frontend, and the facade may not carry the driver — and the rest of the block remains a description maintained by reading. A test covering one class is not a contract covering the table, and this record states which is which rather than letting the recovered check be read as more than it is.

### 5. The profile's "frontend, proc-macro" omission is amended, not reinterpreted

**Decided.** ADR 0081 was admitted by *applying* a test ADR 0077 stated, because `tiler-runtime` was not the crate the clause named. No equivalent move is available here: `tiler` and `tiler-macros` are exactly the frontend and proc-macro crates the clause withheld. So this follows ADR 0082's shape instead — the clause is superseded on this point rather than read to permit what it plainly refused.

It continues to withhold everything else it names. `tiler-candle` is not admitted, no reusable Metal-*runtime* crate is admitted, and ADR 0082's residue stands: a runtime pipeline-state cache, a compiler plan cache, and a general-purpose content-addressed store remain outside the profile. What ADR 0056 set was the rule that a crate is admitted when evidence requires one, not a fixed count; the evidence here is that Rust's proc-macro restriction makes a single frontend crate impossible and that a consumer must name exactly one.

### 6. Admission stabilizes neither the grammar nor the runtime adapter

**Decided.** `implementation_status` is `partial` and states a real gap. `tensor!` has no grammar: empty input states its delivery policy, validates it through the one canonical constructor, and expands to the inert `::tiler::__private::expansion_anchor()`; any non-empty input is a spanned `compile_error!` naming the tickets that own the region syntax. Empty input is a sentinel for "no region yet" rather than a case the eventual grammar accepts, and those tickets replace the body rather than extend it.

The facade re-exports no frontend or runtime types. Tom decided on 2026-07-31 that generated tokens route through facade-owned paths and that the exact re-exports arrive with their owning tickets where they are reviewed — `define-inline-symbol-binding-and-runtime-value-adaptation` for symbol binding and runtime value adaptation, and `promote-artifact-family-selection-for-the-frontend`, which reviewed its question and answered *none*, because no generated token and no consumer-written expression names a selection type.

## Consequences

- The workspace carries eleven reusable libraries and two non-published proof executables. The frontend rows are `tiler-macros -> [tiler-metal-aot]` and `tiler -> [tiler-macros]`. As with ADRs 0077, 0081, 0082, and 0085, that count is an ordinal about the crates being admitted rather than a new cap. **Correction — 2026-07-31, later the same day.** Both rows widened when `define-inline-symbol-binding-and-runtime-value-adaptation` landed: `tiler-macros -> [tiler-ir, tiler-metal-aot]` (the binding module derives a real `ShapeEnv` instead of restating the promoted symbolic-index profile) and `tiler -> [tiler-ir, tiler-macros]` (the value boundary re-exports `tiler_ir::program::StorageScalar` rather than minting a second storage-scalar authority). The no-inward-edge rule of item 4 is untouched — both new edges point *out* of the frontend — and [`docs/architecture.md`](../architecture.md) holds the live rows, per this record's own convention that an admission record restates the profile as of its acceptance.
- `tiler` additionally carries a third-party development dependency on `trybuild`, which the packaging block does not show because the block lists intra-workspace edges; `tiler-ir` carries the same one. ADR 0077 item 2 warns that `-> []` rows look alike while claiming different things, and the same hazard runs the other way: a row's edge list is not the crate's complete closure except where a record says so, which today is `tiler-metal-aot` alone.
- **ADR 0077 item 3's six-library block is not edited, and that is the established treatment rather than an omission.** Its block restates the profile as of its own acceptance; [`docs/architecture.md`](../architecture.md) already records that "ADR 0077's own six-library restatement is likewise an ordinal about the crate it admits and is not a cap", and ADRs 0081, 0082, and 0085 each added a library without touching it. Editing it now would retroactively rewrite an accepted record and would make one of four admissions behave differently from the others. The architecture contract holds the live profile; an admission record holds what was true when it was accepted.
- The frontend's compile-time evidence is out-of-tree by construction. `crates/tiler/tests/facade.rs` drives `trybuild` cases that compile as a separate crate, because an in-crate test resolves `crate::__private` and cannot tell a working expansion from one whose absolute path is wrong. What those fixtures cannot isolate is the manifest — `trybuild` copies the crate under test's `[dependencies]` into the generated project — so they prove that nothing a consumer *writes* or a macro *emits* names anything but `tiler`, while item 4's resolved-graph invariant is `dependency_direction`'s.
- The generated absolute path is fixed and its one bounded exposure is recorded rather than absorbed: `::tiler::__private::expansion_anchor()` resolves only while the consumer's dependency is named `tiler`, and a consumer renaming it in `[dependencies]` gets a resolution error at the call site rather than a wrong result. `resolve-the-generated-facade-path-under-crate-renaming` owns whether that stays acceptable.
- The expansion cache's root chooser gains a crate that could own it and does not gain a decision. [The build-tool exercise](../research/cache/build-tool-exercise.md) deferred the choice with the trigger "the first proc-macro frontend crate", on the ground that there was no caller to own it; the trigger has fired and the policy is still unmade. `tiler-macros` neither opens a cache nor names a root today, and Q-ART-004 remains the open question of record.

  **Correction — 2026-07-31.** The sentence above is retained as what was true when this record was accepted, and half of it stopped being true the same day. `tiler-macros` still opens no cache, and it now names a root: `crates/tiler-macros/src/cache_root.rs` resolves one, and [ADR 0089](0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) records the policy Tom accepted. Q-ART-004 is also narrower than this bullet leaves it — its root half is answered and its accounting and collection half is what remains open. The trigger this admission fired is spent rather than pending, and a reader who acts on the uncorrected sentence would redo a decision that has been made. The correction is appended rather than substituted for the same reason ADR 0077's six-library block is not edited: an admission record holds what was true when it was accepted, which is a reason to date a later fact beside it and not a reason to leave a present-tense claim standing after it became false.
- `ticketsplease.toml`'s `implementation/frontend` scope maps to a real package. It owns both crates through its globs and maps to `tiler-macros` for crate expansion, because that is the one with reverse dependents: `tiler` re-exports from it, while nothing may depend on `tiler`.

## Alternatives considered

**One crate.** The cheapest option and the one a reader reaches for first. Eliminated by the language: a `proc-macro` crate exports nothing but macros, so a single `tiler` carrying both the macro and the runtime types cannot exist. This is a hard constraint rather than a trade.

**Make `tiler-macros` the crate consumers import, with no facade.** Removes a crate and a re-export. Eliminated because it forces one of two losses: the public path becomes `tiler_macros::tensor!`, which is not the path Tom ratified, or generated tokens must name internal crates such as `::tiler_artifact::` directly, handing every consumer a dependency it never declared. The second is the concrete defect, not a style preference — a consumer's build would break on an internal crate it cannot see.

**Put the `tiler-metal-aot` edge on the facade instead.** Simpler to explain: one crate holds the frontend's outward edges. Eliminated for the two costs in item 3, the first of which is not a preference — a process-spawning Apple toolchain driver would enter the build graph of every consumer on every platform, including consumers that select `FallbackOnly` and compile no Metal at all.

**Copy the selection vocabulary into the frontend, or move it beneath the driver.** Both eliminated in `promote-artifact-family-selection-for-the-frontend` and not relitigated here: copying creates two canonical encoders over one identity subject whose bytes are folded into artifact identity, and moving it below `tiler-metal-aot` spends the empty closure ADR 0077 item 2 decides.

**Defer this record until the grammar lands.** Attractive because `implementation_status` would then be `implemented` rather than `partial`, and because a record written after the grammar could describe a real expansion. Eliminated because the corpus does not stay silent while a record is withheld — it kept asserting an absence that was false, including as an executable check that reported the opposite of the prose around it. That is the exact state ADR 0077 was written to end, and waiting would reproduce it.

## Traceability

The [prototype crate layout research](../research/workspace/prototype-crate-layout-and-msrv.md) is the evidence that the crate set mechanically enforces Tiler's layer separation rather than being a packaging convenience, which is what makes admitting one a decision. The [proc-macro build environment research](../research/macro-environment/proc-macro-build-environment.md) is the evidence behind the host-execution properties item 3 reasons from, and the [embedded artifact cost research](../research/embedding/embedded-artifact-costs.md) behind what a completed expansion must emit. The [build-tool exercise](../research/cache/build-tool-exercise.md) is the measurement whose deferred cache-root question this admission's trigger fires.

[The frontend and proc-macro integration contract](../integration/frontends.md) owns the inline delivery profile these crates will implement; [the system architecture](../architecture.md) owns the packaging profile and component ownership this record amends. ADR 0049 owns the canonical artifact-family selection the macro crate's edge exists to reach, ADR 0053 owns the family-gated delivery and its refusal of silent fallback, ADR 0045 owns the host-dependency bound on an inline macro's provider snapshot, and ADR 0075 is why this admission is Tom's acceptance rather than a derivation. The work records are [`admit-the-tiler-facade-and-proc-macro-crate-boundary`](../../tickets/admit-the-tiler-facade-and-proc-macro-crate-boundary.md) for the members and the accepted surface, [`promote-artifact-family-selection-for-the-frontend`](../../tickets/promote-artifact-family-selection-for-the-frontend.md) for the driver edge and its placement, and [`record-the-frontend-crate-admission-in-the-design-corpus`](../../tickets/record-the-frontend-crate-admission-in-the-design-corpus.md) for this record.
