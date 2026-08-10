---
id: catalog-the-kani-push-slice-framing-research-and-spike
title: Catalog the Kani push_slice framing research and spike
status: done
priority: p3
dependencies: [spike-kani-push-slice-framing-over-a-symbolic-byte-run]
related: [catalog-the-kani-verification-research-and-spike]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---

## User-visible outcome

The manually maintained research and experiment catalogs route readers to the bounded `push_slice` framing result and state the same metadata as the records themselves.

## Facts read before filing — 2026-08-10, base `49d38237`

- `docs/research/README.md` owns its hand-maintained rows and lives in `contracts/navigation`; the Kani rows sort in **Artifacts, build, and toolchains**.
- `spikes/README.md` owns the parallel experiment catalog in that same scope and already explains the authorized Kani 0.67.0 host prerequisite for the predecessor spike.
- The dependency's branch adds `docs/research/verification/kani-push-slice-framing.md` with `disposition: "pending"`, evidence classes `executable-model`, `bounded-measurement`, `primary-source-synthesis`, and `informs: ["tiler.contract.correctness-and-testing"]`.
- It also adds `spikes/verification/kani-push-slice-framing/README.md` with `experiment_status: "reproducible"`, evidence classes `executable-model`, `bounded-measurement`, and support for the new research record.

## Per-Fact audit at `993da43e4955c6f99ce80d5412ea67886bddc11d` — 2026-08-10

| Ticket Fact | Verdict | Evidence |
| --- | --- | --- |
| The catalogs are hand-maintained in `contracts/navigation`, and the Kani rows sort in **Artifacts, build, and toolchains** | **verified** | `docs/research/README.md` says "The rows below are **maintained by hand**" and contains the predecessor row under `### Artifacts, build, and toolchains`; `spikes/README.md` says the rows are "**maintained by hand**" and likewise contains its predecessor row in that section. `ticketsplease.toml` maps both files to `contracts/navigation`. |
| `spikes/README.md` already explains the authorized Kani 0.67.0 host prerequisite for the predecessor spike | **verified but imprecise for this outcome** | Its `## Running a spike` paragraph begins "One Cargo-workspace spike" and names only the inexhaustible-encoder spike. It accurately records Kani 0.67.0, its bundled nightly, and the authorized host addition, but cannot yet apply to both catalogued Kani spikes. Correct the singular framing without changing either spike's proof claims. |
| The dependency adds the push-slice research record with the stated disposition, evidence classes, and `informs` id | **verified** | `docs/research/verification/kani-push-slice-framing.md` frontmatter has `disposition: "pending"`, `evidence_classes: ["executable-model", "bounded-measurement", "primary-source-synthesis"]`, and `informs: ["tiler.contract.correctness-and-testing"]`. |
| The dependency adds the push-slice spike README with the stated status, evidence classes, and support target | **verified** | `spikes/verification/kani-push-slice-framing/README.md` frontmatter has `experiment_status: "reproducible"`, `evidence_classes: ["executable-model", "bounded-measurement"]`, and `supports: ["tiler.research.verification.kani-push-slice-framing"]`. |

The imprecision leaves the ticket's purpose unchanged: land the two catalog rows and generalize only the prerequisite paragraph so both Kani spikes are described accurately.

## Verbatim-landable rows

Research catalog, beside the existing Kani record:

```md
- [Kani bounded verification of `push_slice` framing](verification/kani-push-slice-framing.md) — pending; executable-model, bounded-measurement, primary-source-synthesis; informs: [Correctness and testing](../correctness-and-testing.md); experiments: [Kani bounded verification of `push_slice` framing](../../spikes/verification/kani-push-slice-framing/README.md)
```

Experiment catalog, beside the existing Kani spike:

```md
- [Kani bounded verification of `push_slice` framing](verification/kani-push-slice-framing/README.md) — reproducible; executable-model, bounded-measurement; supports: [Kani bounded verification of `push_slice` framing](../docs/research/verification/kani-push-slice-framing.md)
```

## Closes when

Both rows are present, their links resolve after the dependency lands, and the surrounding Kani host-prerequisite prose still applies to both spikes without claiming that either follows the repository `rust-toolchain.toml` pin.

## Work log — 2026-08-10

The new research and spike rows were inserted immediately after their
alphabetical predecessor in **Artifacts, build, and toolchains**. The Kani
prerequisite paragraph now names both catalogued spikes, preserves the
predecessor's authorization provenance, and says that the newer spike reuses
that installation without an install or update.

`make citations` was deliberately perturbed by changing the new research-row
target to `verification/kani-push-slice-framing-MISSING.md`; it failed with
`no tracked file or directory at docs/research/verification/kani-push-slice-framing-MISSING.md`.
The path was restored before the passing citation check.

**Full-gate carry.** This delta changes only `docs/research/README.md`,
`spikes/README.md`, and this ticket. It touches none of the `AGENTS.md`
full-gate-trigger paths (`crates/`, `prototypes/`, root Cargo manifests,
`.config/`, `Makefile`, toolchain/format files, `deps.sh`, or
`check-citations.sh`), so it carries the prior full gate while rerunning
`make citations` and `tkt lint`.
