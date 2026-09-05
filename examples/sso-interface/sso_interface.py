"""Loopback-only, fail-closed SSO interface prototype; no authentication.

No credentials, identity assertions, network providers or session stores are used.
This is not connected to ustc-agentd. See README.zh-CN.md for the exact boundary.
"""

from __future__ import annotations

import argparse
import json
from http.server import BaseHTTPRequestHandler, HTTPServer

BASE = "/api/v1/auth/sso/ustc"
ROUTES = {BASE + "/status": "GET", BASE + "/start": "POST", BASE + "/callback": "GET"}
SCHEMA_VERSION = "uca-sso-reservation/v1"


def envelope(**values: object) -> dict[str, object]:
    return {"schema_version": SCHEMA_VERSION, "provider": "ustc", **values}


class SsoInterfaceHandler(BaseHTTPRequestHandler):
    """Fixed responses only: caller inputs can never become an identity."""

    protocol_version = "HTTP/1.0"
    server_version = "UcaSsoInterface/1"
    sys_version = ""

    def setup(self) -> None:
        super().setup()
        self.connection.settimeout(2.0)

    def log_message(self, format: str, *args: object) -> None:
        # BaseHTTPRequestHandler otherwise logs callback queries, possibly secrets.
        pass

    def send_error(self, code: int, message: str | None = None,
                   explain: str | None = None) -> None:
        # Never echo malformed request lines, callback values or parser diagnostics.
        self.respond(code, envelope(error="invalid_http_request"))

    def respond(self, status: int, body: dict[str, object],
                allow: str | None = None) -> None:
        payload = json.dumps(body, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        self.close_connection = True
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("Pragma", "no-cache")
        self.send_header("Referrer-Policy", "no-referrer")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Connection", "close")
        if allow is not None:
            self.send_header("Allow", allow)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(payload)

    def handle_request(self) -> None:
        hosts = self.headers.get_all("Host", [])
        assert isinstance(self.server, HTTPServer)
        port = self.server.server_port
        allowed_hosts = {f"127.0.0.1:{port}", f"localhost:{port}"}
        if port == 80:
            allowed_hosts.update({"127.0.0.1", "localhost"})
        if len(hosts) != 1 or hosts[0] not in allowed_hosts:
            self.respond(400, envelope(error="invalid_host"))
            return
        if len(self.path) > 4096:
            self.respond(414, envelope(error="request_target_too_long"))
            return
        lengths = self.headers.get_all("Content-Length", [])
        if self.headers.get_all("Transfer-Encoding", []) or lengths not in ([], ["0"]):
            self.respond(400, envelope(error="request_body_not_supported"))
            return
        path = self.path.partition("?")[0]
        allowed = ROUTES.get(path)
        if allowed is None:
            self.respond(404, envelope(error="route_not_found"))
            return
        if self.command != allowed:
            self.respond(405, envelope(error="method_not_allowed"), allowed)
            return
        if path == BASE + "/status":
            self.respond(200, envelope(enabled=False, state="not_configured", protocol=None))
            return
        self.respond(503, envelope(
            error="sso_not_configured",
            message="尚未配置学校统一身份认证；正式接入需相关授权及服务端配置。",
        ))

    do_GET = handle_request
    do_POST = handle_request
    do_PUT = handle_request
    do_DELETE = handle_request
    do_PATCH = handle_request
    do_OPTIONS = handle_request
    do_HEAD = handle_request
    do_TRACE = handle_request
    do_CONNECT = handle_request


def make_server(port: int = 8891) -> HTTPServer:
    """No host argument: this prototype must never bind to the public network."""
    return HTTPServer(("127.0.0.1", port), SsoInterfaceHandler)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8891)
    args = parser.parse_args()
    if not 0 <= args.port <= 65535:
        parser.error("port must be in 0..65535 (0 chooses an unused port)")
    with make_server(args.port) as server:
        print(f"SSO interface NOT CONFIGURED: http://127.0.0.1:{server.server_port}", flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
