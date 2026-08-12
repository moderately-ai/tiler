---
id: establish-vector-execution-form-numerical-authority
title: Establish numerical authority for exact arithmetic execution forms
status: awaiting-decision
priority: p1
dependencies: []
related: [declare-cpu-vector-realization-facts-in-the-target-profile, define-plural-operation-specific-vector-realization-requirements, earn-cpu-feature-level-execution-environments-from-host-observation]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, cpu, vector, public-boundary, correctness, identity]
---

## User-visible outcome

A target can attest the numerical behaviour of the exact scalar, fixed-vector, or scalable-vector arithmetic realization that will execute. A fact measured for a scalar path, another instruction family, or another physical provider cannot silently license a packed path.

## Source-first Fact audit — 2026-08-12

1. **Verified.** `ScalarArithmeticSubject`, anchor `One scalar-arithmetic policy subject`, is explicitly the caller's scalar arithmetic policy: `ArithmeticType` plus complete `ResolvedValueType`. It carries neither the reached operation nor its physical execution form.
2. **Verified.** `DeclaredBehaviour`, anchor `One line of a target profile's honourability declaration`, keys a numerical row by dimension, arithmetic type, resolved type, behaviour, and source. `NumericalRequirement`, anchor `A candidate requirement`, has the same dtype-wide subject. Neither names an operation, lane form, or implementation provider.
3. **Verified.** `DeliveredRealizationEvidence::materialize`, anchor `Every occurrence the packaged program covers`, repeats each selected dtype-wide fact at every reached consuming operation. The operation is used only to find a `PolicyLocus`; it is not retained in `SelectedObligationRow`.
4. **Verified.** Artifact `NumericalObligation`, anchor `One locus-specific numerical obligation`, carries a policy-subject index, dimension, semantic occurrence/locus, required behaviour, and evidence index. `EntryPolicyBinding`, anchor `The association binding one packaged executable entry to its policy subject`, binds only the caller policy. The delivered record cannot distinguish two execution forms of the same operation or two providers implementing it differently.
5. **Verified with a narrower conclusion.** The adopted CPU-vector research refuted a universal scalar-versus-packed class distinction on the examined AArch64 and x86 SIMD controls, but retained concrete per-instruction exceptions: reciprocal estimates under `FPCR.AH`, min/max exemptions, `FZ16`, and x87's lack of `FTZ`/`DAZ`. Therefore a two-valued `Scalar | Vector` path tag is insufficient even though scalar evidence must not license vector execution.
6. **Imprecise in the adopted research.** `cpu-vector-realization-facts.md`, anchor `Runtime feature detection never enters feasibility`, says the existing variant filter performs host feature detection. Current runtime `variant_eligibility`, anchor `against the host's stated ExecutionEnvironment and against nothing else`, only compares a caller-stated environment. The linked host-qualification ticket owns earning CPUID/HWCAP evidence; this ticket must not imply that authority already exists.
7. **Verified.** The delivered-realization bytes are a separately domain-tagged, length-framed record in both artifact identity and manifest encoding. Its grammar can step independently without changing the outer manifest row shape. Existing readers will reject a new delivered-record domain before interpreting its new tables.

The old one-paragraph Fact was directionally correct but too coarse. The problem is not merely adding an execution-path enum; it is binding numerical evidence to the exact reached operation and the exact selected implementation realization without duplicating semantic policy, target capability, or runtime eligibility authority.

## Decision packet — 2026-08-12

### Recommended authority split

Keep five orthogonal subjects separate and join them explicitly:

| Subject | Owns | Must not own |
| --- | --- | --- |
| `ScalarArithmeticSubject` | Caller-visible numerical policy over a complete scalar value type | Physical lane shape, provider, instruction family |
| `ArithmeticApplicationSubject` | Exact scalar operation key, canonical attributes, and ordered operand/result resolved-type identities | Target support or measured behaviour |
| `ArithmeticExecutionRealization` | Scalar, fixed-vector with literal nonzero lanes, or scalable-vector form; selected physical provider identity; provider-owned versioned execution-variant key | Caller policy or runtime host qualification |
| `NumericalDimension` plus `DimensionBehaviour` | The exact numerical property attested | Operation or provider identity |
| `FactSourceProvenance` plus `HonouringMeans` | Who supports the claim, under what build/environment/validity scope, and by what means | The subject being claimed |

