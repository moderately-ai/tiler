---
id: decide-the-tiler-metal-public-facade-surface
title: Decide the tiler-metal public facade surface
status: in-progress
priority: p1
dependencies: [prototype-metal-kir-lowering, check-synchronization-realization-before-the-routing-commit, carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit]
related: [choose-one-owner-for-apple-target-vocabulary, realize-parallel-reduction-strategies-on-metal, honor-the-precise-fp32-metal-compilation-requirement, apply-the-accepted-tiler-metal-public-facade]
scopes: [implementation/metal, implementation/metal-aot, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, metal, facade]
claimed_from: todo
assignee: worker-metal-facade
lease_expires_at: 1786977676
---
## User-visible outcome

The `tiler-metal` crate has one accepted exact public facade, or one explicit typed deferral with a reconsideration trigger. Its crate-level and module-level maturity statements no longer leave a consequential public boundary orphaned behind terminal implementation tickets.

## Exact-current discovery — repaired 2026-08-17 at `73af3a9a484320891553d3d575926b349ecb6b93`

- **False — the whole crate is not one undifferentiated draft.** The current `crates/tiler-metal/src/lib.rs` says `Most public items in this crate are reviewed *draft* boundaries` and immediately records `direct_requirement` as an accepted exception. `tickets/validate-macos-metal-profile-host-applicability.md`, anchor `Tom accepted the reviewed boundary as merged`, separately records Tom's acceptance of the exact `tiler_metal::applicability` packet at `6c1cd1e`. The original anchor `Every public item in this crate is a reviewed *draft* boundary` no longer exists.
- **Verified — the newly consumed synchronization subset remains held.** `crates/tiler-metal/src/synchronization_requirement.rs`, anchor `exact surface returns to Tom`, still says its exact public API is not accepted.
- **False as an architectural-open-question claim; verified only that the historical proposal is terminal.** `tickets/prototype-metal-kir-lowering.md`, anchors `whole public surface of tiler-metal` and `remain open for Tom`, is `done`. Current source and contracts have since settled both questions it left open: `emit_translation_unit` accepts a portfolio slice and emits the deduplicated zero-or-more-entry set as one translation unit, while `MetalTargetFacts::buffer_binding_limit` is the emitter-owned source-realization limit. `tiler-build` separately projects the compiler's offered `BufferBindings` capacity from the same authoritative row and rejects any declaration whose compiler capacity exceeds the emission limit. The remaining question is exact Rust exposure, not either semantic ownership rule.
- **False — a live owner now exists; verified only that no queue row exists.** This ticket is `in-progress` under the live `worker-metal-facade` claim. `.ticketsplease/decision-queue.md` contains no row for this facade, intentionally: the coordinator has held this packet behind the single presented LiveRow decision rather than presenting it concurrently.

This is not authority to delete the draft labels. Their statement is truthful; the missing piece is the live decision or explicit deferral they promise.

## Current-base correction — 2026-08-17 at `d002cd55406522922e5eb750c8c4d9033dde4469`

The discovery's blanket maturity verdicts were stale. `tiler_metal::applicability` is the exact packet Tom accepted at `6c1cd1e`; `direct_requirement` is the exact packet Tom accepted through `carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`, including its landed final two visibility narrowings. `synchronization_requirement` still has no exact-surface acceptance. Later contracts settle buffer-capacity ownership and require multi-kernel translation-unit consumption semantically; only their exact Rust facade remains undecided. This ticket therefore re-audits the complete current public census rather than inheriting the four discovery bullets.

## Required decision packet

- Re-audit every public module, re-export, type, trait, function, constructor, accessor, error, and exhaustiveness promise at the exact packet base, including the direct-requirement, synchronization-requirement, applicability, target, emission, record, diagnostic, and target-correspondence surfaces and every cross-crate caller.
- Reconcile each surface with ADR 0074 section 7 and ADR 0075. Separate already accepted exact subsets from merely implemented drafts; a terminal implementation ticket is not acceptance provenance.
- Apply the Pareto-complete decision gate to the whole-facade survivors: accept the current facade, minimize it, split accepted narrow modules from a deferred remainder, keep a labelled draft with an explicit trigger, or any materially distinct current-source alternative. Eliminate cosmetic variants and any option that moves target, compiler, runtime, or device authority into this source-emission crate.
- Fix the exact included and excluded Rust surface, compatibility posture, error vocabulary, host-memory/runtime consequences, and downstream migration. State the strongest counterargument, reversal evidence, and independent subject perturbations for every survivor.
- Decide the two unresolved prototype questions from current evidence rather than inheriting their 2026-07 wording. Include every subsequently added public module so the result is not example-shaped around the original emitter.
- Update crate/module maturity prose, decision/navigation catalogs, and graph state only after Tom accepts an exact surface. If deferral survives, record it in `deferred` with a `## Trigger check log`; do not queue a non-ready packet.

