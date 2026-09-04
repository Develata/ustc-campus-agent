#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT"
work=$(mktemp -d)
project="uca-mvp-smoke-$$"
compose() {
  docker compose --project-name "$project" "$@"
}
curl_request() {
  curl --connect-timeout 2 --max-time 10 --fail --silent --show-error "$@"
}
curl_health() {
  curl --connect-timeout 1 --max-time 1 --fail --silent "$@"
}
auth_curl_request() {
  curl_request -b "$work/session.cookies" "$@"
}
login_local_access() {
  python3 - "$work/login-request.json" <<'PY'
import json, pathlib, sys
pathlib.Path(sys.argv[1]).write_text(json.dumps({
    'schema': 'ustc-local-access-login/v1',
    'username': 'admin',
    'password': 'compose smoke password',
}), encoding='utf-8')
PY
  curl_request -c "$work/session.cookies" \
    -H 'content-type: application/json' \
    --data-binary "@$work/login-request.json" \
    "$base/api/v1/auth/login" > "$work/login.json"
}
cleanup() {
  compose down --volumes --remove-orphans >/dev/null 2>&1 || true
  rm -rf "$work"
}
trap cleanup EXIT

UCA_ADMIN_PASSWORD_HASH_SOURCE=./mock-provider-key.txt compose config --quiet
UCA_ADMIN_PASSWORD_HASH_SOURCE=./mock-provider-key.txt compose build --pull=false
admin_hash=$(
  printf '%s\n' 'Y29tcG9zZSBzbW9rZSBwYXNzd29yZA==' | \
    docker run --rm -i --pull never --read-only --cap-drop ALL \
      --security-opt no-new-privileges --user 65532:65532 \
      --entrypoint /app/ustc-agentctl ustc-campus-agent-mvp:0.1.0 \
      admin hash-password
)
case "$admin_hash" in
  '$argon2id$v=19$m=19456,t=2,p=1$'*) ;;
  *) printf 'administrator password hashing returned an invalid verifier\n' >&2; exit 65 ;;
esac
printf '%s' "$admin_hash" > "$work/admin-password.phc"
unset admin_hash
chmod 0600 "$work/admin-password.phc"
export UCA_ADMIN_PASSWORD_HASH_SOURCE="$work/admin-password.phc"
printf '\302\240%s\302\240\r\n' 'unused-placeholder-for-deterministic-mock-mode' > "$work/normalized-mock-provider-key.txt"
chmod 0600 "$work/normalized-mock-provider-key.txt"
for key_source in ./mock-provider-key.txt "$work/normalized-mock-provider-key.txt"; do
  if placeholder_output=$(UCA_AGENT_PROVIDER=openai-compatible \
    UCA_AGENT_BASE_URL=https://provider.example.invalid/v1 \
    UCA_AGENT_MODEL=smoke-model \
    UCA_AGENT_CONTEXT_TOKENS=131072 \
    UCA_AGENT_API_KEY_SOURCE="$key_source" \
    compose run -e LC_ALL=C --rm --no-deps mvp 2>&1); then
    printf 'openai-compatible mode accepted a mock provider placeholder variant\n' >&2
    exit 1
  fi
  case "$placeholder_output" in
    *'bundled mock provider placeholder is forbidden'*) ;;
    *)
      printf 'unexpected placeholder-key rejection for %s\n%s\n' "$key_source" "$placeholder_output" >&2
      exit 1
      ;;
  esac
done

printf '%s\n' 'non-secret-entrypoint-probe-value' > "$work/private-source-key.txt"
chmod 0600 "$work/private-source-key.txt"
probe_output=$(UCA_AGENT_PROVIDER=openai-compatible \
  UCA_AGENT_BASE_URL=https://provider.example.invalid/v1 \
  UCA_AGENT_MODEL=smoke-model \
  UCA_AGENT_CONTEXT_TOKENS=131072 \
  UCA_AGENT_API_KEY_SOURCE="$work/private-source-key.txt" \
  compose run -e LC_ALL=C -e UCA_ENTRYPOINT_SECRET_PROBE=1 --rm --no-deps mvp 2>&1)
case "$probe_output" in
  *'ENTRYPOINT_SECRET_PROBE=PASS uid=65532 key=65532:65532:600 caps=none'*) ;;
  *) printf 'real-provider secret privilege probe failed\n%s\n' "$probe_output" >&2; exit 1 ;;
esac

compose up -d
published=$(compose port mvp 8787)
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
base="http://127.0.0.1:${port}"

healthy=0
for _ in $(seq 1 150); do
  if curl_health "$base/healthz" > "$work/health.json"; then
    healthy=1
    break
  fi
  sleep 1
done
if [ "$healthy" -ne 1 ]; then
  compose ps
  compose logs --no-color --tail 200
  printf 'health timeout\n' >&2
  exit 1
fi

