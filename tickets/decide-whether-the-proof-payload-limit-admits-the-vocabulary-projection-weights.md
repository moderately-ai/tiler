---
id: decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights
title: Decide whether the proof payload limit admits the vocabulary-projection weights
status: in-progress
priority: p2
dependencies: []
related: [route-the-realization-conformance-half-into-the-conformance-crate]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-decide-wh
lease_expires_at: 1786159357
---
## The hard stop, measured

`w_vocab_slice` is the one L3 contraction cell that **cannot route**, and the reason is exact rather than approximate. Its `[8192, 1024]` weights operand is **33,554,432 bytes**; `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES` is **16,777,216** (`crates/tiler-artifact/src/proof/mod.rs`, search `MAX_PROOF_PAYLOAD_BYTES`). **Exactly a factor of two** — coordinator-verified by arithmetic and by the constant.

Observed as `Limit(ProofLimitExceeded { kind: PayloadBytes, attempted: 33554432, limit: 16777216 })`.

The conformance work that found this **did not touch `tiler-artifact`**: it derived the exclusion from the constant and pinned it to the doubling arithmetic with a test, so shrinking the cell's `n` to 4096 fails three tests at once. The exclusion is derived, not hand-asserted — which means raising or keeping this limit is a live decision rather than a number someone can quietly edit.

## What is actually being decided

`MAX_PROOF_PAYLOAD_BYTES` is **`pub`**, so it is a public boundary under ADR 0075 and its value is part of the artifact contract. Three readings, and they are genuinely different:

- **The limit is right and this cell is out of scope for proof payloads.** A 32 MB operand embedded in a proof is a different thing from a kernel's own bytes, and the vocabulary projection is the largest tensor in the pinned workload. If so, say what evidence *does* cover that cell, because it is currently the only L3 cell with none.
- **The limit is an arbitrary round number that has not been re-derived since it was set.** Then the question is what it should be derived *from* — a real bound on what a consumer must hold in memory to validate a proof, rather than a doubling.
- **The payload should not carry weights at all.** If a proof can reference an operand rather than embed it, the limit stops binding and the identity question moves instead. That is the largest change and the one that most needs stating before anyone raises a constant.

## Read before deciding

`crates/tiler-artifact/src/proof/` in full — particularly what a payload is required to contain and why a bound exists at all. The constant's own documentation is the first evidence; `AGENTS.md` ranks a reasoned bound above a round number, so establish which this is.

**Do not raise the limit as the default move.** A limit doubled to fit the one case that exceeded it is a limit that will be doubled again, and this repository treats that shape as a defect rather than a fix.

## Closes when

The reading is established with evidence; if the value changes, every pinned identity that folds it is recomputed on the merged tree and reported; and the L3 cell either routes or has its exclusion recorded with the evidence that covers it instead. A change to a `pub` constant in the artifact contract is Tom's under ADR 0075.

## Findings, 2026-08-07

Read in full at base `6d1bd6e8`: `crates/tiler-artifact/src/proof/{mod,model,builder,codec}.rs` and the limit sites in `tests.rs`; `docs/artifact-abi.md` "Proof-case evidence sidecar"; the consumption sites in `crates/tiler-conformance/src/{envelope.rs,envelope/tests.rs,publication/proof.rs}` (read, never edited — that crate has a live branch). No constant was changed.

### Per-Fact audit of the ticket's own claims

**Verified — the arithmetic and the observed refusal.** `MAX_PROOF_PAYLOAD_BYTES` is `16 * 1024 * 1024` at `crates/tiler-artifact/src/proof/mod.rs` (search `MAX_PROOF_PAYLOAD_BYTES`). `w_vocab_slice` is `m = 1, n = 8192, k = 1024` in `L3_CORRECTNESS_CELLS`; `largest_payload_bytes` maxes `m*k`, `n*k`, `m*n` and scales by `F32_BYTES = 4`, so its weights operand is `8192 * 1024 * 4 = 33,554,432` — exactly `2 ×` the bound. The refusal is raised at one of two `proof_limit(…, MAX_PROOF_PAYLOAD_BYTES, ProofLimitKind::PayloadBytes)` sites, `place` in `builder.rs` on the producer path and `encode_manifest` in `codec.rs`; the reader's third site is in `read_payloads`.

**Verified — the exclusion is derived, not hand-asserted.** `L3CorrectnessCell::fits_one_proof_payload` reads the constant through `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES` rather than a literal. `envelope::tests::the_routed_members_are_exactly_the_publishable_cells` derives the publishable set from that predicate and compares it with the hand-written `CONTRACTION_MEMBERS`; `the_unpublishable_cell_is_named_against_the_bound_that_stops_it` pins `limit == 16_777_216` and `largest_payload_bytes() == limit * 2`.

