---
id: convert-docs-prose-to-unwrapped-source-form
title: Convert the documentation corpus to unwrapped prose source form
status: deferred
priority: p3
dependencies: []
related: [record-prose-wrapping-convention-for-docs]
scopes: [contracts/artifacts, contracts/decisions, contracts/foundation, contracts/integrations, contracts/navigation, contracts/numerics, contracts/optimizer, research/apple-targets, research/artifacts, research/cache, research/cost-model, research/documentation, research/embedding, research/extensions, research/indexing, research/kernel-ir, research/macro-environment, research/numerics, research/placement, research/program-planning, research/reference, research/region-search, research/runtime, research/scheduling, research/semantic-graph, research/shapes, research/target-profiles, research/transfers, research/workspace]
shared_scopes: []
paths: []
tags: [documentation, conventions]
---
`docs/document-metadata.md` records unwrapped prose as the convention and accepts a mixed corpus as the transitional state: a paragraph converts when it is edited, and an untouched document keeps whatever form it has. That policy is deliberately gradual, and this ticket is the alternative it declines to take today. Do not start it without Tom accepting the trade-off below.

**Measurement** (base `f57e23b`, command below, over the 154 files under `docs/**`): 133 hard-wrapped, 15 mixed, 6 unwrapped. 148 files contain at least one wrapped paragraph or wrapped list item, so a full conversion touches roughly 96% of the corpus. A file counts as hard-wrapped when a `paragraph` token contains a `softbreak`, and as unwrapped when a single-line paragraph exceeds 80 columns while every whitespace token in it would have fitted — the second clause keeps an unbreakable URL from reading as an authoring choice. The file-level split is stable against that threshold: at 84, 88, and 96 columns it moves only to 135/13/6, 136/12/6, and 137/11/6.

```sh
uv run --locked python - <<'EOF'
import pathlib, re
from markdown_it import MarkdownIt
LINK = re.compile(r"\]\([^)]*\)")
def longish(l):
    c = LINK.sub("](#)", l)
    return len(c) > 80 and max((len(t) for t in c.split()), default=0) <= 80
md = MarkdownIt("commonmark").enable("table")
tally = {}
for p in sorted(pathlib.Path("docs").rglob("*.md")):
    text = p.read_text()
    body = text[text.find("\n---\n", 3) + 5:] if text.startswith("---\n") else text
    lines = body.split("\n")
    toks, depth, w, u = md.parse(body), 0, 0, 0
    for i, t in enumerate(toks):
        depth += (t.type == "list_item_open") - (t.type == "list_item_close")
        if t.type != "inline" or not i or toks[i - 1].type != "paragraph_open" or depth:
            continue
        span = t.map or toks[i - 1].map
        if any(c.type == "softbreak" for c in (t.children or [])):
            w += 1
        elif any(longish(l) for l in lines[span[0]:span[1]]):
            u += 1
    key = "mixed" if w and u else "hard-wrapped" if w else "unwrapped" if u else "none"
    tally[key] = tally.get(key, 0) + 1
print(tally)
EOF
```

**Inference:** the conversion is mechanically safer than its size suggests. Joining the `softbreak`s inside `paragraph` tokens is content-preserving by construction, and the result is verifiable by rendering every file before and after and diffing the HTML with whitespace normalized, so "documents nobody reviewed" is not the risk it appears to be. `git blame` damage is recoverable through a `.git-blame-ignore-revs` entry, which exists for exactly this class of commit.

**The real cost is scheduling, not review.** This ticket declares 29 scopes because a corpus-wide reflow collides with every concurrent documentation branch. It is only cheap on a quiescent tree.

**Trigger for reconsideration:** no other documentation ticket is `in-progress` or `review`, and Tom accepts a one-commit reflow. Until then the gradual policy in `docs/document-metadata.md` stands and this ticket stays parked.

## Outcome

Not started.
