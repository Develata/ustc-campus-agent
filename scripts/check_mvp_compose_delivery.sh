#!/usr/bin/env bash
set -euo pipefail

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"
source_commit=$(git rev-parse --verify HEAD)
if [ "${#source_commit}" -ne 40 ]; then
  printf 'source commit must be a full 40-hex identity\n' >&2
  exit 64
fi
case "$source_commit" in
  *[!0-9a-f]*) printf 'source commit must be lowercase hex\n' >&2; exit 64 ;;
esac

work=$(mktemp -d)
cleanup() {
  rm -rf -- "$work"
}
trap cleanup EXIT
mkdir -p "$work/readback"

UCA_SOURCE_COMMIT="$source_commit" \
  cargo build --locked --release -p ustc-agentd --bin ustc-agentd
for output in "$work/package-a" "$work/package-b"; do
  scripts/package_three_plugin_mvp_compose.sh \
    --binary target/release/ustc-agentd \
    --output-dir "$output" \
    --source-commit "$source_commit"
done
short_commit=${source_commit:0:12}
cmp "$work/package-a/ustc-campus-agent-mvp-compose-${short_commit}.zip" \
    "$work/package-b/ustc-campus-agent-mvp-compose-${short_commit}.zip"
cmp "$work/package-a/ustc-campus-agent-mvp-compose-${short_commit}.tar.gz" \
    "$work/package-b/ustc-campus-agent-mvp-compose-${short_commit}.tar.gz"
(
  cd "$work/package-a"
  sha256sum -c "ustc-campus-agent-mvp-compose-${short_commit}.sha256"
)
tar -xzf "$work/package-a/ustc-campus-agent-mvp-compose-${short_commit}.tar.gz" \
  -C "$work/readback"
