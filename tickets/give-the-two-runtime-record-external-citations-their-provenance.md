---
id: give-the-two-runtime-record-external-citations-their-provenance
title: Give the two runtime-record external citations their provenance
status: done
priority: p2
dependencies: []
related: [extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## What the docs citation check surfaced

`check-citations.sh` gained a `docs/**` population under `extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation`. It reports **two failures in `docs/research/runtime/backend-scoped-route-requirement-answers.md`**, both in the same bullet, and both the same defect: a citation into a source tree that is not this one, spelled as though it were rooted here.

```
FAIL  docs/research/runtime/backend-scoped-route-requirement-answers.md
        citation: `src/device.rs:74-82`
        no file in the tree is or ends with src/device.rs
FAIL  docs/research/runtime/backend-scoped-route-requirement-answers.md
        citation: `MTLDevice.h:242`
        no file in the tree is or ends with MTLDevice.h
```

**Fact — neither is Tiler drift; both are real lines in real external sources.** `src/device.rs` at lines 74–82 is `metal` 0.33.0, cited in the sentence that reads "`metal` 0.33.0 transcribes `Apple1` through `Apple9` as `#[repr(i64)]`". `MTLDevice.h` at line 242 is the installed macOS SDK header, cited in the 2026-08-01 correction that reads "It is that header line in the same SDK."

**How this ticket spells the two extents, and why.** As a bare path plus a prose line number, never pinned as `path:LINE` — the same convention a dated correction uses when it retires a citation, and the reason a bare path carrying no pin is deliberately not checked. A ticket that pinned the broken form would fail the very check it is asking someone to satisfy; this one did, on its first run, before this paragraph existed. The verbatim failing spellings are preserved in the fenced block above, which the checker skips.

**Fact — the checker cannot skip either one, and the reason is deliberate.** It skips a path rooted outside this tree only when the path has a `/` *and* its leading segment is a component of no tracked path — `candle-core/...` and `candle-metal-kernels/...` are skipped on exactly that test. `src` is a component of hundreds of tracked paths, and `MTLDevice.h` has no `/` at all. Widening the rule to cover a bare filename was considered and refused in that ticket: a bare filename is this repository's own shorthand for its own files (306 citations resolve by unique suffix that way), so treating an unresolvable one as external would silently stop reporting the drift the check exists for.

## The repair

Spell each with the provenance it already has in prose, so the path is rooted in the project it names. Both forms are already used in this repository:

- `metal-0.33.0/src/device.rs:74-82` — the version-pinned crate-source form the checker has always recognized (`objc2-metal-0.3.2/src/generated/MTLDevice.rs:238` is the standing example).
- For the SDK header, **this same document already writes the qualified form two hundred lines earlier**, at the "raw constant is a primary-source value" fact: `$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h:233-242`. Match it, or use a leading segment that is a component of nothing here.

Nothing either bullet asserts changes. Re-read both sources at your own base before editing; do not repair a citation by deleting the claim it supports.

## Closes when

`./check-citations.sh` reports no failure in `docs/research/runtime/`, and each repaired citation names the revision or SDK it is about.
