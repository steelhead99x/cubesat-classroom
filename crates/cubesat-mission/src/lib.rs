//! Mission logic for the bench CubeSat, independent of any board.
//!
//! This is the Rust version of `firmware/arduino-cubesat`: the same modes, the
//! same ground commands and the same `CSC1`, `REC`, `BCN`, `EVT`, `ACK` and `NAK`
//! lines, so the station's `sensor-bridge` and the bench guide work unchanged.
//! The firmware feeds it the time, the inhibit input, sensor readings and the
//! bytes that arrive over USB; it writes its replies to any [`core::fmt::Write`].
//!
//! Nothing here transmits on radio. USB serial stands in for the link.

#![cfg_attr(not(test), no_std)]

use core::fmt::Write;

/// How long the CubeSat waits after deployment before it transmits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing {
    /// 30 seconds, for a lesson.
    Bench,
    /// The CubeSat Design Specification's 45 minutes of radio silence after
    /// deployment (CDS Rev 14.1 §2.4.5). The 30-minute wait for deployables
    /// (§2.4.4) does not apply: this CubeSat has none.
    Cds,
}

impl Timing {
    /// Milliseconds from deployment to the first transmission.
    pub fn deploy_delay_ms(self) -> u32 {
        match self {
            Self::Bench => 30_000,
            Self::Cds => 45 * 60_000,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Bench => "bench",
            Self::Cds => "cds",
        }
    }
}

/// Settings fixed at build time.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Name in the beacon.
    pub sat_name: &'static str,
    /// Firmware name and version for the boot line.
    pub firmware: &'static str,
    /// Deployment timing.
    pub timing: Timing,
    /// Lux per light count, times 100 000. Zero until a crew calibrates it with `CAL`.
    pub lux_per_count_e5: u64,
    /// False if this firmware has no driver for the motion sensors yet, so the boot
    /// message says so instead of blaming the board.
    pub imu_driver: bool,
}

/// Why the processor last started, from its reset-reason register. A watchdog
/// reset means the firmware hung and the hardware restarted it: on orbit that is
/// the only way back, so the ground needs to know it happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetCause {
    /// Power was applied (or the register was already clear).
    PowerOn,
    /// The reset button or pin.
    ResetPin,
    /// The watchdog: the firmware stopped checking in.
    Watchdog,
    /// The firmware asked for a reset.
    Software,
    /// The processor locked up on a fault.
    Lockup,
    /// Any other reason, such as waking from system off.
    Unknown,
}

impl ResetCause {
    fn name(self) -> &'static str {
        match self {
            Self::PowerOn => "power_on",
            Self::ResetPin => "reset_pin",
            Self::Watchdog => "watchdog",
            Self::Software => "software",
            Self::Lockup => "lockup",
            Self::Unknown => "unknown",
        }
    }
}

/// Which sensors answered at boot.
#[derive(Clone, Copy, Debug, Default)]
pub struct Present {
    /// HS3003 temperature and humidity.
    pub temp_humidity: bool,
    /// LPS22HB pressure.
    pub pressure: bool,
    /// APDS9960 light and colour.
    pub color: bool,
    /// BMI270 and BMM150 motion sensors.
    pub imu: bool,
}

/// One set of readings. `None` means the sensor did not answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Readings {
    /// The regulated 3.3 V rail, measured by the processor itself.
    pub volt_mv: i32,
    /// Milli-degrees Celsius.
    pub temp_mc: Option<i32>,
    /// Thousandths of a percent relative humidity.
    pub humidity_mpct: Option<i32>,
    /// Pascals.
    pub pressure_pa: Option<i32>,
    /// Light counts: clear, red, green, blue.
    pub color: Option<[u16; 4]>,
    /// Acceleration in milli-g.
    pub accel_mg: Option<[i32; 3]>,
    /// Rotation in milli-degrees per second.
    pub gyro_mdps: Option<[i32; 3]>,
    /// Magnetic field in nanotesla.
    pub mag_nt: Option<[i32; 3]>,
}

