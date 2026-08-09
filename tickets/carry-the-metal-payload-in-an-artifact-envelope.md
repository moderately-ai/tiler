---
id: carry-the-metal-payload-in-an-artifact-envelope
title: Carry the Metal payload in an artifact envelope and round-trip it
status: done
priority: p0
dependencies: [assemble-the-metal-payload-from-emission-and-compilation, relocate-abi-expressions-into-tiler-ir, name-the-resolved-lowering-capability, carry-the-target-profile-descriptor-identity-into-the-plan, split-profile-and-feasibility-rule-identity]
related: [route-the-runtime-proof-through-the-artifact-envelope, prototype-public-compiler-api]
scopes: [implementation/metal-aot, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
---
`assemble-the-metal-payload-from-emission-and-compilation` produced a real `PayloadContent` from a real emission and compilation, and stopped there. This ticket carries it into an envelope and proves the envelope round-trips.

**Fact — why the parent could not finish it.** `ArtifactProgramBuilder` needs, per variant, the `VerifiedKernelProgram` the plan packages, its selected capability providers, its ABI expression arena, its guard, its per-entry bindings and launch contract, and its declared target profile and feasibility rule set. Every one of those reaches a caller only through `ProgramAlternative::artifact_plan`, which is `pub(crate)` in `tiler-compiler` and is not exposed on the `session` boundary. `PlanAlternative` today exposes `stable_id`, `is_fused`, and `kernels` — enough to emit, not enough to package.

**The surface decision this ticket cannot avoid.** Exposing the artifact plan is an ADR 0075 promotion and is Tom's. It is also a genuine design choice rather than a mechanical one: the plan is a large internal record, and handing it out whole would commit the public boundary to its exact shape while the compiler is still moving. The alternative is a narrower seam — `PlanAlternative::build_artifact(&mut ArtifactProgramBuilder)` or similar — that lets the compiler populate a builder the caller owns, exposing the *capability* without the record. Put both to Tom with the trade-off before implementing either.

## The work, once that is settled

Push the carried payload through `push_carried_payload`, build the artifact, encode the envelope, and then **decode the bytes back** and verify. The round trip is the point: identity re-derived from decoded content must equal the identity the manifest carries, the re-encode must be byte-identical, and every closure check must pass. A payload that assembles but does not survive a round trip is not carried.

Assert specifically that the two payload sections appear with the right purposes — `BackendPayloadMetadata` and `BackendPayloadCode` — that the descriptor digest equals `payload_identity` of the metadata bytes, and that the derived feature set contains `tiler.artifact.feature.embedded-payload-code`. Those are the properties the codec already tests against synthetic content; this is where they meet a real compilation.

## Do not

Do not weaken any codec check to make a real artifact fit. If a real bundle is refused, that is either a defect in the assembly or a gap in the model, and both are worth finding — the codec's adversarial cases exist precisely so a well-formed-looking forgery cannot pass, and a real payload should not need an exception.

## Decision — the orchestrator assembles; the compiler exposes a typed view of its construction inputs

Two options were weighed and one was **recommended and then withdrawn**, so the reasoning is recorded rather than the conclusion alone.

**Withdrawn: `PlanAlternative::build_artifact(&mut ArtifactProgramBuilder, …)`**, letting the compiler populate a builder the caller owns. It was recommended first on the grounds that it "matches how this codebase works — checked builders that populate rather than records handed out." That was taste, not evidence, and two facts defeat it.

**Fact — `crates/tiler-compiler/Cargo.toml` declares `tiler-ir` as its only dependency.** So this is not a visibility change at all; it is a **new dependency edge** `tiler-compiler → tiler-artifact`, and a durable one. Edges are far harder to remove than functions: once it exists every artifact concept is reachable from the compiler and later work will reach for it.

**Fact — `docs/artifact-abi.md` records the division the other way**: the private compiler proof "constructs provisional program portfolios and artifact-**construction inputs**". The type is named `ArtifactConstructionPlan`. Inputs, not artifacts. Making the compiler own packaging contradicts an accepted contract.

**Decision: expose a typed read view over the construction inputs and assemble in the orchestrator**, which is the only component that sees both the plan and the backend payload — structurally the same position as the emitter/driver target-vocabulary translation already in `prototypes/serial-sum-compile/src/target.rs`.

**Correctness is what settles it, not preference.** `ArtifactConstructionPlan` carries no ABI expressions: `routing_guard: HostExprId` is an index into the *compiler's* arena and there is no expression arena, binding, or launch contract in it. So whoever assembles must derive accessible byte ranges and launch geometry. Under the withdrawn option that derivation would live in `tiler-compiler` and mint expressions into `tiler-artifact`'s arena, creating a **second** place that knows how to derive an accessible byte range beside the artifact layer's own checks. Two derivations of one fact is the drift hazard, and it is the exact class of defect this session already hit twice — the hand-written mirrors in `governed_scalar_reference.rs`, and a `tiler-ir` doc comment that contradicted `docs/numerical-semantics.md`.

**The view must not leak compiler-internal identifiers.** A caller holding a raw `HostExprId` would be re-deriving cross-references the artifact builder already owns, which is what that layer exists to prevent. The exposed view hands out resolved values; the caller mints its own expressions on its own builder.

**Performance does not decide it.** Assembly runs once per compilation, not per element; a view copy is noise against `xcrun`.

**Fact — no artifact assembler exists anywhere.** `ArtifactProgramBuilder` has no consumer outside `crates/tiler-artifact/src`. So this ticket writes the first one, and the ABI-expression derivation it needs is the same work `complete-program-identity-with-abi-guards-and-routing` describes; the two meet here and should not produce two derivations.

## The decision above is RETRACTED — accepted ADR 0068 assigns this to the compiler

The reading landed and it overturns the decision, which is recorded rather than quietly replaced.

**Fact — `docs/decisions/0068-co-locate-abi-expressions-with-executable-program-ir.md:41-42`, `decision_status: accepted`:** "`tiler-compiler` owns lowering into and construction of `AbiExpr`; it is not the runtime expression authority."

That is directly on point and more specific than the `docs/artifact-abi.md` "construction inputs" phrasing the retracted decision rested on. The compiler is *supposed* to construct ABI expressions. So the withdrawn option was right and the recorded reasoning against it was wrong.

**But it cannot be done today, and the reason is a second divergence.** The same ADR decides: "Place the public `AbiExpr` domain type, admitted roots, validation, canonical identity, and authoritative pure checked evaluation semantics ... in `tiler-ir`." **It is not there.** `AbiRoot` is defined at `crates/tiler-artifact/src/program/expr.rs:159`, and `crates/tiler-ir/src/program/` contains no ABI expression module. The ADR's `implementation_status` is `spike-only`, which is consistent — it is accepted and unimplemented.

**Inference — the whole difficulty is downstream of that one divergence.** With `AbiExpr` where the ADR puts it, `tiler-compiler` constructs ABI expressions over `tiler-ir`, which it already depends on, and **no new dependency edge is needed at all**. The dilemma that produced the retracted decision — expose a plan record, or add `tiler-compiler → tiler-artifact` — exists only because the type is in the wrong crate. Neither option is right; the ordering is.

## Measured gap, for whoever sequences this

**What `push_variant` requires** (`crates/tiler-artifact/src/program/builder.rs:437`), taking `&VerifiedKernelProgram`: a boolean `applicability_guard`, `TargetProfileRef`, `FeasibilityRuleSetRef`, deferred predicates, and one `EntrySpec` per program stage — each with one `BindingSpec` per kernel buffer parameter (`accessible_bytes`, which must statically equal `access.view().window().length`) and a `LaunchSpec` (`grid_threads`, `threads_per_workgroup` which must statically equal `stage.kernel().requirements().threads_per_workgroup`, `zero_work_skips_dispatch`, preconditions).

**What a caller outside `tiler-compiler` can obtain today:** `PlanAlternative::kernels() -> &[VerifiedKernel]` and `Compilation::target_profile_key() -> &str`. That is the entire list. `ArtifactConstructionPlan` is `pub(crate)` in a private `mod program` with **every field private and exactly one accessor** (`lowering_providers`, `program.rs:183`). `KernelProgram::core()` — the only path to a `VerifiedKernelProgram` — is `#[cfg(test)]` (`program.rs:164`), so **no non-test path to the `push_variant` argument exists at all**.

**`grid_threads` is not on a kernel.** `VerifiedKernel::requirements()` gives `threads_per_workgroup`; total grid size lives only on `ScheduledRegion::schedule.launch.grid_threads`, behind the `pub(crate)` compiler wrapper. This is why `prototypes/serial-sum-run/src/main.rs` hardcodes its dispatch geometry and buffer indices instead of deriving them.

**No ABI derivation exists anywhere.** The only constructions of `accessible_bytes` and launch geometry as expressions are hand-written fixtures: `crates/tiler-artifact/src/program/tests.rs:321` `formulas()` and `:358` `entry()`, cloned in the `program/mod.rs:243` doctest and once more in `codec/tests.rs:278`. All assume the two-dimensional serial-sum shape, a hardcoded 4-byte element, and `grid_threads = rows`. The artifact layer *checks* these expressions and never mints one; it does not check `grid_threads` against anything at all.

The scalar truths to lift into expression form already exist and should not be re-derived: grid threads = iteration-shape product (`crates/tiler-compiler/src/physical.rs:424`, invariant at `crates/tiler-ir/src/schedule/builder.rs:286`), bytes = elements x element width (`crates/tiler-compiler/src/program.rs:490`), window length = required bytes (`crates/tiler-ir/src/program/builder.rs:276`).

**Blocked on `relocate-abi-expressions-into-tiler-ir`.** Building either option now would build on a divergence from an accepted decision.

## Unblocked, and the dilemma dissolved rather than decided

`relocate-abi-expressions-into-tiler-ir` landed, and with `AbiExpr` in `tiler-ir` the choice this ticket agonized over stops existing. Both recorded options — hand out the plan record, or add `tiler-compiler → tiler-artifact` — were consequences of the type being in the wrong crate, exactly as the retraction predicted. Neither is taken.

**What landed (commit `d6a69bf`).** `PlanAlternative::abi() -> AbiConstruction<'_>` exposes the applicability guard, per-binding accessible byte ranges, and per-entry launch geometry **as arena positions into a `Vec<ExprNode>`**, plus `kernel_program() -> &VerifiedKernelProgram`. The vocabulary is `tiler_ir::program::abi`, which both crates already depend on, so no compiler-internal identifier crosses and no new dependency edge is needed. The compiler derives (ADR 0068); the orchestrator replays onto the builder's own arena. The replay is mechanical and introduces no second derivation, because the decision about what each expression *says* was made in the compiler.

`KernelProgram::core()` lost its `#[cfg(test)]`; its own doc comment had been deferring to "a reviewed public compiler facade", which `session` now is.

## Two findings from reading the verifier, both of which shape the assembler

**1. The replay must be reachability-pruned, not wholesale.** `crates/tiler-artifact/src/program/verify.rs:29` raises `ArtifactDiagnostic::UnusedExpression` when any expression in the arena is unreachable from a use site. The compiler's canonical graph is nine nodes serving *both* alternatives: position 5 (input elements) is the materialized plan's stage-0 launch count and is referenced by nothing in the fused variant. Replaying the arena wholesale therefore fails whole-artifact verification for the fused plan. The assembler must walk the sub-DAG reachable from that variant's own roots. This is straightforward — operands always precede their nodes, so a single forward pass that skips unreachable positions preserves the ordering invariant, and `push_node` dedupes by content key anyway.

This is a case where the codec was right and the naive assembly was wrong, which is the outcome the **Do not** section above asks for.

**2. Blocked on a real gap: the compiler cannot name the capability a provider supplied.** `verify.rs:26-28` requires at least one `SelectedProvider`, and `SelectedProvider` needs a governed `CapabilityKey`. The compiler's `LoweringProviderIdentity` (`crates/tiler-compiler/src/request.rs:201-204`) carries only `provider: ProviderIdentity` and `capability_revision: LoweringCapabilityRevision`. There is **no capability key or family on it** — `LoweringFamily` exists at `crates/tiler-compiler/src/capability.rs:68` and is recorded on `RegisteredLoweringCapability` (`capability.rs:259-263`), but it is not propagated into the plan's `lowering_providers`.

So an assembler outside `tiler-compiler` has no faithful value for `SelectedProvider::capability`. Inventing a plausible key would put a capability into artifact identity that is not tied to the lowering that actually ran — `AGENTS.md` requires unsupported cases to "reject explicitly rather than silently approximating them", and ADR 0072 makes selected providers part of complete program identity, so an invented key is a wrong identity rather than a cosmetic placeholder.

**This is not a reason to stop; it is one more thing to build.** The fix is to carry the resolved capability's governed key alongside its revision from capability resolution into `LoweringProviderIdentity`, and expose it on the session view beside the provider. That is a `tiler-compiler` change in `capability.rs`/`request.rs`, in this ticket's declared scopes. Split out as `name-the-resolved-lowering-capability` only because it is independently testable and has its own closing condition, not to defer it — this ticket depends on it and cannot close first.

## Hard blocker found by reading: the codec is not reachable from an assembler

The ticket says to "encode the envelope, and then **decode the bytes back** and verify", and treats that as the point of the work. It assumed the codec was reachable. It is not.

**Fact — `crates/tiler-artifact/src/program/codec/encode.rs:71` and `codec/decode.rs:70`** declare `pub(crate) fn encode(envelope: &ArtifactEnvelope)` and `pub(crate) fn decode(bytes: &[u8])`. `codec` is a private module of `program`.

**Fact — `crates/tiler-artifact/src/program/codec/mod.rs:93-100`** states the exclusion deliberately: "The carried-payload vocabulary is the one part of this module that is public. A backend assembler outside this crate must be able to describe what it compiled, and nothing else here is reachable: the envelope, the encoder, the decoder, the rejection vocabulary, and the governed constants all stay `pub(crate)` behind this private module under ADR 0074 convention 7." The next line reads "Promoted on Tom's review, 2026-07-25", recording that the sibling promotion required review.

**Fact — the artifact itself carries no escape hatch.** Reading the whole `impl VerifiedArtifactProgram` block in `model.rs`, its complete public surface is `selected_providers`, `payloads`, `inputs`, `outputs`, `variants`, and `expressions`. There is no `encode`, no byte accessor, and no identity accessor. `ArtifactEnvelope` is `pub(crate)`.

**Inference.** No out-of-crate caller can encode an artifact, decode one, or observe an envelope digest. The round trip this ticket exists to prove is unreachable without promoting `pub(crate)` to `pub`, which ADR 0075 lists as always-ask and which the module's own comment shows was last exercised through review rather than unilaterally.

**What this does not block.** Everything up to `ArtifactProgramBuilder::build()` needs no promotion: the compilation environment, selected providers, `push_carried_payload`, the pruned arena replay, entry and launch specs, `push_variant`, and whole-artifact verification. That is the larger half and it is being built now. Only the encode → decode → re-encode assertions are gated.

**Consequence for this ticket's closing condition.** It cannot close on the assembler alone, because "a payload that assembles but does not survive a round trip is not carried" is its own stated bar. The promotion question is with Tom.

## The hard blocker above is RETRACTED — the codec's capability was promoted

**Fact — `crates/tiler-artifact/src/program/codec/mod.rs:118-127`,** re-exported one level up at `program/mod.rs:313-317`: `pub use view::{ArtifactCodecFailure, DecodedArtifact, SectionPurpose, SectionView, decode_artifact};`, with the comment "The codec's *capability* — encode an artifact, decode bytes back — is public as of `carry-the-metal-payload-in-an-artifact-envelope` ... Promoted on Tom's review, 2026-07-25." `crates/tiler-artifact/src/program/codec/view.rs:83` adds `impl VerifiedArtifactProgram { pub fn encode(&self) -> Result<Vec<u8>, ArtifactCodecFailure> }`.

The blocker read `encode`/`decode` as `pub(crate)` and concluded the capability was unreachable. It missed that a `view` module had been added exposing the capability *without* the envelope: accessors rather than fields, exactly to avoid committing the boundary to the codec's internal layout. So the round trip was reachable and this ticket was not blocked.

## Outcome

**Done. A real compilation and a real `metallib` are carried in the envelope and survive an encode, a decode, and a byte-identical re-encode.**

**Measured, on this host.** Running `cargo run -p tiler-prototype-compile`: the `[4, 1]` proof program compiles to the fused alternative `selected-plan:070999f334e69d51`, emits 1 entry point and 2,291 bytes of MSL, links to 3,667 bytes of `metallib`, and packages into a **32,259-byte envelope with 3 sections, 1 variant, and a 12,620-byte canonical identity**. Sizes are recorded as observations of this toolchain and are not identities — `assemble-the-metal-payload-from-emission-and-compilation` recorded what happens when a byte count is treated as one.

**What landed.** `prototypes/serial-sum-compile/src/bundle.rs` is the first artifact assembler in the workspace. It lives in the producer because that is the only component holding both halves: the compiler owns the plan and its ABI expressions (ADR 0068), the artifact crate owns the envelope, and neither may depend on the other. No new crate dependency edge and no new `pub` item in any library crate.

**Nothing is derived twice.** The target profile key and its exact descriptor, the feasibility rule set key and revision, each capability's provider and governed key, the guard, and every accessible-byte and launch formula are read whole from `tiler_compiler::session`. `push_carried_payload` derives the descriptor digest from the metadata bytes rather than taking one. The only thing the assembler computes is the transliteration of the compiler's arena onto the builder's, and the decision about what each expression *says* was made in the compiler.

**Asserted, in `bundle::tests::a_real_compilation_round_trips_through_the_neutral_envelope`.** Decode succeeds — which *is* the identity proof, since `decode_artifact` re-derives the identity from decoded content and refuses on mismatch (`codec/decode.rs:99-103`); the re-encode is byte-identical; the sections carry `BackendPayloadMetadata` and `BackendPayloadCode` by purpose rather than by position; the descriptor's digest equals `PayloadContent::identity()` of the content handed over, which decode independently re-derived from the metadata *section* (`codec/validate.rs:236-253`), so the two cannot agree by both being wrong; the derived feature set contains `tiler.artifact.feature.embedded-payload-code`; and the payload's compatibility contract is the compiler's profile key and descriptor bytes exactly.

**No codec check was weakened, and none refused a well-formed bundle.** Two boundaries were met and both are the codec being right.

**Retraction — the wholesale arena replay does not fail, and the reason matters.** The ticket predicted that replaying the compiler's arena whole fails whole-artifact verification with `UnusedExpression`, because the canonical graph serves both alternatives. The premise holds — the fused variant's roots reach 8 of 9 positions, leaving position 5 unreachable — but the conclusion is false. `ArtifactProgramBuilder::push_node` deduplicates by content key, and position 5 is `UnsignedLiteral(input_elements)`, byte-for-byte the node at position 1 that the input binding's byte range already multiplies. Replaying it returns the handle already minted, so no unreferenced node ever enters the builder's arena. `bundle::tests::the_pruned_and_wholesale_arena_replays_agree_because_the_builder_dedupes` measures this: it proves the unreachable set is non-empty, proves every node in it duplicates the content of a reachable one, and asserts the two replays encode to identical bytes. The assembler still prunes, because the safety is a property of *this* nine-node graph and not of the discipline — a node with unique content and no use site would fail, and nothing promises one never appears.

**A measured boundary of the envelope, found by packaging the other alternative.** The materialized plan assembles and verifies, encodes, and is then refused on decode as `ArtifactCodecFailure::Unsupported`. Its two stages make the projector derive `tiler.artifact.feature.multi-stage-program` (`codec/model.rs:608-614`), which is deliberately absent from `SUPPORTED_FEATURES` (`codec/model.rs:98-103`) because the neutral program section carries a program's canonical identity and not its dependency graph, so a reader cannot recover the order two stages must run in. Refusing is the fail-closed form of that gap; treating declaration order as execution order would be the silent one. `a_multi_stage_variant_encodes_and_this_reader_refuses_it` pins it so the limit is not rediscovered as a bug. `carry-reconstructable-kernel-programs-in-the-neutral-envelope` owns closing it.

**Two model gaps filed rather than papered over.**

- `record-the-capability-revision-in-selected-provider-identity` (p1). `SelectedProvider` has no field for the capability revision `docs/operation-extensions.md` requires a selected plan to record, and its `capability_api_version: u16` names a fact no component in `tiler-compiler` mints. The assembler carries the compiler's real `u32` revision into that slot through a **checked** narrowing that refuses rather than truncating, with the conflation named at the call site. That was chosen over hard-coding a constant (an invention) and over dropping the value (removing a real identity component); it is still a conflation and the ticket closes it.
- `expose-the-built-artifact-canonical-identity` (p2). `build` derives an identity and stores it in a private field with no accessor, so a producer cannot state "the artifact I built is the artifact these bytes name". The round trip is provable without it — decode compares the re-derivation to the manifest, and byte-identical re-encode pins the manifest to that re-derivation — but the direct comparison a cache key needs is not reachable.

**Maturity, stated apart.** An artifact that assembles, encodes, and re-validates from its own bytes is not an artifact that has run. `route-the-runtime-proof-through-the-artifact-envelope` still owns removing the direct-`metallib` bypass, and it remains blocked on `prototype-runtime-artifact-validation`.

## Current delivery correction — 2026-08-09

The maturity sentence above is the boundary at this ticket's landing, not the
current delivery state. [`prototype-runtime-artifact-validation`](prototype-runtime-artifact-validation.md)
and [`route-the-runtime-proof-through-the-artifact-envelope`](route-the-runtime-proof-through-the-artifact-envelope.md)
are both `done`. The runtime proof now reads envelope bytes, validates their
recorded identity and section digests, classifies compatibility, commits the
route, and loads `RoutedDispatch::object()` rather than the producer's in-memory
`CompiledArtifact`. The later
[`bound-the-backend-entry-key-by-the-identity-it-carries`](bound-the-backend-entry-key-by-the-identity-it-carries.md)
repair also restored the three-contributor program: both direct and envelope
paths now execute the same `4 by 3` subject and agree bit for bit with the
reference. This later hardware delivery does not change this ticket's original
artifact-round-trip outcome.
