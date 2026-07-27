---
id: admit-a-caller-declared-target-profile
title: Admit a caller-declared target profile
status: todo
priority: p1
dependencies: []
related: [express-metal-honourability-in-the-shared-form, prototype-public-compiler-api]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, feasibility, identity]
---
The compiler admits exactly one target profile and offers no way to author another. `express-metal-honourability-in-the-shared-form` needs one and is currently unreachable without it.

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

**Then the owned-type step was attempted and reverted, and the reason is new evidence.** Changing `PrototypeTargetProfile`'s fields to `Cow<'static, _>` — which keeps `governed()` `const` and allocation-free, and is the right shape — produces **57 compiler errors**, not the ~30 mentions the survey estimated. The extra ones are not in `request.rs` or `physical.rs`.

**Fact: `key: &'static str` is load-bearing across the target-applicability vocabulary, which the previous survey did not reach.** `frontier.rs:202` declares `target_profile_keys: Vec<&'static str>`; `TargetApplicability::for_targets` (`frontier.rs:214`) takes `impl IntoIterator<Item = &'static str>`; `ImplementationFrontier::target_profile_key` returns `&'static str`; and `physical.rs` threads the same type into its schedule verifier. `grep -rn "target_profile_key" crates/tiler-compiler/src` reports **56 sites**, and 11 of them bind it as `&'static str` in `frontier.rs`, `physical.rs`, or `selection.rs`.

**Inference: a caller-declared profile cannot have a `&'static str` key**, so this vocabulary has to move before the profile type can. That is a distinct, self-contained change with its own review surface, and doing it inside this ticket would mean landing a 57-error refactor in one commit across four files — the shape that is most likely to need unwinding.

## Revised order

1. ~~Byte pin~~ — **done**, `6e7121f`.
2. **Introduce a validated `TargetProfileKey` and move the applicability vocabulary onto it**, keeping every current caller passing the governed `&'static` key. No behaviour changes and the pin must not move. Split as `introduce-a-validated-target-profile-key`.
3. Owned `PrototypeTargetProfile` via `Cow`, and the `Copy` ripple.
4. The three rejection sites (`verify_request`, `for_target`, `physical.rs:507`) into one validation.
5. The public builder, its typed diagnostics, and the `Unknown`-on-omitted-dimension rule.

Step 2 is what makes step 3 tractable; attempted together they are one 57-error commit.

Note for step 4: `distinguish-the-five-compile-failure-classes` landed on 2026-07-27 and split `CompileFailureClass::Unsupported` into `InvalidRequest` and `UnsupportedCapability`. `InvalidRequest` is currently **unreachable from the public surface** precisely because `compile` builds the request structure itself — an empty target set and a duplicate profile are two of its five sources. This ticket is what makes it reachable, so step 4 adds construction paths rather than also having to widen the failure vocabulary.
