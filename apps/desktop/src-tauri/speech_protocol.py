"""Bounded private resident-worker framing; model and transport are injected."""
from concurrent.futures import ThreadPoolExecutor
import json
import struct
import threading
import time

MAX_AUDIO = 10_000_000
MAX_FRAME = 8192


def request(line):
    if len(line) > MAX_FRAME or not line.endswith(b"\n"):
        raise ValueError("Invalid speech frame length")
    value = json.loads(line)
    if not isinstance(value, dict) or set(value) != {"id", "text", "voice"}:
        raise ValueError("Invalid speech frame fields")
    if type(value["id"]) is not int or not 0 < value["id"] <= 9_007_199_254_740_991:
        raise ValueError("Invalid speech request identity")
    text = value["text"]
    if not isinstance(text, str) or not text.strip() or len(text) > 200 or any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in text):
        raise ValueError("Invalid speech text")
    if value["voice"] not in ("female", "male"):
        raise ValueError("Invalid speech voice")
    return value


def response(stream):
    prefix = stream.read(4)
    if len(prefix) != 4:
        raise ValueError("Truncated speech response")
    length = struct.unpack("<I", prefix)[0]
    if not 0 < length <= MAX_FRAME:
        raise ValueError("Invalid speech response header size")
    header = stream.read(length)
    if len(header) != length:
        raise ValueError("Truncated speech response header")
    value = json.loads(header)
    if not isinstance(value, dict):
        raise ValueError("Invalid speech response fields")
    size = value.get("audio_bytes")
    if type(size) is not int or not 0 <= size <= MAX_AUDIO:
        raise ValueError("Invalid speech response audio size")
    if value.get("id") == 0:
        valid = set(value) == {"id", "ready", "audio_bytes"} and value["ready"] is True and size == 0
    elif "error" in value:
        valid = set(value) == {"id", "voice", "error", "retryable", "audio_bytes"} and isinstance(value["error"], str) and bool(value["error"]) and type(value["retryable"]) is bool and size == 0
    else:
        valid = set(value) == {"id", "voice", "language", "generated_at", "audio_bytes"} and value["language"] == "fr" and type(value["generated_at"]) is int and value["generated_at"] >= 0 and size > 0
    if not valid or type(value.get("id")) is not int or not 0 <= value["id"] <= 9_007_199_254_740_991 or (value["id"] and value.get("voice") not in ("female", "male")):
        raise ValueError("Invalid speech response identity")
    audio = stream.read(size)
    if len(audio) != size:
        raise ValueError("Truncated speech response audio")
    return value, audio


def serve(synthesize, source, destination, capacity=3):
    if not 1 <= capacity <= 8:
        raise ValueError("Invalid speech worker capacity")
    slots = threading.Semaphore(capacity)
    output = threading.Lock()

    def send(header, audio=b""):
        encoded = json.dumps({**header, "audio_bytes": len(audio)}, separators=(",", ":")).encode()
        if len(encoded) > MAX_FRAME:
            raise ValueError("Speech response header too large")
        with output:
            destination.write(struct.pack("<I", len(encoded)) + encoded + audio)
            destination.flush()

    def execute(value):
        try:
            try:
                audio = synthesize(value["text"], value["voice"])
                if not isinstance(audio, bytes) or not audio.startswith(b"OggS") or not 0 < len(audio) <= MAX_AUDIO:
                    raise ValueError("Invalid generated speech audio")
            except Exception as error:
                send({"id": value["id"], "voice": value["voice"], "error": str(error)[:2048], "retryable": not isinstance(error, (ValueError, OSError))})
            else:
                send({"id": value["id"], "voice": value["voice"], "language": "fr", "generated_at": int(time.time())}, audio)
        finally:
            slots.release()

    send({"id": 0, "ready": True})
    previous = 0
    futures = []
    with ThreadPoolExecutor(max_workers=capacity) as pool:
        while line := source.readline(MAX_FRAME + 1):
            value = request(line)
            if value["id"] <= previous:
                raise ValueError("Reused or unordered speech request identity")
            previous = value["id"]
            slots.acquire()
            for future in futures:
                if future.done():
                    future.result()
            futures = [future for future in futures if not future.done()]
            futures.append(pool.submit(execute, value))
        for future in futures:
            future.result()
