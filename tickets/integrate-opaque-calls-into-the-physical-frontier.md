---
id: integrate-opaque-calls-into-the-physical-frontier
title: Integrate opaque calls into the physical frontier as alternatives
status: done
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

## The guarantee half landed too (2026-07-28)

`guaranteed_properties_for(parameter, effects, placement)`. The mirror of the requirement half, with the differences the boundary vocabulary makes deliberately: the layout's guaranteed side, `AfterOwnDispatch`, `CoherentOnProducingAffinity`, and one memory-domain class rather than a set.

**Materialization comes from the effects here, and only here.** `Aliasing` is a statement about *results*: `MayAliasInputs` makes the guarantee an `AliasView`, `Distinct` a `MaterializedBuffer`. The requirement side does not consult it, and a test asserts both at once — that aliasing moves the guaranteed materialization *and* leaves the required one alone. A call that returns views does not thereby accept them, and one field driving both would have said it did.

**`AmbiguousWriteDomain` implements the rule recorded above** rather than picking a domain. Tested with a two-domain placement, so the refusal is exercised rather than assumed.

**Note for whoever admits these:** `MaterializationForm::AliasView` is now constructible, and it is one of the eight `Reserved` values holding `implement-boundary-property-enforcers` closed. An opaque call declaring `MayAliasInputs` will reach it.

## Where the assembly must live, and why (2026-07-28)

`BoundaryRequirement` and `BoundaryGuarantee` have private fields and **no constructors** — they are built with struct literals inside `frontier.rs` (`frontier.rs:564` and its neighbour), which is the only module that can. So the assembly cannot live in `call_declaration` beside the two halves.

**Put it in `frontier.rs`, next to `derive_boundary_contract`.** That is where the scheduled path already assembles a contract from a verified region, and the opaque path assembling one from a declaration belongs beside it — the two are the same operation over different evidence, and a reader comparing them should not have to cross a module. The alternative, giving the contract parts public constructors, widens a type's surface to serve one caller and lets anything build a contract from anything.

**The two halves stay where they are.** `required_properties_for` and `guaranteed_properties_for` read only the declaration, so they belong with it; `frontier.rs` calls them and does the construction. That split is the same one `derive_boundary_contract` already has — it reads a region and constructs here.

**What the assembly does:** group the bindings by tensor role; for each role, derive a requirement from any bound parameter that reads and a guarantee from any that writes. **Parameters sharing a role provably agree** on layout, encoding, and alignment — that is what `check_bindings`' `RoleStorageDisagreement` rule guarantees — so the grouping cannot produce a contract that contradicts itself, and picking "any" bound parameter is well defined rather than arbitrary. That rule was forced by writing the binding check and turns out to be this step's precondition.

Then encode the identity from the `OpaqueCallIdentity` and admit.

*The check, reproducible in one line:* `grep -n 'impl BoundaryRequirement' -A 6 crates/tiler-compiler/src/frontier.rs` — accessors only, no constructor.

## The contract assembly landed (2026-07-28)

`frontier::derive_call_boundary_contract(declaration, bindings)` — the opaque twin of `derive_boundary_contract`, beside it for the reason recorded above.

It groups bindings by tensor role and, for each role, derives a requirement from a bound parameter that reads and a guarantee from one that writes. Picking "a" parameter per role is well defined because `check_bindings` already refuses a binding whose same-role parameters disagree about storage — without that rule this would be picking arbitrarily and calling it a derivation.

Ownership is `TotalRaceFreeWrite`: a call that writes a tensor owns that write completely, and a partial or racing write is not something this vocabulary can express, so admitting one would claim more than the declaration says.

**The test binds the same two parameters to swapped roles** and requires the contract to move with them. That is what confirms the derivation reads the *binding* rather than the parameter's position or its own role name — the distinction the whole named-parameter design exists to preserve, checked at the point it finally matters.

## The identity encoder is generalized; admission is blocked on feasibility (2026-07-28)

`encode_proposal_identity` now takes the subject's canonical **bytes** rather than a `CanonicalScheduledRegionIdentity`, so both paths supply their own. `encode_call_subject` produces them for an opaque proposal.

**The bindings are part of the identity, not only the call.** The same registered call bound to different tensor roles is a different implementation — it computes a different thing — so omitting them would let two such proposals collide, and the collision would surface as one silently shadowing the other in the admitted set. Roles are encoded by exhaustive match rather than from the discriminant, so reordering `TensorRole` is a build error rather than a silent change to every opaque identity ever encoded.

