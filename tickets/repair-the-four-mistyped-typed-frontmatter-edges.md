---
id: repair-the-four-mistyped-typed-frontmatter-edges
title: Repair the four mistyped or dangling typed frontmatter edges
status: review
priority: p3
dependencies: []
related: [govern-the-three-ungoverned-spike-records, reconcile-the-research-and-experiment-catalogs-with-their-frontmatter]
scopes: [contracts/decisions, contracts/artifacts, research/documentation, research/target-profiles, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog, metadata]
claimed_from: todo
assignee: agent-fm-edges
lease_expires_at: 1786052228
---
## User-visible outcome

Every stored typed relationship in the frontmatter graph resolves to a target of the kind the metadata contract types it to, so a reader following an `evidence`, `supports`, or `informs` edge reaches the kind of document that edge promises — and the one dangling id stops making an accepted ADR's evidence list read as complete when it is one short.

## Why this exists

**Measurement — four mistyped or dangling typed edges over a counted population, found while governing the three ungoverned spike records.** Reproduce from the repository root:

```python
import re, pathlib
root = pathlib.Path(".")
def fm(p):
    t = p.read_text(encoding="utf-8", errors="replace")
    return t.split("---", 2)[1] if t.startswith("---") else ""
def one(t, k):
    m = re.search(rf'^{k}:\s*"([^"]+)"', t, re.M); return m.group(1) if m else None
def many(t, k):
    m = re.search(rf'^{k}:\s*\[(.*?)\]', t, re.M | re.S)
    return re.findall(r'"([^"]+)"', m.group(1)) if m else []
kind, path = {}, {}
for p in sorted(root.glob("docs/**/*.md")) + sorted(root.glob("spikes/**/README.md")) + [root / "README.md"]:
    t = fm(p); i, k = one(t, "id"), one(t, "kind")
    if i and k: kind[i], path[i] = k, p
print(f"population: {len(kind)} governed documents with an id and a kind")
bad = checked = 0
for i, k in sorted(kind.items()):
    t = fm(path[i])
    rules = []
    if k == "experiment": rules.append(("supports", ("research",)))
    if k in ("contract", "decision"): rules.append(("evidence", ("research",)))
    if k == "research": rules.append(("informs", ("contract", "decision")))
    for field, allowed in rules:
        for tgt in many(t, field):
            checked += 1
            tk = kind.get(tgt, "<unresolved id>")
            if tk not in allowed:
                print(f"{field.upper():9} {path[i]}: -> {tgt} is {tk}, not {'/'.join(allowed)}"); bad += 1
print(f"evaluated: {checked} typed edges; MISTYPED: {bad}")
```

At `ab75c323` plus the three governed spike records it reports `population: 249 governed documents`, `evaluated: 433 typed edges`, `MISTYPED: 4`:

| Site | Edge | Defect |
| --- | --- | --- |
| [`docs/decisions/0085-admit-tiler-build-as-the-build-time-orchestrator.md`](../docs/decisions/0085-admit-tiler-build-as-the-build-time-orchestrator.md) | `evidence` | names `tiler.research.artifacts.target-neutral-artifact-envelope`; the record's actual id is `tiler.research.artifacts.target-neutral-envelope`, so the edge resolves to nothing |
| [`docs/backends/cpu.md`](../docs/backends/cpu.md) | `evidence` | names `tiler.spike.target-profiles.scalar-cpu-vertical`, an **experiment** |
| [`spikes/target-profiles/scalar-cpu-vertical/README.md`](../spikes/target-profiles/scalar-cpu-vertical/README.md) | `supports` | names `tiler.contract.cpu-backend`, a **contract** |
| [`docs/research/documentation/open-ticket-audit-2026-07-27.md`](../docs/research/documentation/open-ticket-audit-2026-07-27.md) | `informs` | names `tiler.portal.status`, a **portal** |

**Fact — the contract types all three edges, and it refuses the CPU pair by name.** [`docs/document-metadata.md`](../docs/document-metadata.md) types `evidence` as contract-or-ADR to research, `supports` as experiment to research, and `informs` as research to contract (prior art to contract as well). Its section *A decision does not cite an experiment in metadata* states that `evidence` "admits only a `research` target and is deliberately not relaxed to admit an `experiment`", on the ground that a document able to name a harness directly is never pushed to say what bounded universe, environment, and procedure make the measurement carry the weight put on it — naming that boundary being the research record's job. The `cpu.md`/`scalar-cpu-vertical` pair is that exact relaxation, stored twice and in both directions.

**Inference — the dangling ADR edge is the one with a consequence beyond navigation.** ADR 0085 is `decision_status: "accepted"`, and the contract requires an accepted decision to hold at least one `evidence` research record. It lists three and two resolve, so the requirement still holds — but a reader counting its evidence gets three, and the artifact-envelope record it means to cite is not reachable from its frontmatter at all.

