#!/usr/bin/env python3
"""Repository contract checks for USTC Campus Agent.

This intentionally uses only the Python standard library so it can run in CI
before the project chooses additional tooling.
"""

from __future__ import annotations

import ast
import hashlib
import json
import re
import shlex
import subprocess
import sys
import tempfile
import tomllib
import unittest
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALID_GATES = {"pr", "core-demo", "release", "public"}
VALID_ACCEPTANCE_STATUSES = {"planned", "implemented"}
STABLE_CATALOG_PREFIXES = {"AGENT", "AUTH", "FP", "HARNESS", "PKG", "PROC", "SRC"}
MIN_LONG_HORIZON_CASES = 200
VALID_MODULE_STATES = {
    "planned",
    "skeleton",
    "partial-evidence",
    "design-only",
    "bounded-spike",
    "governance-baseline",
}
MODULE_BLUEPRINTS = {
    "M00": "docs/plan/modules/10-platform-control-identity.md",
    "M10": "docs/plan/modules/20-application-api-host.md",
    "M20": "docs/plan/modules/30-market-package-lifecycle.md",
    "M30": "docs/plan/modules/40-agent-harness-runtime.md",
    "M40": "docs/plan/modules/50-tool-gateway-execution.md",
    "M50": "docs/plan/modules/60-model-provider-integration.md",
    "M51": "docs/plan/modules/61-mcp-binding-executor.md",
    "M60": "docs/plan/modules/70-campus-trust-source-pipeline.md",
    "M70": "docs/plan/modules/71-change-radar.md",
    "M71": "docs/plan/modules/72-affairs-navigator.md",
    "M72": "docs/plan/modules/73-opportunity-graph.md",
    "M80": "docs/plan/modules/80-dioxus-multi-client.md",
    "M90": "docs/plan/modules/90-infrastructure-operations.md",
}
S0_REVIEW_PATH = "docs/tasks/02-s0-architecture-review.md"
S0_REVIEW_LANES = {"architecture", "authority", "delivery"}
S0_REVIEW_DECISION_IDS = {
    "S0-A01",
    "S0-A02",
    "S0-A03",
    "S0-A04",
    "S0-A05",
    "S0-A06",
    "S0-A07",
    "S0-A08",
} | {f"S0-{module_id}" for module_id in MODULE_BLUEPRINTS}
VALID_S0_REVIEW_STATUSES = {"InReview", "Complete"}
VALID_S0_REVIEW_OUTCOMES = {"Pending", "Pass", "Conditional", "Reject"}
VALID_S0_REVIEW_DISPOSITIONS = {"Pending", "Accept", "ConditionalAccept", "Reject"}
S0_COMPLETE_REVIEW_LANES_CELL = "`architecture`; `authority`; `delivery`"
CAMPAIGN_AUTHORIZATION_POLICY_PATH = "docs/tasks/00-module-work-policy.md"
CAMPAIGN_AUTHORIZATION_POLICY_BEGIN = "<!-- CAMPAIGN_AUTHORIZATION_POLICY:BEGIN -->"
CAMPAIGN_AUTHORIZATION_POLICY_END = "<!-- CAMPAIGN_AUTHORIZATION_POLICY:END -->"
CAMPAIGN_AUTHORIZATION_POLICY_SHA256 = (
    "c93f351de3c5ae9735c1c4c97194735bc60f0e59ec4dc5992607c81e3151db6c"
)
AUTONOMOUS_CAMPAIGN_GRANT_PATH = "docs/tasks/01-execution-roadmap.md"
AUTONOMOUS_CAMPAIGN_GRANT_BEGIN = "<!-- AUTONOMOUS_CAMPAIGN_GRANT:BEGIN -->"
AUTONOMOUS_CAMPAIGN_GRANT_END = "<!-- AUTONOMOUS_CAMPAIGN_GRANT:END -->"
AUTONOMOUS_CAMPAIGN_GRANT_SHA256 = (
    "aafea72513ad63226eba21587a68b54b7d2a4525c54a9229241f2c6d5042bcc2"
)
AUTONOMOUS_CAMPAIGN_ID = "USTC-MODULES-2026-07-W1"
AUTONOMOUS_CAMPAIGN_TASKBOOKS = {
    "M00-B3": "docs/tasks/campaign-w1-m00-b3.md",
    "M20-B6": "docs/tasks/campaign-w1-m20-b6.md",
    "M30-B0": "docs/tasks/campaign-w1-m30-b0.md",
    "M40-B0": "docs/tasks/campaign-w1-m40-b0.md",
}
AUTONOMOUS_CAMPAIGN_MATRIX_BINDINGS = {
    "MARKET-004": "planned",
    "PKG-020": "planned",
    "HARNESS-001": "planned",
    "HARNESS-003": "planned",
    "AGENT-017": "implemented",
    "AGENT-018": "planned",
}
AUTONOMOUS_CAMPAIGN_CATALOG_ONLY_IDS = (
    "HARNESS-002",
    "AGENT-003",
    "AGENT-004",
    "AGENT-009",
    "AGENT-010",
    "AGENT-011",
    "AGENT-012",
    "AGENT-013",
)
AUTONOMOUS_CAMPAIGN_CATALOG_PATH = "docs/acceptance/platform-baseline.md"
CAMPAIGN_CI_WORKFLOW_PATH = ".github/workflows/ci.yml"
CAMPAIGN_CI_WORKFLOW_SHA256 = (
    "919080325ade109dab32b556cbc97fb3fcd5844e45ad72e3b74ad231cb669146"
)
VALID_CAMPAIGN_LANE_STATUSES = {"queued", "active", "paused", "completed", "rejected"}
S0_REVIEW_AUTHORITY_LINKS = (
    "../../AGENTS.md",
    "../plan/00-engineering-constitution.md",
    "../plan/01-terminology.md",
    "../plan/modules/00-module-map.md",
    "../contracts/module-boundaries.md",
    "00-module-work-policy.md",
    "01-execution-roadmap.md",
)
S0_REVIEW_READING_CHAIN = """repository AGENTS, engineering constitution and terminology
→ module map and all 13 module blueprints
→ module-boundary registry and specific contracts
→ coverage matrix
→ active acceptance matrix and long-horizon catalog
→ module work policy and execution roadmap
→ retained code/tests claimed as bounded evidence"""
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
    "design",
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
    "docs/design/AGENTS.md",
    "docs/design/README.md",
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
    "docs/plan/modules/00-module-map.md",
    "docs/plan/modules/10-platform-control-identity.md",
    "docs/plan/modules/20-application-api-host.md",
    "docs/plan/modules/30-market-package-lifecycle.md",
    "docs/plan/modules/40-agent-harness-runtime.md",
    "docs/plan/modules/50-tool-gateway-execution.md",
    "docs/plan/modules/60-model-provider-integration.md",
    "docs/plan/modules/61-mcp-binding-executor.md",
    "docs/plan/modules/70-campus-trust-source-pipeline.md",
    "docs/plan/modules/71-change-radar.md",
    "docs/plan/modules/72-affairs-navigator.md",
    "docs/plan/modules/73-opportunity-graph.md",
    "docs/plan/modules/80-dioxus-multi-client.md",
    "docs/plan/modules/90-infrastructure-operations.md",
    "docs/features/00-market-browse-install.md",
    "docs/features/01-ustc-affairs-navigator.md",
    "docs/features/02-ustc-change-radar.md",
    "docs/features/03-campus-opportunity-graph.md",
    "docs/features/04-bounded-agent-harness.md",
    "docs/features/05-headless-client-and-agent-integration.md",
    "docs/acceptance/gates.md",
    "docs/acceptance/matrix.tsv",
    "docs/acceptance/platform-baseline.md",
    "docs/acceptance/public-readiness.md",
    "docs/adr/0006-three-default-first-party-plugins.md",
    "docs/adr/0007-finite-agent-harness.md",
    "docs/adr/0008-agent-plugin-tool-boundary.md",
    "docs/adr/0009-dioxus-multi-client-shell.md",
    "docs/adr/0010-typed-client-peer-adapters.md",
    "docs/overview/architecture.md",
    "docs/tasks/00-module-work-policy.md",
    "docs/tasks/01-execution-roadmap.md",
    "docs/tasks/02-s0-architecture-review.md",
    "docs/tasks/campaign-w1-m00-b3.md",
    "docs/tasks/campaign-w1-m20-b6.md",
    "docs/tasks/campaign-w1-m30-b0.md",
    "docs/tasks/campaign-w1-m40-b0.md",
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
    "docs/contracts/market-lifecycle.md",
    "docs/contracts/module-boundaries.md",
    "docs/contracts/permissions.md",
    "docs/contracts/platform-identity.md",
    "docs/contracts/platform-session.md",
    "docs/contracts/plugin-package.md",
    "docs/contracts/source-import.md",
    "docs/contracts/source-retrieval.md",
    "market/review-policy/first-party.md",
    "market/fixtures/course-planning/README.md",
    "market/fixtures/course-planning/minimal-v0.json",
]

EXTERNAL_AGENT_OPERATION_ROWS = {
    "server.info": ("M10", "public-read", "read", "CLI, HTTP", "planned first protocol slice"),
    "capability.list": ("M10", "public-read", "read", "CLI, HTTP", "planned first protocol slice"),
    "market.package.list": ("M20", "public-read", "read", "CLI, HTTP, inbound MCP", "planned first vertical slice"),
    "market.package.get": ("M20", "public-read", "read", "CLI, HTTP", "planned"),
    "affairs.search": ("M71", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "affairs.get": ("M71", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "change.list": ("M70", "public-read", "read", "CLI, HTTP", "planned after owning product contract"),
    "change.get": ("M70", "public-read", "read", "CLI, HTTP", "planned after owning product contract"),
    "program.list": ("M72", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "program.get": ("M72", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "course.search": ("M72", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "course.get": ("M72", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "offering.list": ("M72", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "course.review_linkout": ("M72", "public-linkout", "link-out", "CLI, HTTP, inbound MCP", "planned after owning product contract"),
    "source.provenance": ("M60", "public-read", "read", "CLI, HTTP, inbound MCP", "planned after owning source/product contract"),
    "profile.requirement_status": ("M72", "tenant-private-read", "read", "CLI, HTTP, later inbound MCP", "later private-data slice"),
    "planner.draft.list": ("M72", "tenant-private-read", "read", "CLI, HTTP", "later private-data slice"),
    "planner.draft.get": ("M72", "tenant-private-read", "read", "CLI, HTTP", "later private-data slice"),
    "planner.draft.delete": ("M72", "tenant-private-write", "tenant-local mutation", "CLI, HTTP", "later private-draft slice"),
    "planner.generate": ("M72", "tenant-private-write", "tenant-local draft mutation", "CLI, HTTP, later inbound MCP", "later private-draft slice"),
    "planner.explain": ("M72", "tenant-private-read", "read", "CLI, HTTP, later inbound MCP", "later private-data slice"),
}

EXTERNAL_AGENT_CONTRACT_FILES = (
    "docs/contracts/interfaces.md",
    "docs/contracts/permissions.md",
    "docs/contracts/client-shell.md",
    "docs/contracts/cli.md",
    "docs/features/05-headless-client-and-agent-integration.md",
    "docs/plan/modules/20-application-api-host.md",
    "docs/plan/modules/80-dioxus-multi-client.md",
    "docs/tasks/01-execution-roadmap.md",
)


def fail(msg: str, issues: list[str]) -> None:
    issues.append(msg)


def markdown_cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def parse_markdown_table(
    rel: str,
    expected_header: list[str],
    label: str,
    issues: list[str],
) -> list[tuple[int, list[str]]]:
    path = ROOT / rel
    if not path.is_file():
        fail(f"{label} missing: {rel}", issues)
        return []
    lines = path.read_text(encoding="utf-8").splitlines()
    header_indexes = [
        index
        for index, line in enumerate(lines)
        if line.startswith("|") and markdown_cells(line) == expected_header
    ]
    if len(header_indexes) != 1:
        fail(
            f"{label} header drift: expected one {expected_header!r}, found {len(header_indexes)}",
            issues,
        )
        return []
    header_index = header_indexes[0]
    if header_index + 1 >= len(lines):
        fail(f"{label} separator missing", issues)
        return []
    separator = markdown_cells(lines[header_index + 1])
    if len(separator) != len(expected_header) or not all(
        re.fullmatch(r":?-{3,}:?", cell) for cell in separator
    ):
        fail(f"{label} separator drift: {separator!r}", issues)
        return []

    rows: list[tuple[int, list[str]]] = []
    for index in range(header_index + 2, len(lines)):
        line = lines[index]
        if not line.startswith("|"):
            break
        cells = markdown_cells(line)
        if len(cells) != len(expected_header):
            fail(
                f"{label} row {index + 1} has {len(cells)} columns; "
                f"expected {len(expected_header)}",
                issues,
            )
            continue
        rows.append((index + 1, cells))
    if not rows:
        fail(f"{label} has no rows", issues)
    return rows


def markdown_code_value(cell: str) -> str | None:
    match = re.fullmatch(r"`([^`]+)`", cell)
    return match.group(1) if match else None


def module_id_from_cell(cell: str) -> str | None:
    value = markdown_code_value(cell) or cell
    match = re.match(r"^`?(M\d{2})`?(?:\s|$)", value)
    return match.group(1) if match else None


def collect_module_rows(
    rows: list[tuple[int, list[str]]],
    id_column: int,
    state_column: int | None,
    label: str,
    issues: list[str],
) -> tuple[dict[str, str], dict[str, list[str]]]:
    states: dict[str, str] = {}
    values: dict[str, list[str]] = {}
    for line_no, cells in rows:
        module_id = module_id_from_cell(cells[id_column])
        if module_id is None:
            fail(f"{label} row {line_no} has invalid module ID cell: {cells[id_column]!r}", issues)
            continue
        if module_id in values:
            fail(f"duplicate module ID in {label}: {module_id}", issues)
            continue
        values[module_id] = cells
        if state_column is not None:
            state = markdown_code_value(cells[state_column])
            if state is None or state not in VALID_MODULE_STATES:
                fail(
                    f"{label} row {line_no} has unknown state key for {module_id}: "
                    f"{cells[state_column]!r}",
                    issues,
                )
                continue
            states[module_id] = state
    return states, values


def check_exact_module_ids(label: str, actual: set[str], issues: list[str]) -> None:
    expected = set(MODULE_BLUEPRINTS)
    if actual != expected:
        fail(
            f"{label} module ID set drift: missing={sorted(expected - actual)} "
            f"unexpected={sorted(actual - expected)}",
            issues,
        )


def parse_acceptance_projection(
    cell: str, module_id: str, issues: list[str]
) -> list[tuple[str, str | None]]:
    entries: list[tuple[str, str | None]] = []
    seen_references: set[str] = set()
    seen_gap = False
    for raw_token in cell.split(";"):
        token = markdown_code_value(raw_token.strip())
        if token is None:
            fail(
                f"module coverage acceptance projection has an unstructured token for "
                f"{module_id}: {raw_token.strip()!r}",
                issues,
            )
            continue
        if token == "gap":
            if seen_gap:
                fail(f"duplicate acceptance gap token for {module_id}", issues)
            seen_gap = True
            entries.append(("gap", None))
            continue
        match = re.fullmatch(
            r"(active|long-horizon):([A-Z][A-Z0-9]+)-(\*|[0-9]+)", token
        )
        if match is None:
            fail(
                f"module coverage acceptance projection has an invalid token for "
                f"{module_id}: {token!r}",
                issues,
            )
            continue
        posture, prefix, suffix = match.groups()
        reference = f"{prefix}-{suffix}"
        if reference in seen_references:
            fail(
                f"duplicate acceptance reference in module coverage for "
                f"{module_id}: {reference}",
                issues,
            )
            continue
        seen_references.add(reference)
        entries.append((posture, reference))
    if not entries:
        fail(f"module coverage acceptance projection is empty for {module_id}", issues)
    return entries


def check_module_registry(issues: list[str]) -> None:
    map_rows = parse_markdown_table(
        "docs/plan/modules/00-module-map.md",
        ["ID", "Large module", "State key", "Owns", "Must not own", "Current state"],
        "module map registry",
        issues,
    )
    map_states, map_values = collect_module_rows(map_rows, 0, 2, "module map", issues)
    check_exact_module_ids("module map", set(map_values), issues)

    roadmap_rows = parse_markdown_table(
        "docs/tasks/01-execution-roadmap.md",
        ["Module", "State key", "Current state", "Current module target", "Owner", "Merge gate"],
        "module roadmap lane registry",
        issues,
    )
    roadmap_states, roadmap_values = collect_module_rows(
        roadmap_rows, 0, 1, "module roadmap", issues
    )
    check_exact_module_ids("module roadmap", set(roadmap_values), issues)

    coverage_rows = parse_markdown_table(
        "docs/coverage-matrix.md",
        [
            "Module blueprint",
            "Primary public boundary",
            "Feature projection",
            "Acceptance projection",
        ],
        "module coverage matrix",
        issues,
    )
    coverage_module_rows: list[tuple[int, list[str]]] = []
    for line_no, cells in coverage_rows:
        if module_id_from_cell(cells[0]) is not None:
            coverage_module_rows.append((line_no, cells))
        elif markdown_code_value(cells[0]) != "modules/00-module-map":
            fail(
                f"module coverage row {line_no} has unknown non-module blueprint cell: "
                f"{cells[0]!r}",
                issues,
            )
    _, coverage_values = collect_module_rows(
        coverage_module_rows, 0, None, "module coverage", issues
    )
    check_exact_module_ids("module coverage", set(coverage_values), issues)

    blueprint_states: dict[str, str] = {}
    expected_paths = set(MODULE_BLUEPRINTS.values())
    modules_root = ROOT / "docs/plan/modules"
    actual_paths = {
        path.relative_to(ROOT).as_posix()
        for path in modules_root.glob("*.md")
        if path.name != "00-module-map.md"
    }
    if actual_paths != expected_paths:
        fail(
            "module blueprint path set drift: "
            f"missing={sorted(expected_paths - actual_paths)} "
            f"unexpected={sorted(actual_paths - expected_paths)}",
            issues,
        )

    for expected_id, rel in MODULE_BLUEPRINTS.items():
        path = ROOT / rel
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        id_values = re.findall(r"^- `Module ID`: `([^`]+)`$", text, flags=re.MULTILINE)
        state_values = re.findall(
            r"^- `Implementation State`: `([^`]+)`$", text, flags=re.MULTILINE
        )
        if id_values != [expected_id]:
            fail(
                f"module blueprint ID drift in {rel}: expected={expected_id!r} "
                f"actual={id_values!r}",
                issues,
            )
        if len(state_values) != 1 or state_values[0] not in VALID_MODULE_STATES:
            fail(
                f"module blueprint state drift in {rel}: actual={state_values!r}",
                issues,
            )
        else:
            blueprint_states[expected_id] = state_values[0]

    for module_id in MODULE_BLUEPRINTS:
        states = {
            "map": map_states.get(module_id),
            "blueprint": blueprint_states.get(module_id),
            "roadmap": roadmap_states.get(module_id),
        }
        present_states = {state for state in states.values() if state is not None}
        if len(present_states) > 1:
            fail(
                f"module implementation state drift for {module_id}: "
                + " ".join(f"{source}={state!r}" for source, state in states.items()),
                issues,
            )

    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    catalog_path = ROOT / "docs/acceptance/platform-baseline.md"
    if not matrix_path.is_file() or not catalog_path.is_file():
        fail("module acceptance sources are missing", issues)
        return
    active_ids = {
        row.split("\t", 1)[0]
        for row in matrix_path.read_text(encoding="utf-8").splitlines()[1:]
        if row.strip()
    }
    catalog_ids = set(
        re.findall(
            r"^\| `([A-Z0-9]+-[0-9]+)` \|",
            catalog_path.read_text(encoding="utf-8"),
            flags=re.MULTILINE,
        )
    )
    active_prefixes = {case_id.split("-", 1)[0] for case_id in active_ids}
    catalog_prefixes = {case_id.split("-", 1)[0] for case_id in catalog_ids}
    for module_id, cells in coverage_values.items():
        for posture, reference in parse_acceptance_projection(cells[3], module_id, issues):
            if posture == "gap":
                continue
            assert reference is not None
            prefix, suffix = reference.split("-", 1)
            if suffix == "*":
                in_active = prefix in active_prefixes
                in_catalog = prefix in catalog_prefixes
            else:
                in_active = reference in active_ids
                in_catalog = reference in catalog_ids
            if posture == "active" and not in_active:
                fail(
                    f"active acceptance reference is not registered in matrix.tsv for "
                    f"{module_id}: {reference}",
                    issues,
                )
            elif posture == "long-horizon" and (not in_catalog or in_active):
                fail(
                    f"long-horizon acceptance reference is not catalog-only for "
                    f"{module_id}: {reference}",
                    issues,
                )


def check_s0_architecture_review(issues: list[str]) -> None:
    review_path = ROOT / S0_REVIEW_PATH
    roadmap_path = ROOT / "docs/tasks/01-execution-roadmap.md"
    if not review_path.is_file() or not roadmap_path.is_file():
        fail("S0 architecture review sources are missing", issues)
        return

    review_text = review_path.read_text(encoding="utf-8")
    roadmap_text = roadmap_path.read_text(encoding="utf-8")
    authority_lines = re.findall(
        r"^- `Authority Defers To`: (.+)$", review_text, flags=re.MULTILINE
    )
    if len(authority_lines) != 1:
        fail(
            "S0 architecture review authority chain is missing or duplicated: "
            f"found {len(authority_lines)}",
            issues,
        )
    else:
        authority_links = tuple(
            re.findall(r"\[[^\]]+\]\(([^)]+)\)", authority_lines[0])
        )
        if authority_links != S0_REVIEW_AUTHORITY_LINKS:
            fail(
                "S0 architecture review authority chain drift: "
                f"expected={S0_REVIEW_AUTHORITY_LINKS!r} actual={authority_links!r}",
                issues,
            )
    reading_chain_blocks = re.findall(
        r"^### Authority reading chain\n\n```text\n(.*?)\n```$",
        review_text,
        flags=re.MULTILINE | re.DOTALL,
    )
    if len(reading_chain_blocks) != 1:
        fail(
            "S0 architecture review reading chain is missing or duplicated: "
            f"found {len(reading_chain_blocks)}",
            issues,
        )
    elif reading_chain_blocks[0] != S0_REVIEW_READING_CHAIN:
        fail("S0 architecture review reading chain drift", issues)
    reading_chain_occurrences = review_text.count(S0_REVIEW_READING_CHAIN)
    if reading_chain_occurrences != 1:
        fail(
            "S0 architecture review reading chain occurrence drift: "
            f"expected 1 actual {reading_chain_occurrences}",
            issues,
        )

    status_values = re.findall(
        r"^- `Status`: `([^`]+)`$", review_text, flags=re.MULTILINE
    )
    if len(status_values) != 1 or status_values[0] not in VALID_S0_REVIEW_STATUSES:
        fail(f"S0 architecture review status is invalid: {status_values!r}", issues)
        return
    packet_status = status_values[0]

    roadmap_status_match = re.search(
        r"^### `S0-3` Team review\n\n\*\*Status\*\*: (pending|complete)\.$",
        roadmap_text,
        flags=re.MULTILINE,
    )
    if roadmap_status_match is None:
        fail("S0-3 roadmap status is missing or malformed", issues)
        return
    roadmap_status = roadmap_status_match.group(1)
    expected_roadmap_status = "complete" if packet_status == "Complete" else "pending"
    if roadmap_status != expected_roadmap_status:
        fail(
            "S0 architecture review packet/roadmap status drift: "
            f"packet={packet_status!r} roadmap={roadmap_status!r}",
            issues,
        )

    lane_rows = parse_markdown_table(
        S0_REVIEW_PATH,
        ["Lane ID", "Scope", "Outcome", "Blocking conditions"],
        "S0 architecture review lanes",
        issues,
    )
    lane_outcomes: dict[str, str] = {}
    for line_no, cells in lane_rows:
        lane_id = markdown_code_value(cells[0])
        outcome = markdown_code_value(cells[2])
        if lane_id is None or lane_id not in S0_REVIEW_LANES:
            fail(f"S0 review lane row {line_no} has unknown lane ID: {cells[0]!r}", issues)
            continue
        if lane_id in lane_outcomes:
            fail(f"duplicate S0 review lane: {lane_id}", issues)
            continue
        if outcome is None or outcome not in VALID_S0_REVIEW_OUTCOMES:
            fail(
                f"S0 review lane {lane_id} has invalid outcome: {cells[2]!r}",
                issues,
            )
            continue
        blockers_are_empty = cells[3].strip() == "—"
        if outcome in {"Pending", "Pass"} and not blockers_are_empty:
            fail(
                f"S0 review lane {lane_id} outcome {outcome} must not carry "
                "blocking conditions",
                issues,
            )
        if outcome in {"Conditional", "Reject"} and blockers_are_empty:
            fail(
                f"S0 review lane {lane_id} outcome {outcome} requires "
                "blocking conditions",
                issues,
            )
        lane_outcomes[lane_id] = outcome

    actual_lanes = set(lane_outcomes)
    if actual_lanes != S0_REVIEW_LANES:
        fail(
            "S0 review lane set drift: "
            f"missing={sorted(S0_REVIEW_LANES - actual_lanes)} "
            f"unexpected={sorted(actual_lanes - S0_REVIEW_LANES)}",
            issues,
        )

    decision_rows = parse_markdown_table(
        S0_REVIEW_PATH,
        [
            "Decision ID",
            "Scope",
            "Disposition",
            "Review lanes",
            "Basis",
            "Condition owner",
            "Required evidence",
            "Exit condition",
            "Resolution",
        ],
        "S0 architecture decision ledger",
        issues,
    )
    decisions: dict[str, tuple[str, str]] = {}
    for line_no, cells in decision_rows:
        decision_id = markdown_code_value(cells[0])
        disposition = markdown_code_value(cells[2])
        resolution = markdown_code_value(cells[8])
        if decision_id is None or decision_id not in S0_REVIEW_DECISION_IDS:
            fail(
                f"S0 decision row {line_no} has unknown decision ID: {cells[0]!r}",
                issues,
            )
            continue
        if decision_id in decisions:
            fail(f"duplicate S0 architecture decision: {decision_id}", issues)
            continue
        if disposition is None or disposition not in VALID_S0_REVIEW_DISPOSITIONS:
            fail(
                f"S0 decision {decision_id} has invalid disposition: {cells[2]!r}",
                issues,
            )
            continue
        if resolution not in {"open", "closed"}:
            fail(
                f"S0 decision {decision_id} has invalid resolution: {cells[8]!r}",
                issues,
            )
            continue
        if cells[1].strip() in {"", "—"} or cells[4].strip() in {"", "—"}:
            fail(f"S0 decision {decision_id} requires scope and basis", issues)

        condition_cells_are_empty = all(
            cells[index].strip() == "—" for index in (5, 6, 7)
        )
        condition_cells_are_complete = all(
            cells[index].strip() not in {"", "—"} for index in (5, 6, 7)
        )
        if disposition == "Pending":
            if (
                cells[3].strip() != "—"
                or not condition_cells_are_empty
                or resolution != "open"
            ):
                fail(
                    f"pending S0 decision {decision_id} must have no review lanes or "
                    "conditions and must remain open",
                    issues,
                )
        else:
            if cells[3].strip() != S0_COMPLETE_REVIEW_LANES_CELL:
                fail(
                    f"resolved S0 decision {decision_id} must record all review lanes",
                    issues,
                )
            if disposition == "Accept":
                if not condition_cells_are_empty or resolution != "closed":
                    fail(
                        f"accepted S0 decision {decision_id} must have no condition and "
                        "must be closed",
                        issues,
                    )
            elif disposition == "ConditionalAccept":
                if not condition_cells_are_complete:
                    fail(
                        f"conditional S0 decision {decision_id} requires owner, evidence "
                        "and exit condition",
                        issues,
                    )
            elif disposition == "Reject":
                if not condition_cells_are_complete or resolution != "open":
                    fail(
                        f"rejected S0 decision {decision_id} requires owner, evidence and "
                        "exit condition and must remain open",
                        issues,
                    )
        decisions[decision_id] = (disposition, resolution)

    actual_decisions = set(decisions)
    if actual_decisions != S0_REVIEW_DECISION_IDS:
        fail(
            "S0 architecture decision set drift: "
            f"missing={sorted(S0_REVIEW_DECISION_IDS - actual_decisions)} "
            f"unexpected={sorted(actual_decisions - S0_REVIEW_DECISION_IDS)}",
            issues,
        )

    if packet_status == "Complete":
        non_pass_lanes = sorted(
            lane_id for lane_id, outcome in lane_outcomes.items() if outcome != "Pass"
        )
        if non_pass_lanes:
            fail(f"complete S0 review has non-pass lanes: {non_pass_lanes}", issues)
        unresolved_decisions = sorted(
            decision_id
            for decision_id, (disposition, resolution) in decisions.items()
            if disposition not in {"Accept", "ConditionalAccept"} or resolution != "closed"
        )
        if unresolved_decisions:
            fail(
                f"complete S0 review has unresolved decisions: {unresolved_decisions}",
                issues,
            )


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


def exact_marked_block(
    rel: str,
    begin: str,
    end: str,
    expected_sha256: str,
    label: str,
    issues: list[str],
) -> str | None:
    path = ROOT / rel
    if not path.is_file():
        fail(f"{label} carrier missing: {rel}", issues)
        return None
    text = path.read_text(encoding="utf-8")
    begin_count = text.count(begin)
    end_count = text.count(end)
    if begin_count != 1 or end_count != 1:
        fail(
            f"{label} marker count drift: begin={begin_count} end={end_count}",
            issues,
        )
        return None
    start = text.index(begin)
    finish = text.index(end)
    if finish <= start:
        fail(f"{label} marker order drift", issues)
        return None
    block = text[start : finish + len(end)]
    actual_sha256 = hashlib.sha256(block.encode("utf-8")).hexdigest()
    if actual_sha256 != expected_sha256:
        fail(
            f"{label} exact block drift: expected={expected_sha256} actual={actual_sha256}",
            issues,
        )
    return block


def check_campaign_taskbook_state(issues: list[str]) -> None:
    required_fields = (
        "Campaign ID",
        "Lane",
        "Status",
        "Bound source commit",
        "Repair round",
        "Current blocker identity",
        "Stop reason",
        "Last transition evidence",
        "Next allowed mutation",
    )
    grant_link = (
        "- `Grant carrier`: "
        "[`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)"
    )
    for lane, rel in AUTONOMOUS_CAMPAIGN_TASKBOOKS.items():
        path = ROOT / rel
        if not path.is_file():
            fail(f"campaign taskbook missing for {lane}: {rel}", issues)
            continue
        text = path.read_text(encoding="utf-8")
        values: dict[str, str] = {}
        for field in required_fields:
            pattern = re.compile(rf"^- `{re.escape(field)}`:\s*(.+?)\s*$", re.MULTILINE)
            matches = pattern.findall(text)
            if len(matches) != 1:
                fail(
                    f"campaign taskbook field count drift for {lane} {field}: {len(matches)}",
                    issues,
                )
                continue
            values[field] = matches[0]
        if text.count(grant_link) != 1:
            fail(f"campaign taskbook grant link drift for {lane}", issues)
        if values.get("Campaign ID") != f"`{AUTONOMOUS_CAMPAIGN_ID}`":
            fail(f"campaign taskbook campaign identity drift for {lane}", issues)
        if values.get("Lane") != f"`{lane}`":
            fail(f"campaign taskbook lane identity drift for {lane}", issues)

        status_match = re.fullmatch(r"`([^`]+)`", values.get("Status", ""))
        status = status_match.group(1) if status_match else ""
        if status not in VALID_CAMPAIGN_LANE_STATUSES:
            fail(f"campaign taskbook invalid status for {lane}: {values.get('Status')!r}", issues)

        source_match = re.fullmatch(
            r"`(pending-governance-main|[0-9a-f]{40})`",
            values.get("Bound source commit", ""),
        )
        source = source_match.group(1) if source_match else ""
        if not source_match:
            fail(f"campaign taskbook invalid bound source for {lane}", issues)
        if status in {"active", "completed"} and source == "pending-governance-main":
            fail(f"campaign taskbook active/completed lane has pending source for {lane}", issues)

        round_match = re.fullmatch(r"`([0-2])`", values.get("Repair round", ""))
        repair_round = int(round_match.group(1)) if round_match else -1
        if not round_match:
            fail(f"campaign taskbook invalid repair round for {lane}", issues)
        if (
            repair_round == 2
            and values.get("Current blocker identity") != "`none`"
            and status != "paused"
        ):
            fail(f"campaign taskbook round 2 must be paused for {lane}", issues)
        if status == "paused" and values.get("Stop reason") == "`none`":
            fail(f"campaign taskbook paused lane has no stop reason for {lane}", issues)
        for field in ("Current blocker identity", "Stop reason", "Last transition evidence", "Next allowed mutation"):
            if not values.get(field, "").strip():
                fail(f"campaign taskbook empty mutable state for {lane} {field}", issues)


def check_campaign_acceptance_bindings(
    grant_block: str | None, issues: list[str]
) -> None:
    if grant_block is None:
        return
    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    if not matrix_path.is_file():
        fail("campaign acceptance matrix carrier missing", issues)
        return
    lines = matrix_path.read_text(encoding="utf-8").splitlines()
    if not lines:
        fail("campaign acceptance matrix carrier empty", issues)
        return
    header = lines[0].split("\t")
    if "case_id" not in header or "status" not in header:
        fail("campaign acceptance matrix header drift", issues)
        return
    case_index = header.index("case_id")
    status_index = header.index("status")
    rows: dict[str, list[str]] = {}
    for line in lines[1:]:
        cells = line.split("\t")
        if len(cells) != len(header):
            continue
        case_id = cells[case_index]
        if case_id in rows:
            fail(f"campaign acceptance matrix duplicate row: {case_id}", issues)
            continue
        rows[case_id] = cells

    for case_id, expected_status in AUTONOMOUS_CAMPAIGN_MATRIX_BINDINGS.items():
        if grant_block.count(f"`{case_id}`") != 1:
            fail(f"campaign grant acceptance ID count drift: {case_id}", issues)
        cells = rows.get(case_id)
        if cells is None:
            fail(f"campaign acceptance matrix row missing: {case_id}", issues)
            continue
        actual_status = cells[status_index]
        if actual_status != expected_status:
            fail(
                f"campaign acceptance binding status drift for {case_id}: "
                f"expected={expected_status} actual={actual_status}",
                issues,
            )

    catalog_path = ROOT / AUTONOMOUS_CAMPAIGN_CATALOG_PATH
    if not catalog_path.is_file():
        fail("campaign long-horizon acceptance catalog missing", issues)
        return
    catalog = catalog_path.read_text(encoding="utf-8")
    for case_id in AUTONOMOUS_CAMPAIGN_CATALOG_ONLY_IDS:
        if grant_block.count(f"`{case_id}`") != 1:
            fail(f"campaign grant catalog-only ID count drift: {case_id}", issues)
        row_count = len(
            re.findall(rf"^\| `{re.escape(case_id)}` \|", catalog, flags=re.MULTILINE)
        )
        if row_count != 1:
            fail(
                f"campaign long-horizon catalog row count drift for {case_id}: {row_count}",
                issues,
            )
        if case_id in rows:
            fail(f"campaign catalog-only ID unexpectedly admitted to matrix: {case_id}", issues)


def check_campaign_authorization(issues: list[str]) -> None:
    exact_marked_block(
        CAMPAIGN_AUTHORIZATION_POLICY_PATH,
        CAMPAIGN_AUTHORIZATION_POLICY_BEGIN,
        CAMPAIGN_AUTHORIZATION_POLICY_END,
        CAMPAIGN_AUTHORIZATION_POLICY_SHA256,
        "campaign authorization policy",
        issues,
    )
    grant_block = exact_marked_block(
        AUTONOMOUS_CAMPAIGN_GRANT_PATH,
        AUTONOMOUS_CAMPAIGN_GRANT_BEGIN,
        AUTONOMOUS_CAMPAIGN_GRANT_END,
        AUTONOMOUS_CAMPAIGN_GRANT_SHA256,
        "autonomous campaign grant",
        issues,
    )
    check_campaign_acceptance_bindings(grant_block, issues)
    check_campaign_taskbook_state(issues)
    ci_path = ROOT / CAMPAIGN_CI_WORKFLOW_PATH
    if not ci_path.is_file():
        fail("campaign authorization CI carrier missing", issues)
        return
    ci_text = ci_path.read_text(encoding="utf-8")
    ci_sha256 = hashlib.sha256(ci_text.encode("utf-8")).hexdigest()
    if ci_sha256 != CAMPAIGN_CI_WORKFLOW_SHA256:
        fail(
            "campaign authorization CI workflow exact digest drift: "
            f"expected={CAMPAIGN_CI_WORKFLOW_SHA256} actual={ci_sha256}",
            issues,
        )
    checker_command = "python3 scripts/check_repo_contracts.py"
    command_count = ci_text.count(checker_command)
    if command_count != 1:
        fail(
            "campaign authorization aggregate checker CI binding drift: "
            f"expected 1 actual {command_count}",
            issues,
        )
    unit_test_command = "python3 -m unittest discover -s scripts/tests -p 'test_*.py'"
    unit_test_count = ci_text.count(unit_test_command)
    if unit_test_count != 1:
        fail(
            "campaign authorization mutation-test CI binding drift: "
            f"expected 1 actual {unit_test_count}",
            issues,
        )


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


DESIGN_PACKET_STATUSES = {"Proposal", "Reviewed", "Superseded"}
DESIGN_INDEX_PATH = "docs/design/README.md"


def git_rev_parse_oid(oid: str, object_type: str) -> str | None:
    """Resolve `<oid>^{<object_type>}` in the repository; None when unresolvable."""
    try:
        completed = subprocess.run(
            ["git", "-C", str(ROOT), "rev-parse", "--verify", f"{oid}^{{{object_type}}}"],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    value = completed.stdout.strip()
    return value if re.fullmatch(r"[0-9a-f]{40}", value) else None


def git_commit_tree_hex(commit: str) -> str | None:
    """Resolve `<commit>^{tree}`; None unless `<commit>` names a commit object."""
    if git_rev_parse_oid(commit, "commit") is None:
        return None
    return git_rev_parse_oid(commit, "tree")


def check_design_packets(issues: list[str]) -> None:
    index_rows = parse_markdown_table(
        DESIGN_INDEX_PATH,
        ["Packet", "Scope", "Status", "Source binding"],
        "design packet index",
        issues,
    )
    indexed: dict[str, tuple[str, str, str]] = {}
    for line_no, cells in index_rows:
        link_match = re.fullmatch(r"\[`([^`]+)/`\]\(([^)]+)/\)", cells[0])
        if link_match is None:
            fail(
                f"design packet index row {line_no} has malformed packet link: {cells[0]!r}",
                issues,
            )
            continue
        packet_name, link_target = link_match.groups()
        if packet_name != link_target or not re.fullmatch(r"[a-z0-9][a-z0-9-]*", packet_name):
            fail(
                f"design packet index row {line_no} packet identity drift: {cells[0]!r}",
                issues,
            )
            continue
        status = markdown_code_value(cells[2]) or cells[2].strip()
        if status not in DESIGN_PACKET_STATUSES:
            fail(
                f"design packet index row {line_no} has invalid status: {cells[2]!r}",
                issues,
            )
            continue
        binding_match = re.fullmatch(r"commit `([0-9a-f]{40})`, tree `([0-9a-f]{40})`", cells[3])
        if binding_match is None:
            fail(
                f"design packet index row {line_no} has malformed source binding: {cells[3]!r}",
                issues,
            )
            continue
        commit, tree = binding_match.groups()
        if packet_name in indexed:
            fail(f"duplicate design packet in index: {packet_name}", issues)
            continue
        indexed[packet_name] = (status, commit, tree)

    design_root = ROOT / "docs/design"
    actual_packets = (
        {path.name for path in design_root.iterdir() if path.is_dir()}
        if design_root.is_dir()
        else set()
    )
    if actual_packets != set(indexed):
        fail(
            "design packet directory/index drift: "
            f"missing={sorted(set(indexed) - actual_packets)} "
            f"unexpected={sorted(actual_packets - set(indexed))}",
            issues,
        )

    for packet_name, (status, commit, tree) in indexed.items():
        packet_dir = design_root / packet_name
        if not packet_dir.is_dir():
            continue
        resolved_tree = git_commit_tree_hex(commit)
        if resolved_tree is None:
            fail(
                f"design packet {packet_name} source commit is not resolvable in Git: {commit}",
                issues,
            )
        elif resolved_tree != tree:
            fail(
                f"design packet {packet_name} source binding tree drift: "
                f"index={tree} git={resolved_tree}",
                issues,
            )
        readme_path = packet_dir / "README.md"
        if not readme_path.is_file():
            fail(f"design packet {packet_name} missing README.md", issues)
            continue
        readme_text = readme_path.read_text(encoding="utf-8")
        status_rows = re.findall(r"^\| Status \| `([^`]+)` \|$", readme_text, flags=re.MULTILINE)
        if status_rows != [status]:
            fail(
                f"design packet {packet_name} README status drift: "
                f"index={status!r} readme={status_rows!r}",
                issues,
            )
        commit_rows = re.findall(
            r"^\| Source commit \| `([0-9a-f]{40})` \|$", readme_text, flags=re.MULTILINE
        )
        tree_rows = re.findall(
            r"^\| Source tree \| `([0-9a-f]{40})` \|$", readme_text, flags=re.MULTILINE
        )
        if commit_rows != [commit] or tree_rows != [tree]:
            fail(
                f"design packet {packet_name} README source binding drift: "
                f"index=({commit}, {tree}) readme=({commit_rows}, {tree_rows})",
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
    capability_schema = load_json("market/schemas/capability-registry.schema.json", issues)
    publishers = load_json("market/publishers/first-party.json", issues)
    capabilities = load_json("market/capabilities/registry.json", issues)
    if (
        not isinstance(schema, dict)
        or not isinstance(capability_schema, dict)
        or not isinstance(publishers, dict)
        or not isinstance(capabilities, dict)
    ):
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
    expected_capability_axes = {
        "campus.public_rules.read": (
            "Read",
            "PublicCampusFact",
            "CampusPublic",
            "FirstPartyDefaultOnly",
            "Allow",
            "Active",
        ),
        "campus.public_changes.read": (
            "Read",
            "PublicCampusFact",
            "CampusPublic",
            "FirstPartyDefaultOnly",
            "Allow",
            "Active",
        ),
        "campus.public_plan.read": (
            "Read",
            "PublicCampusFact",
            "CampusPublic",
            "FirstPartyDefaultOnly",
            "Allow",
            "Active",
        ),
        "campus.public_course.read": (
            "Read",
            "PublicCampusFact",
            "CampusPublic",
            "FirstPartyDefaultOnly",
            "Allow",
            "Active",
        ),
        "campus.community_review.linkout": (
            "Linkout",
            "PublicCampusFact",
            "CampusPublic",
            "FirstPartyDefaultOnly",
            "Allow",
            "Active",
        ),
        "user.own_academic_snapshot.read": (
            "Read",
            "UserProfile",
            "TenantPrivateUser",
            "Never",
            "Ask",
            "Active",
        ),
        "user.own_course_preferences.read": (
            "Read",
            "UserProfile",
            "TenantPrivateUser",
            "Never",
            "Ask",
            "Active",
        ),
        "user.own_plan_draft.write": (
            "Write",
            "UserProfile",
            "TenantPrivateUser",
            "Never",
            "Ask",
            "Active",
        ),
    }
    expected_capability_ids = set(expected_capability_axes)
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

    if set(capabilities) != {"schemaVersion", "registryRevision", "capabilities"}:
        fail(f"capability registry top-level keys drifted: {sorted(capabilities)}", issues)
    if capabilities.get("schemaVersion") != "capability-registry/v1":
        fail("capability registry schemaVersion drift", issues)
    revision = capabilities.get("registryRevision")
    if not isinstance(revision, str) or re.fullmatch(
        r"capability-registry:[A-Za-z0-9._:-]{1,108}", revision
    ) is None:
        fail("capability registry revision is malformed", issues)
    if capability_schema.get("properties", {}).get("schemaVersion", {}).get("const") != "capability-registry/v1":
        fail("CapabilityRegistry schema version carrier drift", issues)
    if capability_schema.get("additionalProperties") is not False:
        fail("CapabilityRegistry schema must deny unknown top-level fields", issues)

    capability_rows = capabilities.get("capabilities", [])
    if not isinstance(capability_rows, list):
        fail("capability registry must contain a capabilities list", issues)
        return
    registered: set[str] = set()
    auto_grant: set[str] = set()
    capability_keys = {
        "id",
        "effectClass",
        "dataClass",
        "scopeKind",
        "autoGrant",
        "confirmationDefault",
        "status",
    }
    for item in capability_rows:
        if not isinstance(item, dict):
            fail("capability registry row is not an object", issues)
            continue
        if set(item) != capability_keys:
            fail(f"capability registry row keys drifted: {sorted(item)}", issues)
        if "class" in item or "autoGrantEligible" in item:
            fail("capability registry retained legacy class/autoGrantEligible fields", issues)
        capability_id = item.get("id")
        if not isinstance(capability_id, str) or re.fullmatch(
            r"[a-z][a-z0-9]*(?:\.[a-z0-9_-]+){1,7}", capability_id
        ) is None:
            fail("capability registry contains malformed id", issues)
            continue
        if capability_id in registered:
            fail("capability registry contains duplicate ids", issues)
        registered.add(capability_id)
        axes = (
            item.get("effectClass"),
            item.get("dataClass"),
            item.get("scopeKind"),
            item.get("autoGrant"),
            item.get("confirmationDefault"),
            item.get("status"),
        )
        expected_axes = expected_capability_axes.get(capability_id)
        if expected_axes is None:
            fail(f"capability registry unexpected id: {capability_id}", issues)
        elif axes != expected_axes:
            fail(f"capability registry axes drifted for {capability_id}: {axes}", issues)
        public_default = axes == (
            item.get("effectClass"),
            "PublicCampusFact",
            "CampusPublic",
            "FirstPartyDefaultOnly",
            "Allow",
            "Active",
        ) and item.get("effectClass") in {"Read", "Linkout"}
        tenant_private = item.get("dataClass") in {"TenantPrivateFact", "UserProfile"}
        if item.get("autoGrant") == "FirstPartyDefaultOnly" and not public_default:
            fail(f"capability registry unsafe auto-grant tuple: {capability_id}", issues)
        if tenant_private and (
            item.get("scopeKind") != "TenantPrivateUser"
            or item.get("confirmationDefault") != "Ask"
            or item.get("autoGrant") != "Never"
        ):
            fail(f"capability registry tenant-private coherence drift: {capability_id}", issues)
        if public_default:
            auto_grant.add(capability_id)
    if registered != expected_capability_ids:
        fail(
            "capability registry inventory drift: "
            f"expected={sorted(expected_capability_ids)} actual={sorted(registered)}",
            issues,
        )

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


RUST_DOCTEST_GATE_COMMAND = "cargo test --locked --all-features --doc"


def _yaml_block(lines: list[str], header: str, indent: int) -> list[str] | None:
    exact_header = f"{' ' * indent}{header}"
    matches = [index for index, line in enumerate(lines) if line == exact_header]
    if len(matches) != 1:
        return None
    start = matches[0] + 1
    end = len(lines)
    for index in range(start, len(lines)):
        line = lines[index]
        if not line.strip():
            continue
        leading_spaces = len(line) - len(line.lstrip(" "))
        if leading_spaces <= indent:
            end = index
            break
    return lines[start:end]


def check_rust_doctest_gate(issues: list[str]) -> None:
    gates_rel = "docs/acceptance/gates.md"
    gates_path = ROOT / gates_rel
    if not gates_path.is_file():
        fail(f"Rust doctest gate carrier missing: {gates_rel}", issues)
    elif RUST_DOCTEST_GATE_COMMAND not in {
        line.strip() for line in gates_path.read_text(encoding="utf-8").splitlines()
    }:
        fail(f"Rust doctest gate missing from {gates_rel}", issues)

    ci_rel = ".github/workflows/ci.yml"
    ci_path = ROOT / ci_rel
    if not ci_path.is_file():
        fail(f"Rust doctest gate carrier missing: {ci_rel}", issues)
        return

    ci_lines = ci_path.read_text(encoding="utf-8").splitlines()
    trigger_block = _yaml_block(ci_lines, "on:", 0)
    if trigger_block is None or _yaml_block(trigger_block, "pull_request:", 2) is None:
        fail("Rust doctest CI pull_request trigger missing or ambiguous", issues)

    jobs_block = _yaml_block(ci_lines, "jobs:", 0)
    rust_job = None if jobs_block is None else _yaml_block(jobs_block, "rust:", 2)
    if rust_job is None:
        fail("Rust doctest CI rust job missing or ambiguous", issues)
        return
    if any(line.startswith("    if:") for line in rust_job):
        fail("Rust doctest CI rust job must not be conditional", issues)

    steps_block = _yaml_block(rust_job, "steps:", 4)
    if steps_block is None:
        fail("Rust doctest CI rust steps block missing or ambiguous", issues)
        return

    doc_step = _yaml_block(steps_block, "- name: Doc tests", 6)
    if doc_step is None:
        fail("Rust doctest CI step missing or ambiguous in rust steps", issues)
        return

    exact_run = f"        run: {RUST_DOCTEST_GATE_COMMAND}"
    if doc_step.count(exact_run) != 1:
        fail("Rust doctest CI step must use the exact run command", issues)
    if any(
        line.startswith("        if:") or line.startswith("        continue-on-error:")
        for line in doc_step
    ):
        fail("Rust doctest CI step must be unconditional and blocking", issues)


# ---------------------------------------------------------------------------------------------
# `platform-identity/v0` grammar semantics — the authority chain.
#
# Round 14 froze every function body EXACTLY, but over code with literal payloads stripped. That
# pins control flow and token shape and deliberately does not pin the bytes inside a literal, and
# the exhaustive byte oracle meant to cover the residue carried a mutable delimiter literal of its
# own. So production, oracle, both test corpora, the JSON fixtures, their digests and the
# projection goldens could all be moved from `:` to `?` TOGETHER: every mechanical gate stayed
# green while `a?b` was accepted and `a:b` rejected. The evidence was checking agreement among
# mutable carriers, not agreement with the accepted contract.
#
# The fix is an authority chain with a root outside the mutable set:
#
#     accepted contract grammar (docs/contracts/platform-identity.md)
#             | exact semantic cross-check
#     this table
#             | exact semantic extraction
#     production grammar + Rust exhaustive oracle + bound corpora
#
# The table is not the authority. It is compared field for field against the contract's single
# normative regex, which is PARSED structurally rather than searched for, and against the
# individually anchored normative-consequence lines. Editing this table alone fails; editing the
# contract too is a `platform-identity/v0` change under §9, which is the point.
PLATFORM_IDENTITY_CONTRACT = "docs/contracts/platform-identity.md"
PLATFORM_IDENTITY_GRAMMAR = {
    "regex": "^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$",
    "max_bytes": 128,
    "boundary_class": "ASCII_ALPHANUMERIC",
    "interior_extra_bytes": "-._:",
    "normalization": "NONE",
    "case_sensitive": True,
}
# The contract's regex is matched against this SHAPE and its character classes expanded, so the
# check is a structural parse rather than a substring comparison: a class that gained a byte, a
# repetition bound that moved, or a leading class that stopped matching the trailing one are each
# a different parse, not a different string.
PLATFORM_IDENTITY_GRAMMAR_SHAPE = re.compile(
    r"\A\^\[(?P<lead>[^\]]+)\]\(\?:\[(?P<interior>[^\]]+)\]"
    r"\{0,(?P<bound>\d+)\}\[(?P<tail>[^\]]+)\]\)\?\$\Z"
)
# Each normative field is bound to its OWN anchored carrier in contract §3, by list position and
# exact text. A whole-document substring search would let one surviving mention of `128` prove a
# bound that had moved everywhere it is actually used.
PLATFORM_IDENTITY_CONTRACT_SECTION = "## 3. Shared identifier grammar"
PLATFORM_IDENTITY_CONTRACT_NEXT_SECTION = "## 4. "
PLATFORM_IDENTITY_NORMATIVE_LINES = {
    2: "the first and last byte are ASCII alphanumeric;",
    5: "case is significant;",
}
PLATFORM_IDENTITY_NORMALIZATION_LINE = (
    "no trimming, Unicode normalization, case folding, delimiter rewriting or alternate "
    "spelling occurs;"
)
# The production carriers whose LITERALS are the grammar. Read from comment-stripped but
# literal-PRESERVING source, and only after the same function has been admitted by the exact
# body inventory, so a decoy string, comment or unadmitted helper cannot answer for them.
PLATFORM_IDENTITY_LENGTH_CONSTANT = "MAX_IDENTITY_BYTES"
PLATFORM_IDENTITY_BOUNDARY_FUNCTION = "is_boundary_byte"
PLATFORM_IDENTITY_INTERIOR_FUNCTION = "is_interior_byte"
PLATFORM_IDENTITY_BOUNDARY_PREDICATE = "byte.is_ascii_alphanumeric()"
PLATFORM_IDENTITY_INTERIOR_SHAPE = re.compile(
    r"\A\{ byte\.is_ascii_alphanumeric\(\) \|\| matches!\(byte, (?P<alternatives>[^)]*)\) \}\Z"
)
PLATFORM_IDENTITY_BYTE_LITERAL = re.compile(r"\Ab'(?P<byte>[ -~])'\Z")
# The exhaustive oracle's own delimiter table, bound to its admitted body rather than to the file.
PLATFORM_IDENTITY_ORACLE_BYTE_STRING = re.compile(r"\*b\"(?P<bytes>[^\"]*)\"")
PLATFORM_IDENTITY_VALID_CORPUS_FUNCTION = "valid_values"
# The length bound's EFFECTIVE semantics, which the declared carrier does not pin.
#
# Round 15 bound the DECLARATION `const MAX_IDENTITY_BYTES: usize = 128;` to the contract and froze
# `classify`'s exact body. Neither closes the class, because the body fingerprint is itself one of
# the mutable carriers. A body that declares a LOCAL `const EFFECTIVE_MAX_IDENTITY_BYTES: usize =
# 129;`, compares and reports through it, and keeps the module constant alive as
# `let _ = MAX_IDENTITY_BYTES;` leaves the contract, the checker table and the declaration at 128.
# With the fingerprint, the Rust body table and the bound suite's corpus constant co-mutated, the
# whole gate chain stayed green while an external caller parsed a 129-byte ID and was told the
# bound was 129.
#
# So effective use is proven by ELIMINATION against the contract-bound NAME, not by agreement
# between snapshots:
#   * the name is bound exactly once in the module, by a depth-0 `const` whose value is the
#     contract's number written in digits;
#   * it occurs nowhere else outside the deciding function;
#   * inside that function it occurs exactly twice — as the entire right-hand side of the module's
#     only length comparison, and as the entire reported bound;
#   * that function declares no item, binds no name that could shadow it, spells no integer other
#     than the byte-index offset, and measures exactly one `let bytes = value.as_bytes();`;
#   * across the module there is exactly one length comparison and exactly one constructed
#     `max_bytes` field, the enum's own field type excepted.
#
# Nothing is then left for a second bound to come from: no local constant, no `let`, no alias, no
# helper, no literal. Moving the effective bound requires moving the accepted contract.
PLATFORM_IDENTITY_BOUND_FUNCTION = "classify"
PLATFORM_IDENTITY_BOUND_SUBJECT = "bytes"
PLATFORM_IDENTITY_BOUND_SUBJECT_BINDING = "let bytes = value.as_bytes();"
PLATFORM_IDENTITY_BOUND_OPERATOR = ">"
PLATFORM_IDENTITY_BOUND_FIELD = "max_bytes"
# Source order: the enum variant's own field type, then the single constructed value.
PLATFORM_IDENTITY_BOUND_FIELD_VALUES = ("usize", PLATFORM_IDENTITY_LENGTH_CONSTANT)
# `byte_index: offset + 1` — the only number the deciding function may spell.
PLATFORM_IDENTITY_BOUND_ADMITTED_LITERALS = ("1",)
PLATFORM_IDENTITY_BOUND_FORBIDDEN_ITEM_KEYWORDS = (
    "const",
    "enum",
    "extern",
    "fn",
    "impl",
    "macro_rules",
    "mod",
    "static",
    "struct",
    "trait",
    "type",
    "union",
    "use",
)
# The bound suite's own length constants. `MAX_BYTES` generates every length fixture, so a
# co-mutated copy makes the runtime corpus agree with a wrong implementation instead of with the
# contract; `GRAMMAR_MAX_BYTES` is the suite's contract-parsed number. Both are pinned here, and
# the suite additionally asserts they are equal, so neither can drift alone.
PLATFORM_IDENTITY_TEST_LENGTH_CONSTANTS = ("MAX_BYTES", "GRAMMAR_MAX_BYTES")
# The error names the one rejection branch must construct, per the accepted contract's §2 table.
PLATFORM_IDENTITY_BOUND_ERROR_TYPE = "IdentityValueErrorKind"
PLATFORM_IDENTITY_BOUND_ERROR_VARIANT = "TooLong"
# `TooLong` may be spelled only where the contract puts it: the variant, its rendering, the branch.
PLATFORM_IDENTITY_BOUND_ERROR_VARIANT_SITES = 3
# The runtime half of the closure. Round 16 pinned only that AUTH-011 CALLS its bound helper, so
# deleting the helper's load-bearing tail while leaving the call in place kept every gate green:
# a call site is not a proof body. These bind the proof's own body instead.
PLATFORM_IDENTITY_RUNTIME_PROOF_FUNCTION = "assert_contract_bound_is_the_effective_runtime_limit"
PLATFORM_IDENTITY_RUNTIME_PROOF_CALLER = "assert_effective_max_byte_bound_is_contract_bound"
PLATFORM_IDENTITY_RUNTIME_PROOF_KIND = "TenantId"
PLATFORM_IDENTITY_RUNTIME_PROOF_CONSTANT = "GRAMMAR_MAX_BYTES"
# The generic corpus macro, whose pinned carriers are all substrings. A substring is not a case
# that still reaches it, so a row-skipping control transfer is forbidden outright.
PLATFORM_IDENTITY_CORPUS_MACRO = "assert_kind_enforces_grammar"
# `?` is control transfer too. Round 17 banned `continue` and `return` where reachability is
# claimed, which a helper returning `Result` and a caller writing `let _ = helper();` walk straight
# past: `black_box(Err::<(),()>(()))?` leaves before the proof runs and spells neither word. `break`
# and a labelled break end a corpus loop the same way. All five are refused wherever a carrier
# claims that something downstream of it executes.
PLATFORM_IDENTITY_FORBIDDEN_CONTROL = ("?", "break", "continue", "return")
PLATFORM_IDENTITY_CORPUS_MACRO_FORBIDDEN_CONTROL = PLATFORM_IDENTITY_FORBIDDEN_CONTROL
# Load-bearing proof helpers, each of which must be declared `fn <name>() {` — no parameter, no
# return type. A helper that returns anything can be ignored at the call site, and an ignored
# result is not a proof.
PLATFORM_IDENTITY_PROOF_HELPERS = (
    "assert_effective_max_byte_bound_is_contract_bound",
    "assert_contract_bound_is_the_effective_runtime_limit",
    "assert_corpus_macro_cannot_skip_a_row",
    "assert_classify_is_the_contract_decision_procedure",
    "assert_no_length_past_the_bound_is_accepted",
    "assert_load_bearing_calls_reach_their_helper",
    "assert_sweep_carriers_are_the_contract_extent",
)
# Every evidence call the bound caller must make, each as a plain statement of its own body.
PLATFORM_IDENTITY_CALLER_EVIDENCE_CALLS = (
    "assert_classify_is_the_contract_decision_procedure",
    "assert_load_bearing_calls_reach_their_helper",
    "assert_corpus_macro_cannot_skip_a_row",
    "assert_sweep_carriers_are_the_contract_extent",
    "assert_contract_bound_is_the_effective_runtime_limit",
    "assert_no_length_past_the_bound_is_accepted",
)
# The length sweep. The 128/129 pair fixes the boundary and nothing else, so an accept keyed to
# some OTHER over-bound length walks past it; this drives every length to twice the bound under two
# canonical seeds, which closes that behaviourally rather than only structurally.
PLATFORM_IDENTITY_RUNTIME_SWEEP_FUNCTION = "assert_no_length_past_the_bound_is_accepted"
PLATFORM_IDENTITY_RUNTIME_SWEEP_SEEDS = "RUNTIME_PROOF_SEEDS"
PLATFORM_IDENTITY_RUNTIME_SWEEP_SPAN = "RUNTIME_PROOF_SWEEP"
# …and the VALUES those two carriers must hold. Round 18 bound the sweep's token sequence, which
# fixes the loops and leaves what they range over free: `[&str; 0] = []` swept nothing and
# `= GRAMMAR_MAX_BYTES` swept nothing past the bound, both with every gate green.
PLATFORM_IDENTITY_RUNTIME_SWEEP_SEED_VALUES = ("a", "p")
PLATFORM_IDENTITY_RUNTIME_SWEEP_SEED_DECLARATION = (
    "const {name} : [ & str ; 2 ] = [ , ] ;"  # literal payloads are stripped before comparison
)
PLATFORM_IDENTITY_RUNTIME_SWEEP_SPAN_DECLARATION = "const {name} : usize = 2 * {bound} ;"
# AUTH-011's load-bearing evidence, which must each be a plain statement of the test body. Carrier
# substrings survive `if std::hint::black_box(false) { … }` around the whole block; a statement at
# the body's own depth does not.
PLATFORM_IDENTITY_AUTH011_FUNCTION = "identity_values_enforce_canonical_bounds_and_errors"
PLATFORM_IDENTITY_AUTH011_EVIDENCE_CALLS = (
    "assert_assertion_macros_bite",
    "assert_bound_test_envelope_is_active",
    "assert_grammar_semantics_match_the_contract",
    "assert_effective_max_byte_bound_is_contract_bound",
    "assert_grammar_is_exhaustive_over_bytes",
)
# A plain-statement call is a fact about TOKENS; which function runs is a fact about NAME
# RESOLUTION, and Rust resolves lexically. An item declared in the caller's own body binds the name
# ahead of the module's, so `fn r#assert_no_length_past_the_bound_is_accepted() {}` beside
# `let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();` keeps the real helper used —
# no unused-item lint fires — leaves every token the reachability rules match in place, and sends
# the call to a no-op. A shadow needs a DECLARATION, and a declaration must WRITE the name; those
# two facts are what the rules below turn on, so no spelling has to be enumerated.
PLATFORM_IDENTITY_LOAD_BEARING_HELPERS = tuple(
    sorted(set(PLATFORM_IDENTITY_PROOF_HELPERS) | set(PLATFORM_IDENTITY_AUTH011_EVIDENCE_CALLS))
)
PLATFORM_IDENTITY_SHADOWABLE_CALLERS = (
    PLATFORM_IDENTITY_AUTH011_FUNCTION,
    PLATFORM_IDENTITY_RUNTIME_PROOF_CALLER,
)
PLATFORM_IDENTITY_ITEM_KEYWORDS = (
    "const",
    "enum",
    "extern",
    "fn",
    "impl",
    "macro_rules",
    "mod",
    "static",
    "struct",
    "trait",
    "type",
    "union",
    "use",
)
# The corpus loops, which must be statements of the macro arm rather than of a closure nested in it.
PLATFORM_IDENTITY_CORPUS_LOOPS = ("valid_values", "invalid_values")
PLATFORM_IDENTITY_CORPUS_ARM_ANCHOR = "kind_name"
RUST_LENGTH_COMPARISON = re.compile(
    r"\b(?P<receiver>[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*len\s*\(\s*\)\s*"
    r"(?P<operator><=|>=|==|!=|<|>)\s*(?P<operand>[A-Za-z_][A-Za-z0-9_]*|\d[0-9_]*)"
)
RUST_LEN_CALL = re.compile(r"\.\s*len\s*\(\s*\)")
# Identifier/number runs, two-byte comparison operators whole, every other non-space character on
# its own. Deliberately the same dialect as the bound suite's tokenizer.
RUST_TOKEN = re.compile(r"[A-Za-z0-9_]+|[<>=!]=|[^\s]")
# One whole token that could be a name — the third token of a raw identifier `r # <name>`.
_RUST_IDENTIFIER = re.compile(r"\A[A-Za-z_][A-Za-z0-9_]*\Z")
RUST_INTEGER_LITERAL = re.compile(r"\b\d[0-9_]*\b")
# A `let`/`for` head up to its `=` or `in`: the pattern, which is where a shadowing binding of an
# otherwise contract-bound name would have to appear.
RUST_BINDING_PATTERN = re.compile(r"\b(?:let|for)\b(?P<pattern>[^=;{}]*?)(?:=|\bin\b)")

PLATFORM_IDENTITY_SOURCE = "crates/platform-core/src/identity.rs"
PLATFORM_IDENTITY_TEST = "crates/platform-core/tests/platform_identity.rs"
PLATFORM_CORE_LIB = "crates/platform-core/src/lib.rs"
PLATFORM_INVOCATION_SOURCE = "crates/platform-core/src/invocation.rs"
PLATFORM_MARKET_SOURCE = "crates/platform-core/src/market.rs"
PLATFORM_AUTHORITY_SOURCE = "crates/platform-core/src/market/authority.rs"
PLATFORM_AUTHORITY_TEST = "crates/platform-core/tests/market_authority_assembly.rs"
PLATFORM_SESSION_SOURCE = "crates/platform-core/src/session.rs"
PLATFORM_SESSION_TEST = "crates/platform-core/tests/platform_session.rs"
PLATFORM_CAPABILITY_TEST = "crates/platform-core/tests/market_capability_registry.rs"
PLATFORM_INSTALLATION_TEST = 'crates/platform-core/tests/market_installation_lifecycle.rs'
PLATFORM_INSTALLATION_SOURCE = 'crates/platform-core/src/market/installation.rs'
PLATFORM_GRANT_SOURCE = "crates/platform-core/src/market/grant.rs"
PLATFORM_GRANT_TEST = "crates/platform-core/tests/market_grant_lifecycle.rs"
PLATFORM_UPDATE_SOURCE = "crates/platform-core/src/market/update.rs"
PLATFORM_UPDATE_TEST = "crates/platform-core/tests/market_package_update.rs"
PLATFORM_IDENTITY_KINDS = (
    "TenantId",
    "UserId",
    "SessionId",
    "RequestId",
    "CommandId",
    "CorrelationId",
)
PLATFORM_IDENTITY_CODE_CARRIERS = (
    "pub struct IdentityValueError {",
    "pub enum IdentityValueErrorKind {",
    "pub struct $name {",
    "const MAX_IDENTITY_BYTES: usize = 128;",
    "pub const fn value_kind(&self) -> &'static str {",
    "pub const fn kind(&self) -> IdentityValueErrorKind {",
    "pub fn parse(value: impl Into<String>) -> Result<Self, IdentityValueError> {",
    "pub fn as_str(&self) -> &str {",
    "impl TryFrom<String> for $name {",
    "impl TryFrom<&str> for $name {",
    "impl FromStr for $name {",
    "impl fmt::Display for $name {",
    "impl Serialize for $name {",
    "impl<'de> Deserialize<'de> for $name {",
)
# Naming Serde's entry points cannot close the class: every implemented `visit_*` method is an
# independent construction path, and pinning two arms leaves `visit_bytes` — or a branch inside
# a helper that still contains the parse call — free to build an unvalidated value.
#
# So the rule moved one level down, to the thing every path must reach. A newtype with a private
# field can only be produced by its own tuple/struct-literal syntax inside the defining module,
# so the module is required to contain EXACTLY ONE such construction, and that one site must sit
# inside the checked constructor. Any extra visitor arm, early return, branch, decoy helper or
# future trait impl has to construct the value somewhere, and there is nowhere left to do it.
# The private field is private to the MODULE, not to the macro expansion, so the concrete kind
# names construct exactly as `$name` and `Self` do: a bare `fn f() -> TenantId { TenantId(s) }`
# written beside the generator bypasses `parse` while naming neither placeholder. Counting
# construction sites is only a closure if it counts every spelling of the constructor, so the
# concrete names are derived from the kind list rather than repeated — a seventh kind cannot be
# added here and forgotten there.
PLATFORM_IDENTITY_CONSTRUCTION_FORMS = ("$name", "Self", *PLATFORM_IDENTITY_KINDS)
# …and counting expressions is only a closure if the constructor IS an expression. A tuple
# struct's constructor is also a VALUE: `let ctor = $name; ctor(text)` fills the private field
# while writing neither `$name(` nor `Self(` at the construction site, so it satisfies every
# count and every spelling above. That value cannot be scanned away, because it can be bound,
# aliased, passed as an argument or returned before it is ever called.
#
# So the representation itself carries the rule. The six kinds are NAMED-FIELD structs, which
# have no constructor function item at all — `let ctor = $name;` does not compile — leaving a
# struct literal as the only way to produce one, and a struct literal is syntax that cannot be
# bound. The tuple form is rejected outright rather than merely absent.
PLATFORM_IDENTITY_STRUCT_DECLARATION = "pub struct $name { value: String, }"
PLATFORM_IDENTITY_FORBIDDEN_CONSTRUCTOR_ITEMS = (
    (
        "tuple struct",
        r"\bstruct\s+\$?[A-Za-z_][A-Za-z0-9_]*\s*(?:<[^;{]*>)?\s*\(",
    ),
)
PLATFORM_IDENTITY_ADMITTED_CONSTRUCTIONS = ("Self{",)
PLATFORM_IDENTITY_CONSTRUCTOR_FUNCTION = "parse"
# Every function of the identity module is inventoried WITH ITS EXACT BODY, in source order.
#
# A name inventory freezes the module's shape but says nothing about what each function does,
# and `body.contains(<admitted call>)` says nothing about what else it does: `if value == "x" {
# return Ok(..); }` placed above the admitted call keeps every containment check satisfied and
# never reaches it. One construction site inside `parse` does not close that either — a single
# `Ok(Self { value })` reached through `if value != "x" { classify(&value)?; }` is one site, in
# the right function, guarding nothing.
#
# So the bodies are accounted for exactly, the same total accounting the item, `pub`, `impl`,
# attribute, derive and macro-arm surfaces already carry, one level further down. The cost is
# that any change to this module is drift that must be mirrored here and in the Rust guard;
# that is the intended price of a frozen `v0` implementation.
#
# LIMIT, stated rather than implied: bodies are compared after comments and literal PAYLOADS are
# stripped, so this pins control flow and token shape, not the bytes inside a literal. Changing
# `b':'` to `b'?'` inside `is_interior_byte` preserves this table. That residue is closed by the
# exhaustive grammar oracle in the bound suite, which drives every one of the 256 byte values
# through each position rather than a hand-picked corpus.
PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES = (
    ("value_kind", "{ self.value_kind }"),
    ("kind", "{ self.kind }"),
    (
        "fmt",
        "{ let value_kind = self.value_kind; match self.kind { "
        "IdentityValueErrorKind::Empty => { write!(formatter, ) } "
        "IdentityValueErrorKind::TooLong { max_bytes } => write!( formatter, ), "
        "IdentityValueErrorKind::InvalidStart => write!( formatter, ), "
        "IdentityValueErrorKind::InvalidCharacter { byte_index } => write!( formatter, ), "
        "IdentityValueErrorKind::InvalidEnd => write!( formatter, ), } }",
    ),
    ("is_boundary_byte", "{ byte.is_ascii_alphanumeric() }"),
    ("is_interior_byte", "{ byte.is_ascii_alphanumeric() || matches!(byte, | | | ) }"),
    (
        "classify",
        "{ let bytes = value.as_bytes(); "
        "let Some((&first, after_first)) = bytes.split_first() else { "
        "return Err(IdentityValueErrorKind::Empty); }; "
        "if bytes.len() > MAX_IDENTITY_BYTES { return Err(IdentityValueErrorKind::TooLong { "
        "max_bytes: MAX_IDENTITY_BYTES, }); } "
        "if !is_boundary_byte(first) { return Err(IdentityValueErrorKind::InvalidStart); } "
        "let Some((&last, interior)) = after_first.split_last() else { return Ok(()); }; "
        "for (offset, &byte) in interior.iter().enumerate() { if !is_interior_byte(byte) { "
        "return Err(IdentityValueErrorKind::InvalidCharacter { byte_index: offset + 1, }); } } "
        "if !is_boundary_byte(last) { return Err(IdentityValueErrorKind::InvalidEnd); } Ok(()) }",
    ),
    (
        "parse",
        "{ let value = value.into(); match classify(&value) { Ok(()) => Ok(Self { value }), "
        "Err(kind) => Err(IdentityValueError { value_kind: stringify!($name), kind, }), } }",
    ),
    ("as_str", "{ &self.value }"),
    ("try_from", "{ Self::parse(value) }"),
    ("try_from", "{ Self::parse(value) }"),
    ("from_str", "{ Self::parse(value) }"),
    ("fmt", "{ formatter.write_str(&self.value) }"),
    ("serialize", "{ serializer.serialize_str(&self.value) }"),
    (
        "deserialize",
        "{ let value = String::deserialize(deserializer)?; "
        "$name::parse(value).map_err(de::Error::custom) }",
    ),
)
# Derived, so the name inventory and the body inventory cannot disagree about what exists.
PLATFORM_IDENTITY_ADMITTED_FUNCTIONS = tuple(
    name for name, _ in PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES
)
# The two bodies the whole contract rests on, named separately so their failure message says
# which invariant broke rather than only that the module drifted. Exact equality, not
# containment: one control-flow path from `value.into()` through `classify(&value)` to
# `Ok(Self { value })` and the error mapping, and one path from `String::deserialize` to the
# checked constructor.
PLATFORM_IDENTITY_PARSE_BODY = dict(PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES)["parse"]
PLATFORM_IDENTITY_DESERIALIZE_BODY = dict(PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES)[
    "deserialize"
]
# A hand-written visitor is what reopened this class twice, so the module carries none.
PLATFORM_IDENTITY_FORBIDDEN_SERDE_CARRIERS = (
    ("Visitor", r"\bVisitor\b"),
    ("visit_", r"\bvisit_[a-z_]+\b"),
    ("deserialize_any", r"\bdeserialize_any\b"),
)
PLATFORM_IDENTITY_ERROR_VARIANTS = (
    "Empty,",
    "TooLong {",
    "max_bytes: usize,",
    "InvalidStart,",
    "InvalidCharacter {",
    "byte_index: usize,",
    "InvalidEnd,",
)
PLATFORM_IDENTITY_ALLOWED_IMPORTS = {
    "use std::error::Error;",
    "use std::fmt;",
    "use std::str::FromStr;",
    "use serde::de;",
    "use serde::{Deserialize, Deserializer, Serialize, Serializer};",
}
PLATFORM_IDENTITY_FORBIDDEN_CARRIERS = (
    "uuid",
    "Uuid",
    "ulid",
    "Ulid",
    "nanoid",
    "NanoId",
    "rand",
    "Rng",
    "random",
    "generate",
    "mint",
    "SystemTime",
    "Instant",
    "chrono",
    "std::time",
    "std::net",
    "TcpStream",
    "reqwest",
    "hyper",
    "std::fs",
    "std::process",
    "sqlx",
    "diesel",
    "rusqlite",
    "axum",
    "dioxus",
)
# The identity module's public surface is frozen as an ALLOWLIST, not as a blacklist of bad
# spellings. A blacklist cannot prove negative space: `pub fn new` being absent says nothing
# about `pub fn from_unchecked`, `impl AsMut<String>`, a cross-kind `From` or a `pub type`
# alias for a deferred kind. Anything not listed here fails the check.
# Every `pub` token in identity.rs must be consumed by exactly one of these, qualifiers
# included, so `pub async fn parse` or `pub extern "Rust" fn parse` are distinct declarations
# and fail. An unclassifiable `pub` fails too, which is what makes this complete rather than
# a longer list of forbidden spellings.
PLATFORM_IDENTITY_ADMITTED_PUBLIC_DECLARATIONS = (
    "pub const fn kind",
    "pub const fn value_kind",
    "pub enum IdentityValueErrorKind",
    "pub fn as_str",
    "pub fn parse",
    "pub struct $name",
    "pub struct IdentityValueError",
)
# Likewise every `impl` token, headers joined across lines.
PLATFORM_IDENTITY_ADMITTED_IMPL_DECLARATIONS = (
    "impl $name",
    "impl Deserialize<'de> for $name",
    "impl Error for IdentityValueError",
    "impl FromStr for $name",
    "impl IdentityValueError",
    "impl Serialize for $name",
    "impl TryFrom<&str> for $name",
    "impl TryFrom<String> for $name",
    "impl fmt::Display for $name",
    "impl fmt::Display for IdentityValueError",
    # The single admitted `impl Trait` argument position, on the checked constructor.
    "impl-arg Into<String>",
)
# Siblings may embed data with `include_str!`; only item-splicing carriers are forbidden there.
PLATFORM_CORE_FORBIDDEN_SPLICE_PATTERNS = (
    ("include!", r"\binclude\s*!"),
    ("#[path", r"#\s*\[\s*path\b"),
)
# A bound test that is ignored or de-registered still exits 0, so the attribute envelope of
# every bound test is pinned as well as its body.
PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES = ("#[test]",)
PLATFORM_IDENTITY_FORBIDDEN_TEST_ATTRIBUTE_MARKERS = ("ignore", "cfg", "should_panic")
# `#![cfg(any())]` is an INNER attribute: it excludes the whole integration-test crate, so both
# bound commands report "running 0 tests" at exit 0 and the in-suite guards never run either.
# `#[cfg` does not match it, because the `!` sits between `#` and `[`.
# An attribute is a token sequence, not a string: `#` `!` `[` may be separated by whitespace or
# a comment, so `# /*x*/ ! [cfg(any())]` is the same crate-level exclusion as `#![cfg(any())]`.
# Every carrier below is therefore a whitespace-tolerant pattern over already-stripped code, and
# the two that are really items or macros — `extern crate` and `include!` — are additionally
# accounted for by the item and macro-invocation allowlists rather than screened here.
# `include_str!` stays admitted: the guards read the governed sources through it.
PLATFORM_IDENTITY_FORBIDDEN_TEST_FILE_PATTERNS = (
    ("#[ignore]", r"#\s*\[\s*ignore\s*\]"),
    ("cfg_attr", r"\bcfg_attr\b"),
    ("#[cfg", r"#\s*\[\s*cfg\b"),
    ("#![ (inner attribute)", r"#\s*!\s*\["),
    ("should_panic", r"\bshould_panic\b"),
    ("#[path", r"#\s*\[\s*path\b"),
    ("include!", r"\binclude\s*!"),
    ("extern crate", r"\bextern\s+crate\b"),
)
# The value generator is the one macro admitted in the module, and both its grammar and its
# invocations are frozen. Extending the matcher with an `$extra:item` fragment and forwarding a
# trait implementation through an existing invocation adds real public API without adding any
# new `macro_rules!`, so pinning the definition alone is not enough.
PLATFORM_IDENTITY_MACRO_NAME = "identity_value"
PLATFORM_IDENTITY_MACRO_MATCHER = "($(#[$attribute:meta])* $name:ident) => {"
# A textual single-file allowlist proves nothing if the file can splice in another file, or if
# the crate can grow a module the scan never reads. Both are ordinary, rustfmt- and
# clippy-clean Rust, so the source set and the splicing carriers are pinned too.
PLATFORM_IDENTITY_FORBIDDEN_SPLICE_PATTERNS = (
    ("include!", r"\binclude\s*!"),
    ("include_str!", r"\binclude_str\s*!"),
    ("include_bytes!", r"\binclude_bytes\s*!"),
    ("#[path", r"#\s*\[\s*path\b"),
)
# Enumerated over the WHOLE package, not `src/*.rs`: a `#[path = "../elsewhere.rs"]` module
# compiles a file that a `src`-only glob never sees.
PLATFORM_CORE_SOURCE_FILES = ('src/identity.rs',
 'src/invocation.rs',
 'src/lib.rs',
 'src/market.rs',
 'src/market/authority.rs',
 'src/market/capability.rs',
 'src/market/grant.rs',
 'src/market/installation.rs',
 'src/market/update.rs',
 'src/session.rs',
 'src/source_registry.rs',
 'tests/invocation_resolution.rs',
 'tests/market_authority_assembly.rs',
 'tests/market_capability_registry.rs',
 'tests/market_grant_lifecycle.rs',
 'tests/market_installation_lifecycle.rs',
 'tests/market_package_catalog.rs',
 'tests/market_package_update.rs',
 'tests/platform_identity.rs',
 'tests/platform_session.rs',
 'tests/source_registry.rs',
 'tests/support/invocation_fixture.rs',
 'tests/support/invocation_fixture_executor.rs')
PLATFORM_IDENTITY_ADMITTED_REEXPORT = "pub use crate::identity::{TenantId, UserId};"
PLATFORM_INSTALLATION_ADMITTED_IDENTITY_IMPORT = "use crate::identity::{TenantId, UserId};"
PLATFORM_GRANT_ADMITTED_IDENTITY_IMPORT = "use crate::identity::{TenantId, UserId};"
PLATFORM_UPDATE_ADMITTED_IDENTITY_IMPORT = "use crate::identity::{TenantId, UserId};"
PLATFORM_AUTHORITY_ADMITTED_IDENTITY_IMPORT = "use crate::identity::{TenantId, UserId};"
PLATFORM_SESSION_ADMITTED_IDENTITY_IMPORT = (
    "use crate::identity::{SessionId, TenantId, UserId};"
)
# Cross-file bindings of an admitted kind are admitted by ENUMERATION, never by pattern, and
# each entry is keyed by exact repository-relative file together with exact normalized text.
# Relaxing this into a prefix, regex or predicate over `crate::identity::` would re-open
# precisely the alias class the surrounding rule exists to close: `use crate::identity::TenantId
# as Tenant;` matches every such pattern while changing the name every textual comparison sees.
# Two independent carriers must both admit a binding — it appears in the governed source's item
# allowlist above like any other item, AND it appears here — and neither substitutes for the
# other. Adding a row is registered drift that must be mirrored in the Rust guard.
PLATFORM_IDENTITY_ADMITTED_CROSS_FILE_BINDINGS = (
    (PLATFORM_INVOCATION_SOURCE, PLATFORM_IDENTITY_ADMITTED_REEXPORT),
    (PLATFORM_INSTALLATION_SOURCE, PLATFORM_INSTALLATION_ADMITTED_IDENTITY_IMPORT),
    (PLATFORM_GRANT_SOURCE, PLATFORM_GRANT_ADMITTED_IDENTITY_IMPORT),
    (PLATFORM_UPDATE_SOURCE, PLATFORM_UPDATE_ADMITTED_IDENTITY_IMPORT),
    (PLATFORM_AUTHORITY_SOURCE, PLATFORM_AUTHORITY_ADMITTED_IDENTITY_IMPORT),
    (PLATFORM_SESSION_SOURCE, PLATFORM_SESSION_ADMITTED_IDENTITY_IMPORT),
)
# Which files Cargo compiles into the crate is decided by non-inline `mod` declarations, not by
# a file extension. Pinning the declarations pins the compiled set semantically, so no
# attribute spelling — `#[path]`, `#[cfg_attr(all(), path = "x.txt")]`, or a future one — can
# introduce a module the scan never reads.
PLATFORM_CORE_ADMITTED_MODULE_DECLARATIONS = {'identity.rs': (),
 'invocation.rs': (),
 'lib.rs': ('identity', 'invocation', 'market', 'session', 'source_registry'),
 'market.rs': ('authority', 'capability', 'grant', 'installation', 'update'),
 'market/authority.rs': (),
 'market/capability.rs': (),
 'market/grant.rs': (),
 'market/installation.rs': (),
 'market/update.rs': (),
 'session.rs': (),
 'source_registry.rs': ()}
# Pinning module NAMES is not the same as pinning module SOURCES, and pinning a re-export by
# the spelling `crate::identity` is not the same as accounting for the use tree that contains
# it. `#[path = "identity_hidden.txt"] pub mod identity;` keeps the admitted name while Cargo
# compiles a different file, and `pub use crate::{identity as alias, invocation as other};`
# is a whole-module re-export that never spells `crate::identity`.
#
# So every `mod`/`use`/`type` item of every governed source is accounted for, in source order,
# WITH its attribute envelope, and compared against this exact allowlist. An attribute on an
# admitted module changes its fingerprint; a use tree in any spelling is either listed here or
# rejected; removing an admitted item fails too. The cost is real: an M20 change to the
# protocol import list below must be mirrored here and in the Rust guard. That is the intended
# price of a frozen v0 surface, and the failure message names the drift.
PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS = {'identity.rs': ('use std::error::Error;',
                 'use std::fmt;',
                 'use std::str::FromStr;',
                 'use serde::de;',
                 'use serde::{Deserialize, Deserializer, Serialize, Serializer};',
                 'type Error = IdentityValueError;',
                 'type Error = IdentityValueError;',
                 'type Err = IdentityValueError;'),
 'invocation.rs': ('pub use crate::identity::{TenantId, UserId};',
                   'use std::collections::BTreeSet;',
                   'use std::error::Error;',
                   'use std::fmt;',
                   'use ustc_agent_tool_protocol::{ AgentTool, AgentToolDefinition, '
                   'AgentToolsetView, ProjectionSnapshotId, ProtocolConstructionError, '
                   'ProtocolRunId, ProtocolTurnId, ToolRouteRef, is_valid_tool_name, };',
                   'pub use ustc_agent_tool_protocol::{ ArgumentConstructionError, '
                   'CanonicalArgumentNodeV0, CanonicalArgumentValueV0, InvalidValue, '
                   'SchemaConstructionError, Sha256Digest, UnvalidatedArgumentValueV0, '
                   'UnvalidatedSchemaNodeV0, UnvalidatedToolInputSchemaV0, ValidatedSchemaNodeV0, '
                   'ValidatedToolInputSchemaV0, };'),
 'lib.rs': ('pub mod identity;',
            'pub mod invocation;',
            'pub mod market;',
            'pub mod session;',
            'pub mod source_registry;',
            '#[cfg(test)] mod tests',
            'use super::*;'),
 'market.rs': ('pub mod authority;',
               'pub mod capability;',
               'pub mod grant;',
               'pub mod installation;',
               'pub mod update;',
               'use crate::invocation::{ CapabilityId, CatalogRevision, ComponentKind, PackageId, '
               'PackageVersion, Sha256Digest, };',
               'use serde::Deserialize;',
               'use serde::de::{self, MapAccess, Visitor};',
               'use std::collections::{BTreeMap, BTreeSet};',
               'use std::error::Error;',
               'use std::fmt;',
               'type Value = UniqueStringMap;'),
 'market/authority.rs': ('use crate::identity::{TenantId, UserId};',
                         'use crate::invocation::{ AuthorizedInvocation, CapabilityGrantSnapshot, '
                         'CapabilityId, CatalogPackageRevision, CurrentDenyState, GrantSnapshotId, '
                         'InstallationId, InvocationAuthorityCandidate, '
                         'InvocationAuthorizationError, InvocationPolicySnapshot, '
                         'InvocationResolver, InvocationTarget, ObjectScope, '
                         'PluginInstallationSnapshot, ProjectionResolutionError, ProposedToolCall, '
                         'ResolvedInvocation, ToolProjectionRequest, ToolProjectionSnapshot, '
                         'authorize_call, preflight_projected_call, };',
                         'use std::cell::Cell;',
                         'use std::collections::{BTreeMap, BTreeSet};',
                         'use std::error::Error;',
                         'use std::fmt;',
                         "type ReadTransaction<'a>: InvocationAuthorityReadTransaction where Self: "
                         "'a;",
                         "type ReadTransaction<'a> = InMemoryAuthorityReadTransaction<'a>;",
                         '#[cfg(test)] mod tests',
                         'use super::*;',
                         'use crate::invocation::{ CapabilityClass, CatalogRevision, ComponentId, '
                         'GrantState, GrantVersion, ObjectScope, PackageId, PackageVersion, '
                         'PolicyRevision, PolicySnapshotId, RunId, Sha256Digest, ToolId, TurnId, '
                         '};'),
 'market/capability.rs': ('use crate::invocation::{CapabilityClass, CapabilityId, '
                          'ConfirmationPolicy, Sha256Digest};',
                          'use serde::Deserialize;',
                          'use std::collections::BTreeSet;',
                          'use std::error::Error;',
                          'use std::fmt;',
                          '#[cfg(test)] mod tests',
                          'use super::*;'),
 'market/grant.rs': ('use crate::identity::{TenantId, UserId};',
                     'use crate::invocation::{ CapabilityGrantSnapshot, CapabilityId, '
                     'CatalogRevision, ConfirmationPolicy, GrantSnapshotId, GrantState, '
                     'GrantVersion, InstallationId, InstallationRevision, ObjectScope, PackageId, '
                     'PackageVersion, Sha256Digest, };',
                     'use crate::market::ValidatedPackageManifest;',
                     'use crate::market::capability::{ AutoGrantDisposition, CapabilityDefinition, '
                     'CapabilityPolicyChange, CapabilityRegistry, CapabilityRegistryRevision, '
                     'CapabilityStatus, DataClass, EffectClass, ScopeKind, '
                     'compare_capability_definitions, };',
                     'use crate::market::installation::{InstallationSnapshot, '
                     'ManagedInstallationState};',
                     'use std::collections::{BTreeMap, BTreeSet};',
                     'use std::error::Error;',
                     'use std::fmt;',
                     'pub type GrantSnapshot = GrantAggregate;',
                     '#[cfg(test)] mod tests',
                     'use super::*;',
                     'use crate::invocation::{ComponentId, ComponentKind, ComponentVersion, '
                     'ExecutionIdentity};',
                     'use crate::market::capability::load_capability_registry;',
                     'use crate::market::installation::{ InstallationCommand, '
                     'InstallationCommandId, InstallationConfiguration, InstallationPackagePin, '
                     'InstalledComponentPin, };',
                     'use crate::market::load_package_manifest;'),
 'market/installation.rs': ('use crate::identity::{TenantId, UserId};',
                            'use crate::invocation::{ CatalogRevision, ComponentId, ComponentKind, '
                            'ComponentVersion, ExecutionIdentity, InstallationId, '
                            'InstallationRevision, InstallationState as ResolverInstallationState, '
                            'InstalledComponentIdentity, PackageId, PackageVersion, '
                            'PluginInstallationSnapshot, Sha256Digest, };',
                            'use std::collections::{BTreeMap, BTreeSet};',
                            'use std::error::Error;',
                            'use std::fmt;',
                            'pub type InstallationSnapshot = InstallationAggregate;',
                            '#[cfg(test)] #[allow(clippy::expect_used, clippy::panic, '
                            'clippy::unwrap_used)] mod tests',
                            'use super::*;'),
 'market/update.rs': ('use crate::identity::{TenantId, UserId};',
                      'use crate::invocation::{ CapabilityClass, CapabilityId, '
                      'CatalogComponentRevision, CatalogPackageRevision, CatalogRevision, '
                      'ComponentId, ComponentKind, ExecutionIdentity, GrantSnapshotId, GrantState, '
                      'GrantVersion, InstallationId, InstallationRevision, '
                      'InvocationPolicySnapshot, Sha256Digest, SourcePolicyIdentity, };',
                      'use crate::market::capability::{ CapabilityDefinition, '
                      'CapabilityPolicyChange, CapabilityRegistry, CapabilityRegistryRevision, '
                      'CapabilityStatus, ScopeKind, compare_capability_definitions, };',
                      'use crate::market::grant::{ CurrentInstallationGrantSet, GrantCommand, '
                      'GrantCommandId, GrantCommandOutcome, GrantCommandReceipt, GrantEvent, '
                      'GrantEventKind, GrantEventSequence, GrantInvalidationReason, '
                      'GrantReplayError, GrantRepository, GrantRepositoryError, GrantSnapshot, '
                      'InMemoryGrantRepository, replay as grant_replay, };',
                      'use crate::market::installation::{ ConfigurationRevision, '
                      'InMemoryInstallationRepository, InstallationCommand, InstallationCommandId, '
                      'InstallationCommandOutcome, InstallationCommandReceipt, InstallationEvent, '
                      'InstallationEventKind, InstallationEventSequence, InstallationPackagePin, '
                      'InstallationReplayError, InstallationRepository, '
                      'InstallationRepositoryError, InstallationSnapshot, '
                      'ManagedInstallationState, replay as installation_replay, };',
                      'use crate::market::{ CatalogReadModel, ComponentDeclaration, PackageTier, '
                      'ValidatedPackageManifest, };',
                      'use std::collections::{BTreeMap, BTreeSet};',
                      'use std::error::Error;',
                      'use std::fmt;',
                      'pub type PackageUpdateSnapshot = PackageUpdateAggregate;',
                      'type InstallationCouplingKey = (String, u64, u8, String);',
                      'type GrantCouplingKey = (String, u64, String);',
                      'type PolicyBindingKey = (String, String, String, String);',
                      '#[cfg(test)] mod tests',
                      'use super::*;',
                      'use crate::invocation::{ ComponentVersion, ConfirmationPolicy, '
                      'PolicyRevision, PolicySnapshotId, SourcePolicyId, SourcePolicyIdentity, };',
                      'use crate::market::grant::{ GrantAdmissionEvidence, GrantApprovalId, '
                      'GrantRepository, GrantScope, InMemoryGrantRepository, };',
                      'use crate::market::installation::{ ConfigurationKey, '
                      'EnablePreconditionEvidence, InstallationCommand, InstallationCommandId, '
                      'InstallationConfiguration, InstalledComponentPin, NonSecretText, decide as '
                      'installation_decide, evolve as installation_evolve, };',
                      'use crate::market::load_package_manifest;'),
 'session.rs': ('use std::error::Error;',
                'use std::fmt;',
                'use serde::de;',
                'use serde::{Deserialize, Deserializer, Serialize};',
                'use crate::identity::{SessionId, TenantId, UserId};',
                '#[cfg(test)] mod tests',
                'use super::*;'),
 'source_registry.rs': ('use std::collections::BTreeMap;',
                        'use std::error::Error;',
                        'use std::fmt;',
                        'use std::str::FromStr;',
                        'use serde::de;',
                        'use serde::{Deserialize, Deserializer, Serialize, Serializer};',
                        'use crate::SourceAuthority;',
                        'type Error = SourceValueError;',
                        'type Error = SourceValueError;',
                        'type Err = SourceValueError;',
                        '#[cfg(test)] mod tests',
                        'use super::*;')}
# A macro is the remaining item category that can add API to a governed type without naming it
# in a `use`, a `type` or an `impl` header: `macro_rules! m { ($t:ty) => { impl AsRef<str> for
# $t { .. } } }` plus `m!(TenantId);` implements a trait for an identity kind while every
# self-type scan sees `$t`. Sibling macro definitions are pinned and no sibling macro
# invocation may name a governed kind.
PLATFORM_CORE_ADMITTED_SIBLING_MACROS = {'identity.rs': ('identity_value',),
 'invocation.rs': ('authority_id',),
 'lib.rs': (),
 'market.rs': (),
 'market/authority.rs': ('parsed',),
 'market/capability.rs': (),
 'market/grant.rs': ('category_error', 'parsed'),
 'market/installation.rs': (),
 'market/update.rs': ('parsed',),
 'session.rs': (),
 'source_registry.rs': ('source_value',)}
# Macro INVOCATION names are pinned too, not screened for `include!`. A splicing macro can be
# reached whatever the spelling — `include /* x */ !("f.rs")` contains no `include!` substring —
# so the admitted name set per governed source is exact. `include_str!` stays admitted in
# lib.rs, which legitimately embeds the first-party manifests as data.
PLATFORM_CORE_ADMITTED_MACRO_INVOCATIONS = {'identity.rs': ('concat', 'identity_value', 'matches', 'stringify', 'write'),
 'invocation.rs': ('authority_id', 'format', 'write'),
 'lib.rs': ('assert', 'assert_eq', 'include_str', 'panic'),
 'market.rs': ('matches', 'write'),
 'market/authority.rs': ('assert', 'assert_eq', 'format', 'panic', 'parsed', 'vec', 'write'),
 'market/capability.rs': ('assert', 'assert_eq', 'matches'),
 'market/grant.rs': ('assert',
                     'assert_eq',
                     'assert_ne',
                     'category_error',
                     'concat',
                     'format',
                     'include_bytes',
                     'matches',
                     'panic',
                     'parsed',
                     'unreachable',
                     'vec',
                     'write'),
 'market/installation.rs': ('assert',
                            'assert_eq',
                            'assert_ne',
                            'format',
                            'matches',
                            'panic',
                            'vec',
                            'write'),
 'market/update.rs': ('assert',
                      'assert_eq',
                      'assert_ne',
                      'format',
                      'matches',
                      'panic',
                      'parsed',
                      'unreachable',
                      'vec',
                      'write'),
 'session.rs': ('assert', 'assert_eq', 'matches', 'panic', 'write'),
 'source_registry.rs': ('assert',
                        'assert_eq',
                        'concat',
                        'matches',
                        'source_value',
                        'stringify',
                        'write')}
PLATFORM_IDENTITY_ADMITTED_TEST_MACRO_INVOCATIONS = (
    "assert",
    "assert_eq",
    "assert_kind_enforces_grammar",
    "assert_ne",
    "concat",
    "format",
    "include_str",
    "matches",
    "panic",
    "stringify",
    "vec",
)
PLATFORM_GRANT_ADMITTED_IDENTITY_MACRO_ARGUMENTS = (
    ("parsed", "TenantId,"),
    ("parsed", "TenantId,"),
    ("parsed", "TenantId,"),
    ("parsed", "TenantId,"),
    ("parsed", "UserId,"),
)
PLATFORM_UPDATE_ADMITTED_IDENTITY_MACRO_ARGUMENTS = (
    ("parsed", "TenantId,"),
    ("parsed", "UserId,"),
)
PLATFORM_AUTHORITY_ADMITTED_IDENTITY_MACRO_ARGUMENTS = (
    ("parsed", "TenantId,"),
    ("parsed", "TenantId,"),
    ("parsed", "UserId,"),
    ("parsed", "UserId,"),
)
# Macro names alone are a set and therefore admit duplicates. Freeze every invocation count, then
# freeze `parsed!` arguments separately because that helper constructs typed authority values.
# The latter closes same-count type substitutions without turning arbitrary assertion bodies into
# repository-contract authority.
PLATFORM_GRANT_ADMITTED_MACRO_INVOCATION_COUNTS = (('assert', 15),
 ('assert_eq', 66),
 ('assert_ne', 4),
 ('category_error', 3),
 ('concat', 1),
 ('format', 11),
 ('include_bytes', 2),
 ('matches', 12),
 ('panic', 2),
 ('parsed', 74),
 ('unreachable', 1),
 ('vec', 19),
 ('write', 1))
PLATFORM_GRANT_ADMITTED_PARSED_ARGUMENT_COUNTS = (('CapabilityId,', 8),
 ('CatalogRevision,', 1),
 ('ComponentId,', 1),
 ('ComponentVersion,', 1),
 ('ExecutionIdentity,', 1),
 ('GrantApprovalId,', 5),
 ('GrantApprovalId, approval', 2),
 ('GrantCommandId,', 19),
 ('GrantCommandId, command', 2),
 ('GrantSnapshotId,', 14),
 ('GrantSnapshotId, snapshot', 2),
 ('GrantVersion,', 9),
 ('InstallationCommandId,', 2),
 ('InstallationId,', 1),
 ('InstallationRevision,', 1),
 ('TenantId,', 4),
 ('UserId,', 1))
# Rejecting sibling implementations whose self type NAMES a governed kind is still a blacklist.
# A blanket `impl<T> Extension for T` names no kind and covers all six, so the sibling `impl`
# surface is an allowlist as well. These are M20 items; a genuine M20 addition is drift that
# must be admitted here explicitly rather than arriving unseen.
PLATFORM_CORE_ADMITTED_SIBLING_IMPLS = {'authority.rs': ('impl AuthorityReadRevision',
                  'impl CurrentGrantKey',
                  'impl Error for AuthorityRepositoryError',
                  'impl Error for InvocationRecheckError',
                  'impl Error for ProjectionAssemblyError',
                  'impl InMemoryInvocationAuthorityRepository',
                  'impl InvocationAuthorityReadTransaction for '
                  "InMemoryAuthorityReadTransaction<'_>",
                  'impl InvocationAuthorityRepository for InMemoryInvocationAuthorityRepository',
                  'impl InvocationAuthorityService<R>',
                  'impl InvocationAuthorityService<R>',
                  'impl fmt::Debug for AuthorityReadRevision',
                  "impl fmt::Debug for InMemoryAuthorityReadTransaction<'_>",
                  'impl fmt::Debug for InMemoryInvocationAuthorityRepository',
                  'impl fmt::Display for AuthorityRepositoryError',
                  'impl fmt::Display for InvocationRecheckError',
                  'impl fmt::Display for ProjectionAssemblyError'),
 'capability.rs': ('impl CapabilityDefinition',
                   'impl CapabilityRegistry',
                   'impl CapabilityRegistryRevision',
                   'impl Error for CapabilityRegistryLoadError',
                   'impl Error for CapabilityRegistryRevisionError',
                   'impl fmt::Display for CapabilityRegistryLoadError',
                   'impl fmt::Display for CapabilityRegistryRevisionError',
                   'impl-arg Into<String>'),
 'grant.rs': ('impl CurrentInstallationGrantSet',
              'impl Error for $kind',
              'impl Error for GrantRepositoryError',
              'impl Fixture',
              'impl GrantAdmissionEvidence',
              'impl GrantAggregate',
              'impl GrantApprovalId',
              'impl GrantCommand',
              'impl GrantCommandId',
              'impl GrantCommandReceipt',
              'impl GrantEvent',
              'impl GrantEventSequence',
              'impl GrantRepository for InMemoryGrantRepository',
              'impl GrantScope',
              'impl InMemoryGrantRepository',
              'impl InMemoryGrantRepository',
              'impl fmt::Debug for CurrentInstallationGrantSet',
              'impl fmt::Debug for GrantAdmissionEvidence',
              'impl fmt::Debug for GrantAggregate',
              'impl fmt::Debug for GrantApprovalId',
              'impl fmt::Debug for GrantCommand',
              'impl fmt::Debug for GrantCommandId',
              'impl fmt::Debug for GrantCommandOutcome',
              'impl fmt::Debug for GrantCommandReceipt',
              'impl fmt::Debug for GrantEvent',
              'impl fmt::Debug for InMemoryGrantRepository',
              'impl fmt::Display for $kind',
              'impl fmt::Display for GrantRepositoryError',
              'impl-arg Into<String>',
              'impl-arg Into<String>',
              "impl-arg IntoIterator<Item = &'a GrantEvent>"),
 'identity.rs': ('impl $name',
                 "impl Deserialize<'de> for $name",
                 'impl Error for IdentityValueError',
                 'impl FromStr for $name',
                 'impl IdentityValueError',
                 'impl Serialize for $name',
                 'impl TryFrom<&str> for $name',
                 'impl TryFrom<String> for $name',
                 'impl fmt::Display for $name',
                 'impl fmt::Display for IdentityValueError',
                 'impl-arg Into<String>'),
 'installation.rs': ('impl ConfigurationKey',
                     'impl ConfigurationRevision',
                     'impl EnablePreconditionEvidence',
                     'impl Error for InstallationConstructionError',
                     'impl Error for InstallationDecisionError',
                     'impl Error for InstallationReplayError',
                     'impl Error for InstallationRepositoryError',
                     'impl InMemoryInstallationRepository',
                     'impl InstallationAggregate',
                     'impl InstallationCommand',
                     'impl InstallationCommandId',
                     'impl InstallationCommandReceipt',
                     'impl InstallationConfiguration',
                     'impl InstallationEvent',
                     'impl InstallationEventPayload',
                     'impl InstallationEventSequence',
                     'impl InstallationPackagePin',
                     'impl InstallationRepository for InMemoryInstallationRepository',
                     'impl InstalledComponentPin',
                     'impl ManagedInstallationState',
                     'impl NonSecretText',
                     'impl SecretRef',
                     'impl SecretRefId',
                     'impl fmt::Debug for ConfigurationValue',
                     'impl fmt::Debug for EnablePreconditionEvidence',
                     'impl fmt::Debug for InstallationCommandAction',
                     'impl fmt::Debug for InstallationCommandId',
                     'impl fmt::Debug for InstallationConfiguration',
                     'impl fmt::Debug for InstallationEvent',
                     'impl fmt::Debug for InstallationEventPayload',
                     'impl fmt::Debug for NonSecretText',
                     'impl fmt::Debug for SecretRef',
                     'impl fmt::Debug for SecretRefId',
                     'impl fmt::Display for InstallationConstructionError',
                     'impl fmt::Display for InstallationDecisionError',
                     'impl fmt::Display for InstallationReplayError',
                     'impl fmt::Display for InstallationRepositoryError',
                     'impl-arg Into<String>',
                     'impl-arg Into<String>',
                     'impl-arg Into<String>',
                     'impl-arg Into<String>',
                     "impl-arg IntoIterator<Item = &'a InstallationEvent>"),
 'invocation.rs': ('impl $name',
                   'impl AuthorizedInvocation',
                   'impl Error for InvocationAuthorizationError',
                   'impl Error for ProjectionResolutionError',
                   'impl InvocationResolver',
                   'impl PackageVersion',
                   'impl ResolvedInvocation',
                   'impl ToolProjectionSnapshot',
                   'impl fmt::Display for InvocationAuthorizationError',
                   'impl fmt::Display for ProjectionResolutionError',
                   'impl-arg Into<String>',
                   "impl-arg IntoIterator<Item = &'a str>"),
 'lib.rs': ('impl SourceAuthority',),
 'market.rs': ('impl CatalogReadModel',
               'impl ComponentDeclaration',
               "impl Deserialize<'de> for UniqueStringMap",
               'impl Error for CatalogReadModelError',
               'impl Error for PackageLoadError',
               'impl Error for PackageValidationError',
               'impl InstallPolicy',
               'impl PackageValidationError',
               'impl ValidatedPackageManifest',
               "impl Visitor<'de> for UniqueStringMapVisitor",
               'impl fmt::Display for CatalogReadModelError',
               'impl fmt::Display for PackageLoadError',
               'impl fmt::Display for PackageValidationError'),
 'session.rs': ('impl AuthAdapterId',
                'impl CredentialEvidenceDigest',
                "impl Deserialize<'de> for AuthAdapterId",
                "impl Deserialize<'de> for CredentialEvidenceDigest",
                "impl Deserialize<'de> for SessionCredentialEvidence",
                "impl Deserialize<'de> for SessionDuration",
                'impl Error for SessionDomainError',
                'impl Error for SessionValueError',
                'impl ExpireSession',
                'impl OpenSession',
                'impl RefreshSession',
                'impl RevokeSession',
                'impl SessionCommand',
                'impl SessionCredentialEvidence',
                'impl SessionDuration',
                'impl SessionEvent',
                'impl SessionExpired',
                'impl SessionInstant',
                'impl SessionOpened',
                'impl SessionPolicy',
                'impl SessionRefreshed',
                'impl SessionRevoked',
                'impl SessionSnapshot',
                'impl SessionValueError',
                'impl fmt::Debug for CredentialEvidenceDigest',
                'impl fmt::Display for SessionDomainError',
                'impl fmt::Display for SessionValueError',
                 'impl-arg Into<String>',
                 'impl-arg Into<String>'),
 'source_registry.rs': ('impl $name',
                        "impl Deserialize<'de> for $name",
                        'impl Error for SourceRegistryError',
                        'impl Error for SourceValueError',
                        'impl FromStr for $name',
                        'impl Serialize for $name',
                        'impl SourceDefinition',
                        'impl SourceRegistry',
                        'impl SourceRetrievalPolicy',
                        'impl SourceReviewReceipt',
                        'impl SourceValueError',
                        'impl TryFrom<&str> for $name',
                        'impl TryFrom<String> for $name',
                        'impl fmt::Display for $name',
                        'impl fmt::Display for SourceRegistryError',
                        'impl fmt::Display for SourceValueError',
                        'impl-arg Into<String>'),
 'update.rs': ('impl AcceptedSnapshotForTest for UpdateCommandOutcome',
               'impl AuthorityCarrierBinding',
               'impl Error for UpdateConstructionError',
               'impl Error for UpdateDecisionError',
               'impl Error for UpdateReplayError',
               'impl Error for UpdateRepositoryError',
               'impl GrantEventReference',
               'impl InMemoryPackageUpdateRepository',
               'impl InstallationEventReference',
               'impl PackageUpdateAggregate',
               'impl PackageUpdateId',
               'impl PackageUpdatePlan',
               'impl PackageUpdateRepository for InMemoryPackageUpdateRepository',
               'impl RollbackReadinessEvidence',
               'impl UpdateApprovalEvidence',
               'impl UpdateApprovalId',
               'impl UpdateCommand',
               'impl UpdateCommandId',
               'impl UpdateCommandReceipt',
               'impl UpdateConfirmationEvidence',
               'impl UpdateDecisionContext',
               'impl UpdateEvent',
               'impl UpdateEventPayload',
               'impl UpdateEventSequence',
               'impl UpdateEvidenceId',
               'impl UpdateReadinessEvidence',
               'impl UpdateRevision',
               'impl UpdateState',
               'impl fmt::Debug for GrantEventReference',
               'impl fmt::Debug for InMemoryPackageUpdateRepository',
               'impl fmt::Debug for InstallationEventReference',
               'impl fmt::Debug for PackageUpdateAggregate',
               'impl fmt::Debug for PackageUpdateId',
               'impl fmt::Debug for PackageUpdatePlan',
               'impl fmt::Debug for PlanPackageAuthority',
               'impl fmt::Debug for RollbackReadinessEvidence',
               'impl fmt::Debug for UpdateApprovalEvidence',
               'impl fmt::Debug for UpdateApprovalId',
               'impl fmt::Debug for UpdateCommand',
               'impl fmt::Debug for UpdateCommandAction',
               'impl fmt::Debug for UpdateCommandId',
               'impl fmt::Debug for UpdateCommandOutcome',
               'impl fmt::Debug for UpdateCommandReceipt',
               'impl fmt::Debug for UpdateConfirmationEvidence',
               'impl fmt::Debug for UpdateDecisionContext',
               'impl fmt::Debug for UpdateEvent',
               'impl fmt::Debug for UpdateEventPayload',
               'impl fmt::Debug for UpdateEvidenceId',
               'impl fmt::Debug for UpdateReadinessEvidence',
               'impl fmt::Display for UpdateConstructionError',
               'impl fmt::Display for UpdateDecisionError',
               'impl fmt::Display for UpdateReplayError',
               'impl fmt::Display for UpdateRepositoryError',
               'impl-arg Into<String>',
               'impl-arg Into<String>',
               'impl-arg Into<String>',
               'impl-arg Into<String>',
               'impl-arg Into<String>',
               "impl-arg IntoIterator<Item = &'a UpdateEvent>")}
# `extern crate self as x;` re-roots the crate under a second name, which would make
# `x::identity` a foreign-looking path. It is an item whose keyword is neither `mod`, `use` nor
# `type`, so the item allowlist above cannot see it.
# Kept as a second, independent carrier alongside the `extern` item accounting above.
PLATFORM_CORE_FORBIDDEN_SOURCE_PATTERNS = (("extern crate", r"\bextern\s+crate\b"),)
PLATFORM_CAPABILITY_TEST_FUNCTIONS = ('current_registry_loads_with_exact_eight_definitions', 'enum_risk_and_compatibility_mappings_are_exact', 'source_size_and_malformed_json_fail_closed', 'duplicate_json_keys_fail_closed', 'duplicate_capability_ids_fail_closed', 'invalid_capability_id_grammar_fail_closed', 'missing_extra_and_unknown_fields_fail_closed', 'invalid_schema_version_and_registry_revision_fail_closed', 'forbidden_and_incoherent_combinations_fail_closed', 'auto_grant_candidacy_and_deprecated_revoked_exclusions', 'deterministic_ordering_and_permutation_independent_digest', 'fixed_definition_and_registry_digest_vectors', 'one_field_change_alters_definition_digest', 'registry_revision_does_not_change_definition_digests', 'policy_change_comparator_branches_and_precedence', 'errors_do_not_leak_rejected_source_fragments', 'empty_registry_loads_with_zero_definitions', 'definition_classifier_preserves_existing_policy_matrix', 'definition_classifier_handles_none_added_removed_revoked_and_all_axes', 'definition_classifier_uses_complete_definition_not_digest_or_caller_hint')
PLATFORM_GRANT_TEST_FUNCTIONS = ('checked_grant_ids_versions_and_sequences_are_canonical',
 'closed_scope_algebra_projects_exact_public_and_tenant_private_scopes',
 'non_issue_commands_validate_snapshot_and_expected_version',
 'empty_replay_and_repository_queries_are_deterministic',
 'public_errors_are_category_only_and_secret_safe')
PLATFORM_AUTHORITY_SOURCE_SHA256 = "7fa7210fd0cebca033fd9a597069dadf34af6cc084855ea3587afb413d89f672"
PLATFORM_AUTHORITY_TEST_SHA256 = "ba92339afb73096309948638c10db75dcb2a3714c6cc5f369e17b11aa2648fc3"
PLATFORM_AUTHORITY_ADMITTED_ATTRIBUTE_COUNTS = (
    ((False, "cfg", "cfg(test)"), 1),
    ((False, "derive", "derive(Clone)"), 1),
    ((False, "derive", "derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)"), 1),
    ((False, "derive", "derive(Clone, PartialEq, Eq, PartialOrd, Ord)"), 1),
    ((False, "derive", "derive(Debug, Clone, Copy, PartialEq, Eq)"), 3),
    ((False, "must_use", "must_use"), 3),
    ((False, "test", "test"), 5),
)
PLATFORM_AUTHORITY_ADMITTED_DERIVES = (
    "Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash",
    "Debug, Clone, Copy, PartialEq, Eq",
    "Debug, Clone, Copy, PartialEq, Eq",
    "Debug, Clone, Copy, PartialEq, Eq",
    "Clone, PartialEq, Eq, PartialOrd, Ord",
    "Clone",
)
PLATFORM_AUTHORITY_ADMITTED_PUBLIC_DECLARATIONS = (
    "pub const fn counter",
    "pub const fn new",
    "pub enum AuthorityRepositoryError",
    "pub enum InvocationRecheckError",
    "pub enum ProjectionAssemblyError",
    "pub fn fail_next_precondition_for_testing",
    "pub fn into_repository",
    "pub fn recheck_invocation",
    "pub fn replace_catalog_for_testing",
    "pub fn replace_grant_for_testing",
    "pub fn replace_installation_for_testing",
    "pub fn replace_policy_for_testing",
    "pub fn resolve_projection",
    "pub fn try_from_counter",
    "pub fn try_new",
    "pub struct AuthorityReadRevision",
    "pub struct InMemoryAuthorityReadTransaction",
    "pub struct InMemoryInvocationAuthorityRepository",
    "pub struct InvocationAuthorityService",
    "pub trait InvocationAuthorityReadTransaction",
    "pub trait InvocationAuthorityRepository",
)
PLATFORM_AUTHORITY_ADMITTED_MACRO_INVOCATION_COUNTS = (
    ("assert", 2),
    ("assert_eq", 11),
    ("format", 2),
    ("panic", 1),
    ("parsed", 23),
    ("vec", 7),
    ("write", 3),
)
PLATFORM_AUTHORITY_ADMITTED_PARSED_ARGUMENT_COUNTS = (
    ("CapabilityId,", 3),
    ("CatalogRevision,", 1),
    ("ComponentId,", 1),
    ("GrantSnapshotId,", 1),
    ("GrantVersion,", 1),
    ("InstallationId,", 2),
    ("ObjectScope,", 2),
    ("PackageId,", 1),
    ("PackageVersion,", 1),
    ("PolicyRevision,", 1),
    ("PolicySnapshotId,", 1),
    ("RunId,", 1),
    ("Sha256Digest, format!( , byte.to_string().repeat(64))", 1),
    ("TenantId,", 2),
    ("ToolId,", 1),
    ("TurnId,", 1),
    ("UserId,", 2),
)
PLATFORM_AUTHORITY_FUNCTION_BODY_SHA256 = {
    "resolve_projection": ("86be5f40fb3b13ac296f4da07c78ba1d792ba4fba866d300440b801f2a4ba8ba",),
    "recheck_invocation": ("a6504719b3433dc832800880e797cb6ece146b8862c0a17cee25662e7e045c0b",),
    "verify_precondition": (
        "41b805ea7ac014e23556e98bb374702a08344268f92489a02f0880849394a1e4",
        "2938e403f34160bc3d3739a6e932b5fa1b51825f28b063f76aa70b4e3cc4f6bd",
    ),
    "begin_read": (
        "41b805ea7ac014e23556e98bb374702a08344268f92489a02f0880849394a1e4",
        "9e09e339d8f77741742aacb39116d041530b0e5dbf7602df9173cc47da31dcad",
    ),
}
PLATFORM_INVOCATION_AUTHORITY_FUNCTION_BODY_SHA256 = {
    "preflight_projected_call": ("e500cd0fe2befa0c9dd5c72644bb7b9861144934591952444b8f04aacf0b94d2",),
    "authorize_call": ("da6cb882d0936c5acb1f9091c200ff34aaf124216aaee0e29594969c3894f439",),
}
PLATFORM_AUTHORITY_UNIT_TEST_FUNCTIONS = (
    "repository_rejects_duplicate_keys_and_incoherent_current_index",
    "transaction_loads_separate_carriers_under_one_revision",
    "conflict_revision_exhaustion_and_debug_are_fail_closed",
    "adopted_empty_projection_denial_precedes_pending_conflict",
    "catalog_key_multiplicity_is_rejected_without_deriving_authority",
)
PLATFORM_AUTHORITY_TEST_FUNCTIONS = (
    "projection_and_recheck_assemble_separate_carriers_under_one_verified_revision",
    "projection_missing_catalog_installation_or_grant_keeps_exact_resolver_denial",
    "post_success_transaction_conflict_returns_no_projection_or_authorized_invocation",
    "resolver_or_recheck_denial_precedes_pending_verification_conflict",
    "invalid_name_or_dispatch_preflight_precedes_repository_failure",
    "disable_revoke_stale_expire_and_emergency_block_deny_current_use",
    "historical_projection_is_immutable_after_current_authority_narrows",
    "dispatch_argument_version_scope_and_schema_errors_remain_adopted_recheck_errors",
)
PLATFORM_INVOCATION_PREFIX_TEST = "projected_call_preflight_and_authorize_share_exact_prefix_precedence"
PLATFORM_INVOCATION_PREFIX_TEST_BODY_SHA256 = "e479ca2296589b23729e4ad75d319514aa3ceea04bd815a46a8d4569558a857e"
PLATFORM_AUTHORITY_STATUS_MARKERS = {
    "docs/contracts/market-lifecycle.md": (
        "bounded `M20-B6` update/rollback domain plus semantic fake evidence are implemented",
        "planned `MARKET-001`, `MARKET-002`, `MARKET-003`, `MARKET-004`, `MARKET-007`, `PKG-019`, `PKG-020`, `FP-007`",
        "not a production catalog/publication authority, durable M90 transaction, grant/enable issuer or effect-intent/I/O boundary",
    ),
    "docs/contracts/invocation-resolution.md": (
        "bounded `M20-B5` semantic repository-transaction consumer implemented",
        "no durable or real application composition exists yet",
    ),
    "docs/features/00-market-browse-install.md": (
        "bounded transaction-current authority-assembly evidence",
        "`MARKET-001` through `MARKET-004`, `MARKET-007`, `PKG-020` and the user journey remain planned",
    ),
    "docs/plan/04-market-and-plugin-lifecycle.md": (
        "bounded `M20-B5` transaction-current authority assembly",
        "no durable adapter, effect intent, production composition or acceptance promotion",
    ),
    "docs/plan/modules/00-module-map.md": ("bounded transaction-current authority assembly",),
    "docs/plan/modules/30-market-package-lifecycle.md": (
        "bounded `M20-B5` semantic authority-read transaction/assembly",
        "Remaining work is `M20-B7` production-facing API/composition/fake M40 consumer plus future durable adapters",
    ),
    "docs/tasks/01-execution-roadmap.md": (
        "`M20-B5 invocation-authority`: bounded implementation is complete",
        "`M20-B6 update-rollback`: bounded evidence complete",
    ),
    "docs/overview/architecture.md": (
        "bounded transaction-current authority assembly",
        "without durable adapters, effect intents or I/O",
    ),
    "docs/acceptance/matrix.tsv": (
        "MARKET-003\tmarket\tdisable revoke or emergency block denies discovery projection and invocation immediately\t",
        "MARKET-007\tmarket\targuments and current deny-side authority are rechecked against the frozen projection before effect intent or outbound I/O\t",
        "\tplanned\tbackend",
        "\tplanned\tsecurity",
    ),
}
PLATFORM_INSTALLATION_TEST_FUNCTIONS = ('configuration_values_are_canonical_bounded_and_secret_safe',
 'package_pins_are_exact_canonical_and_duplicate_safe',
 'legal_install_configure_revoke_and_uninstall_transitions_are_explicit',
 'illegal_transitions_fail_closed_with_stable_categories',
 'absence_terminal_and_reinstall_semantics_are_distinct',
 'repository_idempotency_persists_accepted_rejected_and_global_conflicts',
 'repository_failure_injection_is_atomic_and_retryable',
 'replay_accepts_success_histories_and_rejects_gap_duplicate_reorder_and_command_reuse',
 'replay_rejects_impossible_initial_post_terminal_and_redundant_field_mismatches',
 'resolver_projection_maps_managed_states_without_grants_or_resolver_mutation',
 'event_receipt_and_error_debug_display_do_not_leak_configuration_or_secret_material')
PLATFORM_INSTALLATION_ADMITTED_DERIVES = ('Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Default, Clone',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, Copy')
PLATFORM_INSTALLATION_ADMITTED_UNCLASSIFIED_PUBLIC = ('pub(in crate::market) fn fr',
 'pub(in crate::market) fn pa',
 'pub(in crate::market) fn pa',
 'pub(in crate::market) fn ca',
 'pub(in crate::market) fn ma',
 'pub(in crate::market) fn tr')
PLATFORM_INSTALLATION_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS = (
 'from_authority_bindings',
 'package_updated',
 'package_rolled_back',
 'canonical_coupling_digest',
 'matches_package_pin_change',
 'try_from_histories_and_receipts')
PLATFORM_INSTALLATION_EVENT_KIND_VARIANTS = (
 'Installed',
 'Configured',
 'Enabled',
 'Disabled',
 'PackageUpdated',
 'PackageRolledBack',
 'Revoked',
 'Uninstalled')
PLATFORM_GRANT_ADMITTED_DERIVES = ('Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq, PartialOrd, Ord',
 'Debug, Clone, PartialEq, Eq',
 'Clone')
PLATFORM_GRANT_ADMITTED_UNCLASSIFIED_PUBLIC = ('pub(in crate::market) fn fr', 'pub(in crate::market) fn ca', 'pub(in crate::market) fn is', 'pub(in crate::market) fn tr')
PLATFORM_GRANT_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS = (
 'from_authority_bindings',
 'canonical_coupling_digest',
 'is_canonical',
 'try_from_histories_and_receipts')
PLATFORM_GRANT_ADMITTED_PUBLIC_DECLARATIONS = ('pub const fn approval_id',
 'pub const fn capability_definition',
 'pub const fn capability_definition',
 'pub const fn capability_definition_digest',
 'pub const fn capability_definition_digest',
 'pub const fn capability_id',
 'pub const fn capability_id',
 'pub const fn capability_manifest_digest',
 'pub const fn capability_manifest_digest',
 'pub const fn capability_registry_revision',
 'pub const fn capability_registry_revision',
 'pub const fn catalog_revision',
 'pub const fn catalog_revision',
 'pub const fn command_id',
 'pub const fn command_id',
 'pub const fn command_id',
 'pub const fn confirmation_policy',
 'pub const fn confirmation_policy',
 'pub const fn evidence_digest',
 'pub const fn expected_installation_revision',
 'pub const fn get',
 'pub const fn grant_set_digest',
 'pub const fn installation_id',
 'pub const fn installation_id',
 'pub const fn installation_id',
 'pub const fn installation_revision',
 'pub const fn last_approval_id',
 'pub const fn last_sequence',
 'pub const fn object_scope',
 'pub const fn observed_installation_revision',
 'pub const fn outcome',
 'pub const fn package_digest',
 'pub const fn package_digest',
 'pub const fn package_id',
 'pub const fn package_id',
 'pub const fn package_version',
 'pub const fn package_version',
 'pub const fn post_version',
 'pub const fn scope',
 'pub const fn scope',
 'pub const fn scope_kind',
 'pub const fn sequence',
 'pub const fn snapshot_id',
 'pub const fn snapshot_id',
 'pub const fn snapshot_id',
 'pub const fn snapshot_id',
 'pub const fn snapshot_id',
 'pub const fn state',
 'pub const fn tenant_id',
 'pub const fn tenant_id',
 'pub const fn tenant_id',
 'pub const fn tenant_id',
 'pub const fn user_id',
 'pub const fn user_id',
 'pub const fn user_id',
 'pub const fn user_id',
 'pub const fn version',
 'pub enum GrantChangeClass',
 'pub enum GrantCommandOutcome',
 'pub enum GrantConstructionError',
 'pub enum GrantDecisionError',
 'pub enum GrantEventKind',
 'pub enum GrantInvalidationReason',
 'pub enum GrantReplayError',
 'pub enum GrantRepositoryError',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn campus_public',
 'pub fn change_class',
 'pub fn decide',
 'pub fn evolve',
 'pub fn expire',
 'pub fn fail_next_commit_for_testing',
 'pub fn grants',
 'pub fn invalidation_reason',
 'pub fn issue',
 'pub fn kind',
 'pub fn mark_stale',
 'pub fn new',
 'pub fn new',
 'pub fn parse',
 'pub fn parse',
 'pub fn replace',
 'pub fn replay',
 'pub fn revoke',
 'pub fn tenant_private_user',
 'pub fn to_resolver_snapshot',
 'pub struct CurrentInstallationGrantSet',
 'pub struct GrantAdmissionEvidence',
 'pub struct GrantAggregate',
 'pub struct GrantApprovalId',
 'pub struct GrantCommand',
 'pub struct GrantCommandId',
 'pub struct GrantCommandReceipt',
 'pub struct GrantEvent',
 'pub struct GrantEventSequence',
 'pub struct GrantScope',
 'pub struct InMemoryGrantRepository',
 'pub trait GrantRepository',
 'pub type GrantSnapshot')
PLATFORM_INSTALLATION_ADMITTED_PUBLIC_DECLARATIONS = ('pub const fn capability_manifest_digest',
 'pub const fn capability_manifest_digest',
 'pub const fn catalog_revision',
 'pub const fn command',
 'pub const fn command_id',
 'pub const fn command_id',
 'pub const fn component_id',
 'pub const fn component_set_digest',
 'pub const fn component_set_digest',
 'pub const fn configuration',
 'pub const fn configuration_digest',
 'pub const fn configuration_revision',
 'pub const fn digest',
 'pub const fn digest',
 'pub const fn entries',
 'pub const fn evidence_digest',
 'pub const fn execution_identity',
 'pub const fn expected_installation_revision',
 'pub const fn get',
 'pub const fn get',
 'pub const fn grant_set_snapshot_digest',
 'pub const fn id',
 'pub const fn installation_id',
 'pub const fn installation_id',
 'pub const fn installation_id',
 'pub const fn kind',
 'pub const fn kind',
 'pub const fn last_sequence',
 'pub const fn outcome',
 'pub const fn package_digest',
 'pub const fn package_digest',
 'pub const fn package_id',
 'pub const fn package_pin',
 'pub const fn package_version',
 'pub const fn policy_admission_snapshot_digest',
 'pub const fn post_revision',
 'pub const fn revision',
 'pub const fn sequence',
 'pub const fn state',
 'pub const fn tenant_id',
 'pub const fn tenant_id',
 'pub const fn user_id',
 'pub const fn version',
 'pub enum ConfigurationValue',
 'pub enum InstallationCommandOutcome',
 'pub enum InstallationConstructionError',
 'pub enum InstallationDecisionError',
 'pub enum InstallationEventKind',
 'pub enum InstallationReplayError',
 'pub enum InstallationRepositoryError',
 'pub enum ManagedInstallationState',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn components',
 'pub fn configure',
 'pub fn decide',
 'pub fn disable',
 'pub fn enable',
 'pub fn evolve',
 'pub fn fail_next_commit_for_testing',
 'pub fn install',
 'pub fn new',
 'pub fn new',
 'pub fn new',
 'pub fn new',
 'pub fn new',
 'pub fn new',
 'pub fn new',
 'pub fn parse',
 'pub fn parse',
 'pub fn parse',
 'pub fn parse',
 'pub fn replay',
 'pub fn revoke',
 'pub fn to_installed_identity',
 'pub fn to_resolver_snapshot',
 'pub fn uninstall',
 'pub struct ConfigurationKey',
 'pub struct ConfigurationRevision',
 'pub struct EnablePreconditionEvidence',
 'pub struct InMemoryInstallationRepository',
 'pub struct InstallationAggregate',
 'pub struct InstallationCommand',
 'pub struct InstallationCommandId',
 'pub struct InstallationCommandReceipt',
 'pub struct InstallationConfiguration',
 'pub struct InstallationEvent',
 'pub struct InstallationEventSequence',
 'pub struct InstallationPackagePin',
 'pub struct InstalledComponentPin',
 'pub struct NonSecretText',
 'pub struct SecretRef',
 'pub struct SecretRefId',
 'pub trait InstallationRepository',
 'pub type InstallationSnapshot')

PLATFORM_INSTALLATION_ADMITTED_ATTRIBUTE_COUNTS = (((False, 'allow', 'allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)'), 1),
 ((False, 'allow', 'allow(clippy::large_enum_variant)'), 1),
 ((False, 'allow', 'allow(clippy::too_many_arguments)'), 3),
 ((False, 'allow', 'allow(clippy::too_many_arguments, dead_code)'), 1),
 ((False, 'allow', 'allow(dead_code)'), 7),
 ((False, 'cfg', 'cfg(test)'), 1),
 ((False, 'derive', 'derive(Clone, Copy)'), 1),
 ((False, 'derive', 'derive(Clone, PartialEq, Eq)'), 7),
 ((False, 'derive', 'derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)'), 3),
 ((False, 'derive', 'derive(Debug, Clone, Copy, PartialEq, Eq)'), 6),
 ((False, 'derive', 'derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)'), 2),
 ((False, 'derive', 'derive(Debug, Clone, PartialEq, Eq)'), 8),
 ((False, 'derive', 'derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)'), 1),
 ((False, 'derive', 'derive(Debug, Default, Clone)'), 1),
 ((False, 'must_use', 'must_use'), 52),
 ((False, 'test', 'test'), 7))
PLATFORM_UPDATE_TEST_FUNCTIONS = ('checked_public_update_values_and_stage_surface_are_deterministic',
 'empty_public_repository_and_replay_are_non_authoritative',
 'public_errors_and_debug_are_category_only_and_redacted')
PLATFORM_UPDATE_UNIT_TEST_FUNCTIONS = ('checked_update_ids_revisions_and_sequences_are_canonical',
 'update_plan_binds_exact_source_target_catalog_registry_policy_and_digests',
 'change_classifier_is_computed_from_complete_typed_authority_not_caller_hint',
 'change_classifier_precedence_covers_added_expanded_source_policy_tier_component_scope_and_unknown',
 'evidence_values_bind_plan_digest_revision_configuration_and_authority_without_leaking_payloads',
 'state_machine_allows_only_stage_approval_apply_confirm_rollback_cancel_paths',
 'stage_requires_absence_one_slot_nonterminal_installation_and_distinct_reviewed_target',
 'record_approval_requires_coherent_fresh_approval_and_readiness_evidence',
 'apply_requires_ready_disabled_exact_revisions_pins_configuration_and_authority_recheck',
 'rollback_requires_applied_pending_disabled_fresh_readiness_and_current_target_pin',
 'confirm_only_closes_applied_pending_without_runtime_health_or_grant_authority',
 'cancel_and_terminal_installation_reconciliation_have_exact_precedence',
 'event_sequences_revisions_command_ids_and_approval_consumption_are_exact',
 'replay_accepts_reachable_histories_and_rejects_gap_duplicate_reorder_overflow_and_post_terminal',
 'replay_rejects_forged_plan_change_class_evidence_revision_and_identity_bindings',
 'subordinate_installation_and_grant_event_references_are_complete_kind_checked_and_digest_bound',
 'repository_idempotency_command_conflict_approval_conflict_and_failure_injection_are_atomic',
 'repository_rebuilds_current_slot_all_ledgers_consumed_approvals_and_authority_indexes',
 'permission_expansion_and_every_conservative_update_require_exact_approval',
 'apply_and_rollback_preserve_frozen_toolsets_and_change_only_current_future_authority',
 'public_errors_debug_display_and_authority_debug_are_category_only_and_secret_safe')
PLATFORM_UPDATE_ADMITTED_ATTRIBUTE_COUNTS = (((False, 'allow', 'allow(clippy::large_enum_variant)'), 4),
 ((False, 'allow', 'allow(clippy::new_without_default)'), 1),
 ((False, 'allow', 'allow(clippy::too_many_arguments)'), 13),
 ((False, 'allow', 'allow(dead_code)'), 5),
 ((False, 'cfg', 'cfg(test)'), 6),
 ((False, 'derive', 'derive(Clone)'), 1),
 ((False, 'derive', 'derive(Clone, PartialEq, Eq)'), 21),
 ((False, 'derive', 'derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)'), 4),
 ((False, 'derive', 'derive(Debug, Clone, Copy, PartialEq, Eq)'), 6),
 ((False, 'derive', 'derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)'), 1),
 ((False, 'derive', 'derive(Debug, Clone, PartialEq, Eq)'), 1),
 ((False, 'derive', 'derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)'), 1),
 ((False, 'must_use', 'must_use'), 106),
 ((False, 'test', 'test'), 21))
PLATFORM_UPDATE_ADMITTED_DERIVES = ('Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, Copy, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Debug, Clone, PartialEq, Eq',
 'Clone, PartialEq, Eq',
 'Clone',
 'Clone, PartialEq, Eq')
PLATFORM_UPDATE_ADMITTED_PUBLIC_DECLARATIONS = ('pub const fn applied_event_digest',
 'pub const fn applied_installation_revision',
 'pub const fn approval',
 'pub const fn approval_evidence_digest',
 'pub const fn approval_id',
 'pub const fn change_class',
 'pub const fn change_class',
 'pub const fn command',
 'pub const fn command_id',
 'pub const fn command_id',
 'pub const fn confirmation',
 'pub const fn current_configuration_digest',
 'pub const fn current_configuration_revision',
 'pub const fn current_target_installation_revision',
 'pub const fn event_digest',
 'pub const fn evidence_digest',
 'pub const fn evidence_digest',
 'pub const fn evidence_digest',
 'pub const fn evidence_digest',
 'pub const fn evidence_id',
 'pub const fn evidence_id',
 'pub const fn evidence_id',
 'pub const fn expected_update_revision',
 'pub const fn expected_update_revision',
 'pub const fn get',
 'pub const fn installation_id',
 'pub const fn installation_id',
 'pub const fn installation_id',
 'pub const fn installation_revision',
 'pub const fn installation_state_digest',
 'pub const fn kind',
 'pub const fn last_sequence',
 'pub const fn outcome',
 'pub const fn plan',
 'pub const fn plan_digest',
 'pub const fn plan_digest',
 'pub const fn plan_digest',
 'pub const fn post_revision',
 'pub const fn readiness',
 'pub const fn revision',
 'pub const fn rollback_admission_snapshot_digest',
 'pub const fn rollback_capability_authority_digest',
 'pub const fn rollback_catalog_digest',
 'pub const fn rollback_catalog_revision',
 'pub const fn rollback_catalog_revision',
 'pub const fn rollback_component_authority_digest',
 'pub const fn rollback_component_authority_digest',
 'pub const fn rollback_evidence',
 'pub const fn rollback_package',
 'pub const fn rollback_package_digest',
 'pub const fn rollback_pin',
 'pub const fn rollback_pin_digest',
 'pub const fn rollback_registry_digest',
 'pub const fn rollback_registry_revision',
 'pub const fn rollback_registry_revision',
 'pub const fn rollback_source_policy_identity',
 'pub const fn sequence',
 'pub const fn staged_configuration_digest',
 'pub const fn staged_configuration_digest',
 'pub const fn staged_configuration_digest',
 'pub const fn staged_configuration_revision',
 'pub const fn staged_installation_revision',
 'pub const fn staged_installation_revision',
 'pub const fn staged_installation_revision',
 'pub const fn state',
 'pub const fn target_capability_authority_digest',
 'pub const fn target_catalog_digest',
 'pub const fn target_catalog_revision',
 'pub const fn target_catalog_revision',
 'pub const fn target_component_authority_digest',
 'pub const fn target_component_authority_digest',
 'pub const fn target_configuration_admission_snapshot_digest',
 'pub const fn target_package',
 'pub const fn target_package_digest',
 'pub const fn target_pin',
 'pub const fn target_pin_digest',
 'pub const fn target_registry_digest',
 'pub const fn target_registry_revision',
 'pub const fn target_registry_revision',
 'pub const fn target_source_execution_policy_admission_snapshot_digest',
 'pub const fn target_source_policy_identity',
 'pub const fn tenant_id',
 'pub const fn tenant_id',
 'pub const fn update_id',
 'pub const fn update_id',
 'pub const fn update_id',
 'pub const fn update_id',
 'pub const fn update_id',
 'pub const fn update_id',
 'pub const fn user_id',
 'pub const fn user_id',
 'pub const fn verified_rollback_artifact_set_digest',
 'pub const fn verified_rollback_artifact_set_digest',
 'pub const fn verified_target_artifact_set_digest',
 'pub enum UpdateChangeClass',
 'pub enum UpdateCommandOutcome',
 'pub enum UpdateConstructionError',
 'pub enum UpdateDecisionError',
 'pub enum UpdateEventKind',
 'pub enum UpdateReplayError',
 'pub enum UpdateRepositoryError',
 'pub enum UpdateState',
 'pub fn apply',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn as_str',
 'pub fn cancel',
 'pub fn cancel_after_terminal_installation',
 'pub fn confirm_applied_update',
 'pub fn decide',
 'pub fn evolve',
 'pub fn fail_next_commit_for_test',
 'pub fn for_sequence',
 'pub fn new',
 'pub fn new',
 'pub fn next',
 'pub fn parse',
 'pub fn parse',
 'pub fn parse',
 'pub fn parse',
 'pub fn parse',
 'pub fn record_approval',
 'pub fn replay',
 'pub fn rollback',
 'pub fn rollback_capability_definitions',
 'pub fn rollback_component_declarations',
 'pub fn rollback_components',
 'pub fn stage',
 'pub fn target_capability_definitions',
 'pub fn target_component_declarations',
 'pub fn target_components',
 'pub fn try_from_authority_histories',
 'pub struct InMemoryPackageUpdateRepository',
 'pub struct PackageUpdateAggregate',
 'pub struct PackageUpdateId',
 'pub struct PackageUpdatePlan',
 'pub struct RollbackReadinessEvidence',
 'pub struct UpdateApprovalEvidence',
 'pub struct UpdateApprovalId',
 'pub struct UpdateCommand',
 'pub struct UpdateCommandId',
 'pub struct UpdateCommandReceipt',
 'pub struct UpdateConfirmationEvidence',
 'pub struct UpdateDecisionContext',
 'pub struct UpdateEvent',
 'pub struct UpdateEventSequence',
 'pub struct UpdateEvidenceId',
 'pub struct UpdateReadinessEvidence',
 'pub struct UpdateRevision',
 'pub trait PackageUpdateRepository',
 'pub type PackageUpdateSnapshot')
PLATFORM_UPDATE_ADMITTED_UNCLASSIFIED_PUBLIC = ('pub(in crate::market) fn fr',
 'pub(in crate::market) fn fr',
 'pub(in crate::market) fn fr',
 'pub(in crate::market) fn fr',
 'pub(in crate::market) fn fo',
 'pub(in crate::market) fn fo',
 'pub(in crate::market) fn fo',
 'pub(in crate::market) fn fo',
 'pub(in crate::market) fn fo',
 'pub(in crate::market) fn fo',
 'pub(in crate::market) fn fo')
PLATFORM_UPDATE_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS = (
 'from_plan',
 'from_plan',
 'from_bindings',
 'from_bindings',
 'for_stage',
 'for_record_approval',
 'for_apply',
 'for_confirm_applied',
 'for_rollback',
 'for_cancel',
 'for_cancel_after_terminal_installation')
PLATFORM_UPDATE_TEST_ONLY_AUTHORITY_CARRIERS = (
 '#[cfg(test)]\n    fn from_parts(',
 '#[cfg(test)]\nfn synthetic_publications_for_plan_authority(')
PLATFORM_UPDATE_PRODUCTION_CONTEXT_CALLS = (
 'UpdateDecisionContext::for_stage(',
 'UpdateDecisionContext::for_record_approval(',
 'UpdateDecisionContext::for_apply(',
 'UpdateDecisionContext::for_confirm_applied(',
 'UpdateDecisionContext::for_rollback(',
 'UpdateDecisionContext::for_cancel()',
 'UpdateDecisionContext::for_cancel_after_terminal_installation(')
PLATFORM_UPDATE_PRODUCTION_NAMED_CONTEXT_DECLARATIONS = 3
PLATFORM_UPDATE_ADMITTED_MACRO_INVOCATION_COUNTS = (('assert', 25),
 ('assert_eq', 117),
 ('assert_ne', 3),
 ('format', 44),
 ('matches', 22),
 ('panic', 8),
 ('parsed', 119),
 ('unreachable', 14),
 ('vec', 43),
 ('write', 4))
PLATFORM_UPDATE_ADMITTED_PARSED_ARGUMENT_COUNTS = (('CapabilityId, *cap', 1),
 ('CapabilityId, cap_public()', 2),
 ('CapabilityId, capability', 1),
 ('CatalogRevision,', 1),
 ('CatalogRevision, catalog', 2),
 ('CatalogRevision, rev', 1),
 ('ComponentId, id', 1),
 ('ComponentVersion, version', 1),
 ('ExecutionIdentity, exec', 1),
 ('ExecutionIdentity, execution', 1),
 ('GrantApprovalId, format!( )', 1),
 ('GrantCommandId,', 1),
 ('GrantCommandId, format!( )', 2),
 ('GrantSnapshotId, format!( )', 2),
 ('GrantVersion,', 1),
 ('InstallationCommandId,', 12),
 ('InstallationCommandId, format!( )', 3),
 ('InstallationId,', 1),
 ('PackageUpdateId,', 11),
 ('PackageUpdateId, format!( )', 1),
 ('PolicyRevision,', 1),
 ('PolicyRevision, format!( )', 1),
 ('PolicySnapshotId,', 1),
 ('PolicySnapshotId, format!( )', 1),
 ('Sha256Digest, format!( )', 8),
 ('Sha256Digest, format!( , ch.to_string().repeat(64))', 1),
 ('Sha256Digest, source_digest', 1),
 ('SourcePolicyId,', 1),
 ('SourcePolicyId, source_id', 1),
 ('TenantId,', 1),
 ('UpdateApprovalId,', 4),
 ('UpdateApprovalId, format!( )', 2),
 ('UpdateCommandId,', 28),
 ('UpdateCommandId, format!( )', 4),
 ('UpdateEvidenceId,', 10),
 ('UpdateEvidenceId, format!( )', 3),
 ('UpdateRevision,', 3),
 ('UserId,', 1))
# `#` `!` `[` with any whitespace between: an inner attribute is a token sequence, and
# `# /*x*/ ! [cfg(any())]` excludes a module or a whole crate exactly as `#![cfg(any())]` does.
RUST_INNER_ATTRIBUTE_PATTERN = r"#\s*!\s*\["
# The bound test file defines exactly these macros. A local `macro_rules! assert_eq` shadows the
# standard macro for every admitted `assert_eq!` invocation in the suite, so pinning invocation
# NAMES is not enough — the definition that binds the name is the executable carrier.
PLATFORM_IDENTITY_ADMITTED_TEST_MACROS = ("assert_kind_enforces_grammar",)
# The admitted helper is pinned as a COMPLETE executable carrier, not by name plus one matcher
# line. Rust reads the first matching arm, so an earlier `($ignored:expr)` arm intercepts every
# `helper!(TenantId)` call while the real `($kind:ty)` arm below stays unread; and a single arm
# whose body is a no-op neuters the grammar oracle while production is broken. So the arm-matcher
# list is exactly one `($kind:ty)` arm, and its body must still carry the load-bearing checks.
PLATFORM_IDENTITY_TEST_HELPER_MACRO = "assert_kind_enforces_grammar"
PLATFORM_IDENTITY_TEST_HELPER_ARMS = ("($kind:ty)",)
PLATFORM_IDENTITY_HELPER_BODY_CARRIERS = (
    "<$kind>::parse",
    "error.value_kind()",
    "error.kind()",
    "serde_json::from_str",
)
# A block-local `use std::assert as assert_eq;` rebinds `assert_eq!` for the rest of its scope
# without adding a `macro_rules!` or changing an invocation NAME, so a `bite` guard that ran
# earlier cannot see it. Every `use`/`type`/`mod` item of the bound test file is therefore
# accounted for exactly, the same total accounting the governed sources already carry, so any
# macro-aliasing `use` — top-level or inside a block after any guard — is drift that fails.
PLATFORM_IDENTITY_ADMITTED_TEST_ITEMS = (
    "use std::any::TypeId;",
    "use std::collections::hash_map::DefaultHasher;",
    "use std::error::Error;",
    "use std::hash::{Hash, Hasher};",
    # The owned-string deserializer that reaches `visit_string`, which `from_str` never does.
    "use serde::Deserialize;",
    "use serde::de::IntoDeserializer;",
    "use serde::de::value::{BytesDeserializer, Error as SerdeValueError, StringDeserializer};",
    "use ustc_campus_agent_core::identity::{ CommandId, CorrelationId, IdentityValueError, "
    "IdentityValueErrorKind, RequestId, SessionId, TenantId, UserId, };",
    "use ustc_campus_agent_core::invocation;",
)
# Two implementations that are only claimed to agree diverge silently. Both carriers compare
# their own lexer output against this one committed corpus, so a divergence fails whichever
# side drifted rather than surviving until a reviewer happens to probe the right input.
RUST_LEXICAL_CORPUS = "scripts/tests/data/rust_lexical_corpus.json"
MIN_RUST_LEXICAL_CORPUS_CASES = 50
# Classes that have each produced a real divergence or bypass; the corpus is not evidence
# without them.
REQUIRED_RUST_LEXICAL_CORPUS_CASES = (
    "extern/**/crate self as z;",
    "# /*inner*/ ! [allow(dead_code)]",
    'include/*x*/!("a.rs");',
    "let x = foo.r#type;",
    "macro_rules! assert_eq { ($($a:tt)*) => {{ }}; }",
    # An earlier catch-all arm intercepts every call while the real arm stays present below.
    "macro_rules! g { ($x:expr) => {{ 1 }}; ($k:ty) => {{ 2 }}; }",
    # `#` `[` `derive` `(` with spacing: a derive synthesizes an impl no scan sees.
    "# [derive(Clone, Copy)]",
    # Whitespace between `macro_rules` and `!` still defines a macro.
    "macro_rules !shadow { () => {{}}; }",
    # An attribute NAME may be a raw identifier; each of these is the built-in attribute.
    "#[r#derive(Default)]",
    "# [ r#derive ( Copy ) ]",
    "#[r#ignore] #[r#test] fn t() {}",
    "enum E { #[r#default] A }",
    # A `#` after an identifier byte is not an attribute head; a bare `$` names itself.
    "x#[a] pub fn f() {}",
    # A body pin is only as good as the body extractor: a nested generic bound, a brace inside
    # the parameter list, a body-less declaration, a function POINTER type and a raw-identifier
    # name each break a naive one, and a nested function must be seen in its own right.
    "fn f<T: Into<Vec<u8>>>(x: T) {}",
    "fn g(x: [u8; { 2 }]) { 1 }",
    "trait T { fn h(&self); }",
    "let p: fn(u8) -> u8 = q;",
    "fn r#match() { 1 }",
    "fn outer() { fn inner() { 1 } }",
    "fn \u00e9q() { 1 }",
    "let a = \"x\"; /*c*/ let b = b'-'; let c = r#\"r\"#; let d = b\"y\"; // t",
    "#[$]",
)
PLATFORM_IDENTITY_TEST_MACRO_MATCHER = "($kind:ty) => {{"
# No governed source may redefine an assertion, formatting or control macro under any name the
# evidence relies on, whatever the admitted-definition list says.
RUST_SHADOWABLE_MACRO_NAMES = (
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "matches",
    "panic",
    "unreachable",
    "write",
    "writeln",
    "format",
    "concat",
    "stringify",
    "include_str",
    "include_bytes",
    "vec",
)
PLATFORM_CORE_MANIFEST = "crates/platform-core/Cargo.toml"
# Cargo decides the compiled target set, and it can be redirected without touching one line of
# Rust: `[lib] path`, `[package] build`, `[[bin]]`, `[[example]]`, `[[bench]]` and `[[test]]`
# all name source files outside anything a Rust scan reads. The manifest is therefore pinned by
# exact key sets rather than screened for individual keys.
PLATFORM_CORE_ADMITTED_MANIFEST_TABLES = (
    "dependencies",
    "dev-dependencies",
    "lib",
    "lints",
    "package",
)
PLATFORM_CORE_ADMITTED_MANIFEST_PACKAGE_KEYS = (
    "authors",
    "edition",
    "homepage",
    "license",
    "name",
    "repository",
    "rust-version",
    "version",
)
PLATFORM_CORE_ADMITTED_MANIFEST_DEPENDENCIES = (
    "semver",
    "serde",
    "serde_json",
    "ustc-agent-tool-protocol",
)
PLATFORM_CORE_ADMITTED_MANIFEST_DEV_DEPENDENCIES = ("hex",)
PLATFORM_CORE_ADMITTED_MANIFEST_LIB = {"path": "src/lib.rs"}
# The `*.rs` inventory above cannot see a module source that does not end in `.rs`. Fixtures
# are governed by their own digest check, so everything else in the package is pinned here.
PLATFORM_CORE_ADMITTED_NON_SOURCE_FILES = ("Cargo.toml",)
PLATFORM_CORE_GOVERNED_FIXTURE_PREFIX = "tests/fixtures/"
PLATFORM_IDENTITY_ADMITTED_DERIVES = (
    "Debug, Clone, Copy, PartialEq, Eq",
    "Debug, Clone, Copy, PartialEq, Eq",
    "Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash",
)
PLATFORM_IDENTITY_FORBIDDEN_PUBLIC_ITEM_KINDS = (
    "pub type",
    "pub use",
    "pub mod",
    "pub trait",
    "pub static",
    "#[macro_export]",
)
# The AUTH-012 binding runs this exact test, so checking only that the function name exists
# lets a gutted body keep the binding green. These carriers must survive inside its body.
PLATFORM_IDENTITY_AUTH012_TEST = "identity_values_are_exact_and_nominal"
PLATFORM_IDENTITY_AUTH012_BODY_CARRIERS = (
    "assert_public_surface_is_frozen()",
    "assert_bound_test_envelope_is_active()",
    "assert_assertion_macros_bite()",
    "assert_lexer_matches_the_shared_corpus()",
    "serde_json::to_string(",
    "serde_json::from_str",
    "ordered.sort()",
    "hash_of(",
    "TypeId::of::<TenantId>()",
    "TypeId::of::<CorrelationId>()",
)
MIN_PLATFORM_IDENTITY_AUTH012_ASSERTIONS = 16
# The AUTH-011 body carries the grammar evidence, and one of its guards is the only rule that
# reaches the bytes INSIDE a literal — the residue the frozen function-body table cannot pin.
# Dropping that call would leave the delimiter set proven by a hand-picked corpus alone.
PLATFORM_IDENTITY_AUTH011_TEST = "identity_values_enforce_canonical_bounds_and_errors"
PLATFORM_IDENTITY_AUTH011_BODY_CARRIERS = (
    "assert_assertion_macros_bite()",
    "assert_bound_test_envelope_is_active()",
    "assert_kind_enforces_grammar!(TenantId)",
    "assert_kind_enforces_grammar!(CorrelationId)",
    "assert_grammar_is_exhaustive_over_bytes()",
    "assert_grammar_semantics_match_the_contract()",
    # The effective-use half of the grammar: the declared bound and the frozen body table are both
    # mutable, so dropping this call would leave a local constant free to carry a different bound.
    "assert_effective_max_byte_bound_is_contract_bound()",
)
# The exhaustive oracle must actually walk the byte alphabet, not a truncated slice of it.
PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE = "assert_grammar_is_exhaustive_over_bytes"
PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE_CARRIERS = (
    "for byte in 0_u8..=u8::MAX {",
    "IdentityValueErrorKind::InvalidStart",
    "IdentityValueErrorKind::InvalidCharacter { byte_index: 1 }",
    "IdentityValueErrorKind::InvalidEnd",
)
# Each phrase is a pinned rustdoc carrier that must be immediately followed by one
# ```compile_fail fence. Together they cover every forbidden-API category in AUTH-012.
PLATFORM_IDENTITY_COMPILE_FAIL_CATEGORIES = (
    "The private backing field cannot be constructed directly:",
    "A default identity value does not exist:",
    "There is no unchecked constructor:",
    "The backing string cannot be mutated:",
    "One identity kind cannot convert into another:",
    "Identifier shape is not interpreted:",
    "The value does not dereference to its backing string:",
    # rustc itself proving the round-13 bypass class is gone: with a named-field struct the
    # type name is not a value, so it can be neither bound nor called.
    "The type name is not a constructor value:",
    "There is no tuple constructor call:",
)
MIN_PLATFORM_IDENTITY_COMPILE_FAIL_CASES = 9
# A `compile_fail` fence proves only that SOMETHING failed to compile. Swapping its body for an
# unrelated type error keeps the fence, the category prose and the case count while the denied
# API becomes reachable, so each proof is pinned to the expression it advertises.
PLATFORM_IDENTITY_COMPILE_FAIL_EXPRESSIONS = {
    "The private backing field cannot be constructed directly:": "{ value: String::from(",
    "A default identity value does not exist:": "::default()",
    "There is no unchecked constructor:": "::new(",
    "The backing string cannot be mutated:": ".as_mut_str()",
    "One identity kind cannot convert into another:": "::from(",
    "Identifier shape is not interpreted:": ".prefix()",
    "The value does not dereference to its backing string:": "&**",
    "The type name is not a constructor value:": "let build = CommandId;",
    "There is no tuple constructor call:": "let correlation = CorrelationId(String::from(",
}
# Every attribute of every governed source is accounted for by normalized NAME. An attribute
# name is an ordinary identifier and may be written `r#name`, so blacklisting `derive` or
# `ignore` by spelling leaves the raw form open; an exact admitted name set closes the class,
# including attributes nobody predicted. Derive ARGUMENTS are pinned separately, because a
# derive is the one attribute that adds public API.
PLATFORM_CORE_ADMITTED_ATTRIBUTE_NAMES = {'identity.rs': ('$attribute', 'derive', 'doc', 'must_use'),
 'invocation.rs': ('derive', 'must_use'),
 'lib.rs': ('cfg', 'derive', 'must_use', 'serde', 'test'),
 'market.rs': ('derive', 'must_use', 'serde'),
 'market/authority.rs': ('cfg', 'derive', 'must_use', 'test'),
 'market/capability.rs': ('cfg', 'derive', 'must_use', 'serde', 'test'),
 'market/grant.rs': ('allow', 'cfg', 'derive', 'must_use', 'test'),
 'market/installation.rs': ('allow', 'cfg', 'derive', 'must_use', 'test'),
 'market/update.rs': ('allow', 'cfg', 'derive', 'must_use', 'test'),
 'session.rs': ('cfg', 'derive', 'must_use', 'serde', 'test'),
 'source_registry.rs': ('$attribute', 'allow', 'cfg', 'derive', 'doc', 'must_use', 'test')}
# `market/grant.rs` carries lint-affecting `allow` attributes, so names alone are not enough:
# freeze the complete normalized attribute-body multiset. Counts are literal reviewed evidence,
# not a minimum or a projection generated from the governed source at checker runtime.
PLATFORM_GRANT_ADMITTED_ATTRIBUTE_COUNTS = (((False, 'allow', 'allow(clippy::large_enum_variant)'), 1),
 ((False, 'allow', 'allow(clippy::new_without_default)'), 1),
 ((False, 'allow', 'allow(clippy::too_many_arguments)'), 1),
 ((False, 'allow', 'allow(dead_code)'), 3),
 ((False, 'cfg', 'cfg(test)'), 1),
 ((False, 'derive', 'derive(Clone)'), 1),
 ((False, 'derive', 'derive(Clone, PartialEq, Eq)'), 8),
 ((False, 'derive', 'derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)'), 2),
 ((False, 'derive', 'derive(Debug, Clone, Copy, PartialEq, Eq)'), 6),
 ((False, 'derive', 'derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)'), 1),
 ((False, 'derive', 'derive(Debug, Clone, PartialEq, Eq)'), 5),
 ((False, 'derive', 'derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)'), 1),
 ((False, 'must_use', 'must_use'), 66),
 ((False, 'test', 'test'), 20))
PLATFORM_IDENTITY_ADMITTED_TEST_ATTRIBUTE_NAMES = ("test",)
# Pinning dependency NAMES pins nothing about what those names resolve to. `semver = { path =
# "crates/fake-semver" }` keeps the admitted name while Cargo compiles an attacker-authored
# crate, and every Rust scan still sees `semver::Version`. Dependency SPECIFICATIONS and the
# RESOLVED identity in Cargo.lock are therefore pinned too.
#
# The resolved graph is read from the committed `Cargo.lock` rather than from
# `cargo metadata --locked --offline`: `Cargo.lock` IS the file that command resolves against,
# and the `docs-and-contracts` CI job — the one that runs this checker on every pull request —
# installs Python only, with no Rust toolchain, so shelling out to cargo would make the gate
# unrunnable there. Parsing the lock keeps the rule in the always-run carrier.
WORKSPACE_MANIFEST = "Cargo.toml"
WORKSPACE_LOCKFILE = "Cargo.lock"
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
# Exact specification of every workspace dependency. A bare string is an exact version
# requirement; a table is compared key for key, so adding `path`, `git`, `registry` or
# `default-features` is drift rather than an unnoticed redirect.
WORKSPACE_ADMITTED_DEPENDENCIES = {
    "serde": {"version": "1.0.229", "features": ["derive"]},
    "serde_json": "1.0.151",
    "semver": "1.0.27",
    "sha2": "0.10.9",
    "time": {"version": "0.3.54", "features": ["parsing"]},
    # The single admitted local path dependency, pinned to its exact in-repo location.
    "ustc-agent-tool-protocol": {"path": "crates/agent-tool-protocol"},
}
# Tables that redirect where a dependency is fetched from. None may appear in any manifest.
CARGO_FORBIDDEN_SOURCE_TABLES = ("patch", "replace", "source")
# `.cargo/config.toml` can replace a whole registry for the build without touching a manifest.
CARGO_CONFIG_FILENAMES = ("config.toml", "config")
# Exact resolved identity of every direct dependency of the governed package. `source` is the
# registry URL for an external crate and absent for an in-repo path dependency, so a redirect to
# a local fake changes this even when the name and version are preserved verbatim.
PLATFORM_CORE_RESOLVED_DEPENDENCIES = {
    "semver": CRATES_IO_SOURCE,
    "serde": CRATES_IO_SOURCE,
    "hex": CRATES_IO_SOURCE,
    "serde_json": CRATES_IO_SOURCE,
    "ustc-agent-tool-protocol": None,
}
# Exact specification of every platform-core dependency, mirroring the workspace rule.
PLATFORM_CORE_ADMITTED_DEPENDENCY_SPECS = {
    "dependencies": {
        "semver": {"workspace": True},
        "serde": {"workspace": True},
        "serde_json": {"workspace": True},
        "ustc-agent-tool-protocol": {"workspace": True},
    },
    "dev-dependencies": {
        "hex": "0.4.3",
    },
}
PLATFORM_IDENTITY_TEST_FUNCTIONS = (
    "identity_values_enforce_canonical_bounds_and_errors",
    "identity_values_are_exact_and_nominal",
    "identity_errors_never_echo_rejected_input",
    "identity_module_has_no_generation_or_adapter_surface",
    "market_invocation_authority_uses_m00_identity_definitions",
)
# Every row runs the checker first. A Rust test cannot prove that it ran: a redirected
# `[[test]]` target or a renamed function makes `--exact` match nothing, which cargo reports as
# "running 0 tests" at exit 0. The out-of-band carrier that pins the manifest target set and
# the bound function names is therefore part of each binding, not a separate courtesy check.
PLATFORM_IDENTITY_ACCEPTANCE_BINDINGS = {
    "AUTH-011": (
        "python3 scripts/check_repo_contracts.py && "
        "cargo test --locked -p ustc-campus-agent-core --test platform_identity "
        "identity_values_enforce_canonical_bounds_and_errors -- --exact"
    ),
    "AUTH-012": (
        "python3 scripts/check_repo_contracts.py && "
        "cargo test --locked -p ustc-campus-agent-core --test platform_identity "
        "identity_values_are_exact_and_nominal -- --exact && "
        "cargo test --locked -p ustc-campus-agent-core --doc identity"
    ),
    "AUTH-014": (
        "python3 scripts/check_repo_contracts.py && "
        "cargo test --locked -p ustc-campus-agent-core --test platform_identity "
        "identity_errors_never_echo_rejected_input -- --exact"
    ),
    "AUTH-015": (
        "python3 scripts/check_repo_contracts.py && "
        "cargo test --locked -p ustc-campus-agent-core --test platform_identity "
        "identity_module_has_no_generation_or_adapter_surface -- --exact"
    ),
    "AUTH-016": (
        "python3 scripts/check_repo_contracts.py && "
        "cargo test --locked -p ustc-campus-agent-core --test platform_identity "
        "market_invocation_authority_uses_m00_identity_definitions -- --exact"
    ),
}


def strip_rust_comments_and_literals(source: str, keep_literals: bool = False) -> str:
    """Replaces Rust comments and ordinary/byte/raw string and char literals with one space.

    Only code carriers survive, so documentation prose and test sentinels cannot trip a
    forbidden-carrier scan, and a forbidden import cannot hide inside a string.

    Each removed span becomes a single space rather than nothing, because a comment is a token
    SEPARATOR in Rust. Deleting it welds the neighbouring tokens together: `extern/**/crate`
    would become the single identifier `externcrate`, which no scan for `extern crate` and no
    `\bextern\b` token match can see, while Rust still reads two keywords and compiles the
    item. Preserving the boundary is what makes every downstream rule token-accurate.

    With `keep_literals`, comments are still removed but literal SPANS are emitted verbatim.
    That mode exists for one purpose: the grammar's semantics live inside literals — the byte
    set `b'-' | b'.' | b'_' | b':'` and the bound `128` — and a fingerprint taken over
    stripped literals pins control flow while leaving those bytes free to drift. Every
    token-accounting rule keeps using the stripping mode; only the targeted semantic checks
    read this one, so a literal can never satisfy a carrier scan by accident.
    """
    output: list[str] = []
    index = 0
    length = len(source)
    while index < length:
        if source.startswith("//", index):
            newline = source.find("\n", index)
            index = length if newline == -1 else newline
            output.append(" ")
            continue
        if source.startswith("/*", index):
            depth = 1
            index += 2
            while index < length and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            output.append(" ")
            continue
        raw = re.compile(r'(?:b|br|rb)?r(#*)"').match(source, index)
        if raw is not None:
            start = index
            terminator = '"' + raw.group(1)
            end = source.find(terminator, raw.end())
            index = length if end == -1 else end + len(terminator)
            output.append(source[start:index] if keep_literals else " ")
            continue
        quote = re.compile(r'b?"').match(source, index)
        if quote is not None:
            start = index
            index = quote.end()
            while index < length:
                if source[index] == "\\":
                    index += 2
                    continue
                if source[index] == '"':
                    index += 1
                    break
                index += 1
            output.append(source[start:index] if keep_literals else " ")
            continue
        # A char literal has a closing quote; a lifetime does not.
        char_literal = re.compile(r"b?'(?:\\.|[^\\'])'").match(source, index)
        if char_literal is not None:
            start = index
            index = char_literal.end()
            output.append(source[start:index] if keep_literals else " ")
            continue
        output.append(source[index])
        index += 1
    return "".join(output)


RUST_PUBLIC_ITEM_KEYWORDS = (
    "fn",
    "struct",
    "enum",
    "union",
    "trait",
    "type",
    "mod",
    "use",
    "static",
    "const",
)


def rust_public_declarations(code: str) -> tuple[list[str], list[str]]:
    """Accounts for EVERY `pub` token in already-stripped Rust code.

    Returns the sorted declaration fingerprints and any `pub` occurrence that could not be
    classified. Fingerprints keep function qualifiers, so `pub async fn parse` is a different
    declaration from `pub fn parse`. Anything unclassified must fail the caller: a positive
    allowlist is only as complete as the grammar that feeds it.
    """
    fingerprints: list[str] = []
    unclassified: list[str] = []
    declaration = re.compile(
        r"\s*((?:(?:const|async|unsafe|extern(?:\s+\"[^\"]*\")?)\s+)*)"
        rf"({'|'.join(RUST_PUBLIC_ITEM_KEYWORDS)})\b\s*([A-Za-z_$][A-Za-z0-9_]*)?"
    )
    for token in re.finditer(r"\bpub\b", code):
        tail = code[token.end() :]
        if re.match(r"\s*\(", tail):
            # Restricted visibility (`pub(crate)`, `pub(in ...)`) is not admitted here.
            unclassified.append(f"pub{tail[:24].splitlines()[0] if tail else ''}")
            continue
        matched = declaration.match(tail)
        if matched is None:
            head = tail.strip().splitlines()[0] if tail.strip() else ""
            unclassified.append(f"pub {head[:40]}")
            continue
        parts = ["pub"]
        qualifiers = " ".join(matched.group(1).split())
        if qualifiers:
            parts.append(qualifiers)
        parts.append(matched.group(2))
        if matched.group(3):
            parts.append(matched.group(3))
        fingerprints.append(" ".join(parts))
    return sorted(fingerprints), unclassified


def rust_market_restricted_public_functions(code: str) -> tuple[str, ...]:
    """Pins full names of every owner-only `pub(in crate::market) fn` carrier."""
    return tuple(
        re.findall(
            r"\bpub\s*\(\s*in\s+crate::market\s*\)\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)",
            code,
        )
    )


def _rust_balanced_block(code: str, start: int) -> str | None:
    """Returns the brace-balanced block beginning at `start`, which must index a `{`."""
    if start >= len(code) or code[start] != "{":
        return None
    depth = 0
    for index in range(start, len(code)):
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth -= 1
            if depth == 0:
                return code[start : index + 1]
    return None


def _leading_type_path(text: str) -> str | None:
    """Returns the leading type path of `text`, including balanced generic arguments."""
    head = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)", text)
    if head is None:
        return None
    path = head.group(1)
    rest = text[head.end() :]
    if rest.startswith("<"):
        depth = 0
        for index, character in enumerate(rest):
            if character == "<":
                depth += 1
            elif character == ">":
                depth -= 1
                if depth == 0:
                    path += " ".join(rest[: index + 1].split())
                    break
    return path


def rust_impl_self_types(code: str) -> list[str]:
    """Returns the implemented self type of EVERY `impl` token, whatever its line position.

    Position-independent on purpose: the fingerprint used for the identity module's own
    allowlist records only the leading type path of an argument-position `impl`, which would
    drop the `for <Target>` of a real block hidden mid-line behind a decoy `fn`.
    """
    targets: list[str] = []
    for token in re.finditer(r"\bimpl\b", code):
        header = _rust_impl_header(code, token.end())
        if header is None:
            continue
        targets.append(rust_impl_self_type(header))
    return targets


def rust_impl_self_type(header: str) -> str:
    """Returns the implemented self type of an impl header, ignoring generics and `where`."""
    normalized = " ".join(header.split())
    if normalized.startswith("<"):
        depth = 0
        for index, character in enumerate(normalized):
            if character == "<":
                depth += 1
            elif character == ">":
                depth -= 1
                if depth == 0:
                    normalized = normalized[index + 1 :].strip()
                    break
    # A `where` clause follows the self type and must not be folded into it.
    normalized = re.split(r"\bwhere\b", normalized, maxsplit=1)[0].strip()
    target = normalized.rsplit(" for ", 1)[-1] if " for " in normalized else normalized
    return target.strip().rstrip(",").strip()


def _rust_impl_header(code: str, start: int) -> str | None:
    """Returns the text between `impl` and its opening brace, spanning line breaks."""
    angle = 0
    paren = 0
    for index in range(start, len(code)):
        char = code[index]
        if char == "<":
            angle += 1
        elif char == ">":
            angle -= 1
        elif char == "(":
            paren += 1
        elif char == ")":
            paren -= 1
        elif char == ";":
            return None
        elif char == "{" and angle <= 0 and paren <= 0:
            return code[start:index]
    return None


def rust_impl_declarations(code: str) -> tuple[list[str], list[str]]:
    """Accounts for EVERY `impl` token, including multiline headers."""
    fingerprints: list[str] = []
    unclassified: list[str] = []
    for token in re.finditer(r"\bimpl\b", code):
        line_start = code.rfind("\n", 0, token.start()) + 1
        before_on_line = code[line_start : token.start()]
        if before_on_line.strip():
            # A non-line-start `impl` is NOT skipped. Any positional heuristic can be defeated
            # by a decoy: `mod m { fn decoy() {} impl AsRef<str> for TenantId { .. } }` puts a
            # `fn` earlier on the same line while the `impl` is a real item. Instead every such
            # token is fingerprinted as `impl-arg <type path>` and must appear in the admitted
            # allowlist, so only the exact known `impl Trait` argument positions survive.
            argument = _leading_type_path(code[token.end() :])
            if argument is None:
                unclassified.append(f"impl {code[token.end() : token.end() + 40].strip()}")
            else:
                fingerprints.append(f"impl-arg {argument}")
            continue
        header = _rust_impl_header(code, token.end())
        if header is None:
            unclassified.append("impl <no block>")
            continue
        normalized = " ".join(header.split())
        if normalized.startswith("<"):
            depth = 0
            cut = None
            for index, char in enumerate(normalized):
                if char == "<":
                    depth += 1
                elif char == ">":
                    depth -= 1
                    if depth == 0:
                        cut = index
                        break
            if cut is None:
                unclassified.append(f"impl {normalized[:40]}")
                continue
            normalized = normalized[cut + 1 :].strip()
        if not normalized:
            unclassified.append("impl <empty>")
            continue
        fingerprints.append(f"impl {normalized}")
    return sorted(fingerprints), unclassified


# `extern` is governed as an ITEM, not as a forbidden substring. `extern crate self as x;`
# re-roots the crate under a second public name, and a comment may sit between its two
# keywords, so only token-level accounting can see it: after stripping, `extern/**/crate`
# normalizes to `extern crate` and the item fingerprint is compared against the allowlist.
RUST_GOVERNED_ITEM_KEYWORDS = ("extern", "mod", "use", "type")
# No macro may be named with a Rust keyword, so `if !(a || b)` is not an `if!` invocation.
RUST_KEYWORDS = frozenset(
    """
    as async await break const continue crate dyn else enum extern false fn for if impl in let
    loop match mod move mut pub ref return self Self static struct super trait true type union
    unsafe use where while yield
    """.split()
)


def _rust_attribute_envelope_start(code: str, start: int) -> int:
    """Returns the start of the attribute envelope immediately preceding `start`.

    Walks backwards over balanced `#[ ... ]` and `#![ ... ]` groups, so a wrapped or
    comment-interrupted attribute is still attached to the item it decorates. Each step moves
    strictly left, so the walk terminates.
    """
    cursor = start
    while True:
        head = code[:cursor].rstrip()
        if not head.endswith("]"):
            return cursor
        depth = 0
        index = len(head) - 1
        while index >= 0:
            if head[index] == "]":
                depth += 1
            elif head[index] == "[":
                depth -= 1
                if depth == 0:
                    break
            index -= 1
        if index <= 0:
            return cursor
        opener = index - 1
        if head[opener] == "!" and opener >= 1 and head[opener - 1] == "#":
            opener -= 1
        elif head[opener] != "#":
            return cursor
        cursor = opener


def rust_item_declarations(code: str) -> tuple[list[str], list[str]]:
    """Accounts for EVERY `mod`, `use` and `type` token in already-stripped Rust code.

    Returns the source-ordered item fingerprints and any occurrence that could not be
    terminated. A fingerprint spans the complete item header including its attribute envelope
    and visibility, so `#[path = ""] pub mod identity;` is a different declaration from
    `pub mod identity;`, and every spelling of a use tree — grouped, nested, `self`-rooted or
    unqualified — is one fingerprint that is either admitted verbatim or rejected.
    """
    fingerprints: list[str] = []
    unclassified: list[str] = []
    # `re.ASCII` so the word boundary matches Rust's ASCII-only identifier bytes, keeping this
    # scan token-for-token identical to the Rust mirror.
    pattern = rf"\b({'|'.join(RUST_GOVERNED_ITEM_KEYWORDS)})\b"
    for token in re.finditer(pattern, code, flags=re.ASCII):
        # A path segment, a field access, or a raw identifier such as `r#type` — none is an
        # item keyword.
        if code[: token.start()].rstrip().endswith((":", ".")) or code[
            : token.start()
        ].endswith("#"):
            continue
        visibility_start = token.start()
        visibility = re.search(r"\bpub\b(?:\s*\([^)]*\))?\s*$", code[: token.start()])
        if visibility is not None:
            visibility_start = visibility.start()
        item_start = _rust_attribute_envelope_start(code, visibility_start)
        inline_module = token.group(1) == "mod"
        depth = 0
        end = None
        terminator = None
        for index in range(token.end(), len(code)):
            char = code[index]
            if char == "{" and depth == 0 and inline_module:
                end = index
                terminator = char
                break
            if char in "([{":
                depth += 1
            elif char in ")]}":
                depth = max(0, depth - 1)
            elif char == ";" and depth == 0:
                end = index
                terminator = char
                break
        if end is None:
            unclassified.append(f"{token.group(1)} <unterminated>")
            continue
        # Normalize first, then re-attach the terminator: appending it beforehand would leave
        # a space before `;` whenever the item wraps, which the Rust mirror never produces.
        header = " ".join(code[item_start:end].split())
        fingerprints.append(header + ";" if terminator == ";" else header)
    return fingerprints, unclassified


def rust_macro_definitions(code: str) -> list[str]:
    """Returns the names of every `macro_rules!` definition in already-stripped Rust code."""
    return sorted(re.findall(r"\bmacro_rules\s*!\s*([A-Za-z_][A-Za-z0-9_]*)", code))


def rust_macro_arms(code: str) -> list[tuple[str, list[str]]]:
    """Returns `(name, [arm matcher, ...])` for every `macro_rules!` in source order.

    Pinning a macro's NAME and the presence of one matcher line is not the same as pinning the
    rule it applies. Rust selects the FIRST arm whose matcher matches, so an earlier catch-all
    arm — `($ignored:expr) => {{ .. }}` before `($kind:ty) => {{ .. }}` — intercepts every
    existing call while the real arm stays present below it, unread, and every name/line check
    still passes. Each arm is one balanced matcher group, `=>`, one balanced transcriber group
    and an optional `;`, so the complete matcher list is accounted for and either admitted
    verbatim or rejected. A malformed or unterminated arm is surfaced as a sentinel matcher
    rather than dropped, so the caller fails closed on it.
    """
    openers = {"(": ")", "[": "]", "{": "}"}
    definitions: list[tuple[str, list[str]]] = []
    for token in re.finditer(r"\bmacro_rules\s*!\s*([A-Za-z_][A-Za-z0-9_]*)", code):
        name = token.group(1)
        body_open = code.find("{", token.end())
        if body_open == -1:
            definitions.append((name, ["<unterminated-definition>"]))
            continue
        depth = 0
        body_end = None
        for index in range(body_open, len(code)):
            if code[index] == "{":
                depth += 1
            elif code[index] == "}":
                depth -= 1
                if depth == 0:
                    body_end = index
                    break
        if body_end is None:
            definitions.append((name, ["<unterminated-definition>"]))
            continue
        body = code[body_open + 1 : body_end]
        matchers: list[str] = []
        cursor = 0
        length = len(body)
        while True:
            while cursor < length and body[cursor].isspace():
                cursor += 1
            if cursor >= length:
                break
            opener = body[cursor]
            if opener not in openers:
                matchers.append(f"<unparsed:{body[cursor]}>")
                break
            group, cursor = _rust_balanced_group(body, cursor, opener, openers[opener])
            if group is None:
                matchers.append("<unterminated-matcher>")
                break
            matcher = " ".join(group.split())
            while cursor < length and body[cursor].isspace():
                cursor += 1
            if body[cursor : cursor + 2] != "=>":
                matchers.append(f"{matcher} <no-arrow>")
                break
            cursor += 2
            while cursor < length and body[cursor].isspace():
                cursor += 1
            if cursor >= length or body[cursor] not in openers:
                matchers.append(f"{matcher} <no-body>")
                break
            transcriber_open = body[cursor]
            _, cursor = _rust_balanced_group(
                body, cursor, transcriber_open, openers[transcriber_open]
            )
            if cursor is None:
                matchers.append(f"{matcher} <unterminated-body>")
                break
            matchers.append(matcher)
            while cursor < length and body[cursor].isspace():
                cursor += 1
            if cursor < length and body[cursor] == ";":
                cursor += 1
        definitions.append((name, matchers))
    return definitions


def _rust_balanced_group(
    text: str, start: int, opener: str, closer: str
) -> tuple[str | None, int | None]:
    """Consumes one balanced `opener..closer` group beginning at `start`.

    Returns `(group_text, end)` where `end` is the index just past the closing delimiter, or
    `(None, None)` when the group never closes.
    """
    depth = 0
    for index in range(start, len(text)):
        if text[index] == opener:
            depth += 1
        elif text[index] == closer:
            depth -= 1
            if depth == 0:
                return text[start : index + 1], index + 1
    return None, None


def _is_rust_ident_byte(character: str) -> bool:
    """True for the ASCII bytes Rust treats as identifier bytes, matching the Rust mirror."""
    return character.isascii() and (character.isalnum() or character == "_")


def _rust_leading_ident(text: str) -> str:
    """Returns the leading identifier of `text`, allowing a `$` macro-metavariable prefix.

    A byte-for-byte mirror of the Rust carrier's `leading_ident`, including its behaviour on a
    bare `$`: the two lexers are compared case for case, so "close enough" is a divergence.
    """
    end = 0
    if text.startswith("$"):
        end = 1
    while end < len(text) and _is_rust_ident_byte(text[end]):
        end += 1
    return text[:end]


# Reported in place of a name when a `fn` declaration's name is not an ASCII identifier.
RUST_UNNAMED_FUNCTION = "<unnamed>"


# A keyword before the type name means a declaration or an impl header, never a construction.
RUST_NON_CONSTRUCTION_KEYWORDS = frozenset(
    ("struct", "impl", "enum", "union", "trait", "for", "dyn", "as", "type")
)


def rust_newtype_constructions(code: str, names: tuple[str, ...]) -> list[str]:
    """Returns every expression that builds one of `names` through its private form.

    This is the closure the Serde rules could not reach by naming entry points. A newtype whose
    field is private can only be produced by its own tuple/struct-literal syntax inside the
    defining module, so counting THOSE expressions counts every construction path there is —
    an extra `visit_bytes` arm, an early return inside a helper, a branch, a new trait impl or
    a path nobody has thought of. Whichever entry point a deserializer picks, it must reach a
    construction site, and every site is listed here.

    A declaration (`pub struct $name(String);`) and an impl header (`impl Trait for $name {`)
    are not constructions and are excluded by the keyword immediately before the name.
    """
    constructions: list[str] = []
    for name in names:
        for token in re.finditer(re.escape(name) + r"\s*[({]", code):
            at = token.start()
            if at > 0 and _is_rust_ident_byte(code[at - 1]):
                continue
            head = code[:at].rstrip()
            # `Foo::$name(` is an associated call or an enum variant, not this newtype's ctor.
            if head.endswith("::"):
                continue
            keyword = re.search(r"([A-Za-z_][A-Za-z0-9_]*)\s*$", head, flags=re.ASCII)
            if keyword is not None and keyword.group(1) in RUST_NON_CONSTRUCTION_KEYWORDS:
                continue
            # Canonical: the name joined to its delimiter with no whitespace. `Self {` and
            # `Self{` are the SAME construction, so the normalization collapses the gap rather
            # than preserving it — otherwise the admitted list becomes a list of spellings
            # again, and the two carriers can normalize the same source differently.
            constructions.append(name + code[token.end() - 1])
    return constructions


def rust_attributes(code: str) -> tuple[list[tuple[bool, str, str]], list[str]]:
    """Accounts for EVERY attribute in already-stripped Rust code, in source order.

    Returns `[(is_inner, name, body), ...]` plus any attribute that could not be terminated.

    An attribute is a token sequence whose NAME is an ordinary identifier, and Rust accepts an
    identifier written as a raw identifier. `#[r#derive(Default)]` derives exactly as
    `#[derive(Default)]` does, `#[r#ignore]` suppresses a test, `#[r#default]` picks an enum
    default — and none of them contains the substring a literal scan looks for. `#`, `!` and `[`
    may also be separated by whitespace, which is what a comment strips to. So the name is
    normalized (`r#` removed) and the punctuation is matched tolerantly, and callers account for
    the whole attribute set rather than screening for individual spellings: the next equivalent
    carrier is admitted or rejected without anyone having predicted it.
    """
    attributes: list[tuple[bool, str, str]] = []
    unterminated: list[str] = []
    for match in re.finditer(r"#\s*(!)?\s*\[", code):
        # `r#derive` is a raw identifier, not an attribute head: its `#` has an identifier byte
        # before it. The Rust mirror skips those, so this must too or the two lexers disagree.
        # ASCII-only on purpose: Rust tests one BYTE, so a multi-byte character's continuation
        # byte is not an identifier byte, while Python's Unicode-aware `isalnum` would say it is.
        start = match.start()
        if start > 0 and _is_rust_ident_byte(code[start - 1]):
            continue
        bracket = match.end() - 1
        group, _ = _rust_balanced_group(code, bracket, "[", "]")
        if group is None:
            unterminated.append(code[match.start() : match.start() + 24])
            continue
        body = " ".join(group[1:-1].split())
        head = body[2:] if body.startswith("r#") else body
        # A leading `$` is kept: inside a `macro_rules!` transcriber `#[$attribute]` forwards a
        # caller-supplied attribute, and naming that metavariable is more useful than an empty
        # name. This mirrors the Rust `leading_ident` byte for byte — including a bare `$` with
        # no identifier after it — and the corpus pins the pair.
        attributes.append((match.group(1) is not None, _rust_leading_ident(head), body))
    return attributes, unterminated


def rust_attribute_names(attributes: list[tuple[bool, str, str]]) -> tuple[str, ...]:
    """Returns the sorted, deduplicated normalized attribute names."""
    return tuple(sorted({name for _, name, _ in attributes}))


def rust_derive_bodies(code: str) -> list[str]:
    """Returns the normalized argument list of every derive attribute, in source order.

    Built on the shared attribute parser rather than on its own pattern, so a derive reached by
    any equivalent spelling — spaced, comment-split or raw-identifier — is one carrier here. A
    derive synthesizes a trait implementation that appears nowhere as text, so no `use`, `type`
    or `impl` accounting can see it; this is the only place it is counted.
    """
    bodies: list[str] = []
    attributes, _ = rust_attributes(code)
    for _, name, body in attributes:
        if name != "derive":
            continue
        arguments = body[body.index("(") + 1 : body.rindex(")")] if "(" in body else ""
        bodies.append(" ".join(arguments.split()))
    return bodies


def rust_macro_invocation_arguments(code: str) -> tuple[list[tuple[str, str]], list[str]]:
    """Returns `(name, argument)` for every macro invocation plus any unterminated name.

    Delimiter-balanced, and whitespace-tolerant between the name, the `!` and the delimiter,
    because `include /* x */ !("f.rs")` is the same invocation as `include!("f.rs")`. A Rust
    keyword is never a macro name, so `if !(a || b)` is not an invocation of `if`. An
    unterminated invocation is reported rather than dropped: the caller must fail closed on it
    rather than silently account for one fewer macro than the source contains.
    """
    invocations: list[tuple[str, str]] = []
    unterminated: list[str] = []
    closing = {"(": ")", "[": "]", "{": "}"}
    for token in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*!\s*([({\[])", code, flags=re.ASCII):
        name = token.group(1)
        if name == "macro_rules" or name in RUST_KEYWORDS:
            continue
        opener = token.group(2)
        start = token.end() - 1
        depth = 0
        closed = False
        for index in range(start, len(code)):
            if code[index] == opener:
                depth += 1
            elif code[index] == closing[opener]:
                depth -= 1
                if depth == 0:
                    invocations.append((name, " ".join(code[start + 1 : index].split())))
                    closed = True
                    break
        if not closed:
            unterminated.append(name)
    return invocations, unterminated


def rust_attribute_block(code: str, name: str) -> list[str] | None:
    """Returns the attributes attached to `fn <name>`, parsed bracket-balanced.

    Line-based collection cannot see a multiline attribute: the closing `)]` of a wrapped
    `#[cfg_attr(...)]` does not start with `#[`, so a reverse line scan stops early and the
    attribute body is never inspected. This walks backwards over balanced `#[ ... ]` groups
    instead, so multiline `cfg`, `cfg_attr`, `ignore` and `should_panic` are all visible.
    """
    signature = re.search(rf"^\s*fn\s+{re.escape(name)}\s*\(", code, flags=re.MULTILINE)
    if signature is None:
        return None
    attributes: list[str] = []
    cursor = signature.start()
    while True:
        head = code[:cursor].rstrip()
        if not head.endswith("]"):
            break
        depth = 0
        index = len(head) - 1
        while index >= 0:
            if head[index] == "]":
                depth += 1
            elif head[index] == "[":
                depth -= 1
                if depth == 0:
                    break
            index -= 1
        # `#` and `[` may be separated by whitespace (a comment strips to a space), so
        # `# [ignore]` is the same attribute as `#[ignore]`; anchoring on adjacency would let a
        # spaced attribute suppress a bound test unseen.
        prefix = head[:index].rstrip()
        if index <= 0 or not prefix.endswith("#"):
            break
        opener = len(prefix) - 1
        attributes.append(" ".join(head[opener:].split()))
        cursor = opener
    attributes.reverse()
    return attributes


def _rust_function_body(code: str, name: str) -> str | None:
    """Returns the brace-matched body of one Rust function in already-stripped code.

    Tolerates a generic parameter list, so `fn visit_string<E>(..)` is found as readily as a
    plain `fn name(..)`; anchoring on `name(` would silently return `None` for every generic
    function and turn a body pin into a vacuous check. The list is consumed by BALANCED angle
    brackets rather than by `<[^>]*>`, which cannot cross a nested `<..>` — the Rust mirror
    balances, so a regex that stops at the first `>` would disagree with it on any nested bound.
    """
    signature = re.search(rf"\bfn\s+{re.escape(name)}\b", code)
    if signature is None:
        return None
    cursor = signature.end()
    tail = code[cursor:].lstrip()
    cursor = len(code) - len(tail)
    if tail.startswith("<"):
        _, after = _rust_balanced_group(code, cursor, "<", ">")
        if after is None:
            return None
        cursor = after
        tail = code[cursor:].lstrip()
        cursor = len(code) - len(tail)
    if not tail.startswith("("):
        return None
    start = code.find("{", cursor)
    if start == -1:
        return None
    depth = 0
    for index in range(start, len(code)):
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth -= 1
            if depth == 0:
                return code[start : index + 1]
    return None


def rust_string_literals(code: str) -> list[str]:
    """Returns the payload of every string literal of `code`, in source order.

    Needed because "the delimiter appears in the corpus body" is not a property of the corpus: a
    body containing `String::from` contains `:` whatever its test values say, so the first version
    of that check passed while the corpus had been drifted off the contract. Payloads are
    extracted so the delimiters are looked for where the test VALUES live.
    """
    payloads: list[str] = []
    index = 0
    length = len(code)
    while index < length:
        raw = re.compile(r'(?:b|br|rb)?r(#*)"').match(code, index)
        if raw is not None:
            terminator = '"' + raw.group(1)
            end = code.find(terminator, raw.end())
            if end == -1:
                break
            payloads.append(code[raw.end() : end])
            index = end + len(terminator)
            continue
        quote = re.compile(r'b?"').match(code, index)
        if quote is not None:
            start = quote.end()
            index = start
            while index < length:
                if code[index] == "\\":
                    index += 2
                    continue
                if code[index] == '"':
                    break
                index += 1
            payloads.append(code[start:index])
            index += 1
            continue
        index += 1
    return payloads


def rust_functions(code: str) -> tuple[list[tuple[str, str]], list[str]]:
    """Accounts for EVERY `fn` declaration of already-stripped Rust code, in source order.

    Returns `[(name, normalized body), ...]` plus the name of any declaration whose body could
    not be resolved. A body is its brace-matched block with whitespace collapsed, or `";"` for a
    declaration that has none.

    Pinning function NAMES freezes the module's shape but says nothing about what each function
    does, and `body.contains(...)` says nothing about what else it does — a branch that still
    contains the admitted call can return before ever reaching it. Bodies are therefore
    accounted for exactly, the same total accounting the item, `pub`, `impl`, attribute, derive
    and macro-arm surfaces already carry, one level further down. Nested functions are reported
    in their own right as well as inside their enclosing body, so a helper cannot hide in a
    block.
    """
    functions: list[tuple[str, str]] = []
    unresolved: list[str] = []
    for token in re.finditer(r"\bfn\b", code, flags=re.ASCII):
        # `r#fn` is an ordinary identifier, not the item keyword — the same exclusion the item
        # scan makes for `r#type`.
        if code[: token.start()].endswith("#"):
            continue
        tail = code[token.end() :].lstrip()
        cursor = len(code) - len(tail)
        name = _rust_leading_ident(tail)
        if not name:
            # `fn(u8) -> u8` is a function POINTER TYPE, not a declaration: it carries no name.
            if tail.startswith("("):
                continue
            # Anything else is a declaration this scan cannot classify — rustc accepts non-ASCII
            # identifiers and this lexer is deliberately ASCII-only, mirroring Rust's own byte
            # test. Skipping it would drop the declaration from the inventory silently, so it
            # fails closed instead. The marker is a constant, not a source fragment, because the
            # two carriers slice by character and by byte respectively.
            unresolved.append(RUST_UNNAMED_FUNCTION)
            continue
        cursor += len(name)
        tail = code[cursor:].lstrip()
        cursor = len(code) - len(tail)
        if tail.startswith("<"):
            _, after = _rust_balanced_group(code, cursor, "<", ">")
            if after is None:
                unresolved.append(name)
                continue
            cursor = after
            tail = code[cursor:].lstrip()
            cursor = len(code) - len(tail)
        if not tail.startswith("("):
            unresolved.append(name)
            continue
        # The parameter list is consumed as a BALANCED group rather than scanned past, because
        # a const-generic array length such as `x: [u8; { N }]` puts a brace inside it and
        # anchoring the body on the first `{` after the name would slice from there.
        _, after_parameters = _rust_balanced_group(code, cursor, "(", ")")
        if after_parameters is None:
            unresolved.append(name)
            continue
        # A return type and a `where` clause may carry `(`, `[` and `<`, never `{` or `;`, so
        # the first of those two after the parameter list opens the body or ends the item.
        opener = None
        for index in range(after_parameters, len(code)):
            if code[index] in "{;":
                opener = index
                break
        if opener is None:
            unresolved.append(name)
            continue
        if code[opener] == ";":
            functions.append((name, ";"))
            continue
        group, _ = _rust_balanced_group(code, opener, "{", "}")
        if group is None:
            unresolved.append(name)
            continue
        functions.append((name, " ".join(group.split())))
    return functions, unresolved


def _rust_macro_body(code: str, name: str) -> str | None:
    """Returns the brace-matched body of one `macro_rules! <name>` in already-stripped code."""
    signature = re.search(rf"\bmacro_rules\s*!\s*{re.escape(name)}\b", code)
    if signature is None:
        return None
    start = code.find("{", signature.end())
    if start == -1:
        return None
    depth = 0
    for index in range(start, len(code)):
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth -= 1
            if depth == 0:
                return code[start : index + 1]
    return None


def rust_doc_comment_stream(source: str) -> list[str]:
    """Returns the rustdoc payload lines in order, for carriers that live in `///` prose."""
    stream: list[str] = []
    for line in source.splitlines():
        stripped = line.strip()
        for marker in ("///", "//!"):
            if stripped.startswith(marker):
                stream.append(stripped[len(marker) :].strip())
                break
    return stream


def check_rust_lexical_corpus(issues: list[str]) -> None:
    """Pins the shared cross-language lexer corpus so it cannot be quietly emptied.

    The corpus is the retained fingerprint of BOTH carriers' lexers. The Rust carrier compares
    itself against it from inside the bound suite; this validates the Python carrier against it
    here, in the checker every acceptance binding runs — not only in the separate unit test.
    Without that, mutating the Python stripper (for instance deleting a comment separator instead
    of preserving one) would leave `check_repo_contracts.py` green, because the corpus and the
    checker would drift together while nothing that the AUTH bindings run compared them.
    """
    path = ROOT / RUST_LEXICAL_CORPUS
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        fail(f"rust lexical corpus unreadable: {error}", issues)
        return
    cases = payload.get("cases")
    if not isinstance(cases, list) or len(cases) < MIN_RUST_LEXICAL_CORPUS_CASES:
        fail(
            "rust lexical corpus collapsed: expected at least "
            f"{MIN_RUST_LEXICAL_CORPUS_CASES} cases actual="
            f"{len(cases) if isinstance(cases, list) else 'none'}",
            issues,
        )
        return
    sources = [case.get("source") for case in cases if isinstance(case, dict)]
    if len(set(sources)) != len(sources):
        fail("rust lexical corpus contains a duplicate case", issues)
    for required in REQUIRED_RUST_LEXICAL_CORPUS_CASES:
        if required not in sources:
            fail(f"rust lexical corpus lost a required case: {required!r}", issues)
    for case in cases:
        if not isinstance(case, dict) or "source" not in case:
            fail("rust lexical corpus case is malformed", issues)
            continue
        source = case["source"]
        stripped = strip_rust_comments_and_literals(source)
        items, item_unterminated = rust_item_declarations(stripped)
        impls, impl_unclassified = rust_impl_declarations(stripped)
        invocations, macro_unterminated = rust_macro_invocation_arguments(stripped)
        case_attributes, case_attribute_unterminated = rust_attributes(stripped)
        observed = {
            "stripped": stripped,
            "items": items,
            "item_unterminated": item_unterminated,
            "impls": impls,
            "impl_unclassified": bool(impl_unclassified),
            "macro_definitions": rust_macro_definitions(stripped),
            "macro_invocations": [f"{name}!({argument})" for name, argument in invocations],
            "macro_unterminated": sorted(macro_unterminated),
            "macro_arms": [[name, matchers] for name, matchers in rust_macro_arms(stripped)],
            "derives": rust_derive_bodies(stripped),
            "attributes": [[inner, name, body] for inner, name, body in case_attributes],
            "attribute_unterminated": case_attribute_unterminated,
        }
        for field, value in observed.items():
            if case.get(field) != value:
                fail(
                    "python lexer diverged from the shared corpus on "
                    f"{source!r}: {field} expected {case.get(field)!r} actual {value!r}",
                    issues,
                )


def check_cargo_dependency_sources(issues: list[str]) -> None:
    """Pins WHERE every governed dependency comes from, not merely what it is called.

    A dependency name allowlist is satisfied by `semver = { path = "crates/fake-semver" }`: the
    admitted name resolves to an attacker-authored crate, every Rust scan still reads
    `semver::Version`, and the whole gate stays green. Three carriers close that:

    1. dependency SPECIFICATIONS are compared value for value, so `path`, `git` or an alternate
       registry appears as drift rather than as a silent redirect;
    2. `[patch]`, `[replace]` and `[source]` — which redirect a dependency without editing the
       dependency line at all — are rejected in every manifest, as is a `.cargo/config*`
       source replacement, which redirects an entire registry from outside the manifests;
    3. the RESOLVED identity in the committed `Cargo.lock` is pinned, so a redirect that
       survived the first two is still caught by the source URL disappearing from the lock.
    """
    manifest_path = ROOT / WORKSPACE_MANIFEST
    try:
        workspace = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"workspace manifest unreadable: {error}", issues)
        return

    declared = workspace.get("workspace", {}).get("dependencies", {})
    if declared != WORKSPACE_ADMITTED_DEPENDENCIES:
        fail(
            "workspace dependency specifications drifted: expected "
            f"{WORKSPACE_ADMITTED_DEPENDENCIES} actual={declared}",
            issues,
        )

    # A redirect table needs no change to any dependency line, so screen every manifest.
    for manifest_file in sorted(ROOT.rglob("Cargo.toml")):
        label = manifest_file.relative_to(ROOT).as_posix()
        try:
            parsed = tomllib.loads(manifest_file.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            fail(f"cargo manifest unreadable: {label}: {error}", issues)
            continue
        for table in CARGO_FORBIDDEN_SOURCE_TABLES:
            if table in parsed or table in parsed.get("workspace", {}):
                fail(
                    f"cargo manifest redirects dependency sources with [{table}]: {label}",
                    issues,
                )

    for config_directory in sorted(ROOT.rglob(".cargo")):
        for name in CARGO_CONFIG_FILENAMES:
            config = config_directory / name
            if not config.is_file():
                continue
            label = config.relative_to(ROOT).as_posix()
            try:
                parsed_config = tomllib.loads(config.read_text(encoding="utf-8"))
            except (OSError, tomllib.TOMLDecodeError) as error:
                fail(f"cargo config unreadable: {label}: {error}", issues)
                continue
            if "source" in parsed_config:
                fail(f"cargo config replaces a dependency source: {label}", issues)

    core_manifest_path = ROOT / PLATFORM_CORE_MANIFEST
    try:
        core = tomllib.loads(core_manifest_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"platform-core manifest unreadable: {error}", issues)
        return
    for table, admitted in PLATFORM_CORE_ADMITTED_DEPENDENCY_SPECS.items():
        observed = core.get(table, {})
        if observed != admitted:
            fail(
                f"platform-core [{table}] specifications drifted: expected {admitted} "
                f"actual={observed}",
                issues,
            )

    try:
        lock = tomllib.loads((ROOT / WORKSPACE_LOCKFILE).read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"cargo lockfile unreadable: {error}", issues)
        return
    resolved: dict[str, list[str | None]] = {}
    for package in lock.get("package", []):
        resolved.setdefault(package.get("name", ""), []).append(package.get("source"))
    for name, expected_source in PLATFORM_CORE_RESOLVED_DEPENDENCIES.items():
        sources = resolved.get(name)
        if sources is None:
            fail(f"governed dependency missing from {WORKSPACE_LOCKFILE}: {name}", issues)
            continue
        if sources != [expected_source]:
            fail(
                f"governed dependency resolved to an unexpected source: {name} expected "
                f"{expected_source!r} actual={sources!r}",
                issues,
            )


def check_platform_core_manifest(issues: list[str]) -> None:
    """Pins the Cargo target set of `platform-core` by exact key sets.

    Cargo, not Rust, decides which files are compiled into which target. `[lib] path`,
    `[package] build`, `[[bin]]`, `[[example]]`, `[[bench]]` and `[[test]]` each name a source
    file that no Rust scan reads, and `[[test]]` can also rename or unharness the bound
    acceptance test. Screening for individual keys would leave the next one open, so every
    table and key is accounted for.
    """
    manifest_path = ROOT / PLATFORM_CORE_MANIFEST
    try:
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"platform-core manifest unreadable: {error}", issues)
        return
    tables = tuple(sorted(manifest))
    if tables != tuple(sorted(PLATFORM_CORE_ADMITTED_MANIFEST_TABLES)):
        fail(
            "platform-core manifest tables drifted: expected "
            f"{tuple(sorted(PLATFORM_CORE_ADMITTED_MANIFEST_TABLES))} actual={tables}",
            issues,
        )
    package_keys = tuple(sorted(manifest.get("package", {})))
    if package_keys != tuple(sorted(PLATFORM_CORE_ADMITTED_MANIFEST_PACKAGE_KEYS)):
        fail(
            "platform-core [package] keys drifted: expected "
            f"{tuple(sorted(PLATFORM_CORE_ADMITTED_MANIFEST_PACKAGE_KEYS))} "
            f"actual={package_keys}",
            issues,
        )
    if manifest.get("lib") != PLATFORM_CORE_ADMITTED_MANIFEST_LIB:
        fail(
            "platform-core [lib] target drifted: expected "
            f"{PLATFORM_CORE_ADMITTED_MANIFEST_LIB} actual={manifest.get('lib')}",
            issues,
        )
    for table, admitted in (
        ("dependencies", PLATFORM_CORE_ADMITTED_MANIFEST_DEPENDENCIES),
        ("dev-dependencies", PLATFORM_CORE_ADMITTED_MANIFEST_DEV_DEPENDENCIES),
    ):
        declared = tuple(sorted(manifest.get(table, {})))
        if declared != tuple(sorted(admitted)):
            fail(
                f"platform-core [{table}] drifted: expected {tuple(sorted(admitted))} "
                f"actual={declared}",
                issues,
            )


def rust_character_class_bytes(spec: str) -> tuple[tuple[str, ...], bool]:
    """Expands a regex character-class body into its members, reporting a duplicate.

    A leading `-` is a literal, `A-Z` is a range, and the result is the ordered member list so a
    class that spells one byte twice is distinguishable from one that spells it once. This is a
    parse of the class, not a search inside it: a class that gained a byte is a different member
    list rather than a string that still contains the old one.
    """
    members: list[str] = []
    index = 0
    while index < len(spec):
        character = spec[index]
        is_range = (
            index + 2 < len(spec) and spec[index + 1] == "-" and index + 2 != len(spec)
        )
        if is_range:
            members.extend(
                chr(code) for code in range(ord(character), ord(spec[index + 2]) + 1)
            )
            index += 3
            continue
        members.append(character)
        index += 1
    return tuple(members), len(members) != len(set(members))


ASCII_ALPHANUMERIC_BYTES = frozenset(
    character for character in map(chr, range(128)) if character.isalnum()
)


def rust_token_stream(code: str) -> list[str]:
    """Returns `code` as tokens: identifier/number runs, two-byte comparison operators kept whole,
    every other non-space character on its own.

    Mirrors the bound suite's own tokenizer, so both carriers judge the same sequence rather than
    two dialects that could disagree about what a statement is.
    """
    return RUST_TOKEN.findall(code)


def rust_token_depths(tokens: list[str]) -> list[int]:
    """Returns the delimiter depth *before* each token of `tokens`.

    Depth is what separates a statement OF a function from a statement of some block nested inside
    it, and so a guard that decides from one that another condition decides for.
    """
    depths: list[int] = []
    depth = 0
    for token in tokens:
        if token in "})]":
            depth -= 1
        depths.append(depth)
        if token in "{([":
            depth += 1
    return depths


def rust_without_trailing_commas(tokens: list[str]) -> list[str]:
    """Drops a `,` that only separates a last element from its closing delimiter.

    A trailing comma is rustfmt's business, not the language's: normalizing it keeps the admitted
    shapes below statements about STRUCTURE rather than about a formatting policy that may change.
    """
    kept: list[str] = []
    for index, token in enumerate(tokens):
        if token == "," and index + 1 < len(tokens) and tokens[index + 1] in ("}", ")", "]"):
            continue
        kept.append(token)
    return kept


def rust_without_raw_identifiers(tokens: list[str]) -> list[str]:
    """Rewrites every raw identifier `r # <name>` to the plain `<name>` it denotes.

    `r#foo` and `foo` are ONE name to Rust and two strings to every rule that compares tokens, so a
    raw spelling defines a shadow no exact-token scan for the plain one can see. Both lexers split
    it into three tokens; folding them here is what lets a single rule answer for both spellings.
    Raw STRING literals are consumed whole by the literal stripper, so an `r` immediately followed
    by `#` in stripped code is a raw identifier and nothing else.
    """
    kept: list[str] = []
    index = 0
    while index < len(tokens):
        following = tokens[index + 2 : index + 3]
        if (
            tokens[index] == "r"
            and tokens[index + 1 : index + 2] == ["#"]
            and following
            and _RUST_IDENTIFIER.match(following[0]) is not None
        ):
            kept.append(following[0])
            index += 3
            continue
        kept.append(tokens[index])
        index += 1
    return kept


def _rust_declaration_end(code: str, start: int) -> int:
    """Returns the index of the `;` terminating a declaration that begins at `start`.

    Tracked by delimiter depth, because `[&str; 2]` spells a `;` inside the type that does not end
    anything.
    """
    cursor = start
    depth = 0
    while cursor < len(code):
        if code[cursor] in "{([":
            depth += 1
        elif code[cursor] in "})]":
            depth -= 1
        elif code[cursor] == ";" and depth == 0:
            break
        cursor += 1
    return cursor


def rust_declaration_spans(code: str, name: str) -> list[str]:
    """Returns the source span of every `const <name> … ;` of `code`, wherever it is declared.

    Unlike `rust_module_level_declarations` this does not filter by depth, so it is safe to run
    over text whose literal PAYLOADS are still present — the stripper is what makes depth tracking
    meaningful, and a literal brace would otherwise shift every depth after it. The caller pairs it
    with the module-level count, so a declaration spelled inside a string cannot pass unnoticed.
    """
    declaration = re.compile(rf"\bconst\s+{re.escape(name)}\b")
    return [
        code[match.start() : _rust_declaration_end(code, match.end()) + 1]
        for match in declaration.finditer(code)
    ]


def rust_module_level_declarations(code: str, name: str) -> list[str]:
    """Returns the source span of every `const <name> … ;` declared at brace depth zero.

    Depth matters for the same reason it does in `rust_module_level_usize_constants`: a nested
    declaration is a different binding from the module's own. `code` must have its literals
    stripped, or a brace inside one is counted as a scope.
    """
    declaration = re.compile(rf"\bconst\s+{re.escape(name)}\b")
    spans: list[str] = []
    depth = 0
    index = 0
    while index < len(code):
        character = code[index]
        if character in "{([":
            depth += 1
        elif character in "})]":
            depth -= 1
        elif depth == 0:
            match = declaration.match(code, index)
            if match is not None:
                end = _rust_declaration_end(code, match.end())
                spans.append(code[match.start() : end + 1])
                index = end + 1
                continue
        index += 1
    return spans


def rust_statement_positions(
    tokens: list[str], depths: list[int], statement: list[str], depth: int
) -> list[int]:
    """Returns each index where `statement` occurs in `tokens` starting at exactly `depth`."""
    width = len(statement)
    return [
        index
        for index in range(len(tokens) - width + 1)
        if depths[index] == depth and tokens[index : index + width] == statement
    ]


def rust_statement_starts(tokens: list[str], depths: list[int], depth: int) -> set[int]:
    """Returns each index of `tokens` at `depth` that BEGINS a statement.

    A statement begins right after `;`, `{` or `}`, or at the start of the block. The distinction
    matters because a bare subsequence search cannot tell `helper();` from `let _ = helper();`, and
    the second one discards whatever the first one proves.
    """
    starts: set[int] = set()
    previous = "{"
    for index, token in enumerate(tokens):
        if depths[index] == depth and previous in (";", "{", "}"):
            starts.add(index)
        previous = token
    return starts


def rust_declared_signature(code: str, name: str) -> list[str] | None:
    """Returns the token sequence of `fn <name>`'s header, up to and including its opening brace."""
    match = re.search(rf"\bfn\s+{re.escape(name)}\b", code)
    if match is None:
        return None
    opening = code.find("{", match.end())
    if opening == -1:
        return None
    return rust_token_stream(code[match.start() : opening + 1])


def rust_module_level_usize_constants(code: str, name: str) -> tuple[int, ...]:
    """Returns the value of every `const <name>: usize = <digits>;` declared at brace depth zero.

    Depth matters: a declaration nested in a function body, a `mod` or an `impl` is a different
    binding from the module's own, and only the module's own is the one every use resolves to.
    A value that is not plain digits does not match at all, so `= 128 + 1` fails closed rather
    than reading as 128.
    """
    declaration = re.compile(rf"\bconst\s+{re.escape(name)}\s*:\s*usize\s*=\s*(\d[0-9_]*)\s*;")
    values: list[int] = []
    depth = 0
    index = 0
    while index < len(code):
        character = code[index]
        if character in "{([":
            depth += 1
        elif character in "})]":
            depth -= 1
        elif depth == 0:
            match = declaration.match(code, index)
            if match is not None:
                values.append(int(match.group(1).replace("_", "")))
                index = match.end()
                continue
        index += 1
    return tuple(values)


def _platform_identity_effective_bound(issues: list[str]) -> None:
    """Proves the EFFECTIVE length bound resolves to the contract-bound constant.

    The declared carrier and the frozen body fingerprint are both mutable, and a body may legally
    introduce a second semantic constant. So the accounting below eliminates every place a second
    bound could come from instead of comparing the body against a snapshot of itself.
    """
    name = PLATFORM_IDENTITY_LENGTH_CONSTANT
    function = PLATFORM_IDENTITY_BOUND_FUNCTION
    subject = PLATFORM_IDENTITY_BOUND_SUBJECT
    field = PLATFORM_IDENTITY_BOUND_FIELD
    expected = PLATFORM_IDENTITY_GRAMMAR["max_bytes"]
    try:
        source = (ROOT / PLATFORM_IDENTITY_SOURCE).read_text(encoding="utf-8")
        test_source = (ROOT / PLATFORM_IDENTITY_TEST).read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity effective max-byte carrier unreadable: {error}", issues)
        return
    # Literal PAYLOADS are stripped here, unlike the delimiter accounting: this rule is about token
    # structure, and a brace or a digit inside a string literal is neither a scope nor a bound.
    code = strip_rust_comments_and_literals(source)
    test_code = strip_rust_comments_and_literals(test_source)
    flat = " ".join(code.split())

    # 1. The name is bound exactly once in the module, at module level, to the contract's number.
    bindings = re.findall(rf"\b(?:const|static|let)\s+(?:mut\s+)?{re.escape(name)}\b", flat)
    if len(bindings) != 1:
        fail(
            f"platform identity effective max-byte bound: {name} is bound {len(bindings)} times "
            "in the module, expected exactly one module-level const",
            issues,
        )
    declared = rust_module_level_usize_constants(code, name)
    if declared != (expected,):
        fail(
            f"platform identity effective max-byte bound: module-level {name} declares "
            f"{list(declared)}, expected [{expected}] from the accepted contract",
            issues,
        )

    # 2. The deciding function must be an admitted one, so a decoy cannot answer for it.
    if function not in PLATFORM_IDENTITY_ADMITTED_FUNCTIONS:
        fail(
            "platform identity effective max-byte bound: deciding carrier is not an admitted "
            f"function: {function}",
            issues,
        )
        return
    body = _rust_function_body(code, function)
    if body is None:
        fail(
            f"platform identity effective max-byte bound: {function} body unreadable",
            issues,
        )
        return
    body_flat = " ".join(body.split())

    # 3. Exactly two occurrences inside the deciding function, and only the declaration outside it.
    inside = len(re.findall(rf"\b{re.escape(name)}\b", body_flat))
    outside = len(re.findall(rf"\b{re.escape(name)}\b", flat)) - inside
    if inside != 2:
        fail(
            f"platform identity effective max-byte bound: {name} occurs {inside} times in "
            f"{function}, expected exactly 2 (the comparison and the reported bound)",
            issues,
        )
    if outside != 1:
        fail(
            f"platform identity effective max-byte bound: {name} occurs {outside} times outside "
            f"{function}, expected only its declaration",
            issues,
        )

    # 4. The effective comparison: one length measurement, of the admitted subject, against the
    #    contract-bound name itself rather than against anything derived from it.
    admitted_comparison = (subject, PLATFORM_IDENTITY_BOUND_OPERATOR, name)
    comparisons = [
        (match.group("receiver"), match.group("operator"), match.group("operand"))
        for match in RUST_LENGTH_COMPARISON.finditer(body_flat)
    ]
    measurements = len(RUST_LEN_CALL.findall(body_flat))
    if measurements != 1 or comparisons != [admitted_comparison]:
        fail(
            f"platform identity effective max-byte bound: {function} measures length "
            f"{measurements} times and compares {comparisons}, expected exactly one "
            f"{list(admitted_comparison)}",
            issues,
        )

    # 5. The reported bound is the same name, not a second value that merely compares equal today.
    reported = re.findall(rf"\b{field}\s*:\s*([A-Za-z_][A-Za-z0-9_]*|\d[0-9_]*)", body_flat)
    if reported != [name]:
        fail(
            f"platform identity effective max-byte bound: {function} reports {field} as "
            f"{reported}, expected [{name!r}]",
            issues,
        )

    # 6. No item may be declared inside the deciding function: a local `const`, `static`, `fn`,
    #    `use` alias or `macro_rules!` is exactly how a second bound gets in while every mention
    #    of the module constant survives as a decoy.
    declarations = sorted(
        keyword
        for keyword in PLATFORM_IDENTITY_BOUND_FORBIDDEN_ITEM_KEYWORDS
        if re.search(rf"\b{keyword}\b", body_flat)
    )
    if declarations:
        fail(
            f"platform identity effective max-byte bound: {function} declares an item: "
            f"{declarations}",
            issues,
        )

    # 7. No integer other than the byte-index offset, so a bare literal bound cannot appear.
    literals = tuple(RUST_INTEGER_LITERAL.findall(body_flat))
    if literals != PLATFORM_IDENTITY_BOUND_ADMITTED_LITERALS:
        fail(
            f"platform identity effective max-byte bound: {function} spells integer literals "
            f"{list(literals)}, expected {list(PLATFORM_IDENTITY_BOUND_ADMITTED_LITERALS)}",
            issues,
        )

    # 8. No binding may shadow the name, and the measured subject is bound exactly once, to the
    #    whole candidate rather than to a slice of it.
    patterns = [
        " ".join(match.group("pattern").split())
        for match in RUST_BINDING_PATTERN.finditer(body_flat)
    ]
    shadowing = [pattern for pattern in patterns if re.search(rf"\b{re.escape(name)}\b", pattern)]
    if shadowing:
        fail(
            f"platform identity effective max-byte bound: {function} shadows {name}: {shadowing}",
            issues,
        )
    subject_bindings = [
        pattern for pattern in patterns if re.search(rf"\b{re.escape(subject)}\b", pattern)
    ]
    if subject_bindings != [subject]:
        fail(
            f"platform identity effective max-byte bound: {function} binds {subject} as "
            f"{subject_bindings}, expected exactly [{subject!r}]",
            issues,
        )
    if body_flat.count(PLATFORM_IDENTITY_BOUND_SUBJECT_BINDING) != 1:
        fail(
            f"platform identity effective max-byte bound: {function} must measure exactly one "
            f"{PLATFORM_IDENTITY_BOUND_SUBJECT_BINDING!r}",
            issues,
        )

    # 9. Module-wide totals, so a helper cannot hold the comparison or construct the report.
    module_comparisons = [
        (match.group("receiver"), match.group("operator"), match.group("operand"))
        for match in RUST_LENGTH_COMPARISON.finditer(flat)
    ]
    if module_comparisons != [admitted_comparison]:
        fail(
            "platform identity effective max-byte bound: module length comparisons are "
            f"{module_comparisons}, expected exactly one {list(admitted_comparison)}",
            issues,
        )
    module_fields = tuple(
        re.findall(rf"\b{field}\s*:\s*([A-Za-z_][A-Za-z0-9_]*|\d[0-9_]*)", flat)
    )
    if module_fields != PLATFORM_IDENTITY_BOUND_FIELD_VALUES:
        fail(
            f"platform identity effective max-byte bound: module {field} fields are "
            f"{list(module_fields)}, expected {list(PLATFORM_IDENTITY_BOUND_FIELD_VALUES)}",
            issues,
        )

    # 10. The bound suite's own length constants, pinned to the contract rather than to the
    #     implementation they generate fixtures for.
    for constant in PLATFORM_IDENTITY_TEST_LENGTH_CONSTANTS:
        values = rust_module_level_usize_constants(test_code, constant)
        if values != (expected,):
            fail(
                f"platform identity effective max-byte bound: bound suite {constant} declares "
                f"{list(values)}, expected [{expected}] from the accepted contract",
                issues,
            )


def _platform_identity_admitted_bound_statement() -> list[str]:
    """The one admitted max-byte rejection statement, spelled from the CONTRACT's own names.

    Assembled here rather than read out of `classify`, so it states what the contract requires
    instead of agreeing with whatever the implementation currently happens to be.
    """
    name = PLATFORM_IDENTITY_LENGTH_CONSTANT
    return [
        # if bytes.len() > MAX_IDENTITY_BYTES {
        "if",
        PLATFORM_IDENTITY_BOUND_SUBJECT,
        ".",
        "len",
        "(",
        ")",
        PLATFORM_IDENTITY_BOUND_OPERATOR,
        name,
        "{",
        # return Err(IdentityValueErrorKind::TooLong { max_bytes: MAX_IDENTITY_BYTES });
        "return",
        "Err",
        "(",
        PLATFORM_IDENTITY_BOUND_ERROR_TYPE,
        ":",
        ":",
        PLATFORM_IDENTITY_BOUND_ERROR_VARIANT,
        "{",
        PLATFORM_IDENTITY_BOUND_FIELD,
        ":",
        name,
        "}",
        ")",
        ";",
        "}",
    ]


def _platform_identity_admitted_runtime_proof() -> list[str]:
    """The whole admitted body of the runtime boundary proof, in tokens.

    String literals are stripped before comparison, so the panic and assertion MESSAGES are not
    part of this: prose is not the proof. What is pinned is the shape — the last admitted length
    parses and is retained, the next one is refused, and the refusal reports the contract's number.
    """
    kind = PLATFORM_IDENTITY_RUNTIME_PROOF_KIND
    bound = PLATFORM_IDENTITY_RUNTIME_PROOF_CONSTANT
    field = PLATFORM_IDENTITY_BOUND_FIELD
    return (
        ["{"]
        # let admitted = "a".repeat(GRAMMAR_MAX_BYTES);
        + ["let", "admitted", "=", ".", "repeat", "(", bound, ")", ";"]
        # let refused = "a".repeat(GRAMMAR_MAX_BYTES + 1);
        + ["let", "refused", "=", ".", "repeat", "(", bound, "+", "1", ")", ";"]
        # let Ok(parsed) = TenantId::parse(admitted.clone()) else { panic!(…); };
        + ["let", "Ok", "(", "parsed", ")", "=", kind, ":", ":", "parse"]
        + ["(", "admitted", ".", "clone", "(", ")", ")"]
        + ["else", "{", "panic", "!", "(", ")", ";", "}", ";"]
        # assert_eq!(parsed.as_str(), admitted, …);
        + ["assert_eq", "!", "(", "parsed", ".", "as_str", "(", ")", ",", "admitted", ")", ";"]
        # let Err(error) = TenantId::parse(refused) else { panic!(…); };
        + ["let", "Err", "(", "error", ")", "=", kind, ":", ":", "parse", "(", "refused", ")"]
        + ["else", "{", "panic", "!", "(", ")", ";", "}", ";"]
        # assert_eq!(error.kind(), IdentityValueErrorKind::TooLong { max_bytes: … }, …);
        + ["assert_eq", "!", "(", "error", ".", "kind", "(", ")", ","]
        + [PLATFORM_IDENTITY_BOUND_ERROR_TYPE, ":", ":", PLATFORM_IDENTITY_BOUND_ERROR_VARIANT]
        + ["{", field, ":", bound, "}", ")", ";"]
        + ["}"]
    )


def _platform_identity_admitted_classify_body() -> list[str]:
    """The whole admitted body of the deciding function, as the contract's decision procedure.

    §5 fixes the error precedence exactly, and §3 fixes what each step tests, so the deciding
    function has one admitted shape and this assembles it from those names. Binding the guard alone
    was not enough: an early accept keyed to a literal — `if value == "aaa…129" { return Ok(()); }`
    — adds a step *before* the guard while leaving the guard, the constant, every count and every
    elimination rule intact, and literal payloads are stripped before comparison, so both frozen
    fingerprints could be synchronized to `if value == { return Ok(()); }` and stay green.

    A step that the contract does not name is therefore refused outright, rather than being left to
    a fingerprint that one commit can move.
    """
    name = PLATFORM_IDENTITY_LENGTH_CONSTANT
    error = PLATFORM_IDENTITY_BOUND_ERROR_TYPE
    subject = PLATFORM_IDENTITY_BOUND_SUBJECT
    return (
        ["{"]
        # let bytes = value.as_bytes();
        + ["let", subject, "=", "value", ".", "as_bytes", "(", ")", ";"]
        # §5.1 empty — let Some((&first, after_first)) = bytes.split_first() else { … };
        + ["let", "Some", "(", "(", "&", "first", ",", "after_first", ")", ")", "="]
        + [subject, ".", "split_first", "(", ")", "else"]
        + ["{", "return", "Err", "(", error, ":", ":", "Empty", ")", ";", "}", ";"]
        # §5.2 too long — the guard bound in full below
        + _platform_identity_admitted_bound_statement()
        # §5.3 invalid start
        + ["if", "!", "is_boundary_byte", "(", "first", ")"]
        + ["{", "return", "Err", "(", error, ":", ":", "InvalidStart", ")", ";", "}"]
        # a one-byte value is decided by the first-byte rule alone
        + ["let", "Some", "(", "(", "&", "last", ",", "interior", ")", ")", "="]
        + ["after_first", ".", "split_last", "(", ")", "else"]
        + ["{", "return", "Ok", "(", "(", ")", ")", ";", "}", ";"]
        # §5.4 invalid interior byte, reported by index
        + ["for", "(", "offset", ",", "&", "byte", ")", "in"]
        + ["interior", ".", "iter", "(", ")", ".", "enumerate", "(", ")", "{"]
        + ["if", "!", "is_interior_byte", "(", "byte", ")", "{"]
        + ["return", "Err", "(", error, ":", ":", "InvalidCharacter", "{"]
        + ["byte_index", ":", "offset", "+", "1", "}", ")", ";", "}", "}"]
        # §5.5 invalid end
        + ["if", "!", "is_boundary_byte", "(", "last", ")"]
        + ["{", "return", "Err", "(", error, ":", ":", "InvalidEnd", ")", ";", "}"]
        # otherwise canonical
        + ["Ok", "(", "(", ")", ")"]
        + ["}"]
    )
    # `name` participates through the guard statement above.


def _platform_identity_classify_procedure(issues: list[str]) -> None:
    """Binds the deciding function to the contract's decision procedure, step for step."""
    function = PLATFORM_IDENTITY_BOUND_FUNCTION
    try:
        source = (ROOT / PLATFORM_IDENTITY_SOURCE).read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity decision procedure carrier unreadable: {error}", issues)
        return
    code = strip_rust_comments_and_literals(source)
    body = _rust_function_body(code, function)
    if body is None:
        fail(f"platform identity decision procedure: {function} body unreadable", issues)
        return
    tokens = rust_without_trailing_commas(rust_token_stream(body))
    expected = _platform_identity_admitted_classify_body()
    if tokens != expected:
        fail(
            f"platform identity decision procedure: {function} is {tokens}, expected {expected} — "
            "the deciding function admits exactly the steps §5 names, in that order, so a step the "
            "contract does not name cannot be added ahead of the bound",
            issues,
        )

    # Literal payloads are stripped before every structural rule, so a value-keyed comparison is
    # invisible to them. The deciding function tests length and per-byte class; it has no business
    # holding a literal at all, and saying so closes the blind spot at its source.
    literals = rust_string_literals(_rust_function_body(source, function) or "")
    if literals:
        fail(
            f"platform identity decision procedure: {function} spells string literals {literals}, "
            "expected none — a literal in the deciding function is a value-keyed branch",
            issues,
        )


def _platform_identity_admitted_runtime_sweep() -> list[str]:
    """The whole admitted body of the length sweep, in tokens.

    Structural rules can only refuse a value-keyed branch they can see. This one is behavioural:
    every length to twice the bound, under two canonical seeds, accepted iff it is within the
    contract's bound and refused reporting that same number otherwise.
    """
    seeds = PLATFORM_IDENTITY_RUNTIME_SWEEP_SEEDS
    span = PLATFORM_IDENTITY_RUNTIME_SWEEP_SPAN
    bound = PLATFORM_IDENTITY_RUNTIME_PROOF_CONSTANT
    kind = PLATFORM_IDENTITY_RUNTIME_PROOF_KIND
    return (
        ["{"]
        # let mut admitted = 0; let mut refused = 0;
        + ["let", "mut", "admitted", "=", "0", ";"]
        + ["let", "mut", "refused", "=", "0", ";"]
        # for seed in RUNTIME_PROOF_SEEDS { for length in 1..=RUNTIME_PROOF_SWEEP {
        + ["for", "seed", "in", seeds, "{"]
        + ["for", "length", "in", "1", ".", ".", "=", span, "{"]
        # let candidate = seed.repeat(length);
        + ["let", "candidate", "=", "seed", ".", "repeat", "(", "length", ")", ";"]
        # let parsed = TenantId::parse(candidate.clone());
        + ["let", "parsed", "=", kind, ":", ":", "parse"]
        + ["(", "candidate", ".", "clone", "(", ")", ")", ";"]
        # within the bound it must parse…
        + ["if", "length", "<=", bound, "{"]
        + ["assert", "!", "(", "parsed", ".", "is_ok", "(", ")", ")", ";"]
        + ["admitted", "+", "=", "1", ";", "}"]
        # …and past it, it must be refused reporting the contract's number
        + ["else", "{"]
        + ["let", "Err", "(", "error", ")", "=", "parsed", "else"]
        + ["{", "panic", "!", "(", ")", ";", "}", ";"]
        + ["assert_eq", "!", "(", "error", ".", "kind", "(", ")", ","]
        + [PLATFORM_IDENTITY_BOUND_ERROR_TYPE, ":", ":", PLATFORM_IDENTITY_BOUND_ERROR_VARIANT]
        + ["{", PLATFORM_IDENTITY_BOUND_FIELD, ":", bound, "}", ")", ";"]
        + ["refused", "+", "=", "1", ";", "}"]
        + ["}", "}"]
        # …and the sweep's own extent, counted rather than claimed: emptying the seeds or halving
        # the span leaves every token above in place and makes both of these wrong.
        + ["assert_eq", "!", "(", "admitted", ",", "2", "*", bound, ")", ";"]
        + ["assert_eq", "!", "(", "refused", ",", "2", "*", bound, ")", ";"]
        + ["}"]
    )


def _platform_identity_effective_guard(issues: list[str]) -> None:
    """Proves the max-byte comparison DECIDES, and that the runtime proof of it still runs.

    Round 16 proved the comparison `bytes.len() > MAX_IDENTITY_BYTES` occurs inside `classify`. It
    never proved the comparison is what the rejection branch turns on, so a wrapper that keeps every
    declared carrier alive while making the branch unreachable —
    `if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES { … }` — passed this checker,
    all 271 suite tests, fmt, clippy and every cargo gate with both body fingerprints co-mutated,
    while an external crate parsed a 200-byte identity through the public API.

    Two things were missing, and both are bound below. An occurring comparison is not a controlling
    condition, so the guard and its branch are matched as ONE structural unit at the function's own
    statement depth. And a call site is not a proof body: the same mutation deleted the runtime
    128/129 tail while leaving its call in place, so the proof's whole body is bound here too.
    """
    try:
        source = (ROOT / PLATFORM_IDENTITY_SOURCE).read_text(encoding="utf-8")
        test_source = (ROOT / PLATFORM_IDENTITY_TEST).read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity deciding-guard carrier unreadable: {error}", issues)
        return
    code = strip_rust_comments_and_literals(source)
    test_code = strip_rust_comments_and_literals(test_source)
    function = PLATFORM_IDENTITY_BOUND_FUNCTION

    # 1. The guard and its rejection branch are one admitted statement, at `classify`'s own
    #    statement depth. A prefix, suffix or wrapper predicate belongs to the max-byte closure —
    #    it is not the unrelated-branch residual the contract leaves to review — and a copy nested
    #    inside `if false { … }` sits at a deeper depth and cannot answer for the real one.
    body = _rust_function_body(code, function)
    if body is None:
        fail(f"platform identity deciding guard: {function} body unreadable", issues)
        return
    body_tokens = rust_without_trailing_commas(rust_token_stream(body))
    body_depths = rust_token_depths(body_tokens)
    admitted = _platform_identity_admitted_bound_statement()
    positions = rust_statement_positions(body_tokens, body_depths, admitted, 1)
    if len(positions) != 1:
        fail(
            f"platform identity deciding guard: {function} must contain exactly one top-level "
            f"{admitted}, found {len(positions)} — the comparison must be the ENTIRE controlling "
            "condition of the rejection branch, not merely a token that occurs in it",
            issues,
        )
    elif body_tokens[positions[0] + len(admitted) : positions[0] + len(admitted) + 1] == ["else"]:
        # An alternate branch is a second outcome for the same decision.
        fail(
            f"platform identity deciding guard: {function} guard has an alternate branch",
            issues,
        )

    # 2. The variant is constructed nowhere else, so a second rejection path cannot report a second
    #    bound while the admitted one is disabled.
    variants = rust_token_stream(code).count(PLATFORM_IDENTITY_BOUND_ERROR_VARIANT)
    if variants != PLATFORM_IDENTITY_BOUND_ERROR_VARIANT_SITES:
        fail(
            f"platform identity deciding guard: {PLATFORM_IDENTITY_BOUND_ERROR_VARIANT} is spelled "
            f"{variants} times in the module, expected "
            f"{PLATFORM_IDENTITY_BOUND_ERROR_VARIANT_SITES} (the variant, its rendering and the one "
            "rejection branch)",
            issues,
        )

    # 3. The runtime proof's whole body, not merely its name: deleting the load-bearing tail while
    #    keeping the call is exactly what stayed green through Round 16.
    proof = PLATFORM_IDENTITY_RUNTIME_PROOF_FUNCTION
    proof_body = _rust_function_body(test_code, proof)
    if proof_body is None:
        fail(f"platform identity runtime bound: {proof} body unreadable", issues)
        return
    proof_tokens = rust_without_trailing_commas(rust_token_stream(proof_body))
    expected_proof = _platform_identity_admitted_runtime_proof()
    if proof_tokens != expected_proof:
        fail(
            f"platform identity runtime bound: {proof} body is {proof_tokens}, expected "
            f"{expected_proof} — the boundary must be driven through the public API at "
            f"{PLATFORM_IDENTITY_RUNTIME_PROOF_CONSTANT} and one byte past it",
            issues,
        )

    # 4. The length sweep's body, for the same reason and against a wider class: an accept keyed to
    #    an over-bound length other than the first one passes the 128/129 pair untouched.
    sweep = PLATFORM_IDENTITY_RUNTIME_SWEEP_FUNCTION
    sweep_body = _rust_function_body(test_code, sweep)
    if sweep_body is None:
        fail(f"platform identity runtime bound: {sweep} body unreadable", issues)
        return
    sweep_tokens = rust_without_trailing_commas(rust_token_stream(sweep_body))
    expected_sweep = _platform_identity_admitted_runtime_sweep()
    if sweep_tokens != expected_sweep:
        fail(
            f"platform identity runtime bound: {sweep} body is {sweep_tokens}, expected "
            f"{expected_sweep} — every length to twice the bound must be admitted exactly when the "
            "contract admits it",
            issues,
        )

    # 5. And both are actually reached: called once each, as a statement of the caller rather than
    #    under a condition, in a caller that cannot leave early before them.
    caller = PLATFORM_IDENTITY_RUNTIME_PROOF_CALLER
    caller_body = _rust_function_body(test_code, caller)
    if caller_body is None:
        fail(f"platform identity runtime bound: {caller} body unreadable", issues)
        return
    caller_tokens = rust_without_raw_identifiers(
        rust_without_trailing_commas(rust_token_stream(caller_body))
    )
    caller_depths = rust_token_depths(caller_tokens)
    starts = rust_statement_starts(caller_tokens, caller_depths, 1)
    for reached in PLATFORM_IDENTITY_CALLER_EVIDENCE_CALLS:
        calls = [
            index
            for index in rust_statement_positions(
                caller_tokens, caller_depths, [reached, "(", ")", ";"], 1
            )
            if index in starts
        ]
        if len(calls) != 1:
            fail(
                f"platform identity runtime bound: {caller} must call {reached} exactly once as a "
                f"top-level statement, found {len(calls)} — `let _ = {reached}();` discards what "
                "the proof establishes and is not a call for this purpose",
                issues,
            )
    skips = sorted(set(PLATFORM_IDENTITY_FORBIDDEN_CONTROL).intersection(caller_tokens))
    if skips:
        fail(
            f"platform identity runtime bound: {caller} may not {skips} past the proof",
            issues,
        )

    # 6. Load-bearing helpers return nothing. A helper that returns `Result` can be ignored at its
    #    call site, and `black_box(Err::<(),()>(()))?` then leaves before the proof runs while
    #    spelling neither `return` nor `continue`.
    for helper in PLATFORM_IDENTITY_PROOF_HELPERS:
        signature = rust_declared_signature(test_code, helper)
        expected_signature = ["fn", helper, "(", ")", "{"]
        if signature != expected_signature:
            fail(
                f"platform identity runtime bound: {helper} is declared {signature}, expected "
                f"{expected_signature} — a proof helper takes no argument and returns nothing, so "
                "no caller can discard its outcome",
                issues,
            )

    # 7. The generic corpus macro may not skip a row. Every carrier pinned inside it is a SUBSTRING,
    #    and a substring is not a case that still reaches it: a `continue` guarded on the expected
    #    error kind keeps all of them while the over-length rows stop executing, and so does a
    #    `break`, or a `?` inside an ignored closure wrapped around the loop.
    macro = PLATFORM_IDENTITY_CORPUS_MACRO
    macro_body = _rust_macro_body(test_code, macro)
    if macro_body is None:
        fail(f"platform identity runtime bound: {macro} body unreadable", issues)
        return
    macro_tokens = rust_without_raw_identifiers(rust_token_stream(macro_body))
    transfers = sorted(set(PLATFORM_IDENTITY_FORBIDDEN_CONTROL).intersection(macro_tokens))
    if transfers:
        fail(
            f"platform identity runtime bound: {macro} may not {transfers} past a row",
            issues,
        )

    # 8. …and each corpus loop is a statement of the macro ARM, not of something nested inside it.
    #    An ignored closure around the loop changes no keyword and leaves every pinned substring in
    #    place; it changes the loop's depth, which is the thing that decides whether rows run.
    macro_depths = rust_token_depths(macro_tokens)
    anchors = [
        depth
        for token, depth in zip(macro_tokens, macro_depths)
        if token == PLATFORM_IDENTITY_CORPUS_ARM_ANCHOR
    ]
    if not anchors:
        fail(
            f"platform identity runtime bound: {macro} has no {PLATFORM_IDENTITY_CORPUS_ARM_ANCHOR} "
            "binding to locate the arm's own statement depth",
            issues,
        )
        return
    arm_depth = min(anchors)
    for corpus in PLATFORM_IDENTITY_CORPUS_LOOPS:
        depths_of_loop = [
            macro_depths[index]
            for index, token in enumerate(macro_tokens)
            if token == corpus and macro_tokens[index - 1 : index] == ["in"]
        ]
        if depths_of_loop != [arm_depth]:
            fail(
                f"platform identity runtime bound: {macro} iterates {corpus} at depths "
                f"{depths_of_loop}, expected exactly one at the arm's own statement depth "
                f"{arm_depth} — a loop nested in a closure is not a loop the arm runs",
                issues,
            )

    # 9. AUTH-011's own evidence is reached. The registered carriers are substrings, and every one
    #    of them survives `if std::hint::black_box(false) { … }` wrapped around the whole block
    #    while the test still reports `1 passed`.
    auth011 = PLATFORM_IDENTITY_AUTH011_FUNCTION
    auth011_body = _rust_function_body(test_code, auth011)
    if auth011_body is None:
        fail(f"platform identity runtime bound: {auth011} body unreadable", issues)
        return
    auth011_tokens = rust_without_raw_identifiers(
        rust_without_trailing_commas(rust_token_stream(auth011_body))
    )
    auth011_depths = rust_token_depths(auth011_tokens)
    auth011_starts = rust_statement_starts(auth011_tokens, auth011_depths, 1)
    for evidence in PLATFORM_IDENTITY_AUTH011_EVIDENCE_CALLS:
        reached = [
            index
            for index in rust_statement_positions(
                auth011_tokens, auth011_depths, [evidence, "(", ")", ";"], 1
            )
            if index in auth011_starts
        ]
        if len(reached) != 1:
            fail(
                f"platform identity runtime bound: {auth011} must call {evidence} exactly once as a "
                f"plain statement of its own body, found {len(reached)}",
                issues,
            )
    auth011_transfers = sorted(
        set(PLATFORM_IDENTITY_FORBIDDEN_CONTROL).intersection(auth011_tokens)
    )
    if auth011_transfers:
        fail(
            f"platform identity runtime bound: {auth011} may not {auth011_transfers} past its "
            "evidence",
            issues,
        )


def _platform_identity_helper_resolution(issues: list[str]) -> None:
    """Proves each load-bearing call reaches the file-level helper whose name it spells.

    A plain-statement call is a fact about tokens. Which function it runs is a fact about NAME
    RESOLUTION, and Rust resolves lexically, so an item declared in the caller's own body binds the
    name ahead of the module's:

        let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();
        fn r#assert_no_length_past_the_bound_is_accepted() {}
        assert_no_length_past_the_bound_is_accepted();

    The decoy keeps the real helper used, so no unused-item lint fires; the raw identifier is the
    same name to Rust and a different string to every textual rule; and the call — unchanged, still
    a plain statement at the caller's own depth — runs the no-op. That passed this checker, all 303
    suite tests, fmt, clippy and every cargo gate while the load-bearing runtime proof and length
    sweep did not execute at all.

    Enumerating spellings would not close it, so this turns on two facts instead. A shadow needs a
    DECLARATION, so no caller may declare an item; and a declaration must WRITE the name, so no
    caller may spell a load-bearing name more than the once its own call spends. `use x as helper;`,
    `let helper = …`, `const helper: … `, a `mod`, a `macro_rules!` and a closure parameter are each
    caught by one or the other, in any spelling.
    """
    try:
        test_source = (ROOT / PLATFORM_IDENTITY_TEST).read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity helper-resolution carrier unreadable: {error}", issues)
        return
    test_code = strip_rust_comments_and_literals(test_source)
    module_tokens = rust_without_raw_identifiers(rust_token_stream(test_code))

    # 1. Each load-bearing helper is declared exactly once in the module, so every rule that reads
    #    "the" helper's body reads the one body its call could resolve to.
    for helper in PLATFORM_IDENTITY_LOAD_BEARING_HELPERS:
        declared = sum(
            1
            for index in range(len(module_tokens) - 1)
            if module_tokens[index] == "fn" and module_tokens[index + 1] == helper
        )
        if declared != 1:
            fail(
                f"platform identity helper resolution: {helper} is declared {declared} times in "
                f"{PLATFORM_IDENTITY_TEST}, expected exactly one — a second declaration of the same "
                "name is a shadow, whether or not it is spelled raw",
                issues,
            )

    for caller in PLATFORM_IDENTITY_SHADOWABLE_CALLERS:
        body = _rust_function_body(test_code, caller)
        if body is None:
            fail(f"platform identity helper resolution: {caller} body unreadable", issues)
            continue
        tokens = rust_without_raw_identifiers(rust_token_stream(body))

        # 2. The caller declares no item at all. An item declared in a body binds its name ahead of
        #    the module's for the whole of that body, including a `use` glob that names nothing.
        items = sorted(set(PLATFORM_IDENTITY_ITEM_KEYWORDS).intersection(tokens))
        if items:
            fail(
                f"platform identity helper resolution: {caller} declares {items} — a local item "
                "binds its name ahead of the module's, so a call it shadows proves nothing",
                issues,
            )

        # 3. …and it spends each load-bearing name once, on the call. A binding that is not an item
        #    still has to write the name, and this is where it would have to write it.
        for helper in PLATFORM_IDENTITY_LOAD_BEARING_HELPERS:
            spelled = tokens.count(helper)
            if spelled > 1:
                fail(
                    f"platform identity helper resolution: {caller} spells {helper} {spelled} "
                    "times, expected at most the one call — a second mention is how a shadow is "
                    "written",
                    issues,
                )


def _platform_identity_sweep_carriers(issues: list[str]) -> None:
    """Binds the VALUES the length sweep is driven by, not merely the names its body spells.

    The sweep's token sequence is bound above, which fixes the loops and leaves what they range
    over free. `const RUNTIME_PROOF_SEEDS: [&str; 0] = [];` left every bound token in place and
    swept nothing; `const RUNTIME_PROOF_SWEEP: usize = GRAMMAR_MAX_BYTES;` left every bound token in
    place and swept nothing PAST the bound, which is the half that matters. Both passed this
    checker, all 303 suite tests and every cargo gate.

    So each declaration is bound whole, at module level, exactly once — an alias, a helper call, a
    `const` expression, a `cfg` twin and a macro-generated span all fail the shape rather than being
    enumerated. The seeds' payloads are read from the unstripped span, because the stripper removes
    exactly the bytes this rule is about.
    """
    seeds = PLATFORM_IDENTITY_RUNTIME_SWEEP_SEEDS
    span = PLATFORM_IDENTITY_RUNTIME_SWEEP_SPAN
    bound = PLATFORM_IDENTITY_RUNTIME_PROOF_CONSTANT
    try:
        test_source = (ROOT / PLATFORM_IDENTITY_TEST).read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity sweep-carrier file unreadable: {error}", issues)
        return
    code = strip_rust_comments_and_literals(test_source)
    kept = strip_rust_comments_and_literals(test_source, keep_literals=True)

    expected = {
        seeds: PLATFORM_IDENTITY_RUNTIME_SWEEP_SEED_DECLARATION.format(name=seeds),
        span: PLATFORM_IDENTITY_RUNTIME_SWEEP_SPAN_DECLARATION.format(name=span, bound=bound),
    }
    for name, shape in expected.items():
        # Shape and count on stripped code, where a brace is a scope; payloads on the kept text,
        # because the stripper removes exactly the bytes the seeds rule is about.
        declarations = rust_module_level_declarations(code, name)
        payload_spans = rust_declaration_spans(kept, name)
        if len(declarations) != 1 or len(payload_spans) != 1:
            fail(
                f"platform identity sweep carrier: {name} is declared {len(declarations)} times at "
                f"module level and spelled {len(payload_spans)} times as a declaration, expected "
                "exactly one of each — a second declaration is a `cfg` twin",
                issues,
            )
            continue
        declared = " ".join(rust_token_stream(declarations[0]))
        if declared != shape:
            fail(
                f"platform identity sweep carrier: {name} is declared `{declared}`, expected "
                f"`{shape}` — the sweep's extent may not come from an alias, a helper, a macro or "
                "an expression this rule cannot evaluate",
                issues,
            )
            continue
        if name == seeds:
            values = tuple(rust_string_literals(payload_spans[0]))
            if values != PLATFORM_IDENTITY_RUNTIME_SWEEP_SEED_VALUES:
                fail(
                    f"platform identity sweep carrier: {name} is {values}, expected "
                    f"{PLATFORM_IDENTITY_RUNTIME_SWEEP_SEED_VALUES} — two distinct single-byte "
                    "seeds the grammar admits, so each sweeps lengths rather than multiples",
                    issues,
                )

    # And the number both of them are stated in terms of is still the contract's own.
    declared_bound = rust_module_level_usize_constants(code, bound)
    if declared_bound != (PLATFORM_IDENTITY_GRAMMAR["max_bytes"],):
        fail(
            f"platform identity sweep carrier: {bound} is {declared_bound}, expected exactly one "
            f"module-level {PLATFORM_IDENTITY_GRAMMAR['max_bytes']} — the span and both coverage "
            "counts are stated in terms of it",
            issues,
        )


def _platform_identity_contract_grammar(issues: list[str]) -> None:
    """Cross-checks the checker's semantic table against the accepted contract."""
    path = ROOT / PLATFORM_IDENTITY_CONTRACT
    try:
        contract = path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity contract unreadable: {error}", issues)
        return

    fences = re.findall(r"^```regex\n(.*?)\n```$", contract, flags=re.MULTILINE | re.DOTALL)
    if len(fences) != 1:
        fail(
            "platform identity contract must carry exactly one normative regex carrier: "
            f"found {len(fences)}",
            issues,
        )
        return
    carrier = fences[0].strip()
    expected = PLATFORM_IDENTITY_GRAMMAR["regex"]
    if carrier != expected:
        fail(
            "platform identity grammar-contract mismatch: contract regex "
            f"{carrier!r} != checker table {expected!r}",
            issues,
        )
        return

    shape = PLATFORM_IDENTITY_GRAMMAR_SHAPE.match(carrier)
    if shape is None:
        fail(
            f"platform identity contract regex is not the frozen shape: {carrier!r}",
            issues,
        )
        return
    if shape.group("lead") != shape.group("tail"):
        fail(
            "platform identity grammar-contract mismatch: leading and trailing boundary "
            f"classes differ ({shape.group('lead')!r} vs {shape.group('tail')!r})",
            issues,
        )
    boundary, boundary_duplicate = rust_character_class_bytes(shape.group("lead"))
    interior, interior_duplicate = rust_character_class_bytes(shape.group("interior"))
    if boundary_duplicate or interior_duplicate:
        fail("platform identity contract regex repeats a character-class byte", issues)
    if frozenset(boundary) != ASCII_ALPHANUMERIC_BYTES:
        fail(
            "platform identity grammar-contract mismatch: boundary class is not "
            f"{PLATFORM_IDENTITY_GRAMMAR['boundary_class']}",
            issues,
        )
    extras = tuple(sorted(frozenset(interior) - ASCII_ALPHANUMERIC_BYTES))
    admitted = tuple(sorted(PLATFORM_IDENTITY_GRAMMAR["interior_extra_bytes"]))
    if extras != admitted:
        fail(
            "platform identity grammar-contract mismatch: interior delimiter set "
            f"{''.join(extras)!r} != checker table {''.join(admitted)!r}",
            issues,
        )
    if frozenset(interior) - ASCII_ALPHANUMERIC_BYTES != frozenset(interior) - frozenset(
        boundary
    ):
        fail(
            "platform identity grammar-contract mismatch: interior class does not extend the "
            "boundary class",
            issues,
        )
    max_bytes = int(shape.group("bound")) + 2
    if max_bytes != PLATFORM_IDENTITY_GRAMMAR["max_bytes"]:
        fail(
            "platform identity grammar-contract mismatch: contract regex admits "
            f"{max_bytes} bytes != checker table {PLATFORM_IDENTITY_GRAMMAR['max_bytes']}",
            issues,
        )

    # Each remaining field is bound to its own anchored normative line, by list position and
    # exact text. A whole-document substring search would let one surviving mention prove a
    # value that had moved everywhere it is actually used.
    start = contract.find(PLATFORM_IDENTITY_CONTRACT_SECTION)
    end = contract.find(PLATFORM_IDENTITY_CONTRACT_NEXT_SECTION, max(start, 0))
    if start == -1 or end == -1:
        fail("platform identity contract lost its normative grammar section", issues)
        return
    numbered = {
        int(match.group(1)): match.group(2).strip()
        for match in re.finditer(
            r"^(\d+)\. (.+)$", contract[start:end], flags=re.MULTILINE
        )
    }
    for position, text in PLATFORM_IDENTITY_NORMATIVE_LINES.items():
        if numbered.get(position) != text:
            fail(
                f"platform identity grammar-contract mismatch: normative line {position} is "
                f"{numbered.get(position)!r}, expected {text!r}",
                issues,
            )
    length_line = f"encoded length is `1..={PLATFORM_IDENTITY_GRAMMAR['max_bytes']}` bytes;"
    if numbered.get(1) != length_line:
        fail(
            "platform identity grammar-contract mismatch: max-byte line is "
            f"{numbered.get(1)!r}, expected {length_line!r}",
            issues,
        )
    interior_line = numbered.get(3, "")
    quoted = tuple(sorted(frozenset(re.findall(r"`(.)`", interior_line))))
    if not interior_line.startswith("interior bytes are ASCII alphanumeric or one of "):
        fail(
            f"platform identity grammar-contract mismatch: interior line is {interior_line!r}",
            issues,
        )
    elif quoted != admitted:
        fail(
            "platform identity grammar-contract mismatch: interior line names "
            f"{''.join(quoted)!r} != checker table {''.join(admitted)!r}",
            issues,
        )
    if PLATFORM_IDENTITY_GRAMMAR["normalization"] != "NONE" or numbered.get(
        6
    ) != PLATFORM_IDENTITY_NORMALIZATION_LINE:
        fail(
            "platform identity grammar-contract mismatch: normalization line is "
            f"{numbered.get(6)!r}",
            issues,
        )
    if not PLATFORM_IDENTITY_GRAMMAR["case_sensitive"]:
        fail("platform identity checker table must keep case significant", issues)


def _platform_identity_semantic_literals(issues: list[str]) -> None:
    """Extracts the grammar's LITERAL semantics from production, oracle and bound corpus.

    Read from comment-stripped but literal-PRESERVING source, because these bytes are exactly
    what the general body fingerprint drops. Each carrier is located through the same function
    extractor the exact body inventory uses and is required to be an ADMITTED function, so a
    decoy string, a comment or an unadmitted helper cannot answer in its place.
    """
    admitted_functions = frozenset(PLATFORM_IDENTITY_ADMITTED_FUNCTIONS)
    admitted = tuple(sorted(PLATFORM_IDENTITY_GRAMMAR["interior_extra_bytes"]))

    source_path = ROOT / PLATFORM_IDENTITY_SOURCE
    test_path = ROOT / PLATFORM_IDENTITY_TEST
    try:
        source = source_path.read_text(encoding="utf-8")
        test_source = test_path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"platform identity semantic carrier unreadable: {error}", issues)
        return
    code = strip_rust_comments_and_literals(source, keep_literals=True)
    test_code = strip_rust_comments_and_literals(test_source, keep_literals=True)

    # 1. The length bound, from the declaration rather than from any mention of the number.
    declaration = re.search(
        rf"\bconst\s+{PLATFORM_IDENTITY_LENGTH_CONSTANT}\s*:\s*usize\s*=\s*(\d+)\s*;", code
    )
    if declaration is None:
        fail(
            f"platform identity length bound missing: {PLATFORM_IDENTITY_LENGTH_CONSTANT}",
            issues,
        )
    elif int(declaration.group(1)) != PLATFORM_IDENTITY_GRAMMAR["max_bytes"]:
        fail(
            "platform identity grammar-contract mismatch: production max bytes "
            f"{declaration.group(1)} != contract {PLATFORM_IDENTITY_GRAMMAR['max_bytes']}",
            issues,
        )

    # 2. The boundary class, as the whole admitted body of the boundary predicate.
    for name, expected in (
        (PLATFORM_IDENTITY_BOUNDARY_FUNCTION, PLATFORM_IDENTITY_BOUNDARY_PREDICATE),
    ):
        if name not in admitted_functions:
            fail(f"platform identity boundary carrier is not an admitted function: {name}", issues)
            continue
        body = _rust_function_body(code, name)
        if body is None:
            fail(f"platform identity boundary carrier unreadable: {name}", issues)
            continue
        if " ".join(body.split()) != f"{{ {expected} }}":
            fail(
                "platform identity grammar-contract mismatch: production boundary class is "
                f"{' '.join(body.split())!r}, expected exactly {{ {expected} }}",
                issues,
            )

    # 3. The interior delimiter set, as an ordered byte-literal list so a duplicated delimiter
    #    and a missing one are distinguishable from the correct set.
    if PLATFORM_IDENTITY_INTERIOR_FUNCTION not in admitted_functions:
        fail(
            "platform identity interior carrier is not an admitted function: "
            f"{PLATFORM_IDENTITY_INTERIOR_FUNCTION}",
            issues,
        )
    else:
        body = _rust_function_body(code, PLATFORM_IDENTITY_INTERIOR_FUNCTION)
        if body is None:
            fail(
                "platform identity interior carrier unreadable: "
                f"{PLATFORM_IDENTITY_INTERIOR_FUNCTION}",
                issues,
            )
        else:
            shape = PLATFORM_IDENTITY_INTERIOR_SHAPE.match(" ".join(body.split()))
            if shape is None:
                fail(
                    "platform identity interior carrier is not the frozen shape: "
                    f"{' '.join(body.split())!r}",
                    issues,
                )
            else:
                declared: list[str] = []
                malformed: list[str] = []
                for alternative in shape.group("alternatives").split("|"):
                    literal = PLATFORM_IDENTITY_BYTE_LITERAL.match(alternative.strip())
                    if literal is None:
                        malformed.append(alternative.strip())
                        continue
                    declared.append(literal.group("byte"))
                if malformed:
                    fail(
                        f"platform identity interior delimiter is not a byte literal: {malformed}",
                        issues,
                    )
                if tuple(sorted(declared)) != admitted or len(declared) != len(admitted):
                    fail(
                        "platform identity grammar-contract mismatch: production interior "
                        f"delimiters {''.join(sorted(declared))!r} "
                        f"(count {len(declared)}) != contract {''.join(admitted)!r} "
                        f"(count {len(admitted)})",
                        issues,
                    )

    # 4. The production restatement of the regex, so the two normative texts cannot diverge.
    restatements = [
        match.group(1)
        for line in rust_doc_comment_stream(source)
        for match in re.finditer(r"`(\^\[[^`]*\$)`", line)
    ]
    if restatements != [PLATFORM_IDENTITY_GRAMMAR["regex"]]:
        fail(
            "platform identity grammar-contract mismatch: production regex restatement "
            f"{restatements} != [contract regex]",
            issues,
        )

    # 5. The exhaustive oracle's own delimiter table, bound to its admitted body — not proven by
    #    one literal appearing somewhere in the file.
    oracle = _rust_function_body(test_code, PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE)
    if oracle is None:
        fail(
            "platform identity exhaustive grammar oracle unreadable: "
            f"{PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE}",
            issues,
        )
    else:
        tables = PLATFORM_IDENTITY_ORACLE_BYTE_STRING.findall(oracle)
        if len(tables) != 1:
            fail(
                "platform identity exhaustive grammar oracle must carry exactly one delimiter "
                f"table: found {len(tables)}",
                issues,
            )
        elif tuple(sorted(tables[0])) != admitted or len(tables[0]) != len(admitted):
            fail(
                "platform identity grammar-contract mismatch: oracle interior delimiters "
                f"{tables[0]!r} != contract {''.join(admitted)!r}",
                issues,
            )
        if PLATFORM_IDENTITY_BOUNDARY_PREDICATE not in " ".join(oracle.split()):
            fail(
                "platform identity exhaustive grammar oracle lost its boundary predicate: "
                f"{PLATFORM_IDENTITY_BOUNDARY_PREDICATE}",
                issues,
            )

    # 6. The bound valid corpus must exercise every contract-derived delimiter, so a corpus
    #    drifted alongside production is caught even where a runtime assertion would still pass.
    corpus = _rust_function_body(test_code, PLATFORM_IDENTITY_VALID_CORPUS_FUNCTION)
    if corpus is None:
        fail(
            "platform identity valid corpus unreadable: "
            f"{PLATFORM_IDENTITY_VALID_CORPUS_FUNCTION}",
            issues,
        )
    else:
        values = "".join(rust_string_literals(corpus))
        missing = [byte for byte in admitted if byte not in values]
        if missing:
            fail(
                "platform identity valid corpus does not exercise every contract delimiter: "
                f"missing {''.join(missing)!r}",
                issues,
            )


def check_platform_identity_grammar_authority(issues: list[str]) -> None:
    """Binds grammar SEMANTICS to the accepted contract rather than to agreement among carriers."""
    _platform_identity_contract_grammar(issues)
    _platform_identity_semantic_literals(issues)
    _platform_identity_effective_bound(issues)
    _platform_identity_effective_guard(issues)
    _platform_identity_classify_procedure(issues)
    _platform_identity_helper_resolution(issues)
    _platform_identity_sweep_carriers(issues)


def _check_bound_rust_test_file(
    rel_path: str,
    expected_functions: tuple[str, ...],
    label: str,
    issues: list[str],
) -> None:
    path = ROOT / rel_path
    if not path.is_file():
        fail(f"{label} test carrier missing: {rel_path}", issues)
        return
    code = strip_rust_comments_and_literals(path.read_text(encoding="utf-8"))
    for marker, pattern in PLATFORM_IDENTITY_FORBIDDEN_TEST_FILE_PATTERNS:
        if marker == "#![ (inner attribute)":
            continue
        if re.search(pattern, code):
            fail(f"{label} tests must execute unconditionally: {marker!r} is forbidden", issues)
    if re.search(r"#\s*!\s*\[\s*(?:cfg|cfg_attr)\b", code):
        fail(f"{label} tests must execute unconditionally: inner cfg is forbidden", issues)
    registered_tests = 0
    for function in expected_functions:
        if not re.search(rf"^fn {function}\(\)", code, flags=re.MULTILINE):
            fail(f"{label} acceptance test missing: {function}", issues)
            continue
        attributes = rust_attribute_block(code, function)
        if attributes != list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES):
            fail(
                f"{label} acceptance test {function} attribute envelope drifted: "
                f"expected {list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES)} actual={attributes}",
                issues,
            )
        else:
            registered_tests += 1
    if registered_tests != len(expected_functions):
        fail(
            f"{label} acceptance test registration drift: expected "
            f"{len(expected_functions)} executable tests actual={registered_tests}",
            issues,
        )


def _platform_core_installation_surface_enabled(market_code: str) -> bool:
    installation_path = ROOT / PLATFORM_INSTALLATION_SOURCE
    return installation_path.is_file() or re.search(
        r"^\s*pub\s+mod\s+installation\s*;", market_code, flags=re.MULTILINE
    ) is not None


def _check_market_installation_surface(market_code: str, issues: list[str]) -> bool:
    """Pins the M20-B3-s1 nested installation module once that surface appears."""
    enabled = _platform_core_installation_surface_enabled(market_code)
    if not enabled:
        return False
    installation_path = ROOT / PLATFORM_INSTALLATION_SOURCE
    if not re.search(r"^\s*pub\s+mod\s+installation\s*;", market_code, flags=re.MULTILINE):
        fail("market installation module declaration missing from crates/platform-core/src/market.rs", issues)
    if not installation_path.is_file():
        fail(f"market installation carrier missing: {PLATFORM_INSTALLATION_SOURCE}", issues)
        return True
    governed = strip_rust_comments_and_literals(installation_path.read_text(encoding="utf-8"))
    label = PLATFORM_INSTALLATION_SOURCE
    if re.search(r"\bcfg_attr\b", governed):
        fail(f"platform-core source must not carry cfg_attr: {label}", issues)
    if re.search(RUST_INNER_ATTRIBUTE_PATTERN, governed):
        fail(f"platform-core source must not carry an inner attribute: {label}", issues)
    for carrier, pattern in PLATFORM_CORE_FORBIDDEN_SOURCE_PATTERNS + PLATFORM_CORE_FORBIDDEN_SPLICE_PATTERNS:
        if re.search(pattern, governed):
            fail(f"platform-core source must not carry {carrier!r}: {label}", issues)
    items, unterminated = rust_item_declarations(governed)
    if unterminated:
        fail(f"unterminated platform-core item declaration in {label}: {unterminated}", issues)
    admitted_items = list(PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS["market/installation.rs"])
    if items != admitted_items:
        fail(f"market installation item declarations drifted: expected {admitted_items} actual={items}", issues)
    attributes, unterminated_attributes = rust_attributes(governed)
    if unterminated_attributes:
        fail(f"unterminated attribute in {label}: {unterminated_attributes}", issues)
    observed_attributes = rust_attribute_names(attributes)
    admitted_attributes = tuple(sorted(PLATFORM_CORE_ADMITTED_ATTRIBUTE_NAMES["market/installation.rs"]))
    if observed_attributes != admitted_attributes:
        fail(f"market installation attribute names drifted: expected {admitted_attributes} actual={observed_attributes}", issues)
    observed_attribute_counts = tuple(sorted(Counter(attributes).items()))
    if observed_attribute_counts != PLATFORM_INSTALLATION_ADMITTED_ATTRIBUTE_COUNTS:
        fail(
            "market installation attribute inventory drifted: expected "
            f"{PLATFORM_INSTALLATION_ADMITTED_ATTRIBUTE_COUNTS} actual={observed_attribute_counts}",
            issues,
        )
    public_declarations, unclassified_public = rust_public_declarations(governed)
    if tuple(unclassified_public) != PLATFORM_INSTALLATION_ADMITTED_UNCLASSIFIED_PUBLIC:
        fail(
            "market installation has an unclassified public declaration: "
            f"expected {PLATFORM_INSTALLATION_ADMITTED_UNCLASSIFIED_PUBLIC} "
            f"actual={unclassified_public}",
            issues,
        )
    restricted_public = rust_market_restricted_public_functions(governed)
    if restricted_public != PLATFORM_INSTALLATION_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS:
        fail(
            "market installation restricted public function surface drifted: "
            f"expected={PLATFORM_INSTALLATION_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS} "
            f"actual={restricted_public}",
            issues,
        )
    event_kind = re.findall(
        r"\bpub\s+enum\s+InstallationEventKind\s*\{([^{}]*)\}",
        governed,
        flags=re.DOTALL,
    )
    observed_event_kind_variants = (
        tuple(re.findall(r"(?m)^\s*([A-Za-z_][A-Za-z0-9_]*)\s*,", event_kind[0]))
        if len(event_kind) == 1
        else ()
    )
    if observed_event_kind_variants != PLATFORM_INSTALLATION_EVENT_KIND_VARIANTS:
        fail(
            "market installation event kind variants drifted: "
            f"expected={PLATFORM_INSTALLATION_EVENT_KIND_VARIANTS} "
            f"actual={observed_event_kind_variants}",
            issues,
        )
    if public_declarations != sorted(PLATFORM_INSTALLATION_ADMITTED_PUBLIC_DECLARATIONS):
        fail(f"market installation public declaration surface drifted: actual={public_declarations}", issues)
    impl_declarations, unclassified_impls = rust_impl_declarations(governed)
    if unclassified_impls:
        fail(f"market installation has an unclassified impl declaration: {unclassified_impls}", issues)
    if impl_declarations != sorted(PLATFORM_CORE_ADMITTED_SIBLING_IMPLS["installation.rs"]):
        fail(f"market installation implementation surface drifted: actual={impl_declarations}", issues)
    derives = sorted(rust_derive_bodies(governed))
    if derives != sorted(PLATFORM_INSTALLATION_ADMITTED_DERIVES):
        fail(f"market installation derive surface drifted: {derives}", issues)
    definitions = rust_macro_definitions(governed)
    if definitions != sorted(PLATFORM_CORE_ADMITTED_SIBLING_MACROS["market/installation.rs"]):
        fail(f"market installation macro definitions drifted: actual={definitions}", issues)
    invocations, unterminated_macros = rust_macro_invocation_arguments(governed)
    if unterminated_macros:
        fail(f"unterminated platform-core macro invocation in {label}: {sorted(unterminated_macros)}", issues)
    invoked = tuple(sorted({name for name, _ in invocations}))
    admitted_invocations = tuple(sorted(PLATFORM_CORE_ADMITTED_MACRO_INVOCATIONS["market/installation.rs"]))
    if invoked != admitted_invocations:
        fail(f"market installation macro invocations drifted: expected {admitted_invocations} actual={invoked}", issues)
    _check_bound_rust_test_file(
        PLATFORM_INSTALLATION_TEST,
        PLATFORM_INSTALLATION_TEST_FUNCTIONS,
        "market installation",
        issues,
    )
    return True


def _check_market_grant_surface(market_code: str, issues: list[str]) -> None:
    """Pins the reviewed M20-B4-I grant authority and its bound external evidence."""
    grant_path = ROOT / PLATFORM_GRANT_SOURCE
    if not re.search(r"^\s*pub\s+mod\s+grant\s*;", market_code, flags=re.MULTILINE):
        fail("market grant module declaration missing from crates/platform-core/src/market.rs", issues)
    if not grant_path.is_file():
        fail(f"market grant carrier missing: {PLATFORM_GRANT_SOURCE}", issues)
        return
    governed = strip_rust_comments_and_literals(grant_path.read_text(encoding="utf-8"))
    label = PLATFORM_GRANT_SOURCE
    if re.search(r"\bcfg_attr\b", governed):
        fail(f"platform-core source must not carry cfg_attr: {label}", issues)
    if re.search(RUST_INNER_ATTRIBUTE_PATTERN, governed):
        fail(f"platform-core source must not carry an inner attribute: {label}", issues)
    for carrier, pattern in PLATFORM_CORE_FORBIDDEN_SOURCE_PATTERNS + PLATFORM_CORE_FORBIDDEN_SPLICE_PATTERNS:
        if re.search(pattern, governed):
            fail(f"platform-core source must not carry {carrier!r}: {label}", issues)
    items, unterminated = rust_item_declarations(governed)
    if unterminated:
        fail(f"unterminated platform-core item declaration in {label}: {unterminated}", issues)
    admitted_items = list(PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS["market/grant.rs"])
    if items != admitted_items:
        fail(f"market grant item declarations drifted: expected {admitted_items} actual={items}", issues)
    attributes, unterminated_attributes = rust_attributes(governed)
    if unterminated_attributes:
        fail(f"unterminated attribute in {label}: {unterminated_attributes}", issues)
    observed_attribute_counts = tuple(
        sorted((attribute, attributes.count(attribute)) for attribute in set(attributes))
    )
    if observed_attribute_counts != PLATFORM_GRANT_ADMITTED_ATTRIBUTE_COUNTS:
        fail(
            "market grant attribute inventory drifted: expected "
            f"{PLATFORM_GRANT_ADMITTED_ATTRIBUTE_COUNTS} actual={observed_attribute_counts}",
            issues,
        )
    public_declarations, unclassified_public = rust_public_declarations(governed)
    if tuple(unclassified_public) != PLATFORM_GRANT_ADMITTED_UNCLASSIFIED_PUBLIC:
        fail(
            "market grant has an unclassified public declaration: "
            f"expected {PLATFORM_GRANT_ADMITTED_UNCLASSIFIED_PUBLIC} actual={unclassified_public}",
            issues,
        )
    restricted_public = rust_market_restricted_public_functions(governed)
    if restricted_public != PLATFORM_GRANT_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS:
        fail(
            "market grant restricted public function surface drifted: "
            f"expected={PLATFORM_GRANT_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS} "
            f"actual={restricted_public}",
            issues,
        )
    if public_declarations != sorted(PLATFORM_GRANT_ADMITTED_PUBLIC_DECLARATIONS):
        fail(f"market grant public declaration surface drifted: actual={public_declarations}", issues)
    impl_declarations, unclassified_impls = rust_impl_declarations(governed)
    if unclassified_impls:
        fail(f"market grant has an unclassified impl declaration: {unclassified_impls}", issues)
    if impl_declarations != sorted(PLATFORM_CORE_ADMITTED_SIBLING_IMPLS["grant.rs"]):
        fail(f"market grant implementation surface drifted: actual={impl_declarations}", issues)
    derives = sorted(rust_derive_bodies(governed))
    if derives != sorted(PLATFORM_GRANT_ADMITTED_DERIVES):
        fail(f"market grant derive surface drifted: {derives}", issues)
    definitions = rust_macro_definitions(governed)
    if definitions != sorted(PLATFORM_CORE_ADMITTED_SIBLING_MACROS["market/grant.rs"]):
        fail(f"market grant macro definitions drifted: actual={definitions}", issues)
    invocations, unterminated_macros = rust_macro_invocation_arguments(governed)
    if unterminated_macros:
        fail(f"unterminated platform-core macro invocation in {label}: {sorted(unterminated_macros)}", issues)
    invoked = tuple(sorted({name for name, _ in invocations}))
    admitted_invocations = tuple(sorted(PLATFORM_CORE_ADMITTED_MACRO_INVOCATIONS["market/grant.rs"]))
    if invoked != admitted_invocations:
        fail(f"market grant macro invocations drifted: expected {admitted_invocations} actual={invoked}", issues)
    observed_invocation_counts = tuple(
        sorted(
            (name, sum(1 for invoked_name, _ in invocations if invoked_name == name))
            for name in invoked
        )
    )
    if observed_invocation_counts != PLATFORM_GRANT_ADMITTED_MACRO_INVOCATION_COUNTS:
        fail(
            "market grant macro invocation counts drifted: expected "
            f"{PLATFORM_GRANT_ADMITTED_MACRO_INVOCATION_COUNTS} "
            f"actual={observed_invocation_counts}",
            issues,
        )
    parsed_arguments = tuple(argument for name, argument in invocations if name == "parsed")
    observed_parsed_argument_counts = tuple(
        sorted(
            (argument, parsed_arguments.count(argument))
            for argument in set(parsed_arguments)
        )
    )
    if observed_parsed_argument_counts != PLATFORM_GRANT_ADMITTED_PARSED_ARGUMENT_COUNTS:
        fail(
            "market grant parsed macro arguments drifted: expected "
            f"{PLATFORM_GRANT_ADMITTED_PARSED_ARGUMENT_COUNTS} "
            f"actual={observed_parsed_argument_counts}",
            issues,
        )
    _check_bound_rust_test_file(
        PLATFORM_GRANT_TEST, PLATFORM_GRANT_TEST_FUNCTIONS, "market grant", issues
    )



def _check_market_update_surface(market_code: str, issues: list[str]) -> None:
    """Pins the reviewed M20-B6 package-update authority and bound external evidence."""
    update_path = ROOT / PLATFORM_UPDATE_SOURCE
    if not re.search(r"^\s*pub\s+mod\s+update\s*;", market_code, flags=re.MULTILINE):
        fail("market update module declaration missing from crates/platform-core/src/market.rs", issues)
    if not update_path.is_file():
        fail(f"market update carrier missing: {PLATFORM_UPDATE_SOURCE}", issues)
        return
    governed = strip_rust_comments_and_literals(update_path.read_text(encoding="utf-8"))
    label = PLATFORM_UPDATE_SOURCE
    if re.search(r"\bcfg_attr\b", governed):
        fail(f"platform-core source must not carry cfg_attr: {label}", issues)
    if re.search(RUST_INNER_ATTRIBUTE_PATTERN, governed):
        fail(f"platform-core source must not carry an inner attribute: {label}", issues)
    for carrier, pattern in PLATFORM_CORE_FORBIDDEN_SOURCE_PATTERNS + PLATFORM_CORE_FORBIDDEN_SPLICE_PATTERNS:
        if re.search(pattern, governed):
            fail(f"platform-core source must not carry {carrier!r}: {label}", issues)
    items, unterminated = rust_item_declarations(governed)
    if unterminated:
        fail(f"unterminated platform-core item declaration in {label}: {unterminated}", issues)
    admitted_items = list(PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS["market/update.rs"])
    if items != admitted_items:
        fail(f"market update item declarations drifted: expected {admitted_items} actual={items}", issues)
    attributes, unterminated_attributes = rust_attributes(governed)
    if unterminated_attributes:
        fail(f"unterminated attribute in {label}: {unterminated_attributes}", issues)
    observed_attribute_counts = tuple(sorted(Counter(attributes).items()))
    if observed_attribute_counts != PLATFORM_UPDATE_ADMITTED_ATTRIBUTE_COUNTS:
        fail("market update attribute inventory drifted: expected " f"{PLATFORM_UPDATE_ADMITTED_ATTRIBUTE_COUNTS} actual={observed_attribute_counts}", issues)
    public_declarations, unclassified_public = rust_public_declarations(governed)
    if tuple(unclassified_public) != PLATFORM_UPDATE_ADMITTED_UNCLASSIFIED_PUBLIC:
        fail("market update has an unclassified public declaration: " f"expected {PLATFORM_UPDATE_ADMITTED_UNCLASSIFIED_PUBLIC} actual={unclassified_public}", issues)
    restricted_public = rust_market_restricted_public_functions(governed)
    if restricted_public != PLATFORM_UPDATE_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS:
        fail(
            "market update restricted public function surface drifted: "
            f"expected={PLATFORM_UPDATE_ADMITTED_RESTRICTED_PUBLIC_FUNCTIONS} "
            f"actual={restricted_public}",
            issues,
        )
    for carrier in PLATFORM_UPDATE_TEST_ONLY_AUTHORITY_CARRIERS:
        if governed.count(carrier) != 1:
            fail(
                "market update test-only authority carrier drifted: "
                f"expected exactly one {carrier!r}",
                issues,
            )
    tests_module = governed.find("mod tests")
    production = governed[:tests_module] if tests_module >= 0 else governed
    observed_context_calls = tuple(
        (call, production.count(call)) for call in PLATFORM_UPDATE_PRODUCTION_CONTEXT_CALLS
    )
    expected_context_calls = tuple(
        (call, 1) for call in PLATFORM_UPDATE_PRODUCTION_CONTEXT_CALLS
    )
    if observed_context_calls != expected_context_calls:
        fail(
            "market update production context constructor reachability drifted: "
            f"expected={expected_context_calls} actual={observed_context_calls}",
            issues,
        )
    named_context_declarations = production.count("UpdateDecisionContext {")
    if named_context_declarations != PLATFORM_UPDATE_PRODUCTION_NAMED_CONTEXT_DECLARATIONS:
        fail(
            "market update production direct context construction drifted: "
            f"expected={PLATFORM_UPDATE_PRODUCTION_NAMED_CONTEXT_DECLARATIONS} "
            f"actual={named_context_declarations}",
            issues,
        )
    if public_declarations != sorted(PLATFORM_UPDATE_ADMITTED_PUBLIC_DECLARATIONS):
        fail(f"market update public declaration surface drifted: actual={public_declarations}", issues)
    impl_declarations, unclassified_impls = rust_impl_declarations(governed)
    if unclassified_impls:
        fail(f"market update has an unclassified impl declaration: {unclassified_impls}", issues)
    if impl_declarations != sorted(PLATFORM_CORE_ADMITTED_SIBLING_IMPLS["update.rs"]):
        fail(f"market update implementation surface drifted: actual={impl_declarations}", issues)
    derives = sorted(rust_derive_bodies(governed))
    if derives != sorted(PLATFORM_UPDATE_ADMITTED_DERIVES):
        fail(f"market update derive surface drifted: {derives}", issues)
    definitions = rust_macro_definitions(governed)
    if definitions != sorted(PLATFORM_CORE_ADMITTED_SIBLING_MACROS["market/update.rs"]):
        fail(f"market update macro definitions drifted: actual={definitions}", issues)
    invocations, unterminated_macros = rust_macro_invocation_arguments(governed)
    if unterminated_macros:
        fail(f"unterminated platform-core macro invocation in {label}: {sorted(unterminated_macros)}", issues)
    invoked = tuple(sorted({name for name, _ in invocations}))
    admitted_invocations = tuple(sorted(PLATFORM_CORE_ADMITTED_MACRO_INVOCATIONS["market/update.rs"]))
    if invoked != admitted_invocations:
        fail(f"market update macro invocations drifted: expected {admitted_invocations} actual={invoked}", issues)
    observed_invocation_counts = tuple(sorted(Counter(name for name, _ in invocations).items()))
    if observed_invocation_counts != PLATFORM_UPDATE_ADMITTED_MACRO_INVOCATION_COUNTS:
        fail("market update macro invocation counts drifted: expected " f"{PLATFORM_UPDATE_ADMITTED_MACRO_INVOCATION_COUNTS} actual={observed_invocation_counts}", issues)
    parsed_arguments = tuple(argument for name, argument in invocations if name == "parsed")
    observed_parsed_argument_counts = tuple(sorted(Counter(parsed_arguments).items()))
    if observed_parsed_argument_counts != PLATFORM_UPDATE_ADMITTED_PARSED_ARGUMENT_COUNTS:
        fail("market update parsed macro arguments drifted: expected " f"{PLATFORM_UPDATE_ADMITTED_PARSED_ARGUMENT_COUNTS} actual={observed_parsed_argument_counts}", issues)
    source_test_names = tuple(match.group(1) for match in re.finditer(r"#\s*\[\s*test\s*\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", governed))
    if source_test_names != PLATFORM_UPDATE_UNIT_TEST_FUNCTIONS:
        fail("market update unit test inventory drifted: " f"expected={PLATFORM_UPDATE_UNIT_TEST_FUNCTIONS} actual={source_test_names}", issues)
    _check_bound_rust_test_file(PLATFORM_UPDATE_TEST, PLATFORM_UPDATE_TEST_FUNCTIONS, "market update", issues)

def check_platform_authority_implementation(issues: list[str]) -> None:
    source_path = ROOT / PLATFORM_AUTHORITY_SOURCE
    test_path = ROOT / PLATFORM_AUTHORITY_TEST
    invocation_path = ROOT / PLATFORM_INVOCATION_SOURCE
    invocation_test_path = ROOT / "crates/platform-core/tests/invocation_resolution.rs"
    if not all(path.is_file() for path in (source_path, test_path, invocation_path, invocation_test_path)):
        fail("market authority source/test carrier missing", issues)
        return

    source = strip_rust_comments_and_literals(source_path.read_text(encoding="utf-8"))
    test = strip_rust_comments_and_literals(test_path.read_text(encoding="utf-8"))
    invocation = strip_rust_comments_and_literals(invocation_path.read_text(encoding="utf-8"))
    invocation_test = strip_rust_comments_and_literals(
        invocation_test_path.read_text(encoding="utf-8")
    )

    def digest(value: str) -> str:
        return hashlib.sha256(value.encode("utf-8")).hexdigest()

    if digest(source) != PLATFORM_AUTHORITY_SOURCE_SHA256:
        fail("market authority normalized source digest drifted", issues)
    if digest(test) != PLATFORM_AUTHORITY_TEST_SHA256:
        fail("market authority external test normalized digest drifted", issues)

    public_declarations, unclassified_public = rust_public_declarations(source)
    if unclassified_public:
        fail(f"market authority has unclassified public declarations: {unclassified_public}", issues)
    if tuple(public_declarations) != PLATFORM_AUTHORITY_ADMITTED_PUBLIC_DECLARATIONS:
        fail(
            "market authority public declaration surface drifted: "
            f"expected={PLATFORM_AUTHORITY_ADMITTED_PUBLIC_DECLARATIONS} "
            f"actual={tuple(public_declarations)}",
            issues,
        )

    attributes, unterminated_attributes = rust_attributes(source)
    if unterminated_attributes:
        fail(f"market authority has unterminated attributes: {unterminated_attributes}", issues)
    observed_attribute_counts = tuple(sorted(Counter(attributes).items()))
    if observed_attribute_counts != PLATFORM_AUTHORITY_ADMITTED_ATTRIBUTE_COUNTS:
        fail(
            "market authority attribute multiset drifted: "
            f"expected={PLATFORM_AUTHORITY_ADMITTED_ATTRIBUTE_COUNTS} "
            f"actual={observed_attribute_counts}",
            issues,
        )
    observed_derives = tuple(rust_derive_bodies(source))
    if observed_derives != PLATFORM_AUTHORITY_ADMITTED_DERIVES:
        fail(
            "market authority derive surface drifted: "
            f"expected={PLATFORM_AUTHORITY_ADMITTED_DERIVES} actual={observed_derives}",
            issues,
        )

    invocations, unterminated_macros = rust_macro_invocation_arguments(source)
    if unterminated_macros:
        fail(f"market authority has unterminated macro invocations: {unterminated_macros}", issues)
    observed_macro_counts = tuple(sorted(Counter(name for name, _ in invocations).items()))
    if observed_macro_counts != PLATFORM_AUTHORITY_ADMITTED_MACRO_INVOCATION_COUNTS:
        fail(
            "market authority macro invocation counts drifted: "
            f"expected={PLATFORM_AUTHORITY_ADMITTED_MACRO_INVOCATION_COUNTS} "
            f"actual={observed_macro_counts}",
            issues,
        )
    observed_parsed_counts = tuple(
        sorted(Counter(argument for name, argument in invocations if name == "parsed").items())
    )
    if observed_parsed_counts != PLATFORM_AUTHORITY_ADMITTED_PARSED_ARGUMENT_COUNTS:
        fail(
            "market authority parsed macro arguments drifted: "
            f"expected={PLATFORM_AUTHORITY_ADMITTED_PARSED_ARGUMENT_COUNTS} "
            f"actual={observed_parsed_counts}",
            issues,
        )

    def check_function_hashes(
        code: str,
        expected: dict[str, tuple[str, ...]],
        label: str,
    ) -> None:
        functions, unterminated = rust_functions(code)
        if unterminated:
            fail(f"{label} has unterminated function bodies: {unterminated}", issues)
        for name, admitted_hashes in expected.items():
            observed_hashes = tuple(digest(body) for function, body in functions if function == name)
            if observed_hashes != admitted_hashes:
                fail(
                    f"{label} function body drifted for {name}: "
                    f"expected={admitted_hashes} actual={observed_hashes}",
                    issues,
                )

    check_function_hashes(source, PLATFORM_AUTHORITY_FUNCTION_BODY_SHA256, "market authority")
    check_function_hashes(
        invocation,
        PLATFORM_INVOCATION_AUTHORITY_FUNCTION_BODY_SHA256,
        "invocation authority prefix",
    )

    source_test_names = tuple(
        match.group(1)
        for match in re.finditer(
            r"#\s*\[\s*test\s*\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
            source,
        )
    )
    if source_test_names != PLATFORM_AUTHORITY_UNIT_TEST_FUNCTIONS:
        fail(
            "market authority unit test inventory drifted: "
            f"expected={PLATFORM_AUTHORITY_UNIT_TEST_FUNCTIONS} actual={source_test_names}",
            issues,
        )
    _check_bound_rust_test_file(
        PLATFORM_AUTHORITY_TEST,
        PLATFORM_AUTHORITY_TEST_FUNCTIONS,
        "market authority assembly",
        issues,
    )

    prefix_body = _rust_function_body(invocation_test, PLATFORM_INVOCATION_PREFIX_TEST)
    if prefix_body is None:
        fail("invocation shared-prefix regression test missing", issues)
    else:
        if not re.search(
            rf"#\s*\[\s*test\s*\]\s*fn\s+{PLATFORM_INVOCATION_PREFIX_TEST}\s*\(",
            invocation_test,
        ):
            fail("invocation shared-prefix regression test is not active", issues)
        if digest(prefix_body) != PLATFORM_INVOCATION_PREFIX_TEST_BODY_SHA256:
            fail("invocation shared-prefix regression test body drifted", issues)

    for relative, markers in PLATFORM_AUTHORITY_STATUS_MARKERS.items():
        path = ROOT / relative
        if not path.is_file():
            fail(f"market authority status carrier missing: {relative}", issues)
            continue
        content = path.read_text(encoding="utf-8")
        for marker in markers:
            if marker not in content:
                fail(f"market authority status marker missing in {relative}: {marker!r}", issues)

    matrix = (ROOT / "docs/acceptance/matrix.tsv").read_text(encoding="utf-8").splitlines()
    rows = {fields[0]: fields for line in matrix[1:] if (fields := line.split("\t"))}
    for case_id in ("MARKET-003", "MARKET-004", "MARKET-007", "PKG-020"):
        fields = rows.get(case_id)
        if fields is None or len(fields) != 7 or fields[5] != "planned":
            fail(f"{case_id} must remain a planned acceptance row after bounded B6", issues)


def check_platform_identity_implementation(issues: list[str]) -> None:
    source_path = ROOT / PLATFORM_IDENTITY_SOURCE
    test_path = ROOT / PLATFORM_IDENTITY_TEST
    lib_path = ROOT / PLATFORM_CORE_LIB
    invocation_path = ROOT / PLATFORM_INVOCATION_SOURCE
    market_path = ROOT / PLATFORM_MARKET_SOURCE

    for rel, path in (
        (PLATFORM_IDENTITY_SOURCE, source_path),
        (PLATFORM_IDENTITY_TEST, test_path),
        (PLATFORM_CORE_LIB, lib_path),
        (PLATFORM_INVOCATION_SOURCE, invocation_path),
        (PLATFORM_MARKET_SOURCE, market_path),
    ):
        if not path.is_file():
            fail(f"platform identity carrier missing: {rel}", issues)
            return

    source = source_path.read_text(encoding="utf-8")
    code = strip_rust_comments_and_literals(source)
    docs = rust_doc_comment_stream(source)
    lib_code = strip_rust_comments_and_literals(lib_path.read_text(encoding="utf-8"))
    invocation_code = strip_rust_comments_and_literals(
        invocation_path.read_text(encoding="utf-8")
    )
    market_code = strip_rust_comments_and_literals(market_path.read_text(encoding="utf-8"))
    _check_market_installation_surface(market_code, issues)
    _check_market_grant_surface(market_code, issues)
    _check_market_update_surface(market_code, issues)
    test_code = strip_rust_comments_and_literals(test_path.read_text(encoding="utf-8"))

    if not re.search(r"^\s*pub mod identity;$", lib_code, flags=re.MULTILINE):
        fail("platform-core must export the M00 identity module", issues)

    for carrier in PLATFORM_IDENTITY_CODE_CARRIERS:
        if carrier not in code:
            fail(f"platform identity public definition missing: {carrier!r}", issues)
    for variant in PLATFORM_IDENTITY_ERROR_VARIANTS:
        if variant not in code:
            fail(f"platform identity error taxonomy carrier missing: {variant!r}", issues)

    # THE closure for every construction path, Serde's included. A private field can only be
    # filled by this module's own tuple/struct-literal syntax, so requiring exactly one such
    # expression — inside the checked constructor — leaves an extra visitor arm, an early
    # return, a branch or a future trait impl with nowhere to build the value.
    # Function declarations are inventoried too: a bare helper is invisible to the `pub` scan
    # and to the `mod`/`use`/`type` item accounting, and it is where a construction would hide.
    declared_functions = sorted(
        re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", code, flags=re.ASCII)
    )
    if declared_functions != sorted(PLATFORM_IDENTITY_ADMITTED_FUNCTIONS):
        fail(
            "platform identity function inventory drifted: expected "
            f"{sorted(PLATFORM_IDENTITY_ADMITTED_FUNCTIONS)} actual={declared_functions}",
            issues,
        )
    # …and their exact BODIES, because a name inventory says nothing about what a function does
    # and a containment check says nothing about what else it does. An early return above the
    # admitted call, a branch that skips `classify`, or a construction reached without it are
    # all body-level edits that leave every other rule in this file satisfied.
    declared_bodies, unresolved_bodies = rust_functions(code)
    if unresolved_bodies:
        fail(
            f"platform identity function body unreadable: {unresolved_bodies}",
            issues,
        )
    expected_bodies = [list(entry) for entry in PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES]
    actual_bodies = [[name, body] for name, body in declared_bodies]
    if actual_bodies != expected_bodies:
        drift = next(
            (
                f"{actual[0]}: {actual[1]!r}"
                for actual, expected in zip(actual_bodies, expected_bodies)
                if actual != expected
            ),
            f"count {len(actual_bodies)} != {len(expected_bodies)}",
        )
        fail(f"platform identity function body drifted: {drift}", issues)
    constructions = rust_newtype_constructions(code, PLATFORM_IDENTITY_CONSTRUCTION_FORMS)
    if constructions != list(PLATFORM_IDENTITY_ADMITTED_CONSTRUCTIONS):
        fail(
            "platform identity value is constructed outside the checked constructor: expected "
            f"{list(PLATFORM_IDENTITY_ADMITTED_CONSTRUCTIONS)} actual={constructions}",
            issues,
        )
    else:
        # One construction is only safe if it is the one inside `parse`.
        constructor_body = _rust_function_body(code, PLATFORM_IDENTITY_CONSTRUCTOR_FUNCTION)
        if constructor_body is None or not rust_newtype_constructions(
            constructor_body, PLATFORM_IDENTITY_CONSTRUCTION_FORMS
        ):
            fail(
                "platform identity value construction does not live in "
                f"{PLATFORM_IDENTITY_CONSTRUCTOR_FUNCTION}",
                issues,
            )
    # The two bodies the contract rests on, checked by exact equality and named separately so a
    # failure says which invariant broke rather than only that the module drifted.
    for label, function, expected in (
        ("checked constructor", PLATFORM_IDENTITY_CONSTRUCTOR_FUNCTION, PLATFORM_IDENTITY_PARSE_BODY),
        ("Deserialize", "deserialize", PLATFORM_IDENTITY_DESERIALIZE_BODY),
    ):
        body = _rust_function_body(code, function)
        if body is None:
            fail(f"platform identity {label} body unreadable", issues)
            continue
        normalized = " ".join(body.split())
        if normalized != expected:
            fail(
                f"platform identity {label} body is not the frozen one: "
                f"expected {expected!r} actual={normalized!r}",
                issues,
            )
    # A named-field struct has no constructor function item, so `let ctor = $name;` does not
    # compile and a struct literal — syntax, not a value — is the only way to produce one. The
    # tuple form is rejected outright rather than merely absent.
    if PLATFORM_IDENTITY_STRUCT_DECLARATION not in " ".join(code.split()):
        fail(
            "platform identity value representation drifted: expected "
            f"{PLATFORM_IDENTITY_STRUCT_DECLARATION!r}",
            issues,
        )
    for carrier, pattern in PLATFORM_IDENTITY_FORBIDDEN_CONSTRUCTOR_ITEMS:
        if re.search(pattern, code):
            fail(
                "platform identity module must declare no constructor function item: "
                f"{carrier!r} is forbidden",
                issues,
            )
    # A hand-written visitor is what reopened this class twice; the module carries none, so
    # there is no per-method arm set for evidence to keep enumerating.
    for carrier, pattern in PLATFORM_IDENTITY_FORBIDDEN_SERDE_CARRIERS:
        if re.search(pattern, code):
            fail(
                "platform identity module must not hand-write a Serde visitor: "
                f"{carrier!r} is forbidden",
                issues,
            )

    # The generator macro and its invocations are structurally frozen. Without this, the
    # matcher can be widened to accept `$(, $extra:item)*` and an existing invocation can
    # forward a real trait implementation, adding public API with no new macro definition.
    macro_definitions = re.findall(r"macro_rules\s*!\s*([A-Za-z_][A-Za-z0-9_]*)", code)
    if macro_definitions != [PLATFORM_IDENTITY_MACRO_NAME]:
        fail(
            "platform identity module macro definitions drifted: expected exactly "
            f"['{PLATFORM_IDENTITY_MACRO_NAME}'] actual={macro_definitions}",
            issues,
        )
    matcher_present = any(
        " ".join(line.split()) == PLATFORM_IDENTITY_MACRO_MATCHER for line in code.splitlines()
    )
    if not matcher_present:
        fail(
            "platform identity value generator matcher drifted from the frozen grammar: "
            f"expected {PLATFORM_IDENTITY_MACRO_MATCHER!r}",
            issues,
        )
    # One arm only. A macro may carry several arms, so pinning one matcher line does not stop
    # a second arm from being added beside it to forward an arbitrary item.
    definition = re.search(r"macro_rules\s*!\s*[A-Za-z_][A-Za-z0-9_]*\s*\{", code)
    macro_body = None if definition is None else _rust_balanced_block(code, definition.end() - 1)
    if macro_body is None:
        fail("platform identity value generator definition is unreadable", issues)
    else:
        # Only arm separators count. A `=>` inside the transcriber (the generated `match`)
        # sits at a deeper brace level and must not be mistaken for a second arm.
        arms = 0
        depth = 0
        for index, character in enumerate(macro_body):
            if character == "{":
                depth += 1
            elif character == "}":
                depth -= 1
            elif character == "=" and depth == 1 and macro_body[index : index + 2] == "=>":
                arms += 1
        if arms != 1:
            fail(
                f"platform identity value generator must have exactly one match arm: {arms}",
                issues,
            )
    for invocation in re.finditer(r"identity_value!\s*\{", code):
        body = _rust_balanced_block(code, invocation.end() - 1)
        if body is None:
            fail("platform identity value generator invocation is unterminated", issues)
            continue
        argument = body[1:-1].strip()
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", argument):
            fail(
                "platform identity value generator invocation must pass exactly one kind "
                f"name: {argument[:60]!r}",
                issues,
            )

    # Deliberately not anchored to column 0: an indented or nested invocation defines a real,
    # publicly reachable type just as a top-level one does, so the scan must find it anywhere.
    generator_invocations = len(re.findall(r"identity_value!\s*[({\[]", code))
    if generator_invocations != len(PLATFORM_IDENTITY_KINDS):
        fail(
            "platform identity value-kind count drift: expected "
            f"{len(PLATFORM_IDENTITY_KINDS)} actual={generator_invocations}",
            issues,
        )
    for kind in PLATFORM_IDENTITY_KINDS:
        if not re.search(rf"^\s*{kind}\s*$", code, flags=re.MULTILINE):
            fail(f"platform identity value kind missing: {kind}", issues)
    # The six kinds are macro-generated, so exactly two `pub struct` carriers may exist: the
    # error wrapper and the one inside the generator. A hand-written seventh kind is rejected.
    declared_structs = len(re.findall(r"\bpub struct\b", code))
    if declared_structs != 2:
        fail(
            f"platform identity public struct count drift: expected 2 actual={declared_structs}",
            issues,
        )
    declared_enums = len(re.findall(r"\bpub enum\b", code))
    if declared_enums != 1:
        fail(
            f"platform identity public enum count drift: expected 1 actual={declared_enums}",
            issues,
        )

    compile_fail_cases = sum(1 for line in docs if line == "```compile_fail")
    if compile_fail_cases < MIN_PLATFORM_IDENTITY_COMPILE_FAIL_CASES:
        fail(
            "platform identity compile-fail API proofs shrank: expected>="
            f"{MIN_PLATFORM_IDENTITY_COMPILE_FAIL_CASES} actual={compile_fail_cases}",
            issues,
        )
    for category in PLATFORM_IDENTITY_COMPILE_FAIL_CATEGORIES:
        if category not in docs:
            fail(
                f"platform identity compile-fail category carrier missing: {category!r}",
                issues,
            )
            continue
        position = docs.index(category)
        fence_at = next(
            (
                offset
                for offset, line in enumerate(docs[position + 1 :], start=position + 1)
                if line.startswith("```")
            ),
            None,
        )
        if fence_at is None or docs[fence_at] != "```compile_fail":
            fail(
                "platform identity compile-fail category is not proven by a compile_fail "
                f"fence: {category!r}",
                issues,
            )
            continue
        # A fence proves only that SOMETHING failed to compile. Without pinning the expression,
        # swapping the body for an unrelated type error keeps the fence, the prose and the case
        # count green while the API the category names becomes reachable.
        closing = next(
            (
                offset
                for offset, line in enumerate(docs[fence_at + 1 :], start=fence_at + 1)
                if line.startswith("```")
            ),
            None,
        )
        if closing is None:
            fail(f"platform identity compile-fail block is unterminated: {category!r}", issues)
            continue
        block = "\n".join(docs[fence_at + 1 : closing])
        expression = PLATFORM_IDENTITY_COMPILE_FAIL_EXPRESSIONS[category]
        if expression not in block:
            fail(
                "platform identity compile-fail proof does not exercise the API its category "
                f"denies: {category!r} must contain {expression!r}",
                issues,
            )

    declared_imports = {
        line.strip() for line in code.splitlines() if line.strip().startswith("use ")
    }
    for unadmitted in sorted(declared_imports - PLATFORM_IDENTITY_ALLOWED_IMPORTS):
        fail(f"platform identity module declared an unadmitted import: {unadmitted!r}", issues)
    for carrier in PLATFORM_IDENTITY_FORBIDDEN_CARRIERS:
        if carrier in code:
            fail(
                f"platform identity module gained a forbidden dependency carrier: {carrier!r}",
                issues,
            )
    # The module is exactly one file. `include!` would splice arbitrary public items in from a
    # second file that no scan reads, and a submodule declaration would do the same.
    for label, pattern in PLATFORM_IDENTITY_FORBIDDEN_SPLICE_PATTERNS:
        if re.search(pattern, code):
            fail(
                f"platform identity module must not splice external source: {label!r}",
                issues,
            )
    if re.search(r"^\s*(?:pub\s+)?mod\b", code, flags=re.MULTILINE):
        fail("platform identity module must not declare a submodule", issues)
    if re.search(RUST_INNER_ATTRIBUTE_PATTERN, code):
        fail("platform identity module must not carry an inner attribute", issues)
    check_platform_core_manifest(issues)
    check_cargo_dependency_sources(issues)
    check_rust_lexical_corpus(issues)
    _check_bound_rust_test_file(
        PLATFORM_CAPABILITY_TEST,
        PLATFORM_CAPABILITY_TEST_FUNCTIONS,
        "market capability registry",
        issues,
    )
    # Pin the module graph of every governed source, identity.rs included.
    source_root = ROOT / "crates/platform-core/src"
    for source in sorted(source_root.rglob("*.rs")):
        label = source.relative_to(ROOT).as_posix()
        source_key = source.relative_to(source_root).as_posix()
        governed = strip_rust_comments_and_literals(source.read_text(encoding="utf-8"))
        admitted_modules = PLATFORM_CORE_ADMITTED_MODULE_DECLARATIONS.get(source_key)
        if admitted_modules is None:
            fail(f"ungoverned platform-core source: {label}", issues)
            continue
        declared = tuple(
            sorted(re.findall(r"\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", governed))
        )
        if declared != tuple(sorted(admitted_modules)):
            fail(
                f"platform-core module declarations drifted in {label}: expected "
                f"{tuple(sorted(admitted_modules))} actual={declared}",
                issues,
            )
        if re.search(r"\bcfg_attr\b", governed):
            fail(f"platform-core source must not carry cfg_attr: {label}", issues)
        # Total accounting over attributes, by NORMALIZED name: `#[r#derive(Default)]` is the
        # same attribute as `#[derive(Default)]` and no spelling blacklist can enumerate the
        # equivalents. An unadmitted attribute name is drift whatever it is called.
        source_attributes, unterminated_attributes = rust_attributes(governed)
        if unterminated_attributes:
            fail(
                f"unterminated attribute in {label}: {unterminated_attributes}",
                issues,
            )
        observed_names = rust_attribute_names(source_attributes)
        admitted_attributes = PLATFORM_CORE_ADMITTED_ATTRIBUTE_NAMES.get(source_key)
        if admitted_attributes is None:
            # Fail closed rather than raising: an unregistered source must be reported alongside
            # every other issue, not abort the run and skip the remaining checks.
            fail(f"ungoverned platform-core source attributes: {label}", issues)
            continue
        admitted_names = tuple(sorted(admitted_attributes))
        if observed_names != admitted_names:
            fail(
                f"platform-core attribute names drifted in {label}: expected "
                f"{admitted_names} actual={observed_names}",
                issues,
            )
        # `#` `!` `[` is a token sequence, so the same carrier is rejected the same way in
        # every governed source rather than only in the one file that first needed it.
        if re.search(RUST_INNER_ATTRIBUTE_PATTERN, governed):
            fail(f"platform-core source must not carry an inner attribute: {label}", issues)
        for carrier, pattern in PLATFORM_CORE_FORBIDDEN_SOURCE_PATTERNS:
            if re.search(pattern, governed):
                fail(
                    f"platform-core source must not carry {carrier!r}: {label}",
                    issues,
                )
        # The module NAME pin above says nothing about which file that name is compiled from,
        # and no substring scan can enumerate the spellings of a use tree. Both are settled by
        # accounting for every `mod`/`use`/`type` item with its attribute envelope.
        items, unterminated = rust_item_declarations(governed)
        if unterminated:
            fail(
                f"unterminated platform-core item declaration in {label}: {unterminated}",
                issues,
            )
        admitted_items = list(PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS[source_key])
        if items != admitted_items:
            fail(
                f"platform-core item declarations drifted in {label}: expected "
                f"{admitted_items} actual={items}",
                issues,
            )
        admitted_macros = PLATFORM_CORE_ADMITTED_SIBLING_MACROS[source_key]
        definitions = rust_macro_definitions(governed)
        if definitions != sorted(admitted_macros):
            fail(
                f"platform-core macro definitions drifted in {label}: expected "
                f"{sorted(admitted_macros)} actual={definitions}",
                issues,
            )
        for definition in definitions:
            if definition in RUST_SHADOWABLE_MACRO_NAMES:
                fail(
                    f"platform-core source redefines the standard {definition}! macro: {label}",
                    issues,
                )
        # Invocation NAMES are pinned too: a splicing macro reached by any spelling adds items
        # from a file no scan reads, and no substring can enumerate those spellings.
        invocations, unterminated_macros = rust_macro_invocation_arguments(governed)
        if unterminated_macros:
            fail(
                f"unterminated platform-core macro invocation in {label}: "
                f"{sorted(unterminated_macros)}",
                issues,
            )
        invoked = tuple(sorted({name for name, _ in invocations}))
        admitted_invocations = tuple(sorted(PLATFORM_CORE_ADMITTED_MACRO_INVOCATIONS[source_key]))
        if invoked != admitted_invocations:
            fail(
                f"platform-core macro invocations drifted in {label}: expected "
                f"{admitted_invocations} actual={invoked}",
                issues,
            )
        if source_key != "identity.rs":
            identity_macro_arguments = tuple(
                sorted(
                    (name, argument)
                    for name, argument in invocations
                    if any(
                        re.search(rf"\b{kind}\b", argument)
                        for kind in PLATFORM_IDENTITY_KINDS
                    )
                )
            )
            if source_key == "market/grant.rs":
                admitted_identity_macro_arguments = tuple(
                    sorted(PLATFORM_GRANT_ADMITTED_IDENTITY_MACRO_ARGUMENTS)
                )
                if identity_macro_arguments != admitted_identity_macro_arguments:
                    fail(
                        "market grant identity macro arguments drifted: expected "
                        f"{admitted_identity_macro_arguments} "
                        f"actual={identity_macro_arguments}",
                        issues,
                    )
            elif source_key == "market/update.rs":
                admitted_identity_macro_arguments = tuple(
                    sorted(PLATFORM_UPDATE_ADMITTED_IDENTITY_MACRO_ARGUMENTS)
                )
                if identity_macro_arguments != admitted_identity_macro_arguments:
                    fail(
                        "market update identity macro arguments drifted: expected "
                        f"{admitted_identity_macro_arguments} "
                        f"actual={identity_macro_arguments}",
                        issues,
                    )
            elif source_key == "market/authority.rs":
                admitted_identity_macro_arguments = tuple(
                    sorted(PLATFORM_AUTHORITY_ADMITTED_IDENTITY_MACRO_ARGUMENTS)
                )
                if identity_macro_arguments != admitted_identity_macro_arguments:
                    fail(
                        "market authority identity macro arguments drifted: expected "
                        f"{admitted_identity_macro_arguments} "
                        f"actual={identity_macro_arguments}",
                        issues,
                    )
            else:
                for name, argument in identity_macro_arguments:
                    fail(
                        "platform identity kind passed to a macro outside the M00 "
                        f"identity module: {label}: {name}!({argument})",
                        issues,
                    )

    package_root = ROOT / "crates/platform-core"
    actual_sources = tuple(
        sorted(
            path.relative_to(package_root).as_posix() for path in package_root.rglob("*.rs")
        )
    )
    if actual_sources != tuple(sorted(PLATFORM_CORE_SOURCE_FILES)):
        fail(
            "platform-core source file set drifted: expected "
            f"{tuple(sorted(PLATFORM_CORE_SOURCE_FILES))} actual={actual_sources}",
            issues,
        )
    # A module source need not end in `.rs`, so the `*.rs` inventory above cannot see one. The
    # complete non-fixture inventory is pinned instead; fixtures carry their own digest check.
    admitted_files = set(PLATFORM_CORE_SOURCE_FILES) | set(
        PLATFORM_CORE_ADMITTED_NON_SOURCE_FILES
    )
    actual_files = {
        path.relative_to(package_root).as_posix()
        for path in package_root.rglob("*")
        if path.is_file()
        and not path.relative_to(package_root)
        .as_posix()
        .startswith(PLATFORM_CORE_GOVERNED_FIXTURE_PREFIX)
    }
    if actual_files != admitted_files:
        fail(
            "platform-core package inventory drifted: expected "
            f"{tuple(sorted(admitted_files))} actual={tuple(sorted(actual_files))}",
            issues,
        )

    for kind in PLATFORM_IDENTITY_FORBIDDEN_PUBLIC_ITEM_KINDS:
        if kind in code:
            fail(
                f"platform identity module declared a forbidden public item kind: {kind!r}",
                issues,
            )
    for line in code.splitlines():
        stripped = line.strip()
        if stripped.startswith("pub const ") and not stripped.startswith("pub const fn "):
            fail(f"platform identity module declared a public constant: {stripped!r}", issues)

    public_declarations, unclassified_public = rust_public_declarations(code)
    if unclassified_public:
        fail(
            "platform identity module has an unclassified public declaration: "
            f"{unclassified_public}",
            issues,
        )
    if public_declarations != sorted(PLATFORM_IDENTITY_ADMITTED_PUBLIC_DECLARATIONS):
        fail(
            "platform identity public declaration surface drifted from the admitted "
            f"allowlist: actual={public_declarations}",
            issues,
        )
    impl_declarations, unclassified_impls = rust_impl_declarations(code)
    if unclassified_impls:
        fail(
            f"platform identity module has an unclassified impl declaration: "
            f"{unclassified_impls}",
            issues,
        )
    if impl_declarations != sorted(PLATFORM_IDENTITY_ADMITTED_IMPL_DECLARATIONS):
        fail(
            "platform identity implementation surface drifted from the admitted allowlist: "
            f"actual={impl_declarations}",
            issues,
        )
    derives = sorted(rust_derive_bodies(code))
    if derives != sorted(PLATFORM_IDENTITY_ADMITTED_DERIVES):
        fail(
            f"platform identity derive surface drifted from the admitted allowlist: {derives}",
            issues,
        )

    for function in PLATFORM_IDENTITY_TEST_FUNCTIONS:
        if not re.search(rf"^fn {function}\(\)", test_code, flags=re.MULTILINE):
            fail(f"platform identity acceptance test missing: {function}", issues)
            continue
        # `#[ignore]` or a removed `#[test]` still exits 0 for the registered binding, so the
        # attribute envelope is pinned: the test must actually execute.
        attributes = rust_attribute_block(test_code, function)
        if attributes is None:
            fail(f"platform identity acceptance test unreadable: {function}", issues)
            continue
        # The envelope is pinned exactly, not merely screened for bad markers: any extra
        # attribute on a bound test can change whether it executes.
        if attributes != list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES):
            fail(
                f"platform identity acceptance test {function} attribute envelope drifted: "
                f"expected {list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES)} "
                f"actual={attributes}",
                issues,
            )
        for attribute in attributes:
            for marker in PLATFORM_IDENTITY_FORBIDDEN_TEST_ATTRIBUTE_MARKERS:
                if marker in attribute:
                    fail(
                        f"platform identity acceptance test {function} carries a "
                        f"non-executing attribute: {attribute}",
                        issues,
                    )
    registered_tests = len(re.findall(r"^#\[test\]$", test_code, flags=re.MULTILINE))
    if registered_tests != len(PLATFORM_IDENTITY_TEST_FUNCTIONS):
        fail(
            "platform identity acceptance test registration drift: expected "
            f"{len(PLATFORM_IDENTITY_TEST_FUNCTIONS)} #[test] carriers actual={registered_tests}",
            issues,
        )
    # The envelope guard is carried by a second bound test as well, so ignoring the AUTH-012
    # test alone cannot silence it inside the suite.
    auth011_body = _rust_function_body(
        test_code, "identity_values_enforce_canonical_bounds_and_errors"
    )
    if auth011_body is None or "assert_bound_test_envelope_is_active()" not in auth011_body:
        fail(
            "platform identity identity_values_enforce_canonical_bounds_and_errors lost the "
            "bound-test envelope guard",
            issues,
        )

    for marker, pattern in PLATFORM_IDENTITY_FORBIDDEN_TEST_FILE_PATTERNS:
        if re.search(pattern, test_code):
            fail(
                "platform identity acceptance tests must execute unconditionally: "
                f"{marker!r} is forbidden in {PLATFORM_IDENTITY_TEST}",
                issues,
            )
    # A local `macro_rules! assert_eq` leaves every admitted `assert_eq!` invocation name in
    # place while making the assertion type-check-only, so definitions are pinned as well as
    # invocations, and no definition may shadow a macro this suite's assertions depend on.
    test_definitions = rust_macro_definitions(test_code)
    if test_definitions != sorted(PLATFORM_IDENTITY_ADMITTED_TEST_MACROS):
        fail(
            f"macro definitions drifted in {PLATFORM_IDENTITY_TEST}: expected "
            f"{sorted(PLATFORM_IDENTITY_ADMITTED_TEST_MACROS)} actual={test_definitions}",
            issues,
        )
    for definition in test_definitions:
        if definition in RUST_SHADOWABLE_MACRO_NAMES:
            fail(
                f"{PLATFORM_IDENTITY_TEST} redefines the standard {definition}! macro",
                issues,
            )
    # The admitted helper is pinned as a complete executable carrier: the arm-matcher list must
    # be exactly one `($kind:ty)` arm. One matcher line's PRESENCE says nothing about arm order,
    # so an earlier `($ignored:expr)` arm intercepting every call — with the real arm left
    # present below — would pass a presence check; widening the sole matcher would let one call
    # site forward an arbitrary item exactly as a widened generator matcher would.
    helper_arms = [
        arms
        for name, arms in rust_macro_arms(test_code)
        if name == PLATFORM_IDENTITY_TEST_HELPER_MACRO
    ]
    if helper_arms != [list(PLATFORM_IDENTITY_TEST_HELPER_ARMS)]:
        fail(
            f"admitted test helper macro arms drifted in {PLATFORM_IDENTITY_TEST}: expected "
            f"[{list(PLATFORM_IDENTITY_TEST_HELPER_ARMS)}] actual={helper_arms}",
            issues,
        )
    # Pinning the arm shape does not stop the sole arm's body being gutted to a no-op while
    # production is broken, so the grammar oracle's load-bearing checks are pinned too.
    helper_body = _rust_macro_body(test_code, PLATFORM_IDENTITY_TEST_HELPER_MACRO)
    if helper_body is None:
        fail(
            f"admitted test helper macro body unreadable in {PLATFORM_IDENTITY_TEST}",
            issues,
        )
    else:
        for carrier in PLATFORM_IDENTITY_HELPER_BODY_CARRIERS:
            if carrier not in helper_body:
                fail(
                    f"admitted test helper macro lost a grammar-oracle carrier in "
                    f"{PLATFORM_IDENTITY_TEST}: {carrier!r}",
                    issues,
                )
    # Total accounting over the bound test file's `use`/`type`/`mod` items, the same rule the
    # governed sources carry. A block-local `use std::assert as assert_eq;` — dropped after any
    # in-suite guard has already run — rebinds `assert_eq!` for the rest of its scope while the
    # invocation NAME set is unchanged, so nothing but item accounting sees it.
    # The bound test file carries exactly one attribute name: `test`. `#[r#ignore]` normalizes to
    # `ignore` and is rejected here even though it contains no `#[ignore]` substring.
    test_attributes, unterminated_test_attributes = rust_attributes(test_code)
    if unterminated_test_attributes:
        fail(
            f"unterminated attribute in {PLATFORM_IDENTITY_TEST}: "
            f"{unterminated_test_attributes}",
            issues,
        )
    observed_test_attribute_names = rust_attribute_names(test_attributes)
    admitted_test_attribute_names = tuple(
        sorted(PLATFORM_IDENTITY_ADMITTED_TEST_ATTRIBUTE_NAMES)
    )
    if observed_test_attribute_names != admitted_test_attribute_names:
        fail(
            f"bound test attribute names drifted in {PLATFORM_IDENTITY_TEST}: expected "
            f"{admitted_test_attribute_names} actual={observed_test_attribute_names}",
            issues,
        )
    test_items, unterminated_test_items = rust_item_declarations(test_code)
    if unterminated_test_items:
        fail(
            f"unterminated item declaration in {PLATFORM_IDENTITY_TEST}: "
            f"{unterminated_test_items}",
            issues,
        )
    if test_items != list(PLATFORM_IDENTITY_ADMITTED_TEST_ITEMS):
        fail(
            f"bound test item declarations drifted in {PLATFORM_IDENTITY_TEST}: expected "
            f"{list(PLATFORM_IDENTITY_ADMITTED_TEST_ITEMS)} actual={test_items}",
            issues,
        )
    test_invocations, unterminated_test_macros = rust_macro_invocation_arguments(test_code)
    if unterminated_test_macros:
        fail(
            f"unterminated macro invocation in {PLATFORM_IDENTITY_TEST}: "
            f"{sorted(unterminated_test_macros)}",
            issues,
        )
    invoked_in_test = tuple(sorted({name for name, _ in test_invocations}))
    admitted_in_test = tuple(sorted(PLATFORM_IDENTITY_ADMITTED_TEST_MACRO_INVOCATIONS))
    if invoked_in_test != admitted_in_test:
        fail(
            f"macro invocations drifted in {PLATFORM_IDENTITY_TEST}: expected "
            f"{admitted_in_test} actual={invoked_in_test}",
            issues,
        )

    # The AUTH-011 body, same rule: a named test whose body was emptied keeps its exact binding
    # green. Its exhaustive-grammar guard is pinned here and its own body pinned below, because
    # it is the only carrier that reaches the bytes inside a literal.
    auth011_body = _rust_function_body(test_code, PLATFORM_IDENTITY_AUTH011_TEST)
    if auth011_body is None:
        fail(
            f"platform identity acceptance test body unreadable: "
            f"{PLATFORM_IDENTITY_AUTH011_TEST}",
            issues,
        )
    else:
        for carrier in PLATFORM_IDENTITY_AUTH011_BODY_CARRIERS:
            if carrier not in " ".join(auth011_body.split()):
                fail(
                    f"platform identity {PLATFORM_IDENTITY_AUTH011_TEST} lost an essential "
                    f"evidence carrier: {carrier!r}",
                    issues,
                )
    oracle_body = _rust_function_body(test_code, PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE)
    if oracle_body is None:
        fail(
            f"platform identity exhaustive grammar oracle missing: "
            f"{PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE}",
            issues,
        )
    else:
        for carrier in PLATFORM_IDENTITY_EXHAUSTIVE_ORACLE_CARRIERS:
            if carrier not in " ".join(oracle_body.split()):
                fail(
                    "platform identity exhaustive grammar oracle lost a carrier: "
                    f"{carrier!r}",
                    issues,
                )

    # A named test whose body was emptied keeps its exact binding green, so pin the essential
    # assertion carriers of the AUTH-012 test rather than only its name.
    auth012_body = _rust_function_body(test_code, PLATFORM_IDENTITY_AUTH012_TEST)
    if auth012_body is None:
        fail(
            f"platform identity acceptance test body unreadable: "
            f"{PLATFORM_IDENTITY_AUTH012_TEST}",
            issues,
        )
    else:
        for carrier in PLATFORM_IDENTITY_AUTH012_BODY_CARRIERS:
            if carrier not in auth012_body:
                fail(
                    f"platform identity {PLATFORM_IDENTITY_AUTH012_TEST} lost an essential "
                    f"evidence carrier: {carrier!r}",
                    issues,
                )
        assertions = auth012_body.count("assert")
        if assertions < MIN_PLATFORM_IDENTITY_AUTH012_ASSERTIONS:
            fail(
                f"platform identity {PLATFORM_IDENTITY_AUTH012_TEST} assertion count "
                f"collapsed: expected>={MIN_PLATFORM_IDENTITY_AUTH012_ASSERTIONS} "
                f"actual={assertions}",
                issues,
            )

    # Unanchored and delimiter-agnostic on purpose. An indented, nested or brace-delimited
    # invocation such as `pub mod compat { authority_id!(TenantId); }` compiles and exposes a
    # second publicly reachable tenant type, which contract §6 calls an incomplete M00-B1.
    for kind in ("TenantId", "UserId"):
        if re.search(rf"authority_id!\s*[({{\[]\s*{kind}\s*[)}}\]]", invocation_code):
            fail(
                f"invocation authority reintroduced a local {kind} definition; "
                "M00 owns tenant/user identity",
                issues,
            )
    # Structurally independent second guard: a hand-written duplicate anywhere else in the
    # crate is caught even though it never touches the `authority_id!` generator.
    for path in sorted((ROOT / "crates/platform-core/src").rglob("*.rs")):
        if path.resolve() == source_path.resolve():
            continue
        sibling_code = strip_rust_comments_and_literals(path.read_text(encoding="utf-8"))
        for kind in ("TenantId", "UserId"):
            if re.search(rf"\bstruct\s+{kind}\b", sibling_code):
                fail(
                    f"duplicate {kind} definition outside the M00 identity module: "
                    f"{path.relative_to(ROOT).as_posix()}",
                    issues,
                )
        # The frozen surface belongs to the value kinds, not to one file: an implementation
        # written in a sibling module adds exactly the same externally reachable API. The
        # sibling `impl` surface is an allowlist, not a kind blacklist, because a blanket
        # `impl<T> Extension for T` names no kind and covers all six.
        sibling_impls, sibling_unclassified = rust_impl_declarations(sibling_code)
        if sibling_unclassified:
            fail(
                "unclassified impl declaration in "
                f"{path.relative_to(ROOT).as_posix()}: {sibling_unclassified}",
                issues,
            )
        admitted_impls = PLATFORM_CORE_ADMITTED_SIBLING_IMPLS.get(path.name)
        if admitted_impls is None:
            fail(
                f"ungoverned platform-core sibling: {path.relative_to(ROOT).as_posix()}",
                issues,
            )
        elif sibling_impls != sorted(admitted_impls):
            fail(
                "platform-core sibling implementation surface drifted in "
                f"{path.relative_to(ROOT).as_posix()}: actual={sibling_impls}",
                issues,
            )
        # Every `impl` token, whatever its line position: inherent impls carry no `for`, Rust's
        # orphan rule does not stop a second inherent block from another file in the same
        # crate, and a `where` clause follows the self type rather than belonging to it.
        for target in rust_impl_self_types(sibling_code):
            if target.rsplit("::", 1)[-1] in PLATFORM_IDENTITY_KINDS:
                fail(
                    "platform identity value implementation outside the M00 identity module: "
                    f"{path.relative_to(ROOT).as_posix()}: impl … {target}",
                    issues,
                )
        # Every `use`/`type` binding of an admitted kind is rejected, private ones included,
        # rather than resolved. A local alias does not change Rust's self type, so
        # `use crate::identity::TenantId as Tenant; impl AsRef<str> for Tenant { .. }` is a
        # real implementation for the governed type while every textual comparison sees
        # `Tenant`. Refusing to create the alias removes the thing that would need resolving.
        for statement in re.finditer(r"\b(?:pub\s+)?(?:use|type)\b[^;]*;", sibling_code):
            normalized = " ".join(statement.group(0).split())
            mentions_kind = any(
                re.search(rf"\b{kind}\b", normalized) for kind in PLATFORM_IDENTITY_KINDS
            )
            # A whole-module re-export (`pub use crate::identity as identity_alias;`) names no
            # kind yet publishes every one of them under a second path, so the module path is
            # governed exactly like the type names.
            mentions_module = re.search(r"\b(?:crate|self|super)::identity\b", normalized)
            relative = path.relative_to(ROOT).as_posix()
            admitted = any(
                relative == admitted_file and normalized == admitted_text
                for admitted_file, admitted_text in (
                    PLATFORM_IDENTITY_ADMITTED_CROSS_FILE_BINDINGS
                )
            )
            if (mentions_kind or mentions_module) and not admitted:
                fail(
                    "platform identity value alias or import outside the M00 identity module: "
                    f"{path.relative_to(ROOT).as_posix()}: {normalized}",
                    issues,
                )
        for carrier, pattern in PLATFORM_CORE_FORBIDDEN_SPLICE_PATTERNS:
            if re.search(pattern, sibling_code):
                fail(
                    "platform-core source must not splice external source: "
                    f"{path.relative_to(ROOT).as_posix()}: {carrier!r}",
                    issues,
                )
    reexported: set[str] = set()
    for match in re.finditer(
        r"pub use crate::identity::(?:\{([^}]*)\}|([A-Za-z_][A-Za-z0-9_]*))\s*;",
        invocation_code,
    ):
        names = match.group(1) or match.group(2) or ""
        reexported.update(name.strip() for name in names.split(",") if name.strip())
    for kind in ("TenantId", "UserId"):
        if kind not in reexported:
            fail(
                f"invocation authority must publicly re-export the M00 {kind} definition",
                issues,
            )
    if not re.search(
        r"authority_id!\s*[({\[]\s*PolicySnapshotId\s*[)}\]]", invocation_code
    ):
        fail(
            "invocation PolicySnapshotId must remain M20-owned, unrenamed and unmigrated",
            issues,
        )
    if "PolicySnapshotId" in reexported:
        fail(
            "invocation PolicySnapshotId must not alias a platform identity value",
            issues,
        )

    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    if not matrix_path.is_file():
        fail("platform identity acceptance source is missing", issues)
        return
    matrix_rows = {
        row.split("\t")[0]: row.split("\t")
        for row in matrix_path.read_text(encoding="utf-8").splitlines()[1:]
        if row.strip()
    }
    for case_id, binding in PLATFORM_IDENTITY_ACCEPTANCE_BINDINGS.items():
        row = matrix_rows.get(case_id)
        if row is None or len(row) != 7:
            fail(f"platform identity acceptance row missing: {case_id}", issues)
            continue
        if row[3] != binding:
            fail(f"platform identity acceptance binding drift in {case_id}", issues)
        if row[5] != "implemented":
            fail(
                f"platform identity acceptance status drift in {case_id}: {row[5]!r}",
                issues,
            )

    checker_path = ROOT / "scripts/check_repo_contracts.py"
    if not checker_path.is_file():
        fail("platform identity carrier missing: scripts/check_repo_contracts.py", issues)
        return
    main_body = checker_path.read_text(encoding="utf-8").split("\ndef main() -> int:", 1)
    for required in (
        "check_platform_identity_implementation(issues)",
        "check_platform_authority_implementation(issues)",
        # The semantic authority chain is worthless as a module-level function nobody calls.
        "check_platform_identity_grammar_authority(issues)",
    ):
        invoked = len(main_body) == 2 and any(
            line.strip() == required for line in main_body[1].splitlines()
        )
        if not invoked:
            fail(
                f"{required.split('(')[0]} must be invoked from repository main()",
                issues,
            )


PLATFORM_SESSION_CONTRACT = "docs/contracts/platform-session.md"
PLATFORM_SESSION_VERSION = "`Version`: `platform-session/v0`"
PLATFORM_SESSION_DOMAIN = "platform-session"
PLATFORM_SESSION_GATE = "pr"
PLATFORM_SESSION_CASES = ("AUTH-017", "AUTH-018", "AUTH-019", "AUTH-020")
# Every binding runs the repository checker before its Rust leg. `platform-identity/v0` §4 gives
# the reason: a redirected `[[test]]` target or a renamed function makes `--exact` match nothing,
# which cargo reports as `running 0 tests` at exit zero, and a guard written inside the suite is
# exactly what such a change replaces. Only an out-of-band carrier detects that.
PLATFORM_SESSION_BINDING_PREFIX = "python3 scripts/check_repo_contracts.py && "
PLATFORM_SESSION_BINDING_ROW = re.compile(
    r"^\| `(?P<case>AUTH-[0-9]{3})` \| `(?P<binding>[^`]+)` \|$", flags=re.MULTILINE
)
PLATFORM_SESSION_CATALOG_ROW = re.compile(
    r"^\| `(?P<case>AUTH-[0-9]{3})` \| (?P<assertion>.+?) \| ", flags=re.MULTILINE
)
# The bound function name inside a binding. Existence of the test FILE is not evidence that the
# named function survives in it: `--exact` against a renamed or deleted function is `running 0
# tests` at exit zero, which is the whole reason each binding runs this checker first. So a row
# may claim `implemented` only when the function it names is still present.
PLATFORM_SESSION_BOUND_FUNCTION = re.compile(
    r"--test platform_session (?P<function>[A-Za-z0-9_]+) -- --exact"
)


def check_platform_session_contract(issues: list[str]) -> None:
    """Bind `platform-session/v0` to the active acceptance matrix in both directions.

    Neither direction implies the other, so both are checked.

    Downward, the contract is the authority: its §12 binding table specifies which test
    proves which case, so an active row whose binding differs from that table is drift —
    a row silently repointed at a test the contract never specified would otherwise gate
    nothing the contract asked for.

    Upward, a `planned` row may not be promoted from documentation alone. `AUTH-017..020`
    may read `implemented` only once BOTH carriers the contract names actually exist. The
    implementation does not exist yet, so today this rule pins all four to `planned`; it
    keeps holding after B2 lands, because deleting the suite while the row still claims
    `implemented` fails here rather than passing quietly.
    """
    contract_path = ROOT / PLATFORM_SESSION_CONTRACT
    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    catalog_path = ROOT / "docs/acceptance/platform-baseline.md"
    if not contract_path.is_file():
        fail(f"platform session contract missing: {PLATFORM_SESSION_CONTRACT}", issues)
        return
    if not matrix_path.is_file() or not catalog_path.is_file():
        fail("platform session acceptance sources are missing", issues)
        return
    contract = contract_path.read_text(encoding="utf-8")

    for required in (PLATFORM_SESSION_VERSION, PLATFORM_SESSION_SOURCE, PLATFORM_SESSION_TEST):
        if required not in contract:
            fail(f"platform session contract carrier missing: {required!r}", issues)

    binding_rows = list(PLATFORM_SESSION_BINDING_ROW.finditer(contract))
    specified = {match.group("case"): match.group("binding") for match in binding_rows}
    if set(specified) != set(PLATFORM_SESSION_CASES):
        fail(
            "platform session contract binding table drift: "
            f"expected={sorted(PLATFORM_SESSION_CASES)} actual={sorted(specified)}",
            issues,
        )
    # A dict comprehension keeps the LAST match, so a second row for the same case anywhere in
    # the document would silently shadow the real table and never be reported. One row per case,
    # counted, closes that: the shadowing row is drift whichever side of the table it sits on.
    if len(binding_rows) != len(PLATFORM_SESSION_CASES):
        fail(
            "platform session contract binding table has duplicate or stray rows: "
            f"expected={len(PLATFORM_SESSION_CASES)} actual={len(binding_rows)}",
            issues,
        )

    matrix_rows = {
        row.split("\t")[0]: row.split("\t")
        for row in matrix_path.read_text(encoding="utf-8").splitlines()[1:]
        if row.strip()
    }
    catalog_assertions = {
        match.group("case"): match.group("assertion")
        for match in PLATFORM_SESSION_CATALOG_ROW.finditer(
            catalog_path.read_text(encoding="utf-8")
        )
    }
    source_path = ROOT / PLATFORM_SESSION_SOURCE
    test_path = ROOT / PLATFORM_SESSION_TEST
    carriers_exist = source_path.is_file() and test_path.is_file()
    test_source = test_path.read_text(encoding="utf-8") if test_path.is_file() else ""
    source_text = source_path.read_text(encoding="utf-8") if source_path.is_file() else ""

    for case_id in PLATFORM_SESSION_CASES:
        row = matrix_rows.get(case_id)
        if row is None or len(row) != 7:
            fail(f"platform session acceptance row missing: {case_id}", issues)
            continue
        if row[1] != PLATFORM_SESSION_DOMAIN:
            fail(
                f"platform session acceptance domain drift in {case_id}: {row[1]!r}",
                issues,
            )
        if case_id not in catalog_assertions:
            fail(
                f"platform session acceptance case missing from long-horizon catalog: "
                f"{case_id}",
                issues,
            )
        elif row[2] != catalog_assertions[case_id]:
            fail(
                f"platform session acceptance assertion drift between matrix and "
                f"catalog in {case_id}",
                issues,
            )
        expected_binding = specified.get(case_id)
        if expected_binding is not None and row[3] != expected_binding:
            fail(
                f"platform session acceptance binding drift in {case_id}: "
                "matrix row does not equal the contract §12 binding table",
                issues,
            )
        if not row[3].startswith(PLATFORM_SESSION_BINDING_PREFIX):
            fail(
                f"platform session acceptance binding in {case_id} must run the "
                "repository checker before its Rust leg",
                issues,
            )
        if row[4] != PLATFORM_SESSION_GATE:
            fail(
                f"platform session acceptance gate drift in {case_id}: {row[4]!r}",
                issues,
            )
        expected_lib = PLATFORM_SESSION_LIB_BINDING_CASES.get(case_id)
        lib_bound = PLATFORM_SESSION_LIB_BINDING.search(row[3])
        if expected_lib is None:
            if lib_bound is not None:
                fail(
                    f"platform session acceptance binding in {case_id} carries an unexpected "
                    "library leg",
                    issues,
                )
        elif lib_bound is None or lib_bound.group("function") != expected_lib:
            fail(
                f"platform session acceptance binding in {case_id} must name the exact library "
                f"fixture {expected_lib}",
                issues,
            )
        bound = PLATFORM_SESSION_BOUND_FUNCTION.search(row[3])
        if bound is None:
            fail(
                f"platform session acceptance binding in {case_id} names no exact "
                "platform_session test function",
                issues,
            )
        if row[5] == "implemented":
            if not carriers_exist:
                fail(
                    f"platform session acceptance status in {case_id} claims "
                    f"'implemented' while {PLATFORM_SESSION_SOURCE} or "
                    f"{PLATFORM_SESSION_TEST} is absent",
                    issues,
                )
            elif bound is not None and not re.search(
                rf"\bfn\s+{re.escape(bound.group('function'))}\b", test_source
            ):
                fail(
                    f"platform session acceptance status in {case_id} claims "
                    f"'implemented' while {PLATFORM_SESSION_TEST} declares no "
                    f"{bound.group('function')}",
                    issues,
                )
            elif lib_bound is not None and not re.search(
                rf"\bfn\s+{re.escape(lib_bound.group('function'))}\b", source_text
            ):
                fail(
                    f"platform session acceptance status in {case_id} claims "
                    f"'implemented' while {PLATFORM_SESSION_SOURCE} declares no "
                    f"{lib_bound.group('function')}",
                    issues,
                )

    checker_path = ROOT / "scripts/check_repo_contracts.py"
    if not checker_path.is_file():
        fail("platform session carrier missing: scripts/check_repo_contracts.py", issues)
        return
    main_body = checker_path.read_text(encoding="utf-8").split("\ndef main() -> int:", 1)
    required_call = "check_platform_session_contract(issues)"
    invoked = len(main_body) == 2 and any(
        line.strip() == required_call for line in main_body[1].splitlines()
    )
    if not invoked:
        fail("check_platform_session_contract must be invoked from repository main()", issues)



# `platform-session/v0` §11.2 carries the ROOT of B1's closure, not the whole apparatus: the two
# grammars exist as fenced normative carriers in the contract, and the implementation's admitted
# byte classes, length bound and digest shape are extracted from source and cross-checked against
# them. Frozen per-function body fingerprints, construction-expression counting and a second lexer
# with a corpus differential are deliberately NOT required — a defect in `AuthAdapterId`'s grammar
# admits a malformed adapter label into one session's provenance record, while a defect in B1's
# mints identities repository-wide. The session values whose failure would matter — revision,
# deadline, terminal precedence, replay equality — are closed by executable adversarial fixtures
# under the four bound tests instead, because those are behavioural properties a fixture can
# actually falsify.
PLATFORM_SESSION_ADAPTER_REGEX = "^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$"
PLATFORM_SESSION_ADAPTER_INTERIOR_EXTRA = "-._:"
PLATFORM_SESSION_DIGEST_REGEX = "^sha256:[0-9a-f]{64}$"
# A structural parse, not a substring comparison: an algorithm prefix that changed, a byte class
# that gained a member or a digit count that moved are each a different parse.
PLATFORM_SESSION_DIGEST_SHAPE = re.compile(
    r"\A\^(?P<prefix>[A-Za-z0-9]+:)\[(?P<class>[^\]]+)\]\{(?P<digits>\d+)\}\$\Z"
)
PLATFORM_SESSION_LENGTH_CONSTANT = "MAX_ADAPTER_ID_BYTES"
PLATFORM_SESSION_DIGEST_DIGITS_CONSTANT = "DIGEST_HEX_DIGITS"
PLATFORM_SESSION_DIGEST_PREFIX_DECLARATION = re.compile(
    r'const DIGEST_PREFIX: &str = "(?P<prefix>[^"]*)";'
)
PLATFORM_SESSION_BOUNDARY_FUNCTION = "is_adapter_boundary_byte"
PLATFORM_SESSION_INTERIOR_FUNCTION = "is_adapter_interior_byte"
PLATFORM_SESSION_DIGEST_FUNCTION = "classify_digest"
PLATFORM_SESSION_BOUNDARY_PREDICATE = "byte.is_ascii_alphanumeric()"
PLATFORM_SESSION_DIGEST_CLASS_CARRIER = "matches!(byte, b'0'..=b'9' | b'a'..=b'f')"
# §11 forbids these by CATEGORY; two of them are concrete in-repository names because
# `platform-core` already depends on them for M20 work. "Or reference" is the load-bearing half:
# `semver::Version::parse(...)` written inside a function body compiles, declares no item, and is
# therefore invisible to the item allowlist — so this is a per-file carrier scan, not item
# accounting. `Instant` is deliberately absent from the list: it is a substring of this module's
# own `SessionInstant`, so the clock carriers are spelled in the forms that would actually import
# or call one.
PLATFORM_SESSION_FORBIDDEN_CARRIERS = (
    "ustc_agent_tool_protocol",
    "semver",
    "uuid",
    "Uuid",
    "ulid",
    "Ulid",
    "nanoid",
    "NanoId",
    "rand",
    "Rng",
    "random",
    "generate",
    "mint",
    "SystemTime",
    "Instant::now",
    "chrono",
    "std::time",
    "std::net",
    "TcpStream",
    "reqwest",
    "hyper",
    "std::fs",
    "std::process",
    "sqlx",
    "diesel",
    "rusqlite",
    "axum",
    "dioxus",
    "cookie",
    "Cookie",
    "oauth",
    "OAuth",
    "oidc",
    "Oidc",
    "sha2::",
    "Sha256",
    "hmac",
    "Hmac",
    "hex::",
    "std::env",
    "std::io",
)
# B2 computes no digest. The admitted "lightweight digest-shape validation" is byte-class
# validation of an already-supplied string, not a cryptographic dependency.
PLATFORM_SESSION_FORBIDDEN_DIGEST_CARRIERS = (
    ("hashing function", r"\bfn\s+hash\b"),
    ("digest call", r"::digest\s*\("),
    ("streaming update", r"\.update\s*\("),
    ("streaming finalize", r"\.finalize\s*\("),
)
PLATFORM_SESSION_FORBIDDEN_PUBLIC_ITEM_KINDS = (
    "pub type",
    "pub use",
    "pub mod",
    "pub trait",
    "pub static",
    "pub union",
    "pub macro",
)
# The private same-module fixtures §13 binds to the library legs of `AUTH-018` and `AUTH-019`.
# They exist because `SessionSnapshot` at `revision == u64::MAX` is unreachable from an
# integration test — no public constructor, no `Deserialize`, and `evolve` only ever increments by
# one — while §7 item 5 and §8's checked increment both have observable behaviour there. They add
# no public or feature-gated hook: the module is `#[cfg(test)]`-gated and the public-declaration
# allowlist above is unchanged by their presence, so a `pub` on any of them fails.
PLATFORM_SESSION_TEST_MODULE = "tests"
PLATFORM_SESSION_LIB_TEST_FUNCTIONS = (
    "terminal_precedence_holds_at_the_revision_ceiling",
    "revision_ceiling_fails_closed_on_decide_and_evolve",
)
PLATFORM_SESSION_LIB_BINDING = re.compile(
    r"--lib session::tests::(?P<function>[A-Za-z0-9_]+) -- --exact"
)
# Which case each library leg belongs to, so a leg cannot be moved to another row unnoticed.
PLATFORM_SESSION_LIB_BINDING_CASES = {
    "AUTH-018": "terminal_precedence_holds_at_the_revision_ceiling",
    "AUTH-019": "revision_ceiling_fails_closed_on_decide_and_evolve",
}
PLATFORM_SESSION_TEST_FUNCTIONS = (
    "session_open_pins_immutable_scope_and_checked_deadlines",
    "session_lifecycle_precedence_is_deterministic_and_terminal",
    "session_revision_and_replay_are_exact_and_fail_closed",
    "session_domain_has_no_credential_or_adapter_surface",
)
PLATFORM_SESSION_ADMITTED_TEST_ATTRIBUTE_NAMES = ("test",)
PLATFORM_SESSION_ADMITTED_PUBLIC_DECLARATIONS = (
    'pub const fn absolute_expires_at',
    'pub const fn absolute_timeout',
    'pub const fn absolute_timeout',
    'pub const fn admits_at',
    'pub const fn as_millis',
    'pub const fn as_unix_millis',
    'pub const fn auth_adapter_id',
    'pub const fn auth_adapter_id',
    'pub const fn authenticated_at',
    'pub const fn authenticated_at',
    'pub const fn cause',
    'pub const fn credential_evidence',
    'pub const fn credential_evidence',
    'pub const fn credential_not_after',
    'pub const fn credential_not_after',
    'pub const fn effective_expires_at',
    'pub const fn effective_expires_at',
    'pub const fn evidence_digest',
    'pub const fn evidence_digest',
    'pub const fn expected_revision',
    'pub const fn expected_revision',
    'pub const fn expected_revision',
    'pub const fn expected_revision',
    'pub const fn expected_revision',
    'pub const fn expired_at',
    'pub const fn from_millis',
    'pub const fn from_unix_millis',
    'pub const fn idle_timeout',
    'pub const fn idle_timeout',
    'pub const fn kind',
    'pub const fn last_transition_at',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn new',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn observed_at',
    'pub const fn opened_at',
    'pub const fn opened_at',
    'pub const fn policy',
    'pub const fn policy',
    'pub const fn revision',
    'pub const fn sequence',
    'pub const fn sequence',
    'pub const fn sequence',
    'pub const fn sequence',
    'pub const fn sequence',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn session_id',
    'pub const fn status',
    'pub const fn tenant_id',
    'pub const fn tenant_id',
    'pub const fn user_id',
    'pub const fn user_id',
    'pub const fn value_kind',
    'pub enum EventDerivedField',
    'pub enum SessionCommand',
    'pub enum SessionDomainError',
    'pub enum SessionEvent',
    'pub enum SessionExpiryCause',
    'pub enum SessionStatus',
    'pub enum SessionValueErrorKind',
    'pub fn as_str',
    'pub fn as_str',
    'pub fn decide',
    'pub fn evolve',
    'pub fn new',
    'pub fn parse',
    'pub fn parse',
    'pub struct AuthAdapterId',
    'pub struct CredentialEvidenceDigest',
    'pub struct ExpireSession',
    'pub struct OpenSession',
    'pub struct RefreshSession',
    'pub struct RevokeSession',
    'pub struct SessionCredentialEvidence',
    'pub struct SessionDuration',
    'pub struct SessionExpired',
    'pub struct SessionInstant',
    'pub struct SessionOpened',
    'pub struct SessionPolicy',
    'pub struct SessionRefreshed',
    'pub struct SessionRevoked',
    'pub struct SessionSnapshot',
    'pub struct SessionValueError',
)
PLATFORM_SESSION_ADMITTED_DERIVES = (
    'Clone, PartialEq, Eq, PartialOrd, Ord, Serialize',
    'Debug, Clone, Copy, PartialEq, Eq',
    'Debug, Clone, Copy, PartialEq, Eq',
    'Debug, Clone, Copy, PartialEq, Eq',
    'Debug, Clone, Copy, PartialEq, Eq',
    'Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize',
    'Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize',
    'Debug, Clone, Copy, PartialEq, Eq, Serialize',
    'Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize',
    'Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize',
    'Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize',
    'Debug, Clone, PartialEq, Eq, Serialize, Deserialize',
    'Debug, Clone, PartialEq, Eq, Serialize, Deserialize',
    'Debug, Clone, PartialEq, Eq, Serialize, Deserialize',
    'Debug, Clone, PartialEq, Eq, Serialize, Deserialize',
    'Debug, Clone, PartialEq, Eq, Serialize, Deserialize',
    'Deserialize',
)
PLATFORM_SESSION_ADMITTED_TEST_ITEMS = (
    'use ustc_campus_agent_core::identity::{SessionId, TenantId, UserId};',
    'use ustc_campus_agent_core::session::{ AuthAdapterId, CredentialEvidenceDigest, EventDerivedField, ExpireSession, OpenSession, RefreshSession, RevokeSession, SessionCommand, SessionCredentialEvidence, SessionDomainError, SessionDuration, SessionEvent, SessionExpired, SessionExpiryCause, SessionInstant, SessionOpened, SessionPolicy, SessionRefreshed, SessionRevoked, SessionSnapshot, SessionStatus, SessionValueErrorKind, decide, evolve, };',
)
PLATFORM_SESSION_ADMITTED_TEST_MACRO_INVOCATIONS = (
    'assert',
    'assert_eq',
    'concat',
    'format',
    'include_str',
    'panic',
    'vec',
)


def check_platform_session_implementation(issues: list[str]) -> None:
    """Freeze the `M00-B2` session module's surface, grammars and bound evidence.

    `check_platform_session_contract` binds the contract's §12 table to the matrix. This binds
    the contract to the CODE: the two §2.2 grammars, the public declaration and derive surface,
    the §11 dependency negative space, and the bound test file's own executable envelope.

    The module joins `platform-identity/v0` §4's total accounting on the same terms through the
    shared `PLATFORM_CORE_*` tables above, which is where its file inventory, `mod`/`use`/`type`
    items, `impl` surface, macros and attribute names are frozen. What is here is what those
    shared tables cannot express: a per-file forbidden-carrier scan, because a path-qualified call
    inside a function body declares no item; and the grammar cross-check §11.2 requires.
    """
    source_path = ROOT / PLATFORM_SESSION_SOURCE
    test_path = ROOT / PLATFORM_SESSION_TEST
    contract_path = ROOT / PLATFORM_SESSION_CONTRACT
    lib_path = ROOT / PLATFORM_CORE_LIB
    for rel, path in (
        (PLATFORM_SESSION_SOURCE, source_path),
        (PLATFORM_SESSION_TEST, test_path),
        (PLATFORM_SESSION_CONTRACT, contract_path),
        (PLATFORM_CORE_LIB, lib_path),
    ):
        if not path.is_file():
            fail(f"platform session carrier missing: {rel}", issues)
            return

    source = source_path.read_text(encoding="utf-8")
    code = strip_rust_comments_and_literals(source)
    literal_code = strip_rust_comments_and_literals(source, keep_literals=True)
    test_code = strip_rust_comments_and_literals(test_path.read_text(encoding="utf-8"))
    contract = contract_path.read_text(encoding="utf-8")
    lib_code = strip_rust_comments_and_literals(lib_path.read_text(encoding="utf-8"))

    if not re.search(r"^\s*pub mod session;$", lib_code, flags=re.MULTILINE):
        fail("platform-core must export the M00 session module", issues)

    # The enumerated cross-file identity binding is required POSITIVELY here, not only admitted
    # by the exception in `check_platform_identity_implementation`. An admitted binding that was
    # deleted, renamed or re-spelled would otherwise pass both carriers by simply not appearing.
    if PLATFORM_SESSION_ADMITTED_IDENTITY_IMPORT not in " ".join(code.split()):
        fail(
            "platform session module lost the enumerated identity binding: expected "
            f"{PLATFORM_SESSION_ADMITTED_IDENTITY_IMPORT!r}",
            issues,
        )

    # Both §2.2 grammars exist as fenced normative carriers in the contract, and the contract's
    # own regex — not a checker-local copy — is what the source is compared against.
    for regex in (PLATFORM_SESSION_ADAPTER_REGEX, PLATFORM_SESSION_DIGEST_REGEX):
        if f"```regex\n{regex}\n```" not in contract:
            fail(
                f"platform session contract lost a fenced normative grammar: {regex!r}",
                issues,
            )
            return

    shape = PLATFORM_IDENTITY_GRAMMAR_SHAPE.match(PLATFORM_SESSION_ADAPTER_REGEX)
    if shape is None:
        fail("platform session adapter grammar is not a parseable shape", issues)
        return
    if shape.group("lead") != shape.group("tail"):
        fail(
            "platform session adapter grammar: leading and trailing boundary classes differ "
            f"({shape.group('lead')!r} vs {shape.group('tail')!r})",
            issues,
        )
    boundary, boundary_duplicate = rust_character_class_bytes(shape.group("lead"))
    interior, interior_duplicate = rust_character_class_bytes(shape.group("interior"))
    if boundary_duplicate or interior_duplicate:
        fail("platform session adapter grammar repeats a character-class byte", issues)
    if frozenset(boundary) != ASCII_ALPHANUMERIC_BYTES:
        fail(
            "platform session adapter grammar boundary class is not ASCII alphanumeric",
            issues,
        )
    if frozenset(interior) - ASCII_ALPHANUMERIC_BYTES != frozenset(interior) - frozenset(
        boundary
    ):
        fail(
            "platform session adapter grammar interior class does not extend the boundary class",
            issues,
        )
    # The delimiters are what the source predicate actually enumerates; the alphanumerics reach
    # it through `is_ascii_alphanumeric()` and are checked above rather than listed twice.
    interior_extra = tuple(sorted(frozenset(interior) - ASCII_ALPHANUMERIC_BYTES))
    expected_max_bytes = int(shape.group("bound")) + 2
    declared_bound = rust_module_level_usize_constants(code, PLATFORM_SESSION_LENGTH_CONSTANT)
    if declared_bound != (expected_max_bytes,):
        fail(
            f"platform session adapter length bound drifted: {PLATFORM_SESSION_LENGTH_CONSTANT} "
            f"is {list(declared_bound)}, contract regex implies {expected_max_bytes}",
            issues,
        )

    boundary_body = _rust_function_body(literal_code, PLATFORM_SESSION_BOUNDARY_FUNCTION)
    if boundary_body is None:
        fail(
            "platform session adapter boundary predicate missing: "
            f"{PLATFORM_SESSION_BOUNDARY_FUNCTION}",
            issues,
        )
    elif " ".join(boundary_body.split()) != f"{{ {PLATFORM_SESSION_BOUNDARY_PREDICATE} }}":
        fail(
            "platform session adapter boundary predicate drifted: expected "
            f"{PLATFORM_SESSION_BOUNDARY_PREDICATE!r}",
            issues,
        )

    interior_body = _rust_function_body(literal_code, PLATFORM_SESSION_INTERIOR_FUNCTION)
    interior_shape = (
        None
        if interior_body is None
        else PLATFORM_IDENTITY_INTERIOR_SHAPE.match(" ".join(interior_body.split()))
    )
    if interior_shape is None:
        fail(
            "platform session adapter interior predicate is not the admitted shape: "
            f"{PLATFORM_SESSION_INTERIOR_FUNCTION}",
            issues,
        )
    else:
        admitted_bytes = []
        for alternative in interior_shape.group("alternatives").split("|"):
            literal = PLATFORM_IDENTITY_BYTE_LITERAL.match(alternative.strip())
            if literal is None:
                fail(
                    "platform session adapter interior predicate admits a non-literal byte: "
                    f"{alternative.strip()!r}",
                    issues,
                )
                continue
            admitted_bytes.append(literal.group("byte"))
        if tuple(sorted(admitted_bytes)) != tuple(sorted(interior_extra)):
            fail(
                "platform session adapter interior byte class drifted from the contract: source "
                f"admits {sorted(admitted_bytes)}, contract regex admits "
                f"{sorted(interior_extra)}",
                issues,
            )
        if tuple(sorted(interior_extra)) != tuple(sorted(PLATFORM_SESSION_ADAPTER_INTERIOR_EXTRA)):
            fail(
                "platform session adapter interior class table drifted from the contract regex",
                issues,
            )

    digest_shape = PLATFORM_SESSION_DIGEST_SHAPE.match(PLATFORM_SESSION_DIGEST_REGEX)
    if digest_shape is None:
        fail("platform session digest grammar is not a parseable shape", issues)
        return
    digest_prefix = PLATFORM_SESSION_DIGEST_PREFIX_DECLARATION.search(literal_code)
    if digest_prefix is None:
        fail("platform session digest prefix constant missing or not a literal", issues)
    elif digest_prefix.group("prefix") != digest_shape.group("prefix"):
        fail(
            "platform session digest prefix drifted from the contract: source "
            f"{digest_prefix.group('prefix')!r}, contract {digest_shape.group('prefix')!r}",
            issues,
        )
    declared_digits = rust_module_level_usize_constants(
        code, PLATFORM_SESSION_DIGEST_DIGITS_CONSTANT
    )
    if declared_digits != (int(digest_shape.group("digits")),):
        fail(
            "platform session digest length drifted: "
            f"{PLATFORM_SESSION_DIGEST_DIGITS_CONSTANT} is {list(declared_digits)}, contract "
            f"regex implies {digest_shape.group('digits')}",
            issues,
        )
    digest_body = _rust_function_body(literal_code, PLATFORM_SESSION_DIGEST_FUNCTION)
    if digest_body is None:
        fail(f"platform session digest validator missing: {PLATFORM_SESSION_DIGEST_FUNCTION}", issues)
    elif PLATFORM_SESSION_DIGEST_CLASS_CARRIER not in " ".join(digest_body.split()):
        fail(
            "platform session digest byte class drifted: expected "
            f"{PLATFORM_SESSION_DIGEST_CLASS_CARRIER!r}",
            issues,
        )
    elif digest_shape.group("class") != "0-9a-f":
        fail(
            "platform session digest contract class is not the admitted lowercase hexadecimal "
            f"set: {digest_shape.group('class')!r}",
            issues,
        )

    # §11's dependency prohibition has no other carrier once B2 exists: a path-qualified call
    # inside a function body declares no item, so the item allowlist cannot see it.
    for carrier in PLATFORM_SESSION_FORBIDDEN_CARRIERS:
        if carrier in code:
            fail(
                f"platform session module gained a forbidden dependency carrier: {carrier!r}",
                issues,
            )
    for label, pattern in PLATFORM_SESSION_FORBIDDEN_DIGEST_CARRIERS:
        if re.search(pattern, code):
            fail(
                f"platform session module must compute no digest: {label!r} is forbidden",
                issues,
            )
    for label, pattern in PLATFORM_IDENTITY_FORBIDDEN_SPLICE_PATTERNS:
        if re.search(pattern, code):
            fail(f"platform session module must not splice external source: {label!r}", issues)
    if re.search(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*;", code):
        fail(
            "platform session module must not declare a file-backed submodule: a `mod name;` "
            "compiles a second file that no scan reads",
            issues,
        )
    declared_modules = re.findall(r"\bmod\s+([A-Za-z_][A-Za-z0-9_]*)", code)
    if declared_modules != [PLATFORM_SESSION_TEST_MODULE]:
        fail(
            "platform session module declarations drifted: expected exactly "
            f"['{PLATFORM_SESSION_TEST_MODULE}'] actual={declared_modules}",
            issues,
        )
    if f"#[cfg(test)] mod {PLATFORM_SESSION_TEST_MODULE}" not in (
        PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS["session.rs"]
    ):
        fail(
            "platform session fixture module must be admitted as a cfg(test)-gated item",
            issues,
        )
    if re.search(RUST_INNER_ATTRIBUTE_PATTERN, code):
        fail("platform session module must not carry an inner attribute", issues)
    for kind in PLATFORM_SESSION_FORBIDDEN_PUBLIC_ITEM_KINDS:
        if kind in code:
            fail(
                f"platform session module declared a forbidden public item kind: {kind!r}",
                issues,
            )
    for line in code.splitlines():
        stripped = line.strip()
        if stripped.startswith("pub const ") and not stripped.startswith("pub const fn "):
            fail(f"platform session module declared a public constant: {stripped!r}", issues)

    # The public surface is an ALLOWLIST over the declaration grammar, not a blacklist of bad
    # spellings: `pub fn new` being absent says nothing about `pub fn from_unchecked`, a cross-kind
    # `From`, a `pub type` alias or a generic state setter. An unclassifiable `pub` fails too.
    public_declarations, unclassified_public = rust_public_declarations(code)
    if unclassified_public:
        fail(
            "platform session module has an unclassified public declaration: "
            f"{unclassified_public}",
            issues,
        )
    if public_declarations != sorted(PLATFORM_SESSION_ADMITTED_PUBLIC_DECLARATIONS):
        fail(
            "platform session public declaration surface drifted from the admitted allowlist: "
            f"actual={public_declarations}",
            issues,
        )
    derives = sorted(rust_derive_bodies(code))
    if derives != sorted(PLATFORM_SESSION_ADMITTED_DERIVES):
        fail(
            f"platform session derive surface drifted from the admitted allowlist: {derives}",
            issues,
        )

    # The bound test file. A renamed or de-registered function makes `--exact` match nothing,
    # which cargo reports as `running 0 tests` at exit zero, so the four names, the `#[test]`
    # count and each attribute envelope are pinned out of band.
    for function in PLATFORM_SESSION_TEST_FUNCTIONS:
        if not re.search(rf"^fn {function}\(\)", test_code, flags=re.MULTILINE):
            fail(f"platform session acceptance test missing: {function}", issues)
            continue
        attributes = rust_attribute_block(test_code, function)
        if attributes is None:
            fail(f"platform session acceptance test unreadable: {function}", issues)
            continue
        if attributes != list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES):
            fail(
                f"platform session acceptance test {function} attribute envelope drifted: "
                f"expected {list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES)} "
                f"actual={attributes}",
                issues,
            )
        for attribute in attributes:
            for marker in PLATFORM_IDENTITY_FORBIDDEN_TEST_ATTRIBUTE_MARKERS:
                if marker in attribute:
                    fail(
                        f"platform session acceptance test {function} carries a non-executing "
                        f"attribute: {attribute}",
                        issues,
                    )
    # The library-leg fixtures, held to the same envelope rule: a renamed or de-registered one
    # makes `--exact` match nothing, which cargo reports as `running 0 tests` at exit zero.
    for function in PLATFORM_SESSION_LIB_TEST_FUNCTIONS:
        if not re.search(rf"^    fn {function}\(\)", code, flags=re.MULTILINE):
            fail(f"platform session library fixture missing: {function}", issues)
            continue
        attributes = rust_attribute_block(code, function)
        if attributes != list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES):
            fail(
                f"platform session library fixture {function} attribute envelope drifted: "
                f"expected {list(PLATFORM_IDENTITY_REQUIRED_TEST_ATTRIBUTES)} "
                f"actual={attributes}",
                issues,
            )
    lib_registered = len(re.findall(r"^    #\[test\]$", code, flags=re.MULTILINE))
    if lib_registered != len(PLATFORM_SESSION_LIB_TEST_FUNCTIONS):
        fail(
            "platform session library fixture registration drift: expected "
            f"{len(PLATFORM_SESSION_LIB_TEST_FUNCTIONS)} #[test] carriers actual={lib_registered}",
            issues,
        )

    registered_tests = len(re.findall(r"^#\[test\]$", test_code, flags=re.MULTILINE))
    if registered_tests != len(PLATFORM_SESSION_TEST_FUNCTIONS):
        fail(
            "platform session acceptance test registration drift: expected "
            f"{len(PLATFORM_SESSION_TEST_FUNCTIONS)} #[test] carriers actual={registered_tests}",
            issues,
        )
    for marker, pattern in PLATFORM_IDENTITY_FORBIDDEN_TEST_FILE_PATTERNS:
        if re.search(pattern, test_code):
            fail(
                "platform session acceptance tests must execute unconditionally: "
                f"{marker!r} is forbidden in {PLATFORM_SESSION_TEST}",
                issues,
            )
    test_definitions = rust_macro_definitions(test_code)
    if test_definitions:
        fail(
            f"macro definitions drifted in {PLATFORM_SESSION_TEST}: expected none actual="
            f"{test_definitions}",
            issues,
        )
    test_attributes, unterminated_test_attributes = rust_attributes(test_code)
    if unterminated_test_attributes:
        fail(
            f"unterminated attribute in {PLATFORM_SESSION_TEST}: "
            f"{unterminated_test_attributes}",
            issues,
        )
    observed_test_attribute_names = rust_attribute_names(test_attributes)
    admitted_test_attribute_names = tuple(sorted(PLATFORM_SESSION_ADMITTED_TEST_ATTRIBUTE_NAMES))
    if observed_test_attribute_names != admitted_test_attribute_names:
        fail(
            f"bound test attribute names drifted in {PLATFORM_SESSION_TEST}: expected "
            f"{admitted_test_attribute_names} actual={observed_test_attribute_names}",
            issues,
        )
    # A block-local `use std::assert as assert_eq;` rebinds the macro for the rest of its scope
    # while adding no definition and changing no invocation name, so the bound file's items are
    # accounted for in full, exactly as the governed sources are.
    test_items, unterminated_test_items = rust_item_declarations(test_code)
    if unterminated_test_items:
        fail(
            f"unterminated item declaration in {PLATFORM_SESSION_TEST}: "
            f"{unterminated_test_items}",
            issues,
        )
    if test_items != list(PLATFORM_SESSION_ADMITTED_TEST_ITEMS):
        fail(
            f"bound test item declarations drifted in {PLATFORM_SESSION_TEST}: expected "
            f"{list(PLATFORM_SESSION_ADMITTED_TEST_ITEMS)} actual={test_items}",
            issues,
        )
    test_invocations, unterminated_test_macros = rust_macro_invocation_arguments(test_code)
    if unterminated_test_macros:
        fail(
            f"unterminated macro invocation in {PLATFORM_SESSION_TEST}: "
            f"{sorted(unterminated_test_macros)}",
            issues,
        )
    invoked_in_test = tuple(sorted({name for name, _ in test_invocations}))
    admitted_in_test = tuple(sorted(PLATFORM_SESSION_ADMITTED_TEST_MACRO_INVOCATIONS))
    if invoked_in_test != admitted_in_test:
        fail(
            f"macro invocations drifted in {PLATFORM_SESSION_TEST}: expected "
            f"{admitted_in_test} actual={invoked_in_test}",
            issues,
        )

    checker_path = ROOT / "scripts/check_repo_contracts.py"
    if not checker_path.is_file():
        fail("platform session carrier missing: scripts/check_repo_contracts.py", issues)
        return
    main_body = checker_path.read_text(encoding="utf-8").split("\ndef main() -> int:", 1)
    required_call = "check_platform_session_implementation(issues)"
    invoked = len(main_body) == 2 and any(
        line.strip() == required_call for line in main_body[1].splitlines()
    )
    if not invoked:
        fail(
            "check_platform_session_implementation must be invoked from repository main()",
            issues,
        )


P1_SOURCE_REVISION_PROPOSAL = "docs/tasks/p1-source-revision-readiness-proposal.md"
P1_SOURCE_IMPORT_CONTRACT = "docs/contracts/source-import.md"
P1_SOURCE_REVISION_PACKET_BEGIN = "<!-- P1_SOURCE_REVISION_EXACT_PACKET:BEGIN -->"
P1_SOURCE_REVISION_PACKET_END = "<!-- P1_SOURCE_REVISION_EXACT_PACKET:END -->"
P1_SOURCE_REVISION_PACKET_SHA256 = (
    "11529705aca4e19ae52bde8ec5a69571c2c3cc6d1157057ac77dd69f78aa65f9"
)
P1_SOURCE_REVISION_PACKET_BYTES = 8479
P1_SOURCE_IMPORT_V0_SECTION_HEADING = "## 15. `source-import/v0` — historical evidence retained"
P1_SOURCE_IMPORT_V0_SECTION_BYTES = 1689
P1_SOURCE_IMPORT_V0_SECTION_SHA256 = (
    "3253b1fbdef9f09b973b85332a1d8ea2d662d46a50d0308ef24902d21ab3276c"
)
P1_SOURCE_IMPORT_V1_METADATA_LINE = "- `Status`: Proposed as `source-import/v1` under R3 M60-B2 two-layer transport architecture"
P1_SOURCE_REVISION_SOURCE_COMMIT = "2f4de29032560ff3e13d9994b33a3aff14243f44"
P1_SOURCE_REVISION_SOURCE_TREE = "53e266c47fdb07d50a734faa24bb11ac4bc5527d"
P1_0_ALLOWED_PATHS = (
    "docs/contracts/source-import.md",
    "docs/plan/modules/70-campus-trust-source-pipeline.md",
    "docs/tasks/01-execution-roadmap.md",
    "docs/tasks/p1-source-revision-readiness-proposal.md",
    "scripts/check_repo_contracts.py",
    "scripts/tests/test_check_repo_contracts.py",
)
P1_1_PACKET_ALLOWED_PATHS = (
    "crates/platform-core/src/lib.rs",
    "crates/platform-core/src/source_registry.rs",
    "crates/platform-core/tests/source_registry.rs",
    "docs/acceptance/matrix.tsv",
    "docs/acceptance/platform-baseline.md",
    "docs/contracts/source-import.md",
    "docs/plan/modules/70-campus-trust-source-pipeline.md",
    "docs/tasks/01-execution-roadmap.md",
    "docs/tasks/p1-source-revision-readiness-proposal.md",
    "scripts/check_repo_contracts.py",
    "scripts/tests/test_check_repo_contracts.py",
)
P1_1_SCOPE_AMENDMENT_PATH = "crates/platform-core/tests/platform_identity.rs"
P1_1_ALLOWED_PATHS = P1_1_PACKET_ALLOWED_PATHS + (P1_1_SCOPE_AMENDMENT_PATH,)
P1_PLANNED_MATRIX_CASES = ("SRC-010", "SRC-011", "SRC-012")
P1_IMPLEMENTED_MATRIX_CASES = ("SRC-001",)
P1_IMPLEMENTED_MATRIX_BINDING = (
    "cargo test --locked -p ustc-campus-agent-core --test source_registry"
)


def _p1_exact_fenced_paths(packet: str, heading: str) -> tuple[str, ...] | None:
    start = packet.find(heading)
    if start == -1:
        return None
    fence_start = packet.find("```text\n", start)
    fence_end = packet.find("\n```", fence_start + len("```text\n"))
    if fence_start == -1 or fence_end == -1:
        return None
    body = packet[fence_start + len("```text\n") : fence_end]
    return tuple(line for line in body.splitlines() if line)


M60_B2_PROPOSAL_PATH = "docs/tasks/m60-b2-retrieval-policy-readiness-proposal.md"
M60_B2_PROPOSAL_BEGIN = "<!-- M60_B2_RETRIEVAL_POLICY_PROPOSAL:BEGIN -->"
M60_B2_PROPOSAL_END = "<!-- M60_B2_RETRIEVAL_POLICY_PROPOSAL:END -->"
# Superseded V10 packet digest — checked as historical evidence only
M60_B2_SUPERSEDED_PACKET_SHA256 = "ba36425adc164ca9b3ec75addd4be2e4b299b5f8a8cfb75cf6a710679acd32ab"
M60_B2_REPLACEMENT_PACKET_BYTES = 33046
M60_B2_REPLACEMENT_PACKET_SHA256 = "34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e"
M60_B2_SOURCE_RETRIEVAL_PATH = "docs/contracts/source-retrieval.md"
M60_B2_SOURCE_IMPORT_PATH = "docs/contracts/source-import.md"
M60_B2_REPLACEMENT_STAGE = "`M60_B2_CONTRACT_ACCEPTED`"
M60_B2_PACKET_STATUS = "`ACCEPTED`"
M60_B2_REQUIRED_STAGE_LINE = "- `Stage`: `M60_B2_CONTRACT_ACCEPTED`"
M60_B2_REQUIRED_STATUS_LINE = "- `Packet status`: `ACCEPTED`"
M60_B2_REQUIRED_DIGEST_LINE = (
    "- `R4 replacement packet digest`: "
    f"`sha256:{M60_B2_REPLACEMENT_PACKET_SHA256}` over "
    f"`{M60_B2_REPLACEMENT_PACKET_BYTES}` bytes beginning immediately after the `BEGIN` "
    "marker newline and ending immediately before the `END` marker token, including the "
    "final packet newline"
)
M60_B2_PROJECTED_PATHS = (
    M60_B2_PROPOSAL_PATH,
    "docs/contracts/source-import.md",
    "docs/contracts/source-retrieval.md",
    "docs/contracts/module-boundaries.md",
    "docs/plan/05-campus-trust-kernel.md",
    "docs/plan/modules/00-module-map.md",
    "docs/plan/modules/70-campus-trust-source-pipeline.md",
    "docs/tasks/01-execution-roadmap.md",
    "docs/coverage-matrix.md",
)


def check_m60_b2_packet_digest(issues: list[str]) -> None:
    path = ROOT / M60_B2_PROPOSAL_PATH
    if not path.is_file():
        fail(f"M60-B2 proposal carrier missing: {M60_B2_PROPOSAL_PATH}", issues)
        return
    raw_bytes = path.read_bytes()
    begin_bytes = M60_B2_PROPOSAL_BEGIN.encode("utf-8")
    end_bytes = M60_B2_PROPOSAL_END.encode("utf-8")
    begin_lf = (M60_B2_PROPOSAL_BEGIN + "\n").encode("utf-8")

    begin_count = raw_bytes.count(begin_bytes)
    end_count = raw_bytes.count(end_bytes)
    if begin_count != 1 or end_count != 1:
        fail(
            "M60-B2 R4 replacement packet marker count drift: "
            f"begin={begin_count} end={end_count}",
            issues,
        )
        return
    if raw_bytes.count(begin_lf) != 1:
        fail(
            "M60-B2 R3 replacement packet BEGIN marker must be followed by exactly one LF",
            issues,
        )
        return
    start = raw_bytes.index(begin_lf) + len(begin_lf)
    finish = raw_bytes.index(end_bytes)
    if finish <= start:
        fail("M60-B2 R4 replacement packet marker order drift", issues)
        return
    block = raw_bytes[start:finish]
    block_len = len(block)
    if block_len == 0:
        fail("M60-B2 R4 replacement packet is empty", issues)
        return
    if block_len == 77276:
        fail(
            "M60-B2 R4 replacement packet byte count equals the superseded V10 packet "
            "(77276 bytes); the replacement must differ",
            issues,
        )
    actual_sha256 = hashlib.sha256(block).hexdigest()
    if actual_sha256 == M60_B2_SUPERSEDED_PACKET_SHA256:
        fail(
            "M60-B2 R4 replacement packet digest equals the superseded V10 digest; "
            "the replacement must differ",
            issues,
        )

    if block_len != M60_B2_REPLACEMENT_PACKET_BYTES:
        fail(
            "M60-B2 R4 replacement packet byte count drift: "
            f"expected={M60_B2_REPLACEMENT_PACKET_BYTES} "
            f"actual={block_len}",
            issues,
        )
    if actual_sha256 != M60_B2_REPLACEMENT_PACKET_SHA256:
        fail(
            "M60-B2 R4 replacement packet digest drift: "
            f"expected={M60_B2_REPLACEMENT_PACKET_SHA256} "
            f"actual={actual_sha256}",
            issues,
        )

    text = raw_bytes.decode("utf-8")

    stage_section = _extract_markdown_section(text, "## Mutable state")
    if stage_section is None:
        fail("M60-B2 proposal missing '## Mutable state' section", issues)
    else:
        if M60_B2_REQUIRED_STAGE_LINE not in stage_section:
            fail(
                f"M60-B2 R3 stage missing from '## Mutable state' section: "
                f"expected {M60_B2_REQUIRED_STAGE_LINE}",
                issues,
            )
        if stage_section.count(M60_B2_REQUIRED_STAGE_LINE) != 1:
            fail(
                f"M60-B2 R4 stage must appear exactly once in '## Mutable state', "
                f"found {stage_section.count(M60_B2_REQUIRED_STAGE_LINE)}",
                issues,
            )
        if M60_B2_REQUIRED_STATUS_LINE not in stage_section:
            fail(
                f"M60-B2 R4 packet status missing from '## Mutable state' section: "
                f"expected {M60_B2_REQUIRED_STATUS_LINE}",
                issues,
            )
        if stage_section.count(M60_B2_REQUIRED_STATUS_LINE) != 1:
            fail(
                f"M60-B2 R4 packet status must appear exactly once in '## Mutable state', "
                f"found {stage_section.count(M60_B2_REQUIRED_STATUS_LINE)}",
                issues,
            )
        if stage_section.count("- `Stage`: ") != 1:
            fail(
                "M60-B2 '## Mutable state' must have exactly one Stage line "
                f"(found {stage_section.count('- `Stage`: ')})",
                issues,
            )
        if stage_section.count("- `Packet status`: ") != 1:
            fail(
                "M60-B2 '## Mutable state' must have exactly one Packet status line "
                f"(found {stage_section.count('- `Packet status`: ')})",
                issues,
            )
        if stage_section.count(M60_B2_REQUIRED_DIGEST_LINE) != 1:
            fail(
                "M60-B2 R4 replacement packet digest metadata must bind the exact "
                "payload byte count and SHA-256",
                issues,
            )
        if stage_section.count("- `R4 replacement packet digest`: ") != 1:
            fail(
                "M60-B2 '## Mutable state' must have exactly one R4 replacement packet "
                "digest line",
                issues,
            )

    for old_stage in (
        "`M60_B2_ACCEPTED_WITH_AMENDMENT`",
    ):
        if '- `Stage`: ' + old_stage in text:
            subscript = text.find('- `Stage`: ' + old_stage)
            before = max(0, subscript - 200)
            if "SUPERSEDED" not in text[before : subscript + 200] and "HISTORICAL" not in text[before : subscript + 200]:
                fail(
                    f"M60-B2 superseded stage {old_stage} appears without SUPERSEDED/HISTORICAL marker",
                    issues,
                )

    # R11 exact semantic acceptance receipt (CURRENT) — exactly one, ACCEPTED, with the
    # exact acceptance decision.  Fails closed on missing or duplicated acceptance receipt.
    r11_receipt_header = "### M60-B2 R11 exact semantic acceptance receipt (CURRENT)"
    r11_receipt_count = text.count(r11_receipt_header)
    if r11_receipt_count != 1:
        fail(
            "M60-B2 proposal must have exactly one R11 exact semantic acceptance receipt "
            f"(CURRENT) section, found {r11_receipt_count}",
            issues,
        )
    else:
        r11_receipt = _extract_markdown_section(text, r11_receipt_header)
        if r11_receipt is None or "- `Receipt status`: `ACCEPTED`" not in r11_receipt:
            fail(
                "M60-B2 R11 exact semantic acceptance receipt missing "
                "- `Receipt status`: `ACCEPTED`",
                issues,
            )
        if "`ACCEPT_EXACT_M60_B2_R11_PACKET`" not in r11_receipt:
            fail(
                "M60-B2 R11 exact semantic acceptance receipt missing "
                "decision `ACCEPT_EXACT_M60_B2_R11_PACKET`",
                issues,
            )

    # Historical R4 replacement direction receipt — preserved as explicitly historical
    # with PROPOSED_NOT_ACCEPTED.  The immutable block and this historical receipt are the
    # only legitimate remaining PROPOSED_NOT_ACCEPTED carriers.
    r4_historical_header = (
        "### M60-B2 R4 replacement direction receipt (HISTORICAL — SUPERSEDED BY R11 ACCEPTANCE)"
    )
    r4_historical_count = text.count(r4_historical_header)
    if r4_historical_count != 1:
        fail(
            "M60-B2 proposal must preserve exactly one historical R4 replacement direction "
            f"receipt (HISTORICAL — SUPERSEDED BY R11 ACCEPTANCE) section, found {r4_historical_count}",
            issues,
        )
    else:
        r4_receipt = _extract_markdown_section(text, r4_historical_header)
        if r4_receipt is None or "- `Receipt status`: `PROPOSED_NOT_ACCEPTED`" not in r4_receipt:
            fail(
                "M60-B2 historical R4 replacement direction receipt must preserve "
                "- `Receipt status`: `PROPOSED_NOT_ACCEPTED`",
                issues,
            )

    retrieval_path = ROOT / M60_B2_SOURCE_RETRIEVAL_PATH
    if not retrieval_path.is_file():
        fail(f"M60-B2 source-retrieval contract missing: {M60_B2_SOURCE_RETRIEVAL_PATH}", issues)
    elif not retrieval_path.read_bytes().strip():
        fail(f"M60-B2 source-retrieval contract empty: {M60_B2_SOURCE_RETRIEVAL_PATH}", issues)

    import_path = ROOT / M60_B2_SOURCE_IMPORT_PATH
    if not import_path.is_file():
        fail(f"M60-B2 source-import contract missing: {M60_B2_SOURCE_IMPORT_PATH}", issues)
    elif not import_path.read_bytes().strip():
        fail(f"M60-B2 source-import contract empty: {M60_B2_SOURCE_IMPORT_PATH}", issues)

    _check_source_retrieval_contract_metadata(issues)
    _check_source_retrieval_lifecycle_precondition(issues)
    _check_source_retrieval_transport_boundary_projection(issues)
    _check_source_import_owner_projection(issues)

    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    if not matrix_path.is_file():
        fail("M60-B2 acceptance matrix carrier missing", issues)
        return
    matrix_lines = matrix_path.read_text(encoding="utf-8").splitlines()
    if not matrix_lines:
        fail("M60-B2 acceptance matrix empty", issues)
        return
    header = matrix_lines[0].split("\t")
    try:
        case_index = header.index("case_id")
        status_index = header.index("status")
    except ValueError as ex:
        fail(f"M60-B2 acceptance matrix header missing column: {ex}", issues)
        return
    src010_rows = []
    for line in matrix_lines[1:]:
        fields = line.split("\t")
        if len(fields) != len(header):
            continue
        if fields[case_index] == "SRC-010":
            src010_rows.append(fields)
    if len(src010_rows) != 1:
        fail(
            f"M60-B2: expected exactly one SRC-010 row, found {len(src010_rows)}", issues,
        )
    elif src010_rows[0][status_index] != "planned":
        fail("M60-B2: SRC-010 must remain planned in acceptance matrix", issues)

    for line in matrix_lines[1:]:
        fields = line.split("\t")
        if len(fields) != len(header):
            continue
        if fields[case_index] == "SRC-014":
            fail("M60-B2: SRC-014 must remain catalog-only (not in active matrix)", issues)
            break

    _check_m60_roadmap_section_status(issues)
    _check_module_boundaries_transport_projection(issues)
    _check_coverage_matrix_m60_projection(issues)
    _check_m60_blueprint_status_projection(issues)


def _check_source_retrieval_transport_boundary_projection(issues: list[str]) -> None:
    retrieval_path = ROOT / M60_B2_SOURCE_RETRIEVAL_PATH
    retrieval_text = retrieval_path.read_text(encoding="utf-8")

    # Section-scoped boundary projection: exactly one line in §1.1 must state
    # M90 never receives EffectReadyRetrievalPlan and never returns BoundedFetch.
    boundary_section = _extract_markdown_section(
        retrieval_text, "### 1.1 Two-layer M60/M90 transport boundary"
    )
    if boundary_section is None:
        fail(
            "source-retrieval contract missing "
            "'### 1.1 Two-layer M60/M90 transport boundary' section",
            issues,
        )
    else:
        m90_never_line = (
            "- M90 never receives `EffectReadyRetrievalPlan` and never returns "
            "or constructs `BoundedFetch`, a domain receipt, "
            "or an authority-bearing carrier."
        )
        if boundary_section.count(m90_never_line) != 1:
            fail(
                "source-retrieval contract §1.1 must carry exactly one line stating "
                "M90 never receives EffectReadyRetrievalPlan and never returns BoundedFetch",
                issues,
            )

    # Section-scoped one-call port signature in §9.3.
    port_section = _extract_markdown_section(
        retrieval_text, "### 9.3 SourceTransportPort (M90 public port)"
    )
    if port_section is None:
        fail(
            "source-retrieval contract missing '### 9.3 SourceTransportPort (M90 public port)'",
            issues,
        )
    else:
        port_sig = "pub trait SourceTransportPort: Send + Sync {"
        if port_section.count(port_sig) != 1:
            fail(
                "source-retrieval contract §9.3 must carry exactly one public one-call "
                "SourceTransportPort trait signature",
                issues,
            )
        if "fn transport" not in port_section:
            fail(
                "source-retrieval contract §9.3 must define the transport method",
                issues,
            )
        if "Result<RetrievalTransportSuccess, SourceTransportError>" not in port_section:
            fail(
                "source-retrieval contract §9.3 must use the transport-only "
                "RetrievalTransportSuccess / SourceTransportError error split",
                issues,
            )
        if "Result<RetrievalTransportSuccess, SourceFetchFailure>" in port_section:
            fail(
                "source-retrieval contract §9.3 must not expose domain SourceFetchFailure "
                "through the public M90 transport port",
                issues,
            )

    # Section-scoped request ownership / no lifetime / no public constructor in §9.1.
    request_section = _extract_markdown_section(
        retrieval_text, "### 9.1 RetrievalTransportRequest"
    )
    if request_section is None:
        fail(
            "source-retrieval contract missing '### 9.1 RetrievalTransportRequest'",
            issues,
        )
    else:
        owned_prefix = (
            "`RetrievalTransportRequest` is a non-authority, private-field, owned struct "
            "containing exactly the adapter inputs needed for deterministic transport. It owns "
            "all values ("
        )
        owned_suffix = ") and carries no lifetime parameter."
        owned_start = request_section.find(owned_prefix)
        owned_sentence_count = request_section.count(owned_prefix)
        owned_end = request_section.find(owned_suffix, owned_start + len(owned_prefix))
        if owned_sentence_count != 1 or owned_start == -1 or owned_end == -1:
            owned_tokens: tuple[str, ...] = ()
            fail(
                "source-retrieval contract §9.1 must carry exactly one authoritative "
                "RetrievalTransportRequest owned-field sentence, found "
                f"{owned_sentence_count}",
                issues,
            )
        else:
            owned_body = request_section[owned_start + len(owned_prefix) : owned_end]
            owned_tokens = tuple(token.strip() for token in owned_body.split(","))
        request_owned_values = (
            "`RetrievalAttemptId`",
            "`SourceId`",
            "`SourceAuthorityRevision`",
            "`RetrievalDnsName`",
            "`SerializedRetrievalRequest`",
            "`SourceMediaType`",
            "`u32 maximum_response_bytes`",
            "`u32 maximum_elapsed_seconds`",
            "`SourceRetrievalProtocolVersion`",
            "`PublicIpPolicyVersion`",
        )
        if owned_tokens != request_owned_values:
            fail(
                "source-retrieval contract §9.1 authoritative RetrievalTransportRequest "
                f"owned-field sequence drifted: expected={request_owned_values!r} "
                f"actual={owned_tokens!r}",
                issues,
            )
        accessor_label = "The exact read-only accessor surface is:\n\n"
        accessor_label_start = request_section.find(accessor_label)
        if accessor_label_start == -1:
            accessor_body = ""
            fail(
                "source-retrieval contract §9.1 must carry the exact accessor-surface label",
                issues,
            )
        else:
            fence_start = request_section.find(
                "```rust\n", accessor_label_start + len(accessor_label)
            )
            fence_end = request_section.find("\n```", fence_start + len("```rust\n"))
            if fence_start == -1 or fence_end == -1:
                accessor_body = ""
                fail(
                    "source-retrieval contract §9.1 must carry one Rust accessor fence "
                    "immediately after the exact accessor-surface label",
                    issues,
                )
            else:
                accessor_body = request_section[
                    fence_start + len("```rust\n") : fence_end
                ]
        request_accessors = (
            "pub fn RetrievalTransportRequest::attempt_id(&self) -> &RetrievalAttemptId",
            "pub fn RetrievalTransportRequest::source_id(&self) -> &SourceId",
            "pub fn RetrievalTransportRequest::authority_revision(&self) -> SourceAuthorityRevision",
            "pub fn RetrievalTransportRequest::canonical_host(&self) -> &RetrievalDnsName",
            "pub fn RetrievalTransportRequest::serialized_request(&self) -> &SerializedRetrievalRequest",
            "pub fn RetrievalTransportRequest::expected_media_type(&self) -> &SourceMediaType",
            "pub fn RetrievalTransportRequest::maximum_response_bytes(&self) -> u32",
            "pub fn RetrievalTransportRequest::maximum_elapsed_seconds(&self) -> u32",
            "pub fn RetrievalTransportRequest::protocol_version(&self) -> &SourceRetrievalProtocolVersion",
            "pub fn RetrievalTransportRequest::public_ip_policy_version(&self) -> &PublicIpPolicyVersion",
        )
        actual_accessors = tuple(line for line in accessor_body.splitlines() if line)
        if actual_accessors != request_accessors:
            fail(
                "source-retrieval contract §9.1 exact RetrievalTransportRequest accessor "
                f"surface drifted: expected={request_accessors!r} actual={actual_accessors!r}",
                issues,
            )
        if "no lifetime parameter" not in request_section:
            fail(
                "source-retrieval contract §9.1 must declare no lifetime parameter",
                issues,
            )
        if "No public constructor, Serde, `Default`, URL/header builder conversion, arbitrary headers or retry/proxy/cookie/auth knobs." not in request_section:
            fail(
                "source-retrieval contract §9.1 must declare the exact authority-sentence: "
                "No public constructor, Serde, Default, URL/header builder conversion, "
                "arbitrary headers or retry/proxy/cookie/auth knobs.",
                issues,
            )
        if "not `Clone`" not in request_section:
            fail(
                "source-retrieval contract §9.1 must declare not Clone",
                issues,
            )
        if "`Clone + Debug + Eq`" in request_section or "cloneable non-authority" in request_section:
            fail(
                "source-retrieval contract §9.1 must keep RetrievalTransportRequest move-only",
                issues,
            )

    # Section-scoped raw observation + consuming projection in §9.2.
    success_section = _extract_markdown_section(
        retrieval_text, "### 9.2 RetrievalTransportSuccess"
    )
    if success_section is None:
        fail(
            "source-retrieval contract missing '### 9.2 RetrievalTransportSuccess'",
            issues,
        )
    else:
        if "into_parts" not in success_section:
            fail(
                "source-retrieval contract §9.2 must declare consuming into_parts",
                issues,
            )
        if "No observation constructor returns or embeds `RetrievalPolicyError`, `BoundedFetch`, receipt, phase carrier or authority witness." not in success_section:
            fail(
                "source-retrieval contract §9.2 must declare the exact authority-sentence: "
                "No observation constructor returns or embeds RetrievalPolicyError, "
                "BoundedFetch, receipt, phase carrier or authority witness.",
                issues,
            )
        if "into_parts(self)" not in success_section:
            fail(
                "source-retrieval contract §9.2 must retain a consuming success projection",
                issues,
            )
        success_tokens = (
            "pub fn RetrievalTransportSuccess::new(\n    response_head_bytes: Vec<u8>,\n    body_bytes: Vec<u8>,\n    wire_bytes_after_headers: u64,\n    peer_socket_addr: std::net::SocketAddr,\n    dns_transport_observation: DnsTransportObservation,\n    framing: RetrievalBodyFraming,\n    elapsed_milliseconds: u64,\n) -> Result<RetrievalTransportSuccess, SourceTransportError>",
            "pub struct RetrievalTransportSuccessParts {\n    pub response_head_bytes: Vec<u8>,\n    pub body_bytes: Vec<u8>,\n    pub wire_bytes_after_headers: u64,\n    pub peer_socket_addr: std::net::SocketAddr,\n    pub dns_transport_observation: DnsTransportObservation,\n    pub framing: RetrievalBodyFraming,\n    pub elapsed_milliseconds: u64,\n}",
            "pub fn RetrievalTransportSuccess::into_parts(self) -> RetrievalTransportSuccessParts",
            "pub fn RetrievalTransportSuccess::dns_transport_observation(&self) -> &DnsTransportObservation",
            "The constructor may return only `SourceTransportError::ObservationShapeRejected`",
        )
        for token in success_tokens:
            if success_section.count(token) != 1:
                fail(f"source-retrieval contract §9.2 exact transport success API drifted: {token}", issues)

    dns_section = _extract_markdown_section(
        retrieval_text, "### 7.3 DNS resolution and peer binding"
    )
    if dns_section is None:
        fail("source-retrieval contract missing '### 7.3 DNS resolution and peer binding'", issues)
    else:
        dns_tokens = (
            "pub fn DnsTransportObservation::new(\n    queried_host: String,\n    cname_chain: Vec<String>,\n    complete_addresses: Vec<std::net::Ipv4Addr>,\n) -> Result<DnsTransportObservation, SourceTransportError>",
            "It does not apply the v0 public-IP table, sort/deduplicate addresses, reject alias loops, select a peer or return `RetrievalPolicyError`.",
            "M60 independently reruns the policy over every successful raw observation",
        )
        for token in dns_tokens:
            if dns_section.count(token) != 1:
                fail(f"source-retrieval contract §7.3 raw DNS/M60 policy split drifted: {token}", issues)

    trait_section = _extract_markdown_section(
        retrieval_text, "### 11.2 Carrier linearity rules"
    )
    inventory_section = _extract_markdown_section(
        retrieval_text, "### 11.5 Complete named Rust inventory"
    )
    required_trait_rows = (
        "| `RetrievalReplayIdentity` | no | no | no | no | no (owner-private) | yes | no |",
        "| `RetrievalAttemptReceipt` | no | no | no | no | no (owner-private) | yes | no |",
        "| `RetrievalPlanCandidate` | yes | no | no | no | only `derive_candidate` | yes | no |",
        "| `BodyObservation` | no | no | no | no | yes (checked) | yes | no |",
        "| `DnsTransportObservation` | no | no | no | no | yes (shape-only) | yes | no |",
        "| `DnsResolutionObservation` | no | no | no | no | no (owner-private) | yes | no |",
        "| `RetrievalTransportSuccess` | no | no | no | no | yes (shape-only) | yes | no |",
        "| `RetrievalTransportSuccessParts` | no | no | no | no | only `RetrievalTransportSuccess::into_parts` | yes | no |",
    )
    if trait_section is None:
        fail("source-retrieval contract missing '### 11.2 Carrier linearity rules'", issues)
    else:
        for row in required_trait_rows:
            if trait_section.count(row) != 1:
                fail(f"source-retrieval contract §11.2 trait projection drifted: {row}", issues)
    if inventory_section is None:
        fail("source-retrieval contract missing '### 11.5 Complete named Rust inventory'", issues)
    else:
        public_heading = "Public non-authority values (directly or through checked pure construction):"
        private_heading = "Crate-private/internal values (no public constructor, no Serde, no Clone unless specified):"
        if inventory_section.count(public_heading) != 1 or inventory_section.count(private_heading) != 1:
            fail("source-retrieval contract §11.5 inventory headings drifted", issues)
        else:
            public_inventory, private_inventory = inventory_section.split(private_heading, 1)
            if "| `SourceFetchFailure` | §11 |" in public_inventory:
                fail("source-retrieval contract §11.5 must keep SourceFetchFailure out of public inventory", issues)
            if private_inventory.count("| `SourceFetchFailure` | §11 |") != 1:
                fail("source-retrieval contract §11.5 must classify SourceFetchFailure crate-private exactly once", issues)


def _check_source_import_owner_projection(issues: list[str]) -> None:
    import_path = ROOT / M60_B2_SOURCE_IMPORT_PATH
    import_text = import_path.read_text(encoding="utf-8")
    if "ACCEPT_EXACT_M60_B2_R11_PACKET" not in import_text:
        fail(
            "source-import contract Metadata must record the R11 acceptance decision "
            "`ACCEPT_EXACT_M60_B2_R11_PACKET`",
            issues,
        )
    if "SourceStatusEvidenceId::new(String) -> Result<SourceStatusEvidenceId, SourceValueError>" not in import_text:
        fail(
            "source-import contract must project exact SourceStatusEvidenceId::new constructor "
            "returning Result<SourceStatusEvidenceId, SourceValueError>",
            issues,
        )
    if "SourceMediaType::parse(&str) -> Result<SourceMediaType, SourceValueError>" not in import_text:
        fail(
            "source-import contract must project exact SourceMediaType::parse constructor",
            issues,
        )


def _check_module_boundaries_transport_projection(issues: list[str]) -> None:
    boundaries_path = ROOT / "docs/contracts/module-boundaries.md"
    boundaries_text = boundaries_path.read_text(encoding="utf-8")
    exact_row = "| `B-M60-M90-SOURCE-TRANSPORT` | `M60` | `M90` | M60 produces owned move-only `RetrievalTransportRequest` from `EffectReadyRetrievalPlan`; M90 implements `SourceTransportPort::transport`, performs DNS/TLS/HTTP/framing/body I/O under M60-provided bounds, returns only `RetrievalTransportSuccess` or transport-only `SourceTransportError`; M90 never receives `EffectReadyRetrievalPlan`, never names domain `SourceFetchFailure`, and never returns `BoundedFetch` | accepted contract; implementation planned |"
    if boundaries_text.count(exact_row) != 1:
        fail(
            "module-boundaries must project the exact B-M60-M90-SOURCE-TRANSPORT "
            "move-only/transport-error/non-authority row",
            issues,
        )
    if "B-M60-M90-SOURCE-FETCH" in boundaries_text:
        fail(
            "module-boundaries must not retain the superseded B-M60-M90-SOURCE-FETCH boundary",
            issues,
        )
    if "B-M60-M90-RETRIEVAL-CLOCK" in boundaries_text:
        fail(
            "module-boundaries must keep RetrievalClockPort M60-internal, not register an M90 clock boundary",
            issues,
        )


def _check_coverage_matrix_m60_projection(issues: list[str]) -> None:
    coverage_path = ROOT / "docs/coverage-matrix.md"
    coverage_text = coverage_path.read_text(encoding="utf-8")
    if "B-M60-M90-SOURCE-TRANSPORT" not in coverage_text:
        fail(
            "coverage-matrix M60 row must project B-M60-M90-SOURCE-TRANSPORT",
            issues,
        )
    if "B-M60-M90-SOURCE-FETCH" in coverage_text:
        fail(
            "coverage-matrix must not retain the superseded B-M60-M90-SOURCE-FETCH",
            issues,
        )
    if "B-M60-M90-RETRIEVAL-CLOCK" in coverage_text:
        fail(
            "coverage-matrix must not project an M60-M90 retrieval-clock boundary",
            issues,
        )


def _check_m60_blueprint_status_projection(issues: list[str]) -> None:
    blueprint_path = ROOT / "docs/plan/modules/70-campus-trust-source-pipeline.md"
    blueprint_text = blueprint_path.read_text(encoding="utf-8")
    required = (
        "operational Suspended/Revoked accepted as contract authority in source-import/v1, not implemented;",
        "The accepted `source-import/v1` and `source-retrieval/v0` contracts (R11 two-layer edition) define current contract authority for modules 1–2. The lifecycle precondition (operational `Suspended`/`Revoked`) must be satisfied by module 1 before module 2 can be implemented. No implementation exists for v1 or retrieval-policy; retained implementation remains forbidden until a separately admitted implementation packet.",
    )
    for token in required:
        if blueprint_text.count(token) != 1:
            fail(f"M60 blueprint accepted-contract authority projection drifted: {token}", issues)
    if "operational Suspended/Revoked accepted as authority in source-import/v1;" in blueprint_text:
        fail(
            "M60 blueprint must not promote source-import/v1 lifecycle beyond accepted-contract authority",
            issues,
        )
    if "operational Suspended/Revoked proposed as authority in source-import/v1, not accepted;" in blueprint_text:
        fail(
            "M60 blueprint must not regress to proposed-only lifecycle authority after R11 acceptance",
            issues,
        )


def check_p1_source_revision_contract(issues: list[str]) -> None:
    proposal_path = ROOT / P1_SOURCE_REVISION_PROPOSAL
    contract_path = ROOT / P1_SOURCE_IMPORT_CONTRACT
    for rel, path in (
        (P1_SOURCE_REVISION_PROPOSAL, proposal_path),
        (P1_SOURCE_IMPORT_CONTRACT, contract_path),
    ):
        if not path.is_file():
            fail(f"P1 source/revision carrier missing: {rel}", issues)
            return

    proposal_bytes = proposal_path.read_bytes()
    contract_bytes = contract_path.read_bytes()
    begin = (P1_SOURCE_REVISION_PACKET_BEGIN + "\n").encode()
    end = P1_SOURCE_REVISION_PACKET_END.encode()
    if proposal_bytes.count(begin) != 1 or proposal_bytes.count(end) != 1:
        fail("P1 source/revision packet markers must each occur exactly once", issues)
        return
    packet = proposal_bytes.split(begin, 1)[1].split(end, 1)[0]
    packet_sha = hashlib.sha256(packet).hexdigest()
    if len(packet) != P1_SOURCE_REVISION_PACKET_BYTES or packet_sha != P1_SOURCE_REVISION_PACKET_SHA256:
        fail(
            "P1 source/revision exact packet drift: "
            f"expected bytes={P1_SOURCE_REVISION_PACKET_BYTES} sha256={P1_SOURCE_REVISION_PACKET_SHA256} "
            f"actual bytes={len(packet)} sha256={packet_sha}",
            issues,
        )

    contract = contract_bytes.decode("utf-8")

    # Preserve the exact historical v0 section 15 frozen digest — not the full
    # evolving v1 file.  The section heading and terminator are uniquely named;
    # controller-computed authoritative values are inputs, not derived from actual.
    v0_section_start = contract.find(P1_SOURCE_IMPORT_V0_SECTION_HEADING)
    if v0_section_start == -1:
        fail(
            "source-import contract missing historical v0 section "
            f"{P1_SOURCE_IMPORT_V0_SECTION_HEADING!r}",
            issues,
        )
    else:
        v0_section_end = contract.find("\n## 16.", v0_section_start)
        if v0_section_end == -1:
            fail(
                "source-import contract missing v0 section terminator '## 16.'",
                issues,
            )
        else:
            v0_section_bytes = contract[v0_section_start:v0_section_end].encode("utf-8")
            if len(v0_section_bytes) != P1_SOURCE_IMPORT_V0_SECTION_BYTES:
                fail(
                    "P1 source-import historical v0 section §15 byte count drift: "
                    f"expected={P1_SOURCE_IMPORT_V0_SECTION_BYTES} "
                    f"actual={len(v0_section_bytes)}",
                    issues,
                )
            v0_section_sha256 = hashlib.sha256(v0_section_bytes).hexdigest()
            if v0_section_sha256 != P1_SOURCE_IMPORT_V0_SECTION_SHA256:
                fail(
                    "P1 source-import historical v0 section §15 digest drift: "
                    f"expected={P1_SOURCE_IMPORT_V0_SECTION_SHA256} "
                    f"actual={v0_section_sha256}",
                    issues,
                )

    # Validate current v1 Metadata separately — exact lines, each exactly once.
    metadata = _extract_markdown_section(contract, "## Metadata")
    if metadata is None:
        fail("source-import contract missing '## Metadata' section", issues)
    else:
        required_metadata_lines = (
            "- `Status`: Accepted contract as `source-import/v1` under the R11 M60-B2 two-layer transport architecture; supersedes the accepted V10 `DEC-M60-B2-ACCEPTANCE`; `source-import/v0` retained as explicitly historical (see §15)",
            "- `Version`: `source-import/v1`",
            "- `Last Review`: `2026-08-12`",
            "- `Predecessor`: [`source-import/v0`](#15-source-importv0-historical-evidence-retained) — accepted for bounded `M60-B1 source-registry` (P1-1), remains the authority for the existing `crates/platform-core/src/source_registry.rs` B1 implementation",
            "- `Accepted Per`: `ACCEPT_EXACT_M60_B2_R11_PACKET` — Develata accepted the exact `33046`-byte semantic packet (`sha256:34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e`) on 2026-08-13; prior V10 `DEC-M60-B2-ACCEPTANCE` is explicitly superseded historical evidence",
            "- `Owning Blueprint`: [`M60 Campus Trust and Source Pipeline`](../plan/modules/70-campus-trust-source-pipeline.md)",
            "- `Depends On`: [`module-boundaries.md`](module-boundaries.md), [`source-retrieval.md`](source-retrieval.md), and the existing crate-root `SourceAuthority` comparison policy",
            "- `Acceptance`: `SRC-001` is `implemented`; `SRC-010`, `SRC-011`, `SRC-012` remain `planned`; catalog-only `SRC-002`–`SRC-009` and `SRC-013` remain non-admitted; `SRC-014` remains catalog-only/non-admitted",
            "- `Primary Code`: `crates/platform-core/src/source_registry.rs` for bounded B1 (P1-1 review candidate, still under `source-import/v0`); no v1 Rust implementation exists",
        )
        for line in required_metadata_lines:
            if metadata.count(line) != 1:
                fail(f"P1 source-import v1 Metadata line missing/duplicated: {line!r}", issues)
        if metadata.count("- `Status`:") != 1:
            fail("P1 source-import '## Metadata' must have exactly one Status line", issues)
        if metadata.count("- `Version`:") != 1:
            fail("P1 source-import '## Metadata' must have exactly one Version line", issues)
        if metadata.count("- `Predecessor`:") != 1:
            fail("P1 source-import '## Metadata' must have exactly one Predecessor line", issues)
        if metadata.count("- `Accepted Per`:") != 1:
            fail("P1 source-import '## Metadata' must have exactly one Accepted Per line", issues)
        if metadata.count("- `Acceptance`:") != 1:
            fail("P1 source-import '## Metadata' must have exactly one Acceptance line", issues)
        if metadata.count("- `Primary Code`:") != 1:
            fail("P1 source-import '## Metadata' must have exactly one Primary Code line", issues)

    proposal = proposal_bytes.decode("utf-8")
    packet_text = packet.decode("utf-8")
    required_outer = (
        f"- `Source commit`: `{P1_SOURCE_REVISION_SOURCE_COMMIT}`",
        f"- `Source tree`: `{P1_SOURCE_REVISION_SOURCE_TREE}`",
        f"- `Packet digest`: `sha256:{P1_SOURCE_REVISION_PACKET_SHA256}` over `{P1_SOURCE_REVISION_PACKET_BYTES}` bytes",
        "- `Source candidate`: `ustc-teach-calendar-fall` remains `Proposed`",
        "- `Stage`: `P1_1_PR_OPEN`",
        "- `P1-0 review`: `GO`; exact-candidate receipt recorded below",
        "- `P1-0 local commit`: committed and pushed at exact HEAD `7491cb640b7d670822dbb39f165897e4145795a3`",
        "- `P1-1 implementation`: candidate present (bounded `M60-B1 source-registry` implemented as a review candidate)",
        "- `P1-1 review`: `GO`; exact-candidate receipt recorded below",
        "- `P1-1 local commit`: implementation commit `9616c3f074dd0a006794c4be76b66698a337a9ac`; marker-external status follow-ups remain separate",
        "- `P1-1 push`: feature branch pushed; PR `#38` open",
        "- `Remote shipping`: feature-branch push, PR, Actions CI/run and workflow dispatch authorized; merge/tag/release remain forbidden",
        "- `PR`: `#38` (`https://github.com/Develata/ustc-campus-agent/pull/38`), open against `main`",
        "- `Semantic implementation head`: `9616c3f074dd0a006794c4be76b66698a337a9ac`",
        "- `First PR exact-head CI`: run `31285180718`, `rust` and `docs-and-contracts` both `success`",
        "- `Status-follow-up rule`: any later marker-external status-only commit must receive replacement exact-head PR CI before merge eligibility",
    )
    for token in required_outer:
        if proposal.count(token) != 1:
            fail(f"P1 source/revision outer metadata missing/duplicated: {token}", issues)

    amendment_tokens = (
        "## P1-1 implementation scope amendment",
        "- `Status`: explicitly authorized by Develata on 2026-08-08 and admitted into the P1-1 closure candidate",
        "- `Develata authorization`: `授权该单一路径的窄范围 scope amendment（推荐）`",
        "- `Original packet`: unchanged at `sha256:11529705aca4e19ae52bde8ec5a69571c2c3cc6d1157057ac77dd69f78aa65f9` over `8479` bytes",
        f"- `Added path`: `{P1_1_SCOPE_AMENDMENT_PATH}`",
        "- `Permitted delta`: add only `source_registry` to that exact module-declaration expectation; do not weaken its lexical, compile-negative, identity or closure assertions",
        "- `§11 waiver`: waive the exact-packet stop condition for this one closure-carrier path only",
        "- `Boundary`: the amended P1-1 writable set is the original eleven paths plus this one closure carrier; no other scope, dependency, source approval, retrieval, PR, merge, tag or release authority is added",
    )
    for token in amendment_tokens:
        if proposal.count(token) != 1:
            fail(f"P1-1 implementation scope amendment missing/duplicated: {token}", issues)

    if _p1_exact_fenced_paths(packet_text, "## 3. P1-0 contract/readiness scope") != P1_0_ALLOWED_PATHS:
        fail("P1-0 exact writable path allowlist drifted", issues)
    if _p1_exact_fenced_paths(packet_text, "## 4. P1-1 exact implementation scope") != P1_1_PACKET_ALLOWED_PATHS:
        fail("P1-1 exact packet writable path allowlist drifted", issues)

    packet_tokens = (
        "preserve M60 as `planned`, M70 as `design-only` and every acceptance row's current status",
        "It must remain `Proposed`; no concrete row is instantiated as `Approved` in production data, fixtures or docs.",
        "This packet authorizes no push, PR, merge, tag, release or deployment.",
        "promote only matrix row `SRC-001`",
        "SRC-010 planned\nSRC-011 planned\nSRC-012 planned",
        "It adds no dependency, adapter, network access, DNS/redirect logic, clock, persistence, parser, raw snapshot, normalized snapshot, source revision, diff, baseline or product feed",
    )
    for token in packet_tokens:
        if packet_text.count(token) != 1:
            fail(f"P1 source/revision packet semantic carrier drifted: {token}", issues)

    receipt_tokens = (
        "### P1-0 exact-candidate GO receipt",
        "manifest `sha256:32e7f67c16b5b23e39435d45a40b662e6f6fa719a3c4d28d97ff7d5f9b82b645`",
        "archive `sha256:989b126f594fa048c1dddf65083f36581e9b77fa81ca7bd95891f15703076d26`",
        "- `Verdict`: `GO`",
        "- `Blockers`: `[]`",
        "- `P1-1 recommendation`: `ELIGIBLE`",
        "- `Report`: `sha256:d4f9cb7706d4f7b114c0e40541deb041dbe24fb6df57356d70be04540b5e4bae`",
    )
    p1_0_receipt_start = proposal.find("### P1-0 exact-candidate GO receipt")
    p1_0_receipt_end = proposal.find("\n## ", p1_0_receipt_start)
    p1_0_receipt = (
        proposal[p1_0_receipt_start:p1_0_receipt_end]
        if p1_0_receipt_start != -1 and p1_0_receipt_end != -1
        else ""
    )
    for token in receipt_tokens:
        if p1_0_receipt.count(token) != 1:
            fail(f"P1-0 GO receipt missing/duplicated: {token}", issues)

    p1_1_receipt_tokens = (
        "### P1-1 amended exact-candidate GO receipt",
        "12-path manifest `sha256:f69f7293db1bedfefa1c96c406988f119180721fadeca3df8724025c28b69a08`",
        "archive `sha256:b9b9d3ab33a17e0e8dcf3aed552f91fd312913f65cc225f2f4b9f1a5886e93d9`",
        "full Python 508/508 PASS",
        "- `Verdict`: `GO`",
        "- `Blockers`: `[]`",
        "- `Commit recommendation`: `ELIGIBLE`",
        "- `Report`: `sha256:03ce12a655766c9d9a4f224a4175168ae6146a9edfaf1b9ac8510507364735f2`",
        "- `Closure`: this receipt is marker-external and changes no reviewed Rust/API/contract/acceptance semantics; final commit still requires focused closeout gates and a narrow exact-delta review",
    )
    p1_1_receipt_start = proposal.find("### P1-1 amended exact-candidate GO receipt")
    p1_1_receipt = proposal[p1_1_receipt_start:] if p1_1_receipt_start != -1 else ""
    for token in p1_1_receipt_tokens:
        if p1_1_receipt.count(token) != 1:
            fail(f"P1-1 GO receipt missing/duplicated: {token}", issues)

    blueprint = (ROOT / "docs/plan/modules/70-campus-trust-source-pipeline.md").read_text(
        encoding="utf-8"
    )
    contract_tokens = (
        "- `Status`: Accepted blueprint; `source-import/v1` and `source-retrieval/v0` accepted as contract authority under R11 per `ACCEPT_EXACT_M60_B2_R11_PACKET` (2026-08-13); `M60` overall remains `planned`; bounded `M60-B1 source-registry` is implemented under `source-import/v0` (P1-1); operational `Suspended`/`Revoked` lifecycle precondition applies before any live B2 retrieval adapter; the superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical evidence only",
        "- `Implementation State`: `planned`",
        "- `Version`: `m60-campus-trust/v0.3`",
        "- `Current Contract`: accepted [`source-import/v1`](../../contracts/source-import.md) and [`source-retrieval/v0`](../../contracts/source-retrieval.md) under R11 (`ACCEPT_EXACT_M60_B2_R11_PACKET`); bounded B1 implementation remains under [`source-import/v0`](../../contracts/source-import.md#15-source-importv0--historical-evidence-retained) (P1-1 review candidate)",
        "SourceStatus: Proposed | Approved | Suspended | Revoked",
        "operational Suspended/Revoked accepted as contract authority in source-import/v1, not implemented;",
        "The accepted `source-import/v1` and `source-retrieval/v0` contracts (R11 two-layer edition) define current contract authority for modules 1–2. The lifecycle precondition (operational `Suspended`/`Revoked`) must be satisfied by module 1 before module 2 can be implemented. No implementation exists for v1 or retrieval-policy; retained implementation remains forbidden until a separately admitted implementation packet.",
        "SourceAuthorityRevision (non-zero monotone; CAS on every mutation)",
        "Proposed  + approve(full receipt)                 -> Approved(revision+1)",
        "Proposed  + revoke(evidence)                      -> Revoked(revision+1, prior_approval=None)",
        "`SourceRetrievalPolicy` has exactly six fields (expanded from v0's two):",
        "No constructor takes `SourceStatus`; no `approved`, `from_parts`, builder, `TryFrom` or Serde path may bypass the registry approval transition.",
        "`ModelInference` is rejected by `SourceDefinition::proposed`.",
        "This candidate family stays `Proposed`. No concrete USTC source is approved.",
        "duplicate `SourceId` or duplicate canonical `SourceUrl` is rejected without replacing the first definition;",
        "DuplicateUrl { url: SourceUrl }",
        "P1-0 records one reviewed **candidate family**, not an approved registry row:",
        "`SRC-001` is `implemented`; `SRC-010`, `SRC-011`, `SRC-012` remain `planned`",
        "retrieval-policy metadata with six fields whose limits later adapters must enforce",
        "SourceDefinitionBody::new(\n    owner: SourceOwner,\n    url: SourceUrl,\n    authority: SourceAuthority,\n    retrieval_policy: SourceRetrievalPolicy,\n) -> Result<Self, SourceValueError>",
        "SourceRetrievalPolicy::new(\n    minimum_interval_seconds: u32,\n    maximum_response_bytes: u32,\n    maximum_elapsed_seconds: u32,\n    expected_media_type: SourceMediaType,\n    protocol_version: SourceRetrievalProtocolVersion,\n    public_ip_policy_version: PublicIpPolicyVersion,\n) -> Result<Self, SourceValueError>",
        "P1-1 implemented bounded `M60-B1 source-registry` as a review candidate under `source-import/v0`",
        "added `crates/platform-core/src/source_registry.rs`;",
        "promoted `SRC-001` to `implemented` bound to `cargo test --locked -p ustc-campus-agent-core --test source_registry`;",
        "kept `SRC-010`, `SRC-011`, `SRC-012` and every catalog-only SRC row unchanged.",
        "It does **not** fetch a URL, resolve DNS, follow a redirect, read a clock, parse HTML/PDF, persist raw bytes, normalize records, compute a semantic diff, advance a baseline or publish a product event.",
    )
    combined = contract + "\n" + blueprint
    for token in contract_tokens:
        if combined.count(token) != 1:
            fail(f"P1 source-import truth projection missing/duplicated: {token}", issues)

    constructor_section = _extract_markdown_section(contract, "### 4.1 Constructors and traits")
    if constructor_section is None:
        fail("P1 source-import missing '### 4.1 Constructors and traits'", issues)
    else:
        required_constructor_tokens = (
            "SourceDefinitionBody::new(\n    owner: SourceOwner,\n    url: SourceUrl,\n    authority: SourceAuthority,\n    retrieval_policy: SourceRetrievalPolicy,\n) -> Result<Self, SourceValueError>",
            "SourceRetrievalPolicy::new(\n    minimum_interval_seconds: u32,\n    maximum_response_bytes: u32,\n    maximum_elapsed_seconds: u32,\n    expected_media_type: SourceMediaType,\n    protocol_version: SourceRetrievalProtocolVersion,\n    public_ip_policy_version: PublicIpPolicyVersion,\n) -> Result<Self, SourceValueError>",
            "No constructor takes `SourceStatus`; no `approved`, `from_parts`, builder, `TryFrom` or Serde path may bypass the registry approval transition.",
        )
        for token in required_constructor_tokens:
            if constructor_section.count(token) != 1:
                fail(
                    f"P1 source-import §4.1 constructor authority drifted: {token}",
                    issues,
                )

    candidate_start = contract.find("## 12. Concrete source candidate: proposed only")
    candidate_end = contract.find("## 13. P1-1 implementation slice", max(candidate_start, 0))
    if candidate_start == -1 or candidate_end == -1:
        fail("P1 concrete source candidate section missing", issues)
    else:
        candidate = contract[candidate_start:candidate_end]
        required_candidate_lines = (
            "- `Proposed source family label` (not a B1 `SourceId`): `ustc-teach-calendar-fall`",
            "- `Proposed 2025 SourceId`: `ustc-teach-calendar-fall-2025`",
            "- `Proposed 2026 SourceId`: `ustc-teach-calendar-fall-2026`",
            "- `Owner`: 中国科学技术大学教务处 / `www.teach.ustc.edu.cn`",
            "- `2025 URL`: `https://www.teach.ustc.edu.cn/calendar/19081.html`",
            "- `2026 URL`: `https://www.teach.ustc.edu.cn/calendar/20135.html`",
            "This candidate family stays `Proposed`. No concrete USTC source is approved.",
        )
        for token in required_candidate_lines:
            if candidate.count(token) != 1:
                fail(f"P1 concrete source candidate carrier drifted: {token}", issues)
        # Reject false approval using exact lines/structure, not accidental word counts.
        if candidate.count("Approved") != 0:
            fail(
                "P1 concrete source candidate must not contain 'Approved' — "
                "the candidate family is Proposed, not approved",
                issues,
            )
        if "This candidate family stays `Proposed`. No concrete USTC source is approved." not in candidate:
            fail(
                "P1 concrete source candidate must carry the exact Proposed-only sentence: "
                "'This candidate family stays `Proposed`. No concrete USTC source is approved.'",
                issues,
            )

    matrix_path = ROOT / "docs/acceptance/matrix.tsv"
    if not matrix_path.is_file():
        fail("P1 source/revision matrix carrier missing", issues)
    else:
        rows = {
            fields[0]: fields
            for line in matrix_path.read_text(encoding="utf-8").splitlines()[1:]
            if (fields := line.split("\t"))
        }
        for case_id in P1_PLANNED_MATRIX_CASES:
            fields = rows.get(case_id)
            if fields is None or len(fields) < 6 or fields[5] != "planned":
                fail(f"P1-1 requires {case_id} to remain planned", issues)
        for case_id in P1_IMPLEMENTED_MATRIX_CASES:
            fields = rows.get(case_id)
            if (
                fields is None
                or len(fields) < 6
                or fields[5] != "implemented"
                or fields[3] != P1_IMPLEMENTED_MATRIX_BINDING
            ):
                fail(
                    f"P1-1 requires {case_id} to be implemented with exact binding "
                    f"{P1_IMPLEMENTED_MATRIX_BINDING!r}",
                    issues,
                )

    roadmap = (ROOT / "docs/tasks/01-execution-roadmap.md").read_text(encoding="utf-8")
    for rel, text, token in (
        (
            "docs/plan/modules/70-campus-trust-source-pipeline.md",
            blueprint,
            "- `Implementation State`: `planned`",
        ),
        (
            "docs/tasks/01-execution-roadmap.md",
            roadmap,
            "The source remains `Proposed`; raw HTML remains outside Git; no acceptance row or module state is promoted by P1-0.",
        ),
        (
            "docs/tasks/01-execution-roadmap.md",
            roadmap,
            "Develata's later operation-specific instructions authorize the feature-branch push, PR, Actions CI/run and workflow dispatch; merge, tag, release, source approval and retrieval remain unauthorized.",
        ),
    ):
        if text.count(token) != 1:
            fail(f"P1 source/revision status projection drifted in {rel}: {token}", issues)

    forbidden_fixture_names = {
        "calendar-2025-fall.body",
        "calendar-2026-fall.body",
        "uca-p1-calendar-source-recon-20260808.tar.gz",
    }
    tracked_like = {
        path.name
        for path in ROOT.rglob("*")
        if path.is_file() and ".git" not in path.parts
    }
    leaked = sorted(forbidden_fixture_names & tracked_like)
    if leaked:
        fail(f"P1 raw source evidence must remain outside repository: {leaked}", issues)
P1_SOURCE_REGISTRY_SOURCE = "crates/platform-core/src/source_registry.rs"
P1_SOURCE_REGISTRY_TEST = "crates/platform-core/tests/source_registry.rs"
P1_SOURCE_REGISTRY_LIB = "crates/platform-core/src/lib.rs"
P1_SOURCE_REGISTRY_IDENTITY_TEST = P1_1_SCOPE_AMENDMENT_PATH
P1_SOURCE_REGISTRY_IDENTITY_MODULE_EXPECTATION = """&[
                "identity",
                "invocation",
                "market",
                "session",
                "source_registry",
            ] as &[&str],"""
P1_SOURCE_REGISTRY_IDENTITY_ITEM_EXPECTATION = '    "pub mod source_registry;",'

P1_SOURCE_REGISTRY_PUB_TYPES = (
    "pub enum SourceValueErrorKind",
    "pub struct SourceValueError",
    "pub struct SourceRetrievalPolicy",
    "pub struct SourceReviewReceipt",
    "pub enum SourceReviewState",
    "pub struct SourceDefinition",
    "pub enum SourceRegistryError",
    "pub struct SourceRegistry",
)

P1_SOURCE_REGISTRY_MACRO_TYPES = (
    "SourceId",
    "SourceOwner",
    "SourceUrl",
    "SourceReviewerId",
    "SourceReviewEvidenceId",
)

P1_SOURCE_REGISTRY_FORBIDDEN_CARRIERS = (
    "std::fs",
    "std::io",
    "std::net",
    "std::path",
    "std::process",
    "std::sync",
    "std::thread",
    "std::time::Instant",
    "std::time::SystemTime",
    "tokio",
    "reqwest",
    "hyper",
    "ureq",
    "serde_json",
    "include_str",
    "include_bytes",
)

P1_SOURCE_REGISTRY_NO_SERDE_AGGREGATES = (
    "SourceDefinition",
    "SourceReviewState",
    "SourceReviewReceipt",
    "SourceRegistry",
    "SourceRegistryError",
    "SourceRetrievalPolicy",
    "SourceValueError",
    "SourceValueErrorKind",
)

P1_SOURCE_REGISTRY_EXPECTED_TEST_FUNCTIONS = frozenset({
    "source_id_family_enforces_grammar_and_precedence",
    "source_id_family_values_are_nominally_distinct",
    "source_owner_enforces_grammar_and_precedence",
    "source_url_enforces_grammar_and_precedence",
    "source_url_rejects_non_ascii_in_correct_position",
    "source_value_errors_never_echo_rejected_input",
    "retrieval_policy_enforces_bounds_and_precedence",
    "review_receipt_is_total_and_exposes_all_fields",
    "proposed_rejects_model_inference",
    "proposed_accepts_non_model_inference_authorities",
    "definition_accessors_return_correct_types",
    "registry_starts_empty",
    "propose_then_get_works",
    "propose_rejects_duplicate_source_id",
    "propose_rejects_duplicate_url",
    "approve_missing_rejects",
    "approve_then_approved_works",
    "approve_already_approved_preserves_first_receipt",
    "approved_rejects_missing_and_proposed",
    "failed_operations_preserve_registry_unchanged",
    "registry_error_display_and_source_chain",
    "no_aggregate_serde_decode_exists",
    "no_concrete_approved_ustc_source_in_production_data",
})

P1_SOURCE_REGISTRY_NO_IO_DECLARATION = "performs no I/O, reads no clock, computes no digest, resolves no"


def check_p1_source_registry_implementation(issues: list[str]) -> None:
    """Verify the bounded M60-B1 source-registry implementation against source-import/v0.

    This is the P1-1 implementation gate: it verifies that the Rust source-registry
    kernel and its acceptance test satisfy every invariant the contract freezes for
    bounded B1 — exact public type surface, no Default, no I/O or dependency widening,
    no public struct fields, no Serde on aggregates, no forbidden constructors on
    SourceDefinition, exact test function inventory, and the lib.rs module declaration.
    """
    source_path = ROOT / P1_SOURCE_REGISTRY_SOURCE
    test_path = ROOT / P1_SOURCE_REGISTRY_TEST
    lib_path = ROOT / P1_SOURCE_REGISTRY_LIB
    identity_test_path = ROOT / P1_SOURCE_REGISTRY_IDENTITY_TEST
    for rel, path in (
        (P1_SOURCE_REGISTRY_SOURCE, source_path),
        (P1_SOURCE_REGISTRY_TEST, test_path),
        (P1_SOURCE_REGISTRY_LIB, lib_path),
        (P1_SOURCE_REGISTRY_IDENTITY_TEST, identity_test_path),
    ):
        if not path.is_file():
            fail(f"P1-1 source-registry carrier missing: {rel}", issues)
            return

    source = source_path.read_text(encoding="utf-8")
    test = test_path.read_text(encoding="utf-8")
    lib = lib_path.read_text(encoding="utf-8")
    identity_test = identity_test_path.read_text(encoding="utf-8")
    stripped = strip_rust_comments_and_literals(source)

    # lib.rs module declaration and the pre-existing external authority-test closure carrier.
    if "pub mod source_registry;" not in lib:
        fail("P1-1 source-registry module declaration missing in lib.rs", issues)
    if identity_test.count(P1_SOURCE_REGISTRY_IDENTITY_MODULE_EXPECTATION) != 1:
        fail(
            "P1-1 platform-identity module expectation must admit source_registry exactly once",
            issues,
        )
    if identity_test.count(P1_SOURCE_REGISTRY_IDENTITY_ITEM_EXPECTATION) != 1:
        fail(
            "P1-1 platform-identity item expectation must admit source_registry exactly once",
            issues,
        )

    # Exact public type surface — every contract type must be present.  The
    # eight non-macro types have direct `pub struct`/`pub enum` declarations;
    # the five nominal value types are generated by `source_value!` and appear
    # as invocation arguments, not as `pub struct $name`.
    for decl in P1_SOURCE_REGISTRY_PUB_TYPES:
        if decl not in source:
            fail(f"P1-1 source-registry public type missing: {decl!r}", issues)
    for macro_type in P1_SOURCE_REGISTRY_MACRO_TYPES:
        if f"source_value!" not in source or macro_type not in source:
            fail(
                f"P1-1 source-registry macro-generated type missing: {macro_type!r}",
                issues,
            )

    # No Default: neither derive nor impl.
    if re.search(r"derive\([^)]*\bDefault\b", source):
        fail("P1-1 source-registry must not derive Default on any type", issues)
    if "impl Default for" in source:
        fail("P1-1 source-registry must not impl Default on any type", issues)

    # No I/O, network, clock, persistence, or dependency widening.  Checked
    # against the comment-stripped source so compile_fail doctests that name a
    # carrier (e.g. `serde_json::from_str`) do not false-positive.
    for carrier in P1_SOURCE_REGISTRY_FORBIDDEN_CARRIERS:
        if carrier in stripped:
            fail(
                f"P1-1 source-registry must not widen I/O or dependencies: {carrier!r}",
                issues,
            )

    # The source must carry its no-I/O declaration verbatim in its module doc.
    if P1_SOURCE_REGISTRY_NO_IO_DECLARATION not in source:
        fail("P1-1 source-registry no-I/O declaration missing", issues)

    # No public struct field: every struct field must be private.  The pattern
    # `pub <word>:` matches a field declaration with a `pub` visibility prefix
    # but not `pub fn`, `pub const fn`, `pub struct`, `pub enum`, `pub use`, or
    # `pub mod` — those have `fn`/`const`/`struct`/`enum`/`use`/`mod` as the
    # next token, not a field name followed by `:`.
    pub_fields = re.findall(r"pub\s+\w+\s*:", source)
    if pub_fields:
        fail(
            f"P1-1 source-registry must not expose public struct fields: {pub_fields}",
            issues,
        )

    # No Serde on aggregates: only the five nominal value types may implement
    # Serialize/Deserialize.  The macro generates them for `$name`, so no
    # concrete aggregate type name should appear in a Serde impl.
    for aggregate in P1_SOURCE_REGISTRY_NO_SERDE_AGGREGATES:
        if f"impl Serialize for {aggregate}" in source:
            fail(
                f"P1-1 source-registry aggregate must not implement Serialize: {aggregate}",
                issues,
            )
        if re.search(rf"impl\s+Deserialize<'\w+>\s+for\s+{aggregate}\b", source):
            fail(
                f"P1-1 source-registry aggregate must not implement Deserialize: {aggregate}",
                issues,
            )

    # No forbidden constructors on SourceDefinition: the only public
    # constructor is `proposed`.  No `new`, `approved`, `from_parts`, or
    # `builder` may appear in its impl block.  Brace-match the impl block on
    # the stripped source so nested braces in match arms do not confuse the
    # boundary.
    impl_marker = "impl SourceDefinition {"
    impl_start = stripped.find(impl_marker)
    if impl_start != -1:
        depth = 0
        for i in range(impl_start, len(stripped)):
            if stripped[i] == "{":
                depth += 1
            elif stripped[i] == "}":
                depth -= 1
                if depth == 0:
                    impl_body = stripped[impl_start : i + 1]
                    for forbidden in (
                        "pub fn new(",
                        "pub fn approved(",
                        "pub fn from_parts(",
                        "pub fn builder(",
                    ):
                        if forbidden in impl_body:
                            fail(
                                f"P1-1 source-registry SourceDefinition must not expose "
                                f"{forbidden[:-1]!r}",
                                issues,
                            )
                    break

    # No constructor takes SourceReviewState as a parameter: the review state
    # is set only by the registry approval transition, never by a definition
    # constructor.  Check that `SourceReviewState` does not appear as a
    # parameter type in `pub fn proposed`.
    proposed_match = re.search(r"pub fn proposed\(([^)]*)\)", source, re.DOTALL)
    if proposed_match and "SourceReviewState" in proposed_match.group(1):
        fail(
            "P1-1 source-registry SourceDefinition::proposed must not take SourceReviewState",
            issues,
        )

    # Exact test function inventory: the acceptance test must carry exactly the
    # 23 contract-mandated test functions.
    test_functions = set(re.findall(r"#\[test\]\s*\n\s*fn (\w+)\(", test))
    missing = P1_SOURCE_REGISTRY_EXPECTED_TEST_FUNCTIONS - test_functions
    extra = test_functions - P1_SOURCE_REGISTRY_EXPECTED_TEST_FUNCTIONS
    if missing:
        fail(f"P1-1 source-registry acceptance test missing functions: {sorted(missing)}", issues)
    if extra:
        fail(f"P1-1 source-registry acceptance test has extra functions: {sorted(extra)}", issues)

    # The test file must reference the bound acceptance row.
    if "SRC-001" not in test:
        fail("P1-1 source-registry acceptance test must reference SRC-001", issues)


def _extract_markdown_section(text: str, heading: str) -> str | None:
    start = text.find(heading + "\n")
    if start == -1:
        return None
    content_start = start + len(heading) + 1
    heading_level = len(heading.split()[0])
    end = len(text)
    for match in re.finditer(r"^(#+)\s", text[content_start:], flags=re.MULTILINE):
        if len(match.group(1)) <= heading_level:
            end = content_start + match.start()
            break
    return text[content_start:end]


def _check_source_retrieval_contract_metadata(issues: list[str]) -> None:
    retrieval_path = ROOT / M60_B2_SOURCE_RETRIEVAL_PATH
    retrieval_text = retrieval_path.read_text(encoding="utf-8")
    metadata = _extract_markdown_section(retrieval_text, "## Metadata")
    if metadata is None:
        fail(
            "source-retrieval contract missing '## Metadata' section",
            issues,
        )
        return
    if metadata.count("- `Status`:") != 1:
        fail(
            "source-retrieval contract '## Metadata' must have exactly one Status line",
            issues,
        )
    if metadata.count("- `Version`:") != 1:
        fail(
            "source-retrieval contract '## Metadata' must have exactly one Version line",
            issues,
        )
    if "ACCEPT_EXACT_M60_B2_R11_PACKET" not in metadata:
        fail(
            "source-retrieval contract '## Metadata' Status line must record the R11 "
            "acceptance decision `ACCEPT_EXACT_M60_B2_R11_PACKET` under the R11 two-layer "
            "transport architecture",
            issues,
        )
    if "- `Version`: `source-retrieval/v0`" not in metadata:
        fail(
            "source-retrieval contract '## Metadata' Version line does not record "
            "source-retrieval/v0",
            issues,
        )


def _check_source_retrieval_lifecycle_precondition(issues: list[str]) -> None:
    retrieval_path = ROOT / M60_B2_SOURCE_RETRIEVAL_PATH
    retrieval_text = retrieval_path.read_text(encoding="utf-8")
    stop_section = _extract_markdown_section(
        retrieval_text, "## 12. Non-claims and stop conditions"
    )
    if stop_section is None:
        fail(
            "source-retrieval contract missing "
            "'## 12. Non-claims and stop conditions' section",
            issues,
        )
        return
    if (
        "Operational `Suspended`/`Revoked` lifecycle and monotone "
        "`SourceAuthorityRevision` must be present before any live B2 retrieval adapter"
    ) not in stop_section:
        fail(
            "source-retrieval contract stop-condition section must carry the operational "
            "Suspended/Revoked lifecycle and SourceAuthorityRevision precondition",
            issues,
        )


def _check_m60_roadmap_section_status(issues: list[str]) -> None:
    roadmap_path = ROOT / "docs/tasks/01-execution-roadmap.md"
    if not roadmap_path.is_file():
        fail("M60-B2 roadmap carrier missing", issues)
        return
    roadmap_text = roadmap_path.read_text(encoding="utf-8")
    section = _extract_markdown_section(
        roadmap_text, "## 12. M60 lane — Campus Trust and Source Pipeline"
    )
    if section is None:
        fail(
            "roadmap missing '## 12. M60 lane — Campus Trust and Source Pipeline' section",
            issues,
        )
        return
    if section.count("**Status**:") != 1:
        fail(
            "M60 owning roadmap section must have exactly one **Status** line",
            issues,
        )
    if "R11 two-layer M60/M90 transport architecture" not in section:
        fail(
            "M60 owning roadmap section Status must reference the R11 two-layer "
            "M60/M90 transport architecture",
            issues,
        )
    if "PROPOSED_NOT_ACCEPTED" in roadmap_text and "R4 replacement direction receipt" not in roadmap_text:
        conditions = (
            "PROPOSED_NOT_ACCEPTED",
        )
        proposal_path = ROOT / M60_B2_PROPOSAL_PATH
        if proposal_path.is_file():
            proposal = proposal_path.read_text(encoding="utf-8")
            for condition in conditions:
                if condition not in proposal:
                    fail(
                        f"M60-B2 proposal must record R3 replacement package status "
                        f"{condition!r}",
                        issues,
                    )


def check_external_agent_access_contract(issues: list[str]) -> None:
    """Bind the accepted M10/M80 external-Agent amendment to its owning carriers."""

    rows = parse_markdown_table(
        "docs/contracts/interfaces.md",
        [
            "Operation ID",
            "Owner",
            "Permission class",
            "Effect class",
            "Initial projections",
            "Status",
        ],
        "external-Agent application-operation registry",
        issues,
    )
    actual_operations: dict[str, tuple[str, ...]] = {}
    for line_no, cells in rows:
        operation_id = markdown_code_value(cells[0])
        if operation_id is None:
            fail(
                f"external-Agent operation row {line_no} has an invalid operation ID",
                issues,
            )
            continue
        if operation_id in actual_operations:
            fail(f"duplicate external-Agent operation ID: {operation_id}", issues)
            continue
        actual_operations[operation_id] = tuple(
            markdown_code_value(cell) or cell for cell in cells[1:]
        )
    if actual_operations != EXTERNAL_AGENT_OPERATION_ROWS:
        fail(
            "external-Agent application-operation registry drift: "
            f"missing={sorted(set(EXTERNAL_AGENT_OPERATION_ROWS) - set(actual_operations))} "
            f"unexpected={sorted(set(actual_operations) - set(EXTERNAL_AGENT_OPERATION_ROWS))} "
            f"changed={sorted(operation_id for operation_id in set(actual_operations) & set(EXTERNAL_AGENT_OPERATION_ROWS) if actual_operations[operation_id] != EXTERNAL_AGENT_OPERATION_ROWS[operation_id])}",
            issues,
        )

    permission_rows = parse_markdown_table(
        "docs/contracts/permissions.md",
        ["Class", "Data/effect boundary", "Initial client posture"],
        "external-Agent permission classes",
        issues,
    )
    expected_permission_classes = {
        "public-read",
        "public-linkout",
        "tenant-private-read",
        "tenant-private-write",
    }
    actual_permission_classes = {
        value
        for _, cells in permission_rows
        if (value := markdown_code_value(cells[0])) is not None
    }
    if actual_permission_classes != expected_permission_classes:
        fail(
            "external-Agent permission class set drift: "
            f"expected={sorted(expected_permission_classes)} "
            f"actual={sorted(actual_permission_classes)}",
            issues,
        )

    texts = {
        rel: (ROOT / rel).read_text(encoding="utf-8")
        for rel in EXTERNAL_AGENT_CONTRACT_FILES
        if (ROOT / rel).is_file()
    }
    missing_carriers = sorted(set(EXTERNAL_AGENT_CONTRACT_FILES) - set(texts))
    if missing_carriers:
        fail(f"external-Agent contract carrier missing: {missing_carriers}", issues)
        return

    joined = "\n".join(
        path.read_text(encoding="utf-8")
        for path in (ROOT / "docs").rglob("*.md")
        if path.is_file()
    )
    for forbidden in ("`plan.list`", "`plan.get`", "`review.linkout`"):
        if forbidden in joined:
            fail(f"ambiguous external-Agent operation alias is forbidden: {forbidden}", issues)

    required_fragments = {
        "docs/contracts/interfaces.md": (
            "The first retained protocol proof is `server.info` plus `capability.list`; the first shared CLI/inbound-MCP vertical slice is `market.package.list`.",
            "The first remote profile uses reviewed MCP Streamable HTTP.",
            "`planner.generate` creates only a tenant-local draft. It does not enroll, register, pay or submit a transaction to any external campus system.",
        ),
        "docs/contracts/permissions.md": (
            "Skill text describes when and how to call reviewed CLI/MCP operations",
            "raw USTC passwords;",
            "automatic enrollment, registration, payment, form submission or any other external campus-system mutation;",
        ),
        "docs/contracts/client-shell.md": (
            "The first retained MCP slice exposes only a reviewed `public-read` projection, initially `market.package.list`.",
            "Windows is an admitted future M80 peer target",
            "not part of the current required-target gate",
        ),
        "docs/contracts/cli.md": (
            "These commands project `server.info`, `capability.list` and `market.package.list`",
            "server-mediated browser pairing",
        ),
        "docs/features/05-headless-client-and-agent-integration.md": (
            "This is the M10/M80 client-access lane order. It does not replace the product implementation order",
        ),
        "docs/plan/modules/80-dioxus-multi-client.md": (
            "Windows is an explicitly admitted later desktop peer target but not a current required release gate",
            "automatic enrollment, registration, payment or external campus transaction submission;",
        ),
        "docs/tasks/01-execution-roadmap.md": (
            "This M10/M80 lane runs alongside—and does not reorder—the first-party product lane",
            "Windows is not in the current required-target gate.",
        ),
    }
    for rel, fragments in required_fragments.items():
        text = texts[rel]
        for fragment in fragments:
            if text.count(fragment) != 1:
                fail(
                    f"external-Agent semantic projection missing/duplicated in {rel}: {fragment!r}",
                    issues,
                )

    matrix_rows = {
        line.split("\t", 1)[0]: line.split("\t")
        for line in (ROOT / "docs/acceptance/matrix.tsv").read_text(encoding="utf-8").splitlines()[1:]
        if line.strip()
    }
    matrix_requirements = {
        "CLIENT-007": ("one operation registry",),
        "CLIENT-009": (
            "required initial host matrix",
            "does not require a Windows binary",
            "or imply Windows GUI support",
        ),
        "CLIENT-010": ("Streamable HTTP", "begin public-read", "applicable current grant state", "invalidate widened schema grants"),
    }
    for case_id, fragments in matrix_requirements.items():
        row = matrix_rows.get(case_id)
        if row is None:
            fail(f"external-Agent acceptance row missing: {case_id}", issues)
            continue
        if len(row) != 7 or row[5] != "planned":
            fail(f"external-Agent acceptance posture drift for {case_id}", issues)
            continue
        for fragment in fragments:
            if fragment not in row[2]:
                fail(
                    f"external-Agent acceptance assertion drift for {case_id}: missing {fragment!r}",
                    issues,
                )


# --- M90 CI evidence shard slice carriers ---

CI_TRANSITION_LEDGER_PATH = "docs/acceptance/ci-transition-ledger.md"
CI_V2_FIXTURE_PATH = "scripts/tests/fixtures/ci-v2.yml"
# The inert future fixture is governed by explicit semantic invariants below
# and is intentionally NOT whole-file fingerprinted: docs/acceptance/gates.md
# (Fingerprint moratorium / replacement-before-deletion) forbids new whole
# mutable CI workflow hashes when narrower semantic checks exist.
CI_V2_SHARD_RUNNER_COMMAND = "python3 scripts/run_checker_shards.py"
CI_V2_CHECKER_COMMAND = "python3 scripts/check_repo_contracts.py"
CHECKER_TEST_INVENTORY_PATH = "scripts/checker_test_inventory.json"
CHECKER_TEST_INVENTORY_SCHEMA_VERSION = "checker-test-inventory/v1"
CHECKER_TESTS_DIR_REL = "scripts/tests"
CHECKER_TEST_PATTERN = "test_*.py"
RUN_CHECKER_SHARDS_PATH = "scripts/run_checker_shards.py"
CI_TRANSITION_LEDGER_IDS = (
    "CI-TR-001",
    "CI-TR-002",
    "CI-TR-003",
    "CI-TR-004",
    "CI-TR-005",
    "CI-TR-006",
)
CI_TRANSITION_LEDGER_HEADER = (
    "| ID | Legacy invariant | Legacy carrier | Future v2 carrier | Mechanical test | Slice state |"
)
CI_TRANSITION_LEDGER_STATE_DECLARATIONS = (
    "remains the legacy/full CI authority",
    "is inert test data only",
    "No selective omission, path filtering, or package-selective Rust is active",
    "No acceptance matrix status is promoted by this slice",
    "selective activation remain later prerequisites",
)
CI_TRANSITION_LEDGER_ROW_CELLS: tuple[tuple[str, tuple[str, ...]], ...] = (
    (
        "CI-TR-001",
        (
            "active workflow exact digest remains frozen",
            f".github/workflows/ci.yml` SHA-256 `{CAMPAIGN_CI_WORKFLOW_SHA256}",
            "inert fixture is governed by explicit semantic invariants and is intentionally not whole-file fingerprinted",
            "freezes the active workflow digest and enforces the fixture's semantic invariants",
            "inert-not-active",
        ),
    ),
    (
        "CI-TR-002",
        (
            "pull_request` plus push-to-main trigger semantics remain",
            ".github/workflows/ci.yml` `on: pull_request`",
            "scripts/tests/fixtures/ci-v2.yml` preserves `on: pull_request`",
            "validates trigger semantics in both active and fixture",
            "inert-not-active",
        ),
    ),
    (
        "CI-TR-003",
        (
            "stable `rust` and `docs-and-contracts` job names remain",
            ".github/workflows/ci.yml` jobs `rust` and `docs-and-contracts`",
            "scripts/tests/fixtures/ci-v2.yml` preserves job names `rust` and `docs-and-contracts`",
            "validates job names in both active and fixture",
            "inert-not-active",
        ),
    ),
    (
        "CI-TR-004",
        (
            "full Python discovery is replaced only in the inert fixture",
            "python3 -m unittest discover",
            "python3 scripts/run_checker_shards.py --jobs 4",
            "bidirectional inventory coverage and exact-union fan-in",
            "inert-not-active",
        ),
    ),
    (
        "CI-TR-005",
        (
            "full Rust fmt/clippy/workspace-test/doctest commands remain",
            "cargo fmt --all -- --check",
            "cargo test --workspace --all-features --doc --locked",
            "validates all four Rust commands in fixture",
            "inert-not-active",
        ),
    ),
    (
        "CI-TR-006",
        (
            "repository checker remains an exact executable command",
            "python3 scripts/check_repo_contracts.py",
            "python3 scripts/check_repo_contracts.py` exactly once",
            "validates checker command presence, count, and after-runner ordering in fixture",
            "inert-not-active",
        ),
    ),
)
CI_V2_REQUIRED_ACTION_SHAS = (
    "actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803",
    "actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1",
    "dtolnay/rust-toolchain@032958afbdc797a9164d3bc0b56325c1308924a5",
    "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
)
CI_V2_REQUIRED_RUST_COMMANDS = (
    "cargo fmt --all -- --check",
    "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
    "cargo test --workspace --all-targets --all-features --locked",
    "cargo test --workspace --all-features --doc --locked",
)
CI_V2_REQUIRED_RUNNER_ARGS = (
    CI_V2_SHARD_RUNNER_COMMAND,
    "--jobs 4",
    "--timeout-seconds 1800",
    "--inventory scripts/checker_test_inventory.json",
    "--require-clean",
    "--require-runner-image-identity",
)
CI_V2_PYTHON_VERSION = "3.13.5"
CI_V2_PYPREFIX_LINE = "PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache"
RUN_CHECKER_SHARDS_REQUIRED_SUBSTRINGS = (
    "--jobs",
    "--timeout-seconds",
    "--inventory",
    "--evidence-dir",
    "--require-clean",
    "--require-runner-image-identity",
    "PYTHONPYCACHEPREFIX",
    "os.setsid",
)


def check_ci_transition_ledger(issues: list[str]) -> None:
    path = ROOT / CI_TRANSITION_LEDGER_PATH
    if not path.is_file():
        fail(f"CI transition ledger missing: {CI_TRANSITION_LEDGER_PATH}", issues)
        return
    text = path.read_text(encoding="utf-8")
    if not text.strip():
        fail(f"CI transition ledger empty: {CI_TRANSITION_LEDGER_PATH}", issues)
        return
    for declaration in CI_TRANSITION_LEDGER_STATE_DECLARATIONS:
        if declaration not in text:
            fail(f"CI transition ledger state declaration missing: {declaration[:60]!r}", issues)
    lines = text.splitlines()
    header_index = None
    for i, line in enumerate(lines):
        if line.strip() == CI_TRANSITION_LEDGER_HEADER:
            header_index = i
            break
    if header_index is None:
        fail("CI transition ledger header row missing or drifted", issues)
        return
    if header_index + 1 >= len(lines):
        fail("CI transition ledger table separator missing", issues)
        return
    separator = lines[header_index + 1].strip()
    if not all(ch in "|-" for ch in separator) or "|" not in separator:
        fail("CI transition ledger table separator malformed", issues)
    data_lines: list[str] = []
    for line in lines[header_index + 2:]:
        if line.strip().startswith("|"):
            data_lines.append(line)
        elif data_lines:
            break
    if len(data_lines) != len(CI_TRANSITION_LEDGER_IDS):
        fail(
            f"CI transition ledger row count drift: expected {len(CI_TRANSITION_LEDGER_IDS)} actual {len(data_lines)}",
            issues,
        )
    seen_ids: list[str] = []
    row_specs = {row_id: cells for row_id, cells in CI_TRANSITION_LEDGER_ROW_CELLS}
    for index, (expected_id, line) in enumerate(zip(CI_TRANSITION_LEDGER_IDS, data_lines)):
        cells = markdown_cells(line)
        if len(cells) != 6:
            fail(f"CI transition ledger row {expected_id} cell count drift: expected 6 actual {len(cells)}", issues)
            continue
        row_id = cells[0].strip()
        expected_cell = f"`{expected_id}`"
        if row_id != expected_cell:
            fail(f"CI transition ledger row {index} ID drift: expected {expected_cell} actual {row_id}", issues)
        if row_id in seen_ids:
            fail(f"CI transition ledger duplicate row ID: {row_id}", issues)
        seen_ids.append(row_id)
        for cell_index, cell in enumerate(cells):
            if not cell.strip():
                fail(f"CI transition ledger row {expected_id} cell {cell_index} is empty", issues)
        state = cells[5].strip()
        if state != "`inert-not-active`":
            fail(f"CI transition ledger row {expected_id} slice state drift: expected `inert-not-active` actual {state}", issues)
        expected_substrings = row_specs.get(expected_id)
        if expected_substrings is None:
            continue
        for cell_index, expected_substring in enumerate(expected_substrings, start=1):
            if expected_substring not in cells[cell_index]:
                fail(
                    f"CI transition ledger row {expected_id} cell {cell_index} semantic drift: "
                    f"expected substring {expected_substring[:60]!r} not in {cells[cell_index][:80]!r}",
                    issues,
                )
    if len(seen_ids) == len(CI_TRANSITION_LEDGER_IDS):
        for expected_id in CI_TRANSITION_LEDGER_IDS:
            if f"`{expected_id}`" not in seen_ids:
                fail(f"CI transition ledger missing row: {expected_id}", issues)


def _check_ci_v2_pyprefix_placement(text: str, issues: list[str]) -> None:
    """Indentation-aware proof that PYTHONPYCACHEPREFIX sits at the
    ``docs-and-contracts`` job-level ``env:`` block and nowhere else.

    The accepted shape is:
      ``  docs-and-contracts:`` (2 spaces)
      ``    env:``              (4 spaces, job-level)
      ``      PYTHONPYCACHEPREFIX: ...`` (6 spaces)

    Any other placement (workflow-level, step-level, sibling job, or duplicate)
    is rejected. Substring-only checks would miss a step-level relocation that
    preserves the literal line.
    """
    lines = text.splitlines()
    pyprefix_occurrences = [
        (idx, line)
        for idx, line in enumerate(lines)
        if CI_V2_PYPREFIX_LINE in line
    ]
    if len(pyprefix_occurrences) != 1:
        fail(
            "CI v2 fixture PYTHONPYCACHEPREFIX occurrence count drift: "
            f"expected 1 actual {len(pyprefix_occurrences)}",
            issues,
        )
        return
    pyprefix_idx, pyprefix_line = pyprefix_occurrences[0]
    pyprefix_indent = len(pyprefix_line) - len(pyprefix_line.lstrip(" "))
    if pyprefix_indent != 6:
        fail(
            f"CI v2 fixture PYTHONPYCACHEPREFIX indent drift: expected 6 spaces actual {pyprefix_indent}",
            issues,
        )
    env_idx: int | None = None
    for idx in range(pyprefix_idx - 1, -1, -1):
        candidate = lines[idx]
        stripped = candidate.lstrip(" ")
        indent = len(candidate) - len(stripped)
        if indent >= pyprefix_indent:
            continue
        if stripped == "env:":
            env_idx = idx
            break
        if stripped.startswith("env:") or stripped.startswith("steps:") or stripped.startswith("runs-on:") or stripped.startswith("timeout-minutes:"):
            break
    if env_idx is None:
        fail("CI v2 fixture PYTHONPYCACHEPREFIX has no enclosing env: block", issues)
        return
    env_line = lines[env_idx]
    env_indent = len(env_line) - len(env_line.lstrip(" "))
    if env_indent != 4:
        fail(
            f"CI v2 fixture enclosing env: indent drift: expected 4 spaces actual {env_indent}",
            issues,
        )
    job_idx: int | None = None
    for idx in range(env_idx - 1, -1, -1):
        candidate = lines[idx]
        stripped = candidate.lstrip(" ")
        indent = len(candidate) - len(stripped)
        if indent >= env_indent:
            continue
        if indent == 2 and stripped.startswith("docs-and-contracts:"):
            job_idx = idx
            break
        if indent == 2 and stripped and not stripped.startswith("#"):
            break
    if job_idx is None:
        fail(
            "CI v2 fixture PYTHONPYCACHEPREFIX env: is not under docs-and-contracts job",
            issues,
        )
    for idx, line in enumerate(lines):
        if idx == pyprefix_idx:
            continue
        if CI_V2_PYPREFIX_LINE.split(":")[0] in line and "PYTHONPYCACHEPREFIX" in line:
            fail(
                f"CI v2 fixture extra PYTHONPYCACHEPREFIX occurrence at line {idx + 1}: {line.strip()!r}",
                issues,
            )


def _ci_v2_executable_command_positions(
    text: str, command: str, *, exact_argv: bool
) -> list[int]:
    """Line offsets where ``command`` is the shell argv, not display text.

    Both an inline YAML ``run:`` payload and a block-scalar shell line are
    admitted.  ``shlex`` tokenization rejects prefixes such as ``echo`` that
    merely print the command.  The checker command is exact; the shard runner
    admits its taskbook-bound arguments after the executable prefix.
    """
    expected = shlex.split(command)
    positions: list[int] = []
    offset = 0
    for line in text.splitlines(keepends=True):
        candidate = line.strip()
        if not candidate or candidate.startswith("#"):
            offset += len(line)
            continue
        if candidate.startswith("run:"):
            candidate = candidate.removeprefix("run:").strip()
            if candidate in {"|", "|-", "|+", ">", ">-", ">+"}:
                offset += len(line)
                continue
        try:
            argv = shlex.split(candidate)
        except ValueError:
            offset += len(line)
            continue
        matches = argv == expected if exact_argv else argv[: len(expected)] == expected
        if matches:
            positions.append(offset)
        offset += len(line)
    return positions


def check_ci_v2_inert_fixture(issues: list[str]) -> None:
    path = ROOT / CI_V2_FIXTURE_PATH
    if not path.is_file():
        fail(f"CI v2 inert fixture missing: {CI_V2_FIXTURE_PATH}", issues)
        return
    text = path.read_text(encoding="utf-8")
    if not text.strip():
        fail(f"CI v2 inert fixture empty: {CI_V2_FIXTURE_PATH}", issues)
        return
    if "on:\n  pull_request:" not in text:
        fail("CI v2 fixture missing pull_request trigger", issues)
    if "push:\n    branches:" not in text or "- main" not in text:
        fail("CI v2 fixture missing push-to-main trigger", issues)
    if "  rust:" not in text:
        fail("CI v2 fixture missing rust job", issues)
    if "  docs-and-contracts:" not in text:
        fail("CI v2 fixture missing docs-and-contracts job", issues)
    for action_sha in CI_V2_REQUIRED_ACTION_SHAS:
        if action_sha not in text:
            fail(f"CI v2 fixture missing pinned action SHA: {action_sha}", issues)
    if "toolchain: 1.97.1" not in text:
        fail("CI v2 fixture missing Rust toolchain 1.97.1", issues)
    if "components: rustfmt, clippy" not in text:
        fail("CI v2 fixture missing rustfmt/clippy components", issues)
    if f"python-version: {CI_V2_PYTHON_VERSION}" not in text:
        fail(f"CI v2 fixture missing Python {CI_V2_PYTHON_VERSION}", issues)
    if CI_V2_PYPREFIX_LINE not in text:
        fail("CI v2 fixture missing job-level PYTHONPYCACHEPREFIX", issues)
    _check_ci_v2_pyprefix_placement(text, issues)
    for command in CI_V2_REQUIRED_RUST_COMMANDS:
        if command not in text:
            fail(f"CI v2 fixture missing Rust command: {command}", issues)
    for arg in CI_V2_REQUIRED_RUNNER_ARGS:
        if arg not in text:
            fail(f"CI v2 fixture missing runner argument: {arg}", issues)
    # CI-TR-006: the executable shard-runner command and the exact repository
    # checker command must each appear exactly once, with the checker after
    # the full shard runner. Commented/display-only copies are not executable
    # and do not count.
    runner_positions = _ci_v2_executable_command_positions(
        text, CI_V2_SHARD_RUNNER_COMMAND, exact_argv=False
    )
    if len(runner_positions) != 1:
        fail(
            f"CI v2 fixture shard runner command count drift: expected 1 actual {len(runner_positions)}",
            issues,
        )
    checker_positions = _ci_v2_executable_command_positions(
        text, CI_V2_CHECKER_COMMAND, exact_argv=True
    )
    if len(checker_positions) != 1:
        fail(
            f"CI v2 fixture checker command count drift: expected 1 actual {len(checker_positions)}",
            issues,
        )
    if runner_positions and checker_positions and checker_positions[0] < runner_positions[0]:
        fail(
            "CI v2 fixture ordering drift: repository checker must run after the shard runner",
            issues,
        )
    if "if: ${{ always() }}" not in text:
        fail("CI v2 fixture missing always() evidence upload condition", issues)


def _live_checker_test_ids() -> list[str]:
    """Discover the live checker suite exactly as the shard runner does.

    Leaf test IDs are collected with multiplicity; any duplicate leaf ID, any
    ``_FailedTest`` import failure, and an empty discovery all raise
    ValueError so the caller fails closed with a typed diagnostic instead of
    a traceback.
    """
    loader = unittest.TestLoader()
    suite = loader.discover(
        start_dir=str(ROOT / CHECKER_TESTS_DIR_REL), pattern=CHECKER_TEST_PATTERN
    )
    stack: list[object] = [suite]
    ids: list[str] = []
    while stack:
        item = stack.pop()
        if isinstance(item, unittest.TestCase):
            test_id = item.id()
            if "_FailedTest" in test_id:
                raise ValueError(f"discovery produced _FailedTest: {test_id}")
            ids.append(test_id)
        elif isinstance(item, unittest.TestSuite):
            stack.extend(item)
    if not ids:
        raise ValueError("discovery produced zero tests")
    seen: set[str] = set()
    duplicates: set[str] = set()
    for test_id in ids:
        if test_id in seen:
            duplicates.add(test_id)
        else:
            seen.add(test_id)
    if duplicates:
        raise ValueError(
            f"discovery produced duplicate test IDs (leaf multiplicity collapsed by set): "
            f"{sorted(duplicates)}"
        )
    return sorted(ids)


def check_checker_test_inventory(issues: list[str]) -> None:
    path = ROOT / CHECKER_TEST_INVENTORY_PATH
    if not path.is_file():
        fail(f"checker test inventory missing: {CHECKER_TEST_INVENTORY_PATH}", issues)
        return
    text = path.read_text(encoding="utf-8")
    if not text.strip():
        fail(f"checker test inventory empty: {CHECKER_TEST_INVENTORY_PATH}", issues)
        return
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        fail(f"checker test inventory invalid JSON: {exc}", issues)
        return
    if not isinstance(data, dict):
        fail("checker test inventory top-level value must be an object", issues)
        return
    if data.get("schema_version") != CHECKER_TEST_INVENTORY_SCHEMA_VERSION:
        fail(
            f"checker test inventory schema_version drift: expected {CHECKER_TEST_INVENTORY_SCHEMA_VERSION!r} actual {data.get('schema_version')!r}",
            issues,
        )
    test_ids = data.get("test_ids")
    if not isinstance(test_ids, list):
        fail("checker test inventory test_ids must be a list", issues)
        return
    for entry in test_ids:
        if not isinstance(entry, str) or not entry:
            # Fail closed on the first malformed entry: set()/sorted() below
            # raise TypeError on unhashable/mixed-type entries, so returning
            # after recording the typed issue is the only traceback-free path.
            fail(f"checker test inventory test_ids contains non-string/empty entry: {entry!r}", issues)
            return
    if len(test_ids) != len(set(test_ids)):
        fail("checker test inventory test_ids contains duplicates", issues)
    if test_ids != sorted(test_ids):
        fail("checker test inventory test_ids must be sorted", issues)
    expected_count = data.get("expected_count")
    if not isinstance(expected_count, int) or expected_count != len(test_ids):
        fail(f"checker test inventory expected_count drift: expected {len(test_ids)} actual {expected_count!r}", issues)
    for test_id in test_ids:
        if "_FailedTest" in test_id:
            fail(f"checker test inventory contains _FailedTest ID: {test_id}", issues)
    # Structural validation is necessary but not sufficient: a stale inventory
    # that is internally consistent would pass active CI and later break the
    # inert sharded workflow. Synchronize the inventory against live discovery
    # with the same leaf-ID/duplicate-ID/_FailedTest semantics as the runner.
    try:
        live_ids = _live_checker_test_ids()
    except Exception as exc:  # noqa: BLE001 - discovery failure must be a typed issue, not a traceback
        fail(
            f"checker test inventory live discovery failed: {type(exc).__name__}: {exc}",
            issues,
        )
        return
    missing_from_live = sorted(set(test_ids) - set(live_ids))
    unexpected_in_live = sorted(set(live_ids) - set(test_ids))
    if missing_from_live or unexpected_in_live:
        fail(
            f"checker test inventory/live discovery drift: "
            f"missing_from_live={missing_from_live} unexpected_in_live={unexpected_in_live}",
            issues,
        )


def check_run_checker_shards_runner(issues: list[str]) -> None:
    path = ROOT / RUN_CHECKER_SHARDS_PATH
    if not path.is_file():
        fail(f"checker shard runner missing: {RUN_CHECKER_SHARDS_PATH}", issues)
        return
    text = path.read_text(encoding="utf-8")
    if not text.strip():
        fail(f"checker shard runner empty: {RUN_CHECKER_SHARDS_PATH}", issues)
        return
    for substring in RUN_CHECKER_SHARDS_REQUIRED_SUBSTRINGS:
        if substring not in text:
            fail(f"checker shard runner missing required substring: {substring!r}", issues)


EXPECTED_MAIN_CALLS = (
    "check_key_files_present_and_nonempty(issues)",
    "check_campaign_authorization(issues)",
    "check_docs_topology(issues)",
    "check_design_packets(issues)",
    "check_no_retired_docs_references(issues)",
    "check_markdown_links(issues)",
    "check_no_obvious_secrets(issues)",
    "check_market(issues)",
    "check_course_fixture(issues)",
    "check_invocation_fixtures(issues)",
    "check_agent_plugin_dependency_direction(issues)",
    "check_acceptance_matrix(issues)",
    "check_acceptance_catalog(issues)",
    "check_rust_doctest_gate(issues)",
    "check_platform_identity_grammar_authority(issues)",
    "check_platform_authority_implementation(issues)",
    "check_platform_identity_implementation(issues)",
    "check_platform_session_contract(issues)",
    "check_platform_session_implementation(issues)",
    "check_p1_source_revision_contract(issues)",
    "check_p1_source_registry_implementation(issues)",
    "check_m60_b2_packet_digest(issues)",
    "check_external_agent_access_contract(issues)",
    "check_module_registry(issues)",
    "check_s0_architecture_review(issues)",
    "check_ci_transition_ledger(issues)",
    "check_ci_v2_inert_fixture(issues)",
    "check_checker_test_inventory(issues)",
    "check_run_checker_shards_runner(issues)",
    "check_source_sensitive_guard_registry(issues)",
)


# --- Source-sensitive guard registry (R1: fingerprint moratorium enforcement) ---
#
# Every top-level checker function that reads source bytes/text (via
# .read_text() or .read_bytes()) is classified as source-sensitive and pinned
# here by a location-insensitive normalized AST digest
# (ast.dump(..., include_attributes=False) hashed with SHA-256).
#
# This makes the fingerprint moratorium in docs/acceptance/gates.md
# §Fingerprint moratorium executable: a new lexical guard added inside any
# registered function changes its AST digest and fails CI, rather than
# silently false-greening.
#
# Guard family for all entries: source-text-carrier.
# Failure mode for all entries: fail-closed.
# Scanned paths: each function's body names the ROOT-relative paths it reads.
#
# To legitimately change a registered function's body, update its digest here
# in the same reviewed slice.  Only status "active" is admitted in this
# campaign; any other status fails closed.  Replacement transitions (removing
# a function from the moratorium in favor of an authority-native mechanism)
# require a future separately reviewed mechanism with owner/mechanism/evidence
# validation; no replacement path is implemented here.

CHECKER_SOURCE_REL = "scripts/check_repo_contracts.py"

SOURCE_SENSITIVE_GUARD_REGISTRY: dict[str, dict[str, str]] = {
    "_check_bound_rust_test_file": {"digest": "7822293db5dfa565407409bdba528c656c53ce28f0af83efdc80176aa55069f9", "status": "active"},
    "_check_coverage_matrix_m60_projection": {"digest": "c850fe6abc43b6ca20c79ad5a7127ee8ff4d5dc470afb7a0a44bbf8882ffc65d", "status": "active"},
    "_check_m60_blueprint_status_projection": {"digest": "e0282bf17f30094ce634687b110cbd5bcd9aa40e4925c5cba87fa2305468559f", "status": "active"},
    "_check_m60_roadmap_section_status": {"digest": "a16c63013807a3c3ab1eaadc6390df41c8538626aac19484be6b9b60e1b47213", "status": "active"},
    "_check_market_grant_surface": {"digest": "ff8dcbbfb94b444d72cfba57bbb04fb9260ab1396302dbb0568c347afb372b69", "status": "active"},
    "_check_market_installation_surface": {"digest": "1c68b54aba48bc26905ccbe84f3a7036bc7db9b0dddb981e88b1abfe1f5b4c84", "status": "active"},
    "_check_market_update_surface": {"digest": "390fffb8fddf706548a040b8fa8cf43246b7f62b026ef7e9a490cf8e3c46f8f5", "status": "active"},
    "_check_module_boundaries_transport_projection": {"digest": "564a9530d71a508568c0b263481ffc6996b3e2a61341681e0dc1702b30a50126", "status": "active"},
    "_check_source_import_owner_projection": {"digest": "bfaf91dfccb1296823c5f87ed025db7fa2ce3f3bb66b6bf3c739ef57e746adb4", "status": "active"},
    "_check_source_retrieval_contract_metadata": {"digest": "b0b37ecb4ed41d3cdcf708198df8eee392c028862ee0cd4b7ee66d41a8c77dc5", "status": "active"},
    "_check_source_retrieval_lifecycle_precondition": {"digest": "241fe2966abd080b3bdd6d1bc0d92774298c89968064de18e2d39646f9c07f4f", "status": "active"},
    "_check_source_retrieval_transport_boundary_projection": {"digest": "fdc12d98f975664db68ea051fc4be7da4803b70f951a33396b018a8bc012689d", "status": "active"},
    "_platform_identity_classify_procedure": {"digest": "735d6e8dc9cccd710699ffbc51a926589069920861ea686ba2bd21c1ffe0fbcf", "status": "active"},
    "_platform_identity_contract_grammar": {"digest": "ff1d942e93247a5565a8ed08d6c65a4c01e6f057e6fe6e8d6ab5ac9a39b1b548", "status": "active"},
    "_platform_identity_effective_bound": {"digest": "16d9d31dfb7d439566507a310148862e44de238f70df4e8043326ff052464a36", "status": "active"},
    "_platform_identity_effective_guard": {"digest": "086f7fd4f7c232a6bec9ae9d416470f3d001ffdb63a2075aa8d93c4ecd82e3e2", "status": "active"},
    "_platform_identity_helper_resolution": {"digest": "412e931dad6a68d8deb1c6ea1c81cd400f44ad8f2314b88249fc8187dabdff38", "status": "active"},
    "_platform_identity_semantic_literals": {"digest": "58113172f1b052d099af76cbb82478646b7a9e331f3870367f2b064340cda086", "status": "active"},
    "_platform_identity_sweep_carriers": {"digest": "3203275b5308a3324782847e9ec7f4217f23a7528aaf6673110666e313f01ef4", "status": "active"},
    "check_acceptance_catalog": {"digest": "5b19c84422fe487470642f121348b895248d38e612d39f4bb4e4ea03d0f06b8c", "status": "active"},
    "check_acceptance_matrix": {"digest": "35fad9655eb38702f34c65a3aa6209b79f66256b20183f5c20a67c049e9cd4bd", "status": "active"},
    "check_agent_plugin_dependency_direction": {"digest": "eb43b13ee8c08d397671619ea288b221eb2dc2620066869f8ea9c31c66ac5e50", "status": "active"},
    "check_campaign_acceptance_bindings": {"digest": "60025429af3b75fe7de3adb4043988509f76f20db66dd383888a10b9f04522c2", "status": "active"},
    "check_campaign_authorization": {"digest": "7a1184e0d9f133a48a79c9ba6f66bc912252fa1732c7b13faa0b99373466571a", "status": "active"},
    "check_campaign_taskbook_state": {"digest": "85b30689fd0ae05695c337d8808e919f69639ace8d98643c90a4b47522fe2d65", "status": "active"},
    "check_cargo_dependency_sources": {"digest": "0f645288c48f56eb3c8282f5fe9b9c0e379ae8ff82e1c06f64f740278a77ad7d", "status": "active"},
    "check_checker_test_inventory": {"digest": "7c92107dae4d99854ed8680abbf2b5648f0d20fe6ddaac8c42d2ed5076ca98ed", "status": "active"},
    "check_ci_transition_ledger": {"digest": "ac88ca9043508ae22fe535334368e80aa02c701454c4d085fcd3cc1343d6d1f9", "status": "active"},
    "check_ci_v2_inert_fixture": {"digest": "f6780f6177d741126b3818bb9199a6b55dcd28242b95408122f50c28a41ba3c3", "status": "active"},
    "check_design_packets": {"digest": "743558920d241f208a6a10c7264b70f5fa4b80a77321472d750b05e3a2bf144c", "status": "active"},
    "check_external_agent_access_contract": {"digest": "b5f042f9d195b8a82d15059da493f2ecb92f372aa79733a7d0fb598c3881bd79", "status": "active"},
    "check_invocation_fixtures": {"digest": "8aecb5e13723a1eac615e534f5fad317a5cf7b7d4fe29c406d7272be5e0cc454", "status": "active"},
    "check_key_files_present_and_nonempty": {"digest": "556c93bd959c3dbc31fa6e3b8f25a1ac3ff8a66ae1909110ad87690a224b4157", "status": "active"},
    "check_m60_b2_packet_digest": {"digest": "eb0e11c0b609edfb0f2c016010119a7a821e078b547bdd0cf91ad477802a6bd4", "status": "active"},
    "check_markdown_links": {"digest": "8094c14c99d77223442ef4ea92d214dd31860aa3744b2c35960b36383db473b7", "status": "active"},
    "check_module_registry": {"digest": "d35ade46455588776b2d380a78f411c30621830f3fdeb8139f8a49153cadd4d3", "status": "active"},
    "check_no_obvious_secrets": {"digest": "43072df6164d2bc92e69015291368559f593a49fd2b39d813de034e5f50b2f79", "status": "active"},
    "check_no_retired_docs_references": {"digest": "64599f57afeb2ce5fb60ccaac747cfc8ddaefc6feaf2536ad32eb0c42afd1114", "status": "active"},
    "check_p1_source_registry_implementation": {"digest": "7e04d788b46c0844256dc4184d667381206e544663de3b630dd4d50feefe4c1d", "status": "active"},
    "check_p1_source_revision_contract": {"digest": "f413cb4f58e9e0d17205583ca69105d3bc171e5dfb5cc1031cc4debc135fa992", "status": "active"},
    "check_platform_authority_implementation": {"digest": "64b767f09d0d29268af33e15bdf60d6fd879b61365abd45c1be2caa2319b92f4", "status": "active"},
    "check_platform_core_manifest": {"digest": "3082cf7fedfaf39080d287a036c8875f762751bb8832121ca2d7cd81d5947d62", "status": "active"},
    "check_platform_identity_implementation": {"digest": "ba1d808291f8faa3d3a9acbc16a71854af2c8bc461115432c49a4aeb3ec23d72", "status": "active"},
    "check_platform_session_contract": {"digest": "e3a2e5ef5ca953bdf2739ac3072df8bcfed0ebece4a893f52980fb7ca3b15c1b", "status": "active"},
    "check_platform_session_implementation": {"digest": "f1a25036ae6940b332c258af80f2e23815071ca19cb1d5db79d7a4f8b844be8f", "status": "active"},
    "check_rust_doctest_gate": {"digest": "372200f9ce289b3af148b7e7001408498b3d817098d7013692f016b766d2ec58", "status": "active"},
    "check_rust_lexical_corpus": {"digest": "ac8c09ce53ca4949c8e5c61eca0a958301735d7cdb2dc0ecd00eb3472a0e2aef", "status": "active"},
    "check_run_checker_shards_runner": {"digest": "c6abd4f4d049cb1836f333badf20d0d08bec3ea319a0549812f076b8b419de39", "status": "active"},
    "check_s0_architecture_review": {"digest": "4073abe0147a1c30653047aedfe168b8ed6684281f4c445c4ee803a428b8760e", "status": "active"},
    "exact_marked_block": {"digest": "71fc49a8490fad3b9bc262667c0fb91f1e00fe554b8d1fe04495fdf1c72c80aa", "status": "active"},
    "load_json": {"digest": "e4537fd821653c24dee9db7592a94a514962b69e839ed24399af78f773dab0db", "status": "active"},
    "parse_markdown_table": {"digest": "8a76fe3b1a30cbc620fce319e66105b37ef7520b2d9e56f17f45f815c40ccdaf", "status": "active"},
}

SOURCE_SENSITIVE_SELF_FUNCTION = "check_source_sensitive_guard_registry"


def _classify_source_sensitive_functions(source: str) -> dict[str, str]:
    """Return {function_name: normalized_ast_digest} for every top-level
    function whose body contains a ``.read_text()`` or ``.read_bytes()`` call.
    """
    tree = ast.parse(source)
    classified: dict[str, str] = {}
    for node in ast.iter_child_nodes(tree):
        if not isinstance(node, ast.FunctionDef):
            continue
        for sub in ast.walk(node):
            if (
                isinstance(sub, ast.Call)
                and isinstance(sub.func, ast.Attribute)
                and sub.func.attr in ("read_text", "read_bytes")
            ):
                dump = ast.dump(node, include_attributes=False)
                classified[node.name] = hashlib.sha256(dump.encode("utf-8")).hexdigest()
                break
    return classified


def check_source_sensitive_guard_registry(issues: list[str]) -> None:
    """Fail-closed AST governance for source-sensitive checker functions.

    Enforces docs/acceptance/gates.md §Fingerprint moratorium: every top-level
    checker function that reads source bytes/text must be explicitly registered
    in SOURCE_SENSITIVE_GUARD_REGISTRY with status ``active`` and a
    normalized-AST SHA-256 digest that matches its current body.  Only
    ``active`` is admitted; any other status fails closed.  Replacement
    transitions require a future separately reviewed mechanism with
    owner/mechanism/evidence validation; no replacement path is implemented.
    """
    checker_path = ROOT / CHECKER_SOURCE_REL
    if not checker_path.is_file():
        fail("source-sensitive guard registry: checker source missing", issues)
        return
    source = checker_path.read_text(encoding="utf-8")
    try:
        classified = _classify_source_sensitive_functions(source)
    except SyntaxError as exc:
        fail(
            f"source-sensitive guard registry: checker source unparseable: {exc}",
            issues,
        )
        return

    # Every classified function must be registered, active, and digest-exact,
    # except the self-function (it reads the checker's own source to verify
    # the registry; pinning its digest would create a bootstrap paradox).
    classified.pop(SOURCE_SENSITIVE_SELF_FUNCTION, None)
    for name, digest in classified.items():
        entry = SOURCE_SENSITIVE_GUARD_REGISTRY.get(name)
        if entry is None:
            fail(
                f"source-sensitive guard registry: unregistered source-sensitive "
                f"function {name!r} (add it to SOURCE_SENSITIVE_GUARD_REGISTRY "
                f"with status 'active' and its current digest)",
                issues,
            )
            continue
        if entry["status"] != "active":
            fail(
                f"source-sensitive guard registry: {name!r} has status "
                f"{entry['status']!r}; only 'active' is admitted (replacement "
                f"transitions require a future separately reviewed mechanism)",
                issues,
            )
            continue
        if entry["digest"] != digest:
            fail(
                f"source-sensitive guard registry: body digest mismatch for "
                f"{name!r} (expected {entry['digest'][:16]}, got {digest[:16]}); "
                f"update SOURCE_SENSITIVE_GUARD_REGISTRY in the same reviewed slice",
                issues,
            )

    # Every registered function must be active and still classified (still
    # read source).  A non-active status or a function that lost its carrier
    # without a reviewed replacement is a silent weakening.
    for name, entry in SOURCE_SENSITIVE_GUARD_REGISTRY.items():
        if entry["status"] != "active":
            fail(
                f"source-sensitive guard registry: registered function {name!r} "
                f"has status {entry['status']!r}; only 'active' is admitted",
                issues,
            )
            continue
        if name not in classified:
            fail(
                f"source-sensitive guard registry: registered active function "
                f"{name!r} is no longer source-sensitive (reviewed removal "
                f"requires a future separately reviewed replacement mechanism)",
                issues,
            )


def main() -> int:
    issues: list[str] = []
    check_key_files_present_and_nonempty(issues)
    check_campaign_authorization(issues)
    check_docs_topology(issues)
    check_design_packets(issues)
    check_no_retired_docs_references(issues)
    check_markdown_links(issues)
    check_no_obvious_secrets(issues)
    check_market(issues)
    check_course_fixture(issues)
    check_invocation_fixtures(issues)
    check_agent_plugin_dependency_direction(issues)
    check_acceptance_matrix(issues)
    check_acceptance_catalog(issues)
    check_rust_doctest_gate(issues)
    check_platform_identity_grammar_authority(issues)
    check_platform_authority_implementation(issues)
    check_platform_identity_implementation(issues)
    check_platform_session_contract(issues)
    check_platform_session_implementation(issues)
    check_p1_source_revision_contract(issues)
    check_p1_source_registry_implementation(issues)
    check_m60_b2_packet_digest(issues)
    check_external_agent_access_contract(issues)
    check_module_registry(issues)
    check_s0_architecture_review(issues)
    check_ci_transition_ledger(issues)
    check_ci_v2_inert_fixture(issues)
    check_checker_test_inventory(issues)
    check_run_checker_shards_runner(issues)
    check_source_sensitive_guard_registry(issues)
    if issues:
        print("contract-check: FAIL")
        for issue in issues:
            print(f"- {issue}")
        return 1
    print("contract-check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
