#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'usage: %s --binary PATH --output-dir DIR --source-commit SHA\n' "$0" >&2
  exit 64
}

binary=
output_dir=
source_commit=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --binary) [ "$#" -ge 2 ] || usage; binary=$2; shift 2 ;;
    --output-dir) [ "$#" -ge 2 ] || usage; output_dir=$2; shift 2 ;;
    --source-commit) [ "$#" -ge 2 ] || usage; source_commit=$2; shift 2 ;;
    *) usage ;;
  esac
done
[ -n "$binary" ] && [ -n "$output_dir" ] && [ -n "$source_commit" ] || usage

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
case "$source_commit" in
  *[!0-9a-f]*|'') printf 'source commit must be lowercase hexadecimal\n' >&2; exit 65 ;;
esac
[ "${#source_commit}" -eq 40 ] || { printf 'source commit must be 40 hexadecimal characters\n' >&2; exit 65; }
[ -f "$binary" ] && [ ! -L "$binary" ] && [ -x "$binary" ] || {
  printf 'binary must be an executable regular non-symlink file: %s\n' "$binary" >&2
  exit 66
}
[ ! -e "$output_dir" ] && [ ! -L "$output_dir" ] || {
  printf 'output path already exists: %s\n' "$output_dir" >&2
  exit 73
}

binary_version=$("$binary" --version)
[ "$binary_version" = 'ustc-agentd 0.1.0' ] || {
  printf 'unexpected binary version: %s\n' "$binary_version" >&2
  exit 65
}
python3 - "$binary" <<'PY'
from pathlib import Path
from struct import unpack_from
import sys

path = Path(sys.argv[1])
raw = path.read_bytes()
if len(raw) < 64 or raw[:4] != b'\x7fELF':
    raise SystemExit('binary must be an ELF executable')
if raw[4] != 2 or raw[5] != 1:
    raise SystemExit('binary must be a little-endian ELF64 executable')
e_type, e_machine = unpack_from('<HH', raw, 16)
if e_type not in (2, 3) or e_machine != 62:
    raise SystemExit('binary must be an x86-64 ELF executable')
e_phoff = unpack_from('<Q', raw, 32)[0]
e_phentsize, e_phnum = unpack_from('<HH', raw, 54)
interpreters = []
for index in range(e_phnum):
    offset = e_phoff + index * e_phentsize
    if offset + e_phentsize > len(raw) or e_phentsize < 56:
        raise SystemExit('binary has a malformed ELF program-header table')
    p_type = unpack_from('<I', raw, offset)[0]
    if p_type == 3:
        p_offset = unpack_from('<Q', raw, offset + 8)[0]
        p_filesz = unpack_from('<Q', raw, offset + 32)[0]
        value = raw[p_offset:p_offset + p_filesz].rstrip(b'\0')
        interpreters.append(value)
if interpreters != [b'/lib64/ld-linux-x86-64.so.2']:
    raise SystemExit(f'binary must use the x86-64 GNU/Linux loader, got {interpreters!r}')
PY

template_files=(Dockerfile compose.yaml container-entrypoint.sh .env.example README.md mock-provider-key.txt smoke.sh start.ps1 start.cmd start.sh stop.ps1 stop.cmd reset.ps1 reset.cmd)
for file in "${template_files[@]}"; do
  source_path="$repo_root/deploy/mvp-compose/$file"
  [ -f "$source_path" ] && [ ! -L "$source_path" ] || {
    printf 'missing package template: %s\n' "$source_path" >&2
    exit 66
  }
done
python3 - "$repo_root/deploy/mvp-compose" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
for name in ('start.ps1', 'stop.ps1', 'reset.ps1', 'start.cmd', 'stop.cmd', 'reset.cmd'):
    path = root / name
    raw = path.read_bytes()
    if raw.startswith((b'\xef\xbb\xbf', b'\xff\xfe', b'\xfe\xff')):
        raise SystemExit(f'Windows launcher must not contain a BOM: {path}')
    try:
        text = raw.decode('ascii')
    except UnicodeDecodeError as exc:
        raise SystemExit(
            f'Windows launcher must be ASCII-only for Windows PowerShell 5.1: {path}'
        ) from exc
    if '\x00' in text or not text.endswith('\n'):
        raise SystemExit(f'Windows launcher must be NUL-free and LF-terminated: {path}')
print('WINDOWS_LAUNCHER_ASCII_CHECK=PASS')
PY
fixture_files=(
  fixtures/affairs/proc-011-reviewed.json
  fixtures/change-radar/academic-calendar-demo-reviewed.json
  fixtures/change-radar/evidence/academic-calendar-r1.reviewed.txt
  fixtures/change-radar/evidence/academic-calendar-r1.normalized.json
  fixtures/change-radar/evidence/academic-calendar-r2.reviewed.txt
  fixtures/change-radar/evidence/academic-calendar-r2.normalized.json
  fixtures/opportunity-graph/course-planning-demo-reviewed.json
  market/fixtures/course-planning/minimal-v0.json
)
for file in "${fixture_files[@]}"; do
  source_path="$repo_root/$file"
  [ -f "$source_path" ] && [ ! -L "$source_path" ] || {
    printf 'missing package fixture: %s\n' "$source_path" >&2
    exit 66
  }
done

output_created=0
package_complete=0
cleanup_partial_output() {
  status=$?
  if [ "$status" -ne 0 ] && [ "$output_created" -eq 1 ] && [ "$package_complete" -eq 0 ]; then
    rm -rf -- "$output_dir"
  fi
  exit "$status"
}
trap cleanup_partial_output EXIT

