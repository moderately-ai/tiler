---
id: record-the-settled-governed-key-grammar-in-the-contracts
title: Record the settled governed-key grammar in the contracts
status: in-progress
priority: p2
dependencies: []
related: [reconcile-the-two-target-profile-key-grammars]
scopes: [contracts/artifacts, contracts/decisions, contracts/foundation, research/extensions]
shared_scopes: []
paths: []
tags: [identity, documentation]
claimed_from: todo
assignee: worker-key-grammar
lease_expires_at: 1785581736
---
## User-visible outcome

No contract or accepted decision still describes the artifact layer's governed-key grammar as unsettled or as length-only, and the shared `TargetProfileKey` spelling is indexed where a reader looking up an ambiguous name will find it.

## Why this slice exists

**Fact.** `reconcile-the-two-target-profile-key-grammars` settled the question: the artifact layer's six `governed_key!` types now enforce the same alphabet as `tiler_compiler::target::TargetProfileKey` (ASCII lowercase, digits, `.`, `-`, `_`), while the byte bounds stay deliberately different — 128 is one producer's minting bound, 256 is the artifact layer's admission bound. That ticket held only implementation scopes, so three sentences it invalidated are still in the tree:

- `docs/artifact-abi.md:101` records the asymmetry as one that ticket "owns and that record deliberately does not settle".
- `docs/artifact-abi.md:286` says a governed key "is bounded at 256 UTF-8 bytes because this layer governs what a producer may name" without mentioning that the layer now also governs the spelling.
- `docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md:103` states as **Fact** that "there is no alphabet, case, separator, or namespace check in the crate", with a reproduction (`grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/`) that no longer returns nothing; `:149` lists "whether the artifact layer should enforce the compiler's key alphabet" as an open question.

**Inference.** ADR 0090's item 10 itself is not superseded — it governs namespace *minting*, not spelling, and the record already separates the two ("and separately from the namespace question"). What changed is the Fact paragraph beside it and the open question it deferred, so this is a correction and a closed question rather than a new decision.

The glossary row is the second half. `docs/glossary.md` already indexes names denoting several unrelated subjects, and `TargetProfileKey` — one type in `tiler-compiler` that a compilation is assessed against, one in `tiler-artifact` that a packaged program carries, with different bounds and, until now, different grammars — is not among them. The research record that surfaced this (`docs/research/extensions/backend-provider-composition.md`) got it wrong precisely by using one of each and describing them in one sentence, which is the failure mode a glossary row exists to prevent.

## Implementation keys

- The rustdoc at `crates/tiler-artifact/src/program/keys.rs` and `crates/tiler-compiler/src/target.rs` is the settled contract for the code-level subject and already carries the derivation; these edits cite rather than re-derive it, and must not restate it in a way that can drift.
- Re-run each reproduction command before rewriting the sentence that prints it. ADR 0090's is now a positive control rather than a negative one, and a record whose stated reproduction does not reproduce is worse than one that is merely stale.
- Correct the research record's finding 2 in the same pass; it is the origin of the conflation.
- Do not restate the 128-versus-256 difference as an unresolved gap. It is a decided asymmetry with a direction argument, and a contract sentence that reopens it invites a future worker to "fix" it by narrowing the artifact bound to one producer's number.

## Closes when

Every sentence above states current behaviour, the ADR's open question is closed by pointing at where its answer lives, and the glossary indexes the shared `TargetProfileKey` name with both subjects.

## Outcome

Documentation-only, at base `7ad2aca`. Six sentences corrected across four files, plus two glossary rows and three drifted source citations. No file under `crates/`, `spikes/`, or `prototypes/` was touched.

**Two of this ticket's three cited locations had moved, and one had already been half-corrected.** The `docs/artifact-abi.md:286` sentence is at `:295` after the `v13` delivery-position step. And ADR 0090's `:103` **Fact** paragraph was already given a `**Corrected 2026-08-01:**` appendix by `446f6fb` ("Close the D10 promotion and correct the record's two staleness items") a few hours before this ticket was claimed. That correction was right about the alphabet and left three things wrong that this ticket's own implementation keys name: it deleted the reproduction sentence outright rather than re-running it, so the paragraph asserted a positive claim with no reproduction at all; its bold **Fact** lead still stated the falsified asymmetry in the present tense, which is what a reader skimming bold leads takes away; and its `keys.rs:73-85` citation is present-tense while the validator has moved to `keys.rs:124-143`.

