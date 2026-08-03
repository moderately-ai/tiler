---
id: place-index-refinement-evidence-under-an-ir-owned-verifier
title: Place index-refinement evidence under an IR-owned verifier
status: in-progress
priority: p1
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes:
  - implementation/ir
  - implementation/compiler
  - contracts/foundation
  - contracts/artifacts
  - implementation/metal-aot
  - contracts/optimizer
  - contracts/numerics
  - research/extensions
shared_scopes:
  - project/tickets
claimed_from: todo
assignee: agent-index-receipt
lease_expires_at: 1785786159
---

## User-visible outcome

A dependency-neutral authority can eventually mint one opaque checked receipt that binds an exact verified semantic graph, its typed graph-local occurrence, the governed numerical contract, and the verified index region that realizes it. This ticket establishes that authority; the dependent `bind-stage-coverage-to-index-refinement-identity` owns program-builder consumption, stale/wrong-domain receipt refusal, and program/artifact identity changes.

## Correctness derivation

**Fact:** `IndexRefinementIdentity` and the lowering-capability authority are currently owned by `tiler-compiler`, while `KernelProgramBuilder`, `program::SemanticOccurrence`, and `SemanticGraphIdentity` live in `tiler-ir`. The dependency direction is `tiler-compiler -> tiler-ir`; storing the compiler type in IR would create a cycle.

**Fact:** a `VerifiedIndexRegion` contains index/access structure, scalar operations, boundary types/shapes, and scalar-registry authority. It contains no governed association to a tensor-semantic operation, semantic graph, graph-local occurrence ordinal, host attributes, or resolved numerical contract.

**Fact:** the compiler lowering registry resolves an operation/signature to a provider-attributed `LoweringCapabilityAuthority`. That capability and the compiler's exact occurrence are the only current authorities relating a provider-emitted region to an operation. Interface, scalar-containment, bounds, and write-ownership checks can reject malformed emissions, but cannot distinguish two same-interface operations or two occurrences differing only in attributes or numerical contract.

**Measurement:** commit `cbbe0432170259db2ddb388082ad0b5e32b111fd` implemented the narrower structural verifier as a concrete draft. Review constructed the same-interface counterexample: changing operation, attributes, or numerical contract left every verifier predicate true, so a receipt still minted. The draft also carried only caller-supplied opaque occurrence bytes; it could not prove their correspondence to `program::SemanticOccurrence(u32)` in one `SemanticGraphIdentity`. The implementation and its identity-domain step are withdrawn by the correction commit following this record.

**Inference:** folding operation, attributes, contract, graph bytes, or an ordinal into identity does not prove their association. A public caller could supply a self-consistent lie. An opaque receipt must be minted from an authority that already governs both the checked semantic subject and the lowering realization, not from independently supplied identity components.

## Eliminated candidates

1. **IR structural verification plus copied semantic bytes — eliminated by correctness.** It accepts a same-interface region for the wrong operation, attributes, contract, graph, or occurrence ordinal.
2. **An unchecked or publicly constructible compiler-token wrapper in IR — eliminated by correctness.** Private fields cannot be minted across crates; a public byte constructor recreates the forbidden pairing.
3. **Keep the receipt compiler-owned and store it directly in `tiler-ir::program` — eliminated by dependency direction.** It introduces an `tiler-ir -> tiler-compiler` cycle.
4. **Treat provider emission or registration alone as proof — eliminated by the accepted refinement contract.** Resolution says which authority was selected; successful construction alone proves neither semantic association nor realization.

## Decision accepted by Tom — 2026-08-03

Tom approved the recommended authority move in the T3 Code orchestration
conversation: the dependency-neutral portion of lowering/refinement authority
may move into `tiler-ir`, so an IR-owned verifier can bind a checked
semantic-program occurrence and the resolved lowering authority to the emitted
region.

The approval covers the ownership split below, not the eventual concrete public
API. The exact verifier, receipt, subject, identity, and error boundary remains
a tested draft that returns to Tom under ADR 0075 before acceptance.

