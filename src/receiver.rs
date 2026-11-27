//! The classroom receiver's `stats` track: a desk simulation, and a bridge for a real
//! RTL-SDR.
//!
//! A simulated satellite passes over the station on a fixed cycle. Each second the
//! station works out the slant range, elevation and Doppler shift, turns the range into
//! a received level in dBFS for the chosen receiver (the CSC-R1P pedal or an RTL-SDR
//! Blog V4), and publishes one versioned JSON snapshot labelled `desk simulation`.
//!
//! The bridge reads `rtl_power` rows from a real receiver on standard input and
//! publishes the same snapshot, labelled `received RF` (or `hardware bench` for a
//! signal generator or load on the bench), with the sky fields left empty.
//! Nothing here transmits.
//!
//! The track contract is described in docs/classroom-receiver.md.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Result, bail, ensure};
use jiff::tz::TimeZone;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{Link, Timestamp, connect, first_object, unix_ms, with_broadcast};

/// Track name for receiver snapshots.
pub(crate) const STATS_TRACK: &str = "stats";
/// Version of the snapshot layout. Readers refuse other versions.
pub(crate) const STATS_VERSION: u32 = 1;
/// Broadcast the cubesat.app phone app subscribes to (its `BROADCAST`).
pub(crate) const BROADCAST: &str = "live";
/// Below this SNR the carrier is not counted as heard.
const HEARD_SNR_DB: f64 = 3.0;

const EARTH_RADIUS_KM: f64 = 6371.0;
const MU_KM3_S2: f64 = 398_600.441_8;
const LIGHT_KM_S: f64 = 299_792.458;

/// Which receiver the simulation stands in for.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Source {
    /// The CSC-R1P guitar-pedal receiver (hardware/sdr-receiver/pedal).
    Pedal,
    /// An RTL-SDR Blog V4 dongle on the station computer.
    RtlSdrV4,
}

impl Source {
    /// Highest gain setting in dB. The pedal's GAIN knob sets the ADC's 0–42 dB
    /// gain; the V4 tops out at its tuner's 49.6 dB step.
    fn max_gain_db(self) -> f64 {
        match self {
            Self::Pedal => 42.0,
            Self::RtlSdrV4 => 49.6,
        }
    }

    fn default_passband_hz(self) -> f64 {
        match self {
            Self::Pedal => 1500.0,
            Self::RtlSdrV4 => 3000.0,
        }
    }

    /// Noise floor in dBFS with the gain at 0 dB, in the analysis bandwidth.
    /// Planning numbers for the classroom, not measurements.
    fn noise_at_zero_gain_dbfs(self) -> f64 {
        match self {
            Self::Pedal => -96.0,
            Self::RtlSdrV4 => -92.0,
        }
    }
}

/// Amateur satellite bands the receiver covers.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Band {
    /// 2 m, 145.8–146.0 MHz satellite segment.
    #[value(name = "2m")]
    #[serde(rename = "2m")]
    TwoMetre,
    /// 70 cm, 435–438 MHz satellite segment.
    #[value(name = "70cm")]
    #[serde(rename = "70cm")]
    SeventyCm,
}

impl Band {
    fn default_hz(self) -> u64 {
        match self {
            Self::TwoMetre => 145_800_000,
            Self::SeventyCm => 437_800_000,
        }
    }

    fn range_hz(self) -> (u64, u64) {
        match self {
            Self::TwoMetre => (144_000_000, 148_000_000),
            Self::SeventyCm => (420_000_000, 450_000_000),
        }
    }
}

/// Settings for one simulated pass cycle.
#[derive(Clone, Debug)]
pub(crate) struct Scenario {
    pub source: Source,
    pub band: Band,
    pub nominal_hz: u64,
    pub altitude_km: f64,
    pub max_elevation_deg: f64,
    /// Seconds from the start of one pass to the start of the next.
    pub cycle_s: u64,
    pub gain_db: f64,
    /// Received level in dBFS at 1000 km slant range with the gain at 0 dB.
    pub level_at_1000km_dbfs: f64,
    /// Retune for Doppler, as a tracking station would.
    pub track_doppler: bool,
    /// The receiver hears a carrier within this many Hz of where it is tuned. Off-tune carriers
    /// outside it are not heard or decoded.
    pub passband_hz: f64,
}

/// Where a snapshot came from. Shown beside every value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Provenance {
    DeskSimulation,
    HardwareBench,
    ReceivedRf,
}

impl Provenance {
    fn words(self) -> &'static str {
        match self {
            Self::DeskSimulation => "desk simulation",
            Self::HardwareBench => "hardware bench",
            Self::ReceivedRf => "received RF",
        }
    }
}

/// Whether the station hears the satellite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Contact {
    /// The carrier is heard.
    Acquired,
    /// The satellite is up but the carrier is not heard (off tune, too weak).
    Lost,
    /// Nothing to hear.
    Idle,
}

/// One sensor value in a snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Reading {
    pub value: f64,
    pub unit: String,
}

/// One `stats` snapshot, version 1.
///
/// The fields up to `sensors` are the cubesat.app contract (its `StationSnapshot`); the
/// receiver details are optional extensions to that contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Stats {
    pub v: u32,
    pub seq: u64,
    /// A new id each time the publisher starts, so viewers order readings when `seq` restarts.
    pub session: String,
    pub station: String,
    /// What is being heard, such as "Simulated satellite".
    pub source: String,
    pub provenance: Provenance,
    pub utc: String,
    /// When the bridge read the row, separately from the receive capture time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_utc: Option<String>,
    pub contact: Contact,
    /// Where the receiver is tuned.
    pub frequency_hz: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_dbfs: Option<f64>,
    /// Uncalibrated FFT power reported by rtl_power.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_db_rel: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_deg: Option<f64>,
    /// Decoded beacon values, present only when the signal is strong enough.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensors: Option<BTreeMap<String, Reading>>,

    pub receiver: Source,
    pub band: Band,
    /// The published downlink frequency.
    pub nominal_hz: u64,
    pub gain_db: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_dbfs: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_db_rel: Option<f64>,
    /// `None` with no signal.
    pub snr_db: Option<f64>,
    /// Whether the receiver's input is near full scale. Only a source that sees the ADC samples
    /// can tell (the simulator); `rtl_power` prints averaged FFT bin power, not sample levels,
    /// so the bridge leaves this out rather than guess.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clipping: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doppler_hz: Option<f64>,
    /// Received carrier minus tuned frequency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_hz: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_km: Option<f64>,
    /// Seconds since the satellite rose, or until it rises when negative.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_s: Option<f64>,
}

