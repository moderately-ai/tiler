---
id: integrate-opaque-calls-into-the-physical-frontier
title: Integrate opaque calls into the physical frontier as alternatives
status: todo
priority: p1
dependencies: [implement-opaque-physical-call-providers]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, extensions]
---
Split from `implement-opaque-physical-call-providers`, which delivered the declaration and registration machinery. This is the remainder, and it is different in kind: every piece landed so far was **additive** — new modules beside the existing frontier — while this one must change `frontier.rs` and the surrounding physical-planning path.

## What exists and must not be rebuilt

| Piece | Module |
| --- | --- |
| uncertain pressure estimates, provenance, explicit `Unknown` | `crate::estimate` |
| effects, motion, aliasing, conservative meet | `crate::effects` |
| typed failure stages and the fallback boundary | `crate::failure_stage` |
| named, role-typed ABI | `crate::call_abi` |
| affinity and memory-domain placement | `crate::call_placement` |
| cross-declaration coherence | `crate::call_declaration` |
| identity and registration | `crate::call_registry` |

Applicability is **already solved**: `frontier::TargetApplicability` resolves which providers apply to a target profile, over governed `TargetProfileKey`s with canonical deduplicated ordering. Do not add a second predicate over that question.

## The three remaining items, and why each is here rather than in the parent

**Additive coexistence with scheduled kernels.** An opaque call and a scheduled kernel must be able to be alternatives for one region. `ProposalBody::OpaqueCall` already exists as a variant the bounded frontier rejects explicitly (`frontier.rs`, alongside `KernelSubprogram` and `View`), so this is admitting a rejected variant rather than inventing one. That rejection is a real edit to existing enumeration code, which is why it did not belong with the additive slices.

**Numerical guarantees.** An opaque call's numerical realization has to be stated and checked against the region's contract; nothing landed so far touches numerics. `crate::honourability` and the `NumericalRealization` on `IndexRegion` are the existing authorities — check what they already answer before adding.

**Deterministic rejection and explain behaviour.** The typed errors exist (`PlacementError`, `AbiError`, `IncoherentDeclaration`, `CallRegistrationError`) but nothing emits explain records for them. The `pipeline/tests.rs` rule census is what will catch an unreported rejection, and its `tiler.cost.analytical.v1` entry is the worked example of how a new rule's record count is pinned.

## Structural consequence to expect, not to be surprised by

Admitting `ProposalBody::OpaqueCall` makes `MaterializationForm::OpaqueRuntimeValue` reachable, and that variant is currently one of eight `Reserved` values holding `implement-boundary-property-enforcers` closed. The trigger test `frontier::tests::the_bounded_profile_admits_no_undischarged_boundary` is expected to fire as part of this work. Do not repair it by widening the bounded property sets back into agreement — its firing is the signal that the enforcers ticket has become startable, and its message names the mismatch.

## Closes when

- An opaque call and a scheduled kernel can be alternatives for one region, and the frontier admits both without either being preferred by construction.
- A registered call's declarations are verified against the region and target profile at admission, with a typed rejection naming which declaration failed.
- An unknown or absent numerical realization rejects rather than inheriting the region's, for the same reason an undeclared effect is conservative.
- Every rejection emits a typed explain record; the rule census in `pipeline/tests.rs` is updated in the same change.
- Unknown pressure estimates still cannot establish hard feasibility — the absence of a conversion from `ResourceEstimate` is preserved, not worked around at the integration point.

## Sizing the type change, measured rather than estimated (2026-07-28)

Admitting `ProposalBody::OpaqueCall` is not a one-line change to the rejecting match. `AdmittedImplementation.verified` is a concrete `VerifiedScheduledRegion` (`frontier.rs:802`), and an opaque call is not one — it has no schedule, no index region, and no iteration domain. That field must become a sum over a scheduled region and an opaque call, and **every consumer must then say what it does for a call that has neither**.

There are nine `.verified()` sites, and they fall into three groups rather than one:

*Still answerable for an opaque call* — these read provenance-level facts a call also has:
- `selection.rs:1101`, `selection.rs:1260` — `semantic_members()`, for the identity cross-check.
- `selection.rs:1106` — `target_profile_key()`.