**Verified — `MAX_PROOF_PAYLOAD_BYTES` is `pub` and is therefore contract.** It is one of the eight `MAX_PROOF_*` budgets. `promote-the-proof-sidecar-facade`'s outcome records *why* they went public, and the reason is procedural rather than semantic: "the public constructors' `# Errors` sections link to them, and a rustdoc link from a `pub` item to a `pub(crate)` constant is a warning the gate converts into a failure; and it matches the `program` module's own posture." The promotion ticket's own enumeration of the reviewed surface did not list them. So the constant is contract, and its *value* has never been separately reviewed.

**Incidental, and it bears on the source above.** `promote-the-proof-sidecar-facade`'s outcome states that "`crates/tiler-artifact/tests/proof_sidecar_facade.rs` is a new integration test — a separate crate, so it links `tiler-artifact` the way a consumer does". That path does not exist at `6d1bd6e8` and `git log --all` shows it never existed; `crates/tiler-artifact` has no `tests/` directory. The property it claims *is* delivered, by the module-level doctest in `proof/mod.rs`, which says so in its own words ("a doctest compiles as its own crate, and naming an item that is not `pub` fails to compile rather than failing an assertion"). So the claim is right about the guarantee and wrong about the artifact. Flagged rather than fixed: correcting a closed ticket's outcome record is the coordinator's. The promotion rationale quoted above is separately corroborated — the eight budgets are in fact `pub` and the `# Errors` sections do link them.

**Imprecise — "the only L3 cell with no evidence."** The cell has no *routed conformance member*, which is the gap. It is not uncovered. `crates/tiler-reference/tests/contraction_profile_cells.rs` reproduces its retained `direct` digest `88b01ae7…` on the recorded M4 Max run (8,388,608 steps, 1 × 16,384 slabs, 79 ms), `the_staged_index_region_oracle_reaches_the_vocabulary_cell` walks its index region, and `envelope::tests` cross-checks the pinned digest against the retained `workload.tsv`. What is missing is the device-dispatch bit comparison against a published sidecar expectation inside `tiler-conformance` — one rung, not the whole ladder.

**False as stated — "a proof payload is required to embed the operand bytes, so the bound is a memory bound."** The bound guards no allocation the container bound does not already guard. `decode_proof_sidecar` checks `bytes.len() <= MAX_PROOF_SIDECAR_BYTES` (256 MiB) before anything else; `read_payloads` then reads a declared length, bounds it at `MAX_PROOF_PAYLOAD_BYTES`, and calls `Cursor::take`, which is *itself* bounded by `remaining()`. The copy is `content.to_vec()`, and the sum of all such copies is the sum of the framed lengths, which is already bounded by the container. Peak decode footprint is roughly `3 ×` the container — the caller's bytes, the `contents` copy, and the whole-container re-encode the canonicality backstop performs — and that is governed by the 256 MiB bound, sixteen times larger. On the producer side `place` bounds one `bytes.clone()` of bytes the caller already holds. The per-payload bound is a **semantic admission policy on the size of one operand**, not a resource guard.

### Which reading the evidence supports: reading 2, and the bound is not derived from anything

**The constant's own documentation is one line with no derivation:** "Maximum bytes of one case payload — one input or one expected output." Compare its immediate neighbour in the same block, which *is* derived and says so — `MAX_PROOF_INTERFACE_ENTRIES`: "Deliberately equal to the artifact model's own interface bound: a sidecar binds one payload per declared entry, so a looser bound here would admit a container no artifact could ever associate with." The module's `# Limits` section derives why a bound *exists* ("checked before any allocation proportional to it, in both directions") but nothing derives this bound's *value*. `docs/artifact-abi.md`'s "Governed budgets" Fact lists all nine and derives exactly two of them — the framed-payload bound from the case and interface bounds, and the interface bound from the artifact model's. "16 MiB per case payload" is stated and not derived there either. `git log -S MAX_PROOF_PAYLOAD_BYTES` returns one commit, `725f18b0` "Implement the proof-case evidence sidecar", whose message argues the association, the bound interface, and the deliberately underived storage width, and says nothing about any budget's value.