/// Spacecraft mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// In the deployer: powered on the bench but not allowed to transmit.
    Inhibited,
    /// Deployed, waiting out the deploy timer before transmitting.
    Deploying,
    /// Normal operations.
    Nominal,
    /// Slow readings, some commands refused.
    Safe,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Inhibited => "INHIBITED",
            Self::Deploying => "DEPLOYING",
            Self::Nominal => "NOMINAL",
            Self::Safe => "SAFE",
        }
    }
}

/// Status light colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgb {
    /// Red on.
    pub r: bool,
    /// Green on.
    pub g: bool,
    /// Blue on.
    pub b: bool,
}

const NOMINAL_PERIOD_MS: u32 = 1000;
const SAFE_PERIOD_MS: u32 = 10_000;
const BEACON_MS: u32 = 10_000;
const SAFE_BELOW_MV: i32 = 3000;
const RAIL_PLAUSIBLE_MV: i32 = 1700;
const DEBOUNCE_MS: u32 = 50;
const STORE: usize = 64;
const LINE: usize = 72;
const RECENT: usize = 8;
/// Brightest reference CAL accepts: full sunlight is about 120 000 lux. The cap also
/// keeps the fixed-point arithmetic in range.
const MAX_REFERENCE_MILLI_LUX: u64 = 200_000_000;

#[derive(Clone, Copy, Debug, Default)]
struct Frame {
    seq: u32,
    t_ms: u32,
    temp_mc: Option<i32>,
    volt_mv: i32,
    pressure_pa: Option<i32>,
    light_clear: Option<u16>,
}

/// The whole spacecraft state.
pub struct Mission {
    config: Config,
    present: Present,
    reset: ResetCause,
    mode: Mode,
    mode_since: u32,
    in_contact: bool,
    contact_since: u32,
    orbit_up_ms: u32,
    orbit_down_ms: u32,
    period_ms: u32,
    last_sample: u32,
    last_t: u32,
    last_beacon: u32,
    last_note: u32,
    seq: u32,
    lux_per_count_e5: u64,
    last: Option<Readings>,
    store: [Frame; STORE],
    store_head: usize,
    store_count: usize,
    store_dropped: u32,
    recent: [Option<u32>; RECENT],
    recent_next: usize,
    line: [u8; LINE],
    line_len: usize,
    line_too_long: bool,
    rbf_stable: bool,
    rbf_raw: bool,
    rbf_changed_at: u32,
    flash: Option<(Rgb, u32)>,
}

fn since(now: u32, then: u32) -> u32 {
    now.wrapping_sub(then)
}

impl Mission {
    /// Start up. `inhibit_installed` is the remove-before-flight jumper's state and
    /// `reset` why the processor started. Writes the boot lines.
    pub fn boot(
        config: Config,
        present: Present,
        reset: ResetCause,
        now: u32,
        inhibit_installed: bool,
        out: &mut impl Write,
    ) -> Self {
        let mut m = Self {
            config,
            present,
            reset,
            mode: Mode::Inhibited,
            mode_since: now,
            in_contact: true,
            contact_since: now,
            orbit_up_ms: 0,
            orbit_down_ms: 0,
            period_ms: NOMINAL_PERIOD_MS,
            last_sample: now,
            last_t: now,
            last_beacon: now,
            last_note: now,
            seq: 0,
            lux_per_count_e5: config.lux_per_count_e5,
            last: None,
            store: [Frame::default(); STORE],
            store_head: 0,
            store_count: 0,
            store_dropped: 0,
            recent: [None; RECENT],
            recent_next: 0,
            line: [0; LINE],
            line_len: 0,
            line_too_long: false,
            rbf_stable: inhibit_installed,
            rbf_raw: inhibit_installed,
            rbf_changed_at: now,
            flash: None,
        };
        let _ = write!(
            out,
            "# bench: boot {} {}\r\n",
            config.sat_name, config.firmware
        );
        let _ = write!(
            out,
            "# bench: timing={} deploy_delay_s={}\r\n",
            config.timing.name(),
            config.timing.deploy_delay_ms() / 1000
        );
        let _ = write!(out, "# bench: reset={}\r\n", reset.name());
        if reset == ResetCause::Watchdog || reset == ResetCause::Lockup {
            let _ = out.write_str(
                "# bench: the firmware hung and the hardware restarted it; check the last log\r\n",
            );
        }
        if !present.temp_humidity {
            let _ = out.write_str(
                "# bench: HS3003 temperature sensor not found (is this a Rev2 board?)\r\n",
            );
        }
        if !present.pressure {
            let _ = out.write_str("# bench: LPS22HB pressure sensor not found\r\n");
        }
        if !present.color {
            let _ = out.write_str("# bench: APDS9960 light sensor not found\r\n");
        }
        if !config.imu_driver {
            let _ = out.write_str(
                "# bench: motion sensors (BMI270/BMM150) are not read by this firmware yet\r\n",
            );
        } else if !present.imu {
            let _ = out.write_str(
                "# bench: BMI270/BMM150 motion sensors not found (is this a Rev2 board?)\r\n",
            );
        }
        help(out);
        if inhibit_installed {
            let _ = out.write_str("# bench: mode=INHIBITED reason=rbf_jumper_installed\r\n");
        } else {
            m.mode = Mode::Deploying;
            let _ = out.write_str(
                "# bench: no remove-before-flight jumper at boot; deploy timer started\r\n",
            );
        }
        m
    }