python3 - "$work/health.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value == {'schema': 'ustc-agentd-health/v1', 'status': 'ok'}, value
PY
unauthorized_status=$(curl --connect-timeout 2 --max-time 10 --silent \
  --output "$work/unauthorized.json" --write-out '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary '{"schema":"ustc-agent-chat-request/v1","messages":[{"role":"user","content":"hello"}],"opportunity_context":null}' \
  "$base/api/v1/agent/chat")
[ "$unauthorized_status" = 401 ] || {
  printf 'unauthenticated chat returned HTTP %s\n' "$unauthorized_status" >&2
  exit 1
}
login_local_access
python3 - "$work/login.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['schema'] == 'ustc-local-access/v1', value
assert value['authenticated'] is True, value
assert value['account']['username'] == 'admin', value
assert value['provider']['mode'] == 'mock', value
PY
curl_request "$base/" > "$work/index.html"
for marker in 'AFFAIRS NAVIGATOR' 'CHANGE RADAR' 'OPPORTUNITY GRAPH'; do
  grep -Fq "$marker" "$work/index.html"
done

curl_request \
  -H 'x-ustc-client-protocol-major: 1' \
  "$base/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate" \
  > "$work/affairs.json"
python3 - "$work/affairs.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['kind'] == 'available', value
assert value['terminal']['outcome']['kind'] == 'found', value
PY

for spec in \
  'affairs|成绩单证明怎么办|affairs_navigator_get|transcript-certificate' \
  'change|校历最近有什么变化|change_radar_get|academic-calendar'; do
  IFS='|' read -r label prompt tool answer_marker <<EOF
$spec
EOF
  python3 - "$prompt" "$work/chat-request.json" <<'PY'
import json, pathlib, sys
pathlib.Path(sys.argv[2]).write_text(json.dumps({
    'schema': 'ustc-agent-chat-request/v1',
    'messages': [{'role': 'user', 'content': sys.argv[1]}],
    'opportunity_context': None,
}), encoding='utf-8')
PY
  auth_curl_request \
    -H 'content-type: application/json' \
    --data-binary "@$work/chat-request.json" \
    "$base/api/v1/agent/chat" > "$work/chat-$label.json"
  python3 - "$work/chat-$label.json" "$tool" "$answer_marker" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['schema'] == 'ustc-agent-chat-response/v1', value
assert value['provider'] == {'mode': 'mock', 'model': 'deterministic-mock-v1'}, value
assert value['tool_trace'][0]['tool'] == sys.argv[2], value
assert value['tool_trace'][0]['status'] == 'succeeded', value
assert sys.argv[3] in value['answer'], value
PY
done

cat > "$work/profile-request.json" <<'JSON'
{"consent":true,"request_id":"req:compose:profile","correlation_id":"corr:compose:profile","idempotency_key":"idem:compose:profile","consented_at":1787792400000,"completed_courses":["MATH1001","MATH1002","CS1001","PHYS1001"],"min_credits":9,"max_credits":12,"preference_weights":[{"course_code":"MATH2001","weight":9},{"course_code":"MATH2003","weight":8},{"course_code":"CS2006","weight":7},{"course_code":"PHYS2003","weight":5},{"course_code":"HUM2001","weight":4},{"course_code":"GEN2001","weight":3},{"course_code":"LANG2001","weight":2}]}
JSON
auth_curl_request \
  -H 'content-type: application/json' \
  -H 'x-ustc-opportunity-confirmation: confirmed' \
  --data-binary "@$work/profile-request.json" \
  "$base/api/v1/opportunity/profiles" > "$work/profile.json"
profile_id=$(python3 - "$work/profile.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['terminal']['kind'] == 'profile_created', value
print(value['terminal']['profile']['profile_snapshot_id'])
PY
)
python3 - "$profile_id" "$work/chat-opportunity-request.json" <<'PY'
import json, pathlib, sys
pathlib.Path(sys.argv[2]).write_text(json.dumps({
    'schema': 'ustc-agent-chat-request/v1',
    'messages': [{'role': 'user', 'content': '帮我规划课程'}],
    'opportunity_context': {'profile_snapshot_id': sys.argv[1]},
}), encoding='utf-8')
PY
auth_curl_request \
  -H 'content-type: application/json' \
  -H 'x-ustc-opportunity-confirmation: confirmed' \
  --data-binary "@$work/chat-opportunity-request.json" \
  "$base/api/v1/agent/chat" > "$work/chat-opportunity-before.json"
python3 - "$work/chat-opportunity-before.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['tool_trace'][0]['tool'] == 'opportunity_graph_plan_current_profile', value
assert value['tool_trace'][0]['status'] == 'succeeded', value
assert 'MATH2001' in value['answer'], value
PY

auth_curl_request \
  -H 'content-type: application/json' \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  -d '{"confirm_publish":true}' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/published.json"
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/status-before.json"
python3 - "$work/published.json" "$work/status-before.json" <<'PY'
import json, pathlib, sys
published = json.loads(pathlib.Path(sys.argv[1]).read_text())
status = json.loads(pathlib.Path(sys.argv[2]).read_text())
assert published['outcome']['kind'] == 'published', published
assert status['publication_revision'] is not None, status
assert status['control_evidence_event_count'] > 0, status
PY
auth_curl_request \
  -H 'content-type: application/json' \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  -d '{"confirm_publish":true}' \
  "$base/api/v1/demo/administrator/changes/publication" \
  > "$work/change-published.json"
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/changes/publication" \
  > "$work/change-status-before.json"
python3 - "$work/change-published.json" "$work/change-status-before.json" <<'PY'
import json, pathlib, sys
published = json.loads(pathlib.Path(sys.argv[1]).read_text())
status = json.loads(pathlib.Path(sys.argv[2]).read_text())
assert published['outcome']['kind'] == 'published', published
assert status['review_count'] > 0, status
assert status['publication_count'] > 0, status
assert status['publication_receipt_id'], status
assert status['control_evidence_event_count'] > 0, status
PY
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/status-before.json"

compose restart
healthy=0
for _ in $(seq 1 60); do
  if curl_health "$base/healthz" >/dev/null; then
    healthy=1
    break
  fi
  sleep 1
done
[ "$healthy" -eq 1 ] || { printf 'restart health timeout\n' >&2; exit 1; }
login_local_access
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/status-after.json"
cmp "$work/status-before.json" "$work/status-after.json"
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/changes/publication" \
  > "$work/change-status-after.json"
cmp "$work/change-status-before.json" "$work/change-status-after.json"
auth_curl_request \
  -H 'content-type: application/json' \
  -H 'x-ustc-opportunity-confirmation: confirmed' \
  --data-binary "@$work/chat-opportunity-request.json" \
  "$base/api/v1/agent/chat" > "$work/chat-opportunity-after.json"
python3 - "$work/chat-opportunity-after.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['tool_trace'][0]['tool'] == 'opportunity_graph_plan_current_profile', value
assert value['tool_trace'][0]['status'] == 'succeeded', value
assert 'MATH2001' in value['answer'], value
PY

compose down
compose up -d
healthy=0
for _ in $(seq 1 60); do
  if curl_health "$base/healthz" >/dev/null; then
    healthy=1
    break
  fi
  sleep 1
done
[ "$healthy" -eq 1 ] || { printf 'down-up health timeout\n' >&2; exit 1; }
login_local_access
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/status-after-down-up.json"
cmp "$work/status-before.json" "$work/status-after-down-up.json"
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/changes/publication" \
  > "$work/change-status-after-down-up.json"
cmp "$work/change-status-before.json" "$work/change-status-after-down-up.json"
auth_curl_request \
  -H 'content-type: application/json' \
  -H 'x-ustc-opportunity-confirmation: confirmed' \
  --data-binary "@$work/chat-opportunity-request.json" \
  "$base/api/v1/agent/chat" > "$work/chat-opportunity-after-down-up.json"
python3 - "$work/chat-opportunity-after-down-up.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['tool_trace'][0]['tool'] == 'opportunity_graph_plan_current_profile', value
assert value['tool_trace'][0]['status'] == 'succeeded', value
assert 'MATH2001' in value['answer'], value
PY

UCA_RESET_CONFIRMATION=RESET COMPOSE_PROJECT_NAME="$project" ./reset.sh
compose up -d
healthy=0
for _ in $(seq 1 60); do
  if curl_health "$base/healthz" >/dev/null; then
    healthy=1
    break
  fi
  sleep 1
done
[ "$healthy" -eq 1 ] || { printf 'reset health timeout\n' >&2; exit 1; }
login_local_access
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/affairs-status-after-reset.json"
curl_request \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/changes/publication" \
  > "$work/change-status-after-reset.json"
profile_status=$(curl \
  --connect-timeout 2 \
  --max-time 10 \
  --silent \
  --show-error \
  --cookie "$work/session.cookies" \
  --header 'x-ustc-opportunity-confirmation: confirmed' \
  --output "$work/profile-after-reset.json" \
  --write-out '%{http_code}' \
  "$base/api/v1/opportunity/profiles/$profile_id")
[ "$profile_status" = 404 ] || {
  printf 'reset retained Opportunity profile: HTTP %s\n' "$profile_status" >&2
  exit 1
}
python3 - \
  "$work/affairs-status-after-reset.json" \
  "$work/change-status-after-reset.json" \
  "$work/profile-after-reset.json" <<'PY'
import json, pathlib, sys
affairs = json.loads(pathlib.Path(sys.argv[1]).read_text())
change = json.loads(pathlib.Path(sys.argv[2]).read_text())
profile = json.loads(pathlib.Path(sys.argv[3]).read_text())
assert affairs['schema'] == 'ustc-affairs-publication-status/v1', affairs
assert affairs['publication_revision'] == 1, affairs
assert isinstance(affairs['publication_receipt_id'], str) and affairs['publication_receipt_id'], affairs
assert affairs['control_evidence_event_count'] == 0, affairs
assert change['schema'] == 'ustc-change-publication-status/v1', change
assert change['review_count'] == 0, change
assert change['publication_count'] == 0, change
assert change['publication_receipt_id'] is None, change
assert profile['kind'] == 'opportunity_rejected', profile
assert profile['rejection']['kind'] == 'missing_profile', profile
PY

compose ps
printf 'MVP_COMPOSE_SMOKE=PASS\n'
