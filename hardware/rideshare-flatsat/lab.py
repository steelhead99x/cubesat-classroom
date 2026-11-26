"""FlatSat evidence exercises. Uses only the Python standard library."""
import argparse
import csv
import math
import socket
import sys
from pathlib import Path


def rows(stream):
    for row in csv.reader(stream):
        if len(row) != 8 or row[0] not in {"hardware bench", "desk simulation"}:
            continue
        try:
            node, seq, ms, x, y, z = map(int, [row[1], *row[3:]])
            int(row[2], 16)
        except ValueError:
            continue
        if 1 <= node <= 5 and len(row[2]) == 8 and min(seq, ms) >= 0:
            yield row


def simulate(nodes, directory):
    """Write node-N.csv (the onboard logs) and ground.csv (what the ground received). The
    five-node run drops two of node 3's readings from ground.csv, and also writes
    ground-recovered.csv, the complete feed, for the repeat comparison."""
    directory.mkdir(parents=True, exist_ok=False)
    with (directory / "ground.csv").open("w", newline="") as ground, \
            (directory / "ground-recovered.csv").open("w", newline="") as complete:
        receiver = csv.writer(ground)
        recovered = csv.writer(complete)
        for node in range(1, nodes + 1):
            with (directory / f"node-{node}.csv").open("w", newline="") as onboard:
                logger = csv.writer(onboard)
                for seq in range(10):
                    angle = math.radians(90 * (seq >= 5) + 30 * (node - 1))
                    row = ["desk simulation", node, f"{node:08x}", seq, seq * 100,
                           round(1000 * math.sin(angle)), 0, round(1000 * math.cos(angle))]
                    logger.writerow(row)
                    recovered.writerow(row)
                    if nodes != 5 or node != 3 or seq not in (4, 5):
                        receiver.writerow(row)


def compare(local, ground):
    def load(paths):
        result = {}
        duplicates = 0
        for path in paths:
            with path.open(newline="") as stream:
                for row in rows(stream):
                    key = tuple(row[:4])
                    if key in result:
                        if result[key] != row:
                            raise ValueError(f"conflicting records for {key}")
                        duplicates += 1
                    result[key] = row
        return result, duplicates

    saved, _ = load(local)
    received, duplicates = load([ground])
    if not saved:
        raise ValueError("no valid onboard records")
    print(f"onboard={len(saved)} ground={len(received)} duplicate_ground={duplicates}")
    for key in sorted(saved.keys() | received.keys()):
        if key not in received:
            print("missing at ground", *key)
        elif key not in saved:
            print("ground only", *key)
        elif received[key] != saved[key]:
            print("payload mismatch", *key)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="cmd", required=True)
    demo = commands.add_parser("simulate")
    demo.add_argument("nodes", type=int, choices=[1, 3, 5])
    demo.add_argument("directory", type=Path)
    audit = commands.add_parser("compare")
    audit.add_argument("ground", type=Path)
    audit.add_argument("onboard", type=Path, nargs="+")
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
        compare(args.onboard, args.ground)
    else:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            if args.cmd == "forward":
                for row in rows(sys.stdin):
                    sock.sendto(",".join(row).encode("ascii"), (args.address, args.port))
            else:
                sock.bind((args.address, args.port))
                while True:
                    packet, _ = sock.recvfrom(65535)
                    try:
                        row = next(rows([packet.decode("ascii")]), None)
                    except (UnicodeError, csv.Error):
                        continue
                    if row:
                        print(",".join(row), flush=True)


if __name__ == "__main__":
    main()