The composed `ArithmeticExecutionSubject` is the checked tuple `(policy subject, exact application, execution realization)`. Its fields are private and canonically encoded. A target-profile producer constructs the application through the frozen scalar registry and constructs only a well-formed provider/variant/form tuple; it cannot claim that a future selected proposal matches it. The compiler independently derives the same subject from the selected proposal and admits the target row only by exact equality. No string rendering, digest, wildcard, or inferred default participates in equality.

`ArithmeticExecutionForm` is a total tagged sum:

- `Scalar`;
- `FixedVector { lanes: NonZeroLaneCount }`; and
- `ScalableVector`.

The form alone is not the realization identity. `ArithmeticExecutionRealization` also carries the selected `ProviderIdentity` and a provider-owned, portable, versioned `ArithmeticExecutionVariantKey`. This is load-bearing: one provider may legitimately offer two instruction strategies for the same operation and lane form, and an installed provider must not borrow a row measured for the governed provider. Changing the implementation behind a retained variant requires changing the provider or variant revision.

The exact application includes canonical attributes and ordered input/output types because scalar operation keys may be polymorphic and attributes may select numerically distinct behaviour. It excludes graph-local value IDs, coordinates, shapes unrelated to scalar typing, masking, address space, and alignment. Those belong respectively to occurrence navigation or the accepted vector realization/applicability subjects; copying them here would create overlapping authorities and mismatch states.

### Construction and consumption

The verified schedule and scalar program derive the application and lane form. The selected physical proposal supplies the provider and execution-variant identity. The compiler, not the caller or target-profile builder, joins those facts for each reached arithmetic application and refuses a proposal whose stated realization cannot be re-derived.

Target-profile declarations key each numerical dimension by the complete execution subject. The existing broad public methods such as `declare_input_subnormals(ScalarArithmetic, ...)` are replaced rather than retained as wildcard compatibility paths. A producer may attach one provenance record to several exact rows, so one measurement or normative guarantee can cover scalar and packed forms without turning them into one ambiguous set-valued fact.

A scalar-epilogue plan derives both subjects for every reached arithmetic application: the fixed-vector body realization and the scalar epilogue realization. Every consumed numerical dimension must resolve for both. A declaration for only one path leaves the other `Unknown`; `Unrealizable` remains a typed rejection. There is no implication from scalar to vector, from one fixed width to another, from fixed to scalable, or from one provider/variant to another.

The accepted plural vector-requirement algebra reuses `ArithmeticExecutionSubject` for its arithmetic member rather than restating operation, dtype, or lane shape. Load/store realization, masking, memory domain, alignment applicability, and runtime ISA qualification stay in their existing sibling subjects. This is the nonduplicating seam the original ticket asked for.

The CPU backend is the real consumer. Its emitter must consume the selected execution realization and either emit that exact form or refuse before publication. No mock provider, fake device, or reference-evaluator vector mode is introduced. Profiles remain silent until normative or measured evidence exists. The reference evaluator continues to compute the semantic oracle; it is not evidence about physical packed instructions.

### Delivered evidence and identity

The target profile's checked and complete descriptor grammars change because every numerical row's subject changes. Step `PROFILE_DESCRIPTOR_DOMAIN` and `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN`, and mint a new governed feasibility-rule-set key because the predicate vocabulary changes rather than merely its comparison algorithm.

The delivered-realization record keeps the caller policy-subject table and adds a canonical execution-subject table. Each `NumericalObligation` references both its policy subject and exact execution subject. A plural entry-to-execution binding records every execution subject reached by each packaged entry; the existing singular `EntryPolicyBinding` remains the policy cross-check. Builder and decoder reject dangling, duplicate, wrong-provider, wrong-entry, and obligation/evidence subject mismatches.

Step `DELIVERED_REALIZATION_DOMAIN` from `v2` to `v3`. No outer manifest schema or artifact-domain step is required solely for this change: both the manifest and artifact identity already length-frame the delivered record's independently domain-tagged canonical bytes. Artifact identities and cache subjects still move transitively because their nested record bytes change. If implementation instead inserts fields into the fixed entry record, that different design owes a manifest major and artifact-domain step and is therefore dominated here.

Schedule and KIR identities change only where their accepted vector/provider realization carriers add new bytes. Do not duplicate the delivered evidence table in either identity. Existing scalar programs may preserve their structural bytes only if the execution realization is already injectively derived from selected provider provenance and scalar-program identity; any new stored execution-variant field must be identity-bearing and its owning domain must follow that codec's append/replace rule.

