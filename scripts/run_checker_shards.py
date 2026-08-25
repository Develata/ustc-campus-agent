#!/usr/bin/env python3
"""Process-isolated exact-inventory checker shard runner.

This is the M90 CI evidence shard slice runner. It discovers the full Python
checker suite, partitions it into bounded process-isolated shards, runs each
shard in its own subprocess/process group, and fan-in verifies the exact
expected test IDs against the unique union of reported terminal outcomes.

Standard library only (Python 3.13). No threads for test execution. No
network. No arbitrary shell command execution from JSON/YAML/TSV. The runner
is an internal checker orchestration tool, not a public/operator CLI.

See docs/acceptance/ci-transition-ledger.md for the legacy-to-v2 invariant
mapping and docs/acceptance/gates.md for the broader gate context.
"""

from __future__ import annotations

import argparse
import errno
import hashlib
import json
import os
import platform
import re
import signal
import subprocess
import sys
import tempfile
import time
import traceback
import unittest
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------------------------

SCHEMA_VERSION = "checker-shard-runner/v1"
INVENTORY_SCHEMA_VERSION = "checker-test-inventory/v1"
TESTS_DIR_REL = "scripts/tests"
TEST_PATTERN = "test_*.py"
INVENTORY_DEFAULT_REL = "scripts/checker_test_inventory.json"
CHECKER_REL = "scripts/check_repo_contracts.py"
RUNNER_REL = "scripts/run_checker_shards.py"
GRACEFUL_TERMINATION_SECONDS = 5

VALID_TERMINAL_STATUSES = (
    "passed",
    "failed",
    "error",
    "skipped",
    "expected_failure",
    "unexpected_success",
    "not_run",
    "timeout",
)
PASSING_STATUS = "passed"

# Environment variable names that carry GitHub runner image identity.
IMAGE_OS_ENV = "ImageOS"
IMAGE_VERSION_ENV = "ImageVersion"


class GitIdentityError(ValueError):
    """Raised when Git identity (HEAD/tree/branch/status) is unavailable or malformed."""


class LaunchFailure(Exception):
    """Raised when a child spawn fails after earlier children already started."""

    def __init__(self, message: str, owned_handles: list["ChildHandle"]) -> None:
        super().__init__(message)
        self.owned_handles = owned_handles


# ---------------------------------------------------------------------------------------------
# Small helpers
# -----------------------------------------------------------------------------


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _sha256_file(path: Path) -> str:
    return _sha256_bytes(path.read_bytes())


def _canonical_json(obj: Any) -> bytes:
    """Serialize to deterministic JSON bytes (sorted keys, no trailing whitespace)."""
    return json.dumps(obj, sort_keys=True, indent=2, ensure_ascii=False).encode("utf-8") + b"\n"


def _atomic_write(path: Path, data: bytes) -> None:
    """Write bytes to path atomically; raise on any failure."""
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=path.name + ".", dir=str(path.parent))
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
        os.replace(tmp_name, path)
    except Exception:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


def _atomic_write_json(path: Path, obj: Any) -> None:
    _atomic_write(path, _canonical_json(obj))


def _framed_digest(entries: list[tuple[str, str]]) -> str:
    """SHA-256 over sorted (path, sha256) pairs with explicit framing.

    Each entry is framed as ``f"{path}\\0{sha256}\\n"`` so a path containing
    a newline or NUL cannot collide with a different framing.
    """
    h = hashlib.sha256()
    for path, digest in sorted(entries):
        h.update(f"{path}\0{digest}\n".encode("utf-8"))
    return h.hexdigest()


def _git_rev_parse(repo: Path, ref: str) -> str:
    """Resolve ``ref`` to a 40-hex object name.

    Raises :class:`GitIdentityError` if Git is unavailable, returns a non-zero
    exit code, or yields a value that is not a 40-hex SHA-1. Callers that need
    a sentinel (e.g. legacy status probes) wrap this in try/except.
    """
    try:
        completed = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--verify", ref],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise GitIdentityError(
            f"git rev-parse {ref!r} unavailable: {type(exc).__name__}: {exc}"
        ) from exc
    if completed.returncode != 0:
        raise GitIdentityError(
            f"git rev-parse {ref!r} failed (exit {completed.returncode}): "
            f"{completed.stderr.strip()}"
        )
    value = completed.stdout.strip()
    if not re.fullmatch(r"[0-9a-f]{40}", value):
        raise GitIdentityError(
            f"git rev-parse {ref!r} returned non-40-hex value: {value!r}"
        )
    return value


def _git_branch(repo: Path) -> str:
    """Return the abbreviated branch name (nonempty).

    Raises :class:`GitIdentityError` on Git failure or empty branch.
    """
    try:
        completed = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise GitIdentityError(
            f"git branch unavailable: {type(exc).__name__}: {exc}"
        ) from exc
    if completed.returncode != 0:
        raise GitIdentityError(
            f"git branch failed (exit {completed.returncode}): "
            f"{completed.stderr.strip()}"
        )
    value = completed.stdout.strip()
    if not value:
        raise GitIdentityError("git branch returned empty branch name")
    return value


