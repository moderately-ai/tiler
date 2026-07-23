---
schema: "tiler-doc/v1"
id: "tiler.spike.documentation.integrity-gate"
kind: "experiment"
title: "Documentation integrity gate"
topics: ["documentation", "validation", "provenance"]
experiment_status: "reproducible"
implementation_status: "implemented"
evidence_classes: ["executable-model"]
supports: ["tiler.research.documentation.information-architecture-audit"]
entrypoints: ["scripts/docs.py", "scripts/tests/test_docs.py"]
last_verified: "2026-07-23"
ticket: "docs-integrity-gate"
---

# Documentation integrity gate

This checker uses the repository's locked CommonMark parser to validate the
governed Markdown schemas, typed graph, ticket and entrypoint references, local
links, open-question shape, and deterministic generated catalogs.

From the repository root:

```sh
uv run --locked python scripts/docs.py validate
uv run --locked python scripts/docs.py render --check
uv run --locked python scripts/check_repository.py
```

The executable model checks structural integrity. It cannot prove that prose is
complete, evidence is scientifically sufficient, or a reader will interpret a
mixed contract correctly; independent acceptance reading remains necessary.

## Traceability

- **Supported audit:** [information architecture](../../docs/research/documentation/information-architecture-audit.md).
- **Explicit non-support:** the [blank-agent narrative](../../docs/research/documentation/blank-agent-acceptance-audit.md) lacks retained prompts and outputs; this checker cannot establish reader interpretation.
- **Normative owner:** [documentation metadata](../../docs/document-metadata.md).
- **Work record:** [docs-integrity-gate](../../tickets/docs-integrity-gate.md).
