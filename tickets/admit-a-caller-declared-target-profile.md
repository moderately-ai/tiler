---
id: admit-a-caller-declared-target-profile
title: Admit a caller-declared target profile
status: awaiting-decision
priority: p1
dependencies: []
related: [express-metal-honourability-in-the-shared-form, prototype-public-compiler-api]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, feasibility, identity]
---
The compiler admits exactly one target profile and offers no way to author another. `express-metal-honourability-in-the-shared-form` needs one and is currently unreachable without it.

## Decision needed (2026-07-28)

**The question, atomic:** in what spelling does a caller outside `tiler-compiler` declare the per-dimension numerical honourability of a profile it authored?

Nothing else in this ticket is blocked. Steps 1 and 2 of the revised order have landed, step 3 and step 4 are mechanical, and step 5 — the public builder — cannot have a signature until this is answered, because a declaration *is* a set of per-dimension rows and the row type is currently crate-private.

**Fact — this is wrapper vocabulary, not new concepts.** `NumericalDimension` (`crates/tiler-compiler/src/honourability.rs:58`), `DimensionBehaviour` (`:132`), and `HonouringMeans` (`:179`) are all `pub(crate)` in the private `honourability` module. `DimensionBehaviour` is exactly two variants, `Subnormals(SubnormalMode)` and `Transform(NumericalPermission)`, and **both payloads are already `pub` in `tiler-ir`** (`crates/tiler-ir/src/schedule/numerics.rs:56`, `:73`). So no option below exposes a concept a caller cannot already name; they differ in what vocabulary the caller has to learn and in what the compiler can change later without breaking them.

| Option | Enables | Prevents |
| --- | --- | --- |
| **(a) Promote all three types as-is from a public `honourability` module.** | The caller declares in the compiler's own vocabulary, so a rejection naming a dimension names something the caller wrote. `HonouringMeans` becomes expressible, which is what lets a caller say *how* a dimension is honoured rather than only that it is. Adding a dimension is a visible additive change. | Freezes three enums the compiler currently reshapes freely; `NumericalDimension`'s membership becomes a public compatibility surface. A caller can then construct a `DimensionBehaviour` the target does not admit and only learn at validation — `NumericalDimension::admits` exists precisely because that pairing can be wrong. |
| **(b) Re-export through the existing target-profile surface, no new public module.** | Smallest public footprint; the types are named where they are used and the module structure stays free to move. | The three types are still frozen — a re-export is not weaker than a promotion, it only hides where they live. Buys nothing over (a) except a shorter path, and costs a reader the module that documents the vocabulary. |
| **(c) A builder taking `(SubnormalMode, NumericalPermission)` pairs that never names `DimensionBehaviour`.** | Exposes zero new types; the caller uses vocabulary already public in `tiler-ir`. The compiler keeps all three enums private and reshapable. | Cannot express `HonouringMeans` at all, so a caller cannot distinguish "supported exactly" from the other means — and that distinction is load-bearing in `GOVERNED_TARGET_HONOURABILITY`. Worse, it cannot express a dimension whose behaviour is neither a subnormal mode nor a transform permission, so a third variant of `DimensionBehaviour` would be a silent gap in the caller's declaration rather than a build error. |

**Recommendation: (a).** The fail-closed `Unknown`-on-omitted-dimension rule requires the caller and the compiler to agree on what the dimension *set* is; (c) has no way to state that set, so the rule degrades from a typed refusal into an unstated convention. **The counterpoint is real and worth Tom's attention:** (a) makes `NumericalDimension` a public compatibility surface at a point when the dimension list is still growing — `compose-numerical-honourability-and-retire-the-strict-boolean` widened it once already — so each future dimension becomes an additive public change with a migration story rather than an internal edit. If Tom judges that vocabulary too unsettled to publish, (c) is the fallback, and the honest cost is that `HonouringMeans` stops being caller-declarable and the profile builder has to fix it at `SupportedExactly`.

**Constraint on the answer, restated plainly:** whichever spelling is chosen must not settle `express-metal-honourability-in-the-shared-form`'s three-way siting choice by implication. That ticket's `## The ownership decision` section carries a live decision between `tiler-ir` owning the vocabulary, a checked adapter owned by an orchestrator, and a third crate, and it records that the same choice also answers ADR 0076's open question. Cite it by construct rather than by line — `grep -n "This also decides ADR 0076" tickets/express-metal-honourability-in-the-shared-form.md` — because that ticket is under active revision and its line numbers moved while this section was being written. If the spelling chosen here forces one of those sitings, say so explicitly and record it as an accepted decision there rather than leaving it implicit in a signature.