**`16 * 1024 * 1024` is this workspace's default large number, not a bound.** Twenty named constants equal it across six crates, and they bound four different *kinds* of quantity: canonical bytes (`MAX_OPERATION_RESULT_CANONICAL_BYTES`, `MAX_REGISTRY_CANONICAL_BYTES`, `MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES`, `MAX_BOUNDARY_CANONICAL_BYTES`, `MAX_INDEX_CANONICAL_BYTES`, `MAX_ACCESS_CANONICAL_BYTES`, `MAX_SCALAR_CANONICAL_BYTES`, `MAX_SCALAR_REGISTRY_CANONICAL_BYTES`, `MAX_KERNEL_IDENTITY_BYTES`, `MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES`, `MAX_REFERENCE_REGISTRY_IDENTITY_BYTES`, `MAX_SUBJECT_BYTES`, `MAX_PAYLOAD_SOURCE_BYTES`, `MAX_PROOF_PAYLOAD_BYTES`), element counts (`MAX_REFERENCE_TENSOR_ELEMENTS`), step counts (`MAX_EVALUATION_STEPS`, `REFERENCE_STEP_LIMIT` twice, `REFERENCE_DEFAULT_STEP_ALLOWANCE`), and proof cells (`MAX_FINITE_DOMAIN_PROOF_CELLS`). Two of these have already caused a documented misreading: `assemble-the-causal-self-attention-block-program` records a correction whose stated cause is "Both are `16 * 1024 * 1024`, which is how the two came to be read as one." A number that means bytes, elements, steps, and cells simultaneously is not carrying information about any of them.

**It is also internally inconsistent with the budgets around it.** `max_payloads()` is `MAX_PROOF_CASES * 2 * MAX_PROOF_INTERFACE_ENTRIES = 2,097,152`; at the per-payload bound that admits 32 TiB of payload against a `MAX_PROOF_SIDECAR_BYTES` of 256 MiB. The container bound is the one that actually binds a multi-payload sidecar, and the per-payload bound is the one that actually binds a single-payload one, and neither was chosen with reference to the other.

**The pinned workload is already against the ceiling.** Largest payload per cell: `w_decode_kv` 4 MiB, `w_prefill_q` 8 MiB, `w_prefill_o` 8 MiB, `w_prefill_mlp_in` 12 MiB, `w_prefill_mlp_out` 12 MiB, `w_vocab_slice` 32 MiB. Five of six clear the bound, and the widest of those clears it by `1.33 ×`. A single `f32` weight matrix at `n = 8192, k = 1024` — small for the class of model this profile is drawn from — exceeds it. This is not one outlier cell against a considered bound; it is a bound that the workload it was never measured against has already overtaken, and `w_prefill_mlp_in` at 12 MiB is the next one to go.

**Reading 1 is not supported.** It would require the bound to encode a deliberate policy that a 32 MiB operand is out of scope for proof payloads. Nothing states such a policy, and the same document that would have to carry it instead states the opposite posture: the identity "folds payload *digests* rather than payload bytes … That is what keeps it usable as a key for a sidecar carrying **megabytes of evidence**." The container was designed for exactly this size class.

**Reading 3 is architecturally reachable but does not pay for itself here.** The manifest already carries, per payload, `(canonical ordinal, exact length, content digest)`, and `derive_identity` folds only those digests — so an "external payload" variant would leave the sidecar identity bit-identical and is a smaller change than it sounds. It fails on the consumer side rather than the format side: the runner must materialize the full operand regardless, because it has to upload it to a device buffer to dispatch the kernel at all, so referencing rather than embedding saves the runner nothing at the moment the bound is claimed to protect. It also converts the container's central property — "holding a `DecodedProofSidecar` is itself the evidence that the bytes passed every check" — into integrity-of-a-reference, needing a resolution mechanism, a new rejection vocabulary, and a decision about what an unresolvable reference means. Worth stating as the ticket asked; not worth doing for this.

### One contract consequence that is independent of which value is chosen

The budgets are **not versioned**, and `SIDECAR_FORMAT`/`MANIFEST_SCHEMA` do not fold them. A producer built against a raised bound emits a sidecar an older reader refuses as `ProofCodecError::Limit(ProofLimitExceeded { kind: PayloadBytes, … })`, which `classification` maps to `ProofFailureClass::Limit`. That class is documented as the resource-exhaustion answer, while `Unsupported` is the version-skew answer — and `ProofFailureClass`'s own doc says "The classes answer different questions, and collapsing them would make a version skew look like corruption." Here it is the reverse: a genuine version skew presents as a resource limit. Any decision that moves this value should decide at the same time whether the move bumps the manifest schema minor.

### The population that would move under a change

