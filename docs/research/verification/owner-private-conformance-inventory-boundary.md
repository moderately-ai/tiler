---
schema: "tiler-doc/v1"
id: "tiler.research.verification.owner-private-conformance-inventory-boundary"
kind: "research"
title: "Owner-private conformance inventory boundary"
topics: ["verification", "conformance", "identity", "registries", "architecture"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.correctness-and-testing", "tiler.contract.architecture"]
ticket: "decide-how-owner-private-conformance-inventories-cross-crate-boundaries"
---

# Owner-private conformance inventory boundary

**Status:** decision-ready research; new crate and public extension-surface decisions remain with Tom

**Reviewed:** 2026-08-24 at `36534fc0595a6f838f39bbb2ea070a86426af274`

## Result

**Proposal.** Tiler should use a federated, declaration-first conformance spine rather than a cross-crate inventory scraper. A capability, rewrite, strategy, verifier invariant, runtime route, or retained claim must not enter the accepted owner configuration unless its owner also supplies a stable subject identity and family-owned obligations. Conformance then consumes immutable projections of those declarations and joins them to independently validated evidence; it never discovers the denominator from tests, explain text, source searches, or successful runs.

The federation needs two observation routes because the populations are materially different:

1. **Repository-governed private populations** use owner-local reporters. The declaration table remains private and on the normal construction or verification path; an owner unit-test target reads the same table and writes a bounded canonical manifest to an explicit file. A repository-level command orchestrates those reporters and the cross-layer evidence member. It does not parse test-harness stdout.
2. **Runtime-configured or externally extensible populations** use immutable snapshots carried by the actual frozen registry, installed-provider environment, compilation, or verified product. Qualification of an arbitrary configured environment cannot be reconstructed by a repository-local reporter. The extension owner must either expose a read-only declaration projection or preserve it in an already public receipt. This is a consequential public-boundary change and is split by extension seam.

Both routes target one neutral, versioned manifest/evidence protocol, but the protocol owns only syntax, bounded canonical encoding, validation, and common predicate composition. It owns no subjects, obligation meaning, applicability, support status, profile membership, or evidence authority. The smallest coherent common home is a new dependency-bottom workspace crate, provisionally called `tiler-conformance-protocol`. Owner crates may use it as a development dependency for private reporters; an accepted dynamic extension snapshot may justify a normal edge from its owner; and `tiler-conformance` uses it to validate and join manifests. Admitting the crate and each normal owner edge are Tom's decisions under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md).

**Unresolved exception.** [ADR 0077](../../decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) gives `tiler-metal-aot` an empty workspace and third-party dependency closure, including development dependencies. A top-of-graph adapter can enumerate its existing `CompileStage::ALL`, but that enum does not owner-mint conformance subject IDs, obligation IDs, revisions, or evidence predicates. Therefore “adapt the public enum in conformance” is not a complete route: it would make the adapter a second authority. The AOT owner must either expose a zero-dependency owner-native immutable declaration projection that can be translated bijectively, gain an explicitly accepted protocol dependency that amends ADR 0077, or remain an explicit unknown family. This is a separate decision/pilot and blocks any claim that every owner uses the common protocol.

This is not an immediate accessor project. The first implementation work is to prove the declaration/transport contract across unlike pilots: a frozen registry, a private verifier invariant set, a dynamically installed provider environment, and the zero-dependency AOT exception. If any pilot needs defaults, source parsing, mutable access, a second identity authority, or a configuration in which subjects and declarations disappear together, the protocol design is not ready.

## Exact-base Fact audit

| ticket premise | verdict | source anchor and meaning |
| --- | --- | --- |
| The semantic registry already has an exact immutable inventory. | **verified per frozen instance; imprecise as a system denominator** | [`semantic/registry.rs`](../../../crates/tiler-ir/src/semantic/registry.rs), `struct FrozenRegistryData`, `operation_definitions`, and `value_type_definitions`: the frozen maps are exact for one registry. `index_realization_laws` remains `pub(crate)`, and no construction rule proves that every system subject must enter this registry. |
| Reference capabilities and validators are exact but owner-private. | **verified** | [`reference/registry.rs`](../../../crates/tiler-reference/src/registry.rs), `struct FrozenReferenceRegistryData` and `struct FrozenReferenceRegistry`: the ordered maps contribute to `canonical_identity`, while resolution and enumeration remain private. |
| The lowering registry exposes enough to recover its complete capability population. | **false** | [`capability.rs`](../../../crates/tiler-compiler/src/capability.rs), `impl FrozenLoweringCapabilityRegistry`: public reads expose identity, count, semantic/scalar snapshots, providers, and occurrence resolution, not the exact capability rows. Count plus provider names cannot reconstruct family, operation, signature, revision, or authority. |
| One compiler rewrite registry can be projected as the optimizer denominator. | **false** | [`rewrite.rs`](../../../crates/tiler-compiler/src/rewrite.rs), `struct RuleRegistry`, and [`normalize.rs`](../../../crates/tiler-compiler/src/normalize.rs), `normalize_semantics` and `explore_algebraic_alternatives_owned`: separate registries are constructed for canonical CSE and conditional algebraic exploration. The private `rules()` method is exact only for one instance. |
| Installed physical providers already expose the exact compilation environment. | **verified for provider identities; incomplete for provider-owned strategies and obligations** | [`physical_provider.rs`](../../../crates/tiler-compiler/src/physical_provider.rs), `struct InstalledPhysicalProviders`, `identities`, and `offered_identities`: one compilation can name its governed and caller-installed providers exactly. Provider strategy vocabularies and their obligations are not a required declaration on the public trait. |
| Public compilation and verified-product views are useful evidence receipts. | **verified; they are not a denominator** | [`session.rs`](../../../crates/tiler-compiler/src/session.rs), `struct Compilation` and `struct PlanAlternative`, plus the public verified schedule, kernel, and program identities: these views name offered/selected authorities and products reached by a run. They cannot enumerate capabilities or invariants that were never selected or reached. |
| Typed verifier diagnostics close the schedule, kernel, and program obligation universes. | **false** | [`schedule/error.rs`](../../../crates/tiler-ir/src/schedule/error.rs), `enum ScheduledRegionDiagnostic`; [`kernel/error.rs`](../../../crates/tiler-ir/src/kernel/error.rs), `enum KernelDiagnostic`; and [`program/error.rs`](../../../crates/tiler-ir/src/program/error.rs), `enum KernelProgramDiagnostic`: the types are bounded failure classifications, but many independent checks deliberately collapse to one variant. For example, [`kernel/verify.rs`](../../../crates/tiler-ir/src/kernel/verify.rs), `KernelDiagnostic::ReductionContract`, is returned from many distinct invariant checks. Error classes cannot stand in for invariant declarations. |
| `tiler-conformance` can directly consume private owner state under `cfg(test)`. | **false** | [`tiler-conformance/src/lib.rs`](../../../crates/tiler-conformance/src/lib.rs) keeps every module test-only and every item private, but Rust compiles a dependency as a normal library when another crate's test target consumes it. The retained [visibility fixture](../../../spikes/verification/owner-private-conformance-boundary/README.md) demonstrates that a dependency's `#[cfg(test)]` item is absent and its `pub(crate)` item inaccessible. |
| A Cargo feature supplies a private test-support seam. | **false** | The same fixture shows the feature succeeds only after the owner exposes a conditional `pub` item. Feature-gated public API remains public, is additive under Cargo feature unification, and cannot be described as crate-private merely because ordinary builds leave it disabled. |
| A test-only process emitter is sufficient as the long-term design. | **false** | The fixture proves only that an owner unit test can read private state and write a file. Without construction-gated declarations, canonical schema, exact owner-set closure, explicit output identity, and a validated join, the emitter can faithfully export an incomplete table. Transport is not authority. |
| Source revision plus toolchain identity prevents conditional denominator shrinkage. | **false** | The fixture gates a subject, declaration row, and implementation behind one Cargo feature. The default reporter emits two rows and the feature-enabled reporter emits three at the same source and toolchain. Configuration must be explicit or applicability must be target-independent data. |
| A new common schema belongs in `tiler-ir`. | **false** | [`architecture.md`](../../architecture.md), anchors `Keep logical IR, access relations, fusion alternatives` and `tiler-metal-aot -> []`: conformance obligations cross semantic, compiler, artifact, runtime, cache, toolchain, and performance owners. Putting their common protocol in semantic IR assigns it cross-layer evidence meaning and forces dependencies through a semantic crate, including across the AOT driver's accepted empty closure. |

These verdicts preserve the earlier [owner-universe conclusion](conformance-claim-universe-by-owner.md): exact current-container identity is not closed-world completeness, and evidence receipts are not feature rows.

## The architectural invariant

**Proposal.** The denominator is closed only at an owner construction boundary. For every admitted subject family, there must be a functionally load-bearing relation:

```text
owner declaration
  ├── stable subject identity + revision rule
  ├── family-owned obligations + applicability vocabulary
  ├── construction/registration/verification use
  └── immutable manifest projection
```

The important arrow is declaration to construction. A side manifest that merely describes independently written implementation is not authoritative: an implementation can be added without touching it. Conversely, making tests enumerate their own cases does not close the feature population because test deletion then shrinks both apparent coverage and the apparent denominator.

The construction gate differs by family:

- a registry rejects registration without a complete declaration and folds its declaration root into the frozen registry identity;
- a static built-in is constructed from an owner declaration table rather than being listed again for conformance;
- a rewrite provider cannot register without its identity and obligation declaration;
- a public physical provider cannot install without a provider declaration whose identity agrees with its provenance;
- a verifier executes a declaration-driven checker graph or typestate chain whose removal changes a checked owner root; stable semantic obligations remain distinct from checker/site identities, and only the outer boundary maps violations to existing public diagnostics;
- a runtime route or compilation stage is an exhaustive owner vocabulary whose transitions name the applicable obligations;
- a retained performance claim is admitted through its record owner and never inferred from benchmark prose.

No Rust type can prevent a maintainer from writing wholly new code that bypasses an intended constructor. The repository guard therefore also needs subject perturbations at each owner entry point. The meaningful claim is narrower: within the accepted architecture, every executable registration and every verifier checker invocation is reached through an enumerated declaration, and a deliberate bypass makes a named negative control fail. Merely tagging returned failures is insufficient: deleting the call to an entire checker can otherwise leave every surviving error tagged while malformed input becomes accepted.

**Proposal — configuration closure.** Owner declarations should be target-independent whenever the code can express that: target and feature conditions belong in explicit applicability predicates rather than in `#[cfg]` around both implementation and declaration. When a family genuinely cannot be compiled into one reporter configuration, its manifest is configuration-scoped and binds the target triple, Cargo feature set, effective `rustc --print cfg`, toolchain, and any build-script-produced configuration. The system-universe authority names the exact required configuration matrix. An unexecuted configuration is `Unknown`; running only the default host cannot silently define it away.

## Three planes that must remain separate

```text
owner declaration manifests  ── define subjects and obligations ──┐
                                                                  ├─ canonical join ─ evaluated cells
execution/proof/measurement receipts ── report observations ──────┘

goal profile ── selects accepted obligation revisions; never creates subjects
```

1. **Declaration plane.** Owner-minted subject and obligation declarations form the system universe. Addition and removal change owner roots. Removal creates a tombstone or another retained fail-loud historical disposition.
2. **Evidence plane.** Existing semantic/reference authorities, selected-plan receipts, verified schedule/kernel/program identities, artifacts, toolchain outcomes, runtime completion, measurements, proofs, and normative statements supply observations. Evidence cannot add or remove declaration rows.
3. **Policy plane.** A goal profile selects obligation revisions and exclusions under the separately decided authority policy. Profile change is reported independently from evidence change.

This separation is what lets `audit` remain green over honest red/yellow capability cells, makes `qualify` fail until selected obligations pass, and prevents a profile edit or test deletion from masquerading as implementation progress.

## Observation routes by population

| population | authoritative source | observation route | why this route is necessary |
| --- | --- | --- | --- |
| standard semantic operations, value types, index laws, reference capabilities, governed lowerings, private rewrites | owner-private declaration-backed registries | owner-local reporter writes a canonical manifest to an explicit path | avoids public compiler/test API and reads the same rows construction uses |
| schedule, kernel, program, artifact/proof/publication, runtime, and cache invariant obligations | owner-private semantic-obligation declarations plus a closed checker/invocation graph | owner-local reporter; declaration-driven executor or typestate chain proves every admitted checker remains invoked, while private checker/site witnesses map many-to-one to stable obligations and public diagnostics | public error variants and prose clauses are coarser than individual invariants; failure tagging alone cannot detect deletion of a whole checker call |
| exact typed public vocabularies such as numerical dimensions, availability phases, AOT compile stages, and runtime failure stages | owner enum plus exhaustive owner mapping | direct immutable adapter only when the owner also supplies stable subject/obligation declarations; otherwise owner-local reporter, accepted owner-native projection, or explicit unknown | an exhaustive enum closes vocabulary, not conformance meaning; the adapter must not mint identities or obligations |
| arbitrary frozen semantic/reference/lowering registries | the exact frozen registry instance | opt-in immutable manifest projection bound to the registry snapshot identity | a repository reporter can describe only governed defaults, not caller-installed rows |
| arbitrary installed physical providers and their strategies | the actual installed provider environment plus provider-owned declarations | declaration required at installation; immutable projection carried through `Compilation`/receipts | global enumeration of every linkable third-party provider is impossible; the exact configured universe is possible |
| selected plans, schedules, KIR, complete programs, artifacts, compilation, runtime, cache publication, and measurements | existing verified products and typed outcomes | identity joins and validated receipts | these are evidence for declared obligations, not new inventory accessors |
| retained performance and normative claims | accepted record owner or external authority | signed/retained claim manifest reference | source code and successful runs cannot mint normative or historical authority |

**Unsupported result.** Tiler cannot authoritatively enumerate every provider that could exist outside the current dependency graph. It can enumerate the exact governed repository universe and the exact installed environment of one compilation. A profile that claims arbitrary external-provider completeness is invalid until the provider declaration boundary is accepted and the exact installed set is bound.

## Transport and protocol

### Owner-local reporter

**Proposal.** The repository command invokes one exact owner unit-test reporter per private family. The reporter receives an explicit output directory and invocation nonce, writes one bounded temporary file, fsyncs and atomically renames it, and returns nonzero on construction, validation, encoding, or publication failure. The orchestrator rejects missing, duplicate, stale, extra, or wrong-owner files. It never parses test names or stdout, and a zero-row manifest is valid only when the family declaration explicitly permits an empty population.

The reporter carries:

- protocol version and owner/family identity;
- exact source revision and toolchain identity;
- declaration configuration identity: target triple, enabled Cargo features, effective rustc cfg, and build-script configuration where applicable;
- declaration-root identity and exact row count;
- canonical subject and obligation declarations or a bounded reference to an immutable owner snapshot;
- reporter implementation identity;
- predecessor/tombstone information required by the selected change policy; and
- a completion record covering the exact expected owner-family set.

The owner-local route is deliberately a repository qualification mechanism, not a consumer build step. Ordinary users do not run it, no `build.rs` consumes it, and no generated manifest is required to compile or execute Tiler.

### Runtime-configured snapshots

**Proposal.** A public extension seam exposes only immutable declaration data already required to validate the installed configuration. It exposes neither provider implementations nor mutation. The snapshot identity must be derived at freeze/install time, agree with the configuration identity carried into compilation, and remain reachable from the resulting receipt. A caller cannot hand the compiler an unrelated conformance manifest beside a provider; installation constructs both from one declaration.

For built-in profiles this surface is unnecessary and should not be added pre-emptively. Each public accessor or required trait method is a separate ADR 0075 decision with a demonstrated arbitrary-configuration consumer.

### Neutral protocol crate

**Proposal.** A dependency-bottom crate owns only:

- bounded stable identifier and revision wrappers;
- canonical manifest-envelope and common obligation-predicate encodings;
- validation limits, canonical ordering, duplicate/unknown rejection, and content identity;
- observation/evidence reference envelopes; and
- no I/O, process orchestration, registry traits, family semantics, profile policy, reporting colors, or owner inventories.

It should depend on no workspace crate. Canonical bytes are the output; `tiler-conformance` may use `tiler-digest` to form governed roots without forcing the digest or semantic stack into every ordinary owner. This does not by itself solve `tiler-metal-aot`: even a dependency-free protocol crate would still violate the driver's accepted empty dependency closure, while adapting only `CompileStage::ALL` would invent the missing conformance declarations outside the owner.

The strongest counterargument is that a new crate and distributed adapters are expensive for test infrastructure. The reversal evidence would be a complete design using an existing owner that (a) is reachable by every family without violating dependency direction or `tiler-metal-aot`'s closure, (b) does not acquire cross-layer semantic authority, and (c) lets private owners emit one canonical format without duplicating its validator. The current dependency graph has no such owner: `tiler-conformance` is top-only, `tiler-ir` has semantic responsibility, `tiler-digest` may not acquire governed domains or schema, and the AOT driver must remain empty.

## Decision packet

### Eliminated candidates

| candidate | disposition | elimination |
| --- | --- | --- |
| status quo: grep, test census, or registry counts | eliminated | a new feature can land outside the search vocabulary or reuse an old diagnostic while the denominator stays green |
| one hand-authored central universe manifest | eliminated | creates a second authority and permits implementation plus manifest to drift; an authorized profile editor could delete rows |
| expose every private iterator through minimal public accessors | eliminated as the universal route | creates public end-user surface for static verifier and optimizer internals, yet still does not make declaration load-bearing |
| Cargo feature for `test-support` | eliminated | the item is conditionally public, feature-unified, and available to any consumer enabling the feature; the visibility fixture demonstrates the mechanism |
| `#[cfg(test)]` items called directly by `tiler-conformance` | eliminated | dependency test configuration does not propagate across crate boundaries |
| source parser, proc-macro inventory collector, linker registration, or `build.rs` generation | eliminated | adds ambient discovery/build steps, makes completeness depend on syntax/linkage, and conflicts with self-contained ordinary consumer builds |
| move the common algebra into `tiler-ir` | eliminated | wrong cross-layer owner, broad dependency pollution, and incompatible with the AOT driver's empty closure |
| make owners depend on `tiler-conformance` | eliminated | reverses the accepted top-of-evidence graph and turns the harness into a reusable library |
| parse rendered explain text | eliminated | renderers are not a typed schema and explain records describe reached decisions rather than unreached subjects |
| defer all private families | eliminated as a destination | honest yellow is acceptable temporarily, but permanent unknown populations cannot support authoritative system progress |

### Nondominated frontier

| frontier component | correctness and strictness | maintainability/compatibility | host/runtime cost | strongest counterargument | reversal evidence |
| --- | --- | --- | --- | --- | --- |
| declaration-backed owner-local reporter | closes private governed populations without public access and can fail on subject additions | distributed owner code, but declaration and construction remain co-located; protocol centralizes only syntax | extra test processes and manifest I/O; zero kernel/runtime cost | orchestration may become slow and operationally fragile | a same-process mechanism that preserves privacy and dependency direction, or measurements showing the bounded reporter lane is unaffordable |
| immutable snapshot on an accepted extension seam | exact for arbitrary frozen/installed configurations and bound to real receipts | public contract and versioning cost limited to seams with a real external consumer | small freeze-time metadata and receipt-size cost; no search | exposes more extension metadata than ordinary compilation needs | proof that repository-governed profiles are the only supported qualification target, or a private transport for arbitrary caller-created registries |
| dependency-bottom neutral protocol crate | one canonical validator/identity grammar without acquiring owner meaning | one new crate and adapters; prevents duplicated wire formats and keeps conformance top-only | build cost must be measured; no production cost when used only as dev dependency | crate proliferation for a test concern | an existing dependency-bottom owner satisfying all responsibility and closure constraints, or a verified zero-duplication protocol with no shared crate |
| AOT owner-native zero-dependency projection, accepted protocol edge, or explicit unknown | preserves AOT ownership without silently violating its closure; exact choice remains open | projection adds a public local schema and bijection proof; an edge amends ADR 0077; unknown delays closure | projection/unknown preserve build closure; an edge adds measured build cost | no candidate preserves closure, avoids public surface, and supplies common typed declarations simultaneously | a private owner emitter that produces the common protocol without duplicated encoding, or evidence that AOT conformance meaning is normatively owned elsewhere |
| explicit unknown/unsupported result | preserves honesty while an owner is not declaration-ready | makes incomplete architecture visible and keeps migrations bounded | minimal | delays qualification | a completed owner declaration route removes the unknown |

The components compose; they are not mutually exclusive architecture choices. Owner-local reporting is the default for repository-private subjects, immutable projections are reserved for exact external configurations, the neutral protocol prevents their wire models from forking, and explicit unknown is the only correct interim state.

### Independent derivation from failure cases

Starting from attacks rather than desired APIs reaches the same architecture:

1. Delete a test: if the denominator shrinks, tests were incorrectly treated as subjects. Therefore declarations must be independent of evidence.
2. Add a verifier guard returning an existing diagnostic: if no manifest changes, diagnostic variants were incorrectly treated as obligations. Therefore verification paths need stable owner obligation IDs.
3. Delete the call to a whole checker: if the obligation manifest and surviving tagged failures stay valid, failure tagging was incorrectly treated as invocation completeness. Therefore a declaration-driven checker graph or typestate sequence must make removal visible without changing semantic obligation identity.
4. Add a rewrite in a second registry: if a projected first registry remains complete, instance enumeration was incorrectly treated as the compiler universe. Therefore construction must be declaration-backed across all accepted registries.
5. Install a third-party provider: if a repository manifest remains the claimed denominator, global and configured universes were conflated. Therefore installed declarations must travel with the actual configuration.
6. Gate a subject and its declaration behind one Cargo feature/target predicate: if the default reporter accepts the smaller population, build configuration was incorrectly omitted from universe identity. Therefore declarations are target-independent with explicit applicability or the authority binds an exact required configuration matrix.
7. Enable two Cargo dependents with different features: if one exposes an internal accessor to the other, a test feature was incorrectly treated as privacy. Therefore private built-ins need a process boundary or an accepted public projection.
8. Edit profile, verifier, tests, and baseline together: repository checks alone can be rewritten. Therefore profile/exception authority and owner-manifest changes require the separately selected protected/signing/transparency authority, and denominator changes are never evidence progress.

This derivation rules out the same universal-accessor, test-feature, source-parser, and central-manifest designs without assuming the current crate layout is optimal.

## Schema and identity consequences

- `OwnerId`, `FamilyId`, `SubjectId`, and `ObligationId` are separate stable domains. Concatenated display strings are not canonical identity.
- A subject identity is owner-minted and independent of test, evidence, profile, and report identities.
- Stable semantic `ObligationId` is distinct from `CheckerId` and optional `ViolationSiteId`. Several implementation sites may witness one obligation, and a checker may discharge several declared obligations; refactoring sites does not silently revise semantic obligation identity. The checker graph/root is separate invocation-completeness evidence.
- Declaration revision changes when applicability, matcher, required evidence, or obligation meaning changes; editorial prose does not.
- A frozen registry/configuration identity commits to its declaration root. A receipt claiming configuration `R` cannot present a manifest rooted in `S`.
- Owner-manifest identity commits to protocol version, owner/family, canonical declarations, tombstones, exact cardinality, and predecessor/change-policy fields. Registration order does not enter canonical identity.
- The system-universe root commits to the exact expected owner-family set and their roots. An omitted owner is an audit failure, not an empty family.
- A target-independent declaration root carries applicability as data. A configuration-scoped root commits to target, features, effective cfg, and build-produced configuration and is insufficient unless the accepted required-configuration matrix is complete.
- Evidence roots remain separate and refer to existing authoritative identities. Copying plan, artifact, device, or numerical fields into the protocol does not make the copy authoritative.
- Goal-profile identity selects exact obligation revisions and exclusions. Profile-root movement is reported separately from owner-universe and evidence-root movement.
- Unknown tags, duplicate canonical members, missing roots, unrecognized owners, identity disagreement, over-limit manifests, and unsupported protocol versions fail audit before any color is produced.

## Threat model

**Fact.** Repository-local typing and tests can prevent accidental omission and make ordinary manipulation loud. They cannot contain an actor authorized to rewrite owner declarations, implementation, protocol validator, profile, evidence, and baseline in one change. The selected [conformance authority threat model](conformance-authority-threat-model.md) therefore protects accepted universe/profile roots outside the ordinary evidence producer, while this design supplies the exact roots it must protect.

Required negative controls include:

- add a subject at its real construction site without adding a declaration: owner build or construction fails;
- delete or rename a declaration without a tombstone: predecessor/change-policy validation fails;
- add a verifier return path without an obligation ID: private verifier result type fails to compile or its path census fails;
- remove the whole invocation of one declared verifier checker: checker-graph/typestate completion fails even though no remaining error lacks an ID;
- gate both a subject and its declaration behind one feature or target cfg: target-independent declaration validation rejects it, or the required configuration matrix reports the missing configuration `Unknown`;
- emit an owner manifest from a different source revision or reporter identity: audit rejects it;
- omit one expected owner-family file, add an extra file, duplicate a subject, or report a false row count: audit rejects the whole universe;
- substitute a different frozen-registry declaration root into a compilation receipt: identity join fails;
- install a provider whose declared identity disagrees with provenance, or which omits a promised strategy family: installation or owner validation fails;
- weaken a goal profile while evidence is unchanged: regression reports policy/denominator movement, never capability progress;
- turn a newly reached yellow into red: audit remains green and the report records knowledge gain rather than evidence regression; and
- tamper with a receipt or evidence root: audit fails before qualification evaluates cells.

## Migration sequence

Use `shadow, prove, then prune` and do not expose all existing internals at once.

1. Decide the new neutral protocol crate's exact responsibility, dependency closure, and public/non-public package surface, and separately decide how `tiler-metal-aot` can supply owner-minted declarations without silently violating ADR 0077.
2. Specify manifest envelopes, bounded canonical encoding, owner-set completion, identity domains, and failure behavior without family semantics.
3. Pilot three structurally different owners:
   - the private standard reference registry, proving a frozen-registry declaration root;
   - one private kernel/program verifier obligation family, comparing failure tagging with a declaration-driven checker executor/table and a typestate chain, separating semantic obligation identity from checker/site witnesses, and proving deletion of a whole checker invocation fails; and
   - the physical-provider environment, proving exact configured-universe binding without claiming a global third-party universe.
4. Measure clean/incremental build cost, reporter process cost, manifest size, validation memory, and failure behavior. Stop if the protocol reaches kernel/runtime fast paths or makes ordinary consumers run a conformance build step.
5. Only after the pilots pass, split owner migrations by semantic/reference, optimizer/planner, IR verifier, artifact/proof, runtime/cache, toolchain, and retained-performance authority.
6. Dual-run bounded source censuses and new owner manifests. Every old row maps to a stable new subject/obligation identity; unexplained additions and losses fail.
7. Assemble the system-universe root only when the exact expected owner-family set is explicit. Unknown families remain explicit blockers.
8. Build report and `audit`/`regress`/`qualify` commands over validated declarations and receipts.
9. Retire source censuses and duplicate tests only after the replacement proves the same subject, obligation, oracle, negative/refusal population, execution boundary, and exact count.

## Follow-up graph

The next tickets should remain research/decision work until Tom accepts the boundary:

```text
accepted P+K+M+T authority decision + owner-boundary decision
  └── specify dependency-bottom protocol and canonical owner-manifest closure
      ├── pilot declaration-backed frozen reference registry
      ├── pilot ID-bearing verifier obligations
      └── decide provider declaration and configured-universe public surface
          └── bind installed declarations to compilation receipts

specify canonical owner-manifest envelope
  └── decide zero-dependency Metal-AOT declaration route

all private-registry, verifier, provider, and AOT results + measured costs
  └── decide migration of each remaining owner family
      └── assemble exact system-universe root
```

Existing optimizer/planner, artifact/proof, runtime/cache, performance, receipt-join, profile, and reporting tickets remain the family-semantic owners. The protocol specification depends on the accepted P+K+M+T authority decision. The canonical receipt/freshness ticket then depends on the concrete protocol, making the `P`, `K`, and transitively `M`/`T` design graph consume the same owner-root and closure model rather than advancing a second one. This ticket must not rewrite family obligation meaning.

## Decision requested from Tom

Accept or reject this boundary direction:

> Admit a dependency-bottom, authority-neutral conformance protocol crate and use a federated declaration-first architecture: owner-local reporters for private repository-governed populations, immutable declaration snapshots only on extension seams that must qualify arbitrary configured environments, and existing verified products/receipts as a separate evidence plane.

Acceptance does not approve any concrete accessor, provider trait method, manifest field set, serialization tag, reporter command, AOT exception route, or migration. Those remain bounded child decisions after the pilots. The AOT family stays explicitly unknown until Tom separately chooses among an owner-native public projection, amending ADR 0077 for a protocol edge, or deferral. Rejection of the general direction should identify which accepted constraint may move: owner privacy, the conformance crate's top-only dependency direction, arbitrary configured-environment qualification, or one canonical validated protocol.

## Reproduction

```sh
git rev-parse HEAD
git status --short --branch
cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[] | [.name, ([.dependencies[] | select(.path != null) | .name] | sort | join(","))] | @tsv' \
  | sort
sh spikes/verification/owner-private-conformance-boundary/check.sh
rg -n 'RuleRegistry::new|\.register\(' crates/tiler-compiler/src/normalize.rs
rg -n 'pub fn canonical_identity|pub fn capability_count|pub fn providers|fn resolve\(' crates/tiler-compiler/src/capability.rs
rg -n 'pub fn identities|fn offered_identities|fn providers' crates/tiler-compiler/src/physical_provider.rs
rg -n 'KernelDiagnostic::ReductionContract' crates/tiler-ir/src/kernel/verify.rs
```

The fixture output at this base was:

```text
error[E0425]: cannot find function `test_only_inventory` in crate `owner`
error[E0603]: function `private_inventory` is private
owner-private boundary fixture: 2 refusals, 1 conditional-public success, 1 private emitter success, 1 configuration-dependent population
```

## Evidence boundary

This report is source-derived architecture research plus a bounded Rust visibility experiment. It does not prove the proposed protocol schema, owner-set closure, dynamic-provider contract, verifier instrumentation, host cost, or migration equivalence. It adds no crate, public API, feature, serialization format, build step, or conformance verdict. Those absences are intentional stop conditions, not implementation gaps silently treated as accepted.
