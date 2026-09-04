#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf '%s\n' "usage: $0 --apk PATH --server-binary PATH --evidence-dir PATH [--api-level N]"
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
apk=""
server_binary=""
evidence_dir=""
api_level="35"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --apk)
      test "$#" -ge 2 || { usage >&2; exit 64; }
      apk="$2"
      shift 2
      ;;
    --server-binary)
      test "$#" -ge 2 || { usage >&2; exit 64; }
      server_binary="$2"
      shift 2
      ;;
    --evidence-dir)
      test "$#" -ge 2 || { usage >&2; exit 64; }
      evidence_dir="$2"
      shift 2
      ;;
    --api-level)
      test "$#" -ge 2 || { usage >&2; exit 64; }
      api_level="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 64
      ;;
  esac
done

for path in "$apk" "$server_binary"; do
  if [ -z "$path" ] || [ ! -f "$path" ]; then
    printf 'required file missing: %s\n' "$path" >&2
    exit 66
  fi
done
if [ ! -x "$server_binary" ]; then
  printf 'server binary is not executable: %s\n' "$server_binary" >&2
  exit 69
fi
if [ -z "$evidence_dir" ]; then
  usage >&2
  exit 64
fi

android_home="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [ -z "$android_home" ]; then
  printf '%s\n' 'ANDROID_HOME or ANDROID_SDK_ROOT is required' >&2
  exit 69
fi
adb="$android_home/platform-tools/adb"
avdmanager="$android_home/cmdline-tools/latest/bin/avdmanager"
emulator="$android_home/emulator/emulator"
for executable in "$adb" curl python3 timeout "$avdmanager" "$emulator"; do
  if ! command -v "$executable" >/dev/null 2>&1 && [ ! -x "$executable" ]; then
    printf 'required executable missing: %s\n' "$executable" >&2
    exit 69
  fi
done

mkdir -p "$evidence_dir"
run_root="$(mktemp -d "${RUNNER_TEMP:-/tmp}/uca-android-smoke.XXXXXX")"
state_dir="$run_root/state"
export ANDROID_AVD_HOME="$run_root/avd"
mkdir -p "$state_dir" "$ANDROID_AVD_HOME"
chmod 0700 "$state_dir"
avd_name="uca-android-smoke-${GITHUB_RUN_ID:-$$}"
serial="emulator-5554"
server_pid=""
emulator_pid=""

cleanup() {
  "$adb" -s "$serial" emu kill >/dev/null 2>&1 || true
  if [ -n "$emulator_pid" ]; then
    kill "$emulator_pid" >/dev/null 2>&1 || true
    wait "$emulator_pid" >/dev/null 2>&1 || true
  fi
  if [ -n "$server_pid" ]; then
    kill "$server_pid" >/dev/null 2>&1 || true
    wait "$server_pid" >/dev/null 2>&1 || true
  fi
  "$avdmanager" delete avd --name "$avd_name" >/dev/null 2>&1 || true
  rm -rf "$run_root"
}
trap cleanup EXIT

"$server_binary" serve-web \
  --bind 127.0.0.1:8787 \
  --fixture "$repo_root/fixtures/affairs/proc-011-reviewed.json" \
  --change-fixture "$repo_root/fixtures/change-radar/academic-calendar-demo-reviewed.json" \
  --opportunity-fixture "$repo_root/fixtures/opportunity-graph/course-planning-demo-reviewed.json" \
  --opportunity-catalog "$repo_root/market/fixtures/course-planning/minimal-v0.json" \
  --opportunity-profile-store "$state_dir/opportunity-profiles.json" \
  --store "$state_dir/affairs-records.json" \
  --idempotency "$state_dir/affairs-idempotency.json" \
  --session-store "$state_dir/m00-sessions.json" \
  >"$evidence_dir/server.log" 2>&1 &
server_pid=$!
server_ready=0
for _ in $(seq 1 60); do
  if curl --fail --silent --show-error http://127.0.0.1:8787/ >/dev/null; then
    server_ready=1
    break
  fi
  sleep 1
done
test "$server_ready" = 1

if "$adb" devices | grep -Eq '^emulator-[0-9]+[[:space:]]+device$'; then
  printf '%s\n' 'an emulator is already running; refusing ambiguous device ownership' >&2
  exit 70
fi
printf 'no\n' | "$avdmanager" create avd \
  --force \
  --name "$avd_name" \
  --package "system-images;android-${api_level};google_apis;x86_64"
avd_config="$ANDROID_AVD_HOME/$avd_name.avd/config.ini"
python3 - "$avd_config" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
key = "disk.dataPartition.size="
replacement = f"{key}1G"
updated = [replacement if line.startswith(key) else line for line in lines]
if not any(line.startswith(key) for line in lines):
    updated.append(replacement)
path.write_text("\n".join(updated) + "\n", encoding="utf-8")
PY
grep -Fqx 'disk.dataPartition.size=1G' "$avd_config"
"$emulator" \
  -avd "$avd_name" \
  -port 5554 \
  -no-window \
  -no-audio \
  -no-boot-anim \
  -gpu swiftshader_indirect \
  -partition-size 1024 \
  -no-snapshot \
  -wipe-data \
  >"$evidence_dir/emulator.log" 2>&1 &
