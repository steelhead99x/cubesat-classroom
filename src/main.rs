mod balloon;
mod bridge;
mod classroom;
mod flight;
mod passes;

use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use moq_tokio::moq_net::{self, Timestamp};
use serde::{Deserialize, Serialize};
use url::Url;

const TRACK: &str = "telemetry.v1";
const MAX_AGE: Duration = Duration::from_secs(5);
const DEADLINE: Duration = Duration::from_secs(10);

#[derive(Parser)]
#[command(about = "CubeSat Classroom ground station")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the two-relay classroom mission and ground console on localhost.
    Classroom {
        /// Local HTTP port for the ground console.
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Optional directory with temperature_mc, voltage_mv and light_lux files.
        #[arg(long)]
        sensor_dir: Option<PathBuf>,
    },
    /// Record a bench pressure and temperature data set for a balloon quest.
    BalloonLog {
        #[arg(long)]
        sensor_dir: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u32).range(1..=86400))]
        samples: u32,
    },
    /// Turn Arduino, XIAO or Raspberry Pi readings into sensor files for the station.
    SensorBridge {
        /// Folder to write temperature_mc, voltage_mv, light_lux and pressure_pa into.
        #[arg(long, default_value = "run/sensors")]
        out: PathBuf,
        /// Serial device, file or `-` (standard input) carrying `CSC1 key=value` lines.
        #[arg(long)]
        input: Option<PathBuf>,
        /// Read a Linux file as a sensor: name=path[*scale]. Repeat for each sensor.
        #[arg(long = "map")]
        maps: Vec<String>,
        /// Remove a value if no new reading arrives within this many seconds.
        #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u64).range(1..=3600))]
        stale_secs: u64,
        /// How often to read mapped files, in milliseconds.
        #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(100..=60_000))]
        poll_ms: u64,
    },
    /// Predict when satellites pass over a location, from a TLE file.
    ///
    /// Elements older than 14 days are flagged in the output. A pass already under way
    /// starts "now"; one still under way at the end of the window is left out.
    Passes {
        /// TLE file in three-line format (name line plus two element lines).
        #[arg(long)]
        tle: PathBuf,
        /// Only satellites whose name contains this text (case-insensitive).
        #[arg(long)]
        satellite: Option<String>,
        /// Observer latitude in degrees (north positive).
        #[arg(long, allow_hyphen_values = true)]
        lat: f64,
        /// Observer longitude in degrees (east positive).
        #[arg(long, allow_hyphen_values = true)]
        lon: f64,
        /// Observer altitude in metres.
        #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
        alt_m: f64,
        /// Hours ahead to search.
        #[arg(long, default_value_t = 24.0)]
        hours: f64,
        /// Ignore passes whose highest point is below this elevation, in degrees.
        #[arg(long, default_value_t = 10.0, allow_hyphen_values = true)]
        min_elevation: f64,
        /// Print JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Desk simulation of a suborbital research flight: phases, height and what a logger should see.
    ///
    /// Defaults use public Virgin Galactic figures (Payload User's Guide, 'Galactic 07' flight facts).
    /// The output is a classroom prediction, not flight data.
    FlightProfile {
        /// Write a CSV timeline to this file.
        #[arg(long, conflicts_with = "json")]
        output: Option<PathBuf>,
        /// Height at release from the carrier aircraft, in feet.
        #[arg(long, default_value_t = 44_562.0)]
        release_ft: f64,
        /// Highest point, in miles.
        #[arg(long, default_value_t = 54.4)]
        apogee_miles: f64,
        /// Rocket burn, in seconds.
        #[arg(long, default_value_t = 60.0)]
        burn_s: f64,
        /// Minutes from take-off to release (an assumption; ask the provider).
        #[arg(long, default_value_t = 50.0)]
        climb_min: f64,
        /// Minutes from take-off to landing.
        #[arg(long, default_value_t = 70.0)]
        total_min: f64,
        /// Cabin pressure as an equivalent height, in feet.
        #[arg(long, default_value_t = 5_500.0)]
        cabin_ft: f64,
        /// Seconds between rows.
        #[arg(long, default_value_t = 1.0)]
        step_s: f64,
        /// Print JSON instead of a summary.
        #[arg(long)]
        json: bool,
    },
    /// Start a loopback-only QUIC relay and print its URL and certificate pin.
    Relay {
        #[arg(long, default_value_t = 4443)]
        port: u16,
    },
    /// Publish synthetic sensor readings at 1 Hz over MoQ.
    Simulate {
        #[command(flatten)]
        link: Link,
    },
    /// Publish three configured Linux sensor values at 1 Hz over MoQ.
    PublishSensors {
        #[command(flatten)]
        link: Link,
        #[command(flatten)]
        inputs: SensorInputs,
    },
    /// Read a fresh telemetry value on the ground.
    ReadTelemetry {
        #[command(flatten)]
        link: Link,
        #[arg(long, default_value = "status")]
        reading: Reading,
    },
    /// Verify reception of the existing moq-subghz H.264 broadcast over QUIC.
    VerifyVideo {
        #[command(flatten)]
        link: Link,
        /// Write received Annex-B H.264 to stdout for a player or file.
        #[arg(long)]
        stream: bool,
    },
}

