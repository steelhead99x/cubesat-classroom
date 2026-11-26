"""Tests for ground.py: python3 -m unittest test_ground (from this directory)."""
import csv
import tempfile
import unittest
from pathlib import Path

import ground


class Ground(unittest.TestCase):
    def test_face_figures_match_the_solar_face_page(self):
        # 0.74 W and 2.42 W rated (hardware/solar-face/README.md)
        self.assertAlmostEqual(ground.FACES["si"]["vmp"] * ground.FACES["si"]["imp"], 0.737, places=2)
        self.assertAlmostEqual(ground.FACES["tj"]["vmp"] * ground.FACES["tj"]["imp"], 2.42, places=2)
        self.assertAlmostEqual(ground.charge_ma("si", 500), ground.charge_ma("si", 1000) / 2)

    def test_simulated_run_reports_four_light_and_four_dark_periods(self):
        with tempfile.TemporaryDirectory() as d:
            run = Path(d) / "run"
            ground.simulate(run, cycles=4)
            runs = ground.report(run / "power.csv")
            self.assertEqual([s for s, *_ in runs], [1, -1] * 4)
            # 45 min of 60 mA dark draw is 45 mAh
            for s, a, b, q in runs:
                if s == -1:
                    self.assertAlmostEqual(q, -45.0, delta=0.5)

    def test_simulation_refuses_to_overwrite_evidence(self):
        with tempfile.TemporaryDirectory() as d:
            ground.simulate(Path(d) / "run", cycles=1)
            with self.assertRaises(FileExistsError):
                ground.simulate(Path(d) / "run", cycles=1)

    def test_rejects_a_file_without_the_logger_columns(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "bad.csv"
            with p.open("w", newline="") as f:
                csv.writer(f).writerows([["label", "node"], ["hardware bench", "1"]])
            with self.assertRaises(SystemExit):
                ground.report(p)

    def test_small_currents_count_as_idle(self):
        samples = [(0, 2.0, 3.8), (60, 2.0, 3.8), (120, -40.0, 3.8), (180, -40.0, 3.8)]
        runs = ground.periods(samples, threshold_ma=5.0)
        self.assertEqual([s for s, *_ in runs], [0, -1])


if __name__ == "__main__":
    unittest.main()
