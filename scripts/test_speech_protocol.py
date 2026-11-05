"""V05 controlled worker/transport tests; synthetic audio is never integration proof."""
import importlib.util
import io
import json
from pathlib import Path
import struct
import threading
import unittest

PATH = Path(__file__).resolve().parents[1] / "apps/desktop/src-tauri/speech_protocol.py"
SPEC = importlib.util.spec_from_file_location("speech_protocol", PATH)
protocol = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(protocol)


def frame(identity, voice="female"):
    return json.dumps({"id": identity, "text": "école", "voice": voice}).encode() + b"\n"


class Protocol(unittest.TestCase):
    def test_identity_voice_payload_and_bounded_shared_calls(self):
        source = io.BytesIO(frame(1) + frame(2, "male") + frame(3))
        destination = io.BytesIO()
        barrier = threading.Barrier(3)
        active = 0
        maximum = 0
        lock = threading.Lock()

        def worker(text, voice):
            nonlocal active, maximum
            with lock:
                active += 1
                maximum = max(maximum, active)
            barrier.wait(timeout=2)
            with lock:
                active -= 1
            return b"OggS" + voice.encode()

        protocol.serve(worker, source, destination)
        destination.seek(0)
        self.assertTrue(protocol.response(destination)[0]["ready"])
        replies = dict((header["id"], (header, audio)) for header, audio in (protocol.response(destination) for _ in range(3)))
        self.assertEqual(maximum, 3)
        for identity, voice in ((1, "female"), (2, "male"), (3, "female")):
            self.assertEqual(replies[identity][0]["voice"], voice)
            self.assertEqual(replies[identity][0]["language"], "fr")
            self.assertEqual(replies[identity][1], b"OggS" + voice.encode())

    def test_worker_failure_and_invalid_audio_are_visible_and_do_not_stop_next_request(self):
        for error in (RuntimeError("worker exited"), None):
            def worker(text, voice):
                if voice == "female":
                    if error:
                        raise error
                    return b"invalid"
                return b"OggSmale"
            destination = io.BytesIO()
            protocol.serve(worker, io.BytesIO(frame(1) + frame(2, "male")), destination, capacity=1)
            destination.seek(0)
            protocol.response(destination)
            header, audio = protocol.response(destination)
            self.assertIn("error", header)
            self.assertEqual(audio, b"")
            self.assertEqual(protocol.response(destination)[1], b"OggSmale")

    def test_malformed_duplicate_and_truncated_frames_are_rejected(self):
        for value in (b"x" * 8193, b"{}\n", frame(True), frame(0), frame(1)[:-1], frame(1) + frame(1)):
            with self.assertRaises(ValueError):
                protocol.serve(lambda *_: b"OggSfixture", io.BytesIO(value), io.BytesIO())
        for value in (b"", struct.pack("<I", 8193), struct.pack("<I", 20) + b"{}"):
            with self.assertRaises(ValueError):
                protocol.response(io.BytesIO(value))
        header = json.dumps({"audio_bytes": 10_000_001}).encode()
        with self.assertRaises(ValueError):
            protocol.response(io.BytesIO(struct.pack("<I", len(header)) + header))


if __name__ == "__main__":
    unittest.main()
