# Preserved primary sources for the rewrite-search formalism survey

These records keep the optimizer literature behind [the rewrite-search formalism record](../rewrite-search-formalism.md) reproducible when an upstream URL moves, changes, or disappears. Preservation is licence-aware rather than uniform, on the discipline the [numerics source record](../../numerics/sources/README.md) established: a document whose own terms permit dissemination is vendored here byte-for-byte, and a document whose terms do not — or whose terms could not be read — keeps bibliographic identity, a retrieval fingerprint where one exists, and an official acquisition route, with no bytes checked in.

**Retrieval date for every record below: 2026-08-05.** Every retrieval, digest, and licence reading in this file was performed on that date.

**Acquisition note for the six formerly-flagged documents, 2026-08-05.** The six rows this record filed as `pending-acquisition` were pulled by hand by Tom on that date, under [`acquire-the-six-flagged-optimizer-literature-sources`](../../../../tickets/acquire-the-six-flagged-optimizer-literature-sources.md), because every automated route had failed for bot-protection or dead-host reasons rather than paywall ones. **The bytes were relayed without their URLs.** Each of the six records below therefore states the canonical acquisition route and the digest over the exact staged bytes, and does *not* assert a retrieval URL it did not observe — the digest is the identity a re-acquisition is checked against, and the route is where to go, not where these bytes provably came from. Where a staged filename is itself evidence of the route, the record says so and says that it is an inference from the filename.

The population here is mostly metadata-only and that is the expected shape, not a gap. Academic papers are overwhelmingly published under publisher copyright with no redistribution grant; the ACM Digital Library opened its back catalogue to free *reading* in January 2026, which is a reading permission and not a licence to redistribute. Twenty of thirty records are therefore metadata-only. The nine that are vendored are vendored because each one's own text or its arXiv submission metadata carries an explicit dissemination grant — and, usefully, those nine are the equality-saturation lineage plus the one hand-written rewrite-strategy baseline it is measured against, which is the corpus the record's elimination leans on hardest.

**One record is `pending-acquisition`, and that class carries a specific meaning here that it does not carry in the numerics record: it is the source this survey could not reach.** It names the exact reference, what was tried, and what it would decide — it is a work item, not a document someone declined to fetch. No claim anywhere in [the record](../rewrite-search-formalism.md) rests on a `pending-acquisition` row: a source that was not read is never cited as if it were, and where the record needs a fact one would supply, it says so and names the row. `verify-sources.sh` counts the class, so the answer to "what is still unread" is one command rather than a reading exercise.

**The remaining row is a *search* request, and the 2026-08-05 acquisition did not close it.** A candidate was pulled and read, and reading it is what established that it is not the successor the row asks for; it is now its own vendored record (`relational-contextual-eqsat-arxiv-2507.11897v1`) and the search request stays open beside it. That separation is deliberate: an acquired document that answers a *different* question must not be filed under the id of the question nobody has answered, because the row's whole value is that it cannot be mistaken for a real citation.

Digests are deliberately not repeated in this file. `expected-sources.tsv` is the single authority for the expected population, each record's classification, and each SHA-256; `verify-sources.sh` enforces it. A digest written twice drifts silently, and the copy nothing checks is the one a reader would trust.

## What these records claim, and what they do not

- **Preserved source.** The bytes under this directory are an unmodified copy of what the named upstream source served for the named document at the named version. Nothing here is transcribed, reflowed, summarized, or corrected; a paper's claim quoted in Tiler prose is never the authority, the preserved file is.
- **Retrieval fingerprint.** For a metadata-only record, the recorded digest pins the exact byte stream that was read when the survey's claims about that document were checked, and then discarded. It is evidence that a re-acquired copy is or is not the same bytes; it is not permission, and its absence from this directory is deliberate.
- **Tiler inference.** Every elimination, mapping, and tractability conclusion drawn from these documents lives in [the record](../rewrite-search-formalism.md), not here. This directory records provenance only. Preserving a paper does not endorse its conclusions, and a measurement inside a paper stays bounded by that paper's own environment.

## Population boundary

This record closes over every document [the rewrite-search formalism record](../rewrite-search-formalism.md) cites as primary evidence, in five groups that match the survey's own sections. A citation the survey adds later extends this record and restates the boundary here rather than starting a second manifest, because the check's value comes from a single declared population and separate manifests would each be able to agree with themselves.

**Database optimizer lineage** — Selinger et al.'s access-path selection, the Volcano optimizer generator, the Cascades framework, Graefe's query-evaluation survey, Orca, and Xu's Columbia thesis. These carry the memo, group/expression separation, physical-property vector, enforcer, and guidance mechanisms the record maps Tiler's obligations onto; Orca carries the one production instance of a general property-enforcement framework, which is what the record's Cascades elimination has to answer to; and Columbia carries the only measurement in the group of what a Cascades memo costs at scale and of what the extra search efficiency was bought with.

**Equality saturation** — the extended journal version of Tate et al., Stepp's thesis, egg, egglog, Denali, the extraction-complexity results, colored e-graphs and the relational-contextual survey that reads them, and both the sketch-guided and guided equality-saturation papers. These carry the congruence-closure representation, the saturation-or-timeout contract, the extraction problem's complexity and its origin, the earliest measured instance of an e-graph plus a solver-based selection, the mechanism for reasoning under mutually inconsistent assumptions in one e-graph, and the one published measurement of what unguided saturation costs on a *schedule-shaped* rewrite space, which is the measurement the record's elimination turns on.

**Tensor-graph and tensor-compiler search** — MetaFlow, TASO, Tensat, SPORES, the two Halide autoschedulers, Ansor, and TVM. These carry the relaxed-substitution backtracking search, the verified-substitution generation model, e-graph saturation applied to tensor graphs at a real vocabulary size, e-graph search over linear algebra with an ILP extractor, and the algorithm/schedule split that the record argues Tiler already has in a different place.

**Rewrite-strategy baselines** — Elevate, the hand-written strategy language over the same RISE array language and the same seven matrix-multiplication optimization goals that guided equality saturation is measured on. It is the group of one that makes the guided-saturation numbers comparable to something rather than only to unguided saturation.

**Phase ordering** — Click and Cooper, Whitfield and Soffa, and Touati and Barthou. These carry the combining-beats-ordering argument, the enabling/disabling interaction vocabulary, and the undecidability result that bounds what any formalism can promise.

Apple Metal, dtype, and operation-taxonomy evidence stays under [its](../../apple-targets/sources/README.md) [own](../../numerics/sources/README.md) records; nothing here duplicates a row from either.

## Filename map

| File under this directory | Source id |
| --- | --- |
| `eqsat-lmcs-7-1-10-2011/equality-saturation.pdf` | `eqsat-lmcs-7-1-10-2011` |
| `egg-arxiv-2004.03082v3/egg.pdf` | `egg-arxiv-2004.03082v3` |
| `egglog-arxiv-2304.04332v4/better-together.pdf` | `egglog-arxiv-2304.04332v4` |
| `egraph-circuits-arxiv-2408.17042v2/e-graphs-as-circuits.pdf` | `egraph-circuits-arxiv-2408.17042v2` |
| `guided-eqsat-popl-2024/guided-equality-saturation.pdf` | `guided-eqsat-popl-2024` |
| `sketch-guided-eqsat-arxiv-2111.13040v2/sketch-guided-equality-saturation.pdf` | `sketch-guided-eqsat-arxiv-2111.13040v2` |
| `colored-egraphs-arxiv-2305.19203v1/colored-e-graph.pdf` | `colored-egraphs-arxiv-2305.19203v1` |
| `elevate-icfp-2020/achieving-high-performance-the-functional-way.pdf` | `elevate-icfp-2020` |
| `relational-contextual-eqsat-arxiv-2507.11897v1/towards-relational-contextual-equality-saturation.pdf` | `relational-contextual-eqsat-arxiv-2507.11897v1` |

Twenty-one source ids retain no bytes here: twenty metadata-only records and one pending-acquisition record, all below.

## Vendored records

### `eqsat-lmcs-7-1-10-2011`

