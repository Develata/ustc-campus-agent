#!/usr/bin/env python3
"""Exercise the Android WebView over its CDP debug socket using stdlib only."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import socket
import struct
import time
import urllib.request
from dataclasses import dataclass
from urllib.parse import urlparse

GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"


class SmokeFailure(RuntimeError):
    pass


@dataclass
class WebSocket:
    sock: socket.socket

    @classmethod
    def connect(cls, url: str, timeout: float = 10.0) -> "WebSocket":
        parsed = urlparse(url)
        if parsed.scheme != "ws" or not parsed.hostname:
            raise SmokeFailure(f"invalid CDP WebSocket URL: {url}")
        port = parsed.port or 80
        sock = socket.create_connection((parsed.hostname, port), timeout=timeout)
        sock.settimeout(timeout)
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        path = parsed.path or "/"
        if parsed.query:
            path += "?" + parsed.query
        request = (
            f"GET {path} HTTP/1.1\r\n"
            f"Host: {parsed.hostname}:{port}\r\n"
            "Upgrade: websocket\r\n"
            "Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            "Sec-WebSocket-Version: 13\r\n\r\n"
        )
        sock.sendall(request.encode("ascii"))
        response = cls._recv_headers(sock)
        status, *header_lines = response.decode("iso-8859-1").split("\r\n")
        if " 101 " not in status:
            sock.close()
            raise SmokeFailure(f"CDP WebSocket upgrade failed: {status}")
        headers = {}
        for line in header_lines:
            if ":" in line:
                name, value = line.split(":", 1)
                headers[name.strip().lower()] = value.strip()
        expected = base64.b64encode(hashlib.sha1((key + GUID).encode("ascii")).digest()).decode(
            "ascii"
        )
        if headers.get("sec-websocket-accept") != expected:
            sock.close()
            raise SmokeFailure("CDP WebSocket accept digest mismatch")
        return cls(sock)

    @staticmethod
    def _recv_headers(sock: socket.socket) -> bytes:
        data = bytearray()
        while b"\r\n\r\n" not in data:
            chunk = sock.recv(4096)
            if not chunk:
                raise SmokeFailure("CDP WebSocket closed during handshake")
            data.extend(chunk)
            if len(data) > 64 * 1024:
                raise SmokeFailure("oversized CDP WebSocket handshake")
        head, _separator, extra = data.partition(b"\r\n\r\n")
        if extra:
            raise SmokeFailure("unexpected framed data in CDP handshake response")
        return bytes(head)

    def close(self) -> None:
        try:
            self._send_frame(0x8, b"")
        finally:
            self.sock.close()

    def send_json(self, value: object) -> None:
        encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        self._send_frame(0x1, encoded)

    def receive_json(self) -> dict[str, object]:
        fragments = bytearray()
        active_opcode: int | None = None
        while True:
            final, opcode, payload = self._read_frame()
            if opcode == 0x8:
                raise SmokeFailure("CDP WebSocket closed before response")
            if opcode == 0x9:
                self._send_frame(0xA, payload)
                continue
            if opcode in (0x1, 0x2):
                if active_opcode is not None:
                    raise SmokeFailure("nested CDP WebSocket message")
                active_opcode = opcode
                fragments.extend(payload)
            elif opcode == 0x0:
                if active_opcode is None:
                    raise SmokeFailure("orphan CDP WebSocket continuation frame")
                fragments.extend(payload)
            elif opcode == 0xA:
                continue
            else:
                raise SmokeFailure(f"unsupported CDP WebSocket opcode: {opcode}")
            if len(fragments) > 4 * 1024 * 1024:
                raise SmokeFailure("oversized CDP WebSocket response")
            if final:
                if active_opcode != 0x1:
                    raise SmokeFailure("CDP returned a non-text message")
                return json.loads(fragments.decode("utf-8"))

    def _send_frame(self, opcode: int, payload: bytes) -> None:
        mask = os.urandom(4)
        length = len(payload)
        header = bytearray([0x80 | opcode])
        if length < 126:
            header.append(0x80 | length)
        elif length <= 0xFFFF:
            header.append(0x80 | 126)
            header.extend(struct.pack("!H", length))
        else:
            header.append(0x80 | 127)
            header.extend(struct.pack("!Q", length))
        header.extend(mask)
        masked = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        self.sock.sendall(bytes(header) + masked)

    def _read_frame(self) -> tuple[bool, int, bytes]:
        first, second = self._recv_exact(2)
        final = bool(first & 0x80)
        opcode = first & 0x0F
        masked = bool(second & 0x80)
        length = second & 0x7F
        if length == 126:
            length = struct.unpack("!H", self._recv_exact(2))[0]
        elif length == 127:
            length = struct.unpack("!Q", self._recv_exact(8))[0]
        mask = self._recv_exact(4) if masked else b""
        payload = self._recv_exact(length)
        if masked:
            payload = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        return final, opcode, payload

    def _recv_exact(self, size: int) -> bytes:
        data = bytearray()
        while len(data) < size:
            chunk = self.sock.recv(size - len(data))
            if not chunk:
                raise SmokeFailure("CDP WebSocket closed unexpectedly")
            data.extend(chunk)
        return bytes(data)


class Cdp:
    def __init__(self, web_socket: WebSocket) -> None:
        self.web_socket = web_socket
        self.next_id = 1

    def call(self, method: str, params: dict[str, object] | None = None) -> dict[str, object]:
        request_id = self.next_id
        self.next_id += 1
        self.web_socket.send_json(
            {"id": request_id, "method": method, "params": params or {}}
        )
        while True:
            response = self.web_socket.receive_json()
            if response.get("id") != request_id:
                continue
            if "error" in response:
                raise SmokeFailure(f"CDP {method} failed: {response['error']}")
            result = response.get("result")
            if not isinstance(result, dict):
                raise SmokeFailure(f"CDP {method} returned no result")
            return result

    def evaluate(self, expression: str) -> object:
        result = self.call(
            "Runtime.evaluate",
            {
                "expression": expression,
                "returnByValue": True,
                "awaitPromise": True,
            },
        )
        remote = result.get("result")
        if not isinstance(remote, dict):
            raise SmokeFailure("CDP Runtime.evaluate returned no remote object")
        if remote.get("subtype") == "error":
            raise SmokeFailure(f"JavaScript evaluation failed: {remote}")
        return remote.get("value")


def discover_page(endpoint: str, expected_origin: str, timeout: float) -> dict[str, object]:
    deadline = time.monotonic() + timeout
    last_targets: object = None
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(endpoint.rstrip("/") + "/json", timeout=5) as response:
                targets = json.load(response)
            last_targets = targets
            for target in targets:
                if (
                    isinstance(target, dict)
                    and target.get("type") == "page"
                    and str(target.get("url", "")).startswith(expected_origin)
                    and str(target.get("webSocketDebuggerUrl", "")).startswith("ws://")
                ):
                    return target
        except (OSError, ValueError):
            pass
        time.sleep(1)
    raise SmokeFailure(f"WebView page not discovered; last targets={last_targets!r}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--devtools", default="http://127.0.0.1:9222")
    parser.add_argument("--origin", default="http://127.0.0.1:8787/")
    parser.add_argument("--timeout-seconds", type=float, default=60)
    args = parser.parse_args()

    target = discover_page(args.devtools, args.origin, args.timeout_seconds)
    web_socket_url = str(target["webSocketDebuggerUrl"]).replace(
        "ws://localhost:", "ws://127.0.0.1:"
    )
    web_socket = WebSocket.connect(web_socket_url)
    try:
        cdp = Cdp(web_socket)
        cdp.call("Runtime.enable")
        initial = cdp.evaluate(
            """(() => ({
              title: document.title,
              origin: location.origin,
              ready: document.readyState,
              chat: Boolean(document.querySelector('#chat-form') && document.querySelector('#chat-input'))
            }))()"""
        )
        if not isinstance(initial, dict):
            raise SmokeFailure(f"invalid initial page state: {initial!r}")
        if initial.get("origin") != args.origin.rstrip("/"):
            raise SmokeFailure(f"unexpected WebView origin: {initial!r}")
        if initial.get("ready") != "complete" or initial.get("chat") is not True:
            raise SmokeFailure(f"WebView chat surface is not ready: {initial!r}")

        submitted = cdp.evaluate(
            """(() => {
              const input = document.querySelector('#chat-input');
              const form = document.querySelector('#chat-form');
              if (!input || !form) return false;
              input.value = '成绩单证明怎么办';
              input.dispatchEvent(new Event('input', { bubbles: true }));
              form.requestSubmit();
              return true;
            })()"""
        )
        if submitted is not True:
            raise SmokeFailure("failed to submit Android WebView chat request")

        deadline = time.monotonic() + args.timeout_seconds
        state: object = None
        while time.monotonic() < deadline:
            state = cdp.evaluate(
                """(() => {
                  const answer = document.querySelector('.chat-message[data-role=assistant] .chat-message-body');
                  const trace = document.querySelector('.chat-message[data-role=assistant] .chat-tool-trace');
                  return {
                    busy: document.querySelector('#chat-surface')?.getAttribute('aria-busy'),
                    answer: answer?.textContent || '',
                    trace: trace?.textContent || ''
                  };
                })()"""
            )
            if (
                isinstance(state, dict)
                and state.get("busy") == "false"
                and "办事流程：" in str(state.get("answer", ""))
                and "办理步骤：" in str(state.get("answer", ""))
                and "办事导航" in str(state.get("trace", ""))
            ):
                break
            time.sleep(1)
        else:
            raise SmokeFailure(f"Android WebView chat did not converge: {state!r}")

        print(
            "android-webview-smoke: PASS "
            f"title={initial.get('title')!r} origin={initial.get('origin')!r} "
            "journey=affairs-chat"
        )
        return 0
    finally:
        web_socket.close()


if __name__ == "__main__":
    raise SystemExit(main())
