---
schema: "tiler-doc/v1"
id: "tiler.research.shapes.symbolic-semantic-extents"
kind: "research"
title: "Symbolic extents in the semantic program"
topics: ["shapes", "extents", "semantic-graph", "identity", "frontend", "specialization", "inline-dx"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir"]
depends_on: ["tiler.research.shapes.shape-environment-contract", "tiler.research.indexing.index-access-model", "tiler.research.runtime.autoregressive-state-and-kv-cache"]
ticket: "carry-symbolic-extents-into-the-semantic-program"
---

# Symbolic extents in the semantic program

**Status:** durable design record for the gap between an inline region's `sym n` and the public logical program it denotes. It is a research outcome, not a capability: nothing here widens a type, registers an operation, moves an identity domain, or authorizes implementation. What it delivers is three eliminations, the identity-composition analysis, seven atomic decisions a public-boundary acceptance must make, and seven dependency-ordered delivery tickets.

## Traceability

- **Work record:** [`carry-symbolic-extents-into-the-semantic-program`](../../../tickets/carry-symbolic-extents-into-the-semantic-program.md).
- **Why now.** [`prototype-inline-proc-macro-frontend`](../../../tickets/prototype-inline-proc-macro-frontend.md) delivered the region syntax Tom approved on 2026-07-30 and found that its central feature cannot reach the compiler; [`prototype-inline-aot-integration-proof`](../../../tickets/prototype-inline-aot-integration-proof.md) then delivered the whole expansion-time AOT flow — parse, verify, optimize, emit Metal, cache, compile, embed, decode, route — for a *literal* region, and its boundary packet item 4 records the consequence in the negative: a region with a symbolic extent cannot state a selected artifact family, and the refusal names this ticket by id. The gap is therefore no longer a missing feature at the end of a chain; it is the one thing standing between the approved syntax and a flow that already works.
- **Governing authorities read as evidence, not edited:** [the shape environment contract](shape-environment-contract.md) for the fixed-rank decision, typed root bindings, pre-dispatch host evaluability, the three distinguishable identities, and the specialization boundary; [`docs/ir.md`](../../ir.md) for the constraint and proof context and the reserved symbolic profile; [the sequence-extending family record](sequence-extending-tensor-family.md) and [the L5 state record](../runtime/autoregressive-state-and-kv-cache.md) for the workload evidence and the specialization refusals; [the L4 attention vertical](../program-planning/first-attention-program-vertical.md) for the binding-versus-graph-change principle; [`promote-the-symbolic-index-profile-to-a-public-boundary`](../../../tickets/promote-the-symbolic-index-profile-to-a-public-boundary.md) for the surface Tom accepted on 2026-07-31, one day before this record.
- **Inspected source, at this branch's base commit `bc39282`:** `crates/tiler-ir/src/shape.rs`, `crates/tiler-ir/src/shape/env.rs`, `crates/tiler-ir/src/shape/env/constraint.rs`, `crates/tiler-ir/src/index/sourced.rs`, `crates/tiler-ir/src/index/model.rs`, `crates/tiler-ir/src/semantic/{identity,interface,program,operation}.rs`, `crates/tiler-macros/src/{region,binding,aot}.rs`, `crates/tiler-compiler/src/request.rs`.

Claims are labelled **Fact** when traced to inspected source at that commit or to a merged record, **Inference** when derived from stated facts, and **Proposal** when not yet accepted or tested. **This record contains no measurements and takes none.** Every byte and count figure is arithmetic over quantities L1, L5, and the AOT proof already state.

## What is actually missing

**Fact — the semantic layer has no symbol in it at all.** `crates/tiler-ir/src/shape.rs:1` calls itself "Target-independent **fixed** shape vocabulary", `Extent` wraps a `u64`, `Shape` is a `Vec<Extent>`, and `SemanticProgramBuilder::input` and `input_resolved` both take a `Shape` by value. The exact check is `grep -rn "ShapeSymbol\|SourcedExtent\|SourcedShape\|ShapeEnv" crates/tiler-ir/src/semantic.rs crates/tiler-ir/src/semantic/`, which returns two doc-comment hits and no code; the positive control is the same pattern over `crates/tiler-ir/src/index/`, which returns ten files.

**Corrected 2026-08-07 — the check above read "returns nothing", and it was already false when this record was written rather than falsified since.** At this record's own base commit `bc39282` the same grep returned `crates/tiler-ir/src/semantic/slice.rs` and `crates/tiler-ir/src/semantic/softmax/tests.rs`, and it still returns exactly those two and nothing else. **The Fact's own claim survives, and only the check offered to establish it was wrong.** Both hits are *prose about the index layer's vocabulary* sited inside a semantic module — the slice family's `SymbolicOffsetUnsupported` doc comment saying a bound extent symbol in a coordinate position is not expressible, and a softmax test's doc comment naming the type a symbolic extent would have — so neither is a symbol reaching a semantic value, and `SemanticProgramBuilder::input` still takes a `Shape`. The narrower check that does return nothing is the same pattern with doc-comment lines excluded, and it is what the *Reproducible checks* section now carries. This repair is independent of the 2026-08-07 relocation and is recorded separately from it for that reason. The positive control's count moved for a third, unrelated reason: six files at `bc39282` and ten at `cd86cac1`, because [`admit-symbolic-index-expression-coefficients`](../../../tickets/admit-symbolic-index-expression-coefficients.md) widened the index layer, not because anything left the semantic one.

**Fact — the environment does exist at expansion time, and only the model is deferred.** `crates/tiler-macros/src/binding.rs:466` constructs a real `ShapeEnvBuilder`, declares each `sym` as a `ShapeSymbol` in the fixed scope `tiler.inline-region.v1`, binds it to `BindingSource::InputDimension { key, axis }` at `AvailabilityPhase::LiveDevicePreflight` with `FactProvenance::RuntimeValidated`, and exposes the result's `ShapeEnvIdentity`. So an expansion already holds a verified `ShapeEnv`; what it does not hold is a *value* for `n`.

**Inference — the gap is therefore a vocabulary gap and not an availability gap.** The frontend is not waiting for information. It has the declarations, the bindings, the provenance, and the identity, and it is refused because the type it must hand `SemanticProgramBuilder::input` cannot carry a symbol. `crates/tiler-macros/src/region.rs:568` records that as `ProgramEvidence::DeferredSymbolicExtent` and refuses to substitute a representative extent, on the ground that a program built over invented extents "would be a different program, and its identity would name something no consumer wrote".

**Fact — the accepted contract already decided that semantic extents may be symbolic.** The shape environment contract's fixed-rank decision reads: "Every tensor value in a `SemanticTensorGraph` submitted to Tiler's optimizer has statically known rank. Each axis extent may be a static integer or a scoped symbolic expression evaluated later." `docs/ir.md:412` describes a value's authority as "the value's authoritative ranked shape-expression vector and `ShapeEnv`", and `docs/ir.md:816` states that `ShapeExpr` "is the one this contract names at every layer that computes an extent". **Inference — so the fixed `Shape` is the bounded implementation profile, not the contract**, which `docs/ir.md:1111` says outright: "Completing this bounded static-extent profile will not complete the symbolic contract above." What this record decides is the spelling, not whether.

## Q1 — how a symbolic semantic shape is spelled

Four candidates. The fourth is not in the ticket; it is forced by the one-vocabulary key the index promotion landed, and eliminating the ticket's three without naming it would have left the survivor set empty.

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **W1** — widen `Shape`/`Extent` so an extent may be a symbol | **No** | Four independent grounds, below. |
| **W2** — a distinct sourced shape defined at the semantic layer, mirroring `SourcedShape` | **No** | It is the second constant-or-symbol enum that `SourcedExtent`'s own accepted documentation exists to prevent. |
| **W3** — specialize before build: fix extents from a caller-supplied environment, then construct | **No** | Three independent grounds, and two of them are the ticket's own Do-nots reached by construction rather than by choice. |
| **W4** — relocate the promoted `SourcedExtent`/`SourcedShape`/`ExtentSources` vocabulary from `tiler_ir::index` to `tiler_ir::shape` and have the semantic layer consume the one vocabulary | **Yes** | The only candidate that adds no second authority, no second encoder, and no invented value. |

**W1 fails on four grounds, any one of which is sufficient.**

1. **It breaks an accepted boundary that depends on `Shape` meaning wholly-literal.** `SourcedShape::Static` holds a `Shape`, and its documented normalization invariant is that "Construction collapses an all-literal extent vector into `Self::Static`, so `Self::Sourced` holds at least one symbol and a boundary has exactly one spelling." If a `Shape` may itself hold a symbol, that invariant is not merely violated — it becomes unstatable, and `SourcedShape::as_static` stops being "a fact about the boundary rather than about which constructor authored it". Tom accepted that surface on 2026-07-31.
2. **It reintroduces the exact optional-accessor defect the index promotion removed.** `Shape::element_count` returns `Option<usize>` today and `None` means "not representable on this host". Under W1 `None` would additionally mean "symbolic", and a caller reading `None` as an overflow would be silently wrong. The promotion ticket names this failure shape directly: a rule about a pair of accessors "was unenforceable: a third source kind would have made both `None` and every consumer reading 'not static, therefore symbolic' would have been silently wrong with no test failing anywhere near it."
3. **It has no answer for exact static evidence.** ADR 0067's `StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>` refines a value against exact `u64` extents. A symbol has no `u64`, so refinement needs a *second* answer for the symbolic case whatever W1 does to `Shape` — which is the total view, arrived at by a longer route.
4. **It converts every static call site rather than leaving it alone.** `Shape::new`'s const rank assertion, `from_dims`, `without_axes`, and a `Display` that writes bare integers each either change meaning or acquire a case they cannot answer. The index precedent rejected the analogous churn in as many words: "Do not add `Option<ShapeEnv>` to every static call site."

**W2 fails on one ground, and it is the accepted key from the promotion that landed the day before this record.** `SourcedExtent`'s documentation states that it is "the crate's *one* constant-or-symbol vocabulary" and gives the reason: "a second divisor enum would give a frontend two ways to spell the same fact, two encodings to fold into identity, and two places to extend when a third source kind arrives." A semantic-layer mirror is precisely that second enum, one layer up, and it is worse than the divisor case it was written about, because both copies would fold `ShapeEnvIdentity` into identity with nothing checking that the two encoders agree. **Inference — the doc's scoping phrase "index-layer magnitude" is what W4 corrects, not what licenses W2**: the type is `Extent | ShapeSymbol` and both of those live in `crate::shape`; nothing in it is index-specific.

**W3 fails on three grounds.**

1. **The values do not exist at the only moment W3 could run.** A region's `sym n` is bound by `InputDimension` at `LiveDevicePreflight`, which is run time. Specializing at expansion needs a value, and there are exactly three places to get one: invent it (the ticket's first Do-not), reuse another specialization's artifact (the second), or construct the program in generated runtime code (the third, and the runtime source JIT the accepted inline developer experience forbids outright). **Inference — W3 is not a candidate that loses a trade-off; every route to it terminates in a Do-not.**
2. **It contradicts an accepted decision.** The shape environment contract's specialization boundary reads: "runtime extents remain symbolic in the logical plan by default. Specializing an extent to a concrete value is a physical-planning decision", and it names the cost of the opposite: specializing in the logical plan "discards generality, splits logical identity by shape, and risks compile-time and artifact proliferation."
3. **It collapses two identities the contract requires to stay distinguishable** — graph identity `histogram(input, W)` and specialized identity `W = 32`. **Fact — the index layer already refuses that collapse and has an executable statement of it.** `SourcedExtent::encode` encodes a symbol rather than a resolved value "because the accepted contract keeps `graph identity`, `interface identity`, and `specialized identity` distinguishable, and folding a bound value in here would collapse the first into the last", and the test at `crates/tiler-ir/src/index/sourced.rs:1598` asserts that a boundary sized by a symbol is a different program from one sized by that symbol's value *even when the environment pins the symbol to that value*.

