#!/usr/bin/env python3
"""One real-model Skill + Calendar smoke in disposable loopback state (stdlib only).

Explicit endpoint/model/key-file required. No credentials or model prose are logged.
Never attaches to the user's running preview or changes its provider configuration.
"""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


class SmokeFailure(Exception):
    pass


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def smoke(args):
    repo = Path(__file__).resolve().parent.parent
    binary = Path(args.binary).resolve() if args.binary else repo / "target/debug/ustc-agentd"
    key = Path(args.key_file).resolve(strict=True)
    if not binary.is_file() or not key.is_file():
        raise SmokeFailure("binary or private key file is unavailable")
    env = {k: v for k, v in os.environ.items()
           if not k.startswith("UCA_") and not k.lower().endswith("_proxy")}
    env.update(UCA_AGENT_PROVIDER="openai-compatible",
               UCA_AGENT_BASE_URL=args.provider_base_url,
               UCA_AGENT_MODEL=args.model, UCA_AGENT_API_KEY_FILE=str(key),
               UCA_AGENT_TIMEOUT_MS="15000", UCA_AGENT_CONTEXT_TOKENS=str(args.context_tokens))
    # Reserve no persistent port or state. The subprocess owns only this temp directory.
    with tempfile.TemporaryDirectory(prefix="uca-model-plugin-smoke-") as directory:
        state = Path(directory)
        os.chmod(state, 0o700)
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        base = f"http://127.0.0.1:{port}"
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())

        def request(path, value=None):
            data = None if value is None else json.dumps(value).encode()
            req = urllib.request.Request(base + path, data=data, headers={
                "Content-Type": "application/json", "X-USTC-Client-Protocol-Major": "1"})
            try:
                with opener.open(req, timeout=50) as response:
                    body = response.read(1024 * 1024 + 1)
                    if len(body) > 1024 * 1024:
                        raise SmokeFailure("oversize local response")
                    return json.loads(body)
            except urllib.error.HTTPError as error:
                known = {"invalid_chat_request", "provider_not_configured", "provider_unauthorized",
                         "provider_rate_limited", "provider_timeout", "provider_unavailable",
                         "provider_protocol_error", "context_budget_exceeded", "tool_call_rejected",
                         "tool_result_too_large", "tool_budget_exhausted", "turn_budget_exhausted",
                         "opportunity_confirmation_required", "composition_unavailable", "internal_chat_error"}
                code = None
                try:
                    failure = json.loads(error.read(4096))
                    if failure.get("schema") == "ustc-agent-chat-error/v1" and failure.get("error") in known:
                        code = failure["error"]
                except (ValueError, AttributeError, TypeError):
                    pass
                detail = f" ({code})" if code else ""
                raise SmokeFailure(f"local API {path} returned HTTP {error.code}{detail}") from None

        command = [str(binary), "serve-web", "--bind", f"127.0.0.1:{port}",
                   "--fixture", "fixtures/affairs/proc-011-reviewed.json",
                   "--change-fixture", "fixtures/change-radar/academic-calendar-demo-reviewed.json",
                   "--opportunity-fixture", "fixtures/opportunity-graph/course-planning-demo-reviewed.json",
                   "--opportunity-catalog", "market/fixtures/course-planning/minimal-v0.json",
                   "--opportunity-profile-store", str(state / "profiles.json"),
                   "--store", str(state / "affairs.json"),
                   "--idempotency", str(state / "idempotency.json"),
                   "--session-store", str(state / "sessions.json")]
        process = subprocess.Popen(command, cwd=repo, env=env,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            deadline = time.monotonic() + 15
            while True:
                if process.poll() is not None:
                    raise SmokeFailure("isolated server exited; check model configuration")
                try:
                    view = request("/api/v1/plugins")
                    break
                except (urllib.error.URLError, TimeoutError):
                    if time.monotonic() >= deadline:
                        raise SmokeFailure("isolated server startup timeout") from None
                    time.sleep(0.1)
            package = next(p for p in view["packages"] if p["package_id"] == "ustc.campus-guide")

            def action(intent):
                response = request("/api/v1/plugins/commands", {
                    "schema": "plugin-command/v1", "request_id": "smoke-" + intent["action"],
                    "intent": intent})
                if response.get("accepted") is not True:
                    raise SmokeFailure("plugin lifecycle command rejected")
                return response

            installed = action({"action": "install", **{k: package[k] for k in
                               ("package_id", "version", "catalog_revision", "package_digest")}})
            binding = {"installation_id": installed["installation_id"],
                       "expected_revision": installed["revision"]}
            checked = request("/api/v1/plugins/probe", {"schema": "plugin-probe/v1", **binding})
            action({"action": "grant", **binding, "capability": "campus.public_rules.read"})
            action({"action": "enable", **binding, "readiness_digest": checked["readiness_digest"]})
            started = time.monotonic()
            # Stateless route avoids a second automatic topic-title model request.
            result = request("/api/v1/agent/chat", {"schema": "ustc-agent-chat-request/v1",
                "messages": [{"role": "user", "content":
                    "请实际调用已启用的校园使用指南 Skill，读取其入口，并列出我的个人日历事项。"
                    "两项都要调用工具，最后用一句中文概括结果。不要创建或删除事项。"}]})
            trace = [{"tool": t.get("tool"), "status": t.get("status")}
                     for t in result.get("tool_trace", [])]
            successful = {t["tool"] for t in trace if t["status"] == "succeeded"}
            passed = {"plugin_tool", "simple_calendar_items"}.issubset(successful)
            passed = passed and all(t["status"] == "succeeded" for t in trace)
            print(json.dumps({"schema": "uca-model-plugin-smoke/v1", "passed": passed,
                              "elapsed_seconds": round(time.monotonic() - started, 2),
                              "provider": result.get("provider"), "tool_trace": trace,
                              "usage": result.get("usage"), "scope": "isolated synthetic state"},
                             ensure_ascii=False))
            if not passed:
                raise SmokeFailure("model did not complete both tools successfully; no automatic retry")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provider-base-url", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--key-file", required=True, help="Existing private key file; never a key literal")
    parser.add_argument("--context-tokens", type=int, default=32768, help="Conservative request budget, no larger than model capacity")
    parser.add_argument("--binary", help="Defaults to this checkout's target/debug/ustc-agentd")
    args = parser.parse_args()
    if not 16384 <= args.context_tokens <= 1048576:
        parser.error("context-tokens must be in 16384..1048576")
    try:
        smoke(args)
    except (SmokeFailure, OSError, ValueError, KeyError, StopIteration) as error:
        # Do not print provider responses, environment, paths or unexpected exception text.
        print("SMOKE FAIL: " + (str(error) if isinstance(error, SmokeFailure) else type(error).__name__))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
