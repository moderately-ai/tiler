# Preserved primary sources for the measured-feedback tuning-loop design

These records keep the autotuning and adaptive-execution literature behind [the measured-feedback tuning loop](../measured-feedback-tuning-loop.md) reproducible when an upstream URL moves, changes, or disappears. The discipline is the one the [rewrite-search source record](../../region-search/sources/README.md) and the [numerics source record](../../numerics/sources/README.md) established, with one deliberate difference stated up front.

**Every record here is metadata-only with a retrieval fingerprint, and none is vendored.** That is a *deliberate uniform classification and not a licence reading*: no document below had its licence terms read, so no verdict about redistribution has been reached for any of them, and vendoring one without that reading would be exactly the unchecked assertion this discipline exists to prevent. Several are arXiv submissions whose grant is plausibly permissive, and that plausibility is not evidence. [`vendor-the-tuning-loop-primary-sources-after-reading-each-licence`](../../../../tickets/vendor-the-tuning-loop-primary-sources-after-reading-each-licence.md) owns the reading and any subsequent vendoring; until it lands, every row's digest is the identity a re-acquisition is checked against and nothing here claims permission.

**Retrieval date for every reachable record below: 2026-08-07**, by `curl` from the recorded URL on the coordination host, followed by `pdftotext` (Poppler) into the text actually read. The digest in each row is a SHA-256 over the exact retrieved PDF byte stream. Those bytes were read and then discarded; they are not under this directory.

## What these records claim, and what they do not

- **Retrieval fingerprint.** The recorded digest pins the exact byte stream that was read when the design record's claims about that document were checked. It is evidence that a re-acquired copy is or is not the same bytes. It is not permission, and its absence from this directory is deliberate.
- **Quotation.** Every sentence [the design record](../measured-feedback-tuning-loop.md) quotes from one of these documents was read in the extracted text of the exact byte stream digested here, not in an abstract, a summary, or a secondary description. Where the design record could not verify a claim it says so and names the row.
- **Tiler inference.** Every decision, elimination, and mapping drawn from these documents lives in the design record, not here. Preserving a reference does not endorse its conclusions, and a measurement inside a paper stays bounded by that paper's own environment.

## Population boundary

This record closes over every document the design record cites as primary evidence, in four groups matching its own sections. A citation the design record adds later extends this record and restates the boundary here rather than starting a second manifest.

**Tensor-program autotuning** — AutoTVM, Ansor, TVM, and the two Halide autoschedulers. These carry the measured-search loop the design record's *where measurement enters* section is decided against: the measurement budget an operator-level tuner actually spends, the surrogate-model-plus-hardware-batch structure, the transfer mechanism and its stated invariance requirement, and — from the two Halide records read together — the one directly comparable pair of a no-benchmarking analytic scheduler and a learned one with optional ground-truth autotuning.

**General program autotuning** — OpenTuner. It is the group of one that carries a tuning *store* as an explicit architectural component rather than an implementation detail, which is the subject of the design record's identity-discipline section.

**Adaptive execution and parametric plans** — the adaptive-query-processing survey, Kabra and DeWitt's mid-query re-optimization, Ioannidis et al.'s parametric query optimization, and Reddy and Haritsa's plan diagrams. These carry the twenty-year prior art on the shape-transfer question: what a plan's validity region is, when an observed parameter invalidates a cached plan, why keeping one plan per parameter point is refused, and the measured geometry of how coarse the winner partition actually is.

**Benchmarking statistics** — Hoefler and Belli. It is the source of the twelve rules the design record's statistical section is written against, including the two it records itself as failing.

## Reachable records

