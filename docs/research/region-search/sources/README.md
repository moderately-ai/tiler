# Preserved primary sources for the rewrite-search formalism survey

These records keep the optimizer literature behind [the rewrite-search formalism record](../rewrite-search-formalism.md) reproducible when an upstream URL moves, changes, or disappears. Preservation is licence-aware rather than uniform, on the discipline the [numerics source record](../../numerics/sources/README.md) established: a document whose own terms permit dissemination is vendored here byte-for-byte, and a document whose terms do not — or whose terms could not be read — keeps bibliographic identity, a retrieval fingerprint where one exists, and an official acquisition route, with no bytes checked in.

**Retrieval date for every record below: 2026-08-05.** Every retrieval, digest, and licence reading in this file was performed on that date.

The population here is mostly metadata-only and that is the expected shape, not a gap. Academic papers are overwhelmingly published under publisher copyright with no redistribution grant; the ACM Digital Library opened its back catalogue to free *reading* in January 2026, which is a reading permission and not a licence to redistribute. Sixteen of twenty-nine records are therefore metadata-only. The seven that are vendored are vendored because each one's own text or its arXiv submission metadata carries an explicit dissemination grant — and, usefully, those seven are the equality-saturation lineage, which is the corpus the record's elimination leans on hardest.

**Six records are `pending-acquisition`, and that class carries a specific meaning here that it does not carry in the numerics record: these are the sources this survey could not reach.** Each one names the exact reference, what was tried, and what it would decide — they are work items for a manual pull, not documents someone declined to fetch. No claim anywhere in [the record](../rewrite-search-formalism.md) rests on a `pending-acquisition` row: a source that was not read is never cited as if it were, and where the record needs a fact one of them would supply, it says so and names the row. `verify-sources.sh` counts the class, so the answer to "what is still unread" is one command rather than a reading exercise.

Digests are deliberately not repeated in this file. `expected-sources.tsv` is the single authority for the expected population, each record's classification, and each SHA-256; `verify-sources.sh` enforces it. A digest written twice drifts silently, and the copy nothing checks is the one a reader would trust.

## What these records claim, and what they do not

- **Preserved source.** The bytes under this directory are an unmodified copy of what the named upstream source served for the named document at the named version. Nothing here is transcribed, reflowed, summarized, or corrected; a paper's claim quoted in Tiler prose is never the authority, the preserved file is.
- **Retrieval fingerprint.** For a metadata-only record, the recorded digest pins the exact byte stream that was read when the survey's claims about that document were checked, and then discarded. It is evidence that a re-acquired copy is or is not the same bytes; it is not permission, and its absence from this directory is deliberate.
- **Tiler inference.** Every elimination, mapping, and tractability conclusion drawn from these documents lives in [the record](../rewrite-search-formalism.md), not here. This directory records provenance only. Preserving a paper does not endorse its conclusions, and a measurement inside a paper stays bounded by that paper's own environment.

## Population boundary

This record closes over every document [the rewrite-search formalism record](../rewrite-search-formalism.md) cites as primary evidence, in four groups that match the survey's own sections. A citation the survey adds later extends this record and restates the boundary here rather than starting a second manifest, because the check's value comes from a single declared population and separate manifests would each be able to agree with themselves.

**Database optimizer lineage** — Selinger et al.'s access-path selection, the Volcano optimizer generator, the Cascades framework, Graefe's query-evaluation survey, and Orca. These carry the memo, group/expression separation, physical-property vector, enforcer, and guidance mechanisms the record maps Tiler's obligations onto, and Orca carries the one production instance of a general property-enforcement framework, which is what the record's Cascades elimination has to answer to.

**Equality saturation** — the extended journal version of Tate et al., Stepp's thesis, egg, egglog, the extraction-complexity result, colored e-graphs, and both the sketch-guided and guided equality-saturation papers. These carry the congruence-closure representation, the saturation-or-timeout contract, the extraction problem's complexity and its origin, the mechanism for reasoning under mutually inconsistent assumptions in one e-graph, and the one published measurement of what unguided saturation costs on a *schedule-shaped* rewrite space, which is the measurement the record's elimination turns on.

**Tensor-graph and tensor-compiler search** — TASO, Tensat, SPORES, the two Halide autoschedulers, Ansor, and TVM. These carry the verified-substitution generation model, e-graph saturation applied to tensor graphs at a real vocabulary size, e-graph search over linear algebra with an ILP extractor, and the algorithm/schedule split that the record argues Tiler already has in a different place.

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

Twenty-two source ids retain no bytes here: sixteen metadata-only records and six pending-acquisition records, all below.

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
- **Boundary:** this record does not preserve the peer-reviewed source of the inapproximability result itself; that is the metadata-only `sparse-extraction-oopsla-2024` row below, which is the one record here with no retrieved bytes at all.

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

## Pending-acquisition records — named requests for a manual pull

