---
id: decide-how-owner-private-conformance-inventories-cross-crate-boundaries
title: Decide how owner-private conformance inventories cross crate boundaries
status: done
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, define-the-conformance-obligation-and-evidence-requirement-algebra]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification, contracts/navigation]
paths: []
tags: [research, decision, conformance-progress, architecture]
---
# Decide how owner-private conformance inventories cross crate boundaries

## Goal

A decision-ready boundary for observing owner-private capability inventories and evidence from the conformance system without publishing mutable compiler internals, duplicating authorities, or reversing dependency direction.

## Work

1. Read the complete construction and consumption paths for the semantic registry iterator, reference capabilities, compiler rewrite registry, lowering and physical providers, public compilation/session views, typed explain ownership, schedule/KIR identities, and the conformance crate's dependency/public-surface rules.
2. Inventory which required subjects are already observable, which are owner-private, and which have no canonical identity yet.
3. Compare: minimal immutable owner accessors; owner-emitted machine-readable manifests; feature-gated test-support surfaces; owner-local reporters composed by an orchestrator; moving neutral vocabulary into `tiler-ir`; a new shared crate; and deferral.
4. Account for Rust's cross-crate `cfg(test)` behavior: a dependency is not compiled as its own test target merely because the consuming crate is running tests.
5. Eliminate options that expose construction/mutation, create a second authority, require a consumer build step, make `tiler-conformance` a dependency of owners, or publish a boundary without a real consumer.
6. State schema, identity, versioning, dependency, host-cost, and future-consumer consequences for every survivor.
7. Use independent derivation for any public-boundary recommendation and present only the nondominated frontier.

## Non-goals

- Do not add an accessor, feature, crate, serialization format, or public API.
- Do not move layer-local tests into `tiler-conformance`.
- Do not make rendered explain text a parse contract.

## Stop conditions

Stop for Tom if the dominant solution changes a consequential public crate/module/type boundary or introduces a new crate or generated workflow.

## Acceptance

- Every required private subject has a proposed observation route or an explicit unsupported result.
- Dependency direction and authority ownership remain valid for every survivor.
- The packet gives the smallest safe boundary, its strongest counterargument, and evidence that could reverse it.
- Follow-up implementation is split by owner and public-boundary risk.

## Refs

- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`decide-the-backend-provider-conformance-harness-public-surface`](decide-the-backend-provider-conformance-harness-public-surface.md)
- [`inventory-the-closed-world-conformance-claim-universe-by-owner`](inventory-the-closed-world-conformance-claim-universe-by-owner.md)

## Outcome

Research completed on 2026-08-24 at `36534fc0595a6f838f39bbb2ea070a86426af274`. The full decision packet is [Owner-private conformance inventory boundary](../docs/research/verification/owner-private-conformance-inventory-boundary.md); its Rust visibility negative control is retained in the [owner-private boundary fixture](../spikes/verification/owner-private-conformance-boundary/README.md).

### Fact verdicts

- **Verified per instance, imprecise as the system denominator:** the frozen semantic registry has public exact operation/type projections, but its index-law map is private and no independent construction rule proves every system subject must enter that registry.
- **Verified:** reference capabilities and validators are exact ordered private maps folded into registry identity, with no public enumeration.
- **False:** public lowering-registry identity, count, providers, and occurrence resolution do not reconstruct exact capability rows.
- **False:** one `RuleRegistry::rules()` result is not the compiler-wide rewrite universe. Canonical CSE and conditional algebraic exploration construct separate registries.
- **Verified narrowly:** `InstalledPhysicalProviders` and `Compilation` can name the exact offered provider identities for one compilation, but the provider trait does not require a complete strategy/obligation declaration.
- **Verified as evidence, false as denominator:** public compilation, plan, schedule, kernel, program, artifact, and runtime identities name reached products; they cannot name unreached subjects.
- **False:** verifier diagnostic enums are not invariant manifests. Multiple independent checks intentionally collapse to the same diagnostic, including many kernel reduction checks returning `KernelDiagnostic::ReductionContract`.
- **False:** tagging every surviving verifier failure with an obligation ID closes invocation coverage. Deleting the entire `verify_reduction` call can leave all remaining failures tagged and the declaration manifest unchanged while malformed reductions become accepted.
- **False:** `#[cfg(test)]` and `pub(crate)` items cannot be consumed directly across a crate boundary. The fixture obtains `E0425` and `E0603` respectively.
- **False:** a Cargo feature creates private test support. The fixture succeeds only by making the item conditionally `pub`, and feature unification makes that an ordinary consumer-visible surface.
- **False:** the test-only emitter alone is the design. It proves private transport only; without declaration-backed construction and a canonical exact owner-set join it can export an honestly incomplete table.
- **False:** source revision and toolchain alone bind a reporter population. The fixture emits two default rows and three feature-enabled rows after gating the subject, declaration, and implementation together; configuration/applicability must be authoritative too.
- **False:** `tiler-ir` is a neutral common-schema owner. It would acquire cross-layer evidence responsibility and contaminate dependency direction, including the AOT driver's accepted empty closure.

