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


if __name__ == "__main__":
    for name, t in list(globals().items()):
        if name.startswith("test_"):
            t()
            print("ok ", name)