**Inference — W3 is not eliminated for a frontend that genuinely holds its extents when it builds.** That frontend writes literals, which is the fully-literal subset that already compiles, delivers, and embeds today. W3 fails as a mechanism for the symbolic case, which is the case this ticket exists for.

**W4 survives, and what it costs is stated rather than hidden.** The vocabulary moves module, so `tiler_ir::index`'s accepted re-export paths change or gain aliases; every semantic consumer of `&Shape` migrates to the total view; and the semantic-graph identity domain moves because a static extent's bytes change. Those are real and they are enumerated as decisions A1, A3, and A5 below rather than absorbed.

*Updated 2026-08-07:* the first of those three costs is now paid, and it was paid the harder of the two ways this sentence offered — the five items are `tiler_ir::shape`'s and `tiler_ir::index` gained no alias, so the call sites moved rather than a re-export absorbing them. The two costs the move might have carried with it did not follow: the relocation ticket reports canonical bytes compared against its base over a static and a symbolic region and found byte-identical, and `INDEX_REGION_DOMAIN` is still `tiler.index-region.v11` at `crates/tiler-ir/src/index/builder.rs:100`. A3's and A5's costs are unpaid and remain theirs.

**Proposal — the shape of W4 in one block, using the vocabulary that already exists.**

```text
tiler_ir::shape::SourcedExtent  = Static(Extent) | Symbol(ShapeSymbol)     // relocated, unchanged
tiler_ir::shape::SourcedShape   = Static(Shape)  | Sourced(Vec<SourcedExtent>)

SemanticProgramBuilder::try_standard_with_shape_environment(Arc<ShapeEnv>)
SemanticProgramBuilder::input_sourced::<T>(InputKey, Vec<SourcedExtent>) -> Result<Value<T>, BuildError>
SemanticProgram::shape(ValueId) -> Result<&SourcedShape, HandleError>      // was &Shape
SemanticProgram::extent_sources() -> Option<&ExtentSources>                // as VerifiedIndexRegion has
```