**No byte stream was ever retrieved for any record in this section, and nothing in [the record](../rewrite-search-formalism.md) is cited from one.** Each states the exact reference, what was tried, and — the part that makes it a work item rather than a bibliography entry — what it would decide for the survey, so that acquiring it is worth someone's time or demonstrably is not. The digest field for every row here is `-`; there is nothing to fingerprint and nothing is invented. A source that could not be read is never summarized from its abstract, its title, or a secondary description of it: that would be a fabricated citation wearing a real reference's clothes.

Ordered by what the survey would gain, strongest first.

### `sparse-extraction-oopsla-2024` — the inapproximability proof itself

- **Document:** Amir Kafshdar Goharshady, Chun Kit Lam, Lionel Parreaux, "Fast and Optimal Extraction for Sparse Equality Graphs", *Proc. ACM Program. Lang.* 8(OOPSLA2):2551–2577, 2024. DOI `10.1145/3689801`. PACMPL is gold open access.
- **Attempted 2026-08-05, three routes, all failed:** `https://dl.acm.org/doi/pdf/10.1145/3689801` returned HTTP 403 to a non-browser client (Cloudflare bot protection, not a paywall — the article is open access); `https://cse.hkust.edu.hk/~parreaux/publication/oopsla24a/oopsla24a.pdf` returned HTTP 404; `https://www.cse.ust.hk/~parreaux/publication/oopsla24a/paper.pdf` returned HTTP 404. The project page `https://cse.hkust.edu.hk/~parreaux/publication/oopsla24a/` is the place to find the correct filename.
- **Acquisition route:** the ACM DOI from a browser.
- **What it would decide.** The record's elimination of equality saturation as the whole search rests in part on extraction being *constant-factor inapproximable*, not merely NP-hard. NP-hardness is now established from a source that was read (`stepp-thesis-ucsd-2011`, chapter 8). The inapproximability claim is currently carried **only** by `egraph-circuits-arxiv-2408.17042v2` citing this paper — that is, the record states a result it has read a citation of but not a proof of, and says so at the point of use. Acquiring this converts one link in the elimination from a relayed claim into a read one. It also carries a positive result the record would want: a *fast optimal* extraction algorithm for sparse e-graphs, which could weaken elimination reason 2 if Tiler's e-graphs turn out to be sparse.

### `columbia-optimizer-thesis-1998` — the independent Cascades reimplementation

- **Document:** Yongwen Xu, "Efficiency in the Columbia Database Query Optimizer", MSc thesis, Portland State University, 1998 — the Columbia optimizer, the best-known independent reimplementation of Cascades, with measurements of the search-efficiency techniques Cascades only proposed.
- **Attempted 2026-08-05:** `https://web.cecs.pdx.edu/~len/Columbia/Columbia.pdf` failed to connect (curl exit before any HTTP status; the host did not respond).
- **Acquisition route:** the PSU CS department or the Columbia optimizer's GitHub mirrors; a PDXScholar deposit may exist.
- **What it would decide.** Cascades' own paper says the system "has not gone through a thorough evaluation and tuning phase" and reports no performance study at all. Columbia is where the group-pruning, lower-bound, and `promise`-function techniques were actually measured. The record's elimination of a Cascades memo is *not* on efficiency grounds, so this would not change the verdict; what it would supply is a calibrated sense of what a memo costs at scale, which the record currently does not have and does not claim to.

### `conditional-eqsat-successor-unlocated` — the successor literature to colored e-graphs

- **Document: not located.** This is a *search* request rather than a fetch request, and the id is deliberately marked `unlocated` so it cannot be mistaken for a real citation. `colored-egraphs-arxiv-2305.19203v1` (2023) is the paper this survey found for reasoning under mutually inconsistent assumptions inside one e-graph, and it is an arXiv preprint whose evaluation is on theory exploration rather than compiler optimization. There is very likely a peer-reviewed successor or sibling line — conditional, contextual, or assumption-indexed equality saturation — that the survey did not find.
- **Attempted 2026-08-05:** one guessed arXiv identifier (`2404.05623`, hypothesized to be "Equality Saturation Modulo Theories") was fetched and turned out to be an entirely unrelated active-learning paper. **It is recorded here rather than dropped because it is the exact failure mode the flagging discipline exists to catch:** a fetched PDF whose identity was not checked would have entered the survey as a citation to a paper that does not say what was attributed to it.
- **What it would decide.** The record's elimination reason "an e-graph whose e-class membership depended on a resolved numerical contract would be one structure per contract" is already partially refuted by the colored-e-graph mechanism, and the record now says so. A mature, evaluated version of that mechanism would refute it further and would be the strongest available argument for revisiting the deferred component-level adoption at stage 1. This is the single highest-value item in this section for *changing* a conclusion rather than confirming one.

### `denali-pldi-2002` — goal-directed superoptimization over an e-graph

