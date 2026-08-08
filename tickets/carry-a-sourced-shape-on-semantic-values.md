---
id: carry-a-sourced-shape-on-semantic-values
title: Carry a sourced shape on semantic values instead of a fixed shape
status: in-progress
priority: p1
dependencies: [relocate-the-sourced-extent-vocabulary-to-the-shape-module]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/ir, implementation/compiler, implementation/reference, implementation/artifact, implementation/frontend, contracts/foundation, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, semantic-graph, api]
claimed_from: todo
assignee: w-carry-a-s
lease_expires_at: 1786162139
---
## User-visible outcome

A semantic value's shape may name a declared `ShapeEnv` symbol, so a program whose extents are bound at run time is constructible, verifiable, and inspectable through one total view.

## Why this exists

**Fact.** `ValueFact` and `ValueDefinition` hold a `Shape`, and `SemanticProgramBuilder::input`/`input_resolved` take one by value; `SemanticProgram::shape` returns `Result<&Shape, HandleError>`. Reproduce with `grep -n "pub fn shape" -A 3 crates/tiler-ir/src/semantic/program.rs`.

**Fact.** The accepted contract already admits symbolic semantic extents: "Each axis extent may be a static integer or a scoped symbolic expression evaluated later" ([the shape environment contract](../docs/research/shapes/shape-environment-contract.md)), and `docs/ir.md` records that completing the static profile "will not complete the symbolic contract above".

## Implementation keys

- Take the environment at construction — `SemanticProgramBuilder::try_standard_with_shape_environment(Arc<ShapeEnv>)` beside `try_standard()` — with no setter. The index layer's own decision record found that a repeatable setter's stated invariant was not held by its body, and a public one is a defect a consumer can reach.
- Add `input_sourced` and `input_resolved_sourced` beside the existing constructors. Do not add an environment argument to every static call site.
- Replace `SemanticProgram::shape`'s return with the total `&SourcedShape` view rather than adding an optional symbolic accessor beside the fixed one. The paired-accessor shape is the defect the index promotion removed, and it fails silently when a third source kind arrives.
- Expose the resolving environment as `SemanticProgram::extent_sources() -> Option<&ExtentSources>`, matching `VerifiedIndexRegion::extent_sources`: a symbol means nothing without the environment that declares it.
- A symbolic extent's phase ceiling is `EXTENT_PHASE_CEILING`. An input whose binding arrives later is refused at the constructor, not at build.
- Identity is out of scope and belongs to `fold-the-shape-environment-into-semantic-identity`; this ticket must not change canonical bytes. If that is impossible without a temporary inconsistency, say so and land the two together rather than shipping an unkeyed symbolic program.

## Evidence

- A symbolic program builds; the same program with a foreign symbol is refused as undeclared; the same program with a post-ceiling binding is refused as too late; each refusal paired with the accepted neighbour that differs only in the refused fact.
- A wholly literal program still returns `SourcedShape::Static` and its `as_static` borrow, so the normalization invariant holds at this layer too.
- Every new check perturbed once and observed failing before restoration.

## Public boundary

The builder constructors, `SemanticProgram::shape`'s return type, and `extent_sources` are all ADR 0075 items. `shape` changing its return type is the consequential one, because it moves every existing caller.

## Not started 2026-08-07 — dispatched, measured, and stopped; this ticket is not deliverable as scoped

A worker took this on 2026-08-07, **committed nothing** (`git diff` against its base empty, `cargo check --workspace` exit 0), and stopped on three of its four stop conditions after measuring each rather than inferring it. The findings below are the repair this ticket needs before it is dispatched again.

### 1. The scope set was wrong by four crates, and the measurement method matters

The ticket declared `implementation/ir` alone. The change moves **45 call sites across 5 crates** — `tiler-compiler` 24, `tiler-ir` 11, `tiler-reference` 7, `tiler-artifact` 2, `tiler-macros` 1 — and the compiler sites are real source (`request.rs` ×11, `normalize.rs` ×8, `program.rs` ×2, `region.rs`, `pipeline/conformance.rs`), not fixtures.

**How that was measured is worth reusing.** A plain rename reports only 8 in-crate errors, because the build fails inside `tiler-ir` and never reaches dependents — misleading, and the worker tried it first and discarded it as unsound. Attaching `#[deprecated]` instead warns without breaking the build, so dependents still compile and the whole population appears in one `cargo check --workspace --all-targets`. **Scopes are now corrected** to `implementation/ir`, `implementation/compiler`, `implementation/reference`, `implementation/artifact`, `implementation/frontend`, `contracts/foundation`. The precedent is decisive: `relocate-the-sourced-extent-vocabulary-to-the-shape-module` declared three scopes for a *14*-site move, so crate-graph reverse-dependency expansion does not cover dependents for scope purposes.