| Source id | Digest (SHA-256) | Bytes |
| --- | --- | --- |
| `autotvm-nips-2018` | `fe270467011c5b68c379f91aa615e59eea7a19a2d54139624afacff169721f1a` | 4 354 983 |
| `ansor-osdi-2020` | `44a4d77988c6bd03276720d982cc86d620f525cfd3b8cc6a40e79fee9f4093a3` | 738 791 |
| `tvm-osdi-2018` | `6032d5a54db0ec552168a5f5295a954edea6a24d91ae03bed1e0da0d858c9fa9` | 1 224 692 |
| `opentuner-pact-2014` | `0582e325cf2989123957e21ef9cd4638b03fc5ffaba5edc7fa0181e705311b23` | 476 488 |
| `halide-autoscheduler-2016` | `6b0b0d143d2764073ffada9902d499ad939bab9a425de2d6c8aea21f7452f7f0` | 2 064 348 |
| `halide-autoscheduler-2019` | `e4dd35a0c36ad631c8dfc6882c3ea72ae128d657128806ffa84ae3513edb79c0` | 3 845 934 |
| `aqp-survey-fntdb-2007` | `9307f20bd31e92583f63279b32ae560f899fc080554551134785d9b0785ed48e` | 2 486 286 |
| `kabra-dewitt-sigmod-1998` | `1f27c41f30a47da27ea9edfa4ea86460cd411a995fce8d4926c1fa639a8d1e2c` | 1 826 069 |
| `pqo-vldb-1992` | `c8713911209ed030b2d4355e5a967c24dff777753175fea14132d8056b1003da` | 1 293 795 |
| `plan-diagrams-vldb-2005` | `401c6f66e28231e7056bc6ac57f41d7ae130ecdbe3adbc5e42db95e2cd5be7b8` | 499 633 |
| `hoefler-belli-sc-2015` | `1b5e6210d83c87f7eae495dd518e78b2be4a17bb2a71f820a59ce6c962a3051e` | 4 444 145 |

### `autotvm-nips-2018`

- **Document:** Tianqi Chen, Lianmin Zheng, Eddie Yan, Ziheng Jiang, Thierry Moreau, Luis Ceze, Carlos Guestrin, Arvind Krishnamurthy, "Learning to Optimize Tensor Programs", 32nd Conference on Neural Information Processing Systems (NeurIPS 2018), Montréal, Canada. arXiv:1805.08166v2.
- **Retrieved from:** `https://arxiv.org/pdf/1805.08166v2`, 2026-08-07.
- **Cited for:** the rank-loss objective and its justification; the batched propose-then-measure-then-update loop; the transfer-learning mechanism and its stated prerequisite that a transferable representation be invariant across source and target domains; the global-plus-local model decomposition; the measured 2x-to-10x search speedup from transfer; and the paper's *silence* on measurement noise, which the design record names as an evidence gap rather than filling.

### `ansor-osdi-2020`

- **Document:** Lianmin Zheng, Chengfan Jia, Minmin Sun, Zhao Wu, Cody Hao Yu, Ameer Haj-Ali, Yida Wang, Jun Yang, Danyang Zhuo, Koushik Sen, Joseph E. Gonzalez, Ion Stoica, "Ansor: Generating High-Performance Tensor Programs for Deep Learning", 14th USENIX Symposium on Operating Systems Design and Implementation (OSDI 2020). arXiv:2006.06762v5.
- **Retrieved from:** `https://arxiv.org/pdf/2006.06762v5`, 2026-08-07.
- **Cited for:** the measurement budget an operator tuner spends (1,000 measurement trials per test case; 1,000 × *n* per network) and the wall-clock it implies; the statement that querying the learned cost model is orders of magnitude faster than actual measurement; the iterative measure-then-retrain fine-tuning loop; the task scheduler's role in allocating a measurement budget across subgraphs; and the explicit cache-flushing remark, which is the one sentence in this corpus that treats measurement protocol as an engineering variable that changes how many repetitions are needed.

### `tvm-osdi-2018`

- **Document:** Tianqi Chen, Thierry Moreau, Ziheng Jiang, Lianmin Zheng, Eddie Yan, Meghan Cowan, Haichen Shen, Leyuan Wang, Yuwei Hu, Luis Ceze, Carlos Guestrin, Arvind Krishnamurthy, "TVM: An Automated End-to-End Optimizing Compiler for Deep Learning", 13th USENIX Symposium on Operating Systems Design and Implementation (OSDI 2018). arXiv:1802.04799v3.
- **Retrieved from:** `https://arxiv.org/pdf/1802.04799v3`, 2026-08-07.
- **Cited for:** context on the algorithm/schedule separation the autotuner sits inside. The design record leans on it least of the four tensor-compiler records and cites no sentence of it as load-bearing.

### `opentuner-pact-2014`

