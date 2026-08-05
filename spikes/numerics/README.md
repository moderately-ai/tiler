---
schema: "tiler-doc/v1"
id: "tiler.portal.spikes.numerics"
kind: "portal"
title: "Numerical experiments"
topics: ["numerics", "experiments"]
---

# Numerical experiments

These bounded experiments probe reduction semantics, observed region error, a
narrow sound-analysis workflow, which AIR intrinsic each MSL transcendental
spelling selects, and what the pinned language-model reference computes at the
inputs where two plausible formulas stop agreeing. Their individual READMEs
state prerequisites, commands, evidence strength, and limitations:

- [Metal transcendental emission probe](metal_transcendental_emission/README.md)
- [Reduction contract](reduction_contract/README.md)
- [Region accuracy observations](region_accuracy/README.md)
- [Sound accuracy analysis](sound_accuracy/README.md)
- [Transformer reference semantics](transformer_reference_semantics/README.md)
- [BF16 through the second-dtype seams](bf16-second-dtype/README.md)
- [The delivered-realization record, redesigned from typed evidence](delivered-realization-record/README.md)

The last two are Rust workspaces rather than Python programs, so neither is
reached by the acceptance check below, for the same structural reason the
emission probe is not: that checker works from the standard library alone, and
these build against the repository crates. Each runs by hand from its own
directory using the invocation its README records, and each records its own
perturbations there. The BF16 entry corrects an omission rather than adding a
spike — it has been listed in the top-level [experiment catalog](../README.md)
since it landed and was missing from this portal.

The two 2026-07-31 probes stand outside the acceptance check below and say so
here rather than appearing to be covered by it. The emission probe is a shell
and MSL harness, not a Python program at all. The reference-semantics probe is
Python but depends on pinned `torch` and `transformers` wheels, so running it
inside a checker that must work from the standard library alone would make the
checker's own success depend on a several-hundred-megabyte resolution. Each
records in its own README the perturbations that show its rows can change.

Run the complete Python witness acceptance check from the repository root:

```sh
uv run --with mpmath python spikes/numerics/check_witnesses.py
```

The checker rejects executable `assert` syntax in every governed witness, then
runs each program with ordinary and optimized Python, applies a 60-second
per-process deadline, and requires byte-identical output. Removable verdicts
therefore fail structurally rather than relying on output parity to reveal
them.

Use the repository-level [experiment catalog](../README.md) to relate these
spikes to research reports. None is production compiler scaffolding.
