#!/usr/bin/env python3
"""Spherical Earth, vacuum path and first-order one-way link budget for design review."""
import argparse
import json
import math
from pathlib import Path

C_M_S = 299_792_458
EARTH_RADIUS_KM = 6371.0
BOLTZMANN_DBW_K_HZ = -228.599


def slant_km(altitude_km, elevation_deg):
    r = EARTH_RADIUS_KM
    projection = r * math.sin(math.radians(elevation_deg))
    return math.sqrt((r + altitude_km) ** 2 - r**2 + projection**2) - projection


def budget(profile, distance_km, radial_speed_km_s):
    frequency_hz = profile["center_mhz"] * 1e6
    fspl_db = 20 * math.log10(4 * math.pi * distance_km * 1000 * frequency_hz / C_M_S)
    eirp_dbw = profile["pa_dbw"] + profile["tx_gain_dbi"] - profile["tx_feed_loss_db"]
    cn0_dbhz = eirp_dbw + profile["rx_gt_db_per_k"] - fspl_db - profile["other_losses_db"] - BOLTZMANN_DBW_K_HZ
    ebn0_db = cn0_dbhz - 10 * math.log10(profile["information_bps"])
    return dict(
        slant_km=distance_km,
        fspl_db=fspl_db,
        cn0_dbhz=cn0_dbhz,
        ebn0_db=ebn0_db,
        margin_db=ebn0_db - profile["required_ebn0_db"],
        abs_doppler_khz=frequency_hz * radial_speed_km_s / C_M_S,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", type=Path)
    parser.add_argument("--altitude-km", type=float, default=550)
    parser.add_argument("--elevation-deg", type=float, default=15)
    parser.add_argument("--radial-speed-km-s", type=float, default=7)
    args = parser.parse_args()
    if not (math.isfinite(args.altitude_km) and args.altitude_km > 0
            and 0 <= args.elevation_deg <= 90
            and 0 <= args.radial_speed_km_s < C_M_S / 1000):
        parser.error("require positive finite altitude, elevation 0–90 and nonnegative sublight radial speed")
    scenario = json.loads(args.profile.read_text())
    distance = slant_km(args.altitude_km, args.elevation_deg)
    print(scenario["label"])
    print(f"{distance:.1f} km slant at {args.elevation_deg:g}° elevation. Doppler uses a separate assumed radial speed.")
    print(f"{'Band':<9} {'FSPL dB':>8} {'C/N0 dBHz':>10} {'Eb/N0 dB':>9} {'Margin dB':>10} {'|Doppler| kHz':>14}")
    for profile in scenario["profiles"]:
        row = budget(profile, distance, args.radial_speed_km_s)
        print(f'{profile["name"]:<9} {row["fspl_db"]:8.1f} {row["cn0_dbhz"]:10.1f} '
              f'{row["ebn0_db"]:9.1f} {row["margin_db"]:10.1f} {row["abs_doppler_khz"]:14.1f}')


if __name__ == "__main__":
    main()