- **Document:** Ross Tate, Michael Stepp, Zachary Tatlock, Sorin Lerner, "Equality Saturation: A Complete Approach to Optimization", *Logical Methods in Computer Science* 7(1:10), 28 March 2011, 80 pages. This is the extended journal version of the POPL 2009 paper "Equality Saturation: A New Approach to Optimization"; the journal version is preserved rather than the conference one because it is both longer and openly licensed, and the conference version is not.
- **Owner:** the authors, published by Logical Methods in Computer Science.
- **DOI:** `10.2168/LMCS-7(1:10)2011`. Landing page: `https://lmcs.episciences.org/1016`.
- **Retrieved from:** `https://lmcs.episciences.org/1016/pdf` (1 639 330 bytes, `application/pdf`).
- **Licence, read in the document itself:** the first page states "This work is licensed under the Creative Commons Attribution-NoDerivs License. To view a copy of this license, visit http://creativecommons.org/licenses/by-nd/2.0/ or send a letter to Creative Commons, 171 Second St, Suite 300, San Francisco, CA 94105, USA".
- **Verdict:** vendored. CC BY-ND 2.0 permits redistribution of the unmodified work with attribution; the preserved file is the complete unmodified byte stream, and the licence notice travels inside it. The NoDerivs term is why the only correct future edit is replacing the whole document with a newly retrieved edition — an extracted page or a reflowed copy would be a derivative this licence does not cover.
- **Reproducibility, 2026-08-05:** two retrievals of the same URL produced the identical digest.
- **Cited for:** the phase-ordering and global-profitability-heuristic argument the survey opens its equality-saturation section with, the E-PEG representation, and the Pseudo-Boolean-solver extraction that is the ancestor of every ILP extractor cited below.

### `egg-arxiv-2004.03082v3`

- **Document:** Max Willsey, Chandrakana Nandi, Yisu Remy Wang, Oliver Flatt, Zachary Tatlock, Pavel Panchekha, "egg: Fast and Extensible Equality Saturation", arXiv:2004.03082v3. Published as *Proc. ACM Program. Lang.* 5(POPL), Article 23, 2021, DOI `10.1145/3434304`.
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2004.03082v3` (2 090 193 bytes, `application/pdf`).
- **Licence:** CC0 1.0 Universal (public-domain dedication), stated as the submission licence on the arXiv abstract page `https://arxiv.org/abs/2004.03082`, which links `https://creativecommons.org/publicdomain/zero/1.0/`. **Identity note:** unlike the LMCS and POPL records here, this grant is arXiv submission metadata rather than text inside the PDF, so a reader verifying the verdict checks the abstract page, not the preserved bytes.
- **Verdict:** vendored. CC0 waives copyright to the extent possible and imposes no condition on redistribution.
- **Reproducibility, 2026-08-05:** retrieved twice by independent routes — once from the unversioned `https://arxiv.org/pdf/2004.03082` and once from the version-pinned `.../2004.03082v3` — and compared byte-for-byte: identical. The version pin is the identity, because the unversioned path serves whatever the latest version is.
- **Cited for:** rebuilding and the deferred congruence invariant, e-class analyses, and — the load-bearing part — the `is_saturated_or_timeout` contract that makes every practical equality-saturation run a *budgeted* one rather than a saturated one.

### `egglog-arxiv-2304.04332v4`

- **Document:** Yihong Zhang, Yisu Remy Wang, Oliver Flatt, David Cao, Philip Zucker, Eli Rosenthal, Zachary Tatlock, Max Willsey, "Better Together: Unifying Datalog and Equality Saturation", arXiv:2304.04332v4. Published as *Proc. ACM Program. Lang.* 7(PLDI), Article 125, 2023, DOI `10.1145/3591239`.
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2304.04332v4` (1 153 378 bytes, `application/pdf`).
- **Licence:** CC BY 4.0, stated as the submission licence on `https://arxiv.org/abs/2304.04332`, which links `https://creativecommons.org/licenses/by/4.0/`. The same identity note as `egg` applies: the grant is arXiv metadata, not PDF text.
- **Verdict:** vendored. CC BY 4.0 permits redistribution of the unmodified work with attribution.
- **Reproducibility, 2026-08-05:** retrieved twice by independent routes (unversioned and version-pinned) and compared byte-for-byte: identical.
- **Cited for:** the unification of e-matching with Datalog-style bottom-up evaluation, and the semi-naive incremental evaluation that changes the cost model of rule application — the mechanism the record names as what would have to exist before a Tiler saturation engine were worth building.

### `egraph-circuits-arxiv-2408.17042v2`

- **Document:** Glenn Sun, Yihong Zhang, Haobin Ni, "E-Graphs as Circuits, and Optimal Extraction via Treewidth", arXiv:2408.17042v2 (14 November 2024).
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2408.17042v2` (608 474 bytes, `application/pdf`).
- **Licence:** CC BY 4.0, stated as the submission licence on `https://arxiv.org/abs/2408.17042`.
- **Verdict:** vendored.
- **Reproducibility, 2026-08-05:** retrieved twice by independent routes (unversioned and version-pinned) and compared byte-for-byte: identical.
- **Cited for:** the complexity of extraction. Its introduction states that "E-graph extraction is known to be NP-hard" and, citing Goharshady, Lam and Parreaux, that it is "hard to approximate to any constant factor". That pair of facts is what the record uses to reject "extract the optimum from a saturated e-graph" as a plan Tiler could adopt without a stated approximation contract, and it is preserved here rather than cited from a live page because the whole elimination rests on it.
- **Boundary — superseded 2026-08-05, and the supersession is the point.** This bullet read "this record does not preserve the peer-reviewed source of the inapproximability result itself; that is the metadata-only `sparse-extraction-oopsla-2024` row below, which is the one record here with no retrieved bytes at all". `sparse-extraction-oopsla-2024` has now been retrieved and read: it is a metadata-only record with a real digest, its licence forbids vendoring here for a stated reason, and the record's inapproximability claim no longer travels through this document. What survives of the boundary is narrower and worth keeping: this paper is still a *citing* source for that result and not its proof, so a reader checking the proof reads the OOPSLA record's own Section 3, which this record's own text points at correctly.
- **Cross-check that this paper's attribution is accurate, 2026-08-05.** Having read the cited paper, the attribution holds in both directions: Goharshady, Lam and Parreaux do prove constant-factor inapproximability, and they in turn name this paper — "Sun et al. [2024] independently devised an FPT algorithm (with respect to treewidth) for e-graph extraction by reducing the problem to cyclic monotone Boolean circuits". The two results are independent and concurrent rather than one deriving from the other, which is why the corpus keeps both.

### `guided-eqsat-popl-2024`

- **Document:** Thomas Kœhler, Andrés Goens, Siddharth Bhat, Tobias Grosser, Phil Trinder, Michel Steuwer, "Guided Equality Saturation", *Proc. ACM Program. Lang.* 8(POPL), Article 58, January 2024, 32 pages. DOI `10.1145/3632900`.
- **Owner:** the authors.
- **Retrieved from:** `https://steuwer.info/files/publications/2024/POPL-Guided-Equality-Saturation.pdf` (597 136 bytes, `application/pdf`) — an author's institutional copy of the camera-ready.
- **Licence, read in the document itself:** the first page carries "This work is licensed under a Creative Commons Attribution 4.0 International License. © 2024 Copyright held by the owner/author(s). ACM 2475-1421/2024/1-ART58".
- **Verdict:** vendored, on that sentence read in the copy under this directory. PACMPL is gold open access and this article carries the CC BY badge in its own text, so the grant does not rest on the venue's reputation.
- **Acquisition note:** two other routes were attempted on the retrieval date and neither yielded bytes. `https://dl.acm.org/doi/pdf/10.1145/3632900` returns HTTP 403 to a non-browser client (Cloudflare bot protection, not a paywall — the article is open access). `https://inria.hal.science/hal-04372044/document` and `https://inria.hal.science/hal-04372044v1/file/popl24.pdf` both returned an Anubis anti-bot interstitial served as `text/html`, which `file` correctly reports as "HTML document text" rather than a PDF. **This record therefore has one retrieval route, not two, and its digest is not corroborated by an independent route** — unlike every other vendored record here. A reader re-acquiring it should expect to compare the document's own DOI, article number, and page count rather than a second digest.
- **Cited for:** the record's decisive measurement. Its Tables 2 and 3 report that unguided equality saturation over the RISE array language's rewrite rules fails to reach five of seven matrix-multiplication optimization goals within one hour and 60 GB of RAM, reaching the `blocking` goal only after more than an hour and about 35 GB with about 5 M rules applied over an e-graph of about 4 M e-nodes and 2 M e-classes — while the same goals are reached in seconds under 0.5 GB with at most three human-supplied sketch guides, over e-graphs of order 10⁴. Those goals are tiling, vectorization, loop permutation, array packing, cache blocking, and multithreading: a *schedule* space expressed as rewrites, which is the space Tiler would be putting into an e-graph if it chose saturation as its whole-search formalism.
- **Measurement boundary, restated because the record depends on it:** those numbers were measured on a Scala equality-saturation engine "inspired by egg", not on egg itself, with the rule scheduler deliberately disabled, on an Intel Xeon E5-2640 v2 with 60 GB available to the JVM, over the RISE lambda-calculus encoding. They bound *that* configuration. They are not a proof that a Rust egg-based saturation over a different encoding would fail at the same point, and the record must not be read as claiming one.

