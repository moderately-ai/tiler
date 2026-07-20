---
schema: "tiler-doc/v1"
id: "tiler.portal.root"
kind: "portal"
title: "Tiler"
topics: ["orientation"]
---

# Tiler

Tiler is an experimental, consumer-independent Rust compiler toolkit for
optimizing declarative tensor programs and producing efficient parallel compute
kernels. It applies ideas from database optimizers and compiler systems—typed
logical plans, equivalence rules, physical properties, bounded search, cost
models, and explainability—to tensor computation.

The repository is currently design- and research-first. It contains accepted
architecture decisions, proposed and accepted contract material, primary-source
research, and executable feasibility spikes. It does not yet contain a
production compiler implementation.

## Choose a route

- **Understand the project:** start with the [documentation portal](docs/README.md)
  and its short architecture route.
- **Check current state:** read [project status](docs/status.md), then use the
  live ticketsplease commands linked there.
- **Inspect evidence:** use the [research catalog](docs/research/README.md) and
  [experiment catalog](spikes/README.md).
- **Continue the work:** read [AGENTS.md](AGENTS.md) and the
  [work-tracking guide](docs/work-tracking.md) before editing.

Accepted ADRs govern durable architectural choices. Detailed design documents
remain explicit about proposed sections and implementation maturity; a measured
spike is evidence, not production support.