*Not answerable, and must reject or degrade explicitly*:
- `physical.rs:870` — `lower_scheduled_region(scheduled.verified())`. Lowering an opaque call is not lowering a scheduled region; this is where the two paths genuinely diverge.
- `pipeline/planning.rs:509` — collects verified regions for the plan.
- `frontier.rs:2113`, `selection.rs:2404` — test sites.

***Silently wrong if left alone*** — and this is the group worth flagging, because it is code landed earlier in this session and the failure is not a compile error in the obvious place:
- `component_cost.rs:433` (`Indexing`) and `component_cost.rs:513` (`RedundantWork`) both do `.verified().region().index` to read `iteration_shape` and `accesses`. An opaque call has no index region, so both must report `CostValue::Unknown` for any plan containing one — **not zero**. `component_cost::tests::unknown_is_not_a_zero` exists precisely for this substitution, and a plan whose indexing cost silently became zero would be ranked as free.
- `component_cost.rs:479` (`RedundantWork`) additionally reads `semantic_members()`, which *is* answerable — so that arm needs a partial answer rather than a wholesale `Unknown`, and deciding which is a judgement to make deliberately rather than by whichever branch the borrow checker accepts first.

**`MemoryTraffic` is already safe by construction**: it matches on `numerical.profile_key` and falls to `Unknown` on anything unrecognized, so an opaque call reaches the wildcard rather than a wrong number. That was written as a dtype guard and turns out to cover this too — worth noting because the other two arms were written the same day and are not.

*The check that establishes this list, reproducible in one line:* `grep -rn '\.verified()' crates/tiler-compiler/src/` returns nine sites; `grep -n 'struct AdmittedImplementation' -A 12 crates/tiler-compiler/src/frontier.rs` shows the field is concrete.

## Started: the body sum (2026-07-28)

`frontier::ImplementationBody` — `Scheduled(VerifiedScheduledRegion)` or `Opaque(RegisteredCall)`. This is what `AdmittedImplementation.verified` must become; the field itself is unchanged so far.

**A sum rather than a trait, deliberately.** A trait would let both bodies answer one interface, and that interface would have to be the *intersection* of what they can say — which is small, and which hides that the difference matters. Lowering a scheduled region and invoking an opaque call are not two implementations of one operation; the second is a call into code this compiler did not produce. A sum makes every consumer state which it handles, and `AGENTS.md`'s requirement that unsupported cases reject explicitly rather than silently approximating is exactly what a trait's shared default would erode.

The accessors return `Option` rather than panicking: a consumer needing a schedule and holding an opaque call has to say what it does about that, and the type is where it is made to.

## The field swap is blocked, and the fix improves the design

Attempting it: the two `.verified()` sites I classed as "still answerable for an opaque call" — `semantic_members()` and `target_profile_key()` — are answerable *in principle* and not *in fact*. Both live on `VerifiedScheduledRegion` (`physical.rs:85`, `physical.rs:93`), and `RegisteredCall` holds only `{ identity, declaration }` (`call_registry.rs:121`). There is nowhere for an opaque call's members or target key to come from.

**Two ways out, and the cheaper one is wrong.**

*Add the fields to `RegisteredCall`.* Direct, and it makes registration carry facts that belong to an *admission* rather than to a registration. A call is registered once and admitted per region and per target, so the same `RegisteredCall` would have to hold different members for different admissions — either duplicated per admission, or wrong.

*Move `semantic_members` and `target_profile_key` onto `AdmittedImplementation` itself.* They are properties of the admission: *what* was implemented and *for where*. Both bodies then answer them because neither has to — the container does, once, and `ImplementationBody` holds only what genuinely differs. This also makes the sum smaller and the two "still answerable" sites stop being a special case at all.

**Done 2026-07-28.** `AdmittedImplementation` now holds `semantic_members` and `target_profile_key` directly, set at admission from the verified region. `VerifiedScheduledRegion` keeps its own for its own uses; the three consumers in `selection.rs` read the admission's.

The move is currently invisible — the fields are derived from `verified` at construction, so they cannot disagree with it yet. That changes the moment the body becomes a sum: the derivation goes away and the fields become the only source, which is the point. `selection.rs:1101` already checks the admission's members against the cover region's, so a construction site that set them wrong would fail there rather than pass quietly.

**Then, in order:** swap `verified: VerifiedScheduledRegion` to `body: ImplementationBody`, and work the remaining six `.verified()` sites. Three groups, and only one is a judgement call:

