"""Controlled CDP sequences for the Android smoke's readiness and result ownership."""

import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "test_android_webview_cdp.py"
SPEC = importlib.util.spec_from_file_location("android_webview_smoke", SCRIPT)
smoke = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = smoke
SPEC.loader.exec_module(smoke)

ORIGIN = "http://127.0.0.1:8787/"


class Clock:
    def __init__(self):
        self.now = 0.0

    def monotonic(self):
        return self.now

    def sleep(self, seconds):
        self.now += seconds


def ready(can_send=True):
    return {"title": "Campus Agent", "origin": ORIGIN.rstrip("/"),
            "ready": "complete", "chat": True, "canSend": can_send}


def answer(count=1, text="办事流程：成绩单证明。办理步骤：核对官方入口。", busy="false"):
    return {"busy": busy, "assistantCount": count, "answer": text, "trace": "办事导航"}


class FakeCdp:
    def __init__(self, readiness, replies, before=0, submitted=True):
        self.readiness = list(readiness)
        self.replies = list(replies)
        self.before = before
        self.submitted = submitted
        self.ready_calls = 0
        self.submit_calls = 0
        self.result_calls = 0

    @staticmethod
    def consume(values):
        return values.pop(0) if len(values) > 1 else values[0]

    def evaluate(self, expression):
        if expression == smoke.READY_EXPRESSION:
            self.ready_calls += 1
            return self.consume(self.readiness)
        if expression == smoke.SUBMIT_EXPRESSION:
            self.submit_calls += 1
            return {"submitted": self.submitted, "assistantCount": self.before}
        expected = smoke.RESULT_EXPRESSION.replace("__ASSISTANT_INDEX__", str(self.before))
        if expression != expected:
            raise AssertionError("result query must select this submission's assistant index")
        self.result_calls += 1
        return self.consume(self.replies)


class AndroidSmokeTests(unittest.TestCase):
    def setUp(self):
        self.clock = Clock()
        self.patch_clock = patch.object(smoke.time, "monotonic", self.clock.monotonic)
        self.patch_sleep = patch.object(smoke.time, "sleep", self.clock.sleep)
        self.patch_clock.start()
        self.patch_sleep.start()
        self.addCleanup(self.patch_clock.stop)
        self.addCleanup(self.patch_sleep.stop)

    def test_preexisting_correct_answer_cannot_pass_without_a_new_reply(self):
        cdp = FakeCdp([ready()], [answer(count=1)], before=1)
        with self.assertRaisesRegex(smoke.SmokeFailure, "new chat reply did not converge"):
            smoke.run_chat_smoke(cdp, ORIGIN, 1)
        self.assertEqual(cdp.submit_calls, 1)
        self.assertEqual(self.clock.now, 1)

    def test_delayed_model_or_conversation_readiness_waits_before_submit(self):
        cdp = FakeCdp([ready(False), ready(False), ready()], [answer()])
        self.assertEqual(smoke.run_chat_smoke(cdp, ORIGIN, 2), ready())
        self.assertEqual(cdp.ready_calls, 3)
        self.assertEqual(cdp.submit_calls, 1)
        self.assertEqual(self.clock.now, 0.5)

    def test_correct_new_reply_and_its_trace_pass_after_existing_history(self):
        cdp = FakeCdp([ready()], [answer(count=2, busy="true"), answer(count=3)], before=2)
        self.assertEqual(smoke.run_chat_smoke(cdp, ORIGIN, 2), ready())
        self.assertEqual(cdp.result_calls, 2)

    def test_new_unrelated_answer_does_not_reuse_old_success(self):
        cdp = FakeCdp([ready()], [answer(count=2, text="这次请求失败。")], before=1)
        with self.assertRaisesRegex(smoke.SmokeFailure, "new chat reply did not converge"):
            smoke.run_chat_smoke(cdp, ORIGIN, 1)

    def test_readiness_timeout_never_submits(self):
        cdp = FakeCdp([ready(False)], [answer()])
        with self.assertRaisesRegex(smoke.SmokeFailure, "did not become send-ready"):
            smoke.run_chat_smoke(cdp, ORIGIN, 1)
        self.assertEqual(cdp.submit_calls, 0)

    def test_ready_and_reply_waits_share_one_bounded_deadline(self):
        cdp = FakeCdp([ready(False), ready(False), ready(False), ready()], [answer(count=0)])
        with self.assertRaises(smoke.SmokeFailure):
            smoke.run_chat_smoke(cdp, ORIGIN, 1)
        self.assertEqual(self.clock.now, 1)

    def test_submit_readiness_race_is_reported_without_retry(self):
        cdp = FakeCdp([ready()], [answer()], submitted=False)
        with self.assertRaisesRegex(smoke.SmokeFailure, "unavailable before submission"):
            smoke.run_chat_smoke(cdp, ORIGIN, 1)
        self.assertEqual(cdp.submit_calls, 1)
        self.assertEqual(cdp.result_calls, 0)


if __name__ == "__main__":
    unittest.main()