**Inference — inference over symbolic extents is a behavioural addition and not a free consequence.** The elementwise rule the registry enforces, quoted verbatim by `crates/tiler-macros/src/region.rs:249`, is "operand shapes must match or one operand must be scalar". For two symbolic extents, "match" is `ExtentSources::proves_equal`, which is one-sided: `true` is a proof and `false` means *not proved*, never *proved different*. So the refusal stays a refusal and nothing is approximated — but the registry must be given the environment to ask, and a not-proved pair needs a typed reason distinct from the existing shape mismatch. That is decision A6 and delivery ticket 3.

## Q2 — where `ShapeEnvIdentity` enters identity

**Fact — the four subjects and their frame.** `SemanticIdentity` owns exactly four separately typed subjects — `graph`, `reached_definitions`, `admission_provenance`, `registry_snapshot` — with private fields and no public constructor, so that "downstream code" cannot assemble "components from different programs". `SemanticGraphIdentity`'s own documentation reads: "This identifies graph meaning. Provider implementations, registry snapshots, and compilation provenance are deliberately excluded."

**Fact — the current shape encoding is untagged, so it cannot absorb a symbol.** `encode_shape` in `crates/tiler-ir/src/semantic/identity.rs:377` pushes the rank and then eight raw big-endian bytes per extent, with no discriminator; the domain is `tiler.semantic-graph.v2\0`. The positive control is `SourcedExtent::encode`, which pushes `self.tag()` first and is the reason the index-region domain moved from `v8` to `v9` — "a *constant* divisor's bytes changed even though its meaning did not."

