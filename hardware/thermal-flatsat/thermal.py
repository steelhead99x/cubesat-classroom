"""Summarize a cold-pack bench log or model an ideal liquid-nitrogen heat budget."""
import argparse
import csv
import math
from pathlib import Path

# NIST gives 160.6 J/cm³ at the normal boiling point.
NITROGEN_J_PER_LITER = 160_600


def bench(path, limit_c):
    """Read one run. Return its duration, starting temperatures and first crossing."""
    if not math.isfinite(limit_c):
        raise ValueError("temperature limit must be finite")
    with path.open(newline="") as stream:
        reader = csv.DictReader(stream)
        columns = {"elapsed_s", "cold_c", "ambient_c"}
        if not columns.issubset(reader.fieldnames or ()):
            raise ValueError("expected elapsed_s,cold_c,ambient_c")
        samples = []
        for row in reader:
            values = tuple(float(row[column]) for column in ("elapsed_s", "cold_c", "ambient_c"))
            if not all(math.isfinite(value) for value in values):
                raise ValueError(f"line {reader.line_num}: measurements must be finite")
            if values[0] < 0 or samples and values[0] <= samples[-1][0]:
                raise ValueError(f"line {reader.line_num}: elapsed time must increase")
            samples.append(values)
    if len(samples) < 2:
        raise ValueError("at least two samples are needed")
    start, cold, ambient = samples[0]
    crossed = 0.0 if cold >= limit_c else None
    if crossed is None:
        for (t0, c0, _), (t1, c1, _) in zip(samples, samples[1:]):
            if c0 < limit_c <= c1:
                crossed = t0 - start + (t1 - t0) * (limit_c - c0) / (c1 - c0)
                break
    return samples[-1][0] - start, cold, ambient, crossed


def model(liters, heat_w, target_hours, solar_w, light_fraction, load_w, cop):
    """Return ideal passive hold, heat-leak target and average electrical margin."""
    if not all(math.isfinite(value) for value in (liters, heat_w, target_hours, solar_w, light_fraction, load_w, cop)):
        raise ValueError("model inputs must be finite")
    if min(liters, heat_w, target_hours) <= 0 or min(solar_w, load_w, cop) < 0 or not 0 <= light_fraction <= 1:
        raise ValueError("use positive volume, heat leak and duration, nonnegative power and COP, and a light fraction from 0 to 1")
    energy_j = liters * NITROGEN_J_PER_LITER
    cooler_w = heat_w / cop if cop else None
    average_solar_w = solar_w * light_fraction
    margin_w = average_solar_w - load_w - cooler_w if cooler_w is not None else None
    return energy_j / heat_w / 3600, energy_j / (target_hours * 3600), cooler_w, margin_w


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    report = commands.add_parser("bench", help="summarize a cold-pack CSV")
    report.add_argument("csv", type=Path)
    report.add_argument("--limit-c", type=float, default=10.0)
    estimate = commands.add_parser("model", help="compare ideal heat and electrical budgets")
    estimate.add_argument("--liters", type=float, required=True)
    estimate.add_argument("--heat-w", type=float, required=True, help="assumed heat leak into the cold stage")
    estimate.add_argument("--target-hours", type=float, default=24)
    estimate.add_argument("--solar-w", type=float, default=0, help="usable electrical watts while lit")
    estimate.add_argument("--light-fraction", type=float, default=0)
    estimate.add_argument("--load-w", type=float, default=0, help="average electronics load")
    estimate.add_argument("--cop", type=float, default=0, help="assumed cold-stage heat lifted per electrical watt")
    args = parser.parse_args()
    try:
        if args.command == "bench":
            duration, cold, ambient, crossed = bench(args.csv, args.limit_c)
            print(f"Recorded {duration / 60:.2f} min. Started at {cold:.2f} °C cold, {ambient:.2f} °C ambient.")
            if crossed is None:
                print(f"Stayed below {args.limit_c:g} °C through the last sample.")
            else:
                print(f"Reached {args.limit_c:g} °C at {crossed / 60:.2f} min (linear interpolation).")
        else:
            hold, leak, power, margin = model(args.liters, args.heat_w, args.target_hours,
                                             args.solar_w, args.light_fraction, args.load_w, args.cop)
            print(f"Desk simulation: {hold:.2f} h ideal passive hold.")
            print(f"Target of {args.target_hours:g} h requires an average heat leak of at most {leak:.4f} W in this model.")
            if power is not None:
                print(f"Estimated cooler input {power:.3f} W. Average solar margin after electronics and cooling {margin:+.3f} W.")
    except (OSError, ValueError, TypeError) as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
