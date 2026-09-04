#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT"

normalize_value() {
  local value=$1
  value=${value%$'\r'}
  while [[ "$value" == [[:space:]]* ]]; do value=${value#?}; done
  while [[ "$value" == *[[:space:]] ]]; do value=${value%?}; done
  case "$value" in
    \"*\") value=${value#\"}; value=${value%\"} ;;
    \'*\') value=${value#\'}; value=${value%\'} ;;
  esac
  printf '%s' "$value"
}

normalize_dotenv_value() {
  local value=$1
  value=${value%$'\r'}
  while [[ "$value" == [[:space:]]* ]]; do value=${value#?}; done
  while [[ "$value" == *[[:space:]] ]]; do value=${value%?}; done
  case "$value" in
    \"*)
      if [[ "$value" =~ ^\"([^\"]*)\"[[:space:]]*(#.*)?$ ]]; then
        value=${BASH_REMATCH[1]}
      fi
      ;;
    \'*)
      if [[ "$value" =~ ^\'([^\']*)\'[[:space:]]*(#.*)?$ ]]; then
        value=${BASH_REMATCH[1]}
      fi
      ;;
    *)
      if [[ "$value" =~ ^(.*[^[:space:]])[[:space:]]+#.*$ ]]; then
        value=${BASH_REMATCH[1]}
      fi
      ;;
  esac
  if [[ "$value" == *'$'* ]]; then
    printf 'Compose interpolation is not supported for security-critical .env values; use a literal value\n' >&2
    return 66
  fi
  normalize_value "$value"
}

trim_rust_whitespace() {
  local value=$1
  local matched ws
  local unicode_ws=(
    $'\xC2\x85' $'\xC2\xA0' $'\xE1\x9A\x80'
    $'\xE2\x80\x80' $'\xE2\x80\x81' $'\xE2\x80\x82' $'\xE2\x80\x83'
    $'\xE2\x80\x84' $'\xE2\x80\x85' $'\xE2\x80\x86' $'\xE2\x80\x87'
    $'\xE2\x80\x88' $'\xE2\x80\x89' $'\xE2\x80\x8A' $'\xE2\x80\xA8'
    $'\xE2\x80\xA9' $'\xE2\x80\xAF' $'\xE2\x81\x9F' $'\xE3\x80\x80'
  )
  while :; do
    case "$value" in
      [[:space:]]*) value=${value#?}; continue ;;
    esac
    matched=0
    for ws in "${unicode_ws[@]}"; do
      case "$value" in "$ws"*) value=${value#"$ws"}; matched=1; break ;; esac
    done
    [ "$matched" -eq 1 ] || break
  done
  while :; do
    case "$value" in
      *[[:space:]]) value=${value%?}; continue ;;
    esac
    matched=0
    for ws in "${unicode_ws[@]}"; do
      case "$value" in *"$ws") value=${value%"$ws"}; matched=1; break ;; esac
    done
    [ "$matched" -eq 1 ] || break
  done
  printf '%s' "$value"
}

validate_dotenv_contract() {
  local line trimmed key
  local provider_count=0
  local key_source_count=0
  if [ -L .env ]; then
    printf '.env must be a regular non-symlink file\n' >&2
    return 66
  fi
  if [ ! -e .env ]; then
    return 0
  fi
  if [ ! -f .env ] || [ ! -r .env ]; then
    printf '.env must be a readable regular non-symlink file\n' >&2
    return 66
  fi
  while IFS= read -r line || [ -n "$line" ]; do
    trimmed=$(trim_rust_whitespace "$line")
    if [[ "$trimmed" =~ ^(export[[:space:]]+)?(UCA_AGENT_PROVIDER|UCA_AGENT_API_KEY_SOURCE)([[:space:]]*=|[[:space:]]*$) ]]; then
      key=${BASH_REMATCH[2]}
      case "$line" in
        "$key"=*) ;;
        *)
          printf '%s must use an exact column-zero KEY=value assignment in .env\n' "$key" >&2
          return 66
          ;;
      esac
      case "$key" in
        UCA_AGENT_PROVIDER)
          provider_count=$((provider_count + 1))
          [ "$provider_count" -eq 1 ] || {
            printf 'duplicate UCA_AGENT_PROVIDER definitions are forbidden in .env\n' >&2
            return 66
          }
          ;;
        UCA_AGENT_API_KEY_SOURCE)
          key_source_count=$((key_source_count + 1))
          [ "$key_source_count" -eq 1 ] || {
            printf 'duplicate UCA_AGENT_API_KEY_SOURCE definitions are forbidden in .env\n' >&2
            return 66
          }
          ;;
      esac
      normalize_dotenv_value "${line#*=}" >/dev/null || return
    fi
  done < .env
}

dotenv_value() {
  local key=$1
  local line
  [ -f .env ] && [ ! -L .env ] || return 1
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      "$key"=*) normalize_dotenv_value "${line#*=}"; return ;;
    esac
  done < .env
  return 1
}

