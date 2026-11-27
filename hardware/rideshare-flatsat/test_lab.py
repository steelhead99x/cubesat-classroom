"""Checks for lab.py. Run: python3 test_lab.py"""
import subprocess
import sys
import tempfile
from pathlib import Path

LAB = Path(__file__).with_name("lab.py")


def lab(*args):
    return subprocess.run([sys.executable, str(LAB), *map(str, args)], capture_output=True, text=True)


def test_strict_compare():
    # Review finding: the five-node fixture reported missing rows but exited 0.
    run = Path(tempfile.mkdtemp()) / "run-5"
    assert lab("simulate", 5, run).returncode == 0
    nodes = sorted(run.glob("node-*.csv"))
    plain = lab("compare", run / "ground.csv", *nodes)
    assert plain.returncode == 0 and "discrepancies=2" in plain.stdout, plain
    strict = lab("compare", "--strict", run / "ground.csv", *nodes)
    assert strict.returncode == 1 and strict.stdout.count("missing at ground") == 2, strict
    recovered = lab("compare", "--strict", run / "ground-recovered.csv", *nodes)
    assert recovered.returncode == 0 and "discrepancies=0" in recovered.stdout, recovered


def test_one_and_three_nodes_match():
    for n in (1, 3):
        run = Path(tempfile.mkdtemp()) / f"run-{n}"
        assert lab("simulate", n, run).returncode == 0
        r = lab("compare", "--strict", run / "ground.csv", *sorted(run.glob("node-*.csv")))
        assert r.returncode == 0, r


def test_parser_accepts_only_recorder_rows():
    sys.path.insert(0, str(LAB.parent))
    import lab as L
    good = "desk simulation,1,00abcdef,0,0,-5,0,1000"
    assert list(L.rows([good]))
    for session in ("0x123456", "+1234567", " 1234567", "1_234567", "00ABCDEF"):
        assert not list(L.rows([good.replace("00abcdef", session)])), session
    # A stray quote must not swallow the rows after it.
    assert len(list(L.rows(['"garbage', good, good]))) == 2


def test_forwarder_survives_serial_noise():
    import socket
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as rx:
        rx.bind(("127.0.0.1", 0))
        rx.settimeout(5)
        port = rx.getsockname()[1]
        noise = b'\xff boot junk\n"garbage\ndesk simulation,1,00000001,0,0,0,0,1000\r\n' \
                b'desk simulation,1,00000001,1,100,0,0,1000\n'
        # A strict UTF-8 locale is what a classroom desktop uses; bad bytes must not crash it.
        r = subprocess.run([sys.executable, str(LAB), "forward", "127.0.0.1", "--port", str(port)],
                           input=noise, capture_output=True, env={"PYTHONIOENCODING": "utf-8:strict"})
        assert r.returncode == 0, r.stderr
        got = [rx.recv(1000).decode() for _ in range(2)]
    assert got[0].endswith(",0,0,0,0,1000") and got[1].split(",")[3] == "1", got


def test_compare_orders_sequences_as_numbers():
    d = Path(tempfile.mkdtemp())
    (d / "node.csv").write_text("".join(f"desk simulation,1,00000001,{n},0,0,0,0\n" for n in (2, 10)))
    (d / "ground.csv").write_text("")
    out = lab("compare", d / "ground.csv", d / "node.csv", d / "node.csv").stdout
    assert out.index(" 2\n") < out.index(" 10\n") and "duplicate_onboard=2" in out, out


def test_receiver_survives_hostile_datagrams():
    import socket
    import time
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    rx = subprocess.Popen([sys.executable, str(LAB), "receive", "127.0.0.1", "--port", str(port)],
                          stdout=subprocess.PIPE, text=True)
    try:
        time.sleep(0.5)
        good = b"desk simulation,1,00000001,0,0,0,0,1000"
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as tx:
            for bad in (b"a\nb", b"x" * 60000, b'"', b"\x00\xff", good):
                tx.sendto(bad, ("127.0.0.1", port))
        time.sleep(0.5)
        assert rx.poll() is None, "the receiver must keep running"
    finally:
        rx.terminate()
        out, _ = rx.communicate(timeout=5)
    assert out.strip() == good.decode(), out


def test_forwarder_drops_an_endless_line():
    import socket
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as rx:
        rx.bind(("127.0.0.1", 0))
        rx.settimeout(5)
        port = rx.getsockname()[1]
        data = b"z" * 300000 + b"\ndesk simulation,1,00000001,7,0,0,0,1000\n"
        r = subprocess.run([sys.executable, str(LAB), "forward", "127.0.0.1", "--port", str(port)],
                           input=data, capture_output=True)
        assert r.returncode == 0, r.stderr
        assert rx.recv(1000).decode().split(",")[3] == "7"


if __name__ == "__main__":
    for name, t in list(globals().items()):
        if name.startswith("test_"):
            t()
            print("ok ", name)
