#!/usr/bin/env python3
"""Repository contract checks for USTC Campus Agent.

This intentionally uses only the Python standard library so it can run in CI
before the project chooses additional tooling.
"""

from __future__ import annotations

import hashlib
import json
import re
import shlex
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALID_GATES = {"pr", "core-demo", "release", "public"}
VALID_ACCEPTANCE_STATUSES = {"planned", "implemented"}
STABLE_CATALOG_PREFIXES = {"AGENT", "FP", "HARNESS", "PKG", "PROC", "SRC"}
MIN_LONG_HORIZON_CASES = 200
INVOCATION_FIXTURES = {
    "arguments-golden-v0.json",
    "call-dispatch-denials-v0.json",
    "call-precedence-v0.json",
    "grant-scope-stale-v0.json",
    "identity-mismatch-v0.json",
    "installation-authority-v0.json",
    "post-projection-revoke-v0.json",
    "projection-precedence-v0.json",
    "schema-golden-v0.json",
    "scope-capability-source-v0.json",
    "tool-definition-mutation-v0.json",
    "tool-identity-mismatch-v0.json",
    "valid-synthetic-v0.json",
}
INVOCATION_FIXTURE_DIGESTS = {
    "arguments-golden-v0.json": "9624d3f50c6d50d9871c42476b345d686276173d091c0dd9adcaf80d6e3cde1a",
    "call-dispatch-denials-v0.json": "976291a00d4d9446049fc36fd97dc1a50d7e87b183ea8d13002ac52dfa7ed373",
    "call-precedence-v0.json": "c6fc030cae68ed163ab482bff9038d04914459ed54d9d1b92c1c99b6be15e3c4",
    "grant-scope-stale-v0.json": "1d287d368df49eb9cb68b5d90490ec68a32820b5cb4de267f10b0825801f05fe",
    "identity-mismatch-v0.json": "67d8e1f4043335a2c064ea18a5149270d536b7ef886a238f267b1e4f95534f8c",
    "installation-authority-v0.json": "ecfa34988e6e8cc98e0f876bd8c89857b934531f77859415701947185d1504c7",
    "post-projection-revoke-v0.json": "c85a5cfd2b333de8b6d9f7372f1386c235f48f798e03ad2ff862eefbc960a450",
    "projection-precedence-v0.json": "239d2bcd5434ac19506d98ea675438a1655d8715b4f5f5f98655a93b8fa3dc1e",
    "schema-golden-v0.json": "89ddd43523f868de889851ea83e2512a073b6ae49d2301bbba6f5d5fd7fd8bd4",
    "scope-capability-source-v0.json": "9127908fd89dd8326d02e46d3852bc9f6f2ba537ac92feb975ee4cb69a16e182",
    "tool-definition-mutation-v0.json": "dd1ca3fa664f320cadba84e3bb18d528ec679130f6477546d3fbe036f9c5e064",
    "tool-identity-mismatch-v0.json": "b4e94adb2415e28850f3073c9b1e33abc8fd24a4e7e231572527e139d88d0706",
    "valid-synthetic-v0.json": "4058327f9da3509741c0625381853255b2218143b9a19184ad2be053247283eb",
}
INVOCATION_FIXTURE_TEST_COMMAND = (
    "cargo test --locked -p ustc-campus-agent-core --test invocation_resolution "
    "executable_synthetic_fixture_matrix_is_complete -- --exact"
)
INVOCATION_COMPOSITION_FIXTURE_TEST_COMMAND = (
    "cargo test --locked -p ustc-agentd --test resolved_run_spec "
    "fixture_run_spec_mapping_constructs_run_and_denial_constructs_neither -- --exact"
)
AGENT_ALLOWED_DIRECT_DEPENDENCIES = {
    "serde": ("serde", "registry"),
    "serde_json": ("serde_json", "registry"),
    "ustc-agent-tool-protocol": (
        "ustc-agent-tool-protocol",
        "path:crates/agent-tool-protocol",
    ),
}
AGENT_FORBIDDEN_SOURCE_MARKERS = {
    "ustc_campus_agent_core",
    "ustc_campus_agent_adapters",
    "ustc_campus_agent_course_planning",
    "PluginPackage",
    "ComponentKind",
    "McpServerComponent",
    "NativeRustComponent",
}
AGENT_FORBIDDEN_SOURCE_PATTERNS = {
    r"\bAgentToolDefinition\b": "projection authority type AgentToolDefinition",
    r"\bAgentToolsetView\b": "projection authority type AgentToolsetView",
    r"\bAgentTool\b": "projection authority type AgentTool",
    r"\bcfg_attr\b": "cfg_attr conditional compilation",
    r"\binclude(?:_bytes|_str)?\s*!\s*\(": "include macro",
}
ALLOWED_AGENT_TEST_CFG = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
EXPECTED_DOC_DIRECTORIES = {
    "acceptance",
    "adr",
    "contracts",
    "features",
    "guides",
    "overview",
    "plan",
    "tasks",
}
EXPECTED_DOC_ROOT_FILES = {"AGENTS.md", "README.md", "coverage-matrix.md"}
RETIRED_DOC_DIRECTORIES = {
    "architecture",
    "collaboration",
    "decisions",
    "development",
    "legacy",
    "operations",
    "public",
}
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
    "docs/README.md",
    "docs/AGENTS.md",
    "docs/coverage-matrix.md",
    "docs/plan/AGENTS.md",
    "docs/plan/00-engineering-constitution.md",
    "docs/plan/01-terminology.md",
    "docs/plan/02-product-positioning.md",
    "docs/plan/03-platform-authority.md",
    "docs/plan/04-market-and-plugin-lifecycle.md",
    "docs/plan/05-campus-trust-kernel.md",
    "docs/plan/06-first-party-plugins.md",
    "docs/plan/07-runtime-and-integration.md",
    "docs/plan/08-security-and-delivery.md",
    "docs/features/00-market-browse-install.md",
    "docs/features/01-ustc-affairs-navigator.md",
    "docs/features/02-ustc-change-radar.md",
    "docs/features/03-campus-opportunity-graph.md",
    "docs/features/04-bounded-agent-harness.md",
    "docs/acceptance/gates.md",
    "docs/acceptance/matrix.tsv",
    "docs/acceptance/platform-baseline.md",
    "docs/acceptance/public-readiness.md",
    "docs/adr/0006-three-default-first-party-plugins.md",
    "docs/adr/0007-finite-agent-harness.md",
    "docs/adr/0008-agent-plugin-tool-boundary.md",
    "docs/adr/0009-dioxus-multi-client-shell.md",
    "docs/overview/architecture.md",
    "docs/tasks/01-execution-roadmap.md",
    "docs/guides/contributing.md",
    "docs/guides/development.md",
    "docs/guides/github-pages-brief.md",
    "docs/contracts/cli.md",
    "docs/contracts/agent-runtime.md",
    "docs/contracts/agent-harness.md",
    "docs/contracts/agent-plugin-boundary.md",
    "docs/contracts/client-shell.md",
    "docs/contracts/data-models.md",
    "docs/contracts/interfaces.md",
    "docs/contracts/invocation-resolution.md",
    "docs/contracts/permissions.md",
    "docs/contracts/plugin-package.md",
    "docs/contracts/source-import.md",
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

    registered = set(KEY_FILES)
    contracts_directory = ROOT / "docs/contracts"
    if contracts_directory.is_dir():
        for path in sorted(contracts_directory.glob("*.md")):
            rel = path.relative_to(ROOT).as_posix()
            if rel not in registered:
                fail(f"current contract not registered as key file: {rel}", issues)