**Admission is blocked, and on something real.** An `AdmittedImplementation` needs `ProvenEvidence`, and its only producer is `physical::verify_schedule_with_feasibility` — which bundles the feasibility decision with *verifying a schedule*. An opaque call has no schedule to verify, so there is no way to obtain evidence for one without either fabricating it or splitting that function.

**Fabricating it is the failure this whole ticket has been avoiding.** `ProvenEvidence` for a call nothing proved would tell hard feasibility a call was admissible on no evidence — the same substitution `resources()` refused when it declined to default the opaque arm, and the same one the absent `ResourceEstimate` → `ResourceRequirements` conversion exists to prevent. The admission block was written and then **reverted** rather than landed with a fabricated proof; the contract derivation is wired and its result discarded, so the code path is exercised and nothing is claimed.

**Correction: the split already exists.** `physical::assess_region(region, requirements, work_items, target) -> Result<ProvenEvidence, PhysicalError>` (`physical.rs:655`) *is* the resource-only feasibility check, and `verify_schedule_with_feasibility` calls it after verifying the schedule. Nothing needs splitting; the single hard-feasibility decision is already one function.

**What an opaque call cannot supply is `work_items`.** The scheduled path passes `verified.region().schedule.work_items`. A declaration states `ResourceRequirements` — which includes `threads_per_workgroup` — but not how many work items a dispatch of the call performs, and the two are different quantities: threads per group is a shape, work items is a count.

`region: RegionId` is the other argument, used only for error attribution, and an opaque admission has a region subject to attribute to.

**So the remaining gap is one number, and it needs a decision about where it comes from.** Two candidates:

- **A fifth declaration field.** A call declares the work items a dispatch performs. Honest and static, and wrong if the count depends on the bound tensors' shapes — which for most real calls it does.
- **Derived from the bound tensors at admission.** The frontier knows the region subject, so it could compute the count the way the scheduled path does. Correct for shape-dependent calls, and it requires the call to say *how* it scales — which is a declaration field of a different kind, closer to the launch geometry a scheduled region carries.

**Decided and landed 2026-07-28: the call declares how its work scales.** `WorkScaling` is `PerElementOf(parameter)` or `Fixed(count)`, a sixth part of the declaration, validated in `check` — a scaling naming a parameter the ABI does not declare is `WorkScalingNamesUnknownParameter`.

*Why not a plain number.* A fixed count is honest for a call that does the same work whatever it is given and wrong for most real ones. Forcing shape-dependent calls to state one would make them lie, and a lie here is a feasibility verdict that is confidently incorrect: too small admits a call the target cannot run, too large rejects one it can.

*Why per-parameter rather than per-call.* The count follows a *particular* tensor. A call reducing a large input to a small output does work proportional to the input, and only the call knows which — naming the parameter says so, where naming nothing would leave the frontier to guess.

The test drives both accepting forms plus the rejection, so a check refusing everything fails it.

**Two arguments still have no source at this seam, and I was wrong to say every one did.**

*`PerElementOf` cannot be evaluated where the proposal is admitted.* `FrontierRegionSubject` carries a role and `semantic_members` — identifiers, not shapes. Evaluating a per-element scaling needs the element count of the tensor bound to that parameter, and the frontier does not hold tensor shapes. A `Fixed` scaling evaluates trivially; a shape-dependent one does not.

*`assess_region` wants a `RegionId` for error attribution*, and an opaque call has none — it is not a region, which is the whole reason `ImplementationBody` exists.

**Neither should be papered over.** Passing a synthesized `RegionId` makes a feasibility error attribute to a region that does not exist; substituting any work count for an unevaluable scaling is the confidently-wrong verdict `WorkScaling` was designed to prevent.

**Two honest ways forward, and they differ in scope:**

- **Admit `Fixed` scalings now, reject `PerElementOf` with a typed rejection** naming shapes-unavailable-at-this-seam. Small, lands real admission, and leaves shape-dependent calls — most real ones — refused for a stated reason rather than mis-admitted.
- **Give the frontier the shapes.** `enumerate_frontier` already takes a `VerifiedTargetRequest`, and that type exposes `serial_sum() -> &NormalizedSerialSum` (`request.rs:820`) — the normalized program, which carries the tensors and therefore their shapes. **So no new parameter is needed**; resolving a `SemanticMemberId` to a tensor shape through that program is the work, and it is a lookup rather than a plumbing change. If it resolves, both scalings evaluate and this is the better answer.

