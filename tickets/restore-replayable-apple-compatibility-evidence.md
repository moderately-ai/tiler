---
id: restore-replayable-apple-compatibility-evidence
title: Restore replayable Apple compatibility evidence
status: done
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

The 2026-07-21 record's exact producer bytes are reachable at repository commit `b0fdba7`; the earlier claim that no reachable revision contains them is false. The recorded digests match that revision's harness (`b37ba8…`), validator (`63f579…`), `pyproject.toml` (`e35ae6…`), `uv.lock` (`e8a38c…`), and `copy.metal`. Current-tree replay fails for a different reason: the post-Python-tooling validator expects a three-row manifest, while the retained historical record has five rows.

The formerly documented root `uv run --project . --locked …` command is no longer a valid current-tree replay because the root Python environment was deliberately deleted. The retained outputs, settings, logs, and record remain inspectable evidence; this ticket restores a self-contained historical replay rather than reviving deleted repository tooling.

## Implementation keys

Recover the exact producer set from commit `b0fdba7`, retain those immutable bytes beside the historical result, and make the replay select the retained historical validator and five-row manifest explicitly. The replay must first verify every retained producer digest, so a locally edited historical harness cannot validate its own changed identity.

Do not route this historical replay through the deleted root Python environment or the current three-row validator. A new measurement is unnecessary unless the exact `b0fdba7` producer set itself proves incomplete after byte-for-byte recovery.

Prove the validator can reject by perturbing one retained producer field and observing failure before recording the positive replay.

## Restoration evidence (2026-07-31)

**Fact — the producer set recovered byte for byte, so no new measurement was needed.** All five inputs the retained `input-manifest.tsv` names resolve at `b0fdba7` and every recovered digest equals the one the record states: `compatibility_probe.sh` `b37ba89d5b0aaab5e0b0f6b1ecc1fc791b4885339c23ee42414794940b4ae745`, `copy.metal` `fa7e29166a2f6aa8372681eb580f43592b5c8a5600ebf4f02c69682c6892b4e7`, `validate_compatibility_record.py` `63f5793f24a264d9cbb52ec1a9c5176c2b03feb930bce7e8b465c6eae421165f`, `pyproject.toml` `e35ae608cc3a54cad121a353e61873ae3f03ed3c526ad9df31397c7668b16a6e`, `uv.lock` `e8a38c0d668ed4db44d8a4c6b208ecdd39f9942a924ad738fa7d040b9c935bf3`. Five files recovered against five manifest rows, counted rather than assumed. They are retained under `spikes/apple-targets/results/2026-07-21-xcode26.6-metal32023.883/producers/`, laid out by the repository-relative paths the manifest uses, as data rather than as executables.

**Fact — `b0fdba7` is the unique commit carrying those bytes, and the record's own base revision is not it.** Hashing the blob at every commit that touched each file (`git log --format=%H -- <path> | while read -r c; do git show "${c}:<path>" | shasum -a 256; done`; six commits for the harness, four for the validator) leaves `b0fdba7` as the only match. The record's `probe.repository_base_revision` `f3004a1` carries `1425b635…` and `05f69902…` instead, so the run was taken from a tree based on `f3004a1` with the producer edits uncommitted; they landed one commit later. The recorded base revision alone therefore cannot recover what ran, which is why the bytes are retained rather than merely referenced.

**Fact — the current-tree validator's refusal is a producer-generation difference.** `python3 spikes/apple-targets/validate_compatibility_record.py <record>` exits 2 with `retained input manifest does not match producer fields`: it expects the three-row manifest the post-`e197176` harness writes, and this record correctly binds five.

**Measurement — the restored replay, run from the repository root on macOS 27.0 arm64.** `spikes/apple-targets/replay_retained_compatibility_record.sh spikes/apple-targets/results/2026-07-21-xcode26.6-metal32023.883` exits 0 after printing `verified` lines for the manifest and all five producers and then `verified_producers=5`. It needs no Apple toolchain and no Python project environment; both retained validators are standard-library-only. It verifies the manifest digest, then each retained byte against both its manifest row and the record's own producer field, and only then executes the retained validator.

**Measurement — every rejection was observed, not assumed.** Seven perturbations, each restored with the positive replay re-run afterwards: an appended comment in the retained validator, and separately in the retained harness, fails at the producer-digest step before the validator runs; a zeroed `probe.validator_sha256` fails the record-to-bytes cross-check; a `short` record digest fails the SHA-256 shape check; a duplicated record key fails `record does not carry exactly one probe.source_sha256 row`; a deleted `producers/uv.lock` fails as a missing producer; and an extra empty file under `producers/` fails the population count. `uv run --with pytest pytest spikes/apple-targets` passes 89 tests.

**Fact — the first draft of the replay answered two of those with the wrong reason, and running them is what caught it.** `fail` inside a command substitution exits only the subshell, so the malformed-digest and duplicate-key cases printed their true reason and then carried on with an empty value and printed a second, misleading mismatch. The digest helpers answer through globals now and each perturbation yields exactly one message. The script exited nonzero either way, which is why running the failures rather than reasoning about them was necessary.

**Fact — Git stored the retained producers as the same objects `b0fdba7` holds.** `git ls-files -s` reports blob IDs `a722b9e8`, `abc38f60`, `feab1835`, `12b7cd2e`, and `0b281eb2`, identical to `git ls-tree b0fdba7` for the five paths. The retention adds no new bytes to the object store and byte identity is confirmed independently of the SHA-256 comparison.

**Fact — `results/.gitignore` no longer ignores the retained kernel by filename.** Its bare `copy.metal` rule matched the retained producer copy; it now names the harness's two staging directories (`**/src-a/copy.metal`, `**/src-b/copy.metal`). Both directions were checked with `git check-ignore`: the five producers are trackable and synthetic staging and compile products stay ignored.

## Graph maintenance

When replay is restored, update `docs/research/apple-targets/artifact-compatibility.md` to distinguish the historical record from the new or recovered reproducible evidence and close this ticket. Any newly observed family, MSL-version, or deployment-minimum behavior belongs in a separate compatibility-widening ticket rather than being inferred from successful replay.

The report now carries a **Replay status (restored 2026-07-31)** paragraph stating both corrections and the exact invocation, and its "Exact probes" section records the replay command with the boundary that nothing is recompiled. The historical row's provenance paragraphs are unchanged. No family, MSL-version, or deployment-minimum behaviour was observed, so no compatibility-widening ticket is filed.