### 2. A stale Fact, and it is why the ticket missed a public accessor

The first Fact says "`ValueFact` and `ValueDefinition` hold a `Shape`". **`ValueDefinition` does not and never has** — it is `Input { input_index } | OperationResult { operation, result_index }` (`crates/tiler-ir/src/semantic/operation.rs:1564`), unchanged since its founding commit. The type holding the field is **`ValueData`** (`:1593`), and its public reader is **`ValueRef::shape`** (`:1637`) — a second public accessor this ticket's Public boundary section never names and which must widen identically. `ValueFact::shape` (`:1018`) is the one the ticket does name. Verified independently by the coordinator.

The misnaming is not cosmetic: it is the direct cause of the missed accessor.

### 3. Two of this ticket's own requirements are jointly unsatisfiable, and the graph proves it

It requires **both** "must not change canonical bytes" and "a symbolic program builds". `encode_shape` (`semantic/identity.rs:384`) writes rank then eight untagged big-endian bytes per extent, and every value's shape is encoded. A symbolic extent has no encoding there; an untagged-static/tagged-symbolic hybrid would be collision-ambiguous and would still leave `ShapeEnvIdentity` unfolded, so two programs spelled identically over differently bound environments would share an identity — **exactly the unkeyed symbolic program this ticket forbids shipping.**

Delivering it needs the `v2 → v3` tagged encoding and the fifth `SemanticIdentity` subject, which are [`fold-the-shape-environment-into-semantic-identity`](fold-the-shape-environment-into-semantic-identity.md)'s stated keys — and that ticket's Evidence demands every pinned identity be recomputed, which is the opposite of this one's.

**The coordinator then tried to add the dependency edge and the engine refused it as a cycle**: `fold` already depends on `carry`. So the mutual dependency is a mechanical fact, not an argument — **neither can be dispatched first, and they must land as one unit.** This ticket's own escape clause anticipated it: "If that is impossible without a temporary inconsistency, say so and land the two together."

### 4. The pinned population, enumerated

79 pinned literals across 16 `.rs` files, but **only three move on a semantic-graph encoding change**, all in `crates/tiler-build/src/metal_plan.rs` — `ARTIFACT_IDENTITY`, `CACHE_SUBJECT`, `FIXED_CONTENT_BYTES` — because the fixture builds a `SemanticProgram` and the artifact preimage folds the graph identity, with the cache subject composing over it. `crates/tiler/src/route/tests.rs`'s `IDENTITY_DOMAIN` moves additionally **if** the artifact domain steps. The `index/law.rs` and `schedule/builder.rs` pins do **not** move despite their fixtures building semantic programs.

**Whoever takes the combined unit must recompute those three on the *merged* tree, never from its own base** — two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.

## Dispatch as one unit with `fold-the-shape-environment-into-semantic-identity`

Not before the compiler and foundation scopes are free. [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md) stays separate: keeping `ValueFact` on `Shape` means only inputs can be symbolic until it lands, which is a coherent boundary rather than a partial state.

## Merged with the identity fold, 2026-08-07 — this ticket is now the combined unit

[`fold-the-shape-environment-into-semantic-identity`](fold-the-shape-environment-into-semantic-identity.md) is **closed `superseded` into this one**, and its dependent re-pointed here. The two were mutually dependent — `fold` declared a dependency on this ticket, and the 2026-08-07 measurement established this ticket cannot be delivered without `fold`'s encoding step. `tkt link` refused the reciprocal edge as a **cycle**, which is the mechanical proof that neither can go first.

They were left as two tickets after that finding, and the board went on offering this one as `ready` — so a worker could have claimed it and hit exactly the wall the last one measured. Merging removes the hazard rather than documenting it.

### What the merged ticket owes, from `fold`'s own keys

- **`tiler.semantic-graph.v2 → v3`**: a tagged extent encoding, replacing `encode_shape`'s eight untagged big-endian bytes per extent. Untagged-static beside tagged-symbolic is collision-ambiguous, which is why a hybrid is not an option.
- **The fifth `SemanticIdentity` subject**, folding `ShapeEnvIdentity` — without it, two programs spelled identically over differently bound environments share an identity, which is the unkeyed symbolic program this ticket forbids shipping.
- Every pinned identity recomputed. That obligation is `fold`'s and it **contradicts this ticket's original "must not change canonical bytes"** — the contradiction is resolved by landing them together, and the byte-stability requirement applies only to programs with no symbolic extent.