The `RegionId` is separable from that choice: `assess_region`'s first argument is used only to attribute errors, so it should become something both callers can supply — a subject identifier rather than a region one. That is a small change to one signature with two callers.

*The check, reproducible in one line:* `grep -n 'struct FrontierRegionSubject' -A 5 crates/tiler-compiler/src/frontier.rs` — a role and members, no shapes.

*The check, reproducible in one line:* `grep -n 'fn assess_region' -A 6 crates/tiler-compiler/src/physical.rs` — four arguments, of which only `work_items` has no source in an `OpaqueCallDeclaration`.

*The check, reproducible in one line:* `grep -n 'ProvenEvidence' crates/tiler-compiler/src/frontier.rs` — it is a field and an accessor here, produced only by that one call.

**Then:** admit, lowering rejection, numerical guarantees, and explain records for the three rejections.

## Work resolution landed (2026-07-28)

`frontier::resolve_work_items(work, bindings, request)`. `Fixed` resolves directly; `PerElementOf` resolves through the tensor role its parameter is bound to, reading `input_elements` or `output_elements` from `request.serial_sum()`. No new parameter was needed — the frontier already held the request.

**The test requires the two roles to give *different* counts** and asserts the fixture actually does, so a resolution that ignored the binding and returned one count for everything fails. That is the same property the contract-assembly test checks one level up, and it is worth checking twice because "follows the binding" is the thing every shortcut here would break.

**`Intermediate` returns `None`, and that is a decision rather than a gap left open.** The normalized request states element counts for the program's input and output. An intermediate exists because a particular *cover* chose to materialize between two regions, so its element count is a property of that cover — which the frontier does not hold when enumerating for a subject. Deriving one from the input's would be right for the pointwise case and wrong for any cover materializing something smaller. An unbound name declines for the same reason.

**Remaining for admission:** `assess_region`'s `RegionId` argument, which an opaque call has none of and which is used only to attribute errors. That argument should take a subject identifier both callers can supply. Then: resolve work, assess, construct with `ImplementationBody::Opaque`. After that, lowering rejection, numerical guarantees, and explain records for the rejections.

## The split I said was unnecessary is necessary, for a different reason (2026-07-28)

I corrected myself earlier that `assess_region` is already the resource-only feasibility check and nothing needed splitting. That was right about the *computation* and wrong about the *error attribution*, and the second is what forces a split.

`region: RegionId` is not one argument threaded through — it appears in **five** error constructions inside `assess_region`, and every `PhysicalError` variant it builds carries a `region: RegionId` field. So an opaque call cannot call it without either inventing a region to blame or widening `PhysicalError` to hold a subject that may not be a region.

**Both of those are worse than a split.** An invented region attributes a real feasibility rejection to something that does not exist, and a reader chasing it finds nothing. Widening `PhysicalError` changes a type used across the physical path to serve one new caller, which is the same trade rejected when the contract parts were left without public constructors.

**The split that works:** a core that assesses and returns the *cause* without attributing it —

```text
fn assess_resources(requirements, work_items, target)
    -> Result<ProvenEvidence, RejectionCause>
```

— with today's `assess_region` becoming a thin wrapper that maps that cause onto `PhysicalError` with its `RegionId`, and the opaque path mapping the same cause onto a `FrontierRejection` naming the call. One feasibility decision, two attributions, which is what ADR 0043's single decision actually requires: the *verdict* is shared, and only the blame differs.

The cause is already a type — `RejectionCause` with `Numerical` and `Capability` variants, matched on in the body today — so the core's error type exists and does not need inventing. `checked_target_profile` and `region_proposal` also fail, and their errors currently go through `feasibility_intrinsic`; the core should return those unattributed too.

*The check, reproducible in one line:* `grep -n 'region' crates/tiler-compiler/src/physical.rs | sed -n '/fn assess_region/,/^$/p'` — or read `physical.rs:655` through its closing brace and count the uses: five, all attribution.

## Opaque calls are admitted (2026-07-28)

`enumerate_frontier` now admits a registered, well-bound, feasible opaque call: it derives the boundary contract from the declaration, resolves the work count through the bindings, proves feasibility with `physical::assess_resources`, encodes the identity over the call *and its bindings*, and pushes an `AdmittedImplementation` carrying `ImplementationBody::Opaque`.

**The feasibility split landed first and made this possible.** `assess_resources` returns `ProvenEvidence` or an unattributed `ResourceVerdict`; `assess_region` is now a thin wrapper attributing that verdict to a `RegionId`, and the opaque path attributes the same verdict to the call. One decision, two attributions — and the split was behaviour-preserving, which the existing feasibility tests confirmed without needing a change.

