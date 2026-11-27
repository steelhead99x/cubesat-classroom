//! Desk simulation: 915 MHz frames in, Media over QUIC tracks out.
//!
//! Virtual receivers apply the loss model. One assembler drops duplicate copies.
//! A scheduler publishes `telemetry` before any optional media. Nothing transmits.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use downlink::{
    AirConfig, AirKind, Assembler, AudioFrame, Budget, Channel, ChannelConfig, FeedTrack, GroupKey,
    LIVE_BROADCAST, Piece, Recovered, RelayHealth, Scheduler, Service, StationId, Telemetry,
    encode_object,
};
use moq_net::track::{self, Subscription};
use tokio::sync::{mpsc, watch};

use super::{Link, Reading, Timestamp, connect, moq_net, unix_ms};

/// Relay address for the live feed. The broadcast defaults to [`LIVE_BROADCAST`],
/// not the older `cubesat/classroom/telemetry` name used by `simulate`.
#[derive(clap::Args, Clone, Debug)]
pub(crate) struct LiveLink {
    /// Relay URL, usually the `moqt://` line printed by `relay`.
    #[arg(long, env = "CUBESAT_RELAY")]
    relay: url::Url,
    /// Certificate fingerprint printed beside the relay URL.
    #[arg(long, env = "CUBESAT_TLS_FINGERPRINT")]
    fingerprint: Option<String>,
    /// Broadcast name. School apps subscribe to this name.
    #[arg(long, default_value = LIVE_BROADCAST)]
    broadcast: String,
}

impl LiveLink {
    fn to_link(&self) -> Link {
        Link {
            relay: self.relay.clone(),
            fingerprint: self.fingerprint.clone(),
            broadcast: self.broadcast.clone(),
        }
    }
}

/// `live-feed` arguments. The process does not transmit.
#[derive(clap::Args, Debug)]
#[command(
    about = "Publish a desk-simulated 915 MHz downlink to a Media over QUIC relay. Does not transmit."
)]
pub(crate) struct FeedArgs {
    /// Relay, fingerprint, and broadcast name.
    #[command(flatten)]
    link: LiveLink,
    /// `simulation`, `part15`, or `part97`. A label on the frame, not an authorization.
    #[arg(long, value_enum, default_value_t = ServiceArg::Simulation)]
    service: ServiceArg,
    /// Control-operator station ID. Required for `--service part97`. Not a student name.
    #[arg(long)]
    station_id: Option<String>,
    /// Virtual receivers that hear each frame. Each drops frames on its own.
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=8))]
    relays: u8,
    /// Chance, in parts per thousand, that a virtual receiver drops a frame.
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u16).range(0..=1000))]
    loss_per_mille: u16,
    /// Dropped frames per loss, including the first. 1 drops a single frame.
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=64))]
    burst: u16,
    /// Reed-Solomon parity shards added to each object. 0 sends the payload only.
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..=16))]
    parity: u8,
    /// Seed for the virtual loss draw.
    #[arg(long, default_value_t = 0x0915)]
    seed: u64,
    /// Also publish optional image, audio, and video tracks.
    #[arg(long)]
    media: bool,
    /// Fan-out budget in bits per second. Telemetry JSON is still sent.
    /// Media that does not fit is dropped. Omit for no cap.
    #[arg(long)]
    budget_bps: Option<u64>,
    /// Milliseconds between snapshots.
    #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(50..=60_000))]
    interval_ms: u64,
    /// Millidegree Celsius file. Requires `--voltage-mv` and `--light-lux` too. Each file must
    /// be rewritten for every reading (as `sensor-bridge` does): its modification time is the
    /// capture time, and a file older than 5 s pauses publishing. Values under /sys and /proc
    /// are read live instead.
    #[arg(long)]
    temperature_mc: Option<std::path::PathBuf>,
    /// Millivolt file.
    #[arg(long)]
    voltage_mv: Option<std::path::PathBuf>,
    /// Lux file.
    #[arg(long)]
    light_lux: Option<std::path::PathBuf>,
    /// Optional pascal file. A missing read leaves pressure empty; it does not stop telemetry.
    #[arg(long)]
    pressure_pa: Option<std::path::PathBuf>,
}

/// `read-live` arguments. Subscribes to one track and ignores the others.
#[derive(clap::Args)]
#[command(about = "Read one live-feed track. Reference client for docs/client-contract.md.")]
pub(crate) struct ReadArgs {
    /// Relay, fingerprint, and broadcast name.
    #[command(flatten)]
    link: LiveLink,
    /// `telemetry`, `telemetry.cbor`, `health`, `image`, `audio`, or `video`.
    #[arg(long, default_value = "telemetry")]
    track: String,
    /// Which telemetry field to print. Ignored for other tracks.
    #[arg(long, default_value = "status")]
    reading: Reading,
    /// Print telemetry or health as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum ServiceArg {
    Simulation,
    #[value(name = "part15")]
    Part15,
    #[value(name = "part97")]
    Part97,
}

impl ServiceArg {
    fn service(self) -> Service {
        match self {
            Self::Simulation => Service::Simulation,
            Self::Part15 => Service::Part15,
            Self::Part97 => Service::Part97,
        }
    }
}