## Exact-base public-surface audit — 2026-08-17

Base: `73af3a9a484320891553d3d575926b349ecb6b93`.

Read before deriving this packet: repository `AGENTS.md`; the ticketsplease skill; this ticket; all three dependency tickets; both related tickets; the acceptance records named below; `docs/README.md`; the complete ADR index; accepted ADRs 0002, 0043, 0049, 0053, 0074, 0075, 0076, 0077, 0086, 0090, and 0092; `docs/architecture.md`; `docs/backends/metal.md`; the first-macOS Metal compile-profile authority ledger; every public module file in `crates/tiler-metal/src` plus its private target correspondence; the public correspondence inputs in `crates/tiler-metal-aot/src/{lib,input,diagnostic}.rs`; the root `tiler` facade; and every out-of-crate use of `tiler_metal::` under `crates/` and `prototypes/`, including the construction and consumption paths in `tiler-build`, `tiler-conformance`, and the two runtime prototypes. Searches located sites; the verdicts below come from reading their enclosing construction, validation, refusal, and consumption paths.

### Fact verdicts

1. **False — the whole crate is not one draft.** The current crate root says `Most public items`, not `Every public item`, and names the accepted direct-requirement exception. The current applicability surface composes Tom's accepted host-applicability packet at `6c1cd1e`, the accepted exhaustive-family/raw-constant correction recorded by `close-the-metal-gpu-family-out-of-crate-total-map`, and the accepted fallible observer from `decide-the-unnameable-gpu-enumerator-channel`. `express-metal-honourability-in-the-shared-form` separately ratified `MetalSubnormalArithmetic::subnormal_mode` as the owner-side total projection used by the accepted bounded F32 adapter. The ticket's original whole-crate verdict and its anchor were both false.

2. **Verified — synchronization is still a draft.** `synchronization_requirement.rs`, anchor `exact surface returns to Tom`, remains explicit. Its authority is nevertheless complete: `evaluate_synchronization` first maps the whole neutral subject into the exact kernel spelling and then calls the same private `barrier_realization` used by emission. It invents no device fact and exposes no second spelling authority.

3. **False in part — the prototype is terminal, but its two alleged architectural questions are closed.** `emit_translation_unit` takes a borrowed slice, sorts by whole `CanonicalKernelIdentity`, removes exact duplicates, accepts the empty set, and emits one entry per distinct member. `docs/architecture.md`, anchor `Multi-kernel and multi-entry programs are general end to end`, records that zero-or-more contract; `docs/backends/metal.md`, anchor `aggregates all entry points needed`, records the one- or multi-kernel build use. The emitter checks `buffers + input extents` separately for each entry against `MetalTargetFacts::buffer_binding_limit`; `BoundMetalCompileDeclaration` checks the compiler's `BufferBindings` offer is no greater than that emission limit. The two values are projections of one authoritative row, not competing owners.

4. **False in part — this ticket is now the live owner.** The claim is live and `in-progress`. The absence of a decision-queue row is true at this base and intentional, because LiveRow is the single presented decision; it is not evidence that the facade lacks an owner.

5. **Verified — there is no hidden public re-export.** The seven root modules are the only `tiler-metal` public module entries: `applicability`, `diagnostic`, `direct_requirement`, `emit`, `record`, `synchronization_requirement`, and `target`. `target_correspondence`, unit tests, applicability tests, and golden compilation are private or test-only. The root `tiler` facade does not re-export `tiler-metal`.