- `physical.rs:870` — `lower_scheduled_region(scheduled.verified())`. Lowering an opaque call is not lowering a scheduled region; this is where the paths genuinely diverge and must reject with a typed reason.
- `pipeline/planning.rs:509`, `frontier.rs` and `selection.rs` test sites — mechanical, follow the compiler.
- `component_cost.rs:433` and `:513` — **the judgement**. `Indexing` and `RedundantWork` read `.region().index` for an iteration shape and an access list an opaque call does not have. They must report `CostValue::Unknown`, **not zero**: a plan whose indexing cost silently became zero would be ranked as free. `MemoryTraffic` is already safe — it falls to its wildcard on an unrecognized profile key — which it does by accident of its dtype guard rather than by design.

**A caution from doing this slice.** Two edits in a row landed in the wrong place — one inserted a definition between an existing `#[derive]` and its struct, silently reassigning the derive; the other omitted a test import. Both were caught immediately by the compiler, but the first is the shape worth watching in this file: `frontier.rs` is 2000+ lines of adjacent doc-commented items, and anchoring an insertion on a `struct` line rather than on its attributes puts the new item inside the previous one's annotations. Anchor on the doc comment, or check the diff.

## The swap has an eighth site, and it is the one that matters (2026-07-28)

Attempting the field swap: the compiler finds **seven** consumers, not six. The one my earlier survey missed is `AdmittedImplementation::resources()` (`frontier.rs:932`), which reads `self.verified.requirements()` — and it is the hardest of them.

`resources()` returns `ResourceRequirements` unconditionally, and hard feasibility consults it. An opaque call has resource requirements too, but **`RegisteredCall` carries none**: the declaration holds ABI, effects, and placement, and the ticket's own three evidence classes put *exact or proven-upper-bound* requirements in `ResourceRequirements` and *uncertain* pressure in `crate::estimate` — a registered call currently has neither.

Three ways out, and two are the failure this ticket exists to prevent:

- **Default the opaque arm.** `ResourceRequirements::default()` compiles and silently tells feasibility an opaque call needs nothing. That is a wrong answer to a hard-feasibility question, which is the exact substitution `AGENTS.md` forbids when it says feasibility must reject explicitly rather than hide behind a cost.
- **Return the estimate instead.** `crate::estimate` deliberately has *no* conversion into `ResourceRequirements`, and that absence is the enforcement — see its module header. Routing around it here would defeat the type-level guarantee built for exactly this moment.
- **Make `resources()` return `Option`, and give `RegisteredCall` a declared `ResourceRequirements`.** A provider that wants its call admitted must state requirements it can *prove*; that is what the first evidence class is for, and an opaque call with none is one feasibility cannot admit. This is the surviving option.

**Unblocked 2026-07-28.** `OpaqueCallDeclaration` now carries a proven `ResourceRequirements` as its fourth part. A provider that wants its call admitted must state requirements it can prove; a call with none is one feasibility cannot admit, which is what the first evidence class is for.

The coherence check gained a rule the new part makes possible: **buffer bindings below the parameter count is incoherent.** Every parameter must be bound, so a call declaring one binding for two parameters cannot be invoked. The two numbers come from different declarations and neither can see the other, which is exactly what this check is for. Tested against a sufficient count as well, so a comparison the wrong way round fails rather than passes.

`resources()` on the declaration is what `AdmittedImplementation::resources()` will read for the opaque arm — the missing answer that reverted the swap.

*The check, reproducible in one line:* apply the swap and run `cargo check -p tiler-compiler --all-targets`; the seven sites are listed, and `frontier.rs:932` is the one with no mechanical answer.

## The field swap landed (2026-07-28)

`AdmittedImplementation` holds `body: ImplementationBody` instead of a bare `VerifiedScheduledRegion`. All seven consumers dispositioned:

- **`resources()`** — matches, and both arms answer from their own authority: a scheduled region derives its requirements, an opaque call declares them as proven. Neither is defaulted; feasibility is never told a call needs nothing because nobody said.
- **`component_cost` `Indexing`, `RedundantWork`, `MemoryTraffic`** — `scheduled()?` inside the fold, so a plan containing an opaque call reports `CostValue::Unknown`. **Not zero**, which would rank such a plan as free. `MemoryTraffic` already had a wildcard for an unrecognized dtype and now has this too.
- **`plan_region_order`** — filters rather than rejects. It is an ordering helper, not an admission check; the stage that must *lower* a plan is where an unlowerable body refuses, and doing it in both places would put the refusal where it has less to say.
- **Two test sites** — one now reads the admission's members directly, the other asserts a scheduled admission explicitly.