- **Document:** Rajeev Joshi, Greg Nelson, Keith Randall, "Denali: A Goal-Directed Superoptimizer", *PLDI 2002*, pp. 304–314. DOI `10.1145/512529.512566`. A longer preliminary version exists as Compaq SRC Research Report 171 (2001).
- **Attempted 2026-08-05:** not fetched. The ACM DOI is expected to return HTTP 403 as every other `dl.acm.org` route in this record did; the HP Labs host serving SRC-RR-171 is dead; the commonly cited University of Washington course copy is now Shibboleth-gated. Wayback captures of both were located but not retrieved.
- **Acquisition route:** the ACM DOI from a browser, or `https://web.archive.org/web/20190219174909/http://www.hpl.hp.com/techreports/Compaq-DEC/SRC-RR-171.pdf`.
- **What it would decide.** Denali is where an e-graph was first driven by a *goal* rather than saturated — the search is directed at proving a target expression rather than deriving everything. That is architecturally the same move as sketch guidance, twenty years earlier, and it is cited as such by both egg and the guided-equality-saturation line. It would strengthen the record's deferral (goal-directed use of an e-graph is the shape Tiler would want) without changing any elimination.

### `metaflow-mlsys-2019` — the backtracking search TASO and Tensat both compare against

- **Document:** Zhihao Jia, James Thomas, Todd Warszawski, Mingyu Gao, Matei Zaharia, Alex Aiken, "Optimizing DNN Computation with Relaxed Graph Substitutions", *Proc. MLSys* 1, 2019.
- **Attempted 2026-08-05:** not fetched; identified from TASO's and Tensat's citations while reading them.
- **Acquisition route:** `https://proceedings.mlsys.org/` for the 2019 proceedings.
- **What it would decide.** It is the *relaxed* substitution search — deliberately admitting individually-worsening substitutions and backtracking — which is the closest published system to the record's own finding that a search must not prune on cost before a composition completes. It would supply an independent, measured instance of that finding rather than the modelled one the record's spike provides. Worth acquiring for exactly that reason.

### `elevate-icfp-2020` — the manual-strategy baseline

- **Document:** Bastian Hagedorn, Johannes Lenfers, Thomas Kœhler, Xueying Qin, Sergei Gorlatch, Michel Steuwer, "Achieving High-Performance the Functional Way: A Functional Pearl on Expressing High-Performance Optimizations as Rewrite Strategies", *Proc. ACM Program. Lang.* 4(ICFP), 2020.
- **Attempted 2026-08-05:** not fetched; identified from `guided-eqsat-popl-2024`'s citations while reading it.
- **Acquisition route:** the ICFP 2020 PACMPL issue; gold open access.
- **What it would decide.** The guided-equality-saturation paper benchmarks its rule-application counts against Elevate's hand-written strategies and observes they are "in the same order of magnitude". The record cites the e-graph sizes but not that comparison, because the baseline was not read. Acquiring it would let the record state how far from a hand-written optimal rewrite sequence a guided search lands — a number that bears on how much search is worth buying at all, which is a question the cost-model thread will eventually ask.

## Verifying this record

```sh
docs/research/region-search/sources/verify-sources.sh
```

The check reads `expected-sources.tsv` and enforces a declared population — 29 records, of which 7 vendored, 16 metadata-only, and 6 pending-acquisition — before it inspects anything, so a manifest that lost rows fails rather than agreeing with itself. It then verifies that ids are unique, that every vendored file exists and matches its recorded digest, that no metadata-only or pending record retains local bytes, and that every file present on disk is claimed by exactly one record.

The pending-acquisition count is the one number here a reader is expected to want to *change*. When a flagged source is pulled, its row moves class, its digest is filled in, the counts at the top of `verify-sources.sh` move with it, and the section above turns into a record of what the document actually said.

**The check was watched failing on five perturbations before this record was committed**, in a scratch copy of this directory rather than in place: a mutated vendored digest (`digest mismatch for egg-arxiv-2004.03082v3/egg.pdf`), a dropped manifest row (`manifest holds 18 records, expected 19`, plus the metadata-only count), an unrecorded stray file (`stray.txt is present on disk but absent from the manifest`), a deleted vendored file (`vendored file … is missing`), and an emptied manifest (three count failures plus one unclaimed-file failure per vendored document). Each returned exit status 1; the unperturbed run before and after returned exit 0 and the `OK: 19 records verified` line. A check nobody has watched say no is not a check.

Adding or refreshing a source means updating the manifest row, the counts declared at the top of `verify-sources.sh`, and the record above in the same change.

`.gitattributes` here marks the preserved files `-text -whitespace`. Both settings protect the bytes: end-of-line conversion on a checkout would silently break every recorded digest, and `git diff --check` would otherwise report trailing whitespace that belongs to the upstream document and must not be removed. The record's own files stay under the normal checks.

An upstream revision that changes a claim is not handled by refreshing bytes. Refreshing a preserved document updates evidence only; changing what Tiler concludes requires an explicit contract or ADR review.
