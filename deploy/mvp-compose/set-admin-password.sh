#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT"
initial=0
if [ "${1:-}" = --initial ]; then
  initial=1
elif [ "$#" -ne 0 ]; then
  printf 'usage: %s [--initial]\n' "$0" >&2
  exit 64
fi

docker info >/dev/null
verifier=secrets/admin-password.phc
if [ "$initial" -eq 1 ] && [ -e "$verifier" ]; then
  printf 'local administrator password verifier already exists\n' >&2
  exit 73
fi
if [ -L secrets ] || { [ -e secrets ] && [ ! -d secrets ]; }; then
  printf 'secrets must be a regular directory, not a symlink\n' >&2
  exit 73
fi
if [ -e "$verifier" ]; then
  [ -f "$verifier" ] && [ ! -L "$verifier" ] || {
    printf 'local administrator password verifier must be a regular file\n' >&2
    exit 73
  }
  printf 'Type ROTATE to replace the local administrator password: '
  IFS= read -r answer
  [ "$answer" = ROTATE ] || { printf 'password rotation cancelled\n' >&2; exit 64; }
fi
printf 'New local administrator password: '
stty -echo 2>/dev/null || true
IFS= read -r password
stty echo 2>/dev/null || true
printf '\nConfirm local administrator password: '
stty -echo 2>/dev/null || true
IFS= read -r confirmation
stty echo 2>/dev/null || true
printf '\n'
[ "$password" = "$confirmation" ] || { unset password confirmation; printf 'passwords do not match\n' >&2; exit 64; }
[ "${#password}" -ge 12 ] || { unset password confirmation; printf 'password must contain at least 12 characters\n' >&2; exit 64; }
encoded=$(printf '%s' "$password" | base64 | tr -d '\r\n')
unset password confirmation

docker compose build mvp
image_id=$(docker compose images -q mvp)
if [[ -z "$image_id" || "$image_id" == *' '* || "$image_id" == *$'\n'* ]]; then
  printf 'could not resolve exactly one built MVP image\n' >&2
  exit 1
fi
hash=$(printf '%s\n' "$encoded" | docker run --rm -i --pull never --read-only --cap-drop ALL --security-opt no-new-privileges --user 65532:65532 --entrypoint /app/ustc-agentctl "$image_id" admin hash-password)
unset encoded
case "$hash" in
  '$argon2id$v=19$m=19456,t=2,p=1$'*) ;;
  *) printf 'password hashing command returned an invalid verifier\n' >&2; exit 65 ;;
esac
mkdir -p secrets
chmod 700 secrets
temporary="secrets/.admin-password.$$.tmp"
umask 077
printf '%s' "$hash" > "$temporary"
unset hash
chmod 600 "$temporary"
mv -f -- "$temporary" "$verifier"
printf 'Local deployment access password verifier updated.\n'
if [ "$initial" -eq 0 ]; then
  docker compose up -d --force-recreate --no-deps mvp
  printf 'Existing browser sessions were invalidated. Run ./start.sh to verify readiness.\n'
fi