### The pinned population, already enumerated

79 literals across 16 `.rs` files, of which **exactly three move** on a semantic-graph encoding change — `ARTIFACT_IDENTITY`, `CACHE_SUBJECT` and `FIXED_CONTENT_BYTES` in `crates/tiler-build/src/metal_plan.rs` — because that fixture builds a `SemanticProgram` and the artifact preimage folds the graph identity, with the cache subject composing over it. `crates/tiler/src/route/tests.rs`'s `IDENTITY_DOMAIN` moves **additionally if the artifact domain steps**. The `index/law.rs` and `schedule/builder.rs` pins do **not** move, despite their fixtures building semantic programs.

**Recompute all of them on the merged tree, never from a branch base** — two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.

### Dispatch conditions

Not before `implementation/compiler`, `implementation/reference`, `implementation/artifact`, `implementation/frontend` and `contracts/foundation` are all free — that is a wide batch and it will need most of the board quiet. [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md) stays separate: keeping `ValueFact` on `Shape` means only inputs can be symbolic until it lands, which is a coherent boundary rather than a partial state.

## Audit 2026-08-07 — per-Fact verification at base `ad999a52`, tree left clean

A worker claimed the combined unit, audited every Fact against source at the base, measured the call-site population independently, and **committed no source change**. `git status` clean; the only mutation is this block. The measurement below reproduces one earlier claim exactly, corrects three, and finds one **missing scope** and one **false claim in the merge block**.

### Per-Fact audit

| Claim | Where | Verdict |
| --- | --- | --- |
| `ValueFact` holds a `Shape` | `semantic/operation.rs:997` | **verified** |
| `ValueDefinition` holds a `Shape` | original body, Fact 1 | **false** — already corrected by repair §2; it is `pub(super)` and holds no shape (`operation.rs:1564`) |
| `ValueData` (`:1593`), `ValueRef::shape` (`:1637`), `ValueFact::shape` (`:1018`) | repair §2 | **verified**, all three line numbers exact |
| `SemanticProgramBuilder::input`/`input_resolved` take `Shape` by value | `program.rs:493`, `:516` | **verified** |
| `SemanticProgram::shape -> Result<&Shape, HandleError>` | `program.rs:237` | **verified** |
| Contract admits symbolic semantic extents | `shape-environment-contract.md:89-91` | **verified** — the sentence is hard-wrapped; a single-line grep returns nothing, which is a false negative, not absence |
| `docs/ir.md` "will not complete the symbolic contract above" | `docs/ir.md:1158` | **verified** (the research record cites it as `:1111`; that line reference has drifted) |
| `SemanticIdentity` owns exactly four subjects; graph doc excludes providers/snapshots/compilation provenance | `semantic/identity.rs:40-45`, `:21-22` | **verified** |
| `encode_shape` writes rank then eight untagged big-endian bytes, domain `tiler.semantic-graph.v2\0` | `semantic/identity.rs:384-389`, `:17` | **verified** |
| 45 call sites across 5 crates — compiler 24, ir 11, reference 7, artifact 2, macros 1 | repair §1 | **verified exactly**, by the `#[deprecated]` method the repair block recommends |
| per-file compiler breakdown "`normalize.rs` ×8" | repair §1 | **imprecise** — 9 warnings at 7 distinct lines. The stated sub-counts sum to 23, not the 24 the same sentence claims |
| "the compiler sites are real source, not fixtures" | repair §1 | **imprecise** — broadly right, but `request.rs:4491` and `normalize.rs:1160` are `#[cfg(test)]` *items*, not module boundaries, so the claim needs per-site checking rather than the range reading it invites |

### The merge block's artifact claim is false

> "The artifact program subject and the expansion cache's `ComposedSubject` already carry the semantic subjects, so no cache facet, artifact section, or crate dependency is added."

**True for the cache** — `crates/tiler-cache` never names `SemanticIdentity`; `ComposedSubject` takes opaque facet bytes. **False for the artifact.** `project_semantic` (`tiler-artifact/src/program/codec/model.rs:1018-1033`) does not carry `SemanticIdentity`; it projects **three named typed subjects** into a wire struct. `docs/artifact-abi.md:414` states this deliberately: "Only the three reached subjects travel". A fourth would need a new subject newtype, manifest schema major step, `tiler.artifact-program.v15 → v16`, and an ABI-doc edit under **`contracts/artifacts` — a scope this ticket does not hold and which `w-decide-wh` currently claims.**