(
  cd "$work/readback/ustc-campus-agent-mvp-compose"
  sha256sum -c SHA256SUMS
  test "$(bin/ustc-agentd source-commit)" = "$source_commit"
  cmp "$repo_root/LICENSE.md" LICENSE.md
  grep -Fxq 'source_repository=https://github.com/Develata/ustc-campus-agent' BUILD-INFO.txt
  grep -Fxq 'license=MIT' BUILD-INFO.txt
  grep -Fq 'https://github.com/Develata/ustc-campus-agent' README.md
  grep -Fq 'MIT License' README.md
  grep -Eq '^[0-9a-f]{64}  LICENSE\.md$' SHA256SUMS
  mkdir -p "$work/fake-bin" "$work/launcher-secrets"
  printf '%s\n' '#!/usr/bin/env sh' ': > "${DOCKER_MARKER:?}"' 'exit 23' \
    > "$work/fake-bin/docker"
  chmod 0755 "$work/fake-bin/docker"
  printf '%s\n' 'non-secret-launcher-test-value' > "$work/launcher-secrets/key.txt"
  chmod 0644 "$work/launcher-secrets/key.txt"
  rm -f "$work/docker-called"
  if UCA_AGENT_PROVIDER=openai-compatible \
    UCA_AGENT_API_KEY_SOURCE="$work/launcher-secrets/key.txt" \
    DOCKER_MARKER="$work/docker-called" \
    PATH="$work/fake-bin:$PATH" \
    ./start.sh > "$work/unsafe-key.out" 2>&1; then
    printf 'start.sh accepted an unsafe provider key mode\n' >&2
    exit 1
  fi
  grep -Fq 'no group/world permission bits' "$work/unsafe-key.out"
  test ! -e "$work/docker-called"

  cp .env "$work/original.env"
  unicode_whitespace=(
    $'\xC2\x85' $'\xC2\xA0' $'\xE1\x9A\x80'
    $'\xE2\x80\x80' $'\xE2\x80\x81' $'\xE2\x80\x82'
    $'\xE2\x80\x83' $'\xE2\x80\x84' $'\xE2\x80\x85'
    $'\xE2\x80\x86' $'\xE2\x80\x87' $'\xE2\x80\x88'
    $'\xE2\x80\x89' $'\xE2\x80\x8A' $'\xE2\x80\xA8'
    $'\xE2\x80\xA9' $'\xE2\x80\xAF' $'\xE2\x81\x9F'
    $'\xE3\x80\x80'
  )
  printf '%s\r\n' \
    'UCA_AGENT_PROVIDER="openai-compatible"' \
    'UCA_AGENT_API_KEY_SOURCE="./normalized-mock-provider-key.txt"' > .env
  unicode_case=0
  for whitespace in "${unicode_whitespace[@]}"; do
    unicode_case=$((unicode_case + 1))
    printf '%s%s%s\r\n' \
      "$whitespace" \
      'unused-placeholder-for-deterministic-mock-mode' \
      "$whitespace" > normalized-mock-provider-key.txt
    chmod 0600 normalized-mock-provider-key.txt
    rm -f "$work/docker-called"
    if (
      unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE
      LC_ALL=C DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
    ) > "$work/unicode-whitespace-${unicode_case}.out" 2>&1; then
      printf 'start.sh accepted Unicode-whitespace placeholder case %s\n' "$unicode_case" >&2
      exit 1
    fi
    grep -Fq 'bundled mock provider placeholder is forbidden' \
      "$work/unicode-whitespace-${unicode_case}.out"
    test ! -e "$work/docker-called"
  done
  test "$unicode_case" -eq 19
  mv "$work/original.env" .env
  rm -f normalized-mock-provider-key.txt
  printf 'UNICODE_WHITESPACE_MATRIX=PASS cases=%s\n' "$unicode_case"

  cp .env "$work/original.env"
  printf '%s\n' \
    'UCA_AGENT_PROVIDER=openai-compatible # local provider picked by $MODE' \
    "UCA_AGENT_API_KEY_SOURCE=$work/launcher-secrets/key.txt" > .env
  rm -f "$work/docker-called"
  if (
    unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE
    DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
  ) > "$work/commented-real-unsafe.out" 2>&1; then
    printf 'start.sh accepted an unsafe key through a commented real-provider value\n' >&2
    exit 1
  fi
  grep -Fq 'no group/world permission bits' "$work/commented-real-unsafe.out"
  test ! -e "$work/docker-called"

  commented_key="$work/launcher-secrets/key # local.txt"
  printf '%s\n' 'non-secret-launcher-test-value' > "$commented_key"
  chmod 0600 "$commented_key"
  printf '%s\n' \
    'UCA_AGENT_PROVIDER="openai-compatible" # local provider' \
    "UCA_AGENT_API_KEY_SOURCE=\"$commented_key\" # owner-only source" > .env
  rm -f "$work/docker-called"
  if (
    unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE
    DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
  ) > "$work/commented-real-safe.out" 2>&1; then
    printf 'fake docker unexpectedly succeeded for commented real-provider values\n' >&2
    exit 1
  fi
  test -e "$work/docker-called"

  printf '%s\n' 'UCA_AGENT_PROVIDER=mock # local provider' > .env
  rm -f "$work/docker-called"
  if (
    unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE
    DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
  ) > "$work/commented-mock.out" 2>&1; then
    printf 'fake docker unexpectedly succeeded for commented mock-provider value\n' >&2
    exit 1
  fi
  test -e "$work/docker-called"

  printf '%s\n' \
    'UCA_AGENT_PROVIDER=openai-compatible' \
    "UCA_AGENT_API_KEY_SOURCE=\"$commented_key\"" > .env
  rm -f "$work/docker-called"
  if (
    unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE
    DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
  ) > "$work/quoted-hash-path.out" 2>&1; then
    printf 'fake docker unexpectedly succeeded for quoted hash path\n' >&2
    exit 1
  fi
  test -e "$work/docker-called"

  printf 'DOTENV_LITERAL_MATRIX=PASS cases=4\n'

  assert_dotenv_rejected() {
    local name=$1
    local expected=$2
    local output
    shift 2
    printf '%s\n' "$@" > .env
    rm -f "$work/docker-called"
    if (
      unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE MODE KEY_SOURCE
      LC_ALL=C DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
    ) > "$work/$name.out" 2>&1; then
      printf 'start.sh unexpectedly accepted unsafe .env case %s\n' "$name" >&2
      exit 1
    fi
    test ! -e "$work/docker-called"
    output=$(<"$work/$name.out")
    case "$output" in
      *"$expected"*) ;;
      *) printf 'start.sh reported the wrong failure for .env case %s\n' "$name" >&2; exit 1 ;;
    esac
  }

  assert_dotenv_rejected provider-interpolation \
    'Compose interpolation is not supported for security-critical .env values' \
    'UCA_AGENT_PROVIDER=${MODE:-openai-compatible}' \
    "UCA_AGENT_API_KEY_SOURCE=$work/unsafe-key.txt"
  assert_dotenv_rejected key-source-interpolation \
    'Compose interpolation is not supported for security-critical .env values' \
    'UCA_AGENT_PROVIDER=mock' \
    'UCA_AGENT_API_KEY_SOURCE=${KEY_SOURCE:-./secrets/key.txt}'
  assert_dotenv_rejected duplicate-provider \
    'duplicate UCA_AGENT_PROVIDER definitions are forbidden in .env' \
    'UCA_AGENT_PROVIDER=mock' \
    'UCA_AGENT_PROVIDER=openai-compatible'
  assert_dotenv_rejected duplicate-key-source \
    'duplicate UCA_AGENT_API_KEY_SOURCE definitions are forbidden in .env' \
    'UCA_AGENT_PROVIDER=mock' \
    'UCA_AGENT_API_KEY_SOURCE=./first.txt' \
    'UCA_AGENT_API_KEY_SOURCE=./second.txt'
  assert_dotenv_rejected leading-provider \
    'UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env' \
    ' UCA_AGENT_PROVIDER=openai-compatible'
  assert_dotenv_rejected bare-provider \
    'UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env' \
    'UCA_AGENT_PROVIDER'
  assert_dotenv_rejected export-provider \
    'UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env' \
    'export UCA_AGENT_PROVIDER=openai-compatible'
  assert_dotenv_rejected spaced-equals-provider \
    'UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env' \
    'UCA_AGENT_PROVIDER =openai-compatible'
  assert_dotenv_rejected unicode-leading-provider \
    'UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env' \
    $'\xC2\xA0UCA_AGENT_PROVIDER=openai-compatible'

  printf '%s\n' 'UCA_AGENT_PROVIDER=openai-compatible' > "$work/symlinked.env"
  rm -f .env
  ln -s "$work/symlinked.env" .env
  rm -f "$work/docker-called"
  if (
    unset UCA_AGENT_PROVIDER UCA_AGENT_API_KEY_SOURCE
    DOCKER_MARKER="$work/docker-called" PATH="$work/fake-bin:$PATH" ./start.sh
  ) > "$work/symlinked-dotenv.out" 2>&1; then
    printf 'start.sh unexpectedly accepted a symlinked .env\n' >&2
    exit 1
  fi
  test ! -e "$work/docker-called"
  case "$(<"$work/symlinked-dotenv.out")" in
    *'.env must be a regular non-symlink file'*) ;;
    *) printf 'start.sh reported the wrong failure for symlinked .env\n' >&2; exit 1 ;;
  esac
  rm .env

  mv "$work/original.env" .env
  printf 'DOTENV_FAIL_CLOSED_MATRIX=PASS cases=10\n'

  chmod 0600 "$work/launcher-secrets/key.txt"
  rm -f "$work/docker-called"
  if UCA_AGENT_PROVIDER=openai-compatible \
    UCA_AGENT_API_KEY_SOURCE="$work/launcher-secrets/key.txt" \
    DOCKER_MARKER="$work/docker-called" \
    PATH="$work/fake-bin:$PATH" \
    ./start.sh > "$work/safe-key.out" 2>&1; then
    printf 'fake docker unexpectedly succeeded\n' >&2
    exit 1
  fi
  test -e "$work/docker-called"
  UCA_MVP_PORT=${UCA_MVP_PORT:-18789} ./smoke.sh
)
printf 'MVP_COMPOSE_DELIVERY=PASS source_commit=%s\n' "$source_commit"
