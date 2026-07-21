---
schema: "tiler-doc/v1"
id: "tiler.spike.documentation.integrity-gate"
kind: "experiment"
title: "Documentation integrity gate"
topics: ["documentation", "validation", "provenance"]
experiment_status: "reproducible"
implementation_status: "implemented"
evidence_classes: ["executable-model"]
supports: ["tiler.research.documentation.information-architecture-audit", "tiler.research.documentation.blank-agent-acceptance-audit"]
entrypoints: ["scripts/docs.py", "scripts/tests/test_docs.py"]
last_verified: "2026-07-20"
ticket: "docs-integrity-gate"
---

# Documentation integrity gate

This standard-library-only checker validates the governed Markdown schemas,
typed graph, ticket and entrypoint references, local links, open-question shape,
and deterministic generated catalogs.

From the repository root:

```sh
python3 -B scripts/docs.py validate
python3 -B scripts/docs.py render --check
uv run --locked pytest
```

The executable model checks structural integrity. It cannot prove that prose is
complete, evidence is scientifically sufficient, or a reader will interpret a
mixed contract correctly; independent acceptance reading remains necessary.

## Traceability

- **Supported audits:** [information architecture](../../docs/research/documentation/information-architecture-audit.md) and [blank-agent acceptance](../../docs/research/documentation/blank-agent-acceptance-audit.md).
- **Normative owner:** [documentation metadata](../../docs/document-metadata.md).
- **Work record:** [docs-integrity-gate](../../tickets/docs-integrity-gate.md).