`semantic_members()` and `target_profile_key()` are unaffected: they moved onto the admission last change, which is why the swap did not have to answer them per-body. That sequencing was the point.

## What remains

- **Admit `ProposalBody::OpaqueCall`** in `enumerate_frontier`. Larger than it looks, and the shape is settled below.
- **Lowering must reject an opaque body** with a typed reason, at the stage that lowers.
- **Numerical guarantees** — an opaque call's realization stated and checked against the region's contract; nothing yet touches numerics.
- **Explain records** for the typed rejections; the census will move there and only there.

**Expect the boundary-enforcers trigger to fire** once `OpaqueCall` is admitted — `MaterializationForm::OpaqueRuntimeValue` becomes reachable, and `frontier::tests::the_bounded_profile_admits_no_undischarged_boundary` is designed to fail at exactly that moment. Do not repair it by widening the bounded property sets; its firing is the signal that `implement-boundary-property-enforcers` has become startable.

## Admitting the variant needs the registry threaded, and there is a precedent (2026-07-28)

`ProposalBody::OpaqueCall(ReservedProposalSeam)` carries a **placeholder**, not a payload — every reserved variant does. So admitting it means deciding what a provider actually proposes, and the two candidates differ in whether registration means anything:

- **Propose a whole `RegisteredCall`.** No registry parameter needed, and it makes registration decorative: a provider could propose a call it never registered, and the registry would stop being the authority on what calls exist.
- **Propose an `OpaqueCallIdentity`, resolved against the registry.** Registration becomes the gate it was built to be — an unregistered identity is a typed rejection rather than an admitted call.

