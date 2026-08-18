---
id: apply-the-accepted-tiler-metal-public-facade
title: Apply the accepted tiler-metal public facade
status: in-progress
priority: p1
dependencies: [decide-the-tiler-metal-public-facade-surface, honor-the-precise-fp32-metal-compilation-requirement]
related: []
scopes: [implementation/metal, implementation/build, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-facade-application
lease_expires_at: 1787088563
---
## User-visible outcome

After Tom accepts the exact packet in `decide-the-tiler-metal-public-facade-surface`, source visibility, exhaustiveness, maturity prose, tests, and navigation agree with that one accepted facade. No draft item is promoted by implication.

## Implementation keys

- Re-read the decision ticket and current source at the implementation base. Apply only the exact accepted delta; if Tom amends or rejects the packet, update or close this carrier rather than interpreting the answer.
- Preserve the already accepted `applicability`, `direct_requirement`, and subnormal-projection surfaces. Remove `#[non_exhaustive]` from `MetalNumericalRequirement`; keep its three-variant map exhaustive in every out-of-crate total consumer.
- Narrow only the eight methods the accepted packet names. Add external negative evidence that authority-bearing spelling helpers are unreachable and positive evidence for every retained public inspection route.
- Update crate/module maturity prose and the hand-maintained decision/navigation catalogs only after acceptance provenance is recorded. Do not create a compatibility shim or root re-export alias.
- Preserve portfolio set semantics, per-entry binding capacity ownership, diagnostic precedence, target/AOT vocabulary correspondence, emitted source, artifact/cache identities, schemas, domains, and pins.

## Required evidence

Use typed/`variant_count` censuses for the exact language, platform, GPU-family, arithmetic-type, requirement, diagnostic, and synchronization populations where applicable. Perturb the requirement type with a temporary extra variant and record both the emitter derivation and build adapter total map failing. Independently perturb empty, duplicate, reverse-order, two-distinct-member, capacity, and synchronization subjects. Run focused package and external-API tests, Clippy, rustdoc, both workspace test modes, ticket lint, citations, exact-base guard, and the proportional repository publication gate.

## Stop boundary

Blocked until the decision dependency is `done` with Tom's exact acceptance recorded. This carrier authorizes no target-fact redesign, emitted-byte change, crate move, new dependency edge, runtime/device surface, public constructor for an opaque output, or AOT vocabulary consolidation.

## Closes when

The accepted public census is mechanically exact, every excluded item is externally unreachable, all maturity and catalog prose names only accepted subsets, independent exact-commit review passes, and the implementation is integrated with no identity/schema/source-byte movement.

## Census re-audit — 2026-08-18 at base `1ae823c9b3d0f9d6e58fdf27ee4841d62af84782`

The accepted packet was reviewed at `459541d239e391eec24efdef6d2e46a612e16d0d`; three landings postdate it (subgroup declaration in tiler-build, elementary-dimensions in tiler-metal emit/tests, partitioned-copy in tiler-metal lib/tests). Every packet census claim was re-verified at this base before editing; files were read in full, greps located sites only.

- **Verified — seven public modules, no hidden re-export.** `crates/tiler-metal/src/lib.rs` declares exactly `applicability`, `diagnostic`, `direct_requirement`, `emit`, `record`, `synchronization_requirement`, and `target` public; `golden_compilation`, `target_correspondence`, and the test modules are `#[cfg(test)]`. `emit`'s only public item is `emit_translation_unit` with the packet's exact signature (`kernels: &[&VerifiedKernel]` borrowed slice); `realization_requirements`, `barrier_realization`, `reserve_symbol`, and the rest are `pub(crate)`.
- **Verified — all eight narrowed helpers were still out-of-crate-unused.** Grep over every `tiler_metal`-referencing file under `crates/` and `prototypes/` (41 files) for `.attribute()`, `.declared_type()`, `MetalFloatArithmeticType::{ALL, COUNT}`, and `.as_str()` receivers of the four owning types found no consumer: every `.as_str()`/`::ALL`/`::COUNT` hit resolves to other types (`MetalGpuFamily`, `MetalHostPredicate`, AOT's `ApplePlatform`/`AppleSdk`, profile keys, plain strings). Out-of-crate uses of the four owning types are variant constructions and the retained `subnormal_mode` only. No new consumer appeared; nothing forced a stop.
- **Verified — exactly one out-of-crate `MetalNumericalRequirement` total map.** `tiler-build::metal_assembly::validate_numerical_selection` (anchor `let satisfied = match requirement`) carried three explicit arms plus the `_ => false` wildcard the landed P0 repair had to retain while the enum stayed `#[non_exhaustive]`. The only other out-of-crate touches are `Display` uses in `payload_metadata` and matcher patterns in tiler-build's own tests; `tiler-ir/src/schedule/witness.rs` names the type in prose only. In-crate, `record.rs` `flag`/`rule` and `golden_compilation::realization_honours` are the exhaustive maps. The subgroup-declaration module (`metal_subgroup_declaration.rs`) references no `tiler_metal` item.
- **Verified — target/record/diagnostic/synchronization censuses match the packet.** `MslLanguageVersion` 12 variants, `MetalPlatform` 10, both with `variant_count`-sized `ALL`; `MetalFloatArithmeticType` 3; `MetalNumericalRequirement` 3; `MetalNumericalGap` 3; `MetalOperationFamily` 5; `BarrierRejection` 4; `MetalEmitError` 14 variants with only `Handle` carrying an `Error::source`; `MetalSynchronizationRefusal` 5 with `rule`/`required`; `MetalGpuFamily` still 5 (Apple10 widening remains deferred).
- **Verified — the maturity prose to update.** The crate root said `Most public items in this crate are reviewed *draft* boundaries`; `target.rs`, `record.rs`, `diagnostic.rs`, `emit.rs`, and `applicability.rs` each carried an `Every public item here is a reviewed *draft* boundary` line; `synchronization_requirement.rs` carried `exact surface returns to Tom`. `docs/status.md` (anchor `The remaining whole-\`tiler-metal\` public-facade maturity decision`) still pointed at the decision as remaining; `.ticketsplease/decision-queue.md` row 13 already records the acceptance and needed no edit; no other doc or ticket names the decision as live.

## Delivery record — 2026-08-18

Source delta (commit `bfe78523f8cc6f4824e4ffcd7371f094096e1c72`, docs/ticket delta follows on the same branch):

1. `record.rs`: removed `#[non_exhaustive]` from `MetalNumericalRequirement` and documented the 5b classification (a wildcard has no correct verdict in either direction; the pre-repair precise refusal is cited as the observed failure).
2. `metal_assembly.rs` (tiler-build): deleted the `_ => false` wildcard so the three-variant selection map is total, with a why-comment; no pin, no `metal_plan.rs`, no artifact codec or runtime file touched.
3. `target.rs`: `LaunchIndexRealization::{attribute, declared_type}`, `MetalFloatArithmeticType::{ALL, COUNT, as_str}`, and `MetalSubnormalArithmetic::as_str` are `pub(crate)`; `diagnostic.rs`: `MetalOperationFamily::as_str` and `BarrierRejection::as_str` are `pub(crate)`. Each owning public type carries external `compile_fail,E0624` doctests (rustdoc compiles doctests as an external crate) plus a positive doctest for the retained route: `Display` renders the identical stable text for `MetalFloatArithmeticType` (`"bf16"`), `MetalSubnormalArithmetic` (`"preserves-subnormals"`), `MetalOperationFamily` (`"builtin"`), and `BarrierRejection` (`"ordering"`), and the launch selection stays structurally matchable through `MetalEmissionRealization.launch_index`. The applicability compile-fail set (no forged `NativeTranslationAuthority`, no profile-ref or byte arguments) and the direct-requirement `apple_constant_value` compile-fail all still pass, covering every retained public inspection route's negative twin.
4. Maturity prose: every module's draft line replaced by an accepted-boundary record naming Tom's 2026-08-18 acceptance and ADR 0075's accepted-is-not-stabilized posture; `applicability.rs` and the crate root preserve the earlier separate acceptance provenance (`6c1cd1e` packet, family/raw-constant correction, fallible observer, `direct_requirement`, `subnormal_mode` ratification) distinctly from the whole-facade acceptance. `docs/status.md` gained a dated correction quoting the retired "remaining decision" sentence.
5. Nothing else: no compatibility shim, no root re-export alias, no new constructor, no public trait or derive change, no emitted-byte/identity/schema/domain/pin movement — the pin/golden tests ran unmodified and green (245/245 in tiler-metal + tiler-build).

## Perturbation evidence — 2026-08-18

Every perturbation edited the subject, never an assertion; each was reverted and the tree verified byte-identical to `bfe78523` (`git status --porcelain` and `git diff HEAD --stat` both empty) before the next.

- **Fourth requirement variant, unarmed:** `cargo check -p tiler-metal` fails `error[E0004]: non-exhaustive patterns: MetalNumericalRequirement::TemporaryFourthRequirement not covered` at `record.rs` `flag` (match at :140) and `rule` (:150) — the crate's own flag/rule derivation refuses the widening.
- **Fourth variant, armed in-crate:** `cargo check -p tiler-build` fails `error[E0004]` at `metal_assembly.rs:344` (`let satisfied = match requirement`) — the out-of-crate total map is the build error the exhaustiveness exists to force; `cargo check -p tiler-metal --all-targets` additionally fails at `golden_compilation.rs:359` (`realization_honours`).
- **Precise AOT selection:** flipping the landed arm to `Fp32Functions::Fast` fails `elementary_request_and_preparation_require_precise_fp32_functions`: `panicked at crates/tiler-build/src/metal_assembly.rs:883: the precise selection satisfies the elementary unit: UnsatisfiedNumericalRequirement { requirement: PreciseFp32Functions }`.
- **Empty portfolio:** refusing an empty `ordered` in `emit_translation_unit` fails `an_empty_portfolio_emits_a_declaration_free_translation_unit`: `called Result::unwrap() on an Err value: UnresolvedValue`.
- **Duplicate portfolio:** removing `order_kernels`' `dedup_by` fails `repeating_a_kernel_emits_one_entry_point`: `assertion left == right failed: left: 3, right: 1`.
- **Order-reversed portfolio:** removing `order_kernels`' `sort_by` fails `entry_points_are_ordered_by_canonical_identity` (`assertion failed: identities.windows(2).all(|pair| pair[0] < pair[1])`) and `portfolio_order_does_not_change_emitted_bytes`, whose printed both-sides diff shows caller order reordering the two `kernel void` bodies in the emitted source.
- **Two-distinct-kernel portfolio:** `ordered.truncate(1)` fails `a_portfolio_shares_one_prologue_and_one_helper`: `assertion left == right failed: left: 1, right: 2` at the `kernel void ` count.
- **Capacity:** widening the `emit_entry_point` comparison by one admits the two-binding signature against limit 1 — `a_signature_exceeding_the_binding_table_is_rejected` fails `called Result::unwrap_err() on an Ok value: MetalTranslationUnit { ... buffer_binding_limit: 1 ... }` (the test itself already lowers the fixture's capacity below one entry's `buffers + extents`).
- **Synchronization, per dimension independently:** admitting `Atomic` as a barrier kind fails `every_unadmitted_kind_is_refused_by_name` (`atomic has no Metal construct: left: Ok(())`), plus the census (`only the fence dimension is free: left: 8, right: 4`) and the emitted-text sweep; rounding a `Device` arrival to `Workgroup` fails `a_device_wide_arrival_has_no_kernel_spelling` (`left: Ok(()), right: Err(UnspellableExecutionScope { .. scope: Device })`); rounding a `Subgroup` publication to `Workgroup` fails `a_subgroup_publication_has_no_kernel_spelling` (`right: Err(UnspellableVisibilityScope { .. scope: Subgroup })`); weakening `SequentiallyConsistent` to acquire-release fails `no_ordering_but_acquire_release_has_a_kernel_spelling` (`sequentially-consistent has no BarrierOrdering spelling: left: Ok(())`); dropping the workgroup fence derivation in `spell` fails `the_derived_staged_handoff_is_realized` (`left: "threadgroup_barrier(mem_flags::mem_none);", right: "threadgroup_barrier(mem_flags::mem_threadgroup);"`) and `every_admitted_subject_reaches_emitted_text` (`left: [], right: [Workgroup]`).

## Commands — 2026-08-18

`cargo check -p tiler-metal -p tiler-build`; `cargo clippy -p tiler-metal -p tiler-build --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-metal`; `cargo test -p tiler-metal --doc` (10 pass + 13 compile-fail pass); `cargo nextest run -p tiler-metal -p tiler-build` (245/245); `cargo fmt --check`; then on the completed delta `cargo nextest run --workspace`, `cargo test --workspace --doc`, `tkt lint`, `make citations`, `git diff --check`, and `tkt guard tkt/apply-the-accepted-tiler-metal-public-facade --format json` — results recorded in the worker report. Status stays `in-progress` pending the independent exact-commit review the close condition requires.
