---
id: correct-the-one-region-premise-in-the-concatenate-absence-check
title: Correct the one-region premise in the concatenate lowering record's absence check
status: review
priority: p3
dependencies: []
related: [lower-a-two-region-occurrence-through-one-index-access-capability, correct-the-one-region-per-occurrence-claim-in-the-records]
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-concat-premise
lease_expires_at: 1786052103
---
## What is stale

`docs/research/indexing/concatenate-fusion-role-and-lowering.md:182` opens absence check 5 with "One index-access capability emits one region, and resolution is by exact signature, so a variadic family needs one capability per arity." The first clause stopped being true on 2026-08-06: [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) gave `IndexAccessLoweringProvider` a defaulted `lower_sequence`, and `GovernedRootMeanSquareScaleF32` is a shipped provider that emits an ordered chain of regions for one occurrence.

The check's *conclusion* — resolution is by exact signature, so a variadic family needs one capability per arity — is unaffected and still holds. What is wrong is the premise it is stated from, which now reads as a general claim about index-access capabilities that the source refutes.

## Why it is a separate ticket

[`correct-the-one-region-per-occurrence-claim-in-the-records`](correct-the-one-region-per-occurrence-claim-in-the-records.md) swept `docs/` for this claim and corrected every site it could reach. This one sits in `research/indexing`, a scope that ticket does not hold and that three open tickets do, so it was reported rather than edited.

## What this must do

Restate check 5's premise so the arity conclusion is derived from signature-exact resolution alone, and verify the reworded check against `crates/tiler-compiler/src/capability.rs` rather than against this description. Confirm the surrounding absence checks in that block still say what their positive controls demonstrate.

## Closes when

Absence check 5 states a premise the source supports, its conclusion about per-arity capabilities is unchanged, and `grep -rn "emits one region" docs/` returns nothing that contradicts `lower_sequence`.

## Outcome

**2026-08-06, at `cfe906cc` on `tkt/correct-the-one-region-premise-in-the-concatenate-absence-check`, based on `6f0e9997`.** Docs-only: `docs/research/indexing/concatenate-fusion-role-and-lowering.md` is the sole file touched.

**Fact — check 5's premise is now resolution alone, verified against `crates/tiler-compiler/src/capability.rs` and not against this ticket's description.** `LoweringSignature` (`:140-145`) is `operands: Vec<ResolvedValueType>` and `results: Vec<ResolvedValueType>` under a derived `PartialEq`, so signature equality is exact ordered list equality including length; `LoweringCapabilityRegistry::resolve` (`:1403-1420`) filters the registry on `key.family == selector.family && &key.operation == selector.operation && &key.signature == selector.signature` and returns `MissingCapability` when nothing matches. Two admitted arities of one operation are therefore two signatures and two capabilities, with no appeal to how many regions a capability emits. The check reads:

```sh
# 5. Resolution is by exact signature, so a variadic family needs one
#    capability per admitted arity. A `LoweringSignature` is the ordered
#    operand and result type lists, compared for equality; `resolve` filters
#    the registry on the exact (family, operation, signature) triple. Two
#    arities of one operation are therefore two signatures and two
#    capabilities, and the conclusion rests on this alone.
grep -n 'pub struct LoweringSignature' -B 2 -A 5 crates/tiler-compiler/src/capability.rs
grep -n 'fn resolve_index_access' -A 25 crates/tiler-compiler/src/capability.rs
#    Positive control: the second read reaches the MissingCapability
#    construction, so the failure path is present rather than inferred.
#    Restated 2026-08-06: this check opened "one index-access capability emits
#    one region, and resolution is by exact signature". The first clause is
#    refuted — the trait carries `lower` beside a defaulted `lower_sequence`,
#    so one capability may emit an ordered chain of regions for one occurrence
#    — and it was never load-bearing here. The third read is what shows that,
#    and is a control on the premise rather than on the conclusion.
grep -n 'fn lower_sequence' -B 18 -A 6 crates/tiler-compiler/src/capability.rs
```

The seven-capabilities conclusion at `:90` is untouched and its own derivation — the exact triple, plus the type lists — is what the reworded check now checks.

