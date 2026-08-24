---
schema: "tiler-doc/v1"
id: "tiler.research.verification.conformance-obligation-evidence-algebra"
kind: "research"
title: "Conformance obligation and evidence-requirement algebra"
topics: ["verification", "conformance", "evidence", "identity"]
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
catalog_group: "artifacts-build-toolchains"
informs: ["tiler.contract.correctness-and-testing"]
ticket: "define-the-conformance-obligation-and-evidence-requirement-algebra"
---
# Conformance obligation and evidence-requirement algebra

**Status:** research complete; schema remains private and unimplemented

**Reviewed:** 2026-08-24 at `6b6787f0f26b9775769e3cee9e1c5779c9eb431e`

## Result

**Proposal.** Use family-owned obligation declarations compiled into a small canonical predicate algebra over immutable evidence atoms. Do not use a maturity ladder, a scalar score, or one universal family schema. The common layer owns only composition, exact matching, canonical join validation, and derived reporting; each subject owner retains the meaning and semantic validation of its obligation and each evidence producer retains the authority it can actually mint.

This hybrid is the only nondominated candidate from the decision packet below. A predicate without family-owned constructors invites meaningless combinations; family-only schemas cannot support common audit, regression, qualification, or receipt validation. The combination provides common machine semantics without pretending that proof, execution, normative authority, and measurement are ordered versions of one thing.

## Source-first audit

The ticket states no numeric Facts. Its governing premises were re-read rather than inherited:

| premise | verdict | source anchor and consequence |
| --- | --- | --- |
| Evidence classes are categories rather than one total order. | **verified** | [Documentation metadata](../../document-metadata.md), anchor `These classes are categories, not a total strength ordering`, explicitly rejects a general ranking. |
| A run may report authority supplied by an owner but cannot mint implementation maturity or normative support. | **verified** | [The deferred executed-run ledger ticket](../../../tickets/derive-the-conformance-evidence-ledger-cells-from-executed-runs.md), anchors `The maturity ladder cannot be stamped` and `A run can report the class its inputs carried`, separates observation from source-owned authority. |
| Accuracy evidence already has a family-specific discharge rule. | **verified** | [`ConformanceEvidenceClass`](../../../crates/tiler-ir/src/semantic/accuracy/evidence.rs) and `ConformanceEvidence::discharge` admit formal proof, exhaustive finite evidence, and applicable normative guarantees for a hard accuracy requirement while refusing empirical qualification and unknown evidence. This rule must remain owned by accuracy rather than being generalized into a universal order. |
| Index evidence methods are derivations, not confidence levels. | **verified** | [`IndexDomainSoundProof`](../../../crates/tiler-ir/src/index/predicate.rs), anchor `The variants name derivations rather than confidence levels`, and the separate fact-source vocabulary demonstrate independent method and premise axes. |
| Machine observation distinguishes absence from reached-stage failure. | **verified** | [`Measured`](../../../crates/tiler-conformance/src/measurement.rs), anchors `Measured::Unavailable` and `Measured::Failed`, keeps an absent environment separate from a stage a qualified host reached and that refused. |
| Owner population and evidence population are different. | **verified** | [The owner-universe audit](conformance-claim-universe-by-owner.md), anchor `Tests and receipts are evidence, not feature rows`, retains unknown owner populations instead of inferring features from tests. |
| Applicability cannot be inferred from artifact equality or successful execution. | **verified** | [`refuse_to_offer_the_declared_profile`](../../../crates/tiler-conformance/src/applicability.rs), source anchor `producer-declared equality, NOT`, separates an artifact's declared profile from authority to offer it. |

**Fact.** Existing vocabularies disagree for principled reasons. `ConformanceEvidenceClass`, `IndexDomainEvidence`, `FusionEvidenceClass`, `EvidenceBasis`, `ToolchainEvidence`, `Measured`, and the several feasibility/refusal outcomes answer different owner questions. Replacing them with a cross-repository enum would erase their discharge rules, scopes, and construction authority. This report therefore specifies a join vocabulary, not a replacement vocabulary.

## The semantic split

Five objects must remain distinct:

