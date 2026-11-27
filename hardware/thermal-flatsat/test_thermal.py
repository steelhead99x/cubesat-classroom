from pathlib import Path
import unittest

from thermal import bench, model


class ThermalTest(unittest.TestCase):
    def test_bench_interpolates_and_handles_an_unreached_limit(self):
        fixture = Path(__file__).with_name("example.csv")
        self.assertEqual(bench(fixture, 10), (1200, 4, 21, 700))
        self.assertIsNone(bench(fixture, 20)[-1])
        self.assertEqual(bench(fixture, 4)[-1], 0)

    def test_model_separates_thermal_and_electrical_power(self):
        hours, leak, cooler, margin = model(0.05, 0.10, 24, 0.50, 0.60, 0.20, 0.05)
        self.assertAlmostEqual(hours, 22.3056, places=4)
        self.assertAlmostEqual(leak, 0.09294, places=5)
        self.assertEqual(cooler, 2)
        self.assertAlmostEqual(margin, -1.9)
        self.assertIsNone(model(0.05, 0.10, 24, 0, 0, 0, 0)[-1])

    def test_nonfinite_inputs_are_rejected(self):
        with self.assertRaises(ValueError):
            model(0.05, float("nan"), 24, 0, 0, 0, 0)