    /// Current mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// True while the CubeSat is in range of the ground station.
    pub fn in_contact(&self) -> bool {
        self.in_contact
    }

    /// Readings stored while out of contact.
    pub fn stored(&self) -> usize {
        self.store_count
    }

    /// Run the clock: debounce the inhibit, run timers, take and send readings.
    /// `sense` takes one set of readings.
    pub fn poll(
        &mut self,
        now: u32,
        inhibit_installed: bool,
        sense: &mut impl FnMut() -> Readings,
        out: &mut impl Write,
    ) {
        self.watch_inhibit(now, inhibit_installed, out);
        match self.mode {
            Mode::Inhibited => {
                if since(now, self.last_note) >= 10_000 {
                    self.last_note = now;
                    let _ = out.write_str("# bench: INHIBITED. Pull the D2 jumper to deploy.\r\n");
                }
                return;
            }
            Mode::Deploying => {
                let delay = self.config.timing.deploy_delay_ms();
                let waited = since(now, self.mode_since);
                if waited >= delay {
                    self.set_mode(Mode::Nominal, "deploy_timer_done", now, out);
                    self.contact_since = now;
                    self.beacon(now, out);
                    self.last_beacon = now;
                    self.last_sample = now;
                } else if since(now, self.last_note) >= 5000 {
                    self.last_note = now;
                    let _ = write!(
                        out,
                        "# bench: deploying, {} s to go\r\n",
                        (delay - waited).div_ceil(1000)
                    );
                }
                return;
            }
            Mode::Nominal | Mode::Safe => {}
        }

        if self.orbit_up_ms > 0 {
            if self.in_contact && since(now, self.contact_since) >= self.orbit_up_ms {
                self.lose_contact(now);
            } else if !self.in_contact && since(now, self.contact_since) >= self.orbit_down_ms {
                self.acquire(now, out);
            }
        }

        if since(now, self.last_sample) >= self.period_ms {
            self.last_sample = now;
            let reading = self.take(now, sense);
            if reading.volt_mv < RAIL_PLAUSIBLE_MV {
                // The nRF52840 stops below about 1.7 V, so this is a bad measurement.
                if since(now, self.last_note) >= 10_000 {
                    self.last_note = now;
                    let _ = out.write_str(
                        "# bench: the 3.3 V rail reading looks wrong; safe-mode check skipped\r\n",
                    );
                }
            } else if self.mode == Mode::Nominal && reading.volt_mv < SAFE_BELOW_MV {
                self.set_mode(Mode::Safe, "rail_low", now, out);
            }
            if self.in_contact {
                self.send_live(&reading, out);
                self.flash = Some((
                    Rgb {
                        r: false,
                        g: true,
                        b: false,
                    },
                    now.wrapping_add(60),
                ));
            } else {
                self.store_frame(&reading);
                self.flash = Some((
                    Rgb {
                        r: true,
                        g: true,
                        b: false,
                    },
                    now.wrapping_add(30),
                ));
            }
        }
        if self.in_contact && since(now, self.last_beacon) >= BEACON_MS {
            self.last_beacon = now;
            self.beacon(now, out);
        }
    }

