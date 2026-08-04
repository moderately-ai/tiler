---
id: canonicalize-index-refinement-occurrence-ordinals
title: Canonicalize index-refinement occurrence ordinals
status: awaiting-decision
priority: p1
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/ir, implementation/compiler, contracts/decisions, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, identity, implementation]
claimed_from: ready
assignee: agent-canonical-v2
lease_expires_at: 1785863038
---
## User-visible outcome

Equivalent semantic graphs authored in different valid insertion orders mint identical occurrence-bound index-refinement receipts, while different canonical occurrences remain distinct.

## Defect

`SemanticGraphIdentity` canonicalizes operation traversal, but `IndexRefinementSubject::derive` retained the caller storage ordinal as `SemanticOccurrence`. Two programs with equal canonical graph bytes and reversed independent constant insertion therefore gave occurrence 0 different operation attributes and minted different receipt identities. The pair `(canonical graph identity, occurrence)` did not stably name one operation.

Independent review then found that commit `538fb77d86a34515f270ca93fdb83b094df700f9` repaired only the retained coordinate and is not landable. Its `SemanticOccurrence` argument still means a storage-order selector at the public call boundary but a canonical coordinate in the returned subject; compiler `SemanticMemberId` and the current stage-coverage path remain storage-order; and `canonical_operation_ordinal_for_verified` recomputes the complete canonical traversal for every derived occurrence, making an all-occurrence lowering quadratic. The commit is preserved as evidence and must not merge.

## One-coordinate-system public draft

The storage selector and retained identity coordinate must be different types with one meaning each.

- `IndexRefinementSubject::derive` selects through an existing graph-owned `OperationId`, not through `SemanticOccurrence`. `OperationId` is already the non-serializable capability for one verified program operation and makes a foreign-program selector a typed handle error. This is the recommended exact public signature draft: `derive(program: &SemanticProgram, operation: OperationId, numerical_contract: NumericalContractIdentity)`.
- `SemanticOccurrence` means only the canonical operation coordinate paired with `SemanticGraphIdentity`. It is returned by the derived subject/receipt and used anywhere a durable occurrence identity is stored or encoded; it is never accepted as an arena selector.
- `ProgramData` caches a storage-operation-index to canonical-occurrence map once beside its existing canonical value IDs. Derivation performs one checked selector lookup and one O(1) map lookup; deriving all occurrences is O(n), not O(n²).
- Compiler recognition may continue using storage-order `SemanticMemberId` internally, but it resolves that member to the program's graph-owned `OperationId` before subject derivation. Current compiler stage coverage must use the canonical occurrence carried by the verified refinement/receipt, never `SemanticOccurrence::new(member.0)` or an equivalent wrapping of a storage member.
- No independently assembled storage-to-canonical pair or raw canonical-ordinal constructor is added to the public path.

This signature changes an existing consequential public method. The implementation remains a tested draft until Tom accepts the exact signature; neither a green gate nor this correctness derivation accepts it implicitly.

## Required perturbations

Build two equal graphs with two differently-valued independent constants inserted in opposite order but published under the same ordered names, then prove all directions rather than sorting away the coordinate under test:

1. graph identities are equal;
2. the same named semantic operation has different storage selectors/`OperationId`s across the two authored programs but the same retained canonical occurrence and receipt identity;
3. the two distinct constants in either one graph retain distinct canonical occurrences and distinct receipt identities;
4. selecting each graph's other `OperationId` selects the other operation rather than being normalized to the requested one;
5. a foreign graph's `OperationId` is refused through the existing typed handle error;
6. compiler fused and materialized stage coverage uses those canonical receipt occurrences and stays deterministic in both authoring directions;
7. the existing cross-occurrence completion refusal remains effective.

A scale test derives every occurrence of a wide independent-operation graph and establishes one cached canonical traversal plus linear lookup work, rather than relying on elapsed time.

## Identity and pin analysis

Before any version decision, enumerate the exact changed identity population from construction through consumption: refinement subject, admitted authority/resolution if the subject coordinate reaches them, completed receipt, compiler request/explain qualifiers and pins, current kernel-program stage/program identity where compiler coverage previously wrapped a storage member, and every downstream recorded artifact pin that actually nests one of those subjects. State the population the search ran over and prove each pin can fail before recomputing it.

Do not infer “no version step” merely because the old coordinate was defective. If every previously valid byte string moves or the subject grammar changes, advance the owning domain and ledger it; if a domain remains unchanged, give per-field injectivity and subject-equivalence reasoning. Recompute pins on the final merged tree, never from `538fb77d` or a worker report.

## Closes when

