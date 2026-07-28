# Vendored Apple Metal sources

These primary documents are retained with the research they support so future compatibility and language-vocabulary audits do not depend on an untracked download.

| File | Document identity | SHA-256 |
| --- | --- | --- |
| `apple-metal-feature-set-tables-2025-10-20.pdf` | Apple Metal Feature Set Tables; PDF creation date 2025-10-20 | `cee50f0c32a9af4a3cc4eeb8ab0d3d5d6444173f15800771ad2316f48603e07e` |
| `apple-metal-shading-language-specification-v4-2025-10-23.pdf` | Metal Shading Language Specification, Version 4, dated 2025-10-23 | `eed87a82d4d2d475423b91b3c529c5313a85433f83e22b7fe3ec50e90254f44a` |
| `apple-metal-shading-language-specification-v4.1-2026-06-04.pdf` | Metal Shading Language Specification, Version 4.1, dated 2026-06-04 | `41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5` |

The documents are Apple publications and retain Apple's copyright notices. Their presence here is evidence preservation, not a claim that every language revision or GPU family they describe is implemented, compiled by the pinned toolchain, or execution-measured by Tiler. Research records must continue to distinguish normative specification, SDK/toolchain availability, successful compilation, artifact compatibility, and device execution.

Reproduce the retained hashes with:

```sh
shasum -a 256 docs/research/apple-targets/sources/*.pdf
```