6. **Verified — the Apple target vocabulary remains deliberately duplicated.** `MslLanguageVersion`/`MetalPlatform`/`MetalDeploymentMinimum` and AOT's `MslVersion`/`ApplePlatform`/`DeploymentMinimum` have different owners and dependency requirements. The two variant vocabularies are exhaustive under ADR 0074 convention 5b; the private development-only correspondence maps 12 language revisions and 10 platforms totally in both directions and compares the deployment components. No shared crate or normal dependency edge is justified.

7. **False current conformance claim found during the census — `MetalNumericalRequirement` cannot remain non-exhaustive.** `tiler-build::metal_assembly::validate_numerical_selection` must map each emitted requirement to the AOT selection that honours it. It currently has arms only for `SafeMathMode` and `NoFloatingPointContraction`; the required wildcard maps `PreciseFp32Functions` to `false`, so an elementary-function unit is refused even when `Fp32Functions::Precise` is selected. Same-crate `golden_compilation::realization_honours` has the truthful three-arm exhaustive map. This is ADR 0074 convention 5b exactly: a wildcard cannot derive the correct flag from an unknown requirement. The current whole facade is therefore not an admissible survivor.

8. **Verified — no facade-owned canonical identity or schema exists.** The unit borrows whole kernel identities and emitted source later enters the existing AOT payload/artifact identities. Module visibility, enum exhaustiveness, and method visibility are not encoded. The proposed correction changes no emitted source, target-profile descriptor, artifact grammar, cache subject, domain tag, schema version, or existing pin. Its only behavioural correction is whether the already-declared precise compiler selection is correctly accepted; the remaining deltas are source compatibility and maturity.

## Exact public census and maturity

The current root modules remain public; module ownership is useful and no root-level aliases are added.

| module | disposition | exact authority |
| --- | --- | --- |
| `applicability` | already accepted; unchanged | device-free host observation/policy and the fallible highest-family walk; no profile construction or Metal device object |
| `direct_requirement` | already accepted; unchanged | the sole public comparison of derived index arithmetic with normalized family observation |
| `target` subnormal projection | already accepted subset; unchanged | `MetalSubnormalArithmetic::subnormal_mode` and the `MetalTargetFacts` reachability needed by the bounded build adapter |
| `target`, `emit`, `record`, `diagnostic` remainder | accept after the exact minimization below | pure structured-kernel-to-MSL translation, explicit target inputs, immutable output, and typed refusals |
| `synchronization_requirement` | accept exactly as below | pure direct-requirement comparison sharing emission's private barrier authority |

Acceptance does not make any target fact observed, make the native translator attributable, admit a runtime route, or stabilize crates for external release. ADR 0075's pre-alpha posture remains: a later source break is cheap, explicit, and reviewed; acceptance says the current boundary is intentional rather than accidental.

## Exact proposed facade

An implementation that publishes an extra constructor, spelling helper, re-export, catch-all total map, target observer, runtime object, compatibility shim, or identity encoding has left this proposal and must stop for review.

### Already accepted modules, byte-for-byte public shape unchanged

- `tiler_metal::applicability`: exhaustive `MetalGpuFamily::{Apple5, Apple6, Apple7, Apple8, Apple9}` with `ALL`, `COUNT`, `as_str`, `apple_constant`, and `Display`; opaque `AppleGpuFamilyConstant` with `value` and `Display`; generic `try_observe_highest_gpu_family<E>(impl FnMut(AppleGpuFamilyConstant) -> Result<bool, E>) -> Result<MetalGpuFamilySupport, E>`; exhaustive `MetalGpuFamilySupport::{Highest(MetalGpuFamily), NoneNamed}`; non-exhaustive `MetalHostPredicate::{OsFamily, OsVersion, OsBuild, Architecture, DeviceName, GpuFamily, NativeTranslationAuthority}` with `ALL`, `COUNT`, `as_str`, and `Display`; `MetalHostObservation` and its six observing builders/six readers; closed `MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9` and its seven readers; uninhabited `NativeTranslationAuthority`; unforgeable `MetalHostEligibility` with `policy` and `observation`; non-exhaustive `MetalHostApplicabilityRefusal::{Unobserved, OsFamilyMismatch, OsVersionMismatch, OsBuildMismatch, ArchitectureMismatch, DeviceNameMismatch, GpuFamilyMismatch, UnknownNativeTranslationAuthority}` with `predicate`, `rule`, `Display`, and `Error`; and `evaluate_metal_host_applicability(MetalHostApplicabilityPolicy, &MetalHostObservation) -> Result<MetalHostEligibility, MetalHostApplicabilityRefusal>`. The private uninhabited authority remains an internal structural stop, not a caller argument.
- `tiler_metal::direct_requirement`: exhaustive `AppleFamilyFloor::Apple3` with `Display`; `minimum_gpu_family(IndexArithmetic) -> AppleFamilyFloor`; non-exhaustive `MetalIndexArithmeticRefusal::{UndecidableBelowVocabulary { required, floor, lowest_observable }, Unobserved { required, floor }}` with `rule`, `required`, `floor`, `Display`, and `Error`; and `evaluate_index_arithmetic(IndexArithmetic, Option<MetalGpuFamilySupport>) -> Result<MetalGpuFamily, MetalIndexArithmeticRefusal>`. `evaluate_against` and `AppleFamilyFloor::apple_constant_value` remain crate-private.
- The ratified target seam `MetalSubnormalArithmetic::subnormal_mode` remains public and total.

