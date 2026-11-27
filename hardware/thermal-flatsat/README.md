# Thermal FlatSat

A ground-test fork of the [solar FlatSat](../rideshare-flatsat-ground/README.md). Use the existing logger workflow to ask how long a cold stage stays below a chosen temperature and whether solar power can support the electronics. The bench uses a refrigerator-cooled reusable pack. Liquid nitrogen storage is a desk study in this version.

**Status:** Bench instructions, a synthetic fixture and analysis tools are available. Physical measurements and a cryogenic payload design remain open.

## Compare two bench runs

1. Use the XIAO logger you already have, two temperature probes rated for the test range, a reusable cold pack, and a dry tray. Keep the logger, USB cable and solar or battery hardware dry and outside the cold area. Start with USB power.
2. Put the two probes side by side at room temperature and record their readings. Place one against the cold pack, and place the other nearby in room air. Keep the probe positions fixed between runs.
3. Start a run with the bare pack. Write down the start time, room temperature and pack temperature. Record both temperatures at a fixed interval until the pack crosses your chosen limit or the session ends.
4. Repeat with the same pack starting at about the same temperature, with a removable layer of insulation around it. Compare the time below your chosen limit. Record the material, initial temperatures, sample interval and any gaps. Use paper and pencil for observations.
5. Save readings as `elapsed_s,cold_c,ambient_c`. Keep the raw XIAO log and note any conversion. Mark these results **hardware bench**. Tag the included `example.csv` **desk simulation**.

```sh
python3 thermal.py bench example.csv --limit-c 10
python3 -m unittest discover
```

A useful first comparison has two runs starting within 1 °C of each other, with the same probe positions and sampling interval. The script estimates the first crossing between recorded samples. Report a run that ends before crossing as a lower bound on hold time.

## Model the long-duration question

Nitrogen boils at about 77.4 K at one atmosphere. NIST reports about 160.6 kJ of latent heat per liter at that point. In a constant-heat-leak model, the latent heat divided by the heat leak gives an ideal passive hold time. The heat leak in this calculation is an assumed input. A cold-pack test can compare two bench arrangements without providing a heat-leak value at 77 K. [NIST cryogenic fluid properties](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=50016)

```sh
python3 thermal.py model --liters 0.05 --heat-w 0.10 --target-hours 24 \
  --solar-w 0.50 --light-fraction 0.60 --load-w 0.20 --cop 0.05
```

The example gives about 22.3 hours of ideal passive hold and a 2 W cooler input at the assumed coefficient of performance. The average electrical margin is about −1.9 W. Each input is illustrative. Use measured usable panel watts from the solar FlatSat when available. A positive average margin would still leave eclipse storage, startup power and warm-side heat rejection to check.

A future liquid-nitrogen experiment belongs in a qualified cryogenic lab using approved equipment and procedures. A flight concept would require separate review of containment, pressure management, venting, vacuum performance and launch-provider acceptance. The cold-pack bench records and model can support a design discussion about which questions to test first.
