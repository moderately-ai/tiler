---
id: retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell
title: Retire the independent proof-payload limit and route the vocabulary cell
status: done
priority: p1
dependencies: [decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights, enforce-proof-sidecar-byte-budgets-before-producer-allocation]
related: [route-the-realization-conformance-half-into-the-conformance-crate]
scopes: [implementation/artifact, contracts/artifacts, implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, proof, conformance, public-boundary]
---
## User-visible outcome

One proof payload is admitted whenever the complete proof sidecar remains within its governed container budget. The 32 MiB vocabulary-projection weights route through the ordinary L3 conformance member without a workload special case, and there is no second arbitrary payload-size authority to drift.

## Required delivery

- Remove public `MAX_PROOF_PAYLOAD_BYTES`, `ProofLimitKind::PayloadBytes`, their producer/decoder checks, rustdoc links, displays, tests, and contract rows. Do not retain a deprecated alias in this pre-production tree.
- Make framed-length decoding rely on checked representability, remaining input, and the already-established complete-sidecar bound, with typed malformed/truncated/sidecar-limit distinctions and no partial decoded value.
- Add `w_vocab_slice` to `CONTRACTION_MEMBERS` through the same constructor as the other retained L3 cells. Update derived publishable populations, publication counts, docs, and negative controls; `cases_for(L3CorrectnessCell)` already synthesizes its operands and must not gain a special row.
- Pin that the routed case's complete payload content is within `MAX_PROOF_SIDECAR_BYTES`; separately perturb one total beyond the container and observe the unchanged atomic refusal.
- Record the public breaking removal and exact included/excluded facade after an independent exact-commit review.

## Non-goals

Changing the 256 MiB container limit, a wire/schema version step, payload chunking, compression, external references, lazy resolution, streaming decode, or changing proof-sidecar content identity.

## Closes when

The independent limit no longer exists, all proof byte admission is container-based, the seventh contraction route publishes and validates, no existing content identity moves unexpectedly, and full artifact/conformance gates plus independent review pass.

## Fact audit — 2026-08-12 at `611fefee`

The ticket as dispatched has no Facts section. The implicit claims in Required delivery, plus the accepted decision's construction sites, were re-read at this base: `crates/tiler-artifact/src/proof/{mod,budget,builder,codec,tests}.rs`, `docs/artifact-abi.md` "Governed budgets", `crates/tiler-conformance/src/envelope.rs`, `envelope/tests.rs`, and `publication/proof.rs`. Repairing them does not change what the ticket is for.

- **Verified — public `MAX_PROOF_PAYLOAD_BYTES` is still `16 * 1024 * 1024`.** Search `pub const MAX_PROOF_PAYLOAD_BYTES` in `crates/tiler-artifact/src/proof/mod.rs`. Its rustdoc is still the underived "Maximum bytes of one case payload — one input or one expected output."
- **Verified — `ProofLimitKind::PayloadBytes` is still a public variant.** `byte_budget` returns `Some(MAX_PROOF_PAYLOAD_BYTES)`. `governed_byte_resources_are_pinned_from_the_limit_kind` still lists it in the `variant_count`-sized `ALL`.
- **Verified — three enforcement sites still exist, and they are not the sites the decision ticket named.** `budget::project_layout` still does `proof_limit(len, MAX_PROOF_PAYLOAD_BYTES, ProofLimitKind::PayloadBytes)` before accumulating framed sizes. `builder::resolve_slots` still applies the same check before placing a key. `codec::read_payloads` still does `cursor.count(MAX_PROOF_PAYLOAD_BYTES, ProofLimitKind::PayloadBytes)`.
- **False if restated from the 62df964e producer — there is no `place` clone after only the per-payload check.** `place` is gone. `push_case` projects through `project_with` from lengths, then `take_placed` moves the caller-owned `Vec<u8>`. `encode_manifest` no longer independently checks a per-payload bound. The prerequisite at `7513cda9` landed.
- **Verified — `CONTRACTION_MEMBERS` is `[ContractionMember; 6]` with `l3_member(0)` through `l3_member(4)`.** `w_vocab_slice` is `L3_CORRECTNESS_CELLS[5]` and is excluded by a hand-written index. Search `five of the six` in `envelope.rs`.
- **Verified — `cases_for(ProofFamily::L3CorrectnessCell)` already synthesizes operands for every `L3_CORRECTNESS_CELLS` extent**, including `1 × 8192 × 1024`. No special operand-table row is required.
- **Verified — complete payload content of `w_vocab_slice` is 33,591,296 bytes**, under `MAX_PROOF_SIDECAR_BYTES` (`256 * 1024 * 1024` = 268,435,456). Activations `1*1024*4 = 4,096`, weights `8192*1024*4 = 33,554,432`, expected `1*8192*4 = 32,768`.
- **Verified — no content identity folds a `MAX_PROOF_*` value.** `derive_identity` folds the identity domain, `MANIFEST_SCHEMA`, subjects, keys, and payload *digests*. No identity-domain step is required. `SIDECAR_FORMAT` and `MANIFEST_SCHEMA` remain `(1, 0)`.
- **Verified — the contract still lists the independent payload row.** `docs/artifact-abi.md` "Governed budgets" still says "16 MiB per case payload" in the bound sentence.
- **The ticket's own Required delivery is still the work.** Removing the independent public constant and `PayloadBytes` classification, admitting one payload when the complete sidecar stays in the container, and routing `w_vocab_slice` through `l3_member(5)` are unchanged by this audit.