- **Document:** Jason Ansel, Shoaib Kamil, Kalyan Veeramachaneni, Jonathan Ragan-Kelley, Jeffrey Bosboom, Una-May O'Reilly, Saman Amarasinghe, "OpenTuner: An Extensible Framework for Program Autotuning", 23rd International Conference on Parallel Architectures and Compilation Techniques (PACT 2014).
- **Retrieved from:** `http://groups.csail.mit.edu/commit/papers/2014/ansel-pact14-opentuner.pdf`, 2026-08-07.
- **Cited for:** the results database as the *only* channel between the search module and the measurement module, and the ensemble-of-techniques design that this separation is what makes possible. The design record uses the first as its architectural precedent for a tuning store that is a first-class component rather than a memo, and the second as evidence that the store's value is not caching but sharing.

### `halide-autoscheduler-2016`

- **Document:** Ravi Teja Mullapudi, Andrew Adams, Dillon Sharlet, Jonathan Ragan-Kelley, Kayvon Fatahalian, "Automatically Scheduling Halide Image Processing Pipelines", ACM Transactions on Graphics (SIGGRAPH 2016).
- **Retrieved from:** `https://graphics.cs.cmu.edu/projects/halidesched/mullapudi16_halidesched.pdf`, 2026-08-07.
- **Cited for:** the claim that the algorithm "does not require costly (and often impractical) auto-tuning" while remaining competitive with expert-authored schedules. This is the analytic-only pole of the comparison the design record's *where measurement enters* section is decided on.

### `halide-autoscheduler-2019`

- **Document:** Andrew Adams, Karima Ma, Luke Anderson, Riyadh Baghdadi, Tzu-Mao Li, Michaël Gharbi, Benoit Steiner, Steven Johnson, Kayvon Fatahalian, Frédo Durand, Jonathan Ragan-Kelley, "Learning to Optimize Halide with Tree Search and Random Programs", ACM Transactions on Graphics 38(4), Article 121, July 2019.
- **Retrieved from:** `https://escholarship.org/content/qt5h71f534/qt5h71f534.pdf?t=px2g3z`, 2026-08-07.
- **Cited for:** the measured split between what a learned cost model buys with no benchmarking and what ground-truth autotuning adds on top of it, and the description of the earlier Halide autotuner that "relied entirely on ground-truth benchmarking" and could take days. It is the other pole of the same comparison, and the two Halide records together are the only place in this corpus where both poles are measured on one system.

### `aqp-survey-fntdb-2007`

- **Document:** Amol Deshpande, Zachary Ives, Vijayshankar Raman, "Adaptive Query Processing", *Foundations and Trends in Databases* 1(1), 2007, pp. 1–140.
- **Retrieved from:** `https://www.cs.umd.edu/~amol/papers/fnt-aqp.pdf`, 2026-08-07.
- **Cited for:** the four-part measure/analyze/plan/actuate adaptivity loop; the statement that the bane of parametric query optimization is deciding what plans to keep and that the space of optimal plans is superexponential in the parameter count; the validity-range mechanism from progressive optimization; and the two progressive-parametric implementations, Bounded and Ellipse. This is the record the design record's shape-transfer section leans on hardest.

### `kabra-dewitt-sigmod-1998`

- **Document:** Navin Kabra, David J. DeWitt, "Efficient Mid-Query Re-Optimization of Sub-Optimal Query Execution Plans", ACM SIGMOD 1998.
- **Retrieved from:** `https://www.cs.cmu.edu/~natassa/courses/15-721/papers/reopt.pdf`, 2026-08-07.
- **Cited for:** the annotated query execution plan — a plan that carries the optimizer's own estimates so a runtime observation can be compared against them — and the resulting separation between deciding *what to collect* and deciding *whether to re-optimize*. The design record uses the annotation idea and rejects the re-optimization half for a stated reason.

### `pqo-vldb-1992`

- **Document:** Yannis E. Ioannidis, Raymond T. Ng, Kyuseok Shim, Timos K. Sellis, "Parametric Query Optimization", 18th International Conference on Very Large Data Bases (VLDB 1992), pp. 103–114.
- **Retrieved from:** `https://www.vldb.org/conf/1992/P103.PDF`, 2026-08-07.
- **Cited for:** the definition of the problem — identifying several plans, each optimal for a *subset* of run-time parameter values — which is the exact shape of Tiler's shape-transfer question stated thirty-four years earlier.

### `plan-diagrams-vldb-2005`

