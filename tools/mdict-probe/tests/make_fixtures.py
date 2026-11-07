"""Write project-owned fixtures with the independent mdict-utils writer."""

import argparse
import datetime
import hashlib
import io
import json
from pathlib import Path
import struct
import sys
import wave
import zlib
from unittest.mock import patch


class Rows:
    def __init__(self, rows):
        self.rows = rows

    def __len__(self):
        return len(self.rows)

    def items(self):
        return self.rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--writer-parent", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    sys.path.insert(0, str(args.writer_parent.resolve()))
    from writer_support.writemdict import MDictWriter

    def png_chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))

    image = b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0))
    image += png_chunk(b"IDAT", zlib.compress(b"\0\x20\x60\xff\xff")) + png_chunk(b"IEND", b"")
    audio = io.BytesIO()
    with wave.open(audio, "wb") as output:
        output.setparams((1, 2, 8000, 0, "NONE", "not compressed"))
        output.writeframes(struct.pack("<128h", *([0, 1024, 0, -1024] * 32)))
    resources = {"\\theme.css": b"p { color: #2060ff; }\n", "\\img\\œil.png": image,
                 "\\audio\\bonjour.wav": audio.getvalue()}
    entries = Rows([
        ("être", '<p>state</p><link href="theme.css"><img src="img/œil.png">'
                 '<a href="sound://audio/bonjour.wav">sound</a>'),
        ("œil", "<p>eye</p>"), ("aujourd'hui", "<p>today, straight apostrophe</p>"),
        ("aujourd’hui", "<p>today, curved apostrophe</p>"), ("cote", "<p>rating</p>"),
        ("côte", "<p>coast</p>"), ("côté", "<p>side</p>"),
        ("double", "<p>first sense</p>"), ("double", "<p>second sense</p>"),
        ("redirect", "@@@LINK=être"),
    ])
    variants = [("v1-utf8-none", "1.2", "utf8", 0, False),
                ("v1-utf16-none", "1.2", "utf16", 0, False),
                ("v2-utf8-none", "2.0", "utf8", 0, False),
                ("v2-utf8-zlib", "2.0", "utf8", 2, False),
                ("v2-utf16-zlib", "2.0", "utf16", 2, False),
                ("v2-utf8-encrypted-index", "2.0", "utf8", 2, True)]
    args.output.mkdir(parents=True, exist_ok=True)
    manifest = []
    fixture_date = datetime.date(2026, 11, 7)
    for name, version, encoding, compression, encrypted in variants:
        for kind, content in [("mdx", entries), ("mdd", resources)]:
            path = args.output / f"{name}.{kind}"
            writer = MDictWriter(content, title="Lexique compatibility fixture",
                                 description="Original Lexique test data, MIT",
                                 version=version, encoding=encoding, compression_type=compression,
                                 encrypt_index=encrypted, block_size=128, is_mdd=kind == "mdd")
            with patch("writer_support.writemdict.datetime.date") as writer_date, path.open("xb") as output:
                writer_date.today.return_value = fixture_date
                writer.write(output)
            data = path.read_bytes()
            assert len(data) < 16384
            manifest.append(dict(file=path.name, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
                                 version=version, encoding=encoding if kind == "mdx" else "UTF-16LE",
                                 compression=compression, encrypted_index=encrypted,
                                 entries=len(content)))
    source = (args.output / "v2-utf8-none.mdx").read_bytes()
    header_size = struct.unpack(">I", source[:4])[0]
    header = source[4:4 + header_size].decode("utf-16le").replace('Version="2.0"', 'Version="3.0"')
    header_bytes = header.encode("utf-16le")
    unsupported = struct.pack(">I", len(header_bytes)) + header_bytes
    unsupported += struct.pack("<I", zlib.adler32(header_bytes)) + source[8 + header_size:]
    unsupported_path = args.output / "unsupported-v3-header.mdx"
    unsupported_path.write_bytes(unsupported)
    manifest.append(dict(file=unsupported_path.name, bytes=len(unsupported),
                         sha256=hashlib.sha256(unsupported).hexdigest(),
                         expected="reject unsupported v3 header; remaining payload is v2"))
    for key, data in resources.items():
        (args.output / key.rsplit("\\", 1)[-1]).write_bytes(data)
    with wave.open(io.BytesIO(resources["\\audio\\bonjour.wav"]), "rb") as sound:
        assert sound.getnframes() == 128 and sound.getnchannels() == 1
    (args.output / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                                             encoding="utf-8")


if __name__ == "__main__":
    main()