1. A **subject** is something in the owner-derived system universe: a capability, invariant, refusal contract, performance claim, or other named claim.
2. A **case** is a finite declared stimulus or construction used to interrogate a subject. A subject may have no executable case, and a case may exercise several subjects without merging them.
3. An **obligation** is a family owner-authored proposition about one subject under an exact applicability scope. It states what observation would satisfy or violate the proposition and which evidence is sufficient.
4. An **observation** is what a route actually produced: a value, exact typed refusal, invariant result, unavailable boundary, reached-stage failure, proof result, or normative statement reference. It carries no pass/fail judgment of its own.
5. An **evaluated cell** joins one obligation to a validated evidence set and records the derived verdict. Its color is presentation, not stored authority.

Conflating any adjacent pair creates a known failure. Treating a test as a subject lets test deletion shrink the denominator. Treating an observation as a verdict makes an expected refusal look failed. Treating a verdict as evidence lets a green bit replace its oracle and context. Treating color as stored authority lets a hand edit manufacture progress.

## Canonical conceptual model

This is a language-neutral design requirement, not a proposed Rust public API.

```text
SubjectRef {
  owner_authority_id,
  family_id,
  subject_id,
  subject_revision
}

CaseRef {
  case_manifest_id,
  case_id,
  case_revision
}

ObligationDeclaration {
  obligation_id,
  subject: SubjectRef,
  case: optional CaseRef,
  applicability_requirement,
  observation_matcher,
  evidence_requirement,
  declaration_revision
}

EvidenceAtom {
  atom_id,
  subject: SubjectRef,
  case: optional CaseRef,
  producer_authority_id,
  producer_role,
  kind,
  context,
  observed_at_source_revision,
  observation
}

EvaluatedCell {
  obligation_id,
  obligation_revision,
  evidence_set_id,
  evaluator_identity,
  verdict,
  unmet_predicates,
  invalid_atoms
}
```

Every identifier is required and versioned. `optional CaseRef` means that a normative contract or invariant can be a subject without fabricating an executable fixture; it does not allow an executable obligation to omit its case. Family validation decides whether a case is required.

### Observation vocabulary

Raw observations are a tagged product rather than the eventual four-way verdict:

- `Produced(value_or_digest_ref)` records a value-bearing route;
- `Refused(stage, exact_code, detail_ref)` records a typed expected or unexpected refusal;
- `InvariantHeld(receipt_ref)` and `InvariantViolated(stage, exact_code, detail_ref)` record structural checks;
- `Unavailable(stage, reason_code, environment_ref)` records a route that could not be attempted in the required environment;
- `ReachedFailure(stage, exact_code, detail_ref)` records a route whose prerequisites were present and whose reached stage failed;
- `ProofEstablished(theorem_ref, assumptions_ref, checker_ref)` and `ProofRefuted(counterexample_ref)` preserve proof semantics;
- `NormativeStatement(statement_ref, authority_ref, scope_ref)` refers to authority supplied by its actual owner rather than asserting it; and
- `NoObservation(reason_code)` is explicit absence and is never synthesized from a missing file.

`Refused` is not intrinsically failure. The obligation's matcher decides whether the exact refusal is the promised result. `Unavailable` is never a defect and an availability requirement remains unobserved until its required lane supplies evidence; when promised prerequisites are present and a reached stage fails, the producer must emit `ReachedFailure` rather than relabel the defect unavailable. `ReachedFailure` cannot be downgraded to unavailability.

### Verdict vocabulary

The evaluator emits exactly the accepted four semantic outcomes:

- `Passed`: the observation matcher passed and the complete evidence predicate is satisfied;
- `Failed`: the required route ran and produced a wrong value, wrong or missing refusal, disproved invariant, reached-stage failure, or a result that violated the obligation; an invalid receipt is instead an audit failure with no trusted verdict;
- `NotObserved`: no valid current evidence set can evaluate the proposition, including absent, unavailable, expired, or below-requirement evidence; and
- `NotApplicable`: an accepted applicability authority explicitly excludes this subject/case from this profile.