**Inference — none of the four is a rendering slip, which is why this is a ticket rather than a sweep.** Repointing the ADR edge to the record's real id is mechanical. The other three each need a reading: whether the CPU vertical's evidence lands on a research record that carries the claim — the same judgement [`govern-the-three-ungoverned-spike-records`](govern-the-three-ungoverned-spike-records.md) made for the reduction-crossover sweep, which it resolved by naming the research record carrying the claim and leaving the contract's ordinary body link to the harness in place — and whether the open-ticket audit's `informs` target is a portal because the audit informs no contract or decision at all, or because the right target was never recorded.

## Required work

- Repoint ADR 0085's `evidence` entry to `tiler.research.artifacts.target-neutral-envelope`, verified against that record's own frontmatter rather than against this table.
- Resolve the `cpu.md`/`scalar-cpu-vertical` pair by reading what the vertical measured and what each candidate research record claims. Remove both mistyped halves; keep the contract's prose link to the harness, which is the mechanism the metadata contract prescribes for a non-research document reaching an experiment.
- Resolve the open-ticket audit's `informs: ["tiler.portal.status"]` — either to a contract or decision the audit actually informs, or by removing the edge and saying in the commit why the audit informs no normative document.
- Update the rendered `experiments:` and `supports:` clauses in [`docs/research/README.md`](../docs/research/README.md) and [`spikes/README.md`](../spikes/README.md) in the same change as any edge that moves; both are hand-maintained derived views with no generator.
- Re-run the check above and the reconciliation check in [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md); report `MISTYPED: 0` and `DISCREPANCIES: 0` over their counted populations, and watch each fail once on a deliberate perturbation before trusting either.

## Explicit non-goals

No validator, no generator, no gate, and no schema change. The corpus deliberately maintains the frontmatter graph by hand; this ticket repairs four edges rather than automating the check that found them. Do not widen `evidence` or `supports` to admit the kinds they currently refuse — the metadata contract records the measured reason it declined that relaxation, and reversing it is an ADR-level decision rather than a repair.

## Closes when

The typed-edge check reports `MISTYPED: 0` over its named population, each of the three judgement calls is recorded with the reading that decided it rather than with the cheaper edit, and both derived catalogs render what the repaired frontmatter says.

## Outcome — 2026-08-06

Delivered on `tkt/repair-the-four-mistyped-typed-frontmatter-edges` at **`f74542fe`** (the four edges and their catalog coherence), **`9476f4c0`** (pre-existing apple-targets catalog drift, so the reconciliation check can report zero), and the ticket-and-contract commit that follows this text. Base `e7823309`.

**Measurement — the check on this branch's base, and the filing table's numbers had drifted.** At `e7823309` it reports `population: 277 governed documents`, `evaluated: 479 typed edges`, `MISTYPED: 4`, against `249`/`433`/`4` at the filing base. The population grew by 28 records and the edge count by 46 as documents landed; the defect set is byte-identical to the four rows above, so nothing new entered and nothing named was already repaired. The contract's own cited run at `ba2b7693` also reproduces exactly — `253`/`439`/`6` — and its sixth edge, unnamed in its prose, is a second contract target on `spikes/numerics/delivered-realization-record/README.md`, repaired before this base by `ddac423b`.

**Measurement — after the repair, at `9476f4c0`:** `population: 277 governed documents`, `evaluated: 477 typed edges`, `MISTYPED: 0`. The edge count drops by two rather than four because two of the four repairs retarget an edge and two remove one.

### Per site, with the contract rule that decided it

