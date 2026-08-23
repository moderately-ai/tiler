---
id: name-the-physical-selection-role-in-the-expansion-subject-facet-doc
title: Name the physical-selection role in the expansion subject facet doc
status: todo
priority: p3
dependencies: []
related: [repair-the-artifact-identity-prose-the-v22-run-falsified]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, artifact, identity, cache]
---
## User-visible outcome

The expansion cache's artifact-program subject facet describes the artifact facts it keys on in the vocabulary the artifact layer actually uses, so a reader deriving the facet's contents from its doc does not miss a subject that already moves artifact identity.

## Why this exists

Found 2026-08-22 by the sibling scan of [`repair-the-artifact-identity-prose-the-v22-run-falsified`](repair-the-artifact-identity-prose-the-v22-run-falsified.md), which repaired the same two defects in `crates/tiler-artifact` but could not reach this file: `crates/tiler-cache/**` maps to `implementation/cache`, which that ticket does not hold.

**Fact — the facet doc is role-unqualified and predates the `v22` run.** `crates/tiler-cache/src/expansion/subject.rs`, anchor `requirements, and selected capability providers.` on `SubjectFacet::ArtifactProgram`. It says "selected capability providers" without saying *lowering*, which the compilation-environment role separation made ambiguous, and it names no physical-selection run. Since `tiler.artifact-program.v22` each variant carries a required non-empty run of selected physical implementations, folded into canonical artifact identity by `crates/tiler-artifact/src/program/model.rs`, anchor `push_selected_physical_implementation_run(bytes, &variant.selected_physical_implementations);`.

**Unverified by that scan, and the real question here.** Whether the facet's *derivation* — not only its prose — already covers the physical-selection run. If the facet is derived from the artifact's canonical identity it covers the run for free and only the prose is stale; if it re-enumerates artifact facts independently, then a cache keyed on it cannot distinguish two plans admitted by different physical authorities, which is the same wrong-identity defect the `v22` step exists to close and is a correctness bug rather than a doc repair. Read the construction site before deciding which this is; do not repair the prose against this ticket's Facts without that read.

## Required work

- Re-audit both Facts at your own base, then read the facet's construction and consumption sites in full.
- Decide which of the two cases above holds, and say so with the evidence.
- If it is prose only, qualify the role and name the run. If the derivation genuinely omits the run, stop, record the finding, and split the correctness repair into its own ticket rather than folding it into a doc change.

## Non-goals

Changing any encoded byte, cache key, or subject derivation under the prose-only reading. Re-deriving the `v22` step, which landed gated.

## Closes when

The facet doc states the provider role and the physical-selection run's status in the facet, the derivation question is answered from source with evidence, and any correctness remainder is filed as its own ticket.

## Worker answer, audit, repair, and sibling scan — 2026-08-22 at `d46a4f4473a5c97625ef1c0a07e30962cd65fec2`

### The open question: the facet **derives**. This is a doc repair, and there is no correctness remainder.

`SubjectFacet::ArtifactProgram` does not re-enumerate artifact facts. It carries one opaque byte run that *is* `tiler-artifact`'s canonical artifact identity, so the physical-selection run entered the cache subject at `v22` with no change in `tiler-cache` and no possibility of two physical selections sharing a key. The derivation was read end to end at this base, in four links, each in the file named:

1. `crates/tiler-cache/src/expansion/subject.rs` declares the facet as bytes, not fields — `pub artifact_program: &'bytes [u8]` on `SubjectFacets` — and the module doc already fixes the rule: a facet's bytes are `counted, length-prefixed, tagged with the role they fill, and never parsed`. `ComposedSubject::compose` reads the run's length and emptiness and nothing else.
2. `crates/tiler-build/src/payload_cache.rs`, the one production supplier, anchor `let expected_artifact = pending.canonical_identity().clone();`, passes it straight through at `artifact_program: expected_artifact.as_bytes(),`. `pending` is a `&VerifiedArtifactProgram`.
3. `crates/tiler-artifact/src/program/model.rs`, anchor `pub const fn canonical_identity(&self) -> &CanonicalArtifactProgramIdentity`, returns the `identity` field of `VerifiedArtifactProgram`, and `CanonicalArtifactProgramIdentity` is a private-field newtype whose only constructor is that crate's encoder.
4. That encoder, anchor `pub(super) fn encode_identity(`, folds every variant through `push_variant(&mut bytes, envelope, &arena, variant, order, &payload_keys)?;`, and `push_variant` calls `push_selected_physical_implementation_run(bytes, &variant.selected_physical_implementations);`.