/// Who is publishing: the station slug and this run's session id.
#[derive(Clone, Debug)]
pub(crate) struct Publisher {
    pub station: String,
    pub session: String,
}

impl Publisher {
    pub(crate) fn new(station: &str) -> Result<Self> {
        ensure!(
            valid_station(station),
            "--station must be 1–40 lower-case letters, digits and hyphens, like `room-12`; use a nickname, not a name or address"
        );
        Ok(Self {
            station: station.into(),
            session: session_id(unix_ms()?, std::process::id()),
        })
    }
}

/// The app's station rule: lower-case letters, digits and inner hyphens, 1–40 characters.
fn valid_station(s: &str) -> bool {
    let b = s.as_bytes();
    let ok = |c: &u8| c.is_ascii_lowercase() || c.is_ascii_digit();
    !b.is_empty()
        && b.len() <= 40
        && ok(&b[0])
        && ok(&b[b.len() - 1])
        && b.iter().all(|c| ok(c) || *c == b'-')
}

/// A 12-character base-36 session id from the start time and process id.
fn session_id(unix_ms: u64, pid: u32) -> String {
    let mut x = unix_ms ^ (u64::from(pid) << 40) ^ 0x9e37_79b9_7f4a_7c15;
    // Mix the bits so ids from nearby start times look unrelated.
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    (0..12)
        .map(|_| {
            let d = digits[(x % 36) as usize] as char;
            x /= 36;
            d
        })
        .collect()
}

/// Where the satellite is relative to the station at one moment.
#[derive(Clone, Copy, Debug)]
struct Geometry {
    range_km: f64,
    elevation_deg: f64,
}

impl Scenario {
    fn orbit_radius_km(&self) -> f64 {
        EARTH_RADIUS_KM + self.altitude_km
    }

    /// Angular rate of the sub-satellite point, radians per second.
    fn angular_rate(&self) -> f64 {
        let r = self.orbit_radius_km();
        (MU_KM3_S2 / r).sqrt() / r
    }

    /// Earth-central angle at which the satellite sits on the horizon.
    fn horizon_angle(&self) -> f64 {
        (EARTH_RADIUS_KM / self.orbit_radius_km()).acos()
    }

    /// Cross-track central angle that gives the requested highest elevation.
    fn cross_track_angle(&self) -> f64 {
        let el = self.max_elevation_deg.to_radians();
        let ratio = EARTH_RADIUS_KM / self.orbit_radius_km();
        // Central angle for elevation el: psi = acos(ratio cos el) - el.
        (ratio * el.cos()).acos() - el
    }

    /// Seconds from rise to set.
    pub fn pass_duration_s(&self) -> f64 {
        let lam = self.horizon_angle();
        let c = self.cross_track_angle();
        let half = (lam.cos() / c.cos()).clamp(-1.0, 1.0).acos();
        2.0 * half / self.angular_rate()
    }

    /// Geometry `t` seconds after the highest point of the pass.
    fn geometry(&self, t: f64) -> Geometry {
        let r = self.orbit_radius_km();
        let along = self.angular_rate() * t;
        let psi = (along.cos() * self.cross_track_angle().cos())
            .clamp(-1.0, 1.0)
            .acos();
        let range_km =
            (EARTH_RADIUS_KM.powi(2) + r * r - 2.0 * EARTH_RADIUS_KM * r * psi.cos()).sqrt();
        let elevation = (r * psi.cos() - EARTH_RADIUS_KM).atan2(r * psi.sin());
        Geometry {
            range_km,
            elevation_deg: elevation.to_degrees(),
        }
    }

    /// Doppler shift in Hz at `t` seconds after the highest point.
    fn doppler_hz(&self, t: f64) -> f64 {
        let dt = 0.5;
        let rate = (self.geometry(t + dt).range_km - self.geometry(t - dt).range_km) / (2.0 * dt);
        -(self.nominal_hz as f64) * rate / LIGHT_KM_S
    }