### `target`

- Exhaustive `MslLanguageVersion` with its 12 current variants, `ALL`, `COUNT`, `revision`, `semantic_name`, and `Display`.
- Exhaustive `MetalPlatform` with its 10 current variants, `ALL`, `COUNT`, `as_str`, and `Display`.
- `MetalDeploymentMinimum` with private `u16` components, `new`, `major`, `minor`, and `Display`.
- Non-exhaustive `LaunchIndexRealization`, currently only `ThreadPositionInGridUInt`.
- Caller-constructed `MetalEmissionRealization { pub launch_index: LaunchIndexRealization }` and `new`.
- Non-exhaustive `MetalFloatArithmeticType::{F32, F16, Bf16}` and `Display`.
- Opaque `MetalUnstatedSubnormalArithmetic` with `arithmetic_type`, `rule`, and `Display`; construction remains private to the comparison authority.
- Non-exhaustive `MetalSubnormalArithmetic::{FlushesToZero { zero_sign }, PreservesSubnormals}`, its accepted `subnormal_mode`, and `Display`.
- Caller-constructed `MetalSubnormalArithmeticFacts` with `unmeasured`, consuming `stating` (which panics/const-fails on a duplicate dtype statement), and checked `behaviour`.
- Exhaustive `MetalFlushedZeroSign::{PreservesSign, AlwaysPositive}`.
- Leaf input descriptor `MetalTargetFacts` with public fields `language: MslLanguageVersion`, `platform: MetalPlatform`, `deployment_minimum: MetalDeploymentMinimum`, `subnormal_arithmetic: MetalSubnormalArithmeticFacts`, and `buffer_binding_limit: u32`, plus the same five-argument `new`.

The capacity field stays a raw `u32`: zero is a valid fail-closed emitter fact, the governed buffer namespace is `u32`, and no invariant or unit conversion exists for a newtype to enforce. It is per entry, not a portfolio-wide sum. The compiler capacity is allowed to be lower; only `compiler > emission` is unsound and refused by the bound declaration.

### `emit`, `record`, and `diagnostic`

The one public emitter remains exactly:

```rust
pub fn emit_translation_unit(
    kernels: &[&tiler_ir::kernel::VerifiedKernel],
    target: &MetalTargetFacts,
    emission: MetalEmissionRealization,
) -> Result<MetalTranslationUnit, MetalEmitError>;
```

The slice is borrowed, accepts zero members, treats multiplicity and input order as non-semantic, and returns entries in ascending whole canonical-identity order. This is the cheapest representation of the accepted set contract: no owned kernel collection, no caller-selected entry order, no single-kernel special path, and no nonempty wrapper. Identical kernels emit once; different whole identities that collide only in their bounded symbol digest reject as `SymbolCollision`.

`record` exposes:

- exhaustive `MetalNumericalRequirement::{SafeMathMode, NoFloatingPointContraction, PreciseFp32Functions}` with `flag`, `rule`, and `Display`;
- non-exhaustive `MetalNumericalGap::{SubnormalFlushInArithmetic, SubnormalPreservationInArithmetic, FlushedZeroSignMismatch}` with `rule` and `Display`;
- immutable `MetalBufferBinding` with `index: u32` and `parameter: BufferParameter` readers and no public constructor;
- immutable `MetalEntryPoint` with `symbol`, borrowed whole `kernel_identity`, ordered `buffers`, and `input_extent_count` readers and no public constructor; and
- immutable `MetalTranslationUnit` with `target`, `emission_realization`, `source`, `entry_points`, `numerical_requirements`, `numerical_gaps`, `unstated_subnormal_arithmetic`, and `require_declared_realization`. Its fields and constructor remain private.