**Fact — `ShapeEnv` reaches neither the artifact crate nor the cache crate.** The exact check is `grep -rn "ShapeEnv" crates/tiler-artifact/src crates/tiler-cache/src`, which returns nothing; the positive control is `grep -rl "ShapeEnv" crates/ --include="*.rs" | cut -d/ -f2 | sort -u`, which returns `tiler`, `tiler-compiler`, `tiler-ir`, `tiler-macros`, `tiler-reference`. In `tiler-compiler` the only occurrence is `StaticShapeEnvironment`, a struct whose single field is a `schema_version: u32` and which `verify_request` compares against `governed()` — a version gate that reserves the seam and carries no symbol.

**Inference — the environment must enter identity exactly once, at the semantic layer, and the downstream composition then needs no change at all.** The artifact program subject is `CanonicalArtifactProgramIdentity`, and `encode_identity(envelope: &ArtifactEnvelope)` is the whole signature; the expansion cache subject is `ComposedSubject` over exactly two facets, `ArtifactProgram` and `BackendCompilations`. Because the semantic subjects already travel inside the artifact-program facet, a `ShapeEnvIdentity` folded into `SemanticIdentity` reaches the cache key with no new facet, no new dependency, and no crate learning what a shape environment is. **Inference — a third cache facet would be the second-authority failure `compose-the-complete-expansion-cache-subject` eliminated**, and it would require `tiler-cache` to parse an encoding it deliberately does not own.

**Inference — the ticket's requirement that "two regions declaring one interface remain one subject" is already satisfied at the frontend, by construction rather than by this decision.** `REGION_SCOPE` is the constant `b"tiler.inline-region.v1"` and its documentation states why: "the scope participates in `ShapeEnvIdentity`, so a unique scope per expansion would give two textually identical regions two identities and defeat the expansion cache that identity exists to key." The canonical binding source is the least occurrence by interface key then axis rather than the first written, and `declaration_order_does_not_change_the_environment` asserts it. So the risk this half of Q2 guards against is closed upstream; what remains is where the identity lands.

**Two candidates for where.**

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **I-A** — fold `ShapeEnvIdentity` into `SemanticGraphIdentity` | **No** | It puts interface provenance into a subject documented to identify graph meaning. |
| **I-B** — a fifth subject on `SemanticIdentity` | **Yes** | It is the only candidate that keeps the contract's graph/interface split representable. |