### `sketch-guided-eqsat-arxiv-2111.13040v2`

- **Document:** Thomas Kœhler, Phil Trinder, Michel Steuwer, "Sketch-Guided Equality Saturation: Scaling Equality Saturation to Complex Optimizations of Functional Programs", arXiv:2111.13040v2 (3 June 2022).
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2111.13040` (2 226 098 bytes, `application/pdf`).
- **Licence:** CC BY 4.0, stated as the submission licence on `https://arxiv.org/abs/2111.13040`.
- **Verdict:** vendored.
- **Identity note:** this is the *predecessor* of `guided-eqsat-popl-2024` and a different paper, not a preprint of it — different author list, different title, and only the later one carries the Lean 4 case study. Conflating them is a live hazard: the POPL 2024 paper has no arXiv version, and this id is what a search for one turns up.
- **Cited for:** one fact that qualifies the guided-equality-saturation measurement in the record's favour rather than against it. This paper reports that before its work "there are only naive encodings of the lambda calculus for equality saturation", and that its efficient encoding "reduces the runtime and memory consumption of equality saturation by orders of magnitude". The POPL 2024 tables the record leans on are labelled "with efficient lambda calculus encoding", so the 60 GB failures are what remains *after* an orders-of-magnitude encoding improvement, not what an unoptimized encoding costs.

### `colored-egraphs-arxiv-2305.19203v1`

- **Document:** Eytan Singher, Shachar Itzhaky, "Colored E-Graph: Equality Reasoning with Conditions", arXiv:2305.19203v1 (30 May 2023). Technion.
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2305.19203` (1 005 565 bytes, `application/pdf`).
- **Licence:** CC BY 4.0, stated as the submission licence on `https://arxiv.org/abs/2305.19203`.
- **Verdict:** vendored.
- **Cited for:** the mechanism that partially refutes one of the record's own elimination reasons, which is why it is preserved rather than cited from a live page. The record argued that an e-graph whose e-class membership depended on a resolved numerical contract would be one structure per contract. This paper's key observation is exactly that "each duplicate e-graph (with an added assumption) corresponds to a coarsened congruence relation", and its contribution is a single structure representing all of them — "a memory-efficient equivalent of multiple copies of an e-graph, with a much lower overhead", by sharing as much as possible while "carefully tracking which conclusion is true under which assumption". The record records the correction rather than keeping the refuted claim.
- **Boundary:** this is an arXiv preprint with no venue recorded on the abstract page as of the retrieval date, and its evaluation is on theory-exploration workloads rather than compiler optimization. It establishes that the mechanism exists and is cheaper than duplication; it establishes nothing about Tiler's contract dimensions, and the record must not read it as more than that.
- **Standing as of 2026-08-05, from a source that surveys it:** `relational-contextual-eqsat-arxiv-2507.11897v1` below is a July 2025 survey of exactly this space by a third group, and it names colored e-graphs "a promising approach" while recording one limitation — that the work "targets the egg library, which implements non-relational e-matching". That is independent corroboration that the mechanism is real and that it had not been superseded as of that date, and it is *not* a peer-review upgrade: the surveying document is itself an ongoing-work paper. The boundary above stands unchanged.

### `elevate-icfp-2020`

- **Document:** Bastian Hagedorn, Johannes Lenfers, Thomas Kœhler, Xueying Qin, Sergei Gorlatch, Michel Steuwer, "Achieving High-Performance the Functional Way: A Functional Pearl on Expressing High-Performance Optimizations as Rewrite Strategies", *Proc. ACM Program. Lang.* 4(ICFP), Article 92, August 2020, 29 pages. DOI `10.1145/3408974`.
- **Owner:** the authors. PACMPL is gold open access.
- **Acquired:** 2026-08-05, by manual browser retrieval; the URL was not relayed with the bytes. The canonical route is the ACM DOI, which returned HTTP 403 to a non-browser client on the survey's own retrieval attempts. The staged filename was `3408974.pdf`, which is the ACM Digital Library's own naming convention for the article behind that DOI — **an inference from the filename, not an observed URL.** The identity that matters is checked from the document instead: its own reference format, article number 92, volume 4, ICFP, and 29 pages all agree with the expected citation, and `pdfinfo` reports 29 pages produced by `acmart 2020/04/30`.
- **Licence, read in the document itself:** page 1 carries "This work is licensed under a Creative Commons Attribution 4.0 International License. © 2020 Copyright held by the owner/author(s). 2475-1421/2020/8-ART92".
- **Verdict:** vendored, on that sentence read in the copy under this directory — the same ground as `guided-eqsat-popl-2024`, and like it the grant is in the PDF text rather than in venue reputation or submission metadata.
- **Cited for:** the hand-written baseline the record had cited a comparison *against* without having read. Two facts it supplies. Its Section 6.2 counts "the number of successfully applied rewrite steps" for the same seven matrix-multiplication versions guided equality saturation is measured on — baseline, blocking, vectorization, loop-perm, array-packing, cache-blocks, parallel — reporting 657 steps for the baseline, "about 40,000 steps for the next three versions and about 63,000 for the most complicated optimizations", and that "applying the strategies to the RISE expression took less than two seconds per version on a commodity notebook with our unoptimized implementation". Its Section 6.3 reports that the generated code "performs competitively with TVM" and that "the most optimized parallel RISE generated version improves the performance over the baseline by about 110×".
- **Counting boundary, stated because the comparison is the whole point:** Elevate counts traversals as rewrite steps — "`id(fun(x,x))` counts as one rewrite step whereas `(body(id))(fun(x,x))` counts as two steps because we also count the traversal into the function body" — so its step counts are not interchangeable with an e-graph's rule-application counts without that adjustment. The wall-clock figure (under two seconds per version) is the comparable quantity and is the one the record uses.

### `relational-contextual-eqsat-arxiv-2507.11897v1`

- **Document:** Tyler Hou, Shadaj Laddad, Joseph M. Hellerstein, "Towards Relational Contextual Equality Saturation", arXiv:2507.11897v1 (16 July 2025), 7 pages. UC Berkeley.
- **Owner:** the authors.
- **Acquired:** 2026-08-05, by manual browser retrieval as a candidate for the `conditional-eqsat-successor-unlocated` search request; the URL was not relayed with the bytes. The staged filename was `2507.11897v1.pdf` and the canonical route is `https://arxiv.org/abs/2507.11897`. `pdfinfo` reports the title, all three authors, `arXiv GenPDF`, and a 16 July 2025 creation date, and the first page carries the arXiv stamp `arXiv:2507.11897v1 [cs.PL] 16 Jul 2025`.
- **Licence, read in the document itself:** page 1 carries "This work is licensed under a Creative Commons Attribution 4.0 International License."
- **Verdict:** vendored. The grant is in the PDF text, which is stronger evidence than the arXiv submission metadata the `egg` and `egglog` records rest on; no abstract-page check is needed to verify this one.
- **Identity finding — this is not the document the search request asks for, and reading it is what established that.** The request is for a *peer-reviewed successor* to colored e-graphs. This is a 7-page ongoing-work paper with no venue anywhere in it: its abstract says "in this paper, we share our ongoing work to extend this to relational equality saturation in egglog", its contribution is to "summarize the existing approaches", "outline its main applications, and identify key challenges", and its conclusion says "we plan to further develop our set-theoretic model" and "we aim to explore whether existing Datalog systems like egglog and Soufflé can be adapted". It carries no evaluation and no implementation. `conditional-eqsat-successor-unlocated` therefore stays `pending-acquisition`.
- **Cited for:** a bounded *negative* result, which is the useful thing a survey of a space supplies when it finds no mature system in it. As of July 2025 a third group surveying contextual equality saturation lists exactly three approaches and finds each of them expensive: `ASSUME` e-nodes pushed down the expression tree (Coward et al. 2023), where "these 'extra' `ASSUME` e-nodes significantly expand the size of the e-graph, harming performance"; top-down context annotation with subgraph copying (Drewery 2022), where "such a rewrite must make independent copies of all contextually-rewritten e-classes and their ancestors" so that "when multiple contexts are nested, this can again lead to a combinatorial explosion of the e-graph size"; and colored e-graphs, named "a promising approach" and limited by targeting egg's non-relational e-matching. That bounds how far the record may read the colored-e-graph refutation: the mechanism is the best available and the space around it was still open work.
- **Relayed measurement, marked as relayed:** this paper states that in Coward et al.'s RTL tool "optimization of a floating point subtactor took 22 minutes, with the majority of time spent in e-graph expansion", and that context-aware optimization there "reduced circuit area by 41% and delay by 33%". **Neither number was read in Coward et al.**, which this corpus does not hold, so both travel with this paper's authority and not the original's. The record uses only the qualitative half.
- **Boundary:** an ongoing-work paper is a weaker authority than anything else vendored here. It is preserved because it is evidence about the *state of a literature* at a date, which is exactly the kind of claim a reader cannot re-derive later, and for no other purpose. Nothing it proposes is a result.