**The second, and it requires `enumerate_frontier` to take a registry.** That is 21 call sites (16 in `frontier.rs`'s own tests, 4 in `selection.rs`, and one real caller at `pipeline/planning.rs:224`), plus deciding where the registry enters the compile path.

**The precedent answers that last part rather than leaving it open.** `crate::capability` already solves the same problem: `LoweringCapabilityRegistryBuilder` with `install_governed_scalar_lowering` / `install_governed_index_access`, built by the caller and threaded in. An opaque-call registry should follow it — same shape, same lifetime, same reason. Read how the capability registry reaches the frontier and thread the call registry the same way rather than inventing a second mechanism; `capability.rs` is `pub mod` and the serial-sum prototype builds one, so the pattern is exercised end to end.

**Do not put the registry on `CompilationRequest`.** A request describes a program to compile; the set of available opaque implementations is a property of the *compiler's configuration*, exactly as lowering capabilities are. Putting it on the request would make two callers compiling the same program with different registries look like two different requests, which would poison request identity and the caching built on it.

## The registry is threaded and the identity is the payload (2026-07-28)

`enumerate_frontier` takes an `&OpaqueCallRegistry`, and `ProposalBody::OpaqueCall` carries an `OpaqueCallIdentity` rather than a `ReservedProposalSeam` placeholder.

**A provider proposes an identity, not a call.** Registration is therefore the authority on which calls exist — a provider cannot propose one it never registered. That was the choice recorded last change; it is now enforced rather than intended.

**`FrontierRejection::UnregisteredCall` is deliberately not `UnsupportedVariant`.** They say different things and are actionable differently: an unsupported variant is *this compiler's* limitation, while an unregistered identity is a provider naming something that does not exist. Reporting the second as the first would tell a caller to wait for a feature when the fix is to register the call. It carries the identity, so the rejection names what was missing.

The existing seam test now asserts the distinction — an unregistered opaque proposal must **not** appear among the unsupported-variant kinds, and must appear by name among the unregistered ones. It previously asserted the opposite, correctly for the behaviour that existed.

**Still rejected, and that is not yet a defect:** a *registered* identity still falls through to `UnsupportedVariant`, because admitting one needs feasibility, a boundary contract, and a cost derived from the declaration rather than from a scheduled region. That is the next step and the last structural one.

**The empty registry at `pipeline/planning.rs:224` is correct for now, not a placeholder to hurry past.** Threading a caller-supplied one is premature, and the reason is one line up the file:

```rust
let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
```

`pipeline/planning.rs:170`. **The frontier's providers are a hardcoded array of Tiler's own governed provider.** There is no caller-supplied provider on this path at all — so there is no provider that could propose an opaque call, and a caller-supplied *registry* would be a set of callable things nothing can name. The registry would be threaded, correct, and unreachable.

**So the ordering is the other way round from what the earlier note implied.** Caller-supplied *providers* come first; a caller-supplied call registry is only meaningful once something outside this crate can propose. That is a larger and separate piece of work — it is the extension point `crate::capability` provides for lowering, and the physical-provider path simply does not have one yet.

*The check, reproducible in one line:* `grep -n 'providers' crates/tiler-compiler/src/pipeline/planning.rs` — four hits, and the definition at 170 is a fixed one-element array.

**Revised remaining work, in order:**

1. **Admit a registered identity.** The shape is settled below; it needs no caller-supplied anything, since a test can register a call and propose it.
2. Lowering rejects an opaque body with a typed reason.
3. Numerical guarantees for an opaque call, checked against the region's contract.
4. Explain records for the typed rejections.
5. *Separately, and probably its own ticket:* caller-supplied physical providers, after which the call registry becomes reachable and worth threading.

## What admitting a registered identity actually requires (2026-07-28)

`admit_verified` (`frontier.rs:1418`) is the model, and reading it shows which of its four derivations are region-specific and which the declaration already answers.

Its own doc states the invariant to preserve: *"The provider supplies only the applicability predicate and the cost estimate; the contract and the identity are derived here from the verified region, so a provider can neither declare a boundary it does not honour nor forge an identity."* An opaque admission must keep that — the provider supplies applicability and cost, and everything else is derived.

| What `admit_verified` derives | For an opaque call |
| --- | --- |
| `boundary` via `derive_boundary_contract(&verified)` | **derivable from the declaration**, and this is what the declaration's four parts were built for — see below |
| `identity` via `verified.canonical_identity()` | must come from the `OpaqueCallIdentity`; a call has no region to be canonical over |
| `feasibility` (passed in) | the same predicate check, against `declaration.resources()` rather than a region's requirements |
| `cost` (passed in) | unchanged — the provider supplies it, as for a scheduled region |

**The boundary mapping, which is the part that looks hard and is not.** Every property the contract must state has exactly one authority in the declaration, and that correspondence is why those four parts exist:

- `StorageLayout`, `StorageEncoding`, `Alignment` — the ABI. A parameter's role does not give these; they must be *declared*, which means the ABI is one field short and this is where that shows up.
- `Materialization` — the effects' `Aliasing`. `MayAliasInputs` is what makes a result a view rather than a buffer.
- `ExecutionAffinity`, `MemoryDomain` — the placement, directly.
- `Availability`, `Visibility` — the effects' `Motion`. An `Ordered` call's result is available after its own dispatch; a `Free` one's ordering is unconstrained.

**Gap closed 2026-07-28.** `CallAbi::declare` takes `ParameterSpec { name, role, layout, encoding, alignment }`, so every binding states all three. They are per-parameter rather than per-declaration because a two-parameter call may want a dense row-major input and a differently-laid-out output, and a declaration-level answer could not say so.

**None of the three has a default**, for the same reason `CallEffects` has no `Default`: a boundary contract must state all three, and a guess would be a claim the provider never made. Tests supply them through a local `spec()` helper carrying the bounded profile's answers, which keeps the tests about names and roles without pretending the fields are optional.

Note that `is_compatible_with` now compares whole specs rather than name and role — two calls agreeing on names and roles but disagreeing on layout are not interchangeable, and before this change the type could not tell.

**The boundary derivation is now writable end to end:** every property in `CANONICAL_PROPERTIES` has an authority in the declaration.

*The check, reproducible in one line:* read `admit_verified` and `derive_boundary_contract` in `frontier.rs`, and `CANONICAL_PROPERTIES` in `boundary.rs`, against `OpaqueCallDeclaration`'s four parts.

## The derivation needs a parameter-to-tensor-role mapping, which nothing supplies (2026-07-28)

Writing the boundary derivation: `BoundaryRequirement` is keyed by `tensor: TensorRole` (`frontier.rs:364`), not by parameter. A contract says what is required *of the region's input tensor*, not *of binding slot 2*.

An opaque call's ABI names parameters; the boundary vocabulary names tensor roles. **Nothing maps between them.** A call with two inputs and one output has three parameters and the region has some number of tensors in each role, and the correspondence is a fact only the *proposal* knows — the provider is claiming "this call implements this region, with parameter `x` bound to that tensor".

**Guessing it is exactly the failure this ticket's own ABI work exists to prevent.** The module header for `call_abi` records why parameters are named rather than positional: a positional ABI is checkable only for arity, so swapping an input for an output passes every check a position can support. Inferring a parameter's tensor role from its `ParameterRole` or its slot would reintroduce that, one level up — `In` does not tell you *which* input.

**The binding check landed 2026-07-28.** `call_abi::check_bindings(abi, bindings)` validates a parameter-to-role mapping: every declared parameter bound, no undeclared one bound, none bound twice.

**And one rule the shape forced, which is the interesting part.** A boundary contract states **one** answer per tensor role, while an ABI states storage per *parameter*. So two parameters bound to the same role must agree on layout, encoding, and alignment — two inputs wanting different layouts is a coherent thing for a call to want and an incoherent thing for one contract to say. `RoleStorageDisagreement` refuses it rather than resolving it by whichever parameter was seen first, and it names the *first* parameter so the report is stable rather than iteration-order dependent.

The test covers the accepting case (without which the three rejections would pass against a check that refused everything) and confirms the same two parameters on *different* roles are judged separately — one answer per role, not one answer overall.

It is generic over the role type, so it does not drag `TensorRole` into `call_abi`; the admission supplies it.

## The binding is threaded and checked at admission (2026-07-28)

`ProposalBody::OpaqueCall` now carries an `OpaqueCallProposal { call, bindings }`, and `enumerate_frontier` validates both halves of the provider's claim in order:

1. **Unregistered identity** → `UnregisteredCall`. The registry is the authority on which calls exist.
2. **Registered but ill-bound** → `MalformedBinding`, carrying the ABI's own typed fault so the rejection says *which* parameter and *how* — not merely that something was wrong.
3. **Registered and well-bound** → still `UnsupportedVariant`, because admitting one needs the boundary contract and identity derived from the declaration. That is now the only thing left.

**`MalformedBinding` is deliberately not `UnregisteredCall`.** The call exists and the provider described how to bind it wrongly; conflating them would send someone to register a call that is already registered. Three rejections now separate three distinct provider mistakes, and each names the thing that was wrong.

**Remaining:** derive the boundary contract from the declaration (the mapping table is above) and the identity from the `OpaqueCallIdentity`, then admit. After that: lowering rejects an opaque body, numerical guarantees, and explain records for these three rejections.

*The check, reproducible in one line:* `grep -n 'struct BoundaryRequirement' -A 6 crates/tiler-compiler/src/frontier.rs` — keyed by `TensorRole`, with no parameter or slot anywhere in the boundary vocabulary.

## `ParameterSpec.layout` has the wrong type, and I put it there (2026-07-28)

Writing the derivation against the mapping table: `ParameterSpec` carries `layout: LayoutGuarantee`, and that is wrong for half the parameters.

`crate::boundary` deliberately types the two directions differently. `LayoutGuarantee` has one variant, `DenseRowMajor` — the only layout the bounded profile produces. `LayoutRequirement` has two, adding `UnitStrideOnAxis { axis, rank }`, because a consumer can ask for something a producer does not volunteer. That asymmetry is load-bearing and is documented at both types.

An **`In`** parameter *requires* a layout of the tensor bound to it — it can legitimately ask for unit stride on an axis. An **`Out`** parameter *guarantees* the layout it writes. One field of one type cannot carry both, and I gave it the guarantee type, which silently forbids any opaque call from requiring a strided input. That is not a limitation anyone chose; it is a field typed by whichever direction I thought of first.

**The fix, and it should keep the two directions apart rather than widen one.** Either:

- `ParameterSpec.layout: ParameterLayout` with `Required(LayoutRequirement)` / `Guaranteed(LayoutGuarantee)`, validated against the role at `declare` — `In` must be `Required`, `Out` must be `Guaranteed`, `InOut` needs both; or
- two fields, `requires: Option<LayoutRequirement>` and `guarantees: Option<LayoutGuarantee>`, with the role deciding which must be present.

Prefer the first: an `Option` pair admits all four combinations and three of them are malformed, so the type would be stating a constraint the constructor then has to re-check. `AGENTS.md`'s preference for making unrepresentable states unrepresentable applies directly.

**Encoding and alignment do not have this problem** — `StorageEncoding` and `ByteAlignment` are used unchanged on both sides of the boundary relation, which is why only layout is affected. Worth stating so the fix is not applied to all three by reflex.

*The check, reproducible in one line:* `grep -n 'enum LayoutRequirement' -A 16 crates/tiler-compiler/src/boundary.rs` — two variants against `LayoutGuarantee`'s one.

**Fixed 2026-07-28** with the role-validated sum, as recommended. `ParameterLayout` is `Required` / `Guaranteed` / `Both`, and `declare` refuses a parameter whose layout states a direction its role does not have. `matches` is an exhaustive match over the pairing rather than a pair of boolean tests, so a fourth role or a fourth layout shape is a build error instead of a combination nobody considered being silently admitted.

Two tests. The first drives all three roles accepting *and* both wrong-direction rejections, so a check that refused everything — or that only understood `In` — fails. The second pins that an input may require `UnitStrideOnAxis`: that is exactly what the single guarantee-typed field silently forbade, so it is asserted rather than left implied by the enum's existence.

**Encoding and alignment were left alone**, as noted above — they are used unchanged on both sides of the boundary relation, and applying the fix to all three by reflex would have added two enums that state nothing.

## The last asymmetry in the mapping: memory domain (2026-07-28)

Working the mapping table through to code, one property does not transfer symmetrically and it is the same asymmetry `crate::boundary` documents at the type.

A **requirement** names `AdmittedMemoryDomains` — a *set*, because a consumer that can read from either shared or device storage says so. A **guarantee** names one `MemoryDomainClass`, because an allocation is in exactly one. `CallPlacement` carries the set, which serves the requirement side directly and does **not** answer the guarantee side when the set has more than one member.

So the derivation needs a rule, and there are only two honest ones:

- **Exactly one admitted domain is required to derive a guarantee.** A call that writes must say which domain it writes into; a call admitting two and guaranteeing neither has not described where its output lives. This is a coherence check in the same family as `FewerBindingsThanParameters` — the placement and the ABI's write roles disagree, and neither can see the other.
- **A second placement field**, naming the write domain separately from the admitted read set. More expressive, and it lets a call read from either while writing to one.

Prefer the **first** until a call needs otherwise: it adds no field, it makes the common case exact, and the failure it produces is a legible rejection rather than a wrong guarantee. The second is what to reach for when a real call wants asymmetric read and write domains, and the check makes that case announce itself rather than being silently mis-derived.

Every other property transfers directly: layout and encoding and alignment from the parameter spec now that each states its own direction, materialization and availability and visibility from the effects, affinity from the placement.

*The check, reproducible in one line:* `grep -n 'struct AdmittedMemoryDomains' -B 8 crates/tiler-compiler/src/boundary.rs` — the doc there states the set-versus-one asymmetry and why it is deliberate.

## The requirement half of the derivation landed (2026-07-28)

`call_declaration::required_properties_for(parameter, placement)` builds the `RequiredProperties` for the tensor bound to one read parameter. Every governed property comes from its stated authority — layout, encoding, and alignment from the parameter's own spec; affinity and admitted domains from the placement; availability and visibility fixed for a read, since needing the value after the producing dispatch and readable without further coherence is what reading it *means*.

**Materialization is `MaterializedBuffer` and deliberately does not come from the effects.** `Aliasing` says whether a *result* may share storage with an input; it says nothing about the form an input arrives in. Reading it as though it did would let a call that returns views also declare it accepts them — two different claims from one field.

**A write-only parameter returns `None`**: its layout is a guarantee, so there is nothing to require, and manufacturing one would put a made-up layout into a contract. Tested, alongside a count assertion against `CANONICAL_PROPERTIES` — that one catches a derivation that silently omits a dimension, which matters because a requirement no guarantee speaks to fails *closed*, so an omission would make the boundary compose only by accident.

**Remaining for the derivation:** the guarantee half — same shape, taking the layout's `Guaranteed` side, `AfterOwnDispatch` and `CoherentOnProducingAffinity`, and the single memory-domain class subject to the rule recorded above. Then assemble both into a `BoundaryContract` keyed by tensor role, encode the identity from the `OpaqueCallIdentity`, and admit.
