from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
import unittest
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

    def test_current_docs_topology_passes(self) -> None:
        self.assertEqual(self.check_docs_topology(), [])

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


if __name__ == "__main__":
    unittest.main()
