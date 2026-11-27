"""Tests for ground.py: python3 -m unittest test_ground (from this directory)."""
import csv
import io
from contextlib import redirect_stdout
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
            runs = ground.report(run / "power.csv", max_gap_s=10)
            self.assertEqual([s for s, *_ in runs], [1, -1] * 4)
            # 45 min of 60 mA dark draw is 45 mAh
            for s, a, b, q in runs:
                if s == -1:
                    self.assertAlmostEqual(q, -45.0, places=6)

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
            with self.assertRaises(ValueError):
                ground.report(p)

    def test_small_currents_count_as_idle(self):
        samples = [(0, 2.0, 3.8), (60, 2.0, 3.8), (120, -40.0, 3.8), (180, -40.0, 3.8)]
        runs = ground.periods(samples, threshold_ma=5.0, max_gap_s=60)
        self.assertEqual([s for s, *_ in runs], [0, -1])


    def test_report_excludes_gaps_and_does_not_infer_illumination(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "power.csv"
            path.write_text("t_ms,current_ma,bus_v,power_mw\n"
                            "0,60,3.8,228\n1000,60,3.8,228\n"
                            "60000,60,3.8,228\n61000,-60,3.8,-228\n62000,-60,3.8,-228\n")
            output = io.StringIO()
            with redirect_stdout(output):
                runs = ground.report(path)
            self.assertEqual([(s, a, b) for s, a, b, _ in runs], [(1, 0, 1), (1, 60, 61), (-1, 61, 62)])
            self.assertAlmostEqual(sum(q for *_, q in runs), 60 / 3600)
            self.assertIn("Excluded 59.0 s", output.getvalue())
            self.assertNotIn("breaks even", output.getvalue())

    def test_logger_restart_duplicate_and_nonfinite_readings_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "power.csv"
            for row in ("0,60,3.8,228", "1000,60,3.8,228", "1500,nan,3.8,0"):
                with self.subTest(row=row):
                    path.write_text("t_ms,current_ma,bus_v,power_mw\n1000,60,3.8,228\n" + row + "\n")
                    with self.assertRaisesRegex(ValueError, "line 3"):
                        ground.report(path)

    def test_invalid_simulation_does_not_create_evidence_directory(self):
        with tempfile.TemporaryDirectory() as d:
            run = Path(d) / "run"
            with self.assertRaises(ValueError):
                ground.simulate(run, cell_mah=0)
            self.assertFalse(run.exists())

    def test_simulation_steps_end_at_light_and_dark_boundaries(self):
        with tempfile.TemporaryDirectory() as d:
            run = Path(d) / "run"
            ground.simulate(run, lit_min=1, dark_min=1, cycles=1, step_s=17)
            samples = ground.load(run / "power.csv")
            runs = ground.periods(samples, max_gap_s=17)
            self.assertEqual([(sign, start, end) for sign, start, end, _ in runs], [(1, 0, 60), (-1, 60, 120)])


if __name__ == "__main__":
    unittest.main()