So two artifacts differing only in their selected physical implementations differ in the identity bytes at step 4, therefore in the facet run at step 2, therefore in the framed composed subject, therefore in the key. **No identity, subject, or cache-key value moves under this ticket** — the delta is doc comments only, and `git diff -U0` filtered to non-`///` lines is empty.

The corollary is what the repair had to record: the enumeration in this doc was never load-bearing. It is a courtesy list in a crate that deliberately owns no artifact encoding, and it will go stale again at `v23` unless the doc says so. It now does.

### Per-Fact verdict

**Fact 1 — verified, and its anchor resolved before the repair retired it.** `grep -n 'requirements, and selected capability providers.' crates/tiler-cache/src/expansion/subject.rs` returned exactly 1 at this base. Both defects were real: the sentence said `selected capability providers` where the identity folds `SelectedLoweringProvider` rows specifically (`encode_identity` pushes `SelectedLoweringProvider::canonical_key` under `ArtifactEntityKind::LoweringProvider`), and it named no physical-selection run.

**Fact 2 — verified.** `grep -n 'push_selected_physical_implementation_run(bytes, &variant.selected_physical_implementations);' crates/tiler-artifact/src/program/model.rs` returned exactly 1. Its three supporting sub-claims were checked at their own sources rather than taken from the ticket: `MANIFEST_SCHEMA` is `(22, 0)` at `crates/tiler-artifact/src/program/codec/encode.rs`, anchor `pub(super) const MANIFEST_SCHEMA: (u16, u16) = (22, 0);`; the domain exists at `crates/tiler-artifact/src/domains.rs`, anchor `b"tiler.artifact-program.physical-selection.v1\0"`; and **required non-empty** is enforced on both sides — the builder refuses at `crates/tiler-artifact/src/program/builder.rs`, anchor `return Err(ArtifactBuildError::EmptySelectedPhysicalImplementations);`, and the decoder refuses a zero count at `crates/tiler-artifact/src/program/codec/decode.rs` inside `fn parse_selected_physical_run(`.

Neither Fact was false or imprecise, so the ticket needed no repair.

### Repair

`crates/tiler-cache/src/expansion/subject.rs`, `SubjectFacet::ArtifactProgram`. The retired clause read **"requirements, and selected capability providers."** — preserved verbatim here so the string stays greppable and this ticket's own Fact anchor still resolves somewhere. It now qualifies the providers as `the selected lowering-capability providers`, names `each variant's run of selected physical implementations — which physical authority implemented each cover-region occurrence`, and adds the paragraph that answers the open question in place: the list illustrates the role and is not an enumeration this crate keeps in step, because the bytes are the artifact layer's canonical identity wrapped rather than restated.

Derived from the **encoder**, not from a second document: the role qualification from `encode_identity`'s `SelectedLoweringProvider` fold and the run from `push_variant`. The sibling lane's repaired prose in `tiler-artifact` was read only to match vocabulary (`selected lowering-capability providers`, `run of selected physical implementations`) after the same facts had been derived independently.

The two crate references are real intra-doc links rather than code font, so they fail loudly if either item is renamed.

**No dated correction note, deliberately, and this is a reasoned deviation from the standing instruction.** The repo reserves the `*(Corrected <date>. This read …)*` form for a claim that was affirmatively *withdrawn* and that a reader might otherwise reinstate — `crates/tiler-artifact/src/program/codec/error.rs` (`This claimed the symbolic case "passes to the`) and `crates/tiler-artifact/src/program/builder.rs` (`both retired clauses were true when written`) are the live examples. The retired clause here made no completeness claim: no `only`, no `in this order`. It was an open illustrative list, so the same class as the sibling lane's own edits, every one of which was made in place with no note (`5a7044bf`). Adding one would put a correction apparatus over a sentence nobody could have reinstated. The retired wording is preserved in this record instead, which is where the grep count is held.

