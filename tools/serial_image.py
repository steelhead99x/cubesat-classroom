#!/usr/bin/env python3
"""Rebuild a photo sent by the XIAO bench CubeSat's DOWNLINK command.

Save the Serial Monitor output to a text file, then run:

    python3 tools/serial_image.py capture.txt photo.jpg

The script finds the last `IMG BEGIN ... IMG END` block, puts the hex chunks
back in order, and checks the byte count and CRC-32 that the CubeSat sent.
Missing chunks are reported by offset and filled with zeros, so a crew can see
which part of the picture was lost. Other lines (CSC1, REC, # bench) are ignored.
"""

import argparse
import re
import sys
import zlib

BEGIN = re.compile(r"^IMG BEGIN\b(.*)$")
CHUNK = re.compile(r"^IMG (\d+) ([0-9A-Fa-f]+)$")


def parse_fields(text):
    return dict(part.split("=", 1) for part in text.split() if "=" in part)


def last_block(lines):
    block, header = None, None
    for raw in lines:
        line = raw.strip()
        match = BEGIN.match(line)
        if match:
            header, block = parse_fields(match.group(1)), []
        elif line == "IMG END" and block is not None:
            yield header, block
            block = None
        elif block is not None:
            block.append(line)
    if block is not None:
        yield header, block  # the END line never arrived


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("log", help="saved Serial Monitor text, or - for standard input")
    parser.add_argument("output", help="where to write the JPEG")
    args = parser.parse_args()

    source = sys.stdin if args.log == "-" else open(args.log, encoding="utf-8", errors="replace")
    with source:
        blocks = list(last_block(source))
    if not blocks:
        sys.exit("No IMG BEGIN line found. Send PHOTO, then DOWNLINK, and save all of the output.")
    header, chunks = blocks[-1]

    try:
        expected = int(header["bytes"])
        crc_sent = int(header["crc32"], 16)
    except (KeyError, ValueError):
        sys.exit("The IMG BEGIN line is missing bytes= or crc32=.")

    image = bytearray(expected)
    have = []
    for line in chunks:
        match = CHUNK.match(line)
        if not match:
            continue
        offset, data = int(match.group(1)), match.group(2)
        if len(data) % 2:
            continue  # a cut-off line; treat it as lost
        piece = bytes.fromhex(data)
        if offset + len(piece) > expected:
            continue
        image[offset:offset + len(piece)] = piece
        have.append((offset, offset + len(piece)))

    have.sort()
    gaps, position = [], 0
    for start, end in have:
        if start > position:
            gaps.append((position, start))
        position = max(position, end)
    if position < expected:
        gaps.append((position, expected))

    with open(args.output, "wb") as out:
        out.write(image)

    crc_got = zlib.crc32(image) & 0xFFFFFFFF
    print(f"photo {header.get('n', '?')}: {expected} bytes written to {args.output}")
    if gaps:
        lost = sum(end - start for start, end in gaps)
        print(f"missing {lost} bytes in {len(gaps)} gap(s):")
        for start, end in gaps:
            print(f"  bytes {start} to {end - 1}")
    if crc_got == crc_sent:
        print(f"CRC-32 {crc_got:08X} matches: the photo arrived intact")
    else:
        print(f"CRC-32 {crc_got:08X} does not match the {crc_sent:08X} the CubeSat sent")
        sys.exit(1)


if __name__ == "__main__":
    main()
