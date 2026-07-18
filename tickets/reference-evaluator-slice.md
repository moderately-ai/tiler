---
id: reference-evaluator-slice
title: Define and exercise the normative reference evaluator slice
status: todo
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract, numerical-policy-contract]
related: []
scopes: [research/reference]
shared_scopes: []
paths: []
tags: [tiler-research, spike, correctness]
---
Select one representative end-to-end tensor pipeline and define its normative semantic evaluation independently of optimization and GPU scheduling. Cover broadcasting, reshape or view semantics, dtype conversion, materialization boundaries, multiple graph outputs, and error cases.

Deliver executable or mechanically checkable reference cases suitable for rewrite, fusion, backend, and differential tests. The evaluator may be deliberately slow; its contract must be precise enough to decide correctness.
