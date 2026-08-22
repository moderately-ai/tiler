---
id: repair-the-artifact-identity-prose-the-v22-run-falsified
title: Repair the artifact identity prose the v22 run falsified
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, artifact, identity]
claimed_from: todo
assignee: worker-abiprose
lease_expires_at: 1787441390
---
## User-visible outcome

The normative descriptions of what an artifact manifest contains and what its canonical identity folds both name the physical-selection run, so a reader deriving either from prose gets the same answer the encoder gives.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit, immediately after the `v21`→`v22` step landed. Both sites are present-tense normative claims that the step falsified, and the audit's wider drift census over `v21`/`v22`/`(21,0)`/`91,945` found **everything else current** — the live source, the ABI identity ledger, the Metal profile authority ledger, and all six spike harnesses. These two are the exception, not a class.

**Fact — the ABI's ordered manifest enumeration omits exactly one element.** `docs/artifact-abi.md`, anchor `deferred predicates, live-device route requirements, executable entries, execution order`. This undated present-tense **Fact** paragraph is *the* ordered enumeration of manifest contents and the physical-selection run is missing from it. The same document's own v22 paragraph places the run "between the feasibility-rule revision and the deferred-predicate run", which the audit confirmed against `push_variant` directly: profile → feasibility key → feasibility revision → the run → deferred.

**Fact — the module doc's identity enumeration omits it too, while its own doctest was updated.** `crates/tiler-artifact/src/program/mod.rs`, anchor `and their entry mappings, and the capability providers`. It lists what canonical identity folds and stops at the selected capability providers; the run **is** folded. That the doctest below it moved and the prose did not is the tell.

**Fact — two sibling sentences are now role-unqualified.** The same ABI paragraph says "the selected capability providers" without saying *lowering*, and `program/mod.rs` at anchor `construction-time authority used to prove` still describes `CompilationEnvironment` in its pre-split single-role framing. Both were exact before the role separation and are ambiguous after it.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict; each anchor was grepped against the file it names and returns exactly 1.
- Repair each against the **encoder**, not against another document — derive the ordering from `push_variant` and the identity contents from the fold, and say which you read.
- Qualify the role-ambiguous sentences.
- Check the siblings of both sites: any other ordered enumeration of manifest contents, or of what identity folds, in either file or in the identity ledger. Report findings **and** clean results.

## Non-goals

Changing any encoded byte, ordering, or domain; re-deriving the v22 step, which landed gated; and repairing prose outside the two named files unless the sibling scan finds it.

## Worker audit, repairs, and sibling scan — 2026-08-22 at `6f3c2594b63040dd330bd5ceb95dabc82559aa24`

**Per-Fact verdict: all three verified, none false, none imprecise — no ticket Fact needed repair.** Each anchor was re-grepped against the file its citation names at this base and returns exactly 1, and each claim was then checked by full read of the *encoder*, never of a second document.

1. **Verified.** `docs/artifact-abi.md`'s ordered manifest enumeration omitted the physical-selection run. The position the ticket states is the position both encoders write and the decoder reads: `crates/tiler-artifact/src/program/model.rs`, anchor `fn push_variant(`, writes profile key, profile descriptor, feasibility key, feasibility revision, then `push_selected_physical_implementation_run(bytes, &variant.selected_physical_implementations);`, then the deferred run; `crates/tiler-artifact/src/program/codec/encode.rs`, anchor `fn encode_variants(`, writes the same order through the same shared run encoder; and `crates/tiler-artifact/src/program/codec/decode.rs` reads `let selected_physical_implementations = parse_selected_physical_run(cursor)?;` immediately before the deferred vector. The document's own v22 paragraph agrees, but the derivation above does not depend on it.
2. **Verified.** `crates/tiler-artifact/src/program/mod.rs`'s identity enumeration omitted the run. The fold is `crates/tiler-artifact/src/program/model.rs`, anchor `pub(super) fn encode_identity(`, which folds each variant through `push_variant` and therefore folds the run. The doctest below the prose already carries the post-split two-role call (`CompilationEnvironment::new([provider.clone()], [implementer.clone()])?`), confirming the ticket's tell.
3. **Verified, both siblings.** The ABI paragraph's providers are `SelectedLoweringProvider` rows (`crates/tiler-artifact/src/program/codec/encode.rs`, anchor `fn encode_provenance_tables`), so the unqualified "capability providers" was role-ambiguous. `CompilationEnvironment` now holds two independently bounded, canonicalized, and consulted roles (`crates/tiler-artifact/src/program/builder.rs`, anchors `pub struct CompilationEnvironment {` and `offered_physical_providers: Vec<ProviderIdentity>,`), so the single-role framing was stale.

### Repairs

