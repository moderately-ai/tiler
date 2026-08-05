---
id: reconcile-the-research-and-experiment-catalogs-with-their-frontmatter
title: Reconcile the research and experiment catalogs with their frontmatter
status: review
priority: p3
dependencies: []
related: [design-model-level-qualification-and-optimization, land-the-model-level-qualification-record]
scopes: [contracts/navigation, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog]
claimed_from: todo
assignee: agent-catalogs
lease_expires_at: 1785942011
---
## User-visible outcome

Every row of the two hand-maintained catalogs says what the frontmatter behind it says, so a reader following an `experiments:` or `supports:` link reaches the evidence that record actually claims — and the rows that silently lost an experiment when a spike grew a new `supports` edge stop hiding it.

## Why this exists

**Fact — the catalogs are derived views with no generator and no gate.** [`docs/document-metadata.md`](../docs/document-metadata.md) fixes the derivation: "the ADR catalog names each decision's research records, and the research catalog names each research record's experiments". The research catalog's `experiments:` clause is therefore the inverse of the experiment records' `supports:` lists, and the experiment catalog's `supports:` clause renders those lists directly. Nothing checks either, so an experiment record that gains a `supports` edge leaves the research row that should render it silently short.

**Measurement — seven discrepancies, found while landing the L8 qualification record and reproducible in one command.** The check below rebuilds both derivations from frontmatter and compares them against the rendered rows, naming its population (77 research records against 77 catalog rows, 31 experiment records against 28 experiment-catalog rows). At the commit that filed this ticket it reports:

| Row | Discrepancy |
| --- | --- |
| `docs/research/apple-targets/numerical-behaviour.md` | missing `BF16 through the second-dtype seams`, which `supports` it |
| `docs/research/numerics/mature-dtype-taxonomy.md` | no `experiments:` clause at all; `BF16 through the second-dtype seams` `supports` it |
| `docs/research/target-profiles/physical-feasibility-model.md` | no `experiments:` clause; `Bounded scalar CPU backend vertical` `supports` it |
| `docs/research/runtime/runtime-execution-contract.md` | missing `One inline region dispatched on Metal hardware` |
| `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md` | renders `Transformer reference semantics`; the record's title is `Transformer reference-semantics probe` |
| `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` | renders `Apple Metal target compatibility and numerical spikes`, which does **not** `supports` it — the only row where the rendered claim has no backing edge |
| `spikes/README.md`, the `BF16 through the second-dtype seams` row | missing `BF16 computation, accumulator, and conversion` from its `supports` list |

**Inference — six of the seven are mechanical and one is not.** The authority-ledger row is the only one where the fix is a judgement rather than a rendering: either the row's link is stale and is removed, or the spike genuinely supports the ledger and the missing `supports` edge is added to `spikes/apple-targets/README.md`. Decide it by reading what the ledger cites, not by picking the cheaper edit — and if the edge is added, that is a `research/apple-targets` change and needs the scope.

**Measurement — re-run at this branch's base `5f810e9a`, the population had grown to 83 research records against 83 rows and 33 experiment records against 36 rows, and the table above had drifted in three ways.** One row it named had been retitled: the runtime-execution-contract row is short of `Inline regions dispatched on Metal hardware`, not `One inline region dispatched on Metal hardware`. One row it did not name had appeared: `docs/research/runtime/dynamic-kv-physical-layout.md` carries no `experiments:` clause while `Dynamic KV physical-layout comparison` `supports` it. And the check **aborted** on the experiment catalog with a `KeyError: None` at `spikes/target-profiles/metal-grid-axis-extent/README.md`, so every experiment row after it went unexamined — including the one the ledger judgement lands on. Three rows point at READMEs carrying no governed experiment frontmatter; the check now reports and counts them as `UNGOVERNED` instead of aborting, and [`govern-the-three-ungoverned-spike-records`](govern-the-three-ungoverned-spike-records.md) owns repairing them. Two further gaps were symmetric to checks the run already performed and were repaired here: the experiment catalog was missing rows for three records it renders nowhere (`metal_transcendental_emission`, `transformer_reference_semantics`, `inline-dispatch`), and the check now asserts that direction as `MISSING experiment rows` exactly as it already did for the research catalog.

**Fact — the authority-ledger judgement resolved toward adding the edge, on what the ledger reads rather than on what it links.** The ledger's own evidence boundary states that "the grid axis, every numerical row, and every dispatchability row come from retained measurement directories", and the directory it then reads is `spikes/apple-targets/results/2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`: it transcribes that record's `environment.*` and `probe.*` keys, pins its `probe.harness_sha256 17b8b8dd…` and repository base revision, sources both dispatchability rows and all four subnormal rows from its `case.macos.*` keys, and its "Reproducible checks" section instructs the reader to `cd` into that same results directory. That is a direct reading of the spike's retained record, not a transitive route through the `apple-targets` research records the ledger also cites, so the rendered link was right and the missing `supports` edge was the defect. Removing the link would have been the cheaper edit and would have left the catalog asserting that a ledger built out of a retained measurement directory has no experiment reproducing it.

