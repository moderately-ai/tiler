---
id: prototype-proof-case-sidecar
title: Implement the proof-case evidence sidecar
status: done
priority: p0
dependencies: [prototype-neutral-artifact-codec, prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/reference, implementation/artifact, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, artifact, numerics]
---
Implement a separate versioned proof sidecar containing stable case keys, bit-preserving inputs, normative expected bytes, semantic/numerical/reference identities, digests, and exact envelope association. Validate limits, uniqueness, corruption and mismatch; never make it runtime artifact semantics.

## Outcome

`crates/tiler-artifact/src/proof/` holds the bounded, versioned proof-case evidence sidecar: a transactional builder, a canonical encoding, a fail-closed reader, and two explicit envelope-association checks. It landed as a **crate-private draft authority** under ADR 0074 convention 7 — a private `mod proof` in the crate root whose items are `pub(crate)`, with the module-level `#![allow(dead_code, unused_imports, reason = "…")]` naming what it reserves and which slices consume it. **Promotion to a public facade is Tom's under ADR 0075 and has not been made**; `promote-the-proof-sidecar-facade` owns it and is now a dependency of `prototype-metal-aot-slice`.

151 tests pass in `tiler-artifact`, 46 of them the sidecar's own (`grep -c '#\[test\]' crates/tiler-artifact/src/proof/tests.rs`), and the complete gate `uv run --locked python scripts/check_repository.py` is green.

### The separation from artifact semantics is structural, not a convention

`tiler.research.workspace.prototype-crate-layout-and-msrv` states the rule — "The sidecar is not part of artifact semantics" — and this implementation makes it checkable rather than asserted. Nothing in `crate::program` references `crate::proof`; no envelope section carries a proof case; an artifact decodes, validates, and would dispatch with no sidecar present. The dependency runs one way: a sidecar names an artifact, an artifact never names a sidecar.

The two containers share exactly two things, and the sharing is argued rather than convenient. The **governed digest algorithm**, because `docs/artifact-abi.md` requires every digest use in this crate to name one governed algorithm explicitly, and a sidecar that chose its own would be unverifiable by a reader that knows only the governed tag. And **`envelope_digest`**, because that function *is* the association. Both reach `crate::proof` through four named `pub(crate)` re-exports on `crate::program` (`DIGEST_BYTES`, `Digest`, `DigestAlgorithm`, `envelope_digest`) rather than through a crate-visible `mod codec`; the crate-visible module was tried first and rejected, because it raised six `private_interfaces` warnings by exposing `pub(in crate::program)` row types the codec deliberately keeps internal. Framing, schema, vocabulary, limits, and failure classification are the sidecar's own.

### Two facts the builder derives instead of accepting

**The association.** `ProofSidecarBuilder::new` takes the `VerifiedArtifactProgram` itself, encodes it, and digests the exact bytes. A producer cannot pair a sidecar with an identity it did not compute or bytes it did not write — the failure mode an `(identity, digest)` parameter pair would have made easy and silent. This is the "carry complete identity across boundaries" rule applied by removing the opportunity to get it wrong.

**The bound interface.** The keys a case supplies payloads for are the artifact's own declared inputs and outputs, in the artifact's interface order. A case names its keys; the builder places them. It cannot introduce an undeclared key, omit a declared one, or reorder them, and each of those three is a separately named rejection because a producer reacts differently to each.

One fact is deliberately **not** derived: the semantic graph the expectations were evaluated over. It is supplied as a typed `SemanticGraphIdentity` and compared against the artifact's, because the risk the check exists to catch is a producer that reference-evaluated a different program from the one it compiled. Deriving it would make the check tautological.

### What was checked, and the one thing that was not invented

Per case: stable key validity and uniqueness, one payload per declared entry in each direction, per-payload byte bounds. Across cases: every case agrees on each interface entry's byte length — provable from the sidecar alone, because an artifact's declared shapes are fixed. Against the artifact: key-by-key interface agreement, and every payload a whole number of elements of its declared shape.

