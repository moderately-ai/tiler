---
schema: "tiler-doc/v1"
id: "tiler.research.cost-model.reading-note.halide-autoscheduler-2019"
kind: "research"
title: "Reading note: Adams et al., Learning to Optimize Halide with Tree Search and Random Programs (2019)"
topics: ["cost-model", "autotuning", "measurement", "prior-art"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "informational"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.cost-model"]
ticket: "close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward"
---

# Reading note: Adams et al., Learning to Optimize Halide with Tree Search and Random Programs (2019)

Distilled from the byte stream pinned as `halide-autoscheduler-2019` in [the source record](../sources/README.md#halide-autoscheduler-2019), read 2026-08-07. That row carries the citation, digest, provenance, and licence verdict. **Note for anyone re-checking the licence:** this copy is an eScholarship deposit whose page 1 is a repository cover sheet with no rights statement; the ACM notice is on page 2, the article's own first page.

**Why this note exists.** Read together with `halide-autoscheduler-2016`, this is the only place in the corpus where **both poles of the measurement question are measured on one system** — an analytic scheduler that does no benchmarking, and a learned one with optional ground-truth autotuning. [The measured-feedback tuning loop](../measured-feedback-tuning-loop.md)'s decision about *where measurement enters* is settled against this pair.

## What the paper measures

The system searches a large space of nested tilings with a backtracking beam search, guided by a **learned cost model** trained by benchmarking randomly generated Halide programs and schedules. Benchmarking is optional at compile time: the model alone can drive the search, and sampling plus benchmarking can be added given extra time.

**Measurement (theirs, against the production Halide autoscheduler of Mullapudi et al. 2016 as baseline).**

- **With no autotuning at all: 75% faster on average.** "our algorithm outperforms the production Halide autoscheduler … by 75% on average with no autotuning".
- **With hours of measurement: up to 135% on average.** "and up to 135% on average with a few hours per application of autotuning."
- Robustness: "never significantly underperforming the prior method, but at times outperforming it by an order of magnitude."
- The cost of measurement is why the model exists: "we wish to evaluate hundreds of thousands of potential schedules for each algorithm, and compiling and benchmarking a single schedule takes several seconds."
- The budget ladder is explicit — "in tens of minutes we can autotune to find better results than the model alone", and "with hours to sample and benchmark, we can also iteratively fine-tune the cost model on the samples measured so far."

**On the earlier Halide autotuner**, which is the other end of the historical arc: it "relied entirely on ground-truth benchmarking" and could take days.

## What this settles for Tiler

**Inference — the ratio, not the headline, is the load-bearing number.** Over a common baseline, going from *no measurement at compile time* to *hours of measurement per application* moved 75% to 135%, a factor of roughly 1.34. The model itself bought the 75%. **So on the one directly comparable pair in this corpus, the analytic model is where the large factor sits and in-loop measurement is the smaller increment on top.**

That is the evidence behind the design record's decision not to spend its budget on an in-loop tuner. Tiler's analytic model currently reports two of nine components as `Unknown` and has never been calibrated against a device on any axis but one, so the unclaimed factor is on the model side — and an in-loop tuner would be optimizing the term this measurement says is worth least, while costing the product constraint the project has already fixed.

**No conclusion in the design record changes. This note proposes no edit to it.**

## Bounds, and one thing this paper does not say

**Bounded measurement.** The numbers are for Halide image-processing and learning pipelines, on the paper's own x86 hardware, against one named baseline, with the cost model trained on a cluster. They do not transfer to Metal, to Tiler's operation families, or to any other baseline, and nothing in the design record treats them as if they did.

**The ratio is an inference from two averages over a shared baseline, not a controlled ablation.** The paper does not report a single experiment isolating "same search, model only" against "same search, model plus autotuning" per application; the 75% and 135% figures are averages reported for two operating modes. **The 1.34 factor is therefore an honest reading of the reported numbers and not a measured speedup**, and it should be quoted with that qualification wherever it is used.

**The paper is also evidence that measurement is not optional for building the model.** The cost model is only cheap at *compile* time — training it required benchmarking thousands of generated programs, and the fine-tuning mode re-enters measurement deliberately. **Inference.** A repository that wants an accurate analytic model without ever touching a device is not the position this paper supports; what it supports is that *per-compilation* measurement is the expensive and least rewarding place to spend, which is a claim about where measurement enters rather than whether it is ever needed. The design record's deferred trigger — a compilation mode that may touch a device — is the right shape for that distinction.