**Reproductions, re-run at `7ad2aca` before each sentence was rewritten.**

- `grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/` → one line, `crates/tiler-artifact/src/program/keys.rs:121`, the `admits` closure. It returned nothing at `e6a47d9`, so this record's evidence of *absence* is now its positive control, which is what both ADR 0090 and the research record now say in place of the old claim.
- `grep -n "is_ascii\|InvalidByte" crates/tiler-compiler/src/target.rs` → six lines (200, 252, 264, 1166, 1194, 3425). Still six, as the original positive control recorded; the admitted-byte closure it cited at line 226 is now at 252, so that citation is stated as historical rather than silently refreshed.
- `grep -rn "NoncanonicalKeyByte" crates/tiler-artifact/src/` → the variant at `program/error.rs:221`, its refusal site at `keys.rs:136`, its exhaustiveness arm at `error.rs:528`, and seven asserting tests at `program/tests.rs:2857-2915`. This is the check that can say no, and the tests are where it does.

**Sentences changed, old → new.**

1. `docs/artifact-abi.md:101`. Old: "It also records that the artifact layer's governed-key validator enforces length alone while the compiler's same-named `TargetProfileKey` enforces an alphabet, an asymmetry [`reconcile-the-two-target-profile-key-grammars`] owns and that record deliberately does not settle." New: past tense for what ADR 0090 recorded, then "[the ticket] has since settled it, and the two now admit the same alphabet while their byte bounds stay deliberately apart — see 'Governed budgets' below for what this layer enforces and why the bounds differ." The alphabet is named once in this file rather than at both sites, so a widening cannot leave one of them stale.
2. `docs/artifact-abi.md:295`, first sentence. Old: "A governed key is bounded at 256 UTF-8 bytes because this layer governs what a producer may name." New: the key is bounded at 256 UTF-8 bytes *and* spelled in ASCII lowercase, ASCII digits, `.`, `-`, and `_`, "because this layer governs what a producer may name — the spelling as much as the length", with the byte-comparison ground; then the compiler admitting exactly that alphabet so every key it mints is packageable, and the bounds deliberately *not* agreeing because 128 is a minting bound and 256 an admission bound, with the smaller-mints-into-larger direction stated as what makes the difference safe rather than a gap.
3. `docs/decisions/0090:103`, the item-10 **Fact** lead and its reproduction. Lead moved to past tense ("was weaker … which was a real asymmetry"), the drafting-era line citation is marked drafting-era, the consequence the original Fact carried (a foreign producer's key the compiler would refuse) is preserved rather than dropped, and the deleted reproduction is restored as a positive control with its re-run output at `7ad2aca`. The correction points at the two module rustdocs for the derivation instead of re-deriving it.
4. `docs/decisions/0090:149`, the open question. Retained with its resolution rather than deleted, in ADR 0074's stated idiom for this exact case ("A question a later ticket answers is retained here with its resolution and the owner that supplied it"). It now reads "resolved 2026-08-01 by [the ticket]. It does, for all six governed keys and not for profile keys alone", restates what the question was, and keeps the ground that decided it — authority over a key settles its spelling as well as its bound, whereas a byte bound is a resource ceiling one layer may set for itself, which is why the alphabets reconciled and the bounds did not. The namespace-governance question item 10 leaves open is explicitly untouched.
5. `docs/research/extensions/backend-provider-composition.md:224`, finding 2's correction. The original is pinned to its own commit (**Fact, at `e6a47d9`**) with the tense fixed, and a new **Corrected 2026-08-01** paragraph records that the spike's original sentence — "length and alphabet only" — is now *right*, that `metal_plan.rs` launders nothing any more, and that the collision this correction exposed is what closed it. It restates the two things that survive the fix: the bounds stay apart deliberately, and the two types stay two types, now indexed in the glossary.
6. `docs/research/extensions/backend-provider-composition.md:262` and `:390` (**not cited by this ticket; found by sweep, same scope**). `:262` listed "a profile-key grammar guaranteed at the artifact boundary" among what the record does not supply — now marked supplied on 2026-08-01. `:390`, decision D9, opened "Today neither the grammar beyond a length bound nor the namespace is governed" — now pinned to `e6a47d9`, with the grammar half recorded as decided and the namespace half named as the remaining open one, plus the note that ADR 0090 item 10 accepted D9's recommendation for it.

**Glossary rows.** Two rows, following the file's existing one-row-per-subject idiom for an ambiguous name (`Add`, `Select`, `Minimum`/`Maximum`, `F32Add`/`F32Multiply`, the three `Execution environment` rows), inserted between `Target profile` and `Target property binding`:

- **`TargetProfileKey` (compiler)** — `tiler_compiler::target::TargetProfileKey` at `target.rs:216`; the key of the profile a compilation is *assessed against*; at most `MAX_TARGET_PROFILE_KEY_BYTES` (128), refusing as `TargetProfileKeyError::InvalidByte { index, value }`; 128 named as a *minting* bound and not a statement about other producers; distinct from the artifact type, joined only by byte comparison and never by conversion in the reading direction.
- **`TargetProfileKey` (artifact)** — `tiler_artifact::program::TargetProfileKey` at `keys.rs:256`; the key a packaged program *carries*; one of the six `governed_key!` types; at most `MAX_GOVERNED_KEY_BYTES` (256), refusing as `ArtifactBuildError::NoncanonicalKeyByte`; 256 named as an *admission* bound; the shared alphabet cited to the artifact contract rather than re-enumerated; the direction argument stated explicitly so that narrowing this bound to the compiler's number reads as the defect it would be; and the reminder that a key alone is never compatibility evidence, a profile being referenced as key plus exact descriptor identity.

**Sweep findings beyond the cited sentences.**

- **Corrected here, in held scope.** `docs/artifact-abi.md:295`'s closing clause said a target-profile descriptor identity "stays under the artifact layer's own 1,024-byte digest bound … and for which no authority publishes a bound yet". Both halves are false and the file already contradicted itself about it: `keys.rs` bounds `TargetProfileDescriptorDigest` by its own `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` of 64 KiB, `crates/tiler-compiler/src/target/feasibility.rs:728` refuses past an equal bound where a descriptor is minted, and the **Measurement** paragraph directly below at `:297` already says the three identities were separated. Rewritten to state the 64 KiB ceiling, the equal compiler-side refusal, and that the equality is held by review because neither crate depends on the other. This was invalidated by the descriptor-bound work, not by the key-grammar reconciliation, so it is disclosed rather than folded in silently.
- **Corrected here, in held scope.** `docs/glossary.md:25` and `:92` cited `keys.rs:182` and `:187` for the `BackendKey` and `RepresentationKey` doc strings; the reconciliation's added module documentation moved them to `:246` and `:251`. Verified with `grep -n "A governed backend family key\|A governed executable-representation key" crates/tiler-artifact/src/program/keys.rs`.
- **Checked and left alone.** `docs/artifact-abi.md:239` ("the neutral layer … does own the owner's governed-key grammar") and `docs/glossary.md:25` ("the neutral artifact layer validates the key's grammar") were already true and are more true now. `docs/decisions/README.md` and `docs/README.md` carry no claim about the key grammar and no frontmatter changed, so no catalog block needed editing.
- **Reported, not corrected — two rustdoc claims outside this ticket's scopes.** `crates/tiler-artifact/src/program/keys.rs:82-83` says "`tiler_compiler` publishes `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`", but that constant is `pub(crate)` at `crates/tiler-compiler/src/target/feasibility.rs:1555` inside a `pub(crate) mod feasibility` (`target.rs:129`), so nothing outside the crate can read it — "publishes" overstates a bound that is in fact held by review. Symmetrically, that constant's own doc comment justifies its value as "the largest value `tiler-artifact` will hold: that crate's `MAX_OPAQUE_IDENTITY_BYTES`", but `MAX_OPAQUE_IDENTITY_BYTES` is 1,024 and no longer bounds a descriptor; the number 64 KiB agrees with `tiler-artifact`'s `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` but the comment names the wrong constant for the agreement. Both are `crates/` edits under no scope this ticket holds, and both are the kind of doc-comment claim a later worker reads as fact.

**Commands.** The `grep` reproductions above; `tkt lint`; `git diff --check`; `tkt guard --base b6e14e2`; `make full`.
