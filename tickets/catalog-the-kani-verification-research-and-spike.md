---
id: catalog-the-kani-verification-research-and-spike
title: Catalog the Kani verification research record and spike
status: in-progress
priority: p3
dependencies: []
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, catalog, verification]
claimed_from: todo
assignee: coord
lease_expires_at: 1786176329
---
## Per-Fact audit at `68ba010a`, 2026-08-08

Every claim below was re-read in full at this base before any edit. The bodies that follow carry the corrected text.

| Ticket Fact | Verdict | Evidence |
| --- | --- | --- |
| The corpus rule in `docs/document-metadata.md` is "edit the affected catalog entry in the same change that edits the metadata behind it" | **verified, with a searchability caveat** | `docs/document-metadata.md "Edit the affected catalog entry in the same change that"`, in the section `Validation and catalog updates`. The source sentence is hard-wrapped and begins with a capital, so the ticket's rendered-prose form greps as **absent** — the dangerous reading, since absence looks like removal. The single-line fragment cited here is the durable anchor. |
| `spike-kani-bounded-verification-on-one-inexhaustible-encoder` holds `scopes: [research/verification]` | **verified** | `tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md "scopes: [research/verification]"`; that ticket is `status: done`. |
| `research/verification` maps to `docs/research/verification/**` and `spikes/verification/**` | **verified** | `ticketsplease.toml "research/verification"`, whose glob list is exactly those two. |
| `contracts/navigation` carries both `docs/research/README.md` and `spikes/README.md` | **verified** | `ticketsplease.toml "contracts/navigation"`; both names appear in that glob list. |
| "the **generated** research catalog in `docs/research/README.md`" | **false** | The file says the opposite of itself: `docs/research/README.md "The rows below are **maintained by hand**, and nothing checks them."`, continuing "A renderer once produced this block from frontmatter; it was deleted along with the rest of the repository's Python tooling". `spikes/README.md "maintained by hand"` says the same of the other catalog, and `docs/document-metadata.md "There is no validator and no renderer."` says it of the corpus. `make-the-research-catalog-generated-or-stop-claiming-it-is` is open on exactly this wording. Corrected below; a worker who believed it would have waited for a renderer that does not exist. |
| "`verification/` is a new catalog section subject in both files; check whether the surrounding grouping needs a heading" | **false premise** | Catalog sections are not per-directory. `docs/document-metadata.md "Its controlled values are"` fixes `catalog_group` to seven values, and the research record carries `catalog_group: "artifacts-build-toolchains"` — an existing heading in both files — so a new heading is not admissible without amending that contract. The existing **Artifacts, build, and toolchains** section already mixes `apple-targets/`, `artifacts/`, `cache/`, `embedding/`, and `macro-environment/`, so a new directory has never implied a new section. A row only. |
| Both rows restate frontmatter that must be re-read | **verified, and both rows are correct as written** | Research record `docs/research/verification/kani-bounded-encoder-verification.md`: `disposition: "pending"`, `evidence_classes: ["executable-model", "bounded-measurement", "primary-source-synthesis"]`, `informs: ["tiler.contract.correctness-and-testing"]`. That id resolves to `docs/correctness-and-testing.md "tiler.contract.correctness-and-testing"`, whose `title` is `Correctness and testing` — the exact link text the row uses. Spike record: `experiment_status: "reproducible"`, `evidence_classes: ["executable-model", "bounded-measurement"]`, and `spikes/verification/kani-encoder-injectivity/README.md "tiler.research.verification.kani-bounded-encoder-verification"` as its `supports` target, matching the research record's `id`. Both rows landed verbatim. |
| The spike is unreachable, so the catalogs are the gap | **verified as to the catalogs, and the spike is otherwise correctly linked** | The `spikes/` convention is satisfied in both directions already: the research record links its reproduction at `docs/research/verification/kani-bounded-encoder-verification.md "**Reproduction:**"`, and the spike links back at `spikes/verification/kani-encoder-injectivity/README.md "This file is the invocation record."`. The catalog rows were the only missing route. |

