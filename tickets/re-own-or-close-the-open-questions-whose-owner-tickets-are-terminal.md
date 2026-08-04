---
id: re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal
title: Re-own or close the open questions whose owner tickets are terminal
status: review
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-oq-sweep
lease_expires_at: 1785878542
---
## User-visible outcome

Every question in `docs/open-questions.md` either has a live owner, a stated demand trigger, or is closed into the durable contract that answers it — so no question sits owned by a terminal ticket, which is ownership in name and orphanhood in fact.

## Why

**Fact — the failure mode has already happened once.** Q-ART-008 was owned by `prototype-artifact-family-delivery`, which closed `done` with the close condition unmet, leaving the question unowned until a later worker noticed. A 2026-07-31 audit (reproduce with the script in this ticket's provenance, or re-derive: extract each `Q-*` section's `tickets/*.md` references and compare against ticket statuses) found eight questions whose every referenced ticket is now `done` or `closed`: Q-SEM-001, Q-SEM-007, Q-PLAN-001, Q-PLAN-007, Q-PLAN-009, Q-ART-002, Q-PKG-002, and Q-PKG-003 — the last owned by `prototype-inline-proc-macro-frontend`, which closed the same day. Thirty-three further questions reference no ticket at all; most are deliberate demand-triggered reservations, but nothing distinguishes a stated trigger from an accidental orphan except reading.

## Work

For each of the eight: read the question against what its terminal owner actually delivered; close it into the contract that now answers it, re-point it at the live successor ticket, or record why it stays open with a stated trigger. For the thirty-three unreferenced: verify each states a closure trigger a reader can evaluate; give any that does not either a trigger or an owner. Do not close a question whose answer does not live in a durable contract or an accepted ADR.

## Closes when

The audit re-run reports zero questions owned solely by terminal tickets, and every unreferenced question carries an explicit trigger.

## Four named questions the stated Work would pass over (2026-08-01)

**Why this widening is needed rather than implied.** The Work above says of the thirty-three unreferenced questions: "verify each states a closure trigger a reader can evaluate." Four questions **satisfy that check and are nonetheless stale** — each states an evaluable trigger, and the trigger has fired, or its owner has gone terminal, or the evidence it waits on has already been supplied. This ticket's outcome could therefore be met with all four defects standing. They are named here so the sweep reaches them; each was verified against the tree at base `0017345`.

- **Q-ART-006 — rust-analyzer cold and warm expansion costs** (`docs/open-questions.md:247`). Its last open column is stated at `:256-258`: "What remains is the *edit* column: that needs a real language-server session rather than `analysis-stats`, which loads a project and expands once." That measurement was supplied at [`avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`](avoid-toolchain-resolution-on-a-warm-expansion-cache-hit.md):79-85 — a real LSP session (initialize, didOpen, then didChange edits each followed by a `textDocument/semanticTokens/full` round trip) under `rust-analyzer 1.97.0-nightly`, expansions counted exactly, in-region edits at 137–217 ms. That ticket is `done` and has **no Graph-maintenance section** (its headings are `Why this exists`, `Closes when`, `Outcome`), so nothing propagated the result. Close the question into the durable record that now carries it, or restate the remainder.
- **Q-SEM-004 — First-profile transcendental tuples** (`:97`). Both reasons the question gives at `:102` for staying open were discharged on 2026-08-01. "Adopting the `exp` bound needs a registered cross-metric implication because Apple's ULP definition is a different key" — that implication is registered, as `RegisteredImplication::ScaledMetric` at `crates/tiler-compiler/src/target/accuracy.rs:139` with its derivation attached. "Adopting any correctly rounded entry needs the rounding mode Metal's §8.2 declines to fix" — `docs/roadmap.md:408` records the observation that Gap 4's rounding-mode question does not bind an entry stated as a ULP bound, and that a faithful contract is metric-free. What is genuinely still open is the **reference half**, which the question itself calls "wholly open". Restate the remainder as that and give it an owner; do not close it on the backend half alone.
- **Q-PLAN-011 — CPU execution and vector profile** (`:331`). Its trigger at `:334` is "the CPU backend enters the active roadmap", and it sits under a deferred-until-an-explicit-trigger heading. The trigger fired: [`prototype-a-bounded-scalar-cpu-backend-vertical`](prototype-a-bounded-scalar-cpu-backend-vertical.md) is `done`, ADR 0093's CPU vector-lane tier is accepted, and three implementation tickets are filed against it. Shared with [`sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired`](sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired.md), which names it among its starters — coordinate, and do not both make the edit.
- **Q-SEM-015 — Tensor contraction** (`:298`). Its Owner/tracking line at `:300` names [`scope-einsum-contraction-support`](scope-einsum-contraction-support.md), which is `done` — the exact terminal-owner pattern this ticket exists for, missed by the original audit because the line also names the Milestone 6 framing and so does not read as unowned. Its trigger bullet at `:301` reserves a contraction choice that had no node until now; repoint that clause at [`decide-whether-a-contraction-may-consume-more-than-two-operands`](decide-whether-a-contraction-may-consume-more-than-two-operands.md). The third reserved choice in the same bullet was decided on 2026-08-01 — declined, recorded on [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) — so `:301`'s description of it as an open choice needs correcting too.