## Metadata-only records

Each record below states the exact bytes that were read and then discarded, so a re-acquired copy is checkable against the reading this survey rests on. None of them may be checked in.

### `graefe-csur-1993`

- **Document:** Goetz Graefe, "Query Evaluation Techniques for Large Databases", *ACM Computing Surveys* 25(2):73–170, June 1993. DOI `10.1145/152610.152611`.
- **Owner:** ACM.
- **Retrieved from:** `https://cs.uwaterloo.ca/~david/cs848s13/graefe.pdf` (850 238 bytes, `application/pdf`), a course mirror; the canonical open-access location is the DOI, which returns HTTP 403 to non-browser clients.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only, with the third-party-mirror identity caveat.
- **Cited for:** the logical/physical algebra distinction that both Volcano and Cascades cite it for — Cascades' own reference list names it at exactly the sentence introducing "logical and physical operators".

### `orca-sigmod-2014`

- **Document:** Mohamed A. Soliman, Lyublena Antova, Venkatesh Raghavan, Amr El-Helw, Zhongxian Gu, Entong Shen, George C. Caragea, Carlos Garcia-Alvarado, Foyzur Rahman, Michalis Petropoulos, Florian Waas, Sivaramakrishnan Narayanan, Konstantinos Krikellas, Rhonda Baldwin, "Orca: A Modular Query Optimizer Architecture for Big Data", *SIGMOD 2014*, pp. 337–348. DOI `10.1145/2588555.2595637`.
- **Owner:** ACM.
- **Retrieved from:** `https://15721.courses.cs.cmu.edu/spring2017/papers/15-optimizer2/p337-soliman.pdf` (1 352 228 bytes, `application/pdf`), a course mirror.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only, with the third-party-mirror identity caveat.
- **Cited for:** the record's Cascades elimination has to answer to a production system, and this is it — "a modern top-down query optimizer based on the Cascades optimization framework". Three of its mechanisms are cited: the Memo of groups holding logically equivalent group expressions; the extensible **property-enforcement framework** driven by "formal property specifications", in which required properties include sort order, distribution, "output columns, rewindability, common table expressions and data partitioning" and each operator controls enforcer placement; and **multi-stage optimization**, where a stage is "a complete optimization workflow using a subset of transformation rules and (optional) time-out and cost threshold", terminating when a plan beats the threshold, the time-out fires, or the rule subset is exhausted. That last mechanism is the closest thing in the DB literature to Tiler's deterministic budgets, and the record notes precisely where it stops short.

### `stepp-thesis-ucsd-2011`

- **Document:** Michael Stepp, "Equality Saturation: Engineering Challenges and Applications", PhD dissertation, University of California, San Diego, 2011 (advisor Sorin Lerner).
- **Owner:** the author / UC San Diego.
- **Retrieved from:** `https://rosstate.org/publications/eqsat/MikeThesis.pdf` (4 392 501 bytes, `application/pdf`). Also available at `https://goto.ucsd.edu/~mstepp/publications/thesis.pdf` and via eScholarship at `https://escholarship.org/uc/item/85f640cc`.
- **Licence:** no licence or redistribution grant stated in the retrieved copy.
- **Verdict:** metadata-only. Absent permission is not permission.
- **Cited for:** the origin of the extraction-hardness result, which closes an attribution chain the record would otherwise have had to leave resting on a blog note. Its chapter 8 carries "8.2 The MIN-SAT Problem", "8.3 NP-Hardness of the PEG Selection Problem", "8.4 Reduction from PEG Selection to Pseudo-Boolean", and "8.6 Reduction of Stateful PEG Selection to ILP" — so NP-hardness of optimal extraction is established here in 2011, independently of and earlier than the note usually cited for it. **What it does not carry is the inapproximability result**; that remains `sparse-extraction-oopsla-2024` below, and the record says so where it uses it.

### `selinger-sigmod-1979`

- **Document:** P. Griffiths Selinger, M. M. Astrahan, D. D. Chamberlin, R. A. Lorie, T. G. Price, "Access Path Selection in a Relational Database Management System", *Proc. ACM SIGMOD 1979*, pp. 23–34. DOI `10.1145/582095.582099`.
- **Owner:** ACM.
- **Retrieved from:** `https://courses.cs.duke.edu/compsci516/cps216/spring03/papers/selinger-etal-1979.pdf` (223 459 bytes, `application/pdf`), a course mirror. The canonical open-access location is `https://dl.acm.org/doi/pdf/10.1145/582095.582099`, which returns HTTP 403 to non-browser clients.
- **Licence:** ACM copyright. No redistribution grant. Free to read since ACM opened its Digital Library in January 2026; reading permission is not redistribution permission.
- **Verdict:** metadata-only. Absent permission is not permission.
- **Identity caveat:** the retrieved copy is a third-party mirror, so its digest pins *the copy that was read* and is not evidence about what ACM serves. A future audit should acquire the document from the DOI above.
- **Cited for:** bottom-up dynamic programming over join orders, and interesting orders — the mechanism by which a plan that is not locally cheapest is retained because a property it guarantees may pay for itself upstream. The survey uses it as the origin of Tiler's boundary-property Pareto frontier, not as a search algorithm to adopt.

### `volcano-icde-1993`

- **Document:** Goetz Graefe, William J. McKenna, "The Volcano Optimizer Generator: Extensibility and Efficient Search", *Proc. 9th IEEE ICDE*, 1993, pp. 209–218. DOI `10.1109/ICDE.1993.344061`.
- **Owner:** IEEE.
- **Retrieved from:** `https://15721.courses.cs.cmu.edu/spring2017/papers/14-optimizer1/graefe-icde1993.pdf` (1 257 723 bytes, `application/pdf`), a course mirror. The canonical location `https://www.computer.org/csdl/proceedings-article/icde/1993/00344061/12OmNzTH0Rw` is paywalled; unlike the ACM records here, IEEE has not opened this back catalogue.
- **Licence:** IEEE copyright, no redistribution grant.
- **Verdict:** metadata-only, with the same third-party-mirror identity caveat as the Selinger record.
- **Cited for:** directed dynamic programming — `FindBestPlan(LogExpr, PhysProp, Limit)` — the physical property vector as an abstract data type with a *cover* comparison, the excluding physical property vector that stops an enforcer's input from re-deriving the property being enforced, and branch-and-bound cost limits passed down into subexpression optimization. The survey maps four of Tiler's existing structures onto these directly.

### `cascades-debull-18-3-1995`

- **Document:** Goetz Graefe, "The Cascades Framework for Query Optimization", *IEEE Data Engineering Bulletin* 18(3):19–29, September 1995. The Data Engineering Bulletin assigns no DOIs.
- **Owner:** IEEE Computer Society Technical Committee on Data Engineering.
- **Retrieved from:** `http://sites.computer.org/debull/95SEP-CD.pdf` (269 523 bytes, `application/pdf`) — the publisher's own free copy of the complete September 1995 issue, in which this article occupies pp. 19–29. **The URL is HTTP-only:** the HTTPS form fails certificate validation on the retrieval date, so a fetcher that forces HTTPS will not reach it. An article-only course mirror at `https://15721.courses.cs.cmu.edu/spring2016/papers/graefe-ieee1995.pdf` (52 701 bytes) was retrieved separately and read; the recorded digest is over the publisher's whole-issue PDF, which is the authoritative route.
- **Licence:** freely published by the TCDE; the issue states no licence or redistribution grant.
- **Verdict:** metadata-only. Free publication is not a redistribution grant, and ambiguity resolves against redistribution.
- **Cited for:** the memo, the group/expression separation, optimization *tasks* as reorderable objects, promise and condition functions, guidance structures, pattern memory, enforcer rules as ordinary rules, and the two sentences the survey's elimination quotes: that without guidance "exhaustive enumeration of all equivalent logical expressions cannot be avoided", and that "if such guidance is incorrect, incorrect pruning of the search space may occur".