    /// Snapshot at whole second `sequence` of the run.
    pub fn snapshot(&self, publisher: &Publisher, sequence: u64, captured_unix_ms: u64) -> Stats {
        let duration = self.pass_duration_s();
        // Each cycle starts 60 s before rise so a class sees the satellite come up.
        let in_cycle = (sequence % self.cycle_s) as f64;
        let pass_s = in_cycle - 60.0;
        let t = pass_s - duration / 2.0;
        let geo = self.geometry(t);
        let in_view = geo.elevation_deg >= 0.0 && pass_s >= 0.0 && pass_s <= duration;

        let doppler_hz = if in_view { self.doppler_hz(t) } else { 0.0 };
        let tuned_hz = if self.track_doppler && in_view {
            (self.nominal_hz as f64 + doppler_hz).round() as u64
        } else {
            self.nominal_hz
        };
        let received_hz = self.nominal_hz as f64 + doppler_hz;
        // The carrier is only heard (and decoded) inside the receiver's passband.
        let in_band = (received_hz - tuned_hz as f64).abs() <= self.passband_hz;
        let noise_dbfs = self.source.noise_at_zero_gain_dbfs() + self.gain_db;
        let signal_dbfs = if in_view {
            self.level_at_1000km_dbfs + self.gain_db + 20.0 * (1000.0 / geo.range_km).log10()
        } else {
            f64::NEG_INFINITY
        };
        let power = |db: f64| 10f64.powf(db / 10.0);
        // The ADC sees the carrier wherever it is in the dongle's bandwidth, so an off-tune
        // carrier can still clip the input.
        let clipping = 10.0 * (power(noise_dbfs) + power(signal_dbfs)).log10() > -1.0;
        // The level and SNR are measured in the passband: signal and noise add as power there.
        let heard_dbfs = if in_band {
            signal_dbfs
        } else {
            f64::NEG_INFINITY
        };
        let dbfs = (10.0 * (power(noise_dbfs) + power(heard_dbfs)).log10()).min(0.0);
        let snr_db = if in_view && in_band {
            signal_dbfs - noise_dbfs
        } else {
            f64::NEG_INFINITY
        };
        let sensors = (snr_db >= 10.0 && !clipping).then(|| beacon(sequence));
        let contact = if !in_view {
            Contact::Idle
        } else if snr_db >= HEARD_SNR_DB {
            Contact::Acquired
        } else {
            Contact::Lost
        };

        Stats {
            v: STATS_VERSION,
            seq: sequence,
            session: publisher.session.clone(),
            station: publisher.station.clone(),
            source: "Simulated satellite".into(),
            provenance: Provenance::DeskSimulation,
            utc: utc_string(captured_unix_ms),
            received_utc: None,
            contact,
            frequency_hz: tuned_hz,
            level_dbfs: Some(round1(dbfs)),
            level_db_rel: None,
            elevation_deg: Some(round1(geo.elevation_deg)),
            sensors,
            receiver: self.source,
            band: self.band,
            nominal_hz: self.nominal_hz,
            gain_db: round1(self.gain_db),
            noise_dbfs: Some(round1(noise_dbfs)),
            noise_db_rel: None,
            snr_db: snr_db.is_finite().then(|| round1(snr_db)),
            clipping: Some(clipping),
            doppler_hz: Some(round1(doppler_hz)),
            offset_hz: Some(round1(received_hz - tuned_hz as f64)),
            range_km: Some(round1(geo.range_km)),
            pass_s: Some(round1(pass_s)),
        }
    }
}

/// Simulated beacon contents, with the bench CubeSat's sensor names.
fn beacon(sequence: u64) -> BTreeMap<String, Reading> {
    let r = |value: f64, unit: &str| Reading {
        value,
        unit: unit.into(),
    };
    BTreeMap::from([
        (
            "temperature".into(),
            r(round1(18.0 + (sequence % 11) as f64 / 2.0), "°C"),
        ),
        (
            "voltage".into(),
            r((3900 + sequence % 61) as f64 / 1000.0, "V"),
        ),
    ])
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Format Unix milliseconds as an RFC 3339 UTC time, such as 2026-11-27T18:30:05.120Z.
pub(crate) fn utc_string(unix_ms: u64) -> String {
    let secs = unix_ms / 1000;
    let ms = unix_ms % 1000;
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{ms:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// `receiver-sim` arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Desk simulation of the classroom receiver's `stats` track (pedal or RTL-SDR V4). Does not use a radio."
)]
pub(crate) struct SimArgs {
    /// Relay URL. Leave out to print JSON lines instead of publishing.
    #[arg(long, env = "CUBESAT_RELAY")]
    relay: Option<Url>,
    /// Certificate fingerprint printed beside the relay URL.
    #[arg(long, env = "CUBESAT_TLS_FINGERPRINT")]
    fingerprint: Option<String>,
    /// Broadcast name. The phone app subscribes to `live`.
    #[arg(long, default_value = BROADCAST)]
    broadcast: String,
    /// Station slug, a nickname such as `room-12`. Not a name, school or address.
    #[arg(long, default_value = "home")]
    station: String,
    /// Receiver to stand in for.
    #[arg(long, value_enum, default_value_t = Source::Pedal)]
    source: Source,
    /// Band to tune.
    #[arg(long, value_enum, default_value_t = Band::TwoMetre)]
    band: Band,
    /// Downlink frequency in Hz. Defaults to 145800000 on 2 m and 437800000 on 70 cm.
    #[arg(long)]
    frequency_hz: Option<u64>,
    /// Orbit height in km.
    #[arg(long, default_value_t = 420.0)]
    altitude_km: f64,
    /// Highest elevation of each pass in degrees.
    #[arg(long, default_value_t = 45.0)]
    max_elevation_deg: f64,
    /// Seconds between the starts of passes. Real passes come hours apart; the
    /// default repeats every 15 minutes so a lesson sees several.
    #[arg(long, default_value_t = 900, value_parser = clap::value_parser!(u64).range(60..=86_400))]
    cycle_s: u64,
    /// Receiver gain in dB.
    #[arg(long, default_value_t = 20.0)]
    gain_db: f64,
    /// Received level at 1000 km with the gain at 0 dB, in dBFS.
    #[arg(long, default_value_t = -80.0, allow_hyphen_values = true)]
    level_at_1000km_dbfs: f64,
    /// Retune for Doppler during the pass.
    #[arg(long)]
    track_doppler: bool,
    /// Receive half-bandwidth in Hz. Defaults to 1500 for the pedal and 3000 for the V4.
    #[arg(long)]
    passband_hz: Option<f64>,
    /// Seconds of output when printing (0 = one full cycle).
    #[arg(long, default_value_t = 0)]
    seconds: u64,
    /// Second to start at, to jump into a pass.
    #[arg(long, default_value_t = 0)]
    start_s: u64,
}

impl SimArgs {
    pub(crate) fn scenario(&self) -> Result<Scenario> {
        let nominal_hz = self.frequency_hz.unwrap_or(self.band.default_hz());
        let (low, high) = self.band.range_hz();
        ensure!(
            (low..=high).contains(&nominal_hz),
            "{nominal_hz} Hz is outside the {:?} band ({low}–{high} Hz)",
            self.band
        );
        ensure!(
            (160.0..=2000.0).contains(&self.altitude_km),
            "altitude must be 160–2000 km"
        );
        ensure!(
            (1.0..=90.0).contains(&self.max_elevation_deg),
            "highest elevation must be 1–90 degrees"
        );
        let max_gain = self.source.max_gain_db();
        ensure!(
            (0.0..=max_gain).contains(&self.gain_db),
            "gain must be 0–{max_gain} dB for {:?}",
            self.source
        );
        let scenario = Scenario {
            source: self.source,
            band: self.band,
            nominal_hz,
            altitude_km: self.altitude_km,
            max_elevation_deg: self.max_elevation_deg,
            cycle_s: self.cycle_s,
            gain_db: self.gain_db,
            level_at_1000km_dbfs: self.level_at_1000km_dbfs,
            track_doppler: self.track_doppler,
            passband_hz: self
                .passband_hz
                .unwrap_or(self.source.default_passband_hz()),
        };
        ensure!(
            scenario.passband_hz > 0.0 && scenario.passband_hz.is_finite(),
            "--passband-hz must be above 0"
        );
        ensure!(
            scenario.pass_duration_s() + 60.0 < self.cycle_s as f64,
            "a {:.0} s pass does not fit a {} s cycle; raise --cycle-s",
            scenario.pass_duration_s(),
            self.cycle_s
        );
        Ok(scenario)
    }
}

pub(crate) async fn run(args: SimArgs) -> Result<()> {
    let scenario = args.scenario()?;
    let publisher = Publisher::new(&args.station)?;
    let Some(relay) = args.relay.clone() else {
        let count = if args.seconds == 0 {
            scenario.cycle_s
        } else {
            args.seconds
        };
        let start = unix_ms()?;
        for i in 0..count {
            let sequence = args.start_s + i;
            let stats = scenario.snapshot(&publisher, sequence, start + i * 1000);
            println!("{}", serde_json::to_string(&stats)?);
        }
        return Ok(());
    };
    eprintln!(
        "Desk simulation: publishing {} snapshots to {}/{} once a second. No radio is used.",
        STATS_TRACK, relay, args.broadcast
    );
    let link = Link {
        relay,
        fingerprint: args.fingerprint.clone(),
        broadcast: args.broadcast.clone(),
    };
    let origin = moq_tokio::origin::spawn();
    let broadcast = origin.create_broadcast(&link.broadcast)?;
    let track = broadcast.create_track(STATS_TRACK, None)?;
    broadcast.announce(Default::default())?;
    let client = connect(&link)?.with_publisher(&origin);
    let session = client.connect(link.relay);
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut sequence = args.start_s;
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let stats = scenario.snapshot(&publisher, sequence, unix_ms()?);
                sequence += 1;
                let mut group = track.append_group()?;
                group.write_frame(Timestamp::now(), serde_json::to_vec(&stats)?)?;
                group.finish()?;
            }
            result = session.closed() => {
                result?;
                bail!("relay session closed");
            }
        }
    }
}

