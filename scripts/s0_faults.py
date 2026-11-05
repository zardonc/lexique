"""Deterministic #25 clocks, worker outcomes and publication/GC rendezvous.

These are fixture controls, not implementations of future selection/GC protocols.
"""
from dataclasses import dataclass
import hashlib
import threading

POINTS = ("intent_committed", "tmp_written", "tmp_synced", "renamed", "readback",
    "db_complete", "gc_marked", "gc_deleted", "gc_finished")


@dataclass
class Clock:
    now: int

    def set(self, now):
        self.now = now


class Rendezvous:
    """A test waits for entry, performs its competing action, then releases it."""
    def __init__(self, point):
        if point not in POINTS:
            raise ValueError("Unknown publication/GC boundary")
        self.point = point
        self.entered = threading.Event()
        self.released = threading.Event()

    def reach(self, point):
        if point == self.point:
            self.entered.set()
            if not self.released.wait(5):
                raise TimeoutError(f"Fixture barrier not released: {point}")

    def wait(self):
        if not self.entered.wait(5):
            raise TimeoutError(f"Fixture boundary not reached: {self.point}")

    def release(self):
        self.released.set()


class Worker:
    """Inject into the existing speech_protocol.serve worker boundary."""
    def __init__(self, barrier=None, error=None, audio=b"OggSfixture"):
        self.barrier, self.error, self.audio = barrier, error, audio
        self.calls = []

    def __call__(self, text, voice):
        self.calls.append((text, voice))
        if self.barrier:
            self.barrier.reach(self.barrier.point)
        if self.error:
            raise self.error
        return self.audio


def files(root, decoded_audio=None):
    """Real files with declared integrity expectations, including an unknown fault."""
    root.mkdir(exist_ok=False)
    header = b"OggS" + b"x" * 8188
    values = {"header_only": header, "invalid_header": b"BAD!" + header[4:],
        "truncated": header[:12], "empty": b"", "wrong_sha": header[:-1] + b"y",
        "oversize": b"OggS" + b"x" * 10_000_000}
    if decoded_audio is not None:
        values["decoded_source"] = decoded_audio
        values["truncated"] = decoded_audio[:len(decoded_audio) // 2]
    manifest = {}
    for name, value in values.items():
        (root / f"{name}.ogg").write_bytes(value)
        manifest[name] = {"bytes": len(value), "sha256": hashlib.sha256(value).hexdigest(),
            "expected_sha256": hashlib.sha256(header if name == "wrong_sha" else value).hexdigest(),
            "playback_evidence": False, "validation": "unknown" if name == "wrong_sha" else "fixture"}
    manifest["missing"] = {"bytes": 0, "exists": False}
    return manifest
