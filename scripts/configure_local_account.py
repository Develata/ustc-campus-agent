#!/usr/bin/env python3
"""Explicit local-operator provisioning. No HTTP route; no raw credential file."""
import argparse
import fcntl
import getpass
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("config", type=Path)
    parser.add_argument("login_name")
    parser.add_argument("--bootstrap-administrator", action="store_true")
    parser.add_argument("--tenant", default="tenant:campus")
    parser.add_argument("--hash-helper", type=Path, default=Path(__file__).resolve().parents[1] / "target/debug/examples/account_password_hash")
    args = parser.parse_args()
    if not re.fullmatch(r"[a-z0-9][a-z0-9._-]{1,62}[a-z0-9]", args.login_name):
        parser.error("login name must be 3-64 lowercase ASCII characters")
    path = args.config.absolute()
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    parent = path.parent.stat()
    if path.parent.is_symlink() or stat.S_IMODE(parent.st_mode) != 0o700 or parent.st_uid != os.getuid():
        parser.error("configuration parent must be an owner-private 0700 directory")
    password = getpass.getpass("Password (12-256 characters): ")
    if password != getpass.getpass("Repeat password: "):
        parser.error("passwords differ")
    result = subprocess.run([str(args.hash_helper)], input=password.encode(), capture_output=True, check=False)
    del password
    if result.returncode:
        parser.error("credential preparation failed; build the account_password_hash example first")
    lock = os.open(str(path) + ".operator-lock", os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if path.exists() or path.is_symlink():
            info = path.lstat()
            if not stat.S_ISREG(info.st_mode) or stat.S_IMODE(info.st_mode) != 0o600 or info.st_uid != os.getuid():
                parser.error("configuration must be an owner-private 0600 regular file")
            config = json.loads(path.read_text())
            if args.bootstrap_administrator or config.get("schema") != "platform-local-accounts/v1" or config.get("tenant_id") != args.tenant:
                parser.error("bootstrap/configuration conflicts with existing authority")
        else:
            if not args.bootstrap_administrator:
                parser.error("initial configuration requires explicit --bootstrap-administrator")
            config = {"schema": "platform-local-accounts/v1", "tenant_id": args.tenant, "accounts": []}
        if len(config["accounts"]) >= 128 or any(a["login_name"] == args.login_name for a in config["accounts"]):
            parser.error("account name unavailable or capacity reached")
        config["accounts"].append({"user_id": "user:" + uuid.uuid4().hex, "login_name": args.login_name,
            "password_hash": result.stdout.decode(), "active": True, "administrator": args.bootstrap_administrator, "credential_generation": 1})
        fd, temporary = tempfile.mkstemp(prefix=".accounts-", dir=path.parent)
        try:
            with os.fdopen(fd, "w") as output:
                json.dump(config, output, separators=(",", ":"))
                output.flush()
                os.fsync(output.fileno())
            os.replace(temporary, path)
            directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
    finally:
        os.close(lock)
    print("Account configured. The user must log in separately.")


if __name__ == "__main__":
    main()