**The check this adds to the closing condition.** A question passes only if a reader can evaluate its trigger *and* the trigger has not already fired. "States an evaluable trigger" was the original bar and all four of these clear it; evaluating each trigger against the tree is what this widening requires.

## Outcome (2026-08-04)

The audit was re-derived from the tree rather than taken from the 2026-07-31 list, and the list was **wrong in both directions**. It named eight; the tree had ten, because two more went terminal after the audit was written — `prototype-candle-metal-adapter` closed under Q-RUNTIME-002, and `decide-the-expansion-cache-collection-schedule` closed under Q-ART-004, which had itself been retargeted onto that ticket precisely because its previous owner went terminal. Q-SEM-015 is the eleventh and was caught by the 2026-08-01 widening rather than by either audit, for the reason that widening gives: its owner line also names the Milestone 6 framing, so it does not read as unowned.

**Ten of the eleven closed or re-owned; the eleventh, Q-SEM-015, needed only its owner line repointed.** Six questions closed into durable authorities and five stayed open with a live owner or a restated trigger.

### Per-question disposition

| Question | Disposition | Ground |
| --- | --- | --- |
| Q-SEM-001 numerical-policy presets | **closed** into [numerical semantics](../docs/numerical-semantics.md) | Closed by *supersession*: the four-value preset enumeration was eliminated 2026-08-01, so the "preset-to-canonical expansion table" has no subject. What replaced it is stronger — eleven governed dimensions, a declaration keyed by dimension *and* scalar-arithmetic subject, and a contract key that is the canonical injective encoding of the dimension vector under `tiler.contract.f32.v2`. Round-trip = that injectivity, checked exhaustively rather than sampled; rejection = `RequestError::NoResolvableNumericalContract`. |
| Q-PLAN-001 initial bounded search representation | **closed** into [the optimizer contract](../docs/compiler/optimizer.md) | Both halves are in the contract: `EnumerateRegionCandidates` is general over an arbitrary verified DAG and **is checked against an exhaustive subset oracle**, and the memo half is a standing reservation with the contract requiring plan quality to be measured against the tiny-graph oracle *before* a memo architecture is chosen. General memo search and partitioning are Q-PLAN-002/Q-PLAN-005's. |
| Q-ART-002 private lockstep serialization | **closed** into [the artifact contract](../docs/artifact-abi.md) | Its "Implemented envelope profile" section carries the deterministic encoder/decoder, the reversed-declaration-order byte-identity measurement, non-canonical refusal by re-encode-and-compare, version stepping, and the typed rejection vocabulary. The layout staying `pub(crate)` is the question's own "does not promise a public stable format" clause holding, not a gap. |
| Q-ART-004 expansion-cache root, accounting, GC | **closed**, both halves, into [ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) and [the frontend contract](../docs/integration/frontends.md) | Root half closed 2026-07-31. Collection half closed 2026-08-04 on Tom's decision — automatic eviction, env-configured, no maintenance command — with the schedule, the `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE` spelling, the deliberate non-surfacing of the report, and the measured race behaviour at 1/8/32 writer processes all in the contract. The size ceiling is a *decision to exclude* with its own triggers, not an unowned gap. |
| Q-ART-006 rust-analyzer cold/warm costs | **closed** into [the frontend contract](../docs/integration/frontends.md) | The edit column the entry was waiting on was supplied 2026-08-01 over a real LSP session by [`avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`](avoid-toolchain-resolution-on-a-warm-expansion-cache-hit.md), which carried **no graph-maintenance section**, so nothing propagated it — exactly the failure this ticket exists for. The one absent cell, the cold *interactive* round trip, is parked in that contract with its own trigger. |
| Q-PKG-003 proc-macro to Metal-AOT visibility | **closed** into [ADR 0088](../docs/decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) | The `tiler-macros` → `tiler-metal-aot` edge is accepted and the facade is forbidden one, so the driver never enters a consumer's build graph. The audit is mechanical — `crates/tiler/tests/dependency_direction.rs` reads what Cargo resolved and names its population first — and the compile/UI half is `crates/tiler/tests/facade/`. |
| Q-SEM-007 transactional rewrite API | **re-pointed + trigger restated** | The engine half is delivered and every named property landed; the contract states them normatively. What is open is a *public boundary*: `crates/tiler-compiler/src/lib.rs` declares `mod rewrite;` without `pub`, so there is no API. Trigger: the first rule provider that must live outside `tiler-compiler`; closure is Tom's under ADR 0075. The one non-boundary residue — CSE's stage-owned explain constants carrying no provider identity — is recorded rather than dropped. |
| Q-PLAN-007 first Metal capability keys | **re-pointed** at four live tickets | Both prototypes said in their own outcomes that they implement one private named fixture without closing the mature profile. The live rows are `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` (the current bound is a representability *floor* that a retained sweep measures as collapsing the parallel-reduction comparable domain to one shape), `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`, `declare-metal-subgroup-realization-facts-in-the-target-profile`, and `reconcile-the-operation-identity-and-governed-key-grammars`. |
| Q-PLAN-009 capability providers and phases | **re-pointed** at five live tickets | Phases: `name-a-host-process-availability-phase`. Providers: `expose-explicit-backend-provider-and-selection-policy-composition`, `drive-an-external-physical-implementation-provider-through-compilation`, `disclose-offered-and-selected-physical-provider-sets-separately` (no physical provider's identity reaches explain output at all today), `resolve-or-retire-the-scalar-lowering-provider-seam`, and `publish-the-backend-provider-conformance-suite`. |
| Q-RUNTIME-002 affine-strided Candle layouts | **trigger restated** | No live ticket owns it and that is correct rather than an omission: the boundary is *delivered*, with `AffineStridedLayout` and `BroadcastView` refused by name and a nonzero-offset contiguous view accepted as the positive control. Trigger: a selected region whose Candle operand is non-contiguous and for which falling back to Candle's own kernels is unavailable or unacceptable. The `is_contiguous`-ignores-extent-1-strides finding is kept beside it, because the first probe written for this tested the wrong refusal. |
| Q-PKG-002 Rust data APIs and capability traits | **re-pointed** at four live nodes | [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) settled *intent* on 2026-07-25; what stays open is promotion, which ADR 0074 convention 7 and ADR 0075 make Tom's per facade. Live: `accept-the-public-compiler-facade-boundary`, `accept-the-public-route-requirement-answer-boundary`, `resolve-or-retire-the-scalar-lowering-provider-seam` (ADR 0078's own remaining open question), and `audit-dead-code-admissions-after-public-boundary-promotions`. |
| Q-SEM-015 tensor contraction | **owner line repointed** | `scope-einsum-contraction-support` filed the question and closed `done` the moment it existed — correct for a scoping ticket, wrong as a standing owner. The framing is the live authority; `decide-whether-a-contraction-may-consume-more-than-two-operands` now owns the one reserved semantic choice, linked at the clause that reserves it. Its body already carried ADR 0095's decline correctly, so the widening's third correction was already applied. |

