from __future__ import annotations

import importlib.util
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
RUNNER_PATH = REPO_ROOT / "scripts/run_checker_shards.py"

SPEC = importlib.util.spec_from_file_location("run_checker_shards", RUNNER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {RUNNER_PATH}")
runner = importlib.util.module_from_spec(SPEC)
sys.modules["run_checker_shards"] = runner
SPEC.loader.exec_module(runner)

PASS_TEST = (
    "import unittest\n"
    "class TestPass(unittest.TestCase):\n"
    "    def test_pass(self) -> None:\n"
    "        pass\n"
)

FAIL_TEST = (
    "import unittest\n"
    "class TestFail(unittest.TestCase):\n"
    "    def test_fail(self) -> None:\n"
    "        self.assertTrue(False, 'synthetic failure')\n"
)

ERROR_TEST = (
    "import unittest\n"
    "class TestError(unittest.TestCase):\n"
    "    def test_error(self) -> None:\n"
    "        raise RuntimeError('synthetic error')\n"
)

SKIP_TEST = (
    "import unittest\n"
    "class TestSkip(unittest.TestCase):\n"
    "    @unittest.skip('synthetic skip')\n"
    "    def test_skip(self) -> None:\n"
    "        pass\n"
)

EXPECTED_FAILURE_TEST = (
    "import unittest\n"
    "class TestExpectedFailure(unittest.TestCase):\n"
    "    @unittest.expectedFailure\n"
    "    def test_expected_failure(self) -> None:\n"
    "        self.assertTrue(False, 'synthetic expected failure')\n"
)

UNEXPECTED_SUCCESS_TEST = (
    "import unittest\n"
    "class TestUnexpectedSuccess(unittest.TestCase):\n"
    "    @unittest.expectedFailure\n"
    "    def test_unexpected_success(self) -> None:\n"
    "        self.assertTrue(True)\n"
)

SLOW_TEST = (
    "import time\n"
    "import unittest\n"
    "class TestSlow(unittest.TestCase):\n"
    "    def test_slow(self) -> None:\n"
    "        time.sleep(3)\n"
)

CRASH_TEST = (
    "import os\n"
    "import unittest\n"
    "class TestCrash(unittest.TestCase):\n"
    "    def test_crash(self) -> None:\n"
    "        os._exit(1)\n"
)

BAD_IMPORT_TEST = (
    "import nonexistent_module_xyz\n"
)


class RunnerTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        self.pycache_temp = tempfile.TemporaryDirectory()
        self.pycache = str(Path(self.pycache_temp.name) / "pyprefix")
        Path(self.pycache).mkdir()
        self.evidence = Path(self.pycache_temp.name) / "evidence"
        scripts = self.root / "scripts"
        tests = scripts / "tests"
        tests.mkdir(parents=True)
        shutil.copy2(RUNNER_PATH, scripts / "run_checker_shards.py")
        (scripts / "check_repo_contracts.py").write_text("# minimal\n", encoding="utf-8")
        subprocess.run(
            ["git", "init", "-b", "main"],
            cwd=self.root,
            capture_output=True,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.email", "t@t"],
            cwd=self.root,
            capture_output=True,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "t"],
            cwd=self.root,
            capture_output=True,
            check=True,
        )

    def tearDown(self) -> None:
        self.tempdir.cleanup()
        self.pycache_temp.cleanup()

    def write_test(self, name: str, content: str) -> Path:
        path = self.root / "scripts" / "tests" / name
        path.write_text(content, encoding="utf-8")
        return path

    def write_inventory(
        self,
        test_ids: list[str],
        schema_version: str = "checker-test-inventory/v1",
        expected_count: int | None = None,
        raw: str | None = None,
    ) -> Path:
        path = self.root / "scripts" / "checker_test_inventory.json"
        if raw is not None:
            path.write_text(raw, encoding="utf-8")
            return path
        if expected_count is None:
            expected_count = len(test_ids)
        data = {
            "schema_version": schema_version,
            "test_ids": test_ids,
            "expected_count": expected_count,
        }
        path.write_text(
            json.dumps(data, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        return path

    def git_commit_all(self) -> None:
        subprocess.run(
            ["git", "add", "."],
            cwd=self.root,
            capture_output=True,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-m", "init"],
            cwd=self.root,
            capture_output=True,
            check=True,
        )

    def _purge_stale_test_modules(self) -> None:
        stale = [
            name
            for name in list(sys.modules)
            if name.startswith("test_") and "." not in name
        ]
        for name in stale:
            del sys.modules[name]
        sys.path_importer_cache.clear()

    def discover_ids(self) -> list[str]:
        self._purge_stale_test_modules()
        loader = unittest.TestLoader()
        suite = loader.discover(
            start_dir=str(self.root / "scripts" / "tests"),
            pattern="test_*.py",
        )
        ids: set[str] = set()
        stack: list[Any] = [suite]
        while stack:
            item = stack.pop()
            if isinstance(item, unittest.TestCase):
                ids.add(item.id())
            elif isinstance(item, unittest.TestSuite):
                stack.extend(item)
        return sorted(ids)

    def run_runner(
        self,
        *extra_args: str,
        env_extra: dict[str, str] | None = None,
        timeout: int = 120,
    ) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        if env_extra:
            env.update(env_extra)
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "4",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
            *extra_args,
        ]
        return subprocess.run(
            cmd,
            capture_output=True,
            env=env,
            text=True,
            timeout=timeout,
        )

    def read_summary(self) -> dict[str, Any]:
        return json.loads(
            (self.evidence / "summary.json").read_text(encoding="utf-8")
        )

    def read_plan(self) -> dict[str, Any]:
        return json.loads(
            (self.evidence / "plan" / "plan.json").read_text(encoding="utf-8")
        )

    def read_shard_report(self, shard_id: int) -> dict[str, Any]:
        return json.loads(
            (self.evidence / f"shard-{shard_id}" / "report.json").read_text(
                encoding="utf-8"
            )
        )

    def setup_passing_suite(self, count: int = 3) -> list[str]:
        for i in range(count):
            self.write_test(
                f"test_pass_{i}.py",
                f"import unittest\nclass TestPass{i}(unittest.TestCase):\n"
                f"    def test_pass(self) -> None:\n        pass\n",
            )
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        return ids


class DiscoveryAndPartitionTests(RunnerTestBase):
    def test_01_deterministic_discovery_sorted_ids(self) -> None:
        self.write_test("test_zebra.py", PASS_TEST)
        self.write_test("test_alpha.py", PASS_TEST)
        self.write_test("test_middle.py", PASS_TEST)
        ids = self.discover_ids()
        self.assertEqual(ids, sorted(ids))
        self.assertTrue(len(ids) >= 3)
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = self.read_plan()
        self.assertEqual(plan["expected_test_ids"], ids)

    def test_02_deterministic_partition_exact_union(self) -> None:
        ids = self.setup_passing_suite(8)
        result = self.run_runner("--jobs", "3")
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = self.read_plan()
        shards = plan["shards"]
        self.assertGreater(len(shards), 0)
        for shard in shards:
            self.assertGreater(shard["shard_count"], 0)
        union = sorted(tid for shard in shards for tid in shard["test_ids"])
        self.assertEqual(union, ids)
        self.assertEqual(len(union), len(set(union)))


class InventoryValidationTests(RunnerTestBase):
    def test_03_inventory_missing_id_fails(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        ids = self.discover_ids()
        augmented = sorted(ids + ["test_nonexistent.SomeClass.test_method"])
        self.write_inventory(augmented)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("inventory/live discovery bidirectional drift", result.stderr)

    def test_04_inventory_unexpected_id_fails(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        self.write_test("test_pass_1.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids[:-1])
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("inventory/live discovery bidirectional drift", result.stderr)

    def test_05_duplicate_inventory_id_fails(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids + ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicates", result.stderr)

    def test_19_unsorted_inventory_fails(self) -> None:
        self.write_test("test_zebra.py", PASS_TEST)
        self.write_test("test_alpha.py", PASS_TEST)
        ids = self.discover_ids()
        unsorted_ids = list(reversed(ids))
        self.write_inventory(unsorted_ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("sorted", result.stderr)

    def test_20_expected_count_mismatch_fails(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids, expected_count=len(ids) + 1)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("expected_count drift", result.stderr)

    def test_21_malformed_inventory_not_json_fails(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        self.discover_ids()
        self.write_inventory([], raw="{not json")
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not valid JSON", result.stderr)

    def test_21_malformed_inventory_wrong_schema(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids, schema_version="wrong/v9")
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("schema_version drift", result.stderr)

    def test_21_malformed_inventory_not_object(self) -> None:
        self.write_test("test_pass_0.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory([], raw='[1, 2, 3]')
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must be an object", result.stderr)


class ChildExecutionTests(RunnerTestBase):
    def test_06_zero_tests_fails(self) -> None:
        self.write_inventory([])
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("discovery", result.stderr)

    def test_07_failed_test_id_fails_before_execution(self) -> None:
        self.write_test("test_bad.py", BAD_IMPORT_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)

    def test_08_skipped_test_fails(self) -> None:
        self.write_test("test_skip.py", SKIP_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")
        self.assertIn("skipped", summary["outcome_counts"])

    def test_09_expected_failure_fails(self) -> None:
        self.write_test("test_xfail.py", EXPECTED_FAILURE_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")
        self.assertIn("expected_failure", summary["outcome_counts"])

    def test_10_unexpected_success_fails(self) -> None:
        self.write_test("test_xsuccess.py", UNEXPECTED_SUCCESS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")
        self.assertIn("unexpected_success", summary["outcome_counts"])

    def test_11_assertion_failure_fails_with_evidence(self) -> None:
        self.write_test("test_fail.py", FAIL_TEST)
        self.write_test("test_error.py", ERROR_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")
        counts = summary["outcome_counts"]
        self.assertGreater(counts.get("failed", 0) + counts.get("error", 0), 0)
        for shard_id_str in summary["per_shard_evidence"]:
            shard_id = int(shard_id_str)
            report = self.read_shard_report(shard_id)
            for test_id, outcome in report["outcomes"].items():
                if outcome in ("failed", "error"):
                    self.assertIn(test_id, report.get("details", {}))


class ProcessIsolationTests(RunnerTestBase):
    def test_12_child_crash_fails(self) -> None:
        self.write_test("test_crash.py", CRASH_TEST)
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner("--jobs", "1")
        self.assertNotEqual(result.returncode, 0)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")

    def test_13_timeout_terminates_child_fails(self) -> None:
        self.write_test("test_slow.py", SLOW_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "1",
            "--timeout-seconds", "2",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        result = subprocess.run(
            cmd,
            capture_output=True,
            env=env,
            text=True,
            timeout=60,
        )
        self.assertNotEqual(result.returncode, 0)

    def test_14_stale_evidence_dir_rejected(self) -> None:
        self.evidence.mkdir(parents=True)
        (self.evidence / "stale.txt").write_text("stale", encoding="utf-8")
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not empty", result.stderr)

    def test_36_sigterm_ignoring_group_descendant_killed(self) -> None:
        """F3: cleanup must kill a same-group descendant that ignores SIGTERM.

        The leader exits on SIGTERM; without group-wide re-check and SIGKILL
        the descendant would survive. Only the exact child/process-group
        identities created by this test are created and signalled.
        """
        pid_file = Path(self.pycache_temp.name) / "f3_descendant.pid"
        descendant_code = (
            "import signal\n"
            "import time\n"
            "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
            "time.sleep(300)\n"
        )
        leader_code = (
            "import subprocess\n"
            "import sys\n"
            "import time\n"
            "descendant = subprocess.Popen([sys.executable, '-c', sys.argv[1]])\n"
            "with open(sys.argv[2], 'w') as handle:\n"
            "    handle.write(str(descendant.pid))\n"
            "time.sleep(300)\n"
        )
        leader = subprocess.Popen(
            [
                sys.executable,
                "-c",
                leader_code,
                descendant_code,
                str(pid_file),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        descendant_pid: int | None = None
        try:
            deadline = time.monotonic() + 15
            while not pid_file.is_file():
                if leader.poll() is not None:
                    self.fail("leader exited before publishing descendant pid")
                if time.monotonic() > deadline:
                    self.fail("descendant pid file was not written in time")
                time.sleep(0.05)
            descendant_pid = int(pid_file.read_text(encoding="utf-8").strip())
            leader_pgid = os.getpgid(leader.pid)
            self.assertEqual(leader_pgid, leader.pid, "leader must own its group")
            self.assertEqual(
                os.getpgid(descendant_pid),
                leader_pgid,
                "descendant must share the leader's process group",
            )
            self.assertTrue(self._pid_is_live(descendant_pid))
            runner._terminate_process_group(leader)
            self.assertIsNotNone(leader.poll(), "leader must be reaped by cleanup")
            self.assertFalse(
                self._pid_is_live(descendant_pid),
                "SIGTERM-ignoring group descendant must not survive cleanup",
            )
        finally:
            if leader.poll() is None:
                leader.kill()
                leader.wait()
            if descendant_pid is not None and self._pid_is_live(descendant_pid):
                try:
                    os.kill(descendant_pid, signal.SIGKILL)
                except OSError:
                    pass

    @staticmethod
    def _pid_is_live(pid: int) -> bool:
        """True when ``pid`` exists in a non-zombie state (Linux /proc aware)."""
        stat_path = Path("/proc") / str(pid) / "stat"
        try:
            data = stat_path.read_bytes()
        except OSError:
            try:
                os.kill(pid, 0)
            except OSError:
                return False
            return True
        close = data.rfind(b")")
        if close < 0:
            return True
        fields = data[close + 2:].split()
        if not fields:
            return True
        return fields[0] != b"Z"


class FanInTests(RunnerTestBase):
    def test_15_report_tampering_rejected(self) -> None:
        ids = self.setup_passing_suite(3)
        result = self.run_runner("--jobs", "1")
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = self.read_plan()
        shard_ids = [s["shard_id"] for s in plan["shards"]]
        target_shard = shard_ids[0]
        report_path = self.evidence / f"shard-{target_shard}" / "report.json"
        report = json.loads(report_path.read_text(encoding="utf-8"))
        first_id = next(iter(report["outcomes"]))
        report["outcomes"][first_id] = "failed"
        report_path.write_text(
            json.dumps(report, indent=2) + "\n", encoding="utf-8"
        )
        child_results = [
            {
                "shard_id": sid,
                "exit_code": 0,
                "report_path": str(
                    self.evidence / f"shard-{sid}" / "report.json"
                ),
                "stdout_path": str(
                    self.evidence / f"shard-{sid}" / "stdout.log"
                ),
                "stderr_path": str(
                    self.evidence / f"shard-{sid}" / "stderr.log"
                ),
                "timed_out": False,
            }
            for sid in shard_ids
        ]
        _, errors = runner._verify_reports(plan, child_results)
        self.assertTrue(
            any("non-pass" in e or "drift" in e for e in errors),
            f"expected tampering rejection, got {errors}",
        )

    def test_17_evidence_carries_exact_fields(self) -> None:
        self.write_test(
            "test_buffered_output.py",
            "import sys\n"
            "import unittest\n"
            "class TestBufferedOutput(unittest.TestCase):\n"
            "    def test_pass(self) -> None:\n"
            "        sys.stdout.reconfigure(line_buffering=False, write_through=False)\n"
            "        sys.stdout.write('buffered-child-output-marker\\n')\n",
        )
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        result = self.run_runner()
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = self.read_plan()
        self.assertEqual(plan["schema_version"], runner.SCHEMA_VERSION)
        self.assertIn("argv", plan)
        self.assertIn("started_at_epoch", plan)
        repo_fields = plan["repository"]
        self.assertIn("branch", repo_fields)
        self.assertIn("head", repo_fields)
        self.assertIn("head_tree", repo_fields)
        self.assertIn("porcelain_status_digest", repo_fields)
        env_fields = plan["environment"]
        self.assertIn("python_version", env_fields)
        self.assertIn("python_executable", env_fields)
        self.assertIn("platform", env_fields)
        self.assertIn("image_os", env_fields)
        self.assertIn("image_version", env_fields)
        self.assertIn("pycache_prefix", env_fields)
        self.assertIn("runner_image_identity_available", env_fields)
        cfg = plan["configuration"]
        self.assertIn("jobs", cfg)
        self.assertIn("timeout_seconds", cfg)
        self.assertIn("require_clean", cfg)
        self.assertIn("require_runner_image_identity", cfg)
        self.assertIn("source_digests", plan)
        self.assertIn("aggregate_framed_input_digest", plan)
        self.assertIn("inventory_digest", plan)
        self.assertEqual(plan["expected_test_ids"], ids)
        self.assertEqual(plan["expected_count"], len(ids))
        self.assertGreater(plan["actual_shard_count"], 0)
        self.assertIn("plan_digest", plan)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "passed")
        self.assertIn("wall_seconds", summary)
        self.assertIn("outcome_counts", summary)
        self.assertIn("per_shard_evidence", summary)
        self.assertIn("plan_digest", summary)
        stdout_payload = b""
        for shard in plan["shards"]:
            shard_id = shard["shard_id"]
            stdout_path = self.evidence / f"shard-{shard_id}" / "stdout.log"
            report = self.read_shard_report(shard_id)
            stdout_payload += stdout_path.read_bytes()
            self.assertEqual(
                report["stdout_log_digest"],
                runner._sha256_file(stdout_path),
            )
        self.assertIn(b"buffered-child-output-marker", stdout_payload)

    def test_18_atomic_write_failure_no_pass(self) -> None:
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        self.evidence.mkdir(parents=True)
        os.chmod(self.evidence, 0o555)
        try:
            result = self.run_runner()
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((self.evidence / "summary.json").exists())
        finally:
            os.chmod(self.evidence, 0o755)

    def test_22_concurrent_mutation_fails(self) -> None:
        self.write_test("test_slow.py", SLOW_TEST)
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "1",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            text=True,
        )
        plan_path = self.evidence / "plan" / "plan.json"
        deadline = time.monotonic() + 15
        while not plan_path.is_file():
            if time.monotonic() > deadline:
                proc.kill()
                self.fail("plan.json was not written in time")
            time.sleep(0.1)
        time.sleep(0.3)
        checker_path = self.root / "scripts" / "check_repo_contracts.py"
        checker_path.write_text("# mutated\n", encoding="utf-8")
        stdout, stderr = proc.communicate(timeout=60)
        self.assertNotEqual(proc.returncode, 0, stderr)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")
        self.assertTrue(
            any("post-launch source digest drift" in e for e in summary["errors"]),
            f"expected post-launch drift, got {summary['errors']}",
        )


class CLIBoundaryTests(RunnerTestBase):
    def test_16_require_clean_rejects_dirty(self) -> None:
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        (self.root / "scripts" / "check_repo_contracts.py").write_text(
            "# dirty change\n", encoding="utf-8"
        )
        result = self.run_runner("--require-clean")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("--require-clean", result.stderr)

        evidence_f10_parent = Path(self.pycache_temp.name) / "evidence_f10_parent"
        env_f10 = os.environ.copy()
        env_f10["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd_typo = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "4",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(evidence_f10_parent),
            "--require-cleann",
        ]
        result_typo = subprocess.run(cmd_typo, capture_output=True, env=env_f10, text=True, timeout=60)
        self.assertNotEqual(result_typo.returncode, 0, f"parent typo must fail: {result_typo.stderr}")
        self.assertFalse(
            (evidence_f10_parent / "plan" / "plan.json").is_file(),
            "no plan must be written on parent typo",
        )
        self.assertFalse(
            (evidence_f10_parent / "summary.json").is_file(),
            "no summary must be written on parent typo",
        )
        self.assertNotIn("PASS", result_typo.stdout)

        dummy_report = Path(self.pycache_temp.name) / "dummy_report.json"
        cmd_child_unknown = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--__child-shard", "0",
            "--__plan-path", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--__report-path", str(dummy_report),
            "--unknown-child-extra",
        ]
        result_child = subprocess.run(cmd_child_unknown, capture_output=True, env=env_f10, text=True, timeout=60)
        self.assertNotEqual(result_child.returncode, 0, f"child unknown arg must fail: {result_child.stderr}")
        self.assertFalse(dummy_report.is_file(), "no report must be written on unknown child arg")

    def test_23_missing_runner_image_identity_fails(self) -> None:
        ids = self.setup_passing_suite(2)
        env = os.environ.copy()
        env.pop("ImageOS", None)
        env.pop("ImageVersion", None)
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "4",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
            "--require-runner-image-identity",
        ]
        result = subprocess.run(
            cmd,
            capture_output=True,
            env=env,
            text=True,
            timeout=120,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("ImageOS", result.stderr)

    def test_24_pycache_prefix_validation(self) -> None:
        ids = self.setup_passing_suite(2)
        env = os.environ.copy()
        env.pop("ImageOS", None)
        env.pop("ImageVersion", None)
        env.pop("PYTHONPYCACHEPREFIX", None)
        cmd_base = [
            sys.executable,
            "-B",
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "4",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        env_missing = env.copy()
        env_missing.pop("PYTHONPYCACHEPREFIX", None)
        r1 = subprocess.run(
            cmd_base,
            capture_output=True,
            env=env_missing,
            text=True,
            timeout=60,
        )
        self.assertNotEqual(r1.returncode, 0)
        self.assertIn("PYTHONPYCACHEPREFIX", r1.stderr)

        env_relative = env.copy()
        env_relative["PYTHONPYCACHEPREFIX"] = "relative/path"
        ev2 = Path(self.pycache_temp.name) / "ev_relative"
        r2 = subprocess.run(
            [*cmd_base[:-1], str(ev2)],
            capture_output=True,
            env=env_relative,
            text=True,
            timeout=60,
        )
        self.assertNotEqual(r2.returncode, 0)
        self.assertIn("absolute", r2.stderr)

        env_repo_local = env.copy()
        env_repo_local["PYTHONPYCACHEPREFIX"] = str(self.root / "inner_pycache")
        ev3 = Path(self.pycache_temp.name) / "ev_repo_local"
        r3 = subprocess.run(
            [*cmd_base[:-1], str(ev3)],
            capture_output=True,
            env=env_repo_local,
            text=True,
            timeout=60,
        )
        self.assertNotEqual(r3.returncode, 0)
        self.assertIn("outside the Git repository", r3.stderr)

        env_ok = env.copy()
        env_ok["PYTHONPYCACHEPREFIX"] = self.pycache
        r4 = self.run_runner()
        self.assertEqual(r4.returncode, 0, r4.stderr)
        plan = self.read_plan()
        self.assertEqual(
            plan["environment"]["pycache_prefix"],
            str(Path(self.pycache).resolve()),
        )

    def test_25_repo_local_evidence_dir_rejected(self) -> None:
        ids = self.setup_passing_suite(2)
        repo_local_evidence = self.root / "evidence_inside_repo"
        repo_local_evidence.mkdir(parents=True, exist_ok=True)
        result = self.run_runner("--evidence-dir", str(repo_local_evidence))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must not live inside the repository", result.stderr)

    def test_26_repo_local_evidence_via_symlink_rejected(self) -> None:
        ids = self.setup_passing_suite(2)
        symlink_target = self.root / "outside_target"
        symlink_target.mkdir(parents=True, exist_ok=True)
        symlink_path = self.root / "evidence_symlink"
        os.symlink(symlink_target, symlink_path)
        result = self.run_runner("--evidence-dir", str(symlink_path))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must not live inside the repository", result.stderr)

    def test_27_git_identity_failure_fail_closed(self) -> None:
        ids = self.setup_passing_suite(2)
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        env["PATH"] = "/nonexistent"
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "1",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        result = subprocess.run(
            cmd,
            capture_output=True,
            env=env,
            text=True,
            timeout=60,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(
            "plan construction failed" in result.stderr or "git" in result.stderr.lower(),
            f"expected git identity failure, got: {result.stderr}",
        )

    def test_28_git_identity_failure_post_launch_fails(self) -> None:
        self.write_test("test_slow.py", SLOW_TEST)
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "1",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            text=True,
        )
        plan_path = self.evidence / "plan" / "plan.json"
        deadline = time.monotonic() + 15
        while not plan_path.is_file():
            if time.monotonic() > deadline:
                proc.kill()
                self.fail("plan.json was not written in time")
            time.sleep(0.1)
        time.sleep(0.3)
        git_dir = self.root / ".git"
        moved = self.root / ".git_quarantined"
        if git_dir.is_dir():
            git_dir.rename(moved)
        try:
            stdout, stderr = proc.communicate(timeout=60)
        finally:
            if moved.is_dir():
                moved.rename(git_dir)
        self.assertNotEqual(proc.returncode, 0, stderr)

    def test_35_require_clean_revalidated_at_plan_construction(self) -> None:
        """F2: --require-clean must be revalidated on the plan's own baseline.

        The pre-discovery cleanliness check passes, then a mutation lands
        during discovery; the plan-freeze recheck must reject that dirty
        baseline through ``ERROR: plan construction failed`` with no plan and
        no summary PASS. This proves the recheck plus existing post-launch
        comparison, not filesystem-level atomicity.
        """
        import contextlib
        import io
        from unittest import mock

        ids = self.setup_passing_suite(2)
        evidence = Path(self.pycache_temp.name) / "evidence_f2_plan_freeze"
        original_discover = runner.discover_test_ids

        def mutate_during_discovery() -> list[str]:
            (self.root / "scripts" / "dirty_marker.py").write_text(
                "# mutation landing during discovery\n", encoding="utf-8"
            )
            return original_discover()

        stdout_capture = io.StringIO()
        stderr_capture = io.StringIO()
        with mock.patch.object(runner, "_repo_root", return_value=self.root):
            with mock.patch.object(
                runner, "discover_test_ids", side_effect=mutate_during_discovery
            ):
                with mock.patch.dict(
                    os.environ, {"PYTHONPYCACHEPREFIX": self.pycache}
                ):
                    with contextlib.redirect_stdout(stdout_capture):
                        with contextlib.redirect_stderr(stderr_capture):
                            returncode = runner._parent_main(
                                [
                                    "--jobs", "1",
                                    "--timeout-seconds", "30",
                                    "--inventory",
                                    str(
                                        self.root
                                        / "scripts"
                                        / "checker_test_inventory.json"
                                    ),
                                    "--evidence-dir", str(evidence),
                                    "--require-clean",
                                ]
                            )
        self._purge_stale_test_modules()
        self.assertNotEqual(returncode, 0)
        self.assertIn("plan construction failed", stderr_capture.getvalue())
        self.assertIn("--require-clean", stderr_capture.getvalue())
        self.assertIn("dirty at plan construction", stderr_capture.getvalue())
        self.assertNotIn("PASS", stdout_capture.getvalue())
        self.assertFalse(
            (evidence / "plan" / "plan.json").is_file(),
            "no plan must be written when the plan-freeze recheck rejects dirt",
        )
        self.assertFalse(
            (evidence / "summary.json").is_file(),
            "no summary must be written when the plan-freeze recheck rejects dirt",
        )


class InventoryInSourceDigestsTests(RunnerTestBase):
    def test_29_inventory_in_source_digests(self) -> None:
        ids = self.setup_passing_suite(2)
        result = self.run_runner()
        self.assertEqual(result.returncode, 0, result.stderr)
        plan = self.read_plan()
        self.assertIn(
            "scripts/checker_test_inventory.json",
            plan["source_digests"],
            "inventory must appear in source_digests",
        )
        self.assertEqual(
            plan["source_digests"]["scripts/checker_test_inventory.json"],
            plan["inventory_digest"],
            "inventory digest in source_digests must equal the dedicated inventory_digest",
        )

        custom_inv_path = self.root / "scripts" / "custom_inventory.json"
        custom_inv_path.write_text(
            json.dumps(
                {
                    "schema_version": "checker-test-inventory/v1",
                    "test_ids": ids,
                    "expected_count": len(ids),
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )
        evidence_f11_custom = Path(self.pycache_temp.name) / "evidence_f11_custom"
        env_f11 = os.environ.copy()
        env_f11["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd_custom = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "4",
            "--timeout-seconds", "30",
            "--inventory", "scripts/custom_inventory.json",
            "--evidence-dir", str(evidence_f11_custom),
        ]
        result_custom = subprocess.run(cmd_custom, capture_output=True, env=env_f11, text=True, timeout=120)
        self.assertEqual(result_custom.returncode, 0, result_custom.stderr)
        plan_custom = json.loads(
            (evidence_f11_custom / "plan" / "plan.json").read_text(encoding="utf-8")
        )
        self.assertIn(
            "scripts/custom_inventory.json",
            plan_custom["source_digests"],
            "custom inventory relative path must appear in source_digests",
        )
        self.assertNotIn(
            "scripts/checker_test_inventory.json",
            plan_custom["source_digests"],
            "default inventory path must be absent when a different in-repo inventory is selected",
        )
        self.assertEqual(
            plan_custom["source_digests"]["scripts/custom_inventory.json"],
            plan_custom["inventory_digest"],
            "custom inventory digest in source_digests must equal the dedicated inventory_digest",
        )

        external_inv = Path(self.pycache_temp.name) / "external_inventory.json"
        external_inv.write_text(
            json.dumps(
                {
                    "schema_version": "checker-test-inventory/v1",
                    "test_ids": ids,
                    "expected_count": len(ids),
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )
        evidence_f11_external = Path(self.pycache_temp.name) / "evidence_f11_external"
        cmd_external = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "4",
            "--timeout-seconds", "30",
            "--inventory", str(external_inv),
            "--evidence-dir", str(evidence_f11_external),
        ]
        result_external = subprocess.run(cmd_external, capture_output=True, env=env_f11, text=True, timeout=120)
        self.assertNotEqual(result_external.returncode, 0, "out-of-repo inventory must fail")
        self.assertIn("plan construction failed", result_external.stderr)
        self.assertFalse(
            (evidence_f11_external / "plan" / "plan.json").is_file(),
            "no plan must be written for out-of-repo inventory",
        )
        self.assertFalse(
            (evidence_f11_external / "summary.json").is_file(),
            "no summary must be written for out-of-repo inventory",
        )
        self.assertNotIn("PASS", result_external.stdout)

    def test_30_inventory_mutation_after_plan_fails(self) -> None:
        self.write_test("test_slow.py", SLOW_TEST)
        self.write_test("test_pass.py", PASS_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "1",
            "--timeout-seconds", "30",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            text=True,
        )
        plan_path = self.evidence / "plan" / "plan.json"
        deadline = time.monotonic() + 15
        while not plan_path.is_file():
            if time.monotonic() > deadline:
                proc.kill()
                self.fail("plan.json was not written in time")
            time.sleep(0.1)
        time.sleep(0.3)
        inv_path = self.root / "scripts" / "checker_test_inventory.json"
        original = inv_path.read_text(encoding="utf-8")
        inv_path.write_text(original + "\n", encoding="utf-8")
        stdout, stderr = proc.communicate(timeout=60)
        self.assertNotEqual(proc.returncode, 0, stderr)
        summary = self.read_summary()
        self.assertEqual(summary["status"], "failed")
        self.assertTrue(
            any("inventory digest drift" in e or "post-launch source digest drift" in e for e in summary["errors"]),
            f"expected inventory drift, got {summary['errors']}",
        )


class DuplicateDiscoveryTests(RunnerTestBase):
    def test_31_load_tests_duplicate_id_rejected(self) -> None:
        dup_test = (
            "import unittest\n"
            "def load_tests(loader, tests, pattern):\n"
            "    suite = unittest.TestSuite()\n"
            "    class TestDup(unittest.TestCase):\n"
            "        def test_dup(self) -> None:\n"
            "            pass\n"
            "    suite.addTest(TestDup('test_dup'))\n"
            "    suite.addTest(TestDup('test_dup'))\n"
            "    return suite\n"
        )
        self.write_test("test_dup.py", dup_test)
        from unittest import mock
        with mock.patch.object(runner, "_tests_root", return_value=self.root / "scripts" / "tests"):
            self._purge_stale_test_modules()
            with self.assertRaises(ValueError) as ctx:
                runner.discover_test_ids()
        self.assertIn("duplicate test IDs", str(ctx.exception))


class ParentTimeoutReportTests(RunnerTestBase):
    def test_32_timeout_writes_parent_owned_report(self) -> None:
        self.write_test("test_slow.py", SLOW_TEST)
        ids = self.discover_ids()
        self.write_inventory(ids)
        self.git_commit_all()
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = self.pycache
        cmd = [
            sys.executable,
            str(self.root / "scripts" / "run_checker_shards.py"),
            "--jobs", "1",
            "--timeout-seconds", "2",
            "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
            "--evidence-dir", str(self.evidence),
        ]
        result = subprocess.run(
            cmd,
            capture_output=True,
            env=env,
            text=True,
            timeout=60,
        )
        self.assertNotEqual(result.returncode, 0)
        plan = self.read_plan()
        shard_id = plan["shards"][0]["shard_id"]
        report_path = self.evidence / f"shard-{shard_id}" / "report.json"
        self.assertTrue(report_path.is_file(), "parent-timeout report must exist")
        report = json.loads(report_path.read_text(encoding="utf-8"))
        self.assertEqual(report.get("report_owner"), "parent-timeout")
        expected_ids = plan["shards"][0]["test_ids"]
        self.assertEqual(sorted(report["outcomes"].keys()), sorted(expected_ids))
        for outcome in report["outcomes"].values():
            self.assertEqual(outcome, "timeout")
        self.assertEqual(report["plan_digest"], plan["plan_digest"])
        self.assertEqual(
            report["runner_digest"],
            plan["source_digests"].get("scripts/run_checker_shards.py"),
        )
        self.assertEqual(
            report["inventory_digest"],
            plan["inventory_digest"],
        )

    def test_33_late_child_report_after_parent_timeout_rejected(self) -> None:
        plan: dict[str, Any] = {
            "plan_digest": "d" * 64,
            "shards": [{"shard_id": 0, "test_ids": ["m.T.test_a"]}],
            "source_digests": {"scripts/run_checker_shards.py": "r" * 64, "scripts/check_repo_contracts.py": "c" * 64, "scripts/checker_test_inventory.json": "i" * 64},
            "inventory_digest": "i" * 64,
            "aggregate_framed_input_digest": "a" * 64,
            "expected_test_ids": ["m.T.test_a"],
        }
        report_dir = self.evidence / "shard-0"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_path = report_dir / "report.json"
        late_child_report = {
            "schema_version": runner.SCHEMA_VERSION,
            "shard_id": 0,
            "plan_digest": plan["plan_digest"],
            "runner_digest": plan["source_digests"]["scripts/run_checker_shards.py"],
            "checker_digest": plan["source_digests"]["scripts/check_repo_contracts.py"],
            "inventory_digest": plan["inventory_digest"],
            "aggregate_framed_input_digest": plan["aggregate_framed_input_digest"],
            "expected_test_ids": ["m.T.test_a"],
            "expected_count": 1,
            "outcomes": {"m.T.test_a": "passed"},
            "reported_count": 1,
            "details": {},
            "stdout_log_digest": "",
            "stderr_log_digest": "",
        }
        runner._atomic_write_json(report_path, late_child_report)
        stdout_path = report_dir / "stdout.log"
        stderr_path = report_dir / "stderr.log"
        stdout_path.write_text("", encoding="utf-8")
        stderr_path.write_text("", encoding="utf-8")
        child_results = [
            {
                "shard_id": 0,
                "exit_code": -9,
                "report_path": str(report_path),
                "stdout_path": str(stdout_path),
                "stderr_path": str(stderr_path),
                "timed_out": True,
                "parent_timeout_report": True,
            }
        ]
        _, errors = runner._verify_reports(plan, child_results)
        self.assertTrue(
            any("overwritten by late child report" in e for e in errors),
            f"expected late child report rejection, got {errors}",
        )


class TransactionalSpawnTests(RunnerTestBase):
    def test_34_spawn_failure_cleans_prior_handles(self) -> None:
        from unittest import mock

        plan: dict[str, Any] = {
            "plan_digest": "d" * 64,
            "shards": [
                {"shard_id": 0, "test_ids": ["m.T.test_a"]},
                {"shard_id": 1, "test_ids": ["m.T.test_b"]},
            ],
            "source_digests": {"scripts/run_checker_shards.py": "r" * 64, "scripts/check_repo_contracts.py": "c" * 64, "scripts/checker_test_inventory.json": "i" * 64},
            "inventory_digest": "i" * 64,
            "aggregate_framed_input_digest": "a" * 64,
            "expected_test_ids": ["m.T.test_a", "m.T.test_b"],
        }
        plan_dir = self.evidence / "plan"
        plan_dir.mkdir(parents=True, exist_ok=True)
        plan_path = plan_dir / "plan.json"
        runner._atomic_write_json(plan_path, plan)

        call_count = {"n": 0}
        real_spawn = runner._spawn_child

        def flaky_spawn(*, shard_id, plan_path, evidence_dir, pycache_prefix, timeout_seconds):
            call_count["n"] += 1
            if call_count["n"] == 2:
                raise OSError("synthetic spawn failure on shard 1")
            return real_spawn(
                shard_id=shard_id,
                plan_path=plan_path,
                evidence_dir=evidence_dir,
                pycache_prefix=pycache_prefix,
                timeout_seconds=timeout_seconds,
            )

        with mock.patch.object(runner, "_spawn_child", side_effect=flaky_spawn):
            with self.assertRaises(runner.LaunchFailure) as ctx:
                runner._run_children(
                    plan=plan,
                    plan_path=plan_path,
                    evidence_dir=self.evidence,
                    pycache_prefix=Path(self.pycache),
                    timeout_seconds=30,
                )
            self.assertGreaterEqual(len(ctx.exception.owned_handles), 1)
            for handle in ctx.exception.owned_handles:
                self.assertIsNotNone(handle.proc.poll(), "owned child must be reaped")
                self.assertTrue(handle.stdout_handle.closed, "stdout handle must be closed")
                self.assertTrue(handle.stderr_handle.closed, "stderr handle must be closed")

        self.setup_passing_suite(2)
        evidence_f12 = Path(self.pycache_temp.name) / "evidence_f12"

        def raise_launch_failure(**kwargs: Any) -> list[dict[str, Any]]:
            raise runner.LaunchFailure("synthetic launch failure", [])

        with mock.patch.object(runner, "_repo_root", return_value=self.root):
            with mock.patch.object(runner, "_run_children", side_effect=raise_launch_failure):
                with mock.patch.dict(os.environ, {"PYTHONPYCACHEPREFIX": self.pycache}):
                    returncode = runner._parent_main([
                        "--jobs", "1",
                        "--timeout-seconds", "30",
                        "--inventory", str(self.root / "scripts" / "checker_test_inventory.json"),
                        "--evidence-dir", str(evidence_f12),
                    ])
        self.assertNotEqual(returncode, 0, "_parent_main must return nonzero on LaunchFailure")
        summary_f12 = json.loads(
            (evidence_f12 / "summary.json").read_text(encoding="utf-8")
        )
        self.assertEqual(summary_f12["status"], "failed")
        self.assertTrue(
            any("child launch failed" in e for e in summary_f12["errors"]),
            f"expected launch-failure diagnostic, got {summary_f12['errors']}",
        )


if __name__ == "__main__":
    unittest.main()