### `taso-sosp-2019`

- **Document:** Zhihao Jia, Oded Padon, James Thomas, Todd Warszawski, Matei Zaharia, Alex Aiken, "TASO: Optimizing Deep Learning Computation with Automatic Generation of Graph Substitutions", *SOSP '19*, pp. 47–62. DOI `10.1145/3341301.3359630`.
- **Owner:** ACM.
- **Retrieved from:** `https://theory.stanford.edu/~aiken/publications/papers/sosp19.pdf` (4 647 032 bytes, `application/pdf`), an author's institutional copy.
- **Licence:** ACM copyright; no redistribution grant in the author copy.
- **Verdict:** metadata-only.
- **Cited for:** generated-and-verified graph substitutions — candidate substitutions enumerated from operator building blocks and then discharged against operator properties by a first-order theorem prover — and the maintainability argument its introduction makes with the figure that TensorFlow r1.14 "includes 155 substitutions implemented in approximately 53K lines of C++ code". The survey cites it as the precedent for *where* a rewrite's soundness obligation lives, which is the question ADR 0095 already answers differently for Tiler.

### `tensat-arxiv-2101.01332v2`

- **Document:** Yichen Yang, Phitchaya Mangpo Phothilimthana, Yisu Remy Wang, Max Willsey, Sudip Roy, Jacques Pienaar, "Equality Saturation for Tensor Graph Superoptimization", arXiv:2101.01332v2. Published in *Proc. MLSys* 4, 2021.
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2101.01332v2` (610 997 bytes, `application/pdf`).
- **Licence:** the arXiv abstract page `https://arxiv.org/abs/2101.01332` records the default `arXiv.org perpetual, non-exclusive license 1.0` (`http://arxiv.org/licenses/nonexclusive-distrib/1.0/`), which grants arXiv a distribution licence and grants a third party nothing.
- **Verdict:** metadata-only. The arXiv non-exclusive licence is the single most common reason a paper here is not vendored, and it is worth stating plainly: it is not an open licence.
- **Reproducibility, 2026-08-05:** retrieved twice (unversioned and version-pinned) with identical digests, so the fingerprint is reproducible from the recorded URL.
- **Cited for:** the only published attempt to run equality saturation over a tensor computation graph at production vocabulary size. The survey uses three things from it: that a *valid* rewrite can introduce a cycle into the e-graph so that "the e-graph can (and likely will) contain cycles" while an extracted graph must not, which forces either an ILP acyclicity constraint or explicit cycle filtering during exploration; that the ILP extractor becomes the bottleneck precisely because of that constraint; and its headline that the approach found graphs "up to 16% faster" while "reducing the optimization time by up to 300x" against TASO's backtracking search.

### `spores-pvldb-13-11-2020`

- **Document:** Yisu Remy Wang, Shana Hutchison, Jonathan Leang, Bill Howe, Dan Suciu, "SPORES: Sum-Product Optimization via Relational Equality Saturation for Large Scale Linear Algebra", *PVLDB* 13(11):1919–1932, 2020. DOI `10.14778/3407790.3407799`.
- **Owner:** the authors; publication rights licensed to the VLDB Endowment.
- **Retrieved from:** `https://www.vldb.org/pvldb/vol13/p1919-wang.pdf` (2 383 277 bytes, `application/pdf`).
- **Licence, read on page 1 of the retrieved copy:** "This work is licensed under the Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International License."
- **Verdict: metadata-only, and this one is a judgement rather than an absence.** CC BY-NC-ND does grant redistribution of the unmodified work, but only for non-commercial purposes. This repository belongs to a commercial entity, so the grant's condition is not clearly satisfied, and the fail-closed reading is not to vendor. Recording the reasoning rather than the outcome matters here: a future reader who concludes the non-commercial condition *is* satisfied for this repository can move the row to `vendored` without re-litigating the licence, because the licence is not in doubt — only its applicability is.
- **Cited for:** e-graph search over linear algebra with an explicit ILP extractor. **A correction this record carries because the survey nearly made the error:** SPORES does *not* claim extraction is NP-hard. It formulates extraction as an integer linear program and offers a greedy alternative, and the strings `NP-hard` and `NP-complete` do not occur in it — reproduce with `pdftotext -layout p1919-wang.pdf - | grep -ci 'NP-hard\|NP-complete'`, which answers `0`. The hardness citation belongs to `egraph-circuits-arxiv-2408.17042v2` and `sparse-extraction-oopsla-2024`, not to this paper.
- **Bibliographic discrepancy:** the paper's own front matter says volume 13, number 11; the ACM Digital Library indexes it under number 12. The paper's own numbering is used here.

### `halide-autoscheduler-siggraph-2019`

- **Document:** Andrew Adams, Karima Ma, Luke Anderson, Riyadh Baghdadi, Tzu-Mao Li, Michaël Gharbi, Benoit Steiner, Steven Johnson, Kayvon Fatahalian, Frédo Durand, Jonathan Ragan-Kelley, "Learning to Optimize Halide with Tree Search and Random Programs", *ACM Trans. Graph.* 38(4), Article 121, 2019. DOI `10.1145/3306346.3322967`.
- **Owner:** ACM.
- **Retrieved from:** `https://halide-lang.org/papers/halide_autoscheduler_2019.pdf` (3 708 756 bytes, `application/pdf`), the project's own copy.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only.
- **Cited for:** the algorithm/schedule separation ("it separates the algorithm — what you want to compute — from the schedule — how you want to compute it"), beam search over a schedule parameterization, and a learned cost model trained on hundreds of thousands of random programs. The survey cites it for what it *does not* search: it does not search algebraic rewrites at all.

### `halide-autoscheduler-siggraph-2016`

- **Document:** Ravi Teja Mullapudi, Andrew Adams, Dillon Sharlet, Jonathan Ragan-Kelley, Kayvon Fatahalian, "Automatically Scheduling Halide Image Processing Pipelines", *ACM Trans. Graph.* 35(4), Article 83, 2016. DOI `10.1145/2897824.2925952`.
- **Owner:** ACM.
- **Retrieved from:** `https://graphics.cs.cmu.edu/projects/halidesched/mullapudi16_halidesched.pdf` (2 064 348 bytes, `application/pdf`), an author's institutional copy.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only.
- **Cited for:** the earlier, greedy-grouping form of the same schedule search, preserved so the survey's claim that the Halide lineage searches schedules rather than rewrites is checked against two generations of it rather than one.

### `ansor-arxiv-2006.06762v5`

- **Document:** Lianmin Zheng, Chengfan Jia, Minmin Sun, Zhao Wu, Cody Hao Yu, Ameer Haj-Ali, Yida Wang, Jun Yang, Danyang Zhuo, Koushik Sen, Joseph E. Gonzalez, Ion Stoica, "Ansor: Generating High-Performance Tensor Programs for Deep Learning", arXiv:2006.06762v5. Published at *OSDI 2020*.
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/2006.06762v5` (738 791 bytes, `application/pdf`).
- **Licence:** arXiv non-exclusive licence 1.0.
- **Verdict:** metadata-only.
- **Cited for:** the hierarchical search space that separates sketch generation from annotation sampling, and its explicit complaint that template-guided predecessors have a "restricted search space and ineffective exploration strategy". The survey reads Ansor's two levels as the same factoring Tiler already has between region/cover enumeration and per-region physical frontiers.

### `tvm-arxiv-1802.04799v3`

- **Document:** Tianqi Chen, Thierry Moreau, Ziheng Jiang, Lianmin Zheng, Eddie Yan, Meghan Cowan, Haichen Shen, Leyuan Wang, Yuwei Hu, Luis Ceze, Carlos Guestrin, Arvind Krishnamurthy, "TVM: An Automated End-to-End Optimizing Compiler for Deep Learning", arXiv:1802.04799v3. Published at *OSDI 2018*.
- **Owner:** the authors.
- **Retrieved from:** `https://arxiv.org/pdf/1802.04799v3` (1 224 692 bytes, `application/pdf`).
- **Licence:** arXiv non-exclusive licence 1.0.
- **Verdict:** metadata-only.
- **Cited for:** the two-level graph-then-operator structure that the Halide and Ansor rows are read against, and the ML-based cost model that makes the schedule search a *learned* one — the alternative Tiler's cost-model thread has not adopted.