**Resolution that keeps the unit inside its scopes.** Make the fifth subject **total** over an empty-environment identity (this is the elimination `fold` asked for and left open; optional makes "declares no symbols" and "empty environment" two states for one fact). The subject is then constant across every artifact-reachable program, because the compiler refuses a symbolic program at `normalize.rs`'s rebuild, so no two artifacts can differ by it. The artifact keeps three subjects, `docs/artifact-abi.md:154` and `:414` stay true unedited, and the artifact domain does **not** step — only its values move, which is exactly the precedent `docs/artifact-abi.md:229` and `:231` already state for a nested domain. That coupling is load-bearing and needs its own test, not an assumption.

### Byte stability is not "only for static programs" — it is abandoned

The merge block says the byte-stability requirement "applies only to programs with no symbolic extent". That is **self-contradictory**: `SourcedExtent::encode` prepends a tag byte (`sourced.rs:222-228`) where `encode_shape` writes none, so a wholly static program's graph bytes move too. `fold`'s own Fact says so outright. If static bytes did not move, no domain step and no pin recompute would be needed — yet the same block demands both. The coherent reading is that byte stability is **dropped entirely**, which is why the domain steps; do not restate it as a surviving requirement.

### Missing scope: `implementation/build`

The three pins the ticket says must move — `ARTIFACT_IDENTITY`, `CACHE_SUBJECT`, `FIXED_CONTENT_BYTES` at `crates/tiler-build/src/metal_plan.rs:1493-1497` — are under **`implementation/build`**, which is **not** in this ticket's `scopes`. That is the same defect the 2026-08-07 repair found and fixed for five other crates, missed once more because the pins were enumerated without mapping them back through `ticketsplease.toml`. It is currently **unclaimed**, so it can simply be added. `crates/tiler/src/route/tests.rs` is `implementation/frontend` and is already held.

### The domain cascade, enumerated from source

A fifth subject forces a step at every site that enumerates the subject set into a versioned preimage:

- `tiler.semantic-graph.v2 → v3` — `semantic/identity.rs`
- `tiler.compiler.request-subject.v5 → v6` — `tiler-compiler/src/request.rs:2949-2960`
- `tiler.program-alternative.v1 → v2` — `tiler-compiler/src/pipeline.rs:285-303`
- index refinement receipt — `tiler-ir/src/index/refinement.rs:3746-3748`

Not affected, and checked rather than assumed: `capability.rs:1631` and `tiler-reference/src/identity.rs:49-51` encode a **different** three-subject authority type (`LoweringCapabilityAuthority` / `SemanticCapabilityAuthority`), not `SemanticIdentity`.

`crates/tiler-conformance` is safe to leave to its live branch: it reads only `.graph()` (`publication/proof.rs:893`), and its hex pins are `result_sha256` numerical-oracle values, not identity bytes — none move.

### Stale claims this change must repair, all in declared scopes

- `docs/ir.md:904` — "no semantic construction path names it" and an extent "reaches no semantic value at layer 1" both become false.
- `docs/ir.md:858` — the canonical-identity statement `fold` already named.
- `shape/sourced.rs:19-31` and `:92-96` — "the index layer is the only consumer at this commit" and "**This case has no implementation at this commit**" become false the moment the semantic layer carries a sourced shape.

### Why the tree was left clean rather than partially cut

Part A forces the v3 tagged encoding; v3 forces the fifth subject, or a symbolic program ships unkeyed; the fifth subject forces the cascade above. There is therefore **no smaller coherent unit than the whole** — every intermediate stopping point is a half-stepped identity domain, which `AGENTS.md` names as not a coherent boundary. The base state is the only other gated boundary, so that is what is preserved.

**Before the next dispatch:** add `implementation/build`; state the total-vs-optional elimination and the artifact-omission argument above as decided, since they change what the ticket delivers; and treat the pin list as a hypothesis — recompute every pin on the merged tree and report unmoved ones too.

## Coordinator corrections after the 2026-08-07 mapping run

The worker preserved the base state rather than half-stepping an identity domain, which is correct: Part A forces `v3`, `v3` forces the fifth subject or the program ships unkeyed, and the fifth subject forces a four-domain cascade. **There is no smaller coherent unit than the whole**, and `AGENTS.md` names a half-stepped identity domain as not a coherent boundary. No pin table or collision probe is reported because none was computed — reporting recomputed pins that were not recomputed is the exact failure this ticket exists to stop.

