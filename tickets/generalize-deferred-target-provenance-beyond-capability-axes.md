---
id: generalize-deferred-target-provenance-beyond-capability-axes
title: Generalize deferred target provenance beyond quantitative capability axes
status: done
priority: p1
dependencies: [admit-an-atomic-subgroup-realization-subject-to-target-profiles]
related: [decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality]
scopes: [implementation/compiler, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, feasibility, provenance, public-boundary, identity, decision, needs-tom]
---
## User-visible outcome

The compiler can defer an exact target obligation without pretending every obligation is an independently declarable quantitative capability axis.

## Original Fact — 2026-08-11

`DeferredPredicate`, `DeferredSet`, `EntryDeferredPredicate`, explain generation, and public `PreparedEntryTargetRequirementRef::capability_axis` all assume a `CapabilityAxis`. Subgroup width must instead be confirmed from the complete accepted `SubgroupRealizationSubject`; adding an independent axis would decompose that atomic subject and permit unsupported partial conjunctions.

## Source-first Fact audit — 2026-08-12

Exact base: `bea61831b35df30ca7a6d758dfb263bc37c673f8`.

- **Verified — the executable deferred path is axis-only.** `target/feasibility.rs`, anchors `pub(crate) struct DeferredPredicate` and `pub(crate) struct DeferredSet`, stores `CapabilityAxis` beside `PreparedEntryTargetRequirement` and sorts that population by `(phase, axis)`. `program.rs`, anchor `pub(crate) struct EntryDeferredPredicate`, forwards that same value to an exact program entry. `frontier.rs`, anchor `fn encode_proposal_identity`, folds the axis key, required quantity, and complete requirement. `pipeline/trace.rs`, anchor `ExplainEvent::DeferredTargetRequirement`, derives the rule and predicate key from the axis. `session.rs`, anchor `pub fn capability_axis`, exposes the assumption publicly.
- **Verified — an independent subgroup axis would lose accepted atomicity.** The accepted boundary in `admit-an-atomic-subgroup-realization-subject-to-target-profiles` is one exact `SubgroupRealizationSubject` over width, arithmetic type, and transfer. ADR 0094 decision 7 requires prepared-pipeline width to confirm that same realization. A `CapabilityAxis::SubgroupThreads` would preserve only the width and could not explain which arithmetic/transfer realization authorized the query.
- **Imprecise — not every deferred feasibility state belongs to this generalization.** `DeferredSet` also carries `DeferredDimension` rows for later numerical honourability, but `physical.rs`, anchor `if deferred.dimensions().is_empty()`, refuses those rows before executable planning. `ArtifactConstructionPlan::entry_deferred_predicates` intentionally forwards only executable prepared-entry predicates. This ticket must generalize that executable population, not claim one enum covers every deferred feasibility state.
- **False — a new public deferred-subject view is required.** The only out-of-crate production consumer, `tiler-build::assemble_plan_artifact`, reads `entry()` and forwards `requirement()` whole; it never reads `capability_axis()`. The only current call to that accessor is a same-crate compiler test. Replacing one premature diagnostic accessor with a larger public enum would create a second public vocabulary with no consumer.
- **Verified — the artifact ABI already carries the complete executable requirement.** `PreparedEntryTargetRequirement` includes the prepared-entry query, provider, phase, required value, and relation. Artifact construction binds it to the exact entry and mints the predicate. No subgroup-specific artifact row or manifest grammar is needed.
- **Imprecise — the original identity instruction overstates movement.** The canonical request subject does not encode `DeferredPredicate`; this ticket alone does not move request identity. `ImplementationProposalIdentity` does encode it. Existing proposal and explain bytes can remain stable if the new subject arm is appended under a discriminator disjoint from every nonempty capability-axis key and the subgroup explain record receives a fresh event tag. Subgroup-bearing proposals and traces are new values; no existing value needs to move.

## Decision packet — revised 2026-08-12

The decision is whether the compiler should expose a new public subject vocabulary or keep the new provenance internal while preserving the already-complete public artifact carrier.

### Recommended boundary

Use one private, closed, exhaustive compiler enum for executable deferred provenance, conceptually `ExecutableDeferredTargetSubject::{CapabilityAxis(CapabilityAxis), SubgroupWidthConfirmation(SubgroupRealizationSubject)}`. The subgroup arm carries the complete atomic subject; its required width is derived from that subject and checked against the `PreparedEntryTargetRequirement`. No caller supplies the subject and width independently.

Remove public `PreparedEntryTargetRequirementRef::capability_axis()` and do not replace it with a public subject enum. Keep `entry()` and `requirement()`, which are exactly what the artifact assembler consumes. If a future named out-of-crate diagnostic consumer needs the static provenance, return to Tom with that consumer and the smallest view it requires.

Make construction validate each subject/requirement pair atomically. The capability arm must retain its governed axis relation and quantity. The subgroup arm must require the accepted prepared subgroup query, `PreparedKernelPreflight`, `ObservedEqualsRequired`, and a required value exactly equal to the subject width. A mismatch is malformed compiler output, never a deferred route, fallback, or normalized value.