- **Document:** Naveen Reddy, Jayant R. Haritsa, "Analyzing Plan Diagrams of Database Query Optimizers", 31st International Conference on Very Large Data Bases (VLDB 2005), pp. 1228–1239.
- **Retrieved from:** `https://www.vldb.org/conf/2005/papers/p1228-reddy.pdf`, 2026-08-07.
- **Cited for:** the measured geometry of plan optimality regions over the selectivity space — 68 plans reducible to 7 without increasing any query point's estimated cost by more than 10 percent — and the resulting hypothesis that optimizers make fine-grained plan choices not merited by the coarseness of the cost space. This is the empirical argument the design record's refusal of a per-shape winner table rests on.

### `hoefler-belli-sc-2015`

- **Document:** Torsten Hoefler, Roberto Belli, "Scientific Benchmarking of Parallel Computing Systems: Twelve Ways to Tell the Masses when Reporting Performance Results", SC '15: Proceedings of the International Conference for High Performance Computing, Networking, Storage and Analysis, 2015.
- **Retrieved from:** `https://htor.inf.ethz.ch/publications/img/hoefler-scientific-benchmarking.pdf`, 2026-08-07.
- **Cited for:** rules 1 through 12, of which the design record quotes 3, 5, 6, 7, 8, 9, and 10 verbatim and records the repository as currently failing 6 and partially failing 8.

## Unreachable

Three documents the design record wanted and did not read. Each row names the exact reference, what was attempted, and the decision it would have informed. **No claim in the design record rests on any of them**, and each is named at the point where the record says what it could not establish. [`acquire-the-three-unreachable-adaptive-execution-sources`](../../../../tickets/acquire-the-three-unreachable-adaptive-execution-sources.md) owns closing this class; the class is declared with three members rather than being left implicit, so an emptied channel that stops being counted is not one a re-opened request slips back into unnoticed.

### `markl-pop-sigmod-2004` — unreachable

- **Document:** Volker Markl, Vijayshankar Raman, David Simmen, Guy Lohman, Hamid Pirahesh, Miso Cilimdzic, "Robust Query Processing through Progressive Optimization", ACM SIGMOD 2004, pp. 659–670. DOI `10.1145/1007568.1007642`.
- **Attempted, 2026-08-07:** `https://cs.uwaterloo.ca/~david/cs848/paper-progressive-optimization.pdf` (HTTP 404). No further route was tried; the ACM Digital Library landing page is the canonical acquisition route.
- **What it would have decided:** the design record's shape-transfer section adopts a *validity range* attached to a stored result, and it takes that mechanism from the [adaptive-query-processing survey's](#aqp-survey-fntdb-2007) description of this paper rather than from the paper. The survey states that validity ranges are associated with plans and that re-optimization is invoked when observed parameters fall outside them; what the survey does not state, and this paper would, is **how a validity range is computed and how conservative it is**. The design record therefore specifies that a Tiler validity region must be *declared* rather than inferred, and explicitly does not claim that this matches the POP construction.

### `cole-graefe-sigmod-1994` — unreachable

- **Document:** Richard L. Cole, Goetz Graefe, "Optimization of Dynamic Query Evaluation Plans", ACM SIGMOD 1994, pp. 150–160. DOI `10.1145/191839.191872`.
- **Attempted, 2026-08-07:** `https://www.vldb.org/conf/1994/P150.PDF` (HTTP 404 — the page number matched a VLDB volume, not the SIGMOD proceedings this paper is in; the attempt was misdirected rather than the document being absent). No second route was tried.
- **What it would have decided:** whether the *choose-plan* operator — a plan node that defers a choice to run time and evaluates a predicate over run-time bindings to pick a branch — is a better shape for Tiler's deferred-measurement case than the availability-phase resolution the repository already has. The design record decides that question against the repository's own [`AvailabilityPhase`](../../../decisions/0043-use-typed-phased-target-feasibility.md) ladder without this evidence, and names it as the one place a reader should expect the argument to be thinner than the rest.

### `graefe-ward-sigmod-1989` — unreachable

- **Document:** Goetz Graefe, Karen Ward, "Dynamic Query Evaluation Plans", ACM SIGMOD 1989, pp. 358–366. DOI `10.1145/67544.66960`.
- **Attempted, 2026-08-07:** not attempted. It is recorded here because it is the origin of the dynamic-plan line the row above continues, and a reader tracing that line should not have to rediscover that this record skipped it.
- **What it would have decided:** the same question as the row above, at its origin.
