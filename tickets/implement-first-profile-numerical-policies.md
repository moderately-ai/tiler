---
id: implement-first-profile-numerical-policies
title: Implement first-profile numerical policy presets
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/ir, implementation/reference, implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, numerics, mature-product]
---
Implement typed strict/relaxed numerical dimensions and per-operation/per-dtype conformance for reassociation, reciprocal transforms, approximations, exceptional values, signed zero, contraction, materialization rounding, and reduction order. Preserve compound/quantized seams and fail closed where evidence is Unknown.
