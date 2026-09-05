"""Real loopback HTTP tests: no real SSO or application integration claim."""
from __future__ import annotations

from contextlib import closing, redirect_stderr
import http.client
import io
import json
from pathlib import Path
import socket
import threading
import unittest

from sso_interface import BASE, ROUTES, SCHEMA_VERSION, make_server


class SsoInterfaceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.server = make_server(0)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls) -> None:
        cls.server.shutdown()
        cls.thread.join(timeout=5)
        cls.server.server_close()
        if cls.thread.is_alive():
            raise AssertionError("test server did not stop")

    def request(self, method: str, path: str, *, body: bytes | None = None,
                headers: dict[str, str] | None = None):
        with closing(http.client.HTTPConnection("127.0.0.1", self.server.server_port, timeout=5)) as conn:
            conn.request(method, path, body=body, headers=headers or {})
            response = conn.getresponse()
            body = response.read()
            headers = dict(response.getheaders())
            self.assertEqual(headers["Cache-Control"], "no-store")
            self.assertEqual(headers["Referrer-Policy"], "no-referrer")
            self.assertEqual(headers["Connection"], "close")
            for name in ("Set-Cookie", "Location", "Access-Control-Allow-Origin"):
                self.assertNotIn(name, headers)
            if method != "HEAD":
                self.assertEqual(len(body), int(headers["Content-Length"]))
            return response.status, headers, body

    def test_status_is_disabled_not_authenticated(self):
        status, _, body = self.request("GET", BASE + "/status")
        self.assertEqual(status, 200)
        self.assertEqual(json.loads(body), {
            "schema_version": SCHEMA_VERSION, "provider": "ustc", "enabled": False,
            "state": "not_configured", "protocol": None,
        })

    def test_start_is_unavailable(self):
        status, _, body = self.request("POST", BASE + "/start")
        self.assertEqual(status, 503)
        self.assertEqual(json.loads(body)["error"], "sso_not_configured")

    def test_callbacks_never_authenticate_or_echo_inputs(self):
        for query in ("", "?code=SYNTHETIC_SECRET&state=SYNTHETIC_STATE", "?ticket=SYNTHETIC_SECRET",
                      "?user_id=admin&authenticated=true", "?code=a&code=b&return_url=https://example.invalid"):
            with self.subTest(query=query):
                status, _, body = self.request("GET", BASE + "/callback" + query)
                self.assertEqual(status, 503)
                payload = json.loads(body)
                self.assertEqual(payload["error"], "sso_not_configured")
                self.assertEqual(set(payload), {"schema_version", "provider", "error", "message"})
                self.assertNotIn(b"SYNTHETIC", body)
                self.assertNotIn(b"example.invalid", body)

    def test_unknown_provider_and_path_are_not_found(self):
        for path in ("/api/v1/auth/sso/other/status", BASE, BASE + "/status/", "/login"):
            with self.subTest(path=path):
                status, _, body = self.request("GET", path)
                self.assertEqual(status, 404)
                self.assertEqual(json.loads(body)["error"], "route_not_found")

    def test_wrong_methods_do_not_start_or_accept_login(self):
        for path, allowed in ROUTES.items():
            for method in ("GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD", "TRACE", "CONNECT"):
                if method == allowed:
                    continue
                with self.subTest(path=path, method=method):
                    status, headers, body = self.request(method, path)
                    self.assertEqual(status, 405)
                    self.assertEqual(headers["Allow"], allowed)
                    if method == "HEAD":
                        self.assertEqual(body, b"")
                    else:
                        self.assertEqual(json.loads(body)["error"], "method_not_allowed")

    def test_external_host_is_rejected(self):
        for host in ("example.invalid", "example.invalid:" + str(self.server.server_port), "127.0.0.1:1"):
            with self.subTest(host=host):
                status, _, body = self.request("GET", BASE + "/status", headers={"Host": host})
                self.assertEqual(status, 400)
                self.assertEqual(json.loads(body)["error"], "invalid_host")

    def test_localhost_host_is_accepted(self):
        status, _, _ = self.request("GET", BASE + "/status", headers={"Host": f"localhost:{self.server.server_port}"})
        self.assertEqual(status, 200)

    def test_body_is_rejected_without_reflection(self):
        status, _, body = self.request("POST", BASE + "/start", body=b"password=SYNTHETIC_SECRET")
        self.assertEqual(status, 400)
        self.assertEqual(json.loads(body)["error"], "request_body_not_supported")
        self.assertNotIn(b"SYNTHETIC_SECRET", body)

    def test_transfer_encoding_is_rejected(self):
        status, _, body = self.request("POST", BASE + "/start", headers={"Transfer-Encoding": "chunked"})
        self.assertEqual(status, 400)
        self.assertEqual(json.loads(body)["error"], "request_body_not_supported")

    def test_long_target_is_rejected(self):
        status, _, _ = self.request("GET", BASE + "/callback?code=" + "x" * 4096)
        self.assertEqual(status, 414)

    def raw(self, data: bytes) -> bytes:
        with socket.create_connection(("127.0.0.1", self.server.server_port), timeout=5) as conn:
            conn.sendall(data)
            parts = []
            while part := conn.recv(65536):
                parts.append(part)
            return b"".join(parts)

    def test_duplicate_and_missing_host_fail_closed(self):
        for hosts in ("", f"Host: localhost:{self.server.server_port}\r\nHost: example.invalid\r\n"):
            with self.subTest(hosts=hosts):
                body = self.raw(f"GET {BASE}/status HTTP/1.1\r\n{hosts}\r\n".encode())
                self.assertIn(b"400 Bad Request", body)
                self.assertIn(b"invalid_host", body)

    def test_parser_errors_and_queries_are_not_logged(self):
        captured = io.StringIO()
        with redirect_stderr(captured):
            self.request("GET", BASE + "/callback?code=SYNTHETIC_SECRET")
            response = self.raw(b"GET /SYNTHETIC_SECRET HTTP/bad\r\n\r\n")
        self.assertEqual(captured.getvalue(), "")
        self.assertNotIn(b"SYNTHETIC_SECRET", response)
        self.assertIn(b"invalid_http_request", response)

    def test_malformed_headers_fail_closed_without_reflection(self):
        for malformed in ("BrokenHeader", "Bad Header: x", ": bad"):
            with self.subTest(header=malformed):
                request = (f"GET {BASE}/status HTTP/1.1\r\n"
                           f"Host: 127.0.0.1:{self.server.server_port}\r\n"
                           f"{malformed}\r\n\r\n").encode()
                response = self.raw(request)
                self.assertIn(b"400 Bad Request", response)
                self.assertIn(b"invalid_http_request", response)
                self.assertNotIn(malformed.encode(), response)
                self.assertNotIn(b"Set-Cookie:", response)

    def test_only_loopback_is_bound(self):
        self.assertEqual(self.server.server_address[0], "127.0.0.1")

    def test_openapi_matches_real_http_routes_and_bodies(self):
        spec = json.loads(Path(__file__).with_name("openapi.json").read_text(encoding="utf-8"))
        self.assertEqual(spec["openapi"], "3.1.0")
        self.assertEqual(set(spec["paths"]), set(ROUTES))
        for path, method in ROUTES.items():
            with self.subTest(path=path):
                self.assertEqual(set(spec["paths"][path]), {method.lower()})
                status, _, body = self.request(method, path)
                expected = spec["paths"][path][method.lower()]["responses"][str(status)]["content"]["application/json"]["example"]
                self.assertEqual(json.loads(body), expected)


if __name__ == "__main__":
    unittest.main()