## The check, so the table above can be refuted rather than only believed

Run from the repository root. It counts what it examined, so a run that reached nothing is distinguishable from a run that found nothing.

```python
import re, pathlib, collections
root = pathlib.Path(".")
def fm(p):
    t = p.read_text(encoding="utf-8")
    return t.split("---", 2)[1] if t.startswith("---") else ""
def one(t, k):
    m = re.search(rf'^{k}:\s*"([^"]+)"', t, re.M); return m.group(1) if m else None
def many(t, k):
    m = re.search(rf'^{k}:\s*\[(.*?)\]', t, re.M | re.S)
    return re.findall(r'"([^"]+)"', m.group(1)) if m else []

spikes = {one(fm(p), "id"): (one(fm(p), "title"), many(fm(p), "supports"))
          for p in sorted(root.glob("spikes/**/README.md")) if 'kind: "experiment"' in fm(p)}
research = {one(fm(p), "id"): (one(fm(p), "title"), p)
            for p in sorted(root.glob("docs/research/**/*.md")) if 'kind: "research"' in fm(p)}
inv = collections.defaultdict(list)
for title, sup in spikes.values():
    for r in sup:
        inv[r].append(title)

cat = (root / "docs/research/README.md").read_text(encoding="utf-8")
rows = [l for l in cat.split("GENERATED RESEARCH CATALOG -->")[1].split("<!-- END")[0].split("\n")
        if l.startswith("- [")]
print(f"population: {len(rows)} research rows, {len(research)} research records, {len(spikes)} experiments")
bad = 0
seen = set()
for row in rows:
    title, rel = re.match(r"- \[([^\]]*)\]\(([^)]+)\)", row).groups()
    rid = one(fm(root / "docs/research" / rel), "id"); seen.add(rid)
    m = re.search(r"; experiments: (.*)$", row)
    got = sorted(re.findall(r"\[([^\]]+)\]\(", m.group(1))) if m else []
    if title != research[rid][0]:
        print(f"TITLE  {rel}: {title!r} != {research[rid][0]!r}"); bad += 1
    if got != sorted(inv.get(rid, [])):
        print(f"EXPMTS {rel}: row {got} != supports-inverse {sorted(inv.get(rid, []))}"); bad += 1
if set(research) - seen:
    print(f"MISSING rows for {sorted(set(research) - seen)}"); bad += 1

erows = [l for l in (root / "spikes/README.md").read_text(encoding="utf-8").split("\n")
         if l.startswith("- [") and "supports:" in l]
print(f"population: {len(erows)} experiment rows")
ungoverned, eseen = 0, set()
for row in erows:
    title, rel = re.match(r"- \[([^\]]*)\]\(([^)]+)\)", row).groups()
    sid = one(fm(root / "spikes" / rel), "id")
    if sid not in spikes:
        print(f"UNGOVERNED spikes/{rel}: row target carries no experiment frontmatter")
        bad += 1; ungoverned += 1; continue
    stitle, sup = spikes[sid]; eseen.add(sid)
    want = [research[r][0] for r in sup if r in research]
    got = re.findall(r"\[([^\]]+)\]\(", row.split("supports:")[1])
    if title != stitle:
        print(f"TITLE    spikes/{rel}: {title!r} != {stitle!r}"); bad += 1
    if [x for x in want if x not in got]:
        print(f"SUPPORTS spikes/{rel}: row {got} misses {[x for x in want if x not in got]}"); bad += 1
if set(spikes) - eseen:
    print(f"MISSING experiment rows for {sorted(set(spikes) - eseen)}"); bad += 1
print(f"evaluated: {len(erows) - ungoverned} experiment rows against {len(spikes)} records; {ungoverned} ungoverned")
print("DISCREPANCIES:", bad)
```

## Required work

- Re-run the check on the branch's own base rather than trusting this table; the corpus moves, and a row repaired meanwhile must not be "fixed" back.
- Repair the six mechanical rows so each rendered clause equals its frontmatter derivation.
- Resolve the authority-ledger row by reading the ledger and the spike, and say in the commit which way it went and on what evidence.
- Re-run the check and report the remaining count, which must be zero — or name each survivor and why it is not a defect.

## Explicit non-goals

No generator, no gate, and no schema change. The corpus deliberately keeps these catalogs as hand-maintained prose; the metadata contract already records why a transitive rendering was measured and rejected. This ticket repairs the rows, it does not automate them.

## Closes when

The check reports zero discrepancies over a named population apart from the three `UNGOVERNED` rows [`govern-the-three-ungoverned-spike-records`](govern-the-three-ungoverned-spike-records.md) owns, its failing perturbation has been watched (remove a row, drop a clause, mistype a title), and the authority-ledger row's resolution is recorded with the evidence that decided it.