struct Tick {
    telemetry: Telemetry,
    image: Option<Vec<u8>>,
    audio: Option<Vec<u8>>,
    video: Option<Vec<u8>>,
}

struct Gateway {
    air: AirConfig,
    channel: Channel,
    assembler: Assembler,
    scheduler: Scheduler,
}

impl Gateway {
    fn new(air: AirConfig, channel: ChannelConfig, budget: Budget) -> Result<Self> {
        air.check()?;
        Ok(Self {
            air,
            channel: Channel::new(&channel)?,
            assembler: Assembler::new(),
            scheduler: Scheduler::new(budget),
        })
    }

    fn ingest(&mut self, tick: &Tick) -> Result<Vec<Piece>> {
        let sequence = u32::try_from(tick.telemetry.sequence)
            .context("telemetry sequence does not fit in an air frame")?;
        let mut pieces = Vec::new();
        if let Some(recovered) = self.send(
            AirKind::Telemetry,
            sequence,
            &tick.telemetry.to_board_body()?,
        )? {
            let mut snapshot = Telemetry::from_board_body(
                &recovered.payload,
                u64::from(recovered.sequence),
                recovered.service,
                recovered.station_id.as_deref(),
            )?;
            // The air frame carries whole seconds, so decoding makes the reading look up to a
            // second older than it is, enough to push a fresh reading past read-live's age
            // limit. send() only returns this tick's own frame, so put back its exact time.
            if snapshot.captured_unix_ms == tick.telemetry.captured_unix_ms / 1000 * 1000 {
                snapshot.captured_unix_ms = tick.telemetry.captured_unix_ms;
            }
            pieces.push(Piece {
                track: FeedTrack::Telemetry,
                sequence: snapshot.sequence,
                payload: snapshot.to_json()?,
            });
            pieces.push(Piece {
                track: FeedTrack::TelemetryCbor,
                sequence: snapshot.sequence,
                payload: snapshot.to_cbor()?,
            });
        }
        self.push_media(
            sequence,
            AirKind::Image,
            FeedTrack::Image,
            tick.image.as_deref(),
            &mut pieces,
        )?;
        self.push_media(
            sequence,
            AirKind::Audio,
            FeedTrack::Audio,
            tick.audio.as_deref(),
            &mut pieces,
        )?;
        self.push_media(
            sequence,
            AirKind::Video,
            FeedTrack::Video,
            tick.video.as_deref(),
            &mut pieces,
        )?;
        let health =
            RelayHealth::from_metrics(tick.telemetry.captured_unix_ms, &self.assembler.metrics());
        pieces.push(Piece {
            track: FeedTrack::Health,
            sequence: tick.telemetry.sequence,
            payload: health.to_json()?,
        });
        for piece in pieces {
            self.scheduler.push(piece);
        }
        Ok(self.scheduler.drain())
    }

    fn push_media(
        &mut self,
        sequence: u32,
        kind: AirKind,
        track: FeedTrack,
        payload: Option<&[u8]>,
        pieces: &mut Vec<Piece>,
    ) -> Result<()> {
        let Some(payload) = payload else {
            return Ok(());
        };
        if let Some(recovered) = self.send(kind, sequence, payload)? {
            pieces.push(Piece {
                track,
                sequence: u64::from(sequence),
                payload: recovered.payload,
            });
        }
        Ok(())
    }

    fn send(&mut self, kind: AirKind, sequence: u32, payload: &[u8]) -> Result<Option<Recovered>> {
        let frames = match encode_object(&self.air, kind, sequence, payload) {
            Ok(frames) => frames,
            Err(downlink::Error::Payload(msg)) if kind.is_media() => {
                eprintln!("Skipped {kind:?} on the air path: {msg}");
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        };
        let mut found = None;
        for frame in frames {
            for heard in self.channel.observe(&frame) {
                if let Some(recovered) = self.assembler.push(heard)
                    && recovered.kind == kind
                    && recovered.sequence == sequence
                {
                    found = Some(recovered);
                }
            }
        }
        if found.is_none() {
            self.assembler.give_up(GroupKey {
                kind,
                session: self.air.session,
                sequence,
            });
        }
        Ok(found)
    }
}

struct Cache {
    sequence: u64,
    json: Vec<u8>,
    cbor: Vec<u8>,
}

/// Telemetry to publish again when the relay session comes back. Media is not repeated.
fn catch_up(cache: &Cache) -> [Piece; 2] {
    [
        Piece {
            track: FeedTrack::Telemetry,
            sequence: cache.sequence,
            payload: cache.json.clone(),
        },
        Piece {
            track: FeedTrack::TelemetryCbor,
            sequence: cache.sequence,
            payload: cache.cbor.clone(),
        },
    ]
}

fn remember(cache: &mut Option<Cache>, pieces: &[Piece]) {
    let json = pieces
        .iter()
        .find(|piece| piece.track == FeedTrack::Telemetry);
    let cbor = pieces
        .iter()
        .find(|piece| piece.track == FeedTrack::TelemetryCbor);
    if let (Some(json), Some(cbor)) = (json, cbor) {
        *cache = Some(Cache {
            sequence: json.sequence,
            json: json.payload.clone(),
            cbor: cbor.payload.clone(),
        });
    }
}

struct Tracks {
    telemetry: track::Producer,
    telemetry_cbor: track::Producer,
    health: track::Producer,
    image: track::Producer,
    audio: track::Producer,
    video: track::Producer,
}

impl Tracks {
    fn open(
        origin: &moq_net::origin::Producer,
        broadcast_name: &str,
    ) -> Result<(moq_net::broadcast::Producer, Self)> {
        let broadcast = origin.create_broadcast(broadcast_name)?;
        let open = |track: FeedTrack| {
            let info = track::Info::default()
                .with_priority(track.priority())
                .with_max_age(Duration::from_millis(track.publisher_max_age_ms()));
            broadcast.create_track(track.name(), Some(info))
        };
        let tracks = Self {
            telemetry: open(FeedTrack::Telemetry)?,
            telemetry_cbor: open(FeedTrack::TelemetryCbor)?,
            health: open(FeedTrack::Health)?,
            image: open(FeedTrack::Image)?,
            audio: open(FeedTrack::Audio)?,
            video: open(FeedTrack::Video)?,
        };
        broadcast.announce(Default::default())?;
        Ok((broadcast, tracks))
    }