### The fired-trigger four from the 2026-08-01 widening

- **Q-ART-006** — closed, above. The widening was right that the edit column was supplied and that nothing propagated it.
- **Q-SEM-004** — restated, and the widening's own framing was corrected by reading. Both stated reasons are discharged as it says. But the widening's claim that "the reference half remains wholly open" **does not survive checking**: `crates/tiler-reference/src/accuracy.rs` supplies the certified enclosures and the three-way conformance decision, and three families are reference-evaluated against exact rational enclosures for their subordinate transcendentals. The accurate remainder is the *tuple selection* this question always named — a general `Exp`, `Log`, `Sin`, or `Gelu` key has no reference evaluator for the same reason it has no backend row: it is not registered, and three separate tests hold that line. Tracking is the matrix's transcendental row, which states the same thing from the delivery side.
- **Q-PLAN-011** — trigger fired, question **moved out of the deferred section** into the milestone-owned contracts, given three live implementation owners and a `Close:` clause it did not have. The spike's own graph-maintenance note recorded `docs/open-questions.md` as deliberately unchanged, which is why the entry stood after its trigger was spent. Accepted-model-versus-implemented is kept explicit: ADR 0093 accepts seven decisions and not one line of public Rust, thirteen public-boundary items return to Tom, and [the CPU backend contract](../docs/backends/cpu.md) stays proposed.
- **Q-SEM-015** — owner line repointed, above.

### The two the widening did not name, found by evaluating every trigger

