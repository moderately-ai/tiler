---
id: restore-replayable-apple-compatibility-evidence
title: Restore replayable Apple compatibility evidence
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [evidence, apple, compatibility]
---
## User-visible outcome

The retained Apple artifact-compatibility record can be validated by the exact producer and validator revisions it names, or a replacement measurement is retained whose record, inputs, harness, and validator all reproduce from the checked-in tree.

## Why this is needed

The 2026-07-21 record reports producer digests that are not present in any reachable revision of the current harness and validator. The current one-line replay fails:

```sh
uv run --project . --locked python spikes/apple-targets/validate_compatibility_record.py spikes/apple-targets/results/2026-07-21-xcode26.6-metal32023.883/record.tsv
```

It reports `retained input manifest does not match producer fields`. The retained outputs, settings, logs, and record remain inspectable evidence, but the checkout cannot presently reproduce the validation claim using the exact historical producer blobs the record identifies.

## Implementation keys

First try to recover the exact historical `compatibility_probe.sh` and `validate_compatibility_record.py` bytes by their recorded SHA-256 digests from durable repository or external history. If they are recovered, retain them beside the result and make the replay command select those files explicitly.

If the exact producers cannot be recovered, run a fresh bounded compatibility measurement with the current checked-in harness and validator, retain its complete input manifest and raw outputs, and update the research note without rewriting the historical row as though it had become replayable. Keep the old record for provenance.

Prove the validator can reject by perturbing one retained producer field and observing failure before recording the positive replay.

## Graph maintenance

When replay is restored, update `docs/research/apple-targets/artifact-compatibility.md` to distinguish the historical record from the new or recovered reproducible evidence and close this ticket. Any newly observed family, MSL-version, or deployment-minimum behavior belongs in a separate compatibility-widening ticket rather than being inferred from successful replay.
