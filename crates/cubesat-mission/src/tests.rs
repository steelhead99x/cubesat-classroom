//! Scripted bench sessions: the same lesson a class runs, checked line by line.

use super::*;

fn config(timing: Timing) -> Config {
    Config {
        sat_name: "CSC-RS",
        firmware: "nano33-cubesat test",
        timing,
        lux_per_count_e5: 0,
        imu_driver: true,
    }
}

fn all_sensors() -> Present {
    Present {
        temp_humidity: true,
        pressure: true,
        color: true,
        imu: false,
    }
}

fn reading(volt_mv: i32) -> Readings {
    Readings {
        volt_mv,
        temp_mc: Some(23_410),
        humidity_mpct: Some(41_000),
        pressure_pa: Some(101_325),
        color: Some([200, 50, 60, 70]),
        ..Readings::default()
    }
}

/// A bench session: a mission, a clock, the inhibit jumper and the serial output.
struct Bench {
    m: Mission,
    now: u32,
    jumper: bool,
    rail: i32,
    out: String,
}

impl Bench {
    fn new(timing: Timing, jumper: bool) -> Self {
        Self::after(ResetCause::PowerOn, timing, jumper)
    }

    fn after(reset: ResetCause, timing: Timing, jumper: bool) -> Self {
        let mut out = String::new();
        let m = Mission::boot(config(timing), all_sensors(), reset, 0, jumper, &mut out);
        Self {
            m,
            now: 0,
            jumper,
            rail: 3300,
            out,
        }
    }

    /// Advance the clock in 10 ms steps, polling as the firmware does.
    fn run(&mut self, ms: u32) {
        let rail = self.rail;
        for _ in 0..ms / 10 {
            self.now += 10;
            self.m
                .poll(self.now, self.jumper, &mut || reading(rail), &mut self.out);
        }
    }

    fn send(&mut self, line: &str) -> String {
        let start = self.out.len();
        let rail = self.rail;
        for b in line.bytes().chain(*b"\n") {
            self.m.receive(
                b,
                self.now,
                &mut || reading(rail),
                &mut || rail,
                &mut self.out,
            );
        }
        self.out[start..].to_string()
    }

    fn take_output(&mut self) -> String {
        std::mem::take(&mut self.out)
    }

    fn deploy(&mut self) {
        self.jumper = false;
        self.run(31_000);
        self.take_output();
    }
}

#[test]
fn boots_inhibited_with_the_jumper_and_says_so() {
    let mut b = Bench::new(Timing::Bench, true);
    let boot = b.take_output();
    assert!(boot.contains("# bench: boot CSC-RS"));
    assert!(boot.contains("timing=bench deploy_delay_s=30"));
    assert!(boot.contains("BMI270/BMM150 motion sensors not found"));
    assert!(boot.contains("mode=INHIBITED reason=rbf_jumper_installed"));
    b.run(11_000);
    assert!(b.take_output().contains("Pull the D2 jumper to deploy"));
    assert_eq!(b.m.mode(), Mode::Inhibited);
    // The radio is off: commands are not heard.
    assert!(b.send("PING").contains("command not heard"));
}

#[test]
fn deploys_after_the_timer_then_beacons_and_sends_telemetry() {
    let mut b = Bench::new(Timing::Bench, true);
    b.take_output();
    b.jumper = false;
    b.run(100);
    assert!(
        b.take_output()
            .contains("# bench: mode=DEPLOYING reason=rbf_jumper_removed")
    );
    b.run(29_000);
    assert_eq!(b.m.mode(), Mode::Deploying);
    assert!(
        !b.take_output().contains("CSC1"),
        "nothing transmitted while deploying"
    );
    b.run(1_500);
    let out = b.take_output();
    assert!(out.contains("EVT mode=NOMINAL reason=deploy_timer_done"));
    assert!(out.contains("BCN CSC-RS mode=NOMINAL"));
    b.run(1_000);
    let live = b.take_output();
    let line = live
        .lines()
        .find(|l| l.starts_with("CSC1"))
        .expect("a CSC1 line");
    for field in [
        "seq=1",
        "mode=NOMINAL",
        "temp_mc=23410",
        "volt_mv=3300",
        "pressure_pa=101325",
        "light_clear=200",
    ] {
        assert!(line.contains(field), "{field} missing from {line}");
    }
    assert!(!line.contains("light_lux"), "no lux before calibration");
}

