---
id: decide-whether-the-refinement-subject-identity-should-carry-its-environment
title: Decide whether the refinement subject identity should carry its environment
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, indexing, decision]
---
## User-visible outcome

Either the refinement subject identity carries the environment a symbolic extent resolves against, or the record states why it deliberately does not — so two subjects that differ only in environment are known to be the same subject or different ones, rather than the question going unasked.

**Decided 2026-08-23 at base `9b61b563`: the identity must carry the environment.** The exclusion is not safe, and the ticket closes on the decision and its evidence; the identity step itself is [`step-the-refinement-subject-identity-to-carry-its-shape-environment`](step-the-refinement-subject-identity-to-carry-its-shape-environment.md), because folding the environment in steps `tiler.ir.index-refinement-subject.v2` and that needs its own coherent change across the owning version, the domain ledger, and every nested pin.

## Why this exists

Filed 2026-08-22 by the coordinator from the sibling sweep of [`destructure-the-gather-bounds-subject-in-its-identity-encoder`](destructure-the-gather-bounds-subject-in-its-identity-encoder.md), which landed as `f197697f`. That lane found it while checking whether a sub-agent's census was right — it was not, and correcting it surfaced this. The lane declined it as a question about *what the identity should encode*, which its non-goals excluded. That judgement was right and is why this is a separate ticket.

## Fact audit at `9b61b563`

The ticket carried four marked claims. Every one was re-read in full at this base and every one is **verified**; the first is presented as two paragraphs below, because its field census and its grep are separate checks with separate commands.

**Fact — verified.** `IndexRefinementSubject` in `crates/tiler-ir/src/index/refinement/subject.rs` declares **fourteen** fields. Thirteen carry `pub(super)`; the fourteenth, anchored `environment: SubjectEnvironment,`, is private to the module. Reproduced with `awk '/^pub struct IndexRefinementSubject \{/,/^\}/' crates/tiler-ir/src/index/refinement/subject.rs | grep -c ':'` → `14`, the same pipeline through `grep -c 'pub(super)'` → `13`, and through `grep -v 'pub(super)'` → the `environment` line alone.

**Fact — verified.** `grep -c "environment" crates/tiler-ir/src/index/refinement/identity.rs` returns **0**. The subject identity encoder never reads it. What it does write for the graph is anchored `push_slice(&mut bytes, subject.graph.as_bytes());`.

**Fact — verified.** The population is **one** field, not two. `identity: Box<[u8]>` caches this encoder's own output and is correctly self-excluded; only `environment` lacks a recorded justification.

**Inference — verified, and now settled in one direction.** `encode_region` in `crates/tiler-ir/src/index/builder/identity.rs` folds environment identity in, anchored `push_slice(&mut out, sources.environment_identity().as_bytes());` under a comment anchored `is part of what this`. The refinement subject encoder does the opposite. The two layers had no stated reason to differ, and the evidence below shows the refinement encoder is the one that is wrong.

**Fact — verified, and it answers less than it looks like it answers.** `SubjectEnvironment`'s doc carries the anchor `compared by identity only`, and its `PartialEq` compares `left.identity() == right.identity()`. So an environment identity exists and is already what the type compares by — which means the subject's *structural* equality already separates two environments while the subject's *canonical identity bytes* do not. The two answers disagree inside one type, which is the defect rather than the answer.

## Fact added by this lane — why the graph subject does not cover the environment

`encode_subject_identity` writes `subject.graph`, a `SemanticGraphIdentity`, and that is **one of five** semantic subjects, not the whole `SemanticIdentity`. `crates/tiler-ir/src/semantic/identity.rs` deliberately keeps the environment out of the graph subject, anchored `A separate subject rather than part of`, on the reasoning that folding it there would make two programs of identical meaning that source one extent from a different input report different *graph* identity. `crates/tiler-ir/src/semantic/program.rs` makes the environment a total fifth subject instead, anchored `the fifth semantic subject is`. So the refinement subject identity carries one of the five and silently drops the one that governs how a symbolic extent resolves.