/// Label for snapshots from a real receiver.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub(crate) enum RealLabel {
    /// Signals from the sky through an antenna.
    ReceivedRf,
    /// A signal generator, test source or load on the bench.
    HardwareBench,
}

impl RealLabel {
    fn provenance(self) -> Provenance {
        match self {
            Self::ReceivedRf => Provenance::ReceivedRf,
            Self::HardwareBench => Provenance::HardwareBench,
        }
    }
}

/// `receiver-bridge` arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Publish `stats` from a real receiver: pipe `rtl_power` rows in on standard input. Does not transmit.",
    after_help = "Example, with an RTL-SDR Blog V4 tuned to the ISS downlink at 30 dB gain:\n  rtl_power -f 145.79M:145.81M:500 -g 30 -i 1 - | cubesat-classroom receiver-bridge --gain-db 30 --relay moqt://..."
)]
pub(crate) struct BridgeArgs {
    /// Relay URL. Leave out to print JSON lines instead of publishing.
    #[arg(long, env = "CUBESAT_RELAY")]
    relay: Option<Url>,
    /// Certificate fingerprint printed beside the relay URL.
    #[arg(long, env = "CUBESAT_TLS_FINGERPRINT")]
    fingerprint: Option<String>,
    /// Broadcast name. The phone app subscribes to `live`.
    #[arg(long, default_value = BROADCAST)]
    broadcast: String,
    /// Station slug, a nickname such as `room-12`. Not a name, school or address.
    #[arg(long, default_value = "home")]
    station: String,
    /// Receiver feeding rtl_power.
    #[arg(long, value_enum, default_value_t = Source::RtlSdrV4)]
    source: Source,
    /// Band the receiver is on.
    #[arg(long, value_enum, default_value_t = Band::TwoMetre)]
    band: Band,
    /// Frequency the signal is expected on, in Hz. Defaults to the band's preset.
    #[arg(long)]
    frequency_hz: Option<u64>,
    /// What is being received, shown in the app, such as "ISS (ZARYA)".
    #[arg(long, default_value = "Unidentified signal")]
    satellite: String,
    /// Gain passed to rtl_power with -g, in dB. Recorded in each snapshot.
    #[arg(long)]
    gain_db: f64,
    /// Bins within this many Hz of the frequency count as signal; the rest set the noise floor.
    #[arg(long, default_value_t = 3000.0)]
    passband_hz: f64,
    /// `received-rf` for an antenna, `hardware-bench` for a test source or load.
    #[arg(long, value_enum, default_value_t = RealLabel::ReceivedRf)]
    label: RealLabel,
    /// IANA time zone of the rtl_power log. Defaults to this computer's time zone.
    #[arg(long, value_parser = parse_rtl_power_tz)]
    rtl_power_tz: Option<TimeZone>,
}

fn parse_rtl_power_tz(name: &str) -> Result<TimeZone, jiff::Error> {
    TimeZone::get(name)
}

