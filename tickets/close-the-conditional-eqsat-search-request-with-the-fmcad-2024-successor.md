---
id: close-the-conditional-eqsat-search-request-with-the-fmcad-2024-successor
title: Close the conditional-eqsat search request with the FMCAD 2024 successor
status: in-progress
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
