---
id: reconcile-the-document-metadata-validator-claim-with-its-own-validation-section
title: Reconcile the document-metadata validator claim with its own validation section
status: done
priority: p3
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`docs/document-metadata.md` stops asserting a link-validation gate that does not exist, so a reader planning to rely on it learns the truth the corpus elsewhere states plainly: nothing validates the documentation corpus, and a reader is the only check.

## Why

**Fact — found by `specify-the-consumer-neutral-backend-provider-composition-contract` while checking its own links, 2026-07-31.** `docs/document-metadata.md` line 114 asserts that `validate_links` "fails the repository gate", while the same document's validation section states there is no validator. `grep -rn "validate_links" .` finds the name only in that prose — no script, target, or test carries it, and `make full`'s stages (fmt, check, clippy, nextest, doc-tests, rustdoc, release numerical tests, tkt lint, shellcheck) contain no link step. AGENTS.md's docs-maintenance section states the true position: local link targets are ordinary hand-maintained prose and a broken link costs a reader rather than a gate.

## Closes when

The false sentence is corrected or removed, the document's two halves agree, and any other sentence in it that promises tooling the tree does not contain is swept in the same change.