#[test]
fn cds_timing_waits_45_minutes() {
    let mut b = Bench::new(Timing::Cds, false);
    assert!(b.take_output().contains("timing=cds deploy_delay_s=2700"));
    b.run(44 * 60_000);
    assert_eq!(b.m.mode(), Mode::Deploying);
    b.run(60_000 + 100);
    assert_eq!(b.m.mode(), Mode::Nominal);
}

#[test]
fn commands_are_acknowledged_and_duplicates_refused() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    assert!(b.send("ping").starts_with("ACK - PING seq="));
    assert!(b.send("7 PING").starts_with("ACK 7 PING"));
    assert_eq!(b.send("7 PING"), "NAK 7 PING duplicate\r\n");
    assert!(b.send("SENSE").contains("CSC1"));
    assert!(
        b.send("STATUS")
            .contains("sensors=temp+humidity,pressure,color,rail")
    );
    assert_eq!(b.send("RATE 0"), "NAK - RATE use 1 to 60 seconds\r\n");
    assert_eq!(b.send("RATE 5"), "ACK - RATE rate_ms=5000\r\n");
    assert_eq!(b.send("FLY"), "NAK - FLY unknown command; try HELP\r\n");
}

#[test]
fn lose_the_signal_store_then_forward() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    b.send("LOS");
    b.take_output();
    b.run(3_050);
    let quiet = b.take_output();
    assert!(
        !quiet.contains("CSC1") && !quiet.contains("BCN"),
        "nothing sent out of contact: {quiet}"
    );
    assert_eq!(b.m.stored(), 3);
    assert_eq!(b.send("PING"), "", "commands get no answer out of range");
    let pass = b.send("AOS");
    assert!(pass.starts_with("EVT AOS stored=3 dropped=0"));
    assert_eq!(pass.lines().filter(|l| l.starts_with("REC ")).count(), 3);
    assert!(
        !pass.contains("CSC1"),
        "stored readings never use the live prefix"
    );
    assert_eq!(b.m.stored(), 0);
}

#[test]
fn store_keeps_the_newest_64_and_counts_the_rest() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    b.send("LOS");
    b.run(70_050);
    let pass = b.send("AOS");
    assert!(pass.starts_with("EVT AOS stored=64 dropped=6"), "{pass}");
}

/// A serial buffer like the firmware's: all or nothing, emptied after each poll.
struct Link {
    cap: usize,
    tick: String,
    sent: String,
}

impl Write for Link {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.tick.len() + s.len() > self.cap {
            return Err(core::fmt::Error);
        }
        self.tick.push_str(s);
        Ok(())
    }
}

impl Link {
    fn flush(&mut self) {
        self.sent.push_str(&std::mem::take(&mut self.tick));
    }
}