- **Q-SHAPE-007 indirect gather/scatter — trigger FIRED, re-owned.** [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) (`todo`, p1) states its own trigger as active now, and already enumerates this question's four closure items. The evidence is the pinned workload's *first* operation: with no admitted access class it is not expressible at all. The **scatter half has not fired** and is kept separate, so the entry stays in the deferred section with one half re-owned rather than being wholly promoted. `docs/roadmap.md`'s "Gather and scatter stay out until Q-SHAPE-007 triggers" was corrected in the same change, because leaving it would have had two authorities disagreeing about whether the trigger fired.
- **Q-RUNTIME-001 Candle arity — trigger sharpened, not fired.** As written it is already literally satisfied: the first complete-model program needs eighteen inputs and three outputs and no partitioning fixes it. But that record answers it by *not choosing Candle* and deliberately files nothing to move the arity, so the trigger now reads on a region selected to run **through the Candle wrapper**. A region that exceeds the arity and is routed elsewhere does not fire it.

### The other unreferenced questions

Every remaining trigger was evaluated against the tree and is unfired. Three were sharpened rather than left as-is, because each was evaluable only after work a reader should not have to redo: **Q-SHAPE-006** now records that a contraction needs no piecewise map and that the sub-tensor and rotary families are blocked on a *carrier* gap rather than an expressiveness class, and names the one live piecewise pressure — concatenate lowering needs a piecewise read *or* two write roots, and the second is available, so the trigger has not fired; **Q-ART-003** now carries the measured headroom (47,803 bytes, 4.56% of the per-invocation ceiling) and the fact that an Apple artifact family is not a delivery platform in its sense, so iOS is Q-ART-011's axis; **Q-RUNTIME-001** as above. Q-SEM-011 is unfired and the reason is load-bearing rather than incidental — the 2026-08-04 KV supersession made the retained tensors *ordinary* program inputs and outputs, and `OperationEffect` still has one variant, `Pure`.

### Also corrected

The implementation-graph list under "Milestone-owned implementation contracts" mapped those contracts to thirty bounded coding tickets, **every one of which is now `done`**. Presented as a live mapping it is the same failure mode this ticket exists for at list scale, so the list is now stated as a record of what landed, with the live graph named as the ticket board and the roadmap's support matrix.

### The audit, and proof it can say no

Encoded as the file's own stated rule — "Each question has one owner and an explicit way to close or reconsider it" — rather than as a proxy for it. A `### Q-*` section is bounded at the next heading of any level **or at a closure record**, a paragraph opening `**Q-<id> —`, because a closure record is not part of the open question above it. Three checks: `DEAD-POINTER` (names tickets, all terminal, states no `Trigger:`-class clause — a `Close:` clause does not rescue it, because `Close:` says what closure looks like and never what causes the work); `NO-OWNER` (no live ticket and no owning contract link); `NO-CLOSURE` (no `Close`/`Trigger`/`Run when`/`Closing measurement` clause).

**A terminal ticket cited beside a live one or beside a stated trigger is history, not a pointer, and correctly does not fail.** Seven entries cite a terminal ticket after this sweep and every one is a retarget note.

Results, same script both sides:

| tree | open questions | DEAD-POINTER | NO-OWNER | NO-CLOSURE | exit |
| --- | ---: | ---: | ---: | ---: | ---: |
| base `6447a901` | 50 | **10** | 0 | 1 | 1 |
| after this ticket | 44 | **0** | 0 | 0 | 0 |

The ten at base are exactly Q-SEM-001, Q-SEM-007, Q-PLAN-001, Q-PLAN-007, Q-PLAN-009, Q-ART-002, Q-ART-004, Q-RUNTIME-002, Q-PKG-002, and Q-PKG-003. Q-SEM-015 is not among them and is not detectable by this check, because its section names live tickets elsewhere in its body — which is the limitation to know about this audit, and the reason the 2026-08-01 widening's per-question reading was necessary rather than redundant.

**Proved able to fail before being trusted.** A synthetic entry citing only `prototype-metal-runtime-proof` (`done`) with a `Close:` clause and no trigger was appended, the audit reported `FAIL Q-SYNTH-999 DEAD-POINTER` and exited 1, and the entry was removed; the audit then reported zero failures and exited 0, with `git diff --stat` unchanged across the perturbation.

### Reproducing the audit

Save as `/tmp/oq_audit.py` and run `python3 /tmp/oq_audit.py <repo-root>`. It is deliberately not checked in: nothing in this repository renders or validates the docs corpus, no `make` target reaches it, and a checked-in script that no gate runs is a claim that something is being checked when nothing is.