### Scope gap, fixed — and it was the coordinator's

`implementation/build` was missing. The three pins this ticket says must move live in `crates/tiler-build/src/metal_plan.rs`, which maps to **`implementation/build`** — coordinator-verified against `ticketsplease.toml`, and absent from the six declared scopes. The pins were enumerated without being mapped through the scope table, which is the *same defect* the 2026-08-07 repair fixed for five other crates. Now added; the set is **seven**.

### A false claim in the merge block, struck

The merge block stated the artifact program subject "already carries the semantic subjects, so no artifact section is added". **True for the cache** — `tiler-cache` never names `SemanticIdentity`. **False for the artifact**: `docs/artifact-abi.md` states deliberately that "**Only the three reached subjects travel**: the semantic graph identity, the reached definitions, and the admission provenance" — coordinator-verified. A fourth subject would need a manifest schema major step and an ABI-doc edit under **`contracts/artifacts`**, which this ticket does not hold.

Also struck: "byte-stability applies only to programs with no symbolic extent" is **self-contradictory** — tagging moves static bytes too, which is precisely why the domain steps. **Byte stability is abandoned, not narrowed.** Say that plainly rather than preserving a qualifier that cannot hold.

### The resolution that keeps the unit inside its scopes

Make the fifth subject **total over an empty-environment identity** — the elimination the superseded `fold` ticket explicitly left open. It is then constant across every artifact-reachable program, because the compiler refuses symbolic programs at `normalize.rs`'s rebuild, so **the artifact keeps three subjects and its domain does not step**.

That coupling is load-bearing and **needs a test, not an assumption**: the property "no symbolic program reaches the artifact" is what makes the fourth subject invisible there, and if it ever fails the artifact silently ships an unkeyed program. Pin it.

This is an internal design choice that *avoids* a public-boundary step rather than taking one, so it eliminates under `AGENTS.md` and is not reserved. If the test shows the coupling does not hold, that is a different ticket and Tom's.

### Smaller corrections to this ticket's own text

- **`normalize.rs ×8` is imprecise** — nine warnings at seven distinct lines, and the repair block's sub-counts sum to 23 while the same sentence says 24.
- **"compiler sites are real source, not fixtures" is imprecise** — two of them are `#[cfg(test)]` *items* rather than module boundaries, so the range reading they invite is unsound.
- The 45-site measurement **was reproduced exactly** by the `#[deprecated]` method (compiler 24, ir 11, reference 7, artifact 2, macros 1), and the probe reverted.
- Fact 2's quote is verified but **hard-wrapped** at its source, so a naive grep returns a false negative. The reproduce command should account for that.

### Release trigger for redispatch

All **seven** scopes free simultaneously. Recheck with `tkt claims` plus a scope scan of live `tkt/*` branches.

## Delivered 2026-08-07 — the combined unit landed whole

Both halves in one commit: the semantic layer carries a sourced shape, and `SemanticIdentity` gained its fifth subject. Every check below was run on this branch at its own HEAD with `CARGO_TARGET_DIR=./target`.

### Per-Fact audit at base `0132c0c3`, before any edit