Keep canonical collections exhaustive over the private enum. Sort by phase and a compiler-owned canonical subject key. Reject a duplicate exact subject and reject two different static subjects that mint the same executable query for one entry. Keep `DeferredDimension` separate until a numerical deferred row is actually executable.

Preserve existing identities append-only. In proposal identity, retain the current nonempty length-framed capability-axis key spelling and reserve a structurally disjoint escape for atomic subjects, followed by an explicit subject-family tag and the complete canonical subgroup subject. In explain, keep capability event tag 8 byte-for-byte and give subgroup confirmation a fresh event tag carrying width, arithmetic, transfer, entry, and complete requirement. The in-memory event may share the typed subject enum; its encoder chooses the legacy or new tag exhaustively. This needs no proposal-domain, explain-schema, renderer, request, artifact, or manifest version step because no previously encodable value changes. The feasibility rule-set key advances once in the atomic subgroup delivery, because that work adds the predicate vocabulary itself.

### Why this is future-proof and MECE

The two private variants partition executable deferred predicates by proof shape: a quantitative relation over one governed axis, or a later confirmation attached to one already-proven atomic realization. They neither overlap nor leave subgroup confirmation disguised as an axis. A future proof shape adds a variant and stops every same-crate encoder, sorter, explainer, and validator at compile time. Public artifact forwarding remains stable because all proof shapes lower to the already-generic requirement.

### Alternatives ranked

1. **Private exhaustive subject enum; remove the unused public axis accessor; append identity/explain encodings.** Best correctness, strictness, identity stability, and maintenance. Host cost is one small enum and an O(1) comparison per deferred predicate; runtime and artifact work are unchanged.
2. **The same private enum plus a new public `#[non_exhaustive]` borrowed subject view.** Correct and cheap, but exposes a vocabulary no current consumer needs and creates a second compatibility obligation. Reconsider only with a named consumer.
3. **Retag every deferred subject uniformly and step `physical-implementation-proposal.v2` plus the existing explain event.** Correct and mechanically simple, but moves every affected existing proposal/trace and their downstream pins for no semantic gain. It is dominated by the append-only encoding.
4. **Add `CapabilityAxis::SubgroupThreads`, publish only the width, or infer the static subject from the query.** Rejected: each decomposes or reconstructs the accepted atomic subject and can admit or explain a partial conjunction.
5. **Add a subgroup-specific artifact row.** Rejected: duplicates a complete generic ABI carrier and creates codec/runtime/version work without additional proof power.

### Strongest counterpoint and reversal evidence

The append-only escape makes the proposal encoder slightly less visually uniform than retagging every arm. That cost is bounded to one documented helper and its injectivity tests. Reverse to a domain-wide retag only if implementation cannot prove structural separation from every capability-axis encoding, or if a decoder is introduced that cannot reject or frame the escape. Neither is true at this base.

## Accepted boundary — 2026-08-12

Tom accepted the recommended private exhaustive subject enum and append-only identity strategy in this conversation. Executable deferred provenance distinguishes quantitative capability axes from subgroup-width confirmation of the complete atomic subgroup realization. The compiler derives and validates the requirement from that subject; no independent subgroup capability axis, reconstructed subject, fallback, or subgroup-specific artifact row is admitted.

The unused public `PreparedEntryTargetRequirementRef::capability_axis()` accessor is removed without replacement. The supported artifact translation retains only `entry()` and the complete generic `requirement()`. A future public subject view requires a named external consumer and a separate boundary review.

Existing capability proposal and explain bytes remain unchanged. New atomic subjects use structurally disjoint append-only proposal encoding and a fresh explain event carrying the complete subject. Deferred numerical dimensions remain a separate non-executable family. The atomic subgroup delivery owns the single feasibility-rule-set vocabulary advance.

## Required delivery after acceptance

- Introduce the private required executable deferred subject and atomically validate its pairing with the complete requirement.
- Keep canonical sorting, duplicate/conflict rejection, explanation, proposal identity, and exact-entry forwarding exhaustive over the new subject. Remove the falsely total public axis accessor without a replacement public vocabulary.
- Lower both subject families into the existing generic artifact `PreparedEntryTargetRequirement`; do not add a subgroup-specific artifact row or an independently satisfiable subgroup fact.
- Preserve old capability proposal/explain bytes, encode every subgroup subject field under fresh append-only discriminators, and rederive only pins whose represented population actually changes.
- Perturb subject kind, atomic subgroup width/arithmetic/transfer, entry, query, and relation independently with unchanged checks.

## Source-first re-audit — 2026-08-18

Exact base: `075d2d447b89d8f9b96fe6baa90157334a4359f6`. Per-bullet verdicts against the 2026-08-12 audit, each re-read in full at this base before any edit.