- **Accepted — move the minimal authority.** `tiler-ir` owns the sealed subject and receipt: exact `SemanticGraphIdentity`, typed `program::SemanticOccurrence`, operation/attributes, a dependency-neutral numerical-contract identity, resolved signature, admitted scalar/type authority, and the checked region. The compiler continues to own registry search, provider selection, frontier policy, and explain attribution, but its registered capability carries an IR-owned admitted realization authority that the verifier can consume. This preserves dependency direction and makes downstream receipt consumption non-forgeable, at the cost of a larger public IR boundary than the withdrawn draft.
- **Rejected — keep all lowering authority compiler-owned.** Executable-program coverage could not retain a proof-derived per-occurrence receipt in the current crate graph. A separate lower proof crate or a relocation of executable-program assembly would become a new architecture task and public boundary before this path could proceed.

A separate shared proof crate is not recommended: it adds a crate and another dependency seam while the governed subjects—semantic graph identity, program occurrence, scalar/index region, and program builder—already belong to `tiler-ir`.

## Required evidence after the decision

- The receipt subject is derived from a verified semantic program and typed graph-local ordinal; no independent ordinal/graph byte pairing exists.
- Same-interface wrong-operation, wrong-attribute, and wrong-contract cases refuse before receipt minting, with each check deliberately perturbed once.
- Two occurrences over reusable region content remain occurrence-distinct, and changed verified-region content moves identity.
- A proof gap mints no receipt.
- The exact public verifier/type/error boundary returns to Tom as a tested draft; this ticket does not self-accept it.
- Targeted `tiler-ir` and `tiler-compiler` nextest, doctests, Clippy, formatting, ticket lint, diff checks, and scope guard pass.

## Scope expansion authorized by Tom — 2026-08-03

Tom explicitly approved adding `implementation/metal-aot`,
`contracts/optimizer`, `contracts/numerics`, and `research/extensions` in the T3
Code orchestration conversation. The correction changes the public installation
boundary back to the exact `(lowering, scalars)` pair: request verification
derives realization-law authority from the exact program semantic registry and
requires full lowering/scalar/program semantic and law-sidecar coherence, so a
lowering installer cannot supply or replace it. That replacement updates the
out-of-crate install conformance in `prototypes/serial-sum-compile/src/main.rs`
and the contract/research sentences that described the rejected triple. No
legacy overload is retained.

## Corrected authority shape and public draft inventory

**Fact:** generic region structure, scalar-registry containment, and lowering
registration cannot prove semantic realization. The rejected fixed point
positively demonstrated the counterexample by minting a multiply receipt from a
same-interface add region.

**Inference:** the semantic proof must be independent of the selected lowering
provider and must not be an arbitrary callback that can inspect and approve the
candidate it is meant to check. The surviving shape is an immutable typed law
sidecar registered in the semantic provider's same atomic transaction as its
operation. The sidecar is excluded from semantic graph/snapshot identity because
it does not change Layer 1 meaning, but has a separate bounded, versioned law
snapshot identity. IR interprets the law without access to the candidate, builds
the structurally verified expected canonical region, then compares its canonical
identity with the selected lowering's region. The current exact-canonical rule
deliberately rejects semantically equivalent alternate logical index forms;
physical alternatives remain a later planning concern.