/// Turn one rtl_power row into a snapshot, or `None` if the row holds no usable bins
/// none near the frequency, or a capture time that is invalid or cannot be resolved.
fn from_rtl_power(
    row: &str,
    args: &BridgeArgs,
    publisher: &Publisher,
    sequence: u64,
    time_zone: &TimeZone,
    received_unix_ms: Option<u64>,
) -> Option<Stats> {
    let captured_unix_ms = sweep_unix_ms(row, time_zone, received_unix_ms)?;
    let tuned_hz = args.frequency_hz.unwrap_or(args.band.default_hz());
    let (bins, _) = bench_tools::sweep::parse(row);
    let near = |f: f64| (f - tuned_hz as f64).abs() <= args.passband_hz;
    let signal: Vec<f64> = bins.iter().filter(|b| near(b.0)).map(|b| b.1).collect();
    let rest: Vec<f64> = bins.iter().filter(|b| !near(b.0)).map(|b| b.1).collect();
    let peak = signal.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !peak.is_finite() {
        return None;
    }
    let noise = (!rest.is_empty()).then(|| bench_tools::util::median(&rest));
    let snr_db = noise.map(|floor| peak - floor);
    Some(Stats {
        v: STATS_VERSION,
        seq: sequence,
        session: publisher.session.clone(),
        station: publisher.station.clone(),
        source: args.satellite.clone(),
        provenance: args.label.provenance(),
        utc: utc_string(captured_unix_ms),
        received_utc: received_unix_ms.map(utc_string),
        contact: if snr_db.is_some_and(|snr| snr >= HEARD_SNR_DB) {
            Contact::Acquired
        } else {
            Contact::Idle
        },
        frequency_hz: tuned_hz,
        // Relative dB from rtl_power's FFT, which can be above 0; it is not full scale, so it
        // is not capped there and says nothing about clipping.
        level_dbfs: None,
        level_db_rel: Some(round1(peak)),
        elevation_deg: None,
        sensors: None,
        receiver: args.source,
        band: args.band,
        nominal_hz: tuned_hz,
        gain_db: round1(args.gain_db),
        noise_dbfs: None,
        noise_db_rel: noise.map(round1),
        snr_db: snr_db.map(round1),
        clipping: None,
        doppler_hz: None,
        offset_hz: None,
        range_km: None,
        pass_s: None,
    })
}

/// Convert the local time at the start of an rtl_power row to Unix milliseconds.
///
/// When the clocks go back, an hour of local times happens twice. The bridge knows when it
/// read the row (`received_unix_ms`), and a sweep is captured before it is read, so it takes
/// the later of the two that is not after the read time. Without a read time, or for a time
/// that never happened (clocks going forward), the row is skipped.
fn sweep_unix_ms(row: &str, time_zone: &TimeZone, received_unix_ms: Option<u64>) -> Option<u64> {
    use jiff::tz::AmbiguousOffset;
    let mut fields = row.split(',').map(str::trim);
    let (date, time) = (fields.next()?, fields.next()?);
    let civil: jiff::civil::DateTime = format!("{date}T{time}").parse().ok()?;
    let ms = |offset: jiff::tz::Offset| {
        let stamp = offset.to_timestamp(civil).ok()?;
        u64::try_from(stamp.as_millisecond()).ok()
    };
    match time_zone.to_ambiguous_timestamp(civil).offset() {
        AmbiguousOffset::Unambiguous { offset } => ms(offset),
        AmbiguousOffset::Gap { .. } => None,
        AmbiguousOffset::Fold { before, after } => {
            let read = received_unix_ms?;
            // `after` is the second time round, so it is the later instant.
            [ms(after)?, ms(before)?].into_iter().find(|&t| t <= read)
        }
    }
}

pub(crate) async fn bridge(args: BridgeArgs) -> Result<()> {
    let time_zone = args.rtl_power_tz.clone().unwrap_or_else(TimeZone::system);
    let tuned_hz = args.frequency_hz.unwrap_or(args.band.default_hz());
    let (low, high) = args.band.range_hz();
    ensure!(
        (low..=high).contains(&tuned_hz),
        "{tuned_hz} Hz is outside the {:?} band ({low}–{high} Hz)",
        args.band
    );
    ensure!(
        args.passband_hz.is_finite() && args.passband_hz > 0.0,
        "--passband-hz must be finite and above 0"
    );
    ensure!(
        (0.0..=args.source.max_gain_db()).contains(&args.gain_db),
        "--gain-db is outside this receiver’s gain range"
    );
    ensure!(
        !args.satellite.is_empty() && args.satellite.chars().count() <= 80,
        "--satellite must be 1–80 characters"
    );
    let publisher = Publisher::new(&args.station)?;
    // Read standard input on its own thread so a slow relay never blocks rtl_power.
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    std::thread::spawn(move || {
        use std::io::BufRead;
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if tx.blocking_send((line, unix_ms().ok())).is_err() {
                break;
            }
        }
    });
    let mut sequence = 0;
    let Some(relay) = args.relay.clone() else {
        while let Some((line, received)) = rx.recv().await {
            if let Some(stats) =
                from_rtl_power(&line, &args, &publisher, sequence, &time_zone, received)
            {
                println!("{}", serde_json::to_string(&stats)?);
                sequence += 1;
            }
        }
        return Ok(());
    };
    eprintln!(
        "Publishing {} snapshots ({}) to {}/{}. Nothing transmits.",
        STATS_TRACK,
        args.label.provenance().words(),
        relay,
        args.broadcast
    );
    let link = Link {
        relay,
        fingerprint: args.fingerprint.clone(),
        broadcast: args.broadcast.clone(),
    };
    let origin = moq_tokio::origin::spawn();
    let broadcast = origin.create_broadcast(&link.broadcast)?;
    let track = broadcast.create_track(STATS_TRACK, None)?;
    broadcast.announce(Default::default())?;
    let client = connect(&link)?.with_publisher(&origin);
    let session = client.connect(link.relay);
    loop {
        tokio::select! {
            line = rx.recv() => {
                let Some((line, received)) = line else { bail!("rtl_power input ended") };
                let Some(stats) = from_rtl_power(&line, &args, &publisher, sequence, &time_zone, received) else { continue };
                sequence += 1;
                let mut group = track.append_group()?;
                group.write_frame(Timestamp::now(), serde_json::to_vec(&stats)?)?;
                group.finish()?;
            }
            result = session.closed() => {
                result?;
                bail!("relay session closed");
            }
        }
    }
}