    fn get(&self, track: FeedTrack) -> &track::Producer {
        match track {
            FeedTrack::Telemetry => &self.telemetry,
            FeedTrack::TelemetryCbor => &self.telemetry_cbor,
            FeedTrack::Health => &self.health,
            FeedTrack::Image => &self.image,
            FeedTrack::Audio => &self.audio,
            FeedTrack::Video => &self.video,
        }
    }
}

fn write_piece(tracks: &Tracks, piece: &Piece) -> Result<()> {
    let mut group = tracks.get(piece.track).append_group()?;
    group.write_frame(Timestamp::now(), piece.payload.clone())?;
    group.finish()?;
    Ok(())
}

fn subscription(track: FeedTrack) -> Subscription {
    // max_age 0 asks for the newest group only, so a slow reader skips a backlog
    // and a late reader still gets the current snapshot.
    Subscription::default()
        .with_priority(track.priority())
        .with_max_age(Duration::ZERO)
}

/// Run until Ctrl-C. Virtual loss stays inside this process.
pub(crate) async fn serve(args: FeedArgs) -> Result<()> {
    let link = args.link.to_link();
    let interval = Duration::from_millis(args.interval_ms);
    let service = args.service.service();
    let station = station_id(service, args.station_id.as_deref())?;
    let session = u16::try_from(unix_ms()? % 65_536).unwrap_or(1).max(1);
    let gateway = Gateway::new(
        AirConfig {
            service,
            station_id: station,
            session,
            parity: args.parity,
            max_shard: 0,
            interleave_depth: 1,
        },
        ChannelConfig {
            relays: args.relays,
            loss_per_mille: args.loss_per_mille,
            burst: args.burst,
            reorder: 0,
            seed: args.seed,
        },
        match args.budget_bps {
            Some(bps) => Budget::from_bps(bps, interval),
            None => Budget::unlimited(),
        },
    )?;
    let pictures = if args.media {
        Some(test_pictures()?)
    } else {
        None
    };
    let (tx, rx) = mpsc::channel(8);
    let (shutdown, stop) = watch::channel(false);
    let sampling = tokio::spawn(sample_loop(args, pictures, tx, stop));
    tokio::select! {
        result = publish_feed(link, gateway, rx) => result?,
        _ = tokio::signal::ctrl_c() => {
            let _ = shutdown.send(true);
        }
    }
    let _ = sampling.await;
    Ok(())
}

async fn sample_loop(
    args: FeedArgs,
    pictures: Option<[Vec<u8>; 4]>,
    tx: mpsc::Sender<Tick>,
    mut stop: watch::Receiver<bool>,
) {
    let mut tick = tokio::time::interval(Duration::from_millis(args.interval_ms));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut sequence = 0_u64;
    let mut paused = false;
    loop {
        tokio::select! {
            _ = tick.tick() => {}
            _ = stop.changed() => return,
        }
        if *stop.borrow() {
            return;
        }
        sequence += 1;
        let Ok(now) = unix_ms() else {
            eprintln!("System clock unavailable; not publishing.");
            continue;
        };
        let telemetry = match read_sample(&args, sequence, now) {
            Ok(sample) => {
                if paused {
                    eprintln!("Sensor inputs recovered.");
                }
                paused = false;
                sample
            }
            Err(err) => {
                if !paused {
                    eprintln!("Publishing paused due to sensor input error: {err:#}");
                }
                paused = true;
                continue;
            }
        };
        let media = media_for(sequence, now, pictures.as_ref());
        if tx
            .send(Tick {
                telemetry,
                image: media.image,
                audio: media.audio,
                video: media.video,
            })
            .await
            .is_err()
        {
            return;
        }
    }
}

struct OptionalMedia {
    image: Option<Vec<u8>>,
    audio: Option<Vec<u8>>,
    video: Option<Vec<u8>>,
}

fn media_for(sequence: u64, now: u64, pictures: Option<&[Vec<u8>; 4]>) -> OptionalMedia {
    let Some(pictures) = pictures else {
        return OptionalMedia {
            image: None,
            audio: None,
            video: None,
        };
    };
    OptionalMedia {
        image: sequence.is_multiple_of(4).then(|| pictures[0].clone()),
        audio: AudioFrame::tone(u32::try_from(sequence).unwrap_or(u32::MAX), now, 80)
            .encode()
            .ok(),
        video: sequence
            .is_multiple_of(2)
            .then(|| pictures[(sequence as usize / 2) % pictures.len()].clone()),
    }
}

fn read_sample(args: &FeedArgs, sequence: u64, now: u64) -> Result<Telemetry> {
    let service = args.service.service();
    let station = args
        .station_id
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let (Some(temperature), Some(voltage), Some(light)) = (
        args.temperature_mc.as_ref(),
        args.voltage_mv.as_ref(),
        args.light_lux.as_ref(),
    ) else {
        if args.temperature_mc.is_some() || args.voltage_mv.is_some() || args.light_lux.is_some() {
            bail!("pass --temperature-mc, --voltage-mv, and --light-lux together");
        }
        return Ok(Telemetry::synthetic(sequence, now, service, station));
    };
    // Each value is stamped with its file's last-modified time, never with "now": a file left
    // behind from an old run must not be published as a fresh reading.
    let (temperature_mc, t_at) = read_fresh_integer::<i32>(temperature, now)?;
    let (voltage_mv, v_at) = read_fresh_integer::<u32>(voltage, now)?;
    let (light_lux, l_at) = read_fresh_integer(light, now)?;
    let pressure = match &args.pressure_pa {
        Some(path) => match read_fresh_integer::<u32>(path, now) {
            Ok((value, _)) => Some(value),
            Err(err) => {
                eprintln!("Pressure file skipped: {err:#}");
                None
            }
        },
        None => None,
    };
    // The reading is only as fresh as its oldest required value.
    let captured = t_at.min(v_at).min(l_at);
    encodable(Telemetry::from_sensors(
        sequence,
        captured,
        service,
        station,
        downlink::SensorReading {
            temperature_c: f64::from(temperature_mc) / 1000.0,
            bus_voltage_v: f64::from(voltage_mv) / 1000.0,
            light_lux,
            pressure_pa: pressure,
        },
    ))
}

/// Refuse a reading the air frame can't carry (for example 65536 mV, or a temperature past
/// ±327.67 °C). The sample loop treats this like any other bad sensor input: it pauses
/// publishing and resumes when a valid reading comes back, instead of the gateway failing.
fn encodable(telemetry: Telemetry) -> Result<Telemetry> {
    telemetry
        .to_board_body()
        .context("sensor reading is outside the range the air frame can carry")?;
    Ok(telemetry)
}

fn is_live_pseudo_file(path: &std::path::Path) -> bool {
    path.starts_with("/sys") || path.starts_with("/proc")
}

/// Read an integer sensor file and return it with its capture time: the file's last-modified
/// time in Unix milliseconds. Fails when the file is older than the station's freshness limit,
/// or dated further in the future than the allowed clock skew.
pub(crate) fn read_fresh_integer<T: std::str::FromStr>(
    path: &std::path::Path,
    now: u64,
) -> Result<(T, u64)>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    // Kernel pseudo-files (hwmon, IIO) produce a new value on every read, but their
    // modification time never changes, so the read itself is the capture time.
    if is_live_pseudo_file(path) {
        return Ok((super::read_integer(path)?, now));
    }
    // One open file for both the time and the value. A writer that replaces the file
    // atomically (write a temp file, then rename) swaps the directory entry, but this handle
    // stays on the file it opened, so the value can never be paired with another write's time.
    let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    fresh_integer_from(file, path, now)
}

