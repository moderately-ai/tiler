---
id: close-the-conditional-eqsat-search-request-with-the-fmcad-2024-successor
title: Close the conditional-eqsat search request with the FMCAD 2024 successor
status: review
priority: p2
dependencies: []
related: []
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [sources, region-search, eqsat]
claimed_from: todo
assignee: agent-eqsat-close
lease_expires_at: 1786040225
---

## What Tom pulled (2026-08-06)

`/Users/tsanterre/Downloads/Proceedings of the 24th Conference on Formal Methods in Computer- Aided Design - FMCAD 2024_II.pdf` — 15,603,336 bytes, sha256 `1b50d43cbc32f5f22532e2bbcbe776fda4c054238dea9edc5744c8f07be112e4`, pdfinfo title "Proceedings of the 24th Conference on Formal Methods in Computer-Aided Design – FMCAD 2024", eds. Narodytska/Rümmer. Its table of contents lists **"Easter Egg: Equality Reasoning Based on E-Graphs with Multiple Assumptions", Eytan Singher and Shachar Itzhaky, page 70** — the colored-e-graphs authors at a peer-reviewed venue, and the abstract states experiments (colored e-graphs supporting large assumption/term counts at ~10x lower space with slightly improved performance). The volume's front matter licenses the work CC BY 4.0, and per-article CC BY 4.0 notices appear throughout.

## Why this closes the search request

`conditional-eqsat-successor-unlocated` (docs/research/region-search/sources/README.md) asks for a peer-reviewed successor to `colored-egraphs-arxiv-2305.19203v1` with an evaluation. This is exactly that, by identity finding rather than assumption: same authors, same construct, named venue, experiments present. The prior candidate (arXiv:2507.11897) was rejected on identity; this one passes the same test.

## The work

