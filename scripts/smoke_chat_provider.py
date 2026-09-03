#!/usr/bin/env python3
"""Credential-safe opt-in smoke for the configured Responses endpoint."""

from __future__ import annotations

import ipaddress
import json
import os
import sys
from typing import NoReturn
import urllib.error
import urllib.parse
import urllib.request

MAX_RESPONSE_BYTES = 512 * 1024
REQUIRED_ENV = (
    "USTC_AGENT_MODEL_BASE_URL",
    "USTC_AGENT_MODEL_API_KEY",
    "USTC_AGENT_MODEL",
)


def fail(message: str) -> NoReturn:
    print(f"live-provider-smoke: FAIL {message}", file=sys.stderr)
    raise SystemExit(1)


def endpoint_from_origin(raw: str) -> str:
    parsed = urllib.parse.urlsplit(raw)
    hostname = parsed.hostname
    if (
        hostname is None
        or parsed.username is not None
        or parsed.password is not None
        or parsed.path not in ("", "/")
        or parsed.query
        or parsed.fragment
    ):
        fail("invalid configured origin")
    if parsed.scheme == "http":
        try:
            if not ipaddress.ip_address(hostname).is_loopback:
                fail("insecure non-loopback origin")
        except ValueError:
            fail("insecure non-loopback origin")
    elif parsed.scheme != "https":
        fail("unsupported configured origin scheme")
    return urllib.parse.urlunsplit(
        (parsed.scheme, parsed.netloc, "/v1/responses", "", "")
    )


def main() -> None:
    missing = [name for name in REQUIRED_ENV if not os.environ.get(name)]
    if missing:
        print(f"live-provider-smoke: NOT_RUN missing={','.join(missing)}")
        return

    base_url = os.environ["USTC_AGENT_MODEL_BASE_URL"]
    api_key = os.environ["USTC_AGENT_MODEL_API_KEY"]
    model = os.environ["USTC_AGENT_MODEL"]
    if base_url.strip() != base_url or api_key.strip() != api_key or model.strip() != model:
        fail("configured values contain surrounding whitespace")
    if not (1 <= len(api_key) <= 4096) or not (1 <= len(model.encode()) <= 256):
        fail("configured value exceeds bounds")

    timeout_raw = os.environ.get("USTC_AGENT_MODEL_TIMEOUT_SECS", "30")
    try:
        timeout = int(timeout_raw)
    except ValueError:
        fail("invalid timeout")
    if not 1 <= timeout <= 120:
        fail("invalid timeout")

    payload = json.dumps(
        {
            "model": model,
            "store": False,
            "max_output_tokens": 32,
            "input": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "input_text",
                            "text": "Reply with one short acknowledgement and do not call tools.",
                        }
                    ],
                }
            ],
        },
        separators=(",", ":"),
    ).encode()
    request = urllib.request.Request(
        endpoint_from_origin(base_url),
        data=payload,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            length = response.headers.get("Content-Length")
            if length is not None and int(length) > MAX_RESPONSE_BYTES:
                fail("response exceeds bound")
            body = response.read(MAX_RESPONSE_BYTES + 1)
    except urllib.error.HTTPError:
        fail("provider rejected request")
    except (urllib.error.URLError, TimeoutError, OSError, ValueError):
        fail("provider request failed")

    if len(body) > MAX_RESPONSE_BYTES:
        fail("response exceeds bound")
    try:
        decoded = json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError):
        fail("provider returned malformed JSON")
    if decoded.get("status") != "completed" or decoded.get("model") != model:
        fail("provider returned an unexpected terminal")
    outputs = decoded.get("output")
    if not isinstance(outputs, list) or not any(
        isinstance(item, dict) and item.get("type") == "message" for item in outputs
    ):
        fail("provider returned no completed message")
    print(f"live-provider-smoke: PASS model={model}")


if __name__ == "__main__":
    main()