There is no `ExpectedUnsupported`, `Skipped`, `Ignored`, `XFail`, or defaulted `NotApplicable`. Unsupported required capability is `NotObserved` until attempted and then `Failed` if the reached route refuses where the profile required success. An expected refusal is `Passed` only when the obligation named that exact typed refusal before the run.

### Evidence atoms

Evidence kind is an unordered tagged set. One receipt may contribute several atoms only when each atom names its own authority and subject:

- `Semantic` — semantic construction, inference, or evaluation under an exact semantic authority;
- `Reference` — comparison or evaluation under an exact independent oracle and numerical contract;
- `Compiled` — accepted compilation output under exact source, target, and toolchain identities;
- `Executed` — a route committed and completed under exact program, runtime, and device identities;
- `Measured` — a measurement with workload, metric, boundary, procedure, and raw record;
- `ExhaustiveFinite` — every member of an explicitly identified finite universe was checked, with cardinality and enumerator identity;
- `Proof` — a theorem under explicit assumptions and proof/checker identity;
- `NormativeAuthority` — a statement supplied by the governing owner within an exact scope; and
- owner-specific evidence references admitted only through a versioned typed atom schema with a declared matcher, validator, and canonical identity; opaque payloads and callbacks cannot satisfy a requirement.

These tags do not imply one another globally. A `Measured` atom normally cites an `Executed` receipt, but it cannot substitute for `Proof`; `Compiled` cannot substitute for `Executed`; `NormativeAuthority` is not stronger or weaker than a device run—it answers a different question. Where a family has a sound implication, such as accuracy's accepted discharge rule, the family-owned obligation constructor encodes that explicit alternative.

### Evidence requirements

The common canonical predicate algebra is intentionally small:

```text
Requirement :=
    Atom(typed evidence predicate)
  | All(non-empty canonical set of Requirement)
  | Any(non-empty canonical set of Requirement)
```

An atomic predicate may require:

- an exact evidence kind and producer role;
- an exact producer authority or an exact versioned authority-set identity supplied by the obligation owner;
- exact subject, case, oracle, numerical contract, target, toolchain, plan, artifact, runtime route, device/environment, or proof-assumption identities;
- an exact outcome matcher, including a typed refusal code and stage;
- freshness under a named owner policy; and
- an exact finite-universe identity and cardinality where exhaustive evidence is claimed.

There is no generic `Not`, numeric minimum maturity, wildcard authority, wildcard context, or implicit coercion. Negation makes absence satisfy requirements; ranks make incomparable evidence substitute; wildcards make a receipt silently generalize. A family needing a negative proposition declares a positive exact matcher such as `Refused(validation, shape.rank-mismatch)`.

`All` and `Any` are canonical unordered sets: duplicate members are invalid, empty sets are invalid, and encodings sort by child identity. This prevents construction order from changing an obligation while keeping alternative evidence routes explicit. A family may refuse `Any` entirely when its authority permits no alternatives.

The recursive representation is governed before allocation: schema version, maximum depth, maximum node count, maximum atom/context bytes, and maximum acceptable-alternative count are explicit inputs to validation and participate in the evaluator identity. Cycles, over-budget values, and size arithmetic overflow fail audit before canonical sorting or evaluation. Evaluation is lazy over the explicit profile population and indexed by subject/case/atom kind; the design never materializes a universal feature-by-case-by-target product.

### Applicability

Applicability is evaluated before evidence sufficiency but is not inferred from evidence. The profile selects an owner-derived subject and carries either:

- `Required(applicability_scope_id)`; or
- `Excluded(exception_id, policy_authority_id, reason_code, scope_id, revision)`.

Only a valid accepted exclusion yields `NotApplicable` and gray. Missing authority, an expired exception, a mismatched subject, or a hand-authored support label makes the profile invalid; the harness-integrity audit fails and qualification cannot run. The cell remains `NotObserved` for display because no legitimate proposition was evaluated—it is never silently converted to gray.

### Freshness and historical retention

Freshness is a predicate, not deletion and not wall-clock intuition. Each evidence atom retains its production source revision and immutable context. The obligation names a policy owner and policy revision that decides whether that evidence remains current for the selected profile revision. Typical policies are exact-source, unchanged-subject-identity, unchanged-oracle-and-toolchain, or a named expiry epoch owned by the measurement contract.

