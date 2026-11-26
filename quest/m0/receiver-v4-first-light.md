# [S] First light with the RTL-SDR Blog V4

## Goal

Get an RTL-SDR Blog V4 running, then record its noise floor on 2 m and 70 cm with a 50 Ω load and with an antenna. Every later receiver quest compares against these two sweeps. Label them "hardware bench".

## Plan

**Time:** 40 minutes. **Group:** up to 4 learners. **Bring:**
- an RTL-SDR Blog V4
- a 50 Ω SMA load (a few dollars)
- the station antenna, or the dipole kit
- a computer with Python 3

No login is needed.

The V4 needs the RTL-SDR Blog drivers. Most current SDR apps include them. For the command-line tools, follow the [bench README](/hardware/sdr-receiver/bench/README.md#install). Receiving is legal without a licence. Nothing in this quest transmits.

1. **Check the dongle.** Plug it in and run `rtl_test -t`. Write down the tuner it reports: a V4 reports the R828D.
2. **Check for DC on the antenna socket.** The bias tee puts DC on the socket, which can damage a load, an antenna that is a short circuit at DC, a filter or a noise source.
   - Run `rtl_biast -b 0`. If it reports an error, stop.
   - With `rtl_test -t` running (the dongle is open, so a forced-on bias tee would be on), a mentor measures the DC voltage between the SMA's centre pin and its outer shell. It should be under 0.5 V. Several volts means the bias tee is on: stop, and see "Bias tee" in the [bench README](/hardware/sdr-receiver/bench/README.md#bias-tee).
   - Write the reading in your log. `sweep.py record` won't run until you add `--dc-checked` to say this check was done with this dongle. It also turns the bias tee off before every sweep and stops if that fails.
3. **Sweep the load.** Screw the 50 Ω load on the SMA. Run:

   ```sh
   cd hardware/sdr-receiver/bench
   python3 sweep.py record --band 2m   --gain 20 --label "50 ohm load" --out load-2m.csv --dc-checked
   python3 sweep.py record --band 70cm --gain 20 --label "50 ohm load" --out load-70cm.csv --dc-checked
   ```

4. **Sweep the antenna.** Swap the load for the antenna and repeat with `--label "antenna, <where it is>"` and `--out ant-2m.csv`, `ant-70cm.csv`.
5. **Compare.** Run `python3 sweep.py compare load-2m.csv ant-2m.csv`, then the same for 70 cm. The change is how much noise your site adds on top of the dongle's own noise.
6. Share the crew nickname, the date, the dongle and the gain, and the four summaries from `sweep.py summary *.csv`. Keep the CSV and `.json` files together.

### Check your work

- All four sweeps used the same gain. A sweep with automatic gain can't be compared.
- The load sweep is flat, with no tall peaks. A peak with the load fitted comes from the dongle or your computer, not the sky. Write down its frequency.
- Every file is labelled "hardware bench". None of these numbers are dBm.

### Levels

- **Beginner:** say which is louder, the load or the antenna, and by how much.
- **Intermediate:** repeat the antenna sweep with the laptop on battery and then on its charger. Does the floor change?
- **Professional:** sweep at gains 10, 20, 30 and 40. Find the gain above which the load's floor stops rising faster than the antenna's: that's where the dongle's own noise stops mattering.

### Talk it over

- Why can't you compare sweeps taken at different gains?
- A satellite signal shows up as a peak above the antenna's floor. What happens to a weak satellite if your site's floor is 10 dB higher than the load's?

## Required

- [Predict a satellite pass](/quest/m0/predict-a-pass.md) - so the crew knows when there will be something to hear.
