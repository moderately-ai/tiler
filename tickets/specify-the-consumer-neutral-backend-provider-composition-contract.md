---
id: specify-the-consumer-neutral-backend-provider-composition-contract
title: Specify the consumer-neutral backend-provider composition contract
status: in-progress
priority: p1
dependencies: [define-backend-device-and-execution-context-vocabulary, prototype-a-forkless-custom-metal-physical-provider, prototype-a-bounded-scalar-cpu-backend-vertical]
related: [draft-public-extension-seam-ownership-adr, runtime-execution-contract, target-profile-feasibility-model]
scopes: [research/extensions, research/program-planning, research/artifacts, research/runtime, contracts/foundation]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, design, research]
claimed_from: todo
assignee: worker-specify-the-co
lease_expires_at: 1785550314
---
## User-visible outcome

A concrete consumer-neutral design explains how statically linked backend components compose from compilation through execution without one monolithic `Device` trait and without requiring custom implementations to maintain Tiler forks.

## Why this slice exists

The corpus defines semantic and lowering providers but does not define `BackendProvider`, provider bundles, emitter registration, runtime-adapter registration, partial backend reuse, or cross-backend selection. The Metal and CPU spikes must constrain this design before any public abstraction is accepted.

## Implementation keys

- Synthesize only requirements exercised by both concrete spikes or forced by accepted correctness contracts.
- Separate target-profile authority, physical implementation proposals, backend emission/artifact production, and live runtime adaptation. State which pieces may be supplied independently and how a partial provider reuses another backend's pieces.
- Keep build-time producers and runtime adapters independently installable and join them only through governed backend, representation, target-profile, payload schema, compatibility contract, entry mapping, and execution-policy identities.
- Carry and validate producer/provider provenance separately without presuming it equals the independently selected runtime-adapter identity. The responsibility matrix must identify which subjects are compared, which are retained only as provenance, and which are selected independently.
- Define explicit per-session builders and immutable frozen registries; forbid global discovery, registration-order precedence, last-wins replacement, and ambient provider mutation.
- Preserve propose-then-reverify, typed explain outcomes, deterministic identity, hard-feasibility versus cost separation, and the one-way routing commit.
- Specify trust and linkage: trusted Rust code statically linked into one binary; native dynamic loading, stable plugin ABI, untrusted code, hot reload, and cross-process callbacks remain deferred.
- Specify the minimum conformance obligations and every unsupported case.
- Provide small end-to-end examples for standard Metal plus a partial custom provider and for a CPU backend.

## Closes when

The research record contains an exact responsibility/identity/lifecycle matrix, concrete interface sketches grounded in the spikes, eliminated alternatives, a proposed dependency direction, and the atomic decisions a durable ADR must make.

## What the Metal spike supplies

From `prototype-a-forkless-custom-metal-physical-provider`, evidence at commit `7b1e3a7e15b09dd3ea65c88759699655c462be4a`, harness at [`spikes/extensions/forkless-physical-provider/`](../spikes/extensions/forkless-physical-provider/README.md). Six constraints this contract can now treat as measured rather than assumed.

**The seam to design is registration and re-verification, not a way to express an implementation.** A proposal body is a `tiler_ir::schedule::ScheduledRegion`, already fully public and constructible from an out-of-workspace crate. Whatever the contract names, it does not need a new implementation vocabulary.

**Partial reuse of Metal is already available and needs nothing from this contract.** `tiler-metal` does not depend on `tiler-compiler`; it consumes verified kernels and knows nothing about who proposed them. A provider crate reuses `lower_scheduled_region` and `emit_translation_unit` unchanged. So "how does a partial provider reuse another backend's pieces" has a concrete answer for the emission piece — it depends on `tiler-ir` and the emitter crate directly, and the composition contract does not mediate that edge.