## Worker report — 2026-08-12

At `611fefee`, the independent 16 MiB payload gate was still public and still enforced at `project_layout`, `resolve_slots`, and `read_payloads`. The prerequisite producer no longer clones first. `CONTRACTION_MEMBERS` was six long and omitted `l3_member(5)`. Identity does not fold any `MAX_PROOF_*` value; `SIDECAR_FORMAT` and `MANIFEST_SCHEMA` stay `(1, 0)`.

Removed `MAX_PROOF_PAYLOAD_BYTES` and `ProofLimitKind::PayloadBytes` with no deprecated alias. One payload is admitted when the complete sidecar stays in `MAX_PROOF_SIDECAR_BYTES`. Framed-length decoding distinguishes unrepresentable, `Truncated`, and `Limit(SidecarBytes)` and yields no partial value. `w_vocab_slice` is `l3_member(5)` through the same constructor as the other L3 cells. `cases_for(L3CorrectnessCell)` gained no special row.

Pinned: vocab complete payload content is 33,591,296 against a 268,435,456 container. A sidecar total beyond the container is still an atomic `SidecarBytes` refusal. On this host the ordinary gate published and compared `w_vocab_slice`; executed digest `88b01ae776f42bdb2f2d1092ddfd039e20e652d28393a6e2ec19e5cc1d9803c8` matches the retained measurement. Sidecar on disk was 33,675,007 bytes (one case).

Public breaking removal (labelled draft until independent exact-commit review accepts the included/excluded facade):

- Removed from the accepted proof facade: `pub const MAX_PROOF_PAYLOAD_BYTES`, `ProofLimitKind::PayloadBytes`.
- Still included: the remaining public budgets (`MAX_PROOF_CASES`, `MAX_PROOF_CASE_KEY_BYTES`, `MAX_PROOF_INTERFACE_ENTRIES`, `MAX_PROOF_SUBJECT_BYTES`, `MAX_PROOF_MANIFEST_BYTES`, `MAX_PROOF_SIDECAR_BYTES`, `MAX_PROOF_IDENTITY_BYTES`), the rest of `ProofLimitKind`, the builder, the reader, the case vocabulary, the read views, and the four typed rejection vocabularies.
- Still excluded: the wire form, framing magic, domain separators, schema versions, manifest encoder, and identity deriver.

Out of scope and still stale: ADR 0106's current-status paragraph still says five of six and names the decision ticket as `awaiting-decision`. That document is `contracts/decisions`.

Checks: `cargo nextest run -p tiler-artifact` (266 passed, 1 skipped); `cargo nextest run -p tiler-conformance` (75 passed, 1 skipped); `cargo test -p tiler-artifact --doc`; `cargo test -p tiler-conformance --doc`; `cargo clippy -p tiler-artifact -p tiler-conformance --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-artifact -p tiler-conformance --no-deps`; `tkt lint`; `git diff --check`.
