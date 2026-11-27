"""FlatSat evidence exercises. Uses only the Python standard library."""
import argparse
import csv
import io
import math
import re
import socket
import sys
from pathlib import Path


LABELS = {"hardware bench", "desk simulation"}
SESSION = re.compile(r"[0-9a-f]{8}")
DIGITS = re.compile(r"[0-9]+")
SIGNED = re.compile(r"-?[0-9]+")


MAX_LINE = 200  # a recorder row is under 80 bytes


def rows(lines):
    """Yield valid eight-field records from text lines. Each line is split on commas without the
    csv module, so stray quotes, newlines, NUL bytes or an endless line cannot stop the tools,
    and every field must be exactly the form the recorder writes."""
    for line in lines:
        if len(line) > MAX_LINE:
            continue
        row = line.rstrip("\r\n").split(",")
        if len(row) != 8 or row[0] not in LABELS:
            continue
        node, session, seq, ms, *axes = row[1:]
        if (DIGITS.fullmatch(node) and 1 <= int(node) <= 5
                and SESSION.fullmatch(session)
                and DIGITS.fullmatch(seq) and DIGITS.fullmatch(ms)
                and all(SIGNED.fullmatch(a) for a in axes)):
            yield row


def bounded_lines(stream):
    """Lines from a text stream, reading at most MAX_LINE + 1 characters at a time, so a serial
    port that never sends a newline cannot use up memory. Over-long lines are dropped whole."""
    while True:
        line = stream.readline(MAX_LINE + 1)
        if not line:
            return
        if len(line) > MAX_LINE and not line.endswith("\n"):
            while True:  # skip the rest of the over-long line
                rest = stream.readline(MAX_LINE + 1)
                if not rest or rest.endswith("\n"):
                    break
            continue
        yield line


def sort_key(key):
    """Order by node and sequence as numbers, so sequence 10 comes after 9."""
    label, node, session, seq = key
    return label, int(node), session, int(seq)


def simulate(nodes, directory):
    """Write node-N.csv (the onboard logs) and ground.csv (what the ground received). The
    five-node run drops two of node 3's readings from ground.csv, and also writes
    ground-recovered.csv, the complete feed, for the repeat comparison."""
    directory.mkdir(parents=True, exist_ok=False)
    with (directory / "ground.csv").open("w", newline="") as ground, \
            (directory / "ground-recovered.csv").open("w", newline="") as complete:
        # The recorder writes "\n" line endings; the simulation matches it.
        receiver = csv.writer(ground, lineterminator="\n")
        recovered = csv.writer(complete, lineterminator="\n")
        for node in range(1, nodes + 1):
            with (directory / f"node-{node}.csv").open("w", newline="") as onboard:
                logger = csv.writer(onboard, lineterminator="\n")
                for seq in range(10):
                    angle = math.radians(90 * (seq >= 5) + 30 * (node - 1))
                    row = ["desk simulation", node, f"{node:08x}", seq, seq * 100,
                           round(1000 * math.sin(angle)), 0, round(1000 * math.cos(angle))]
                    logger.writerow(row)
                    recovered.writerow(row)
                    if nodes != 5 or node != 3 or seq not in (4, 5):
                        receiver.writerow(row)


def compare(local, ground):
    """Print the comparison; return the number of discrepancies found."""
    def load(paths):
        result = {}
        duplicates = 0
        for path in paths:
            with path.open(newline="", errors="replace") as stream:
                for row in rows(bounded_lines(stream)):
                    key = tuple(row[:4])
                    if key in result:
                        if result[key] != row:
                            raise ValueError(f"conflicting records for {key}")
                        duplicates += 1
                    result[key] = row
        return result, duplicates

    saved, duplicate_onboard = load(local)
    received, duplicates = load([ground])
    if not saved:
        raise ValueError("no valid onboard records")
    print(f"onboard={len(saved)} ground={len(received)} "
          f"duplicate_onboard={duplicate_onboard} duplicate_ground={duplicates}")
    problems = 0
    for key in sorted(saved.keys() | received.keys(), key=sort_key):
        if key not in received:
            print("missing at ground", *key)
        elif key not in saved:
            print("ground only", *key)
        elif received[key] != saved[key]:
            print("payload mismatch", *key)
        else:
            continue
        problems += 1
    print(f"discrepancies={problems}")
    return problems


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="cmd", required=True)
    demo = commands.add_parser("simulate")
    demo.add_argument("nodes", type=int, choices=[1, 3, 5])
    demo.add_argument("directory", type=Path)
    audit = commands.add_parser("compare")
    audit.add_argument("ground", type=Path)
    audit.add_argument("onboard", type=Path, nargs="+")
    audit.add_argument("--strict", action="store_true",
                       help="exit 1 if any reading is missing, extra or different")
    for name in ("forward", "receive"):
        cmd = commands.add_parser(name)
        cmd.add_argument("address", help="destination for forward, local interface for receive")
        cmd.add_argument("--port", type=int, default=5715)
    args = parser.parse_args()
    if args.cmd == "simulate":
        try:
            simulate(args.nodes, args.directory)
        except FileExistsError:
            parser.error(f"{args.directory} already exists; choose a new directory to keep earlier evidence")
    elif args.cmd == "compare":
        try:
            problems = compare(args.onboard, args.ground)
        except ValueError as err:
            sys.exit(f"compare: {err}")
        if args.strict and problems:
            sys.exit(1)
    else:
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                if args.cmd == "forward":
                    # Serial ports emit stray bytes at boot; never let them stop the forwarder.
                    serial = io.TextIOWrapper(sys.stdin.buffer, encoding="ascii",
                                              errors="replace", newline="")
                    for row in rows(bounded_lines(serial)):
                        sock.sendto(",".join(row).encode("ascii"), (args.address, args.port))
                else:
                    sock.bind((args.address, args.port))
                    while True:
                        packet, _ = sock.recvfrom(65535)
                        if len(packet) > MAX_LINE:
                            continue
                        text = packet.decode("ascii", errors="replace").splitlines()
                        for row in rows(text[:1]):
                            print(",".join(row), flush=True)
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