- **Verified.** Anchors re-resolved: `pub(crate) struct DeferredPredicate` in `target/feasibility.rs` stored `CapabilityAxis` beside the requirement and `assess` sorted by `then(left.axis.cmp(&right.axis))`; `pub(crate) struct EntryDeferredPredicate` in `program.rs`; `fn encode_proposal_identity` in `frontier.rs` folded `predicate.axis().key()`, the required quantity, and the framed requirement; `ExplainEvent::DeferredTargetRequirement` in `pipeline/trace.rs` derived rule and predicate key from the axis; `pub fn capability_axis` in `session.rs`.
- **Verified.** ADR 0094 decision 7 (`an equality against an atomic target subject ... and a confirmation against the prepared pipeline before routing commit`) and the accepted atomic subject in `admit-an-atomic-subgroup-realization-subject-to-target-profiles` (status `done`) stand as cited; the accepted `SubgroupRealizationSubject` exists in `tiler-ir/src/schedule/subgroup.rs` with whole-subject equality only.
- **Verified as stated.** `physical.rs` anchor `if deferred.dimensions().is_empty()` still refuses numerical deferred rows before executable planning, and `build_artifact_plan_with_lowering` forwards only `deferred.predicates()`.
- **Verified (the "False" verdict stands).** `tiler-build/src/plan_artifact.rs` reads `entry()` and forwards `requirement()` whole; the only `capability_axis()` caller was the same-crate test at anchor `assert_eq!(query.capability_axis(), "threads-per-workgroup")`.
- **Verified.** `PreparedEntryTargetRequirement` in `tiler-ir/src/program/abi.rs` carries the query (key, phase, provider), required value, and relation, and its constructor refuses non-prepared-entry phases.
- **Verified with one stale spelling.** The proposal-identity domain is now `tiler.compiler.physical-implementation-proposal.v3`, not the `v2` the packet's alternative 3 named; the accepted append-only option is unaffected because no domain step is taken. Request identity does not encode deferred predicates; plan and portfolio identities embed the proposal identity opaquely, so preserved capability bytes preserve every downstream pin.

## Delivery — 2026-08-18

Implemented at base `075d2d44` on `tkt/generalize-deferred-target-provenance-beyond-capability-axes`.

- `target/feasibility.rs`: private exhaustive `ExecutableDeferredTargetSubject::{CapabilityAxis, SubgroupWidthConfirmation}`; `DeferredPredicate::new` validates each pair atomically (capability arm: governed relation via `target_property_relation`, axis-admissible quantity; subgroup arm: `ObservedEqualsRequired` and required exactly the subject width, prepared-entry phase held by the requirement type); mismatch is the new `FeasibilityError::MalformedDeferred`, mapped in `physical.rs` (`target-deferred-malformed`) and `target.rs`. `DeferredSet::new` sorts by phase then the compiler-owned `canonical_key` (family tag + subject identity tags; capability tags proven ascending with the derived axis order) and rejects `duplicate-deferred-subject` and `conflicting-deferred-query`; `CheckedTargetProfile` now also refuses one identical query contract on two axes (`duplicate-query-contract`), which keeps the set refusals unreachable from `assess`.
- `frontier.rs`: `encode_deferred_predicate` keeps the capability record byte-for-byte and appends atomic subjects under the zero-length-frame escape (`ATOMIC_DEFERRED_SUBJECT_ESCAPE`) plus family tag `0x01` and the complete canonical subject; proposal-identity domain unchanged at `v3`.
- `explain.rs`: appended event tag 15 `DeferredSubgroupWidthConfirmation` carrying entry, width, arithmetic, transfer, and the complete requirement; validation refuses width/required disagreement and any non-equality relation; capability tag 8 pinned byte-exact in `explain_vocabulary_is_append_only_and_versioned`; schema stays v11 and renderer v9 per the ledger's append rule, with a new ledger entry.
- `pipeline/trace.rs`: `record_target_admissions` is exhaustive over the subject; capability records are unchanged, subgroup confirmations emit rule `target.subgroup-width-confirmation`.
- `session.rs`: public `PreparedEntryTargetRequirementRef::capability_axis()` removed without replacement; `entry()` and `requirement()` retained; the sole test caller now relies on the existing query-key assertions.
- No artifact, manifest, request, renderer, or schema version step; no subgroup-specific artifact row; both families lower into the generic `PreparedEntryTargetRequirement`. No pins rederived because the capability population's bytes are unchanged (checked by the byte-level controls) and no subgroup value is minted on the compile path yet — `assess` still has no subgroup `Later` route, which `carry-subgroup-width-through-exact-prepared-entry-equality` owns.
- Perturbations (subject, never the assertion; each caught with quoted failure text in the worker report): family-tagged capability identity arm; escape-skipping subgroup arm; dropped width-equality guard; dropped duplicate-subject rejection; dropped explain relation refusal; swapped tag-8 field order. Unrepresentable perturbation: `SubgroupTransfer` has one variant, so a transfer-only identity perturbation is exercised via the transfer reason-code in the explain event and documented at the frontier test.
- Commands: `cargo check -p tiler-compiler --all-targets`; `cargo nextest run -p tiler-compiler` (952 passed, 1 skipped); `cargo nextest run -p tiler-build` (94 passed); `cargo test -p tiler-compiler --doc` (16 passed); `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `cargo clippy -p tiler-build --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler`; `cargo fmt -p tiler-compiler --check`.

## Closes when

The compiler can carry a subgroup-width confirmation without inventing a capability axis, and every producer/consumer is exhaustive over the typed provenance.