**I-A fails on the contract's own table.** `ShapeEnvIdentity` bundles three things — `docs/ir.md:858` states that canonical identity "includes symbol declarations, root-binding provenance, and semantic constraints but excludes derived solver caches". Declarations and constraints are graph meaning, and the index layer proves it: `region_identity_names_the_environment_its_symbols_resolve_in` shows that adding a divisibility constraint makes a different region. **Root-binding provenance is not.** The contract's three-identity table puts it on the interface side outright: graph identity is `histogram(input, W)`, interface identity is `W <- TargetProperty(SubgroupWidth)`. Under I-A, two programs of identical meaning that source `n` from input `a` rather than input `b` would report different *graph* identity — a subject whose documentation says it excludes exactly that class of fact.

**I-B survives, and its residue is named rather than papered over.** A fifth subject makes graph meaning and environment separable; it does not separate the environment's own three parts, because `ShapeEnvIdentity` is one opaque identity over all three. Splitting it would reopen `tiler.shape-env.v3`, which the index promotion deliberately did *not* move — "a domain that advanced for a visibility change alone would make two identical subjects carry different domains" — and which Tom accepted on 2026-07-31. **Deferred with a trigger:** the first consumer that must compare two programs of one graph meaning under different binding provenance, which is the artifact/interface identity question the shape environment contract already reserves ("included in an artifact/interface identity whenever the ABI exposes them").

**Inference — the semantic-graph domain must move to `v3` and the shape-env domain must not.** A tagged extent encoding changes a wholly static program's bytes even though its meaning is unchanged, which is exactly the `v8`-to-`v9` precedent; no byte a shape environment encodes changes, so `tiler.shape-env.v3` stays, by the same precedent's second half.

## Q3 — what a frontend does when an extent is unknown until dispatch

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **S-A** — specialize per observed extent and cache the result | **No** | Four independent grounds; for the inline frontend the first is structural. |
| **S-B** — carry the symbol through to a guarded plan | **Yes** | It is the accepted direction, and L5 supplies the first workload where guards and routing are both genuinely required. |

**S-A fails on four grounds.**

1. **For the inline frontend there is no moment at which it could run.** An observed extent exists only in the running consumer, so "specialize per observed extent and cache" means compiling on first sight of a value inside that process. The accepted inline developer experience forbids runtime source JIT outright, and not as a policy that could be traded: each invocation is a self-contained AOT and embedding unit, so there is no compiler in the consumer's target graph to invoke. **Fact —** `crates/tiler-macros` gained `tiler-build`, `tiler-cache`, and `tiler-compiler` as *host* dependencies that are never in a consumer's target graph.
2. **It multiplies artifacts by observed extents, and the multiplier is a workload quantity rather than a hypothetical.** At L5's C1 row the nine executions run `S ∈ {10, …, 18}`, so nine artifacts and nine payloads where one suffices. At B1-d the count is one per decode step and L5 states it directly for the analogous pipeline case — 129 — while the number of *distinct* `S` values across a whole B1-d run is not derivable from any figure this corpus states, because no record fixes that row's prefill length; it is bounded above by the row's final context of 8,320 and is not asserted here. The embedding budget bounds it from the other side: the measured `deliver macos;` proof embedded one 49,432-byte bundle into a 3,364,808-byte binary.
3. **Fresh L5 evidence makes it a refusal rather than a cost.** The runtime execution contract keys a prepared pipeline on, among other things, the specialization values, so "a build that specialized a kernel on `S` would mint a distinct pipeline for every decode step" — nine cold pipeline creations at C1, 129 at B1-d — "and the cache key would then literally track a mutable inference quantity". L5 states the consequence as an owed refusal: "A packaged program that specializes a kernel on `S`, `C`, or any cursor-derived quantity is refused at artifact assembly."
4. **It contradicts the accepted specialization boundary**, which places specializing an extent in physical planning and keeps the logical plan symbolic by default.

**S-B survives and requires nothing new of the frontend.** The frontend declares the symbol, its binding source, and any constraints it means; specialization is downstream, packaged, and chosen per execution. L5 shows the mechanism concretely: two packaged variants over `S`, the tiled plan guarded on `S ≡ 0 (mod 16)` and the direct plan otherwise, selected under `RoutingPolicy::StablePriority` at each step, with the tiled guard holding at exactly one of C1's nine executions — "the same discrimination is available as a packaged guard over a bound fact, at no cache cost."

**Fact — and carrying the symbol through does not require the frontend to bound it, for the region shape this ticket is about.** The natural worry is that an unbounded symbol yields an unprovable region. It does not, when the domain and the axis are the *same* symbol: `a_wholly_undetermined_dynamic_copy_verifies_by_proved_extent_equality` builds a `[n] -> [n]` copy over an environment stating only `m == n`, with no interval anywhere, and the region verifies with `BoundsProofView::ProvedExtentEquality` on every access and `WriteOwnershipProofView::CoordinatePermutation` on the write — "neither interval propagation nor an enumeration closed this; the equality did". `ShapeEnv::proves_equal` is reflexive by construction, since `same_class` is `self.classes.find(left) == self.classes.find(right)`.