    /// Feed one byte from the ground link. A line ends at CR or LF.
    pub fn receive(
        &mut self,
        byte: u8,
        now: u32,
        sense: &mut impl FnMut() -> Readings,
        rail_mv: &mut impl FnMut() -> i32,
        out: &mut impl Write,
    ) {
        if byte == b'\r' || byte == b'\n' {
            if self.line_too_long {
                let _ = out.write_str("NAK - LINE too long\r\n");
            } else if self.line_len > 0 {
                let mut text = [0u8; LINE];
                let len = self.line_len;
                text[..len].copy_from_slice(&self.line[..len]);
                self.handle_line(&text[..len], now, sense, rail_mv, out);
            }
            self.line_len = 0;
            self.line_too_long = false;
        } else if self.line_len < LINE - 1 {
            self.line[self.line_len] = byte;
            self.line_len += 1;
        } else {
            self.line_too_long = true;
        }
    }

    /// Status light for this moment.
    pub fn light(&mut self, now: u32) -> Rgb {
        match self.mode {
            Mode::Deploying => {
                return Rgb {
                    r: false,
                    g: false,
                    b: (now / 500).is_multiple_of(2),
                };
            }
            Mode::Safe if self.flash.is_none() => {
                return Rgb {
                    r: (now / 1000).is_multiple_of(4),
                    g: false,
                    b: false,
                };
            }
            _ => {}
        }
        match self.flash {
            Some((colour, until)) if (until.wrapping_sub(now) as i32) > 0 => colour,
            Some(_) => {
                self.flash = None;
                Rgb::default()
            }
            None => Rgb::default(),
        }
    }

    fn watch_inhibit(&mut self, now: u32, installed: bool, out: &mut impl Write) {
        if installed != self.rbf_raw {
            self.rbf_raw = installed;
            self.rbf_changed_at = now;
        }
        if since(now, self.rbf_changed_at) < DEBOUNCE_MS || installed == self.rbf_stable {
            return;
        }
        self.rbf_stable = installed;
        if installed {
            self.set_mode(Mode::Inhibited, "rbf_jumper_installed", now, out);
        } else if self.mode == Mode::Inhibited {
            self.set_mode(Mode::Deploying, "rbf_jumper_removed", now, out);
        }
    }

    fn set_mode(&mut self, next: Mode, why: &str, now: u32, out: &mut impl Write) {
        if next == self.mode {
            return;
        }
        self.mode = next;
        self.mode_since = now;
        self.period_ms = if next == Mode::Safe {
            SAFE_PERIOD_MS
        } else {
            NOMINAL_PERIOD_MS
        };
        if next == Mode::Deploying {
            // A fresh deployment starts in contact with an empty store, like a power-up
            // (CDS Rev 14.1 §2.3.2.4: refitting the inhibit resets the timers).
            self.in_contact = true;
            self.orbit_up_ms = 0;
            self.orbit_down_ms = 0;
            self.store_count = 0;
            self.store_dropped = 0;
        }
        if matches!(next, Mode::Nominal | Mode::Safe) {
            let _ = write!(out, "EVT mode={} reason={why}\r\n", next.name());
        } else {
            let _ = write!(out, "# bench: mode={} reason={why}\r\n", next.name());
        }
        if next != Mode::Safe {
            self.flash = None;
        }
    }

    fn take(&mut self, now: u32, sense: &mut impl FnMut() -> Readings) -> Readings {
        self.seq = self.seq.wrapping_add(1);
        self.last_t = now;
        let r = sense();
        self.last = Some(r);
        r
    }

