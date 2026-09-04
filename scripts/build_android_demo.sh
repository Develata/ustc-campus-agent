#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf '%s\n' "usage: $0 [--output-dir PATH] [--source-commit HEX|local]"
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project_dir="$repo_root/apps/ustc-android-demo"
output_dir="$repo_root/dist/android"
source_commit="${UCA_SOURCE_COMMIT:-}"
build_tools_version="${ANDROID_BUILD_TOOLS_VERSION:-36.0.0}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-dir)
      test "$#" -ge 2 || { usage >&2; exit 64; }
      output_dir="$2"
      shift 2
      ;;
    --source-commit)
      test "$#" -ge 2 || { usage >&2; exit 64; }
      source_commit="$2"
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

if [ -z "$source_commit" ]; then
  source_commit="$(git -C "$repo_root" rev-parse HEAD)"
fi
if [ "$source_commit" != local ] && ! [[ "$source_commit" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'invalid source commit: %s\n' "$source_commit" >&2
  exit 65
fi

android_home="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [ -z "$android_home" ]; then
  printf '%s\n' 'ANDROID_HOME or ANDROID_SDK_ROOT is required' >&2
  exit 69
fi
apksigner="$android_home/build-tools/$build_tools_version/apksigner"
aapt2="$android_home/build-tools/$build_tools_version/aapt2"
for executable in "$project_dir/gradlew" "$apksigner" "$aapt2"; do
  if [ ! -x "$executable" ]; then
    printf 'required executable missing: %s\n' "$executable" >&2
    exit 69
  fi
done

mkdir -p "$output_dir"
set +e
"$project_dir/gradlew" \
  --project-dir "$project_dir" \
  --no-daemon \
  -PucaSourceCommit="$source_commit" \
  :app:testDebugUnitTest \
  :app:lintDebug \
  :app:assembleDebug
gradle_rc=$?
set -e
for report in \
  "$project_dir/app/build/reports/lint-results-debug.txt" \
  "$project_dir/app/build/reports/lint-results-debug.sarif"; do
  if [ -f "$report" ]; then
    cp "$report" "$output_dir/"
  fi
done
if [ "$gradle_rc" -ne 0 ]; then
  printf 'android Gradle gate failed: exit=%s\n' "$gradle_rc" >&2
  exit "$gradle_rc"
fi

built_apk="$project_dir/app/build/outputs/apk/debug/app-debug.apk"
test -s "$built_apk"
artifact="ustc-campus-agent-android-debug-${source_commit}.apk"
artifact_path="$output_dir/$artifact"
cp "$built_apk" "$artifact_path"

"$apksigner" verify --verbose --print-certs "$artifact_path" \
  > "$output_dir/apksigner.txt"
"$aapt2" dump badging "$artifact_path" \
  > "$output_dir/aapt2-badging.txt"
grep -Fq "package: name='com.develata.ustccampusagent.debug'" \
  "$output_dir/aapt2-badging.txt"
grep -Fq "launchable-activity: name='com.develata.ustccampusagent.MainActivity'" \
  "$output_dir/aapt2-badging.txt"
(
  cd "$output_dir"
  sha256sum "$artifact" > "$artifact.sha256"
)

EVIDENCE_DIR="$output_dir" \
APK_ARTIFACT="$artifact" \
SOURCE_COMMIT="$source_commit" \
python3 -c 'import json, os, pathlib; pathlib.Path(os.environ["EVIDENCE_DIR"], "build-info.json").write_text(json.dumps({"schema":"ustc-android-build/v1","source_commit":os.environ["SOURCE_COMMIT"],"artifact":os.environ["APK_ARTIFACT"],"build_type":"debug","application_id":"com.develata.ustccampusagent.debug","gradle":"9.6.0","agp":"9.4.0","compile_sdk":36,"target_sdk":36,"min_sdk":26}, sort_keys=True, indent=2)+"\n")'

printf 'android-build: PASS\n'
printf 'artifact=%s\n' "$artifact_path"
printf 'source_commit=%s\n' "$source_commit"