- `docs/artifact-abi.md`, the ordered manifest enumeration: the variant clause now reads `each with its packaged program's section reference, guard, declared target profile and feasibility rule set, selected physical-implementation run, deferred predicates,` and the provider clause says `the selected lowering-capability providers`. Two further omissions the sibling scan found in the same "in this order" sentence are repaired with it, because a sentence that claims an order cannot be half-repaired: the semantic-subject run's fourth member (`the three reached semantic subjects and the retained shape environment`) and the delivered-realization record, which the encoder writes after the variants and before the section descriptors.
- `crates/tiler-artifact/src/program/mod.rs`: the identity enumeration names the retained shape environment and each variant's run of selected physical implementations, and qualifies the providers as lowering-capability; the `CompilationEnvironment` sentence states both roles and that the proof is of an offer *in the role it was selected for*.
- Four further live-source sites the sibling scan found, all inside `crates/tiler-artifact` and all corrected in place — see the findings below: `crates/tiler-artifact/src/program/codec/mod.rs`'s manifest-contents module doc, `crates/tiler-artifact/src/program/model.rs`'s `CanonicalArtifactProgramIdentity` doc, and three accessor/reader doc lines in `.../codec/model.rs` and `.../codec/decode.rs`.

### Sibling scan

Subject: any other ordered enumeration of manifest contents, or of what canonical identity folds, in the two named files or in the identity ledger.

**Findings.**

- *Same-sentence omissions in the ABI ordered enumeration*, repaired above. The retained shape environment is written by `crates/tiler-artifact/src/program/codec/encode.rs` inside the semantic-subject run and read at `crates/tiler-artifact/src/program/codec/decode.rs`, anchor `retained_shape: super::super::retained::RetainedShapeEnvironment::from_bytes(`; the variant's program-section reference is the variant record's first field (`let program_section = cursor.u32()?;`); the delivered-realization record is framed at `let realization = decode_realization(cursor.slice()?).map_err(|cause| {`.
- *`crates/tiler-artifact/src/program/codec/model.rs`, the semantic accessor* claimed to return "the three reached semantic subjects" while `SemanticSubjects` has carried four members since `v17`. A false present-tense claim in live code; corrected. The struct's own doc directly above it was already current and correctly names the fifth `SemanticIdentity` subject as travelling, so only the accessor line was stale.
- *Two role-ambiguous provider doc lines* — `crates/tiler-artifact/src/program/codec/model.rs`'s providers accessor and `crates/tiler-artifact/src/program/codec/decode.rs`'s `read_providers` — both return `SelectedLoweringProvider` and both said only "capability providers". Corrected, same class as Fact 3.
- *`crates/tiler-artifact/src/program/codec/mod.rs`'s manifest-contents enumeration* — present-tense normative module doc, and the closest live-code sibling of the ABI enumeration. It omitted the physical-selection run, omitted the delivered-realization record, said "selected capability providers" unqualified, and **claimed the manifest "carries the artifact's canonical identity once"**. That last clause is false independently of the v22 step: the manifest carries the *digest*, and the decoder re-derives the identity, digests it, and compares (`crates/tiler-artifact/src/program/codec/decode.rs`, anchor `return Err(ArtifactCodecError::ArtifactIdentityMismatch);`, reached from the comparison against `parsed.identity_digest`). All four corrected.
- *`crates/tiler-artifact/src/program/model.rs`'s `CanonicalArtifactProgramIdentity` doc carries three separate enumerations, and the run was missing from all three.* This is the doc the repaired module prose points readers to, so leaving it would have half-repaired the very claim the pointer forwards. Corrected: the "what the identity folds" list now names, per variant, which physical authority implemented each cover-region occurrence; the exclusions list's **"only reached admission provenance and selected capability providers do"** was a false *only* once the run began entering identity, and now names the lowering providers and each variant's selected physical implementations; and the pre-compilation-subject list names both selected roles. The run belongs on that last list on the section's own terms — physical selection is settled before backend emission, which is exactly why the v22 step exists to key an expansion cache on a miss.
- *Out of scope, filed as [`name-the-physical-selection-role-in-the-expansion-subject-facet-doc`](name-the-physical-selection-role-in-the-expansion-subject-facet-doc.md): `crates/tiler-cache/src/expansion/subject.rs`, the `SubjectFacet::ArtifactProgram` doc* names "selected capability providers" unqualified and omits the physical-selection run. `crates/tiler-cache/**` maps to `implementation/cache`, which this ticket does not hold, so it is reported rather than edited.
- *Both `three reached semantic subjects` sites above — the ABI enumeration and the codec accessor — are census misses of the closed [`repair-the-stale-three-carried-subject-claims`](repair-the-stale-three-carried-subject-claims.md)*, and the miss is a pattern-anchoring failure rather than a reading failure: that ticket's closing census greps the exact strings `three carried subjects` and `three subjects`, and every site missed here spells it `three reached semantic subjects`, which contains neither. Its **Fact** that `docs/artifact-abi.md` needed no repair was true for the sentence it examined and false for the enumeration it did not.

**Clean results — checked and found current, no change needed.**