def check_docs_topology(issues: list[str]) -> None:
    docs_root = ROOT / "docs"
    if not docs_root.is_dir():
        fail("docs root missing", issues)
        return

    actual_directories = {path.name for path in docs_root.iterdir() if path.is_dir()}
    actual_root_files = {path.name for path in docs_root.iterdir() if path.is_file()}
    if actual_directories != EXPECTED_DOC_DIRECTORIES:
        fail(
            "documentation directory topology drift: "
            f"expected={sorted(EXPECTED_DOC_DIRECTORIES)} actual={sorted(actual_directories)}",
            issues,
        )
    if actual_root_files != EXPECTED_DOC_ROOT_FILES:
        fail(
            "documentation root-file topology drift: "
            f"expected={sorted(EXPECTED_DOC_ROOT_FILES)} actual={sorted(actual_root_files)}",
            issues,
        )


def check_no_retired_docs_references(issues: list[str]) -> None:
    fixture_path = ROOT / "scripts/tests/test_check_repo_contracts.py"
    for path in ROOT.rglob("*"):
        if not path.is_file() or path == fixture_path:
            continue
        if any(part in {".git", "target", ".codegraph"} for part in path.parts):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for directory in RETIRED_DOC_DIRECTORIES:
            needle = f"docs/{directory}/"
            if needle in text:
                fail(f"retired documentation path reference in {path.relative_to(ROOT)}: {needle}", issues)


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


