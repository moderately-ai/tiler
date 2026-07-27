---
id: stop-copying-bytes-the-process-already-owns
title: Stop copying bytes the process already owns
status: in-progress
priority: p2
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/cache, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [performance, cache, artifact]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785180316
---
Copies that exist only because a borrow was available and not taken. Paid on every cache hit.

## Facts

**The cache double-buffers every hit.** `store.rs:601-602`:

```rust
subject: view.subject.to_vec(),
envelope: view.envelope.to_vec(),
```

`bytes` from `read_bounded` is an owned `Vec<u8>` whose contents, minus a 64-byte header and two 52-byte descriptors, *are* `subject ++ envelope` contiguously (`bundle.rs:194-196` writes them back to back and `bundle.rs:364-370` proves contiguity on read). It is then dropped. This duplicates the whole bundle to split one owned buffer into two.

`CachedEntry::subject()` and `envelope_bytes()` already return `&[u8]`, so holding the buffer plus two `Range<usize>` changes **no public API**.

**`decode.rs:209` copies every section** with `content.to_vec()`, though `Cursor::take` (`decode.rs:923`) already returns a borrow with the input lifetime. For a payload-carrying envelope this copies most of the bytes.

**`read_bounded` does not pre-size** (`store.rs:611`) — `Vec::new()` wrapped in `io::Take`, which also defeats `File`'s read-to-end size specialisation. Roughly 12-14 reallocations for a 26 KB bundle. The file's metadata and the bundle's own declared length are both available, and the existing `limit` already bounds it.

**`AdmittedImplementation` is deep-cloned per region per combination** (`selection.rs:1069-1078`) inside the loop bounded by `physical_plan_combinations` = 4,096 per cover — and each clone drags a whole `VerifiedRequestSubject`.

Whole covers are cloned into `sources` (`pipeline.rs:1092`) though the enumeration already owns them and outlives it.

## Closes when

A cache hit copies the bundle once rather than twice; sections are borrowed rather than copied where the lifetime allows; `read_bounded` pre-sizes within its existing bound; the hot clones above are removed or justified in place; measured before and after; `make full` passes.