/// The value and modification time of one open sensor file, with the freshness checks.
fn fresh_integer_from<T: std::str::FromStr>(
    mut file: std::fs::File,
    path: &std::path::Path,
    now: u64,
) -> Result<(T, u64)>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let modified = file
        .metadata()
        .and_then(|meta| meta.modified())
        .with_context(|| format!("read the modification time of {}", path.display()))?;
    let at = u64::try_from(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .context("sensor file is dated before 1970")?
            .as_millis(),
    )?;
    if let Some(age) = now.checked_sub(at) {
        if u128::from(age) > super::MAX_AGE.as_millis() {
            bail!(
                "{} is stale: last written {age} ms ago (limit {} ms)",
                path.display(),
                super::MAX_AGE.as_millis()
            );
        }
    } else if u128::from(at - now) > super::MAX_CLOCK_SKEW.as_millis() {
        bail!(
            "{} is dated {} ms in the future; check the clock",
            path.display(),
            at - now
        );
    }
    let mut text = String::new();
    std::io::Read::read_to_string(&mut file, &mut text)
        .with_context(|| format!("read {}", path.display()))?;
    let value = text
        .trim()
        .parse()
        .with_context(|| format!("parse integer from {}", path.display()))?;
    Ok((value, at.min(now)))
}

async fn publish_feed(
    link: Link,
    mut gateway: Gateway,
    mut ticks: mpsc::Receiver<Tick>,
) -> Result<()> {
    let origin = moq_tokio::origin::spawn();
    let (_broadcast, tracks) = Tracks::open(&origin, &link.broadcast)?;
    let client = connect(&link)?.with_publisher(&origin);
    let mut conn = client.connect(link.relay.clone());
    let mut cache: Option<Cache> = None;
    let mut published_epoch = 0_u64;
    loop {
        tokio::select! {
            status = conn.status() => {
                match status.context("relay connection stopped")? {
                    moq_tokio::Status::Connected => {
                        let epoch = conn.epoch();
                        if epoch != published_epoch {
                            if let Some(saved) = &cache {
                                for piece in catch_up(saved) {
                                    if let Err(err) = write_piece(&tracks, &piece) {
                                        eprintln!("Could not republish {}: {err:#}", piece.track.name());
                                    }
                                }
                            }
                            published_epoch = epoch;
                        }
                    }
                    moq_tokio::Status::Disconnected | moq_tokio::Status::Migrating => {}
                    _ => {}
                }
            }
            tick = ticks.recv() => {
                let Some(tick) = tick else { return Ok(()) };
                let pieces = match gateway.ingest(&tick) {
                    Ok(pieces) => pieces,
                    Err(err) => {
                        eprintln!("Skipped snapshot {}: {err:#}", tick.telemetry.sequence);
                        continue;
                    }
                };
                if !pieces.iter().any(|piece| piece.track == FeedTrack::Telemetry) {
                    eprintln!("Downlink missed a telemetry snapshot on the virtual channel.");
                }
                for piece in &pieces {
                    if let Err(err) = write_piece(&tracks, piece) {
                        eprintln!("Could not publish {}: {err:#}", piece.track.name());
                    }
                }
                remember(&mut cache, &pieces);
                published_epoch = conn.epoch();
            }
        }
    }
}