### Correctness, maintenance, and host cost

The design is exact and fail-closed. It makes the key dimensions mutually exclusive and collectively exhaustive at their own layer: semantic policy, exact operation application, physical realization, numerical property, and evidence provenance cannot substitute for one another.

Compiler work is linear in reached arithmetic applications times consumed dimensions, with canonical sort/dedup over a population already bounded by scalar-program, profile-descriptor, and artifact limits. Provenance remains deduplicated, so several rows backed by one measurement do not repeat compiler/environment records. Runtime dispatch performs no additional numerical query; emitted-kernel performance is unaffected. Artifact growth is proportional to unique reached execution subjects and obligations, not tensor size or lane trip count.

Do not add an independent row-count cap during alpha. Existing complete structural byte/count budgets remain the admission authority until measurements show a narrower bound is required.

## Ranked options

1. **Exact application plus provider-versioned execution realization, reused by vector requirements and numerical evidence.** Best correctness and maintainability, bounded linear host work, no runtime query, and no duplicate target capability vocabulary. This is the recommendation.
2. **Exact application plus lane form, relying on profile compilation selection to distinguish implementations.** Smaller, but unsafe once two installed providers or two strategies under one provider can realize the same application/form differently. It is acceptable only if the physical-provider surface is first narrowed to prove one unique implementation, which current source does not.
3. **Provider-versioned execution realization without exact application.** Distinguishes implementations but lets one row silently cover unrelated operations whose per-instruction exceptions differ. Rejected.
4. **Add only `Scalar | FixedVector | ScalableVector` to the current dtype-wide key.** Fixes the broadest omission but not x87/SSE, reciprocal/min/max, fixed-width, operation, or provider distinctions. Rejected as an attractive partial fix.
5. **One set-valued row covering several paths.** Can be made correct, but subset/coverage rules add a second implication algebra and make missing-path explanations weaker. Separate exact rows sharing provenance are equally expressive and simpler.
6. **Keep scalar facts and measure packed behaviour at runtime.** Wrong authority phase, no generic executable query, environment-dependent, and capable of changing after observation. Rejected.
7. **Keep vector execution unavailable.** Correct and cheapest, but blocks the accepted real CPU vector vertical; use only until this boundary and its host-qualification prerequisites land.

## Strongest counterpoint and reversal evidence

Exact rows are more verbose than one dtype-wide declaration and can repeat the same measurement across many operations. That cost is real, but source evidence already disproves the wildcard: several instruction families differ under one dtype and control state. Reverse to a broader operation-family row only when an authoritative, versioned family definition proves closed membership and uniform behaviour for every member, and a perturbation adding a non-uniform member is rejected. Convenience, shared control registers, or a finite passing corpus are not that proof.

## Required failure-path evidence

- Perturb independently: policy subject, operation key, canonical attributes, each ordered input/output type, scalar/fixed/scalable tag, fixed lane count, provider identity, execution-variant key/revision, every numerical dimension, required behaviour, means, and provenance. Each perturbation must move identity and leave the exact requirement `Unknown` or explicitly `Rejected`; none may satisfy it.
- Construct one scalar-epilogue fixture and remove first its scalar row, then its vector row. Each removal must refuse the corresponding path by name. Two exact rows sharing one source must succeed without merging their subjects.
- Install a second physical provider that offers structurally identical arithmetic with another execution key. It must not reuse the governed provider's numerical row.
- Exercise two variants under one provider and prove the variant key distinguishes them. Reusing a key for changed implementation semantics must fail the provider's identity/revision check.
- Perturb a non-numerical vector property such as alignment or memory domain and prove it moves the vector applicability/realization subject without creating a second numerical spelling.
- Show old delivered-realization bytes fail at the new domain and new bytes fail under the old decoder; prove no outer manifest parser interprets either record under the other's grammar.
- Prove the supported CPU path has a host-earned feature-level execution environment. A caller-stated environment alone must not unlock it.

## Decision request

Accept the exact, provider-versioned arithmetic execution subject and the strict composition above; revise it; or keep vector arithmetic unavailable. Implementation remains blocked on the accepted vector schedule/requirement carrier and host-earned CPU feature qualification.

## Closes when

The accepted subject is implemented through schedule derivation, target resolution, explain evidence, delivered realization, artifact translation, and the real CPU backend; every perturbation above has been observed failing; scalar evidence cannot satisfy a vector obligation; and no mock or compile-host inference supplies missing authority.