**`CallNotAdmissible` covers the three per-target failures** — contract underivable, work unresolvable, target infeasible — each with a stable reason. They are deliberately one variant separate from `MalformedBinding`: a malformed binding is wrong *everywhere*, while these three are this proposal on *this* target.

**The admission test is the payoff and checks more than "it was admitted".** It asserts no rejections, the `OpaqueCall` kind, that `scheduled()` is `None`, and that the boundary and work count match what the same derivation functions produce independently — so an admission that wired something up wrongly fails rather than passing on shape alone.

## What remains

- ~~**Lowering must reject an opaque body**~~ — **done 2026-07-28, and it was already a live defect.** `plan_region_order` *filtered* bodies with no scheduled region, which was harmless while none were admittable and became silently wrong the moment they were. A plan of one scheduled region and one opaque call filtered to a single region, and `build_plan_program` then matched it as a **fused** program — producing a kernel that omits the call's work entirely and reporting success. Nothing downstream compared the region count against the selection count, so it would have surfaced as a wrong result rather than an error.

  It now returns `Option` and declines, with the caller turning that into a typed `unlowerable-opaque-body` refusal at the lowering stage. `pipeline/verify.rs` treats `None` as a schedule-binding failure, which is what it is — an alternative could not have been built from a plan it cannot order.

  **This is the cost of the previous change, paid one turn later.** Admitting opaque calls made a filter that had been correct into a silent omission, and nothing failed when it did.
- ~~**Numerical guarantees**~~ — **done 2026-07-28**, and checking what was already covered was the right first move. `assess_resources` compares the four numerical dimensions carried in `ResourceRequirements` against the **target profile**, so "can this device honour it" was already answered.

  What nothing answered is the different question: whether the call's declared numerics match the **request's resolved contract**. A call permitting contraction is perfectly feasible on a device that offers contraction, and still wrong for a program whose contract forbids it. Admission now compares all four against `request.numerical_contract().realization()` and refuses with `numerical-contract-mismatch`.

  The test declares a call permitting contraction where the governed contract forbids it and requires that exact reason — so a refusal for any *other* cause fails it. The positive admission test passing unchanged confirms its own numerics do match the governed contract, which is what makes the negative test meaningful rather than vacuous.
- ~~**Explain records**~~ — done; see above.
- **`Intermediate` work scaling — resolved 2026-07-28, then REVERTED the same day by audit.** The resolution rested on the claim "the bounded profile has exactly one materialization, the two-region cover's pointwise result." That claim is false: `enumerate_covers` retains the all-singleton cover unconditionally (`cover.rs:556-559`), and `cover.rs:1325-1326` states outright that the fully-materialized cover materializes *every* internal value — including the two rank-0 `F32Constant` results, whose element count is 1, not the input's. So `input_elements` was not exact, and the substitution was precisely the confidently-wrong feasibility verdict `WorkScaling` was designed to prevent — introduced one turn after designing it. The decline is restored, with the falsified premise recorded at the arm. Resolving it correctly needs the cover edge's actual value shape, which arrives with the cover rather than the subject.

## Closing criteria, checked one by one (2026-07-28)

- *An opaque call and a scheduled kernel can be alternatives for one region, and the frontier admits both without either being preferred by construction* — **met**, and asserted directly: both are admitted, both kinds appear, and exactly one carries a schedule. A frontier admitting only one, or ordering them by kind, passes every other test in this file and fails that one.
- *A registered call's declarations are verified against the region and target profile at admission, with a typed rejection naming which declaration failed* — **met**. Four rejections separate four distinct failures: unregistered identity, malformed binding, and `CallNotAdmissible` with a stable reason for contract-underivable, work-unresolvable, numerical-mismatch, and target-infeasible.
- *An unknown or absent numerical realization rejects rather than inheriting the region's* — **met**. The declaration's four numerical dimensions are compared against the request's resolved contract, and a mismatch refuses. Nothing inherits.
- *Every rejection emits a typed explain record; the census updated in the same change* — **met**, and the gap was wider than this ticket: no frontier rejection had ever been recorded, including the three predating it. The census moved 4 → 8.
- *Unknown pressure estimates still cannot establish hard feasibility* — **met**. `crate::estimate` still has no conversion into `ResourceRequirements`; the admission reads the declaration's **proven** resources, and that absence was what forced the declaration to carry them in the first place.