**Proposal — tested public draft, not accepted here:** `tiler-ir::index` adds
`IndexRealizationLaw`,
`FrozenIndexRealizationLawRegistry`, `IndexRealizationLawRegistryIdentity`,
`NumericalContractIdentity`,
`IndexRefinementBoundary`, `IndexRefinementSignature`,
`IndexRefinementSignatureSide`, `MAX_INDEX_REFINEMENT_SIGNATURE_VALUES`,
`MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS`,
`IndexRealizationAuthority`, `IndexRefinementSubject`,
`ResolvedIndexRealization`, ordered `OperandBinding`/`ResultBinding`, pending and
completed `PendingIndexRefinementReceipt`, `IndexRefinementReceipt`,
`IndexRefinementReceiptIdentity`, and `IndexRefinementVerificationOutcome`, plus
typed verification errors. `IndexRealizationLawError` is deliberately
crate-private in the corrected draft: it describes failure inside the sealed
law interpreter and is projected through the public verification error rather
than becoming a second public refusal vocabulary.
`SemanticRegistryRegistrar::register_index_realization_law` is the only public
law-registration path and requires the operation in that same transaction,
nonzero revision, unique ownership, and bounded count/bytes. Residual proof
completion adds read-only `IndexDomainProofAuthority`,
`IndexDomainProofEvidence`, `IndexDomainDisproof`, `IndexDomainProofClaim`,
`IndexDomainProofAssessment`, `IndexRefinementDomainProof`,
`IndexRefinementDomainProofIdentity`, a bounded
`IndexDomainProofBudget`, the public hard ceilings
`MAX_FINITE_DOMAIN_PROOF_CELLS` and
`MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES`, the structural pending-state ceiling
`MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS`, and typed atomic
`IndexDomainProofRefusal`/`IndexDomainProofRefusalKind`; the refusal implements
`Display` and `Error`. There is no proof callback
and no public proof, disproof, or authority constructor: IR runs its closed
exact-finite evaluator and alone seals the opaque receipt. The compiler chooses
only a bounded work budget and projects IR's assessments into explain output;
the superseded compiler evaluator, `AuthorizedIndexDomainProof`, and duplicate
identity encoder are removed. IR disproof conversion preserves the exact
counterexample and enumerated point ordinal. The `SoundProof` lane remains
unsupported until a closed, validated certificate language exists.

**Proposal — compiler public draft, not accepted here:** the provider context
exposes only the narrow `IndexAccessOccurrence`; `session::InstalledCapabilities::installed`
accepts `(lowering, scalars)` while request verification derives the immutable
law snapshot from the exact program semantic registry and validates the full
pair. `LoweringCapabilityRegistryBuilder::new` is now fallible and reports
`LoweringRegistryError::RefinementAuthority`; request mismatch is
`UnsupportedCapability { phase: "capability", rule:
"semantic-authority-pairing" }`. Compiler
registry search, selected-provider provenance, explain, proof-policy
implementation, and planning remain compiler owned. The rejected three-argument
installation spelling and arbitrary verifier install hook are removed.

The public draft removes the previous occurrence façade types
`OccurrenceBoundary`, `LoweredOccurrence`, `OccurrenceValueId`,
`OccurrenceOperand`, and `OccurrenceResult`, plus the compiler-owned
`SemanticOccurrenceIdentity`, `NumericalContractIdentity`,
`SemanticOccurrence`, `OperandBinding`, and `ResultBinding`. It changes
`LoweringCapabilityAuthority::refinement`, makes the provider-facing
`IndexAccessOccurrence` the narrow context, keeps
`IndexAccessLoweringContext::new` crate-private, adds the exact subject and law
registry to `refine_index_region`, and replaces `IndexRefinement::occurrence`
with the sealed receipt. The pending receipt retains its subject rather than a
caller-supplied scalar registry. Proof evidence, claims, verification outcomes,
and refusal kinds remain exhaustive enums in this draft: adding a semantic case
must break internal consumers at compile time rather than let wildcard arms
silently misclassify new proof meaning. This exact public boundary still returns
to Tom; the ticket does not accept it.