#[derive(clap::Args)]
struct Link {
    #[arg(long, env = "CUBESAT_RELAY")]
    relay: Url,
    #[arg(long, env = "CUBESAT_TLS_FINGERPRINT")]
    fingerprint: Option<String>,
    #[arg(long, default_value = "cubesat/classroom/telemetry")]
    broadcast: String,
}

#[derive(clap::Args)]
struct SensorInputs {
    /// File containing the signed integer temperature in millidegrees Celsius.
    #[arg(long)]
    temperature_mc: PathBuf,
    /// File containing the nonnegative integer bus voltage in millivolts.
    #[arg(long)]
    voltage_mv: PathBuf,
    /// File containing the nonnegative integer light level in lux.
    #[arg(long)]
    light_lux: PathBuf,
}

impl SensorInputs {
    fn sample(&self, sequence: u64) -> Result<Sample> {
        let captured_unix_ms = unix_ms()?;
        Ok(Sample {
            sequence,
            captured_unix_ms,
            source: "configured sensor inputs".into(),
            temperature_c: f64::from(read_integer::<i32>(&self.temperature_mc)?) / 1000.0,
            bus_voltage_v: f64::from(read_integer::<u32>(&self.voltage_mv)?) / 1000.0,
            light_lux: read_integer(&self.light_lux)?,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    sequence: u64,
    captured_unix_ms: u64,
    source: String,
    temperature_c: f64,
    bus_voltage_v: f64,
    light_lux: u32,
}

impl Sample {
    fn synthetic(sequence: u64) -> Result<Self> {
        Ok(Self {
            sequence,
            captured_unix_ms: unix_ms()?,
            source: "simulation".into(),
            temperature_c: 18.0 + (sequence % 11) as f64 / 2.0,
            bus_voltage_v: (3900 + sequence % 61) as f64 / 1000.0,
            light_lux: if sequence % 30 < 20 { 460 } else { 0 },
        })
    }

    fn age_ms(&self, now: u64) -> Result<u64> {
        let age = now
            .checked_sub(self.captured_unix_ms)
            .context("capture timestamp is ahead of the receiver clock")?;
        ensure!(
            u128::from(age) <= MAX_AGE.as_millis(),
            "telemetry is stale ({age} ms)"
        );
        Ok(age)
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Reading {
    Temperature,
    Voltage,
    Light,
    Status,
}

impl Reading {
    fn format(self, sample: &Sample, now: u64) -> Result<String> {
        let age = sample.age_ms(now)?;
        let detail = match self {
            Self::Temperature => format!("{:.1} °C payload temperature", sample.temperature_c),
            Self::Voltage => format!("{:.3} V bus voltage", sample.bus_voltage_v),
            Self::Light => format!("{} lx light level", sample.light_lux),
            Self::Status => format!(
                "{:.1} °C payload temperature, {:.3} V bus voltage, {} lx light level",
                sample.temperature_c, sample.bus_voltage_v, sample.light_lux
            ),
        };
        Ok(format!(
            "{detail}. Source {} · sample #{} · captured {} ms ago.",
            sample.source, sample.sequence, age
        ))
    }
}

fn read_integer<T: std::str::FromStr>(path: &Path) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    value
        .trim()
        .parse()
        .with_context(|| format!("parse integer from {}", path.display()))
}

fn unix_ms() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

fn connect(link: &Link) -> Result<moq_tokio::Client> {
    let mut cfg = moq_tokio::connect::Config::default();
    if let Some(pin) = &link.fingerprint {
        cfg.tls.fingerprint = vec![pin.clone()];
    }
    // The relays listen on 127.0.0.1 only. The library's default dual-stack
    // `[::]:0` dial socket fails on hosts with IPv6 disabled (some Windows WSL setups, Docker
    // and managed school machines), so dial loopback relays from IPv4 loopback.
    if is_loopback(&link.relay) {
        cfg.bind = Some(SocketAddr::from(([127, 0, 0, 1], 0)));
    }
    Ok(cfg.init(Default::default())?)
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        // `moqt://` is not a special scheme, so the url crate reports IPv4 hosts as domains.
        Some(url::Host::Domain(name)) => {
            name.eq_ignore_ascii_case("localhost")
                || name
                    .parse::<std::net::Ipv4Addr>()
                    .is_ok_and(|ip| ip.is_loopback())
        }
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        _ => false,
    }
}

async fn relay(port: u16) -> Result<()> {
    let relay = start_relay(port).await?;
    println!("{} {}", relay.url, relay.pin);
    io::stdout().flush()?;
    tokio::signal::ctrl_c().await?;
    Ok(())
}

struct LocalRelay {
    origin: moq_net::origin::Producer,
    url: Url,
    pin: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for LocalRelay {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn start_relay(port: u16) -> Result<LocalRelay> {
    let origin = moq_tokio::origin::spawn();
    let mut cfg = moq_tokio::listen::Config::default();
    cfg.bind = Some(moq_tokio::listen::Bind::Addr(SocketAddr::from((
        [127, 0, 0, 1],
        port,
    ))));
    cfg.tls.generate = vec!["localhost".into()];
    let mut listener = cfg.init(Default::default())?.listen().await?;
    let port = listener.local_addr()?.port();
    let pins = listener.certificates().fingerprints();
    let pin = pins
        .first()
        .context("relay certificate fingerprint missing")?;
    let serving_origin = origin.clone();
    let task = tokio::spawn(async move {
        while let Some(request) = listener.accept().await {
            let origin = serving_origin.clone();
            tokio::spawn(async move {
                if let Ok(session) = request
                    .with_publisher(origin.consume())
                    .with_subscriber(origin)
                    .ok()
                    .await
                {
                    let _ = session.closed().await;
                }
            });
        }
    });
    Ok(LocalRelay {
        origin,
        url: format!("moqt://localhost:{port}").parse()?,
        pin: pin.clone(),
        task,
    })
}

async fn publish(link: Link, sample: impl Fn(u64) -> Result<Sample>) -> Result<()> {
    let origin = moq_tokio::origin::spawn();
    let broadcast = origin.create_broadcast(&link.broadcast)?;
    let track = broadcast.create_track(TRACK, None)?;
    broadcast.announce(Default::default())?;

    let client = connect(&link)?.with_publisher(&origin);
    let session = client.connect(link.relay);
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut sequence = 0;
    let mut input_failed = false;
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let reading = sample(sequence);
                sequence += 1;
                let reading = match reading {
                    Ok(reading) => {
                        if input_failed {
                            eprintln!("All sensor inputs recovered.");
                        }
                        input_failed = false;
                        reading
                    }
                    Err(error) => {
                        if !input_failed {
                            eprintln!("Publishing paused due to sensor input error: {error:#}");
                        }
                        input_failed = true;
                        continue;
                    }
                };
                let mut group = track.append_group()?;
                group.write_frame(Timestamp::now(), serde_json::to_vec(&reading)?)?;
                group.finish()?;
            }
            result = session.closed() => {
                result?;
                bail!("relay session closed");
            }
            signal = tokio::signal::ctrl_c() => return Ok(signal?),
        }
    }
}

async fn with_broadcast<T>(
    link: &Link,
    run: impl AsyncFnOnce(moq_net::broadcast::Consumer) -> Result<T>,
) -> Result<T> {
    let origin = moq_tokio::origin::spawn();
    let consumer = origin.consume();
    let mut announced = consumer.announced();
    let client = connect(link)?.with_subscriber(origin);
    let session = client.connect(link.relay.clone());
    let discover = async {
        loop {
            let update = announced
                .next()
                .await
                .context("announcement stream closed")?;
            if update.prefix.as_str() == link.broadcast && update.kind.is_active() {
                return Ok::<_, anyhow::Error>(consumer.request_broadcast(&link.broadcast).await?);
            }
        }
    };
    tokio::select! {
        result = async {
            let broadcast = tokio::time::timeout(DEADLINE, discover).await.context("broadcast not found within 10 seconds")??;
            run(broadcast).await
        } => result,
        result = session.closed() => {
            result?;
            bail!("relay session closed");
        }
    }
}

async fn first_object(
    broadcast: &moq_net::broadcast::Consumer,
    name: &str,
) -> Result<moq_net::frame::Frame> {
    tokio::time::timeout(DEADLINE, async {
        let mut sub = broadcast.track(name)?.subscribe(None).await?;
        let mut group = sub.recv_group().await?.context("track ended")?;
        group
            .read_frame()
            .await?
            .context("group contained no object")
    })
    .await
    .context("no object received within 10 seconds")?
}

async fn video(broadcast: moq_net::broadcast::Consumer, stream: bool) -> Result<()> {
    use hang::catalog::{Catalog, VideoCodec};
    let catalog = first_object(&broadcast, Catalog::DEFAULT_NAME).await?;
    let catalog: Catalog = Catalog::from_slice(&catalog.payload)?;
    let config = catalog
        .video
        .renditions
        .get("video")
        .context("video track missing")?;
    ensure!(
        matches!(config.codec, VideoCodec::H264(ref codec) if codec.inline),
        "expected in-band H.264 configuration"
    );
    let mut sub = broadcast.track("video")?.subscribe(None).await?;
    let mut count = 0;
    let mut output = io::stdout();
    while let Some(mut group) = tokio::time::timeout(DEADLINE, sub.recv_group()).await?? {
        while let Some(frame) = tokio::time::timeout(DEADLINE, group.read_frame()).await?? {
            let frame = hang::container::Frame::decode(frame.payload)?;
            ensure!(
                frame.payload.starts_with(&[0, 0, 1]) || frame.payload.starts_with(&[0, 0, 0, 1]),
                "video object is missing an Annex-B start code"
            );
            count += 1;
            if !stream {
                println!(
                    "Native QUIC received an AVC3 catalog and a {}-byte H.264 access unit.",
                    frame.payload.len()
                );
                return Ok(());
            }
            output.write_all(&frame.payload)?;
            output.flush()?;
        }
    }
    ensure!(count > 0, "video track ended without any video");
    eprintln!("Received {count} H.264 access units over native QUIC.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = moq_tokio::crypto::install_default();
    match Cli::parse().command {
        Command::Classroom { port, sensor_dir } => {
            classroom::run(port, sensor_dir).await?;
        }
        Command::BalloonLog {
            sensor_dir,
            output,
            samples,
        } => {
            balloon::log(sensor_dir, output, samples).await?;
        }
        Command::SensorBridge {
            out,
            input,
            maps,
            stale_secs,
            poll_ms,
        } => {
            let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let flag = stop.clone();
            tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            });
            let args = bridge::BridgeArgs {
                out,
                input,
                maps,
                stale_secs,
                poll_ms,
            };
            tokio::task::spawn_blocking(move || bridge::run(args, &stop)).await??;
        }
        Command::Passes {
            tle,
            satellite,
            lat,
            lon,
            alt_m,
            hours,
            min_elevation,
            json,
        } => {
            passes::run(passes::PassArgs {
                tle,
                satellite,
                lat,
                lon,
                alt_m,
                hours,
                min_elevation,
                json,
            })?;
        }
        Command::FlightProfile {
            output,
            release_ft,
            apogee_miles,
            burn_s,
            climb_min,
            total_min,
            cabin_ft,
            step_s,
            json,
        } => {
            flight::run(flight::FlightArgs {
                output,
                release_ft,
                apogee_miles,
                burn_s,
                climb_min,
                total_min,
                cabin_ft,
                step_s,
                json,
            })?;
        }
        Command::Relay { port } => relay(port).await?,
        Command::Simulate { link } => publish(link, Sample::synthetic).await?,
        Command::PublishSensors { link, inputs } => {
            publish(link, |sequence| inputs.sample(sequence)).await?;
        }
        Command::ReadTelemetry { link, reading } => {
            let object = with_broadcast(&link, async |broadcast| {
                first_object(&broadcast, TRACK).await
            })
            .await?;
            ensure!(
                object.payload.len() <= 4096,
                "telemetry object exceeds 4 KiB"
            );
            let sample: Sample = serde_json::from_slice(&object.payload)?;
            println!("{}", reading.format(&sample, unix_ms()?)?);
        }
        Command::VerifyVideo { link, stream } => {
            with_broadcast(&link, async |broadcast| video(broadcast, stream).await).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_stale_and_future_data() {
        let sample = Sample::synthetic(1).unwrap();
        let now = sample.captured_unix_ms;
        assert!(Reading::Temperature.format(&sample, now + 5000).is_ok());
        assert!(Reading::Temperature.format(&sample, now + 5001).is_err());
        assert!(Reading::Temperature.format(&sample, now - 1).is_err());
    }

    #[test]
    fn uses_sensor_values_with_provenance() {
        let sample = Sample::synthetic(7).unwrap();
        let now = sample.captured_unix_ms + 100;
        let answer = Reading::Status.format(&sample, now).unwrap();
        assert!(answer.contains("21.5 °C") && answer.contains("3.907 V"));
        assert!(answer.contains("simulation") && answer.contains("100 ms"));
    }

    #[test]
    fn detects_loopback_relays() {
        assert!(is_loopback(&"moqt://localhost:4443".parse().unwrap()));
        assert!(is_loopback(&"moqt://127.0.0.1:4443".parse().unwrap()));
        assert!(!is_loopback(&"moqt://relay.example:443".parse().unwrap()));
        assert!(!is_loopback(&"moqt://[::1]:4443".parse().unwrap()));
    }
}
