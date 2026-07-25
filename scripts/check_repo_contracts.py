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
    "docs/acceptance/gates.md",
    "docs/acceptance/matrix.tsv",
    "docs/acceptance/platform-baseline.md",
    "docs/acceptance/public-readiness.md",
    "docs/adr/0006-three-default-first-party-plugins.md",
    "docs/adr/0007-finite-agent-harness.md",
    "docs/adr/0008-agent-plugin-tool-boundary.md",
    "docs/adr/0009-dioxus-multi-client-shell.md",
    "docs/overview/architecture.md",
    "docs/tasks/00-module-work-policy.md",
    "docs/tasks/01-execution-roadmap.md",
    "docs/tasks/02-s0-architecture-review.md",
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
    "docs/contracts/module-boundaries.md",
    "docs/contracts/permissions.md",
    "docs/contracts/plugin-package.md",
    "docs/contracts/source-import.md",
    "market/review-policy/first-party.md",
    "market/fixtures/course-planning/README.md",
    "market/fixtures/course-planning/minimal-v0.json",
]


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
    check_module_registry(issues)
    check_s0_architecture_review(issues)
    if issues:
        print("contract-check: FAIL")
        for issue in issues:
            print(f"- {issue}")
        return 1
    print("contract-check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