**Inference — so the approved region needs no new syntax, and the bound is owed only where a symbolic extent must be related to a *different* extent.** `sym n; in a, b, c: f32[n]; out (a * b) + c` is exactly the same-symbol case. **Fact — the inline frontend states no constraint at all today:** the exact check is `grep -n "\.require(\|\.guard(\|SemanticInputConstraint\|ExtentRelation" crates/tiler-macros/src/binding.rs`, which returns nothing, while `grep -n "ShapeEnvBuilder::new" crates/tiler-macros/src/binding.rs` returns line 466 — the environment is built, and it declares and binds without constraining. **Inference — that is correct for this profile and insufficient for L5's**, where `S`, `C`, and `T` are three symbols that must be related; closing that is [`admit-an-additive-extent-relation`](../../../tickets/admit-an-additive-extent-relation.md), already filed, and this record composes with it rather than duplicating it.

## Worked example: the approved region against C1-like extents

The region Tom approved, over the nine extents L5's C1 row exercises.

```rust
let d = tiler::tensor! { sym n; in a: f32[n], b: f32[n], c: f32[n]; out (a * b) + c };
```

**Fact — what the expansion holds today.** One `ShapeEnv`: scope `tiler.inline-region.v1`, symbol `n`, one `RootBinding` over `BindingSource::InputDimension { key: "a", axis: 0 }` at `LiveDevicePreflight` with `RuntimeValidated` provenance — `a` because the canonical source is the least occurrence by interface key then axis — and two runtime equality obligations, for `b` axis 0 and `c` axis 0, carried beside the environment because ADR 0008 gives each symbol exactly one root binding. No constraints. `ProgramEvidence::DeferredSymbolicExtent`, and a `deliver macos;` on this region is a spanned `compile_error!` naming this ticket.

**Proposal — what W4 makes it.** Three inputs of `SourcedShape::Sourced([Symbol(n)])`, one `multiply-f32`, one `add-f32`, one output `out` of `SourcedShape::Sourced([Symbol(n)])`; the registry's elementwise rule discharged by `proves_equal(n, n)`; one `SemanticGraphIdentity` and one `ShapeEnvIdentity`.

| `n` | Artifacts under S-A | Artifacts under S-B | Graph identities | `ShapeEnv` identities | Variant guard `n ≡ 0 (mod 4)` |
| --- | --- | --- | --- | --- | --- |
| 10 … 18 (nine values) | 9 | **1** | 9 vs **1** | 9 vs **1** | holds at 12 and 16 — two of nine |

**Inference — the two columns differ by the whole point of the ticket.** Under S-A the nine executions are nine programs, nine identities, nine cache entries, and nine embedded bundles for one computation the consumer wrote once; under S-B they are one program whose `n` is an ABI-bound fact and whose plan selection is a guard evaluation. The guard column is the honest counterweight: two of the nine steps would route to a `mod 4` variant if one were packaged, which is a *physical* portfolio decision this record does not take and which L5 already demonstrates for `S`.

**Inference — and what still refuses, refuses for the same reason it does now.** `in a: f32[n], b: f32[m]` is not one shape: nothing forces `n` and `m` into one class, `proves_equal` returns `false` meaning not-proved, and the operation is refused rather than deferred into a wrong result. `crates/tiler-macros/src/region.rs:490` already states that rule for the expansion-time case — "nothing at expansion time proves `n` and `m` take one value, and treating them as compatible would defer a shape error into a wrong result" — and under W4 it becomes the registry's own answer rather than the frontend's restatement of it.

## The atomic decisions a public-boundary acceptance must make

Each is one decision, stated so it can be accepted or refused on its own. None is self-accepted here.