`diagnostic` exposes non-exhaustive `MetalOperationFamily::{Builtin, Constant, Binary, Compare, Convert}` and `Display`; non-exhaustive `BarrierRejection::{ExecutionScope { scope }, MemoryVisibility { execution, memory }, FencedSpace { space }, Ordering { ordering }}` and `Display`; and non-exhaustive `MetalEmitError` with exactly these variants:

- `UnsupportedAddressSpace { space }`;
- `UnsupportedBufferAccess { space, access }`;
- `UnsupportedOperation { family }`;
- `UnsupportedValueType { value_type }`;
- `UnrecognizedOperation`;
- `UnsupportedBarrier { reason }`;
- `InvalidCanonicalNan { bits }`;
- `UnrealizableNumericalObligation { gap }`;
- `UnstatedSubnormalArithmetic { unstated }`;
- `BufferBindingLimit { required: usize, limit: u32 }`;
- `MalformedKernel { rule: &'static str }`;
- `UnresolvedValue`;
- `SymbolCollision { symbol: String }`; and
- `Handle(VerifiedKernelHandleError)`.

`MetalEmitError::rule`, `Display`, `Error`, and `From<VerifiedKernelHandleError>` remain public. Only `Handle` has an `Error::source`.

### `synchronization_requirement`

Accept the current exact surface:

```rust
pub fn evaluate_synchronization(
    required: Option<tiler_ir::schedule::SynchronizationSubject>,
) -> Result<(), MetalSynchronizationRefusal>;
```

`None` is the canonical no-requirement value and returns `Ok(())`. Non-exhaustive `MetalSynchronizationRefusal` retains exactly `UnadmittedKind { required, kind }`, `UnspellableExecutionScope { required, scope }`, `UnspellableVisibilityScope { required, scope }`, `UnspellableOrdering { required, ordering }`, and `Unrealizable { required, reason }`, plus `rule`, `required`, `Display`, and `Error`. Private `spell` remains the only schedule-to-kernel mapping; private `barrier_realization` remains the only Metal realization authority. No device observation, caller-selected spelling, or route row is added.

### Public spellings deliberately removed

Make these eight currently public, out-of-crate-unused backend spelling helpers crate-private:

- `LaunchIndexRealization::{attribute, declared_type}`;
- `MetalFloatArithmeticType::{ALL, COUNT, as_str}`;
- `MetalSubnormalArithmetic::as_str`;
- `MetalOperationFamily::as_str`; and
- `BarrierRejection::as_str`.

Their public `Display`, structured variants, and outer stable `rule` accessors preserve every diagnostic and inspection use. The methods being narrowed expose implementation text or duplicate `Display`; none is a construction seam or current consumer requirement. Retaining them would enlarge the accepted compatibility surface without enabling a distinct supported caller.

No public constructors are added for `MetalTranslationUnit`, `MetalEntryPoint`, `MetalBufferBinding`, `MetalUnstatedSubnormalArithmetic`, `MetalHostEligibility`, or `NativeTranslationAuthority`. No root re-export aliases are added. The private correspondence and golden/toolchain helpers remain private.

All current public trait implementations and derives are preserved exactly. No public type gains `Default`, conversion, parsing, serialization, or arbitrary construction. The refusal/error types retain their current `Display` and `std::error::Error` implementations, and only the documented `Handle` wrapper exposes an error source.

## Diagnostics and precedence