```python
import re, sys, pathlib
root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
oq = (root / "docs/open-questions.md").read_text()
tickets = root / "tickets"
TERMINAL = {"done", "closed"}
TICKET_REF = re.compile(r"\.\./tickets/([A-Za-z0-9_.-]+)\.md")
DOC_LINK = re.compile(r"\]\((?!\.\./tickets/)([A-Za-z0-9_./-]+\.md)[)#]")
TRIGGER = re.compile(r"(^|\s)\*{0,2}(Trigger|Run when|Closing measurement)\*{0,2}[,:]", re.M)
CLOSE = re.compile(r"(^|\s)\*{0,2}(Close|Closes|Close when)\*{0,2}[,:]", re.M)
CLOSURE_RECORD = re.compile(r"^\*\*Q-[A-Z0-9-]+ —")
_cache = {}

def status_of(name):
    if name not in _cache:
        p = tickets / f"{name}.md"
        st = "MISSING"
        if p.exists():
            st = "NO-STATUS"
            for line in p.read_text().splitlines()[:40]:
                m = re.match(r"^status:\s*(\S+)", line)
                if m:
                    st = m.group(1); break
        _cache[name] = st
    return _cache[name]

lines = oq.splitlines()
stops = [i for i, ln in enumerate(lines)
         if re.match(r"^#{1,6} ", ln) or CLOSURE_RECORD.match(ln)]
stops.append(len(lines))
rows = []
for a, b in zip(stops, stops[1:]):
    m = re.match(r"^### (Q-[A-Z0-9-]+)", lines[a])
    if not m:
        continue
    qid, body = m.group(1), "\n".join(lines[a + 1:b])
    refs = sorted(set(TICKET_REF.findall(body)))
    live = [r for r in refs if status_of(r) not in TERMINAL]
    docs = sorted(set(DOC_LINK.findall(body)))
    trig, clos = bool(TRIGGER.search(body)), bool(CLOSE.search(body))
    faults = []
    if refs and not live and not trig: faults.append("DEAD-POINTER")
    if not live and not docs:          faults.append("NO-OWNER")
    if not (trig or clos):             faults.append("NO-CLOSURE")
    rows.append((qid, faults, refs, live, docs, trig, clos))

bad = [r for r in rows if r[1]]
for qid, faults, refs, live, docs, trig, clos in bad:
    print(f"FAIL {qid:14s} {','.join(faults)}")
    print(f"       live: {live or 'none'} | docs: {docs or 'none'} | Trigger: {trig} | Close: {clos}")
    for r in refs:
        if status_of(r) in TERMINAL:
            print(f"       terminal ref: {status_of(r):8s} {r}")
print(f"\ntotal open questions: {len(rows)}\nfailures: {len(bad)}")
sys.exit(1 if bad else 0)
```

Run it against an older tree with `git archive <rev> docs/open-questions.md tickets | tar -x -C "$(mktemp -d)"`, which is how the base column above was produced.

### Filed, not absorbed

[`call-the-expansion-cache-preflight-on-the-resolved-root`](call-the-expansion-cache-preflight-on-the-resolved-root.md) (p3). Q-ART-004's root half carried a residue in its own body — `ExpansionCache::preflight` is never called — and closing the question would have dropped it. `crates/tiler-macros/src/cache_root.rs:374` cites the report in a doc comment and that doc comment is the only occurrence of the word in the crate: `grep -rn preflight crates/tiler-macros/src/` returns one line and no call site. A diagnostic gap, not a correctness one, since the protocol already fails closed per operation.

### Found while running this, not fixed here

- **`docs/compiler/optimizer.md:548-552` names three `done` tickets as "staged future work"** — `prototype-region-cover-enumeration`, `prototype-physical-implementation-frontier`, and `prototype-complete-physical-plan-selection`. `contracts/optimizer` is outside this ticket's scopes. Reproduce by reading the sentence beginning "Goal-directed property search over those candidates is the staged future work".
- **`docs/integration/frontends.md:150` says the generated-code shape "is illustrative and not delivered: no expansion emits any of it today"**, while `:166` states the generated paths that *are* emitted and `generate-cfg-gated-artifact-family-delivery` and `prototype-macro-embedding-and-cargo-behavior` both landed embedding. `contracts/integrations` is outside this ticket's scopes; [`correct-two-stale-delivery-spans-in-the-frontends-contract`](correct-two-stale-delivery-spans-in-the-frontends-contract.md) already owns that file and is the natural home.
- **`crates/tiler-cache/src/expansion/collect.rs:97-99`** was reported stale by `decide-the-expansion-cache-collection-schedule` and is not this ticket's scope either; recorded here only so the two reports are not read as different findings.
