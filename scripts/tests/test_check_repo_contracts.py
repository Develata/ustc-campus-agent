from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
import unittest
from collections.abc import Callable
from pathlib import Path
from typing import cast

REPO_ROOT = Path(__file__).resolve().parents[2]
CHECKER_PATH = REPO_ROOT / "scripts/check_repo_contracts.py"
SPEC = importlib.util.spec_from_file_location("check_repo_contracts", CHECKER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {CHECKER_PATH}")
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


class MarketContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "market", self.root / "market")
        shutil.copytree(REPO_ROOT / "plugins", self.root / "plugins")
        shutil.copytree(REPO_ROOT / "docs/contracts", self.root / "docs/contracts")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_market(self) -> list[str]:
        issues: list[str] = []
        checker.check_market(issues)
        return issues

    def manifest_path(self, package_id: str) -> Path:
        return self.root / "market/packages" / package_id / "package.json"

    def load_manifest(self, package_id: str) -> dict[str, object]:
        return json.loads(self.manifest_path(package_id).read_text(encoding="utf-8"))

    def write_manifest(self, package_id: str, manifest: dict[str, object]) -> None:
        self.manifest_path(package_id).write_text(
            json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

    def test_three_default_first_party_manifests_pass(self) -> None:
        self.assertEqual(self.check_market(), [])

    def test_missing_default_package_fails_closed(self) -> None:
        shutil.rmtree(self.root / "market/packages/ustc.change-radar")
        self.assertTrue(
            any("default first-party package identity drift" in issue for issue in self.check_market())
        )

    def test_planned_package_cannot_claim_components(self) -> None:
        package_id = "ustc.affairs-navigator"
        manifest = self.load_manifest(package_id)
        manifest["components"] = [
            {
                "type": "NativeRustComponent",
                "path": "apps/nonexistent",
            }
        ]
        self.write_manifest(package_id, manifest)
        self.assertTrue(
            any("planned package must not claim components" in issue for issue in self.check_market())
        )

    def test_default_package_cannot_auto_grant_private_capability(self) -> None:
        package_id = "ustc.opportunity-graph"
        manifest = self.load_manifest(package_id)
        capabilities = cast(list[str], manifest["capabilities"])
        capabilities.append("user.own_academic_snapshot.read")
        self.write_manifest(package_id, manifest)
        self.assertTrue(
            any("capability is not auto-grant-eligible" in issue for issue in self.check_market())
        )

    def test_default_package_version_is_exact(self) -> None:
        package_id = "ustc.affairs-navigator"
        manifest = self.load_manifest(package_id)
        manifest["version"] = "0.2.0"
        self.write_manifest(package_id, manifest)

        self.assertTrue(
            any("default package version drift" in issue for issue in self.check_market())
        )

    def test_default_package_status_is_exact(self) -> None:
        package_id = "ustc.opportunity-graph"
        manifest = self.load_manifest(package_id)
        manifest["implementationStatus"] = "planned"
        self.write_manifest(package_id, manifest)

        self.assertTrue(
            any(
                "default package implementationStatus drift" in issue
                for issue in self.check_market()
            )
        )

    def test_default_package_capabilities_are_exact(self) -> None:
        package_id = "ustc.change-radar"
        manifest = self.load_manifest(package_id)
        manifest["capabilities"] = []
        self.write_manifest(package_id, manifest)

        self.assertTrue(
            any("default package capability set drift" in issue for issue in self.check_market())
        )

    def test_safe_non_first_party_package_is_allowed(self) -> None:
        package_id = "community.example"
        package_directory = self.root / "market/packages" / package_id
        package_directory.mkdir()
        self.write_manifest(
            package_id,
            {
                "id": package_id,
                "version": "0.1.0",
                "publisher": "community",
                "tier": "VerifiedCommunityText",
                "displayName": "Community Example",
                "implementationStatus": "planned",
                "installPolicy": {
                    "class": "UserInstalledPlugin",
                    "defaultInstalled": False,
                    "defaultEnabled": False,
                    "userDisableAllowed": True,
                },
                "components": [],
                "capabilities": ["campus.public_rules.read"],
                "sourcePolicy": {
                    "officialSources": "approved snapshots only",
                    "personalData": "none",
                },
            },
        )

        self.assertEqual(self.check_market(), [])

    def test_component_symlink_cannot_escape_repository(self) -> None:
        package_id = "ustc.opportunity-graph"
        manifest = self.load_manifest(package_id)
        with tempfile.TemporaryDirectory() as outside_directory:
            outside_file = Path(outside_directory) / "component.md"
            outside_file.write_text("outside\n", encoding="utf-8")
            (self.root / "escape.md").symlink_to(outside_file)
            manifest["components"] = [
                {
                    "type": "SkillComponent",
                    "path": "escape.md",
                }
            ]
            self.write_manifest(package_id, manifest)
            self.assertTrue(
                any("component path missing or unsafe" in issue for issue in self.check_market())
            )

    def test_default_package_must_state_personal_data_scope(self) -> None:
        package_id = "ustc.change-radar"
        manifest = self.load_manifest(package_id)
        source_policy = cast(dict[str, str], manifest["sourcePolicy"])
        source_policy.pop("personalData")
        self.write_manifest(package_id, manifest)
        self.assertTrue(
            any("must state personalData scope" in issue for issue in self.check_market())
        )


class DocsTopologyContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_docs_topology(self) -> list[str]:
        issues: list[str] = []
        checker.check_docs_topology(issues)
        return issues

    def check_key_files(self) -> list[str]:
        issues: list[str] = []
        checker.check_key_files_present_and_nonempty(issues)
        return issues

    def test_current_docs_topology_passes(self) -> None:
        self.assertEqual(self.check_docs_topology(), [])

    def test_invocation_contract_is_a_registered_nonempty_key_file(self) -> None:
        issues = self.check_key_files()
        self.assertFalse(any("invocation-resolution.md" in issue for issue in issues))

    def test_missing_invocation_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/invocation-resolution.md"
        path.unlink()
        self.assertIn(
            "key file missing: docs/contracts/invocation-resolution.md",
            self.check_key_files(),
        )

    def test_empty_invocation_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/invocation-resolution.md"
        path.write_text(" \n", encoding="utf-8")
        self.assertIn(
            "key file empty: docs/contracts/invocation-resolution.md",
            self.check_key_files(),
        )

    def test_unregistered_current_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/example-current.md"
        path.write_text("# Example current contract\n", encoding="utf-8")
        self.assertIn(
            "current contract not registered as key file: docs/contracts/example-current.md",
            self.check_key_files(),
        )

    def test_retired_operations_directory_is_rejected(self) -> None:
        (self.root / "docs/operations").mkdir()
        (self.root / "docs/operations/personal-backup.md").write_text(
            "personal backup notes\n",
            encoding="utf-8",
        )
        self.assertTrue(
            any("documentation directory topology drift" in issue for issue in self.check_docs_topology())
        )

    def test_unclassified_root_document_is_rejected(self) -> None:
        (self.root / "docs/misc.md").write_text("unclassified\n", encoding="utf-8")
        self.assertTrue(
            any("documentation root-file topology drift" in issue for issue in self.check_docs_topology())
        )

    def test_retired_docs_reference_outside_markdown_is_rejected(self) -> None:
        codeowners = self.root / ".github/CODEOWNERS"
        codeowners.parent.mkdir()
        codeowners.write_text("/docs/architecture/ @owner\n", encoding="utf-8")
        issues: list[str] = []
        checker.check_no_retired_docs_references(issues)
        self.assertTrue(any("retired documentation path reference" in issue for issue in issues))

    def test_unknown_acceptance_status_is_rejected(self) -> None:
        matrix_path = self.root / "docs/acceptance/matrix.tsv"
        matrix = matrix_path.read_text(encoding="utf-8")
        matrix_path.write_text(
            matrix.replace("\tplanned\t", "\tpassed\t", 1),
            encoding="utf-8",
        )
        issues: list[str] = []
        checker.check_acceptance_matrix(issues)
        self.assertTrue(any("unknown acceptance status" in issue for issue in issues))

    def test_duplicate_long_horizon_case_id_is_rejected(self) -> None:
        catalog_path = self.root / "docs/acceptance/platform-baseline.md"
        with catalog_path.open("a", encoding="utf-8") as catalog:
            catalog.write("\n| `FP-001` | duplicate | rust-unit | PR |\n")
        issues: list[str] = []
        checker.check_acceptance_catalog(issues)
        self.assertTrue(any("duplicate long-horizon acceptance case IDs" in issue for issue in issues))

    def test_stable_active_case_must_exist_in_catalog(self) -> None:
        matrix_path = self.root / "docs/acceptance/matrix.tsv"
        matrix = matrix_path.read_text(encoding="utf-8")
        matrix_path.write_text(matrix.replace("FP-015", "FP-999", 1), encoding="utf-8")
        issues: list[str] = []
        checker.check_acceptance_catalog(issues)
        self.assertTrue(any("active case missing from long-horizon catalog" in issue for issue in issues))

    def test_active_agent_case_must_exist_in_catalog(self) -> None:
        matrix_path = self.root / "docs/acceptance/matrix.tsv"
        matrix = matrix_path.read_text(encoding="utf-8")
        matrix_path.write_text(matrix.replace("AGENT-002", "AGENT-999", 1), encoding="utf-8")
        issues: list[str] = []
        checker.check_acceptance_catalog(issues)
        self.assertTrue(any("active case missing from long-horizon catalog" in issue for issue in issues))


class ModuleRegistryContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_module_registry(self) -> list[str]:
        issues: list[str] = []
        checker.check_module_registry(issues)
        return issues

    def remove_table_row(self, rel: str, prefix: str) -> None:
        path = self.root / rel
        rows = path.read_text(encoding="utf-8").splitlines()
        path.write_text(
            "\n".join(row for row in rows if not row.startswith(prefix)) + "\n",
            encoding="utf-8",
        )

    def test_current_module_registry_passes(self) -> None:
        self.assertEqual(self.check_module_registry(), [])

    def test_missing_blueprint_fails_closed(self) -> None:
        (self.root / checker.MODULE_BLUEPRINTS["M51"]).unlink()
        issues = self.check_module_registry()
        self.assertTrue(any("module blueprint path set drift" in issue for issue in issues), issues)

    def test_unregistered_blueprint_fails_closed(self) -> None:
        path = self.root / "docs/plan/modules/99-unregistered.md"
        path.write_text(
            "# Unregistered\n\n- `Module ID`: `M99`\n"
            "- `Implementation State`: `planned`\n",
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(any("module blueprint path set drift" in issue for issue in issues), issues)

    def test_duplicate_module_map_id_fails_closed(self) -> None:
        path = self.root / "docs/plan/modules/00-module-map.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace(
                "| `M10` | Application Ingress Host",
                "| `M00` | Application Ingress Host",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(any("duplicate module ID in module map: M00" in issue for issue in issues), issues)

    def test_blueprint_module_id_mismatch_fails_closed(self) -> None:
        path = self.root / checker.MODULE_BLUEPRINTS["M10"]
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace("`Module ID`: `M10`", "`Module ID`: `M11`", 1),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(any("module blueprint ID drift" in issue for issue in issues), issues)

    def test_unknown_module_state_key_fails_closed(self) -> None:
        path = self.root / "docs/plan/modules/00-module-map.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace("| `M00` | Platform Control and Identity | `planned` |", "| `M00` | Platform Control and Identity | `complete` |", 1),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(any("unknown state key for M00" in issue for issue in issues), issues)

    def test_module_state_mismatch_fails_closed(self) -> None:
        path = self.root / checker.MODULE_BLUEPRINTS["M80"]
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace(
                "`Implementation State`: `planned`",
                "`Implementation State`: `skeleton`",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(
            any("module implementation state drift for M80" in issue for issue in issues),
            issues,
        )

    def test_missing_roadmap_lane_fails_closed(self) -> None:
        self.remove_table_row(
            "docs/tasks/01-execution-roadmap.md", "| `M51` MCP Binding/Executor"
        )
        issues = self.check_module_registry()
        self.assertTrue(any("module roadmap module ID set drift" in issue for issue in issues), issues)

    def test_missing_coverage_row_fails_closed(self) -> None:
        self.remove_table_row("docs/coverage-matrix.md", "| `M70 ChangeRadar`")
        issues = self.check_module_registry()
        self.assertTrue(
            any("module coverage module ID set drift" in issue for issue in issues), issues
        )

    def test_catalog_only_family_cannot_be_claimed_active(self) -> None:
        path = self.root / "docs/coverage-matrix.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace(
                "`long-horizon:CLIENT-*`",
                "`active:CLIENT-*`",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(
            any(
                "active acceptance reference is not registered in matrix.tsv" in issue
                for issue in issues
            ),
            issues,
        )

    def test_unknown_acceptance_family_fails_closed(self) -> None:
        path = self.root / "docs/coverage-matrix.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace(
                "`long-horizon:CLIENT-*`",
                "`long-horizon:MOBILE-*`",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(
            any(
                "long-horizon acceptance reference is not catalog-only" in issue
                for issue in issues
            ),
            issues,
        )

    def test_active_family_cannot_be_claimed_long_horizon(self) -> None:
        path = self.root / "docs/coverage-matrix.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace(
                "`active:MARKET-*`",
                "`long-horizon:MARKET-*`",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(
            any(
                "long-horizon acceptance reference is not catalog-only" in issue
                for issue in issues
            ),
            issues,
        )

    def test_unstructured_acceptance_projection_fails_closed(self) -> None:
        path = self.root / "docs/coverage-matrix.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(text.replace("| `gap` |", "| active rows missing |", 1), encoding="utf-8")
        issues = self.check_module_registry()
        self.assertTrue(
            any("acceptance projection has an unstructured token" in issue for issue in issues),
            issues,
        )


class S0ArchitectureReviewContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    @property
    def review_path(self) -> Path:
        return self.root / checker.S0_REVIEW_PATH

    @property
    def roadmap_path(self) -> Path:
        return self.root / "docs/tasks/01-execution-roadmap.md"

    def check_review(self) -> list[str]:
        issues: list[str] = []
        checker.check_s0_architecture_review(issues)
        return issues

    def make_complete(self) -> None:
        text = self.review_path.read_text(encoding="utf-8")
        text = text.replace("- `Status`: `InReview`", "- `Status`: `Complete`", 1)
        completed_lines: list[str] = []
        for line in text.splitlines():
            if line.startswith(("| `architecture` |", "| `authority` |", "| `delivery` |")):
                line = line.replace("| `Pending` |", "| `Pass` |", 1)
            elif line.startswith("| `S0-"):
                line = line.replace(
                    "| `Pending` | — |",
                    "| `Accept` | `architecture`; `authority`; `delivery` |",
                    1,
                )
                line = line.replace("| `open` |", "| `closed` |", 1)
            completed_lines.append(line)
        self.review_path.write_text("\n".join(completed_lines) + "\n", encoding="utf-8")

        roadmap = self.roadmap_path.read_text(encoding="utf-8")
        roadmap = roadmap.replace(
            "### `S0-3` Team review\n\n**Status**: pending.",
            "### `S0-3` Team review\n\n**Status**: complete.",
            1,
        )
        self.roadmap_path.write_text(roadmap, encoding="utf-8")

    def mutate_decision(self, decision_id: str, transform: Callable[[str], str]) -> None:
        path = self.review_path
        lines = path.read_text(encoding="utf-8").splitlines()
        changed = False
        output: list[str] = []
        for line in lines:
            if line.startswith(f"| `{decision_id}` |"):
                line = transform(line)
                changed = True
            output.append(line)
        self.assertTrue(changed, decision_id)
        path.write_text("\n".join(output) + "\n", encoding="utf-8")

    def test_current_s0_review_passes(self) -> None:
        self.assertEqual(self.check_review(), [])

    def test_missing_root_agents_authority_deferral_fails_closed(self) -> None:
        text = self.review_path.read_text(encoding="utf-8")
        self.review_path.write_text(
            text.replace("[`../../AGENTS.md`](../../AGENTS.md), ", "", 1),
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("authority chain drift" in issue for issue in issues), issues)

    def test_missing_root_agents_reading_chain_fails_closed(self) -> None:
        text = self.review_path.read_text(encoding="utf-8")
        self.review_path.write_text(
            text.replace(
                "repository AGENTS, engineering constitution and terminology",
                "engineering constitution and terminology",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("reading chain drift" in issue for issue in issues), issues)

    def test_stale_duplicate_reading_chain_cannot_mask_drift(self) -> None:
        text = self.review_path.read_text(encoding="utf-8")
        text = text.replace(
            "repository AGENTS, engineering constitution and terminology",
            "engineering constitution and terminology",
            1,
        )
        text += f"\n```text\n{checker.S0_REVIEW_READING_CHAIN}\n```\n"
        self.review_path.write_text(text, encoding="utf-8")
        issues = self.check_review()
        self.assertTrue(any("reading chain" in issue for issue in issues), issues)

    def test_complete_s0_review_passes(self) -> None:
        self.make_complete()
        self.assertEqual(self.check_review(), [])

    def test_missing_decision_fails_closed(self) -> None:
        self.make_complete()
        lines = self.review_path.read_text(encoding="utf-8").splitlines()
        self.review_path.write_text(
            "\n".join(line for line in lines if not line.startswith("| `S0-M51` |"))
            + "\n",
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("S0 architecture decision set drift" in issue for issue in issues), issues)

    def test_duplicate_decision_fails_closed(self) -> None:
        self.make_complete()
        text = self.review_path.read_text(encoding="utf-8")
        self.review_path.write_text(
            text.replace("| `S0-M51` |", "| `S0-M50` |", 1), encoding="utf-8"
        )
        issues = self.check_review()
        self.assertTrue(any("duplicate S0 architecture decision" in issue for issue in issues), issues)

    def test_invalid_disposition_fails_closed(self) -> None:
        self.make_complete()
        self.mutate_decision(
            "S0-A01", lambda line: line.replace("`Accept`", "`Approved`", 1)
        )
        issues = self.check_review()
        self.assertTrue(any("invalid disposition" in issue for issue in issues), issues)

    def test_complete_review_rejects_pending_lane(self) -> None:
        self.make_complete()
        text = self.review_path.read_text(encoding="utf-8")
        self.review_path.write_text(
            text.replace(
                "| `architecture` | module ownership, acyclic dependencies, composition and replacement seams | `Pass` | — |",
                "| `architecture` | module ownership, acyclic dependencies, composition and replacement seams | `Pending` | — |",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("complete S0 review has non-pass lanes" in issue for issue in issues), issues)

    def test_missing_review_lane_fails_closed(self) -> None:
        self.make_complete()
        lines = self.review_path.read_text(encoding="utf-8").splitlines()
        self.review_path.write_text(
            "\n".join(line for line in lines if not line.startswith("| `delivery` |"))
            + "\n",
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("S0 review lane set drift" in issue for issue in issues), issues)

    def test_duplicate_review_lane_fails_closed(self) -> None:
        self.make_complete()
        text = self.review_path.read_text(encoding="utf-8")
        self.review_path.write_text(
            text.replace("| `delivery` |", "| `authority` |", 1), encoding="utf-8"
        )
        issues = self.check_review()
        self.assertTrue(any("duplicate S0 review lane" in issue for issue in issues), issues)

    def test_complete_review_rejects_pending_decision(self) -> None:
        self.make_complete()

        def make_pending(line: str) -> str:
            line = line.replace("`Accept`", "`Pending`", 1)
            line = line.replace(checker.S0_COMPLETE_REVIEW_LANES_CELL, "—", 1)
            return line.replace("`closed`", "`open`", 1)

        self.mutate_decision("S0-A01", make_pending)
        issues = self.check_review()
        self.assertTrue(any("complete S0 review has unresolved decisions" in issue for issue in issues), issues)

    def test_conditional_decision_requires_every_condition_field(self) -> None:
        self.make_complete()

        def make_partial_condition(line: str) -> str:
            line = line.replace("`Accept`", "`ConditionalAccept`", 1)
            return line.replace("| — | — | — | `closed` |", "| platform | — | checker PASS | `closed` |", 1)

        self.mutate_decision("S0-A01", make_partial_condition)
        issues = self.check_review()
        self.assertTrue(
            any("requires owner, evidence and exit condition" in issue for issue in issues),
            issues,
        )

    def test_complete_closed_conditional_decision_passes(self) -> None:
        self.make_complete()

        def make_closed_condition(line: str) -> str:
            line = line.replace("`Accept`", "`ConditionalAccept`", 1)
            return line.replace(
                "| — | — | — | `closed` |",
                "| platform | checker and review evidence | exact gate PASS | `closed` |",
                1,
            )

        self.mutate_decision("S0-A01", make_closed_condition)
        self.assertEqual(self.check_review(), [])

    def test_complete_review_rejects_rejected_decision(self) -> None:
        self.make_complete()

        def make_rejected(line: str) -> str:
            line = line.replace("`Accept`", "`Reject`", 1)
            line = line.replace(
                "| — | — | — | `closed` |",
                "| platform | owning contract correction | fresh lane PASS | `open` |",
                1,
            )
            return line

        self.mutate_decision("S0-A01", make_rejected)
        issues = self.check_review()
        self.assertTrue(any("complete S0 review has unresolved decisions" in issue for issue in issues), issues)

    def test_incomplete_review_lane_projection_fails_closed(self) -> None:
        self.make_complete()
        self.mutate_decision(
            "S0-A01",
            lambda line: line.replace(
                checker.S0_COMPLETE_REVIEW_LANES_CELL,
                "`architecture`; `authority`",
                1,
            ),
        )
        issues = self.check_review()
        self.assertTrue(any("must record all review lanes" in issue for issue in issues), issues)

    def test_packet_roadmap_status_drift_fails_closed(self) -> None:
        self.make_complete()
        roadmap = self.roadmap_path.read_text(encoding="utf-8")
        self.roadmap_path.write_text(
            roadmap.replace(
                "### `S0-3` Team review\n\n**Status**: complete.",
                "### `S0-3` Team review\n\n**Status**: pending.",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("packet/roadmap status drift" in issue for issue in issues), issues)

    def test_invalid_packet_status_fails_closed(self) -> None:
        self.make_complete()
        text = self.review_path.read_text(encoding="utf-8")
        self.review_path.write_text(
            text.replace("- `Status`: `Complete`", "- `Status`: `Accepted`", 1),
            encoding="utf-8",
        )
        issues = self.check_review()
        self.assertTrue(any("S0 architecture review status is invalid" in issue for issue in issues), issues)


class InvocationFixtureContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "crates", self.root / "crates")
        shutil.copytree(REPO_ROOT / "apps", self.root / "apps")
        acceptance = self.root / "docs/acceptance"
        acceptance.mkdir(parents=True)
        shutil.copy2(REPO_ROOT / "docs/acceptance/matrix.tsv", acceptance / "matrix.tsv")
        shutil.copy2(REPO_ROOT / "Cargo.toml", self.root / "Cargo.toml")
        shutil.copy2(REPO_ROOT / "Cargo.lock", self.root / "Cargo.lock")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_invocation(self) -> list[str]:
        issues: list[str] = []
        checker.check_invocation_fixtures(issues)
        return issues

    def check_agent_plugin_dependency_direction(self) -> list[str]:
        issues: list[str] = []
        checker.check_agent_plugin_dependency_direction(issues)
        return issues

    def test_exact_invocation_fixture_set_and_bindings_pass(self) -> None:
        self.assertEqual(self.check_invocation(), [])

    def test_missing_invocation_fixture_fails_closed(self) -> None:
        path = (
            self.root
            / "crates/platform-core/tests/fixtures/invocation-resolution/schema-golden-v0.json"
        )
        path.unlink()
        self.assertTrue(
            any("invocation-resolution fixture set drift" in issue for issue in self.check_invocation())
        )

    def test_non_synthetic_invocation_fixture_fails_closed(self) -> None:
        path = (
            self.root
            / "crates/platform-core/tests/fixtures/invocation-resolution/valid-synthetic-v0.json"
        )
        fixture = json.loads(path.read_text(encoding="utf-8"))
        fixture["synthetic"] = False
        path.write_text(json.dumps(fixture), encoding="utf-8")
        self.assertTrue(
            any("must remain exactly synthetic" in issue for issue in self.check_invocation())
        )

    def test_missing_and_unknown_invocation_case_fields_fail_closed(self) -> None:
        path = (
            self.root
            / "crates/platform-core/tests/fixtures/invocation-resolution/identity-mismatch-v0.json"
        )
        original = json.loads(path.read_text(encoding="utf-8"))
        for mutation in ("missing", "unknown"):
            fixture = json.loads(json.dumps(original))
            if mutation == "missing":
                fixture["cases"][0].pop("expected")
            else:
                fixture["cases"][0]["unknown"] = "value"
            path.write_text(json.dumps(fixture), encoding="utf-8")
            self.assertTrue(
                any("case fields drift" in issue for issue in self.check_invocation()),
                mutation,
            )
        path.write_text(json.dumps(original, separators=(",", ":")) + "\n", encoding="utf-8")

    def test_duplicate_and_unknown_api_invocation_cases_fail_closed(self) -> None:
        path = (
            self.root
            / "crates/platform-core/tests/fixtures/invocation-resolution/schema-golden-v0.json"
        )
        original = json.loads(path.read_text(encoding="utf-8"))
        fixture = json.loads(json.dumps(original))
        fixture["cases"][1]["name"] = fixture["cases"][0]["name"]
        path.write_text(json.dumps(fixture), encoding="utf-8")
        self.assertTrue(
            any("duplicate invocation fixture case name" in issue for issue in self.check_invocation())
        )
        fixture = json.loads(json.dumps(original))
        fixture["cases"][0]["api"] = "framework_registry"
        path.write_text(json.dumps(fixture), encoding="utf-8")
        self.assertTrue(any("case API is unknown" in issue for issue in self.check_invocation()))
        path.write_text(json.dumps(original, separators=(",", ":")) + "\n", encoding="utf-8")

    def test_invocation_expected_precedence_and_golden_detail_drift_fails_closed(self) -> None:
        path = (
            self.root
            / "crates/platform-core/tests/fixtures/invocation-resolution/valid-synthetic-v0.json"
        )
        original = json.loads(path.read_text(encoding="utf-8"))
        for field in ("expected", "precedence", "recipe"):
            fixture = json.loads(json.dumps(original))
            fixture["cases"][0][field] = "wrong"
            path.write_text(json.dumps(fixture), encoding="utf-8")
            self.assertTrue(
                any("executable details drift" in issue for issue in self.check_invocation()),
                field,
            )
        path.write_text(json.dumps(original, separators=(",", ":")) + "\n", encoding="utf-8")

    def test_invocation_acceptance_binding_drift_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        matrix = path.read_text(encoding="utf-8")
        commands = [
            checker.INVOCATION_FIXTURE_TEST_COMMAND,
            checker.INVOCATION_COMPOSITION_FIXTURE_TEST_COMMAND,
        ]
        for command in commands:
            path.write_text(matrix.replace(command + " && ", "", 1), encoding="utf-8")
            self.assertTrue(
                any(
                    "MARKET-005: implemented invocation binding/status drift" in issue
                    for issue in self.check_invocation()
                ),
                command,
            )

    def test_agent_runtime_dependency_allowlist_fails_closed(self) -> None:
        self.assertEqual(self.check_agent_plugin_dependency_direction(), [])

        manifest_path = self.root / "crates/agent-runtime/Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                "[dev-dependencies]",
                "[dev-dependencies]\nustc-campus-agent-core = { path = \"../platform-core\" }",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "agent-runtime has unapproved direct dependencies" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_dependency_alias_fails_closed(self) -> None:
        manifest_path = self.root / "crates/agent-runtime/Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                "[dev-dependencies]",
                "[dev-dependencies]\nplugin_api = { package = \"ustc-campus-agent-core\", path = \"../platform-core\" }",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "plugin_api->ustc-campus-agent-core" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_unknown_future_dependency_fails_closed(self) -> None:
        manifest_path = self.root / "crates/agent-runtime/Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                "[dev-dependencies]",
                "[dev-dependencies]\nfuture-plugin = { path = \"../future-plugin\" }",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "future-plugin->future-plugin" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_workspace_dependency_redirect_fails_closed(self) -> None:
        manifest_path = self.root / "Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                'serde_json = "1.0.151"',
                'serde_json = { package = "future-plugin", path = "crates/future-plugin" }',
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "serde_json->future-plugin@path:crates/future-plugin" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_same_name_workspace_path_fails_closed(self) -> None:
        manifest_path = self.root / "Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                'serde_json = "1.0.151"',
                'serde_json = { path = "crates/plugin-disguised-as-serde-json" }',
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "serde_json->serde_json@path:crates/plugin-disguised-as-serde-json" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_protocol_workspace_redirect_fails_closed(self) -> None:
        manifest_path = self.root / "Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                'ustc-agent-tool-protocol = { path = "crates/agent-tool-protocol" }',
                'ustc-agent-tool-protocol = { path = "crates/plugin-disguised-as-protocol" }',
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "ustc-agent-tool-protocol->ustc-agent-tool-protocol@path:crates/plugin-disguised-as-protocol"
                in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_alternate_registry_fails_closed(self) -> None:
        manifest_path = self.root / "Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                'serde_json = "1.0.151"',
                'serde_json = { version = "1.0.151", registry = "plugin-registry" }',
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "serde_json->serde_json@registry:plugin-registry" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_repository_source_replacement_fails_closed(self) -> None:
        cargo_directory = self.root / ".cargo"
        cargo_directory.mkdir()
        (cargo_directory / "config.toml").write_text(
            '[source.crates-io]\nreplace-with = "vendored-sources"\n\n'
            '[source.vendored-sources]\ndirectory = "vendor"\n',
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "repository Cargo config is forbidden" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_workspace_source_redirects_fail_closed(self) -> None:
        plugin = self.root / "plugins/serde_json"
        (plugin / "src").mkdir(parents=True)
        (plugin / "Cargo.toml").write_text(
            '[package]\nname = "serde_json"\nversion = "1.0.151"\nedition = "2024"\n',
            encoding="utf-8",
        )
        (plugin / "src/lib.rs").write_text("pub struct PluginCode;\n", encoding="utf-8")
        manifest_path = self.root / "Cargo.toml"
        original_manifest = manifest_path.read_text(encoding="utf-8")
        redirects = {
            "patch": '\n[patch.crates-io]\nserde_json = { path = "plugins/serde_json" }\n',
            "replace": (
                '\n[replace]\n"serde_json:1.0.151" = { path = "plugins/serde_json" }\n'
            ),
        }
        for redirect_table, redirect in redirects.items():
            manifest_path.write_text(original_manifest + redirect, encoding="utf-8")
            issues = self.check_agent_plugin_dependency_direction()
            self.assertTrue(
                any(
                    f"workspace Cargo {redirect_table} table is forbidden" in issue
                    for issue in issues
                ),
                issues,
            )

    def test_agent_runtime_protocol_projection_construction_fails_closed(self) -> None:
        library_path = self.root / "crates/agent-runtime/src/lib.rs"
        with library_path.open("a", encoding="utf-8") as library:
            library.write(
                "\nuse ustc_agent_tool_protocol::{"
                "AgentToolsetView as View, AgentToolDefinition as Definition, AgentTool as Tool};\n"
                "fn illicit_projection() { View::new(()); Definition::new(()); Tool::new(()); }\n"
            )
        issues = self.check_agent_plugin_dependency_direction()
        for description in (
            "projection authority type AgentToolsetView",
            "projection authority type AgentToolDefinition",
            "projection authority type AgentTool",
        ):
            self.assertTrue(
                any(
                    "agent-runtime source crosses the compilation boundary" in issue
                    and description in issue
                    for issue in issues
                ),
                issues,
            )

    def test_agent_runtime_external_library_target_fails_closed(self) -> None:
        manifest_path = self.root / "crates/agent-runtime/Cargo.toml"
        manifest = manifest_path.read_text(encoding="utf-8")
        manifest_path.write_text(
            manifest.replace(
                'path = "src/lib.rs"',
                'path = "../../plugins/agent-runtime-wrapper.rs"',
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "agent-runtime library target must remain exactly src/lib.rs" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_include_code_escape_fails_closed(self) -> None:
        source_path = self.root / "crates/agent-runtime/src/escape.rs"
        source_path.write_text(
            'include!("../../../plugins/plugin-code.rs");\n',
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "agent-runtime source crosses the compilation boundary" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_cfg_attr_path_escape_fails_closed(self) -> None:
        source_path = self.root / "crates/agent-runtime/src/escape.rs"
        source_path.write_text(
            '#[cfg_attr(all(), path /* bypass */ = "../../../plugins/plugin.rs")]\n'
            "mod plugin_impl;\n",
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "agent-runtime source crosses the compilation boundary" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_rustc_dep_info_catches_obfuscated_path(self) -> None:
        plugin_source = self.root / "plugins/plugin_impl.rs"
        plugin_source.parent.mkdir(parents=True, exist_ok=True)
        plugin_source.write_text("pub struct PluginImpl;\n", encoding="utf-8")
        library_path = self.root / "crates/agent-runtime/src/lib.rs"
        with library_path.open("a", encoding="utf-8") as library:
            library.write(
                '\n#[/* comment */ path = "../../../plugins/plugin_impl.rs"]\n'
                "mod plugin_impl;\n"
            )
        self.assertTrue(
            any(
                "agent-runtime rustc library dep-info escapes the owned crate tree" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_test_dep_info_catches_test_only_path(self) -> None:
        plugin_source = self.root / "plugins/test_plugin.rs"
        plugin_source.parent.mkdir(parents=True, exist_ok=True)
        plugin_source.write_text("pub struct TestPlugin;\n", encoding="utf-8")
        library_path = self.root / "crates/agent-runtime/src/lib.rs"
        with library_path.open("a", encoding="utf-8") as library:
            library.write(
                '\n#[cfg(test)]\n#[path = "../../../plugins/test_plugin.rs"]\n'
                "mod test_plugin;\n"
            )
        self.assertTrue(
            any(
                "agent-runtime rustc test dep-info escapes the owned crate tree" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_source_symlink_fails_closed(self) -> None:
        plugin_source = self.root / "plugins/plugin-code.rs"
        plugin_source.parent.mkdir(parents=True, exist_ok=True)
        plugin_source.write_text("pub struct PluginCode;\n", encoding="utf-8")
        source_link = self.root / "crates/agent-runtime/src/plugin_code.rs"
        source_link.symlink_to(plugin_source)
        self.assertTrue(
            any(
                "agent-runtime source tree contains a symlink escape" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )

    def test_agent_runtime_source_boundary_fails_closed(self) -> None:
        source_path = self.root / "crates/agent-runtime/src/forbidden.rs"
        source_path.write_text("use ustc_campus_agent_core::invocation::*;\n", encoding="utf-8")
        self.assertTrue(
            any(
                "agent-runtime source crosses the Agent/Plugin boundary" in issue
                for issue in self.check_agent_plugin_dependency_direction()
            )
        )


if __name__ == "__main__":
    unittest.main()