/// Read one object from one track and print it.
pub(crate) async fn read(args: ReadArgs) -> Result<()> {
    let track = parse_track(&args.track)?;
    let bytes = read_track(&args.link.to_link(), track).await?;
    match track {
        FeedTrack::Telemetry => print_telemetry(&bytes, false, args.reading, args.json)?,
        FeedTrack::TelemetryCbor => print_telemetry(&bytes, true, args.reading, args.json)?,
        FeedTrack::Health => {
            let health = RelayHealth::from_json(&bytes)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&health)?);
            } else {
                println!(
                    "Relay health. Evidence: {}. Recovered {}, lost {}, repaired {}, duplicate copies {}. {} receiver row(s).",
                    health.evidence,
                    health.recovered,
                    health.lost,
                    health.fec_repairs,
                    health.duplicates,
                    health.relays.len()
                );
            }
        }
        FeedTrack::Image | FeedTrack::Video => {
            let jpeg = bytes.starts_with(&[0xff, 0xd8]);
            println!(
                "{} track: {} bytes, {}. Evidence: desk-simulation. This is not a camera measurement.",
                track.name(),
                bytes.len(),
                if jpeg { "JPEG" } else { "not a JPEG" }
            );
        }
        FeedTrack::Audio => {
            let parsed = AudioFrame::decode(&bytes)?;
            println!(
                "audio track: {} PCM bytes, {} Hz, mono, sample #{}. Evidence: desk-simulation.",
                parsed.pcm_s16le.len(),
                parsed.sample_rate_hz,
                parsed.sequence
            );
        }
    }
    Ok(())
}

fn print_telemetry(bytes: &[u8], cbor: bool, reading: Reading, json: bool) -> Result<()> {
    let sample = if cbor {
        Telemetry::from_cbor(bytes)?
    } else {
        Telemetry::from_json(bytes)?
    };
    // Check freshness before printing anything, JSON included: a stale or future-dated
    // reading is an error, never output that looks current.
    let now = unix_ms()?;
    let age = sample.age_ms(now)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&sample)?);
        return Ok(());
    }
    let detail = match reading {
        Reading::Temperature => format!("{:.1} °C payload temperature", sample.temperature_c),
        Reading::Voltage => format!("{:.3} V bus voltage", sample.bus_voltage_v),
        Reading::Light => format!("{} lx light level", sample.light_lux),
        Reading::Status => format!(
            "{:.1} °C payload temperature, {:.3} V bus voltage, {} lx light level",
            sample.temperature_c, sample.bus_voltage_v, sample.light_lux
        ),
    };
    let station = sample
        .station_id
        .as_deref()
        .map(|id| format!(" Station ID {id}."))
        .unwrap_or_default();
    let pressure = sample
        .pressure_pa
        .map(|pa| format!(", {pa} Pa"))
        .unwrap_or_default();
    println!(
        "{detail}{pressure}. Source {} · sample #{} · captured {age} ms ago. Evidence: {} · service {} · {} Hz.{station}",
        sample.source,
        sample.sequence,
        sample.evidence,
        service_name(sample.service),
        sample.frequency_hz
    );
    Ok(())
}