package_name=ustc-campus-agent-mvp-compose
package_dir="$output_dir/$package_name"
output_created=1
mkdir -p "$package_dir/bin" \
  "$package_dir/fixtures/affairs" \
  "$package_dir/fixtures/change-radar" \
  "$package_dir/fixtures/change-radar/evidence" \
  "$package_dir/fixtures/opportunity-graph" \
  "$package_dir/market/fixtures/course-planning"

for file in "${template_files[@]}"; do
  source_path="$repo_root/deploy/mvp-compose/$file"
  cp "$source_path" "$package_dir/$file"
done

install -m 0755 "$binary" "$package_dir/bin/ustc-agentd"
install -m 0755 "$repo_root/deploy/mvp-compose/container-entrypoint.sh" "$package_dir/container-entrypoint.sh"
install -m 0755 "$repo_root/deploy/mvp-compose/smoke.sh" "$package_dir/smoke.sh"
install -m 0755 "$repo_root/deploy/mvp-compose/start.sh" "$package_dir/start.sh"
install -m 0644 "$repo_root/fixtures/affairs/proc-011-reviewed.json" "$package_dir/fixtures/affairs/proc-011-reviewed.json"
install -m 0644 "$repo_root/fixtures/change-radar/academic-calendar-demo-reviewed.json" "$package_dir/fixtures/change-radar/academic-calendar-demo-reviewed.json"
install -m 0644 "$repo_root/fixtures/change-radar/evidence/academic-calendar-r1.reviewed.txt" "$package_dir/fixtures/change-radar/evidence/academic-calendar-r1.reviewed.txt"
install -m 0644 "$repo_root/fixtures/change-radar/evidence/academic-calendar-r1.normalized.json" "$package_dir/fixtures/change-radar/evidence/academic-calendar-r1.normalized.json"
install -m 0644 "$repo_root/fixtures/change-radar/evidence/academic-calendar-r2.reviewed.txt" "$package_dir/fixtures/change-radar/evidence/academic-calendar-r2.reviewed.txt"
install -m 0644 "$repo_root/fixtures/change-radar/evidence/academic-calendar-r2.normalized.json" "$package_dir/fixtures/change-radar/evidence/academic-calendar-r2.normalized.json"
install -m 0644 "$repo_root/fixtures/opportunity-graph/course-planning-demo-reviewed.json" "$package_dir/fixtures/opportunity-graph/course-planning-demo-reviewed.json"
install -m 0644 "$repo_root/market/fixtures/course-planning/minimal-v0.json" "$package_dir/market/fixtures/course-planning/minimal-v0.json"
printf 'UCA_MVP_PORT=8787\nUCA_SOURCE_COMMIT=%s\nUCA_AGENT_PROVIDER=mock\nUCA_AGENT_BASE_URL=\nUCA_AGENT_MODEL=\nUCA_AGENT_TIMEOUT_MS=15000\nUCA_AGENT_API_KEY_SOURCE=./mock-provider-key.txt\n' "$source_commit" > "$package_dir/.env"
printf '%s\n' \
  'schema=ustc-campus-agent-mvp-compose-build/v1' \
  'package_version=0.2.0' \
  "source_commit=$source_commit" \
  'binary_target=x86_64-unknown-linux-gnu' \
  "binary_version=$binary_version" \
  > "$package_dir/BUILD-INFO.txt"

python3 - "$package_dir" <<'PY'
from __future__ import annotations
from hashlib import sha256
from pathlib import Path
import sys
root = Path(sys.argv[1])
rows = []
for path in sorted(root.rglob('*')):
    if path.is_symlink():
        raise SystemExit(f'symlink forbidden in package: {path}')
    if path.is_file() and path.name != 'SHA256SUMS':
        rows.append(f"{sha256(path.read_bytes()).hexdigest()}  {path.relative_to(root).as_posix()}")
(root / 'SHA256SUMS').write_text('\n'.join(rows) + '\n', encoding='utf-8')
PY

python3 - "$package_dir" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name == 'ustc-agentd':
        continue
    raw = path.read_bytes()
    if b'UCA_AGENT_API_KEY=' in raw or b'Authorization: Bearer ' in raw:
        raise SystemExit(f'provider secret carrier forbidden in package: {path}')
    if re.search(rb'(?i)(?:sk-|api[_-]?key\s*[:=]\s*)[A-Za-z0-9_\-]{16,}', raw):
        raise SystemExit(f'possible provider secret forbidden in package: {path}')
print('PROVIDER_SECRET_SCAN=PASS')
PY

archive_prefix="${package_name}-${source_commit:0:12}"
tar_path="$output_dir/${archive_prefix}.tar.gz"
zip_path="$output_dir/${archive_prefix}.zip"
(
  cd "$output_dir"
  tar --sort=name --mtime='UTC 1970-01-01' --owner=0 --group=0 --numeric-owner \
    -czf "$tar_path" "$package_name"
)
python3 - "$package_dir" "$zip_path" <<'PY'
from __future__ import annotations
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo
import sys
root = Path(sys.argv[1])
out = Path(sys.argv[2])
base = root.name
with ZipFile(out, 'w', compression=ZIP_DEFLATED, compresslevel=9) as zf:
    for path in sorted(root.rglob('*')):
        if not path.is_file():
            continue
        rel = Path(base) / path.relative_to(root)
        info = ZipInfo(rel.as_posix(), date_time=(1980, 1, 1, 0, 0, 0))
        mode = path.stat().st_mode & 0o777
        info.external_attr = mode << 16
        zf.writestr(info, path.read_bytes(), compress_type=ZIP_DEFLATED, compresslevel=9)
PY
(
  cd "$output_dir"
  sha256sum "$(basename "$tar_path")" "$(basename "$zip_path")" > "${archive_prefix}.sha256"
)
package_complete=1
printf 'PACKAGE_DIR=%s\nTAR=%s\nZIP=%s\n' "$package_dir" "$tar_path" "$zip_path"
