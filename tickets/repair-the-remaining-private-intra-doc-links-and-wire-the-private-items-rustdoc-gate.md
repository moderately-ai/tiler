---
id: repair-the-remaining-private-intra-doc-links-and-wire-the-private-items-rustdoc-gate
title: Repair the remaining private intra-doc links and wire the private-items rustdoc gate
status: todo
priority: p3
dependencies: []
related: [repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see]
scopes: [implementation/ir, implementation/reference, implementation/frontend, implementation/artifact, implementation/build, implementation/cache, implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items` exits 0, and the `doc` target in the `Makefile` runs it, so a broken or redundant intra-doc link in a private item fails the gate instead of being invisible to it.

## Why this exists — filed 2026-08-18 from `repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see`

That ticket repaired `tiler-compiler`'s sixteen broken private intra-doc links and asked whether `--document-private-items` should join a checked gate. The recommendation there is yes, and the reason is the AGENTS.md rule about a check reaching its subject: the gate's rustdoc step is `cargo doc --workspace --no-deps`, which never renders a private or `pub(crate)` item, so no diagnostic about one can reach it. `tiler-compiler` declares five `pub` modules out of forty-odd, so the unchecked share of that crate alone is most of it.

Wiring it could not land there because the rest of the workspace is not clean, and the compiler ticket's scope was `implementation/compiler`. This ticket owns the remainder and the wiring.

**Measurement (2026-08-18, on base `01fc9682` with that ticket's `tiler-compiler` repairs applied, macOS M3 Pro).** With `tiler-compiler` clean, the workspace private-items run reports **36 diagnostics across seven crates**, and every other member — `tiler-compiler`, `tiler-conformance`, `tiler-digest`, `tiler-metal`, `tiler-metal-aot`, `tiler-runtime`, and the three prototypes — is clean:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked --keep-going
```

| crate | diagnostics |
| --- | --- |
| `tiler-ir` | 18 |
| `tiler-reference` | 6 |
| `tiler-macros` | 4 |
| `tiler-artifact` | 3 |
| `tiler` | 2 |
| `tiler-build` | 2 |
| `tiler-cache` | 1 |

Two lint classes, and they need different judgements. Twenty-four are `rustdoc::redundant_explicit_links` — a doc link whose explicit target repeats the path its link text already resolves to, only reported once private items are rendered. Twelve are `rustdoc::broken_intra_doc_links`: `ArithmeticType::canonical_type_key`, `BuildError::ExtentSource`, `ShapeEnv`, `s`, `super::super::kernel::builder` (`tiler-ir`); `crate::aot::retained`, `crate::aot::open_cache`, `RetainedText::Display`, `buildable_target` (`tiler-macros`); `decode_platform` (`tiler-artifact`); `Compilation`, `DecodedEntry::payload` (`tiler-build`).

**These counts are a measurement at that commit, not a Fact about your base.** Re-run the command and enumerate the population yourself before repairing, per AGENTS.md.

## Required content

- Repair each diagnostic by reading the doc's intent — link to the right item where a path exists, drop the explicit target where the link text already resolves to it, or convert to a plain code span where the referent is `#[cfg(test)]` and therefore unreachable by rustdoc. Never delete a doc's semantic content to satisfy the resolver. Doc-comment changes only.
- Wire the gate. The proposed `Makefile` change is a second command on the existing `doc` target:

  ```make
  doc:
  	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
  	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked
  ```

  Both commands, not one. Measured on `tiler-compiler` (the largest crate) with a warm target directory, the private-items pass costs 1.52s against the public pass's 1.28s, so the added time is not an argument either way. Keeping the public pass is what states that the *shipped* documentation is clean: the private pass renders a different page set, and a future rustdoc that treats a lint differently between the two modes would silently retire the public claim if it were the only run. Note that `rustdoc::private_intra_doc_links` fires in **both** modes today — under `--document-private-items` it says "this link resolves only because you passed `--document-private-items`, but will break without" — so the private run does not lose that check.
- Make the new command fail deliberately before trusting it, per AGENTS.md: break one link in a crate the public run cannot see, quote the failure text, and revert. The compiler ticket recorded that demonstration for its own crate; repeat it here for the wired workspace command.

## Closes when

The workspace private-items run exits 0, the `Makefile` runs it, the new step has been observed failing on a perturbed subject with its message quoted, and `make full` is green.