| Claim | Where | Verdict |
| --- | --- | --- |
| `ValueFact` holds a `Shape`; `ValueFact::shape` | `semantic/operation.rs:997`, `:1018` | **verified** |
| `ValueDefinition` holds no shape, is `pub(super)` | `semantic/operation.rs:1564` | **verified** |
| `ValueData` (`:1593`) and `ValueRef::shape` (`:1637`) | repair §2 | **verified**, both exact |
| `input`/`input_resolved` take `Shape` by value | `semantic/program.rs:493`, `:516` | **verified** |
| `SemanticProgram::shape -> Result<&Shape, HandleError>` | `semantic/program.rs:237` | **verified** |
| Contract admits symbolic semantic extents | `shape-environment-contract.md:89-91` | **verified**; hard-wrapped, so a single-line grep is a false negative |
| `docs/ir.md` "will not complete the symbolic contract above" | `docs/ir.md:1158` | **verified** |
| `SemanticIdentity` owns four subjects; graph doc excludes providers/snapshots/provenance | `semantic/identity.rs:40-45`, `:21-22` | **verified** |
| `encode_shape` writes rank then eight untagged BE bytes; domain `tiler.semantic-graph.v2\0` | `semantic/identity.rs:384-389`, `:17` | **verified** |
| `SourcedExtent::encode` prepends a tag byte | `shape/sourced.rs:222-228` | **verified** |
| `project_semantic` projects three named subjects | `tiler-artifact/src/program/codec/model.rs:1018-1033` | **verified** |
| "Only the three reached subjects travel" | `docs/artifact-abi.md:414` | **verified** |
| 45 call sites: compiler 24, ir 11, reference 7, artifact 2, macros 1 | repair §1 | **verified exactly**, reproduced by the `#[deprecated]` method |
| `normalize.rs ×8` | repair §1 | **imprecise, as the coordinator said** — 9 warnings at 7 distinct lines |
| the three `metal_plan.rs` pins move | repair §4 | **verified**, and the pin list was incomplete — see the table below |
| **`docs/ir.md:858` is "the canonical-identity statement"** | coordinator block | **false** — `:858` is mid-paragraph in the *residual*-identity Fact at `:855`. The statement meant is at **`:869-870`**, and it is hard-wrapped across those two lines ("root-binding\nprovenance"), which is why a grep for the phrase returns nothing. Corrected at `:869-872`. |
| **"index refinement receipt" is a fourth cascade site** (`index/refinement.rs:3746-3748`) | coordinator block | **false, and self-contradicted by the same block.** Those lines encode `SemanticCapabilityAuthority` (`semantic/registry.rs:1902`), a *three*-subject type that is not `SemanticIdentity` — which the same block's own "Not affected, and checked rather than assumed" paragraph says two paragraphs earlier. `tiler.ir.index-realization-authority.v1` does not move. **The cascade is three domains, not four.** |
| **the artifact would need a fourth subject newtype and a `v15 → v16` step** | audit block §"artifact claim is false" | **imprecise, and the real reason is stronger.** The artifact already carries *three of four* subjects and deliberately omits the registry snapshot under ADR 0072, so omitting a fifth needs no new machinery at all. What it needs is the *soundness* argument, which is the coupling below. |

### The resolution taken, and the coupling pinned rather than assumed

The fifth subject is **total over the empty-environment identity** (`shape/env.rs::empty_environment_identity`, read from `ShapeEnvBuilder::new().build()` rather than written out). "Declares no symbols" and "has an empty environment" stay one fact with one spelling, and every downstream enumeration writes five unconditional subjects with no presence tag to frame. This deliberately differs from `crates/tiler-ir/src/index`, whose region bytes carry a presence tag: the difference is the position, not the rule — a fixed subject slot in an identity bundle must hold a value.

`I-A` (folding `ShapeEnvIdentity` into `SemanticGraphIdentity`, which would have avoided the whole cascade) was **not** re-litigated: [the symbolic-semantic-extents record](../docs/research/shapes/symbolic-semantic-extents.md) eliminates it at its Q2 table because `ShapeEnvIdentity` bundles root-binding provenance, which the accepted three-identity table puts on the interface side.

**No symbolic program reaches the artifact, and three independent refusals hold it there** — each stating its own reason rather than deferring to an upstream invariant:

- `tiler-compiler`'s `normalize.rs::static_shape` — a rebuilt draft is minted with no environment, so a rebuilt symbolic input would lose or silently change the environment its identity folds.
- `tiler-ir`'s `KernelProgramBuilder::new` → `KernelProgramBuildError::SymbolicInterfaceExtent` — a covered boundary is a sized quantity.
- `tiler-artifact`'s `ArtifactProgramBuilder::new` → `ArtifactBuildError::SymbolicSemanticInterface` — the envelope projects three subjects and can only omit the fifth while no two artifacts differ by it.

The last two are pinned by `no_symbolic_program_reaches_a_verified_kernel_program` (tiler-ir) and `a_symbolic_semantic_program_never_reaches_the_artifact_builder` (tiler-artifact), each with the accepted neighbour that differs only in the extent's source kind. **`contracts/artifacts` was not edited and `docs/artifact-abi.md:414` stays true unedited.**

### Byte stability

Abandoned, not narrowed, exactly as the coordinator said. `SourcedShape::encode` tags every extent, so a wholly static program's graph bytes move.

### The complete recomputed pin table, measured on this tree

Recomputed by running the suite, never carried from the ticket's hypothesis. The hypothesis was **incomplete by two**.