**Visibility and installation are two separate obligations, and only one is on anyone's list.** The spike's compile-fail evidence shows that publishing `frontier::PhysicalImplementationProvider` would still leave a provider uninstallable: the provider array is a hardcoded literal at `pipeline/planning.rs:171` and the internal request carries no provider field. ADR 0078 item 4 recorded exactly this asymmetry for lowering providers and closed it with `CompileRequest::with_capabilities`; the contract must state the physical analogue explicitly rather than assume a `pub` keyword suffices.

**Observability is a third obligation.** `Compilation::offered_providers` reports lowering providers only, and no public type carries physical-provider provenance. The responsibility matrix therefore owes a disclosure rule: which provider identities a compilation reports, and whether an installed-but-never-selected provider is visible. Today neither is answerable.

**The specialization axis is the schedule, and it is identity-bearing.** `verify_region_subject_binding` compares region id, iteration shape, scalar program, semantic members, and access map, and says nothing about `KernelSchedule`; `threads_per_workgroup` is free under the intrinsic verifier and folded into `CanonicalScheduledRegionIdentity`. Two alternatives of one region therefore carry distinct identities and emit distinct entry-point symbols from identical bodies. That is the concrete shape of "several providers' implementations retained side by side", and it means the additivity claim needs no new identity authority for this case.

**Propose-then-reverify is already partly public, and the split is uneven.** Of the gates `verify_schedule_with_feasibility` runs, only the intrinsic verifier (`ScheduledRegionBuilder::build`) is reachable from outside; the request-authority check, the numerical-realization comparison, the subject binding, and the feasibility assessment are private. The contract must say which of those an out-of-crate provider may pre-run and which stay host-only, because a provider that can pre-run none of them cannot report a typed local failure of its own.

## Graph maintenance

- Release `draft-the-backend-provider-composition-adr` only after both evidence spikes and the vocabulary contract are complete.
- File narrow feasibility tickets for any missing verifier or identity authority; do not hide them inside a proposed universal abstraction.
- Keep public visibility unchanged in this ticket.

## Outcome

The record is [`docs/research/extensions/backend-provider-composition.md`](../docs/research/extensions/backend-provider-composition.md), authored against base `e6a47d9`. No visibility, signature, or behaviour changed; nothing was accepted.

**What it establishes.** A thirteen-row responsibility/identity/lifecycle matrix separating the subjects that are compared (lowering capability key at resolution, target-profile key *and* exact descriptor at load, backend family and representation as a pair, program canonical identity at bind) from those retained as provenance only (semantic admission, registry snapshot, physical-provider identity, payload provenance) and those independently selected (backend emitter, build orchestration, runtime adapter, live device and execution context). The central negative result is that **no `BackendProvider` type is needed**: ten of the thirteen rows already have a mechanism, both spikes wanted strict subsets, and a monolithic trait would have to re-mediate an emitter edge that a Cargo dependency already checks at compile time. The three without one are physical-provider installation, opaque-call registration on the compile path, and build-time orchestration; the runtime adapter's lack of a registry is not a fourth, because independent selection is its mechanism.

**The one row with no seam at all is `tiler-build`.** Verified by full read rather than inferred: `tiler-metal` and `tiler-metal-aot` are unconditional dependencies, all six modules are `metal_*`, the backend and representation are `&str` constants at `metal_assembly.rs:27-28`, and `accept_or_publish_metal_plan` is Metal-typed in its signature. `grep -rniE "(backend|emitter)[_ ]?(registry|register|factory|plugin|dispatcher|selector)" --include='*.rs' crates/` returns nothing; the positive control, the same grep with `lowering`, returns 69 lines. The useful half is that the neutral seam already exists in private — `metal_plan.rs:266`'s `assemble_artifact` takes a `declare_payload` closure and names Metal nowhere — so this is a promotion, not a design. The CPU vertical could only run because it skipped `tiler-build` entirely.

**Eleven atomic decisions** are enumerated for `draft-the-backend-provider-composition-adr`, with D1 (scheduling knowledge as data, code, or a checked combination) prior to D2 and D3, and D3 flagged as one sentence from Tom. Each carries a recommendation and what would refute it.