#[test]
fn a_full_store_drains_through_a_small_buffer_without_losing_a_reading() {
    // Review finding: with the real USB buffer, AOS announced 64 stored readings, only 47
    // whole REC lines got out, and the store was cleared anyway. Now readings go out in
    // batches, each only retired once its whole line was taken.
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    b.send("LOS");
    b.run(70_050);
    assert_eq!(b.m.stored(), 64);
    let mut link = Link {
        cap: 2048,
        tick: String::new(),
        sent: String::new(),
    };
    let rail = b.rail;
    for byte in b"AOS\n" {
        b.m.receive(*byte, b.now, &mut || reading(rail), &mut || rail, &mut link);
    }
    link.flush();
    for _ in 0..40 {
        b.now += 10;
        b.m.poll(b.now, b.jumper, &mut || reading(rail), &mut link);
        link.flush();
    }
    assert!(
        link.sent.contains("EVT AOS stored=64 dropped=6"),
        "{}",
        link.sent
    );
    let seqs: Vec<u32> = link
        .sent
        .lines()
        .filter(|l| l.starts_with("REC "))
        .map(|l| {
            l["REC seq=".len()..]
                .split(' ')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    assert_eq!(seqs.len(), 64, "every stored reading arrives: {seqs:?}");
    assert!(
        seqs.windows(2).all(|w| w[1] == w[0] + 1),
        "in order, none twice: {seqs:?}"
    );
    assert!(
        link.sent
            .lines()
            .all(|l| !l.starts_with("REC ") || l.contains("light_clear=")),
        "every REC line is whole"
    );
    assert_eq!(b.m.stored(), 0);
}

#[test]
fn a_reading_is_retired_only_when_its_whole_line_was_taken() {
    // A link with room for about two REC lines per poll: the rest wait, nothing is lost.
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    b.send("LOS");
    b.run(20_050);
    let stored = b.m.stored();
    assert!(stored > 10);
    b.m.in_contact = true;
    let mut link = Link {
        cap: 200,
        tick: String::new(),
        sent: String::new(),
    };
    for _ in 0..100 {
        b.m.send_stored(&mut link);
        link.flush();
    }
    let recs = link.sent.lines().filter(|l| l.starts_with("REC ")).count();
    assert_eq!(recs, stored);
    assert_eq!(b.m.stored(), 0);
    assert!(link.sent.ends_with("\r\n") && !link.sent.contains("REC seq=REC"));
}

#[test]
fn automatic_orbit_passes() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    assert!(b.send("ORBIT 10 10").contains("contact_s=10 gap_s=10"));
    b.run(10_100);
    assert!(!b.m.in_contact());
    b.run(10_000);
    assert!(b.m.in_contact());
    assert!(b.take_output().contains("EVT AOS stored="));
    assert!(b.send("ORBIT 5 10").contains("NAK"));
}

#[test]
fn safe_mode_by_command_and_on_a_low_rail() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    assert!(b.send("SAFE").contains("EVT mode=SAFE reason=command"));
    assert_eq!(b.send("RATE 5"), "NAK - RATE safe mode keeps 10 s\r\n");
    b.rail = 2900;
    assert_eq!(b.send("NOMINAL"), "NAK - NOMINAL rail still low\r\n");
    b.rail = 3300;
    assert!(
        b.send("NOMINAL")
            .contains("EVT mode=NOMINAL reason=command")
    );
    b.rail = 2900;
    b.run(1_100);
    assert!(b.take_output().contains("EVT mode=SAFE reason=rail_low"));
    // A reading below 1.7 V is a bad measurement, not a dead rail.
    b.send("NOMINAL");
    b.rail = 3300;
    b.send("NOMINAL");
    b.rail = 900;
    b.run(11_000);
    let out = b.take_output();
    assert!(
        out.contains("rail reading looks wrong") && !out.contains("rail_low"),
        "{out}"
    );
}

#[test]
fn light_calibration_in_fixed_point() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    assert!(b.send("CAL abc").contains("NAK"), "needs a number");
    b.send("SENSE");
    assert_eq!(
        b.send("CAL 250"),
        "ACK - CAL lux_per_count=1.25000\r\n# bench: copy that number into LUX_PER_COUNT_E5 to keep it after a reset\r\n"
    );
    assert!(b.send("SENSE").contains("light_lux=250 light_clear=200"));
    assert!(b.send("CAL 12.5").contains("lux_per_count=0.06250"));
    assert!(b.send("CAL 0").contains("light_lux=off"));
    assert!(!b.send("SENSE").contains("light_lux"));
    assert!(b.send("CAL -3").contains("NAK"));
}