The one-coordinate-system public signature is accepted by Tom; selector/canonical types cannot be crossed; the cached mapping and compiler coverage corrections land; every directionality, distinctness, foreign-handle, complexity, and failure perturbation above passes; the full identity/pin blast radius is enumerated and stepped or proved unchanged; affected IR/compiler tests, Clippy, docs, full gate, scope/lint/diff checks pass; and `538fb77d` remains preserved but unmerged.

## Tested public draft and identity analysis

**Proposal — consequential boundary, not yet accepted.** The implemented draft is exactly `IndexRefinementSubject::derive(&SemanticProgram, OperationId, NumericalContractIdentity)`. `OperationId` is used only to select through `SemanticProgram::operation`; a selector from another completed graph fails with the existing typed `HandleError::ForeignGraph { entity: Operation }`. The returned subject retains only the selected operation's `SemanticOccurrence`, defined as its canonical traversal ordinal. No public storage ordinal or caller-assembled `(graph, occurrence)` path was added.

**Fact — bounded work.** `ProgramData` now stores one `canonical_operation_ordinals: Vec<u32>` beside `canonical_value_ids`. Both vectors are produced from one call to `canonical_traversal` during each completed/preview `ProgramData` construction. A subject derivation does one checked `OperationId` lookup and one indexed lookup in that immutable vector. The 1,024-independent-operation test asserts the cache has exactly 1,024 entries, derives all 1,024 subjects, and observes 1,024 distinct canonical occurrences; there is no traversal in the derive path, so full derivation work is O(n), not O(n²).

**Fact — compiler coordinate join.** Compiler `SemanticMemberId` remains storage-order. `project_occurrence` resolves it to the program-owned `OperationId` before derivation. Stage coverage is then projected from the already verifier-minted `IndexRefinementReceipt::occurrence`, through `ResolvedLowering`, rather than independently wrapping `member.0`. Both normal planning and replay verification thread that same completed lowering evidence through fused, materialized, and split program assembly.

**Fact — exact affected identity population.** The source inspection population was every `IndexRefinementSubject`, `IndexRefinementReceipt`, and `IndexRefinementIdentity` construction/consumer under `crates/tiler-ir`, `crates/tiler-compiler`, `crates/tiler-artifact`, and `crates/tiler`, plus every 64-hex literal under `crates/tiler-ir` and `crates/tiler-compiler`. The canonical occurrence is encoded directly in the private `IndexRefinementSubject` identity; `ResolvedIndexRealization` nests that subject identity; `IndexRefinementReceiptIdentity` nests both subject and resolution; compiler `IndexRefinementIdentity` nests the occurrence and receipt; its trailing-byte explain label consequently follows it. `RefinementContentIdentity`, semantic/request identity, realization-law registry identity, lowering-provider identity, schedules, and kernels do not encode the occurrence and do not move. Kernel-program stage coverage encodes the canonical occurrence in the existing field, so `CanonicalKernelProgramIdentity`, and any artifact/program/envelope identity that later nests it, moves only where a stage previously encoded a noncanonical storage member. The search found no literal refinement, receipt, explain-label, kernel-program, or artifact pin derived from this occurrence chain to recompute; the unrelated 64-hex literals were contraction result digests, SHA fixtures, registry/schedule/target-profile pins, and semantic-operation source digests.

**Inference — no identity-domain version step.** No domain tag, field, framing, order, or cardinality changes. For an already canonical subject every byte is unchanged. For the defective population, `(SemanticGraphIdentity, storage ordinal)` did not name one semantic operation across equivalent authorings, so preserving those bytes would preserve a collision in subject meaning. Replacing only that field with the canonical traversal ordinal makes `(graph, occurrence)` injective over retained operations; the subject's operation key, attributes, signature, and contract remain independently framed, and downstream nesting remains injective by its existing length prefixes. Stage coverage likewise keeps the v6 grammar and ascending/deduplicated encoding, but now its occurrence is in the same coordinate system as its semantic graph. This is a semantic correction inside existing domains, not a universal re-baseline or grammar change.

**Measurement — deliberate failures.** Temporarily replacing the cached canonical ordinal with the storage index made `equivalent_authoring_orders_retain_directional_canonical_occurrences` fail with `SemanticOccurrence(0)` versus `SemanticOccurrence(1)`. Temporarily replacing receipt-sourced compiler coverage with `SemanticOccurrence::new(member.0)` made `stage_coverage_uses_verified_canonical_receipt_occurrences` fail with `[SemanticOccurrence(0)]` versus `[SemanticOccurrence(2)]`. The real foreign `OperationId` test reaches the typed `ForeignGraph(Operation)` refusal, the other selector reaches the other canonical operation, distinct constants retain distinct occurrences and receipt identities, and the pre-existing completion cross-wire test still refuses.
