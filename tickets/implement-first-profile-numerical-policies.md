---
id: implement-first-profile-numerical-policies
title: Implement first-profile numerical policy presets
status: in-progress
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [repair-numerical-witness-integrity]
scopes: [implementation/ir, implementation/reference, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, mature-product]
claimed_from: todo
assignee: agent-implement-first-profile-numerical-policies
lease_expires_at: 1785045284
---
Implement typed strict/relaxed numerical dimensions and per-operation/per-dtype conformance for reassociation, reciprocal transforms, approximations, exceptional values, signed zero, contraction, materialization rounding, and reduction order. Preserve compound/quantized seams and fail closed where evidence is Unknown.