#[test]
fn huge_calibration_is_refused_not_overflowed() {
    // A typo like an extra row of zeros used to overflow the fixed-point maths.
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    b.send("SENSE");
    assert!(
        b.send("CAL 200000000000000.999")
            .contains("NAK - CAL use a reference")
    );
    assert!(b.send("CAL 99999999999999999999").contains("NAK"));
    assert!(
        b.send("CAL 200000")
            .contains("ACK - CAL lux_per_count=1000.00000")
    );
    assert!(b.send("SENSE").contains("light_lux=200000 light_clear=200"));
    assert!(b.send("CAL 200000.001").contains("NAK"));
}

#[test]
fn refitting_the_jumper_resets_to_pre_launch() {
    // CDS Rev 14.1 §2.3.2.4: toggling the deployment switch resets the timers.
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    b.send("LOS");
    b.run(2_050);
    assert_eq!(b.m.stored(), 2);
    b.jumper = true;
    b.run(100);
    assert_eq!(b.m.mode(), Mode::Inhibited);
    b.jumper = false;
    b.run(100);
    assert_eq!(b.m.mode(), Mode::Deploying);
    assert_eq!(b.m.stored(), 0);
    assert!(b.m.in_contact());
    b.run(29_000);
    assert_eq!(
        b.m.mode(),
        Mode::Deploying,
        "the deploy timer started again from zero"
    );
}

#[test]
fn jumper_bounce_is_ignored() {
    let mut b = Bench::new(Timing::Bench, true);
    for _ in 0..5 {
        b.jumper = false;
        b.run(20);
        b.jumper = true;
        b.run(20);
    }
    assert_eq!(b.m.mode(), Mode::Inhibited);
}

#[test]
fn long_lines_are_refused_whole() {
    let mut b = Bench::new(Timing::Bench, false);
    b.deploy();
    assert_eq!(b.send(&"X".repeat(100)), "NAK - LINE too long\r\n");
    assert!(b.send("PING").starts_with("ACK"));
}

#[test]
fn status_light_follows_the_mission() {
    let mut b = Bench::new(Timing::Bench, true);
    assert_eq!(b.m.light(0), Rgb::default(), "off while inhibited");
    b.jumper = false;
    b.run(100);
    assert!(
        b.m.light(1000).b && !b.m.light(1500).b,
        "blue blinks while deploying"
    );
    b.run(31_000);
    b.run(990);
    b.run(10);
    assert!(b.m.light(b.now).g, "green blink for a reading sent");
}

#[test]
fn clock_wraparound_does_not_stall_the_mission() {
    let mut out = String::new();
    let start = u32::MAX - 5_000;
    let mut m = Mission::boot(
        config(Timing::Bench),
        all_sensors(),
        ResetCause::PowerOn,
        start,
        false,
        &mut out,
    );
    let mut now = start;
    for _ in 0..3_200 {
        now = now.wrapping_add(10);
        m.poll(now, false, &mut || reading(3300), &mut out);
    }
    assert_eq!(m.mode(), Mode::Nominal);
}

#[test]
fn watchdog_reset_is_reported_at_boot_and_in_status() {
    let mut b = Bench::after(ResetCause::Watchdog, Timing::Bench, false);
    assert!(b.out.contains("# bench: reset=watchdog\r\n"), "{}", b.out);
    assert!(b.out.contains("hardware restarted it"), "{}", b.out);
    b.run(31_000);
    let status = b.send("STATUS");
    assert!(status.contains(" reset=watchdog "), "{status}");
}

#[test]
fn power_on_reset_is_quiet() {
    let mut out = String::new();
    Mission::boot(
        config(Timing::Bench),
        all_sensors(),
        ResetCause::PowerOn,
        0,
        false,
        &mut out,
    );
    assert!(out.contains("# bench: reset=power_on\r\n"));
    assert!(!out.contains("hardware restarted it"));
}