    fn send_live(&self, r: &Readings, out: &mut impl Write) {
        let _ = write!(
            out,
            "CSC1 seq={} t_ms={} mode={}",
            self.seq,
            self.last_t,
            self.mode.name()
        );
        if let Some(t) = r.temp_mc {
            let _ = write!(out, " temp_mc={t}");
        }
        let _ = write!(out, " volt_mv={}", r.volt_mv);
        if let Some(p) = r.pressure_pa {
            let _ = write!(out, " pressure_pa={p}");
        }
        if let Some(h) = r.humidity_mpct {
            let _ = write!(out, " humidity_mpct={h}");
        }
        if let Some([c, red, green, blue]) = r.color {
            if self.lux_per_count_e5 > 0 {
                let _ = write!(
                    out,
                    " light_lux={}",
                    u64::from(c)
                        .saturating_mul(self.lux_per_count_e5)
                        .saturating_add(50_000)
                        / 100_000
                );
            }
            let _ = write!(
                out,
                " light_clear={c} light_r={red} light_g={green} light_b={blue}"
            );
        }
        if let Some([x, y, z]) = r.accel_mg {
            let _ = write!(out, " ax_mg={x} ay_mg={y} az_mg={z}");
        }
        if let Some([x, y, z]) = r.gyro_mdps {
            let _ = write!(out, " gx_mdps={x} gy_mdps={y} gz_mdps={z}");
        }
        if let Some([x, y, z]) = r.mag_nt {
            let _ = write!(out, " mx_nt={x} my_nt={y} mz_nt={z}");
        }
        let _ = out.write_str("\r\n");
    }

    fn store_frame(&mut self, r: &Readings) {
        self.store[self.store_head] = Frame {
            seq: self.seq,
            t_ms: self.last_t,
            temp_mc: r.temp_mc,
            volt_mv: r.volt_mv,
            pressure_pa: r.pressure_pa,
            light_clear: r.color.map(|c| c[0]),
        };
        self.store_head = (self.store_head + 1) % STORE;
        if self.store_count < STORE {
            self.store_count += 1;
        } else {
            self.store_dropped += 1;
        }
    }

    fn send_stored(&mut self, out: &mut impl Write) {
        let first = (self.store_head + STORE - self.store_count) % STORE;
        for i in 0..self.store_count {
            let f = self.store[(first + i) % STORE];
            let _ = write!(out, "REC seq={} t_ms={}", f.seq, f.t_ms);
            if let Some(t) = f.temp_mc {
                let _ = write!(out, " temp_mc={t}");
            }
            let _ = write!(out, " volt_mv={}", f.volt_mv);
            if let Some(p) = f.pressure_pa {
                let _ = write!(out, " pressure_pa={p}");
            }
            if let Some(c) = f.light_clear {
                let _ = write!(out, " light_clear={c}");
            }
            let _ = out.write_str("\r\n");
        }
        self.store_count = 0;
    }

    fn acquire(&mut self, now: u32, out: &mut impl Write) {
        self.in_contact = true;
        self.contact_since = now;
        let _ = write!(
            out,
            "EVT AOS stored={} dropped={}\r\n",
            self.store_count, self.store_dropped
        );
        self.send_stored(out);
        self.store_dropped = 0;
        self.flash = Some((
            Rgb {
                r: true,
                g: true,
                b: true,
            },
            now.wrapping_add(300),
        ));
    }

    fn lose_contact(&mut self, now: u32) {
        // Nothing is sent: the satellite is out of range of the ground station.
        self.in_contact = false;
        self.contact_since = now;
    }

    fn beacon(&self, now: u32, out: &mut impl Write) {
        let _ = write!(
            out,
            "BCN {} mode={} seq={} uptime_s={}\r\n",
            self.config.sat_name,
            self.mode.name(),
            self.seq,
            now / 1000
        );
    }

    fn seen_before(&mut self, id: Option<u32>) -> bool {
        let Some(id) = id else { return false };
        if self.recent.contains(&Some(id)) {
            return true;
        }
        self.recent[self.recent_next] = Some(id);
        self.recent_next = (self.recent_next + 1) % RECENT;
        false
    }