**Provenance.** ADR 0076 item 2's public spelling was explicitly left to Tom when `compose-numerical-honourability-and-retire-the-strict-boolean` landed, and this ticket cannot proceed past step 5 without it.

## Not part of this decision

Three things are already settled and are implementation requirements rather than options: an omitted numerical dimension must resolve `Unknown` rather than becoming trivially satisfiable (the fail-closed direction `GOVERNED_TARGET_HONOURABILITY` already documents for `FlushToZero { AlwaysPositive }`); a caller-supplied profile key enters the request subject and therefore artifact identity, so key uniqueness and governance are identity questions to be recorded, not re-decided; and the quantitative bounds must be validated, because a profile declaring bounds no device has is a way to make an infeasible plan look feasible.

## Fact — declaration does not exist, and selection is not it

`crates/tiler-compiler/src/request.rs` declares `pub(crate) struct PrototypeTargetProfile` with one constructor, `governed()`. `verify_request` rejects any other profile with `UnsupportedCapability { phase: "target", rule: "prototype-target-neutral-baseline-v1" }`, and `for_target` rejects again independently, so the `Vec<PrototypeTargetProfile>` field is structurally plural and admits exactly one distinct value. Twelve sites in that file compare against `PrototypeTargetProfile::governed()`.

`prototype-public-compiler-api` landed the request half of the public boundary on 2026-07-27 — a caller can now install its own `FrozenLoweringCapabilityRegistry` — and deliberately did not land this. Selecting the governed profile and authoring a profile are different capabilities, and only the first exists.

Reproduce with `grep -n "pub(crate) struct PrototypeTargetProfile" crates/tiler-compiler/src/request.rs` and `grep -c "PrototypeTargetProfile::governed()" crates/tiler-compiler/src/request.rs`.

## Why this is filed now rather than discovered later

**The work graph is currently pointing at unreachable work.** With its dependency on `prototype-public-compiler-api` satisfied, `express-metal-honourability-in-the-shared-form` is p0, dependency-satisfied, and the top of `tkt next` — while being impossible to close. A comment on that ticket records the reason; a comment is not an edge, and `tkt next` does not read comments. This ticket exists so the graph states the blockage it already has.

## What the substance is

Not visibility. A caller-authored profile must be *validated*, and deciding what makes one well-formed is the work:

- an honourability declaration is a set of per-dimension rows, and a profile that omits a dimension must stay `Unknown` on it rather than becoming trivially satisfiable — the fail-closed direction `GOVERNED_TARGET_HONOURABILITY` already documents for `FlushToZero { AlwaysPositive }`;
- profile keys enter the request subject and therefore artifact identity, so key uniqueness and key governance are identity questions, not hygiene;
- the quantitative bounds (`max_threads_per_grid_axis`, `max_threads_per_workgroup`, `max_buffer_bindings_per_entry`, `index_bits`, `supports_device_memory`) are consumed by feasibility, and a profile declaring bounds no device has is a way to make an infeasible plan look feasible.

`PrototypeTargetProfile`'s `numerical` field is a `&'static [DeclaredBehaviour]`, and `DeclaredBehaviour` lives in the crate-private `honourability` module. Whether declaration promotes that vocabulary, or takes a different shape at the boundary, interacts directly with the three-way siting decision `express-metal-honourability-in-the-shared-form` carries. **Do not settle that siting here by implication** — if this ticket forces it, say so and record it as an accepted decision rather than leaving it implicit in a signature.

## Closes when

An out-of-crate caller can author and compile against a target profile it declared; a malformed or under-declared profile is refused with a typed diagnostic naming what was wrong; an omitted numerical dimension resolves `Unknown` rather than satisfied; the identity consequence of a caller-supplied profile key is recorded; and `make full` passes.

## Survey correction — the key type is the real blocker, and it is bigger than recorded (2026-07-27)

The byte pin the previous survey asked for **landed** (`6e7121f`): `the_governed_descriptor_bytes_do_not_move` in `physical.rs` asserts the governed profile's 249 canonical descriptor bytes exactly, with the regeneration procedure recorded beside it. Verified by mutating `max_threads_per_workgroup` — the pin fails, so it can say no. Every later step of this refactor now fails there, immediately and with a diff, instead of downstream in a compile-and-package cycle.

**Then the owned-type step was attempted and reverted, and the reason is new evidence.** Changing `PrototypeTargetProfile`'s fields to `Cow<'static, _>` — which keeps `governed()` `const` and allocation-free, and is the right shape — produced **57 compiler errors**, not the ~30 mentions the survey estimated. The extra ones were not in `request.rs` or `physical.rs`. **This measurement was taken on a pre-step-2 tree and is not a current estimate**; it is retained as the recorded evidence for why step 2 was split out, not as a prediction of what step 3 now costs. Step 2 has since landed and absorbed the applicability half of that count.