- **A1 — relocation.** Move `SourcedExtent`, `SourcedShape`, `ExtentSources`, `ExtentSourceError`, and `EXTENT_PHASE_CEILING` from `tiler_ir::index` to `tiler_ir::shape`, and decide whether `tiler_ir::index` keeps its current re-export paths. This re-opens a surface accepted on 2026-07-31; the ground for re-opening is the one-vocabulary key that same acceptance rests on.

  **Corrected 2026-08-07 — this decision named a six-item set and one of the six does not move.** The argument matters more than the count, because a list without it invites the next reader to "finish" the move. **`SymbolicExtentError` stays in `index`.** It is `Source(ExtentSourceError) | Structural(IndexBuildError) | ShapeVocabulary(ShapeError)`, so siting it at the shape layer would make the crate's *base* vocabulary name `crate::index::IndexBuildError` — inverting the exact layering this decision exists to establish — and it would deliver no sharing anyway, since a second consumer refusing a sourced extent puts *its own* build error in the structural slot and therefore needs its own union. Only `ExtentSourceError` is the shared authority, and it is among the five. **`SourcedIndexInteger` stays for the same reason and this record could not have named it:** it is `IndexInteger | ShapeSymbol`, so relocating it inverts the layering identically, and it did not exist when this record was written — it arrived afterwards with the symbolic coefficient.

  **Landed 2026-08-07 under [`relocate-the-sourced-extent-vocabulary-to-the-shape-module`](../../../tickets/relocate-the-sourced-extent-vocabulary-to-the-shape-module.md).** The five moved and the sub-decision was answered against aliases: `tiler_ir::index` kept **no** compatibility re-export, so the old paths do not resolve and callers name one canonical spelling. **The paths are not accepted.** Under ADR 0075 a changed public path is a public boundary exactly as a changed signature is, and [`accept-the-sourced-extent-vocabulary-at-its-shape-module-paths`](../../../tickets/accept-the-sourced-extent-vocabulary-at-its-shape-module-paths.md) is parked for Tom; the spelling below is current, not settled.
- **A2 — builder attachment.** `SemanticProgramBuilder::try_standard_with_shape_environment(Arc<ShapeEnv>)` beside `try_standard()`, with no setter, mirroring `IndexRegionBuilder::new_with_shape_environment` and the reason that constructor won: an environment fixed at construction is a property of the type rather than of a doc comment.
- **A3 — the total view.** `SemanticProgram::shape` returns `&SourcedShape` rather than `&Shape`, with `SourcedShape::as_static` the route to a fixed shape. This is the index promotion's "one total view" key applied at the semantic layer, and it changes every existing caller, including the frontend's inferred-versus-derived check.
- **A4 — the fifth subject.** `SemanticIdentity` gains a shape-environment subject. Sub-decision: whether it is optional, which makes "no symbols" and "an empty environment" two distinguishable states, or total over an empty-environment identity, which makes them one.
- **A5 — the identity domain.** `tiler.semantic-graph.v2` advances to `v3` and the extent encoding becomes tagged, so a wholly static program's canonical bytes change while its meaning does not. `tiler.shape-env.v3` does not move.
- **A6 — the inference contract.** Whether the registry's "operand shapes must match" resolves through `ExtentSources::proves_equal` for symbolic extents, and the typed `BuildError` a not-proved pair returns, distinct from the existing shape-mismatch refusal and naming both extents and the environment.
- **A7 — the frontend's delivery gate.** Whether `deliver` becomes available to a symbolic region once A1–A6 land, or stays refused behind a further condition. This record's finding is that the approved region needs no interval syntax to verify, so no grammar change is implied; the decision is whether that finding is sufficient to lift the refusal at `crates/tiler-macros/src/aot.rs:223`.

## Reproducible checks

Each is one command from the repository root, with the positive control that proves it can return something.

```sh
# 1. No symbol reaches the semantic layer. Corrected 2026-08-07: the bare
#    pattern returns two doc-comment hits and always did, so the check excludes
#    comment lines rather than claiming an empty result it never had.
grep -rn "ShapeSymbol\|SourcedExtent\|SourcedShape\|ShapeEnv" crates/tiler-ir/src/semantic.rs crates/tiler-ir/src/semantic/ | grep -v '///'
#    Returns nothing. Without the filter it returns semantic/slice.rs and
#    semantic/softmax/tests.rs, both prose about the index layer's vocabulary.
#    Positive control: the same unfiltered pattern over crates/tiler-ir/src/index/
#    returns ten files, six when this record was written.

# 2. The semantic shape encoding is untagged, so it cannot absorb a symbol.
grep -n 'fn encode_shape' -A 6 crates/tiler-ir/src/semantic/identity.rs
#    Rank, then eight raw bytes per extent. Positive control: SourcedExtent::encode
#    at crates/tiler-ir/src/shape/sourced.rs:222 pushes self.tag() first. Repointed
#    2026-08-07: the encoder moved module with its type and its bytes did not
#    change, so index/sourced.rs:202 is now an unrelated conversion.

# 3. No shape environment reaches the artifact or cache crates.
grep -rn "ShapeEnv" crates/tiler-artifact/src crates/tiler-cache/src
#    Returns nothing. Positive control:
#    grep -rl "ShapeEnv" crates/ --include="*.rs" | cut -d/ -f2 | sort -u
#    returns tiler, tiler-compiler, tiler-ir, tiler-macros, tiler-reference.

# 4. The expansion already builds a real environment and states no constraint.
grep -n "ShapeEnvBuilder::new" crates/tiler-macros/src/binding.rs
grep -n '\.require(\|\.guard(\|SemanticInputConstraint\|ExtentRelation' crates/tiler-macros/src/binding.rs
#    The first returns line 466; the second returns nothing. Positive control:
#    the same second pattern over crates/tiler-ir/src/index/sourced.rs returns
#    the fixtures that do state relations.

# 5. A wholly undetermined symbolic copy already verifies, with no interval.
grep -n 'fn a_wholly_undetermined_dynamic_copy_verifies_by_proved_extent_equality' -A 22 crates/tiler-ir/src/index/sourced.rs
#    One relation, m == n, and both proofs close. Positive control: the
#    neighbouring test with an empty relation list is still refused.

# 6. A symbolic region is refused a delivery family, naming this ticket.
grep -rn "carry-symbolic-extents-into-the-semantic-program" crates/
#    Three hits: the module doc, the diagnostic at aot.rs:223, and the byte-compared
#    golden. Positive control: the golden is a checked-in .stderr, so the exact
#    sentence is compared rather than merely produced.
```