The semantic-registry public inventory also adds
`SemanticRegistryResource::{IndexRealizationLaws,
IndexRealizationLawBytes}` and the corresponding `RegistryError` law
registration failures: exactly `IndexRealizationLawWithoutOperation`,
`DuplicateIndexRealizationLaw`, and `ZeroIndexRealizationLawRevision`.
`LoweringCapabilityAuthority::refinement()` is an
addition distinct from the removed compiler-owned occurrence/refinement types.
The verification error inventory additionally includes
`ResidualObligationsTooLarge` and `SubjectRealizationLawMismatch` with exact
typed refusal at their respective allocation and authority boundaries.
The shared schedule vocabulary adds `F32NumericalContractKey`,
`NumericalContractKeyError`, and `F32_NUMERICAL_CONTRACT_KEY_DOMAIN`; the
`F32NumericalContractKey::new` draft accepts only the behavioural dimensions
because governed `f32` arithmetic and its canonical NaN payload are invariants,
and `NumericalContractIdentity` implements infallible
`From<F32NumericalContractKey>` for an already-validated key. The
compiler removes `RefinementError::{AliasedOperandInconsistent,
OccurrenceFacts}` and `LoweringRegistryError::UnboundOperand`, and adds
`RefinementError::IrVerifier`.
Only the `IndexDomainProofEvidence::ExhaustiveFinite` variant is
`#[non_exhaustive]`; the evidence, claim, outcome, and refusal-kind enums remain
exhaustive as stated above.
`IndexRealizationAuthority` retains a crate-owned canonical identity for
equality and sealed resolution, but exposes no unused byte accessor. Every
public derived byte identity in this draft is instead an opaque named type with
an `as_bytes` reader, following ADR 0074.

## Identity step and blast radius

The new IR identity domains are append-only, independently tagged v1 domains:
semantic subject, admitted realization authority, frozen realization-law
registry, sealed resolution, residual proof, and final receipt.
Compiler refinement content and occurrence bindings use their corrected v2
domains; their encoders bind the IR receipt and proof identities rather than
copying semantic claims.

The derived law registry is request authority, so omission from request identity
would permit replay under another law set. The capability snapshot
schema therefore steps 1 → 2 and the request-subject domain steps
`tiler.compiler.request-subject.v3` → `v4`, appending the length-framed frozen
law-registry identity after lowering authority. On this corrected tree the only
pinned value moved is the deterministic explain qualifier to
`3a2bda87fc26f899`; it was recomputed from the current tree rather than copied
from either rejected fixed point.
No program, artifact, schedule, kernel, or cache identity changes here: the
dependent `bind-stage-coverage-to-index-refinement-identity` still owns receipt
consumption and program/artifact identity.

## Correction evidence

**Review correction — 2026-08-03:** exact fixed-point review found two
fail-closed defects in the first corrected draft. First, the compiler derived
the realization-law registry from the installed scalar registry rather than the
exact `SemanticProgram` registry. Two registries can have equal semantic
snapshot identities while carrying different law sidecars, so registry B's add
law and add-emitting lowering could be substituted for program A's multiply
law. Verified requests now derive and retain the law registry from the exact
program registry, bind its separate identity into each request subject, and
preflight the installed lowering/scalar authorities against the program's
semantic snapshot. The deliberate A/B fixture has equal semantic snapshots,
makes both scalar operations evaluable, and reaches the exact
`SemanticRealizationMismatch` under A's retained multiply law.

Second, public `ResolvedIndexRealization::complete(pending, verifier)` let an
arbitrary callback return a purported sound proof with arbitrary bytes. The
callback, public authority/evidence/disproof constructors, `Sound` evidence
variant, and duplicate compiler evaluator are removed. Completion now accepts
only `IndexDomainProofBudget` and invokes IR's fixed exact-finite algorithm.
Tests prove a valid residual exhaustively, return a deterministic exact
counterexample for a false predicate, retain `Unknown` for symbolic input and
for budget exhaustion, and reject zero or over-hard-limit budgets. A repository
search over `crates/`, `docs/`, and this ticket finds no remaining verifier
trait, sound-proof constructor/variant, or public proof-authority constructor.

