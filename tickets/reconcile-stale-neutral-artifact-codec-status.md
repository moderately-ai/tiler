---
id: reconcile-stale-neutral-artifact-codec-status
title: Reconcile the stale neutral-artifact-codec status statements
status: todo
priority: p2
dependencies: []
related: [record-the-implemented-artifact-envelope-in-the-contract, prototype-neutral-artifact-codec]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifact]
---
`record-the-implemented-artifact-envelope-in-the-contract` recorded the implemented codec in `docs/artifact-abi.md` and reported three governed records it could not edit, because it held `contracts/artifacts` alone. This ticket owns them.

**Fact — verified by reading at `f57e23b`.** Three statements describe the neutral artifact codec as unimplemented and are overtaken by `prototype-neutral-artifact-codec`:

- `docs/research/artifacts/target-neutral-artifact-envelope.md` carries `implementation_status: "spike-only"` in frontmatter, a status line reading "serialization implementation remains future work", and a closing Traceability sentence, "Production serialization, authenticity, and version-skew policy remain unimplemented." Serialization is implemented; authenticity and version-skew policy genuinely are not, so the sentence must be split rather than deleted. Scope: `research/artifacts`.
- `docs/status.md:103` says "Separate tickets now track the neutral artifact codec, Metal lowering and offline driver, …". The neutral artifact codec ticket is `done`. Scope: `contracts/navigation`.
- `docs/roadmap.md:125` lists "artifact codec/bundle assembly" among the Metal split still ahead. The codec half has landed; bundle assembly has not. Scope: `contracts/navigation`.

**The constraint that makes this non-mechanical.** `docs/artifact-abi.md` deliberately states `partial`, not implemented, because every codec item is `pub(crate)` behind an unaccepted facade under ADR 0074 convention 7. Do not let any of the three edits above upgrade past that: the accurate claim is that a bounded lockstep codec exists behind an unaccepted facade, and each record must agree with `docs/artifact-abi.md` rather than out-run it. `partial` is the matching frontmatter value for the research record.

**Closes when** the three records state the implemented subset at the same maturity `docs/artifact-abi.md` does, and `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
