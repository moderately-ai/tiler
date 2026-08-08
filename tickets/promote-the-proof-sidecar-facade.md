---
id: promote-the-proof-sidecar-facade
title: Promote the proof-sidecar facade
status: done
priority: p0
dependencies: [prototype-proof-case-sidecar]
related: [prototype-metal-aot-slice, prototype-metal-runtime-proof]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, api, review]
---
The proof-case evidence sidecar landed as a crate-private draft authority in `crates/tiler-artifact/src/proof/` under ADR 0074 convention 7. Its two intended consumers are out-of-crate binaries — the producer `prototypes/serial-sum-compile` and the runner `prototypes/serial-sum-run` — and neither can reach a crate-private module, so the capability exists and is unreachable until its facade is accepted.

**Fact — this is ADR 0075's always-ask category, twice over.** Making the sidecar reachable means a new `pub mod proof` in the `tiler-artifact` crate root, which is "a new publicly reachable namespace", and promoting items from `pub(crate)` to `pub`, which is its own always-ask item. Neither is the coordinator's to merge. The precedent is exact: `prototype-neutral-artifact-codec` landed the envelope codec crate-private, and `carry-the-metal-payload-in-an-artifact-envelope` promoted `decode_artifact`, `DecodedArtifact`, and the payload vocabulary on Tom's review on 2026-07-25.

**Fact — the surface to review is bounded and already written.** `crates/tiler-artifact/src/proof/mod.rs` re-exports it in one place: the builder (`ProofSidecarBuilder`, `ProofCaseSpec`, `ProofProvenance`), the product and its read views (`VerifiedProofSidecar`, `ProofCaseRef`, `ProofPayloadRef`, `CanonicalProofSidecarIdentity`), the vocabulary (`ProofCaseKey`, the three provenance subjects), the reader (`decode_proof_sidecar`, `DecodedProofSidecar`), and the four error enums plus their classification. Every item conforms to ADR 0074 conventions 1 through 6 and is tested; what remains is the interface decision, not the implementation.

**Two questions worth putting to Tom with the diff**, rather than deciding here:

- Whether the whole surface is promoted, or only the *capability* the way the envelope codec's was — bytes out, a validated view back, accessors rather than types — leaving the builder crate-private until a producer needs it. The envelope codec's promotion took the narrower shape deliberately.
- Whether `DecodedProofSidecar::bind_to_artifact` is promoted at all. It is the stronger check and it requires the caller to hold a `VerifiedArtifactProgram`, which is exactly the binding-by-identity posture that `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is deciding. Promoting it commits a second public call site to that posture before that ticket closes.

**Not closed by** widening visibility without review. A crate-private authority that is unreachable is the accepted staging state (ADR 0074 convention 7), not a defect to be worked around.

## Also closes the contract gap the sidecar deliberately left open

**Fact — no governed contract records the sidecar's format.** `docs/artifact-abi.md` should not be the one: the sidecar is not artifact semantics, and its "The governed digest" section correctly describes the envelope's three domain separators while the sidecar adds four of its own (`tiler.proof-sidecar.manifest.v1`, `…manifest-digest.v1`, `…payload-digest.v1`, `…identity.v1`). Writing the format down while nothing outside `tiler-artifact` can reach it would document an unreachable format, which is why `prototype-proof-case-sidecar` recorded the gap with this promotion as its trigger rather than filing a ticket that would sit ready and unactionable.

**Two things the contract must state, whichever document takes it.** That the sidecar is producer evidence and never artifact semantics — a sidecar names an artifact, an artifact never names a sidecar, and an artifact validates and dispatches with none present. And the measurement boundary the implementation already pins as a test: a validated sidecar is evidence of *integrity and association* and not of *authenticity*, because every digest and identity in it is derived from its own content, so a re-sealed forgery validates and binds. A consumer that treats sidecar payloads as anything but test data has read a guarantee the container does not make.

**Fact — one union property crosses the two containers and is checked in only one place.** `crate::proof::tests::no_governed_domain_of_this_crate_prefixes_another` checks all eighteen governed domains together, because one algorithm hashes both containers in one process; the envelope codec's own three-domain test carries no note pointing at it. Whoever takes this ticket should decide whether that cross-reference belongs in the codec's test or in the contract.

## Decision — Tom, 2026-07-25

**Approved: promote the proof-sidecar facade.** The container is crate-private today and the producer and runner are different crates by construction, so a public facade is the only shape that lets a case written by one be verified by the other. This unblocks `prototype-metal-aot-slice`, which was otherwise dispatchable straight into an unreachable API.

The container's stated limit does not change with promotion: authenticity is not claimed, a re-sealed forgery validates and binds, and `a_forged_case_is_indistinguishable_from_a_real_one_by_the_container_alone` exists so a reader cannot infer a stronger guarantee from a public API than the private one offered.

## Outcome

**Done.** `tiler_artifact::proof` is a `pub mod` and the surface this ticket enumerated is `pub`. `crates/tiler-artifact/tests/proof_sidecar_facade.rs` is a new integration test — a separate crate, so it links `tiler-artifact` the way a consumer does — which builds an artifact through the public `program` API, writes a two-case sidecar through the builder, encodes it, decodes it through the public reader, asserts a byte-identical re-encode, runs **both** association checks, and reads an expected payload back bit for bit. That is the approval's stated property executed across a real crate boundary; a crate-private module passes every in-crate test and delivers none of it.

The complete gate is green and `tkt guard` reports no scope escape against `06be974`.

### Both shape questions are resolved by research, so neither went to Tom

**Question 1 — whole surface, or the narrow capability shape the envelope codec took.** The whole surface, and the envelope-codec analogy does not transfer. That promotion could be narrow because the *producer* entry point was already public: `VerifiedArtifactProgram` existed, and `encode` was a method on it. The sidecar has no such entry point. `ProofSidecarBuilder` is the only way any code can write a case, so "leaving the builder crate-private until a producer needs it" describes a facade through which no case can ever be written — the producer half of the approval's property is unreachable without it. Every remaining item is either constructed by the producer (`ProofCaseSpec`, `ProofProvenance`, `ProofCaseKey`, the two opaque provenance subjects) or named in the type of something a consumer receives (`VerifiedProofSidecar`, `DecodedProofSidecar`, `ProofCaseRef`, `ProofPayloadRef`, `CanonicalProofSidecarIdentity`, `ProofSemanticSubject`, the four rejection vocabularies and `ProofDirection`, `ProofOrderedSubject`, `ProofLimitKind`, `ProofLimitExceeded`, `ProofFailureClass`).

The eight `MAX_PROOF_*` budgets were promoted alongside, which the ticket's enumeration did not list. Two reasons: the public constructors' `# Errors` sections link to them, and a rustdoc link from a `pub` item to a `pub(crate)` constant is a warning the gate converts into a failure; and it matches the `program` module's own posture, where every governed bound is `pub`.

**What deliberately stayed private, and why.** The wire form: `MAGIC`, `HEADER_BYTES`, `SIDECAR_FORMAT`, `CANONICAL_ENCODING`, `MANIFEST_SCHEMA`, the four domain separators, `encode`, `derive_identity`, `proof_limit`, `Cursor`, `FromSubjectBytes`, `project_interface`, `verify_cases`, `verify_case_payloads`, `BoundInterface`, `InterfaceProjectionError`, and the retained storage records `ProofSidecarData`, `ProofCaseData`, `ProofSubjects`. A public domain separator invites an out-of-crate caller to digest a subject under this container's domain, and a public encoder invites bytes the reader did not derive; neither is needed for a case written by one crate to be verified by another, and ADR 0074 convention 7 keeps what is not needed private. `CanonicalProofSidecarIdentity`'s field stays `pub(in crate::proof)`, so the promoted type still has no constructor outside the encoder that establishes what it means (convention 2).

**Question 2 — `bind_to_artifact`.** Promoted. **The ticket's stated reason for holding it back has expired:** `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is `done`, decided by Tom on 2026-07-25, and it decided *in favour of* the posture the ticket worried about committing to — a decoded envelope is a dispatch record, never a reconstruction, so a consumer that needs the program holds the one it compiled and the envelope binds by identity. Promoting a second call site onto a decided posture costs nothing.

Withholding it was tested against the alternative and does not survive on its own merits either. `ProofAssociationError::Interface` and `::Interfaceable` are produced by `bind_to_artifact` and by nothing else, so a private `bind_to_artifact` leaves two variants of a promoted error enum unreachable from every public entry point — a consumer matching them writes unreachable code. It would also require a targeted `#[allow(dead_code)]` on a public module and a rewrite of the module documentation's "two association strengths" section to stop naming a private item. Both costs buy nothing: no consumer is forced into a weaker check by promotion, because `bind_to_envelope` is exactly the check a bytes-only runner needs.

