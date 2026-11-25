#!/usr/bin/env python3
"""Tests for tools/serial_image.py. Run: python3 tools/test_serial_image.py"""

import os
import sys
import tempfile
import unittest
import zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import serial_image  # noqa: E402

CHUNK = 48


def capture(data, drop=()):
    """Serial Monitor text for a DOWNLINK of `data`, leaving out the chunk offsets in `drop`."""
    crc = zlib.crc32(data) & 0xFFFFFFFF
    lines = ["CSC1 seq=1 noise", f"IMG BEGIN n=1 bytes={len(data)} crc32={crc:08X}"]
    for off in range(0, len(data), CHUNK):
        if off not in drop:
            lines.append(f"IMG {off} {data[off:off + CHUNK].hex().upper()}")
    lines.append("IMG END")
    return "\n".join(lines) + "\n"


def run(text):
    with tempfile.TemporaryDirectory() as tmp:
        log, out = os.path.join(tmp, "capture.txt"), os.path.join(tmp, "photo.jpg")
        with open(log, "w", encoding="utf-8") as f:
            f.write(text)
        with open(os.devnull, "w") as quiet:
            old, sys.stdout = sys.stdout, quiet
            try:
                code = serial_image.main([log, out])
            finally:
                sys.stdout = old
        with open(out, "rb") as f:
            return code, f.read()


# A small JPEG-shaped fixture with a 48-byte run of zeros at offset 96.
FIXTURE = b"\xff\xd8\xff\xe0" + bytes(range(92)) + bytes(CHUNK) + bytes(range(200, 256)) * 2 + b"\xff\xd9"


class SerialImageTests(unittest.TestCase):
    def test_complete_transfer_is_intact(self):
        code, image = run(capture(FIXTURE))
        self.assertEqual(code, 0)
        self.assertEqual(image, FIXTURE)

    def test_missing_zero_chunk_is_incomplete_even_though_crc_matches(self):
        # The reviewer's case: the lost chunk is all zeros, so the zero fill restores the
        # CRC exactly. That must still be reported as an incomplete transfer.
        _, gaps, sent, got = serial_image.rebuild(
            {"bytes": str(len(FIXTURE)), "crc32": f"{zlib.crc32(FIXTURE) & 0xFFFFFFFF:08X}"},
            [l for l in capture(FIXTURE, drop={96}).splitlines() if l.startswith("IMG ") and "BEGIN" not in l and l != "IMG END"],
        )
        self.assertEqual(sent, got, "fixture must reproduce the CRC coincidence")
        self.assertEqual(gaps, [(96, 144)])
        code, _ = run(capture(FIXTURE, drop={96}))
        self.assertEqual(code, 1)

    def test_missing_data_chunk_fails(self):
        code, _ = run(capture(FIXTURE, drop={48}))
        self.assertEqual(code, 1)

    def test_missing_last_chunk_fails(self):
        last = (len(FIXTURE) - 1) // CHUNK * CHUNK
        code, _ = run(capture(FIXTURE, drop={last}))
        self.assertEqual(code, 1)

    def test_gap_finder(self):
        self.assertEqual(serial_image.find_gaps([(0, 10), (20, 30)], 40), [(10, 20), (30, 40)])
        self.assertEqual(serial_image.find_gaps([(0, 10), (5, 40)], 40), [])
        self.assertEqual(serial_image.find_gaps([], 0), [])


if __name__ == "__main__":
    unittest.main()
