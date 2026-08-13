---
id: enforce-proof-sidecar-byte-budgets-before-producer-allocation
title: Enforce proof-sidecar byte budgets before producer allocation
status: in-progress
priority: p1
dependencies: [decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights]
related: [retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, proof, correctness, resource-bounds]
claimed_from: todo
assignee: worker-proof-sidecar-budgets
lease_expires_at: 1786585710
---
## User-visible outcome

Proof-sidecar construction satisfies the same fail-closed resource promise as decoding: manifest, identity, payload framing, and complete-container byte limits are checked with exact arithmetic before proportional producer allocation, and rejection leaves no partially admitted case.

## Verified defect

At exact base `612468048d541a1017640fc5dcbe5ff9160716cf`, `ProofSidecarBuilder::push_case` still passes borrowed payload rows to `place`, which clones every payload after only the per-payload `MAX_PROOF_PAYLOAD_BYTES` check and before any cumulative identity, manifest, framed-payload, or complete-sidecar check. `derive_identity` and `encode_manifest` still grow vectors before checking `MAX_PROOF_IDENTITY_BYTES` / `MAX_PROOF_MANIFEST_BYTES`, and `encode` still appends every framed payload before checking `MAX_PROOF_SIDECAR_BYTES`. This contradicts the `proof` module `# Limits` claim and the `docs/artifact-abi.md` "Governed budgets" Fact that every bound is checked before proportional allocation in both directions.

The payload-limit decision is `done` and did **not** change `MAX_PROOF_PAYLOAD_BYTES`: it remains `16 * 1024 * 1024`. Retirement of that constant is owned by [`retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell`](retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell.md), which depends on this ticket.

## Fact audit — 2026-08-12 at `61246804`

The ticket Fact was dated `62df964ef529aadee4649d4eb9c155152b8c92be`. Re-read in full at this worktree's exact base: `crates/tiler-artifact/src/proof/{mod,builder,codec,model,tests}.rs`, `docs/artifact-abi.md` "Proof-case evidence sidecar" / "Governed budgets", and the accepted decision in [`decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights`](decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights.md).

- **Stale citation, defect still true — the producer still allocates before the aggregate byte bounds.** `place` is still `fn place<K: Clone + Eq>(supplied: &[(K, Vec<u8>)], …)` and still does `placed[position] = Some(bytes.clone())` after `proof_limit(bytes.len(), MAX_PROOF_PAYLOAD_BYTES, ProofLimitKind::PayloadBytes)` only. Search `bytes.clone()` in `builder.rs`. There is still no cumulative sidecar/manifest/identity projection in `push_case`.
- **Verified — `derive_identity` grows, then checks.** It does `let mut bytes = Vec::new();`, extends the domain, schema, subjects, keys, and payload *digests*, then `proof_limit(bytes.len(), MAX_PROOF_IDENTITY_BYTES, ProofLimitKind::IdentityBytes)`. Search `MAX_PROOF_IDENTITY_BYTES` in `codec.rs`.
- **Verified — `encode_manifest` grows with no bound check of its own.** The `MAX_PROOF_MANIFEST_BYTES` check is in `encode`, after `encode_manifest` has already returned the grown vector. Search `encode_manifest` then the following `proof_limit` on `manifest.len()`.
- **Verified — `encode` appends every framed payload, then checks `MAX_PROOF_SIDECAR_BYTES`.** The payload loop does `bytes.extend_from_slice(payload)` and only afterwards `proof_limit(bytes.len(), MAX_PROOF_SIDECAR_BYTES, ProofLimitKind::SidecarBytes)`. Search that `proof_limit` after the payload loop.
- **Verified — the contract still states the false universal.** `proof/mod.rs` `# Limits` still says "Every bound below is checked before any allocation proportional to it, in both directions". `docs/artifact-abi.md` "Governed budgets" still says the same of encoder and reader. The decoder mostly honours the count-before-reserve half (`decode_proof_sidecar` checks `bytes.len()` against `MAX_PROOF_SIDECAR_BYTES` first; `Cursor::count` checks a declared count before the caller reserves). The producer half does not.
- **Verified — `MAX_PROOF_PAYLOAD_BYTES` is still `16 * 1024 * 1024`.** The decision to retire it is accepted; the public constant and `ProofLimitKind::PayloadBytes` remain until the dependent retirement ticket. This ticket must not change that value.
- **Imprecise as dated — the cited base `62df964e` is not this worktree.** The defect text was true there and is still true here; what aged is the base pin and the surrounding contract context (the payload-limit decision's correction already named this ticket as the reason the advertised pre-allocation promise is false).

## Required delivery

- Derive exact encoded manifest, identity, framing, and total sidecar sizes with checked arithmetic before cloning, hashing, reserving, or appending proportional data.
- Preserve transactional `push_case`: overflow or unrepresentable arithmetic leaves the builder unchanged and returns the exact typed limit/overflow reason.
- Consume or move the `Vec<u8>` payloads already owned by `ProofCaseSpec` rather than cloning them where the interface permits.
- Use exact or fallible reservation after validation; do not translate allocation failure into a smaller proof, omitted payload, external reference, or fallback case.
- Keep producer and decoder acceptance coherent and pin the full population of governed byte resources from the owning type rather than a hand count.
- Correct the contract only if a claimed pre-allocation check cannot honestly be provided; do not leave the present false universal in place.

## Required negative controls

Independently perturb manifest size, identity size, cumulative payload framing, sidecar total, arithmetic overflow, and the move-vs-clone path. Assertions remain unchanged and each production subject must fail at its own named boundary.

## Closes when

No producer path performs proportional allocation before its governing size is established, exact package tests and doctests pass, and independent review reconciles every `MAX_PROOF_*` byte bound with both producer and decoder consumption.

## Worker report — 2026-08-12

At `61246804`, `place` still cloned after only the per-payload check; `derive_identity` / `encode_manifest` / `encode` still grew then checked. `MAX_PROOF_PAYLOAD_BYTES` is still `16 * 1024 * 1024` (retirement is the dependent ticket).

Producer paths now project identity, manifest, framed-payload, and complete-sidecar sizes with checked addition and refuse before cloning, hashing, reserving, or appending. `push_case` is transactional and moves caller-owned `Vec<u8>` payloads. Unrepresentable host arithmetic is a named `Unrepresentable` refusal. Independent tests perturb identity size, manifest size, cumulative framing (as the addend that makes the sidecar total fail), sidecar total, overflow, payload bytes, and move-vs-clone. `ProofLimitKind::byte_budget` plus a `variant_count`-sized `ALL` pin the byte-resource population from the type.

Checks: `cargo nextest run -p tiler-artifact` (264 passed, 1 skipped); `cargo test -p tiler-artifact --doc`; `cargo clippy -p tiler-artifact --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-artifact --no-deps`; `tkt lint`; `git diff --check`.