**Measurement — 2026-08-03, post-review fixed point:** the deliberate
multiply-to-add lowering perturbation reaches a typed
`SemanticRealizationMismatch`; the ordinary compile path refuses it before
candidate planning. The separate equal-snapshot A/B regression reaches the
same mismatch through verified request and lowering resolution. The eight
governed realization laws independently reconstruct the same canonical regions
as their current lowering providers. `tiler-ir` nextest passed 675/675;
`tiler-compiler` passed 590/590 with one configured skip; combined
IR/compiler/out-of-crate prototype nextest passed 1,284/1,284 with that same one
configured skip under Xcode 26.6 (17F113), selected per command with
`DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`. No global toolchain
setting was mutated.

**Fact:** affected-crate Clippy with warnings denied, both crates' doctests,
formatting, `git diff --check`, and `tkt lint` pass. The request qualifier pin was
recomputed on this corrected tree as `3a2bda87fc26f899`. Scope guard is run after
the correction commit so it inspects the branch diff rather than the integration
checkout's ticket body.

**Unsupported case:** the law vocabulary is closed to the currently implemented
f32 realization templates. An operation without a same-transaction law, an
unrecognized numerical-contract domain, or a semantically equivalent but
noncanonical logical index form refuses explicitly. Broadening any of those is a
new reviewed law/template boundary, not a lowering-provider escape hatch.

**Second fixed-point correction — 2026-08-03:** review found that public
constructors could still pair semantic authority A with a scalar registry built
over B, and that the closed evaluator's counterexample payload and proof budget
did not bound wide domains or arbitrary-precision integer work. IR law-registry
construction and realization admission now reject semantic/scalar incoherence,
including equal semantic snapshots with unequal law sidecars; the compiler
lowering-registry builder is fallible for the same reason, and request preflight
maps the deliberate A/B mismatch to `capability/semantic-authority-pairing`.
Direct perturbations prove each constructible boundary can say no. Receipt-time
duplicate checks were removed because immutable law and lowering authorities
cannot be constructed incoherently through their checked constructors.

The evaluator now encodes an exact counterexample by its region-bound point
ordinal, returns symbolic tensor-axis extents and unrepresentable work counts as
`UnsupportedFragment`, proves empty domains before multiplication, and governs
both cells and cumulative integer-byte work across the whole completion call.
Obligations sharing an exact ordered static domain are evaluated together over
one union DAG, with access domains and predicate extents cached once. Cell cost
includes semantic planning and, per point, coordinate set/advance, DAG-node and
edge traversal, memo clearing, and every predicate; integer cost uses the
documented conservative byte-width formula and is multiplied by exact point
count. A group reserves both resources atomically, and exhaustion is propagated
in canonical obligation order with the cumulative requirement and the original
whole-call limit. The hard-structure-bounded assessment vector is mandatory
output overhead rather than caller-budgeted semantic work. This grouping and
cost model do not change the mathematical v1 proof identity. Checked arithmetic
prevents a saturated value from being reported as an exact requirement.
Regression tests cover wide-rank cell refusal, a real region whose evaluator
reaches a 4,097-byte exact intermediate while returning bounded disproof
evidence, static-domain/symbolic-axis refusal, zero-domain point counting, and
both budgets' must-stop cases.

**Measurement — second fixed point:** combined `tiler-ir` and
`tiler-compiler` nextest passed 1,271/1,271 with one configured skip. Compiler
doctests passed 2 ordinary and 7 compile-fail cases; IR doctests passed 8
ordinary and 1 compile-fail case with one ignored example. Affected-crate
Clippy with warnings denied, formatting, `git diff --check`, and `tkt lint`
passed. Guard over the 25-file population from the true claim base reported no
under-declared scope and warned about live declared-area collisions.

**Third fixed-point correction — 2026-08-03:** fixed-point review found four
remaining fail-closed gaps. Numerical-contract identities now use one exact
IR-owned v2 codec rather than compiler prefix recognition; all 2,304 coherent
caller-statable vectors round-trip injectively through compiler and IR, while a
12-case malformed matrix and ineligible derived provenance refuse. Signature
iterators stop after the first over-limit value, and 4,097 raw scalar-operation
declarations refuse before deduplication. Realization-law sidecar accounting now
uses the exact shared row encoder—including provider identity and the count
framing—and freeze asserts the tracked total equals the canonical sidecar.