def _git_porcelain_status(repo: Path) -> str:
    """Return ``git status --porcelain`` output.

    Raises :class:`GitIdentityError` on Git failure. The pre-launch and
    post-launch identity paths treat any failure as fatal; the optional
    diagnostic snapshot in the receipt wrapper catches the exception so the
    receipt is still emitted.
    """
    try:
        completed = subprocess.run(
            ["git", "-C", str(repo), "status", "--porcelain"],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise GitIdentityError(
            f"git status unavailable: {type(exc).__name__}: {exc}"
        ) from exc
    if completed.returncode != 0:
        raise GitIdentityError(
            f"git status failed (exit {completed.returncode}): "
            f"{completed.stderr.strip()}"
        )
    return completed.stdout


def _porcelain_status_digest(status: str) -> str:
    return _sha256_bytes(status.encode("utf-8"))


def _repo_root() -> Path:
    """Return the repository root (parent of scripts/)."""
    return Path(__file__).resolve().parents[1]


def _tests_root() -> Path:
    return _repo_root() / TESTS_DIR_REL


def _runner_path() -> Path:
    return _repo_root() / RUNNER_REL


def _checker_path() -> Path:
    return _repo_root() / CHECKER_REL


def _inventory_path(cli_value: str | None) -> Path:
    if cli_value is None:
        return _repo_root() / INVENTORY_DEFAULT_REL
    p = Path(cli_value)
    if not p.is_absolute():
        p = _repo_root() / p
    return p


def _test_source_paths() -> list[Path]:
    return sorted((_tests_root().glob(TEST_PATTERN)))


def _resolve_pycache_prefix() -> tuple[Path, str]:
    """Resolve and validate PYTHONPYCACHEPREFIX.

    Returns (resolved_path, source) where source is "env" or "sys".
    Raises ValueError on missing/relative/repository-local prefix.
    """
    raw = os.environ.get("PYTHONPYCACHEPREFIX")
    source = "env"
    if raw is None or raw == "":
        raw = getattr(sys, "pycache_prefix", None)
        source = "sys"
    if raw is None or raw == "":
        raise ValueError(
            "PYTHONPYCACHEPREFIX must be set to a nonempty absolute path outside the Git repository"
        )
    prefix = Path(raw)
    if not prefix.is_absolute():
        raise ValueError(
            f"PYTHONPYCACHEPREFIX must be an absolute path; got {raw!r}"
        )
    resolved = prefix.resolve()
    repo_resolved = _repo_root().resolve()
    try:
        resolved.relative_to(repo_resolved)
    except ValueError:
        pass
    else:
        raise ValueError(
            f"PYTHONPYCACHEPREFIX must resolve outside the Git repository; got {raw!r}"
        )
    return resolved, source


# ---------------------------------------------------------------------------------------------
# Test discovery
# ---------------------------------------------------------------------------------------------


def discover_test_ids() -> list[str]:
    """Discover the full Python checker suite and return sorted fully-qualified IDs.

    The discovery walks the suite and collects leaf test IDs *with multiplicity*
    so that two distinct leaves that happen to share an ID (e.g. a ``load_tests``
    protocol that re-loads the same TestCase class twice) are observable. Any
    duplicate leaf ID is rejected as a discovery contract violation: the runner
    cannot shard or attribute outcomes for an ambiguous ID, and the inventory
    would otherwise silently collapse the duplicate via its set-based compare.
    """
    loader = unittest.TestLoader()
    suite = loader.discover(start_dir=str(_tests_root()), pattern=TEST_PATTERN)
    leaves = _iter_leaf_tests(suite)
    ids: list[str] = []
    for test in leaves:
        test_id = test.id()
        if _is_failed_test_id(test_id):
            raise ValueError(f"discovery produced _FailedTest: {test_id}")
        ids.append(test_id)
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


def _iter_leaf_tests(suite: unittest.TestSuite) -> list[unittest.TestCase]:
    """Walk a TestSuite tree and return the leaf TestCase instances."""
    leaves: list[unittest.TestCase] = []
    stack: list[Any] = [suite]
    while stack:
        item = stack.pop()
        if isinstance(item, unittest.TestCase):
            leaves.append(item)
        elif isinstance(item, unittest.TestSuite):
            stack.extend(item)
    return leaves


def _is_failed_test_id(test_id: str) -> bool:
    return "_FailedTest" in test_id


# ---------------------------------------------------------------------------------------------
# Custom TestResult (child side)
# ---------------------------------------------------------------------------------------------


class ShardTestResult(unittest.TestResult):
    """Records one terminal outcome per expected test ID."""

    def __init__(self, expected_ids: list[str]) -> None:
        super().__init__()
        self._expected = list(expected_ids)
        self.outcomes: dict[str, str] = {}
        self.details: dict[str, str] = {}

    def _record(self, test: unittest.TestCase, status: str, detail: str = "") -> None:
        test_id = test.id()
        self.outcomes[test_id] = status
        if detail:
            self.details[test_id] = detail

    def addSuccess(self, test: unittest.TestCase) -> None:  # noqa: N802
        super().addSuccess(test)
        self._record(test, "passed")

    def addError(self, test: unittest.TestCase, err: Any) -> None:  # noqa: N802
        super().addError(test, err)
        self._record(test, "error", self._format_exception(err))

    def addFailure(self, test: unittest.TestCase, err: Any) -> None:  # noqa: N802
        super().addFailure(test, err)
        self._record(test, "failed", self._format_exception(err))

    def addSkip(self, test: unittest.TestCase, reason: str) -> None:  # noqa: N802
        super().addSkip(test, reason)
        self._record(test, "skipped", f"skip: {reason}")

    def addExpectedFailure(self, test: unittest.TestCase, err: Any) -> None:  # noqa: N802
        super().addExpectedFailure(test, err)
        self._record(test, "expected_failure", self._format_exception(err))

    def addUnexpectedSuccess(self, test: unittest.TestCase) -> None:  # noqa: N802
        super().addUnexpectedSuccess(test)
        self._record(test, "unexpected_success", "unexpected success")

    @staticmethod
    def _format_exception(err: Any) -> str:
        return "".join(traceback.format_exception(err[0], err[1], err[2]))

    def missing_or_unexpected(self) -> tuple[list[str], list[str]]:
        missing = sorted(set(self._expected) - set(self.outcomes))
        unexpected = sorted(set(self.outcomes) - set(self._expected))
        return missing, unexpected


# ---------------------------------------------------------------------------------------------
# Source digests
# ---------------------------------------------------------------------------------------------


def _compute_source_digests(selected_inventory_path: Path) -> dict[str, str]:
    """Return {repo_relative_path: sha256} for runner, checker, inventory, and all test_*.py inputs.

    The selected inventory file is a framed source input to the runner: the
    plan pins its digest, children re-resolve it, and the post-launch check
    revalidates it. Omitting it from ``source_digests`` would let a concurrent
    edit slip past the aggregate framed digest comparison while still being
    observed by the dedicated ``inventory_digest`` field, so it is included
    here as well. The selected inventory must resolve inside the repository so
    its repository-relative path is a stable ``source_digests`` key and the
    framed aggregate names exactly the same bytes as ``inventory_digest``.
    """
    repo = _repo_root()
    repo_resolved = repo.resolve()
    selected_resolved = selected_inventory_path.resolve()
    try:
        selected_rel = selected_resolved.relative_to(repo_resolved).as_posix()
    except ValueError:
        raise ValueError(
            f"selected inventory must resolve inside the repository: "
            f"{selected_resolved} is not under {repo_resolved}"
        )
    entries: dict[str, str] = {}
    for path in [_runner_path(), _checker_path(), selected_inventory_path, *_test_source_paths()]:
        rel = path.resolve().relative_to(repo_resolved).as_posix()
        entries[rel] = _sha256_file(path)
    assert selected_rel in entries
    return entries


def _aggregate_framed_digest(digests: dict[str, str]) -> str:
    return _framed_digest(list(digests.items()))


# ---------------------------------------------------------------------------------------------
# Inventory validation
# ---------------------------------------------------------------------------------------------


def _load_inventory(path: Path) -> dict[str, Any]:
    if not path.is_file():
        raise FileNotFoundError(f"inventory file missing: {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"inventory file is not valid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise ValueError("inventory top-level value must be an object")
    return data


def _validate_inventory(data: dict[str, Any], live_ids: list[str]) -> None:
    """Validate inventory schema and bidirectional coverage against live discovery."""
    if data.get("schema_version") != INVENTORY_SCHEMA_VERSION:
        raise ValueError(
            f"inventory schema_version drift: expected {INVENTORY_SCHEMA_VERSION!r}, "
            f"got {data.get('schema_version')!r}"
        )
    raw_ids = data.get("test_ids")
    if not isinstance(raw_ids, list):
        raise ValueError("inventory test_ids must be a list")
    for entry in raw_ids:
        if not isinstance(entry, str) or not entry:
            raise ValueError(f"inventory test_ids contains a non-string/empty entry: {entry!r}")
    if len(raw_ids) != len(set(raw_ids)):
        raise ValueError("inventory test_ids contains duplicates")
    inventory_ids = raw_ids
    if inventory_ids != sorted(inventory_ids):
        raise ValueError("inventory test_ids must be sorted")
    expected_count = data.get("expected_count")
    if not isinstance(expected_count, int) or expected_count != len(inventory_ids):
        raise ValueError(
            f"inventory expected_count drift: expected {len(inventory_ids)}, got {expected_count!r}"
        )
    live_set = set(live_ids)
    inv_set = set(inventory_ids)
    missing = sorted(inv_set - live_set)
    unexpected = sorted(live_set - inv_set)
    if missing or unexpected:
        raise ValueError(
            f"inventory/live discovery bidirectional drift: "
            f"missing_from_live={missing} unexpected_in_live={unexpected}"
        )


# ---------------------------------------------------------------------------------------------
# Plan
# ---------------------------------------------------------------------------------------------


def _build_plan(
    *,
    argv: list[str],
    jobs: int,
    timeout_seconds: int,
    inventory_rel: str,
    pycache_prefix: Path,
    pycache_source: str,
    live_ids: list[str],
    require_clean: bool,
    require_runner_image_identity: bool,
) -> dict[str, Any]:
    repo = _repo_root()
    head = _git_rev_parse(repo, "HEAD")
    head_tree = _git_rev_parse(repo, "HEAD^{tree}")
    branch = _git_branch(repo)
    porcelain = _git_porcelain_status(repo)
    # Plan-freeze revalidation: the parent already rejected a dirty tree before
    # discovery, but discovery takes time. Re-reject right here, on the porcelain
    # baseline the plan itself captures, so a mutation that lands during
    # discovery cannot be frozen into the plan as a "clean" baseline. This is a
    # recheck plus the existing post-launch source/status comparison, not a
    # claim of filesystem-level atomicity between the two probes.
    if require_clean and porcelain.strip():
        raise ValueError(
            "--require-clean set but working tree is dirty at plan construction:\n"
            f"{porcelain}"
        )
    inventory_path = _inventory_path(inventory_rel)
    source_digests = _compute_source_digests(inventory_path)
    inventory_digest = _sha256_file(inventory_path)

    image_os = os.environ.get(IMAGE_OS_ENV)
    image_version = os.environ.get(IMAGE_VERSION_ENV)
    if require_runner_image_identity:
        if not image_os or not image_version:
            raise ValueError(
                "--require-runner-image-identity set but ImageOS or ImageVersion is missing/empty"
            )
        runner_image_identity_available = True
    else:
        image_os = None
        image_version = None
        runner_image_identity_available = False

    shards = _partition(live_ids, jobs)

    plan: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "argv": list(argv),
        "started_at_epoch": time.time(),
        "repository": {
            "branch": branch,
            "head": head,
            "head_tree": head_tree,
            "porcelain_status_digest": _porcelain_status_digest(porcelain),
        },
        "environment": {
            "python_version": platform.python_version(),
            "python_executable": sys.executable,
            "platform": platform.platform(),
            "image_os": image_os,
            "image_version": image_version,
            "runner_image_identity_available": runner_image_identity_available,
            "pycache_prefix": str(pycache_prefix),
            "pycache_prefix_source": pycache_source,
        },
        "configuration": {
            "jobs": jobs,
            "timeout_seconds": timeout_seconds,
            "require_clean": require_clean,
            "require_runner_image_identity": require_runner_image_identity,
            "inventory_rel": inventory_rel,
        },
        "source_digests": source_digests,
        "aggregate_framed_input_digest": _aggregate_framed_digest(source_digests),
        "inventory_digest": inventory_digest,
        "expected_test_ids": live_ids,
        "expected_count": len(live_ids),
        "shards": [
            {
                "shard_id": index,
                "test_ids": shard_ids,
                "shard_count": len(shard_ids),
            }
            for index, shard_ids in enumerate(shards)
        ],
        "actual_shard_count": len(shards),
    }
    return plan


def _partition(sorted_ids: list[str], jobs: int) -> list[list[str]]:
    """Deterministically partition sorted IDs across at most ``jobs`` nonempty shards."""
    if jobs < 1:
        raise ValueError("jobs must be >= 1")
    n = len(sorted_ids)
    if n == 0:
        raise ValueError("cannot partition zero tests")
    actual_shards = min(jobs, n)
    base = n // actual_shards
    extra = n % actual_shards
    shards: list[list[str]] = []
    start = 0
    for i in range(actual_shards):
        size = base + (1 if i < extra else 0)
        shards.append(sorted_ids[start : start + size])
        start += size
    if any(len(s) == 0 for s in shards):
        raise ValueError("partition produced an empty shard")
    return shards


def _compute_plan_digest(plan: dict[str, Any]) -> str:
    """SHA-256 over the plan's canonical JSON with plan_digest stripped."""
    copy = {k: v for k, v in plan.items() if k != "plan_digest"}
    return _sha256_bytes(_canonical_json(copy))


def _write_plan(plan_dir: Path, plan: dict[str, Any]) -> Path:
    plan_path = plan_dir / "plan.json"
    plan["plan_digest"] = _compute_plan_digest(plan)
    _atomic_write_json(plan_path, plan)
    return plan_path


# ---------------------------------------------------------------------------------------------
# Parent: spawn children, wait, fan-in
# ---------------------------------------------------------------------------------------------


@dataclass
class ChildHandle:
    """Bookkeeping for a spawned child: Popen plus the resources the parent owns."""

    proc: subprocess.Popen[bytes]
    shard_id: int
    report_path: Path
    stdout_path: Path
    stderr_path: Path
    stdout_handle: Any
    stderr_handle: Any
    timeout_seconds: int


def _spawn_child(
    *,
    shard_id: int,
    plan_path: Path,
    evidence_dir: Path,
    pycache_prefix: Path,
    timeout_seconds: int,
) -> ChildHandle:
    """Spawn one child shard process.

    Transactional: on any failure after a resource (log dir, file handles, the
    subprocess) is acquired, every resource opened so far is closed/reaped
    before the exception propagates. A caller that catches the exception can
    assume no leaked handles or orphan processes from this call.
    """
    report_path = evidence_dir / f"shard-{shard_id}" / "report.json"
    log_dir = evidence_dir / f"shard-{shard_id}"
    log_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = log_dir / "stdout.log"
    stderr_path = log_dir / "stderr.log"
    stdout_handle: Any = None
    stderr_handle: Any = None
    proc: subprocess.Popen[bytes] | None = None
    try:
        stdout_handle = open(stdout_path, "wb")
        stderr_handle = open(stderr_path, "wb")
        env = os.environ.copy()
        env["PYTHONPYCACHEPREFIX"] = str(pycache_prefix)
        env["_UCA_CHILD_STDOUT_PATH"] = str(stdout_path)
        env["_UCA_CHILD_STDERR_PATH"] = str(stderr_path)
        cmd = [
            sys.executable,
            str(_runner_path()),
            "--__child-shard",
            str(shard_id),
            "--__plan-path",
            str(plan_path),
            "--__report-path",
            str(report_path),
        ]
        proc = subprocess.Popen(
            cmd,
            stdout=stdout_handle,
            stderr=stderr_handle,
            env=env,
            preexec_fn=os.setsid,
        )
    except BaseException:
        if proc is not None:
            _terminate_process_group(proc)
            try:
                proc.wait(timeout=GRACEFUL_TERMINATION_SECONDS)
            except subprocess.TimeoutExpired:
                pass
        if stdout_handle is not None:
            try:
                stdout_handle.close()
            except OSError:
                pass
        if stderr_handle is not None:
            try:
                stderr_handle.close()
            except OSError:
                pass
        raise
    return ChildHandle(
        proc=proc,
        shard_id=shard_id,
        report_path=report_path,
        stdout_path=stdout_path,
        stderr_path=stderr_path,
        stdout_handle=stdout_handle,
        stderr_handle=stderr_handle,
        timeout_seconds=timeout_seconds,
    )


def _process_group_alive(pgid: int) -> bool:
    """Return True when process group ``pgid`` has a live (non-zombie) member.

    ``kill(pgid, 0)`` semantics alone treat a not-yet-reaped zombie as alive,
    which would keep cleanup spinning on a member that is already dead; on
    Linux the /proc scan below excludes zombie members. ``ESRCH`` from the
    fallback probe means the group is already gone; any other probe failure
    conservatively counts as alive so cleanup escalates to SIGKILL.
    """
    proc_root = Path("/proc")
    if proc_root.is_dir():
        found_member = False
        try:
            entries = os.listdir(proc_root)
        except OSError:
            entries = []
        for entry in entries:
            if not entry.isdigit():
                continue
            try:
                data = (proc_root / entry / "stat").read_bytes()
            except OSError:
                continue
            close = data.rfind(b")")
            if close < 0:
                continue
            fields = data[close + 2:].split()
            if len(fields) < 3:
                continue
            try:
                member_pgid = int(fields[2])
            except ValueError:
                continue
            if member_pgid != pgid:
                continue
            found_member = True
            if fields[0] != b"Z":
                return True
        if found_member:
            return False
    try:
        os.killpg(pgid, 0)
    except OSError as exc:
        return exc.errno != errno.ESRCH
    return True


def _terminate_process_group(proc: subprocess.Popen[bytes]) -> None:
    """Send SIGTERM then SIGKILL to the child's whole process group; never raise.

    The original PGID is captured before signalling and preserved afterwards:
    a leader that exits on SIGTERM does not imply the group is gone, because
    another member may ignore SIGTERM and stay alive. The leader is reaped
    (waited) as applicable, then the group itself is independently re-checked;
    if any live member remains, SIGKILL is sent to the same owned PGID and the
    check repeats under a bounded deadline. ``ESRCH`` is treated as already
    gone; every OSError path returns instead of raising.
    """
    try:
        pgid = os.getpgid(proc.pid)
    except OSError:
        return
    try:
        os.killpg(pgid, signal.SIGTERM)
    except OSError:
        pass
    try:
        proc.wait(timeout=GRACEFUL_TERMINATION_SECONDS)
    except (subprocess.TimeoutExpired, OSError):
        pass
    deadline = time.monotonic() + GRACEFUL_TERMINATION_SECONDS
    while _process_group_alive(pgid):
        try:
            os.killpg(pgid, signal.SIGKILL)
        except OSError:
            return
        try:
            proc.poll()
        except OSError:
            return
        if time.monotonic() >= deadline:
            return
        time.sleep(0.05)


def _run_children(
    plan: dict[str, Any],
    plan_path: Path,
    evidence_dir: Path,
    pycache_prefix: Path,
    timeout_seconds: int,
) -> list[dict[str, Any]]:
    """Spawn, supervise, and reap all child shards.

    Spawning is incremental: if shard N fails to spawn (raising from
    :func:`_spawn_child`), every already-spawned child is terminated and
    reaped, all owned file handles are closed, and no later shard is spawned.
    The failure is propagated as a :class:`LaunchFailure` carrying the owned
    handles so the caller can still emit a failure summary.

    Per-shard timeouts are parent-owned: when a deadline is exceeded the parent
    terminates the child's process group and atomically writes a parent-owned
    report at the path the child would have written, with terminal ``timeout``
    outcomes for every expected ID and the full plan/runner/checker/inventory
    identity. The fan-in consumes that parent report; a late child report that
    arrives after the parent report was written is rejected.
    """
    handles: list[ChildHandle] = []
    try:
        for shard in plan["shards"]:
            try:
                handle = _spawn_child(
                    shard_id=shard["shard_id"],
                    plan_path=plan_path,
                    evidence_dir=evidence_dir,
                    pycache_prefix=pycache_prefix,
                    timeout_seconds=timeout_seconds,
                )
            except BaseException as exc:
                for owned in handles:
                    _terminate_process_group(owned.proc)
                    try:
                        owned.proc.wait(timeout=GRACEFUL_TERMINATION_SECONDS)
                    except subprocess.TimeoutExpired:
                        pass
                    try:
                        owned.stdout_handle.close()
                    except OSError:
                        pass
                    try:
                        owned.stderr_handle.close()
                    except OSError:
                        pass
                raise LaunchFailure(
                    f"shard {shard['shard_id']} spawn failed: {exc}",
                    owned_handles=handles,
                ) from exc
            handles.append(handle)
    except LaunchFailure:
        raise

    deadline_per_handle = [
        (handle, time.monotonic() + handle.timeout_seconds) for handle in handles
    ]
    timed_out: set[int] = set()
    parent_timeout_reports: dict[int, Path] = {}
    while True:
        any_alive = False
        for handle, deadline in deadline_per_handle:
            if handle.proc.poll() is None:
                any_alive = True
                if time.monotonic() > deadline and handle.shard_id not in timed_out:
                    timed_out.add(handle.shard_id)
                    _terminate_process_group(handle.proc)
                    shard_entry = next(
                        s for s in plan["shards"] if s["shard_id"] == handle.shard_id
                    )
                    parent_report_path = _write_parent_timeout_report(
                        plan=plan,
                        shard=shard_entry,
                        report_path=handle.report_path,
                    )
                    parent_timeout_reports[handle.shard_id] = parent_report_path
        if not any_alive:
            break
        time.sleep(0.05)
    for handle in handles:
        try:
            handle.stdout_handle.close()
        except OSError:
            pass
        try:
            handle.stderr_handle.close()
        except OSError:
            pass
    results = []
    for handle in handles:
        exit_code = handle.proc.returncode if handle.proc.poll() is not None else -1
        results.append(
            {
                "shard_id": handle.shard_id,
                "exit_code": exit_code,
                "report_path": str(handle.report_path),
                "stdout_path": str(handle.stdout_path),
                "stderr_path": str(handle.stderr_path),
                "timed_out": handle.shard_id in timed_out,
                "parent_timeout_report": handle.shard_id in parent_timeout_reports,
            }
        )
    return results


def _write_parent_timeout_report(
    *,
    plan: dict[str, Any],
    shard: dict[str, Any],
    report_path: Path,
) -> Path:
    """Atomically write a parent-owned terminal timeout report.

    The report carries ``report_owner=parent-timeout``, a terminal ``timeout``
    outcome for every expected test ID in the shard, and the same
    plan/runner/checker/inventory/aggregate identity the child would have
    recorded. Fan-in treats this as the authoritative report for the shard;
    a late child report that arrives afterward is rejected.
    """
    expected_ids: list[str] = list(shard.get("test_ids", []))
    outcomes = {test_id: "timeout" for test_id in expected_ids}
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "shard_id": shard["shard_id"],
        "report_owner": "parent-timeout",
        "plan_digest": plan.get("plan_digest"),
        "runner_digest": plan.get("source_digests", {}).get(RUNNER_REL),
        "checker_digest": plan.get("source_digests", {}).get(CHECKER_REL),
        "inventory_digest": plan.get("inventory_digest"),
        "aggregate_framed_input_digest": plan.get("aggregate_framed_input_digest"),
        "expected_test_ids": expected_ids,
        "expected_count": len(expected_ids),
        "outcomes": outcomes,
        "reported_count": len(outcomes),
        "details": {},
        "stdout_log_digest": _sha256_of_missing_or_empty(report_path.parent / "stdout.log"),
        "stderr_log_digest": _sha256_of_missing_or_empty(report_path.parent / "stderr.log"),
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    _atomic_write_json(report_path, report)
    return report_path


def _sha256_of_missing_or_empty(path: Path) -> str:
    if not path.is_file():
        return _sha256_bytes(b"")
    return _sha256_file(path)


# ---------------------------------------------------------------------------------------------
# Fan-in
# ---------------------------------------------------------------------------------------------


def _recompute_source_identity(repo: Path, plan: dict[str, Any]) -> tuple[dict[str, str], str, str, str, str, str]:
    """Recompute all source identities from the live repo for post-launch comparison.

    Raises :class:`GitIdentityError` if any Git identity probe fails. Callers
    catch it and surface it as a fatal post-launch error.
    """
    head = _git_rev_parse(repo, "HEAD")
    head_tree = _git_rev_parse(repo, "HEAD^{tree}")
    branch = _git_branch(repo)
    porcelain = _git_porcelain_status(repo)
    # Recompute file digests using the same paths the plan recorded.
    source_digests: dict[str, str] = {}
    for rel in plan["source_digests"]:
        path = repo / rel
        if not path.is_file():
            source_digests[rel] = "<missing>"
        else:
            source_digests[rel] = _sha256_file(path)
    return source_digests, _porcelain_status_digest(porcelain), head, head_tree, branch, _aggregate_framed_digest(source_digests)


def _verify_reports(
    plan: dict[str, Any],
    child_results: list[dict[str, Any]],
) -> tuple[dict[str, Any], list[str]]:
    """Independently verify shard reports, log digests, and union of outcomes."""
    errors: list[str] = []
    plan_digest = plan["plan_digest"]
    expected_shard_ids = {shard["shard_id"] for shard in plan["shards"]}
    observed_shard_ids = {result["shard_id"] for result in child_results}
    if expected_shard_ids != observed_shard_ids:
        errors.append(
            f"shard report set drift: expected={sorted(expected_shard_ids)} "
            f"observed={sorted(observed_shard_ids)}"
        )

    repo = _repo_root()
    reports: dict[int, dict[str, Any]] = {}
    log_digests: dict[int, dict[str, str]] = {}
    report_digests: dict[int, str] = {}

    for result in child_results:
        shard_id = result["shard_id"]
        report_path = Path(result["report_path"])
        stdout_path = Path(result["stdout_path"])
        stderr_path = Path(result["stderr_path"])
        parent_owned_timeout = bool(result.get("parent_timeout_report", False))
        if not report_path.is_file():
            errors.append(f"shard {shard_id}: report missing at {report_path}")
            continue
        try:
            report = json.loads(report_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            errors.append(f"shard {shard_id}: report is not valid JSON: {exc}")
            continue
        if not isinstance(report, dict):
            errors.append(f"shard {shard_id}: report is not an object")
            continue
        if parent_owned_timeout and report.get("report_owner") != "parent-timeout":
            errors.append(
                f"shard {shard_id}: parent-timeout report was overwritten by late child report"
            )
            continue
        reports[shard_id] = report
        # Verify report binds the exact plan digest and source identity.
        if report.get("plan_digest") != plan_digest:
            errors.append(
                f"shard {shard_id}: report plan_digest drift: "
                f"expected={plan_digest[:16]} got={str(report.get('plan_digest'))[:16]}"
            )
        if report.get("runner_digest") != plan["source_digests"].get(RUNNER_REL):
            errors.append(f"shard {shard_id}: report runner_digest drift")
        if report.get("checker_digest") != plan["source_digests"].get(CHECKER_REL):
            errors.append(f"shard {shard_id}: report checker_digest drift")
        if report.get("inventory_digest") != plan.get("inventory_digest"):
            errors.append(f"shard {shard_id}: report inventory_digest drift")
        if report.get("aggregate_framed_input_digest") != plan.get("aggregate_framed_input_digest"):
            errors.append(f"shard {shard_id}: report aggregate_framed_input_digest drift")

        # Recompute log digests from the closed files.
        if not stdout_path.is_file():
            errors.append(f"shard {shard_id}: stdout log missing at {stdout_path}")
        else:
            actual_stdout_digest = _sha256_file(stdout_path)
            if report.get("stdout_log_digest") != actual_stdout_digest:
                errors.append(
                    f"shard {shard_id}: stdout log digest drift: "
                    f"report={str(report.get('stdout_log_digest'))[:16]} "
                    f"actual={actual_stdout_digest[:16]}"
                )
        if not stderr_path.is_file():
            errors.append(f"shard {shard_id}: stderr log missing at {stderr_path}")
        else:
            actual_stderr_digest = _sha256_file(stderr_path)
            if report.get("stderr_log_digest") != actual_stderr_digest:
                errors.append(
                    f"shard {shard_id}: stderr log digest drift: "
                    f"report={str(report.get('stderr_log_digest'))[:16]} "
                    f"actual={actual_stderr_digest[:16]}"
                )
        # Hash the complete closed child report file.
        report_digests[shard_id] = _sha256_file(report_path)
        log_digests[shard_id] = {
            "stdout_log_digest": report.get("stdout_log_digest", ""),
            "stderr_log_digest": report.get("stderr_log_digest", ""),
        }

        # Verify expected test IDs equal the reported terminal outcomes (no missing/duplicate/extra).
        expected = list(shard["test_ids"] for shard in plan["shards"] if shard["shard_id"] == shard_id)[0]
        reported_outcomes = report.get("outcomes")
        if not isinstance(reported_outcomes, dict):
            errors.append(f"shard {shard_id}: outcomes is not an object")
            continue
        reported_ids = list(reported_outcomes.keys())
        if sorted(reported_ids) != sorted(expected):
            errors.append(
                f"shard {shard_id}: outcome ID set drift: "
                f"missing={sorted(set(expected) - set(reported_ids))} "
                f"extra={sorted(set(reported_ids) - set(expected))}"
            )
        for test_id, status in reported_outcomes.items():
            if status not in VALID_TERMINAL_STATUSES:
                errors.append(f"shard {shard_id}: {test_id} has invalid status {status!r}")
        if report.get("expected_count") != len(expected):
            errors.append(
                f"shard {shard_id}: expected_count drift: "
                f"report={report.get('expected_count')} plan={len(expected)}"
            )
        if report.get("reported_count") != len(reported_outcomes):
            errors.append(
                f"shard {shard_id}: reported_count drift: "
                f"report={report.get('reported_count')} actual={len(reported_outcomes)}"
            )

        # Verify no child nonzero/timeout/signal. Parent-owned timeout reports
        # are expected to have a killed child (nonzero exit, timed_out=True);
        # the terminal ``timeout`` outcomes already surface the failure below.
        if not parent_owned_timeout:
            if result["timed_out"]:
                errors.append(f"shard {shard_id}: child timed out without parent report")
            if result["exit_code"] != 0:
                errors.append(f"shard {shard_id}: child exit code {result['exit_code']}")

    # Verify expected test IDs equal the unique union of reported terminal outcomes.
    all_reported: list[str] = []
    for shard in plan["shards"]:
        report = reports.get(shard["shard_id"])
        if report is None:
            continue
        outcomes = report.get("outcomes", {})
        all_reported.extend(outcomes.keys())
    expected_all = plan["expected_test_ids"]
    if sorted(all_reported) != sorted(expected_all):
        errors.append(
            "union of reported outcomes drift: "
            f"missing={sorted(set(expected_all) - set(all_reported))} "
            f"extra={sorted(set(all_reported) - set(expected_all))} "
            f"duplicates={sorted([t for t in all_reported if all_reported.count(t) > 1])}"
        )

    # All required outcomes must be 'passed'.
    outcome_counts: dict[str, int] = {}
    for shard in plan["shards"]:
        report = reports.get(shard["shard_id"])
        if report is None:
            continue
        for status in report.get("outcomes", {}).values():
            outcome_counts[status] = outcome_counts.get(status, 0) + 1
    for status, count in outcome_counts.items():
        if status != PASSING_STATUS and count > 0:
            errors.append(f"non-pass outcome present: {status} count={count}")

    return (
        {
            "outcome_counts": outcome_counts,
            "report_digests": report_digests,
            "log_digests": log_digests,
        },
        errors,
    )


def _post_launch_source_check(repo: Path, plan: dict[str, Any]) -> list[str]:
    """Immediately before writing PASS, recompute and compare all recorded source identities.

    Git identity failures (HEAD/tree/branch/status unavailable or malformed) are
    fatal: a typed :class:`GitIdentityError` is surfaced as a single explicit
    error string rather than swallowed as a sentinel digest.
    """
    errors: list[str] = []
    try:
        source_digests, porcelain_digest, head, head_tree, branch, agg_digest = _recompute_source_identity(repo, plan)
    except GitIdentityError as exc:
        errors.append(f"post-launch git identity failure: {exc}")
        return errors
    if head != plan["repository"]["head"]:
        errors.append(f"post-launch HEAD drift: plan={plan['repository']['head']} live={head}")
    if head_tree != plan["repository"]["head_tree"]:
        errors.append(f"post-launch HEAD tree drift: plan={plan['repository']['head_tree']} live={head_tree}")
    if branch != plan["repository"]["branch"]:
        errors.append(f"post-launch branch drift: plan={plan['repository']['branch']} live={branch}")
    if porcelain_digest != plan["repository"]["porcelain_status_digest"]:
        errors.append("post-launch porcelain status digest drift")
    for rel, recorded in plan["source_digests"].items():
        actual = source_digests.get(rel, "<missing>")
        if actual != recorded:
            errors.append(f"post-launch source digest drift for {rel}: plan={recorded[:16]} live={actual[:16]}")
    if agg_digest != plan["aggregate_framed_input_digest"]:
        errors.append("post-launch aggregate framed input digest drift")
    inventory_rel = plan["configuration"]["inventory_rel"]
    inventory_path = _inventory_path(inventory_rel)
    if not inventory_path.is_file():
        errors.append(
            f"post-launch inventory missing at {inventory_path}"
        )
    else:
        live_inventory_digest = _sha256_file(inventory_path)
        if live_inventory_digest != plan["inventory_digest"]:
            errors.append(
                f"post-launch inventory digest drift: plan={plan['inventory_digest'][:16]} "
                f"live={live_inventory_digest[:16]}"
            )
    return errors


def _write_summary(
    plan: dict[str, Any],
    evidence_dir: Path,
    fan_in: dict[str, Any],
    errors: list[str],
    wall_seconds: float,
) -> Path:
    summary_path = evidence_dir / "summary.json"
    status = "passed" if not errors else "failed"
    summary = {
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "plan_digest": plan["plan_digest"],
        "wall_seconds": wall_seconds,
        "expected_count": plan["expected_count"],
        "outcome_counts": fan_in.get("outcome_counts", {}),
        "per_shard_evidence": {
            str(shard_id): {
                "report_digest": fan_in.get("report_digests", {}).get(shard_id, ""),
                "log_digests": fan_in.get("log_digests", {}).get(shard_id, {}),
            }
            for shard_id in [shard["shard_id"] for shard in plan["shards"]]
        },
        "errors": errors,
    }
    _atomic_write_json(summary_path, summary)
    return summary_path


def _check_evidence_dir_not_repo_local(evidence_dir: Path, repo: Path) -> str | None:
    """Return an error string if ``evidence_dir`` is inside ``repo`` (after symlink resolution).

    A repo-local evidence directory would let the runner's own writes (plan,
    reports, summary) pollute the porcelain status digest captured at plan
    time, defeating the post-launch source-identity check. The check resolves
    symlinks on both sides so a repo path linked from outside is still
    rejected. Returns ``None`` when the evidence directory is safely outside
    the repository.
    """
    try:
        repo_resolved = repo.resolve()
    except OSError as exc:
        return f"cannot resolve repository root: {exc}"
    try:
        evidence_resolved = evidence_dir.resolve()
    except OSError as exc:
        return f"cannot resolve evidence directory: {exc}"
    try:
        evidence_resolved.relative_to(repo_resolved)
    except ValueError:
        return None
    return (
        f"evidence directory must not live inside the repository: "
        f"{evidence_resolved} is under {repo_resolved}"
    )


# ---------------------------------------------------------------------------------------------
# Parent main
# ---------------------------------------------------------------------------------------------


def _parent_main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        prog="run_checker_shards",
        description="Process-isolated exact-inventory checker shard runner",
    )
    parser.add_argument("--jobs", type=int, required=True, help="Maximum number of parallel child processes")
    parser.add_argument("--timeout-seconds", type=int, required=True, help="Per-shard timeout in seconds")
    parser.add_argument("--inventory", type=str, default=None, help="Path to checker_test_inventory.json")
    parser.add_argument("--evidence-dir", type=str, required=True, help="Directory for plan/reports/summary")
    parser.add_argument("--require-clean", action="store_true", help="Reject any tracked/staged/untracked source change")
    parser.add_argument("--require-runner-image-identity", action="store_true", help="Require ImageOS and ImageVersion env vars")
    args = parser.parse_args(argv)

    if args.jobs < 1:
        print("ERROR: --jobs must be a positive integer", file=sys.stderr)
        return 2
    if args.timeout_seconds < 1:
        print("ERROR: --timeout-seconds must be a positive integer", file=sys.stderr)
        return 2

    evidence_dir = Path(args.evidence_dir).resolve()
    repo_for_evidence_check = _repo_root()
    evidence_repo_local_error = _check_evidence_dir_not_repo_local(
        evidence_dir, repo_for_evidence_check
    )
    if evidence_repo_local_error is not None:
        print(f"ERROR: {evidence_repo_local_error}", file=sys.stderr)
        return 2
    try:
        if evidence_dir.exists():
            if any(evidence_dir.iterdir()):
                print(f"ERROR: evidence directory exists and is not empty: {evidence_dir}", file=sys.stderr)
                return 2
        else:
            evidence_dir.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        print(f"ERROR: evidence setup failed: {exc}", file=sys.stderr)
        return 2

    try:
        pycache_prefix, pycache_source = _resolve_pycache_prefix()
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    repo = _repo_root()
    if args.require_clean:
        try:
            porcelain = _git_porcelain_status(repo)
        except GitIdentityError as exc:
            print(f"ERROR: --require-clean git status unavailable: {exc}", file=sys.stderr)
            return 2
        if porcelain.strip():
            print(f"ERROR: --require-clean set but working tree is dirty:\n{porcelain}", file=sys.stderr)
            return 2

    inventory_rel = args.inventory if args.inventory else INVENTORY_DEFAULT_REL
    inventory_path = _inventory_path(args.inventory)

    try:
        live_ids = discover_test_ids()
    except ValueError as exc:
        print(f"ERROR: discovery failed: {exc}", file=sys.stderr)
        return 2

    try:
        inventory_data = _load_inventory(inventory_path)
        _validate_inventory(inventory_data, live_ids)
    except (ValueError, FileNotFoundError) as exc:
        print(f"ERROR: inventory validation failed: {exc}", file=sys.stderr)
        return 2

    plan_dir = evidence_dir / "plan"
    try:
        plan_dir.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        print(f"ERROR: evidence setup failed: {exc}", file=sys.stderr)
        return 2

    try:
        plan = _build_plan(
            argv=sys.argv,
            jobs=args.jobs,
            timeout_seconds=args.timeout_seconds,
            inventory_rel=inventory_rel,
            pycache_prefix=pycache_prefix,
            pycache_source=pycache_source,
            live_ids=live_ids,
            require_clean=args.require_clean,
            require_runner_image_identity=args.require_runner_image_identity,
        )
    except (GitIdentityError, ValueError) as exc:
        print(f"ERROR: plan construction failed: {exc}", file=sys.stderr)
        return 2

    plan_path = _write_plan(plan_dir, plan)

    start_wall = time.monotonic()
    try:
        child_results = _run_children(
            plan=plan,
            plan_path=plan_path,
            evidence_dir=evidence_dir,
            pycache_prefix=pycache_prefix,
            timeout_seconds=args.timeout_seconds,
        )
    except LaunchFailure as exc:
        child_results = []
        fan_in, report_errors = _verify_reports(plan, child_results)
        post_launch_errors = _post_launch_source_check(repo, plan)
        all_errors = [f"child launch failed: {exc}"] + report_errors + post_launch_errors
        wall_seconds = time.monotonic() - start_wall
        summary_path = _write_summary(plan, evidence_dir, fan_in, all_errors, wall_seconds)
        print(f"checker-shards: FAILED summary={summary_path}")
        return 1
    fan_in, report_errors = _verify_reports(plan, child_results)
    post_launch_errors = _post_launch_source_check(repo, plan)
    all_errors = report_errors + post_launch_errors
    wall_seconds = time.monotonic() - start_wall

    summary_path = _write_summary(plan, evidence_dir, fan_in, all_errors, wall_seconds)
    status = "passed" if not all_errors else "failed"
    print(f"checker-shards: {status.upper()} summary={summary_path}")
    return 0 if status == "passed" else 1


# ---------------------------------------------------------------------------------------------
# Child main
# ---------------------------------------------------------------------------------------------


def _child_main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="run_checker_shards-child", add_help=False)
    parser.add_argument("--__child-shard", type=int, required=True)
    parser.add_argument("--__plan-path", type=str, required=True)
    parser.add_argument("--__report-path", type=str, required=True)
    args = parser.parse_args(argv)

    plan_path = Path(args.__plan_path)
    report_path = Path(args.__report_path)
    shard_id = args.__child_shard

    try:
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"child {shard_id}: cannot read plan: {exc}", file=sys.stderr)
        return 2

    tests_dir = str(_tests_root())
    if tests_dir not in sys.path:
        sys.path.insert(0, tests_dir)

    shard = None
    for entry in plan.get("shards", []):
        if entry.get("shard_id") == shard_id:
            shard = entry
            break
    if shard is None:
        print(f"child {shard_id}: shard {shard_id} not found in plan", file=sys.stderr)
        return 2

    expected_ids: list[str] = list(shard.get("test_ids", []))
    if not expected_ids:
        print(f"child {shard_id}: shard has zero expected test IDs", file=sys.stderr)
        return 2

    # Load exactly those IDs and reject _FailedTest/missing/extra/duplicate/zero.
    loader = unittest.TestLoader()
    loaded_suite = unittest.TestSuite()
    loaded_ids: list[str] = []
    failed_loads: list[str] = []
    for test_id in expected_ids:
        try:
            suite = loader.loadTestsFromName(test_id)
        except Exception as exc:  # noqa: BLE001
            failed_loads.append(f"{test_id}: {exc}")
            continue
        leaves = _iter_leaf_tests(suite)
        for leaf in leaves:
            leaf_id = leaf.id()
            if _is_failed_test_id(leaf_id):
                failed_loads.append(f"{test_id}: produced _FailedTest {leaf_id}")
                continue
            loaded_suite.addTest(leaf)
            loaded_ids.append(leaf_id)
    if failed_loads:
        for msg in failed_loads:
            print(f"child {shard_id}: load failure: {msg}", file=sys.stderr)
        return 2
    if sorted(loaded_ids) != sorted(expected_ids):
        missing = sorted(set(expected_ids) - set(loaded_ids))
        extra = sorted(set(loaded_ids) - set(expected_ids))
        print(
            f"child {shard_id}: loaded ID set drift: missing={missing} extra={extra}",
            file=sys.stderr,
        )
        return 2
    if len(loaded_ids) != len(set(loaded_ids)):
        print(f"child {shard_id}: loaded IDs contain duplicates", file=sys.stderr)
        return 2

    result = ShardTestResult(expected_ids)
    loaded_suite.run(result)

    missing, unexpected = result.missing_or_unexpected()
    if missing or unexpected:
        print(
            f"child {shard_id}: outcome ID set drift: missing={missing} unexpected={unexpected}",
            file=sys.stderr,
        )
        return 2

    # Flush Python's redirected streams before hashing their backing files.
    # Without this boundary, buffered test output can be written only at
    # process exit, after the child report has already frozen an empty digest.
    try:
        sys.stdout.flush()
        sys.stderr.flush()
    except (OSError, ValueError):
        return 2
    stdout_path = Path(os.environ.get("_UCA_CHILD_STDOUT_PATH", "/dev/null"))
    stderr_path = Path(os.environ.get("_UCA_CHILD_STDERR_PATH", "/dev/null"))
    stdout_digest = _sha256_file(stdout_path) if stdout_path.is_file() else ""
    stderr_digest = _sha256_file(stderr_path) if stderr_path.is_file() else ""

    report = {
        "schema_version": SCHEMA_VERSION,
        "shard_id": shard_id,
        "plan_digest": plan.get("plan_digest"),
        "runner_digest": plan.get("source_digests", {}).get(RUNNER_REL),
        "checker_digest": plan.get("source_digests", {}).get(CHECKER_REL),
        "inventory_digest": plan.get("inventory_digest"),
        "aggregate_framed_input_digest": plan.get("aggregate_framed_input_digest"),
        "expected_test_ids": expected_ids,
        "expected_count": len(expected_ids),
        "outcomes": result.outcomes,
        "reported_count": len(result.outcomes),
        "details": result.details,
        "stdout_log_digest": stdout_digest,
        "stderr_log_digest": stderr_digest,
    }

    # Determine exit code: nonzero for any non-pass outcome or evidence failure.
    non_pass = [status for status in result.outcomes.values() if status != PASSING_STATUS]
    try:
        _atomic_write_json(report_path, report)
    except OSError as exc:
        print(f"child {shard_id}: report write failed: {exc}", file=sys.stderr)
        return 2
    return 1 if non_pass else 0


# ---------------------------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------------------------


def main() -> int:
    argv = sys.argv[1:]
    if argv and argv[0] == "--__child-shard":
        # Skip the marker argument so argparse can consume the rest.
        return _child_main(argv)
    return _parent_main(argv)


if __name__ == "__main__":
    raise SystemExit(main())
