---
id: fuzz-the-expansion-cache-framing-paths
title: Fuzz the expansion cache framing and allocation paths
status: done
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/workspace, implementation/cargo-lock]
shared_scopes: [project/tickets]
paths: []
tags: [cache, testing, correctness]
---
The second half of the research note's second follow-up gate: "fuzz every framing and bounded-allocation path".

`tiler-cache`'s bundle decoder has a rejection for every framing field, and each is covered by a directed test. Directed tests prove the checks a reader thought of. A bundle is read from a directory any process on the host may write to, so the interesting inputs are the ones nobody thought of.

## What this ticket owes

- Fuzz `bundle::decode` over arbitrary bytes and over mutations of valid bundles. The property is that it never panics, never allocates past `Limits`, and either returns a view whose sections lie inside the input or a typed rejection.
- Fuzz the entry-path parser over arbitrary strings. The property is that a parsed key round-trips to the exact text it was parsed from, so no two texts can name one entry.
- Include a resealing mutator, as `tiler-artifact`'s codec suite does: a corruption a digest catches proves only that the digest works, and the cases worth finding are the internally consistent ones.
- Start with a bounded in-tree arbitrary-input/property harness using existing
  dependencies. If external fuzz tooling is materially better, admit the named
  dependency deliberately under the declared workspace and lockfile scopes,
  with the comparison that justifies it.

## Outcome — in-tree property harness, no dependency admitted (2026-07-27)

`crates/tiler-cache/src/expansion/fuzz.rs` drives `bundle::decode` and `CacheKey::parse_label` with generated input on every run of the ordinary suite. Five properties, ~19,000 cases, in milliseconds.

**The dependency comparison the ticket asked for, and the answer.** `cargo-fuzz`/`libfuzzer` are materially better at *finding* a crash — they are coverage-guided, so they reach paths a blind generator hits only by luck. They also need a nightly-only sanitizer runtime, a separate target directory, a build outside `make check`, and a corpus nothing here would store or replay. The property named — never panic, never allocate past `Limits`, and either return a view inside the input or a typed rejection — is checkable in-tree, and **a check that runs on every gate is worth more here than a better search nobody runs**. The dependency stays available; the trigger for admitting it is a real defect this misses, and it has not fired. No new dependency, so `implementation/workspace` and `implementation/cargo-lock` went untouched.

**The generator is a `splitmix64` inlined in the file**, seeded fixed, so a failure reproduces from the reported iteration index alone. It decides which inputs are tried and never what counts as a pass, so its statistical quality is not load-bearing — that is stated at the type.

**What is covered:**

- **Arbitrary bytes** — 4,096 cases. Establishes the *entry* is safe.
- **Single-byte mutations of a valid bundle** — 4,096 cases. Reusing a real prefix is what gets past the magic, schema, and algorithm tag, so mutations land on the offsets, lengths, and digests.
- **Truncation and extension** — 2,048 cases, where a length field and the bytes present disagree.
- **The resealing mutator** — 512 cases. Corrupts the subject then *re-encodes*, so every section digest and the declared length agree: the forgery is internally perfect and differs only in that the key it is filed under no longer derives from the subject it carries. It asserts refusal **and** asserts that at least one case reached `KeyNotDerivedFromSubject`, so the test cannot pass by rejecting everything earlier for some other reason.
- **Entry-path round-trip** — 8,448 cases. A real label must survive, and any arbitrary text that parses must render back to exactly itself, because a second spelling of one key means two paths naming one entry.

**The harness was verified able to say no, which for a property test is the whole question.** Deleting the section end-bound check in `decode_sections` makes `single_byte_mutations_of_a_valid_bundle_never_panic_the_decoder` fail — and fail with a **panic inside the decoder at `bundle.rs:407`**, which is precisely the property. That is the evidence that the mutations reach deep into the framing rather than being refused at byte zero, so the case count is real coverage and not 19,000 rejections at the magic. The check was restored and the suite is green: 103 tests pass.