**A width was deliberately not derived.** `bytes / element_count` is checked to be a whole number at least one, but the absolute storage width is never asserted: `tiler_ir::kernel::KernelType` exposes no width, `Bool` is documented as "a one-bit control predicate" whose buffer width is a backend fact this crate does not own, and the enum is `#[non_exhaustive]` so a total map would need a rejecting wildcard. Inventing a width would have asserted a byte count no verifier examined. The divisibility-plus-agreement pair catches the same class of producer error without the invention, and the boundary is recorded in `ProofInterfaceError::PayloadNotWholeElements`'s own documentation.

### Association has two strengths, and they were separated on purpose

`DecodedProofSidecar::bind_to_envelope(&[u8])` is for a consumer holding only bytes: it re-derives the envelope digest over the caller's own bytes, decodes them through the artifact codec, and compares the re-derived artifact identity with the recorded one. `bind_to_artifact(&VerifiedArtifactProgram)` is for a consumer holding the program: same identity comparison, plus a local re-proof of every interface obligation.

Both establish the *same* association — the artifact's canonical identity already folds the ordered named interface — so the second is not a stronger association but a locally re-proven one, which is what a reader wants when the sidecar was written by an older producer. Crucially, the obligation has **one** implementation (`verify_cases`), called by the builder's terminal and by `bind_to_artifact`; a producer-side copy and a consumer-side copy would agree today, drift later, and each half would still pass its own tests.

### A measurement boundary, pinned as a test rather than left as a claim

A validated sidecar is evidence of **integrity and association**. It is **not** evidence of **authenticity**: every digest and identity is derived from the container's own content, so a forger that rewrites an expected value recomputes them all and the result validates and binds. `a_forged_case_is_indistinguishable_from_a_real_one_by_the_container_alone` asserts exactly that, so the limit is a checked-in fact instead of a sentence someone could later mistake for a stronger guarantee. What protects a proof run is the device comparison downstream — a forged expectation makes a correct device fail loudly — which is why `prototype-metal-runtime-proof`'s framing of sidecar payloads as *test data*, never as artifact semantics or an independent reference, is a correctness requirement rather than a stylistic preference.

### Fixture reuse over a second fixture

`crate::program::tests` became `pub(crate)` under `cfg(test)`, and seven of its fixtures moved from `pub(super)` to `pub(crate)`. The sidecar associates with a *real* verified artifact — a real semantic program, a real verified kernel program, a real artifact envelope — and a hand-built second one would be 400 lines that could drift from the artifact model's own. One fixture bug was found and fixed while writing the suite: the "other artifact" fixture originally varied only the kernel's scale constant, which leaves the semantic graph identity unchanged, so the semantic-subject mismatch case was vacuous until it was rebuilt on `build_graph_scaled(…, 3.0)`.

### One gap left open on purpose, with its trigger

No governed contract records the sidecar's format. `docs/artifact-abi.md` deliberately should not: the sidecar is not artifact semantics, and that document's "The governed digest" section correctly describes three envelope domain separators while the sidecar adds four of its own. The union property those separators depend on — no admitted domain prefixing another — is checked across all seven in `crate::proof::tests`, because one algorithm hashes both containers in one process, and the envelope codec's own three-domain test carries no note pointing at it.

Writing the format into a contract now would describe a format no crate can reach. The trigger is the facade promotion, so the closing condition is recorded on `promote-the-proof-sidecar-facade` rather than filed as a ticket that would sit ready and unactionable.

### What is reserved and not implemented

The sidecar carries no case *grouping*, no per-case tolerance, no comparison policy, and no execution ordering. A comparison is a bitwise equality against the recorded bytes, which is the only comparison the numerical contract admits and the only one a container that never interprets its payloads can honestly support. Nothing here dispatches, allocates, or reads a device.