**ADR 0085 `evidence` — repointed, mechanical.** `tiler.research.artifacts.target-neutral-artifact-envelope` became `tiler.research.artifacts.target-neutral-envelope`, read off [that record's own frontmatter](../docs/research/artifacts/target-neutral-artifact-envelope.md) rather than off the table above. *Rule:* "Every document has a stable `id` independent of its path. Paths are presentation; IDs are graph identity." The record's path carries `artifact` and its id does not, which is exactly the trap that rule names. The ADR catalog row already rendered the record by path and title and needed no change — which is the independent confirmation that this record, not some other, is the one the ADR means.

**`docs/backends/cpu.md` `evidence` — dropped, not replaced.** *Rule:* `evidence` is contract-or-ADR **to research**, and *A decision does not cite an experiment in metadata* records that it "admits only a `research` target and is deliberately not relaxed to admit an `experiment`", because a document able to name a harness directly is never pushed to say what bounded universe, environment, and procedure make the measurement carry the weight put on it. The contract names the substitute mechanism in the same section — "an ordinary body link to the checked-in harness" — and `cpu.md` already carries one in its *Traceability* section, where the vertical is described as "the only implementation evidence" and its exact bounds are stated in prose. Nothing was added in its place: `cpu.md` is `proposed`, so it owes no accepted-decision evidence, and the two research records it retains are untouched. The route from the contract to the harness now runs through prose and through `backend-provider-composition`, which `informs` this contract and is `supported` by the vertical — the two-link route the contract prescribes.

**`spikes/target-profiles/scalar-cpu-vertical/README.md` `supports` — retargeted to the record carrying the claim.** `tiler.contract.cpu-backend` became `tiler.research.extensions.backend-provider-composition`. *Rule:* `supports` is experiment **to research**, and [`govern-the-three-ungoverned-spike-records`](govern-the-three-ungoverned-spike-records.md) settled the same shape for the reduction-crossover sweep by naming the research record carrying the claim rather than deleting the edge. *The reading that chose the record:* [`backend-provider-composition`](../docs/research/extensions/backend-provider-composition.md) states in its Reproductions section that "the reproductions are the forkless custom Metal physical provider … and the bounded scalar CPU backend vertical", opens by saying this vertical's "eleven findings are the second half of this record's input", and cites findings 2, 3, 4, 5, 6, 7, 8, 9, 10, and 11 by number through its body. The spike's own README names that record's ticket, `specify-the-consumer-neutral-backend-provider-composition-contract`, as what its Findings are "the payload" for. So the research side already asserted the relationship and only the forward edge was missing; `reproduced_by` is invalid in stored v1 frontmatter, so the experiment record is the only place it can live. `physical-feasibility-model` is unchanged.

**`docs/research/documentation/open-ticket-audit-2026-07-27.md` `informs` — dropped, and the reading is that it informs no normative document.** *Rule:* `informs` is research to contract, with prior art to contract as the only widening; a portal is not an admissible target in any spelling, and research cannot fall back on `related` either, because the optional-field column "is its exhaustive licence" and Research's optional column lists only `adopted_by` and `ticket`. *The reading:* the audit informs no contract or decision that exists. Nothing under `docs/` cites it — the only reference in the corpus is its own research-catalog row. Its `disposition` is `pending`, so nothing has adopted it. Its own *Disposition* section states its product is that "the ticket edits are planning corrections rather than implementation evidence", and its findings reached the tree as six remediation tickets rather than as contract text. What it does inform — the ticket board, and the work-tracking process in `AGENTS.md` — carries no `tiler-doc/v1` identity, so there is no admissible target to retarget to. Retargeting at `tiler.contract.document-metadata` was rejected on the contract's own *Ownership* section, which disclaims "ticketsplease's ticket schema"; retargeting at ADR 0075 was rejected because the audit *applies* its needs-tom categories rather than supplying evidence for them.

**One tension surfaced, not resolved, because resolving it is a schema change this ticket forbids.** The required-field table types `informs` as required on every research record, while the sentence below it requires an `informs` or `adopted_by` destination only of *adopted or partially adopted* research. Dropping the audit's edge is admissible under the second and not under the first. The corpus already sits in that gap once, at `tiler.research.region-search.enforcer-input-property-exclusion` (`informational`, no `informs`, no `adopted_by`), so the audit is the second instance rather than the first. Which sentence governs is a contract change and needs its own ticket.

### Derived views and entry points

Both hand-maintained catalogs moved with the frontmatter in the same commit: the research catalog gained the vertical in `backend-provider-composition`'s `experiments:` clause and lost the audit's `informs:` clause; the experiment catalog renders the vertical's new `supports` target. The ADR catalog needed no edit, for the reason given above. [`docs/document-metadata.md`](../docs/document-metadata.md) carried a `ba2b7693` measurement naming these defects as live; it now carries the re-run beside it, recording that all six are repaired without weakening the paragraph's point that reading did not catch any of them.

### The reconciliation check, and four rows that were not this ticket's

The check in [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) reported `DISCREPANCIES: 4` — **identically at `e7823309` and with the typed-edge repair applied**, so none of the four was caused here. All four are the two rendered views a governed spike record left short: `tiler.spike.apple-targets.evaluation-order` landed with no experiment-catalog row and was missing from `numerical-behaviour`'s clause, `permitted-divergence-oracle` had no `experiments:` clause at all, and the `apple-targets` row missed the oracle. Its ticket is `done`, so nothing owned them; both catalogs are this branch's exclusive `contracts/navigation` scope and no live claim holds it. They are rendered in `9476f4c0` from the frontmatter that already exists, with no judgement and no record edited. After it: `population: 100 research rows, 100 research records, 44 experiments`, `44 experiment rows`, `0 ungoverned`, `DISCREPANCIES: 0`.

### Both checks were watched failing before being trusted

- Typed-edge, repointing ADR 0085's `evidence` at `tiler.portal.status`: `MISTYPED: 1`, naming the file, the target, and `is portal, not research`.
- Reconciliation, dropping the vertical from `backend-provider-composition`'s `experiments:` clause: `DISCREPANCIES: 1`, `EXPMTS … row [...] != supports-inverse [...]`.
- Reconciliation, mistyping the evaluation-order probe's title in the experiment catalog: `DISCREPANCIES: 1`, `TITLE … 'Metal emitted evaluation-order probe' != 'Metal emitted-evaluation-order probe'`.

Every perturbation was reverted and both checks re-run green on a clean tree; `git status` empty afterwards.

`tkt lint` clean, `git diff --check` clean, `tkt guard` reports only this ticket's declared scopes. No Cargo gate applies: the diff is `docs/` and `spikes/**/README.md` only, touching none of the paths that require one.