**Recorded so a reader does not mistake reachability for use:** under the dispatch-record decision a cold consumer *cannot* call `bind_to_artifact`, because no decoder produces a `VerifiedArtifactProgram`. Its caller is a process that compiled the artifact — the producer validating its own output before shipping it. The contract states this rather than leaving it to be discovered.

### The contract landed in `docs/artifact-abi.md`, and this ticket's argument against that is retracted

The ticket argued that document is the wrong home because "the sidecar is not artifact semantics". **Retracted**, for three reasons found by reading the document.

1. Both properties the ticket requires are *relational*. "An artifact never names a sidecar, and validates and dispatches with none present" is a constraint on what this document's decoders and runtimes may require. Stating it anywhere else creates a second authority over artifact semantics; stating it here is a negative ownership claim in the one document a reader consults to ask whether an artifact carries proof cases. The new section leads with the separation rather than burying it as a caveat.
2. The document's "The governed digest" section was already **wrong by omission**, and the fix belongs where the claim is. It read "Three domain separators are governed … and a test proves no admitted domain is a prefix of another". The obligation is over the crate's seven, one algorithm hashes both containers in one process, and the test that establishes it lives in `crate::proof::tests`. Splitting the digest-domain authority across two documents to avoid mentioning the sidecar would have preserved the defect.
3. Scope. `contracts/artifacts` maps to `docs/artifact-abi.md` and `docs/backends/**` only; a new governed document needs `contracts/navigation` edits this ticket does not hold. **Stated separately because it is a constraint, not an argument** — reasons 1 and 2 decide the home, and this one would not have on its own. The trigger for splitting the sidecar into its own contract is a second consumer of the format outside `tiler-artifact`'s own producer/runner pair, at which point the container stops being a detail of how this crate's artifacts are proven.

The new `## Proof-case evidence sidecar` section records the separation and the authenticity boundary as normative statements; the framing, canonical manifest, payload stream, the four domains, the two association checks, the governed budgets, and the rejection classification as facts; and — kept visibly apart — what is reserved, what the container cannot acquire at all, and that no producer or consumer ships one yet.

### The cross-container digest cross-reference went into the codec's test

Both, in fact, and each says what it is. `crate::program::codec::digest`'s `no_governed_domain_is_a_prefix_of_another` now states that it covers three of the crate's seven domains, names the union test as the authority for the property, and says a fourth envelope domain must be added to both. That is where a worker adding an envelope domain looks. The contract states the union obligation normatively, because a test cannot tell a future container's author that the obligation exists.

The test's own doc comment previously read "Every governed domain is a fixed constant of this crate, so the property is checkable here" — true of three domains and false of seven the moment the sidecar landed. That sentence, not the missing pointer, was the defect.

### One adjacent staleness fixed, and one found and not fixed

**Fixed, because this change contradicts it.** `docs/artifact-abi.md` stated that every codec item is `pub(crate)` and that "no crate outside `tiler-artifact` can encode or decode an artifact and no consumer surface has been accepted". False since `carry-the-metal-payload-in-an-artifact-envelope` promoted the codec's capability on 2026-07-25 — `prototypes/serial-sum-compile/src/main.rs:59` imports `decode_artifact`. Writing a facade-status section for the sidecar beside a false facade-status claim for the envelope was not an option, so the status line, the opening Fact, and "Maturity of the implementation" now record which items were promoted and which stayed `pub(crate)`.

**Found and not fixed, needing a ticket.** Two sentences in the same document are overtaken by `carry-reconstructable-kernel-programs-in-the-neutral-envelope` closing: under "Deliberate exclusions", "`carry-reconstructable-kernel-programs-in-the-neutral-envelope` owns deciding what a decoded envelope must reconstruct" — it decided; and item 3 of "Where the implemented profile is narrower than this contract", which records the reconstruction gap as open rather than as a decided contract. Correcting them means stating the dispatch-record decision and its implementation state in the contract, which is `expose-the-dispatch-record-on-a-decoded-artifact`'s subject and which that ticket could not do, holding only `implementation/artifact`. Not taken here rather than written from a state this ticket did not verify.
