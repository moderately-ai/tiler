#!/usr/bin/env python3
"""Check the shape of the checked-in CI workflow.

This deliberately does not compare the workflow against a copy of itself. It
used to: a 95-line Python literal reproduced every step name, both action SHAs,
both runners, and the full `run:` bodies, so any edit to the workflow failed the
gate until an identical edit landed here. That bought nothing. The threat it was
written against is an actor who weakens CI, and that actor edits both files in
the same change; a copy is a second edit, not a second opinion.

What is worth checking is structural, and is what a reviewer reading a diff
would not reliably notice: that there is exactly one workflow rather than a
second one shadowing it, that it is a real file rather than a symlink pointing
somewhere unversioned, and that it parses as YAML with duplicate keys rejected
-- a repeated `on:` or `jobs:` key silently discards the earlier block.

The workflow reads every pinned version from its authority at run time, so
there is no version to restate here either.
"""

from __future__ import annotations

import sys
from pathlib import Path

from ruamel.yaml import YAML
from ruamel.yaml.error import YAMLError

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_NAME = "rust.yml"


def validate_repository(root: Path = ROOT) -> list[str]:
    """Require exactly the one governed workflow and a parseable body."""
    workflow_root = root / ".github/workflows"
    observed = sorted(path.name for path in workflow_root.glob("*.yml"))
    observed += sorted(path.name for path in workflow_root.glob("*.yaml"))
    if observed != [WORKFLOW_NAME]:
        return [f"workflow set: expected ['{WORKFLOW_NAME}'], got {observed}"]
    workflow = workflow_root / WORKFLOW_NAME
    if workflow.is_symlink() or not workflow.is_file():
        return [f"workflow set: {WORKFLOW_NAME} must be a regular non-symlink file"]
    parser = YAML(typ="safe", pure=True)
    parser.version = (1, 2)
    parser.allow_duplicate_keys = False
    try:
        parsed = parser.load(workflow.read_text(encoding="utf-8"))
    except YAMLError as error:
        return [f"workflow: malformed YAML: {error}"]
    if not isinstance(parsed, dict) or "jobs" not in parsed:
        return ["workflow: must be a mapping that declares jobs"]
    return []


def main() -> int:
    """Validate CI and return a process status."""
    errors = validate_repository()
    if errors:
        print("CI validation failed:\n" + "\n".join(errors), file=sys.stderr)
        return 1
    print("CI validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
