---
id: prototype-target-feasibility-authority
title: Implement checked target-profile feasibility authority
status: todo
priority: p0
dependencies: [prototype-typed-explain-infrastructure, harden-compiler-verifier-subject-binding-and-totality]
related: [target-profile-feasibility-model]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, target-profile, feasibility, authority]
---
Implement immutable checked target profiles and typed feasibility predicates,
facts, provenance, evaluation phases, resource limits, and Unknown outcomes.
Hard feasibility is not cost; malformed profiles/proposals are intrinsic errors
and a valid empty feasible set is a distinct result.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