- Portfolio members are sorted by whole identity before validation. A fallible emission reports the first error reached in the first sorted distinct kernel; caller order and duplicate multiplicity cannot change it.
- Per entry, binding capacity is checked before body translation. The required count is the saturating sum of declared buffers and input extents and is compared in widened arithmetic to the `u32` limit. No overflow can turn an over-limit signature into admission.
- Emission returns no partial source. Helper, numerical-requirement, gap, unstated-type, entry, and body collections become public only after every member emits and bounded symbol uniqueness holds.
- `MetalTranslationUnit::require_declared_realization` reports the first governed unstated arithmetic type before any numerical gap, because a missing fact makes the gap comparison incomplete. Otherwise it reports the first gap in governed order.
- `evaluate_synchronization` refuses neutral kind, execution-scope, visibility-scope, and ordering spelling gaps before calling Metal realization; a spelled but unrealizable barrier becomes `Unrealizable` carrying the exact `BarrierRejection`.
- The AOT selection consumer must map all three `MetalNumericalRequirement` variants exhaustively. `PreciseFp32Functions` is honoured exactly by `Fp32Functions::Precise`; no wildcard, default flags, or string parsing remains.

## Pareto-complete frontier

| candidate | correctness and strictness | maintainability/compatibility | host runtime and memory | disposition |
| --- | --- | --- | --- | --- |
| Current whole facade verbatim | fails ADR 0074 5b and already refuses a valid precise selection through a wildcard | preserves eight unused helpers | no migration | eliminated: silently incomplete total map |
| Accepted modules only; keep all other modules draft | correct today if every caller remembers each draft contract | leaves the heavily consumed emitter and synchronization boundaries ownerless and repeats this audit later | identical code, layout, allocation, and dispatch | dominated by the exact minimized acceptance |
| Accept emitter core; defer synchronization | correct, but splits two instances of the same direct-requirement ownership rule without an unresolved synchronization fact | another maturity class and follow-up for a complete, consumed, tested two-item surface | identical | dominated |
| Accept every current item after only fixing precise mapping | correct | commits eight unused backend spelling helpers and duplicate text accessors | identical | dominated by minimization |
| **Minimized whole facade above** | correct, fail-closed, exact total maps; preserves all current supported consumers | one intentional facade, smallest current surface, no compatibility shim | one existing match arm; zero layout/allocation/source-byte change | **sole nondominated survivor** |
| Move translation behind `tiler-build` or a new facade crate | can be made correct only by moving/duplicating target and source-emission authority and by cutting direct IR/conformance consumers off | adds an owner and dependency edge contrary to the accepted crate profile | larger build graph; no kernel benefit | eliminated |
| Typed deferral of the whole remainder | correct but not more explicit than the already-labelled draft and has no missing evidence to name | postpones a complete decision and leaves every current cross-crate use provisional | identical | dominated; no truthful trigger beyond “decide later” |

No genuine product-priority trade-off remains. The recommendation is the sole nondominated minimized facade. Tom still owns whether to accept that exact public boundary; rejection retains the current labelled draft and authorizes no production edit.

### Strongest counterargument and reversal evidence

The strongest counterargument is that accepting the emitter and direct synchronization evaluator together makes a comparatively broad low-level crate intentionally public while the ordinary build path is already wrapped by `tiler-build`. A real external consumer that needs only the build orchestrator and never verified-kernel emission, combined with evidence that direct emission prevents future target-fact or output-record redesign, would reverse the recommendation toward an orchestrator-only facade. No such consumer exists: current `tiler-build`, `tiler-conformance`, and both retained prototypes directly consume the Metal types and emitter, and moving them would change authority rather than hide implementation.

Evidence for restoring any narrowed helper is one concrete out-of-crate consumer whose supported work requires the exact structured value and cannot use the retained variant/accessor/`Display` surface without parsing. Evidence for splitting synchronization is a new unresolved authority dimension — for example a real device-dependent synchronization fact — that makes the current pure subject-only evaluator incomplete. Neither exists at this base.

## Consequences and unsupported population