    #[allow(clippy::too_many_lines)]
    fn handle_line(
        &mut self,
        raw: &[u8],
        now: u32,
        sense: &mut impl FnMut() -> Readings,
        rail_mv: &mut impl FnMut() -> i32,
        out: &mut impl Write,
    ) {
        let mut upper = [0u8; LINE];
        for (u, b) in upper.iter_mut().zip(raw) {
            *u = b.to_ascii_uppercase();
        }
        let Ok(text) = core::str::from_utf8(&upper[..raw.len()]) else {
            let _ = out.write_str("NAK - LINE not text\r\n");
            return;
        };
        let mut words = text.split_ascii_whitespace();
        let Some(mut word) = words.next() else { return };
        let mut id = None;
        if word.as_bytes()[0].is_ascii_digit() {
            id = Some(parse_u32(word));
            let Some(next) = words.next() else { return };
            word = next;
        }
        let arg1 = words.next();
        let arg2 = words.next();

        if matches!(self.mode, Mode::Inhibited | Mode::Deploying) {
            let _ = out.write_str("# bench: radio is off before deployment; command not heard\r\n");
            return;
        }
        if !self.in_contact {
            if word == "AOS" {
                self.orbit_up_ms = 0;
                self.orbit_down_ms = 0;
                self.acquire(now, out);
                ack(out, id, "AOS");
                let _ = out.write_str("\r\n");
            }
            return;
        }
        if self.seen_before(id) {
            nak(out, id, word, "duplicate");
            return;
        }

        match word {
            "PING" => {
                ack(out, id, "PING");
                let _ = write!(out, " seq={}\r\n", self.seq);
            }
            "SENSE" => {
                ack(out, id, "SENSE");
                let _ = out.write_str("\r\n");
                let r = self.take(now, sense);
                self.send_live(&r, out);
            }
            "STATUS" => self.status(id, now, out),
            "RATE" => {
                let s = arg1.map_or(0, parse_u32);
                if !(1..=60).contains(&s) {
                    nak(out, id, "RATE", "use 1 to 60 seconds");
                } else if self.mode == Mode::Safe {
                    nak(out, id, "RATE", "safe mode keeps 10 s");
                } else {
                    self.period_ms = s * 1000;
                    ack(out, id, "RATE");
                    let _ = write!(out, " rate_ms={}\r\n", self.period_ms);
                }
            }
            "SAFE" => {
                ack(out, id, "SAFE");
                let _ = out.write_str("\r\n");
                self.set_mode(Mode::Safe, "command", now, out);
            }
            "NOMINAL" => {
                if self.mode != Mode::Safe {
                    nak(out, id, "NOMINAL", "already nominal");
                } else if rail_mv() < SAFE_BELOW_MV {
                    nak(out, id, "NOMINAL", "rail still low");
                } else {
                    ack(out, id, "NOMINAL");
                    let _ = out.write_str("\r\n");
                    self.set_mode(Mode::Nominal, "command", now, out);
                }
            }
            "LOS" => {
                ack(out, id, "LOS");
                let _ = out.write_str("\r\n");
                self.orbit_up_ms = 0;
                self.orbit_down_ms = 0;
                self.lose_contact(now);
            }
            "AOS" => nak(out, id, "AOS", "already in contact"),
            "ORBIT" => {
                if arg1 == Some("OFF") {
                    self.orbit_up_ms = 0;
                    self.orbit_down_ms = 0;
                    ack(out, id, "ORBIT");
                    let _ = out.write_str(" orbit=off\r\n");
                    return;
                }
                let up = arg1.map_or(0, parse_u32);
                let down = arg2.map_or(0, parse_u32);
                if !(10..=600).contains(&up) || !(10..=600).contains(&down) {
                    nak(out, id, "ORBIT", "use ORBIT <10-600> <10-600> or ORBIT OFF");
                    return;
                }
                self.orbit_up_ms = up * 1000;
                self.orbit_down_ms = down * 1000;
                self.contact_since = now;
                ack(out, id, "ORBIT");
                let _ = write!(out, " contact_s={up} gap_s={down}\r\n");
            }
            "CAL" => self.calibrate(id, arg1, out),
            "HELP" => {
                ack(out, id, "HELP");
                let _ = out.write_str("\r\n");
                help(out);
            }
            other => nak(out, id, other, "unknown command; try HELP"),
        }
    }