**Fact — the applicability vocabulary has moved; the residual is the reporting surface (re-verified 2026-07-28).** `introduce-a-validated-target-profile-key` landed as `690b1e6` ("Give a target profile key its own validated type"). The claims the previous revision made about `frontier.rs:202`/`:214` no longer describe the file. Current declarations: `pub(crate) struct TargetProfileKey(std::borrow::Cow<'static, str>)` at `crates/tiler-compiler/src/request.rs:59`; `target_profile_keys: Vec<TargetProfileKey>` at `frontier.rs:211`; `TargetApplicability::for_targets(keys: impl IntoIterator<Item = TargetProfileKey>)` at `frontier.rs:223`.

**Fact — eight `&'static str` bindings of the key remain, and they are the reported/verified surface rather than the applicability predicate.** `grep -n "&'static str" crates/tiler-compiler/src/{frontier,physical}.rs | grep target_profile_key` returns exactly eight: `frontier.rs:726` (accessor), `:895` (field), `:920` (accessor), `:1077` (rejection-record field), `:1166` (field), `:1178` (accessor); `physical.rs:55` (field), `:85` (accessor). Two adjacent sites are not in that grep and belong to the same step: `selection.rs:1076` binds a derived `Option<&'static str>` for the coherence check, and `PrototypeTargetProfile`'s own `key: &'static str` at `request.rs:572` — which is step 3's subject, not step 2's residue. `grep -rn "target_profile_key" crates/tiler-compiler/src` now reports 62 sites in total.

**Inference: a caller-declared profile cannot have a `&'static str` key**, so the remaining eight bindings plus `request.rs:572` have to move before the profile type can be owned. Step 2 removed the part of that work that fans out furthest; what is left is a narrower, self-contained change with its own review surface.

## Revised order

1. ~~Byte pin~~ — **done**, `6e7121f`.
2. ~~Introduce a validated `TargetProfileKey` and move the applicability vocabulary onto it~~ — **done**, `690b1e6`, split as `introduce-a-validated-target-profile-key` (`status: done`). `TargetProfileKey` is a validated `Cow<'static, str>` newtype at `request.rs:59`, and `TargetApplicability` now holds and takes it. The pin did not move.
3. **Move the eight residual `&'static str` key bindings onto `TargetProfileKey`** — `frontier.rs:726`, `:895`, `:920`, `:1077`, `:1166`, `:1178`, and `physical.rs:55`, `:85` — plus the derived `Option<&'static str>` at `selection.rs:1076`. Then make `PrototypeTargetProfile` owned: `key: &'static str` at `request.rs:572` becomes the validated key type and the remaining fields go to `Cow`, with the `Copy` ripple that follows.
4. The three rejection sites (`verify_request`, whose refusal is `request.rs:1390`; `for_target`; and the governed-profile equality check at `physical.rs:506-511`) into one validation.
5. The public builder, its typed diagnostics, and the `Unknown`-on-omitted-dimension rule. **Blocked on the decision at the top of this ticket** — the builder cannot have a signature until the honourability spelling is chosen.

Step 2 is what made step 3 tractable; attempted together they were one 57-error commit.

Note for step 4: `distinguish-the-five-compile-failure-classes` landed on 2026-07-27 and split `CompileFailureClass::Unsupported` into `InvalidRequest` and `UnsupportedCapability`. `InvalidRequest` is currently **unreachable from the public surface** precisely because `compile` builds the request structure itself — an empty target set and a duplicate profile are two of its five sources. This ticket is what makes it reachable, so step 4 adds construction paths rather than also having to widen the failure vocabulary.

The parked question recorded here on 2026-07-27 was hoisted to `## Decision needed (2026-07-28)` at the top of this ticket on 2026-07-28, with its options, eliminations, and ADR 0076 provenance intact. It is the same question; it is stated first because it is what the ticket is waiting on.
