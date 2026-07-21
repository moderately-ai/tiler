---
schema: "tiler-doc/v1"
id: "tiler.portal.spikes.numerics"
kind: "portal"
title: "Numerical experiments"
topics: ["numerics", "experiments"]
---

# Numerical experiments

These bounded experiments probe reduction semantics, observed region error, and
a narrow sound-analysis workflow. Their individual READMEs state prerequisites,
commands, evidence strength, and limitations:

- [Reduction contract](reduction_contract/README.md)
- [Region accuracy observations](region_accuracy/README.md)
- [Sound accuracy analysis](sound_accuracy/README.md)

Run the complete Python witness acceptance check from the repository root:

```sh
uv run --locked python spikes/numerics/check_witnesses.py
```

The checker rejects executable `assert` syntax in every governed witness, then
runs each program with ordinary and optimized Python, applies a 60-second
per-process deadline, and requires byte-identical output. Removable verdicts
therefore fail structurally rather than relying on output parity to reveal
them.

Use the repository-level [experiment catalog](../README.md) to relate these
spikes to research reports. None is production compiler scaffolding.
