---
id: replace-the-codec-arena-content-key-with-the-existing-comparator
title: Replace the codec arena content key with the existing comparator
status: done
priority: p1
dependencies: []
related: [measure-artifact-decoder-allocation-amplification]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets, research/artifacts, contracts/navigation]
paths: []
tags: [artifact, codec, performance, security]
---
`decode_artifact` allocates a peak of **1,569,620,906 bytes** while validating a
**226,214-byte** envelope, and a forged envelope that will be *rejected*
allocates exactly the same. That is 6,939x amplification from bytes a producer —
or an attacker — chooses. Every consumer that decodes artifact bytes it did not
produce is exposed, the expansion cache validating a stored bundle among them.

Measured in
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md)
and reported in
[the research note](../docs/research/artifacts/decoder-allocation-amplification.md);
`measure-artifact-decoder-allocation-amplification` found it and could not fix
it inside its scopes.

## The cause, exactly

`super::model::expression_keys` derives one canonical content key per ABI arena
node with `tiler_ir::program::abi::expr_key`, which frames each operand's
**whole key** inside its node's key. A chain of depth `d` carries a key linear in
`d`, so an arena of `d` such nodes carries key bytes quadratic in `d`. Peak live
during a decode, no object carried, measured:

| Chain nodes | Envelope bytes | Peak live | x envelope |
| --- | --- | --- | --- |
| 0 | 114,083 | 283,005 | 2.5 |
| 128 | 117,798 | 1,770,867 | 15.0 |
| 512 | 128,550 | 25,997,811 | 202.2 |
| 1,024 | 142,886 | 103,258,099 | 722.7 |
| 2,048 | 171,558 | 411,919,347 | 2,401.1 |
| 4,000 | 226,214 | 1,569,620,906 | 6,938.7 |

Each doubling of the chain multiplies the peak by four. The last row sits at
`MAX_ABI_EXPRESSIONS`, so it is the governed bound measured rather than
extrapolated to.

## The fix already exists and is public

`tiler_ir::program::abi::compare_expr_nodes` is a total, content-derived order
that needs no numbering and is exactly injective, and its own documentation
gives this exact reason for existing: "Materializing a key per node embeds that
node's whole subtree, which is quadratic on a chain ... A comparison walks both
subtrees and stops at the first difference, so it never materializes one."

The identity encoder already uses it, through `canonical_arena_traversal`. The
codec does not, so the crate carries **two definitions of canonical arena order**
that only happen to agree. The three key-based sites are
`codec::model::canonical_expression_order`, the precondition sort in
`codec::model::project_entries`, and the duplicate check in
`codec::decode::parse_expressions`; `codec::validate` reads the same table for
its launch-precondition order check.

## Why it is a schema step rather than a refactor

The two orders are **not** the same relation. `expr_key` frames each operand with
an eight-byte length prefix, so comparing two keys compares operand *lengths*
before operand content, while `compare_expr_nodes` compares structure directly.
Switching therefore changes which byte string is *the* canonical encoding of a
given artifact:

- `MANIFEST_SCHEMA` takes a **major** step, for the reason every earlier one did.
- Every pinned or golden envelope byte in the workspace is rebaselined on the
  merged tree — `tiler-cache`, `tiler-macros`, and the prototypes hold some.
- **Artifact identity does not move.** `encode_identity` numbers the arena
  through `canonical_arena_traversal`, which is invariant to arena permutation,
  and `variant_order` already derives precondition order with the comparator.
  Confirm this on the merged tree rather than taking it from here.

Deleting `codec::model::expression_keys` is the check that the change is
complete: nothing may re-derive a key table on the decode path.

## Closes when

The codec orders and deduplicates arena nodes through `compare_expr_nodes`,
`expression_keys` is gone from the decode path, `MANIFEST_SCHEMA` states the
step with its reason, every rebaselined pin is recomputed on the merged tree, the
spike's arena rows are re-run and recorded beside the retained ones, artifact
identity is confirmed unmoved, and `make full` passes.

## Confirm the forger reach, or record that it is not reachable

The retained rows prove a *producer* can impose the cost and that a decode which
ends in rejection pays it in full. What they do not prove is that a manifest
carrying the chain can be forged from bytes alone. `parse_expressions` runs
inside `parse_manifest`, before any identity check, so the reading is that it
can; confirming it needs one hand-built manifest with a repaired manifest digest.
Do that before sizing the fix, because a producer-only cost and an
attacker-reachable one are different priorities.

## Outcome — 2026-08-06

Landed at `42f0051ff38e3f9afc33a37cd004d02dca9be01e` on
`tkt/replace-the-codec-arena-content-key-with-the-existing-comparator`, from base
`b3d5a9ed9ded4e2348c61e5904dae9f97a789cfb`.

### The forger reach is confirmed, so this was an attacker-reachable cost

Done first, as the ticket asked. `decode` reads the fixed framing header, compares
one digest over the manifest bytes, and calls `parse_manifest`, whose eleventh
statement is `parse_expressions`; `validate` and the identity comparison both run
after `parse_manifest` has returned. A forger holding only bytes recomputes the
one digest in between.