### `click-cooper-toplas-1995`

- **Document:** Cliff Click, Keith D. Cooper, "Combining Analyses, Combining Optimizations", *ACM TOPLAS* 17(2):181–196, March 1995. DOI `10.1145/201059.201061`.
- **Owner:** ACM.
- **Retrieved from:** `https://web.archive.org/web/20240711032801/https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=9f82847648158bacbbac7eabff2f6ef3f3f5baa1` (201 369 bytes, `application/pdf`) — a Wayback Machine capture of a CiteSeerX copy.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only.
- **Identity caveat — the weakest provenance in this record.** This is an archive of an aggregator's copy, two removes from the publisher. The retrieved copy was checked to be the right document by reading its first page: title, both authors, "Rice University", and the abstract beginning "Modern optimizing compilers use several passes over a program's intermediate representation". Author and course copies commonly cited for this paper are dead or Shibboleth-gated as of the retrieval date, and the Rice repository item of the same title is Click's 1995 PhD thesis, which is a different and much longer document and must not be substituted for the TOPLAS paper.
- **Cited for:** the canonical statement of the phase-ordering problem in compilers and the argument that combining analyses discovers facts that neither phase discovers alone — "Combining these phases can lead to the discovery of more facts about the program, exposing more opportunities for optimization". The survey uses it as the oldest form of the same argument equality saturation makes.

### `whitfield-soffa-toplas-1997`

- **Document:** Deborah L. Whitfield, Mary Lou Soffa, "An Approach for Exploring Code Improving Transformations", *ACM TOPLAS* 19(6):1053–1084, November 1997. DOI `10.1145/267959.267960`.
- **Owner:** ACM.
- **Retrieved from:** `https://www.cs.virginia.edu/~soffa/Soffa_Pubs_all/Journals/Approach.Whitfield.1997.pdf` (292 942 bytes, `application/pdf`), the author's institutional publication page.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only.
- **Cited for:** the *enabling* and *disabling* vocabulary — that transformations "interact with one another by creating or destroying the potential for further" transformation — and the Gospel specification language in which those interactions are derived analytically rather than measured. The survey borrows the vocabulary to state Tiler's phase-ordering risk precisely, because "hides a candidate" is imprecise where "disables" and "enables" are not.

### `touati-barthou-cf-2006`