- **Identity/schema:** no domain, encoder, manifest schema, payload schema, cache grammar, artifact identity, source byte, or pin moves. The precise arm changes only a host-side acceptance result for an already identical request. If an implementation changes emitted bytes or an encoded record, it has exceeded this decision.
- **Host runtime:** facade acceptance and visibility changes execute nothing. The precise arm is one branch in the existing requirements scan. Device dispatch, pipeline creation, command buffers, and native translation stay outside this crate.
- **Host memory:** enum attributes and method visibility have zero layout cost. The portfolio continues to allocate an ordered vector of borrowed pointers, output strings, and ordered sets proportional to distinct kernels/helpers/requirements; the decision adds no allocation or retained byte.
- **Target facts:** all are caller statements. This facade does not observe a GPU, infer a compiler profile, attest native translation, or turn source compilation into capability evidence.
- **Portfolio:** zero kernels is legal and yields a declaration-free unit with the deterministic prologue; duplicates collapse; multiple distinct kernels share helpers and one prologue. There is no caller-controlled order or multiplicity identity.
- **Capacity:** binding capacity is per entry, covers buffers plus input-extent metadata in the same MSL buffer namespace, and is independent of portfolio size. Compiler capacity may be conservatively lower and may never exceed emission capacity.
- **Numerics:** producing source is not the final conformance claim. A consumer must call `require_declared_realization` and must satisfy every exhaustive compiler-selection requirement before compilation. No missing dtype fact, gap, or new requirement defaults.
- **Unsupported:** unrecognized KIR vocabulary, unsupported address spaces/access combinations/value types/operations/barriers, invalid NaN declarations, over-capacity signatures, malformed or unresolved verified inputs, bounded-symbol collisions, unstated arithmetic facts, unrealizable numerical obligations, every neutral synchronization kind/scope/ordering with no exact kernel spelling, and every spelled barrier Metal declines remain typed refusals. There is no runtime fallback, source JIT, device query, alternative target, repair, or best-effort output.
- **AOT vocabulary:** `tiler-metal-aot` retains its empty dependency closure and its own exhaustive language/platform vocabulary. The development-only total correspondence remains the build-failing guard; no production crate edge or shared target-vocabulary crate appears.

## Required repair, implementation, and evidence

No production edit is authorized by this packet. The independent P0 `honor-the-precise-fp32-metal-compilation-requirement` owns the already-required consumer correction regardless of Tom's facade answer: add the exact `PreciseFp32Functions => numerical.fp32_functions == Fp32Functions::Precise` arm and prove the precise-positive/fast-negative request and prepared-payload cases without changing public Rust. It is not a facade implementation and need not wait for acceptance.

Only after Tom accepts this exact surface, `apply-the-accepted-tiler-metal-public-facade` must:

1. remove `#[non_exhaustive]` from `MetalNumericalRequirement` and prove every out-of-crate consumer remains an explicit total three-variant map;
2. narrow exactly the eight helpers above, with external compile-fail evidence for at least the two authority-bearing launch spellings and positive evidence that retained `Display`/structured routes suffice;
3. change crate/module draft prose only for the exact accepted modules/items and preserve the already accepted provenance separately;
4. keep target correspondence exhaustive and use `core::mem::variant_count`/typed `ALL` inventories for the language, platform, GPU-family, and arithmetic-type populations where applicable;
5. perturb `MetalNumericalRequirement` with a temporary fourth variant and record both the emitter's derivation and `tiler-build`'s AOT map failing to compile; independently perturb the already-landed precise AOT selection; perturb empty, duplicate, order-reversed, and two-distinct-kernel portfolios; lower the emission capacity below one entry's `buffers + extents`; perturb each synchronization subject dimension independently; and restore every subject before gates; and
6. run focused Metal/build/conformance tests, external API/doc tests, Clippy, rustdoc, both workspace test modes, then the repository publication gate proportional to the production change.

Reopening triggers after acceptance are a real external consumer, a publishable crate/release boundary, a fused operation that invalidates translation-unit-wide numerical realization, a device-dependent synchronization authority, a second launch-index realization, a target fact that cannot remain a caller input, or an AOT target-vocabulary member for which the checked correspondence has no total answer.

## Presentation hold

This packet remains `in-progress`; it is neither presented nor moved to `awaiting-decision`. LiveRow is the one current Tom question. After independent exact-commit review, the coordinator may record this packet at the end of the existing queue, behind items 6 through 12, and later present only the accept-or-reject question for this exact minimized surface. Acceptance provenance must be written here before any implementation carrier is unblocked.

## Stop boundary

This ticket is decision research only. It authorizes no public signature change, module move, compatibility shim, target-vocabulary consolidation, or production implementation before the exact packet passes independent review and Tom accepts it.

## Closes when

One exact current-source facade packet passes independent review and Tom accepts it, or a typed deferral records the evidence and trigger that makes future presentation actionable. Every live draft label then has a live owner or accepted disposition.