/// `read-receiver` arguments.
#[derive(clap::Args, Debug)]
#[command(about = "Read one fresh `stats` snapshot from a relay and print it with its label.")]
pub(crate) struct ReadArgs {
    #[arg(long, env = "CUBESAT_RELAY")]
    relay: Url,
    #[arg(long, env = "CUBESAT_TLS_FINGERPRINT")]
    fingerprint: Option<String>,
    #[arg(long, default_value = BROADCAST)]
    broadcast: String,
}

pub(crate) async fn read(args: ReadArgs) -> Result<()> {
    let link = Link {
        relay: args.relay,
        fingerprint: args.fingerprint,
        broadcast: args.broadcast,
    };
    let object = with_broadcast(&link, async |broadcast| {
        first_object(&broadcast, STATS_TRACK).await
    })
    .await?;
    let stats = parse(&object.payload)?;
    println!("{}", describe(&stats));
    Ok(())
}

/// One line for a person to read. The label always comes first.
pub(crate) fn describe(s: &Stats) -> String {
    let receiver = match s.receiver {
        Source::Pedal => "CSC-R1P pedal",
        Source::RtlSdrV4 => "RTL-SDR V4",
    };
    let sky = match (s.contact, s.elevation_deg, s.range_km, s.doppler_hz) {
        (Contact::Idle, Some(_), ..) => "satellite below the horizon".to_string(),
        (_, Some(el), Some(range), Some(doppler)) => {
            format!("elevation {el:.1}°, range {range:.0} km, Doppler {doppler:+.0} Hz")
        }
        _ => "no pass prediction".to_string(),
    };
    let level = s
        .level_db_rel
        .map(|value| format!("{value:.1} dB relative"))
        .or_else(|| s.level_dbfs.map(|value| format!("{value:.1} dBFS")))
        .unwrap_or_else(|| "level unavailable".into());
    let signal = match (s.contact, s.snr_db) {
        (Contact::Acquired, Some(snr)) => format!("{level}, SNR {snr:.1} dB"),
        (Contact::Lost, _) => format!("carrier not heard, {level}"),
        _ => level,
    };
    let mut line = format!(
        "[{}] {} {} on {}, {} tuned {:.4} MHz, gain {:.0} dB: {sky}; {signal}",
        s.provenance.words(),
        s.utc,
        s.station,
        receiver,
        s.source,
        s.frequency_hz as f64 / 1e6,
        s.gain_db
    );
    if s.clipping == Some(true) {
        line.push_str("; CLIPPING, turn the gain down");
    }
    for (name, r) in s.sensors.iter().flatten() {
        line.push_str(&format!("; {name} {} {}", r.value, r.unit));
    }
    line
}

