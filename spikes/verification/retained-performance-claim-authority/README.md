---
schema: "tiler-doc/v1"
id: "tiler.spike.verification.retained-performance-claim-authority"
kind: "experiment"
title: "Retained performance-claim authority fixture"
topics: ["verification", "conformance", "performance", "identity"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.verification.retained-performance-claim-authority-and-identity"]
entrypoints: ["spikes/verification/retained-performance-claim-authority/README.md", "spikes/verification/retained-performance-claim-authority/check_fixture.sh", "spikes/verification/retained-performance-claim-authority/owner-claims.tsv", "spikes/verification/retained-performance-claim-authority/owner-claims-perturbed.tsv", "spikes/verification/retained-performance-claim-authority/profile-dispositions.tsv"]
last_verified: "2026-08-24"
ticket: "define-retained-performance-claim-authority-and-identity"
---

# Retained performance-claim authority fixture

This executable model tests one property of the proposed design: adding an owner-emitted performance claim without a corresponding profile disposition must fail loudly. It is not the final manifest schema, a claim census, or performance evidence.

Run from the repository root:

```sh
spikes/verification/retained-performance-claim-authority/check_fixture.sh spikes/verification/retained-performance-claim-authority/owner-claims.tsv
spikes/verification/retained-performance-claim-authority/check_fixture.sh spikes/verification/retained-performance-claim-authority/owner-claims-perturbed.tsv
```

The first command prints:

```text
2 claims; 2 dispositions; complete
```

The second command is the subject perturbation. `owner-claims-perturbed.tsv` adds `perf.audit.undisposed@1` to the owner population without editing the profile, and the checker exits nonzero with:

```text
undisposed performance claim: perf.audit.undisposed@1
```

The fixture also rejects duplicate or empty claim identities, duplicate dispositions, and dispositions for subjects the owner manifest did not emit. Its two ordinary rows are illustrative only and deliberately do not claim to mirror the repository's current performance corpus.