## What this ticket did not do, stated rather than implied

- **Lowering an opaque call is not implemented.** A plan containing one is refused with `unlowerable-opaque-body` rather than lowered. The frontier admits them; nothing yet executes one. That is the honest boundary — this ticket was the *physical frontier* integration.
- **No caller-supplied provider or registry.** `pipeline/planning.rs:170` still hardcodes one governed provider, so in the compile path the registry is empty and nothing proposes an opaque call. The seam is exercised by tests, not by a caller. Opening providers is separate work, recorded above.
- **`Intermediate` work scaling rests on a profile assumption** that expires when covers stop being two, stated at the site.

## Assessment corrections (2026-07-28, post-audit)

A 15-agent audit of this ticket's claims against source found four overstatements in the closing section and two live defects. Both defects are now fixed; the claims are corrected here rather than deleted, so the record shows what was wrong.

**Defect fixed: an `InOut` parameter's read requirement was silently dropped.** `derive_call_boundary_contract` selected the read parameter by `!writes()`, which excludes `InOut` (it writes), so the derived contract guaranteed a role the call also reads with no requirement at all — a producer of that tensor was never asked to satisfy anything. Fixed by selecting with `ParameterRole::reads` / `ParameterRole::writes` directly; the regression test binds a single `InOut` parameter and asserts the contract carries both halves, and fails against the old selector (verified by reverting it).

**Defect fixed: the `Intermediate` work-scaling assumption was false.** See the correction under the work-resolution section: `cover.rs` retains the all-singleton cover unconditionally, which materializes *every* internal value — including rank-0 constants — so "exactly one materialization" was wrong and `input_elements` was not exact. The decline is restored.

**Closing-criteria corrections:**
- *"verified against the region and target profile at admission"* — overstated. The contract is **provider-declared and ABI-checked**: it is derived from the declaration and the provider's bindings, validated against the call's own ABI, the registry, the request's numerical contract, and the target — but nothing cross-checks the bindings against the region subject's actual boundary tensors. The subject contributes only `semantic_members`. This diverges from `admit_verified`'s invariant ("derived here from the verified region") and is recorded as such; closing that gap is real follow-up work, not a wording fix.
- *"Every rejection emits a typed explain record"* — overstated. What landed is a per-region-role **count** (`rejected-count` beside `admitted-count`); the stable reason codes exist only on the in-memory frontier and never reach explain output.
- *Numerical guarantees* — the admission compares four of the six dimensions `NumericalRealization` carries. `profile_key` and `canonical_arithmetic_nan_bits` are neither declarable by a call (`ResourceRequirements` carries only the four) nor checked, while the scheduled path requires full equality. Stated at the comparison site's follow-up: a call cannot declare a canonical NaN pattern today, and that must exist before anything lowers one.
- *The enforcers-trigger prediction was wrong, twice, and in both directions.* This ticket predicted `the_bounded_profile_admits_no_undischarged_boundary` would fire during this work and named `OpaqueRuntimeValue` as the value becoming reachable. Neither happened: the test compares two compile-time constants the opaque path never touches (so it structurally cannot fire from this work), and the value that *did* become constructible is `AliasView` (via `MayAliasInputs`), not `OpaqueRuntimeValue` (still unconstructed outside its definition). The enforcers ticket's trigger is being restated separately.

**Also recorded:** everything downstream of admission — selection composition, plan identity, the `Unknown` cost arms, the lowering refusal, verify's `None` arm — is untested with an opaque body and unreachable in the compile path (no provider proposes one). Tracked by `exercise-opaque-admissions-downstream-of-the-frontier`.

## Downstream tested-at-which-level correction (2026-07-28)

The untested half above is now exercised by test-level providers in `selection::tests` and `pipeline::tests`. A scheduled and opaque admission for the same fused region produce two retained plans with distinct identities; omitting the implementation identity collapses them and fails the test. An opaque `MayAliasInputs` producer composed with the scheduled reduction is refused on the `Materialization` property. `Indexing`, `RedundantWork`, and `MemoryTraffic` all report `Unknown` for the retained opaque plan, and substituting `Exact(0)` fails. Lowering refuses that plan with `unlowerable-opaque-body`, while independent verification takes its `None` arm and reports `portfolio-schedule-binding`; restoring the old filtering behaviour fails both checks.

The compile-path boundary is unchanged: `pipeline/planning.rs` still installs only the governed provider and an empty call registry, so no production compilation proposes an opaque call. These are downstream authority tests, not a claim that opaque execution exists.