1. Acquire the paper by the OFFICIAL route where possible: FMCAD proceedings are published open access (TU Wien Academic Press); prefer the individually published paper PDF from the canonical FMCAD/publisher route, recording URL and digest — Tom's volume copy is the fallback source and its digest is above either way. CC BY 4.0 permits vendoring; follow the region-search manifest's row format and licence discipline exactly.
2. Vendor the paper (or the volume's relevant span if no individual PDF exists — state which and why), update the region-search `expected-sources.tsv` and its verifier populations, and run the verifier watching it fail on a perturbation first.
3. Resolve `conditional-eqsat-successor-unlocated`: the search request's row moves from unlocated to the located identity, preserving the request text and the 2507.11897 rejection as the record's history, in the tense-preserving idiom.
4. READ the paper against the formalism record's colored-e-graphs section: does the FMCAD version change any claim the record took from the arXiv preprint (the record cites v1)? Record held/moved per claim, in the array-API re-check shape. The identity finding is the floor; the re-check is the value.

## Closes when

The paper is preserved with digests under its stated licence, the search request resolves to the located identity with its history intact, the verifier passes with the stepped population after being watched failing, and the formalism record's claims are re-checked against the peer-reviewed version with the verdict recorded.

## Outcome (2026-08-06)

**The official individual-paper route exists and was used; Tom's volume was not the source.** The paper's own first-page footer carries its DOI, `10.34727/2024/isbn.978-3-85448-065-5_13`, which resolves (HTTP 302) to the TU Wien institutional repository at `https://repositum.tuwien.at/handle/20.500.12708/200780`. Its `citation_pdf_url` served the article as a standalone 14-page PDF (1 118 753 bytes, sha256 `7e0cd58e39ae4a2cab2cf1f5aef1a0287cebe05dc64deb4989ed19d74b8ef714`), retrieved twice under different user agents with identical bytes. Vendored as `docs/research/region-search/sources/colored-egraphs-fmcad-2024/easter-egg.pdf` under CC BY 4.0 read on page 1 of the retrieved copy. **No page extraction from the volume was needed**, so no derivative was created.

**The volume corroborated identity rather than supplying bytes.** Tom's copy (sha256 `1b50d43c…`) carries the article at PDF pages 85–98 for printed pages 70–83, a constant offset of 15 verified at both ends. The two copies are not byte-identical (the volume was reprocessed by `OneVision PDFengine`, re-encoding its fonts) and were compared as whitespace-normalized token streams: 10 703 against 10 702 tokens, similarity 0.985, and every one of the 152 differences an extraction artifact of that re-encoding. Normalizing the `fi`/`ffi` ligatures and the `φ`/`ϕ` glyph leaves 19, all glyph substitutions or reading-order permutations inside figure and algorithm blocks; as token multisets the two differ by exactly one prime mark that one extraction attached to its identifier. **No content difference — the stop condition on a materially differing official copy did not fire**, and no licence complication arose.

**The search request closed, and not in the shape it expected.** `conditional-eqsat-successor-unlocated` asked for a peer-reviewed *successor* to colored e-graphs. What was found is the peer-reviewed *publication of the same work* — same authors, same construct, same evaluation design, retitled. It satisfies the row's own second closing disjunct exactly (a peer-reviewed contextual equality-saturation system with an implementation and an evaluation), so the row closes by being found; but a reader told only "closed" would wrongly infer a new line of work appeared, and the record says so plainly. The manifest row is gone, the request text and both attempt rounds are preserved verbatim in the sources README, and the residual — no compiler-side evaluation, no genuinely later system — is recorded as an explicitly deferred question with a reconsideration trigger rather than left as a closed row's ghost. It is deliberately *not* re-filed as `pending-acquisition`: no document is named to fetch, and that class's promise is that its members have never been cited.

**Re-check verdict: five claims, four hold, one narrows; no derivation moved.** The coarsening observation holds; the two verbatim quotations survive peer review word for word; the theory-exploration-only measurement boundary holds; the refusal half of elimination B reason 3 is untouched. The one that narrows is "a solved problem in principle" — the published version adds an applicability bound the preprint lacks, naming cloning as the better answer when an assumption is not a modest coarsening. Two evidence-level changes cut opposite ways: the preprint's evaluation carried a caveat that its layered union-find was *simulated* by full per-colour copies, which the published version drops in favour of a real one with the evaluation re-run (strengthening); and the published abstract **withdrew** the preprint's quantified "hundreds of assumptions and millions of terms" scale claim (weakening). This record never cited the withdrawn figure. **Elimination B is unchanged and still rests on four independent reasons.**

**Two defects found and fixed in passing, both in the formalism record.** A **misquotation**: the record attributed "each duplicate e-graph (with an added assumption) corresponds to a coarsened congruence relation" to the preprint, and that string is in neither edition — the preprint reads "corresponds to coarsened congruence relation" inside a sentence that is ungrammatical in the original, so the record had silently repaired a direct quotation. Replaced with the published edition's body sentence and the correction recorded. A **date arithmetic error**: "eighteen months after its preprint" for a 30 May 2023 to 16 July 2025 interval, corrected to twenty-six. Neither changed a conclusion.

**Both editions are retained as separate rows, deliberately.** The preprint is not superseded bytes: it is the only evidence that the earlier memory result was measured on an implementation that still duplicated the union-find per colour, and it carries a scale claim the published version dropped. Refreshing one into the other would delete that.

### Commands run

- `docs/research/region-search/sources/verify-sources.sh` → `OK: 30 records verified (10 vendored, 20 metadata-only, 0 pending-acquisition).`, exit 0.
- **Watched failing on five perturbations first**, in a scratch copy outside the repository, each exercising something this change added: deleted new vendored file; mutated digest on the new row; stray `page-70-extract.txt` in the new directory; the closed request re-filed as pending (tripping `manifest holds 31 records, expected 30` *and* `pending-acquisition records: 1, expected 0`, the case the new `expect_pending=0` exists to catch); and **the same article substituted from the volume's own page span**, correctly rejected on digest — a copy of the right paper is not the officially routed artifact. Each exit 1; unperturbed runs before and after exit 0.
- Local link and anchor check over both edited documents: 37 local links, 0 broken, with a negative control confirming the anchor extractor rejects a fabricated fragment.
- `tkt lint`, `git diff --check`, `tkt guard --base 67139ee6`. No cargo — this ticket touches no gate input.

### Scope

`research/region-search` (`docs/research/region-search/**`) plus the ticket file under the declared shared `project/tickets`. No scope was added; none was required. `spikes/region-search/**` was in scope and untouched — the re-check is a document comparison with no reusable harness, and its reproduction commands are stated inline in the records instead.
