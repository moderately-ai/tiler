---
id: repair-the-four-mistyped-typed-frontmatter-edges
title: Repair the four mistyped or dangling typed frontmatter edges
status: todo
priority: p3
dependencies: []
related: [govern-the-three-ungoverned-spike-records, reconcile-the-research-and-experiment-catalogs-with-their-frontmatter]
scopes: [contracts/decisions, contracts/artifacts, research/documentation, research/target-profiles, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog, metadata]
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
