from __future__ import annotations

import ast
import contextlib
import importlib.util
import io
import hashlib
import json
import re
import shutil
import tempfile
import unittest
from collections.abc import Callable
from pathlib import Path
from typing import cast
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
CHECKER_PATH = REPO_ROOT / "scripts/check_repo_contracts.py"
SPEC = importlib.util.spec_from_file_location("check_repo_contracts", CHECKER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {CHECKER_PATH}")
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)

ADMITTED_INSTALLATION_SOURCE = '\nuse crate::identity::{TenantId, UserId};\nuse crate::invocation::{\n    CapabilityId, CatalogRevision, ComponentId, ComponentKind, ComponentVersion,\n    ExecutionIdentity, InstallationId, InstallationRevision, InstalledComponentIdentity,\n    PackageId, PackageVersion, PluginInstallationSnapshot, Sha256Digest,\n};\nuse std::collections::{BTreeMap, BTreeSet};\nuse std::error::Error;\nuse std::fmt;\n\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationCommandId(String);\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct InstallationEventSequence(u64);\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ConfigurationRevision(u64);\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct ConfigurationKey(String);\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct NonSecretText(String);\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct SecretRefId(String);\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct SecretRef { tenant_id: TenantId, id: SecretRefId }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum ConfigurationValue { Text(NonSecretText), Integer(i64), Boolean(bool), Secret(SecretRef) }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationConfiguration { values: BTreeMap<ConfigurationKey, ConfigurationValue>, digest: Sha256Digest }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstalledComponentPin { component_id: ComponentId, kind: ComponentKind, version: ComponentVersion, digest: Sha256Digest, execution_identity: ExecutionIdentity }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationPackagePin { catalog_revision: CatalogRevision, package_id: PackageId, package_version: PackageVersion, package_digest: Sha256Digest, components: Vec<InstalledComponentPin>, component_set_digest: Sha256Digest, capability_manifest_digest: Sha256Digest }\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum ManagedInstallationState { InstalledDisabled, Enabled, Disabled, Revoked, Uninstalled }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct EnablePreconditionEvidence { installation_id: InstallationId, expected_revision: InstallationRevision, digest: Sha256Digest }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationAggregate { tenant_id: TenantId, user_id: UserId, installation_id: InstallationId, state: ManagedInstallationState }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationCommand { command_id: InstallationCommandId, installation_id: InstallationId }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum InstallationEventKind { Installed, Configured, Enabled, Disabled, Revoked, Uninstalled }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationEvent { sequence: InstallationEventSequence, kind: InstallationEventKind }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationSnapshot { aggregate: InstallationAggregate, resolver: Option<PluginInstallationSnapshot> }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum InstallationDecisionError { Rejected }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum InstallationReplayError { Rejected }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum InstallationRepositoryError { CommandConflict }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InstallationCommandReceipt { command: InstallationCommand }\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct InMemoryInstallationRepository { receipts: BTreeMap<InstallationCommandId, InstallationCommandReceipt>, seen_capabilities: BTreeSet<CapabilityId> }\nstruct PersistedCommandReceipt;\n\nimpl InstallationCommandId { fn parse(value: impl Into<String>) -> Self { Self(value.into()) } }\nimpl InstallationEventSequence {}\nimpl ConfigurationRevision {}\nimpl ConfigurationKey {}\nimpl NonSecretText {}\nimpl SecretRefId {}\nimpl SecretRef {}\nimpl InstallationConfiguration {}\nimpl InstalledComponentPin {}\nimpl InstallationPackagePin {}\nimpl EnablePreconditionEvidence {}\nimpl InstallationAggregate {}\nimpl InstallationCommand {}\nimpl InstallationEvent {}\nimpl InstallationSnapshot {}\nimpl InstallationCommandReceipt {}\nimpl InMemoryInstallationRepository {}\nimpl fmt::Display for InstallationDecisionError { fn fmt(&self, formatter: &mut fmt::Formatter<\'_>) -> fmt::Result { write!(formatter, "installation decision error") } }\nimpl fmt::Display for InstallationReplayError { fn fmt(&self, formatter: &mut fmt::Formatter<\'_>) -> fmt::Result { write!(formatter, "installation replay error") } }\nimpl fmt::Display for InstallationRepositoryError { fn fmt(&self, formatter: &mut fmt::Formatter<\'_>) -> fmt::Result { write!(formatter, "installation repository error") } }\nimpl Error for InstallationDecisionError {}\nimpl Error for InstallationReplayError {}\nimpl Error for InstallationRepositoryError {}\n\npub trait InstallationRepository {\n    fn execute(&mut self, command: InstallationCommand) -> Result<InstallationCommandReceipt, InstallationRepositoryError>;\n}\nimpl InstallationRepository for InMemoryInstallationRepository {\n    fn execute(&mut self, command: InstallationCommand) -> Result<InstallationCommandReceipt, InstallationRepositoryError> {\n        let _ = matches!(InstallationRepositoryError::CommandConflict, InstallationRepositoryError::CommandConflict);\n        let _ = format!("installation receipt");\n        Ok(InstallationCommandReceipt { command })\n    }\n}\n\npub fn decide(_current: Option<&InstallationAggregate>, _command: &InstallationCommand) -> Result<InstallationEvent, InstallationDecisionError> { Err(InstallationDecisionError::Rejected) }\npub fn evolve(_current: Option<InstallationAggregate>, _event: &InstallationEvent) -> Result<InstallationAggregate, InstallationReplayError> { Err(InstallationReplayError::Rejected) }\npub fn replay<\'a>(_events: impl IntoIterator<Item = &\'a InstallationEvent>) -> Result<Option<InstallationAggregate>, InstallationReplayError> { Ok(None) }\n'


class MarketContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "market", self.root / "market")
        shutil.copytree(REPO_ROOT / "plugins", self.root / "plugins")
        shutil.copytree(REPO_ROOT / "docs/contracts", self.root / "docs/contracts")
        shutil.copytree(
            REPO_ROOT / "fixtures/opportunity-graph",
            self.root / "fixtures/opportunity-graph",
        )
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_market(self) -> list[str]:
        issues: list[str] = []
        checker.check_market(issues)
        return issues

    def check_course_fixture(self) -> list[str]:
        issues: list[str] = []
        checker.check_course_fixture(issues)
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

    def registry_path(self) -> Path:
        return self.root / "market/capabilities/registry.json"

    def load_registry(self) -> dict[str, object]:
        return json.loads(self.registry_path().read_text(encoding="utf-8"))

    def write_registry(self, registry: dict[str, object]) -> None:
        self.registry_path().write_text(
            json.dumps(registry, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

    def capability_schema_path(self) -> Path:
        return self.root / "market/schemas/capability-registry.schema.json"

    def test_three_default_first_party_manifests_pass(self) -> None:
        self.assertEqual(self.check_market(), [])

    def test_course_catalog_three_way_digest_binding_fails_closed(self) -> None:
        self.assertEqual(self.check_course_fixture(), [])
        resource_path = (
            self.root
            / "market/packages/ustc.opportunity-graph/components/course-planning-resource-pack.json"
        )
        resource = json.loads(resource_path.read_text(encoding="utf-8"))
        resource["resourceSha256"] = "0" * 64
        resource_path.write_text(json.dumps(resource) + "\n", encoding="utf-8")
        self.assertTrue(
            any(
                "Course Planning catalog digest binding drift" in issue
                for issue in self.check_course_fixture()
            )
        )
        native_path = (
            self.root
            / "market/packages/ustc.opportunity-graph/components/native-rust-component.json"
        )
        native = json.loads(native_path.read_text(encoding="utf-8"))
        native["operations"] = native["operations"][:-1]
        native_path.write_text(json.dumps(native) + "\n", encoding="utf-8")
        self.assertTrue(
            any(
                "Opportunity native-component identity/operation drift" in issue
                for issue in self.check_course_fixture()
            )
        )

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
        registry = self.load_registry()
        rows = cast(list[dict[str, object]], registry["capabilities"])
        private = next(
            row
            for row in rows
            if row.get("id") == "user.own_academic_snapshot.read"
        )
        private["autoGrant"] = "FirstPartyDefaultOnly"
        private["confirmationDefault"] = "Allow"
        self.write_registry(registry)
        issues = self.check_market()
        self.assertTrue(any("capability registry axes drifted" in issue for issue in issues))
        self.assertTrue(
            any("capability registry unsafe auto-grant tuple" in issue for issue in issues)
        )

    def test_capability_registry_rejects_legacy_fields_and_schema_drift(self) -> None:
        registry = self.load_registry()
        rows = cast(list[dict[str, object]], registry["capabilities"])
        rows[0]["autoGrantEligible"] = True
        self.write_registry(registry)
        self.assertTrue(
            any("capability registry row keys drifted" in issue for issue in self.check_market())
        )

        registry = self.load_registry()
        rows = cast(list[dict[str, object]], registry["capabilities"])
        rows[0].pop("autoGrantEligible", None)
        registry["schemaVersion"] = "capability-registry/v2"
        self.write_registry(registry)
        self.assertTrue(
            any("capability registry schemaVersion drift" in issue for issue in self.check_market())
        )

    def test_capability_registry_axes_and_auto_grant_coherence_fail_closed(self) -> None:
        registry = self.load_registry()
        rows = cast(list[dict[str, object]], registry["capabilities"])
        rows[0]["dataClass"] = "UserProfile"
        self.write_registry(registry)
        issues = self.check_market()
        self.assertTrue(any("capability registry axes drifted" in issue for issue in issues))
        self.assertTrue(
            any("capability registry unsafe auto-grant tuple" in issue for issue in issues)
        )

        registry = self.load_registry()
        rows = cast(list[dict[str, object]], registry["capabilities"])
        rows[0]["dataClass"] = "PublicCampusFact"
        rows[0]["autoGrant"] = "Never"
        self.write_registry(registry)
        self.assertTrue(
            any("capability registry axes drifted" in issue for issue in self.check_market())
        )

    def test_capability_registry_schema_carriers_are_pinned(self) -> None:
        schema = json.loads(self.capability_schema_path().read_text(encoding="utf-8"))
        schema["additionalProperties"] = True
        self.capability_schema_path().write_text(
            json.dumps(schema, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "CapabilityRegistry schema must deny unknown top-level fields" in issue
                for issue in self.check_market()
            )
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
        ci_path = self.root / ".github/workflows/ci.yml"
        ci_path.parent.mkdir(parents=True)
        shutil.copy2(REPO_ROOT / ".github/workflows/ci.yml", ci_path)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def replace_once(self, rel: str, old: str, new: str) -> None:
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        self.assertEqual(text.count(old), 1, f"stale or ambiguous mutation target in {rel}")
        updated = text.replace(old, new, 1)
        self.assertNotEqual(updated, text)
        path.write_text(updated, encoding="utf-8")

    def check_docs_topology(self) -> list[str]:
        issues: list[str] = []
        checker.check_docs_topology(issues)
        return issues

    def check_key_files(self) -> list[str]:
        issues: list[str] = []
        checker.check_key_files_present_and_nonempty(issues)
        return issues

    def check_campaign_authorization(self) -> list[str]:
        issues: list[str] = []
        checker.check_campaign_authorization(issues)
        return issues

    def rewrite_campaign_matrix_status(self, case_id: str, status: str) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        status_index = header.index("status")
        found = False
        for index, line in enumerate(lines[1:], start=1):
            cells = line.split("\t")
            if cells[0] == case_id:
                cells[status_index] = status
                lines[index] = "\t".join(cells)
                found = True
                break
        self.assertTrue(found, f"missing matrix row {case_id}")
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def test_current_campaign_authorization_passes(self) -> None:
        self.assertEqual(self.check_campaign_authorization(), [])

    def test_campaign_matrix_binding_status_drift_fails_closed(self) -> None:
        self.rewrite_campaign_matrix_status("HARNESS-001", "implemented")
        self.assertTrue(
            any(
                "campaign acceptance binding status drift for HARNESS-001" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_matrix_binding_missing_row_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        path.write_text(
            "\n".join(line for line in lines if not line.startswith("AGENT-018\t")) + "\n",
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "campaign acceptance matrix row missing: AGENT-018" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_catalog_binding_missing_row_fails_closed(self) -> None:
        self.replace_once(
            checker.AUTONOMOUS_CAMPAIGN_CATALOG_PATH,
            "| `HARNESS-002` |",
            "| `HARNESS-099` |",
        )
        self.assertTrue(
            any(
                "campaign long-horizon catalog row count drift for HARNESS-002" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_catalog_only_id_cannot_be_admitted_silently(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text
            + "HARNESS-002\tharness\tclarification remains bounded\tfuture H0 tests\tpr\tplanned\tbackend\n",
            encoding="utf-8",
        )
        self.assertTrue(
            any(
                "campaign catalog-only ID unexpectedly admitted to matrix: HARNESS-002"
                in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_missing_campaign_policy_marker_fails_closed(self) -> None:
        self.replace_once(
            "docs/tasks/00-module-work-policy.md",
            checker.CAMPAIGN_AUTHORIZATION_POLICY_BEGIN,
            "<!-- removed policy marker -->",
        )
        self.assertTrue(
            any(
                "campaign authorization policy marker count drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_duplicate_campaign_policy_block_fails_closed(self) -> None:
        path = self.root / "docs/tasks/00-module-work-policy.md"
        text = path.read_text(encoding="utf-8")
        start = text.index(checker.CAMPAIGN_AUTHORIZATION_POLICY_BEGIN)
        finish = text.index(checker.CAMPAIGN_AUTHORIZATION_POLICY_END) + len(
            checker.CAMPAIGN_AUTHORIZATION_POLICY_END
        )
        path.write_text(f"{text}\n{text[start:finish]}\n", encoding="utf-8")
        self.assertTrue(
            any(
                "campaign authorization policy marker count drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_policy_content_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/tasks/00-module-work-policy.md",
            "Only Develata may create, activate, amend, relocate, pause or revoke",
            "An agent may create, activate, amend, relocate, pause or revoke",
        )
        self.assertTrue(
            any(
                "campaign authorization policy exact block drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_missing_autonomous_grant_marker_fails_closed(self) -> None:
        self.replace_once(
            "docs/tasks/01-execution-roadmap.md",
            checker.AUTONOMOUS_CAMPAIGN_GRANT_END,
            "<!-- removed grant marker -->",
        )
        self.assertTrue(
            any(
                "autonomous campaign grant marker count drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_duplicate_autonomous_grant_block_fails_closed(self) -> None:
        path = self.root / "docs/tasks/01-execution-roadmap.md"
        text = path.read_text(encoding="utf-8")
        start = text.index(checker.AUTONOMOUS_CAMPAIGN_GRANT_BEGIN)
        finish = text.index(checker.AUTONOMOUS_CAMPAIGN_GRANT_END) + len(
            checker.AUTONOMOUS_CAMPAIGN_GRANT_END
        )
        path.write_text(f"{text}\n{text[start:finish]}\n", encoding="utf-8")
        self.assertTrue(
            any(
                "autonomous campaign grant marker count drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_autonomous_grant_status_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/tasks/01-execution-roadmap.md",
            "- `Status`: `active`",
            "- `Status`: `paused`",
        )
        self.assertTrue(
            any(
                "autonomous campaign grant exact block drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_autonomous_grant_base_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/tasks/01-execution-roadmap.md",
            "b7911859454e659b2fd426ac475958a22b92e5a8",
            "0000000000000000000000000000000000000000",
        )
        self.assertTrue(
            any(
                "autonomous campaign grant exact block drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_autonomous_grant_scope_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/tasks/01-execution-roadmap.md",
            "| `M40-B0` |",
            "| `M50-B1` |",
        )
        self.assertTrue(
            any(
                "autonomous campaign grant exact block drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_checker_remains_invoked_from_main(self) -> None:
        checker_source = CHECKER_PATH.read_text(encoding="utf-8")
        main_body = checker_source.split("\ndef main() -> int:", 1)
        self.assertEqual(len(main_body), 2)
        self.assertEqual(
            sum(
                line.strip() == "check_campaign_authorization(issues)"
                for line in main_body[1].splitlines()
            ),
            1,
        )

    def test_campaign_checker_ci_binding_drift_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "python3 scripts/check_repo_contracts.py",
            "python3 scripts/check_repo_contracts_disabled.py",
        )
        self.assertTrue(
            any(
                "campaign authorization aggregate checker CI binding drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_ci_step_disable_fails_closed(self) -> None:
        self.replace_once(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            "      - name: Run exact-inventory checker shards\n"
            "        run: python3 scripts/run_checker_shards.py",
            "      - name: Run exact-inventory checker shards\n"
            "        if: false\n"
            "        run: python3 scripts/run_checker_shards.py",
        )
        self.assertTrue(
            any(
                "CI v2 active workflow conditional inventory drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_mutation_test_ci_binding_drift_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "python3 scripts/run_checker_shards.py",
            "python3 scripts/run_disabled_checker_shards.py",
        )
        self.assertTrue(
            any(
                "campaign authorization mutation-test CI binding drift" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_missing_campaign_taskbook_fails_closed(self) -> None:
        (self.root / checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M00-B3"]).unlink()
        self.assertTrue(
            any(
                "campaign taskbook missing for M00-B3" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_duplicate_campaign_taskbook_field_fails_closed(self) -> None:
        self.replace_once(
            checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M00-B3"],
            "- `Status`: `queued`",
            "- `Status`: `queued`\n- `Status`: `active`",
        )
        self.assertTrue(
            any(
                "campaign taskbook field count drift for M00-B3 Status" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_invalid_campaign_taskbook_status_fails_closed(self) -> None:
        rel = checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M20-B6"]
        text = (self.root / rel).read_text(encoding="utf-8")
        status_lines = [
            line
            for line in text.splitlines()
            if line.startswith("- `Status`: `") and line.endswith("`")
        ]
        self.assertEqual(len(status_lines), 1, "missing or ambiguous taskbook status")
        self.replace_once(
            rel,
            status_lines[0],
            "- `Status`: `running-unsafely`",
        )
        self.assertTrue(
            any(
                "campaign taskbook invalid status for M20-B6" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_active_campaign_taskbook_requires_exact_source(self) -> None:
        rel = checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M30-B0"]
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        status_pattern = re.compile(r"^- `Status`:\s*(.+?)\s*$", re.MULTILINE)
        source_pattern = re.compile(
            r"^- `Bound source commit`:\s*(.+?)\s*$",
            re.MULTILINE,
        )
        status_matches = list(status_pattern.finditer(text))
        source_matches = list(source_pattern.finditer(text))
        self.assertEqual(len(status_matches), 1, "missing or ambiguous taskbook status")
        self.assertEqual(len(source_matches), 1, "missing or ambiguous taskbook source")

        # Mutate from any checker-admitted live lane state, including whitespace variants.
        updated = status_pattern.sub("- `Status`: `active`", text, count=1)
        updated = source_pattern.sub(
            "- `Bound source commit`: `pending-governance-main`",
            updated,
            count=1,
        )
        self.assertNotEqual(updated, text, "fixture mutation must alter a campaign carrier")
        path.write_text(updated, encoding="utf-8")
        self.assertTrue(
            any(
                "campaign taskbook active/completed lane has pending source for M30-B0" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_campaign_taskbook_round_two_requires_pause(self) -> None:
        self.replace_once(
            checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M40-B0"],
            "- `Repair round`: `0`",
            "- `Repair round`: `2`",
        )
        self.replace_once(
            checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M40-B0"],
            "- `Current blocker identity`: `none`",
            "- `Current blocker identity`: `still-open-after-two-rounds`",
        )
        self.assertTrue(
            any(
                "campaign taskbook round 2 must be paused for M40-B0" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_paused_campaign_taskbook_requires_stop_reason(self) -> None:
        rel = checker.AUTONOMOUS_CAMPAIGN_TASKBOOKS["M40-B0"]
        self.replace_once(rel, "- `Status`: `queued`", "- `Status`: `paused`")
        self.assertTrue(
            any(
                "campaign taskbook paused lane has no stop reason for M40-B0" in issue
                for issue in self.check_campaign_authorization()
            )
        )

    def test_current_docs_topology_passes(self) -> None:
        self.assertEqual(self.check_docs_topology(), [])

    def test_design_governance_files_are_registered_nonempty_key_files(self) -> None:
        issues = self.check_key_files()
        key_files = cast(list[str], getattr(checker, "KEY_FILES"))
        for rel in ("docs/design/AGENTS.md", "docs/design/README.md"):
            self.assertIn(rel, key_files)
            self.assertFalse(any(rel in issue for issue in issues), rel)

    def test_missing_design_governance_file_fails_closed(self) -> None:
        for rel in ("docs/design/AGENTS.md", "docs/design/README.md"):
            with self.subTest(rel=rel):
                path = self.root / rel
                original = path.read_text(encoding="utf-8")
                path.unlink()
                self.assertIn(f"key file missing: {rel}", self.check_key_files())
                path.write_text(original, encoding="utf-8")

    def test_empty_design_governance_file_fails_closed(self) -> None:
        for rel in ("docs/design/AGENTS.md", "docs/design/README.md"):
            with self.subTest(rel=rel):
                path = self.root / rel
                original = path.read_text(encoding="utf-8")
                path.write_text(" \n", encoding="utf-8")
                self.assertIn(f"key file empty: {rel}", self.check_key_files())
                path.write_text(original, encoding="utf-8")

    def test_unknown_extra_docs_directory_fails_closed(self) -> None:
        (self.root / "docs/incoming").mkdir()
        (self.root / "docs/incoming/notes.md").write_text("stray\n", encoding="utf-8")
        self.assertTrue(
            any("documentation directory topology drift" in issue for issue in self.check_docs_topology())
        )

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

    def test_peer_client_governance_files_are_registered_nonempty_key_files(self) -> None:
        issues = self.check_key_files()
        key_files = cast(list[str], getattr(checker, "KEY_FILES"))
        for rel in (
            "docs/features/05-headless-client-and-agent-integration.md",
            "docs/adr/0010-typed-client-peer-adapters.md",
        ):
            self.assertIn(rel, key_files)
            self.assertFalse(any(rel in issue for issue in issues), rel)

    def test_missing_peer_client_governance_file_fails_closed(self) -> None:
        for rel in (
            "docs/features/05-headless-client-and-agent-integration.md",
            "docs/adr/0010-typed-client-peer-adapters.md",
        ):
            with self.subTest(rel=rel):
                path = self.root / rel
                original = path.read_text(encoding="utf-8")
                path.unlink()
                self.assertIn(f"key file missing: {rel}", self.check_key_files())
                path.write_text(original, encoding="utf-8")

    def test_platform_identity_contract_is_a_registered_nonempty_key_file(self) -> None:
        issues = self.check_key_files()
        self.assertFalse(any("platform-identity.md" in issue for issue in issues))

    def test_missing_platform_identity_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/platform-identity.md"
        path.unlink()
        self.assertIn(
            "key file missing: docs/contracts/platform-identity.md",
            self.check_key_files(),
        )

    def test_empty_platform_identity_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/platform-identity.md"
        path.write_text(" \n", encoding="utf-8")
        self.assertIn(
            "key file empty: docs/contracts/platform-identity.md",
            self.check_key_files(),
        )

    def test_platform_session_contract_is_a_registered_nonempty_key_file(self) -> None:
        issues = self.check_key_files()
        self.assertFalse(any("platform-session.md" in issue for issue in issues))

    def test_missing_platform_session_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/platform-session.md"
        path.unlink()
        self.assertIn(
            "key file missing: docs/contracts/platform-session.md",
            self.check_key_files(),
        )

    def test_empty_platform_session_contract_fails_closed(self) -> None:
        path = self.root / "docs/contracts/platform-session.md"
        path.write_text(" \n", encoding="utf-8")
        self.assertIn(
            "key file empty: docs/contracts/platform-session.md",
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
        codeowners.parent.mkdir(exist_ok=True)
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

    def test_active_auth_case_must_exist_in_catalog(self) -> None:
        matrix_path = self.root / "docs/acceptance/matrix.tsv"
        matrix = matrix_path.read_text(encoding="utf-8")
        matrix_path.write_text(matrix.replace("AUTH-011", "AUTH-999", 1), encoding="utf-8")
        issues: list[str] = []
        checker.check_acceptance_catalog(issues)
        self.assertTrue(any("active case missing from long-horizon catalog" in issue for issue in issues))

    def test_rust_doctest_gate_is_declared_in_docs_and_ci(self) -> None:
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertEqual(issues, [])

    def test_blank_line_inside_trigger_block_remains_valid(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "on:\n  pull_request:",
            "on:\n\n  pull_request:",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertEqual(issues, [])

    def test_missing_rust_doctest_docs_gate_fails_closed(self) -> None:
        self.replace_once(
            "docs/acceptance/gates.md",
            checker.RUST_DOCTEST_GATE_COMMAND,
            f"# {checker.RUST_DOCTEST_GATE_COMMAND}",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest gate missing from docs/acceptance/gates.md", issues)

    def test_missing_rust_doctest_ci_gate_fails_closed(self) -> None:
        ci_command = f"        run: {checker.RUST_DOCTEST_GATE_COMMAND}"
        self.replace_once(
            ".github/workflows/ci.yml",
            ci_command,
            f"        # run: {checker.RUST_DOCTEST_GATE_COMMAND}",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI step must use the exact run command", issues)

    def test_conditional_rust_job_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "  rust:\n    name: rust",
            "  rust:\n    if: github.event_name != 'pull_request'\n    name: rust",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI rust job must not be conditional", issues)

    def test_rust_job_outside_jobs_block_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "jobs:\n  rust:",
            "x-disabled:\n  rust:",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI rust job missing or ambiguous", issues)

    def test_doctest_step_outside_steps_block_fails_closed(self) -> None:
        run_line = f"        run: {checker.RUST_DOCTEST_GATE_COMMAND}"
        self.replace_once(
            ".github/workflows/ci.yml",
            f"      - name: Doc tests\n{run_line}",
            f"    x-disabled:\n      - name: Doc tests\n{run_line}",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI step missing or ambiguous in rust steps", issues)

    def test_conditional_rust_doctest_step_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "      - name: Doc tests\n        run:",
            "      - name: Doc tests\n        if: github.event_name != 'pull_request'\n        run:",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI step must be unconditional and blocking", issues)

    def test_nonblocking_rust_doctest_step_fails_closed(self) -> None:
        run_line = f"        run: {checker.RUST_DOCTEST_GATE_COMMAND}"
        self.replace_once(
            ".github/workflows/ci.yml",
            run_line,
            f"{run_line}\n        continue-on-error: true",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI step must be unconditional and blocking", issues)

    def test_missing_pull_request_trigger_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "  pull_request:",
            "  # pull_request:",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI pull_request trigger missing or ambiguous", issues)

    def test_pull_request_token_outside_on_block_fails_closed(self) -> None:
        self.replace_once(
            ".github/workflows/ci.yml",
            "on:\n  pull_request:\n  push:",
            "x-disabled:\n  pull_request:\non:\n  push:",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI pull_request trigger missing or ambiguous", issues)

    def test_multiline_rust_doctest_carrier_requires_contract_update(self) -> None:
        run_line = f"        run: {checker.RUST_DOCTEST_GATE_COMMAND}"
        self.replace_once(
            ".github/workflows/ci.yml",
            run_line,
            f"        run: |\n          {checker.RUST_DOCTEST_GATE_COMMAND}",
        )
        issues: list[str] = []
        checker.check_rust_doctest_gate(issues)
        self.assertIn("Rust doctest CI step must use the exact run command", issues)


class DesignPacketContractTests(unittest.TestCase):
    REAL_SOURCE_COMMIT = "2f4de29032560ff3e13d9994b33a3aff14243f44"
    REAL_SOURCE_TREE = "53e266c47fdb07d50a734faa24bb11ac4bc5527d"

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)
        self.original_resolver = checker.git_commit_tree_hex
        real_tree = self.REAL_SOURCE_TREE
        real_commit = self.REAL_SOURCE_COMMIT
        setattr(
            checker,
            "git_commit_tree_hex",
            lambda commit: real_tree if commit == real_commit else None,
        )

    def tearDown(self) -> None:
        setattr(checker, "git_commit_tree_hex", self.original_resolver)
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def replace_once(self, rel: str, old: str, new: str) -> None:
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        self.assertEqual(text.count(old), 1, f"stale or ambiguous mutation target in {rel}")
        updated = text.replace(old, new, 1)
        self.assertNotEqual(updated, text)
        path.write_text(updated, encoding="utf-8")

    def check_design_packets(self) -> list[str]:
        issues: list[str] = []
        checker.check_design_packets(issues)
        return issues

    def test_current_design_packets_pass(self) -> None:
        self.assertEqual(self.check_design_packets(), [])

    def test_design_packet_invalid_status_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/README.md",
            "| [`m80-default-v0/`](m80-default-v0/) | M80 Default v0 UI design candidate: 17 artifact proposals, 20 required surfaces, 16B disposable static prototype | Reviewed |",
            "| [`m80-default-v0/`](m80-default-v0/) | M80 Default v0 UI design candidate: 17 artifact proposals, 20 required surfaces, 16B disposable static prototype | Draft |",
        )
        self.assertTrue(
            any("design packet index row" in issue and "invalid status" in issue for issue in self.check_design_packets())
        )

    def test_unindexed_design_packet_directory_fails_closed(self) -> None:
        (self.root / "docs/design/stray-packet").mkdir()
        self.assertTrue(
            any("design packet directory/index drift" in issue for issue in self.check_design_packets())
        )

    def test_design_packet_source_binding_malformed_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/README.md",
            f"commit `{self.REAL_SOURCE_COMMIT}`, tree",
            "commit `2f4de29`, tree",
        )
        self.assertTrue(
            any("malformed source binding" in issue for issue in self.check_design_packets())
        )

    def test_design_packet_source_commit_not_resolvable_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/README.md",
            f"commit `{self.REAL_SOURCE_COMMIT}`",
            f"commit `{'0' * 40}`",
        )
        self.assertTrue(
            any("not resolvable in Git" in issue for issue in self.check_design_packets())
        )

    def test_design_packet_source_binding_tree_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/README.md",
            f"tree `{self.REAL_SOURCE_TREE}`",
            f"tree `{'1' * 40}`",
        )
        self.assertTrue(
            any("source binding tree drift" in issue for issue in self.check_design_packets())
        )

    def test_design_packet_readme_status_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/m80-default-v0/README.md",
            "| Status | `Reviewed` |",
            "| Status | `Proposal` |",
        )
        self.assertTrue(
            any("README status drift" in issue for issue in self.check_design_packets())
        )

    def test_design_packet_readme_source_binding_drift_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/m80-default-v0/README.md",
            f"| Source commit | `{self.REAL_SOURCE_COMMIT}` |",
            f"| Source commit | `{'0' * 40}` |",
        )
        self.assertTrue(
            any("README source binding drift" in issue for issue in self.check_design_packets())
        )

    def test_design_packet_missing_readme_fails_closed(self) -> None:
        (self.root / "docs/design/m80-default-v0/README.md").unlink()
        self.assertIn(
            "design packet m80-default-v0 missing README.md",
            self.check_design_packets(),
        )

    def test_design_packet_tree_oid_as_source_commit_fails_closed(self) -> None:
        self.replace_once(
            "docs/design/README.md",
            f"commit `{self.REAL_SOURCE_COMMIT}`",
            f"commit `{self.REAL_SOURCE_TREE}`",
        )
        self.assertTrue(
            any("not resolvable in Git" in issue for issue in self.check_design_packets())
        )

    def test_git_commit_tree_hex_enforces_commit_object_type(self) -> None:
        resolver = self.original_resolver
        setattr(checker, "ROOT", REPO_ROOT)
        try:
            self.assertEqual(resolver(self.REAL_SOURCE_COMMIT), self.REAL_SOURCE_TREE)
            self.assertIsNone(resolver(self.REAL_SOURCE_TREE))
            self.assertIsNone(resolver("0" * 40))
        finally:
            setattr(checker, "ROOT", self.root)


class MarketLifecycleContractTests(unittest.TestCase):
    """The owning market lifecycle contract is registered, non-empty and drift-checked.

    The repository checker registers every current contract in ``KEY_FILES`` and fails when
    one is missing, empty, or introduced without registration. These rows pin that the
    ``market-lifecycle/v0`` contract is admitted through that existing architecture rather
    than through a second checker, and that each failure class is reachable by mutation.
    """

    CONTRACT_REL = "docs/contracts/market-lifecycle.md"

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        ci_path = self.root / ".github/workflows/ci.yml"
        ci_path.parent.mkdir(parents=True)
        shutil.copy2(REPO_ROOT / ".github/workflows/ci.yml", ci_path)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        self.original_key_files = list(checker.KEY_FILES)
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        checker.KEY_FILES[:] = self.original_key_files
        self.temporary_directory.cleanup()

    def contract_path(self) -> Path:
        return self.root / self.CONTRACT_REL

    def check_key_files(self) -> list[str]:
        issues: list[str] = []
        checker.check_key_files_present_and_nonempty(issues)
        return issues

    def test_market_lifecycle_contract_is_registered_and_nonempty(self) -> None:
        # A registered, present, non-empty contract produces no issue naming it. The temp
        # root carries only docs/ plus ci.yml, so unrelated key-file issues are expected;
        # none of them may mention this contract.
        self.assertFalse(
            any("market-lifecycle.md" in issue for issue in self.check_key_files()),
            self.check_key_files(),
        )

    def test_missing_market_lifecycle_contract_fails_closed(self) -> None:
        self.contract_path().unlink()
        self.assertIn(
            f"key file missing: {self.CONTRACT_REL}",
            self.check_key_files(),
        )

    def test_empty_market_lifecycle_contract_fails_closed(self) -> None:
        self.contract_path().write_text(" \n", encoding="utf-8")
        self.assertIn(
            f"key file empty: {self.CONTRACT_REL}",
            self.check_key_files(),
        )

    def test_unregistered_market_lifecycle_contract_fails_closed(self) -> None:
        # The file remains present and non-empty, but is removed from the registration list,
        # so the existing contract-registration architecture must report it as an
        # unregistered current contract rather than admit it silently.
        checker.KEY_FILES.remove(self.CONTRACT_REL)
        self.assertIn(
            f"current contract not registered as key file: {self.CONTRACT_REL}",
            self.check_key_files(),
        )


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

    def test_m00_exact_active_acceptance_reference_requires_matrix_row(self) -> None:
        self.remove_table_row("docs/acceptance/matrix.tsv", "AUTH-011\t")
        issues = self.check_module_registry()
        self.assertTrue(
            any(
                "active acceptance reference is not registered in matrix.tsv for M00: AUTH-011"
                in issue
                for issue in issues
            ),
            issues,
        )

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
        # Anchored on whatever state key the row currently carries rather than on a literal
        # one, so a legitimate module promotion cannot silently stop exercising this check.
        path = self.root / "docs/plan/modules/00-module-map.md"
        rows = path.read_text(encoding="utf-8").splitlines()
        prefix = "| `M50` | Model Provider Integration | "
        matches = [index for index, row in enumerate(rows) if row.startswith(prefix)]
        self.assertEqual(len(matches), 1, matches)
        current_state, separator, remainder = rows[matches[0]][len(prefix) :].partition(" | ")
        self.assertEqual(separator, " | ", rows[matches[0]])
        self.assertIn(current_state.strip("`"), checker.VALID_MODULE_STATES)
        rows[matches[0]] = f"{prefix}`complete` | {remainder}"
        path.write_text("\n".join(rows) + "\n", encoding="utf-8")
        issues = self.check_module_registry()
        self.assertTrue(any("unknown state key for M50" in issue for issue in issues), issues)

    def test_module_state_mismatch_fails_closed(self) -> None:
        path = self.root / checker.MODULE_BLUEPRINTS["M80"]
        text = path.read_text(encoding="utf-8")
        match = re.search(r"`Implementation State`: `([^`]+)`", text)
        if match is None:
            self.fail("M80 blueprint must carry exactly one Implementation State field")
        current_state = match.group(1)
        self.assertIn(current_state, checker.VALID_MODULE_STATES)
        replacement_state = "planned" if current_state != "planned" else "skeleton"
        path.write_text(
            text.replace(
                f"`Implementation State`: `{current_state}`",
                f"`Implementation State`: `{replacement_state}`",
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
                "`long-horizon:AI-*`",
                "`active:AI-*`",
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
                "`long-horizon:AI-*`",
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
        path.write_text(
            text.replace(
                "`active:AUTH-011`",
                "active rows missing",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_module_registry()
        self.assertTrue(
            any("acceptance projection has an unstructured token" in issue for issue in issues),
            issues,
        )


class M60B2RetrievalPolicyContractTests(unittest.TestCase):
    """Pins the accepted R11 semantic packet for the two-layer M60/M90 transport architecture.

    The immutable block between BEGIN/END markers must be exactly 33046 bytes with the bound
    SHA-256. Stage must be M60_B2_CONTRACT_ACCEPTED, packet status ACCEPTED, with one current
    R11 acceptance receipt and the historical R4 direction receipt preserved as
    PROPOSED_NOT_ACCEPTED. The two-layer transport boundary must be projected.
    """

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def replace_once(self, rel: str, old: str, new: str) -> None:
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        self.assertEqual(text.count(old), 1, f"stale or ambiguous mutation target in {rel}")
        updated = text.replace(old, new, 1)
        self.assertNotEqual(updated, text)
        path.write_text(updated, encoding="utf-8")

    def check_packet(self) -> list[str]:
        issues: list[str] = []
        checker.check_m60_b2_packet_digest(issues)
        return issues

    def assert_rejected(self, issues: list[str], marker: str) -> None:
        self.assertTrue(any(marker in issue for issue in issues), issues)

    def test_current_m60_b2_packet_passes(self) -> None:
        self.assertEqual(self.check_packet(), [])

    # --- frozen packet integrity ---

    def test_missing_begin_marker_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_PROPOSAL_PATH,
            checker.M60_B2_PROPOSAL_BEGIN,
            "<!-- removed begin marker -->",
        )
        self.assert_rejected(self.check_packet(), "marker count drift")

    def test_missing_end_marker_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_PROPOSAL_PATH,
            checker.M60_B2_PROPOSAL_END,
            "<!-- removed end marker -->",
        )
        self.assert_rejected(self.check_packet(), "marker count drift")

    def test_duplicate_begin_marker_fails_closed(self) -> None:
        path = self.root / checker.M60_B2_PROPOSAL_PATH
        text = path.read_text(encoding="utf-8")
        start = text.index(checker.M60_B2_PROPOSAL_BEGIN)
        finish = text.index(checker.M60_B2_PROPOSAL_END)
        block = text[start : finish + len(checker.M60_B2_PROPOSAL_END)]
        path.write_text(text + "\n" + block + "\n", encoding="utf-8")
        self.assert_rejected(self.check_packet(), "marker count drift")

    def test_block_content_drift_changes_digest(self) -> None:
        path = self.root / checker.M60_B2_PROPOSAL_PATH
        self.replace_once(
            checker.M60_B2_PROPOSAL_PATH,
            "any push, PR, merge, tag, release, deployment or publication lacks current operation-specific or active-campaign authority.",
            "any push, PR, merge, tag, release, deployment or publication is always authorized.",
        )
        self.assert_rejected(self.check_packet(), "digest drift")

    def test_digest_metadata_drift_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_PROPOSAL_PATH,
            checker.M60_B2_REQUIRED_DIGEST_LINE,
            "- `R4 replacement packet digest`: `sha256:" + "0" * 64 + "` over `1` bytes",
        )
        self.assert_rejected(self.check_packet(), "digest metadata must bind")

    # --- replacement stage ---

    def test_missing_replacement_stage_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_PROPOSAL_PATH,
            checker.M60_B2_REQUIRED_STAGE_LINE,
            "`M60_B2_ACCEPTED_WITH_AMENDMENT`",
        )
        issues = self.check_packet()
        self.assert_rejected(issues, "R3 stage missing")

    def test_replacement_stage_promoted_incorrectly_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_PROPOSAL_PATH,
            checker.M60_B2_REQUIRED_STAGE_LINE,
            "`M60_B2_IMPLEMENTATION_READY`",
        )
        self.assert_rejected(self.check_packet(), "R3 stage missing")

    # --- packet status ---

    def test_packet_status_false_promotion_fails_closed(self) -> None:
        prop_path = self.root / checker.M60_B2_PROPOSAL_PATH
        text = prop_path.read_text(encoding="utf-8")
        changed = text.replace(
            checker.M60_B2_REQUIRED_STATUS_LINE,
            "- `Packet status`: `IMPLEMENTATION_READY`",
            1,
        )
        self.assertNotEqual(changed, text)
        prop_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(self.check_packet(), "packet status missing")

    # --- lifecycle precondition ---

    def test_lifecycle_precondition_removed_fails_closed(self) -> None:
        contract_path = self.root / checker.M60_B2_SOURCE_RETRIEVAL_PATH
        text = contract_path.read_text(encoding="utf-8")
        changed = text.replace(
            "Operational `Suspended`/`Revoked` lifecycle and monotone "
            "`SourceAuthorityRevision` must be present before any live B2 retrieval adapter",
            "Lifecycle is optional; retrieval may proceed without Suspended/Revoked support",
            1,
        )
        self.assertNotEqual(changed, text)
        contract_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "source-retrieval contract stop-condition section must carry the operational",
        )

    # --- two-layer transport boundary projection ---

    def test_transport_boundary_removed_fails_closed(self) -> None:
        contract_path = self.root / checker.M60_B2_SOURCE_RETRIEVAL_PATH
        text = contract_path.read_text(encoding="utf-8")
        changed = text.replace(
            "M90 never receives `EffectReadyRetrievalPlan`",
            "M90 may receive EffectReadyRetrievalPlan",
            1,
        )
        self.assertNotEqual(changed, text)
        contract_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "source-retrieval contract §1.1 must carry exactly one line stating "
            "M90 never receives EffectReadyRetrievalPlan and never returns BoundedFetch",
        )

    def test_source_transport_port_missing_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "pub trait SourceTransportPort: Send + Sync {",
            "pub trait SourceFetchPort: Send + Sync {",
        )
        self.assert_rejected(
            self.check_packet(),
            "source-retrieval contract §9.3 must carry exactly one public one-call "
            "SourceTransportPort trait signature",
        )

    def test_source_transport_request_public_constructor_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "No public constructor, Serde, `Default`, URL/header builder conversion, arbitrary headers or retry/proxy/cookie/auth knobs.",
            "A public constructor may accept arbitrary headers and auth knobs.",
        )
        self.assert_rejected(
            self.check_packet(),
            "source-retrieval contract §9.1 must declare the exact authority-sentence",
        )

    def test_transport_request_response_bound_removed_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "`u32 maximum_response_bytes`",
            "`u32 omitted_response_bound`",
        )
        self.assert_rejected(
            self.check_packet(),
            "authoritative RetrievalTransportRequest owned-field sequence drifted",
        )

    def test_transport_request_deadline_accessor_removed_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "pub fn RetrievalTransportRequest::maximum_elapsed_seconds(&self) -> u32",
            "pub fn RetrievalTransportRequest::omitted_deadline(&self) -> u32",
        )
        self.assert_rejected(
            self.check_packet(),
            "exact RetrievalTransportRequest accessor surface drifted",
        )

    def test_transport_request_accessor_duplicated_fails_closed(self) -> None:
        path = self.root / checker.M60_B2_SOURCE_RETRIEVAL_PATH
        text = path.read_text(encoding="utf-8")
        signature = "pub fn RetrievalTransportRequest::maximum_response_bytes(&self) -> u32"
        self.assertEqual(text.count(signature), 1)
        path.write_text(text.replace(signature, signature + "\n" + signature, 1), encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "exact RetrievalTransportRequest accessor surface drifted",
        )

    def test_transport_request_decoy_prose_cannot_mask_authority_drift(self) -> None:
        path = self.root / checker.M60_B2_SOURCE_RETRIEVAL_PATH
        text = path.read_text(encoding="utf-8")
        field = "`u32 maximum_response_bytes`"
        accessor = "pub fn RetrievalTransportRequest::maximum_elapsed_seconds(&self) -> u32"
        self.assertEqual(text.count(field), 1)
        self.assertEqual(text.count(accessor), 1)
        text = text.replace(field, "`u32 omitted_response_bound`", 1)
        text = text.replace(
            accessor,
            "pub fn RetrievalTransportRequest::omitted_deadline(&self) -> u32",
            1,
        )
        text = text.replace(
            "There are no additional accessors or conversions.",
            "There are no additional accessors or conversions.\n\n"
            "Terminology-only decoys: " + field + " and `" + accessor + "`.",
            1,
        )
        path.write_text(text, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "authoritative RetrievalTransportRequest owned-field sequence drifted",
        )

    def test_transport_request_extra_owned_field_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "`u32 maximum_elapsed_seconds`, `SourceRetrievalProtocolVersion`",
            "`u32 maximum_elapsed_seconds`, `u32 retry_count`, "
            "`SourceRetrievalProtocolVersion`",
        )
        self.assert_rejected(
            self.check_packet(),
            "authoritative RetrievalTransportRequest owned-field sequence drifted",
        )

    def test_transport_request_duplicate_authority_sentence_fails_closed(self) -> None:
        path = self.root / checker.M60_B2_SOURCE_RETRIEVAL_PATH
        text = path.read_text(encoding="utf-8")
        prefix = (
            "`RetrievalTransportRequest` is a non-authority, private-field, owned struct "
            "containing exactly the adapter inputs needed for deterministic transport. It owns "
            "all values ("
        )
        self.assertEqual(text.count(prefix), 1)
        start = text.index(prefix)
        finish = text.index(") and carries no lifetime parameter.", start)
        sentence = text[start : finish + len(") and carries no lifetime parameter.")]
        duplicate = sentence.replace(
            "`u32 maximum_elapsed_seconds`, `SourceRetrievalProtocolVersion`",
            "`u32 maximum_elapsed_seconds`, `u32 retry_count`, "
            "`SourceRetrievalProtocolVersion`",
            1,
        )
        insertion = (
            "There are no additional accessors or conversions.\n\n" + duplicate
        )
        self.assertEqual(text.count("There are no additional accessors or conversions."), 1)
        path.write_text(
            text.replace(
                "There are no additional accessors or conversions.",
                insertion,
                1,
            ),
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_packet(),
            "must carry exactly one authoritative RetrievalTransportRequest "
            "owned-field sentence, found 2",
        )

    def test_transport_request_extra_accessor_fails_closed(self) -> None:
        signature = "pub fn RetrievalTransportRequest::maximum_elapsed_seconds(&self) -> u32"
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            signature,
            signature + "\npub fn RetrievalTransportRequest::retry_count(&self) -> u32",
        )
        self.assert_rejected(
            self.check_packet(),
            "exact RetrievalTransportRequest accessor surface drifted",
        )

    def test_transport_success_domain_authority_constructor_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "No observation constructor returns or embeds `RetrievalPolicyError`, `BoundedFetch`, receipt, phase carrier or authority witness.",
            "An observation constructor may return `BoundedFetch` and an authority witness.",
        )
        self.assert_rejected(
            self.check_packet(),
            "source-retrieval contract §9.2 must declare the exact authority-sentence",
        )

    def test_transport_request_cloneability_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "It is `Debug + Eq`, not `Clone`/`Copy`",
            "It is `Clone + Debug + Eq`",
        )
        self.assert_rejected(self.check_packet(), "must declare not Clone")

    def test_transport_success_consuming_projection_removal_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "pub fn RetrievalTransportSuccess::into_parts(self) -> RetrievalTransportSuccessParts",
            "pub fn RetrievalTransportSuccess::parts(&self) -> &RetrievalTransportSuccessParts",
        )
        self.assert_rejected(self.check_packet(), "must retain a consuming success projection")

    def test_transport_success_exact_parts_type_removal_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "pub fn RetrievalTransportSuccess::into_parts(self) -> RetrievalTransportSuccessParts",
            "pub fn RetrievalTransportSuccess::into_parts(self) -> (Vec<u8>, Vec<u8>)",
        )
        self.assert_rejected(self.check_packet(), "§9.2 exact transport success API drifted")

    def test_transport_success_constructor_trait_row_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "| `RetrievalTransportSuccess` | no | no | no | no | yes (shape-only) | yes | no |",
            "| `RetrievalTransportSuccess` | no | no | no | no | no (private-field) | yes | no |",
        )
        self.assert_rejected(self.check_packet(), "§11.2 trait projection drifted")

    def test_dns_transport_observation_cannot_return_policy_error(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            ") -> Result<DnsTransportObservation, SourceTransportError>",
            ") -> Result<DnsTransportObservation, RetrievalPolicyError>",
        )
        self.assert_rejected(self.check_packet(), "§7.3 raw DNS/M60 policy split drifted")

    def test_public_transport_port_domain_error_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            ") -> Pin<Box<dyn Future<Output = Result<RetrievalTransportSuccess, SourceTransportError>> + Send + 'a>>;",
            ") -> Pin<Box<dyn Future<Output = Result<RetrievalTransportSuccess, SourceFetchFailure>> + Send + 'a>>;",
        )
        self.assert_rejected(self.check_packet(), "must use the transport-only")

    def test_source_fetch_failure_public_inventory_fails_closed(self) -> None:
        path = self.root / checker.M60_B2_SOURCE_RETRIEVAL_PATH
        text = path.read_text(encoding="utf-8")
        private_row = "| `SourceFetchFailure` | §11 |"
        private_heading = "Crate-private/internal values (no public constructor, no Serde, no Clone unless specified):"
        self.assertEqual(text.count(private_row), 1)
        text = text.replace(private_row + "\n", "", 1)
        text = text.replace(private_heading, private_row + "\n\n" + private_heading, 1)
        path.write_text(text, encoding="utf-8")
        self.assert_rejected(self.check_packet(), "must keep SourceFetchFailure out of public inventory")

    def test_carrier_trait_projection_drift_fails_closed(self) -> None:
        self.replace_once(
            checker.M60_B2_SOURCE_RETRIEVAL_PATH,
            "| `RetrievalReplayIdentity` | no | no | no | no | no (owner-private) | yes | no |",
            "| `RetrievalReplayIdentity` | yes | no | no | no | no (owner-private) | yes | no |",
        )
        self.assert_rejected(self.check_packet(), "§11.2 trait projection drifted")

    # --- module boundaries projection ---

    def test_module_boundaries_retains_old_fetch_fails(self) -> None:
        boundaries_path = self.root / "docs/contracts/module-boundaries.md"
        text = boundaries_path.read_text(encoding="utf-8")
        changed = text.replace(
            "B-M60-M90-SOURCE-TRANSPORT",
            "B-M60-M90-SOURCE-FETCH",
            1,
        )
        boundaries_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "module-boundaries must project the exact B-M60-M90-SOURCE-TRANSPORT",
        )

    def test_module_boundaries_domain_error_fails_closed(self) -> None:
        boundaries_path = self.root / "docs/contracts/module-boundaries.md"
        text = boundaries_path.read_text(encoding="utf-8")
        changed = text.replace(
            "returns only `RetrievalTransportSuccess` or transport-only `SourceTransportError`",
            "returns `RetrievalTransportSuccess` or domain `SourceFetchFailure`",
            1,
        )
        self.assertNotEqual(changed, text)
        boundaries_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "module-boundaries must project the exact B-M60-M90-SOURCE-TRANSPORT",
        )

    def test_module_boundaries_m90_clock_fails_closed(self) -> None:
        boundaries_path = self.root / "docs/contracts/module-boundaries.md"
        text = boundaries_path.read_text(encoding="utf-8")
        injected = "| `B-M60-M90-RETRIEVAL-CLOCK` | `M60` | `M90` | clock | accepted contract |\n"
        boundaries_path.write_text(text + injected, encoding="utf-8")
        self.assert_rejected(self.check_packet(), "must keep RetrievalClockPort M60-internal")

    def test_blueprint_false_authority_promotion_fails_closed(self) -> None:
        blueprint_path = self.root / "docs/plan/modules/70-campus-trust-source-pipeline.md"
        text = blueprint_path.read_text(encoding="utf-8")
        changed = text.replace(
            "operational Suspended/Revoked accepted as contract authority in source-import/v1, not implemented;",
            "operational Suspended/Revoked accepted as authority in source-import/v1;",
            1,
        )
        self.assertNotEqual(changed, text)
        blueprint_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(self.check_packet(), "M60 blueprint must not promote")

    def test_blueprint_regression_to_proposed_fails_closed(self) -> None:
        blueprint_path = self.root / "docs/plan/modules/70-campus-trust-source-pipeline.md"
        text = blueprint_path.read_text(encoding="utf-8")
        changed = text.replace(
            "operational Suspended/Revoked accepted as contract authority in source-import/v1, not implemented;",
            "operational Suspended/Revoked proposed as authority in source-import/v1, not accepted;",
            1,
        )
        self.assertNotEqual(changed, text)
        blueprint_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(self.check_packet(), "M60 blueprint must not regress to proposed-only")

    # --- R11 acceptance receipt and historical R4 receipt ---

    def test_r11_acceptance_receipt_missing_fails_closed(self) -> None:
        prop_path = self.root / checker.M60_B2_PROPOSAL_PATH
        text = prop_path.read_text(encoding="utf-8")
        changed = text.replace(
            "### M60-B2 R11 exact semantic acceptance receipt (CURRENT)",
            "### M60-B2 R11 exact semantic acceptance receipt (REMOVED)",
            1,
        )
        self.assertNotEqual(changed, text)
        prop_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "must have exactly one R11 exact semantic acceptance receipt",
        )

    def test_r11_acceptance_receipt_duplicated_fails_closed(self) -> None:
        prop_path = self.root / checker.M60_B2_PROPOSAL_PATH
        text = prop_path.read_text(encoding="utf-8")
        header = "### M60-B2 R11 exact semantic acceptance receipt (CURRENT)"
        self.assertEqual(text.count(header), 1)
        prop_path.write_text(text + "\n" + header + "\n- duplicate receipt\n", encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "must have exactly one R11 exact semantic acceptance receipt",
        )

    def test_historical_r4_receipt_removed_fails_closed(self) -> None:
        prop_path = self.root / checker.M60_B2_PROPOSAL_PATH
        text = prop_path.read_text(encoding="utf-8")
        changed = text.replace(
            "### M60-B2 R4 replacement direction receipt (HISTORICAL — SUPERSEDED BY R11 ACCEPTANCE)",
            "### M60-B2 R4 replacement direction receipt (REMOVED)",
            1,
        )
        self.assertNotEqual(changed, text)
        prop_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "must preserve exactly one historical R4 replacement direction receipt",
        )

    def test_historical_r4_receipt_status_promoted_fails_closed(self) -> None:
        prop_path = self.root / checker.M60_B2_PROPOSAL_PATH
        text = prop_path.read_text(encoding="utf-8")
        self.assertIn("- `Receipt status`: `PROPOSED_NOT_ACCEPTED`", text)
        changed = text.replace(
            "- `Receipt status`: `PROPOSED_NOT_ACCEPTED`",
            "- `Receipt status`: `ACCEPTED`",
            1,
        )
        self.assertNotEqual(changed, text)
        prop_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "historical R4 replacement direction receipt must preserve",
        )

    # --- acceptance matrix ---

    def test_src010_false_promotion_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        header = lines[0].split("\t")
        status_index = header.index("status")
        modified: list[str] = [lines[0]]
        for line in lines[1:]:
            cells = line.split("\t")
            if cells[0] == "SRC-010":
                cells[status_index] = "implemented"
            modified.append("\t".join(cells))
        path.write_text("\n".join(modified) + "\n", encoding="utf-8")
        self.assert_rejected(self.check_packet(), "SRC-010 must remain planned")

    def test_src014_in_active_matrix_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text
            + "SRC-014\tsource\tsuspended/revoked source blocks fetch\tfuture integration test\trelease\tplanned\tsecurity\n",
            encoding="utf-8",
        )
        self.assert_rejected(self.check_packet(), "SRC-014 must remain catalog-only")

    def test_lf_crlf_byte_mutation_fails(self) -> None:
        path = self.root / checker.M60_B2_PROPOSAL_PATH
        raw = path.read_bytes()
        begin = checker.M60_B2_PROPOSAL_BEGIN.encode("utf-8") + b"\n"
        end = checker.M60_B2_PROPOSAL_END.encode("utf-8")
        start = raw.index(begin) + len(begin)
        finish = raw.index(end)
        before = raw[:start]
        block = raw[start:finish]
        after = raw[finish:]
        mutated = block.replace(b"\n", b"\r\n")
        self.assertNotEqual(mutated, block, "CRLF mutation did not change block")
        path.write_bytes(before + mutated + after)
        self.assert_rejected(self.check_packet(), "byte count drift")

    # --- coverage matrix projection ---

    def test_coverage_matrix_missing_transport_boundary_fails(self) -> None:
        coverage_path = self.root / "docs/coverage-matrix.md"
        text = coverage_path.read_text(encoding="utf-8")
        changed = text.replace(
            "B-M60-M90-SOURCE-TRANSPORT",
            "B-M60-M90-SOURCE-FETCH",
            1,
        )
        coverage_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(
            self.check_packet(),
            "coverage-matrix M60 row must project B-M60-M90-SOURCE-TRANSPORT",
        )

    def test_coverage_matrix_m90_clock_fails_closed(self) -> None:
        coverage_path = self.root / "docs/coverage-matrix.md"
        text = coverage_path.read_text(encoding="utf-8")
        changed = text.replace(
            "`B-M60-M90-SOURCE-TRANSPORT`",
            "`B-M60-M90-RETRIEVAL-CLOCK`, `B-M60-M90-SOURCE-TRANSPORT`",
            1,
        )
        self.assertNotEqual(changed, text)
        coverage_path.write_text(changed, encoding="utf-8")
        self.assert_rejected(self.check_packet(), "must not project an M60-M90 retrieval-clock boundary")


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


class RustLexicalStripperTests(unittest.TestCase):
    """Adversarial cases for the scan that the forbidden-carrier checks depend on.

    A bug here fails open: a forbidden carrier hidden in an exotic literal would survive the
    strip and never be matched, or real code would be eaten and never scanned at all.
    """

    def strip(self, source: str) -> str:
        return checker.strip_rust_comments_and_literals(source)

    def test_line_and_nested_block_comments_are_removed(self) -> None:
        stripped = self.strip(
            "// reqwest\n/* outer /* inner SystemTime */ still uuid */\nlet keep = 1;\n"
        )
        self.assertNotIn("reqwest", stripped)
        self.assertNotIn("SystemTime", stripped)
        self.assertNotIn("uuid", stripped)
        self.assertIn("let keep", stripped)

    def test_ordinary_byte_and_raw_string_literals_are_removed(self) -> None:
        stripped = self.strip(
            'let a = "rand";\n'
            'let b = b"uuid";\n'
            'let c = r"chrono";\n'
            'let d = r#"use std::fs; mint"#;\n'
            'let e = br##"ulid "# still"##;\n'
            "let keep = 1;\n"
        )
        for carrier in ("rand", "uuid", "chrono", "std::fs", "mint", "ulid"):
            self.assertNotIn(carrier, stripped, carrier)
        self.assertIn("let keep", stripped)
        for binding in ("let a", "let b", "let c", "let d", "let e"):
            self.assertIn(binding, stripped, binding)

    def test_escaped_quote_does_not_end_a_literal_early(self) -> None:
        stripped = self.strip('let a = "escaped \\" still inside rand";\nlet keep = 1;\n')
        self.assertNotIn("rand", stripped)
        self.assertIn("let keep", stripped)

    def test_lifetimes_are_not_mistaken_for_char_literals(self) -> None:
        # If a lifetime were consumed as a char literal the scanner would swallow real code.
        stripped = self.strip(
            "impl<'de> Deserialize<'de> for Value {}\n"
            "fn kind(&self) -> &'static str { \"x\" }\n"
            "const D: bool = matches!(byte, b'-' | b'.');\n"
        )
        self.assertIn("Deserialize", stripped)
        self.assertIn("for Value", stripped)
        self.assertIn("&'static str", stripped)
        self.assertIn("matches!(byte", stripped)
        self.assertNotIn("b'-'", stripped)

    def test_code_carriers_survive_the_strip(self) -> None:
        stripped = self.strip(
            "use std::fmt;\n/// doc mentioning uuid\npub fn parse() {}\n"
        )
        self.assertIn("use std::fmt;", stripped)
        self.assertIn("pub fn parse()", stripped)
        self.assertNotIn("uuid", stripped)


class RustLexicalDifferentialTests(unittest.TestCase):
    """Pins the evidence lexer against the shared cross-language corpus.

    The same rules are implemented twice, in `scripts/check_repo_contracts.py` and in
    `crates/platform-core/tests/platform_identity.rs`. Both compare their own output against
    this one file, so a divergence fails whichever carrier drifted instead of surviving until a
    reviewer happens to probe the right input. The corpus is deliberately adversarial:
    comment-split keywords, byte-char literals, raw identifiers, nested use trees, restricted
    visibility, multi-line attributes and non-ASCII identifiers.
    """

    CORPUS = REPO_ROOT / "scripts/tests/data/rust_lexical_corpus.json"

    def cases(self) -> list[dict[str, object]]:
        payload = json.loads(self.CORPUS.read_text(encoding="utf-8"))
        return cast(list[dict[str, object]], payload["cases"])

    def test_corpus_is_non_trivial(self) -> None:
        cases = self.cases()
        self.assertGreaterEqual(len(cases), 50)
        sources = [cast(str, case["source"]) for case in cases]
        self.assertEqual(len(set(sources)), len(sources), "duplicate corpus case")
        # The corpus is worthless if it does not carry the classes that actually broke.
        for required in (
            "extern/**/crate self as z;",
            "# /*inner*/ ! [allow(dead_code)]",
            'include/*x*/!("a.rs");',
            "let x = foo.r#type;",
            "macro_rules! assert_eq { ($($a:tt)*) => {{ }}; }",
            "macro_rules! g { ($x:expr) => {{ 1 }}; ($k:ty) => {{ 2 }}; }",
            "# [derive(Clone, Copy)]",
            "macro_rules !shadow { () => {{}}; }",
            "#[r#derive(Default)]",
            "# [ r#derive ( Copy ) ]",
            "#[r#ignore] #[r#test] fn t() {}",
            "enum E { #[r#default] A }",
            "x#[a] pub fn f() {}",
            "#[$]",
            "fn f<T: Into<Vec<u8>>>(x: T) {}",
            "fn g(x: [u8; { 2 }]) { 1 }",
            "trait T { fn h(&self); }",
            "let p: fn(u8) -> u8 = q;",
            "fn r#match() { 1 }",
            "fn outer() { fn inner() { 1 } }",
            "fn \u00e9q() { 1 }",
            "let a = \"x\"; /*c*/ let b = b'-'; let c = r#\"r\"#; let d = b\"y\"; // t",
        ):
            self.assertIn(required, sources, f"corpus lost the {required!r} case")

    def test_lexer_matches_the_shared_corpus(self) -> None:
        for case in self.cases():
            source = cast(str, case["source"])
            with self.subTest(source=source):
                stripped = checker.strip_rust_comments_and_literals(source)
                self.assertEqual(stripped, case["stripped"])
                self.assertEqual(
                    checker.strip_rust_comments_and_literals(source, keep_literals=True),
                    case["stripped_literals"],
                )
                items, item_unterminated = checker.rust_item_declarations(stripped)
                self.assertEqual(items, case["items"])
                self.assertEqual(item_unterminated, case["item_unterminated"])
                impls, impl_unclassified = checker.rust_impl_declarations(stripped)
                self.assertEqual(impls, case["impls"])
                self.assertEqual(bool(impl_unclassified), case["impl_unclassified"])
                self.assertEqual(
                    checker.rust_macro_definitions(stripped), case["macro_definitions"]
                )
                invocations, unterminated = checker.rust_macro_invocation_arguments(stripped)
                self.assertEqual(
                    [f"{name}!({argument})" for name, argument in invocations],
                    case["macro_invocations"],
                )
                self.assertEqual(sorted(unterminated), case["macro_unterminated"])
                self.assertEqual(
                    [[name, matchers] for name, matchers in checker.rust_macro_arms(stripped)],
                    case["macro_arms"],
                )
                self.assertEqual(checker.rust_derive_bodies(stripped), case["derives"])
                found, unterminated_attributes = checker.rust_attributes(stripped)
                self.assertEqual(
                    [[inner, name, body] for inner, name, body in found], case["attributes"]
                )
                self.assertEqual(unterminated_attributes, case["attribute_unterminated"])
                self.assertEqual(
                    checker.rust_string_literals(source), case["string_literals"]
                )
                declared, unresolved_functions = checker.rust_functions(stripped)
                self.assertEqual([[name, body] for name, body in declared], case["functions"])
                self.assertEqual(unresolved_functions, case["function_unresolved"])


class PlatformIdentityGrammarAuthorityTests(unittest.TestCase):
    """Pins the semantic authority chain: contract → checker table → production/oracle/corpora.

    Round 14 froze every function body exactly, but over code with literal payloads stripped, and
    the exhaustive byte oracle meant to cover the residue carried a delimiter table of its own. So
    production, oracle, both corpora, the fixtures, their digests and the projection goldens could
    be moved from `:` to `?` TOGETHER and every mechanical gate stayed green while `a?b` was
    accepted and `a:b` rejected. These tests exist because agreement among mutable carriers is not
    evidence; the root of the chain has to be the accepted contract.
    """

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "crates", self.root / "crates")
        contracts = self.root / "docs/contracts"
        contracts.mkdir(parents=True)
        shutil.copy2(
            REPO_ROOT / checker.PLATFORM_IDENTITY_CONTRACT,
            self.root / checker.PLATFORM_IDENTITY_CONTRACT,
        )
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        self.original_grammar = dict(checker.PLATFORM_IDENTITY_GRAMMAR)
        self.original_digests = dict(checker.INVOCATION_FIXTURE_DIGESTS)
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        # Every module-level table this class mutates is restored, so a test that drifts one
        # cannot leak into a sibling and turn a real rejection into a false green somewhere else.
        setattr(checker, "ROOT", self.original_root)
        checker.PLATFORM_IDENTITY_GRAMMAR.clear()
        checker.PLATFORM_IDENTITY_GRAMMAR.update(self.original_grammar)
        checker.INVOCATION_FIXTURE_DIGESTS.clear()
        checker.INVOCATION_FIXTURE_DIGESTS.update(self.original_digests)
        self.temporary_directory.cleanup()

    def check_grammar(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_identity_grammar_authority(issues)
        return issues

    def contract_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_CONTRACT

    def source_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_SOURCE

    def bound_test_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_TEST

    def rewrite(self, path: Path, old: str, new: str, occurrences: int = 1) -> None:
        text = path.read_text(encoding="utf-8")
        self.assertEqual(
            text.count(old), occurrences, f"stale mutation target in {path.name}: {old!r}"
        )
        path.write_text(text.replace(old, new), encoding="utf-8")

    def assert_rejected(self, issues: list[str], marker: str) -> None:
        self.assertTrue(any(marker in issue for issue in issues), issues)

    def drift_production(self) -> None:
        self.rewrite(
            self.source_path(),
            "matches!(byte, b'-' | b'.' | b'_' | b':')",
            "matches!(byte, b'-' | b'.' | b'_' | b'?')",
        )
        self.rewrite(
            self.source_path(),
            "`^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$`",
            "`^[A-Za-z0-9](?:[-A-Za-z0-9._?]{0,126}[A-Za-z0-9])?$`",
        )

    def drift_oracle(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            'let admitted_interior: [u8; 4] = *b"-._:";',
            'let admitted_interior: [u8; 4] = *b"-._?";',
        )

    def drift_corpora(self) -> None:
        self.rewrite(
            self.bound_test_path(), '"Tenant.Alpha_Beta:Gamma-01"', '"Tenant.Alpha_Beta?Gamma-01"', 2
        )
        self.rewrite(self.bound_test_path(), '"a..__::--b"', '"a..__??--b"')

    # G01 — production alone.
    def test_production_only_delimiter_drift_fails_closed(self) -> None:
        self.drift_production()
        issues = self.check_grammar()
        self.assert_rejected(issues, "production interior delimiters")
        self.assert_rejected(issues, "grammar-contract mismatch")

    # G02 — production plus the oracle that was supposed to catch it.
    def test_production_and_oracle_delimiter_drift_fails_closed(self) -> None:
        self.drift_production()
        self.drift_oracle()
        issues = self.check_grammar()
        self.assert_rejected(issues, "production interior delimiters")
        self.assert_rejected(issues, "oracle interior delimiters")

    # G03/G04 — every mutable carrier moved together. The contract is the only root left.
    def test_full_coordinated_delimiter_drift_fails_closed(self) -> None:
        self.drift_production()
        self.drift_oracle()
        self.drift_corpora()
        issues = self.check_grammar()
        self.assert_rejected(issues, "grammar-contract mismatch")
        self.assert_rejected(issues, "production interior delimiters")

    # G05 — the checker's own table moved as well, contract untouched.
    def test_checker_table_codrift_fails_against_the_accepted_contract(self) -> None:
        self.drift_production()
        self.drift_oracle()
        self.drift_corpora()
        checker.PLATFORM_IDENTITY_GRAMMAR["interior_extra_bytes"] = "-._?"
        issues = self.check_grammar()
        self.assert_rejected(issues, "interior delimiter set")
        self.assert_rejected(issues, "interior line names")

    # G05 variant — the regex field alone, which the structural parse must also reject.
    def test_checker_table_regex_codrift_fails_against_the_accepted_contract(self) -> None:
        checker.PLATFORM_IDENTITY_GRAMMAR["regex"] = (
            "^[A-Za-z0-9](?:[-A-Za-z0-9._?]{0,126}[A-Za-z0-9])?$"
        )
        self.assert_rejected(self.check_grammar(), "contract regex")

    # G06 — the length bound.
    def test_coordinated_max_byte_drift_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "const MAX_IDENTITY_BYTES: usize = 129;",
        )
        self.rewrite(
            self.bound_test_path(),
            "const MAX_BYTES: usize = 128;",
            "const MAX_BYTES: usize = 129;",
        )
        self.assert_rejected(self.check_grammar(), "production max bytes 129 != contract 128")

    def test_checker_table_max_byte_codrift_fails_closed(self) -> None:
        checker.PLATFORM_IDENTITY_GRAMMAR["max_bytes"] = 129
        issues = self.check_grammar()
        self.assert_rejected(issues, "contract regex admits 128 bytes != checker table 129")
        self.assert_rejected(issues, "max-byte line is")

    # G07 — the boundary class.
    def test_boundary_class_drift_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const fn is_boundary_byte(byte: u8) -> bool {\n    byte.is_ascii_alphanumeric()\n}",
            "const fn is_boundary_byte(byte: u8) -> bool {\n"
            "    byte.is_ascii_alphanumeric() || byte == b'_'\n}",
        )
        self.assert_rejected(self.check_grammar(), "production boundary class")

    # G08 — a fifth delimiter, with all four admitted ones preserved.
    def test_extra_interior_delimiter_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "matches!(byte, b'-' | b'.' | b'_' | b':')",
            "matches!(byte, b'-' | b'.' | b'_' | b':' | b'+')",
        )
        self.assert_rejected(self.check_grammar(), "(count 5) != contract")

    # G09 — one delimiter duplicated while another is omitted, so the COUNT alone is unchanged.
    def test_duplicated_interior_delimiter_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "matches!(byte, b'-' | b'.' | b'_' | b':')",
            "matches!(byte, b'-' | b'.' | b'_' | b'-')",
        )
        self.assert_rejected(self.check_grammar(), "production interior delimiters")

    # The contract's own carriers.
    def test_missing_contract_regex_carrier_fails_closed(self) -> None:
        self.rewrite(self.contract_path(), "```regex", "```text")
        self.assert_rejected(
            self.check_grammar(), "exactly one normative regex carrier: found 0"
        )

    def test_duplicate_contract_regex_carrier_fails_closed(self) -> None:
        text = self.contract_path().read_text(encoding="utf-8")
        fence = "```regex\n^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$\n```"
        self.assertIn(fence, text)
        self.contract_path().write_text(text + "\n" + fence + "\n", encoding="utf-8")
        self.assert_rejected(
            self.check_grammar(), "exactly one normative regex carrier: found 2"
        )

    def test_contract_prose_drifting_from_its_regex_fails_closed(self) -> None:
        # The anchored normative line, not a whole-document substring search: the regex still
        # says `:` and the document still mentions it elsewhere.
        self.rewrite(
            self.contract_path(),
            "3. interior bytes are ASCII alphanumeric or one of `.`, `_`, `:`, `-`;",
            "3. interior bytes are ASCII alphanumeric or one of `.`, `_`, `?`, `-`;",
        )
        self.assert_rejected(self.check_grammar(), "interior line names")

    def test_contract_length_prose_drift_fails_closed(self) -> None:
        self.rewrite(
            self.contract_path(),
            "1. encoded length is `1..=128` bytes;",
            "1. encoded length is `1..=129` bytes;",
        )
        self.assert_rejected(self.check_grammar(), "max-byte line is")

    def test_contract_normalization_prose_drift_fails_closed(self) -> None:
        self.rewrite(
            self.contract_path(),
            "6. no trimming, Unicode normalization, case folding",
            "6. values are trimmed and case-folded, Unicode normalization, case folding",
        )
        self.assert_rejected(self.check_grammar(), "normalization line is")

    # The oracle's table must be bound to its admitted BODY, not to the file.
    def test_oracle_delimiter_table_drift_fails_closed(self) -> None:
        self.drift_oracle()
        self.assert_rejected(self.check_grammar(), "oracle interior delimiters")

    def test_valid_corpus_not_exercising_every_delimiter_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(), '"Tenant.Alpha_Beta:Gamma-01"', '"Tenant.Alpha_BetaXGamma-01"', 2
        )
        self.rewrite(self.bound_test_path(), '"a..__::--b"', '"a..__XX--b"')
        self.assert_rejected(
            self.check_grammar(), "valid corpus does not exercise every contract delimiter"
        )

    def test_regenerated_fixture_digests_do_not_repair_a_grammar_mismatch(self) -> None:
        # E07: an ordinary "update the SHAs alongside the content" move. Fixture digests govern
        # fixture bytes; they are not, and must not become, the authority for grammar semantics.
        self.drift_production()
        self.drift_oracle()
        self.drift_corpora()
        fixtures = self.root / "crates/platform-core/tests/fixtures/invocation-resolution"
        for path in sorted(fixtures.glob("*.json")):
            text = path.read_text(encoding="utf-8")
            path.write_text(
                text.replace("tenant:", "tenant?").replace("user:", "user?"), encoding="utf-8"
            )
            checker.INVOCATION_FIXTURE_DIGESTS[path.name] = hashlib.sha256(
                path.read_bytes()
            ).hexdigest()
        issues = self.check_grammar()
        self.assert_rejected(issues, "grammar-contract mismatch")
        self.assert_rejected(issues, "production interior delimiters")

    def test_pristine_grammar_authority_passes(self) -> None:
        self.assertEqual(self.check_grammar(), [])


class PlatformIdentityGrammarHarness(unittest.TestCase):
    """Temp-root harness shared by the grammar-authority rows. Carries no rows of its own.

    Every row built on this calls `check_platform_identity_grammar_authority` directly, which is
    the always-run entry point, so a rejection here is the semantic accounting rejecting the
    mutation rather than a frozen body fingerprint noticing that something moved.
    """

    CLASSIFY_HEAD = (
        "fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {\n"
        "    let bytes = value.as_bytes();"
    )
    CLASSIFY_GUARD = (
        "    if bytes.len() > MAX_IDENTITY_BYTES {\n"
        "        return Err(IdentityValueErrorKind::TooLong {\n"
        "            max_bytes: MAX_IDENTITY_BYTES,\n"
        "        });\n"
        "    }"
    )
    MARKER = "effective max-byte bound"

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "crates", self.root / "crates")
        contracts = self.root / "docs/contracts"
        contracts.mkdir(parents=True)
        shutil.copy2(
            REPO_ROOT / checker.PLATFORM_IDENTITY_CONTRACT,
            self.root / checker.PLATFORM_IDENTITY_CONTRACT,
        )
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        self.original_grammar = dict(checker.PLATFORM_IDENTITY_GRAMMAR)
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        checker.PLATFORM_IDENTITY_GRAMMAR.clear()
        checker.PLATFORM_IDENTITY_GRAMMAR.update(self.original_grammar)
        self.temporary_directory.cleanup()

    def check_grammar(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_identity_grammar_authority(issues)
        return issues

    def source_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_SOURCE

    def bound_test_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_TEST

    def rewrite(self, path: Path, old: str, new: str, occurrences: int = 1) -> None:
        text = path.read_text(encoding="utf-8")
        self.assertEqual(
            text.count(old), occurrences, f"stale mutation target in {path.name}: {old!r}"
        )
        path.write_text(text.replace(old, new), encoding="utf-8")

    def assert_rejected(self, issues: list[str], marker: str) -> None:
        self.assertTrue(any(marker in issue for issue in issues), issues)
        self.assertTrue(any(self.MARKER in issue for issue in issues), issues)

    def drift_corpus_constant(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "const MAX_BYTES: usize = 128;",
            "const MAX_BYTES: usize = 129;",
        )


class PlatformIdentityEffectiveBoundTests(PlatformIdentityGrammarHarness):
    """Pins the EFFECTIVE length bound, which a declared carrier and a frozen body cannot.

    Round 15 bound `const MAX_IDENTITY_BYTES: usize = 128;` to the contract and froze `classify`'s
    exact body. Neither closes the class, because the body fingerprint is itself one of the mutable
    carriers: a body that declares a local `const EFFECTIVE_MAX_IDENTITY_BYTES: usize = 129;`,
    compares and reports through it, and keeps the module constant alive as
    `let _ = MAX_IDENTITY_BYTES;` left the contract, both checker tables and every declared `128` in
    place. With the fingerprints and the bound suite's corpus constant co-mutated with it, the whole
    gate chain stayed green while an external caller parsed a 129-byte ID and was told 129.

    These rows therefore call the grammar-authority entry point only, which consults no body
    fingerprint at all: what must reject them is the effective-use accounting, not agreement between
    snapshots that a single commit can move together.
    """

    # M01 — the reported false green: a local const carries the effective bound while the
    # module-level carrier stays at 128 and stays mentioned, so no `-D warnings` lint fires.
    def test_effective_local_const_shadow_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_HEAD,
            "fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {\n"
            "    const EFFECTIVE_MAX_IDENTITY_BYTES: usize = 129;\n"
            "    let _ = MAX_IDENTITY_BYTES;\n"
            "    let bytes = value.as_bytes();",
        )
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_GUARD,
            "    if bytes.len() > EFFECTIVE_MAX_IDENTITY_BYTES {\n"
            "        return Err(IdentityValueErrorKind::TooLong {\n"
            "            max_bytes: EFFECTIVE_MAX_IDENTITY_BYTES,\n"
            "        });\n"
            "    }",
        )
        self.drift_corpus_constant()
        issues = self.check_grammar()
        self.assert_rejected(issues, "declares an item: ['const']")
        self.assert_rejected(issues, "module length comparisons")
        self.assert_rejected(issues, "bound suite MAX_BYTES declares [129]")

    # M02 — the same drift with no item at all, so an item scan alone would not see it.
    def test_effective_let_shadow_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_HEAD,
            "fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {\n"
            "    let effective_max_identity_bytes = MAX_IDENTITY_BYTES + 1;\n"
            "    let bytes = value.as_bytes();",
        )
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_GUARD,
            "    if bytes.len() > effective_max_identity_bytes {\n"
            "        return Err(IdentityValueErrorKind::TooLong {\n"
            "            max_bytes: effective_max_identity_bytes,\n"
            "        });\n"
            "    }",
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, "length comparison")
        self.assert_rejected(issues, "reports max_bytes as ['effective_max_identity_bytes']")

    # M03 — delegation: `classify` still mentions the contract-bound name, and the helper is
    # admitted into the function inventory, so neither a name scan nor the inventory objects.
    def test_helper_delegated_bound_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {",
            "fn effective_bound() -> usize {\n"
            "    MAX_IDENTITY_BYTES + 1\n"
            "}\n\n"
            "fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {",
        )
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_GUARD,
            "    let ceiling = effective_bound();\n"
            "    let _ = MAX_IDENTITY_BYTES;\n"
            "    if bytes.len() > ceiling {\n"
            "        return Err(IdentityValueErrorKind::TooLong {\n"
            "            max_bytes: ceiling,\n"
            "        });\n"
            "    }",
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, "occurs 2 times outside classify")
        self.assert_rejected(issues, "module max_bytes fields")

    # M04 — report-only drift: the comparison keeps the contract-bound name, so a caller is
    # rejected at the right length but told the wrong bound.
    def test_report_only_bound_drift_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_HEAD,
            "fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {\n"
            "    const EFFECTIVE_MAX_IDENTITY_BYTES: usize = 129;\n"
            "    let bytes = value.as_bytes();",
        )
        self.rewrite(
            self.source_path(),
            "            max_bytes: MAX_IDENTITY_BYTES,",
            "            max_bytes: EFFECTIVE_MAX_IDENTITY_BYTES,",
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, "reports max_bytes as ['EFFECTIVE_MAX_IDENTITY_BYTES']")
        self.assert_rejected(issues, "module max_bytes fields")

    # M05 — the bound suite's corpus constant alone. Every length fixture derives from it, so a
    # co-mutated copy makes the runtime agree with a wrong implementation instead of the contract.
    def test_corpus_length_constant_drift_fails_closed(self) -> None:
        self.drift_corpus_constant()
        self.assert_rejected(self.check_grammar(), "bound suite MAX_BYTES declares [129]")

    def test_suite_contract_length_constant_drift_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "const GRAMMAR_MAX_BYTES: usize = 128;",
            "const GRAMMAR_MAX_BYTES: usize = 129;",
        )
        self.assert_rejected(
            self.check_grammar(), "bound suite GRAMMAR_MAX_BYTES declares [129]"
        )

    # A bare literal bound needs no second name at all.
    def test_literal_bound_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "    if bytes.len() > MAX_IDENTITY_BYTES {",
            "    if bytes.len() > 129 {",
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, "integer literals ['129', '1']")
        self.assert_rejected(issues, "length comparison")

    # An operator that is off by one admits one byte more than the contract does.
    def test_off_by_one_comparison_operator_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "    if bytes.len() > MAX_IDENTITY_BYTES {",
            "    if bytes.len() >= MAX_IDENTITY_BYTES {",
        )
        self.assert_rejected(self.check_grammar(), "length comparison")

    # Renaming the carrier does not help: whatever name the checker binds must hold the
    # contract's value, so the renamed declaration is compared against the contract too.
    def test_renamed_bound_constant_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "const MAX_IDENTITY_BYTES: usize = 129;",
        )
        self.assert_rejected(self.check_grammar(), "module-level MAX_IDENTITY_BYTES declares [129]")

    # A declaration that is not plain digits must fail closed rather than read as its first term.
    def test_computed_bound_declaration_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "const MAX_IDENTITY_BYTES: usize = 127 + 1;",
        )
        self.assert_rejected(self.check_grammar(), "module-level MAX_IDENTITY_BYTES declares []")

    # A second binding of the measured subject re-measures a slice of the candidate.
    def test_rebound_measurement_subject_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "    if bytes.len() > MAX_IDENTITY_BYTES {",
            "    let bytes = &bytes[..1];\n    if bytes.len() > MAX_IDENTITY_BYTES {",
        )
        self.assert_rejected(self.check_grammar(), "binds bytes as")

    def test_pristine_effective_bound_passes(self) -> None:
        self.assertEqual(self.check_grammar(), [])


class PlatformIdentityDecidingGuardTests(PlatformIdentityGrammarHarness):
    """Pins that the max-byte comparison DECIDES, and that the runtime proof of it still runs.

    Round 16 proved the comparison `bytes.len() > MAX_IDENTITY_BYTES` occurs inside `classify`. It
    never proved the comparison is what the rejection branch turns on, so a wrapper that keeps every
    declared carrier alive while making the branch unreachable —
    `if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES { … }` — passed this checker,
    all 271 suite tests, fmt, clippy and every cargo gate with both body fingerprints co-mutated,
    while an external crate parsed a 200-byte identity through the public API. `black_box` rather
    than a bare `false` so no lint collapses the condition.

    Two claims failed, and each has its own rows below: an occurring comparison is not a controlling
    condition, and a call site is not a proof body.
    """

    MARKER = "platform identity"
    GUARD_HEAD = "    if bytes.len() > MAX_IDENTITY_BYTES {"
    GUARD_MISMATCH = "must contain exactly one top-level"
    RUNTIME_TAIL = (
        '    let Err(error) = TenantId::parse(refused) else {\n'
        '        panic!("platform identity runtime bound: an over-length value is accepted");\n'
        "    };\n"
        "    assert_eq!(\n"
        "        error.kind(),\n"
        "        IdentityValueErrorKind::TooLong {\n"
        "            max_bytes: GRAMMAR_MAX_BYTES\n"
        "        },\n"
        '        "platform identity runtime bound: reported bound"\n'
        "    );\n"
    )

    def mutate_guard(self, guard: str) -> list[str]:
        self.rewrite(self.source_path(), self.GUARD_HEAD, guard)
        return self.check_grammar()

    # D01-D05 — the comparison tuple, the reported bound, the declared constant, the subject
    # binding and every module-wide count survive verbatim. Only the DECIDING condition moved.
    def test_constant_false_conjunct_fails_closed(self) -> None:
        issues = self.mutate_guard("    if false && bytes.len() > MAX_IDENTITY_BYTES {")
        self.assert_rejected(issues, self.GUARD_MISMATCH)

    def test_opaque_false_conjunct_fails_closed(self) -> None:
        issues = self.mutate_guard(
            "    if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES {"
        )
        self.assert_rejected(issues, self.GUARD_MISMATCH)

    def test_trailing_false_conjunct_fails_closed(self) -> None:
        issues = self.mutate_guard(
            "    if bytes.len() > MAX_IDENTITY_BYTES && std::hint::black_box(false) {"
        )
        self.assert_rejected(issues, self.GUARD_MISMATCH)

    def test_leading_true_disjunct_fails_closed(self) -> None:
        issues = self.mutate_guard(
            "    if std::hint::black_box(true) || bytes.len() > MAX_IDENTITY_BYTES {"
        )
        self.assert_rejected(issues, self.GUARD_MISMATCH)

    def test_wrapped_comparison_fails_closed(self) -> None:
        issues = self.mutate_guard(
            "    if std::hint::black_box(bytes.len() > MAX_IDENTITY_BYTES) {"
        )
        self.assert_rejected(issues, self.GUARD_MISMATCH)

    # A nested copy is at a deeper statement depth and cannot answer for the real guard.
    def test_nested_guard_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_GUARD,
            "    if std::hint::black_box(false) {\n"
            "        if bytes.len() > MAX_IDENTITY_BYTES {\n"
            "            return Err(IdentityValueErrorKind::TooLong {\n"
            "                max_bytes: MAX_IDENTITY_BYTES,\n"
            "            });\n"
            "        }\n"
            "    }",
        )
        self.assert_rejected(self.check_grammar(), self.GUARD_MISMATCH)

    # D06 — the guard is exact, the rejection is no longer its immediate branch.
    def test_non_immediate_rejection_branch_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_GUARD,
            "    if bytes.len() > MAX_IDENTITY_BYTES {\n"
            "        if !is_boundary_byte(first) {\n"
            "            return Err(IdentityValueErrorKind::TooLong {\n"
            "                max_bytes: MAX_IDENTITY_BYTES,\n"
            "            });\n"
            "        }\n"
            "    }",
        )
        self.assert_rejected(self.check_grammar(), self.GUARD_MISMATCH)

    # An alternate branch is a second outcome for the one decision.
    def test_guard_with_else_branch_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.CLASSIFY_GUARD,
            self.CLASSIFY_GUARD + " else {\n        return Ok(());\n    }",
        )
        self.assert_rejected(self.check_grammar(), "alternate branch")

    # A second construction site could report a second bound while the admitted one is disabled.
    def test_second_rejection_site_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "    if !is_boundary_byte(first) {\n"
            "        return Err(IdentityValueErrorKind::InvalidStart);\n"
            "    }",
            "    if !is_boundary_byte(first) {\n"
            "        return Err(IdentityValueErrorKind::TooLong {\n"
            "            max_bytes: MAX_IDENTITY_BYTES,\n"
            "        });\n"
            "    }",
        )
        self.assert_rejected(self.check_grammar(), "is spelled 4 times in the module")

    # D07 — the load-bearing tail goes, the call site stays. This is what Round 16 could not see.
    def test_deleted_runtime_proof_tail_fails_closed(self) -> None:
        self.rewrite(self.bound_test_path(), self.RUNTIME_TAIL, "")
        self.assert_rejected(self.check_grammar(), "body is")

    def test_flipped_runtime_proof_polarity_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    let Err(error) = TenantId::parse(refused) else {",
            "    let Ok(error) = TenantId::parse(refused) else {",
        )
        self.assert_rejected(self.check_grammar(), "body is")

    def test_moved_runtime_proof_candidate_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            '    let refused = "a".repeat(GRAMMAR_MAX_BYTES + 1);',
            '    let refused = "a".repeat(GRAMMAR_MAX_BYTES + 40);',
        )
        self.assert_rejected(self.check_grammar(), "body is")

    def test_conditional_skip_around_runtime_proof_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    assert_contract_bound_is_the_effective_runtime_limit();",
            "    if std::hint::black_box(false) {\n"
            "        assert_contract_bound_is_the_effective_runtime_limit();\n"
            "    }",
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a top-level statement")

    def test_deleted_runtime_proof_call_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    assert_contract_bound_is_the_effective_runtime_limit();",
            "",
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a top-level statement")

    # D08 — every substring the checker pins inside the generic macro survives a selective skip.
    def test_corpus_macro_row_skip_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "        for (value, expected) in invalid_values() {\n"
            "            let Err(error) = <$kind>::parse(value.clone()) else {",
            "        for (value, expected) in invalid_values() {\n"
            "            if matches!(expected, IdentityValueErrorKind::TooLong { .. }) {\n"
            "                continue;\n"
            "            }\n"
            "            let Err(error) = <$kind>::parse(value.clone()) else {",
        )
        self.assert_rejected(self.check_grammar(), "may not ['continue'] past a row")

    # D12 — the whole coordinated bundle, with both body fingerprints and every Python mutation
    # anchor synchronized, so nothing here fails for drift.
    def test_full_dead_guard_comutation_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            self.GUARD_HEAD,
            "    if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES {",
        )
        # The frozen body table is co-mutated exactly as the reproduction did, so no part of this
        # row's rejection can come from fingerprint drift.
        comutated = tuple(
            (
                name,
                body.replace(
                    "if bytes.len() > MAX_IDENTITY_BYTES {",
                    "if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES {",
                ),
            )
            for name, body in checker.PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES
        )
        self.assertNotEqual(comutated, checker.PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES)
        self.rewrite(self.bound_test_path(), self.RUNTIME_TAIL, "")
        original = checker.PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES
        setattr(checker, "PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES", comutated)
        try:
            issues = self.check_grammar()
        finally:
            setattr(checker, "PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES", original)
        self.assert_rejected(issues, self.GUARD_MISMATCH)
        self.assert_rejected(issues, "body is")

    # D13
    def test_pristine_deciding_guard_passes(self) -> None:
        self.assertEqual(self.check_grammar(), [])


class PlatformIdentityReachabilityTests(PlatformIdentityGrammarHarness):
    """Pins that the deciding function holds no unnamed step, and that every proof actually runs.

    Round 17 bound the max-byte guard and the runtime proof's body, and left two ways past both.

    A step the contract does not name could still be added AHEAD of the guard. An early accept keyed
    to a literal — `if value == "aaa…129" { return Ok(()); }` — leaves the guard, the constant,
    every count and every elimination rule intact, and literal payloads are stripped before all of
    them, so both frozen fingerprints could be synchronized to `if value == { return Ok(()); }`.
    Checker, 288 suite tests, fmt, clippy, 71 tests and 16 doctests all stayed green while a
    129-byte value parsed. Reviewer Task 1's variant keys the accept to 200 bytes instead, which the
    128/129 runtime pair does not even look at.

    And `?` is control transfer. Round 17 banned `continue` and `return` where reachability is
    claimed; a helper changed to return `Result`, a caller writing `let _ = helper();` and one
    `black_box(Err::<(),()>(()))?` leave before the proof runs while spelling neither word. The same
    trick inside an ignored closure skips the corpus macro's over-length rows, `break` ends the loop
    just as quietly, and AUTH-011's own evidence calls survive `if black_box(false) { … }` around
    them with every registered carrier substring in place.
    """

    MARKER = "platform identity"
    GUARD_HEAD = "    if bytes.len() > MAX_IDENTITY_BYTES {"
    PROCEDURE_MISMATCH = "admits exactly the steps"
    RUNTIME_CALL = "    assert_contract_bound_is_the_effective_runtime_limit();"
    SWEEP_CALL = "    assert_no_length_past_the_bound_is_accepted();"
    CORPUS_LOOP_HEAD = (
        "        for (value, expected) in invalid_values() {\n"
        "            let Err(error) = <$kind>::parse(value.clone()) else {"
    )

    CORPUS_LOOP_TAIL = (
        '                "{kind_name} bytes Serde must report the checked constructor\'s error"\n'
        "            );\n"
        "        }\n"
        "    }};"
    )

    def close_corpus_closure(self) -> None:
        """Balances the closure opened around the invalid-row loop, so the macro still parses.

        An unbalanced version is rejected as an unreadable body, which proves nothing about the
        rule under test — the mutation has to be the one the reviewer actually built.
        """
        self.rewrite(
            self.bound_test_path(),
            self.CORPUS_LOOP_TAIL,
            self.CORPUS_LOOP_TAIL.replace(
                "        }\n    }};", "        }\n        Ok(())\n        })();\n    }};"
            ),
        )

    def magic_accept(self, literal: str) -> None:
        """A production early accept for one over-bound literal, ahead of the admitted guard."""
        self.rewrite(
            self.source_path(),
            self.GUARD_HEAD,
            f'    if value == "{literal}" {{\n        return Ok(());\n    }}\n' + self.GUARD_HEAD,
        )

    def synchronize_body_fingerprints(self) -> tuple[tuple[str, str], ...]:
        """Co-mutates the frozen body table exactly as the reproduction did, after literal
        stripping, so no row below can be rejected merely for fingerprint drift."""
        return tuple(
            (
                name,
                body.replace(
                    "if bytes.len() > MAX_IDENTITY_BYTES {",
                    "if value == { return Ok(()); } if bytes.len() > MAX_IDENTITY_BYTES {",
                ),
            )
            for name, body in checker.PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES
        )

    def check_with_bodies(self, bodies: tuple[tuple[str, str], ...]) -> list[str]:
        original = checker.PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES
        setattr(checker, "PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES", bodies)
        try:
            return self.check_grammar()
        finally:
            setattr(checker, "PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES", original)

    # A step the contract does not name, at either over-bound length.
    def test_magic_129_early_accept_fails_closed(self) -> None:
        self.magic_accept("a" * 129)
        self.assert_rejected(self.check_grammar(), self.PROCEDURE_MISMATCH)

    def test_magic_200_early_accept_fails_closed(self) -> None:
        self.magic_accept("p" * 200)
        self.assert_rejected(self.check_grammar(), self.PROCEDURE_MISMATCH)

    def test_literal_in_deciding_function_fails_closed(self) -> None:
        self.magic_accept("a" * 129)
        self.assert_rejected(self.check_grammar(), "spells string literals")

    # …and it stays rejected once every snapshot agrees with it.
    def test_magic_accept_with_synchronized_fingerprints_fails_closed(self) -> None:
        self.magic_accept("a" * 129)
        bodies = self.synchronize_body_fingerprints()
        self.assertNotEqual(bodies, checker.PLATFORM_IDENTITY_ADMITTED_FUNCTION_BODIES)
        issues = self.check_with_bodies(bodies)
        self.assert_rejected(issues, self.PROCEDURE_MISMATCH)
        self.assertFalse([issue for issue in issues if "body drifted" in issue], issues)

    # `?` is control transfer, and a helper that returns something can be ignored.
    def test_question_mark_before_runtime_proof_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.RUNTIME_CALL,
            "    std::hint::black_box(Err::<(), ()>(()))?;\n" + self.RUNTIME_CALL,
        )
        self.assert_rejected(self.check_grammar(), "may not ['?'] past the proof")

    def test_result_returning_proof_helper_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "fn assert_contract_bound_is_the_effective_runtime_limit() {",
            "fn assert_contract_bound_is_the_effective_runtime_limit() -> Result<(), ()> {",
        )
        self.assert_rejected(self.check_grammar(), "is declared")

    def test_ignored_proof_result_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    assert_effective_max_byte_bound_is_contract_bound();",
            "    let _ = assert_effective_max_byte_bound_is_contract_bound();",
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a plain statement")

    def test_ignored_sweep_result_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.SWEEP_CALL,
            "    let _ = assert_no_length_past_the_bound_is_accepted();",
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a top-level statement")

    # The corpus macro: a `?` inside an ignored closure, and a `break`, both keep every substring.
    def test_corpus_closure_question_mark_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.CORPUS_LOOP_HEAD,
            "        let _ = (|| -> Result<(), ()> {\n"
            "        for (value, expected) in invalid_values() {\n"
            "            if matches!(expected, IdentityValueErrorKind::TooLong { .. }) {\n"
            "                std::hint::black_box(Err::<(), ()>(()))?;\n"
            "            }\n"
            "            let Err(error) = <$kind>::parse(value.clone()) else {",
        )
        self.close_corpus_closure()
        self.assert_rejected(self.check_grammar(), "may not ['?'] past a row")

    def test_corpus_closure_changes_loop_depth(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.CORPUS_LOOP_HEAD,
            "        let _ = (|| -> Result<(), ()> {\n" + self.CORPUS_LOOP_HEAD,
        )
        self.close_corpus_closure()
        self.assert_rejected(self.check_grammar(), "at the arm's own statement depth")

    def test_corpus_break_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.CORPUS_LOOP_HEAD,
            "        for (value, expected) in invalid_values() {\n"
            "            if matches!(expected, IdentityValueErrorKind::TooLong { .. }) {\n"
            "                break;\n"
            "            }\n"
            "            let Err(error) = <$kind>::parse(value.clone()) else {",
        )
        self.assert_rejected(self.check_grammar(), "may not ['break'] past a row")

    # AUTH-011's own evidence, wrapped in a region that never runs.
    def test_never_entered_auth011_evidence_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    assert_grammar_semantics_match_the_contract();",
            "    if std::hint::black_box(false) {\n"
            "        assert_grammar_semantics_match_the_contract();\n"
            "    }",
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a plain statement")

    # The length sweep, which is what catches an accept keyed to a length the pair never drives.
    def test_weakened_length_sweep_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "            if length <= GRAMMAR_MAX_BYTES {",
            "            if length <= RUNTIME_PROOF_SWEEP {",
        )
        self.assert_rejected(self.check_grammar(), "body is")

    def test_deleted_length_sweep_call_fails_closed(self) -> None:
        self.rewrite(self.bound_test_path(), self.SWEEP_CALL, "")
        self.assert_rejected(self.check_grammar(), "exactly once as a top-level statement")

    def test_pristine_reachability_passes(self) -> None:
        self.assertEqual(self.check_grammar(), [])


class PlatformIdentityNameResolutionTests(PlatformIdentityGrammarHarness):
    """Pins that a load-bearing call reaches the helper it names, and that the sweep sweeps.

    Round 18 proved each proof is CALLED as a plain statement of its caller. That is a fact about
    tokens; which function runs is a fact about name resolution, and Rust resolves lexically. An
    item declared in the caller's own body binds the name ahead of the module's:

        let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();
        fn r#assert_no_length_past_the_bound_is_accepted() {}
        assert_no_length_past_the_bound_is_accepted();

    The decoy keeps the real helper used so no unused-item lint fires, the raw identifier is the
    same name to Rust and a different string to every textual rule, and the call — untouched — runs
    the no-op. Checker, 303 suite tests, fmt, clippy and every cargo gate stayed green while neither
    the runtime proof nor the length sweep executed.

    Round 18 also froze the sweep's token sequence, which fixes the loops and leaves what they range
    over free. `const RUNTIME_PROOF_SEEDS: [&str; 0] = [];` swept nothing; `RUNTIME_PROOF_SWEEP =
    GRAMMAR_MAX_BYTES` swept nothing past the bound, which is the half that matters. Both were green
    everywhere, and the second one falsified Round 18's own matrix row for a weakened sweep.
    """

    MARKER = "platform identity"
    RUNTIME_CALL = "    assert_contract_bound_is_the_effective_runtime_limit();"
    SWEEP_CALL = "    assert_no_length_past_the_bound_is_accepted();"
    SEEDS = 'const RUNTIME_PROOF_SEEDS: [&str; 2] = ["a", "p"];'
    SPAN = "const RUNTIME_PROOF_SWEEP: usize = 2 * GRAMMAR_MAX_BYTES;"
    SHADOW_SHAPE = "a local item binds its name ahead"
    DECLARED_TWICE = "is declared 2 times in"
    SPELLED_TWICE = "expected at most the one call"
    CARRIER_SHAPE = "may not come from an alias"

    def shadow(self, body: str) -> None:
        """Replaces the caller's two proof calls with `body`, which is the reviewer's mutation."""
        self.rewrite(
            self.bound_test_path(), f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}", body.rstrip("\n")
        )

    # The reviewer's exact mutation, and the plain spelling it is meant to evade.
    def test_raw_identifier_shadow_fails_closed(self) -> None:
        self.shadow(
            "    let _ = crate::assert_contract_bound_is_the_effective_runtime_limit as fn();\n"
            "    let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();\n"
            "    fn r#assert_contract_bound_is_the_effective_runtime_limit() {}\n"
            "    fn r#assert_no_length_past_the_bound_is_accepted() {}\n"
            f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}"
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, self.DECLARED_TWICE)
        self.assert_rejected(issues, self.SHADOW_SHAPE)
        self.assert_rejected(issues, self.SPELLED_TWICE)

    def test_plain_identifier_shadow_fails_closed(self) -> None:
        self.shadow(
            "    let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();\n"
            "    fn assert_no_length_past_the_bound_is_accepted_() {}\n"
            f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}"
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, self.SHADOW_SHAPE)
        self.assert_rejected(issues, self.SPELLED_TWICE)

    # …and the binding forms that are not `fn` at all.
    def test_let_binding_shadow_fails_closed(self) -> None:
        self.shadow(
            "    let assert_no_length_past_the_bound_is_accepted = || {};\n"
            f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}"
        )
        self.assert_rejected(self.check_grammar(), self.SPELLED_TWICE)

    def test_use_alias_shadow_fails_closed(self) -> None:
        self.shadow(
            "    use crate::assert_corpus_macro_cannot_skip_a_row as "
            "assert_no_length_past_the_bound_is_accepted;\n"
            f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}"
        )
        self.assert_rejected(self.check_grammar(), self.SHADOW_SHAPE)

    def test_const_item_shadow_fails_closed(self) -> None:
        self.shadow(
            "    const assert_no_length_past_the_bound_is_accepted: fn() = || {};\n"
            f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}"
        )
        self.assert_rejected(self.check_grammar(), self.SHADOW_SHAPE)

    def test_module_shadow_fails_closed(self) -> None:
        self.shadow(
            "    mod shadow {\n"
            "        pub fn assert_no_length_past_the_bound_is_accepted() {}\n"
            "    }\n"
            "    use shadow::assert_no_length_past_the_bound_is_accepted;\n"
            f"{self.RUNTIME_CALL}\n{self.SWEEP_CALL}"
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, self.DECLARED_TWICE)
        self.assert_rejected(issues, self.SHADOW_SHAPE)

    # The same class planted in AUTH-011's own body rather than in the bound caller's.
    def test_auth011_shadow_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    assert_grammar_is_exhaustive_over_bytes();\n}",
            "    let _ = crate::assert_grammar_is_exhaustive_over_bytes as fn();\n"
            "    fn r#assert_grammar_is_exhaustive_over_bytes() {}\n"
            "    assert_grammar_is_exhaustive_over_bytes();\n}",
        )
        issues = self.check_grammar()
        self.assert_rejected(issues, self.DECLARED_TWICE)
        self.assert_rejected(issues, self.SHADOW_SHAPE)

    # The two new proofs are themselves reachable evidence, so deleting a call is a rejection.
    def test_deleted_resolution_proof_call_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(), "    assert_load_bearing_calls_reach_their_helper();\n", ""
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a top-level statement")

    def test_deleted_sweep_carrier_proof_call_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(), "    assert_sweep_carriers_are_the_contract_extent();\n", ""
        )
        self.assert_rejected(self.check_grammar(), "exactly once as a top-level statement")

    # The sweep's seeds: emptied, shortened, repeated, and split across a `cfg`.
    def test_empty_sweep_seeds_fail_closed(self) -> None:
        self.rewrite(self.bound_test_path(), self.SEEDS, "const RUNTIME_PROOF_SEEDS: [&str; 0] = [];")
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    def test_single_sweep_seed_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(), self.SEEDS, 'const RUNTIME_PROOF_SEEDS: [&str; 1] = ["a"];'
        )
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    def test_duplicate_sweep_seeds_fail_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(), self.SEEDS, 'const RUNTIME_PROOF_SEEDS: [&str; 2] = ["a", "a"];'
        )
        self.assert_rejected(self.check_grammar(), "two distinct single-byte")

    def test_cfg_twinned_sweep_seeds_fail_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.SEEDS,
            f"#[cfg(not(test))]\n{self.SEEDS}\n"
            "#[cfg(test)]\nconst RUNTIME_PROOF_SEEDS: [&str; 0] = [];",
        )
        self.assert_rejected(self.check_grammar(), "a second declaration is a `cfg` twin")

    # …and its span: shortened, zeroed, aliased, computed by a helper, produced by a macro.
    def test_shortened_sweep_span_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(), self.SPAN, "const RUNTIME_PROOF_SWEEP: usize = GRAMMAR_MAX_BYTES;"
        )
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    def test_zero_sweep_span_fails_closed(self) -> None:
        self.rewrite(self.bound_test_path(), self.SPAN, "const RUNTIME_PROOF_SWEEP: usize = 0;")
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    def test_aliased_sweep_span_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.SPAN,
            "const SWEEP_ALIAS: usize = GRAMMAR_MAX_BYTES;\n"
            "const RUNTIME_PROOF_SWEEP: usize = SWEEP_ALIAS;",
        )
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    def test_helper_computed_sweep_span_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.SPAN,
            "const fn sweep_span() -> usize {\n    GRAMMAR_MAX_BYTES\n}\n"
            "const RUNTIME_PROOF_SWEEP: usize = sweep_span();",
        )
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    def test_macro_generated_sweep_span_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            self.SPAN,
            "macro_rules! sweep_span {\n    () => {\n        GRAMMAR_MAX_BYTES\n    };\n}\n"
            "const RUNTIME_PROOF_SWEEP: usize = sweep_span!();",
        )
        self.assert_rejected(self.check_grammar(), self.CARRIER_SHAPE)

    # The contract's own number, which the span and both coverage counts are stated in terms of.
    def test_drifted_sweep_bound_constant_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "const GRAMMAR_MAX_BYTES: usize = 128;",
            "const GRAMMAR_MAX_BYTES: usize = 129;",
        )
        self.assert_rejected(self.check_grammar(), "the span and both coverage")

    # And the counts themselves, which are what makes an emptied carrier answerable at runtime.
    def test_uncounted_sweep_fails_closed(self) -> None:
        self.rewrite(
            self.bound_test_path(),
            "    assert_eq!(\n        admitted,\n        2 * GRAMMAR_MAX_BYTES,",
            "    assert_eq!(\n        admitted,\n        GRAMMAR_MAX_BYTES,",
        )
        self.assert_rejected(self.check_grammar(), "body is")

    def test_pristine_name_resolution_passes(self) -> None:
        self.assertEqual(self.check_grammar(), [])


class PlatformIdentityImplementationContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "crates", self.root / "crates")
        acceptance = self.root / "docs/acceptance"
        acceptance.mkdir(parents=True)
        shutil.copy2(REPO_ROOT / "docs/acceptance/matrix.tsv", acceptance / "matrix.tsv")
        scripts = self.root / "scripts"
        scripts.mkdir()
        shutil.copy2(CHECKER_PATH, scripts / "check_repo_contracts.py")
        corpus = self.root / checker.RUST_LEXICAL_CORPUS
        corpus.parent.mkdir(parents=True)
        shutil.copy2(REPO_ROOT / checker.RUST_LEXICAL_CORPUS, corpus)
        # The workspace manifest and lockfile carry the dependency-source identity rule, which
        # is part of the same identity check.
        for name in (checker.WORKSPACE_MANIFEST, checker.WORKSPACE_LOCKFILE):
            shutil.copy2(REPO_ROOT / name, self.root / name)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_identity(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_identity_implementation(issues)
        return issues

    def source_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_SOURCE

    def invocation_path(self) -> Path:
        return self.root / checker.PLATFORM_INVOCATION_SOURCE

    def market_path(self) -> Path:
        return self.root / "crates/platform-core/src/market.rs"

    def installation_path(self) -> Path:
        return self.root / checker.PLATFORM_INSTALLATION_SOURCE

    def installation_test_path(self) -> Path:
        return self.root / checker.PLATFORM_INSTALLATION_TEST

    def grant_path(self) -> Path:
        return self.root / checker.PLATFORM_GRANT_SOURCE

    def grant_test_path(self) -> Path:
        return self.root / checker.PLATFORM_GRANT_TEST

    def update_path(self) -> Path:
        return self.root / checker.PLATFORM_UPDATE_SOURCE

    def update_test_path(self) -> Path:
        return self.root / checker.PLATFORM_UPDATE_TEST

    def admit_installation_surface(self) -> None:
        path = self.installation_path()
        if not path.is_file():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(ADMITTED_INSTALLATION_SOURCE, encoding="utf-8")
        if "pub mod installation;" not in self.market_path().read_text(encoding="utf-8"):
            self.rewrite(
                self.market_path(),
                "use crate::invocation::{",
                "pub mod installation;\n\nuse crate::invocation::{",
            )

    def bound_test_path(self) -> Path:
        return self.root / checker.PLATFORM_IDENTITY_TEST

    def capability_test_path(self) -> Path:
        return self.root / checker.PLATFORM_CAPABILITY_TEST

    def rewrite(
        self,
        path: Path,
        old: str,
        new: str,
        occurrences: int = 1,
        replacements: int | None = None,
    ) -> None:
        # The exact number of existing occurrences is asserted, not merely membership: if the
        # source drifts so the target no longer appears the expected number of times, the
        # mutation would silently become a no-op and the test would "pass" proving nothing.
        # `replacements` is separate so a mutation can perturb one of several occurrences.
        text = path.read_text(encoding="utf-8")
        self.assertEqual(
            text.count(old),
            occurrences,
            f"stale mutation target in {path.name}: {old!r}",
        )
        updated = text.replace(old, new, occurrences if replacements is None else replacements)
        self.assertNotEqual(updated, text, f"mutation did not change bytes in {path.name}: {old!r}")
        path.write_text(
            updated,
            encoding="utf-8",
        )

    def assert_rejected(self, issues: list[str], marker: str) -> None:
        self.assertTrue(any(marker in issue for issue in issues), issues)

    def test_current_platform_identity_implementation_passes(self) -> None:
        self.assertEqual(self.check_identity(), [])

    def test_missing_market_source_fails_closed(self) -> None:
        self.market_path().unlink()
        self.assert_rejected(self.check_identity(), "platform identity carrier missing")

    def test_admitted_market_installation_surface_passes(self) -> None:
        self.admit_installation_surface()
        self.assertEqual(self.check_identity(), [])

    def test_admitted_current_market_grant_surface_passes(self) -> None:
        self.assertEqual(self.check_identity(), [])

    def test_missing_market_update_nested_file_fails_closed(self) -> None:
        self.assertEqual(self.check_identity(), [])
        self.update_path().unlink()
        issues = self.check_identity()
        self.assert_rejected(issues, "market update carrier missing")
        self.assert_rejected(issues, "platform-core source file set drifted")

    def test_missing_market_update_module_declaration_fails_closed(self) -> None:
        self.rewrite(self.market_path(), "pub mod update;\n", "")
        self.assert_rejected(
            self.check_identity(), "market update module declaration missing"
        )

    def test_unregistered_market_update_source_or_test_fails_closed(self) -> None:
        cases = (
            (
                self.root / "crates/platform-core/src/market/update_shadow.rs",
                "pub struct ExtraUpdateShadow;\n",
                ("platform-core source file set drifted", "ungoverned platform-core source"),
            ),
            (
                self.root / "crates/platform-core/tests/market_package_update_shadow.rs",
                "#[test]\nfn shadow_update_test() {}\n",
                ("platform-core package inventory drifted",),
            ),
        )
        for path, content, markers in cases:
            with self.subTest(path=path.name):
                path.write_text(content, encoding="utf-8")
                issues = self.check_identity()
                for marker in markers:
                    self.assert_rejected(issues, marker)
                path.unlink()

    def test_market_update_public_rename_extra_item_and_visibility_widening_fail_closed(self) -> None:
        original = self.update_path().read_text(encoding="utf-8")
        cases = (
            ("pub struct UpdateApprovalId", "pub struct UpdateApprovalKey"),
            ("pub trait PackageUpdateRepository {", "pub struct ExtraUpdateSurface;\n\npub trait PackageUpdateRepository {"),
            ("struct PlanPackageAuthority {", "pub struct PlanPackageAuthority {"),
        )
        for old, new in cases:
            with self.subTest(old=old):
                self.update_path().write_text(original, encoding="utf-8")
                self.rewrite(self.update_path(), old, new)
                self.assert_rejected(
                    self.check_identity(), "market update public declaration surface drifted"
                )
        self.update_path().write_text(original, encoding="utf-8")
        self.rewrite(
            self.update_path(),
            "pub(in crate::market) fn for_cancel()",
            "pub(in crate::market) fn for_abort()",
        )
        self.assert_rejected(
            self.check_identity(), "market update restricted public function surface drifted"
        )
        self.update_path().write_text(original, encoding="utf-8")
        self.rewrite(
            self.update_path(),
            "#[cfg(test)]\n    fn from_parts(",
            "    fn from_parts(",
        )
        self.rewrite(
            self.update_path(),
            "fn validate_publications(",
            "#[cfg(test)]\nfn validate_publications(",
        )
        self.assert_rejected(
            self.check_identity(), "market update test-only authority carrier drifted"
        )
        self.update_path().write_text(original, encoding="utf-8")
        self.rewrite(
            self.update_path(),
            "UpdateCommandAction::Cancel { .. } => Ok(UpdateDecisionContext::for_cancel()),",
            "UpdateCommandAction::Cancel { .. } => Ok(UpdateDecisionContext { kind: UpdateDecisionContextKind::Cancel }),",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "market update production context constructor reachability drifted"
        )
        self.assert_rejected(
            issues, "market update production direct context construction drifted"
        )
        self.update_path().write_text(original, encoding="utf-8")

    def test_market_update_dependency_import_impl_derive_attribute_and_macro_drift_fail_closed(self) -> None:
        original = self.update_path().read_text(encoding="utf-8")
        cases = (
            ("use std::fmt;", "use std::fmt;\nuse std::time::SystemTime;", "market update item declarations drifted"),
            ("impl UpdateState {", "impl ExtraUpdateImpl {", "market update implementation surface drifted"),
            ("#[derive(Clone, PartialEq, Eq)]\npub struct PackageUpdatePlan", "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct PackageUpdatePlan", "market update derive surface drifted"),
            ("pub enum UpdateConstructionError {", "#[allow(unused_imports)]\npub enum UpdateConstructionError {", "market update attribute inventory drifted"),
            ("macro_rules! parsed {", "macro_rules! parsed_changed {", "market update macro definitions drifted"),
            ("assert_ne!(approval.evidence_digest(), readiness.evidence_digest());", "assert_ne!(approval.evidence_digest(), readiness.evidence_digest());\n        assert!(true);", "market update macro invocation counts drifted"),
        )
        for old, new, marker in cases:
            with self.subTest(marker=marker):
                self.update_path().write_text(original, encoding="utf-8")
                self.rewrite(self.update_path(), old, new)
                self.assert_rejected(self.check_identity(), marker)
        self.update_path().write_text(original, encoding="utf-8")

    def test_market_update_authority_debug_derive_fails_closed(self) -> None:
        self.rewrite(
            self.update_path(),
            "#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub struct PackageUpdateId",
            "#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub struct PackageUpdateId",
        )
        self.assert_rejected(self.check_identity(), "market update derive surface drifted")

    def test_market_update_missing_or_ignored_unit_test_fails_closed(self) -> None:
        original = self.update_path().read_text(encoding="utf-8")
        target = "#[test]\n    fn checked_update_ids_revisions_and_sequences_are_canonical()"
        self.rewrite(self.update_path(), target, "fn checked_update_ids_revisions_and_sequences_are_canonical()")
        self.assert_rejected(self.check_identity(), "market update unit test inventory drifted")
        self.update_path().write_text(original, encoding="utf-8")
        self.rewrite(self.update_path(), target, "#[ignore]\n    #[test]\n    fn checked_update_ids_revisions_and_sequences_are_canonical()")
        issues = self.check_identity()
        self.assert_rejected(issues, "market update attribute inventory drifted")

    def test_market_update_missing_or_ignored_integration_test_fails_closed(self) -> None:
        original = self.update_test_path().read_text(encoding="utf-8")
        target = "#[test]\nfn checked_public_update_values_and_stage_surface_are_deterministic()"
        self.rewrite(self.update_test_path(), target, "fn checked_public_update_values_and_stage_surface_are_deterministic()")
        self.assert_rejected(self.check_identity(), "market update acceptance test registration drift")
        self.update_test_path().write_text(original, encoding="utf-8")
        self.rewrite(self.update_test_path(), target, "#[ignore]\n#[test]\nfn checked_public_update_values_and_stage_surface_are_deterministic()")
        issues = self.check_identity()
        self.assert_rejected(issues, "market update acceptance test")
        self.assert_rejected(issues, "attribute envelope drifted")

    def test_market_installation_update_event_or_restricted_constructor_drift_fails_closed(self) -> None:
        original = self.installation_path().read_text(encoding="utf-8")
        cases = (
            ("pub(in crate::market) fn package_updated(", "pub fn package_updated(", "market installation has an unclassified public declaration"),
            ("    PackageUpdated,\n    PackageRolledBack,", "    PackageUpdatedRenamed,\n    PackageRolledBack,", "market installation event kind variants drifted"),
            ("pub(in crate::market) fn package_rolled_back(", "pub(in crate::market) fn package_rollback_event(", "market installation restricted public function surface drifted"),
            ("pub(in crate::market) fn matches_package_pin_change(", "pub(in crate::market) fn matches_any_pin_change(", "market installation restricted public function surface drifted"),
        )
        for old, new, marker in cases:
            with self.subTest(marker=marker):
                self.installation_path().write_text(original, encoding="utf-8")
                self.rewrite(self.installation_path(), old, new)
                self.assert_rejected(self.check_identity(), marker)
        self.installation_path().write_text(original, encoding="utf-8")

    def test_market_grant_current_set_or_arbitrary_query_drift_fails_closed(self) -> None:
        original = self.grant_path().read_text(encoding="utf-8")
        cases = (
            ("pub struct CurrentInstallationGrantSet {", "pub struct CurrentInstallationGrants {", "market grant public declaration surface drifted"),
            ("pub trait GrantRepository {", "pub fn arbitrary_grant_query() {}\n\npub trait GrantRepository {", "market grant public declaration surface drifted"),
            ("pub(in crate::market) fn canonical_coupling_digest(", "pub(in crate::market) fn coupling_hash(", "market grant restricted public function surface drifted"),
            ("pub(in crate::market) fn is_canonical(", "pub(in crate::market) fn trusts_canonical(", "market grant restricted public function surface drifted"),
        )
        for old, new, marker in cases:
            with self.subTest(marker=marker):
                self.grant_path().write_text(original, encoding="utf-8")
                self.rewrite(self.grant_path(), old, new)
                self.assert_rejected(self.check_identity(), marker)
        self.grant_path().write_text(original, encoding="utf-8")

    def test_market_004_status_promotion_after_b6_fails_closed(self) -> None:
        matrix = self.root / "docs/acceptance/matrix.tsv"
        text = matrix.read_text(encoding="utf-8")
        row = next(line for line in text.splitlines() if line.startswith("MARKET-004\t"))
        self.assertEqual(row.count("\tplanned\t"), 1)
        matrix.write_text(text.replace(row, row.replace("\tplanned\t", "\timplemented\t", 1), 1), encoding="utf-8")
        issues: list[str] = []
        checker.check_platform_authority_implementation(issues)
        self.assert_rejected(issues, "MARKET-004 must remain a planned acceptance row after bounded B6")

    def test_pkg_020_status_promotion_after_b6_fails_closed(self) -> None:
        matrix = self.root / "docs/acceptance/matrix.tsv"
        text = matrix.read_text(encoding="utf-8")
        row = next(line for line in text.splitlines() if line.startswith("PKG-020\t"))
        self.assertEqual(row.count("\tplanned\t"), 1)
        matrix.write_text(text.replace(row, row.replace("\tplanned\t", "\timplemented\t", 1), 1), encoding="utf-8")
        issues: list[str] = []
        checker.check_platform_authority_implementation(issues)
        self.assert_rejected(issues, "PKG-020 must remain a planned acceptance row after bounded B6")

    # Grant mutation map: carrier missing -> carrier diagnostic.
    def test_missing_market_grant_nested_file_fails_closed(self) -> None:
        self.grant_path().unlink()
        self.assert_rejected(self.check_identity(), "market grant carrier missing")

    # Module declaration missing -> declaration diagnostic.
    def test_missing_market_grant_module_declaration_fails_closed(self) -> None:
        self.rewrite(self.market_path(), "pub mod grant;\n", "")
        self.assert_rejected(self.check_identity(), "market grant module declaration missing")

    # Ignored external tests -> exact attribute-envelope diagnostic.
    def test_market_grant_ignored_tests_fail_closed(self) -> None:
        text = self.grant_test_path().read_text(encoding="utf-8")
        test_count = text.count("#[test]\nfn ")
        self.assertEqual(test_count, len(checker.PLATFORM_GRANT_TEST_FUNCTIONS))
        self.grant_test_path().write_text(
            text.replace("#[test]\nfn ", "#[ignore]\n#[test]\nfn "), encoding="utf-8"
        )
        self.assert_rejected(self.check_identity(), "market grant acceptance test")
        self.assert_rejected(self.check_identity(), "attribute envelope drifted")

    # Missing registration -> exact executable-test count diagnostic.
    def test_market_grant_missing_bound_test_fails_closed(self) -> None:
        self.rewrite(
            self.grant_test_path(),
            "#[test]\nfn checked_grant_ids_versions_and_sequences_are_canonical()",
            "fn checked_grant_ids_versions_and_sequences_are_canonical()",
        )
        self.assert_rejected(
            self.check_identity(), "market grant acceptance test registration drift"
        )

    # Public rename -> bidirectional public declaration inventory.
    def test_market_grant_public_surface_rename_fails_closed(self) -> None:
        self.rewrite(self.grant_path(), "pub struct GrantApprovalId", "pub struct GrantApprovalKey")
        self.assert_rejected(self.check_identity(), "market grant public declaration surface drifted")

    # Extra public item -> bidirectional public declaration inventory.
    def test_market_grant_extra_public_surface_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            "pub trait GrantRepository {",
            "pub struct ExtraGrantSurface;\n\npub trait GrantRepository {",
        )
        self.assert_rejected(self.check_identity(), "market grant public declaration surface drifted")

    # Private visibility widening -> bidirectional public declaration inventory.
    def test_market_grant_visibility_widening_fails_closed(self) -> None:
        self.rewrite(self.grant_path(), "struct AuthorityKey {", "pub struct AuthorityKey {")
        self.assert_rejected(self.check_identity(), "market grant public declaration surface drifted")

    # Dependency import addition -> exact source-order item inventory.
    def test_market_grant_dependency_import_drift_fails_closed(self) -> None:
        self.rewrite(self.grant_path(), "use std::fmt;", "use std::fmt;\nuse std::time::SystemTime;")
        self.assert_rejected(self.check_identity(), "market grant item declarations drifted")

    # Checked-in allowlist omission -> bidirectional public declaration inventory.
    def test_market_grant_omitted_allowlist_entry_fails_closed(self) -> None:
        old = checker.PLATFORM_GRANT_ADMITTED_PUBLIC_DECLARATIONS
        self.addCleanup(setattr, checker, "PLATFORM_GRANT_ADMITTED_PUBLIC_DECLARATIONS", old)
        removed = list(old)
        removed.remove("pub struct GrantApprovalId")
        setattr(checker, "PLATFORM_GRANT_ADMITTED_PUBLIC_DECLARATIONS", tuple(removed))
        self.assert_rejected(self.check_identity(), "market grant public declaration surface drifted")

    # Identity alias spelling -> exact item and cross-file binding inventories.
    def test_market_grant_identity_import_alias_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            "use crate::identity::{TenantId, UserId};",
            "use crate::identity::{TenantId as Tenant, UserId};",
        )
        self.assert_rejected(self.check_identity(), "market grant item declarations drifted")
        self.assert_rejected(
            self.check_identity(), "platform identity value alias or import outside the M00 identity module"
        )

    # Attribute multiplicity is exact: an extra lint suppression must not hide in an admitted name.
    def test_market_grant_extra_allow_attribute_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            "pub enum GrantConstructionError {",
            "#[allow(unused_imports)]\npub enum GrantConstructionError {",
        )
        self.assert_rejected(
            self.check_identity(), "market grant attribute inventory drifted"
        )

    # Attribute bodies are exact: replacing one admitted lint suppression is also drift.
    def test_market_grant_allow_attribute_body_drift_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            "    #[allow(dead_code)]\n    #[allow(clippy::too_many_arguments)]\n",
            "    #[allow(unused_imports)]\n    #[allow(clippy::too_many_arguments)]\n",
        )
        self.assert_rejected(
            self.check_identity(), "market grant attribute inventory drifted"
        )

    # Adding Debug to a manually redacted authority type -> exact derive inventory.
    def test_market_grant_authority_derive_drift_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            "#[derive(Clone, PartialEq, Eq)]\npub struct GrantAdmissionEvidence",
            "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct GrantAdmissionEvidence",
        )
        self.assert_rejected(self.check_identity(), "market grant derive surface drifted")

    # Identity-bearing macro multiplicity is an exact multiset, not membership.
    def test_market_grant_identity_macro_multiplicity_drift_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            '            let tenant = parsed!(TenantId, "tenant:grant-tests");\n',
            '            let tenant = parsed!(TenantId, "tenant:grant-tests");\n'
            '            let extra = parsed!(TenantId, "tenant:extra");\n',
        )
        self.assert_rejected(
            self.check_identity(), "market grant identity macro arguments drifted"
        )

    # Non-identity `parsed!` calls are also an exact multiset, not an identity-only exception.
    def test_market_grant_non_identity_parsed_multiplicity_drift_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            '            let capability = parsed!(CapabilityId, "campus.public_plan.read");\n',
            '            let capability = parsed!(CapabilityId, "campus.public_plan.read");\n'
            '            let extra = parsed!(GrantVersion, "grant-version:999");\n',
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "market grant macro invocation counts drifted")
        self.assert_rejected(issues, "market grant parsed macro arguments drifted")

    # A same-count parsed type substitution must fail even though the macro-name count is stable.
    def test_market_grant_parsed_argument_substitution_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            '            let capability = parsed!(CapabilityId, "campus.public_plan.read");\n',
            '            let capability = parsed!(GrantVersion, "grant-version:999");\n',
        )
        self.assert_rejected(
            self.check_identity(), "market grant parsed macro arguments drifted"
        )

    # Counts are exact for every admitted macro, not only for `parsed!`.
    def test_market_grant_non_parsed_macro_count_drift_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            '            let package = load_package_manifest(PACKAGE).expect("reviewed package");\n',
            '            let package = load_package_manifest(PACKAGE).expect("reviewed package");\n'
            "            assert!(true);\n",
        )
        self.assert_rejected(
            self.check_identity(), "market grant macro invocation counts drifted"
        )

    # Invocation-name drift -> exact macro invocation inventory.
    def test_market_grant_macro_invocation_drift_fails_closed(self) -> None:
        self.rewrite(
            self.grant_path(),
            "category_error!(GrantConstructionError,",
            "category_error_changed!(GrantConstructionError,",
        )
        self.assert_rejected(self.check_identity(), "market grant macro invocations drifted")

    # Definition-name drift -> exact macro definition inventory.
    def test_market_grant_macro_definition_drift_fails_closed(self) -> None:
        self.rewrite(self.grant_path(), "macro_rules! parsed {", "macro_rules! parsed_changed {")
        self.assert_rejected(self.check_identity(), "market grant macro definitions drifted")

    def test_missing_market_installation_nested_file_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.installation_path().unlink()
        issues = self.check_identity()
        self.assert_rejected(issues, "market installation carrier missing")
        self.assert_rejected(issues, "platform-core source file set drifted")

    def test_market_installation_ignored_tests_fail_closed(self) -> None:
        self.admit_installation_surface()
        text = self.installation_test_path().read_text(encoding="utf-8")
        test_count = text.count("#[test]\nfn ")
        self.assertEqual(test_count, len(checker.PLATFORM_INSTALLATION_TEST_FUNCTIONS))
        self.installation_test_path().write_text(
            text.replace("#[test]\nfn ", "#[ignore]\n#[test]\nfn "),
            encoding="utf-8",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "market installation acceptance test")
        self.assert_rejected(issues, "attribute envelope drifted")

    def test_market_installation_missing_bound_test_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.rewrite(
            self.installation_test_path(),
            "#[test]\nfn configuration_values_are_canonical_bounded_and_secret_safe()",
            "fn configuration_values_are_canonical_bounded_and_secret_safe()",
        )
        self.assert_rejected(
            self.check_identity(),
            "market installation acceptance test registration drift",
        )

    def test_missing_market_installation_module_declaration_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.rewrite(self.market_path(), "pub mod installation;\n", "")
        self.assert_rejected(
            self.check_identity(),
            "market installation module declaration missing",
        )

    def test_market_installation_public_surface_rename_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.rewrite(self.installation_path(), "pub struct SecretRefId", "pub struct SecretReferenceId")
        self.assert_rejected(
            self.check_identity(),
            "market installation public declaration surface drifted",
        )

    def test_market_installation_extra_public_surface_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.rewrite(
            self.installation_path(),
            "pub trait InstallationRepository {",
            "pub struct ExtraPublicSurface;\n\npub trait InstallationRepository {",
        )
        self.assert_rejected(
            self.check_identity(),
            "market installation public declaration surface drifted",
        )

    def test_market_installation_visibility_widening_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.rewrite(self.installation_path(), "struct CommandLedgerEntry {", "pub struct CommandLedgerEntry {")
        self.assert_rejected(
            self.check_identity(),
            "market installation public declaration surface drifted",
        )

    def test_market_installation_dependency_import_drift_fails_closed(self) -> None:
        self.admit_installation_surface()
        self.rewrite(
            self.installation_path(),
            "use std::fmt;",
            "use std::fmt;\nuse std::time::SystemTime;",
        )
        self.assert_rejected(self.check_identity(), "market installation item declarations drifted")

    def test_market_installation_omitted_allowlist_entry_fails_closed(self) -> None:
        self.admit_installation_surface()
        old = checker.PLATFORM_INSTALLATION_ADMITTED_PUBLIC_DECLARATIONS
        self.addCleanup(setattr, checker, "PLATFORM_INSTALLATION_ADMITTED_PUBLIC_DECLARATIONS", old)
        setattr(
            checker,
            "PLATFORM_INSTALLATION_ADMITTED_PUBLIC_DECLARATIONS",
            tuple(item for item in old if item != "pub struct SecretRefId"),
        )
        self.assert_rejected(
            self.check_identity(),
            "market installation public declaration surface drifted",
        )

    def test_missing_market_catalog_test_fails_closed(self) -> None:
        (self.root / "crates/platform-core/tests/market_package_catalog.rs").unlink()
        self.assert_rejected(self.check_identity(), "platform-core source file set drifted")

    def test_market_capability_registry_ignored_tests_fail_closed(self) -> None:
        text = self.capability_test_path().read_text(encoding="utf-8")
        test_count = text.count("#[test]\nfn ")
        self.assertEqual(test_count, len(checker.PLATFORM_CAPABILITY_TEST_FUNCTIONS))
        self.capability_test_path().write_text(
            text.replace("#[test]\nfn ", "#[ignore]\n#[test]\nfn "),
            encoding="utf-8",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "market capability registry acceptance test")
        self.assert_rejected(issues, "attribute envelope drifted")

    def test_market_capability_registry_missing_bound_test_fails_closed(self) -> None:
        self.rewrite(
            self.capability_test_path(),
            "#[test]\nfn current_registry_loads_with_exact_eight_definitions()",
            "fn current_registry_loads_with_exact_eight_definitions()",
        )
        self.assert_rejected(
            self.check_identity(),
            "market capability registry acceptance test registration drift",
        )

    def test_missing_market_module_export_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod market;",
            "// pub mod market;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform-core module declarations drifted in crates/platform-core/src/lib.rs",
        )

    def test_market_source_item_drift_fails_closed(self) -> None:
        self.rewrite(
            self.market_path(),
            "use std::fmt;",
            "use std::fmt;\nuse std::time::SystemTime;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform-core item declarations drifted in crates/platform-core/src/market.rs",
        )

    def test_serde_json_dependency_role_drift_fails_closed(self) -> None:
        self.rewrite(self.manifest_path(), "serde_json.workspace = true\n", "")
        self.rewrite(
            self.manifest_path(),
            'hex = "0.4.3"',
            'hex = "0.4.3"\nserde_json.workspace = true',
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform-core [dependencies] drifted")
        self.assert_rejected(issues, "platform-core [dev-dependencies] drifted")

    def test_missing_identity_module_fails_closed(self) -> None:
        self.source_path().unlink()
        self.assert_rejected(self.check_identity(), "platform identity carrier missing")

    def test_missing_identity_acceptance_test_file_fails_closed(self) -> None:
        (self.root / checker.PLATFORM_IDENTITY_TEST).unlink()
        self.assert_rejected(self.check_identity(), "platform identity carrier missing")

    def test_missing_module_export_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            "// pub mod identity;",
        )
        self.assert_rejected(
            self.check_identity(), "platform-core must export the M00 identity module"
        )

    def test_missing_public_error_accessor_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "pub const fn value_kind(&self) -> &'static str {",
            "const fn value_kind(&self) -> &'static str {",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity public definition missing"
        )

    def test_missing_validating_serde_carrier_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "impl<'de> Deserialize<'de> for $name {",
            "impl<'de> DeserializeOwnedShim<'de> for $name {",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity public definition missing"
        )

    def test_missing_error_variant_fails_closed(self) -> None:
        self.rewrite(self.source_path(), "InvalidStart,", "InvalidStartRenamed,")
        self.assert_rejected(
            self.check_identity(), "platform identity error taxonomy carrier missing"
        )

    def test_dropped_value_kind_fails_closed(self) -> None:
        text = self.source_path().read_text(encoding="utf-8")
        marker = "identity_value! {"
        cut = text.rindex(marker)
        self.source_path().write_text(text[:cut], encoding="utf-8")
        issues = self.check_identity()
        self.assert_rejected(issues, "platform identity value kind missing: CorrelationId")
        self.assert_rejected(issues, "platform identity value-kind count drift")

    def test_missing_compile_fail_category_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "/// A default identity value does not exist:",
            "/// A default value is discussed elsewhere:",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity compile-fail category carrier missing",
        )

    def test_compile_fail_count_below_the_floor_fails_closed(self) -> None:
        # Nine fences exist; neutralise exactly one so the count drops below the floor.
        self.rewrite(
            self.source_path(),
            "/// ```compile_fail",
            "/// ```rust,ignore",
            occurrences=9,
            replacements=1,
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform identity compile-fail API proofs shrank")
        self.assert_rejected(
            issues, "compile-fail category is not proven by a compile_fail fence"
        )

    def test_prose_only_api_claim_without_carrier_fails_closed(self) -> None:
        # Documentation prose alone must never satisfy the AUTH-012 compile-fail evidence.
        text = self.source_path().read_text(encoding="utf-8")
        stripped = [
            line
            for line in text.splitlines()
            if line.strip() not in {"/// ```compile_fail", "/// ```"}
        ]
        self.source_path().write_text("\n".join(stripped) + "\n", encoding="utf-8")
        issues = self.check_identity()
        self.assert_rejected(issues, "platform identity compile-fail API proofs shrank")

    def test_unadmitted_identity_import_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "use std::fmt;",
            "use std::fmt;\nuse std::collections::BTreeMap;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity module declared an unadmitted import",
        )

    def test_generator_carrier_in_code_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "const MAX_IDENTITY_BYTES: usize = 128;\nfn issued_at() -> SystemTime { todo!() }",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity module gained a forbidden dependency carrier",
        )

    def test_generator_carrier_inside_prose_is_allowed(self) -> None:
        # Ordinary documentation wording must not trip the code-carrier scan.
        self.rewrite(
            self.source_path(),
            "//! Rejected input may itself be credential material",
            "//! This module never calls rand, uuid, ulid, SystemTime or reqwest.\n"
            "//! Rejected input may itself be credential material",
        )
        self.assertEqual(self.check_identity(), [])

    def test_forbidden_default_surface_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "impl Error for IdentityValueError {}",
            "impl Error for IdentityValueError {}\nimpl Default for IdentityValueError {\n"
            "    fn default() -> Self { todo!() }\n}",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity implementation surface drifted from the admitted allowlist",
        )

    def test_missing_acceptance_test_function_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "fn identity_errors_never_echo_rejected_input()",
            "fn identity_errors_are_quiet()",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity acceptance test missing: "
            "identity_errors_never_echo_rejected_input",
        )

    def test_reintroduced_invocation_tenant_definition_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "authority_id!(RunId);",
            "authority_id!(TenantId);\nauthority_id!(RunId);",
        )
        self.assert_rejected(
            self.check_identity(),
            "invocation authority reintroduced a local TenantId definition",
        )

    def test_nested_invocation_tenant_definition_fails_closed(self) -> None:
        # An indented invocation inside a nested module compiles and exposes a second
        # publicly reachable tenant type, so a column-anchored scan would be a real bypass.
        self.rewrite(
            self.invocation_path(),
            "authority_id!(RunId);",
            "pub mod compat {\n    use super::*;\n    authority_id!(TenantId);\n"
            "    authority_id!(UserId);\n}\n\nauthority_id!(RunId);",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "invocation authority reintroduced a local TenantId definition"
        )
        self.assert_rejected(
            issues, "invocation authority reintroduced a local UserId definition"
        )

    def test_brace_delimited_invocation_definition_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "authority_id!(RunId);",
            "authority_id! { TenantId }\nauthority_id!(RunId);",
        )
        self.assert_rejected(
            self.check_identity(),
            "invocation authority reintroduced a local TenantId definition",
        )

    def test_handwritten_duplicate_tenant_outside_identity_module_fails_closed(self) -> None:
        # Never touches the authority_id! generator, so only the structural guard catches it.
        (self.root / "crates/platform-core/src/legacy_scope.rs").write_text(
            "pub struct TenantId(String);\n", encoding="utf-8"
        )
        self.assert_rejected(
            self.check_identity(),
            "duplicate TenantId definition outside the M00 identity module",
        )

    # --- Public negative space. A blacklist of bad spellings cannot prove these absent, so
    # --- each is an ordinary, compiling addition to the public Rust surface.

    def test_public_unchecked_constructor_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "            /// Returns the exact canonical bytes, with case and delimiters preserved.",
            "            pub fn from_unchecked(value: String) -> Self {\n"
            "                Self(value)\n"
            "            }\n\n"
            "            /// Returns the exact canonical bytes, with case and delimiters preserved.",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity public declaration surface drifted from the admitted allowlist",
        )

    def test_qualified_public_constructor_fails_closed(self) -> None:
        # `pub async fn` is externally public and bypasses any scan that only knows `pub fn`.
        pristine = self.source_path().read_text(encoding="utf-8")
        for qualifier in ("async ", 'extern "Rust" ', "const "):
            with self.subTest(qualifier=qualifier):
                self.source_path().write_text(pristine, encoding="utf-8")
                self.rewrite(
                    self.source_path(),
                    "            /// Returns the exact canonical bytes, with case and delimiters preserved.",
                    f"            pub {qualifier}fn from_unchecked(value: String) -> Self {{\n"
                    "                Self(value)\n"
                    "            }\n\n"
                    "            /// Returns the exact canonical bytes, with case and delimiters preserved.",
                )
                self.assert_rejected(
                    self.check_identity(),
                    "platform identity public declaration surface drifted from the "
                    "admitted allowlist",
                )

    def test_unadmitted_public_union_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "identity_value! {\n    /// One platform tenant.",
            "pub union IdentitySurfaceEscape {\n    pub raw: u64,\n}\n\n"
            "identity_value! {\n    /// One platform tenant.",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues,
            "platform identity public declaration surface drifted from the admitted allowlist",
        )
        # The union's own public field is not a declaration keyword, so it must be unclassified.
        self.assert_rejected(
            issues, "platform identity module has an unclassified public declaration"
        )

    def test_restricted_visibility_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "            pub fn as_str(&self) -> &str {",
            "            pub(crate) fn as_str(&self) -> &str {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity module has an unclassified public declaration",
        )

    def test_item_forwarding_macro_impl_fails_closed(self) -> None:
        # A private item macro can expand a real trait impl whose `impl` token is preceded by
        # `(`, so preceding punctuation must not be treated as proof of type position.
        self.rewrite(
            self.source_path(),
            "identity_value! {\n    /// One platform tenant.",
            "macro_rules! emit_identity_item {\n    ($item:item) => {\n        $item\n    };\n}\n\n"
            "emit_identity_item!(impl AsRef<str> for TenantId "
            "{ fn as_ref(&self) -> &str { self.as_str() } });\n\n"
            "identity_value! {\n    /// One platform tenant.",
        )
        issues = self.check_identity()
        # Classified as an argument-position fingerprint that is not in the allowlist, so it
        # is rejected precisely rather than merely as "unclassified".
        self.assert_rejected(
            issues, "platform identity implementation surface drifted from the admitted allowlist"
        )
        self.assert_rejected(
            issues, "platform identity module macro definitions drifted"
        )

    def test_widened_generator_grammar_fails_closed(self) -> None:
        # Adds public API through the EXISTING macro, with no new macro_rules! definition.
        self.rewrite(
            self.source_path(),
            "    ($(#[$attribute:meta])* $name:ident) => {\n        $(#[$attribute])*",
            "    ($(#[$attribute:meta])* $name:ident $(, $extra:item)*) => {\n"
            "        $($extra)*\n        $(#[$attribute])*",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value generator matcher drifted from the frozen grammar",
        )

    def test_generator_invocation_forwarding_an_item_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "    TenantId\n}",
            "    TenantId,\n    impl AsRef<str> for TenantId "
            "{ fn as_ref(&self) -> &str { self.as_str() } }\n}",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value generator invocation must pass exactly one kind name",
        )

    def test_crate_level_inner_cfg_fails_closed(self) -> None:
        # `#![cfg(any())]` excludes the whole integration-test crate: both bound commands then
        # report "running 0 tests" at exit 0, and the in-suite guards never execute either.
        # `use std::any::TypeId;` also appears in the file's own item allowlist constant, so the
        # real import is the first of two occurrences.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "use std::any::TypeId;",
            "#![cfg(any())]\n\nuse std::any::TypeId;",
            occurrences=2,
            replacements=1,
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity acceptance tests must execute unconditionally",
        )

    def manifest_path(self) -> Path:
        return self.root / checker.PLATFORM_CORE_MANIFEST

    def append_to_manifest(self, text: str) -> None:
        path = self.manifest_path()
        path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")

    def workspace_manifest_path(self) -> Path:
        return self.root / checker.WORKSPACE_MANIFEST

    def test_workspace_dependency_redirect_fails_closed(self) -> None:
        # An admitted dependency NAME resolving to an attacker-authored local crate: every Rust
        # scan still reads `semver::Version` while Cargo compiles something else entirely.
        self.rewrite(
            self.workspace_manifest_path(),
            'semver = "1.0.27"',
            'semver = { path = "crates/fake-semver" }',
        )
        self.assert_rejected(
            self.check_identity(), "workspace dependency specifications drifted"
        )

    def test_workspace_hmac_dependency_redirect_fails_closed(self) -> None:
        self.rewrite(
            self.workspace_manifest_path(),
            'hmac = "0.12.1"',
            'hmac = { path = "crates/fake-hmac" }',
        )
        self.assert_rejected(
            self.check_identity(), "workspace dependency specifications drifted"
        )

    def test_workspace_base64_dependency_removal_fails_closed(self) -> None:
        self.rewrite(
            self.workspace_manifest_path(),
            'base64 = "0.22.1"\n',
            "",
        )
        self.assert_rejected(
            self.check_identity(), "workspace dependency specifications drifted"
        )

    def test_direct_dev_dependency_redirect_fails_closed(self) -> None:
        self.rewrite(
            self.manifest_path(),
            'hex = "0.4.3"',
            'hex = { path = "../fake-hex" }',
        )
        self.assert_rejected(
            self.check_identity(), "platform-core [dev-dependencies] specifications drifted"
        )

    def test_resolved_dependency_source_drift_fails_closed(self) -> None:
        # The layer beneath the specifications: a lockfile whose source line was stripped, with
        # both manifests left pristine, is still a redirect.
        lockfile = self.root / checker.WORKSPACE_LOCKFILE
        text = lockfile.read_text(encoding="utf-8")
        # Strip the source of a GOVERNED package, which is what a path redirect does to it.
        block = '[[package]]\nname = "hex"\n'
        self.assertIn(block, text, "stale lockfile mutation target")
        head, _, tail = text.partition(block)
        marker = f'source = "{checker.CRATES_IO_SOURCE}"\n'
        self.assertIn(marker, tail, "hex block lost its source line")
        lockfile.write_text(head + block + tail.replace(marker, "", 1), encoding="utf-8")
        self.assert_rejected(
            self.check_identity(), "governed dependency resolved to an unexpected source"
        )

    def test_patch_table_redirect_fails_closed(self) -> None:
        path = self.workspace_manifest_path()
        path.write_text(
            path.read_text(encoding="utf-8")
            + '\n[patch.crates-io]\nsemver = { path = "crates/fake-semver" }\n',
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_identity(), "redirects dependency sources with [patch]"
        )

    def test_cargo_config_source_replacement_fails_closed(self) -> None:
        # A registry can be replaced wholesale from outside every manifest.
        config = self.root / ".cargo/config.toml"
        config.parent.mkdir(parents=True)
        config.write_text(
            '[source.crates-io]\nreplace-with = "mirror"\n'
            '[source.mirror]\nlocal-registry = "/tmp/mirror"\n',
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_identity(), "cargo config replaces a dependency source"
        )

    def test_explicit_test_target_fails_closed(self) -> None:
        # A `[[test]]` target can point the bound `--test platform_identity` command at a
        # different file: the binding then reports "running 0 tests" at exit 0, and the guard
        # that would have noticed is exactly the one that was replaced.
        self.append_to_manifest('\n[[test]]\nname = "platform_identity"\nharness = false\n')
        self.assert_rejected(self.check_identity(), "platform-core manifest tables drifted")

    def test_extra_cargo_target_tables_fail_closed(self) -> None:
        pristine = self.manifest_path().read_text(encoding="utf-8")
        for table in ("bin", "example", "bench"):
            with self.subTest(table=table):
                self.manifest_path().write_text(pristine, encoding="utf-8")
                self.append_to_manifest(
                    f'\n[[{table}]]\nname = "probe"\npath = "../../probe.rs"\n'
                )
                self.assert_rejected(
                    self.check_identity(), "platform-core manifest tables drifted"
                )

    def test_manifest_build_script_fails_closed(self) -> None:
        self.rewrite(
            self.manifest_path(),
            'version = "0.1.0"',
            'version = "0.1.0"\nbuild = "build.rs"',
        )
        self.assert_rejected(self.check_identity(), "platform-core [package] keys drifted")

    def test_redirected_lib_target_fails_closed(self) -> None:
        self.rewrite(
            self.manifest_path(),
            'path = "src/lib.rs"',
            'path = "src/lib_hidden.rs"',
        )
        self.assert_rejected(self.check_identity(), "platform-core [lib] target drifted")

    def test_added_dependency_fails_closed(self) -> None:
        self.rewrite(
            self.manifest_path(),
            "serde.workspace = true\n",
            'serde.workspace = true\nrand = "0.8"\n',
        )
        self.assert_rejected(self.check_identity(), "platform-core [dependencies] drifted")

    def test_added_dev_dependency_fails_closed(self) -> None:
        self.rewrite(
            self.manifest_path(),
            'hex = "0.4.3"',
            'hex = "0.4.3"\nchrono = "0.4"',
        )
        self.assert_rejected(self.check_identity(), "platform-core [dev-dependencies] drifted")

    def test_non_rust_package_file_fails_closed(self) -> None:
        # A module source need not end in `.rs`, so the `*.rs` inventory cannot see one.
        (self.root / "crates/platform-core/src/identity_hidden.txt").write_text(
            "pub fn hidden() -> u32 {\n    0\n}\n", encoding="utf-8"
        )
        self.assert_rejected(self.check_identity(), "platform-core package inventory drifted")

    def test_identity_inner_attribute_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "use std::error::Error;",
            "#![allow(dead_code)]\n\nuse std::error::Error;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity module must not carry an inner attribute",
        )

    def test_spliced_source_file_fails_closed(self) -> None:
        # `include!` pulls arbitrary public items in from a file no textual scan reads.
        path = self.source_path()
        path.write_text(
            path.read_text(encoding="utf-8") + '\ninclude!("identity_extra.rs");\n',
            encoding="utf-8",
        )
        (self.root / "crates/platform-core/src/identity_extra.rs").write_text(
            "pub fn identity_bypass_marker() -> u32 {\n    42\n}\n", encoding="utf-8"
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "platform identity module must not splice external source"
        )
        self.assert_rejected(issues, "platform-core source file set drifted")

    def test_identity_submodule_declaration_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "mod identity_extra;\n\nconst MAX_IDENTITY_BYTES: usize = 128;",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity module must not declare a submodule"
        )

    def append_to_lib(self, text: str) -> None:
        path = self.root / checker.PLATFORM_CORE_LIB
        path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")

    def test_sibling_impl_behind_same_line_decoy_fn_fails_closed(self) -> None:
        # A prior `fn` item on the same rustfmt-skipped line satisfies any "looks like a
        # function signature" heuristic while the `impl` is a real item.
        self.append_to_lib(
            "\n#[rustfmt::skip]\nmod identity_surface_probe { #[allow(dead_code)] fn decoy() {}"
            " impl AsRef<str> for crate::identity::TenantId"
            " { fn as_ref(&self) -> &str { self.as_str() } } }\n"
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value implementation outside the M00 identity module",
        )

    def test_sibling_trait_impl_with_where_clause_fails_closed(self) -> None:
        # A `where` clause follows the self type; folding it in defeats the name comparison.
        self.append_to_lib(
            "\nimpl AsRef<str> for crate::identity::TenantId\nwhere\n"
            "    crate::identity::TenantId: Sized,\n{\n"
            "    fn as_ref(&self) -> &str {\n        self.as_str()\n    }\n}\n"
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value implementation outside the M00 identity module",
        )

    def test_sibling_inherent_impl_with_where_clause_fails_closed(self) -> None:
        self.append_to_lib(
            "\nimpl crate::identity::TenantId\nwhere\n"
            "    crate::identity::TenantId: Sized,\n{\n"
            "    pub fn unchecked_alias(&self) -> &str {\n        self.as_str()\n    }\n}\n"
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value implementation outside the M00 identity module",
        )

    def test_sibling_impl_through_local_use_alias_fails_closed(self) -> None:
        # A local alias does not change Rust's self type: this is a real implementation for
        # the governed kind while every textual comparison sees `Tenant`.
        self.append_to_lib(
            "\nmod identity_surface_alias_probe {\n"
            "    use crate::identity::TenantId as Tenant;\n\n"
            "    impl AsRef<str> for Tenant {\n"
            "        fn as_ref(&self) -> &str {\n            self.as_str()\n        }\n    }\n}\n"
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_sibling_public_alias_through_local_use_fails_closed(self) -> None:
        self.append_to_lib(
            "\npub mod identity_surface_public_alias_probe {\n"
            "    use crate::identity::TenantId as Tenant;\n\n"
            "    pub type TenantAlias = Tenant;\n}\n"
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_invocation_local_type_alias_fails_closed(self) -> None:
        path = self.invocation_path()
        path.write_text(
            path.read_text(encoding="utf-8")
            + "\nmod alias_probe {\n"
            "    type Tenant = crate::identity::TenantId;\n\n"
            "    impl AsRef<str> for Tenant {\n"
            "        fn as_ref(&self) -> &str {\n            self.as_str()\n        }\n    }\n}\n",
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_cfg_attr_path_module_splice_fails_closed(self) -> None:
        # `#[cfg_attr(all(), path = "x.txt")]` compiles a real Rust module from a non-.rs file:
        # invisible both to a literal `#[path` scan and to an `*.rs` source-set glob.
        self.append_to_lib(
            '\n#[cfg_attr(all(), path = "identity_surface_hidden_probe.txt")]\n'
            "mod identity_surface_hidden_probe;\n"
        )
        (self.root / "crates/platform-core/src/identity_surface_hidden_probe.txt").write_text(
            "use crate::identity::TenantId as Tenant;\n\n"
            "impl AsRef<str> for Tenant {\n"
            "    fn as_ref(&self) -> &str {\n        self.as_str()\n    }\n}\n",
            encoding="utf-8",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform-core module declarations drifted")
        self.assert_rejected(issues, "platform-core source must not carry cfg_attr")

    def test_path_module_to_non_rust_extension_fails_closed(self) -> None:
        self.append_to_lib(
            '\n#[path = "identity_surface_hidden_probe.txt"]\n'
            "mod identity_surface_hidden_probe;\n"
        )
        (self.root / "crates/platform-core/src/identity_surface_hidden_probe.txt").write_text(
            "impl AsRef<str> for crate::identity::TenantId {\n"
            "    fn as_ref(&self) -> &str {\n        self.as_str()\n    }\n}\n",
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_identity(), "platform-core module declarations drifted"
        )

    def test_removed_admitted_module_declaration_fails_closed(self) -> None:
        # The pin is exact in both directions, so it also catches a dropped module.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod invocation;",
            "// pub mod invocation;",
        )
        self.assert_rejected(
            self.check_identity(), "platform-core module declarations drifted"
        )

    def test_whole_module_reexport_fails_closed(self) -> None:
        # Names no kind, yet publishes every one of them under a second public path.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod invocation;",
            "pub mod invocation;\npub use crate::identity as identity_alias;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_admitted_invocation_reexport_is_not_a_false_positive(self) -> None:
        # The one admitted cross-file binding must keep passing.
        self.assertEqual(self.check_identity(), [])

    def test_obfuscated_path_attribute_on_admitted_module_fails_closed(self) -> None:
        # The module NAME stays admitted while Cargo compiles a different file, and the
        # comment breaks the literal `#[path` substring the previous scan searched for.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            '#[/*probe*/ path = "identity_hidden.txt"]\npub mod identity;',
        )
        (self.root / "crates/platform-core/src/identity_hidden.txt").write_text(
            "pub struct Hidden;\n", encoding="utf-8"
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform-core item declarations drifted")
        self.assert_rejected(issues, "platform-core package inventory drifted")

    def test_path_attribute_on_admitted_module_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            '#[path = "identity.rs"]\npub mod identity;',
        )
        self.assert_rejected(self.check_identity(), "platform-core item declarations drifted")

    def test_grouped_use_tree_module_reexport_fails_closed(self) -> None:
        # Every one of these republishes the identity module under a second public path while
        # never spelling `crate::identity`. rustfmt collapses a one-member group into the
        # direct spelling, so the reported carrier keeps a second member to stay fmt-stable.
        path = self.root / checker.PLATFORM_CORE_LIB
        pristine = path.read_text(encoding="utf-8")
        for variant in (
            "pub use crate::{identity as identity_alias, invocation as invocation_alias};",
            "pub use crate::{ identity::{self as identity_alias},"
            " invocation as invocation_alias };",
            "pub use identity as identity_alias;",
            "pub use self::{identity as identity_alias, invocation as invocation_alias};",
            "pub use {identity as identity_alias, invocation as invocation_alias};",
        ):
            with self.subTest(variant=variant):
                # Restore FIRST, unconditionally: `subTest` swallows the assertion error, so a
                # restore placed after it would leak one variant's mutation into the next
                # iteration's "pristine" baseline the moment any variant regressed.
                path.write_text(pristine, encoding="utf-8")
                self.rewrite(path, "pub mod invocation;", f"pub mod invocation;\n{variant}")
                self.assert_rejected(
                    self.check_identity(), "platform-core item declarations drifted"
                )

    def test_restricted_visibility_import_fails_closed(self) -> None:
        # `pub(crate)` must not collapse into the bare fingerprint of an admitted item.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "    use super::*;",
            "    pub(crate) use super::*;",
        )
        self.assert_rejected(self.check_identity(), "platform-core item declarations drifted")

    def test_removed_admitted_item_declaration_fails_closed(self) -> None:
        # Exact in both directions: a dropped import is drift too.
        self.rewrite(
            self.invocation_path(),
            "use std::collections::BTreeSet;\n",
            "",
        )
        self.assert_rejected(self.check_identity(), "platform-core item declarations drifted")

    def test_sibling_macro_definition_fails_closed(self) -> None:
        # A macro implements a trait for a governed kind while the definition shows `$t` and
        # the invocation shows only a macro call, so neither is an `impl` header or a `use`.
        self.append_to_lib(
            "\nmacro_rules! identity_impl {\n"
            "    ($t:ty) => {\n"
            "        impl AsRef<str> for $t {\n"
            "            fn as_ref(&self) -> &str {\n"
            "                self.as_str()\n            }\n        }\n    };\n}\n"
            "identity_impl!(crate::identity::TenantId);\n"
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform-core macro definitions drifted")
        self.assert_rejected(
            issues, "platform identity kind passed to a macro outside the M00 identity module"
        )

    def test_sibling_macro_invocation_naming_a_kind_fails_closed(self) -> None:
        # The definition can also live in the module that already owns an admitted macro.
        self.rewrite(
            self.invocation_path(),
            "authority_id!(RunId);",
            "authority_id!(RunId);\nauthority_id! { TenantId }",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity kind passed to a macro outside the M00 identity module",
        )

    def test_sibling_blanket_impl_fails_closed(self) -> None:
        # Names no governed kind and covers all six, so a kind blacklist cannot see it.
        self.append_to_lib(
            "\npub trait SmuggledCapability {\n"
            "    fn smuggled(&self) -> usize {\n        0\n    }\n}\n\n"
            "impl<T> SmuggledCapability for T {}\n"
        )
        self.assert_rejected(
            self.check_identity(), "platform-core sibling implementation surface drifted"
        )

    def test_removed_sibling_impl_fails_closed(self) -> None:
        # Exact in both directions, like every other allowlist here.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "impl SourceAuthority {",
            "impl SourceAuthorityRenamed {",
        )
        self.assert_rejected(
            self.check_identity(), "platform-core sibling implementation surface drifted"
        )

    def test_extern_crate_self_alias_fails_closed(self) -> None:
        # Re-roots the crate under a second name; its keyword is neither mod, use nor type.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            "extern crate self as core_alias;\npub mod identity;",
        )
        self.assert_rejected(
            self.check_identity(), "platform-core source must not carry 'extern crate'"
        )

    def test_emptied_lexical_corpus_fails_closed(self) -> None:
        # The corpus is the only thing making the two-carrier parity claim checkable.
        corpus = self.root / checker.RUST_LEXICAL_CORPUS
        corpus.write_text(json.dumps({"cases": []}) + "\n", encoding="utf-8")
        self.assert_rejected(self.check_identity(), "rust lexical corpus collapsed")

    def test_lexical_corpus_losing_a_required_case_fails_closed(self) -> None:
        corpus = self.root / checker.RUST_LEXICAL_CORPUS
        payload = json.loads(corpus.read_text(encoding="utf-8"))
        payload["cases"] = [
            case
            for case in payload["cases"]
            if case["source"] != "extern/**/crate self as z;"
        ]
        corpus.write_text(json.dumps(payload) + "\n", encoding="utf-8")
        self.assert_rejected(self.check_identity(), "rust lexical corpus lost a required case")

    def test_corpus_field_disagreeing_with_python_lexer_fails_closed(self) -> None:
        # The checker itself — not only the separate unit test — recomputes each corpus field
        # from the live Python lexer, so a mutated stripper cannot leave the checker green while
        # the corpus and the stripper drift together. Perturbing one expected field stands in for
        # that drift and must be rejected by the checker every AUTH binding runs.
        corpus = self.root / checker.RUST_LEXICAL_CORPUS
        payload = json.loads(corpus.read_text(encoding="utf-8"))
        payload["cases"][0]["stripped"] = payload["cases"][0]["stripped"] + " drift"
        corpus.write_text(json.dumps(payload) + "\n", encoding="utf-8")
        self.assert_rejected(
            self.check_identity(), "python lexer diverged from the shared corpus"
        )

    def test_corpus_macro_arms_disagreeing_with_python_lexer_fails_closed(self) -> None:
        corpus = self.root / checker.RUST_LEXICAL_CORPUS
        payload = json.loads(corpus.read_text(encoding="utf-8"))
        for case in payload["cases"]:
            if case["source"] == "macro_rules! g { ($x:expr) => {{ 1 }}; ($k:ty) => {{ 2 }}; }":
                case["macro_arms"] = [["g", ["($k:ty)"]]]  # drop the intercepting first arm
                break
        else:  # pragma: no cover - the required case must exist
            self.fail("adversarial arm case missing from corpus")
        corpus.write_text(json.dumps(payload) + "\n", encoding="utf-8")
        self.assert_rejected(
            self.check_identity(), "python lexer diverged from the shared corpus"
        )

    def test_shadowed_assertion_macro_in_bound_tests_fails_closed(self) -> None:
        # A local `macro_rules! assert_eq` leaves every admitted `assert_eq!` invocation NAME
        # in place while making the assertion type-check-only.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "const MAX_BYTES: usize = 128;",
            "macro_rules! assert_eq {\n"
            "    ($($argument:tt)*) => {{\n"
            "        let typecheck_only = || { ::core::assert_eq!($($argument)*); };\n"
            "        let _ = typecheck_only;\n"
            "    }};\n}\n\nconst MAX_BYTES: usize = 128;",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "macro definitions drifted in")
        self.assert_rejected(issues, "redefines the standard assert_eq! macro")

    def test_shadowed_assertion_macro_in_governed_source_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            "macro_rules! assert_eq {\n    ($($argument:tt)*) => {{}};\n}\n\n"
            "pub mod identity;",
        )
        self.assert_rejected(
            self.check_identity(), "redefines the standard assert_eq! macro"
        )

    def test_widened_test_helper_matcher_fails_closed(self) -> None:
        # An `$extra:item` fragment lets one call site forward an arbitrary item.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "    ($kind:ty) => {{",
            "    ($kind:ty $(, $extra:item)?) => {{",
        )
        self.assert_rejected(
            self.check_identity(), "admitted test helper macro arms drifted"
        )

    def test_earlier_no_op_helper_arm_fails_closed(self) -> None:
        # Rust reads the FIRST matching arm: a `$ignored:expr` arm captures every
        # `helper!(TenantId)` call — a type path is also an expression path — so the real
        # `$kind:ty` grammar oracle below is never reached, though it is still present.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "    ($kind:ty) => {{",
            "    ($ignored:expr) => {{\n"
            "        let _ = valid_values();\n"
            "        let _ = invalid_values();\n"
            "    }};\n    ($kind:ty) => {{",
        )
        self.assert_rejected(
            self.check_identity(), "admitted test helper macro arms drifted"
        )

    def test_no_op_helper_body_fails_closed(self) -> None:
        # One arm with the exact matcher, but a body gutted to a no-op: the grammar oracle stops
        # exercising `parse`/`value_kind`/`kind`/Serde while production can be arbitrarily wrong.
        path = self.root / checker.PLATFORM_IDENTITY_TEST
        text = path.read_text(encoding="utf-8")
        start = text.index("macro_rules! assert_kind_enforces_grammar {")
        body_open = text.index("{{", start)
        depth = 0
        index = body_open
        while index < len(text):
            if text[index] == "{":
                depth += 1
            elif text[index] == "}":
                depth -= 1
                if depth == 0:
                    break
            index += 1
        end = index + 1  # just past the transcriber's closing `}}`
        replaced = text[:body_open] + "{{ let _ = stringify!($kind); }}" + text[end:]
        self.assertNotEqual(replaced, text, "stale no-op-body mutation target")
        path.write_text(replaced, encoding="utf-8")
        self.assert_rejected(
            self.check_identity(), "admitted test helper macro lost a grammar-oracle carrier"
        )

    def test_block_local_macro_alias_in_bound_test_fails_closed(self) -> None:
        # A block-local `use std::assert as assert_eq;` rebinds `assert_eq!` for the rest of the
        # block without a `macro_rules!` or a changed invocation name, so only item accounting
        # of the test file sees it. It is dropped after the in-suite `bite` guard has run.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "    assert_eq!(tenant.as_str(), raw);",
            "    {\n"
            "        use std::assert as assert_eq;\n"
            "        assert_eq!(std::hint::black_box(true));\n"
            "    }",
        )
        self.assert_rejected(
            self.check_identity(), "bound test item declarations drifted"
        )

    def test_top_level_macro_alias_in_bound_test_fails_closed(self) -> None:
        # The same carrier at file scope: importing a macro under a shadowable name. The import
        # also appears in the file's own allowlist constant, so the real one is the first of two.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "use ustc_campus_agent_core::invocation;",
            "use ustc_campus_agent_core::invocation;\nuse std::assert as assert_ne;",
            occurrences=2,
            replacements=1,
        )
        self.assert_rejected(
            self.check_identity(), "bound test item declarations drifted"
        )

    def test_comment_split_inner_attribute_in_identity_module_fails_closed(self) -> None:
        # `# /*inner*/ ! [allow(dead_code)]` is the same inner attribute as `#![...]`.
        self.rewrite(
            self.source_path(),
            "use std::error::Error;",
            "# /*inner*/ ! [allow(dead_code)]\n\nuse std::error::Error;",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity module must not carry an inner attribute"
        )

    def test_comment_split_inner_attribute_in_sibling_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            "# /*inner*/ ! [allow(dead_code)]\n\npub mod identity;",
        )
        self.assert_rejected(
            self.check_identity(), "platform-core source must not carry an inner attribute"
        )

    def test_comment_split_extern_crate_fails_closed(self) -> None:
        # A comment is a token SEPARATOR: `extern/**/crate` is the same item as `extern crate`,
        # and `#[rustfmt::skip]` keeps the spelling stable under `cargo fmt --check`. The public
        # form publishes `ustc_campus_agent_core::core_alias::identity::TenantId`.
        for form in (
            "extern/**/crate self as core_alias;",
            "pub extern/**/crate self as core_alias;",
        ):
            with self.subTest(form=form):
                path = self.root / checker.PLATFORM_CORE_LIB
                pristine = path.read_text(encoding="utf-8")
                path.write_text(pristine, encoding="utf-8")
                self.rewrite(
                    path,
                    "pub mod identity;",
                    f"#[rustfmt::skip]\n{form}\npub mod identity;",
                )
                issues = self.check_identity()
                self.assert_rejected(
                    issues, "platform-core source must not carry 'extern crate'"
                )
                self.assert_rejected(issues, "platform-core item declarations drifted")
                path.write_text(pristine, encoding="utf-8")

    def test_comment_split_include_splice_fails_closed(self) -> None:
        # Same class in a macro: `include/*x*/!("extra.rs")` contains no `include!` substring.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            'include/*x*/!("extra.rs");\npub mod identity;',
        )
        (self.root / "crates/platform-core/src/extra.rs").write_text(
            "pub fn spliced() -> u32 {\n    7\n}\n", encoding="utf-8"
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "must not splice external source")
        self.assert_rejected(issues, "platform-core source file set drifted")

    def test_unadmitted_macro_invocation_fails_closed(self) -> None:
        # Invocation names are pinned, so a splicing macro is rejected by name rather than by
        # a substring that any spelling defeats.
        self.rewrite(
            self.invocation_path(),
            "authority_id!(RunId);",
            'authority_id!(RunId);\nconst SPLICED: &str = include_str!("invocation.rs");',
        )
        self.assert_rejected(
            self.check_identity(), "platform-core macro invocations drifted"
        )

    def test_unterminated_macro_invocation_fails_closed(self) -> None:
        # Reported rather than dropped: silently accounting for one fewer macro than the source
        # contains is exactly the hole the invocation pin is meant to remove.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            "panic!(unbalanced\npub mod identity;",
        )
        self.assert_rejected(
            self.check_identity(), "unterminated platform-core macro invocation"
        )

    def test_raw_identifier_is_not_an_item_keyword(self) -> None:
        # `r#type` is the standard escape for a reserved word as a field name. It is ordinary
        # Rust and must not be mistaken for a `type` item declaration.
        self.rewrite(
            self.invocation_path(),
            "fn is_valid_identity(value: &str) -> bool {",
            "fn raw_probe(value: &RawProbe) -> u32 {\n    value.r#type\n}\n\n"
            "struct RawProbe {\n    r#type: u32,\n}\n\n"
            "fn is_valid_identity(value: &str) -> bool {",
        )
        issues = self.check_identity()
        self.assertFalse(
            [issue for issue in issues if "item declarations drifted" in issue],
            issues,
        )

    def test_comment_split_inner_attribute_in_test_file_fails_closed(self) -> None:
        # `# /*x*/ ! [cfg(any())]` excludes the whole evidence crate exactly as `#![cfg(any())]`
        # does, and contains no `#![` substring.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "use std::any::TypeId;",
            "# /*x*/ ! [cfg(any())]\n\nuse std::any::TypeId;",
            occurrences=2,
            replacements=1,
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity acceptance tests must execute unconditionally",
        )

    def test_comment_split_ignore_attribute_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_values_are_exact_and_nominal() {",
            "#[test]\n#[ /*x*/ ignore ]\nfn identity_values_are_exact_and_nominal() {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity acceptance tests must execute unconditionally",
        )

    def test_spliced_bound_test_source_fails_closed(self) -> None:
        # The bound test file must not pull its own guards out of an unscanned file.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "use std::any::TypeId;",
            '#[path = "guards_probe.rs"]\nmod guards_probe;\n\nuse std::any::TypeId;',
            occurrences=2,
            replacements=1,
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity acceptance tests must execute unconditionally",
        )

    def test_sibling_public_reexport_alias_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod invocation;",
            "pub mod invocation;\npub use crate::identity::TenantId as TenantAlias;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_sibling_public_type_alias_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod invocation;",
            "pub mod invocation;\npub type TenantAlias = crate::identity::TenantId;",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_path_module_outside_src_fails_closed(self) -> None:
        # `#[path]` compiles a file that a `src/*.rs` glob never sees.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            'pub mod identity;\n#[path = "../identity_extra.rs"]\nmod identity_extra;',
        )
        (self.root / "crates/platform-core/identity_extra.rs").write_text(
            "impl AsRef<str> for crate::identity::TenantId {\n"
            "    fn as_ref(&self) -> &str {\n        self.as_str()\n    }\n}\n",
            encoding="utf-8",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform-core source file set drifted")
        self.assert_rejected(
            issues, "platform-core source must not splice external source"
        )

    def test_inherent_identity_impl_in_sibling_module_fails_closed(self) -> None:
        # Rust's orphan rule does not stop a second INHERENT impl for a local type from
        # another file in the same crate, and an inherent header carries no `for`.
        self.rewrite(
            self.invocation_path(),
            "fn encode_count(count: usize, output: &mut Vec<u8>) {",
            "impl TenantId {\n    pub fn smuggled_capability(&self) -> &str {\n"
            "        self.as_str()\n    }\n}\n\n"
            "fn encode_count(count: usize, output: &mut Vec<u8>) {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value implementation outside the M00 identity module",
        )

    def test_second_generator_match_arm_fails_closed(self) -> None:
        # A macro may carry several arms, so pinning one matcher line does not stop a second
        # arm being added beside it to forward an arbitrary item.
        self.rewrite(
            self.source_path(),
            "macro_rules! identity_value {\n    ($(#[$attribute:meta])* $name:ident) => {",
            "macro_rules! identity_value {\n    (@item $item:item) => {\n        $item\n    };\n"
            "    ($(#[$attribute:meta])* $name:ident) => {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value generator must have exactly one match arm",
        )

    def test_identity_impl_in_sibling_module_fails_closed(self) -> None:
        # The frozen surface belongs to the value kinds, not to one file: an implementation in
        # a sibling module adds exactly the same externally reachable API.
        self.rewrite(
            self.root / checker.PLATFORM_CORE_LIB,
            "pub mod identity;",
            "pub mod identity;\n\nimpl AsRef<str> for identity::TenantId {\n"
            "    fn as_ref(&self) -> &str {\n        self.as_str()\n    }\n}",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value implementation outside the M00 identity module",
        )

    def test_identity_impl_in_invocation_module_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "fn encode_count(count: usize, output: &mut Vec<u8>) {",
            "impl AsRef<str> for UserId {\n    fn as_ref(&self) -> &str {\n"
            "        self.as_str()\n    }\n}\n\n"
            "fn encode_count(count: usize, output: &mut Vec<u8>) {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value implementation outside the M00 identity module",
        )

    def test_multiline_cfg_attr_ignore_fails_closed(self) -> None:
        # A wrapped attribute's closing `)]` does not start with `#[`, so a reverse line scan
        # stops early and never inspects the body.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_values_are_exact_and_nominal() {",
            "#[cfg_attr(\n    all(\n        not(any()),\n        not(any()),\n    ),\n"
            "    ignore\n)]\n#[test]\nfn identity_values_are_exact_and_nominal() {",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "attribute envelope drifted")
        self.assert_rejected(
            issues, "platform identity acceptance tests must execute unconditionally"
        )

    def test_multiline_cfg_zero_test_exclusion_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_values_are_exact_and_nominal() {",
            '#[cfg(any(\n    feature = "never",\n    feature = "also-never",\n))]\n'
            "#[test]\nfn identity_values_are_exact_and_nominal() {",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "attribute envelope drifted")
        self.assert_rejected(
            issues, "platform identity acceptance tests must execute unconditionally"
        )

    def test_multiline_should_panic_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_errors_never_echo_rejected_input() {",
            '#[should_panic(\n    expected = "nothing"\n)]\n'
            "#[test]\nfn identity_errors_never_echo_rejected_input() {",
        )
        self.assert_rejected(self.check_identity(), "attribute envelope drifted")

    def test_removed_envelope_guard_from_second_test_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "    // Asserted from a second bound test as well, so ignoring AUTH-012 alone cannot silence it.\n"
            "    assert_bound_test_envelope_is_active();\n",
            "",
        )
        self.assert_rejected(
            self.check_identity(),
            "identity_values_enforce_canonical_bounds_and_errors lost the bound-test "
            "envelope guard",
        )

    def test_ignored_bound_test_fails_closed(self) -> None:
        # `#[ignore]` leaves the registered binding reporting "1 ignored" and exit 0.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_values_are_exact_and_nominal() {",
            "#[test]\n#[ignore]\nfn identity_values_are_exact_and_nominal() {",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "platform identity acceptance tests must execute unconditionally"
        )
        self.assert_rejected(issues, "carries a non-executing attribute")

    def test_deregistered_bound_test_fails_closed(self) -> None:
        # Removing only `#[test]` makes the exact command report "running 0 tests" and exit 0.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_values_are_exact_and_nominal() {",
            "fn identity_values_are_exact_and_nominal() {",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues,
            "identity_values_are_exact_and_nominal attribute envelope drifted",
        )
        self.assert_rejected(
            issues, "platform identity acceptance test registration drift"
        )

    def test_conditionally_excluded_bound_test_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_errors_never_echo_rejected_input() {",
            '#[test]\n#[cfg(feature = "slow")]\nfn identity_errors_never_echo_rejected_input() {',
        )
        self.assert_rejected(
            self.check_identity(), "carries a non-executing attribute"
        )

    def test_deferred_public_alias_fails_closed(self) -> None:
        # CausationId is owned by the later request-context batch; it must not arrive early.
        self.rewrite(
            self.source_path(),
            "identity_value! {\n    /// One platform tenant.",
            "pub type CausationId = CorrelationId;\n\nidentity_value! {\n    /// One platform tenant.",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity module declared a forbidden public item kind: 'pub type'",
        )

    def test_mutable_backing_access_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "identity_value! {\n    /// One platform tenant.",
            "impl AsMut<String> for TenantId {\n"
            "    fn as_mut(&mut self) -> &mut String { &mut self.0 }\n"
            "}\n\nidentity_value! {\n    /// One platform tenant.",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity implementation surface drifted from the admitted allowlist",
        )

    def test_cross_kind_conversion_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "identity_value! {\n    /// One platform tenant.",
            "impl From<UserId> for TenantId {\n"
            "    fn from(value: UserId) -> Self { TenantId(value.0) }\n"
            "}\n\nidentity_value! {\n    /// One platform tenant.",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity implementation surface drifted from the admitted allowlist",
        )

    def test_derived_default_fails_closed(self) -> None:
        # `#[derive(Default)]` produces a Default impl that a literal "impl Default" scan misses.
        self.rewrite(
            self.source_path(),
            "#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]",
            "#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity derive surface drifted from the admitted allowlist",
        )

    def test_spaced_derive_widens_surface_fails_closed(self) -> None:
        # `# [derive(Hash)]` (whitespace between `#` and `[`) derives exactly like `#[derive(...)]`
        # but a literal `#[derive(` scan misses it, silently admitting an extra trait impl that no
        # use/type/impl accounting can see. The added derive must change the computed surface.
        self.rewrite(
            self.source_path(),
            "pub struct IdentityValueError {",
            "# [derive(Default)]\npub struct IdentityValueError {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity derive surface drifted from the admitted allowlist",
        )

    def test_comment_split_derive_widens_surface_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "pub struct IdentityValueError {",
            "#/*x*/[derive(Default)]\npub struct IdentityValueError {",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity derive surface drifted from the admitted allowlist",
        )

    def test_selective_bypass_keeping_the_parse_call_fails_closed(self) -> None:
        # Round-12 Reproduction A: an early return for one value, with the parse call and every
        # named delegation still present. Naming entry points cannot see this; counting the
        # construction sites can, because the early return has to build the value somewhere.
        self.rewrite(
            self.source_path(),
            "                let value = String::deserialize(deserializer)?;\n"
            "                $name::parse(value).map_err(de::Error::custom)",
            "                let value = String::deserialize(deserializer)?;\n"
            '                if value == "a?b" {\n'
            "                    return Ok($name(value));\n"
            "                }\n"
            "                $name::parse(value).map_err(de::Error::custom)",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity value is constructed outside the checked constructor",
        )

    def test_extra_unchecked_visitor_method_fails_closed(self) -> None:
        # Round-12 Reproduction B: a hand-written visitor whose `visit_bytes` arm constructs
        # directly. Rejected four ways — the extra construction site, both canonical
        # `Deserialize` carriers, and the ban on hand-written visitors altogether.
        self.rewrite(
            self.source_path(),
            "                let value = String::deserialize(deserializer)?;\n"
            "                $name::parse(value).map_err(de::Error::custom)",
            "                struct V;\n"
            "                impl<'v> serde::de::Visitor<'v> for V {\n"
            "                    type Value = $name;\n"
            "                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n"
            '                        f.write_str("s")\n'
            "                    }\n"
            "                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>\n"
            "                    where\n"
            "                        E: de::Error,\n"
            "                    {\n"
            "                        let v = String::from_utf8(v.to_owned()).map_err(E::custom)?;\n"
            "                        Ok($name(v))\n"
            "                    }\n"
            "                }\n"
            "                deserializer.deserialize_string(V)",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "platform identity value is constructed outside the checked constructor"
        )
        self.assert_rejected(issues, "must not hand-write a Serde visitor")
        self.assert_rejected(issues, "Deserialize body is not the frozen one")
        self.assert_rejected(issues, "platform identity function body drifted")

    def test_concrete_kind_name_construction_fails_closed(self) -> None:
        # The private field is private to the MODULE, not to the macro expansion, so a bare
        # helper naming the concrete kind constructs without writing `$name` or `Self`. Counting
        # construction sites only closes the class if it counts every spelling of the ctor.
        self.rewrite(
            self.source_path(),
            "/// Maximum encoded length",
            "fn build_raw_value(value: &str) -> TenantId {\n"
            "    TenantId(value.to_string())\n"
            "}\n\n/// Maximum encoded length",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "platform identity value is constructed outside the checked constructor"
        )
        self.assert_rejected(issues, "platform identity function inventory drifted")

    def test_function_item_constructor_binding_fails_closed(self) -> None:
        # Round-13 blocker, verbatim. A TUPLE struct's constructor is also a VALUE, so binding
        # it to a local builds the private newtype while writing neither `$name(` nor `Self(`
        # at the construction site — satisfying every construction count there is.
        #
        # The representation now removes the value itself: a named-field struct has no
        # constructor function item, so `let ctor = $name;` no longer compiles. This proves the
        # TEXT is rejected as well, because a checker that only passed once rustc failed would
        # be relying on a gate that runs later.
        self.rewrite(
            self.source_path(),
            "                let value = value.into();\n"
            "                match classify(&value) {",
            "                let value = value.into();\n"
            '                if value == "!!! bad payload !!!" {\n'
            "                    let ctor = $name;\n"
            "                    return Ok(ctor(value));\n"
            "                }\n"
            "                match classify(&value) {",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform identity function body drifted")
        self.assert_rejected(
            issues, "platform identity checked constructor body is not the frozen one"
        )

    def test_tuple_struct_representation_fails_closed(self) -> None:
        # Reverting the representation restores the constructor function item, so the class the
        # round-13 blocker exploited would reopen. It is rejected as a representation change
        # rather than waiting for a construction expression to be counted.
        self.rewrite(
            self.source_path(),
            "        pub struct $name {\n            value: String,\n        }",
            "        pub struct $name(String);",
        )
        self.rewrite(self.source_path(), "Ok(Self { value })", "Ok(Self(value))")
        self.rewrite(self.source_path(), "&self.value", "&self.0", occurrences=3, replacements=3)
        issues = self.check_identity()
        self.assert_rejected(issues, "platform identity value representation drifted")
        self.assert_rejected(
            issues, "must declare no constructor function item"
        )
        self.assert_rejected(issues, "platform identity function body drifted")

    def test_inverted_guard_keeping_one_construction_fails_closed(self) -> None:
        # The class exact bodies close that counting construction sites cannot: ONE construction
        # expression, inside `parse`, with the `classify` call still present — and a guard that
        # skips it for a chosen value. Every construction rule in this file is satisfied.
        self.rewrite(
            self.source_path(),
            "                let value = value.into();\n"
            "                match classify(&value) {\n"
            "                    Ok(()) => Ok(Self { value }),\n"
            "                    Err(kind) => Err(IdentityValueError {\n"
            "                        value_kind: stringify!($name),\n"
            "                        kind,\n"
            "                    }),\n"
            "                }",
            "                let value = value.into();\n"
            '                if value != "!!! bad payload !!!" {\n'
            "                    if let Err(kind) = classify(&value) {\n"
            "                        return Err(IdentityValueError {\n"
            "                            value_kind: stringify!($name),\n"
            "                            kind,\n"
            "                        });\n"
            "                    }\n"
            "                }\n"
            "                Ok(Self { value })",
        )
        issues = self.check_identity()
        # Deliberately asserted: the construction rules stay GREEN here, which is why the body
        # pin had to exist.
        self.assertNotIn(
            "platform identity value is constructed outside the checked constructor",
            " ".join(issues),
        )
        self.assert_rejected(issues, "platform identity function body drifted")
        self.assert_rejected(
            issues, "platform identity checked constructor body is not the frozen one"
        )

    def test_dropping_the_exhaustive_grammar_oracle_fails_closed(self) -> None:
        # The frozen body table compares bodies after literal PAYLOADS are stripped, so the
        # bytes inside `matches!(byte, b'-' | b'.' | b'_' | b':')` are pinned by the exhaustive
        # oracle alone. Removing its call must not be silent.
        self.rewrite(
            self.bound_test_path(),
            "    assert_grammar_is_exhaustive_over_bytes();",
            "",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "lost an essential evidence carrier")

    def test_truncating_the_exhaustive_grammar_oracle_fails_closed(self) -> None:
        # …and the oracle must actually walk the byte alphabet rather than a slice of it.
        self.rewrite(
            self.bound_test_path(),
            "    for byte in 0_u8..=u8::MAX {",
            "    for byte in 0_u8..=b'z' {",
        )
        self.assert_rejected(
            self.check_identity(), "exhaustive grammar oracle lost a carrier"
        )

    def test_visitor_with_visit_string_fails_closed(self) -> None:
        # C03: the original round-11 shape — a hand-written visitor whose owned arm constructs.
        self.rewrite(
            self.source_path(),
            "                let value = String::deserialize(deserializer)?;\n"
            "                $name::parse(value).map_err(de::Error::custom)",
            "                struct V;\n"
            "                impl<'v> serde::de::Visitor<'v> for V {\n"
            "                    type Value = $name;\n"
            "                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n"
            '                        f.write_str("s")\n'
            "                    }\n"
            "                    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>\n"
            "                    where\n"
            "                        E: de::Error,\n"
            "                    {\n"
            "                        Ok(Self::Value { value: v })\n"
            "                    }\n"
            "                }\n"
            "                deserializer.deserialize_string(V)",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "must not hand-write a Serde visitor")
        self.assert_rejected(issues, "Deserialize body is not the frozen one")

    def test_alternate_deserialize_branch_fails_closed(self) -> None:
        # C05: the canonical delegation TEXT is still present, so containment is satisfied.
        self.rewrite(
            self.source_path(),
            "                let value = String::deserialize(deserializer)?;\n"
            "                $name::parse(value).map_err(de::Error::custom)",
            "                let value = String::deserialize(deserializer)?;\n"
            '                if value == "a?b" {\n'
            "                    return Ok(Self { value });\n"
            "                }\n"
            "                $name::parse(value).map_err(de::Error::custom)",
        )
        self.assert_rejected(
            self.check_identity(), "Deserialize body is not the frozen one"
        )

    def test_gutted_exhaustive_grammar_oracle_fails_closed(self) -> None:
        # E04: name and call site kept, body replaced by a no-op.
        path = self.bound_test_path()
        text = path.read_text(encoding="utf-8")
        marker = "fn assert_grammar_is_exhaustive_over_bytes() {"
        at = text.index(marker)
        index = text.index("{", at)
        start = index
        depth = 0
        while True:
            if text[index] == "{":
                depth += 1
            elif text[index] == "}":
                depth -= 1
                if depth == 0:
                    break
            index += 1
        path.write_text(
            text[:start] + "{\n    let _ = MAX_BYTES;\n}" + text[index + 1 :], encoding="utf-8"
        )
        self.assert_rejected(self.check_identity(), "exhaustive grammar oracle lost a carrier")

    def test_non_ascii_function_name_fails_closed(self) -> None:
        # rustc accepts non-ASCII identifiers; this lexer is ASCII-only, mirroring Rust's own
        # byte test in the keyword scan. A declaration it cannot name must fail closed rather
        # than drop out of the inventory unseen — otherwise the whole body accounting has a
        # silent exit for anyone willing to name a helper outside ASCII.
        self.rewrite(
            self.source_path(),
            "/// Maximum encoded length",
            "fn \u00e9scape(value: &str) -> TenantId {\n"
            "    TenantId {\n"
            "        value: value.to_string(),\n"
            "    }\n"
            "}\n\n/// Maximum encoded length",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "platform identity function body unreadable")
        self.assert_rejected(
            issues, "platform identity value is constructed outside the checked constructor"
        )

    def test_extra_identity_module_function_fails_closed(self) -> None:
        # An unadmitted helper is invisible to the `pub` scan and to `mod`/`use`/`type` item
        # accounting, so the function inventory is frozen independently of what the helper does.
        self.rewrite(
            self.source_path(),
            "/// Maximum encoded length",
            "fn unrelated_helper(value: &str) -> usize {\n"
            "    value.len()\n"
            "}\n\n/// Maximum encoded length",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity function inventory drifted"
        )

    def test_deserialize_dropping_the_checked_constructor_fails_closed(self) -> None:
        # The construction rule and the body rule are independent: dropping `parse` while
        # constructing nothing new must still fail.
        self.rewrite(
            self.source_path(),
            "                $name::parse(value).map_err(de::Error::custom)",
            "                <$name as std::str::FromStr>::from_str(&value)"
            ".map_err(de::Error::custom)",
        )
        self.assert_rejected(
            self.check_identity(), "Deserialize body is not the frozen one"
        )

    def test_raw_identifier_derive_widens_surface_fails_closed(self) -> None:
        # `#[r#derive(Default)]` derives exactly as `#[derive(Default)]` does. A literal or even
        # whitespace-tolerant `derive` scan misses the raw spelling entirely, so the reachable
        # `TenantId::default()` it grants is invisible to every use/type/impl accounting.
        self.rewrite(
            self.source_path(),
            "#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]\n",
            "#[r#derive(Default)]\n"
            "        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]\n",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform identity derive surface drifted from the admitted allowlist",
        )

    def test_raw_identifier_attribute_name_fails_closed(self) -> None:
        # The class, not the one spelling: any unadmitted attribute name is drift, raw or not.
        for attribute in ("#[r#must_use]\n", "#[r#non_exhaustive]\n"):
            with self.subTest(attribute=attribute):
                path = self.source_path()
                pristine = path.read_text(encoding="utf-8")
                self.rewrite(path, "pub enum IdentityValueErrorKind {", attribute + "pub enum IdentityValueErrorKind {")
                issues = self.check_identity()
                # `must_use` is admitted by name, so only the unadmitted one trips the name set;
                # both are still accounted for rather than skipped.
                if "non_exhaustive" in attribute:
                    self.assert_rejected(issues, "platform-core attribute names drifted")
                path.write_text(pristine, encoding="utf-8")

    def test_raw_identifier_attribute_in_sibling_fails_closed(self) -> None:
        # The rule belongs to every governed source, not only the identity module: an unadmitted
        # attribute smuggled one file over is the same carrier.
        self.rewrite(
            self.invocation_path(),
            "pub enum ComponentKind",
            "#[r#doc(hidden)]\npub enum ComponentKind",
        )
        self.assert_rejected(
            self.check_identity(),
            "platform-core attribute names drifted in crates/platform-core/src/invocation.rs",
        )

    def test_raw_identifier_ignore_on_bound_test_fails_closed(self) -> None:
        # `#[r#ignore]` suppresses a bound test while containing no `#[ignore]` substring.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "#[test]\nfn identity_errors_never_echo_rejected_input()",
            "#[r#ignore]\n#[test]\nfn identity_errors_never_echo_rejected_input()",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "bound test attribute names drifted")
        self.assert_rejected(issues, "attribute envelope drifted")

    def test_compile_fail_proof_not_exercising_denied_api_fails_closed(self) -> None:
        # A `compile_fail` fence proves only that SOMETHING failed to compile: swapping the body
        # for an unrelated type error keeps the fence, the prose and the case count green.
        self.rewrite(
            self.source_path(),
            "/// let tenant = TenantId::default();",
            '/// let tenant: u8 = TenantId::parse("ok").expect("v");',
        )
        self.assert_rejected(
            self.check_identity(),
            "compile-fail proof does not exercise the API its category denies",
        )

    def test_spaced_macro_rules_shadow_fails_closed(self) -> None:
        # `macro_rules !assert_eq` (whitespace before `!`) still defines a macro that shadows the
        # standard `assert_eq!`; a scan requiring `macro_rules!` adjacency misses the definition.
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "const MAX_BYTES: usize = 128;",
            "macro_rules !assert_eq {\n    ($($a:tt)*) => {{}};\n}\n\nconst MAX_BYTES: usize = 128;",
        )
        issues = self.check_identity()
        self.assert_rejected(issues, "macro definitions drifted in")
        self.assert_rejected(issues, "redefines the standard assert_eq! macro")

    def test_public_constant_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "pub const MAX_IDENTITY_BYTES: usize = 128;",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity module declared a public constant"
        )

    def test_vacuous_auth012_test_body_fails_closed(self) -> None:
        # The AUTH-012 binding runs this test by exact name, so an emptied body would keep the
        # binding green while proving nothing about representation, ordering or nominality.
        path = self.root / checker.PLATFORM_IDENTITY_TEST
        text = path.read_text(encoding="utf-8")
        signature = "fn identity_values_are_exact_and_nominal() {"
        self.assertEqual(text.count(signature), 1, "stale mutation target")
        start = text.index(signature)
        depth, index = 0, text.index("{", start)
        while True:
            if text[index] == "{":
                depth += 1
            elif text[index] == "}":
                depth -= 1
                if depth == 0:
                    break
            index += 1
        gutted = signature + '\n    let _ = hash_of(&"synthetic");\n}'
        path.write_text(text.replace(text[start : index + 1], gutted, 1), encoding="utf-8")
        issues = self.check_identity()
        self.assert_rejected(
            issues,
            "identity_values_are_exact_and_nominal lost an essential evidence carrier",
        )
        self.assert_rejected(
            issues, "identity_values_are_exact_and_nominal assertion count collapsed"
        )

    def test_removed_surface_guard_call_fails_closed(self) -> None:
        self.rewrite(
            self.root / checker.PLATFORM_IDENTITY_TEST,
            "    assert_public_surface_is_frozen();",
            "    // assert_public_surface_is_frozen();",
        )
        self.assert_rejected(
            self.check_identity(),
            "lost an essential evidence carrier: 'assert_public_surface_is_frozen()'",
        )

    def test_smuggled_seventh_identity_kind_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "const MAX_IDENTITY_BYTES: usize = 128;\npub struct ActorId(String);",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity public struct count drift"
        )

    def test_smuggled_public_enum_fails_closed(self) -> None:
        self.rewrite(
            self.source_path(),
            "const MAX_IDENTITY_BYTES: usize = 128;",
            "const MAX_IDENTITY_BYTES: usize = 128;\npub enum IdentityKindTag { Tenant }",
        )
        self.assert_rejected(
            self.check_identity(), "platform identity public enum count drift"
        )

    def test_indented_policy_snapshot_identity_is_not_a_false_positive(self) -> None:
        # Re-indenting an unchanged, legitimate declaration must not be reported as removal.
        self.rewrite(
            self.invocation_path(),
            "authority_id!(PolicySnapshotId);",
            "#[rustfmt::skip]\nmod m20 { use super::*; authority_id!(PolicySnapshotId); }",
        )
        issues = self.check_identity()
        self.assertFalse(
            any("PolicySnapshotId must remain M20-owned" in issue for issue in issues),
            issues,
        )

    def test_missing_invocation_reexport_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "pub use crate::identity::{TenantId, UserId};",
            "use crate::identity::{TenantId, UserId};",
        )
        issues = self.check_identity()
        self.assert_rejected(
            issues, "invocation authority must publicly re-export the M00 TenantId definition"
        )
        self.assert_rejected(
            issues, "invocation authority must publicly re-export the M00 UserId definition"
        )

    def test_partial_invocation_reexport_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "pub use crate::identity::{TenantId, UserId};",
            "pub use crate::identity::TenantId;",
        )
        self.assert_rejected(
            self.check_identity(),
            "invocation authority must publicly re-export the M00 UserId definition",
        )

    def test_migrated_policy_snapshot_identity_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "authority_id!(PolicySnapshotId);",
            "pub use crate::identity::SessionId as PolicySnapshotId;",
        )
        self.assert_rejected(
            self.check_identity(),
            "invocation PolicySnapshotId must remain M20-owned, unrenamed and unmigrated",
        )

    def test_aliased_policy_snapshot_identity_fails_closed(self) -> None:
        self.rewrite(
            self.invocation_path(),
            "pub use crate::identity::{TenantId, UserId};",
            "pub use crate::identity::{PolicySnapshotId, TenantId, UserId};",
        )
        self.assert_rejected(
            self.check_identity(),
            "invocation PolicySnapshotId must not alias a platform identity value",
        )

    def matrix_rows(self) -> list[list[str]]:
        path = self.root / "docs/acceptance/matrix.tsv"
        return [row.split("\t") for row in path.read_text(encoding="utf-8").splitlines()]

    def write_matrix(self, rows: list[list[str]]) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        path.write_text(
            "\n".join("\t".join(row) for row in rows) + "\n", encoding="utf-8"
        )

    def test_unpromoted_acceptance_status_fails_closed(self) -> None:
        rows = self.matrix_rows()
        for row in rows:
            if row[0] == "AUTH-014":
                row[5] = "planned"
        self.write_matrix(rows)
        self.assert_rejected(
            self.check_identity(),
            "platform identity acceptance status drift in AUTH-014",
        )

    def test_acceptance_binding_drift_fails_closed(self) -> None:
        rows = self.matrix_rows()
        for row in rows:
            if row[0] == "AUTH-012":
                row[3] = (
                    "cargo test --locked -p ustc-campus-agent-core --test platform_identity "
                    "identity_values_are_exact_and_nominal -- --exact"
                )
        self.write_matrix(rows)
        self.assert_rejected(
            self.check_identity(), "platform identity acceptance binding drift in AUTH-012"
        )

    def test_missing_acceptance_row_fails_closed(self) -> None:
        rows = [row for row in self.matrix_rows() if row[0] != "AUTH-016"]
        self.write_matrix(rows)
        self.assert_rejected(
            self.check_identity(), "platform identity acceptance row missing: AUTH-016"
        )

    def test_checker_not_invoked_from_main_fails_closed(self) -> None:
        path = self.root / "scripts/check_repo_contracts.py"
        self.rewrite(
            path,
            "    check_platform_identity_implementation(issues)\n",
            "    # check_platform_identity_implementation(issues)\n",
        )
        self.assert_rejected(
            self.check_identity(),
            "check_platform_identity_implementation must be invoked from repository main()",
        )


class PlatformAuthorityImplementationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        paths = (
            checker.PLATFORM_AUTHORITY_SOURCE,
            checker.PLATFORM_AUTHORITY_TEST,
            checker.PLATFORM_INVOCATION_SOURCE,
            "crates/platform-core/tests/invocation_resolution.rs",
            *checker.PLATFORM_AUTHORITY_STATUS_MARKERS.keys(),
        )
        for relative in dict.fromkeys(paths):
            source = REPO_ROOT / relative
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def path(self, relative: str) -> Path:
        return self.root / relative

    def rewrite(self, relative: str, old: str, new: str, occurrences: int = 1) -> None:
        path = self.path(relative)
        text = path.read_text(encoding="utf-8")
        self.assertEqual(
            text.count(old), occurrences, f"stale authority mutation target: {old!r}"
        )
        path.write_text(text.replace(old, new, 1), encoding="utf-8")

    def check(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_authority_implementation(issues)
        return issues

    def assert_rejected(self, expected: str) -> None:
        issues = self.check()
        self.assertTrue(any(expected in issue for issue in issues), issues)

    def test_market_authority_projection_passes_exact_repository_state(self) -> None:
        self.assertEqual(self.check(), [])

    def test_market_authority_success_body_bypass_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_SOURCE,
            "        Ok(projection)\n",
            "        Ok(projection.clone())\n",
        )
        self.assert_rejected("market authority normalized source digest drifted")
        self.assert_rejected("market authority function body drifted for resolve_projection")

    def test_market_authority_public_declaration_substitution_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_SOURCE,
            "    pub fn into_repository(self) -> R {",
            "    pub fn extract_repository(self) -> R {",
        )
        self.assert_rejected("market authority public declaration surface drifted")

    def test_market_authority_same_count_derive_substitution_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_SOURCE,
            "#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]",
            "#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]",
        )
        self.assert_rejected("market authority derive surface drifted")

    def test_market_authority_same_count_parsed_type_substitution_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_SOURCE,
            '        let request = ToolProjectionRequest {\n            tenant_id: parsed!(TenantId, "tenant:synthetic"),',
            '        let request = ToolProjectionRequest {\n            tenant_id: parsed!(UserId, "tenant:synthetic"),',
        )
        self.assert_rejected("market authority parsed macro arguments drifted")

    def test_market_authority_ignored_external_test_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_TEST,
            "#[test]\nfn projection_and_recheck_assemble_separate_carriers_under_one_verified_revision",
            "#[test]\n#[ignore]\nfn projection_and_recheck_assemble_separate_carriers_under_one_verified_revision",
        )
        self.assert_rejected("market authority external test normalized digest drifted")
        self.assert_rejected("market authority assembly tests must execute unconditionally")

    def test_market_authority_vacuous_external_test_body_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_TEST,
            "    assert_eq!(actual, expected);",
            "    let _ = (actual, expected);",
        )
        self.assert_rejected("market authority external test normalized digest drifted")

    def test_invocation_prefix_bypass_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_INVOCATION_SOURCE,
            "    let entry = preflight_projected_call(projection, &call)?;",
            "    let entry = &projection.entries()[0];",
        )
        self.assert_rejected("invocation authority prefix function body drifted for authorize_call")

    def test_post_success_verification_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_SOURCE,
            "        transaction\n            .verify_precondition()\n            .map_err(ProjectionAssemblyError::Repository)?;",
            "        transaction\n            .revision();",
        )
        self.assert_rejected("market authority function body drifted for resolve_projection")

    def test_denial_masking_reorder_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_AUTHORITY_SOURCE,
            "        let projection = InvocationResolver::resolve_projection(request, candidates)\n            .map_err(ProjectionAssemblyError::Resolution)?;",
            "        let projection = { transaction.verify_precondition().map_err(ProjectionAssemblyError::Repository)?; InvocationResolver::resolve_projection(request, candidates).map_err(ProjectionAssemblyError::Resolution)? };",
        )
        self.assert_rejected("market authority function body drifted for resolve_projection")

    def test_market_authority_status_nonclaim_deletion_fails_closed(self) -> None:
        self.rewrite(
            "docs/contracts/market-lifecycle.md",
            "not a production catalog/publication authority, durable M90 transaction, grant/enable issuer or effect-intent/I/O boundary",
            "not production-ready",
        )
        self.assert_rejected("market authority status marker missing")

    def test_market_authority_acceptance_promotion_fails_closed(self) -> None:
        matrix = self.path("docs/acceptance/matrix.tsv")
        lines = matrix.read_text(encoding="utf-8").splitlines()
        rewritten: list[str] = []
        for line in lines:
            if line.startswith("MARKET-003\t"):
                fields = line.split("\t")
                self.assertEqual(fields[5], "planned")
                fields[5] = "implemented"
                line = "\t".join(fields)
            rewritten.append(line)
        matrix.write_text("\n".join(rewritten) + "\n", encoding="utf-8")
        self.assert_rejected("MARKET-003 must remain a planned acceptance row")




class ExternalAgentAccessContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_contract(self) -> list[str]:
        issues: list[str] = []
        checker.check_external_agent_access_contract(issues)
        return issues

    def replace_once(self, rel: str, old: str, new: str) -> None:
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        self.assertEqual(text.count(old), 1, f"stale or ambiguous mutation target in {rel}")
        path.write_text(text.replace(old, new, 1), encoding="utf-8")

    def test_current_external_agent_access_contract_passes(self) -> None:
        self.assertEqual(self.check_contract(), [])

    def test_change_list_bounded_projection_removal_fails_closed(self) -> None:
        self.replace_once(
            "docs/contracts/interfaces.md",
            "Bounded loopback-only `affairs.get` and `change.list` proofs",
            "A bounded loopback-only `affairs.get` proof",
        )
        issues = self.check_contract()
        self.assertTrue(any("external-Agent semantic projection" in issue for issue in issues), issues)

    def test_operation_registry_row_removal_fails_closed(self) -> None:
        path = self.root / "docs/contracts/interfaces.md"
        lines = path.read_text(encoding="utf-8").splitlines()
        path.write_text(
            "\n".join(line for line in lines if not line.startswith("| `market.package.list` |"))
            + "\n",
            encoding="utf-8",
        )
        self.assertTrue(
            any("external-Agent application-operation registry drift" in issue for issue in self.check_contract())
        )

    def test_ambiguous_plan_alias_fails_closed(self) -> None:
        self.replace_once(
            "docs/contracts/interfaces.md",
            "`program.list`",
            "`plan.list`",
        )
        issues = self.check_contract()
        self.assertTrue(any("ambiguous external-Agent operation alias" in issue for issue in issues), issues)

    def test_private_operation_cannot_enter_initial_mcp_projection(self) -> None:
        self.replace_once(
            "docs/contracts/interfaces.md",
            "| `profile.requirement_status` | M72 | `tenant-private-read` | read | CLI, HTTP, later inbound MCP |",
            "| `profile.requirement_status` | M72 | `tenant-private-read` | read | CLI, HTTP, inbound MCP |",
        )
        issues = self.check_contract()
        self.assertTrue(any("external-Agent application-operation registry drift" in issue for issue in issues), issues)

    def test_external_write_exclusion_removal_fails_closed(self) -> None:
        self.replace_once(
            "docs/contracts/permissions.md",
            "automatic enrollment, registration, payment, form submission or any other external campus-system mutation;",
            "automatic enrollment may be enabled by a client;",
        )
        issues = self.check_contract()
        self.assertTrue(any("external-Agent semantic projection" in issue for issue in issues), issues)

    def test_windows_required_target_promotion_fails_closed(self) -> None:
        self.replace_once(
            "docs/contracts/client-shell.md",
            "not part of the current required-target gate",
            "part of the current required-target gate",
        )
        issues = self.check_contract()
        self.assertTrue(any("external-Agent semantic projection" in issue for issue in issues), issues)

    def test_client_009_windows_binary_requirement_fails_closed(self) -> None:
        self.replace_once(
            "docs/acceptance/matrix.tsv",
            "this active row covers the required initial host matrix and does not require a Windows binary or imply Windows GUI support",
            "a Windows binary proves the same CLI contract without implying Windows GUI support",
        )
        issues = self.check_contract()
        self.assertTrue(any("external-Agent acceptance assertion drift" in issue for issue in issues), issues)

    def test_client_010_implemented_claim_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines):
            if line.startswith("CLIENT-010\t"):
                fields = line.split("\t")
                fields[5] = "implemented"
                lines[index] = "\t".join(fields)
                break
        else:
            self.fail("missing CLIENT-010 row")
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        issues = self.check_contract()
        self.assertTrue(any("external-Agent acceptance posture drift" in issue for issue in issues), issues)


class PlatformSessionContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(CHECKER_PATH, self.root / "scripts/check_repo_contracts.py")
        # The sandbox mirrors the real carrier state. Before `M00-B2` landed there were none, so
        # every carrier-absence case got its precondition for free; now the promoted rows require
        # them, and a case about absence has to create that absence itself.
        for rel in (checker.PLATFORM_SESSION_SOURCE, checker.PLATFORM_SESSION_TEST):
            real = REPO_ROOT / rel
            if real.is_file():
                (self.root / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(real, self.root / rel)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_session(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_session_contract(issues)
        return issues

    def assert_rejected(self, issues: list[str], fragment: str) -> None:
        self.assertTrue(
            any(fragment in issue for issue in issues),
            f"expected an issue containing {fragment!r}, got {issues!r}",
        )

    def rewrite(self, rel: str, old: str, new: str) -> None:
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        self.assertIn(old, text)
        path.write_text(text.replace(old, new, 1), encoding="utf-8")

    def edit_matrix_cell(self, case_id: str, column: int, value: str) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        rows = path.read_text(encoding="utf-8").splitlines()
        found = False
        for index, row in enumerate(rows):
            if row.startswith(f"{case_id}\t"):
                cells = row.split("\t")
                cells[column] = value
                rows[index] = "\t".join(cells)
                found = True
        self.assertTrue(found, f"{case_id} not present in matrix.tsv")
        path.write_text("\n".join(rows) + "\n", encoding="utf-8")

    def write_implementation_carriers(
        self, source: bool, test: bool, functions: bool = True
    ) -> None:
        """Set the carrier state a promoted row is checked against, authoritatively.

        A carrier this is not asked to write is REMOVED rather than left as the sandbox found
        it, so `source=False` means "absent" whatever the real repository contains.

        `functions=False` writes a test file with no bound function in it, which is the
        stub case: the file exists, so an existence-only gate would admit it.
        """
        source_path = self.root / checker.PLATFORM_SESSION_SOURCE
        test_path = self.root / checker.PLATFORM_SESSION_TEST
        source_path.parent.mkdir(parents=True, exist_ok=True)
        test_path.parent.mkdir(parents=True, exist_ok=True)
        for wanted, path in ((source, source_path), (test, test_path)):
            if not wanted and path.is_file():
                path.unlink()
        if source:
            body = "// placeholder\n"
            if functions:
                # A promoted row is checked against its library leg as well, so the stub source
                # declares the fixtures those legs name.
                body += "#[cfg(test)]\nmod tests {\n" + "".join(
                    f"    #[test]\n    fn {name}() {{}}\n"
                    for name in checker.PLATFORM_SESSION_LIB_TEST_FUNCTIONS
                ) + "}\n"
            source_path.write_text(body, encoding="utf-8")
        if test:
            body = "// placeholder\n"
            if functions:
                contract = (self.root / checker.PLATFORM_SESSION_CONTRACT).read_text(
                    encoding="utf-8"
                )
                names = [
                    match.group("function")
                    for match in checker.PLATFORM_SESSION_BOUND_FUNCTION.finditer(contract)
                ]
                self.assertEqual(len(names), len(checker.PLATFORM_SESSION_CASES))
                body = "".join(f"#[test]\nfn {name}() {{}}\n" for name in names)
            test_path.write_text(body, encoding="utf-8")

    def test_current_platform_session_contract_passes(self) -> None:
        self.assertEqual(self.check_session(), [])

    # All four rows are `implemented`. `AUTH-018` and `AUTH-019` reach that state through an
    # additional exact library-target leg, because the §13 entries they cover need a snapshot at
    # `revision == u64::MAX` that no public call sequence produces; `platform-session/v0` §17
    # records the amendment. Pinning the set here means a silent demotion or a silent promotion
    # of a fifth row fails this test.
    IMPLEMENTED_CASES = ("AUTH-017", "AUTH-018", "AUTH-019", "AUTH-020")
    PLANNED_CASES: tuple[str, ...] = ()

    def test_case_status_split_matches_the_recorded_evidence(self) -> None:
        """`M00-B2` has landed, so this pins the post-implementation truth.

        Its predecessor asserted the opposite — four `planned` rows and absent carriers — which
        was the correct pin while the contract was accepted and unimplemented. What keeps earning
        its place is that every promotion is backed by a carrier that really declares the
        function its binding names, on both the integration and the library leg.
        """
        rows = (self.root / "docs/acceptance/matrix.tsv").read_text(encoding="utf-8")
        contract = (self.root / checker.PLATFORM_SESSION_CONTRACT).read_text(encoding="utf-8")
        test_source = (REPO_ROOT / checker.PLATFORM_SESSION_TEST).read_text(encoding="utf-8")
        self.assertTrue((REPO_ROOT / checker.PLATFORM_SESSION_SOURCE).exists())
        self.assertTrue((REPO_ROOT / checker.PLATFORM_SESSION_TEST).exists())
        self.assertEqual(
            tuple(sorted(self.IMPLEMENTED_CASES + self.PLANNED_CASES)),
            tuple(sorted(checker.PLATFORM_SESSION_CASES)),
        )
        for case_id, expected in (
            [(case, "implemented") for case in self.IMPLEMENTED_CASES]
            + [(case, "planned") for case in self.PLANNED_CASES]
        ):
            row = next(
                line for line in rows.splitlines() if line.startswith(f"{case_id}\t")
            )
            self.assertEqual(row.split("\t")[5], expected, case_id)
        # Every row's bound function exists regardless of its status, so `planned` here means
        # "one required assertion is not executable", never "no evidence was written".
        bound = [
            match.group("function")
            for match in checker.PLATFORM_SESSION_BOUND_FUNCTION.finditer(contract)
        ]
        self.assertEqual(len(bound), len(checker.PLATFORM_SESSION_CASES))
        for function in bound:
            self.assertRegex(test_source, rf"#\[test\]\nfn {function}\(\)")
        # …and the two library legs, which live in the module source rather than the test file.
        source = (REPO_ROOT / checker.PLATFORM_SESSION_SOURCE).read_text(encoding="utf-8")
        lib_bound = [
            match.group("function")
            for match in checker.PLATFORM_SESSION_LIB_BINDING.finditer(contract)
        ]
        self.assertEqual(
            sorted(lib_bound), sorted(checker.PLATFORM_SESSION_LIB_TEST_FUNCTIONS)
        )
        for function in lib_bound:
            self.assertRegex(source, rf"    #\[test\]\n    fn {function}\(\)")

    def test_missing_contract_fails_closed(self) -> None:
        (self.root / checker.PLATFORM_SESSION_CONTRACT).unlink()
        self.assert_rejected(
            self.check_session(),
            "platform session contract missing: docs/contracts/platform-session.md",
        )

    def test_contract_without_primary_code_carrier_fails_closed(self) -> None:
        # Every occurrence, not the first: the path is named in several sections, and a
        # rule that one surviving mention satisfies is not a rule about the contract.
        path = self.root / checker.PLATFORM_SESSION_CONTRACT
        text = path.read_text(encoding="utf-8")
        self.assertIn(checker.PLATFORM_SESSION_SOURCE, text)
        path.write_text(
            text.replace(
                checker.PLATFORM_SESSION_SOURCE, "crates/platform-core/src/elsewhere.rs"
            ),
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_session(), "platform session contract carrier missing"
        )

    def test_contract_binding_table_case_set_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_CONTRACT,
            "| `AUTH-020` | `python3",
            "| `AUTH-021` | `python3",
        )
        self.assert_rejected(
            self.check_session(), "platform session contract binding table drift"
        )

    def test_matrix_binding_that_leaves_the_contract_table_fails_closed(self) -> None:
        self.edit_matrix_cell(
            "AUTH-017",
            3,
            "python3 scripts/check_repo_contracts.py && cargo test --locked "
            "-p ustc-campus-agent-core --test platform_session something_else -- --exact",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance binding drift in AUTH-017",
        )

    def test_binding_without_the_repository_checker_leg_fails_closed(self) -> None:
        stripped = (
            "cargo test --locked -p ustc-campus-agent-core --test platform_session "
            "session_open_pins_immutable_scope_and_checked_deadlines -- --exact"
        )
        self.edit_matrix_cell("AUTH-017", 3, stripped)
        self.rewrite(
            checker.PLATFORM_SESSION_CONTRACT,
            f"`{checker.PLATFORM_SESSION_BINDING_PREFIX}{stripped}`",
            f"`{stripped}`",
        )
        self.assert_rejected(
            self.check_session(),
            "must run the repository checker before its Rust leg",
        )

    def test_domain_drift_fails_closed(self) -> None:
        self.edit_matrix_cell("AUTH-018", 1, "platform-identity")
        self.assert_rejected(
            self.check_session(), "platform session acceptance domain drift in AUTH-018"
        )

    def test_gate_drift_fails_closed(self) -> None:
        self.edit_matrix_cell("AUTH-020", 4, "release")
        self.assert_rejected(
            self.check_session(), "platform session acceptance gate drift in AUTH-020"
        )

    def test_assertion_drift_from_the_catalog_fails_closed(self) -> None:
        self.edit_matrix_cell("AUTH-019", 2, "something the catalog never said")
        self.assert_rejected(
            self.check_session(),
            "assertion drift between matrix and catalog in AUTH-019",
        )

    def test_missing_matrix_row_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/matrix.tsv"
        rows = path.read_text(encoding="utf-8").splitlines()
        path.write_text(
            "\n".join(row for row in rows if not row.startswith("AUTH-019\t")) + "\n",
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_session(), "platform session acceptance row missing: AUTH-019"
        )

    def test_implemented_without_any_carrier_fails_closed(self) -> None:
        self.write_implementation_carriers(source=False, test=False)
        self.edit_matrix_cell("AUTH-017", 5, "implemented")
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance status in AUTH-017 claims 'implemented'",
        )

    def test_implemented_with_only_the_source_carrier_fails_closed(self) -> None:
        self.write_implementation_carriers(source=True, test=False)
        self.edit_matrix_cell("AUTH-017", 5, "implemented")
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance status in AUTH-017 claims 'implemented'",
        )

    def test_implemented_with_only_the_test_carrier_fails_closed(self) -> None:
        self.write_implementation_carriers(source=False, test=True)
        self.edit_matrix_cell("AUTH-018", 5, "implemented")
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance status in AUTH-018 claims 'implemented'",
        )

    def test_implemented_with_a_stub_test_file_fails_closed(self) -> None:
        # The file exists but declares no bound function. `--exact` against a missing
        # function is `running 0 tests` at exit zero, so existence alone must not admit
        # the promotion — this is the case an is_file() gate would wave through.
        self.write_implementation_carriers(source=True, test=True, functions=False)
        self.edit_matrix_cell("AUTH-017", 5, "implemented")
        self.assert_rejected(
            self.check_session(),
            "declares no session_open_pins_immutable_scope_and_checked_deadlines",
        )

    def test_implemented_with_both_carriers_and_bound_function_is_admitted(self) -> None:
        self.write_implementation_carriers(source=True, test=True)
        self.edit_matrix_cell("AUTH-017", 5, "implemented")
        self.assertEqual(self.check_session(), [])

    def test_library_leg_on_an_unbound_case_fails_closed(self) -> None:
        # Each library leg belongs to one case. Attaching one to a row that owns none is drift,
        # not a harmless extra command.
        rows = (self.root / "docs/acceptance/matrix.tsv").read_text(encoding="utf-8")
        current = next(
            line for line in rows.splitlines() if line.startswith("AUTH-017\t")
        ).split("\t")[3]
        self.edit_matrix_cell(
            "AUTH-017",
            3,
            current
            + " && cargo test --locked -p ustc-campus-agent-core --lib "
            "session::tests::terminal_precedence_holds_at_the_revision_ceiling -- --exact",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance binding in AUTH-017 carries an unexpected library leg",
        )

    def test_library_leg_naming_the_wrong_fixture_fails_closed(self) -> None:
        rows = (self.root / "docs/acceptance/matrix.tsv").read_text(encoding="utf-8")
        current = next(
            line for line in rows.splitlines() if line.startswith("AUTH-018\t")
        ).split("\t")[3]
        self.edit_matrix_cell(
            "AUTH-018",
            3,
            current.replace(
                "terminal_precedence_holds_at_the_revision_ceiling",
                "revision_ceiling_fails_closed_on_decide_and_evolve",
            ),
        )
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance binding in AUTH-018 must name the exact library "
            "fixture terminal_precedence_holds_at_the_revision_ceiling",
        )

    def test_binding_naming_no_exact_test_function_fails_closed(self) -> None:
        replacement = "python3 scripts/check_repo_contracts.py && cargo test --locked"
        self.edit_matrix_cell("AUTH-019", 3, replacement)
        self.rewrite(
            checker.PLATFORM_SESSION_CONTRACT,
            "| `AUTH-019` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_revision_and_replay_are_exact_and_fail_closed -- --exact && cargo test --locked -p ustc-campus-agent-core --lib session::tests::revision_ceiling_fails_closed_on_decide_and_evolve -- --exact` |",
            f"| `AUTH-019` | `{replacement}` |",
        )
        self.assert_rejected(
            self.check_session(),
            "names no exact platform_session test function",
        )

    def test_duplicate_binding_row_for_one_case_fails_closed(self) -> None:
        # A dict comprehension keeps the last match, so a stray row placed BEFORE the real
        # table is silently shadowed by it. Counting rows is what catches that direction.
        self.rewrite(
            checker.PLATFORM_SESSION_CONTRACT,
            "| Case | Binding |\n|---|---|\n",
            "| Case | Binding |\n|---|---|\n"
            "| `AUTH-017` | `python3 scripts/check_repo_contracts.py && "
            "cargo test --locked -p ustc-campus-agent-core --test platform_session "
            "stale_stray_row -- --exact` |\n",
        )
        self.assert_rejected(
            self.check_session(),
            "binding table has duplicate or stray rows",
        )

    def test_case_missing_from_the_long_horizon_catalog_fails_closed(self) -> None:
        path = self.root / "docs/acceptance/platform-baseline.md"
        rows = path.read_text(encoding="utf-8").splitlines()
        path.write_text(
            "\n".join(row for row in rows if not row.startswith("| `AUTH-020` |")) + "\n",
            encoding="utf-8",
        )
        self.assert_rejected(
            self.check_session(),
            "acceptance case missing from long-horizon catalog: AUTH-020",
        )

    def test_checker_not_invoked_from_main_fails_closed(self) -> None:
        self.rewrite(
            "scripts/check_repo_contracts.py",
            "    check_platform_session_contract(issues)\n",
            "    # check_platform_session_contract(issues)\n",
        )
        self.assert_rejected(
            self.check_session(),
            "check_platform_session_contract must be invoked from repository main()",
        )


class PlatformSessionImplementationTests(unittest.TestCase):
    """Fail-closed regressions for the `M00-B2` source and evidence surface.

    Each case mutates a real carrier in a throwaway copy of the repository and asserts the
    checker rejects it. Removal, redirection, aliasing, broadening and stale status are covered
    separately, because they fail through different rules and a green run on one says nothing
    about the others.
    """

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        shutil.copytree(REPO_ROOT / "crates", self.root / "crates")
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(CHECKER_PATH, self.root / "scripts/check_repo_contracts.py")
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def check_session(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_session_implementation(issues)
        return issues

    def assert_rejected(self, issues: list[str], fragment: str) -> None:
        self.assertTrue(
            any(fragment in issue for issue in issues),
            f"expected an issue containing {fragment!r}, got {issues!r}",
        )

    def rewrite(self, rel: str, old: str, new: str) -> None:
        path = self.root / rel
        text = path.read_text(encoding="utf-8")
        self.assertIn(old, text)
        path.write_text(text.replace(old, new, 1), encoding="utf-8")

    def test_current_session_implementation_passes(self) -> None:
        self.assertEqual(self.check_session(), [])

    def test_missing_source_carrier_fails_closed(self) -> None:
        (self.root / checker.PLATFORM_SESSION_SOURCE).unlink()
        self.assert_rejected(
            self.check_session(),
            "platform session carrier missing: crates/platform-core/src/session.rs",
        )

    def test_undeclared_module_fails_closed(self) -> None:
        self.rewrite(checker.PLATFORM_CORE_LIB, "pub mod session;\n", "")
        self.assert_rejected(
            self.check_session(), "platform-core must export the M00 session module"
        )

    def test_removed_identity_binding_fails_closed(self) -> None:
        # Removal, not aliasing: an admitted binding that simply disappears would otherwise pass
        # the exception rule by never reaching it, so the binding is required positively too.
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "use crate::identity::{SessionId, TenantId, UserId};",
            "use crate::identity::{SessionId, TenantId};",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session module lost the enumerated identity binding",
        )

    def test_renamed_identity_binding_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "use crate::identity::{SessionId, TenantId, UserId};",
            "use crate::identity::{SessionId, TenantId, UserId as PlatformUser};",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session module lost the enumerated identity binding",
        )

    def test_aliasing_binding_is_refused_by_the_identity_exception(self) -> None:
        # The other half of the same rule, in the checker that owns it: a SECOND binding compiles
        # and leaves the first intact, so only the enumerated exception can refuse it.
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "use crate::identity::{SessionId, TenantId, UserId};",
            "use crate::identity::{SessionId, TenantId, UserId};\n"
            "use crate::identity::TenantId as Tenant;",
        )
        issues: list[str] = []
        checker.check_platform_identity_implementation(issues)
        self.assert_rejected(
            issues,
            "platform identity value alias or import outside the M00 identity module",
        )

    def test_broadened_cross_file_binding_table_admits_no_pattern(self) -> None:
        # The exception is an ENUMERATION keyed by exact file and exact text. This pins that
        # neither key is a prefix: a different file with the admitted text is refused, and so is
        # the admitted file with different text.
        for admitted_file, admitted_text in (
            checker.PLATFORM_IDENTITY_ADMITTED_CROSS_FILE_BINDINGS
        ):
            self.assertRegex(admitted_file, r"\Acrates/platform-core/src/[a-z_/]+\.rs\Z")
            self.assertRegex(admitted_text, r"\A(?:pub )?use crate::identity::\{[^}]*\};\Z")
            self.assertNotIn(" as ", admitted_text)
        self.assertEqual(len(checker.PLATFORM_IDENTITY_ADMITTED_CROSS_FILE_BINDINGS), 6)

    def test_forbidden_dependency_carrier_fails_closed(self) -> None:
        # A path-qualified call inside a function body declares no item, so the item allowlist
        # cannot see it; only the per-file carrier scan can.
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "    let bytes = value.as_bytes();",
            "    let _ = semver::Version::parse(value);\n    let bytes = value.as_bytes();",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session module gained a forbidden dependency carrier: 'semver'",
        )

    def test_tool_protocol_reference_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "    let bytes = value.as_bytes();",
            "    let _ = ustc_agent_tool_protocol::is_valid_tool_name(value);\n"
            "    let bytes = value.as_bytes();",
        )
        self.assert_rejected(
            self.check_session(),
            "forbidden dependency carrier: 'ustc_agent_tool_protocol'",
        )

    def test_broadened_public_surface_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "impl SessionSnapshot {",
            "impl SessionSnapshot {\n    pub fn from_unchecked_parts() -> u64 {\n        0\n    }\n",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session public declaration surface drifted",
        )

    def test_public_alias_of_an_identity_kind_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "use crate::identity::{SessionId, TenantId, UserId};",
            "use crate::identity::{SessionId, TenantId, UserId};\n"
            "pub type SessionOwner = UserId;",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session module declared a forbidden public item kind: 'pub type'",
        )

    def test_adapter_length_bound_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "const MAX_ADAPTER_ID_BYTES: usize = 128;",
            "const MAX_ADAPTER_ID_BYTES: usize = 64;",
        )
        self.assert_rejected(
            self.check_session(), "platform session adapter length bound drifted"
        )

    def test_adapter_interior_byte_class_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "matches!(byte, b'-' | b'.' | b'_' | b':')",
            "matches!(byte, b'-' | b'.' | b'_' | b':' | b'/')",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session adapter interior byte class drifted from the contract",
        )

    def test_digest_prefix_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            'const DIGEST_PREFIX: &str = "sha256:";',
            'const DIGEST_PREFIX: &str = "md5:";',
        )
        self.assert_rejected(
            self.check_session(), "platform session digest prefix drifted from the contract"
        )

    def test_digest_length_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "const DIGEST_HEX_DIGITS: usize = 64;",
            "const DIGEST_HEX_DIGITS: usize = 40;",
        )
        self.assert_rejected(self.check_session(), "platform session digest length drifted")

    def test_digest_byte_class_widened_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "matches!(byte, b'0'..=b'9' | b'a'..=b'f')",
            "matches!(byte, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')",
        )
        self.assert_rejected(self.check_session(), "platform session digest byte class drifted")

    def test_contract_losing_a_fenced_grammar_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_CONTRACT,
            "^sha256:[0-9a-f]{64}$",
            "^sha256:[0-9a-fA-F]{64}$",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session contract lost a fenced normative grammar",
        )

    def test_spliced_source_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "use std::error::Error;",
            'use std::error::Error;\ninclude!("hidden.rs");',
        )
        self.assert_rejected(
            self.check_session(), "platform session module must not splice external source"
        )

    def test_renamed_bound_test_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_TEST,
            "fn session_revision_and_replay_are_exact_and_fail_closed",
            "fn session_revision_and_replay",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance test missing: "
            "session_revision_and_replay_are_exact_and_fail_closed",
        )

    def test_ignored_bound_test_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_TEST,
            "#[test]\nfn session_open_pins_immutable_scope_and_checked_deadlines",
            "#[test]\n#[ignore]\nfn session_open_pins_immutable_scope_and_checked_deadlines",
        )
        self.assert_rejected(
            self.check_session(),
            "attribute envelope drifted",
        )

    def test_crate_level_exclusion_of_the_bound_suite_fails_closed(self) -> None:
        # `#![cfg(any())]` makes every bound command report `running 0 tests` at exit zero and
        # silences any guard written inside the suite, so it is refused out of band.
        self.rewrite(
            checker.PLATFORM_SESSION_TEST,
            "use ustc_campus_agent_core::identity::",
            "#![cfg(any())]\nuse ustc_campus_agent_core::identity::",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session acceptance tests must execute unconditionally",
        )

    def test_test_local_macro_definition_fails_closed(self) -> None:
        # A test-local `macro_rules! assert_eq` rebinds every admitted invocation NAME while
        # making each equality claim type-check-only.
        self.rewrite(
            checker.PLATFORM_SESSION_TEST,
            "fn tenant() -> TenantId {",
            "macro_rules! assert_eq {\n    ($($rest:tt)*) => {};\n}\n\nfn tenant() -> TenantId {",
        )
        self.assert_rejected(self.check_session(), "macro definitions drifted in")

    def test_test_item_alias_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_TEST,
            "fn tenant() -> TenantId {",
            "use std::assert as assert_eq;\n\nfn tenant() -> TenantId {",
        )
        self.assert_rejected(self.check_session(), "bound test item declarations drifted in")

    def test_renamed_library_fixture_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "fn terminal_precedence_holds_at_the_revision_ceiling",
            "fn terminal_precedence_at_ceiling",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session library fixture missing: "
            "terminal_precedence_holds_at_the_revision_ceiling",
        )

    def test_ignored_library_fixture_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "    #[test]\n    fn revision_ceiling_fails_closed_on_decide_and_evolve",
            "    #[test]\n    #[ignore]\n    fn revision_ceiling_fails_closed_on_decide_and_evolve",
        )
        self.assert_rejected(
            self.check_session(), "library fixture revision_ceiling_fails_closed_on_decide_and_evolve"
        )

    def test_file_backed_submodule_fails_closed(self) -> None:
        # The inline `#[cfg(test)] mod tests` is admitted by exact item accounting; a `mod name;`
        # compiles a second file no scan reads and stays forbidden outright.
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "use std::error::Error;",
            "use std::error::Error;\nmod hidden;",
        )
        self.assert_rejected(
            self.check_session(),
            "platform session module must not declare a file-backed submodule",
        )

    def test_second_inline_module_fails_closed(self) -> None:
        self.rewrite(
            checker.PLATFORM_SESSION_SOURCE,
            "#[cfg(test)]\nmod tests {",
            "mod extra {}\n\n#[cfg(test)]\nmod tests {",
        )
        self.assert_rejected(
            self.check_session(), "platform session module declarations drifted"
        )

    def test_checker_not_invoked_from_main_fails_closed(self) -> None:
        self.rewrite(
            "scripts/check_repo_contracts.py",
            "    check_platform_session_implementation(issues)\n",
            "    # check_platform_session_implementation(issues)\n",
        )
        self.assert_rejected(
            self.check_session(),
            "check_platform_session_implementation must be invoked from repository main()",
        )


class RepositoryCheckerRegistrationTests(unittest.TestCase):
    """All check_* functions invoked from main() must match EXPECTED_MAIN_CALLS."""

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
        "check_platform_request_context(issues)",
        "check_p1_source_revision_contract(issues)",
        "check_p1_source_registry_implementation(issues)",
        "check_m60_b2_packet_digest(issues)",
        "check_external_agent_access_contract(issues)",
        "check_module_registry(issues)",
        "check_s0_architecture_review(issues)",
        "check_ci_transition_ledger(issues)",
        "check_ci_v2_inert_fixture(issues)",
        "check_ci_governance_workflow(issues)",
        "check_checker_test_inventory(issues)",
        "check_run_checker_shards_runner(issues)",
        "check_source_sensitive_guard_registry(issues)",
    )

    def test_main_invokes_exactly_the_registered_checks(self) -> None:
        source = CHECKER_PATH.read_text(encoding="utf-8")
        main = source.split("\ndef main() -> int:", 1)
        self.assertEqual(len(main), 2, "main() declaration missing or duplicated")
        # Mirror the live checker's EXPECTED_MAIN_CALLS constant, not the literal above.
        live_calls = getattr(checker, "EXPECTED_MAIN_CALLS")
        self.assertEqual(live_calls, self.EXPECTED_MAIN_CALLS,
                         "EXPECTED_MAIN_CALLS in checker must match test literal")
        actual = tuple(
            line.strip()
            for line in main[1].splitlines()
            if line.strip().startswith("check_") and line.strip().endswith("(issues)")
        )
        self.assertEqual(actual, live_calls, "main() call order drifting from EXPECTED_MAIN_CALLS")


class SourceRegistryContractTests(unittest.TestCase):
    """P1-1 source-registry implementation carrier; exact contract/readiness + Rust gate."""

    FILES = (
        "docs/acceptance/matrix.tsv",
        "docs/contracts/source-import.md",
        "docs/plan/modules/70-campus-trust-source-pipeline.md",
        "docs/tasks/01-execution-roadmap.md",
        "docs/tasks/p1-source-revision-readiness-proposal.md",
        "crates/platform-core/src/lib.rs",
        "crates/platform-core/src/source_registry.rs",
        "crates/platform-core/tests/source_registry.rs",
        "crates/platform-core/tests/platform_identity.rs",
        "scripts/check_repo_contracts.py",
    )

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        for rel in self.FILES:
            destination = self.root / rel
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(REPO_ROOT / rel, destination)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        self.original_packet_sha = cast(
            str, getattr(checker, "P1_SOURCE_REVISION_PACKET_SHA256")
        )
        self.original_packet_bytes = cast(
            int, getattr(checker, "P1_SOURCE_REVISION_PACKET_BYTES")
        )
        self.original_contract_sha = cast(
            str, getattr(checker, "P1_SOURCE_IMPORT_V0_SECTION_SHA256")
        )
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        setattr(checker, "P1_SOURCE_REVISION_PACKET_SHA256", self.original_packet_sha)
        setattr(checker, "P1_SOURCE_REVISION_PACKET_BYTES", self.original_packet_bytes)
        setattr(checker, "P1_SOURCE_IMPORT_V0_SECTION_SHA256", self.original_contract_sha)
        self.temporary_directory.cleanup()

    def path(self, rel: str) -> Path:
        return self.root / rel

    def check_p1(self) -> list[str]:
        issues: list[str] = []
        checker.check_p1_source_revision_contract(issues)
        return issues

    def check_impl(self) -> list[str]:
        issues: list[str] = []
        checker.check_p1_source_registry_implementation(issues)
        return issues

    def rewrite(self, rel: str, old: str, new: str) -> None:
        path = self.path(rel)
        text = path.read_text(encoding="utf-8")
        self.assertEqual(text.count(old), 1, f"missing/duplicate mutation target in {rel}: {old}")
        path.write_text(text.replace(old, new, 1), encoding="utf-8")

    def rebind_packet_digest(self) -> None:
        path = self.path(checker.P1_SOURCE_REVISION_PROPOSAL)
        data = path.read_bytes()
        begin = (checker.P1_SOURCE_REVISION_PACKET_BEGIN + "\n").encode()
        end = checker.P1_SOURCE_REVISION_PACKET_END.encode()
        self.assertEqual(data.count(begin), 1)
        self.assertEqual(data.count(end), 1)
        packet = data.split(begin, 1)[1].split(end, 1)[0]
        digest = hashlib.sha256(packet).hexdigest()
        text = data.decode("utf-8")
        old = (
            f"- `Packet digest`: `sha256:{self.original_packet_sha}` "
            f"over `{self.original_packet_bytes}` bytes"
        )
        current = (
            f"- `Packet digest`: `sha256:{getattr(checker, 'P1_SOURCE_REVISION_PACKET_SHA256')}` "
            f"over `{getattr(checker, 'P1_SOURCE_REVISION_PACKET_BYTES')}` bytes"
        )
        marker = current if current in text else old
        self.assertEqual(text.count(marker), 1)
        replacement = f"- `Packet digest`: `sha256:{digest}` over `{len(packet)}` bytes"
        path.write_text(text.replace(marker, replacement, 1), encoding="utf-8")
        setattr(checker, "P1_SOURCE_REVISION_PACKET_SHA256", digest)
        setattr(checker, "P1_SOURCE_REVISION_PACKET_BYTES", len(packet))

    def rebind_contract_digest(self) -> None:
        text = self.path(checker.P1_SOURCE_IMPORT_CONTRACT).read_text(encoding="utf-8")
        heading = checker.P1_SOURCE_IMPORT_V0_SECTION_HEADING
        v0_start = text.find(heading)
        if v0_start != -1:
            v0_end = text.find("\n## 16.", v0_start)
            if v0_end == -1:
                v0_end = len(text)
            section_bytes = text[v0_start:v0_end].encode("utf-8")
            digest = hashlib.sha256(section_bytes).hexdigest()
            setattr(checker, "P1_SOURCE_IMPORT_V0_SECTION_SHA256", digest)

    def assert_rejected(self, expected: str) -> None:
        issues = self.check_p1()
        self.assertTrue(any(expected in issue for issue in issues), issues)

    def assert_impl_rejected(self, expected: str) -> None:
        issues = self.check_impl()
        self.assertTrue(any(expected in issue for issue in issues), issues)

    def test_p1_0_exact_candidate_passes(self) -> None:
        self.assertEqual(self.check_p1(), [])

    def test_p1_1_source_registry_passes(self) -> None:
        self.assertEqual(self.check_impl(), [])

    def test_packet_digest_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "one construction-ready `source-import/v0` contract",
            "one permissive `source-import/v0` contract",
        )
        self.assert_rejected("P1 source/revision exact packet drift")

    def test_p1_0_missing_path_fails_after_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "## 3. P1-0 contract/readiness scope\n\nP1-0 may edit exactly:\n\n```text\ndocs/contracts/source-import.md\ndocs/plan/modules/70-campus-trust-source-pipeline.md\n",
            "## 3. P1-0 contract/readiness scope\n\nP1-0 may edit exactly:\n\n```text\ndocs/contracts/source-import.md\n",
        )
        self.rebind_packet_digest()
        self.assert_rejected("P1-0 exact writable path allowlist drifted")

    def test_p1_0_extra_path_fails_after_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "scripts/tests/test_check_repo_contracts.py\n```\n\nP1-0 must:",
            "scripts/tests/test_check_repo_contracts.py\nREADME.md\n```\n\nP1-0 must:",
        )
        self.rebind_packet_digest()
        self.assert_rejected("P1-0 exact writable path allowlist drifted")

    def test_p1_1_extra_path_fails_after_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "scripts/tests/test_check_repo_contracts.py\n```\n\nThe implementation is the exact B1",
            "scripts/tests/test_check_repo_contracts.py\nCargo.toml\n```\n\nThe implementation is the exact B1",
        )
        self.rebind_packet_digest()
        self.assert_rejected("P1-1 exact packet writable path allowlist drifted")

    def test_p1_1_scope_amendment_carrier_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "- `Added path`: `crates/platform-core/tests/platform_identity.rs`",
            "- `Added path`: `crates/platform-core/tests/removed.rs`",
        )
        self.assert_rejected("P1-1 implementation scope amendment missing/duplicated")

    def test_p1_1_scope_authorization_receipt_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "- `Develata authorization`: `授权该单一路径的窄范围 scope amendment（推荐）`",
            "- `Develata authorization`: missing",
        )
        self.assert_rejected("P1-1 implementation scope amendment missing/duplicated")

    def test_platform_identity_module_closure_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REGISTRY_IDENTITY_TEST,
            checker.P1_SOURCE_REGISTRY_IDENTITY_MODULE_EXPECTATION,
            '&["identity", "invocation", "market", "session"] as &[&str],',
        )
        self.assert_impl_rejected(
            "P1-1 platform-identity module expectation must admit source_registry exactly once"
        )

    def test_platform_identity_item_closure_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REGISTRY_IDENTITY_TEST,
            checker.P1_SOURCE_REGISTRY_IDENTITY_ITEM_EXPECTATION,
            '    "pub mod removed;",',
        )
        self.assert_impl_rejected(
            "P1-1 platform-identity item expectation must admit source_registry exactly once"
        )

    def test_source_object_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            f"- `Source commit`: `{checker.P1_SOURCE_REVISION_SOURCE_COMMIT}`",
            f"- `Source commit`: `{'0' * 40}`",
        )
        self.assert_rejected("P1 source/revision outer metadata missing/duplicated")

    def test_src_001_reversion_fails_closed(self) -> None:
        matrix = self.path("docs/acceptance/matrix.tsv")
        lines = matrix.read_text(encoding="utf-8").splitlines()
        index = next(i for i, line in enumerate(lines) if line.startswith("SRC-001\t"))
        fields = lines[index].split("\t")
        self.assertEqual(fields[5], "implemented")
        fields[5] = "planned"
        lines[index] = "\t".join(fields)
        matrix.write_text("\n".join(lines) + "\n", encoding="utf-8")
        self.assert_rejected("P1-1 requires SRC-001 to be implemented")

    def test_src_001_binding_drift_fails_closed(self) -> None:
        matrix = self.path("docs/acceptance/matrix.tsv")
        lines = matrix.read_text(encoding="utf-8").splitlines()
        index = next(i for i, line in enumerate(lines) if line.startswith("SRC-001\t"))
        fields = lines[index].split("\t")
        self.assertEqual(fields[5], "implemented")
        fields[3] = "cargo test --locked -p ustc-campus-agent-core --lib"
        lines[index] = "\t".join(fields)
        matrix.write_text("\n".join(lines) + "\n", encoding="utf-8")
        self.assert_rejected("P1-1 requires SRC-001 to be implemented with exact binding")

    def test_m60_state_promotion_fails_closed(self) -> None:
        self.rewrite(
            "docs/plan/modules/70-campus-trust-source-pipeline.md",
            "- `Implementation State`: `planned`",
            "- `Implementation State`: `partial-evidence`",
        )
        self.assert_rejected("P1 source/revision status projection drifted")

    def test_concrete_source_approval_fails_after_contract_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_IMPORT_CONTRACT,
            "- `Proposed 2025 SourceId`: `ustc-teach-calendar-fall-2025`",
            "- `Proposed 2025 SourceId`: `ustc-teach-calendar-fall-2025`\n- `Review state`: `Approved`",
        )
        self.rebind_contract_digest()
        self.assert_rejected("P1 concrete source candidate must not contain 'Approved'")

    def test_duplicate_url_alias_fails_after_contract_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_IMPORT_CONTRACT,
            "- duplicate `SourceId` or duplicate canonical `SourceUrl` is rejected without replacing the first definition;",
            "- duplicate `SourceId` is rejected; duplicate canonical `SourceUrl` aliases are accepted;",
        )
        self.rebind_contract_digest()
        self.assert_rejected("P1 source-import truth projection missing/duplicated")

    def test_model_inference_source_authority_fails_after_contract_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_IMPORT_CONTRACT,
            "`ModelInference` is rejected by `SourceDefinition::proposed`.",
            "`ModelInference` is accepted by `SourceDefinition::proposed`.",
        )
        self.rebind_contract_digest()
        self.assert_rejected("P1 source-import truth projection missing/duplicated")

    def test_source_definition_constructor_must_remain_in_owner_section(self) -> None:
        contract = self.path(checker.P1_SOURCE_IMPORT_CONTRACT)
        text = contract.read_text(encoding="utf-8")
        signature = (
            "SourceDefinitionBody::new(\n"
            "    owner: SourceOwner,\n"
            "    url: SourceUrl,\n"
            "    authority: SourceAuthority,\n"
            "    retrieval_policy: SourceRetrievalPolicy,\n"
            ") -> Result<Self, SourceValueError>"
        )
        self.assertEqual(text.count(signature), 1)
        changed = text.replace(
            signature,
            signature.replace(
                "    retrieval_policy: SourceRetrievalPolicy,\n",
                "    retrieval_policy: SourceRetrievalPolicy,\n    initial_status: SourceStatus,\n",
            ),
            1,
        )
        insertion = "\n\nHistorical stale signature (non-authoritative):\n\n```text\n" + signature + "\n```\n"
        changed = changed.replace("\n## 15. `source-import/v0`", insertion + "\n## 15. `source-import/v0`", 1)
        contract.write_text(changed, encoding="utf-8")
        self.rebind_contract_digest()
        self.assert_rejected("P1 source-import §4.1 constructor authority drifted")

    def test_raw_source_fixture_fails_closed(self) -> None:
        fixture = self.root / "market/fixtures/calendar-2025-fall.body"
        fixture.parent.mkdir(parents=True, exist_ok=True)
        fixture.write_bytes(b"synthetic")
        self.assert_rejected("P1 raw source evidence must remain outside repository")

    def test_remote_shipping_drift_fails_after_packet_digest_rebound(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "This packet authorizes no push, PR, merge, tag, release or deployment.",
            "This packet authorizes push and PR after local tests.",
        )
        self.rebind_packet_digest()
        self.assert_rejected("P1 source/revision packet semantic carrier drifted")

    def test_p1_0_go_receipt_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "### P1-0 exact-candidate GO receipt",
            "### P1-0 review receipt pending",
        )
        self.assert_rejected("P1-0 GO receipt missing/duplicated")

    def test_p1_1_go_receipt_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "### P1-1 amended exact-candidate GO receipt",
            "### P1-1 review receipt pending",
        )
        self.assert_rejected("P1-1 GO receipt missing/duplicated")

    def test_contract_acceptance_reversion_fails_after_contract_digest_rebound(self) -> None:
        # Mutation changes the §15 section's byte count, so the byte-count gate fires
        # before the digest gate. Both assertions capture primary evidence.
        self.rewrite(
            checker.P1_SOURCE_IMPORT_CONTRACT,
            "- `Status`: accepted for bounded `M60-B1 source-registry`, implemented as a P1-1 review candidate",
            "- `Status`: P1-0 candidate; explicit review pending",
        )
        self.rebind_contract_digest()
        issues = self.check_p1()
        self.assertTrue(
            any("P1 source-import historical v0 section §15 byte count drift" in issue
                or "P1 source-import historical v0 section §15 digest drift" in issue
                for issue in issues),
            issues,
        )

    def test_remote_shipping_authority_expansion_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "- `Remote shipping`: feature-branch push, PR, Actions CI/run and workflow dispatch authorized; merge/tag/release remain forbidden",
            "- `Remote shipping`: feature-branch push, PR, Actions CI/run, workflow dispatch and merge authorized",
        )
        self.assert_rejected("P1 source/revision outer metadata missing/duplicated")

    def test_pr_ci_receipt_removal_fails_closed(self) -> None:
        self.rewrite(
            checker.P1_SOURCE_REVISION_PROPOSAL,
            "- `PR`: `#38` (`https://github.com/Develata/ustc-campus-agent/pull/38`), open against `main`",
            "- `PR`: pending",
        )
        self.assert_rejected("P1 source/revision outer metadata missing/duplicated")

    def test_checker_entrypoint_is_bound_outside_the_checker(self) -> None:
        source = self.path("scripts/check_repo_contracts.py").read_text(encoding="utf-8")
        main = source.split("\ndef main() -> int:", 1)
        self.assertEqual(len(main), 2)
        calls = [
            line
            for line in main[1].splitlines()
            if line.strip() == "check_p1_source_revision_contract(issues)"
        ]
        self.assertEqual(calls, ["    check_p1_source_revision_contract(issues)"])

    def test_m60_b2_packet_digest_is_bound_in_main(self) -> None:
        source = self.path("scripts/check_repo_contracts.py").read_text(encoding="utf-8")
        main = source.split("\ndef main() -> int:", 1)
        self.assertEqual(len(main), 2)
        calls = [
            line
            for line in main[1].splitlines()
            if line.strip() == "check_m60_b2_packet_digest(issues)"
        ]
        self.assertEqual(calls, ["    check_m60_b2_packet_digest(issues)"])


class SourceSensitiveGuardRegistryTests(unittest.TestCase):
    """R1: fingerprint moratorium enforcement via fail-closed AST governance."""

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        (self.root / "scripts").mkdir(parents=True)
        self.checker_path = self.root / "scripts" / "check_repo_contracts.py"
        shutil.copy2(CHECKER_PATH, self.checker_path)
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        self.temporary_directory.cleanup()

    def run_governance(self) -> list[str]:
        issues: list[str] = []
        checker.check_source_sensitive_guard_registry(issues)
        return issues

    def rewrite_checker(self, source: str) -> None:
        self.checker_path.write_text(source, encoding="utf-8")

    def test_pristine_checker_passes(self) -> None:
        self.assertEqual(self.run_governance(), [])

    def test_exact_source_authority_guard_mutation_fails_closed(self) -> None:
        source = self.checker_path.read_text(encoding="utf-8")
        marker = (
            '    identity_test = identity_test_path.read_text(encoding="utf-8")\n'
            '    stripped = strip_rust_comments_and_literals(source)\n'
        )
        self.assertEqual(source.count(marker), 1)
        mutation = (
            '    identity_test = identity_test_path.read_text(encoding="utf-8")\n'
            '    stripped = strip_rust_comments_and_literals(source)\n'
            '    if "pub enum SourceAuthority {" not in lib:\n'
            '        fail("R1 mutation probe", issues)\n'
        )
        self.rewrite_checker(source.replace(marker, mutation, 1))
        issues = self.run_governance()
        self.assertTrue(
            any("check_p1_source_registry_implementation" in i and "digest mismatch" in i for i in issues),
            f"expected digest mismatch for check_p1_source_registry_implementation, got {issues}",
        )

    def test_comment_only_change_passes(self) -> None:
        source = self.checker_path.read_text(encoding="utf-8")
        marker = (
            '    identity_test = identity_test_path.read_text(encoding="utf-8")\n'
            '    stripped = strip_rust_comments_and_literals(source)\n'
        )
        self.assertEqual(source.count(marker), 1)
        with_comment = (
            '    identity_test = identity_test_path.read_text(encoding="utf-8")\n'
            '    # R1 comment-only probe: must not change AST digest\n'
            '    stripped = strip_rust_comments_and_literals(source)\n'
        )
        self.rewrite_checker(source.replace(marker, with_comment, 1))
        self.assertEqual(self.run_governance(), [])

    def test_unregistered_new_source_sensitive_function_fails_closed(self) -> None:
        source = self.checker_path.read_text(encoding="utf-8")
        injection = (
            '\n\ndef _r1_probe_unregistered(issues: list[str]) -> None:\n'
            '    path = __import__("pathlib").Path("x")\n'
            '    path.read_text(encoding="utf-8")\n'
        )
        self.rewrite_checker(source + injection)
        issues = self.run_governance()
        self.assertTrue(
            any("_r1_probe_unregistered" in i and "unregistered" in i for i in issues),
            f"expected unregistered function rejection, got {issues}",
        )

    def test_registered_function_no_longer_source_sensitive_fails_closed(self) -> None:
        source = self.checker_path.read_text(encoding="utf-8")
        old_body = (
            'def load_json(rel: str, issues: list[str]) -> object | None:\n'
            '    path = ROOT / rel\n'
            '    try:\n'
            '        return json.loads(path.read_text(encoding="utf-8"))\n'
            '    except Exception as exc:  # noqa: BLE001 - checker should report exact file\n'
            '        fail(f"invalid json {rel}: {exc}", issues)\n'
            '        return None\n'
        )
        new_body = (
            'def load_json(rel: str, issues: list[str]) -> object | None:\n'
            '    path = ROOT / rel\n'
            '    try:\n'
            '        return json.loads("{}")\n'
            '    except Exception as exc:  # noqa: BLE001 - checker should report exact file\n'
            '        fail(f"invalid json {rel}: {exc}", issues)\n'
            '        return None\n'
        )
        self.assertIn(old_body, source)
        self.rewrite_checker(source.replace(old_body, new_body, 1))
        issues = self.run_governance()
        self.assertTrue(
            any("load_json" in i and "no longer source-sensitive" in i for i in issues),
            f"expected removed-carrier rejection for load_json, got {issues}",
        )

    def test_replaced_status_fails_closed(self) -> None:
        target = "load_json"
        original_status = checker.SOURCE_SENSITIVE_GUARD_REGISTRY[target]["status"]
        try:
            checker.SOURCE_SENSITIVE_GUARD_REGISTRY[target]["status"] = "replaced"
            issues = self.run_governance()
            self.assertTrue(
                any(target in i and "replaced" in i and "only 'active' is admitted" in i for i in issues),
                f"expected replaced-status rejection for {target}, got {issues}",
            )
        finally:
            checker.SOURCE_SENSITIVE_GUARD_REGISTRY[target]["status"] = original_status


class CiGovernanceWorkflowTests(unittest.TestCase):
    WORKFLOW_REL = ".github/workflows/ci-governance.yml"

    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory()
        self.root = Path(self._temp.name)
        for source in (REPO_ROOT / ".github/workflows").iterdir():
            if source.suffix not in {".yml", ".yaml"}:
                continue
            destination = self.root / ".github/workflows" / source.name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        self.workflow = self.root / self.WORKFLOW_REL
        self.baseline = self.workflow.read_text(encoding="utf-8")
        self._original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self._original_root)
        self._temp.cleanup()

    def _reset_workflow(self) -> None:
        self.workflow.write_text(self.baseline, encoding="utf-8")

    def _mutate_workflow(self, old: str, new: str) -> None:
        self._reset_workflow()
        text = self.workflow.read_text(encoding="utf-8")
        self.assertEqual(text.count(old), 1, old)
        self.workflow.write_text(text.replace(old, new, 1), encoding="utf-8")

    def _check(self) -> list[str]:
        issues: list[str] = []
        checker.check_ci_governance_workflow(issues)
        return issues

    def _assert_rejected(self, fragment: str = "CI-governance") -> None:
        issues = self._check()
        self.assertTrue(any(fragment in issue for issue in issues), issues)

    def _controller(self) -> dict[str, object]:
        script, error = checker._extract_ci_governance_controller(
            self.workflow.read_text(encoding="utf-8")
        )
        self.assertIsNone(error)
        self.assertIsNotNone(script)
        namespace: dict[str, object] = {"__name__": "ci_governance_test"}
        exec(compile(cast(str, script), self.WORKFLOW_REL, "exec"), namespace)
        return namespace

    def _controller_failure(self, namespace: dict[str, object], call: Callable[[], object]) -> str:
        failure = cast(type[Exception], namespace["ControllerFailure"])
        with self.assertRaises(failure) as caught:
            call()
        return str(caught.exception)

    def _record(
        self,
        namespace: dict[str, object],
        *,
        record_id: int = 41,
        conclusion: str = "failure",
        external_id: str | None = None,
        head_sha: str = "a" * 40,
        app_id: int = 15368,
        app_slug: str = "github-actions",
    ) -> dict[str, object]:
        if external_id is None:
            external = cast(Callable[[int, str], str], namespace["external_identity"])
            external_id = external(1308221459, head_sha)
        return {
            "id": record_id,
            "name": "ci-governance",
            "head_sha": head_sha,
            "status": "completed",
            "conclusion": conclusion,
            "external_id": external_id,
            "app": {"id": app_id, "slug": app_slug},
        }

    def _pull_payload(
        self, head_sha: str, changed_files: int, base_sha: str = "b" * 40
    ) -> dict[str, object]:
        repository = {"full_name": "Develata/ustc-campus-agent"}
        return {
            "state": "open",
            "updated_at": "2026-08-26T00:00:00Z",
            "changed_files": changed_files,
            "head": {"sha": head_sha, "repo": repository},
            "base": {"ref": "main", "sha": base_sha, "repo": repository},
            "user": {"login": "author", "id": 2},
        }

    def _run_controller_scenario(
        self,
        namespace: dict[str, object],
        *,
        event_name: str,
        event: dict[str, object],
        changed_file: dict[str, object],
        initial: dict[str, object] | None,
        conclusion: str,
    ) -> tuple[tuple[str, int], list[tuple[str, str, object, int]], dict[str, bool]]:
        head = "a" * 40
        repository = "Develata/ustc-campus-agent"
        pull = self._pull_payload(head, 1)
        final = self._record(namespace, conclusion=conclusion)
        def stable_observation() -> list[tuple[str, str, object, int]]:
            return [
                ("GET", f"/repos/{repository}/pulls/7", pull, 200),
                ("GET", f"/repos/{repository}/pulls/7/files?per_page=100&page=1", [changed_file], 200),
                ("GET", f"/repos/{repository}/pulls/7/files?per_page=100&page=2", [], 200),
                ("GET", f"/repos/{repository}/pulls/7", pull, 200),
                ("GET", f"/repos/{repository}/pulls/7/files?per_page=100&page=1", [changed_file], 200),
                ("GET", f"/repos/{repository}/pulls/7/files?per_page=100&page=2", [], 200),
                ("GET", f"/repos/{repository}/pulls/7", pull, 200),
            ]
        queue: list[tuple[str, str, object, int]] = [
            ("GET", f"/repos/{repository}", {
                "id": 1308221459,
                "default_branch": "main",
                "owner": {"login": "Develata", "id": 179165876, "type": "User"},
            }, 200),
        ]
        queue.extend(stable_observation())
        if event_name == "issue_comment":
            queue.append(("GET", f"/repos/{repository}/pulls/7", pull, 200))
        initial_runs = [] if initial is None else [initial]
        queue.extend(
            [
                ("GET", f"/repos/{repository}/commits/{head}/check-runs?filter=all&per_page=100&page=1", {"total_count": len(initial_runs), "check_runs": initial_runs}, 200),
                ("GET", f"/repos/{repository}/commits/{head}/check-runs?filter=all&per_page=100&page=2", {"total_count": len(initial_runs), "check_runs": []}, 200),
            ]
        )
        queue.extend(stable_observation())
        mutation_method = "POST" if initial is None else "PATCH"
        mutation_path = (
            f"/repos/{repository}/check-runs"
            if initial is None
            else f"/repos/{repository}/check-runs/41"
        )
        queue.extend(
            [
                (mutation_method, mutation_path, {"id": 41}, 201 if mutation_method == "POST" else 200),
                ("GET", f"/repos/{repository}/commits/{head}/check-runs?filter=all&per_page=100&page=1", {"total_count": 1, "check_runs": [final]}, 200),
                ("GET", f"/repos/{repository}/commits/{head}/check-runs?filter=all&per_page=100&page=2", {"total_count": 1, "check_runs": []}, 200),
                ("GET", f"/repos/{repository}/check-runs/41", final, 200),
            ]
        )
        calls: list[tuple[str, str, object, int]] = []

        def api_request(
            environment: dict[str, str],
            method: str,
            path: str,
            body: object = None,
            expected_status: int = 200,
        ) -> object:
            self.assertEqual(environment["GITHUB_REPOSITORY"], repository)
            self.assertTrue(queue, (method, path))
            expected_method, expected_path, response, status = queue.pop(0)
            self.assertEqual((method, path, expected_status), (expected_method, expected_path, status))
            calls.append((method, path, body, expected_status))
            return response

        environment = {
            "GITHUB_API_URL": "https://api.github.invalid",
            "GITHUB_EVENT_NAME": event_name,
            "GITHUB_EVENT_PATH": "/event.json",
            "GITHUB_REPOSITORY": repository,
            "GITHUB_TOKEN": "token",
        }
        publication = {"attempted": False}
        with mock.patch.dict(
            namespace,
            {
                "read_environment": lambda: environment,
                "read_event": lambda *_: event,
                "api_request": api_request,
            },
        ):
            actual = cast(
                Callable[[dict[str, bool]], tuple[str, int]],
                namespace["run_controller"],
            )(publication)
        self.assertEqual(queue, [])
        return actual, calls, publication

    def test_governance_workflow_baseline_contract(self) -> None:
        self.assertEqual(self._check(), [])
        script, error = checker._extract_ci_governance_controller(self.baseline)
        self.assertIsNone(error)
        tree = ast.parse(cast(str, script))
        self.assertEqual(
            checker._ci_governance_controller_manifest(tree),
            checker.CI_GOVERNANCE_CONTROLLER_AST_MANIFEST,
        )
        namespace = self._controller()
        self.assertEqual(namespace["REPOSITORY_ID"], 1308221459)
        self.assertEqual(namespace["GITHUB_ACTIONS_APP_ID"], 15368)
        external = cast(Callable[[int, str], str], namespace["external_identity"])
        self.assertEqual(
            external(1308221459, "a" * 40),
            "ci-governance:1308221459:" + "a" * 40,
        )

    def test_governance_workflow_grammar_mutations_fail_closed(self) -> None:
        mutations = (
            ("name: ci-governance-controller", "name: ci-governance-shadow"),
            ("    types: [opened, synchronize, reopened]", "    types: [opened, reopened]"),
            ("  checks: write", "  checks: read"),
            ("  group: ci-governance-controller", "  group: ci-governance-per-pr"),
            ("  cancel-in-progress: false", "  cancel-in-progress: true"),
            ("    runs-on: ubuntu-24.04", "    runs-on: ubuntu-latest"),
            ("          python3 - <<'PY'", "          python - <<'PY'"),
        )
        for old, new in mutations:
            with self.subTest(old=old):
                self._mutate_workflow(old, new)
                self._assert_rejected("grammar drift")
        script, error = checker._extract_ci_governance_controller(self.baseline)
        self.assertIsNone(error)
        for index in range(len(checker.CI_GOVERNANCE_CONTROLLER_AST_MANIFEST)):
            with self.subTest(ast_node=index):
                tree = ast.parse(cast(str, script))
                node = tree.body[index]
                if isinstance(node, ast.Import):
                    node.names[0].asname = "_mutated"
                elif isinstance(node, (ast.Assign, ast.AnnAssign)):
                    node.value = ast.Tuple(
                        elts=[cast(ast.expr, node.value)], ctx=ast.Load()
                    )
                elif isinstance(node, (ast.FunctionDef, ast.ClassDef, ast.If)):
                    node.body.insert(0, ast.Expr(value=ast.Constant(value="_mutated")))
                else:
                    self.fail(f"unhandled manifest node: {type(node).__name__}")
                ast.fix_missing_locations(tree)
                rendered = ast.unparse(tree)
                indented = "\n".join(
                    checker.CI_GOVERNANCE_SCRIPT_INDENT + line if line else ""
                    for line in rendered.splitlines()
                )
                self.workflow.write_text(
                    checker.CI_GOVERNANCE_YAML_PREFIX
                    + indented
                    + "\n"
                    + checker.CI_GOVERNANCE_YAML_SUFFIX,
                    encoding="utf-8",
                )
                self._assert_rejected(f"normalized AST node drift at index {index}")

    def test_governance_protected_registry_mutations_fail_closed(self) -> None:
        mutations = (
            (
                'PROTECTED_PREFIXES = (".github/workflows/", ".github/actions/")',
                'PROTECTED_PREFIXES = (".github/workflow/", ".github/actions/")',
            ),
            (
                '    "scripts/check_repo_contracts.py",',
                '    "scripts/check_repo_contract.py",',
            ),
            (
                '    "docs/acceptance/gates.md",',
                '    "docs/acceptance/gates.md",\n              "README.md",',
            ),
        )
        for old, new in mutations:
            with self.subTest(old=old):
                self._mutate_workflow(old, new)
                self._assert_rejected("normalized AST node drift")
        self._reset_workflow()
        script, error = checker._extract_ci_governance_controller(self.baseline)
        self.assertIsNone(error)
        tree = ast.parse(cast(str, script))
        self.assertEqual(
            checker._ci_governance_assignment_value(tree, "PROTECTED_PREFIXES"),
            checker.CI_GOVERNANCE_PROTECTED_PREFIXES,
        )
        self.assertEqual(
            checker._ci_governance_assignment_value(tree, "PROTECTED_PATHS"),
            checker.CI_GOVERNANCE_PROTECTED_PATHS,
        )
        original_paths = checker.CI_GOVERNANCE_PROTECTED_PATHS
        try:
            setattr(
                checker,
                "CI_GOVERNANCE_PROTECTED_PATHS",
                frozenset(original_paths - {"scripts/check_repo_contracts.py"}),
            )
            self._assert_rejected("protected path registry drift")
        finally:
            setattr(checker, "CI_GOVERNANCE_PROTECTED_PATHS", original_paths)

    def test_governance_ordinary_and_protected_paths(self) -> None:
        namespace = self._controller()
        protected = cast(Callable[[list[object]], bool], namespace["is_protected_change"])
        self.assertFalse(protected([{"filename": "README.md", "status": "modified"}]))
        for record in (
            {"filename": "scripts/check_repo_contracts.py", "status": "modified"},
            {"filename": ".github/actions/x/action.yml", "status": "added"},
            {
                "filename": "README.md",
                "status": "renamed",
                "previous_filename": ".github/workflows/old.yml",
            },
        ):
            with self.subTest(record=record):
                self.assertTrue(protected([record]))
        event = {"action": "opened", "pull_request": {"number": 7}}
        for changed_file, conclusion in (
            ({"filename": "README.md", "status": "modified"}, "success"),
            ({"filename": "scripts/check_repo_contracts.py", "status": "modified"}, "failure"),
        ):
            with self.subTest(end_to_end=conclusion):
                actual, calls, publication = self._run_controller_scenario(
                    namespace,
                    event_name="pull_request_target",
                    event=event,
                    changed_file=changed_file,
                    initial=None,
                    conclusion=conclusion,
                )
                self.assertEqual(actual, (conclusion, 41))
                mutations = [call for call in calls if call[0] != "GET"]
                self.assertEqual(len(mutations), 1)
                self.assertEqual(mutations[0][0], "POST")
                self.assertEqual(cast(dict[str, object], mutations[0][2])["conclusion"], conclusion)
                self.assertTrue(publication["attempted"])

    def test_governance_owner_grant_identity_and_event(self) -> None:
        namespace = self._controller()
        evaluate = cast(Callable[[dict[str, str], int, dict[str, object], str], bool], namespace["evaluate_grant"])
        head = "a" * 40
        comment: dict[str, object] = {
            "body": "CI-GOVERNANCE-APPROVED:" + head,
            "user": {"id": 179165876, "login": "Develata", "type": "User"},
            "author_association": "OWNER",
        }
        with mock.patch.dict(namespace, {"fetch_pull": lambda *_: {"head_sha": head}}):
            self.assertTrue(evaluate({}, 7, comment, head))
            mutations = (
                ("body", "CI-GOVERNANCE-APPROVED:" + "b" * 40),
                ("author_association", "MEMBER"),
            )
            for field, value in mutations:
                with self.subTest(field=field):
                    changed = dict(comment)
                    changed[field] = value
                    self.assertFalse(evaluate({}, 7, changed, head))
            for field, value in (("id", 1), ("login", "develata"), ("type", "Bot")):
                with self.subTest(user_field=field):
                    changed = dict(comment)
                    changed["user"] = {**cast(dict[str, object], comment["user"]), field: value}
                    self.assertFalse(evaluate({}, 7, changed, head))
        read_pull = cast(Callable[[dict[str, object]], int], namespace["read_pull_event"])
        self.assertEqual(read_pull({"action": "opened", "pull_request": {"number": 7}}), 7)
        self._controller_failure(namespace, lambda: read_pull({"action": "closed", "pull_request": {"number": 7}}))
        failure = self._record(namespace, conclusion="failure")
        event = {
            "action": "created",
            "issue": {"number": 7, "pull_request": {"url": "x"}},
            "comment": comment,
        }
        actual, calls, publication = self._run_controller_scenario(
            namespace,
            event_name="issue_comment",
            event=event,
            changed_file={"filename": "scripts/check_repo_contracts.py", "status": "modified"},
            initial=failure,
            conclusion="success",
        )
        self.assertEqual(actual, ("success", 41))
        mutations = [call for call in calls if call[0] != "GET"]
        self.assertEqual(mutations, [
            (
                "PATCH",
                "/repos/Develata/ustc-campus-agent/check-runs/41",
                {"conclusion": "success", "status": "completed"},
                200,
            )
        ])
        self.assertTrue(publication["attempted"])

    def test_governance_issue_comment_pr_custody(self) -> None:
        namespace = self._controller()
        read_comment = cast(
            Callable[[dict[str, object]], tuple[int, dict[str, object]]],
            namespace["read_comment_event"],
        )
        comment = {"body": "x"}
        number, actual = read_comment(
            {
                "action": "created",
                "issue": {"number": 7, "pull_request": {"url": "x"}},
                "comment": comment,
            }
        )
        self.assertEqual((number, actual), (7, comment))
        invalid = (
            {"action": "edited", "issue": {"number": 7, "pull_request": {}}, "comment": comment},
            {"action": "created", "issue": {"number": 7}, "comment": comment},
            {"action": "created", "issue": {"number": True, "pull_request": {}}, "comment": comment},
        )
        for event in invalid:
            with self.subTest(event=event):
                self._controller_failure(namespace, lambda event=event: read_comment(event))
        fetch_pull = cast(Callable[[dict[str, str], int], dict[str, object]], namespace["fetch_pull"])
        good = {
            "state": "open",
            "updated_at": "2026-08-26T00:00:00Z",
            "changed_files": 0,
            "head": {"sha": "a" * 40, "repo": {"full_name": "Develata/ustc-campus-agent"}},
            "base": {"ref": "main", "sha": "b" * 40, "repo": {"full_name": "Develata/ustc-campus-agent"}},
            "user": {"login": "author", "id": 2},
        }
        with mock.patch.dict(namespace, {"api_request": lambda *_: good}):
            self.assertEqual(fetch_pull({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7)["head_sha"], "a" * 40)
            for field, value in (("state", "closed"), ("changed_files", True), ("updated_at", "")):
                with self.subTest(field=field):
                    broken = {**good, field: value}
                    with mock.patch.dict(namespace, {"api_request": lambda *_, broken=broken: broken}):
                        self._controller_failure(namespace, lambda: fetch_pull({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7))
        custody_mutations = (
            ("head", "repo", "full_name", "fork/repository"),
            ("base", "repo", "full_name", "fork/repository"),
            ("base", None, "ref", "dev"),
            ("base", None, "sha", "not-a-sha"),
        )
        for section, nested, field, value in custody_mutations:
            with self.subTest(custody=(section, nested, field)):
                broken = json.loads(json.dumps(good))
                target = cast(dict[str, object], broken[section])
                if nested is not None:
                    target = cast(dict[str, object], target[nested])
                target[field] = value
                with mock.patch.dict(namespace, {"api_request": lambda *_, broken=broken: broken}):
                    self._controller_failure(
                        namespace,
                        lambda: fetch_pull(
                            {"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7
                        ),
                    )

    def test_governance_changed_files_pagination_and_renames(self) -> None:
        namespace = self._controller()
        fetch = cast(Callable[[dict[str, str], int, int], list[object]], namespace["fetch_changed_files"])
        validate = cast(Callable[[list[object]], None], namespace["validate_changed_files"])
        pages = iter(
            [
                [{"filename": "README.md", "status": "modified"}],
                [],
            ]
        )
        calls: list[str] = []
        def api(_: object, method: str, path: str, *args: object, **kwargs: object) -> object:
            self.assertEqual(method, "GET")
            calls.append(path)
            return next(pages)
        with mock.patch.dict(namespace, {"api_request": api}):
            records = fetch({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7, 1)
        self.assertEqual(len(calls), 2)
        validate(records)
        validate([{"filename": "new.md", "status": "renamed", "previous_filename": "old.md"}])
        validate([{"filename": "vendor/submodule", "status": "changed"}])
        malformed = (
            [{"filename": "x", "status": "modified", "previous_filename": "old"}],
            [{"filename": "x", "status": "renamed"}],
            [
                {"filename": "x", "status": "renamed", "previous_filename": "old"},
                {"filename": "old", "status": "modified"},
            ],
        )
        for records in malformed:
            with self.subTest(records=records):
                self._controller_failure(namespace, lambda records=records: validate(records))
        pages_3000 = iter(
            [
                [
                    {"filename": f"ordinary/{page * 100 + index}.txt", "status": "modified"}
                    for index in range(100)
                ]
                for page in range(30)
            ]
            + [[]]
        )
        with mock.patch.dict(namespace, {"api_request": lambda *_: next(pages_3000)}):
            exact = fetch({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7, 3000)
        self.assertEqual(len(exact), 3000)
        self._controller_failure(
            namespace,
            lambda: fetch({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7, 3001),
        )
        short_pages = iter(
            [[{"filename": "one", "status": "modified"}], []]
        )
        with mock.patch.dict(namespace, {"api_request": lambda *_: next(short_pages)}):
            self._controller_failure(
                namespace,
                lambda: fetch({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 7, 2),
            )

    def test_governance_head_race_and_replaced_pending_event(self) -> None:
        namespace = self._controller()
        decide = cast(
            Callable[[bool, bool, bool, dict[str, object] | None], dict[str, object]],
            namespace["decide_action"],
        )
        failure = self._record(namespace)
        success = self._record(namespace, conclusion="success")
        self.assertEqual(decide(True, False, False, failure)["conclusion"], "failure")
        self.assertEqual(decide(True, False, False, failure)["record_id"], 41)
        self.assertEqual(decide(True, True, True, failure)["kind"], "patch")
        self.assertEqual(decide(True, False, False, success)["kind"], "none")
        self.assertEqual(decide(True, False, False, success)["record_id"], 41)
        # If main has absorbed the protected bytes while the head is unchanged, the
        # current diff becomes ordinary; upgrading the shared exact-head record is safe.
        ordinary_after_main_advance = decide(False, False, False, failure)
        self.assertEqual(ordinary_after_main_advance["kind"], "patch")
        self.assertEqual(ordinary_after_main_advance["conclusion"], "success")
        self._controller_failure(namespace, lambda: decide(True, True, True, None))
        self._controller_failure(
            namespace,
            lambda: cast(Callable[[dict[str, object]], str], namespace["classify_matching_record"])(
                {**failure, "status": "in_progress", "conclusion": None}
            ),
        )
        observe = cast(
            Callable[[dict[str, str], int], dict[str, object]],
            namespace["observe_change_set"],
        )
        environment = {"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}
        ordinary = [{"filename": "README.md", "status": "modified"}]
        protected_records = [
            {"filename": "scripts/check_repo_contracts.py", "status": "modified"}
        ]
        stable_pull = {
            "base_sha": "b" * 40,
            "changed_files": 1,
            "head_sha": "a" * 40,
            "updated_at": "2026-08-26T00:00:00Z",
        }
        changed_update = {**stable_pull, "updated_at": "2026-08-26T00:01:00Z"}
        with mock.patch.dict(
            namespace,
            {
                "fetch_pull": lambda *_: next(iter_pulls),
                "fetch_changed_files": lambda *_: ordinary,
            },
        ):
            iter_pulls = iter([stable_pull, changed_update])
            self._controller_failure(namespace, lambda: observe(environment, 7))
        changed_base = {**stable_pull, "base_sha": "c" * 40}
        with mock.patch.dict(
            namespace,
            {
                "fetch_pull": lambda *_: next(iter_pulls),
                "fetch_changed_files": lambda *_: ordinary,
            },
        ):
            iter_pulls = iter([stable_pull, changed_base])
            self._controller_failure(namespace, lambda: observe(environment, 7))
        with mock.patch.dict(
            namespace,
            {
                "fetch_pull": lambda *_: next(iter_pulls),
                "fetch_changed_files": lambda *_: next(iter_files),
            },
        ):
            iter_pulls = iter([stable_pull, stable_pull, stable_pull])
            iter_files = iter([ordinary, protected_records])
            self._controller_failure(namespace, lambda: observe(environment, 7))

        initial_observation = {
            "protected": False,
            "pull": {
                "base_sha": "b" * 40,
                "changed_files": 0,
                "head_sha": "a" * 40,
                "updated_at": "2026-08-26T00:00:00Z",
            },
            "records": [],
        }
        drifted_observation = {
            **initial_observation,
            "pull": {
                **cast(dict[str, object], initial_observation["pull"]),
                "base_sha": "c" * 40,
            },
        }
        observations = iter([initial_observation, drifted_observation])
        mutations: list[str] = []
        with mock.patch.dict(
            namespace,
            {
                "read_environment": lambda: {
                    "GITHUB_EVENT_NAME": "pull_request_target",
                    "GITHUB_EVENT_PATH": "x",
                },
                "read_event": lambda *_: {
                    "action": "opened",
                    "pull_request": {"number": 7},
                },
                "fetch_repository": lambda *_: {"id": 1308221459},
                "observe_change_set": lambda *_: next(observations),
                "fetch_check_runs": lambda *_: {"matching": success},
                "post_check_run": lambda *_: mutations.append("post"),
                "patch_check_run": lambda *_: mutations.append("patch"),
            },
        ):
            publication = {"attempted": False}
            self._controller_failure(
                namespace,
                lambda: cast(
                    Callable[[dict[str, bool]], str], namespace["run_controller"]
                )(publication),
            )
        self.assertEqual(mutations, [])

    def test_governance_check_run_inventory_and_duplicates(self) -> None:
        namespace = self._controller()
        partition = cast(
            Callable[[list[dict[str, object]], str, str], dict[str, object]],
            namespace["partition_check_runs"],
        )
        head = "a" * 40
        external = cast(Callable[[int, str], str], namespace["external_identity"])(1308221459, head)
        record = self._record(namespace, external_id=external)
        self.assertIs(partition([record], external, head)["matching"], record)
        for records in (
            [record, {**record, "id": 42}],
            [record, self._record(namespace, record_id=43, external_id="foreign")],
            [self._record(namespace, external_id=external, app_id=1)],
        ):
            with self.subTest(records=records):
                self._controller_failure(namespace, lambda records=records: partition(records, external, head))
        fetch = cast(Callable[[dict[str, str], int, str], dict[str, object]], namespace["fetch_check_runs"])
        pages = iter([
            {"total_count": 1, "check_runs": [record]},
            {"total_count": 1, "check_runs": []},
        ])
        with mock.patch.dict(namespace, {"api_request": lambda *_: next(pages)}):
            self.assertEqual(fetch({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 1308221459, head)["matching"], record)
        for count in (100, 200):
            with self.subTest(exact_page_count=count):
                records = [
                    {
                        **self._record(
                            namespace,
                            record_id=index + 1000,
                            external_id=f"other:{index}",
                        ),
                        "name": "rust",
                    }
                    for index in range(count)
                ]
                payloads = [
                    {"total_count": count, "check_runs": records[offset : offset + 100]}
                    for offset in range(0, count, 100)
                ]
                payloads.append({"total_count": count, "check_runs": []})
                queued = iter(payloads)
                with mock.patch.dict(namespace, {"api_request": lambda *_: next(queued)}):
                    inventory = fetch(
                        {"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"},
                        1308221459,
                        head,
                    )
                self.assertEqual(len(cast(list[object], inventory["records"])), count)
        drift_pages = iter(
            [
                {"total_count": 101, "check_runs": [
                    {
                        **self._record(namespace, record_id=index + 2000, external_id=f"drift:{index}"),
                        "name": "rust",
                    }
                    for index in range(100)
                ]},
                {"total_count": 100, "check_runs": []},
            ]
        )
        with mock.patch.dict(namespace, {"api_request": lambda *_: next(drift_pages)}):
            self._controller_failure(
                namespace,
                lambda: fetch(
                    {"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"},
                    1308221459,
                    head,
                ),
            )

    def test_governance_publication_ambiguity_and_readback(self) -> None:
        namespace = self._controller()
        transport_lost = cast(type[Exception], namespace["TransportLost"])
        with mock.patch.dict(
            namespace,
            {"run_controller": lambda publication: publication.__setitem__("attempted", True) or (_ for _ in ()).throw(transport_lost("lost"))},
        ):
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(cast(Callable[[], int], namespace["main"])(), 1)
            self.assertIn("PUBLICATION_AMBIGUOUS", output.getvalue())
        controller_failure = cast(type[Exception], namespace["ControllerFailure"])
        with mock.patch.dict(
            namespace,
            {"run_controller": lambda publication: publication.__setitem__("attempted", True) or (_ for _ in ()).throw(controller_failure("readback drift"))},
        ):
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(cast(Callable[[], int], namespace["main"])(), 1)
            self.assertIn("PUBLICATION_AMBIGUOUS", output.getvalue())
        for conclusion, expected_status in (("failure", 1), ("success", 0)):
            with self.subTest(terminal_receipt=conclusion), mock.patch.dict(
                namespace,
                {"run_controller": lambda _publication, value=conclusion: (value, 41)},
            ):
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    self.assertEqual(
                        cast(Callable[[], int], namespace["main"])(),
                        expected_status,
                    )
                self.assertEqual(
                    output.getvalue(),
                    f"ci-governance: check 41={conclusion}\n",
                )
        run_controller = cast(
            Callable[[dict[str, bool]], tuple[str, int]],
            namespace["run_controller"],
        )
        head = "a" * 40
        for conclusion, protected in (("failure", True), ("success", False)):
            with self.subTest(noop_readback=conclusion):
                record = self._record(namespace, conclusion=conclusion)
                observation = {
                    "protected": protected,
                    "pull": {
                        "base_sha": "b" * 40,
                        "changed_files": 1,
                        "head_sha": head,
                        "updated_at": "2026-08-26T00:00:00Z",
                    },
                    "records": [{"filename": "README.md", "status": "modified"}],
                }
                verified: list[tuple[object, ...]] = []
                with mock.patch.dict(
                    namespace,
                    {
                        "read_environment": lambda: {
                            "GITHUB_API_URL": "https://api.github.invalid",
                            "GITHUB_EVENT_NAME": "pull_request_target",
                            "GITHUB_EVENT_PATH": "/event.json",
                            "GITHUB_REPOSITORY": "Develata/ustc-campus-agent",
                            "GITHUB_TOKEN": "token",
                        },
                        "read_event": lambda *_: {
                            "action": "opened",
                            "pull_request": {"number": 7},
                        },
                        "fetch_repository": lambda *_: {"id": 1308221459},
                        "observe_change_set": lambda *_: observation,
                        "fetch_check_runs": lambda *_: {"matching": record},
                        "verify_publication": lambda *args: verified.append(args),
                    },
                ):
                    self.assertEqual(
                        run_controller({"attempted": False}),
                        (conclusion, 41),
                    )
                self.assertEqual(len(verified), 1)
                self.assertEqual(
                    verified[0][1:],
                    (1308221459, head, 41, conclusion),
                )
        verify = cast(Callable[..., None], namespace["verify_publication"])
        record = self._record(namespace, conclusion="success")
        responses = iter([{"matching": record}, {**record, "app": {"id": 1, "slug": "foreign"}}])
        with mock.patch.dict(
            namespace,
            {
                "fetch_check_runs": lambda *_: next(responses),
                "api_request": lambda *_: next(responses),
            },
        ):
            self._controller_failure(
                namespace,
                lambda: verify(
                    {"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"},
                    1308221459,
                    "a" * 40,
                    41,
                    "success",
                ),
            )
        post = cast(Callable[..., int], namespace["post_check_run"])
        publication = {"attempted": False}
        captured: list[tuple[str, str, object, int]] = []
        def api(_: object, method: str, path: str, body: object = None, expected_status: int = 200) -> object:
            captured.append((method, path, body, expected_status))
            return {"id": 41}
        with mock.patch.dict(namespace, {"api_request": api, "verify_publication": lambda *_: None}):
            self.assertEqual(post({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}, 1308221459, "a" * 40, "failure", publication), 41)
        self.assertEqual(captured[0][0], "POST")
        self.assertEqual(captured[0][3], 201)
        self.assertTrue(publication["attempted"])

    def test_governance_workflow_authority_inventory(self) -> None:
        self.assertEqual(self._check(), [])
        ci_path = self.root / ".github/workflows/ci.yml"
        ci_baseline = ci_path.read_text(encoding="utf-8")
        bypasses = (
            (
                "permissions:\n  contents: read",
                "permissions: {contents: read, checks: write}",
            ),
            ("permissions:", "permissions: write-all # hidden"),
            ("permissions:", "permissions: &authority"),
            ("permissions:\n  contents: read", "permissions: *authority"),
            (
                "run: python3 scripts/check_repo_contracts.py",
                "run: |\n          python3 scripts/check_repo_contracts.py",
            ),
            (
                "run: python3 scripts/check_repo_contracts.py",
                "run: ${{ github.event.pull_request.title }}",
            ),
            (
                "run: python3 scripts/check_repo_contracts.py",
                "run: python3 -c 'name=\"ci-\"+\"governance\"'",
            ),
            ("  pull_request:", "  pull_request_target:"),
            (
                "    name: rust\n    runs-on: ubuntu-latest\n    timeout-minutes: 20\n    steps:",
                "    name: rust\n    timeout-minutes: 20\n    steps:",
            ),
            (
                "      - name: Repository contract checks\n        run: python3 scripts/check_repo_contracts.py\n",
                "",
            ),
            (
                "      - name: Clippy\n        run: cargo clippy --locked --all-targets --all-features -- -D warnings\n",
                "",
            ),
            (
                "      - name: Format\n        run: cargo fmt --all -- --check\n",
                "      - name: Format\n        run: cargo fmt --all -- --check\n"
                "      - name: Format\n        run: cargo fmt --all -- --check\n",
            ),
            (
                "      - name: Format\n        run: cargo fmt --all -- --check\n"
                "      - name: Clippy\n        run: cargo clippy --locked --all-targets --all-features -- -D warnings\n",
                "      - name: Clippy\n        run: cargo clippy --locked --all-targets --all-features -- -D warnings\n"
                "      - name: Format\n        run: cargo fmt --all -- --check\n",
            ),
            (
                "      - name: Checkout\n"
                "        uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803\n"
                "        with:\n          fetch-depth: 0\n"
                "      - name: Install Python 3.13.5\n"
                "        uses: actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1\n"
                "        with:\n          python-version: 3.13.5\n",
                "      - name: Checkout\n"
                "        uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803\n"
                "      - name: Install Python 3.13.5\n"
                "        uses: actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1\n"
                "        with:\n          fetch-depth: 0\n          python-version: 3.13.5\n",
            ),
        )
        for old, new in bypasses:
            with self.subTest(bypass=old + " -> " + new):
                self.assertEqual(ci_baseline.count(old), 1)
                ci_path.write_text(ci_baseline.replace(old, new, 1), encoding="utf-8")
                issues = self._check()
                self.assertTrue(any("ci.yml" in issue for issue in issues), issues)
                ci_path.write_text(ci_baseline, encoding="utf-8")
        mutations = {
            "foreign-check.yml": "name: other\non: push\npermissions:\n  checks: write\njobs:\n  x:\n    runs-on: ubuntu-24.04\n    steps: []\n",
            "duplicate-name.yml": "name: ci-governance-controller\non: push\npermissions:\n  contents: read\njobs:\n  x:\n    runs-on: ubuntu-24.04\n    steps: []\n",
            "job-permission.yml": "name: other\non: push\npermissions:\n  contents: read\njobs:\n  x:\n    permissions:\n      checks: write\n    uses: x/y/.github/workflows/z.yml@main\n",
        }
        for name, contents in mutations.items():
            with self.subTest(name=name):
                path = self.root / ".github/workflows" / name
                path.write_text(contents, encoding="utf-8")
                issues = self._check()
                self.assertTrue(any(name in issue for issue in issues), issues)
                path.unlink()

    def test_governance_api_schema_and_failure_matrix(self) -> None:
        namespace = self._controller()
        repository = {
            "id": 1308221459,
            "default_branch": "main",
            "owner": {"login": "Develata", "id": 179165876, "type": "User"},
        }
        fetch_repository = cast(Callable[[dict[str, str]], dict[str, object]], namespace["fetch_repository"])
        with mock.patch.dict(namespace, {"api_request": lambda *_: repository}):
            self.assertEqual(fetch_repository({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"})["id"], 1308221459)
        for key, value in (("id", True), ("default_branch", "dev")):
            with self.subTest(key=key):
                broken = {**repository, key: value}
                with mock.patch.dict(namespace, {"api_request": lambda *_, broken=broken: broken}):
                    self._controller_failure(namespace, lambda: fetch_repository({"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}))
        for field, value in (("login", "develata"), ("id", 1), ("type", "Organization")):
            with self.subTest(owner_field=field):
                broken = json.loads(json.dumps(repository))
                cast(dict[str, object], broken["owner"])[field] = value
                with mock.patch.dict(namespace, {"api_request": lambda *_, broken=broken: broken}):
                    self._controller_failure(
                        namespace,
                        lambda: fetch_repository(
                            {"GITHUB_REPOSITORY": "Develata/ustc-campus-agent"}
                        ),
                    )
        validate = cast(Callable[[object], dict[str, object]], namespace["validate_check_run_record"])
        valid = self._record(namespace)
        self.assertEqual(validate(valid), valid)
        for field, value in (("id", True), ("name", None), ("app", {"id": True, "slug": "github-actions"}), ("external_id", 7)):
            with self.subTest(field=field):
                self._controller_failure(namespace, lambda field=field, value=value: validate({**valid, field: value}))
        environment = {
            "GITHUB_TOKEN": "token",
            "GITHUB_API_URL": "https://api.github.invalid",
        }
        requests: list[tuple[str, str]] = []
        class Response:
            status = 201
            def __enter__(self) -> "Response":
                return self
            def __exit__(self, *args: object) -> bool:
                return False
            def read(self) -> bytes:
                return b'{"id":41}'
        def urlopen(request: object, timeout: int) -> Response:
            requests.append((cast(str, getattr(request, "method")), cast(str, getattr(request, "full_url"))))
            self.assertEqual(timeout, 30)
            return Response()
        urllib_module = cast(object, namespace["urllib"])
        with mock.patch.object(getattr(urllib_module, "request"), "urlopen", urlopen):
            response = cast(Callable[..., object], namespace["api_request"])(
                environment,
                "POST",
                "/repos/Develata/ustc-campus-agent/check-runs",
                {"name": "ci-governance"},
                expected_status=201,
            )
        self.assertEqual(response, {"id": 41})
        self.assertEqual(requests, [("POST", "https://api.github.invalid/repos/Develata/ustc-campus-agent/check-runs")])
        class InvalidJsonResponse(Response):
            status = 200
            def read(self) -> bytes:
                return b"not-json"
        with mock.patch.object(
            getattr(urllib_module, "request"),
            "urlopen",
            lambda *_args, **_kwargs: InvalidJsonResponse(),
        ):
            self._controller_failure(
                namespace,
                lambda: cast(Callable[..., object], namespace["api_request"])(
                    environment, "GET", "/repos/Develata/ustc-campus-agent"
                ),
            )
        http_error = getattr(getattr(urllib_module, "error"), "HTTPError")(
            "https://api.github.invalid", 403, "forbidden", {}, None
        )
        with mock.patch.object(
            getattr(urllib_module, "request"),
            "urlopen",
            side_effect=http_error,
        ):
            self._controller_failure(
                namespace,
                lambda: cast(Callable[..., object], namespace["api_request"])(
                    environment, "GET", "/repos/Develata/ustc-campus-agent"
                ),
            )
        transport_lost = cast(type[Exception], namespace["TransportLost"])
        with mock.patch.dict(
            namespace,
            {"run_controller": lambda _publication: (_ for _ in ()).throw(transport_lost("lost"))},
        ):
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(cast(Callable[[], int], namespace["main"])(), 1)
            self.assertIn("transport lost before effect", output.getvalue())
            self.assertNotIn("PUBLICATION_AMBIGUOUS", output.getvalue())


class _M90MutationTestBase(unittest.TestCase):
    FILES: tuple[str, ...] = ()

    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory()
        self.root = Path(self._temp.name)
        for rel in self.FILES:
            dest = self.root / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(REPO_ROOT / rel, dest)
        self._original_root = cast(Path, getattr(checker, "ROOT"))
        setattr(checker, "ROOT", self.root)

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self._original_root)
        self._temp.cleanup()

    def path(self, rel: str) -> Path:
        return self.root / rel

    def rewrite(self, rel: str, old: str, new: str) -> None:
        p = self.path(rel)
        text = p.read_text(encoding="utf-8")
        count = text.count(old)
        assert count == 1, f"mutation target count {count} != 1 in {rel}: {old!r}"
        p.write_text(text.replace(old, new, 1), encoding="utf-8")

    def rewrite_all(self, rel: str, old: str, new: str) -> None:
        p = self.path(rel)
        text = p.read_text(encoding="utf-8")
        assert old in text, f"mutation target absent in {rel}: {old!r}"
        p.write_text(text.replace(old, new), encoding="utf-8")

    def fresh_copy(self, rel: str) -> None:
        shutil.copy2(REPO_ROOT / rel, self.path(rel))

    def assert_rejected(self, issues: list[str], expected: str) -> None:
        self.assertTrue(
            any(expected in i for i in issues),
            f"expected {expected!r} in issues, got {issues}",
        )


class CiTransitionLedgerMutationTests(_M90MutationTestBase):
    FILES = (
        "docs/acceptance/ci-transition-ledger.md",
    )

    def check(self) -> list[str]:
        issues: list[str] = []
        checker.check_ci_transition_ledger(issues)
        return issues

    def test_ledger_passes(self) -> None:
        self.assertEqual(self.check(), [])

    def test_ledger_missing(self) -> None:
        self.path(checker.CI_TRANSITION_LEDGER_PATH).unlink()
        self.assert_rejected(self.check(), "missing")

    def test_ledger_empty(self) -> None:
        self.path(checker.CI_TRANSITION_LEDGER_PATH).write_text("", encoding="utf-8")
        self.assert_rejected(self.check(), "empty")

    def test_ledger_header_drifted(self) -> None:
        self.rewrite(
            checker.CI_TRANSITION_LEDGER_PATH,
            checker.CI_TRANSITION_LEDGER_HEADER,
            "| WRONG | HEADER | | | | |",
        )
        self.assert_rejected(self.check(), "header row missing or drifted")

    def test_ledger_separator_malformed(self) -> None:
        self.rewrite(
            checker.CI_TRANSITION_LEDGER_PATH,
            "|---|---|---|---|---|---|",
            "| === === === === === ===",
        )
        self.assert_rejected(self.check(), "separator malformed")

    def test_ledger_row_count_extra(self) -> None:
        p = self.path(checker.CI_TRANSITION_LEDGER_PATH)
        text = p.read_text(encoding="utf-8")
        lines = text.splitlines(keepends=True)
        last_data = lines[-1]
        p.write_text(text + last_data, encoding="utf-8")
        self.assert_rejected(self.check(), "row count drift")

    def test_ledger_row_count_missing(self) -> None:
        p = self.path(checker.CI_TRANSITION_LEDGER_PATH)
        text = p.read_text(encoding="utf-8")
        lines = text.splitlines(keepends=True)
        p.write_text("".join(lines[:-1]), encoding="utf-8")
        self.assert_rejected(self.check(), "row count drift")

    def test_ledger_row_id_drift_each_row(self) -> None:
        for row_id in checker.CI_TRANSITION_LEDGER_IDS:
            with self.subTest(row_id=row_id):
                self.fresh_copy(checker.CI_TRANSITION_LEDGER_PATH)
                self.rewrite(
                    checker.CI_TRANSITION_LEDGER_PATH,
                    f"`{row_id}`",
                    "`CI-TR-999`",
                )
                self.assert_rejected(self.check(), "ID drift")

    def test_ledger_duplicate_row_id(self) -> None:
        self.rewrite(
            checker.CI_TRANSITION_LEDGER_PATH,
            "`CI-TR-006`",
            "`CI-TR-001`",
        )
        self.assert_rejected(self.check(), "duplicate row ID")

    def test_ledger_empty_cell(self) -> None:
        self.rewrite(
            checker.CI_TRANSITION_LEDGER_PATH,
            "active workflow must remain fail-closed without a mutable whole-file fingerprint",
            "   ",
        )
        self.assert_rejected(self.check(), "is empty")

    def test_ledger_slice_state_drift(self) -> None:
        text = self.path(checker.CI_TRANSITION_LEDGER_PATH).read_text(encoding="utf-8")
        old = "| `active-full-sharded` |\n"
        self.assertIn(old, text)
        self.path(checker.CI_TRANSITION_LEDGER_PATH).write_text(
            text.replace(old, "| `inert-not-active` |\n", 1), encoding="utf-8"
        )
        self.assert_rejected(self.check(), "slice state drift")
        self.fresh_copy(checker.CI_TRANSITION_LEDGER_PATH)
        text = self.path(checker.CI_TRANSITION_LEDGER_PATH).read_text(encoding="utf-8")
        active = "| `guard-required` |\n"
        self.assertIn(active, text)
        self.path(checker.CI_TRANSITION_LEDGER_PATH).write_text(
            text.replace(active, "| `inert-not-active` |\n", 1), encoding="utf-8"
        )
        self.assert_rejected(self.check(), "slice state drift")

    def test_ledger_state_declaration_missing(self) -> None:
        decl = checker.CI_TRANSITION_LEDGER_STATE_DECLARATIONS[0]
        self.rewrite(checker.CI_TRANSITION_LEDGER_PATH, decl, "MUTATED_DECLARATION_REPLACED")
        self.assert_rejected(self.check(), "state declaration missing")
        self.fresh_copy(checker.CI_TRANSITION_LEDGER_PATH)
        second_decl = checker.CI_TRANSITION_LEDGER_STATE_DECLARATIONS[-1]
        self.rewrite(
            checker.CI_TRANSITION_LEDGER_PATH,
            second_decl,
            "MUTATED_SECOND_DECLARATION",
        )
        self.assert_rejected(self.check(), "state declaration missing")

    def test_ledger_row_semantic_drift_each_cell(self) -> None:
        for row_id, expected_substrings in checker.CI_TRANSITION_LEDGER_ROW_CELLS:
            for cell_index, expected_substring in enumerate(expected_substrings, start=1):
                with self.subTest(row_id=row_id, cell=cell_index):
                    self.fresh_copy(checker.CI_TRANSITION_LEDGER_PATH)
                    self._mutate_row_cell(row_id, expected_substring)
                    self.assert_rejected(self.check(), f"row {row_id} cell {cell_index} semantic drift")

    def _mutate_row_cell(self, row_id: str, expected_substring: str) -> None:
        p = self.path(checker.CI_TRANSITION_LEDGER_PATH)
        lines = p.read_text(encoding="utf-8").splitlines(keepends=True)
        row_marker = f"`{row_id}`"
        target_line_index: int | None = None
        for idx, line in enumerate(lines):
            if line.lstrip().startswith(f"| {row_marker} |"):
                target_line_index = idx
                break
        assert target_line_index is not None, f"row {row_id} not found"
        line = lines[target_line_index]
        assert expected_substring in line, f"substring {expected_substring!r} not in row {row_id} line"
        mid = len(expected_substring) // 2
        if mid == 0:
            mid = 1
        mutation = expected_substring[:mid] + "_MUTATED_" + expected_substring[mid:]
        lines[target_line_index] = line.replace(expected_substring, mutation, 1)
        p.write_text("".join(lines), encoding="utf-8")

    def test_ledger_row_cell_count_drift_preserving_id(self) -> None:
        text = self.path(checker.CI_TRANSITION_LEDGER_PATH).read_text(encoding="utf-8")
        old = "| `CI-TR-001` | active workflow must remain fail-closed without a mutable whole-file fingerprint |"
        new = "| `CI-TR-001` |"
        self.assertIn(old, text)
        self.path(checker.CI_TRANSITION_LEDGER_PATH).write_text(
            text.replace(old, new, 1), encoding="utf-8"
        )
        self.assert_rejected(self.check(), "cell count drift")


class CiV2InertFixtureMutationTests(_M90MutationTestBase):
    FILES = (
        "scripts/tests/fixtures/ci-v2.yml",
    )

    def check(self) -> list[str]:
        issues: list[str] = []
        checker.check_ci_v2_inert_fixture(issues)
        return issues

    def test_fixture_passes(self) -> None:
        self.assertEqual(self.check(), [])

    def test_fixture_missing(self) -> None:
        self.path(checker.CI_V2_FIXTURE_PATH).unlink()
        self.assert_rejected(self.check(), "missing")

    def test_fixture_empty(self) -> None:
        self.path(checker.CI_V2_FIXTURE_PATH).write_text("", encoding="utf-8")
        self.assert_rejected(self.check(), "empty")

    def test_fixture_display_only_drift_remains_accepted(self) -> None:
        """F6: no whole-fixture fingerprint; harmless display-only drift is accepted
        while semantic mutations remain rejected."""
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        p.write_text(text + "# harmless display-only drift comment\n", encoding="utf-8")
        self.assertEqual(self.check(), [])
        # Semantic mutations must remain rejected even alongside display-only drift.
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "if: ${{ always() }}", "if: ${{ MUTED() }}")
        self.assert_rejected(self.check(), "missing always() evidence upload condition")

    def test_fixture_missing_pull_request_trigger(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "on:\n  pull_request:", "on:\n  pull_request_MUTED:")
        self.assert_rejected(self.check(), "missing pull_request trigger")

    def test_fixture_missing_push_to_main(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "push:\n    branches:", "push:\n    branches_MUTED:")
        self.assert_rejected(self.check(), "missing push-to-main trigger")

    def test_fixture_missing_rust_job(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "  rust:", "  rust_MUTED:")
        self.assert_rejected(self.check(), "missing rust job")

    def test_fixture_missing_docs_job(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "  docs-and-contracts:", "  docs-and-contracts_MUTED:")
        self.assert_rejected(self.check(), "missing docs-and-contracts job")

    def test_fixture_missing_action_sha(self) -> None:
        sha = checker.CI_V2_REQUIRED_ACTION_SHAS[0]
        self.rewrite_all(checker.CI_V2_FIXTURE_PATH, sha, sha.replace("@", "@0000000000000000000000000000000000000000"))
        self.assert_rejected(self.check(), "missing pinned action SHA")

    def test_fixture_missing_toolchain(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "toolchain: 1.97.1", "toolchain: 1.97.99")
        self.assert_rejected(self.check(), "missing Rust toolchain")

    def test_fixture_missing_rustfmt_clippy(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "components: rustfmt, clippy", "components: rustfmt_MUTED, clippy")
        self.assert_rejected(self.check(), "missing rustfmt/clippy components")

    def test_fixture_missing_python_version(self) -> None:
        self.rewrite(
            checker.CI_V2_FIXTURE_PATH,
            f"python-version: {checker.CI_V2_PYTHON_VERSION}",
            "python-version: 3.13.99",
        )
        self.assert_rejected(self.check(), "missing Python")

    def test_fixture_missing_pyprefix_line(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, checker.CI_V2_PYPREFIX_LINE, "PYTHONPYCACHEPREFIX_MUTED: placeholder")
        self.assert_rejected(self.check(), "missing job-level PYTHONPYCACHEPREFIX")

    def test_fixture_missing_rust_command(self) -> None:
        cmd = checker.CI_V2_REQUIRED_RUST_COMMANDS[0]
        self.rewrite(checker.CI_V2_FIXTURE_PATH, cmd, "cargo MUTED --all -- --check")
        self.assert_rejected(self.check(), "missing Rust command")

    def test_fixture_missing_runner_arg(self) -> None:
        arg = checker.CI_V2_REQUIRED_RUNNER_ARGS[0]
        self.rewrite(checker.CI_V2_FIXTURE_PATH, arg, "python3 scripts/MUTED_shards.py")
        self.assert_rejected(self.check(), "missing runner argument")

    def test_fixture_checker_count_drift(self) -> None:
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        # A commented-out copy is display-only and does not count as executable.
        p.write_text(text + "# python3 scripts/check_repo_contracts.py\n", encoding="utf-8")
        self.assertEqual(self.check(), [])
        # A second executable copy is a real duplicate command and must fail.
        self.fresh_copy(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        p.write_text(
            text + "      - name: Extra check\n        run: python3 scripts/check_repo_contracts.py\n",
            encoding="utf-8",
        )
        self.assert_rejected(self.check(), "checker command count drift: expected 1 actual 2")

    def test_fixture_checker_before_runner_rejected(self) -> None:
        """F5: both commands preserved but the checker before the shard runner
        must be rejected (CI-TR-006 ordering)."""
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        lines = p.read_text(encoding="utf-8").splitlines(keepends=True)
        shard_idx = next(
            i for i, line in enumerate(lines)
            if line.startswith("      - name: Run exact-inventory checker shards")
        )
        checker_idx = next(
            i for i, line in enumerate(lines)
            if line.startswith("      - name: Check repository contracts")
        )
        self.assertLess(shard_idx, checker_idx)

        def block_end(start: int) -> int:
            for j in range(start + 1, len(lines)):
                if lines[j].startswith("      - name: "):
                    return j
            return len(lines)

        shard_end = block_end(shard_idx)
        checker_end = block_end(checker_idx)
        shard_block = lines[shard_idx:shard_end]
        checker_block = lines[checker_idx:checker_end]
        reordered = (
            lines[:shard_idx]
            + checker_block
            + lines[shard_end:checker_idx]
            + shard_block
            + lines[checker_end:]
        )
        p.write_text("".join(reordered), encoding="utf-8")
        self.assert_rejected(self.check(), "ordering drift")

        # A displayed command is not an executable command. Preserve the full
        # command text behind an ``echo`` prefix to prove substring search
        # cannot satisfy CI-TR-006 for either side of the ordering relation.
        for label, command, diagnostic in (
            (
                "checker",
                checker.CI_V2_CHECKER_COMMAND,
                "checker command count drift: expected 1 actual 0",
            ),
            (
                "runner",
                checker.CI_V2_SHARD_RUNNER_COMMAND,
                "shard runner command count drift: expected 1 actual 0",
            ),
        ):
            with self.subTest(display_only=label):
                self.fresh_copy(checker.CI_V2_FIXTURE_PATH)
                self.rewrite(
                    checker.CI_V2_FIXTURE_PATH,
                    command,
                    f"echo {command}",
                )
                self.assert_rejected(self.check(), diagnostic)

    def test_fixture_missing_always_condition(self) -> None:
        self.rewrite(checker.CI_V2_FIXTURE_PATH, "if: ${{ always() }}", "if: ${{ MUTED() }}")
        self.assert_rejected(self.check(), "missing always() evidence upload condition")

    def test_fixture_pyprefix_step_level_env_rejected(self) -> None:
        old = (
            "    timeout-minutes: 40\n"
            "    env:\n"
            "      PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
            "    steps:\n"
            "      - name: Checkout\n"
        )
        new = (
            "    timeout-minutes: 40\n"
            "    steps:\n"
            "      - name: Checkout\n"
            "        env:\n"
            "          PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
        )
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        self.assertIn(old, text)
        p.write_text(text.replace(old, new, 1), encoding="utf-8")
        issues = self.check()
        self.assertTrue(
            any("indent drift" in i or "enclosing env:" in i or "under docs-and-contracts" in i for i in issues),
            f"expected pyprefix placement rejection, got {issues}",
        )

    def test_fixture_pyprefix_workflow_level_env_rejected(self) -> None:
        old = (
            "permissions:\n"
            "  contents: read\n"
            "\n"
            "jobs:\n"
        )
        new = (
            "permissions:\n"
            "  contents: read\n"
            "env:\n"
            "  PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
            "\n"
            "jobs:\n"
        )
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        original_block = (
            "    env:\n"
            "      PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
        )
        text = text.replace(original_block, "", 1)
        self.assertIn(old, text)
        p.write_text(text.replace(old, new, 1), encoding="utf-8")
        issues = self.check()
        self.assertTrue(
            any("indent drift" in i or "enclosing env:" in i or "under docs-and-contracts" in i for i in issues),
            f"expected pyprefix placement rejection, got {issues}",
        )

    def test_fixture_pyprefix_sibling_job_env_rejected(self) -> None:
        old = "  rust:\n    runs-on: ubuntu-latest\n"
        new = (
            "  rust:\n"
            "    runs-on: ubuntu-latest\n"
            "    env:\n"
            "      PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
        )
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        original_block = (
            "    env:\n"
            "      PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
        )
        text = text.replace(original_block, "", 1)
        self.assertIn(old, text)
        p.write_text(text.replace(old, new, 1), encoding="utf-8")
        issues = self.check()
        self.assertTrue(
            any("indent drift" in i or "enclosing env:" in i or "under docs-and-contracts" in i or "occurrence count drift" in i for i in issues),
            f"expected pyprefix placement rejection, got {issues}",
        )

    def test_fixture_pyprefix_duplicate_rejected(self) -> None:
        p = self.path(checker.CI_V2_FIXTURE_PATH)
        text = p.read_text(encoding="utf-8")
        original_line = "      PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n"
        self.assertIn(original_line, text)
        p.write_text(text + "      PYTHONPYCACHEPREFIX: ${{ runner.temp }}/uca-python-cache\n", encoding="utf-8")
        issues = self.check()
        self.assertTrue(
            any("occurrence count drift" in i or "extra PYTHONPYCACHEPREFIX" in i for i in issues),
            f"expected pyprefix duplicate rejection, got {issues}",
        )


class CiV2ActiveWorkflowMutationTests(_M90MutationTestBase):
    FILES = (checker.CAMPAIGN_CI_WORKFLOW_PATH, checker.CI_V2_PR_GATES_PATH)

    def check(self) -> list[str]:
        issues: list[str] = []
        checker.check_ci_v2_active_workflow(issues)
        return issues

    def test_active_workflow_passes(self) -> None:
        self.assertEqual(self.check(), [])

    def test_active_workflow_rejects_job_env_runner_context(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            checker.CI_V2_ACTIVE_PYPREFIX_LINE,
            checker.CI_V2_PYPREFIX_LINE,
        )
        self.assert_rejected(self.check(), "PYTHONPYCACHEPREFIX")

    def test_active_workflow_cannot_cancel_push_runs(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            checker.CI_V2_ACTIVE_CANCEL_POLICY,
            "cancel-in-progress: true",
        )
        self.assert_rejected(self.check(), "cancel-in-progress")

    def test_active_workflow_concurrency_group_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            checker.CI_V2_ACTIVE_CONCURRENCY_GROUP,
            "group: ci-${{ github.ref }}",
        )
        self.assert_rejected(self.check(), "group:")

    def test_active_workflow_push_group_must_use_unique_run_id(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            checker.CI_V2_ACTIVE_CONCURRENCY_GROUP,
            "group: ci-${{ github.workflow }}-"
            "${{ github.event.pull_request.number || github.ref }}",
        )
        self.assert_rejected(self.check(), "group:")

    def test_active_workflow_required_context_name_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            "  contracts:\n    name: docs-and-contracts",
            "  contracts-renamed:\n    name: docs-and-contracts-renamed",
        )
        self.assert_rejected(self.check(), "docs-and-contracts")

    def test_active_workflow_runner_argument_drift_fails_closed(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            "--jobs 4",
            "--jobs 1",
        )
        self.assert_rejected(self.check(), "--jobs 4")

    def test_active_workflow_unpinned_action_fails_closed(self) -> None:
        path = self.path(checker.CAMPAIGN_CI_WORKFLOW_PATH)
        text = path.read_text(encoding="utf-8")
        action = "actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803"
        self.assertEqual(text.count(action), 2)
        path.write_text(text.replace(action, "actions/checkout@v6", 1), encoding="utf-8")
        self.assert_rejected(self.check(), "action inventory drift")

    def test_active_workflow_conditional_rust_job_fails_closed(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            "  rust:\n    name: rust",
            "  rust:\n    name: rust\n    if: false",
        )
        self.assert_rejected(self.check(), "conditional inventory drift")

    def test_active_workflow_display_only_checker_fails_closed(self) -> None:
        self.rewrite(
            checker.CAMPAIGN_CI_WORKFLOW_PATH,
            checker.CI_V2_CHECKER_COMMAND,
            f"echo {checker.CI_V2_CHECKER_COMMAND}",
        )
        self.assert_rejected(self.check(), "checker command count drift")

    def test_active_pr_gate_projection_rejects_serial_discovery(self) -> None:
        self.rewrite(
            checker.CI_V2_PR_GATES_PATH,
            checker.CI_V2_SHARD_RUNNER_COMMAND,
            checker.CAMPAIGN_CI_LEGACY_DISCOVERY_COMMAND,
        )
        self.assert_rejected(self.check(), "PR-gate")


class CheckerTestInventoryMutationTests(_M90MutationTestBase):
    FILES: tuple[str, ...] = ()

    VALID_IDS = [
        "module.TestA.test_a",
        "module.TestA.test_b",
        "module.TestB.test_c",
    ]

    def setUp(self) -> None:
        super().setUp()
        self.write_valid_inventory()
        # Isolate the mutation tests from the real repository's live discovery
        # by binding the live-discovery helper to this class's synthetic IDs.
        live_patcher = mock.patch.object(
            checker, "_live_checker_test_ids", return_value=list(self.VALID_IDS)
        )
        live_patcher.start()
        self.addCleanup(live_patcher.stop)

    def write_valid_inventory(self) -> None:
        data = {
            "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
            "test_ids": list(self.VALID_IDS),
            "expected_count": len(self.VALID_IDS),
        }
        inv_path = self.path(checker.CHECKER_TEST_INVENTORY_PATH)
        inv_path.parent.mkdir(parents=True, exist_ok=True)
        inv_path.write_text(
            json.dumps(data, indent=2) + "\n", encoding="utf-8"
        )

    def write_inventory_raw(self, content: str) -> None:
        self.path(checker.CHECKER_TEST_INVENTORY_PATH).write_text(content, encoding="utf-8")

    def check(self) -> list[str]:
        issues: list[str] = []
        checker.check_checker_test_inventory(issues)
        return issues

    def test_inventory_passes(self) -> None:
        self.assertEqual(self.check(), [])

    def test_inventory_missing(self) -> None:
        self.path(checker.CHECKER_TEST_INVENTORY_PATH).unlink()
        self.assert_rejected(self.check(), "missing")

    def test_inventory_empty(self) -> None:
        self.write_inventory_raw("")
        self.assert_rejected(self.check(), "empty")

    def test_inventory_invalid_json(self) -> None:
        self.write_inventory_raw("{not json")
        self.assert_rejected(self.check(), "invalid JSON")

    def test_inventory_not_object(self) -> None:
        self.write_inventory_raw("[1, 2, 3]")
        self.assert_rejected(self.check(), "must be an object")

    def test_inventory_wrong_schema(self) -> None:
        self.write_inventory_raw(
            json.dumps({
                "schema_version": "wrong/v9",
                "test_ids": list(self.VALID_IDS),
                "expected_count": len(self.VALID_IDS),
            })
        )
        self.assert_rejected(self.check(), "schema_version drift")

    def test_inventory_test_ids_not_list(self) -> None:
        self.write_inventory_raw(
            json.dumps({
                "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                "test_ids": "not_a_list",
                "expected_count": 0,
            })
        )
        self.assert_rejected(self.check(), "test_ids must be a list")

    def test_inventory_duplicates(self) -> None:
        dup_ids = list(self.VALID_IDS) + [self.VALID_IDS[0]]
        self.write_inventory_raw(
            json.dumps({
                "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                "test_ids": dup_ids,
                "expected_count": len(dup_ids),
            })
        )
        self.assert_rejected(self.check(), "contains duplicates")

    def test_inventory_unsorted(self) -> None:
        unsorted_ids = list(reversed(self.VALID_IDS))
        self.write_inventory_raw(
            json.dumps({
                "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                "test_ids": unsorted_ids,
                "expected_count": len(unsorted_ids),
            })
        )
        self.assert_rejected(self.check(), "must be sorted")

    def test_inventory_count_mismatch(self) -> None:
        self.write_inventory_raw(
            json.dumps({
                "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                "test_ids": list(self.VALID_IDS),
                "expected_count": len(self.VALID_IDS) + 1,
            })
        )
        self.assert_rejected(self.check(), "expected_count drift")

    def test_inventory_failed_test_id(self) -> None:
        ids_with_failed = sorted(self.VALID_IDS + ["unittest.loader._FailedTest.test_bad"])
        self.write_inventory_raw(
            json.dumps({
                "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                "test_ids": ids_with_failed,
                "expected_count": len(ids_with_failed),
            })
        )
        self.assert_rejected(self.check(), "_FailedTest ID")

    def test_inventory_malformed_entries_fail_closed_without_traceback(self) -> None:
        """F1: non-string/empty/unhashable entries must yield an issue, never a TypeError."""
        malformed_batches = [
            # Unhashable entry first: set()/sorted() below would raise TypeError
            # if the checker did not return after recording the typed issue.
            [{"unhashable": True}, *self.VALID_IDS],
            [1, *self.VALID_IDS],
            [None, *self.VALID_IDS],
            [["nested"], *self.VALID_IDS],
            ["", *self.VALID_IDS],
        ]
        for batch in malformed_batches:
            with self.subTest(batch=batch):
                self.write_inventory_raw(
                    json.dumps({
                        "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                        "test_ids": batch,
                        "expected_count": len(batch),
                    })
                )
                issues = self.check()
                self.assert_rejected(issues, "non-string/empty entry")

    def test_inventory_stale_against_live_discovery_fails_closed(self) -> None:
        """F4: a structurally valid but stale inventory must be rejected."""
        # Inventory claims an ID live discovery does not produce.
        stale_extra = sorted(self.VALID_IDS + ["module.TestStale.test_stale"])
        self.write_inventory_raw(
            json.dumps({
                "schema_version": checker.CHECKER_TEST_INVENTORY_SCHEMA_VERSION,
                "test_ids": stale_extra,
                "expected_count": len(stale_extra),
            })
        )
        issues = self.check()
        self.assertTrue(
            any(
                "inventory/live discovery drift" in i and "missing_from_live=['module.TestStale.test_stale']" in i
                for i in issues
            ),
            f"expected stale-inventory rejection, got {issues}",
        )

        # Live discovery produces an ID the inventory does not carry.
        self.write_valid_inventory()
        with mock.patch.object(
            checker, "_live_checker_test_ids", return_value=self.VALID_IDS + ["module.TestLive.test_live"]
        ):
            issues = self.check()
        self.assertTrue(
            any(
                "inventory/live discovery drift" in i and "unexpected_in_live=['module.TestLive.test_live']" in i
                for i in issues
            ),
            f"expected unexpected-live rejection, got {issues}",
        )

        # Discovery/import failure must become a typed issue, not a traceback.
        with mock.patch.object(
            checker,
            "_live_checker_test_ids",
            side_effect=ValueError("discovery produced _FailedTest: unittest.loader._FailedTest.test_bad"),
        ):
            issues = self.check()
        self.assertTrue(
            any(
                "live discovery failed" in i and "_FailedTest" in i
                for i in issues
            ),
            f"expected discovery-failure issue, got {issues}",
        )


class RunCheckerShardsRunnerMutationTests(_M90MutationTestBase):
    FILES = (
        "scripts/run_checker_shards.py",
    )

    def check(self) -> list[str]:
        issues: list[str] = []
        checker.check_run_checker_shards_runner(issues)
        return issues

    def test_runner_passes(self) -> None:
        self.assertEqual(self.check(), [])

    def test_runner_missing(self) -> None:
        self.path(checker.RUN_CHECKER_SHARDS_PATH).unlink()
        self.assert_rejected(self.check(), "missing")

    def test_runner_empty(self) -> None:
        self.path(checker.RUN_CHECKER_SHARDS_PATH).write_text("", encoding="utf-8")
        self.assert_rejected(self.check(), "empty")

    def test_runner_missing_each_substring(self) -> None:
        for substring in checker.RUN_CHECKER_SHARDS_REQUIRED_SUBSTRINGS:
            with self.subTest(substring=substring):
                self.fresh_copy(checker.RUN_CHECKER_SHARDS_PATH)
                self.rewrite_all(
                    checker.RUN_CHECKER_SHARDS_PATH,
                    substring,
                    "X" * len(substring),
                )
                self.assert_rejected(self.check(), "missing required substring")
class PlatformRequestContextContractTests(unittest.TestCase):
    """Mutation-proves the dedicated bounded M00-B3 checker."""

    CONTRACT_REL = "docs/contracts/platform-request-context.md"

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        shutil.copytree(REPO_ROOT / "docs", self.root / "docs")
        shutil.copytree(
            REPO_ROOT / "crates/platform-core",
            self.root / "crates/platform-core",
            ignore=shutil.ignore_patterns("target"),
        )
        self.original_root = cast(Path, getattr(checker, "ROOT"))
        self.original_key_files = list(checker.KEY_FILES)
        setattr(checker, "ROOT", self.root)
        self.assertEqual(self.check_request_context(), [])

    def tearDown(self) -> None:
        setattr(checker, "ROOT", self.original_root)
        checker.KEY_FILES[:] = self.original_key_files
        self.temporary_directory.cleanup()

    def check_request_context(self) -> list[str]:
        issues: list[str] = []
        checker.check_platform_request_context(issues)
        return issues

    def test_request_context_contract_missing_fails_closed(self) -> None:
        (self.root / self.CONTRACT_REL).unlink()
        issues = self.check_request_context()
        self.assertIn(
            f"platform request context required carrier missing: {self.CONTRACT_REL}",
            issues,
        )

    def test_request_context_contract_empty_fails_closed(self) -> None:
        (self.root / self.CONTRACT_REL).write_text(" \n", encoding="utf-8")
        issues = self.check_request_context()
        self.assertIn(
            f"platform request context required carrier empty: {self.CONTRACT_REL}",
            issues,
        )

    def test_request_context_contract_unregistered_fails_closed(self) -> None:
        checker.KEY_FILES.remove(self.CONTRACT_REL)
        issues = self.check_request_context()
        self.assertIn(
            f"platform request context contract unregistered: {self.CONTRACT_REL}",
            issues,
        )

    def test_request_context_module_source_registration_drift_fails_closed(self) -> None:
        path = self.root / "crates/platform-core/src/lib.rs"
        text = path.read_text(encoding="utf-8")
        path.write_text(text.replace("pub mod request_context;\n", "", 1), encoding="utf-8")
        issues = self.check_request_context()
        self.assertIn(
            "platform request context module declaration must appear exactly once",
            issues,
        )

    def test_request_context_identity_six_kind_drift_fails_closed(self) -> None:
        path = self.root / "crates/platform-core/src/identity.rs"
        with path.open("a", encoding="utf-8") as handle:
            handle.write("\npub struct CausationId;\n")
        issues = self.check_request_context()
        self.assertIn(
            "platform request context widened the six-kind platform identity owner",
            issues,
        )

    def test_request_context_exact_test_inventory_drift_fails_closed(self) -> None:
        path = self.root / "crates/platform-core/tests/platform_request_context.rs"
        text = path.read_text(encoding="utf-8")
        path.write_text(
            text.replace(
                "fn request_context_forged_permission_tuple_rejected(",
                "fn request_context_forged_permission_tuple_rejected_mutated(",
                1,
            ),
            encoding="utf-8",
        )
        issues = self.check_request_context()
        self.assertTrue(
            any(
                issue.startswith(
                    "platform request context exact 64-test inventory drift"
                )
                for issue in issues
            ),
            issues,
        )
if __name__ == "__main__":
    unittest.main()
