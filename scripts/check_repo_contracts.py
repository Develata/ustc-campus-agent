#!/usr/bin/env python3
"""Repository contract checks for USTC Campus Agent.

This intentionally uses only the Python standard library so it can run in CI
before the project chooses additional tooling.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALID_GATES = {"pr", "core-demo", "release", "public"}
SECRET_PATTERNS = [
    re.compile(r"(?:PB|SA|SB|SC|BA|BC)\d{8}"),
    re.compile(r"USTC_PASSWORD\s*="),
    re.compile(r"USTC_107_COOKIE\s*="),
    re.compile(r"BEGIN (RSA |EC |OPENSSH |)PRIVATE KEY"),
    re.compile(r"ghp_[A-Za-z0-9_]{20,}"),
    re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
]
KEY_FILES = [
    "README.md",
    "AGENTS.md",
    "docs/acceptance/gates.md",
    "docs/acceptance/matrix.tsv",
    "docs/collaboration/agent-workflow.md",
    "docs/collaboration/ownership.md",
    "docs/collaboration/pr-contract.md",
    "docs/collaboration/task-slicing.md",
    "docs/contracts/cli.md",
    "docs/contracts/data-models.md",
    "docs/contracts/interfaces.md",
    "docs/public/github-pages-brief.md",
    "market/review-policy/first-party.md",
    "market/fixtures/course-planning/README.md",
]


def fail(msg: str, issues: list[str]) -> None:
    issues.append(msg)


def check_key_files_present_and_nonempty(issues: list[str]) -> None:
    for rel in KEY_FILES:
        path = ROOT / rel
        if not path.exists():
            fail(f"key file missing: {rel}", issues)
            continue
        if not path.is_file():
            fail(f"key path is not a file: {rel}", issues)
            continue
        if not path.read_text(encoding="utf-8").strip():
            fail(f"key file empty: {rel}", issues)


def check_markdown_links(issues: list[str]) -> None:
    for path in ROOT.rglob("*.md"):
        if any(part in {".git", "target", ".codegraph"} for part in path.parts):
            continue
        text = path.read_text(encoding="utf-8")
        for match in re.finditer(r"\[[^\]]+\]\(([^)]+)\)", text):
            target = match.group(1).strip()
            if not target or target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            if ":" in target.split("#", 1)[0]:
                continue
            clean = target.split("#", 1)[0].split("?", 1)[0]
            if not clean:
                continue
            resolved = (path.parent / clean).resolve()
            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                fail(f"markdown link escapes repo: {path.relative_to(ROOT)} -> {target}", issues)
                continue
            if not resolved.exists():
                fail(f"broken markdown link: {path.relative_to(ROOT)} -> {target}", issues)


def check_no_obvious_secrets(issues: list[str]) -> None:
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        if any(part in {".git", "target", ".codegraph"} for part in path.parts):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                fail(f"possible secret pattern {pattern.pattern!r} in {path.relative_to(ROOT)}", issues)


def load_json(rel: str, issues: list[str]) -> object | None:
    path = ROOT / rel
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - checker should report exact file
        fail(f"invalid json {rel}: {exc}", issues)
        return None


def check_market(issues: list[str]) -> None:
    manifest = load_json("market/packages/ustc.opportunity-graph/package.json", issues)
    publishers = load_json("market/publishers/first-party.json", issues)
    capabilities = load_json("market/capabilities/registry.json", issues)
    if not isinstance(manifest, dict) or not isinstance(publishers, dict) or not isinstance(capabilities, dict):
        return

    required = {"id", "version", "publisher", "tier", "displayName", "components", "capabilities", "sourcePolicy"}
    missing = required - manifest.keys()
    if missing:
        fail(f"manifest missing keys: {sorted(missing)}", issues)
    if manifest.get("id") != "ustc.opportunity-graph":
        fail("flagship package id drift", issues)
    if manifest.get("publisher") != publishers.get("id"):
        fail("manifest publisher does not match publisher registry", issues)
    registered = {item.get("id") for item in capabilities.get("capabilities", []) if isinstance(item, dict)}
    for cap in manifest.get("capabilities", []):
        if cap not in registered:
            fail(f"manifest capability not registered: {cap}", issues)
    for component in manifest.get("components", []):
        if not isinstance(component, dict):
            fail("manifest component is not an object", issues)
            continue
        rel = component.get("path")
        if not isinstance(rel, str) or not (ROOT / rel).exists():
            fail(f"manifest component path missing: {rel}", issues)


def check_acceptance_matrix(issues: list[str]) -> None:
    path = ROOT / "docs/acceptance/matrix.tsv"
    rows = path.read_text(encoding="utf-8").splitlines()
    if not rows:
        fail("acceptance matrix empty", issues)
        return
    header = rows[0].split("\t")
    expected = ["case_id", "domain", "assertion", "binding", "gate", "status", "owner"]
    if header != expected:
        fail(f"acceptance matrix header drift: {header}", issues)
        return
    seen: set[str] = set()
    for line_no, row in enumerate(rows[1:], start=2):
        if not row.strip():
            continue
        cols = row.split("\t")
        if len(cols) != len(expected):
            fail(f"matrix row {line_no} has {len(cols)} columns", issues)
            continue
        case_id = cols[0]
        if case_id in seen:
            fail(f"duplicate case_id: {case_id}", issues)
        seen.add(case_id)
        for gate in cols[4].split(","):
            if gate not in VALID_GATES:
                fail(f"unknown gate {gate!r} in {case_id}", issues)
    if len(seen) < 10:
        fail("acceptance matrix too small for current contract", issues)


def main() -> int:
    issues: list[str] = []
    check_key_files_present_and_nonempty(issues)
    check_markdown_links(issues)
    check_no_obvious_secrets(issues)
    check_market(issues)
    check_acceptance_matrix(issues)
    if issues:
        print("contract-check: FAIL")
        for issue in issues:
            print(f"- {issue}")
        return 1
    print("contract-check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
