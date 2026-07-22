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
    "docs/architecture/03-three-first-party-plugins.md",
    "docs/decisions/ADR-0006-three-default-first-party-plugins.md",
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
    "market/fixtures/course-planning/minimal-v0.json",
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
    schema = load_json("market/schemas/plugin-package.schema.json", issues)
    publishers = load_json("market/publishers/first-party.json", issues)
    capabilities = load_json("market/capabilities/registry.json", issues)
    if not isinstance(schema, dict) or not isinstance(publishers, dict) or not isinstance(capabilities, dict):
        return

    expected_first_party_statuses = {
        "ustc.affairs-navigator": "planned",
        "ustc.change-radar": "planned",
        "ustc.opportunity-graph": "development",
    }
    expected_first_party_versions = {
        package_id: "0.1.0" for package_id in expected_first_party_statuses
    }
    expected_first_party_capabilities = {
        "ustc.affairs-navigator": ["campus.public_rules.read"],
        "ustc.change-radar": [
            "campus.public_rules.read",
            "campus.public_changes.read",
        ],
        "ustc.opportunity-graph": [
            "campus.public_plan.read",
            "campus.public_course.read",
            "campus.community_review.linkout",
        ],
    }
    expected_first_party_ids = set(expected_first_party_statuses)
    required = set(schema.get("required", []))
    allowed = set(schema.get("properties", {}))
    expected_required = {
        "id",
        "version",
        "publisher",
        "tier",
        "displayName",
        "implementationStatus",
        "installPolicy",
        "components",
        "capabilities",
        "sourcePolicy",
    }
    if required != expected_required:
        fail(f"PluginPackage schema required-field drift: {sorted(required)}", issues)
    if not expected_required <= allowed:
        fail("PluginPackage schema does not define every required field", issues)

    capability_rows = capabilities.get("capabilities", [])
    if not isinstance(capability_rows, list):
        fail("capability registry must contain a capabilities list", issues)
        return
    registered = {
        item.get("id")
        for item in capability_rows
        if isinstance(item, dict) and isinstance(item.get("id"), str)
    }
    auto_grant = {
        item.get("id")
        for item in capability_rows
        if isinstance(item, dict) and item.get("autoGrantEligible") is True
    }
    if len(registered) != len(capability_rows):
        fail("capability registry contains duplicate or malformed ids", issues)

    manifests: list[dict[str, object]] = []
    for path in sorted((ROOT / "market/packages").glob("*/package.json")):
        rel_path = path.relative_to(ROOT).as_posix()
        manifest = load_json(rel_path, issues)
        if not isinstance(manifest, dict):
            continue
        manifests.append(manifest)

        missing = required - manifest.keys()
        unexpected = manifest.keys() - allowed
        if missing:
            fail(f"{rel_path}: missing keys: {sorted(missing)}", issues)
        if unexpected:
            fail(f"{rel_path}: unexpected keys: {sorted(unexpected)}", issues)

        package_id = manifest.get("id")
        if package_id != path.parent.name:
            fail(f"{rel_path}: package id does not match directory", issues)
        if not isinstance(package_id, str) or re.fullmatch(
            r"[a-z0-9]+(?:\.[a-z0-9-]+)+", package_id
        ) is None:
            fail(f"{rel_path}: invalid package id", issues)
        if not isinstance(manifest.get("version"), str) or re.fullmatch(
            r"(?:0|[1-9][0-9]*)\.[0-9]+\.[0-9]+", manifest.get("version", "")
        ) is None:
            fail(f"{rel_path}: invalid SemVer package version", issues)
        if not isinstance(manifest.get("displayName"), str) or not manifest.get("displayName"):
            fail(f"{rel_path}: displayName must be a non-empty string", issues)
        if "description" in manifest and not isinstance(manifest.get("description"), str):
            fail(f"{rel_path}: description must be a string", issues)
        if not isinstance(manifest.get("publisher"), str) or not manifest.get("publisher"):
            fail(f"{rel_path}: publisher must be a non-empty string", issues)
        if manifest.get("tier") not in {
            "FirstParty",
            "VerifiedCommunityText",
            "VerifiedRemoteMcp",
        }:
            fail(f"{rel_path}: invalid package tier", issues)

        is_default_first_party = package_id in expected_first_party_ids
        if is_default_first_party:
            if manifest.get("publisher") != publishers.get("id"):
                fail(f"{rel_path}: publisher does not match first-party registry", issues)
            if manifest.get("tier") != "FirstParty":
                fail(f"{rel_path}: default package tier must be FirstParty", issues)
            if manifest.get("version") != expected_first_party_versions[package_id]:
                fail(f"{rel_path}: default package version drift", issues)
        elif manifest.get("publisher") == publishers.get("id") or manifest.get("tier") == "FirstParty":
            fail(f"{rel_path}: unregistered first-party package identity", issues)

        status = manifest.get("implementationStatus")
        if status not in {"planned", "development", "implemented"}:
            fail(f"{rel_path}: invalid implementationStatus", issues)
        elif is_default_first_party and status != expected_first_party_statuses[package_id]:
            fail(f"{rel_path}: default package implementationStatus drift", issues)

        install_policy = manifest.get("installPolicy")
        if not isinstance(install_policy, dict) or set(install_policy) != {
            "class",
            "defaultInstalled",
            "defaultEnabled",
            "userDisableAllowed",
        }:
            fail(f"{rel_path}: malformed installPolicy", issues)
        elif is_default_first_party:
            expected_install_policy = {
                "class": "FirstPartySystemPlugin",
                "defaultInstalled": True,
                "defaultEnabled": True,
                "userDisableAllowed": True,
            }
            if install_policy != expected_install_policy:
                fail(f"{rel_path}: default first-party installPolicy drift", issues)
        elif (
            install_policy.get("class") != "UserInstalledPlugin"
            or install_policy.get("defaultInstalled") is not False
            or install_policy.get("defaultEnabled") is not False
            or not isinstance(install_policy.get("userDisableAllowed"), bool)
        ):
            fail(f"{rel_path}: non-default package installPolicy is unsafe", issues)

        components = manifest.get("components")
        if not isinstance(components, list):
            fail(f"{rel_path}: components must be a list", issues)
            components = []
        if status == "planned" and components:
            fail(f"{rel_path}: planned package must not claim components", issues)
        if status == "implemented" and not components:
            fail(f"{rel_path}: implemented package must declare at least one component", issues)
        for component in components:
            if not isinstance(component, dict):
                fail(f"{rel_path}: component is not an object", issues)
                continue
            if set(component) - {"type", "path", "mode"}:
                fail(f"{rel_path}: component contains unsupported fields", issues)
            if component.get("type") not in {
                "SkillComponent",
                "DeclarativeResourcePack",
                "McpServerComponent",
                "NativeRustComponent",
            }:
                fail(f"{rel_path}: unsupported component type", issues)
            component_path = component.get("path")
            if not isinstance(component_path, str):
                fail(f"{rel_path}: component path must be a string", issues)
                continue
            candidate = Path(component_path)
            try:
                resolved = (ROOT / candidate).resolve(strict=True)
                resolved.relative_to(ROOT.resolve())
            except (OSError, ValueError):
                fail(f"{rel_path}: component path missing or unsafe: {component_path}", issues)
                continue
            if candidate.is_absolute() or ".." in candidate.parts or not resolved.is_file():
                fail(f"{rel_path}: component path missing or unsafe: {component_path}", issues)

        manifest_capabilities = manifest.get("capabilities")
        if not isinstance(manifest_capabilities, list) or not all(
            isinstance(capability, str) for capability in manifest_capabilities
        ):
            fail(f"{rel_path}: capabilities must be a string list", issues)
            manifest_capabilities = []
        if len(set(manifest_capabilities)) != len(manifest_capabilities):
            fail(f"{rel_path}: duplicate capabilities", issues)
        if (
            is_default_first_party
            and manifest_capabilities != expected_first_party_capabilities[package_id]
        ):
            fail(f"{rel_path}: default package capability set drift", issues)
        for capability in manifest_capabilities:
            if capability not in registered:
                fail(f"{rel_path}: capability not registered: {capability}", issues)
            if is_default_first_party and capability not in auto_grant:
                fail(f"{rel_path}: default package capability is not auto-grant-eligible: {capability}", issues)

        source_policy = manifest.get("sourcePolicy")
        if not isinstance(source_policy, dict) or not source_policy or not all(
            isinstance(key, str) and isinstance(value, str) for key, value in source_policy.items()
        ):
            fail(f"{rel_path}: sourcePolicy must be a non-empty string map", issues)
        elif is_default_first_party and not source_policy.get("personalData"):
            fail(f"{rel_path}: default package sourcePolicy must state personalData scope", issues)

    first_party_ids = {
        manifest.get("id")
        for manifest in manifests
        if manifest.get("publisher") == publishers.get("id") and manifest.get("tier") == "FirstParty"
    }
    if first_party_ids != expected_first_party_ids:
        fail(
            "default first-party package identity drift: "
            f"expected={sorted(expected_first_party_ids)} actual={sorted(str(item) for item in first_party_ids)}",
            issues,
        )