Two sibling encoders already fold it. `encode_region` does, above. `crates/tiler-compiler/src/request/subject.rs` writes all five under `tiler.compiler.request-subject.v6`, including the line anchored `self.semantic_identity.shape_environment().as_bytes(),`. The refinement subject identity is the outlier of three.

## Measurement — an environment-only difference is constructible today, from the governed registry alone

Constructed at `9b61b563` and run with `cargo nextest run -p tiler-ir -E 'test(probe_two_environments_one_subject_identity)' --no-capture`. The probe builds three programs over the standard registry — no test-only provider — each with one symbolic input `rows: [n]` and one `tiler::multiply-f32@1` application, differing only in the `ShapeEnv` they are opened with: `n` bound to input axis 0, `n` bound to input axis 1, and `n` bound to axis 0 with the added constraint `1 <= n <= 8`. Verbatim output:

```text
binding axis: environment identities differ = true
binding axis: graph identities equal = true
binding axis: SUBJECT IDENTITY BYTES EQUAL = true
binding axis: subject PartialEq says equal = false
binding axis: REGION IDENTITIES EQUAL = false
binding axis: RESOLUTION IDENTITIES EQUAL = true
binding axis: ResolvedIndexRealization PartialEq says equal = true
binding axis: resolution A accepts region B = false
binding axis: cross refusal kind = SemanticRealizationMismatch
binding axis: residual obligations A/B = 0 / 0
binding axis: RECEIPT IDENTITIES EQUAL = false
binding axis: COVERAGE IDENTITIES EQUAL = false
constraint: environment identities differ = true
constraint: graph identities equal = true
constraint: SUBJECT IDENTITY BYTES EQUAL = true
constraint: subject PartialEq says equal = false
constraint: REGION IDENTITIES EQUAL = false
constraint: RESOLUTION IDENTITIES EQUAL = true
constraint: ResolvedIndexRealization PartialEq says equal = true
constraint: resolution A accepts region B = false
constraint: cross refusal kind = SemanticRealizationMismatch
constraint: residual obligations A/B = 0 / 0
constraint: RECEIPT IDENTITIES EQUAL = false
constraint: COVERAGE IDENTITIES EQUAL = false
```

Read straight down, that is the whole decision. Two subjects differ **only** in environment; their canonical identity bytes are equal; the type's own `PartialEq` says they are different; and the regions they realize are provably different regions. `ResolvedIndexRealization` — whose `PartialEq` is exactly `self.identity == other.identity` in `crates/tiler-ir/src/index/refinement/registry.rs` — therefore reports two resolutions **equal** while each realizes a region the other's verifier refuses. That is an identity conflating two things that are not the same, reachable from the public surface without a test provider.

**What is not affected, stated so the claim is not overdrawn.** The receipt and executable-coverage identities do separate the two, because both nest the realization identity and `encode_region` folds the environment. The verifier is not fooled either: `verify` re-derives its expectation from its own subject and refuses a foreign region with `SemanticRealizationMismatch`. So this is a latent conflation at the subject and resolution keys, not a path that currently mints a wrong receipt. It is still eliminated under the readiness gate, which removes an option that *can* conflate identities rather than only one that already has.

**What made it constructible, and what would have prevented it.** A symbolic operand reaches an operation only when its definition is `ShapeInferenceParticipation::GovernedEnvironmentAware`; a `LiteralOnly` family refuses with `SymbolicOperandUnsupported`. The first attempt at this probe used a test provider built by the public `OperationDefinition::new`, which is `LiteralOnly`, and was refused for exactly that reason. Six governed families are environment-aware today — `tiler::multiply-f32@1` and `tiler::add-f32@1` in `crates/tiler-ir/src/semantic/registry.rs`, both bf16 arithmetics in `crates/tiler-ir/src/semantic/bf16.rs`, and one each in `slice.rs` and `broadcast.rs`, all constructed through `new_governed_environment_aware`. So the wall a reader might have hoped for — "no operation admits a symbolic operand" — does not exist, and the population that can reach the collision is a governed six rather than an empty set.