An atom that fails freshness remains in historical evidence and in regression history. It simply cannot satisfy current qualification. Therefore one cell may show a historically passed observation and a current `NotObserved` verdict without rewriting the old receipt. A refresh adds evidence; it never mutates the historical record.

## Derived presentation and command consequences

Color is derived after receipt validation:

| condition | color | command consequence |
| --- | --- | --- |
| `Passed` and the exact requirement is satisfied | green | may satisfy `qualify` |
| `Failed` | red | fails `qualify`; fails `regress` when the same comparable obligation/context previously passed, while a newly observed red is reported separately as knowledge gained |
| `NotObserved` | yellow | fails `qualify`; does not fail harness-integrity merely for being yellow |
| `NotApplicable` with valid accepted exclusion | gray | outside that profile only; still present in the system universe |
| invalid receipt, profile, authority, or denominator | no trusted color | fails `audit`; no qualification verdict may be published |

A yellow-to-red transition is an increase in knowledge: the route became observable and exposed a defect. `regress` must report it separately from loss of established evidence and must not encourage keeping the route unexecuted. Conversely, green-to-yellow under identical obligation, context, and freshness policy is evidence loss and is a regression.

No scalar completion value is authoritative. A report may show exact counts by verdict, obligation family, and required predicate, but it must preserve the vector and denominator identity. Ticket counts, test counts, and weighted percentages are navigation only.

## Worked obligations

### Semantic/reference agreement

**Obligation.** Structural case `reindex.nan-payload-transport.v1` must infer its declared output and the independent standard reference evaluator must return the literal expected bits under the exact numerical contract.

**Requirement.** `All(Semantic(authority=S, case=C, matcher=inference), Reference(authority=R, oracle=O, context=numerical-contract N, matcher=exact-bits))`.

Correct inference plus a missing reference atom is yellow, not green. Two drivers sharing a case declaration do not merge authorities; literal expected payloads remain oracle-owned. A reference mismatch after both routes run is red.

### Optimizer preservation

**Obligation.** Rewrite/strategy subject `Q` must be production-reachable, retained beside a valid neighbour, selected under a subject cost/profile perturbation, and preserve meaning under an independent oracle.

**Requirement.** `All(Compiled(production route and exact provider/rewrite identity), owner receipt for retention, owner receipt for selection perturbation, Reference(independent oracle), Executed(exact selected plan and completion))` for a profile that claims execution-level preservation. A construction-only `compile().is_ok()` atom satisfies only its exact compilation predicate; it cannot substitute for retention, selection, or preservation.

### Compile-only availability

**Obligation.** A named Metal source/target case must compile under toolchain `T` when a qualified compilation lane is available.

If no qualified toolchain resolves, the raw observation is `Unavailable(compilation, toolchain-unavailable, environment E)` and the verdict is `NotObserved`/yellow. If the toolchain resolves and `metal` rejects the source, the observation is `ReachedFailure(compilation, diagnostic, detail)` and the verdict is red. An early return supplies no atom and is also yellow, while harness audit reports the missing classified outcome.

### Real execution

**Obligation.** Artifact program `P` must pass eligibility/preparation, commit one runtime route, complete terminally, and match oracle `O` under grouping `G`.

**Requirement.** `All(Compiled(artifact A), Executed(route R, terminal completion C), Reference(oracle O, grouping G), exact environment/profile context)`. Compilation without terminal completion is yellow; reached dispatch failure is red; comparison against the wrong grouping is red even if the bits are inside the numerical contract's broader permitted set.

### Normative ownership

**Obligation.** Target profile `P` must be authorized to claim native translation authority.

Only `NormativeAuthority(owner=the accepted authority, statement=translation guarantee, scope=P)` can satisfy it. Artifact equality, successful compilation, successful execution, and repeated measurements are not alternatives unless the governing contract later authorizes one explicitly. The current `UnknownNativeTranslationAuthority` therefore cannot satisfy a qualification requirement and would remain yellow in any profile that required it; a device run cannot turn it green.

### Performance measurement