### Sibling scan

Subject: any other enumeration in `tiler-cache` of what a composed subject covers or what artifact identity folds.

**Search vocabulary, and why it is complete.** The crate is 12,600 lines of Rust. Rather than guess phrasings, the scan swept fifteen terms covering every way such an enumeration could be spelled — the *contents* an artifact-identity list would name (`capability provider`, `plan portfolio`, `ABI binding`, `target requirement`, `routing`, `portfolio`, `physical`, `lowering`), the *act* of enumerating (`canonical subject`, `artifact identity`, `artifact-program`, `identity folds`, `folded`), and the *version* an enumeration would have to move with (`v21`, `v22`). Every hit was then opened and read; the count was never treated as the finding. **`physical` and `lowering` return 0 across the whole crate**, which is the decisive negative: no second site in `tiler-cache` mentions either role, so no second site can carry the defect this ticket repairs.

**Findings.** One, the site this ticket names. No others.

**Clean results — read and found current, no change needed.**

- `crates/tiler-cache/src/expansion/key.rs`, the `# What this crate proves about a key, and what it does not` section, was the highest-risk sibling and is **clean by construction**. It makes single-fact claims (`full artifact identity is the key`, and two plan portfolios being `two facet sets, two composed subjects, and two keys`) and then explicitly forwards the enumeration rather than restating it: `[`super::SubjectFacets`] states what the composition does and does not cover`. That forwarding is why repairing the one site repairs the crate.
- `crates/tiler-cache/src/expansion.rs`'s **Complete cache identity** property paragraph names the two facets by role only — `the backend compilations *and* the artifact program wrapped around them` — and likewise forwards to `[`SubjectFacets`]`. No contents list.
- `SubjectFacets::artifact_program`'s own field doc reads `The artifact program's canonical subject.` — one sentence, no enumeration, and correct as written.
- `crates/tiler-cache/src/expansion/harness.rs` and `.../hot_path.rs` construct facets from labelled stand-ins (`b"tiler.cache.harness.artifact-program-stand-in"`, `b"tiler.cache.hot-path.artifact-program-stand-in"`) and say so; they make no claim about identity contents. `.../tests.rs`'s portfolio tests key on `b"portfolio: one plan variant"` against `b"portfolio: two plan variants"`, which tests the *frame* and is indifferent to what a real identity folds.
- The six `folded` hits outside `subject.rs` are all about hexadecimal case folding in `key.rs`, `fuzz.rs`, and `tests.rs` — an unrelated sense of the word, confirmed by reading each.
- **Closed tickets were deliberately not touched.** `tickets/derive-the-pre-compilation-artifact-program-subject.md` restates the pre-`v22` enumeration verbatim, and is `status: done` — a dated record of the state it was written against, by the same convention the sibling lane applied. `accept-the-tiler-cache-public-boundary`, `compose-the-complete-expansion-cache-subject`, and `decide-the-composed-subject-backend-compilations-shape` are also `done` and mention `SubjectFacets` only by name, never its contents. A repo-wide grep for the retired clause returns exactly two hits, both in `tickets/`, neither in a live document.

### Checks

`make full` on the completed delta, plus the targeted set. The delta is doc comments only but sits under `crates/`, which `AGENTS.md` names as a path that forbids carrying a previous green gate, so the full gate was run rather than the doc-only subset.

**The rustdoc check was proved to reach its subject, each link separately.** Renaming the first to `canonical_identity_XX` produced `error: unresolved link to `tiler_artifact::program::VerifiedArtifactProgram::canonical_identity_XX` … the struct `VerifiedArtifactProgram` has no field or associated item named `canonical_identity_XX`` and `error: could not document `tiler-cache``. Renaming the second to `CanonicalArtifactProgramIdentityXX` produced `error: unresolved link to `tiler_artifact::program::CanonicalArtifactProgramIdentityXX` … no item named `CanonicalArtifactProgramIdentityXX` in module `program``. Both perturbations were reverted. The second was run on its own because a passing rustdoc says nothing about a bracketed path that never parsed as a link — only breaking it proves it is checked.