def check_invocation_fixtures(issues: list[str]) -> None:
    directory = ROOT / "crates/platform-core/tests/fixtures/invocation-resolution"
    actual = {path.name for path in directory.glob("*.json")} if directory.is_dir() else set()
    if actual != INVOCATION_FIXTURES:
        fail(
            "invocation-resolution fixture set drift: "
            f"expected={sorted(INVOCATION_FIXTURES)} actual={sorted(actual)}",
            issues,
        )
        return
    seen_case_names: set[str] = set()
    for name in sorted(INVOCATION_FIXTURES):
        path = directory / name
        actual_digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual_digest != INVOCATION_FIXTURE_DIGESTS[name]:
            fail(f"{name}: invocation fixture executable details drift", issues)
        fixture = load_json(
            f"crates/platform-core/tests/fixtures/invocation-resolution/{name}", issues
        )
        if not isinstance(fixture, dict):
            continue
        if fixture.get("schema_version") != "invocation-resolution-fixture/v0":
            fail(f"{name}: invocation fixture schema drift", issues)
        if fixture.get("synthetic") is not True or fixture.get("fixture") != name:
            fail(f"{name}: invocation fixture must remain exactly synthetic", issues)
        expected_top_level = {"schema_version", "synthetic", "fixture", "cases"}
        if set(fixture) != expected_top_level:
            fail(f"{name}: invocation fixture top-level fields drift", issues)
        cases = fixture.get("cases")
        if not isinstance(cases, list) or not cases:
            fail(f"{name}: invocation fixture cases must be a non-empty object list", issues)
            continue
        for case in cases:
            if not isinstance(case, dict):
                fail(f"{name}: invocation fixture case must be an object", issues)
                continue
            expected_fields = {"name", "api", "recipe", "expected", "precedence"}
            if set(case) != expected_fields:
                fail(f"{name}: invocation fixture case fields drift", issues)
                continue
            if not all(isinstance(case[field], str) and case[field] for field in expected_fields):
                fail(f"{name}: invocation fixture case fields must be non-empty strings", issues)
                continue
            if case["api"] not in {
                "schema_constructor",
                "argument_constructor",
                "resolve_projection",
                "authorize_call",
                "run_spec_mapping",
            }:
                fail(f"{name}: invocation fixture case API is unknown: {case['api']}", issues)
            if case["name"] in seen_case_names:
                fail(f"duplicate invocation fixture case name: {case['name']}", issues)
            seen_case_names.add(case["name"])

    matrix = (ROOT / "docs/acceptance/matrix.tsv").read_text(encoding="utf-8")
    for case_id in ("MARKET-005", "MARKET-006"):
        rows = [row for row in matrix.splitlines() if row.startswith(f"{case_id}\t")]
        if (
            len(rows) != 1
            or "\timplemented\t" not in rows[0]
            or INVOCATION_FIXTURE_TEST_COMMAND not in rows[0]
            or INVOCATION_COMPOSITION_FIXTURE_TEST_COMMAND not in rows[0]
        ):
            fail(f"{case_id}: implemented invocation binding/status drift", issues)