- **The identity ledger is current in every artifact-domain value it states.** Cross-checked against `crates/tiler-artifact/src/domains.rs` and the defining constants, not against another document: `tiler.artifact-program.v22`, `.stage.v4`, `.provider.v3`, `.payload.v1`, `.deferred.v2`, `.physical-selection.v1`, `.delivered-realization.v3`, `tiler.target-environment-compatibility.v1`, `tiler.resolved-value-type.v3`, `tiler.schedule.v7`, `tiler.kernel.v9`, `tiler.kernel-program.v13`, `tiler.semantic-graph.v3`, `tiler.shape-env.v3`, `tiler.contract.f32.v2`, `tiler.contract.bf16.v1`, and `MANIFEST_SCHEMA` at `(22, 0)`.
- **The twenty-governed-domain arithmetic holds.** The crate's enumeration carries seven envelope domains, four proof-sidecar domains, and nine artifact-program domains — twenty, exactly as the no-prefix paragraph and its 2026-08-22 reopening both state.
- **No other crate carries a manifest-contents or identity-fold enumeration.** `tiler-runtime`, `tiler-build`, `tiler-compiler`, `tiler-ir`, `tiler-cache` (beyond the one site above), `tiler-conformance`, `tiler-metal`, `tiler-metal-aot`, `tiler-macros`, `tiler-reference`, `tiler-digest`, and `tiler` carry only single-fact sentences about artifact identity, never a list.
- **`spikes/` is entirely clean.** Every artifact-identity mention across the harnesses is the same single-fact boundary sentence about payload metadata versus object bytes; no spike restates manifest order or identity contents.
- **`docs/decisions/` is clean.** The only enumerations are ADR 0103's decision-time Context **Fact**, which is past-tense about `encode_manifest` and is a record of the state it decided against, and ADR 0072's running clause, whose subject is *kernel-program* identity rather than artifact identity.
- **The remaining coarse enumerations in `docs/` are clean at their own altitude** — `architecture.md`, `ir.md`, `operation-extensions.md`, `backends/metal.md`, and `prior-art/ug.md` each state one-line claims about what artifact identity covers, none of which the v22 step falsifies.
- **Dated and closed records were deliberately not touched**, by repository convention: `docs/research/artifacts/manifest-fixed-content-growth.md`, `docs/research/artifacts/target-neutral-artifact-envelope.md`, `docs/status.md`, and the closed tickets `prototype-neutral-artifact-codec`, `prototype-artifact-program-model`, `derive-the-pre-compilation-artifact-program-subject`, and `decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest` all carry pre-v22 enumerations as history.
- **`crates/tiler-artifact/src/program/codec/mod.rs`'s summary still omits live-device route requirements, execution order, stage dependencies, and the scope cells, and that predates this step.** It is a stated summary rather than an ordered enumeration, and those omissions are not v22 drift; flagged, not rewritten.
- **`crates/tiler-artifact/src/program/builder.rs`'s `read_semantic_interface` doc is correct**, and its "three carried subjects" hit is inside a dated 2026-08-19 correction note quoting its own retired wording. A grep hit there is evidence the string is present, not that the claim stands.
- **`crates/tiler-artifact/src/program/codec/model.rs`'s `SemanticSubjects` struct doc is current.**
- **`docs/artifact-abi.md`'s v22 paragraph, its `v17`, `v18`, `v20`, and `v21` step paragraphs, and `crates/tiler-artifact/src/program/codec/encode.rs`'s `MANIFEST_SCHEMA` doc all state the run's position and rationale correctly** and are consistent with the encoder.
- **`docs/artifact-abi.md`'s selected-provider row paragraph was left deliberately.** Its heading says "a selected capability provider row" without the role, but its body already says "the structured lowering-capability subject", and the closed [`publish-occurrence-bound-selected-physical-implementation-evidence`](publish-occurrence-bound-selected-physical-implementation-evidence.md) cites it by the anchor `a selected capability provider row carries two independent revisions`. Qualifying the heading would rot a live anchor in a closed record — failing as absence, the dangerous direction — to remove an ambiguity the same paragraph already resolves. Flagged rather than changed.
- **`crates/tiler-artifact/src/program/mod.rs`'s identity enumeration still omits execution order, stage dependencies, and the plan-determinism scope cells, and this was deliberate.** The paragraph immediately below it claims *every* listed subject is a compilation input and that identity is therefore derivable before the backend compiler runs. A `Plan` scope cell requires every entry's payload at that position to carry both a declaration and its object, so extending the list to the scope cells would place a pre-compilation-availability claim over them that this ticket did not establish. Left for a ticket that can derive it.

### Checks

`cargo nextest run -p tiler-artifact` (392 passed, 1 skipped), `cargo test -p tiler-artifact --doc` (3 passed), `cargo clippy -p tiler-artifact --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items -p tiler-artifact` all pass at this base.

**The doctest check was proven to reach its subject.** `cargo test --doc` names the `program/mod.rs` walk-through `crates/tiler-artifact/src/lib.rs - program (line 140)` — a merged-doctest attribution, not the file it lives in, so the name alone does not show the module doc is covered. Asserting `assert_eq!(1, 2, "perturbation reaches the program module doctest")` inside that walk-through reddens exactly that test: `test crates/tiler-artifact/src/lib.rs - program (line 140) ... FAILED`, `assertion \`left == right\` failed: perturbation reaches the program module doctest`. The perturbation was reverted.

## Closes when

Both enumerations name the physical-selection run in the position the encoder writes it, the role-ambiguous sentences say which role, the sibling scan is reported with its clean results, and `make citations` is green.