/// Check a snapshot read from a relay before showing it.
pub(crate) fn parse(payload: &[u8]) -> Result<Stats> {
    ensure!(payload.len() <= 4096, "stats object exceeds 4 KiB");
    let stats: Stats = serde_json::from_slice(payload)?;
    ensure!(
        stats.v == STATS_VERSION,
        "stats version {} is not supported (expected {STATS_VERSION})",
        stats.v
    );
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario() -> Scenario {
        Scenario {
            source: Source::Pedal,
            band: Band::TwoMetre,
            nominal_hz: 145_800_000,
            altitude_km: 420.0,
            max_elevation_deg: 45.0,
            cycle_s: 900,
            gain_db: 20.0,
            level_at_1000km_dbfs: -80.0,
            track_doppler: false,
            passband_hz: 1500.0,
        }
    }

    #[test]
    fn pass_length_is_realistic() {
        // An ISS-height pass that peaks at 45 degrees lasts roughly 9 to 10 minutes.
        let d = scenario().pass_duration_s();
        assert!((480.0..660.0).contains(&d), "{d}");
        let overhead = Scenario {
            max_elevation_deg: 90.0,
            ..scenario()
        }
        .pass_duration_s();
        assert!(overhead > d);
    }

    #[test]
    fn peak_matches_requested_elevation() {
        let s = scenario();
        let g = s.geometry(0.0);
        assert!((g.elevation_deg - 45.0).abs() < 0.01, "{}", g.elevation_deg);
        // Horizon at the ends of the pass.
        let edge = s.geometry(s.pass_duration_s() / 2.0);
        assert!(edge.elevation_deg.abs() < 0.05, "{}", edge.elevation_deg);
    }

    #[test]
    fn doppler_goes_high_then_low() {
        let s = scenario();
        let half = s.pass_duration_s() / 2.0;
        let rise = s.doppler_hz(-half + 1.0);
        let set = s.doppler_hz(half - 1.0);
        assert!(rise > 2500.0 && rise < 4000.0, "{rise}");
        assert!(set < -2500.0 && set > -4000.0, "{set}");
        assert!(s.doppler_hz(0.0).abs() < 1.0);
        // Four times the frequency, about four times the shift.
        let uhf = Scenario {
            nominal_hz: 437_800_000,
            band: Band::SeventyCm,
            ..s
        };
        assert!((uhf.doppler_hz(-half + 1.0) / rise - 3.0).abs() < 0.01);
    }

    fn publisher() -> Publisher {
        Publisher {
            station: "room-12".into(),
            session: "abc123def456".into(),
        }
    }

    fn top(s: &Scenario) -> u64 {
        60 + (s.pass_duration_s() / 2.0) as u64
    }

    #[test]
    fn snapshots_are_labelled_and_versioned() {
        let s = scenario();
        let p = publisher();
        let before = s.snapshot(&p, 10, 0);
        assert_eq!(before.provenance, Provenance::DeskSimulation);
        assert_eq!(before.v, 1);
        assert_eq!(before.contact, Contact::Idle);
        assert!(before.sensors.is_none());
        assert!(before.snr_db.is_none());
        assert_eq!(before.level_dbfs, before.noise_dbfs);
        let peak = s.snapshot(&p, top(&s), 0);
        assert_eq!(peak.contact, Contact::Acquired);
        assert!(peak.snr_db.unwrap() > 10.0);
        assert!(peak.sensors.as_ref().unwrap().contains_key("temperature"));
        let json = serde_json::to_vec(&peak).unwrap();
        assert_eq!(parse(&json).unwrap(), peak);
    }

    #[test]
    fn json_matches_the_phone_app_contract() {
        // Field names and values cubesat.app's parseSnapshot (mobile/src/lib/snapshot.ts) needs.
        let s = scenario();
        let v: serde_json::Value =
            serde_json::to_value(s.snapshot(&publisher(), top(&s), 1_795_804_205_120)).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["seq"], top(&s));
        assert_eq!(v["session"], "abc123def456");
        assert_eq!(v["station"], "room-12");
        assert_eq!(v["source"], "Simulated satellite");
        assert_eq!(v["provenance"], "desk-simulation");
        assert_eq!(v["utc"], "2026-11-27T18:30:05.120Z");
        assert_eq!(v["contact"], "acquired");
        assert_eq!(v["frequency_hz"], 145_800_000);
        assert!(v["level_dbfs"].as_f64().unwrap() <= 0.0);
        assert!(v["elevation_deg"].as_f64().unwrap() > 44.0);
        assert_eq!(v["sensors"]["voltage"]["unit"], "V");
        let below = serde_json::to_value(s.snapshot(&publisher(), 0, 0)).unwrap();
        assert_eq!(below["contact"], "idle");
        assert!(below.get("sensors").is_none(), "absent, not null");
    }

    #[test]
    fn off_tune_in_view_is_lost() {
        let s = Scenario {
            level_at_1000km_dbfs: -100.0,
            ..scenario()
        };
        assert_eq!(s.snapshot(&publisher(), 70, 0).contact, Contact::Lost);
    }

    #[test]
    fn doppler_tracking_keeps_the_offset_small() {
        let s = Scenario {
            track_doppler: true,
            ..scenario()
        };
        let st = s.snapshot(&publisher(), 100, 0);
        assert_eq!(st.contact, Contact::Acquired);
        let offset = st.offset_hz.unwrap();
        assert!(offset.abs() <= 0.5, "{offset}");
        assert_ne!(st.frequency_hz, st.nominal_hz);
        let fixed = scenario().snapshot(&publisher(), 100, 0);
        assert!(fixed.offset_hz.unwrap().abs() > 1000.0);
    }

    #[test]
    fn off_tune_the_carrier_is_not_heard() {
        // Just after rise the Doppler shift is about +3.4 kHz, outside the pedal's 1.5 kHz passband.
        let fixed = scenario().snapshot(&publisher(), 61, 0);
        assert!(fixed.offset_hz.unwrap().abs() > 1500.0);
        assert_eq!(fixed.contact, Contact::Lost);
        assert!(fixed.sensors.is_none());
        assert!(fixed.snr_db.is_none());
        let tracked = Scenario {
            track_doppler: true,
            ..scenario()
        }
        .snapshot(&publisher(), 61, 0);
        assert_eq!(tracked.contact, Contact::Acquired);
    }

    #[test]
    fn too_much_gain_clips() {
        let s = Scenario {
            gain_db: 40.0,
            level_at_1000km_dbfs: -30.0,
            ..scenario()
        };
        let st = s.snapshot(&publisher(), top(&s), 0);
        assert_eq!(st.clipping, Some(true));
        assert!(st.level_dbfs.is_some_and(|level| level <= 0.0));
        assert!(st.sensors.is_none(), "a clipped signal does not decode");
    }

    #[test]
    fn wrong_version_is_refused() {
        let mut st = scenario().snapshot(&publisher(), 0, 0);
        st.v = 2;
        let err = parse(&serde_json::to_vec(&st).unwrap()).unwrap_err();
        assert!(format!("{err:#}").contains("version"));
    }

    #[test]
    fn description_starts_with_the_label() {
        let s = scenario();
        let line = describe(&s.snapshot(&publisher(), top(&s), 0));
        assert!(line.starts_with("[desk simulation]"), "{line}");
        assert!(line.contains("temperature"), "{line}");
        assert!(describe(&s.snapshot(&publisher(), 0, 0)).contains("below the horizon"));
    }

    #[test]
    fn station_slugs_follow_the_app_rule() {
        for ok in ["home", "room-12", "a", "abc-def-9"] {
            assert!(valid_station(ok), "{ok}");
        }
        for bad in ["", "-home", "home-", "Room12", "my school", &"a".repeat(41)] {
            assert!(!valid_station(bad), "{bad}");
        }
        let id = session_id(1_795_804_205_120, 4242);
        assert_eq!(id.len(), 12);
        assert!(
            id.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        );
        assert_ne!(id, session_id(1_795_804_205_121, 4242));
    }

    fn bridge_args(extra: &[&str]) -> BridgeArgs {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: BridgeArgs,
        }
        let mut argv = vec!["test", "--gain-db", "30"];
        argv.extend_from_slice(extra);
        Cli::parse_from(argv).args
    }

    #[test]
    fn bridge_finds_a_carrier_in_rtl_power_rows() {
        let args = bridge_args(&[]);
        // 145.790–145.810 MHz in 2.5 kHz bins: noise near -40 dB with a carrier at 145.800.
        let row = "2026-11-27, 18:30:05, 145790000, 145810000, 2500.00, 1000, -40.1, -39.8, -40.3, -40.0, -22.5, -39.9, -40.2, -40.4";
        let st =
            from_rtl_power(row, &args, &publisher(), 7, &TimeZone::UTC, None).expect("a snapshot");
        assert_eq!(st.provenance, Provenance::ReceivedRf);
        assert_eq!(st.receiver, Source::RtlSdrV4);
        assert_eq!(st.contact, Contact::Acquired);
        assert_eq!(st.frequency_hz, 145_800_000);
        assert_eq!(st.level_db_rel, Some(-22.5));
        let snr = st.snr_db.unwrap();
        assert!((snr - 17.6).abs() < 0.3, "{snr}");
        assert!(st.elevation_deg.is_none() && st.doppler_hz.is_none());
        assert!(describe(&st).starts_with("[received RF]"));
    }

    #[test]
    fn bridge_skips_comments_and_rows_away_from_the_frequency() {
        let args = bridge_args(&["--label", "hardware-bench"]);
        assert!(
            from_rtl_power("# a comment", &args, &publisher(), 0, &TimeZone::UTC, None).is_none()
        );
        let far = "2026-11-27, 18:30:05, 146000000, 146010000, 2500.00, 1000, -40, -40, -40, -40";
        assert!(from_rtl_power(far, &args, &publisher(), 0, &TimeZone::UTC, None).is_none());
        let near = "2026-11-27, 18:30:05, 145799000, 145801000, 1000.00, 1000, -30, -31";
        assert_eq!(
            from_rtl_power(near, &args, &publisher(), 0, &TimeZone::UTC, None)
                .unwrap()
                .provenance,
            Provenance::HardwareBench
        );
    }

    #[test]
    fn an_off_tune_carrier_outside_the_passband_is_not_heard_or_decoded() {
        let strong = Scenario {
            level_at_1000km_dbfs: -50.0,
            ..scenario()
        };
        // Early in the pass the Doppler shift is largest; find a second where it is outside
        // the 1.5 kHz passband of a receiver left on the published frequency.
        let rise = (60..900)
            .find(|&q| {
                let st = strong.snapshot(&publisher(), q, 0);
                st.elevation_deg.unwrap() > 5.0 && st.offset_hz.unwrap().abs() > 1500.0
            })
            .expect("a second with a large offset");
        let fixed = strong.snapshot(&publisher(), rise, 0);
        assert_eq!(fixed.contact, Contact::Lost);
        assert!(
            fixed.sensors.is_none(),
            "nothing decodes outside the passband"
        );
        assert!(fixed.snr_db.is_none());
        // A station that follows the Doppler shift hears and decodes the same second.
        let tracked = Scenario {
            track_doppler: true,
            ..strong
        }
        .snapshot(&publisher(), rise, 0);
        assert_eq!(tracked.contact, Contact::Acquired);
        assert!(tracked.sensors.is_some());
    }

    #[test]
    fn bridge_levels_are_relative_and_say_nothing_about_clipping() {
        let args = bridge_args(&[]);
        // rtl_power bins are averaged FFT power and can be above 0 dB without clipping.
        let row = "2026-11-27, 18:30:05, 145799000, 145801000, 1000.00, 1000, -20.0, 4.5";
        let st = from_rtl_power(row, &args, &publisher(), 0, &TimeZone::UTC, None).unwrap();
        assert_eq!(st.level_db_rel, Some(4.5), "relative levels may exceed 0");
        assert!(st.level_dbfs.is_none() && st.noise_dbfs.is_none());
        assert!(
            st.snr_db.is_none(),
            "no out-of-band bins to estimate the floor"
        );
        assert_eq!(st.clipping, None);
        let json = serde_json::to_value(&st).unwrap();
        assert!(
            json.get("clipping").is_none(),
            "left out, not reported as false"
        );
        assert!(!describe(&st).contains("CLIPPING"));
    }

    #[test]
    fn bridge_keeps_capture_and_read_times_separate() {
        let args = bridge_args(&["--rtl-power-tz", "America/Denver"]);
        let zone = args.rtl_power_tz.as_ref().unwrap();
        let row = "2024-01-15, 09:30:05, 145799000, 145801000, 1000.00, 1000, -40, -20";
        let st =
            from_rtl_power(row, &args, &publisher(), 0, zone, Some(1_795_804_205_120)).unwrap();
        assert_eq!(st.utc, "2024-01-15T16:30:05.000Z");
        assert_eq!(st.received_utc.as_deref(), Some("2026-11-27T18:30:05.120Z"));
        let old = from_rtl_power(row, &args, &publisher(), 0, &TimeZone::UTC, None).unwrap();
        let json = serde_json::to_value(&old).unwrap();
        assert!(json.get("received_utc").is_none());
        assert_eq!(parse(&serde_json::to_vec(&old).unwrap()).unwrap(), old);
    }

    #[test]
    fn bridge_skips_bad_and_ambiguous_capture_times() {
        let zone = TimeZone::get("America/Denver").unwrap();
        for row in [
            "2026-02-31, 18:30:05, ...",
            "2026-03-08, 02:30:05, ...",
            "2026-11-01, 01:30:05, ...",
        ] {
            assert!(sweep_unix_ms(row, &zone, None).is_none(), "{row}");
        }
    }

    #[test]
    fn bridge_resolves_the_repeated_hour_from_the_read_time() {
        let zone = TimeZone::get("America/Denver").unwrap();
        let row = "2026-11-01, 01:30:05, ...";
        // 01:30:05 MDT is 07:30:05 UTC; an hour later the clocks repeat it as 08:30:05 UTC.
        let first = 1_793_518_205_000;
        let second = first + 3_600_000;
        // Read just after the first 01:30: it can only be the first one.
        assert_eq!(sweep_unix_ms(row, &zone, Some(first + 900)), Some(first));
        // Read just after the second 01:30: take the later one.
        assert_eq!(sweep_unix_ms(row, &zone, Some(second + 900)), Some(second));
        // A read time before both means the clocks disagree: skip the row.
        assert_eq!(sweep_unix_ms(row, &zone, Some(first - 1)), None);
        // Clocks going forward: 02:30 never happened, whatever the read time.
        assert_eq!(
            sweep_unix_ms("2026-03-08, 02:30:05, ...", &zone, Some(second)),
            None
        );
    }

    #[test]
    fn utc_formatting() {
        assert_eq!(utc_string(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(utc_string(1_795_804_205_120), "2026-11-27T18:30:05.120Z");
        assert_eq!(utc_string(951_782_400_000), "2000-02-29T00:00:00.000Z");
    }
}