def check_course_fixture(issues: list[str]) -> None:
    fixture = load_json("market/fixtures/course-planning/minimal-v0.json", issues)
    if not isinstance(fixture, dict):
        return
    expected_top_level = {
        "schema_version",
        "source_revision",
        "sources",
        "profile",
        "requirements",
        "courses",
        "community_signals",
    }
    unexpected = set(fixture) - expected_top_level
    if unexpected:
        fail(f"Course Planning fixture has unexpected top-level fields: {sorted(unexpected)}", issues)
    if fixture.get("schema_version") != "course-planning/v0":
        fail("Course Planning fixture schema drift", issues)
    if fixture.get("source_revision") != "synthetic-course-planning-v0":
        fail("Course Planning fixture must remain explicitly synthetic", issues)

    sources = fixture.get("sources")
    allowed_authorities = {
        "official_catalog_snapshot",
        "reviewed_official_source",
        "icourse_mirror",
        "community_signal",
    }
    if not isinstance(sources, list):
        fail("Course Planning fixture sources must be a list", issues)
    else:
        for source in sources:
            if not isinstance(source, dict):
                fail("Course Planning fixture source must be an object", issues)
                continue
            if source.get("authority") not in allowed_authorities:
                fail(f"Course Planning fixture source authority is not allowed: {source.get('authority')}", issues)
            if not isinstance(source.get("effective_time"), str):
                fail(f"Course Planning fixture source lacks effective_time: {source.get('id')}", issues)

    courses = fixture.get("courses")
    if not isinstance(courses, list):
        fail("Course Planning fixture courses must be a list", issues)
        return
    unique_codes = {
        course.get("code")
        for course in courses
        if isinstance(course, dict) and isinstance(course.get("code"), str)
    }
    if len(unique_codes) < 20:
        fail("Course Planning fixture must retain at least 20 unique synthetic candidates", issues)

    signals = fixture.get("community_signals")
    signal_fields = {"course_code", "source_id", "score", "link"}
    if not isinstance(signals, list):
        fail("Course Planning community_signals must be a list", issues)
    else:
        for signal in signals:
            if not isinstance(signal, dict):
                fail("Course Planning community signal must be an object", issues)
                continue
            if set(signal) != signal_fields:
                fail(f"Course Planning community signal field drift: {sorted(signal)}", issues)
            link = signal.get("link")
            if not isinstance(link, str) or not link.startswith("https://icourse.club/"):
                fail("Course Planning community signal must remain iCourse link-out-only", issues)

    forbidden_keys = {"password", "cookie", "student_id", "review_text", "review_content", "raw_review"}

    def walk(value: object) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                if key.lower() in forbidden_keys:
                    fail(f"forbidden sensitive fixture field: {key}", issues)
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)

    walk(fixture)


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
    check_course_fixture(issues)
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
