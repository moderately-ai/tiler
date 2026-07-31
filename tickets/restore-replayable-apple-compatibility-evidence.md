---
id: restore-replayable-apple-compatibility-evidence
title: Restore replayable Apple compatibility evidence
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [evidence, apple, compatibility]
claimed_from: todo
assignee: loop-restore-repl
lease_expires_at: 1785520143
---
## User-visible outcome

The retained Apple artifact-compatibility record can be validated by the exact producer and validator revisions it names, or a replacement measurement is retained whose record, inputs, harness, and validator all reproduce from the checked-in tree.

## Why this is needed

The 2026-07-21 record's exact producer bytes are reachable at repository commit `b0fdba7`; the earlier claim that no reachable revision contains them is false. The recorded digests match that revision's harness (`b37ba8…`), validator (`63f579…`), `pyproject.toml` (`e35ae6…`), `uv.lock` (`e8a38c…`), and `copy.metal`. Current-tree replay fails for a different reason: the post-Python-tooling validator expects a three-row manifest, while the retained historical record has five rows.

The formerly documented root `uv run --project . --locked …` command is no longer a valid current-tree replay because the root Python environment was deliberately deleted. The retained outputs, settings, logs, and record remain inspectable evidence; this ticket restores a self-contained historical replay rather than reviving deleted repository tooling.

## Implementation keys

Recover the exact producer set from commit `b0fdba7`, retain those immutable bytes beside the historical result, and make the replay select the retained historical validator and five-row manifest explicitly. The replay must first verify every retained producer digest, so a locally edited historical harness cannot validate its own changed identity.

Do not route this historical replay through the deleted root Python environment or the current three-row validator. A new measurement is unnecessary unless the exact `b0fdba7` producer set itself proves incomplete after byte-for-byte recovery.

Prove the validator can reject by perturbing one retained producer field and observing failure before recording the positive replay.

## Graph maintenance

When replay is restored, update `docs/research/apple-targets/artifact-compatibility.md` to distinguish the historical record from the new or recovered reproducible evidence and close this ticket. Any newly observed family, MSL-version, or deployment-minimum behavior belongs in a separate compatibility-widening ticket rather than being inferred from successful replay.