**And the wall is not even needed for the collision.** `IndexRefinementSubject::derive` attaches `program.extent_sources()` unconditionally, so a wholly literal program opened with an environment also carries one. Two such programs under different environments collide the same way with no symbolic extent anywhere. The environment-aware population bounds only the cases where the realized *region* also diverges, not the cases where the identity collides.

## Options, with eliminations

1. **Fold the environment identity into the subject identity.** Mirrors `encode_region` and `tiler.compiler.request-subject.v6`, and makes the subject's identity agree with its own `PartialEq`. Steps `tiler.ir.index-refinement-subject.v2` to `v3`. **Selected.**
2. **Record a documented exclusion with a reconsideration trigger.** *Eliminated.* It would be recording that `ResolvedIndexRealization`'s public equality may report two resolutions equal when they realize different regions. The readiness gate eliminates an option that conflates identities before ranking, and the measurement above shows the conflation is constructible now rather than reachable only from a future frontier.
3. **Narrow `SubjectEnvironment` away — take the environment as a realization-time parameter instead of a subject field.** *Eliminated, dominated.* `crates/tiler-ir/src/index/law.rs` reads `subject.shape_environment()` at three sites, so realization needs it from somewhere; relocating it to the caller or the resolution leaves the resolution identity still computed as `f(law identity, subject identity)` and still colliding, while additionally breaking the property that a subject is self-sufficient to realize against. It moves the field without answering the question.
4. **Fold the environment into `SemanticGraphIdentity` so the subject inherits it.** *Eliminated, and already settled against.* `crates/tiler-ir/src/semantic/identity.rs` records the reason under the anchor `A separate subject rather than part of`: it would make two programs of identical meaning that source one extent from a different input report different *graph* identity. Out of scope here and correct as it stands.
5. **Further bounded research.** *Eliminated.* The question was answerable by reading plus one construction, and both are done.

Option 1 dominates: it is the only survivor that is top-tier on correctness and strictness, and nothing it costs is paid by any other survivor, because there are none.

## Outcome

Decision recorded here and in the ticket graph, which is deliberately the only place it is pinned. No note was added beside `environment: SubjectEnvironment` in `subject.rs`: a comment saying the field is excluded and a ticket will fix it is change-history rather than an explanation of the code, it would be deleted by the very next commit to touch the line, and it would cost a full gate on `crates/` for no correctness gain. The open p1 successor is the durable record. The identity step is deliberately **not** landed in this ticket, per its own required work: it moves every subject's bytes and therefore every nested resolution and receipt identity, and it steps a domain that `crates/tiler-ir/src/domains.rs` pins. It is a **smaller** step than the prior subject step recorded in [`canonicalize-index-refinement-occurrence-ordinals`](canonicalize-index-refinement-occurrence-ordinals.md), and saying so precisely matters: `encode_executable_coverage_identity` does not read `subject.identity` at all, and kernel-program identity folds the *coverage* identity rather than the receipt identity, so coverage, kernel-program, and artifact identities do not move. The successor ticket carries that claim as a Fact to be re-audited rather than inherited. That work is [`step-the-refinement-subject-identity-to-carry-its-shape-environment`](step-the-refinement-subject-identity-to-carry-its-shape-environment.md), which carries the probe verbatim as its regression.

## Non-goals

The mechanical destructure sweep, which is [`destructure-the-framed-records-in-the-index-region-identity-encoders`](destructure-the-framed-records-in-the-index-region-identity-encoders.md). Changing `encode_region`'s treatment of environment, which is settled and reasoned. Any public surface change. Landing the identity step.

## Closes when

The decision is recorded with its evidence, the constructibility of an environment-only difference is settled by construction rather than by reading alone, and the identity step is split into its own ticket rather than landed here.