### Nondominated design

Use a federated declaration-first spine:

1. Every admitted feature/invariant/claim is constructed or verified through an owner declaration carrying stable subject identity and family-owned obligations. Verifiers additionally need a declaration-driven checker graph or typestate chain; stable semantic obligations are distinct from checker/site witnesses so deleting a checker is loud without making refactoring change obligation identity.
2. Repository-governed private populations use owner-local reporters writing bounded canonical files to an explicit path. The orchestrator never parses test names or stdout.
3. Arbitrary frozen registries and dynamically installed providers use immutable declaration snapshots bound to the exact configuration/compilation identity. These surfaces are added only where arbitrary configured-environment qualification is a real consumer.
4. Existing verified products and receipts remain the evidence plane and are joined by identity; they do not become feature rows.
5. One proposed dependency-bottom protocol crate owns bounded canonical syntax and validation only. It owns no subject meaning, profile, status, evidence authority, I/O, or orchestration.
6. Unknown owner populations remain explicit audit blockers until migrated. Goal-profile edits and owner-universe changes are reported separately from evidence movement.
7. Declarations are target-independent with applicability as data where possible. A genuinely configuration-scoped manifest binds target triple, Cargo features, effective rustc cfg, toolchain, and build-produced configuration, and the accepted universe names the complete required configuration matrix.

`tiler-metal-aot` is a genuine unresolved exception, not a free adapter. ADR 0077 forbids even a development dependency, while `CompileStage::ALL` does not owner-mint conformance subject/obligation identity or meaning. Its child decision must choose an owner-native zero-dependency projection, an explicit amendment admitting a protocol edge, or an honest unknown result.

The design is intentionally not one universal accessor. Static private owners and dynamic public extension environments have different observation requirements; forcing either route onto the other either publishes internals or makes arbitrary configured universes unqualifiable.

### Decision requested

Tom must accept or reject the proposed new dependency-bottom protocol crate and the federated boundary direction. Acceptance does not approve a concrete crate name, field set, encoding tag, accessor, provider trait method, reporter command, AOT exception route, verifier mechanism, configuration matrix, or owner migration. Those remain bounded children after unlike pilots.

### Follow-up family

- [`specify-the-canonical-owner-conformance-manifest-protocol`](specify-the-canonical-owner-conformance-manifest-protocol.md) defines the authority-neutral envelope and exact owner-set completion after this decision.
- [`pilot-a-declaration-backed-private-registry-manifest`](pilot-a-declaration-backed-private-registry-manifest.md) proves one private frozen registry uses one source for construction, identity, and manifest.
- [`pilot-id-bearing-private-verifier-obligations`](pilot-id-bearing-private-verifier-obligations.md) separates semantic obligations from checker/site witnesses and compares failure tagging against declaration-driven executor/table and typestate designs that make deletion of a whole checker invocation loud.
- [`decide-the-installed-provider-conformance-declaration-surface`](decide-the-installed-provider-conformance-declaration-surface.md) owns the consequential public boundary for arbitrary configured provider environments and reconciles it with the no-public-harness recommendation.
- [`decide-the-zero-dependency-metal-aot-conformance-declaration-route`](decide-the-zero-dependency-metal-aot-conformance-declaration-route.md) owns the AOT empty-closure/public-projection conflict; no adapter may invent its obligations in conformance.
- [`measure-the-federated-conformance-manifest-lane`](measure-the-federated-conformance-manifest-lane.md) measures configuration-matrix cost and crash/partial-publication behavior only after the structural paths are concrete.

The protocol ticket now depends on the accepted P+K+M+T authority decision, and the canonical receipt/freshness ticket must depend on the concrete protocol result. Because the protected-review and threshold-signing designs already depend on that receipt ticket, the `P`, `K`, and transitive `M`/`T` graph will consume one owner-root/closure design rather than advancing a parallel canonical manifest.

No implementation, new crate, accessor, feature, serialization format, or public API was added by this ticket.

### Decision — accepted 2026-08-24

Tom accepted the federated owner-declaration direction and the dependency-bottom, authority-neutral protocol crate in principle, directly in the Codex conversation on 2026-08-24. The accepted boundary keeps subject meaning and obligations with their real owners, keeps receipts as evidence rather than denominator authority, uses owner-local reporters for repository-governed private populations, and permits immutable declaration snapshots only where a configured public extension environment is a real conformance consumer.

This acceptance does **not** approve a crate name, dependency edge, field set, encoding or identity domain, public accessor, provider trait method, reporter command, verifier mechanism, required configuration matrix, owner migration, or Metal-AOT exception. Each remains governed by the bounded follow-up ticket that owns it. Tom's subsequent requirement that conformance describe the comprehensive intended Tiler system rather than only the current source revision also remains a separate target-architecture-catalogue prerequisite; this ticket does not make an implementation manifest authoritative for that future target universe.