**Obligation.** Retained claim `C` states metric `M` for workload `W` against baseline `B` in environment `E` under procedure `H`.

**Requirement.** `All(Measured(claim=C, metric=M, workload=W, baseline=B, environment=E, procedure=H, raw-record digest), Executed(subject identities named by C))`. The result is empirical and bounded even when every repetition passes. It does not satisfy proof, exhaustive, portable, or normative performance requirements. A changed target profile, plan, compiler, device, workload, or harness is a different context unless the performance owner explicitly defines a valid equivalence.

## Negative controls

Each downstream implementation must demonstrate these by perturbing the subject or receipt, never the assertion:

| perturbation | required result |
| --- | --- |
| replace the expected oracle/producer authority with another valid authority | receipt remains valid for its own claim but does not satisfy the obligation; cell yellow with `wrong-authority` predicate detail |
| change target, numerical contract, case manifest, selected plan, toolchain, or environment | no context wildcard applies; cell yellow with the exact mismatched field |
| advance the owner freshness policy or subject revision without a successor receipt | historical atom remains queryable; current cell becomes yellow and regression reports evidence loss |
| remove one required atom from an `All` expression | cell becomes yellow and names the missing kind; no default fills it |
| offer empirical measurement where proof or normative authority is required | cell remains yellow; no rank/coercion path exists |
| change an expected typed refusal's stage or code | observed route is retained but the obligation is red |
| tamper with an atom, requirement, profile, or evidence-set identity | `audit` fails before a trusted color or qualification result is emitted |
| add a family obligation that its owner manifest does not classify | owner census or universe audit fails; denominator cannot silently shrink |

## Decision packet

### Independent derivation from counterexamples

Starting from failure cases rather than from the candidate schemas reaches the same structure:

1. A source can compile and then execute incorrect synchronization or ABI behavior, so `Compiled` cannot imply `Executed` or replace it.
2. A bounded device corpus can pass while an unmeasured worst case violates the contract, so `Measured` cannot imply `Proof`, `ExhaustiveFinite`, or `NormativeAuthority`.
3. A real normative guarantee can discharge a requirement without executing a case, so execution is not globally above or below normative authority.
4. The same `Refused(stage, code)` observation passes an expected-refusal obligation and fails a required-success obligation, so raw observation cannot carry verdict.
5. An unavailable host and a qualified host whose compiler rejects valid source both produce no executable result, but only the latter is a defect, so absence cannot share a failure variant.
6. Optimizer preservation requires several coexisting facts—production reachability, retention, selection under perturbation, exact identities, and independent semantic agreement—so a single evidence tag cannot express sufficiency.

These counterexamples independently require exact typed atoms, conjunction, explicit alternatives, family-owned matchers, context/authority binding, and separate observation/verdict. They also independently disprove a total order and a flat unordered-set-only design.

| candidate | verdict | reason |
| --- | --- | --- |
| ordered maturity enum | **eliminated** | proof, normative authority, execution, measurement, and exhaustive evidence are incomparable; a rank permits false substitution and cannot represent conjunctions |
| unordered evidence set alone | **eliminated as complete design** | preserves incomparability but cannot state what suffices, match exact refusals, bind context/freshness, or distinguish alternatives from conjunctions |
| common typed requirement predicate over validated atoms | **frontier component** | gives fail-closed matching, canonical identity, and common command semantics with no rank |
| family-specific schemas only | **eliminated as complete design; retained as constructors** | preserve owner meaning but fragment audit/report semantics and prevent one canonical receipt join |
| hybrid family declarations compiled to the common predicate algebra | **selected nondominated design** | preserves owner authority while providing common exact evaluation and identity; costs distributed family adapters and careful validation |
| blanket deferral | **dominated for the common core** | the necessary conjunction/alternative/exact-match semantics are already established; deferral remains mandatory for a family that cannot map without loss or a new public contract |

The selected design's strongest counterargument is maintenance cost: each family must own declarations and an adapter into the common atoms, while a generic rank would be cheaper. That cost is real and intentional because the cheaper design moves family semantics into implicit conversions. Evidence that would reverse the selection is a proof that all current and anticipated obligations share one sound substitution order and one authority/context model; the audited counterexamples—empirical versus normative accuracy, compilation versus execution, expected refusal versus failure, and applicability versus artifact equality—already falsify that premise.

