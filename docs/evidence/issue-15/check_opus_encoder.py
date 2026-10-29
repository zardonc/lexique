"""Check a locally built libopusenc against an existing PCM WAV sample."""

import argparse
import ctypes
from pathlib import Path
import wave


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("library", type=Path)
    parser.add_argument("sample", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(args.output)
    with wave.open(str(args.sample), "rb") as source:
        if source.getnchannels() != 1 or source.getsampwidth() != 2:
            raise ValueError("Expected mono PCM16")
        rate, frames = source.getframerate(), source.getnframes()
        pcm = source.readframes(frames)
    library = ctypes.CDLL(str(args.library.resolve()))
    library.ope_comments_create.restype = ctypes.c_void_p
    library.ope_comments_destroy.argtypes = [ctypes.c_void_p]
    library.ope_encoder_create_file.argtypes = [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_int,
                                                ctypes.c_int, ctypes.c_int, ctypes.POINTER(ctypes.c_int)]
    library.ope_encoder_create_file.restype = ctypes.c_void_p
    library.ope_encoder_ctl.argtypes = [ctypes.c_void_p, ctypes.c_int]
    library.ope_encoder_write.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_int16), ctypes.c_int]
    library.ope_encoder_drain.argtypes = [ctypes.c_void_p]
    library.ope_encoder_destroy.argtypes = [ctypes.c_void_p]
    comments = library.ope_comments_create()
    if not comments:
        raise RuntimeError("Could not create Opus comments")
    encoder = None
    try:
        error = ctypes.c_int()
        encoder = library.ope_encoder_create_file(str(args.output.resolve()).encode("utf-8"), comments,
                                                   rate, 1, 0, ctypes.byref(error))
        if not encoder or error.value:
            raise RuntimeError(f"Could not create Opus encoder: {error.value}")
        samples = (ctypes.c_int16 * frames).from_buffer_copy(pcm)
        for operation, call in (("set bitrate", lambda: library.ope_encoder_ctl(encoder, 4002, ctypes.c_int(48000))),
                                ("write", lambda: library.ope_encoder_write(encoder, samples, frames)),
                                ("drain", lambda: library.ope_encoder_drain(encoder))):
            result = call()
            if result:
                raise RuntimeError(f"Could not {operation}: {result}")
    finally:
        if encoder:
            library.ope_encoder_destroy(encoder)
        library.ope_comments_destroy(comments)
    if args.output.read_bytes()[:4] != b"OggS":
        raise ValueError("Not an Ogg stream")
    print(f"Encoded {frames} frames at {rate} Hz to {args.output.stat().st_size} bytes")


if __name__ == "__main__":
    main()
