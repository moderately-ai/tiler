---
id: site-the-governed-digest-so-layered-identity-encoders-can-reach-it
title: Site the governed digest so layered identity encoders can reach it
status: in-progress
priority: p2
dependencies: []
related: [decide-whether-executable-coverage-evidence-folds-as-a-digest, decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest]
scopes: [contracts/decisions, implementation/ir, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [identity, decision, public-boundary, architecture]
claimed_from: todo
assignee: agent-digest-crate
lease_expires_at: 1786066635
---
## User-visible outcome

The one question blocking [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) is answered: whether the governed digest is reachable from the crate that mints layered IR identities, or whether those identities stay restricted to canonical-bytes-only folds and the coverage encoding keeps its quadratic.

**This is Tom's. It is a consequential public crate, module, and type boundary, and it moves an ownership two accepted ADRs currently place elsewhere.**

## Why this exists

**Fact — the crate that needs a digest cannot reach one, and the dependency cannot be reversed.** `IndexRefinementExecutableCoverageIdentity` is minted in `crates/tiler-ir/src/index/refinement.rs`. `tiler-ir` is the workspace's bottom crate: `crates/tiler-ir/Cargo.toml` lists exactly `num-bigint`, `num-integer`, and `num-traits`, and no workspace member sits below it. The one governed digest — `DigestAlgorithm`, `Digest`, `DIGEST_BYTES` — lives in `crates/tiler-artifact/src/program/codec/digest.rs`, and `tiler-artifact` depends **on** `tiler-ir`.

**Fact — duplicating it is refused by the module that owns it.** `digest.rs`'s own module doc: being "the *only* place that maps the governed tag to an implementation is the whole point, and that property is what a second component reaching for a hash function would destroy."

**Fact — this exact constraint has been decided once already, in the direction of moving the consumer.** ADR 0082 records that the expansion cache's previously assigned owner was replaced because it "proved unable to reach the governed digest this decision requires it to validate against". That precedent resolved a reachability problem by relocating the *consumer*; here the consumer is the bottom crate and cannot be relocated.

**What it is worth.** ADR 0104 measures the consequence: folding the per-coverage-record graph identity turns kernel-program identity from `134n² + 3650n + 719` into `3525n + 719`, moving the per-invocation embedding-ceiling crossing from between 50 and 51 semantic operations to between 148 and 149 — and, more than the number, turning a quadratic into a linear. The current crossing sits **at** the roadmap's own ≥ 51-operation decoder layer, below the governed `semantic_operations` budget of 62, with no typed refusal at the artifact layer.

## The candidates, with what each enables and prevents

- **Move `DigestAlgorithm` and `Digest` into `tiler-ir`.** Cheapest edit and no new crate. Cost: the shared IR gains a `sha2` dependency and a hashing responsibility, and `docs/artifact-abi.md` plus ADR 0050 currently site the governed digest with the artifact envelope, where `tiler-cache` reaches it. The ownership statement in both would move.
- **Introduce a crate below both** owning the governed algorithm, its tag table, and its domains. Keeps the shared IR free of hashing and gives the "one authority for a governed constant" rule its own home. Cost: a new workspace member and a new public surface, against ADR 0056's four-library framing and the consumer-closure statements `tiler`, `tiler-runtime`, and `tiler-artifact` carry in their manifests.
- **Answer no, and keep layered identities canonical-bytes-only.** Preserves every current statement. Cost: ADR 0104 cannot execute, so the coverage encoding stays quadratic and the embedding ceiling keeps binding at decoder-layer size. That is a decision to accept the ceiling, and it should be recorded as one rather than reached by default — with a named trigger for reopening.

## Explicit non-goals

Not re-deciding ADR 0104's *choice* among the three coverage encodings — that derivation is complete and recorded on its own ticket and record. Not adding a second hashing implementation anywhere, under any of the three answers. Not touching the manifest's identity digest, which is [ADR 0103](../docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md)'s and already lands inside the crate that owns the digest.

## Closes when

Tom has answered, and the answer is either an accepted boundary with ADR 0104 unblocked and its identity-domain step scheduled, or a recorded decision to keep layered identities canonical-bytes-only with the ceiling accepted and a reopening trigger stated.

## Decision — a new bottom crate

**Decided by Tom on 2026-08-06 at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket): the governed digest moves to a new workspace crate below `tiler-ir`**, owning `DigestAlgorithm`, `Digest`, `DIGEST_BYTES`, the tag table, and the domain-separation discipline, with `tiler-artifact` re-exporting so every current consumer keeps its path. Grounds accepted: hashing is a separate responsibility from tensor IR; the one-authority property gets a structural home rather than riding in whichever crate needed it first; future layered-identity consumers reach it without reopening the boundary. ADR 0104 is accepted with this answer and its identity-domain step is scheduled; this ticket becomes the implementation vehicle and closes when the crate exists, the relocation lands with the ownership statements in `docs/artifact-abi.md` and the ADR 0050/0056 framing moved coherently, and ADR 0104's coverage-fold step lands whole with every pin recomputed.