## User-visible outcome

The Kani verification research record and its spike are reachable from the catalogs, so the corpus rule in `docs/document-metadata.md` — "edit the affected catalog entry in the same change that edits the metadata behind it" — is satisfied for them.

## Why this is a separate ticket

`spike-kani-bounded-verification-on-one-inexhaustible-encoder` holds `scopes: [research/verification]`, which maps to `docs/research/verification/**` and `spikes/verification/**`. Both catalogs live in the `contracts/navigation` scope — grep `ticketsplease.toml` for `"contracts/navigation"`, whose glob list carries both `docs/research/README.md` and `spikes/README.md`. The spike worker could not edit them without under-declaring scope, so the rows are preserved verbatim here for whoever holds that scope, per the `AGENTS.md` carrier-ticket practice.

## The two rows to land, verbatim

Into the hand-maintained research catalog in `docs/research/README.md`, under **Artifacts, build, and toolchains** — the heading the record's own `catalog_group: "artifacts-build-toolchains"` names — matching the surrounding row format and alphabetical position:

```
- [Kani bounded verification of inexhaustible identity encoders](verification/kani-bounded-encoder-verification.md) — pending; executable-model, bounded-measurement, primary-source-synthesis; informs: [Correctness and testing](../correctness-and-testing.md); experiments: [Kani bounded verification of inexhaustible identity encoders](../../spikes/verification/kani-encoder-injectivity/README.md)
```

Into the equally hand-maintained spike catalog in `spikes/README.md`, under the same heading and matching that file's row format:

```
- [Kani bounded verification of inexhaustible identity encoders](verification/kani-encoder-injectivity/README.md) — reproducible; executable-model, bounded-measurement; supports: [Kani bounded verification of inexhaustible identity encoders](../docs/research/verification/kani-bounded-encoder-verification.md)
```

## Verify before landing

Both rows are written against the frontmatter as it stands at the spike's commit. Read both records' frontmatter and confirm `disposition`, `evidence_classes`, `informs`, and `supports` still say what the rows claim — the rows restate frontmatter and are wrong the moment it moves. Done at `68ba010a`; see the per-Fact audit above. No heading is added: the record's `catalog_group` names an existing one, and the controlled vocabulary in `docs/document-metadata.md` admits no new value.

## The maturity and evidence rungs these rows claim

Stated explicitly, because a verification spike is where the two ladders blur. **Maturity: spike-only, and the primary path is blocked.** Both records carry `implementation_status: "spike-only"`, and the record's headline Fact is that `crates/tiler-ir` does not compile under Kani 0.67.0's bundled rustc, so no harness runs against the real crate. This is an architectural seam probed, not implemented support and not a tested guarantee.

**Evidence: `executable-model` and `bounded-measurement`, deliberately not `sound-proof` and not `normative-guarantee`.** The rows restate that, and the restraint is load-bearing rather than an oversight. Three harnesses do discharge over their entire input domains with CBMC's unwinding assertion reported `SUCCESS`, which is stronger than a bounded model check — but the subject is a set of verbatim **copies** of the encoders, tied to their sources only by `guard.sh`, which no `make` target runs. The record names that as a *provenance* limitation the existing vocabulary has no slot for, and routes the classification question to Tom rather than deciding it. A row reading `sound-proof` here would assert a property of Tiler's encoders that the evidence establishes only about copies of them.

## Closes when

Both catalogs carry the rows, the links resolve, and the frontmatter each row restates has been re-read rather than trusted.

## Landed

At `68ba010a`, on `tkt/catalog-the-kani-verification-research-and-spike`. Both rows are in place, alphabetically between the `Expansion cache hot-path efficiency` row and the `Proc-macro` row in each file. `spikes/README.md` also gained a prose paragraph: its **Running a spike** section asserted that a Cargo-workspace spike resolves the repository `rust-toolchain.toml` pin without a selector, which this spike measurably does not — Kani selects its own bundled nightly — so cataloguing the row without correcting the sentence would have left the catalog contradicting a row it now carries.