fn service_name(service: Service) -> &'static str {
    match service {
        Service::Simulation => "simulation",
        Service::Part15 => "part15",
        Service::Part97 => "part97",
    }
}

fn parse_track(name: &str) -> Result<FeedTrack> {
    FeedTrack::all()
        .into_iter()
        .find(|track| track.name() == name)
        .with_context(|| format!("unknown track `{name}`"))
}

async fn read_track(link: &Link, track: FeedTrack) -> Result<Vec<u8>> {
    super::with_broadcast(link, async |broadcast| {
        let mut sub = broadcast
            .track(track.name())?
            .subscribe(Some(subscription(track)))
            .await?;
        let mut group = tokio::time::timeout(super::DEADLINE, sub.recv_group())
            .await
            .context("no group within 10 seconds")??
            .context("track ended")?;
        let frame = tokio::time::timeout(super::DEADLINE, group.read_frame())
            .await
            .context("no object within 10 seconds")??
            .context("group contained no object")?;
        ensure_bounded(&frame.payload, 64 * 1024)?;
        Ok(frame.payload.to_vec())
    })
    .await
}

fn ensure_bounded(bytes: &[u8], limit: usize) -> Result<()> {
    if bytes.is_empty() || bytes.len() > limit {
        bail!("object is empty or larger than {limit} bytes");
    }
    Ok(())
}

fn test_pictures() -> Result<[Vec<u8>; 4]> {
    let mut pictures = Vec::new();
    for mark in [40_u8, 90, 140, 200] {
        let image = image::RgbImage::from_fn(32, 24, |x, y| {
            image::Rgb([
                mark,
                u8::try_from(x.saturating_mul(8)).unwrap_or(255),
                u8::try_from(y.saturating_mul(10)).unwrap_or(255),
            ])
        });
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 50).encode_image(&image)?;
        pictures.push(jpeg);
    }
    Ok([
        pictures.remove(0),
        pictures.remove(0),
        pictures.remove(0),
        pictures.remove(0),
    ])
}