emulator_pid=$!

if ! timeout 240 "$adb" -s "$serial" wait-for-device; then
  printf '%s\n' 'emulator did not expose an adb device within 240 seconds' >&2
  exit 70
fi
booted=0
for _ in $(seq 1 180); do
  if [ "$("$adb" -s "$serial" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ]; then
    booted=1
    break
  fi
  sleep 2
done
test "$booted" = 1

"$adb" -s "$serial" shell settings put global window_animation_scale 0
"$adb" -s "$serial" shell settings put global transition_animation_scale 0
"$adb" -s "$serial" shell settings put global animator_duration_scale 0
"$adb" -s "$serial" reverse tcp:8787 tcp:8787
"$adb" -s "$serial" install --no-streaming -r "$apk" | tee "$evidence_dir/adb-install.txt"
grep -Fq Success "$evidence_dir/adb-install.txt"

package=com.develata.ustccampusagent.debug
component="$package/com.develata.ustccampusagent.MainActivity"
"$adb" -s "$serial" shell am start -W -n "$component" \
  | tee "$evidence_dir/activity-start.txt"
grep -Fq 'Status: ok' "$evidence_dir/activity-start.txt"

app_pid=""
for _ in $(seq 1 30); do
  app_pid="$("$adb" -s "$serial" shell pidof "$package" 2>/dev/null | tr -d '\r' || true)"
  if [ -n "$app_pid" ]; then
    break
  fi
  sleep 1
done
test -n "$app_pid"
printf 'app_pid=%s\n' "$app_pid" > "$evidence_dir/runtime.txt"
printf 'serial=%s\n' "$serial" >> "$evidence_dir/runtime.txt"

notice_visible=0
for _ in $(seq 1 30); do
  if "$adb" -s "$serial" shell uiautomator dump /sdcard/uca-window.xml >/dev/null 2>&1 \
    && "$adb" -s "$serial" exec-out cat /sdcard/uca-window.xml \
      > "$evidence_dir/android-ui.xml" \
    && grep -Eq 'resource-id="[^"]*:id/prototype_disclaimer"' "$evidence_dir/android-ui.xml" \
    && grep -Fq '学生竞赛原型 · 非官方 USTC 服务' "$evidence_dir/android-ui.xml"; then
    notice_visible=1
    break
  fi
  sleep 1
done
"$adb" -s "$serial" shell rm -f /sdcard/uca-window.xml >/dev/null 2>&1 || true
test "$notice_visible" = 1

socket_name=""
for _ in $(seq 1 30); do
  socket_name="$("$adb" -s "$serial" shell cat /proc/net/unix \
    | tr -d '\r' \
    | grep -oE 'webview_devtools_remote_[^[:space:]]+' \
    | sort -u || true)"
  if [ -n "$socket_name" ] && [ "$(printf '%s\n' "$socket_name" | wc -l)" -eq 1 ]; then
    break
  fi
  sleep 1
done
test -n "$socket_name"
test "$(printf '%s\n' "$socket_name" | wc -l)" -eq 1
"$adb" -s "$serial" forward tcp:9222 "localabstract:$socket_name"
python3 "$repo_root/scripts/test_android_webview_cdp.py" \
  --devtools http://127.0.0.1:9222 \
  --origin http://127.0.0.1:8787/ \
  --timeout-seconds 90 \
  | tee "$evidence_dir/android-webview-smoke.txt"
"$adb" -s "$serial" exec-out screencap -p > "$evidence_dir/android-emulator.png"
test -s "$evidence_dir/android-emulator.png"

# Prove that the disclaimer belongs to the native shell and remains visible
# when the loopback Rust service is unavailable, rather than matching only the
# Web page copy in an online hierarchy dump.
kill "$server_pid"
wait "$server_pid" || true
server_pid=""
"$adb" -s "$serial" shell am force-stop "$package"
"$adb" -s "$serial" shell am start -W -n "$component" \
  | tee "$evidence_dir/activity-start-offline.txt"
grep -Fq 'Status: ok' "$evidence_dir/activity-start-offline.txt"

offline_notice_visible=0
for _ in $(seq 1 30); do
  if "$adb" -s "$serial" shell uiautomator dump /sdcard/uca-window-offline.xml >/dev/null 2>&1 \
    && "$adb" -s "$serial" exec-out cat /sdcard/uca-window-offline.xml \
      > "$evidence_dir/android-ui-offline.xml" \
    && grep -Eq 'resource-id="[^"]*:id/prototype_disclaimer"' "$evidence_dir/android-ui-offline.xml" \
    && grep -Fq '学生竞赛原型 · 非官方 USTC 服务' "$evidence_dir/android-ui-offline.xml"; then
    offline_notice_visible=1
    break
  fi
  sleep 1
done
"$adb" -s "$serial" shell rm -f /sdcard/uca-window-offline.xml >/dev/null 2>&1 || true
test "$offline_notice_visible" = 1

printf 'android-emulator-smoke: PASS\n'
