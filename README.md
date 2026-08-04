---
schema: "tiler-doc/v1"
id: "tiler.portal.root"
kind: "portal"
title: "Tiler"
topics: ["orientation"]
---

# Tiler

Tiler is an experimental, consumer-neutral ahead-of-time tensor-program
compiler and execution toolkit. It accepts typed logical programs with explicit
inputs and ordered named outputs, performs target-independent logical
optimization and target-aware physical planning, lowers selected plans to
structured kernels and artifacts, and generically binds and executes them. It
applies ideas from database optimizers and compiler systems—typed logical
plans, equivalence rules, physical properties, bounded search, cost models, and
explainability—to tensor computation.

The repository is currently design- and research-first. It contains accepted
architecture decisions, proposed and accepted contract material, primary-source
research, and an executable bounded compiler-to-Metal prototype. It does not
yet contain a production or workload-general compiler implementation.

Tiler compiles tensor programs; it does not own a model, transformer, training
or inference loop, KV cache, sampler, server, or application session. A
consumer may use those workloads as conformance tests and may retain an output
tensor as a later invocation's input, but that composition remains outside the
compiler's semantic and runtime state.

## Choose a route

- **Understand the project:** start with the [documentation portal](docs/README.md)
  and its short architecture route.
- **Check current state:** read [project status](docs/status.md), then use the
  live ticketsplease commands linked there.
- **Inspect evidence:** use the [research catalog](docs/research/README.md) and
  [experiment catalog](spikes/README.md).
- **Continue the work:** read [AGENTS.md](AGENTS.md) and the
  [work-tracking guide](docs/work-tracking.md) before editing.

Bootstrap a macOS development host with `./deps.sh`. Use `./deps.sh --check` for
a non-mutating dependency diagnosis.

Verify with `make check` (format, Clippy, tests) while working, and `make full`
before pushing to `main`. Every target is a single command you can also type
directly; `rust-toolchain.toml` selects the compiler, so plain `cargo` is
already the pinned one. Spikes are not covered by either target — run a spike
from its own directory when you are working on it.

Accepted ADRs govern durable architectural choices. A `mixed` contract treats
unmarked field-level detail as proposed unless an accepted ADR is cited; every
document states implementation maturity. A measured spike is evidence, not
production support.