def check_agent_plugin_dependency_direction(issues: list[str]) -> None:
    agent_manifest_path = ROOT / "crates/agent-runtime/Cargo.toml"
    composition_manifest_path = ROOT / "apps/ustc-agentd/Cargo.toml"
    workspace_manifest_path = ROOT / "Cargo.toml"
    composition_test = ROOT / "apps/ustc-agentd/tests/resolved_run_spec.rs"
    misplaced_test = ROOT / "crates/agent-runtime/tests/resolved_run_spec.rs"

    try:
        agent_manifest = tomllib.loads(agent_manifest_path.read_text(encoding="utf-8"))
        composition_manifest = tomllib.loads(
            composition_manifest_path.read_text(encoding="utf-8")
        )
        workspace_manifest = tomllib.loads(workspace_manifest_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"Agent/Plugin dependency manifest unreadable: {error}", issues)
        return

    workspace = workspace_manifest.get("workspace", {})
    workspace_dependencies = workspace.get("dependencies", {}) if isinstance(workspace, dict) else {}
    if not isinstance(workspace_dependencies, dict):
        fail("workspace dependency table is not an object", issues)
        return
    for redirect_table in ("patch", "replace"):
        if workspace_manifest.get(redirect_table):
            fail(
                f"workspace Cargo {redirect_table} table is forbidden for Agent dependency proof",
                issues,
            )

    package_table = agent_manifest.get("package", {})
    library_table = agent_manifest.get("lib", {})
    if not isinstance(package_table, dict) or package_table.get("build") not in (None, False):
        fail("agent-runtime build script is forbidden", issues)
    if not isinstance(library_table, dict) or library_table.get("path") != "src/lib.rs":
        fail("agent-runtime library target must remain exactly src/lib.rs", issues)
    for target_kind in ("bin", "example", "bench", "test"):
        if target_kind in agent_manifest and agent_manifest[target_kind]:
            fail(f"agent-runtime explicit {target_kind} target is forbidden", issues)

    dependency_identities: set[tuple[str, str, str]] = set()

    def resolve_dependency(alias: str, specification: object) -> tuple[str, str]:
        resolved = specification
        if isinstance(specification, dict) and specification.get("workspace") is True:
            if any(key in specification for key in ("package", "path", "git")):
                return str(specification.get("package", alias)), "invalid-workspace-override"
            if alias not in workspace_dependencies:
                return alias, "missing-workspace-dependency"
            resolved = workspace_dependencies[alias]

        if isinstance(resolved, str):
            return alias, "registry"
        if not isinstance(resolved, dict):
            return alias, "unknown"

        package = str(resolved.get("package", alias))
        if "path" in resolved:
            return package, f"path:{resolved['path']}"
        if "git" in resolved:
            return package, "git"
        if "registry" in resolved:
            return package, f"registry:{resolved['registry']}"
        if "version" in resolved:
            return package, "registry"
        return package, "unknown"

    def collect_dependencies(values: object) -> None:
        if not isinstance(values, dict):
            return
        for alias, specification in values.items():
            alias = str(alias)
            package, source = resolve_dependency(alias, specification)
            dependency_identities.add((alias, package, source))

    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        collect_dependencies(agent_manifest.get(section, {}))
    target_sections = agent_manifest.get("target", {})
    if isinstance(target_sections, dict):
        for target in target_sections.values():
            if not isinstance(target, dict):
                continue
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                collect_dependencies(target.get(section, {}))

    unapproved_dependencies = sorted(
        f"{alias}->{package}@{source}"
        for alias, package, source in dependency_identities
        if AGENT_ALLOWED_DIRECT_DEPENDENCIES.get(alias) != (package, source)
    )
    if unapproved_dependencies:
        fail(
            "agent-runtime has unapproved direct dependencies: "
            f"{unapproved_dependencies}",
            issues,
        )
    else:
        try:
            completed = subprocess.run(
                [
                    "cargo",
                    "metadata",
                    "--locked",
                    "--offline",
                    "--no-deps",
                    "--format-version",
                    "1",
                ],
                cwd=ROOT,
                capture_output=True,
                check=False,
                text=True,
                timeout=30,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            fail(f"cargo metadata dependency resolution failed: {error}", issues)
        else:
            if completed.returncode != 0:
                diagnostic = completed.stderr.strip().splitlines()
                detail = diagnostic[-1] if diagnostic else "unknown cargo metadata error"
                fail(f"cargo metadata dependency resolution failed: {detail}", issues)
            else:
                try:
                    metadata = json.loads(completed.stdout)
                except json.JSONDecodeError as error:
                    fail(f"cargo metadata output is malformed: {error}", issues)
                else:
                    packages = metadata.get("packages", [])
                    matching_packages = [
                        package
                        for package in packages
                        if isinstance(package, dict)
                        and package.get("name") == "ustc-campus-agent-runtime"
                        and Path(str(package.get("manifest_path", ""))).resolve()
                        == agent_manifest_path.resolve()
                    ]
                    if len(matching_packages) != 1:
                        fail(
                            "cargo metadata must resolve exactly one agent-runtime package",
                            issues,
                        )
                        matching_packages = []

                    resolved_dependencies: set[tuple[str, str]] = set()
                    if matching_packages:
                        dependencies = matching_packages[0].get("dependencies", [])
                        if not isinstance(dependencies, list):
                            fail("cargo metadata dependencies are malformed", issues)
                            dependencies = []
                        for dependency in dependencies:
                            if not isinstance(dependency, dict):
                                fail("cargo metadata dependency entry is malformed", issues)
                                continue
                            package_name = str(dependency.get("name", ""))
                            path_value = dependency.get("path")
                            source_detail = dependency.get("source")
                            if path_value is not None:
                                try:
                                    relative_source = (
                                        Path(str(path_value))
                                        .resolve()
                                        .relative_to(ROOT.resolve())
                                        .as_posix()
                                    )
                                except (OSError, ValueError):
                                    source = f"external:{path_value}"
                                else:
                                    source = f"path:{relative_source}"
                            elif isinstance(source_detail, str) and source_detail.startswith(
                                "registry+"
                            ):
                                source = "registry"
                            else:
                                source = f"external:{source_detail}"
                            resolved_dependencies.add((package_name, source))

                    expected_resolved = set(AGENT_ALLOWED_DIRECT_DEPENDENCIES.values())
                    unexpected_resolved = sorted(resolved_dependencies - expected_resolved)
                    if unexpected_resolved:
                        fail(
                            "agent-runtime resolved direct dependency is not allowlisted: "
                            f"{unexpected_resolved}",
                            issues,
                        )
                    declared_packages = {package for _, package, _ in dependency_identities}
                    resolved_packages = {package for package, _ in resolved_dependencies}
                    if declared_packages != resolved_packages:
                        fail(
                            "agent-runtime declared/resolved direct dependency mismatch: "
                            f"declared={sorted(declared_packages)} "
                            f"resolved={sorted(resolved_packages)}",
                            issues,
                        )

    cargo_config_paths = [
        directory / filename
        for directory in (ROOT, ROOT / "crates", ROOT / "crates/agent-runtime")
        for filename in (Path(".cargo/config.toml"), Path(".cargo/config"))
    ]
    for config_path in cargo_config_paths:
        if config_path.exists():
            fail(
                "repository Cargo config is forbidden for Agent dependency proof: "
                f"{config_path.relative_to(ROOT).as_posix()}",
                issues,
            )

    source_root = ROOT / "crates/agent-runtime"
    if source_root.is_symlink():
        fail("agent-runtime crate root must not be a symlink", issues)
    if source_root.is_dir():
        for path in sorted(source_root.rglob("*")):
            if path.is_symlink():
                fail(
                    "agent-runtime source tree contains a symlink escape: "
                    f"{path.relative_to(ROOT).as_posix()}",
                    issues,
                )
        for forbidden_entry in (
            source_root / "build.rs",
            source_root / "src/main.rs",
            source_root / "src/bin",
            source_root / "examples",
            source_root / "benches",
        ):
            if forbidden_entry.exists():
                fail(
                    "agent-runtime extra compilation target is forbidden: "
                    f"{forbidden_entry.relative_to(ROOT).as_posix()}",
                    issues,
                )
        for path in sorted(source_root.rglob("*.rs")):
            text = path.read_text(encoding="utf-8")
            for marker in sorted(AGENT_FORBIDDEN_SOURCE_MARKERS):
                if marker in text:
                    fail(
                        "agent-runtime source crosses the Agent/Plugin boundary: "
                        f"{path.relative_to(ROOT).as_posix()} contains {marker}",
                        issues,
                    )
            for pattern, description in AGENT_FORBIDDEN_SOURCE_PATTERNS.items():
                if re.search(pattern, text):
                    fail(
                        "agent-runtime source crosses the compilation boundary: "
                        f"{path.relative_to(ROOT).as_posix()} uses {description}",
                        issues,
                    )
            conditional_text = ALLOWED_AGENT_TEST_CFG.sub("", text)
            if re.search(r"\bcfg\b", conditional_text):
                fail(
                    "agent-runtime source crosses the compilation boundary: "
                    f"{path.relative_to(ROOT).as_posix()} uses unsupported cfg",
                    issues,
                )

        with tempfile.TemporaryDirectory(prefix="agent-boundary-") as directory:
            for probe_name, probe_arguments in (("library", []), ("test", ["--test"])):
                dep_info_path = Path(directory) / f"agent-runtime-{probe_name}.d"
                try:
                    completed = subprocess.run(
                        [
                            "rustc",
                            "--edition",
                            "2024",
                            "--crate-name",
                            f"agent_boundary_{probe_name}_probe",
                            "--crate-type",
                            "lib",
                            *probe_arguments,
                            "--emit",
                            f"dep-info={dep_info_path}",
                            "crates/agent-runtime/src/lib.rs",
                        ],
                        cwd=ROOT,
                        capture_output=True,
                        check=False,
                        text=True,
                        timeout=30,
                    )
                except (OSError, subprocess.TimeoutExpired) as error:
                    fail(f"rustc {probe_name} dependency discovery failed: {error}", issues)
                    continue
                if not dep_info_path.is_file():
                    diagnostic = completed.stderr.strip().splitlines()
                    detail = diagnostic[-1] if diagnostic else "rustc emitted no dep-info"
                    fail(f"rustc {probe_name} dependency discovery failed: {detail}", issues)
                    continue

                compilation_inputs: set[Path] = set()
                for line in dep_info_path.read_text(encoding="utf-8").splitlines():
                    if not line or line.startswith("#"):
                        continue
                    if ": " in line:
                        dependencies = line.split(": ", 1)[1]
                    elif line.endswith(":"):
                        dependencies = line[:-1]
                    else:
                        fail(
                            f"rustc {probe_name} dep-info contains an unparsed line: {line!r}",
                            issues,
                        )
                        continue
                    try:
                        tokens = shlex.split(dependencies)
                    except ValueError as error:
                        fail(
                            f"rustc {probe_name} dep-info path parsing failed: {error}",
                            issues,
                        )
                        continue
                    for token in tokens:
                        path = Path(token.replace("$$", "$"))
                        compilation_inputs.add(
                            path.resolve() if path.is_absolute() else (ROOT / path).resolve()
                        )
                resolved_source_root = source_root.resolve()
                escaped_inputs = sorted(
                    path.as_posix()
                    for path in compilation_inputs
                    if not path.is_relative_to(resolved_source_root)
                )
                if escaped_inputs:
                    fail(
                        f"agent-runtime rustc {probe_name} dep-info escapes the owned crate tree: "
                        f"{escaped_inputs}",
                        issues,
                    )
                expected_entrypoint = (source_root / "src/lib.rs").resolve()
                if expected_entrypoint not in compilation_inputs:
                    fail(f"rustc {probe_name} dep-info omitted the Agent entrypoint", issues)

    composition_dependencies = composition_manifest.get("dependencies", {})
    if not isinstance(composition_dependencies, dict) or not {
        "ustc-campus-agent-runtime",
        "ustc-campus-agent-core",
    }.issubset(composition_dependencies):
        fail("ustc-agentd must remain the Agent/Plugin composition root", issues)
    if misplaced_test.exists():
        fail("cross-boundary proof must not be owned by agent-runtime", issues)
    if not composition_test.is_file():
        fail("composition-root resolved_run_spec proof is missing", issues)


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
        status = cols[5]
        if status not in VALID_ACCEPTANCE_STATUSES:
            fail(f"unknown acceptance status {status!r} in {case_id}", issues)
    if len(seen) < 10:
        fail("acceptance matrix too small for current contract", issues)


def check_acceptance_catalog(issues: list[str]) -> None:
    catalog_path = ROOT / "docs/acceptance/platform-baseline.md"
    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    catalog_ids = re.findall(
        r"^\| `([A-Z0-9]+-[0-9]+)` \|",
        catalog_path.read_text(encoding="utf-8"),
        flags=re.MULTILINE,
    )
    duplicates = sorted(
        case_id for case_id in set(catalog_ids) if catalog_ids.count(case_id) > 1
    )
    if duplicates:
        fail(f"duplicate long-horizon acceptance case IDs: {duplicates}", issues)
    if len(catalog_ids) < MIN_LONG_HORIZON_CASES:
        fail(
            "long-horizon acceptance catalog unexpectedly shrank: "
            f"expected>={MIN_LONG_HORIZON_CASES} actual={len(catalog_ids)}",
            issues,
        )

    active_rows = matrix_path.read_text(encoding="utf-8").splitlines()[1:]
    for row in active_rows:
        if not row.strip():
            continue
        case_id = row.split("\t", 1)[0]
        prefix = case_id.split("-", 1)[0]
        if prefix in STABLE_CATALOG_PREFIXES and case_id not in catalog_ids:
            fail(f"active case missing from long-horizon catalog: {case_id}", issues)


def main() -> int:
    issues: list[str] = []
    check_key_files_present_and_nonempty(issues)
    check_docs_topology(issues)
    check_no_retired_docs_references(issues)
    check_markdown_links(issues)
    check_no_obvious_secrets(issues)
    check_market(issues)
    check_course_fixture(issues)
    check_invocation_fixtures(issues)
    check_agent_plugin_dependency_direction(issues)
    check_acceptance_matrix(issues)
    check_acceptance_catalog(issues)
    if issues:
        print("contract-check: FAIL")
        for issue in issues:
            print(f"- {issue}")
        return 1
    print("contract-check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