**No content-addressed identity moves.** Verified by full read of `derive_identity` and `encode_manifest`: neither folds any `MAX_PROOF_*` constant. The sidecar identity, the artifact identity, the envelope digest, and every expansion-cache key over them are unaffected, and no already-published member's bytes change. Admitting `w_vocab_slice` *adds* a member with a new identity rather than moving an existing one. The cost the ticket anticipated under "every pinned identity that folds it" does not exist.

What moves is a set of pinned assertions and derived populations, all outside `implementation/artifact` except the first two:

1. `crates/tiler-artifact/src/proof/mod.rs` — the constant and its doc line. *(`implementation/artifact`)*
2. `docs/artifact-abi.md`, "Governed budgets" — "16 MiB per case payload" in the nine-bound sentence. *(`contracts/artifacts`)*
3. `crates/tiler-conformance/src/envelope.rs` — the module doc's "Which cells this routes, and the two bounds that decide it" list; `fits_one_proof_payload`'s doc; `CONTRACTION_MEMBERS`, which is `[ContractionMember; 6]` with a hand-written `l3_member(0..=4)` and would become `; 7]` with `l3_member(5)`; and that constant's doc, which currently explains the absence.
4. `crates/tiler-conformance/src/envelope/tests.rs` — `the_unpublishable_cell_is_named_against_the_bound_that_stops_it` in full (it asserts the literal `16_777_216`, the `limit * 2` relation, and `UNPUBLISHABLE_CELL_CLASS`'s existence); `the_routed_members_are_exactly_the_publishable_cells` (`routed.len() == 5`, and the "exactly one excluded class" assertion).
5. `crates/tiler-conformance/src/publication/proof.rs` — `the_published_contraction_extents_are_the_ones_this_module_is_written_for` (`declared.len() == 6`); and `cases_for` needs an operand table for `1 × 8192 × 1024`, which it currently has no row for.
6. `crates/tiler-artifact/src/proof/` — there is **no** negative test for `MAX_PROOF_PAYLOAD_BYTES` anywhere in the owning crate. The only limit test in `proof/tests.rs` is for `MAX_PROOF_CASES`. The single assertion on this bound's value lives in another crate. A change here should add the missing one rather than continue relying on `tiler-conformance` to notice.
7. Ticket text: `route-the-realization-conformance-half-into-the-conformance-crate.md` (three sites) and this ticket.

Items 3 through 5 are `implementation/conformance`, which has a live branch and was not touched.

### Recommendation

**Derive the bound from the container rather than from the workload, and stop having a separate number.** `MAX_PROOF_PAYLOAD_BYTES` should be a `const` expression over `MAX_PROOF_SIDECAR_BYTES` — the largest single payload a sidecar whose total encoding fits the container bound could carry, which is `MAX_PROOF_SIDECAR_BYTES` less the fixed 69-byte header, the manifest, and the per-payload framing. That is the honest answer to "what must a consumer hold to validate a proof": the container bound already answers it, the per-payload bound never added to it, and a derived expression cannot be doubled to fit a case because there is no round number left to double. It also gives the constant the shape its neighbour `max_payloads()` already has — "derived from the case and interface bounds rather than declared, so the framing bound and the structural bounds cannot disagree" — and the shape `MAX_PROOF_INTERFACE_ENTRIES` already has. `w_vocab_slice` is then admitted as a consequence rather than as the reason.

This is Tom's under ADR 0075 and is not taken here. Two things go with it if it is taken: the schema-minor question in the previous section, and the missing negative test in item 6.

**Strongest counterpoint.** A derived bound of roughly 256 MiB is a *much* weaker per-payload statement than 16 MiB, and it is fair to say it does not bound the operand size at all — it lets one case payload consume the entire container. If the sidecar ever needs a real statement about how large one operand may be — because a consumer streams payloads, or because a proof runner wants to size a staging buffer without decoding — then the bound to want is a small, stated, *reasoned* one, and deriving it away now removes the seam where that reasoning would go. The counter to the counterpoint is that no such consumer exists: `decode_proof_sidecar` is whole-container by construction, requires the complete byte slice up front, and re-encodes the whole thing as its canonicality backstop, so there is no streaming reader for a per-payload bound to serve. If one is ever built, it needs a *new* bound derived from its own buffer strategy, and a 16 MiB number that predates it would not be that bound either.

**What must not happen:** raising 16 MiB to 32 or 64 MiB. `w_prefill_mlp_in` and `w_prefill_mlp_out` already sit at 12 MiB, and the profile this workload is drawn from scales `n` and `k` together, so the next cell added would arrive with the same argument for the next doubling.