## What this record does not decide

- **The exact public surface.** A1 through A7 are drafts. Acceptance of a public crate, module, trait, type, or call-site boundary is Tom's regardless of how the derivation ran.
- **Whether the environment's identity is later split into graph-meaning and interface halves.** Deferred above with its trigger; it reopens an accepted identity domain and needs a consumer that must make the comparison.
- **Any physical portfolio over a symbolic extent.** Which variants are packaged, which guards are worth their cost, and how the portfolio is bounded remain the specialization boundary's own open experiments; this record takes no measurement and packages nothing.
- **Rank polymorphism.** Fixed rank is accepted and unchanged. A symbolic *extent* is not an unknown rank, and nothing here weakens the decision that `Unranked` is outside the initial semantic tensor type.
- **The additive extent relation — settled outside this record.** [`admit-an-additive-extent-relation`](../../../tickets/admit-an-additive-extent-relation.md) delivered the accepted fixed two-addend spelling for `S == C + T`; this record neither owns nor broadens it, and the relation does not turn `SourcedExtent` into an expression tree.

## Delivery tickets filed from this record

Dependency-ordered, smallest vertical first. Public boundaries remain drafts until Tom reviews their exact implementation.

| Order | Ticket | Outcome | Waits on |
| --- | --- | --- | --- |
| 1 | [`relocate-the-sourced-extent-vocabulary-to-the-shape-module`](../../../tickets/relocate-the-sourced-extent-vocabulary-to-the-shape-module.md) | One constant-or-symbol vocabulary lives in `tiler_ir::shape`, reachable by both layers, with the index layer's accepted behaviour unchanged. | — |
| 2 | [`carry-a-sourced-shape-on-semantic-values`](../../../tickets/carry-a-sourced-shape-on-semantic-values.md) | A semantic value's shape is a `SourcedShape`, the builder takes an environment at construction, and `SemanticProgram::shape` is one total view. | 1 |
| 3 | [`resolve-semantic-shape-inference-over-symbolic-extents`](../../../tickets/resolve-semantic-shape-inference-over-symbolic-extents.md) | The registry's elementwise rule decides symbolic operands through `proves_equal`, and a not-proved pair returns a typed refusal naming both extents. | 2 |
| 4 | [`fold-the-shape-environment-into-semantic-identity`](../../../tickets/fold-the-shape-environment-into-semantic-identity.md) | `SemanticIdentity` gains its fifth subject, the graph domain advances to `v3` with a tagged extent encoding, and the shape-env domain stays. | 2 |
| 5 | [`construct-a-symbolic-region-as-a-semantic-program`](../../../tickets/construct-a-symbolic-region-as-a-semantic-program.md) | `ProgramEvidence::DeferredSymbolicExtent` is removed because a `sym n` region builds and verifies as a real `SemanticProgram`. | 3, 4 |
| 6 | [`admit-symbolic-extents-at-the-compiler-request-boundary`](../../../tickets/admit-symbolic-extents-at-the-compiler-request-boundary.md) | `StaticShapeEnvironment`'s version gate is replaced by a request carrying the program's own environment, and an unsupported symbolic case declines with a typed reason. | 5 |
| 7 | [`deliver-an-artifact-family-from-a-symbolic-region`](../../../tickets/deliver-an-artifact-family-from-a-symbolic-region.md) | A `sym n` region states `deliver macos;` and reaches the same expansion-time AOT flow a literal region reaches, with one artifact identity across every bound extent. | 6 |