    fn status(&self, id: Option<u32>, now: u32, out: &mut impl Write) {
        ack(out, id, "STATUS");
        let _ = write!(
            out,
            " mode={} rate_ms={} stored={} uptime_s={} reset={} orbit={} timing={} sensors=",
            self.mode.name(),
            self.period_ms,
            self.store_count,
            now / 1000,
            self.reset.name(),
            if self.orbit_up_ms > 0 { "on" } else { "off" },
            self.config.timing.name()
        );
        let p = self.present;
        for (on, name) in [
            (p.temp_humidity, "temp+humidity,"),
            (p.pressure, "pressure,"),
            (p.color, "color,"),
            (p.imu, "imu,"),
        ] {
            if on {
                let _ = out.write_str(name);
            }
        }
        let _ = out.write_str("rail lux_per_count=");
        write_e5(out, self.lux_per_count_e5);
        let _ = out.write_str("\r\n");
    }

    fn calibrate(&mut self, id: Option<u32>, arg: Option<&str>, out: &mut impl Write) {
        let Some(milli_lux) = arg.and_then(parse_milli) else {
            nak(
                out,
                id,
                "CAL",
                "need CAL <reference lux> and a non-zero light_clear reading",
            );
            return;
        };
        if milli_lux > MAX_REFERENCE_MILLI_LUX {
            nak(out, id, "CAL", "use a reference of 0 to 200000 lux");
            return;
        }
        if milli_lux == 0 {
            self.lux_per_count_e5 = 0;
            ack(out, id, "CAL");
            let _ = out.write_str(" lux_per_count=0 light_lux=off\r\n");
            return;
        }
        let clear = self.last.and_then(|r| r.color).map_or(0, |c| c[0]);
        if clear == 0 {
            nak(
                out,
                id,
                "CAL",
                "need CAL <reference lux> and a non-zero light_clear reading",
            );
            return;
        }
        // lux / counts, times 100 000, from milli-lux.
        self.lux_per_count_e5 = (milli_lux * 100 + u64::from(clear) / 2) / u64::from(clear);
        ack(out, id, "CAL");
        let _ = out.write_str(" lux_per_count=");
        write_e5(out, self.lux_per_count_e5);
        let _ = out.write_str(
            "\r\n# bench: copy that number into LUX_PER_COUNT_E5 to keep it after a reset\r\n",
        );
    }
}

fn help(out: &mut impl Write) {
    let _ = out.write_str("# commands (optionally start with a number, e.g. \"7 PING\"):\r\n");
    let _ = out.write_str("#   PING  SENSE  STATUS  RATE <1-60 s>  SAFE  NOMINAL\r\n");
    let _ =
        out.write_str("#   LOS  AOS  ORBIT <contact s> <gap s>  ORBIT OFF  CAL <lux>  HELP\r\n");
}

fn ack(out: &mut impl Write, id: Option<u32>, cmd: &str) {
    match id {
        Some(n) => {
            let _ = write!(out, "ACK {n} {cmd}");
        }
        None => {
            let _ = write!(out, "ACK - {cmd}");
        }
    }
}

fn nak(out: &mut impl Write, id: Option<u32>, cmd: &str, why: &str) {
    match id {
        Some(n) => {
            let _ = write!(out, "NAK {n} {cmd} {why}\r\n");
        }
        None => {
            let _ = write!(out, "NAK - {cmd} {why}\r\n");
        }
    }
}

/// Leading digits as a number, like C's `atol`; anything else gives 0.
fn parse_u32(s: &str) -> u32 {
    let mut n: u32 = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            break;
        }
        n = n.saturating_mul(10).saturating_add(u32::from(b - b'0'));
    }
    n
}

/// A non-negative decimal such as `250` or `12.5`, in thousandths.
fn parse_milli(s: &str) -> Option<u64> {
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    if !whole.bytes().all(|b| b.is_ascii_digit()) || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut value: u64 = 0;
    for b in whole.bytes() {
        value = value.checked_mul(10)?.checked_add(u64::from(b - b'0'))?;
    }
    let mut milli = value.checked_mul(1000)?;
    for (i, b) in frac.bytes().take(3).enumerate() {
        milli = milli.checked_add(u64::from(b - b'0') * [100, 10, 1][i])?;
    }
    Some(milli)
}

/// Print a fixed-point value with five decimals.
fn write_e5(out: &mut impl Write, v: u64) {
    let _ = write!(out, "{}.{:05}", v / 100_000, v % 100_000);
}

#[cfg(test)]
mod tests;