Confirmed rather than read off the source.
`a_forged_manifest_reaches_the_arena_parser_before_any_identity_check`
(`crates/tiler-artifact/src/program/codec/tests.rs`) takes the ordinary fixture's
encoded bytes, splices a 512-node chain into the manifest in place of the arena
run the fixture wrote, repairs the manifest length, the total length, and the
manifest digest, and changes nothing else. The decode is refused by
`ModelObligation { cause: UnusedExpression }` — raised in `validate`, which is
what proves the whole forged chain was parsed, type-checked, proven distinct, and
proven canonically ordered before anything refused it. Watched failing under one
perturbation: dropping the digest repair reports `ManifestDigestMismatch`, the
shallow refusal the case exists to get past.

So the severity in this ticket's opening paragraph stands as written: ~226 KB of
attacker-chosen bytes made a consumer allocate 1.5 GB before refusing them.

### What landed

`MANIFEST_SCHEMA` is `(14, 0)`. Three key-based sites now reach
`tiler_ir::program::abi::compare_expr_nodes`:

- `codec::model::canonical_expression_order` takes only `&[ExprNode]` and orders
  its ready set through a `ReadyNode` wrapper whose `Ord` is the comparator, with
  the arena position as the tie-break the comparator cannot supply.
- `codec::model::project_entries` no longer sorts preconditions locally at all —
  it calls `super::super::model::canonical_precondition_order`, the same function
  `variant_order` reaches when identity folds them, so the stored order and the
  folded order are one definition.
- `codec::validate` checks the launch-precondition order through a new
  `check_ordered_nodes`, adjacent pairs under the comparator, with no key table.

`codec::model::expression_keys` is **deleted**, and so is every table that fed it:
`ArtifactProgramData::expression_keys` and `ArtifactProgramBuilder::expression_keys`
had no reader left, so the producer stopped deriving quadratic key bytes too.
`expr_key` is no longer imported by `tiler-artifact` outside one test.

`decode::parse_expressions` recognizes a duplicate by **shallow structural
equality over a `HashSet<&ExprNode>`** rather than by comparing keys or by an
`O(n log n)` comparator sort. That is exact, and the induction is written out at
the function: the operand check above it has already proven every operand
strictly precedes its node, so on a duplicate-free prefix two nodes denote the
same expression exactly when they are equal as stored records. It is the same
recognizer `ArtifactProgramBuilder::push_node` uses, now stated from both sides of
the wire, and it is linear rather than quadratic — which matters, because a
comparator sort here would have replaced 1.5 GB of allocation with an
attacker-chosen `O(n log n · depth)` walk.

### Artifact identity did not move — confirmed on this tree, not taken from here

Two independent pieces of evidence, both from the full workspace run:

- `tiler-build`'s `the_standard_metal_path_publishes_its_recorded_identities`
  passes with `ARTIFACT_IDENTITY` and `CACHE_SUBJECT` unchanged. Those are the
  workspace's pinned artifact identity and expansion-cache subject.
- A new case, `permuting_the_arena_moves_the_envelope_bytes_and_not_its_identity`,
  permutes the default fixture's arena, rewrites its references, and asserts the
  identity is equal while the encoding is not. Watched failing under one
  perturbation: asserting the two encodings *equal* fails, which proves the
  permutation reached the wire rather than being normalized away.

### Zero moved pins

No pin in the workspace was rebaselined, and that is a measured result rather than
an absence of searching. The spike re-run reports the **same envelope byte length
in all 93 rows** at both schemas: the two orders agree on every arena these
fixtures build, so the wire's permission to move went unused. `cargo nextest run
--workspace` is green at 2,865 tests with no golden edited. The ticket's
expectation that `tiler-cache`, `tiler-macros`, and the prototypes hold pinned
envelope bytes was checked and is not the case — those crates compute envelope
bytes and compare them to each other, and the only byte-level pins in the
workspace are `tiler-build`'s two identity digests, which are identity rather than
wire and did not move.

### Measured, spike re-run

`spikes/artifacts/decoder-allocation`, run manually from its own directory per its
README, recorded as
`results/decoder-allocation-macos-27.0-2026-08-06-comparator.tsv`:

```sh
cd spikes/artifacts/decoder-allocation
cargo build --release
./target/release/artifact-decoder-allocation --record macos-27.0-2026-08-06-comparator
```

Same environment as the retained runs: Apple M4 Max (`Mac16,6`), 14 logical cores,
macOS 27.0 build `26A5388g`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`,
release profile.

Peak live during `decode_artifact`, no object carried:

| Chain nodes | Envelope | `13.0` peak | ×env | `14.0` peak | ×env | Reduction |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 114,083 | 283,005 | 2.48 | 283,005 | 2.48 | — |
| 128 | 117,798 | 1,770,867 | 15.0 | 299,746 | 2.54 | 5.9× |
| 512 | 128,550 | 25,997,811 | 202.2 | 324,036 | 2.52 | 80.2× |
| 1,024 | 142,886 | 103,258,099 | 722.7 | 429,858 | 3.01 | 240.2× |
| 2,048 | 171,558 | 411,919,347 | 2,401.1 | 554,786 | 3.23 | 742.5× |
| 4,000 | 226,214 | 1,569,620,906 | 6,938.7 | 670,658 | 2.96 | 2,340.4× |

The quadratic term is gone: each doubling used to multiply the peak by four, and
the ratio to the envelope now stays between 2.48 and 3.23 across the whole 31-fold
range. The arena-dependent term — peak less the decoded artifact's own retained
footprint — grows 220,531 → 480,017 rather than 220,531 → 1,569,430,265.

Two more rows moved with them, at 4,000 nodes:

- `forged/identity`, the rejection path, 1,569,620,906 → 556,247 (2,821.8×).
- `encode`, the producer path, 1,569,451,274 → 768,193 (2,043.0×) — the builder's
  key table going with the codec's.

Every section-dimension row (0 / 1 MiB / 16 MiB / 64 MiB of carried object) is
byte-identical to the retained `after` run, which is the check that this change
touched nothing those rows measure.

### The residual, checked rather than assumed

An allocation the attacker chose has been replaced by a *walk* the attacker
chooses, so the walk needs its own bound or this is a defect moved rather than
removed. It is bounded. Once duplicates are refused, every arena node is
pairwise structurally distinct, and `compare_expr_nodes` skips a shared position
outright — so an expensive comparison needs two *positionally distinct* subtrees
agreeing to depth `d`, which costs `m · d ≤ n` nodes to build. The heap does
`O(n log m)` comparisons over an `m`-element ready set, each costing at most `d =
n/m`, so the total is `O(n² log(m) / m)`, maximized at small `m`.

**Measurement, from a scratch probe rather than a retained one.** `m` chains of
depth `d` differing only at their seed literal, sized to `MAX_ABI_EXPRESSIONS`,
driving `n · log2(m)` comparisons — every one at its worst-case depth, which the
real heap does not do — on the same M4 Max under a release profile:

| chains `m` | depth | nodes | comparisons | wall |
| --- | --- | --- | --- | --- |
| 2 | 1,998 | 3,999 | 3,999 | 33.1 ms |
| 4 | 998 | 3,997 | 7,994 | 34.6 ms |
| 8 | 498 | 3,993 | 11,979 | 27.1 ms |
| 16 | 248 | 3,985 | 15,940 | 19.3 ms |
| 64 | 61 | 3,969 | 23,814 | 8.9 ms |
| 256 | 14 | 3,841 | 30,728 | 4.9 ms |

So the worst arena the governed bound admits costs roughly 35 ms of comparison
where it previously cost 1.5 GB of allocation. The probe is deliberately not
retained — it measures one internal function on a shape no fixture builds, and
the spike's own decode rows are the end-to-end evidence — but it is reproducible
from that description, and it is why `parse_expressions` recognizes duplicates by
hashing rather than by an `O(n log n)` comparator sort: the sort would have
multiplied this figure by roughly `log n` for no gain.

### Records updated

- `docs/artifact-abi.md` — the `14.0` step with its reason and the identity
  asymmetry, the canonical-order statement, the identity ledger row, and two
  stale "content key" clauses that the earlier identity flattening had already
  falsified.
- `docs/status.md` — the manifest schema row, `13.0` → `14.0`, with the step.
- `docs/research/artifacts/decoder-allocation-amplification.md` — Section 5
  carries both schemas' rows, and a new Section 8 records the forger-reach answer
  the note previously listed under "what this note does not establish".
  `disposition` moved `pending` → `partially-adopted`.
- `spikes/artifacts/decoder-allocation/README.md` and its harness comment — the
  third retained result file and what the arena dimension now exercises.

### Scopes added, with reasons

- `project/tickets` (shared) — this Outcome.
- `research/artifacts` (shared) — the spike re-run and the research note the
  ticket asked to have the new rows recorded in.
- `contracts/navigation` (shared) — `docs/status.md` names the manifest schema in
  its identity ledger, and a schema step that left it saying `13.0` would break
  the ledger coherence AGENTS.md requires.

### Filed

[`retire-or-re-document-the-now-consumerless-expr-key-in-tiler-ir`](retire-or-re-document-the-now-consumerless-expr-key-in-tiler-ir.md) —
`expr_key` now has no production consumer anywhere in the workspace and its own
documentation still claims `tiler-artifact` derives per-node keys through it.
Correcting or removing it is `implementation/ir` and a public-surface decision.

### Checks

`cargo fmt --all --check`, `cargo check --workspace --all-targets --locked`,
`cargo clippy` with warnings denied, `RUSTDOCFLAGS="-D warnings" cargo doc`,
`cargo nextest run --workspace --locked` (2,865 passed, 7 skipped),
`cargo test --workspace --doc`, `make full`, `tkt lint`, `git diff --check`, and
`tkt guard` against `b3d5a9ed`.
