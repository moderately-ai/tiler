---
id: correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier
title: Correct the architecture citation that drops the inline frontend qualifier
status: todo
priority: p1
dependencies: []
related: [repair-the-eight-dangling-links-in-the-runtime-route-answer-record, accept-the-public-route-requirement-answer-boundary]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [citations, contracts, documentation]
---

`docs/research/runtime/backend-scoped-route-requirement-answers.md` cites `docs/architecture.md` for a sentence that document does not contain, dropping the qualifier the sentence turns on. **This is the failure class the citation checker cannot catch**: the pin resolves, so nothing goes red, and the claim is wrong anyway.

## Facts, verified 2026-08-08 by the coordinator at `bdbeb2b5`

**Fact.** The record asserts **twice** that `docs/architecture.md` states *"`tiler` is the one crate a consumer names."* `docs/architecture.md` says *"the one crate an **inline-frontend** consumer names."* Reproduce with whitespace collapsed, because both files hard-wrap and a line-oriented grep finds neither:

```sh
tr '\n' ' ' < docs/research/runtime/backend-scoped-route-requirement-answers.md | tr -s ' ' | grep -o "the one crate a[^\"]\{0,40\}consumer names" | sort | uniq -c
# 2 the one crate a consumer names
tr '\n' ' ' < docs/architecture.md | tr -s ' ' | grep -o "the one crate a[^\"]\{0,40\}consumer names" | sort | uniq -c
# 1 the one crate an inline-frontend consumer names
```

**Fact.** The qualifier is load-bearing, not decorative. The same paragraph in `docs/architecture.md` explicitly carves out consumers constructing arbitrary semantic programs, denying that `tiler` is the facade for them. The unqualified sentence states a flat monopoly the architecture document specifically refuses.

**Fact.** The citation is pinned to `docs/architecture.md:389`, and `sed -n '389p'` returns an **empty line** at this base. So the pin is stale by number as well as wrong by content — and the checker still passes it, because partial-path resolution finds the file.

**Inference — why this is p1 and not a typo.** ADR 0092 decision item 6 rests on this sentence, and `accept-the-public-route-requirement-answer-boundary` holds a reserved item that turns on whether a **dispatching** consumer may name `tiler-metal` — a distinction the unqualified quote erases entirely. A reader reaching the misquote concludes the boundary question is already answered. It is not.

## What closes this

Both occurrences quoting `docs/architecture.md` accurately, with the qualifier, cited by searchable anchor rather than by line number — the line pin is exactly what rotted here. Where the record's argument depended on the flat reading, say what the qualified sentence does and does not support; do not quietly re-attach the qualifier to a sentence whose conclusion needed the flat version.

**Do not treat this as the only instance.** It survived because it reads plausibly and resolves cleanly. Grep the record — and its siblings — for other quotations attributed to `docs/architecture.md` and verify each against the source with whitespace collapsed. Report the census either way, so "no others" is distinguishable from "did not look".