The corrected codec admits only the three exact rendered lengths reached by
that exhaustive vector population (98, 100, and 102 bytes), rejecting any other
length before scanning or allocating decoded bytes; maximum-plus-two and a
two-megabyte even lowercase-hex payload prove the decoder is never entered.
`F32NumericalContractKey::new` no longer asks callers to repeat the invariant
canonical `f32` arithmetic NaN payload, while the compiler still refuses an
internally incoherent contract before minting a key. A validated key converts
infallibly into `NumericalContractIdentity`. ADR 0074's typed-identity rule is
applied to the frozen law registry and sealed domain proof, with unchanged
canonical bytes; the unused public raw-byte accessor on
`IndexRealizationAuthority` is removed.

Completion now owns one cumulative ledger and groups obligations by exact
ordered static domain. The earlier dense shared-DAG byte counts measured
retained integer width, not a conservative bound on arbitrary-precision work,
and are withdrawn. The corrected num-bigint-0.4.8-specific limb-work formula
makes that pathological fixture refuse above the 64 MiB hard limit before
evaluation. A manageable real shared DAG still proves that one grouped walk
costs less than two separate walks, that the exact grouped charge succeeds, and
that charge minus one refuses with the same requirement. A separate two-domain
fixture proves exhaustion propagates atomically in canonical order. The real
large-intermediate fixture now refuses before evaluation under the work bound;
bounded counterexample identity remains covered by manageable evaluations.

`PendingIndexRefinementReceipt::verify_completion` now revalidates occurrence,
region, scalar authority, interfaces, canonical proof order, and the recomputed
receipt identity before compiler assembly. Crossing a completed receipt between
two distinct real semantic occurrences returns typed
`CompletionReceiptMismatch`; compiler completion maps that refusal through
`RefinementError::IrVerifier` without an assertion or panic. The sealed proof
constructors remain inaccessible to downstream crates, pinned by the new
compile-fail test.

**Measurement — third fixed point:** combined `tiler-ir` and `tiler-compiler`
nextest passed 1,278/1,278 with one configured skip. Affected-crate Clippy with
warnings denied and formatting passed. Compiler doctests passed 2 ordinary and
7 compile-fail cases; IR doctests passed 8 ordinary and 1 compile-fail case with
one ignored example. `git diff --check` and `tkt lint` passed. No identity-domain
version or pinned identity moved: the canonical bytes of every previously valid
numerical key and realization-law sidecar are unchanged; these corrections
reject malformed spellings, account already-encoded provider bytes, and change
only proof execution and completion validation. Guard over the 30-file
population from the true claim base reported no under-declared scope and warned
about declared-area overlap with live work; the coordinator must recheck those
actual branch diffs before integration.

**Fourth fixed-point correction — 2026-08-03:** the operation-specific
canonical realization-law row is now bound privately into both subject and
admitted-authority v1 identities with an explicit absence tag. Resolution and
lowering coherence require exact row equality, so equal semantic/scalar
snapshots cannot substitute a different law or the absence of one. The public
perturbation preserves semantic occurrence authority while removing the law
row; subject identity moves and resolution returns the typed
`SubjectRealizationLawMismatch`. No v2 step is taken because these v1 domains
remain unlanded. The compiler explain qualifier is the only literal derived pin
in the affected request path; it was recomputed by the full compiler suite and
remains `3a2bda87fc26f899`.