fn station_id(service: Service, text: Option<&str>) -> Result<Option<StationId>> {
    match text.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => Ok(Some(StationId::parse(text)?)),
        None if service.requires_station_id() => bail!(
            "amateur mode (Part 97) needs the control operator's station ID on every frame. This program does not transmit."
        ),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sensor_file(name: &str, value: &str, age: Duration, ahead: bool) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("cubesat-live-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, value).unwrap();
        let now = std::time::SystemTime::now();
        let when = if ahead { now + age } else { now - age };
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(when)
            .unwrap();
        path
    }

    #[test]
    fn an_atomic_replace_mid_read_never_mixes_value_and_time() {
        // Review finding: the time came from one lookup and the value from a second open, so a
        // rename in between paired the new value with the old time. Both now come from one
        // open file: here the file is replaced after it is opened, and the old handle still
        // gives the old value with the old time.
        let path = sensor_file("swap", "1111\n", Duration::from_millis(1500), false);
        let file = std::fs::File::open(&path).unwrap();
        let old_at = unix_ms().unwrap() - 1500;
        let next = path.with_extension("next");
        std::fs::write(&next, "2222\n").unwrap();
        std::fs::rename(&next, &path).unwrap();
        let now = unix_ms().unwrap();
        let (value, at): (u32, u64) = fresh_integer_from(file, &path, now).unwrap();
        assert_eq!(value, 1111);
        assert!(
            at.abs_diff(old_at) < 250,
            "time {at} should be the old write, near {old_at}"
        );
        let (value, at): (u32, u64) = read_fresh_integer(&path, now).unwrap();
        assert_eq!(value, 2222);
        assert!(at > old_at + 1000, "the new file has the new time");
    }

    #[test]
    fn fresh_sensor_file_is_stamped_with_its_write_time() {
        let path = sensor_file("fresh", "27410\n", Duration::from_millis(1500), false);
        let now = unix_ms().unwrap();
        let (value, at) = read_fresh_integer::<i32>(&path, now).unwrap();
        assert_eq!(value, 27410);
        let age = now - at;
        assert!(
            (1300..=2500).contains(&age),
            "stamped {age} ms ago, not now"
        );
    }

    #[test]
    fn stale_sensor_file_is_refused() {
        let path = sensor_file("stale", "3300", Duration::from_secs(3600), false);
        let err = read_fresh_integer::<u32>(&path, unix_ms().unwrap()).unwrap_err();
        assert!(format!("{err:#}").contains("stale"), "{err:#}");
    }

    #[test]
    fn future_dated_sensor_file_is_refused() {
        let path = sensor_file("future", "3300", Duration::from_secs(60), true);
        let err = read_fresh_integer::<u32>(&path, unix_ms().unwrap()).unwrap_err();
        assert!(format!("{err:#}").contains("future"), "{err:#}");
        // Within the allowed skew, it counts as just written.
        let near = sensor_file("near", "3300", Duration::from_millis(500), true);
        let now = unix_ms().unwrap();
        assert_eq!(read_fresh_integer::<u32>(&near, now).unwrap().1, now);
    }

    #[test]
    fn kernel_pseudo_files_are_read_live() {
        assert!(is_live_pseudo_file(std::path::Path::new(
            "/sys/class/hwmon/hwmon0/temp1_input"
        )));
        assert!(is_live_pseudo_file(std::path::Path::new("/proc/loadavg")));
        assert!(!is_live_pseudo_file(std::path::Path::new(
            "/var/lib/cubesat/temp_mc"
        )));
        assert!(!is_live_pseudo_file(std::path::Path::new("/system/temp")));
    }

    #[test]
    fn read_live_json_refuses_stale_telemetry() {
        let old = unix_ms().unwrap() - 60_000;
        let stale = Telemetry::synthetic(1, old, Service::Simulation, None)
            .to_json()
            .unwrap();
        assert!(print_telemetry(&stale, false, Reading::Status, true).is_err());
        let fresh = Telemetry::synthetic(2, unix_ms().unwrap(), Service::Simulation, None)
            .to_json()
            .unwrap();
        assert!(print_telemetry(&fresh, false, Reading::Status, true).is_ok());
    }

    fn gateway(parity: u8, loss: u16, budget: Budget) -> Gateway {
        Gateway::new(
            AirConfig {
                parity,
                session: 1,
                ..AirConfig::default()
            },
            ChannelConfig {
                relays: 1,
                loss_per_mille: loss,
                burst: 1,
                reorder: 0,
                seed: 0x0915_0001,
            },
            budget,
        )
        .unwrap()
    }

    fn tick(sequence: u64, image: Option<Vec<u8>>) -> Tick {
        Tick {
            telemetry: Telemetry::synthetic(sequence, 1_700_000_000_000, Service::Simulation, None),
            image,
            audio: None,
            video: None,
        }
    }

    #[test]
    fn local_snapshots_keep_their_millisecond_capture_time() {
        // Review finding: a 4.56 s old reading came out 5.54 s old after the whole-second air
        // frame, past read-live's limit. The published snapshot keeps the exact time.
        let mut gateway = gateway(0, 0, Budget::unlimited());
        let mut sample = tick(1, None);
        sample.telemetry.captured_unix_ms = 1_700_000_004_560;
        let pieces = gateway.ingest(&sample).unwrap();
        let json = pieces
            .iter()
            .find(|piece| piece.track == FeedTrack::Telemetry)
            .expect("telemetry published");
        let snapshot = Telemetry::from_json(&json.payload).unwrap();
        assert_eq!(snapshot.captured_unix_ms, 1_700_000_004_560);
    }

    #[test]
    fn readings_the_air_frame_cannot_carry_are_refused_before_the_gateway() {
        let reading = |mv: u32| {
            Telemetry::from_sensors(
                1,
                1_700_000_000_000,
                Service::Simulation,
                None,
                downlink::SensorReading {
                    temperature_c: 21.0,
                    bus_voltage_v: f64::from(mv) / 1000.0,
                    light_lux: 100,
                    pressure_pa: None,
                },
            )
        };
        let err = encodable(reading(65_536)).unwrap_err();
        assert!(format!("{err:#}").contains("outside the range"), "{err:#}");
        assert!(encodable(reading(65_535)).is_ok());
    }

    #[test]
    fn reconnect_repeats_telemetry_and_not_media() {
        let pieces = catch_up(&Cache {
            sequence: 4,
            json: br#"{"schema":"cubesat.telemetry.v1"}"#.to_vec(),
            cbor: vec![1, 2, 3],
        });
        assert_eq!(pieces[0].track, FeedTrack::Telemetry);
        assert_eq!(pieces[1].track, FeedTrack::TelemetryCbor);
        assert!(pieces.iter().all(|piece| !matches!(
            piece.track,
            FeedTrack::Image | FeedTrack::Audio | FeedTrack::Video
        )));
    }

    #[test]
    fn amateur_mode_requires_a_station_id() {
        let err = Gateway::new(
            AirConfig {
                service: Service::Part97,
                ..AirConfig::default()
            },
            ChannelConfig {
                relays: 1,
                loss_per_mille: 0,
                burst: 1,
                reorder: 0,
                seed: 1,
            },
            Budget::unlimited(),
        );
        let message = err.err().map(|err| err.to_string()).unwrap_or_default();
        assert!(message.contains("station ID"), "{message}");
    }

    #[test]
    fn parity_beats_an_unprotected_virtual_channel() {
        let mut plain = gateway(0, 100, Budget::unlimited());
        let mut coded = gateway(1, 100, Budget::unlimited());
        let mut plain_got = 0;
        let mut coded_got = 0;
        for sequence in 1..=40 {
            if plain
                .ingest(&tick(sequence, None))
                .unwrap()
                .iter()
                .any(|piece| piece.track == FeedTrack::Telemetry)
            {
                plain_got += 1;
            }
            if coded
                .ingest(&tick(sequence, None))
                .unwrap()
                .iter()
                .any(|piece| piece.track == FeedTrack::Telemetry)
            {
                coded_got += 1;
            }
        }
        assert!(
            coded_got > plain_got,
            "parity {coded_got}, plain {plain_got}"
        );
        assert!(coded_got >= 36, "{coded_got}");
    }

    #[test]
    fn a_tight_budget_publishes_telemetry_and_not_the_image() {
        let mut gateway = gateway(0, 0, Budget::from_bps(2_000, Duration::from_secs(1)));
        let pieces = gateway.ingest(&tick(1, Some(vec![7; 3_000]))).unwrap();
        assert!(
            pieces
                .iter()
                .any(|piece| piece.track == FeedTrack::Telemetry)
        );
        assert!(pieces.iter().all(|piece| piece.track != FeedTrack::Image));
    }

    fn install_crypto() {
        let _ = moq_tokio::crypto::install_default();
    }

    async fn relay_link() -> (super::super::LocalRelay, Link) {
        install_crypto();
        let relay = super::super::start_relay(0).await.unwrap();
        let link = Link {
            relay: relay.url.clone(),
            fingerprint: Some(relay.pin.clone()),
            broadcast: LIVE_BROADCAST.into(),
        };
        (relay, link)
    }

    async fn collect(link: &Link, count: usize) -> Result<Vec<u64>> {
        super::super::with_broadcast(link, async |broadcast| {
            let mut sub = broadcast
                .track(FeedTrack::Telemetry.name())?
                .subscribe(Some(subscription(FeedTrack::Telemetry)))
                .await?;
            let mut sequences = Vec::new();
            while sequences.len() < count {
                let Some(mut group) =
                    tokio::time::timeout(Duration::from_secs(8), sub.recv_group())
                        .await
                        .context("timed out waiting for telemetry")??
                else {
                    break;
                };
                if let Some(frame) = group.read_frame().await? {
                    sequences.push(Telemetry::from_json(&frame.payload)?.sequence);
                }
            }
            Ok(sequences)
        })
        .await
    }

    #[tokio::test]
    async fn four_subscribers_see_the_snapshot_without_the_image_track() {
        let (relay, link) = relay_link().await;
        let (tx, rx) = mpsc::channel(4);
        let publisher = tokio::spawn(publish_feed(
            link.clone(),
            gateway(0, 0, Budget::unlimited()),
            rx,
        ));
        let mut readers = Vec::new();
        for _ in 0..4 {
            let link = link.clone();
            readers.push(tokio::spawn(async move { collect(&link, 3).await }));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        for sequence in 1..=3 {
            tx.send(tick(sequence, Some(b"\xff\xd8not-a-real-photo".to_vec())))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(40)).await;
        }
        for reader in readers {
            let sequences = tokio::time::timeout(Duration::from_secs(10), reader)
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(sequences.contains(&3), "{sequences:?}");
        }
        drop(tx);
        let _ = publisher.await;
        drop(relay);
    }

    #[tokio::test]
    async fn a_late_subscriber_receives_the_newest_snapshot() {
        let (relay, link) = relay_link().await;
        let (tx, rx) = mpsc::channel(4);
        let publisher = tokio::spawn(publish_feed(
            link.clone(),
            gateway(0, 0, Budget::unlimited()),
            rx,
        ));
        tokio::time::sleep(Duration::from_millis(400)).await;
        for sequence in 1..=3 {
            tx.send(tick(sequence, None)).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        let late = collect(&link, 1).await.unwrap();
        assert_eq!(late, vec![3], "late joiner saw {late:?}");
        drop(tx);
        let _ = publisher.await;
        drop(relay);
    }

    #[tokio::test]
    async fn scarce_bandwidth_is_reserved_for_telemetry() {
        install_crypto();
        let bandwidth = moq_net::bandwidth::Producer::new();
        bandwidth
            .set(Some(moq_net::bandwidth::Rate::from_bps(16_000)))
            .unwrap();
        let allocator = moq_net::bandwidth::Allocator::new(bandwidth.consume());
        let broadcast = moq_net::broadcast::Info::default().produce();
        let telemetry = broadcast
            .create_track(
                FeedTrack::Telemetry.name(),
                Some(track::Info::default().with_priority(FeedTrack::Telemetry.priority())),
            )
            .unwrap();
        let video = broadcast
            .create_track(
                FeedTrack::Video.name(),
                Some(track::Info::default().with_priority(FeedTrack::Video.priority())),
            )
            .unwrap();
        let _telemetry_sub = telemetry.subscribe(Some(subscription(FeedTrack::Telemetry)));
        let _video_sub = video.subscribe(Some(subscription(FeedTrack::Video)));
        let telemetry_share = allocator.reserve(
            &telemetry.demand(),
            moq_net::bandwidth::Rate::from_bps(16_000),
        );
        let video_share =
            allocator.reserve(&video.demand(), moq_net::bandwidth::Rate::from_bps(500_000));
        assert_eq!(
            telemetry_share.peek().map(|rate| rate.as_bps()),
            Some(16_000)
        );
        assert_eq!(video_share.peek().map(|rate| rate.as_bps()), Some(0));
    }
}