| Pin | Location | Before | After | In the ticket's list? |
| --- | --- | --- | --- | --- |
| `ARTIFACT_IDENTITY` | `tiler-build/src/metal_plan.rs` | `7a2bfe51…4357d` | `e16ce926…08057` | yes |
| `CACHE_SUBJECT` | `tiler-build/src/metal_plan.rs` | `8bdcde64…b1aa2` | `287df982…8104b` | yes |
| `FIXED_CONTENT_BYTES` | `tiler-build/src/metal_plan.rs` | 65,294 | 65,308 | yes |
| explain request qualifier | `tiler-compiler/src/explain.rs` | `f99d1e5eb387f42f` | `940c09e0821665a6` | **no** — moved by `request-subject.v6` |
| `DIFFERING_CARRIER_POSITIONS` | `tiler-artifact/src/program/codec/tests.rs` | 68 | 67 | **no** — see below |
| `IDENTITY_DOMAIN` | `crates/tiler/src/route/tests.rs` | — | **unmoved** | conditional; condition did not fire |
| `index/law.rs` pins | `tiler-ir` | — | **unmoved** | predicted unmoved — held |
| `schedule/builder.rs` pins | `tiler-ir` | — | **unmoved** | predicted unmoved — held |
| every other pinned literal | 16 `.rs` files | — | **unmoved** | — |

`IDENTITY_DOMAIN` not moving is *evidence for the artifact argument*, not luck: it moves only if the artifact domain steps, and the artifact domain did not step.

**`DIFFERING_CARRIER_POSITIONS` 68 → 67 is a chance coincidence, not a structural change.** The count is the two carrier/access tag pairs (4 bytes) plus the two 32-byte digests covering them, less whatever digest bytes coincide. Nothing structural moved; one more digest byte now happens to match. The constant's own doc comment already warned that this is measured rather than derived, which is why it is pinned.

**`FIXED_CONTENT_BYTES` +14 was checked rather than copied.** The fixture reaches 7 extents (input `[2,2]`, two rank-0 constants, two rank-2 results, one rank-1 sum) and the graph identity appears **twice** in the envelope — once as the artifact program's carried subject and once inside the nested kernel program's subject — so a one-byte-per-extent encoding step costs `2 × 7 = 14`. Measured directly with a temporary probe (`graph_len=886 occurrences=2 extents=7`), which was then removed; the arithmetic is now recorded at the test so the next mover can check a delta instead of copying one.

### The domain cascade, as landed

- `tiler.semantic-graph.v2 → v3` — `semantic/identity.rs`, tagged extents through `SourcedShape::encode`.
- `tiler.compiler.request-subject.v5 → v6` — `tiler-compiler/src/request.rs`, fifth `push_slice` before the output count.
- `tiler.program-alternative.v1 → v2` — `tiler-compiler/src/pipeline.rs`, fifth component in the fixed run.
- `tiler.shape-env.v3` — **does not move**, as `fold` required: no byte a shape environment encodes changed.
- `tiler.artifact-program.v15`, the envelope, and the manifest schema — **do not move**.
- `OBLIGATION_DOMAIN` (`semantic/precondition.rs`) — **does not move**, and this is a decision rather than an omission. A precondition subject is always an operand, and an operand is always literal, so nothing an obligation identity can encode changed; `static_subject_shape` is the site that says so and names the ticket that must step it.

### Public boundary, ADR 0075 — the exact draft surface

Added, all `tiler_ir`:

- `SemanticProgramBuilder::try_standard_with_shape_environment(Arc<ShapeEnv>)`
- `SemanticProgramBuilder::input_sourced<T>(InputKey, Vec<SourcedExtent>)`
- `SemanticProgramBuilder::input_resolved_sourced(InputKey, Vec<SourcedExtent>, ResolvedValueType)`
- `SemanticProgram::extent_sources() -> Option<&ExtentSources>`
- `SemanticIdentity::shape_environment() -> &ShapeEnvIdentity`
- `BuildError::{ExtentSource, ShapeVocabulary, SymbolicOperandUnsupported}`
- `ShapeRefineError::SymbolicShape`, `ShapeWitnessError::SymbolicShape`
- `KernelProgramBuildError::SymbolicInterfaceExtent`, `ArtifactBuildError::SymbolicSemanticInterface`, `IndexRefinementVerificationError::SymbolicSemanticBoundary`, `EvaluationError::SymbolicShape` (`tiler_reference`)
- `SourcedShape` gained `Eq`/`Hash`/`PartialEq`

Changed: `SemanticProgram::shape` and `ValueRef::shape` return `&SourcedShape`. `ValueFact::shape` is **unchanged** and still returns `&Shape` — the ticket's own dispatch note requires it, and the `#[deprecated]` measurement confirms the 45-site population excludes it entirely (`ValueFact::shape` has 66 further sites, none of which move).