**Eliminated alternatives**, each with grounds: the monolithic `Device`/`BackendProvider` trait, global discovery, registration-order precedence, last-wins replacement, ambient mutation of a frozen registry, joining producer to adapter by a Rust object or `TypeId`, requiring adapter identity to match producer identity, a mandatory `prepare` stage, treating installed and visible as one obligation, provider-declared cost models, provider-stamped provenance, and an unrestricted scoring callback.

**Findings beyond the two spikes' six and eleven, each verified by full read with a reproducible check and a positive control.**

- A fifth instance of the installation asymmetry, and it is in-crate: the sole production `enumerate_frontier` call site constructs an empty `OpaqueCallRegistry` inline (`pipeline/planning.rs:228`), so no opaque call reaches `session::compile` from any caller. Distinct from ADR 0078's correction, which is about out-of-crate registration. Filed as `register-opaque-calls-on-the-compile-path`.
- **A correction to the CPU vertical's finding 2.** It reports the governed keys as validating "length and alphabet only". `crates/tiler-artifact/src/program/keys.rs:73` validates non-empty and ≤256 bytes and nothing else; the alphabet lives in the *separate same-named* `tiler_compiler::target::TargetProfileKey` (`target.rs:224-241`), and `metal_plan.rs:333` launders the strict spelling into the permissive one. Reproduce with `grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/` (empty); positive control is the identical grep over `crates/tiler-compiler/src/target.rs` (six lines). Filed as `reconcile-the-two-target-profile-key-grammars`.
- `CapabilityAxis` is `pub(crate)` with seven variants reachable only through `declare_*` methods that hard-code their own axis, so the CPU vertical's missing axes are inexpressible rather than merely undeclared.
- `ArtifactExecutionPolicy::RequiresDeviceTranslation` is refused outright by the device-free loader (`load.rs:468-473`), so the two-valued vocabulary is effectively one-valued at the load boundary.
- The exact `preflight`-versus-`prepare` condition: `preflight` suffices iff zero deferred predicates **and** zero route requirements, and once `prepare` is entered both device stages are mandatory even when empty.
- A runtime adapter reports facts and never adjudicates them: the loader refuses a foreign-owned backend requirement before consulting any adapter, and performs the comparison itself on the adapter's observation.
- The propose-then-reverify split is five gates, of which one is public (`ScheduledRegionBuilder::build`), two are correctly host-only (request authority, request-subject binding), and two are half-reachable in the same shape — the provider holds one operand and the host holds the comparison. That asymmetry is atomic decision D6.

**Graph maintenance executed.** `draft-the-backend-provider-composition-adr` needs no release action: all three prerequisites are `done` by reading (`define-backend-device-and-execution-context-vocabulary`, `prototype-a-forkless-custom-metal-physical-provider`, `prototype-a-bounded-scalar-cpu-backend-vertical`), and its dependency list names this ticket alone, so it becomes ready on this ticket's closure. Three narrow tickets filed rather than absorbed: `register-opaque-calls-on-the-compile-path`, `name-a-host-process-availability-phase`, `reconcile-the-two-target-profile-key-grammars`.

**Deliberately not done.** The CPU vertical's experiment record cannot gain a `supports` edge to this research record, because `spikes/target-profiles/**` maps to `research/target-profiles`, outside this ticket's scopes; the citation is prose only, and the edge is a one-line frontmatter addition for whoever next holds that scope. `docs/document-metadata.md:114` asserts that `validate_links` "resolves every local link in every governed document, so a decision citing a harness that has moved or been deleted fails the repository gate", while the same document's own validation section says there is no validator; `grep -rn "validate_links" .` finds the name only in prose. That contradiction is in `contracts/navigation` and unrelated to this ticket's subject, so it is left for a navigation-scoped ticket rather than swept in here.