- **Document:** Sid-Ahmed-Ali Touati, Denis Barthou, "On the Decidability of Phase Ordering Problem in Optimizing Compilation", *Proc. 3rd Conf. on Computing Frontiers (CF '06)*, pp. 147–156. DOI `10.1145/1128022.1128042`.
- **Owner:** ACM.
- **Retrieved from:** `http://www-sop.inria.fr/members/Sid.Touati/publis/CF06.pdf` (147 141 bytes, `application/pdf`), the first author's INRIA publication page. HTTP-only. No HAL deposit or arXiv version was located for this paper; the author's own publication page links only this PDF and its slides.
- **Licence:** ACM copyright; no redistribution grant.
- **Verdict:** metadata-only.
- **Cited for:** the result that bounds what any formalism may claim — that finding a phase sequence yielding an optimal program "is undecidable in two general schemes of optimizing compilation: iterative compilation and library optimization/generation", together with the paper's own simplified decidable instances. The survey cites it to keep Tiler's optimality claim where the optimizer contract already puts it: lowest-cost *valid plan under a stated target profile and numerical contract*, over an enumerated candidate set, not a universal optimum.

### `sparse-extraction-oopsla-2024`

- **Document:** Amir Kafshdar Goharshady, Chun Kit Lam, Lionel Parreaux, "Fast and Optimal Extraction for Sparse Equality Graphs", *Proc. ACM Program. Lang.* 8(OOPSLA2), Article 361, October 2024, 27 pages. DOI `10.1145/3689801`. All three authors at HKUST.
- **Owner:** the authors. PACMPL is gold open access.
- **Acquired:** 2026-08-05, by manual browser retrieval after the survey's three automated routes failed (ACM DOI HTTP 403 to a non-browser client; two author-page guesses HTTP 404); the URL was not relayed with the bytes. The canonical route is the ACM DOI from a browser. The staged filename was `3689801.pdf`, the ACM Digital Library's naming convention for that DOI — **an inference from the filename, not an observed URL.** Identity is instead checked from the document: `pdfinfo` reports the exact title, all three authors, 27 pages, and a producer string containing `oopslab24main-p976-p rev-0a86dc1932-81178 p2551`, whose `p2551` matches the expected first page; the text carries article number 361 and the DOI.
- **Licence, read on page 1 of the retrieved copy:** "This work is licensed under a Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License. © 2024 Copyright held by the owner/author(s)."
- **Verdict: metadata-only, and this one is a judgement rather than an absence — the same judgement `spores-pvldb-13-11-2020` records, reached independently on a different licence.** CC BY-NC-SA 4.0 grants redistribution of the unmodified work, but only for non-commercial purposes. This repository belongs to a commercial entity, so the grant's condition is not clearly satisfied and the fail-closed reading is not to vendor. As with SPORES, the reasoning is recorded rather than only the outcome: a future reader who concludes the non-commercial condition *is* satisfied here can move the row to `vendored` without re-litigating the licence, because the licence is not in doubt — only its applicability is. **The ShareAlike term is an additional reason to be careful even then**, because it would attach a condition to whatever the redistribution is bundled with, which SPORES' NoDerivatives term does not.
- **Cited for — the claim this whole acquisition existed to upgrade.** Its Section 3, "Hardness of Extraction", proves by reduction from Set Cover that "there is no polynomial-time algorithm to approximate the Optimal Extraction problem within any constant approximation factor" unless P=NP, and states that "our proof stands even for a simple cost function that assigns costs of 0 or 1 to all vertices". The record's constant-factor inapproximability claim is therefore **read rather than relayed** as of this date, and it is read at the proof rather than at an abstract.
- **Also cited for — the positive result, and the three reasons it does not reach Tiler.** The paper's algorithm is FPT rather than polynomial: Theorem 4.1 gives `O(n · 3^k · k²)` for a path decomposition of width `k` and Theorem 4.2 gives `O(n · 5^k · k)` for a tree decomposition, linear only when `k` is bounded by a constant. Its measured sparsity is Cranelift's, and three of its own statements bound the transfer. **(a)** "Applications using Egg often requires acyclic extraction which is not covered by this work", and the generalization that would cover it "causes the runtime to increase dramatically, leading to a bound of `O(c^(k²) · k · n)`. This is not very practical even for graphs with small treewidth." **(b)** "we found that benchmarks in Egg, i.e. in the extraction-gym benchmark, often have a large treewidth, so our method is not yet applicable to Egg benchmarks." **(c)** Cranelift's e-graphs are tiny: its Table 1 puts 252 255 of the roughly 281 000 benchmark graphs at 11–50 vertices and only 63 at 1001+.
- **One more measured fact worth keeping, because it is about the value of optimal extraction rather than its cost:** against Cranelift's own sharing-blind heuristic, optimal extraction left most of 108 programs unchanged in code size, averaged "around -2.4%" over the programs that changed at all, and made two programs *larger* — "indicating that the heuristics employed by Cranelift were already close to optimal for e-graphs used in their system".

### `denali-pldi-2002`

- **Document:** Rajeev Joshi, Greg Nelson, Keith Randall, "Denali: A Goal-directed Superoptimizer", *PLDI '02*, June 17–19 2002, Berlin, pp. 304–314. DOI `10.1145/512529.512566`. Compaq Systems Research Center.
- **Owner:** ACM.
- **Acquired:** 2026-08-05, by manual browser retrieval; the URL was not relayed with the bytes. The staged filename was `PLDI2002.pdf`, which indicates **no** route — unlike the two ACM records above it is not a Digital Library name — so this record asserts no retrieval source at all beyond the manual pull. The canonical routes remain the ACM DOI from a browser, or the Wayback capture of Compaq SRC-RR-171 the survey located.
- **Identity, checked from the document:** the first page carries the exact title, all three authors, "Compaq Systems Research Center, 130 Lytton Ave, Palo Alto, CA", and the venue line "PLDI'02, June 17-19, 2002, Berlin, Germany. Copyright 2002 ACM 1-58113-463-0/02/0006".
- **Pagination caveat:** the retrieved copy is paginated 1–11 rather than at the proceedings' 304–314, and its embedded DVI name is `p11-joshi.dvi`. It is the right document at the right venue; it is not a copy carrying the proceedings page numbers, so a citation to a specific page of it must use the article's own numbering or re-derive the offset.
- **Licence:** ACM copyright. The first page carries the era's standard notice — copies permitted "for personal or classroom use", "provided that copies are not made or distributed for profit or commercial advantage", and republication or redistribution "requires prior specific permission and/or a fee". That is explicitly not a grant for this repository.
- **Verdict:** metadata-only, and here the licence text refuses rather than merely omits.
- **Cited for — a correction, not the confirmation the survey expected.** This record and the survey's Part 9 both described Denali as goal-directed *rather than saturating*. The paper does not support that. Its matching phase runs the axioms into the E-graph "until a quiescent state is reached in which the E-graph records all relevant instances of axioms" — that is saturation — and the goal-direction lives in the *selection* phase, where a cycle-bound conjecture ("No program of the target architecture computes the values of the goal terms within K cycles") is refuted by a SAT solver, CHAFF in the prototype. The record now says so.
- **What it does supply, which is more useful than what it was expected to supply.** Three things, all read. **(1)** The earliest statement in this corpus that a practical e-graph run is budgeted rather than saturated, *and* that the budget is what costs the answer its optimality: the paper calls its output "near-optimal" instead of "optimal" and names as the first reason "if the heuristics that are designed to keep the matcher from running forever don't mistakenly stop it from running long enough". **(2)** A measurement that saturation, not extraction, is the dominant cost: the four-byte-swap problem takes "just over a minute" of which "less than 0.3 seconds is spent in the SAT solver", and the checksum problem "took about 4 hours" for a loop body of "10 cycles and 31 instructions". **(3)** The paper's own account of why an e-graph is used at all — "instead of rewriting A as B, it records A = B in its data structure, leaving both A and B around" — with its cost stated in the same breath: "Matching in an E-graph is more expensive than matching a pattern against a simple term DAG. Also, many matches are required to reach quiescence, and the quiescent state may be quite a large E-graph."
- **Boundary:** the paper calls itself "a preliminary report on a new research project", targets "inner loops and critical subroutines" explicitly rather than an ordinary compiler, and its measurements are on a 667 MHz Alpha EV6 with 500 MB of memory. The 2002 wall-clock figures are not comparable to anything modern; what transfers is the *ratio* between the two phases and the paper's own reason for its optimality disclaimer.

### `metaflow-mlsys-2019`

- **Document:** Zhihao Jia, James Thomas, Todd Warszawski, Mingyu Gao, Matei Zaharia, Alex Aiken, "Optimizing DNN Computation with Relaxed Graph Substitutions". The paper's own front matter reads "Proceedings of the 2nd SysML Conference, Palo Alto, CA, USA, 2019".
- **Owner:** the authors — the front matter's only rights statement is "Copyright 2019 by the author(s)".
- **Acquired:** 2026-08-05, by manual browser retrieval; the URL was not relayed with the bytes. The staged filename was `MLSys-2019-optimizing-dnn-computation-with-relaxed-graph-substitutions-Paper.pdf`, which is the `proceedings.mlsys.org` download naming convention — **an inference from the filename, not an observed URL**, and the canonical route remains `https://proceedings.mlsys.org/` for the 2019 proceedings.
- **Bibliographic discrepancy, recorded rather than resolved:** this record's id and the survey both cite it as *Proc. MLSys* 1, 2019, while the paper's own front matter says "2nd SysML Conference". Both are correct usages of a venue that was renamed — the 2019 conference was the second SysML and is indexed as MLSys proceedings volume 1 — and the id is kept as-is because it is what a reader searching the current index will match. The same discipline the SPORES record applies to its volume/number disagreement applies here.
- **Licence:** a bare copyright assertion with no redistribution grant.
- **Verdict:** metadata-only. Absent permission is not permission.
- **Cited for — a measured instance of the record's own phase-ordering finding, with the mechanism quoted exactly.** Its Algorithm 1 enqueues a rewritten graph under `if Cost(G') < α × Cost(G_opt)`, and the paper states that "by setting α = 1, the search algorithm becomes a simple greedy algorithm and only considers graph substitutions that strictly reduce cost. As α increases, the search algorithm explores a larger part of the search space." Its motivating example is a substitution that "downgrades runtime performance (since a convolution with a larger kernel runs slower) but enables additional subsequent kernel fusions" — an enabling relation across a cost regression, measured on real networks rather than modelled.
- **The qualification that runs the other way, recorded because it is the honest half.** MetaFlow does not retain every legal alternative; it retains within a bounded cost relaxation, defaulting to α = 1.05, and that sufficed: its Table 3 reports that the α = 1.05 backtracking search "finds the same optimal graph" as an exhaustive search on AlexNet, VGG16, an Inception module, and ResNet18, while reducing search time by orders of magnitude — ResNet18 from 3.1 hours to 0.99 seconds. Its Figure 10 shows the discovered-graph quality on Inception-v3 improving sharply from α = 1.0 and flattening by about 1.05.
- **The anti-pattern this paper documents, which is the most Tiler-relevant thing in it.** Its cost model handles a hard resource limit by folding it into the cost: it can "minimiz[e] execution time while maintaining a memory usage limit (by returning an infinite cost if the memory usage limit is exceeded)". That is exactly the substitution [the architectural contract](../../../../AGENTS.md) forbids — reject an infeasible plan with an explainable reason, never hide it behind an infinite or arbitrary cost — and it is worth having a published instance of, because it shows the collapse is the natural thing to do when the search has only one retention authority.
- **Boundary:** MetaFlow's substitutions are semantics-preserving graph substitutions with no numerical-contract dimension at all, the same gap as its sibling TASO. Its α is a tuned hyperparameter carrying no bound, and its equivalence to exhaustive search is a measured result on four networks under one cost model, not a guarantee.

### `columbia-optimizer-thesis-1998`

- **Document:** Yongwen Xu, "Efficiency in the Columbia Database Query Optimizer", MSc thesis, Portland State University, presented 12 February 1998, 114 pages. Advisor Leonard Shapiro.
- **Owner:** the author / Portland State University.
- **Acquired:** 2026-08-05, by manual browser retrieval after the survey's route failed to connect at all (the PSU host did not respond); the URL was not relayed with the bytes. The staged filename was `xu-columbia-thesis1998.pdf`, which indicates no particular host.
- **Identity, checked from the document:** the title page reads "EFFICIENCY IN THE COLUMBIA DATABASE QUERY OPTIMIZER … By YONGWEN XU … MASTER OF SCIENCE in COMPUTER SCIENCE … Portland State University … 1998", and the thesis-approval page names the committee and the 12 February 1998 date.
- **Licence:** none stated anywhere in the retrieved copy.
- **Verdict:** metadata-only. Absent permission is not permission — the same reading as `stepp-thesis-ucsd-2011`.
- **Cited for — the calibration the record asked for, and one finding beyond it.** The calibration: with only the *safe* pruning technique enabled ("we only used lower bound group pruning in Columbia to demonstrate the best performance of Columbia while still generates optimal solutions"), a 16-table chain query optimizes in under a minute using under 150 MB, while star queries scale far worse in time, multi-expression count, and memory, and the thesis' own conclusion records that "memory usage can become prohibitive when optimizing very large queries, such as the star queries with greater than 16 tables". That is what a Cascades memo costs at scale, measured, on 4× 200 MHz Pentium Pro with 1 GB.
- **The finding beyond it, which bears on budgets rather than on efficiency.** Columbia's second technique, *global epsilon pruning*, buys its additional search reduction by accepting a bounded loss of optimality, and the thesis measures the trade on an 8-table chain query: epsilon 0 gives the optimal plan at 3 174 multi-expressions; epsilon 15 gives 1 293 multi-expressions at an error of 0.4; epsilon 30 gives 608 at an error of 1.0, where the thesis defines `Error = (cost of optimizer output – cost of optimal plan) / cost of optimal plan` and notes that "an error of 1 indicates the cost of the optimizer output is twice as much as the optimal plan". So the best-known independent Cascades reimplementation found that the memo's cost at scale is paid down with a knob that trades the answer's optimality for search time — the same axis Orca's cost-threshold stage termination sits on, here with an error curve attached.
- **Measurement-validity caveat, and it is a real one.** The thesis' Columbia-versus-Cascades comparison (Figures 31–32) runs the two optimizers *on different machines* — Columbia on the SMP Pentium Pro box, and per its own footnote 18 the Cascades optimizer "on an Ultra-2 Sun workstation with 2 UltraSPARC-II 296MHz CPUs and 252MB Memory … SunOS 5.5.1", timed with the Unix `time` command. The cross-system trend it reports (Columbia pulling further ahead on star queries as tables increase) is suggestive; the absolute ratios are not a controlled measurement and must not be quoted as one.
- **One further observation, relevant to Tiler's oracle discipline rather than to search:** the thesis' future work names as an open problem that "it is difficult to prove if the optimizer produces the optimal plan or a sub-optimal plan just because of a programming bug in the optimizer, especially when optimizing large queries". A memo at scale is hard to validate, which is an argument for the exhaustive oracles Tiler's cover search already carries rather than against a memo.

## Pending-acquisition records — named requests for a manual pull

**No byte stream was ever retrieved for any record in this section, and nothing in [the record](../rewrite-search-formalism.md) is cited from one.** The one remaining record states the exact reference, what was tried, and — the part that makes it a work item rather than a bibliography entry — what it would decide for the survey, so that finding it is worth someone's time or demonstrably is not. Its digest field is `-`; there is nothing to fingerprint and nothing is invented. A source that could not be read is never summarized from its abstract, its title, or a secondary description of it: that would be a fabricated citation wearing a real reference's clothes.

**Five of the original six left this section on 2026-08-05**, when Tom pulled them by hand. Their provenance now lives above — `elevate-icfp-2020` under the vendored records, and `sparse-extraction-oopsla-2024`, `denali-pldi-2002`, `metaflow-mlsys-2019`, and `columbia-optimizer-thesis-1998` under the metadata-only records — and what each one turned out to decide is recorded there and, where it moved a conclusion, in [the record](../rewrite-search-formalism.md) itself. Two of the five did not say what this section predicted they would say: the sparse-extraction result does not weaken the extraction elimination (its own limitations section rules out the acyclic case Tiler needs), and Denali is not the goal-directed-rather-than-saturating precedent this section described (it saturates to quiescence and puts the goal-direction in a SAT-based selection). **Both predictions were written from citations rather than from the documents, which is precisely the gap this section exists to mark**, and both are corrected at the point of use rather than quietly rewritten.

### `conditional-eqsat-successor-unlocated` — the successor literature to colored e-graphs

- **Document: not located.** This is a *search* request rather than a fetch request, and the id is deliberately marked `unlocated` so it cannot be mistaken for a real citation. `colored-egraphs-arxiv-2305.19203v1` (2023) is the paper this survey found for reasoning under mutually inconsistent assumptions inside one e-graph, and it is an arXiv preprint whose evaluation is on theory exploration rather than compiler optimization. There is very likely a peer-reviewed successor or sibling line — conditional, contextual, or assumption-indexed equality saturation — that the survey did not find.
- **Attempted 2026-08-05, first round:** one guessed arXiv identifier (`2404.05623`, hypothesized to be "Equality Saturation Modulo Theories") was fetched and turned out to be an entirely unrelated active-learning paper. **It is recorded here rather than dropped because it is the exact failure mode the flagging discipline exists to catch:** a fetched PDF whose identity was not checked would have entered the survey as a citation to a paper that does not say what was attributed to it.
- **Attempted 2026-08-05, second round — a real candidate, read, and rejected on identity.** arXiv:2507.11897v1, Hou, Laddad and Hellerstein, "Towards Relational Contextual Equality Saturation", was pulled and read in full. It is genuinely about this space and it is now vendored above as `relational-contextual-eqsat-arxiv-2507.11897v1`, but it is a 7-page ongoing-work paper with no venue, no implementation, and no evaluation — its own words are "we share our ongoing work" and "we aim to explore whether existing Datalog systems like egglog and Soufflé can be adapted". **It is not a peer-reviewed successor, so this request stays open.** The candidate's value was diagnostic rather than dispositive, and it is worth stating what it diagnosed: as of July 2025 a third group surveying exactly this space named three approaches, found all three expensive, and called colored e-graphs the promising one, which is *evidence that no mature successor existed to be found* rather than evidence that the search was bad.
- **What it would decide.** The record's elimination reason "an e-graph whose e-class membership depended on a resolved numerical contract would be one structure per contract" is already partially refuted by the colored-e-graph mechanism, and the record now says so. A mature, evaluated version of that mechanism would refute it further and would be the strongest available argument for revisiting the deferred component-level adoption at stage 1. This remains the single highest-value item for *changing* a conclusion rather than confirming one.
- **What would close this row without finding the paper.** A successor to the 2025 candidate above — the same authors' completed work, or any peer-reviewed contextual/relational equality-saturation system with an implementation and an evaluation — closes it by being found. Failing that, a documented search of the EGRAPHS workshop proceedings and of the citation graph forward from `colored-egraphs-arxiv-2305.19203v1`, recorded here with its date and its negative result, closes it as *exhausted* rather than untried. The row must not be closed by a reader concluding from its age that nothing exists.

## Verifying this record

```sh
docs/research/region-search/sources/verify-sources.sh
```

The check reads `expected-sources.tsv` and enforces a declared population — 30 records, of which 9 vendored, 20 metadata-only, and 1 pending-acquisition — before it inspects anything, so a manifest that lost rows fails rather than agreeing with itself. It then verifies that ids are unique, that every vendored file exists and matches its recorded digest, that no metadata-only or pending record retains local bytes, and that every file present on disk is claimed by exactly one record.

The pending-acquisition count is the one number here a reader is expected to want to *change*, and on 2026-08-05 it moved from six to one. When a flagged source is pulled, its row moves class, its digest is filled in, the counts at the top of `verify-sources.sh` move with it, and its section turns into a record of what the document actually said. Note what the count does *not* say: a row that leaves this class has been read, not agreed with, and two of the five that left on that date contradicted the prediction their pending entry carried.

**The check was watched failing on four perturbations on 2026-08-05, over the new population**, in a scratch copy of this directory rather than in place, and each perturbation was chosen to exercise a row this change added rather than one that was already covered: an unrecorded stray file inside a newly vendored directory (`elevate-icfp-2020/stray-page-extract.txt is present on disk but absent from the manifest`); a deleted newly vendored file (`relational-contextual-eqsat-arxiv-2507.11897v1: vendored file … is missing`); a mutated digest on a newly vendored record (`digest mismatch for elevate-icfp-2020/achieving-high-performance-the-functional-way.pdf`); and a newly reclassified row put back to `pending-acquisition`, which failed on both affected counts at once (`metadata-only records: 19, expected 20` and `pending-acquisition records: 2, expected 1`). Each returned exit status 1; the unperturbed run before and after returned exit 0 and the `OK: 30 records verified (9 vendored, 20 metadata-only, 1 pending-acquisition).` line. A check nobody has watched say no is not a check.

**The earlier five-perturbation run is retained as history because it exercised cases this one did not:** a mutated vendored digest on `egg-arxiv-2004.03082v3`, a dropped manifest row (`manifest holds 18 records, expected 19`), an unrecorded stray file, a deleted vendored file, and an emptied manifest (three count failures plus one unclaimed-file failure per vendored document). The emptied-manifest and dropped-row cases have not been re-run against the 30-record population; the count logic they exercise is unchanged, and saying so is more useful than implying they were.

Adding or refreshing a source means updating the manifest row, the counts declared at the top of `verify-sources.sh`, and the record above in the same change.

`.gitattributes` here marks the preserved files `-text -whitespace -diff`. The first two protect the bytes: end-of-line conversion on a checkout would silently break every recorded digest, and `git diff --check` would otherwise report trailing whitespace that belongs to the upstream document and must not be removed. **`-diff` was added on 2026-08-05 and protects the review rather than the bytes:** git's content heuristic renders some PDFs as text, and one document vendored that day arrived as 3 052 added lines in the diff. The stored bytes were never at risk — each new blob was compared against its manifest digest with `git cat-file blob <rev>:<path> | shasum -a 256` and matched — but a diff nobody can read is a diff nobody reviews. The record's own files stay under the normal checks.

An upstream revision that changes a claim is not handled by refreshing bytes. Refreshing a preserved document updates evidence only; changing what Tiler concludes requires an explicit contract or ADR review.