On the required comparison dimensions, the hybrid is the sole surviving complete candidate. It is fail-closed because no evidence coercion, context wildcard, or applicability default exists; maintainable because family meaning stays with its owner and the common evaluator stays small; compatible because new typed atom/predicate versions can be added without reinterpreting old receipts; and host-bounded because populations are lazy and exact while expression depth/count/bytes are governed before allocation. Its cost is additional family adapters, indexed joins, and retained historical atoms. That cost is preferable to a cheap global enum whose silent false substitutions corrupt correctness. Kernel performance is unaffected because no runtime or kernel fast path consumes the algebra.

The common algebra remains deliberately less expressive than arbitrary Boolean logic. A future obligation requiring negation, quorum, temporal sequencing, probabilistic confidence, or quantified ranges must first demonstrate why a new typed node is sound and canonical. It must not encode the feature through an opaque callback or generic expression escape hatch.

## Identity and versioning consequences

- `SubjectRef` is owner-minted and independent of tests, profiles, and evidence. A subject revision changes only under its owner rule; removal creates a tombstone rather than erasing history.
- `CaseRef` is manifest-minted. Shared inputs do not imply shared obligation or oracle identity.
- `obligation_id` is stable across editorial changes; `declaration_revision` changes when matcher, evidence predicate, applicability, or family semantics change, and the canonical declaration-content identity commits to that revision and content.
- An evidence atom identity commits to its observation, producer authority/role, complete context, subject/case references, source revision, and referenced payload digests. Validity is established by the owning validator and audit rather than asserted by a self-declared field. A green bit and presentation color never enter it.
- An evidence-set identity is a canonical set root over validated atom identities. Removing or substituting an atom changes it.
- An evaluated-cell identity commits to obligation revision, evidence-set identity, and evaluator/verifier identity. Re-evaluation never rewrites the source receipts.
- Goal-profile identity commits to selected obligation revisions and accepted exclusions. Denominator changes are policy events, not evidence progress.
- Unknown schema tags, missing required fields, duplicate canonical members, unresolved owner identities, and identity-domain mismatches fail audit closed.

The P+K+M+T authority decision governs who may accept the universe/profile/evaluator/oracle/baseline roots; this algebra does not weaken or duplicate that authority. It specifies what those roots must bind.

## Downstream requirements

1. The owner-private boundary decision must expose immutable owner manifests or snapshots sufficient to construct `SubjectRef` and family obligations without making compiler internals a public end-user API.
2. Structural, optimizer/planner, artifact/proof/publication, runtime/cache, and performance manifests must define family-owned matchers and evidence predicates and must stop if the common algebra would require defaults or loss.
3. The canonical receipt-join design must emit validated atoms by reference to existing owner identities, not copy their fields into a second authority.
4. Machine outcome design must preserve stage, typed unavailability, and reached-stage failure so observations can be evaluated without guessing.
5. Audit/regress/qualify command design must implement the color and transition rules above, preserve denominator changes separately, and refuse invalid roots before reporting qualification.
6. The first goal profile may select only obligations whose subjects, applicability, evidence predicates, and current evidence gaps are explicit. Unknown owner populations remain blockers rather than empty sets.
7. Any later Rust schema or public boundary is a separate Tom decision. This report authorizes neither.

## Stop boundary and unsupported cases

The design does not resolve the public/private owner observation mechanism, canonical receipt serialization, authority providers, first profile, performance-claim population, or family-specific obligations. It cannot represent quorum signatures, terminal transparency history, or external attestations; those protect the roots this algebra consumes and remain in the P+K+M+T design tickets. It also does not claim that every current evidence type has been reconciled or should be replaced.

A downstream family must stop and split if it cannot express its real authority with `Atom`/`All`/`Any`, exact matchers, and explicit applicability without a lossy coercion, default, opaque callback, or new public contract. That stop is a long-term compatibility feature: widening a versioned algebra deliberately is safer than teaching one permissive escape hatch to call everything green.