validate_dotenv_contract

if [ "${UCA_AGENT_PROVIDER+x}" = x ]; then
  provider=$(normalize_value "$UCA_AGENT_PROVIDER")
else
  dotenv_status=0
  provider=$(dotenv_value UCA_AGENT_PROVIDER) || dotenv_status=$?
  case "$dotenv_status" in
    0) ;;
    1) provider=mock ;;
    *) exit "$dotenv_status" ;;
  esac
fi
if [ -z "$provider" ]; then
  printf 'UCA_AGENT_PROVIDER must not be empty\n' >&2
  exit 66
fi
if [ "$provider" = openai-compatible ]; then
  if [ "${UCA_AGENT_API_KEY_SOURCE+x}" = x ]; then
    key_source=$(normalize_value "$UCA_AGENT_API_KEY_SOURCE")
  else
    dotenv_status=0
    key_source=$(dotenv_value UCA_AGENT_API_KEY_SOURCE) || dotenv_status=$?
    case "$dotenv_status" in
      0) ;;
      1) key_source='' ;;
      *) exit "$dotenv_status" ;;
    esac
  fi
  [ -n "$key_source" ] || {
    printf 'UCA_AGENT_API_KEY_SOURCE is required in openai-compatible mode\n' >&2
    exit 66
  }
  case "$key_source" in
    /*) key_path=$key_source ;;
    *) key_path=$ROOT/$key_source ;;
  esac
  if [ ! -f "$key_path" ] || [ ! -r "$key_path" ] || [ -L "$key_path" ]; then
    printf 'provider key source must be a readable regular non-symlink file\n' >&2
    exit 66
  fi
  key_value=$(trim_rust_whitespace "$(cat "$key_path")")
  if [ "$key_value" = unused-placeholder-for-deterministic-mock-mode ]; then
    printf 'the bundled mock provider placeholder is forbidden in openai-compatible mode\n' >&2
    exit 66
  fi
  if key_mode=$(stat -c '%a' "$key_path" 2>/dev/null); then
    :
  elif key_mode=$(stat -f '%Lp' "$key_path" 2>/dev/null); then
    :
  else
    printf 'cannot inspect provider key source permissions\n' >&2
    exit 66
  fi
  case "$key_mode" in
    [0-7]00) ;;
    *) printf 'provider key source must have no group/world permission bits\n' >&2; exit 66 ;;
  esac
fi

docker info >/dev/null
docker compose up --build -d
published=$(docker compose port mvp 8787)
case "$published" in
  127.0.0.1:*) port=${published##*:} ;;
  *) printf 'unexpected Compose published address: %s\n' "$published" >&2; exit 1 ;;
esac
case "$port" in
  ''|*[!0-9]*) printf 'invalid Compose published port: %s\n' "$port" >&2; exit 1 ;;
esac
if [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
  printf 'invalid Compose published port: %s\n' "$port" >&2
  exit 1
fi
url="http://127.0.0.1:${port}"
i=0
while [ "$i" -lt 150 ]; do
  health=$(curl --connect-timeout 1 --max-time 1 --fail --silent "${url}/healthz" || true)
  if printf '%s' "$health" | grep -Fq '"schema":"ustc-agentd-health/v1"' \
    && printf '%s' "$health" | grep -Fq '"status":"ok"'; then
    printf 'MVP is ready: %s\n' "$url"
    exit 0
  fi
  sleep 1
  i=$((i + 1))
done
docker compose ps
docker compose logs --no-color --tail 120
printf 'MVP did not become healthy within 5 minutes\n' >&2
exit 1