**Fact — the same refuted clause stood as a Fact in the body, at `:88`, and was restated rather than left.** It read "`IndexAccessLoweringProvider::lower` … emits through one `IndexAccessLoweringContext` … One capability therefore produces **one index region per occurrence**". It now states that as this record's base reading, records the landing that withdrew it — the defaulted `lower_sequence` (`capability.rs:359-364`) and `GovernedRootMeanSquareScaleF32` (`governed.rs:1044-1074`), which overrides it to emit a two-stage chain and implements `lower` as an explicit refusal — and states that nothing below rested on it. It was inside this ticket's file and its scope, and the closing grep would not have found it, so leaving it would have preserved the corrected sentence's twin one paragraph up.

**Per-check confirmation, each command run verbatim from the block against the base tree.**

1. **Stale, and corrected.** The check read "nine keys and no concatenate". `grep -n 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns eleven sites, and `concatenate_f32_op()` is registered `CoordinateRelation` at `:390-393` — [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) is `done` and held `implementation/compiler` and `contracts/navigation`, neither of which reaches this record. The check now reads eleven keys with the concatenate among them, keeps its positive control, records the prior state as what the elimination ran against, and adds `-A 1` because most entries span two lines and the bare match shows no key.
2. **Stale, and corrected.** The check read "closed over two keys". `is_exact_governed_same_family_pointwise` (`:1316`) binds `concatenate` at `:1322` and its `CoordinateRelation` arm (`:1350-1353`) matches `reindex || broadcast || concatenate`. Three keys, and the arm's own comment states the per-key transfer rather than inheriting it from the role — which is what this record argued for. The check now reads three keys and says only the membership moved.
3. **Holds.** `InvalidWriteDomain` returns the subset rule's doc (`builder.rs:1207-1219`, "any subset of the region's parallel dimensions", reduction dimension still refused) and its one refusal site (`:1342`); the partition read returns `decide_partition_by_interval` (`proof.rs:1122`), its call site (`:329`), and `OutputPartitionRangesOverlap`, `OutputPartitionUncovered`, `OutputPartitionDoubleWritten`; the exhaustive walk's tail shows the uncovered-element scan (`proof.rs:858-868`); `MultipleWriters` is at `program/verify.rs:203`; and `grep -rn DuplicateOutputTensor crates/` returns nothing, exit 1. Every positive control returns what the check claims.
4. **Holds.** `ScalarValueDefinition` (`model.rs:162-170`) has exactly `AccessRead` and `OperationResult`, no conditional; the read also returns `ScalarValueDefinitionView` (`:1045-1055`) with the same two, so the absent select is an absence from an enumerated list in both the internal and the public spelling.
5. **Was stale — this ticket's subject, corrected as above.**
6. **Holds.** The read lands inside `ExplainWriter::new`'s `allowed_providers` vector (`explain.rs:1302-1305`), and the surrounding comment names "the compiler's governed physical-implementation and fusion-capability providers", so the positive control demonstrates what it claims: the fold takes `FusionNumericalCapabilities::governed().provider()` and no role table.

**Fact — the closing grep.** `grep -rn "emits one region" docs/` returns one line, which is the check-5 comment being corrected in this change:

```
docs/research/indexing/concatenate-fusion-role-and-lowering.md:182:# 5. One index-access capability emits one region, and resolution is by exact
```

After the change it returns nothing (exit 1). The broader `grep -rn "one index region per occurrence\|one region per occurrence" docs/` returns `roadmap.md:470`, `roadmap.md:471`, `research/numerics/transformer-nonlinear-normalization-and-reductions.md:413`, `research/semantic-graph/operation-family-delivery-graph.md:58`, and this record's `:88`; the first four state the limit in corrected or past tense under [`correct-the-one-region-per-occurrence-claim-in-the-records`](correct-the-one-region-per-occurrence-claim-in-the-records.md), and `:88` was the last site stating it as current. It now states it as history too.

**Out of scope, reported rather than absorbed.** [`docs/research/indexing/sub-tensor-selection-fusion-role.md`](../docs/research/indexing/sub-tensor-selection-fusion-role.md) carries the same two stale checks from the same landing — its check 1 (`:127`) reads "nine keys and no slice" and its check 2 (`:133`) "closed over two keys". The slice's *absence* still holds; the count is eleven and the arm is closed over three. Its `:110` also reasons about the nine-key count in prose. That record is a different document with its own subject and is not "that block", so it is left for a ticket rather than edited here.