Pending receipt construction now refuses more than 6,144 residual obligations
immediately after exact canonical-law equality and before collection/allocation;
the limit is the closed vocabulary's conservative `3 accesses * 1,024 rank * 2
predicates`. Assessments and proofs share one `Arc`-backed authority instead of
cloning its owned provider strings per obligation. A bound-plus-one perturbation
returns the typed actual/limit error.

The exact-finite evaluator now preflights conservative 8-byte-limb work for the
locked num-bigint 0.4.8 implementation, including operands, prior accumulator,
results, multiplication work, and floor-div/mod quotient/remainder work. Every
calculation is checked `u128`; overflow is unsupported before evaluation, and a
num-bigint revision requires re-auditing the source-specific formula. The
1 MiB-by-1 MiB multiplication fixture exceeds the 64 MiB hard limit; negative
floor-div/mod fixtures establish semantic results and exact/charge-minus-one
boundaries. Unsupported roots roll back only their union-plan state while
retaining planning-ledger work, so same-domain siblings remain independently
proved or disproved in canonical order. Resolved `u64` predicate extents are
carried into the plan and no longer re-resolved or allocated as `BigInt` per
point.

**Measurement — fourth fixed point:** `tiler-ir` nextest passed 698/698;
`tiler-compiler` passed 589/589 with one configured skip. IR doctests passed 8
ordinary and 1 compile-fail case with one ignored example; compiler doctests
passed 2 ordinary and 7 compile-fail cases. Affected-crate Clippy with warnings
denied and formatting passed. The targeted mixed-claim, shared-DAG,
negative-div/mod, and overflow set passed 4/4. Successor review added a
large-domain/tiny-budget all-unsupported regression and replaced the private
law-row mutation with public provider/registrar construction: registries with
equal semantic and scalar snapshots but different or absent operation-specific
rows refuse both resolution and lowering coherence. That successor set passed
3/3.
Successor IR doctests again passed 8 ordinary and 1 compile-fail case with one
ignored example; IR Clippy with warnings denied, formatting, `git diff --check`,
and `tkt lint` passed. Guard over the exact 31-file branch population from the
true claim base reported no under-declared scope and warned about live
declared-area collisions; the coordinator must compare those branches' actual
diffs before integration.

**Fifth fixed-point correction — 2026-08-03:** impossible numerical-contract
key lengths now refuse before scanning or decoded-byte allocation. The complete
2,304-contract population reaches exactly 98, 100, and 102 bytes; a 104-byte
maximum-plus-two key and a two-megabyte even lowercase-hex payload invoke a
decoder that panics if reached and both refuse without entering it. The
`F32NumericalContractKey::new` draft no longer accepts the invariant canonical
`f32` NaN payload as a caller argument, while compiler key minting independently
refuses an incoherent internal arithmetic/NaN pair. An already-validated key
converts infallibly to `NumericalContractIdentity`.

ADR 0074's named-identity convention now covers the frozen law registry and
each sealed residual proof through opaque `IndexRealizationLawRegistryIdentity`
and `IndexRefinementDomainProofIdentity` values with `as_bytes` readers. Their
encoders and bytes did not move; compiler request, refinement-content, and
explain consumers now cross the typed accessor explicitly. The unused public
raw-byte accessor on `IndexRealizationAuthority` is removed rather than
publishing a third identity type with no consumer.

**Measurement — fifth fixed point:** combined `tiler-ir` and `tiler-compiler`
nextest passed 1,289/1,289 with one configured skip. Compiler doctests passed 2
ordinary and 7 compile-fail cases; IR doctests passed 8 ordinary and 1
compile-fail case with one ignored example. Affected-crate Clippy with warnings
denied passed. Deliberately replacing the request qualifier pin with zero made
the exact deterministic-trace test report `3a2bda87fc26f899` from this tree;
restoring that recomputed value made the test pass.

## Graph maintenance

`bind-stage-coverage-to-index-refinement-identity` already depends on this ticket and remains blocked. That dependent owns `CoveredOccurrence`, program-builder receipt-domain/staleness validation, and program/artifact identity domains. Do not claim that evidence here or change those identities before this authority is accepted. Update ADR 0071 and artifact contracts only after the exact authority move is accepted.