Deliberately **not** added: a `try_new_with_shape_environment` for a custom registry, because the ticket names only the standard-registry constructor. A symbolic program over a custom registry is therefore not constructible; that asymmetry is stated rather than hidden.

Not accepted. **This is a labelled draft** until Tom accepts its exact included and excluded surface.

### Evidence, and every new check watched failing

14 new tests. Each was perturbed at its *subject* and observed failing, then restored:

| Perturbation | Tests that failed |
| --- | --- |
| fifth subject made constant (drop the environment fold) | `two_environments_over_one_spelling…`, `a_program_that_declares_no_symbol…` |
| encode a symbol's determined value instead of the symbol | `two_symbols_in_one_environment_are_two_programs` |
| drop `SourcedExtent`'s source tag | the three `metal_plan.rs` pins |
| `ArtifactProgramBuilder` accepts a symbolic interface | `a_symbolic_semantic_program_never_reaches_the_artifact_builder` |
| `KernelProgramBuilder` accepts a symbolic interface | `no_symbolic_program_reaches_a_verified_kernel_program` |
| drop the `ExtentSources::admit` check | `a_foreign_symbol_is_refused…`, `a_post_ceiling_binding_is_refused…` |
| let a symbolic operand through as rank-1 | `a_symbolic_value_is_refused_as_an_operation_operand` |
| `refine` accepts a symbolic value | `rust_side_evidence_and_shape_witnesses_refuse_a_symbolic_value` |
| drop `input_sourced`'s environment | six of the seven symbolic tests |
| break the all-literal normalization in `SourcedShape::sourced` | `a_wholly_literal_program_stays_static_through_every_construction_path` |

**The collision probe was strengthened because the first one did not bite.** `a_symbol_and_the_value_its_environment_pins_are_two_programs` survived the "encode the determined value" perturbation, because the two programs are still separated by `SourcedExtent`'s *tag* alone. `two_symbols_in_one_environment_are_two_programs` is the probe that closes it: both programs carry the symbol tag and **one** environment, so their environment subjects are equal by construction and only the symbol's own bytes can separate them. `a_symbolic_axis_and_a_literal_one_do_not_collide_across_ranks` covers the framing direction.

### Commands, all exit 0 on this tree

```
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --locked --exclude tiler-prototype-run --exclude tiler-prototype-compile --exclude tiler-prototype-candle -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo nextest run --workspace --locked      # 3163 passed, 8 skipped (3149 at base)
cargo test --workspace --doc --locked
tkt lint                                     # ok: no problems found
make citations                               # 923 citations across 491 files resolve
git diff --check
```

A bare `cargo clippy --workspace` fails on four pre-existing findings in `prototypes/serial-sum-run/src/proof.rs`, untouched by this change; the `Makefile`'s `lint` target excludes the prototypes deliberately and is what was run.

### What the next semantic-graph identity step must account for

[`remove-the-workload-shapes-from-the-concatenate-normative-definition`](remove-the-workload-shapes-from-the-concatenate-normative-definition.md) also steps this domain and is not dispatched. Its coordination note says to check whether this ticket landed first; it has. So that ticket steps **`tiler.semantic-graph.v3 → v4`**, not `v2 → v3`, and it must recompute the **five** pins in the table above rather than the three the older text names — the explain request qualifier and `DIFFERING_CARRIER_POSITIONS` are the two the hypothesis missed, and it must hold `implementation/compiler`, `implementation/artifact`, and `implementation/build` to move them. It does **not** need to touch `request-subject` or `program-alternative`: those stepped for the *subject set*, which it does not change, and a value move inside them stays injective.

### Documents this landing falsifies that are outside these seven scopes

Filed as [`repair-the-records-the-sourced-semantic-shape-falsifies`](repair-the-records-the-sourced-semantic-shape-falsifies.md). `docs/ir.md` was repaired here (`:377` four → five subjects with the accessor listed, `:869-872` the environment identity named as the fifth subject, `:904` the Fact narrowed with a dated correction). Nothing outside `contracts/foundation` was edited.

### Scope confirmation

31 files, all inside `implementation/{ir,compiler,reference,artifact,frontend,build}`, `contracts/foundation`, and `project/tickets`. `crates/tiler-conformance/**` untouched — it reads only `.graph()` and its hex pins are `result_sha256` oracle values; the whole crate's tests pass unchanged. `contracts/artifacts` and `contracts/navigation` untouched.
