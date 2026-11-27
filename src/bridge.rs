//! Sensor bridge: turn real hardware readings into the sensor files the station reads.
//!
//! Two sources, one output. A microcontroller (Arduino, Seeed XIAO, Pico) prints one
//! line per reading over USB serial in the `CSC1` format below, or a Raspberry Pi
//! exposes readings as Linux files (thermal zones, hwmon, IIO). The bridge validates
//! each value, writes it atomically to `temperature_mc`, `voltage_mv`, `light_lux`
//! or `pressure_pa` in an output folder, and deletes a file when its value goes
//! stale, so the station reports the sensor as unavailable instead of showing old
//! data as new.
//!
//! Line format (one reading per line, fields in any order, unknown fields ignored):
//!
//! ```text
//! CSC1 temp_mc=21500 volt_mv=3912 light_lux=460 pressure_pa=101325
//! ```
//!
//! With `--rec-log FILE`, the bridge is also the receiver for stored readings from the Rust
//! Nano firmware: it appends each new `REC` line to the file, flushes it to disk, and only
//! then writes `RECACK <seq>` back to the serial device, so the CubeSat deletes a reading
//! only once the station has kept it. A `REC` line seen before (the same text: a reading sent
//! again) is acknowledged again but not saved twice. Whole lines are compared, not `seq`
//! alone, because `seq` starts again from zero when the CubeSat restarts.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};

/// One measurement the station understands: protocol key, output file, valid range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Field {
    pub key: &'static str,
    pub file: &'static str,
    pub min: i64,
    pub max: i64,
}

pub(crate) const FIELDS: [Field; 4] = [
    Field {
        key: "temp_mc",
        file: "temperature_mc",
        min: -80_000,
        max: 150_000,
    },
    Field {
        key: "volt_mv",
        file: "voltage_mv",
        min: 0,
        max: 60_000,
    },
    Field {
        key: "light_lux",
        file: "light_lux",
        min: 0,
        max: 200_000,
    },
    Field {
        key: "pressure_pa",
        file: "pressure_pa",
        min: 1_000,
        max: 120_000,
    },
];

fn field(key: &str) -> Option<Field> {
    FIELDS
        .iter()
        .copied()
        .find(|f| f.key == key || f.file == key)
}

/// Parse one `CSC1` line into validated values. Lines without the prefix are
/// ignored (boot messages, debug prints). Out-of-range or malformed fields are
/// dropped individually and reported, so one bad sensor does not hide the others.
pub(crate) fn parse_line(line: &str) -> (Vec<(Field, i64)>, Vec<String>) {
    let mut values = Vec::new();
    let mut problems = Vec::new();
    let mut parts = line.split_whitespace();
    if parts.next() != Some("CSC1") {
        return (values, problems);
    }
    for part in parts {
        let Some((key, raw)) = part.split_once('=') else {
            problems.push(format!("ignored `{part}`: expected key=value"));
            continue;
        };
        let Some(f) = FIELDS.iter().copied().find(|f| f.key == key) else {
            continue; // forward compatible: unknown keys are fine
        };
        match raw.parse::<i64>() {
            Ok(v) if (f.min..=f.max).contains(&v) => values.push((f, v)),
            Ok(v) => problems.push(format!("{key}={v} is outside {}..={}", f.min, f.max)),
            Err(_) => problems.push(format!("{key}={raw} is not an integer")),
        }
    }
    (values, problems)
}

/// `name=path[*scale]`: read a number from `path`, multiply by `scale`, round.
/// Example for a BME280 on a Pi: `pressure_pa=/sys/bus/iio/devices/iio:device0/in_pressure_input*1000`
/// (the IIO driver reports kilopascals).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mapping {
    pub field: Field,
    pub path: PathBuf,
    pub scale: f64,
}

pub(crate) fn parse_mapping(spec: &str) -> Result<Mapping> {
    let (name, rest) = spec.split_once('=').context("expected name=path[*scale]")?;
    let field = field(name).with_context(|| {
        let names: Vec<_> = FIELDS.iter().map(|f| f.key).collect();
        format!("unknown sensor `{name}`; use one of {}", names.join(", "))
    })?;
    let (path, scale) = match rest.rsplit_once('*') {
        Some((path, scale)) => (
            path,
            scale
                .parse::<f64>()
                .with_context(|| format!("bad scale `{scale}`"))?,
        ),
        None => (rest, 1.0),
    };
    ensure!(!path.is_empty(), "missing path for {name}");
    ensure!(
        scale.is_finite() && scale != 0.0,
        "scale must be a non-zero number"
    );
    Ok(Mapping {
        field,
        path: PathBuf::from(path),
        scale,
    })
}

fn read_mapping(m: &Mapping) -> Result<i64> {
    let text = fs::read_to_string(&m.path).with_context(|| format!("read {}", m.path.display()))?;
    let raw: f64 = text
        .trim()
        .parse()
        .with_context(|| format!("parse number from {}", m.path.display()))?;
    let value = (raw * m.scale).round();
    ensure!(
        value.is_finite(),
        "{} gave a non-finite value",
        m.path.display()
    );
    let value = value as i64;
    ensure!(
        (m.field.min..=m.field.max).contains(&value),
        "{}={value} is outside {}..={}",
        m.field.key,
        m.field.min,
        m.field.max
    );
    Ok(value)
}

/// Writes values atomically and removes them once they go stale.
pub(crate) struct Outbox {
    dir: PathBuf,
    stale_after: Duration,
    updated: HashMap<&'static str, Instant>,
}

impl Outbox {
    /// Creates the folder and removes any sensor files left by an earlier run,
    /// so the station never reads a value this bridge did not just write.
    pub fn new(dir: PathBuf, stale_after: Duration) -> Result<Self> {
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let outbox = Self {
            dir,
            stale_after,
            updated: HashMap::new(),
        };
        outbox.clear();
        Ok(outbox)
    }

    fn clear(&self) {
        for f in FIELDS {
            let _ = fs::remove_file(self.dir.join(f.file));
            let _ = fs::remove_file(self.dir.join(format!(".{}.tmp", f.file)));
        }
    }

    pub fn write(&mut self, field: Field, value: i64, now: Instant) -> Result<()> {
        let target = self.dir.join(field.file);
        let temp = self.dir.join(format!(".{}.tmp", field.file));
        // Create the temporary file fresh, never through an existing file or a planted
        // link, so a shared output folder cannot redirect the write elsewhere.
        let _ = fs::remove_file(&temp);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .with_context(|| format!("create {}", temp.display()))?;
        file.write_all(format!("{value}\n").as_bytes())
            .with_context(|| format!("write {}", temp.display()))?;
        drop(file);
        fs::rename(&temp, &target).with_context(|| format!("replace {}", target.display()))?;
        self.updated.insert(field.file, now);
        Ok(())
    }

    /// Remove files whose last value is older than the stale limit. Returns their names.
    pub fn expire(&mut self, now: Instant) -> Vec<&'static str> {
        let stale: Vec<_> = self
            .updated
            .iter()
            .filter(|(_, at)| now.duration_since(**at) > self.stale_after)
            .map(|(file, _)| *file)
            .collect();
        for file in &stale {
            let _ = fs::remove_file(self.dir.join(file));
            self.updated.remove(file);
        }
        stale
    }
}

impl Drop for Outbox {
    /// When the bridge stops for any reason, its values stop being live, so remove them.
    fn drop(&mut self) {
        self.clear();
    }
}

#[derive(clap::Args)]
pub(crate) struct BridgeArgs {
    /// Folder to write temperature_mc, voltage_mv, light_lux and pressure_pa into.
    #[arg(long, default_value = "run/sensors")]
    pub out: PathBuf,
    /// Serial device, file or `-` (standard input) carrying `CSC1 key=value` lines.
    #[arg(long)]
    pub input: Option<PathBuf>,
    /// Read a Linux file as a sensor: name=path[*scale]. Repeat for each sensor.
    #[arg(long = "map")]
    pub maps: Vec<String>,
    /// Remove a value if no new reading arrives within this many seconds.
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u64).range(1..=3600))]
    pub stale_secs: u64,
    /// Save stored readings (`REC` lines) to this file and acknowledge each one to the
    /// CubeSat once saved. Needs --input to be the serial device.
    #[arg(long)]
    pub rec_log: Option<PathBuf>,
    /// How often to read mapped files, in milliseconds.
    #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(100..=60_000))]
    pub poll_ms: u64,
}

pub(crate) fn run(args: BridgeArgs, stop: &AtomicBool) -> Result<()> {
    let maps = args
        .maps
        .iter()
        .map(|s| parse_mapping(s))
        .collect::<Result<Vec<_>>>()?;
    if args.input.is_none() && maps.is_empty() {
        bail!(
            "give --input (a serial device, a file or - for standard input) or at least one --map"
        );
    }
    let mut outbox = Outbox::new(args.out.clone(), Duration::from_secs(args.stale_secs))?;
    eprintln!(
        "Writing sensor files to {}. Press Ctrl+C to stop.",
        args.out.display()
    );

    let mut receiver = match (&args.rec_log, &args.input) {
        (None, _) => None,
        (Some(_), None) => bail!("--rec-log needs --input, the CubeSat's serial device"),
        (Some(_), Some(p)) if p == Path::new("-") => {
            bail!("--rec-log needs --input to be the serial device, not standard input")
        }
        (Some(log), Some(dev)) => Some(RecLog::open(log, dev)?),
    };
    let lines = args.input.map(spawn_reader).transpose()?;
    let poll = Duration::from_millis(args.poll_ms.max(100));
    let mut next_poll = Instant::now();
    let mut reported: HashSet<&'static str> = HashSet::new();

    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        if !maps.is_empty() && now >= next_poll {
            for m in &maps {
                match read_mapping(m) {
                    Ok(v) => {
                        write_or_warn(&mut outbox, m.field, v, now);
                        reported.remove(m.field.key);
                    }
                    Err(e) => {
                        if reported.insert(m.field.key) {
                            eprintln!("{} unavailable: {e:#}", m.field.key);
                        }
                    }
                }
            }
            next_poll = now + poll;
        }
        if let Some(rx) = &lines {
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(line)) => {
                    if let Some(r) = receiver.as_mut()
                        && let Err(e) = r.handle(&line)
                    {
                        eprintln!("Stored reading not acknowledged: {e:#}");
                    }
                    let (values, problems) = parse_line(&line);
                    for p in problems {
                        eprintln!("Skipped: {p}");
                    }
                    for (f, v) in values {
                        write_or_warn(&mut outbox, f, v, Instant::now());
                    }
                }
                Ok(Err(e)) => bail!("input closed: {e}"),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => bail!("input ended"),
            }
        } else {
            std::thread::sleep(Duration::from_millis(200));
        }
        for file in outbox.expire(Instant::now()) {
            eprintln!("{file} went stale and was removed until a new reading arrives.");
        }
    }
    eprintln!("Stopped. Sensor files removed.");
    Ok(())
}

/// The `seq` of a whole, well-formed `REC` line, if it is one.
///
/// The line must match what the firmware's `rec_line` writes, field for field and in order:
/// `seq` and `t_ms`, an optional `temp_mc`, the required `volt_mv`, then optional
/// `pressure_pa` and `light_clear`, each with a complete integer in range. Anything else (an
/// empty or cut-off value, a missing `volt_mv`, an unknown or repeated field, trailing text) is
/// not acknowledged, so the CubeSat keeps the reading and sends it again.
pub(crate) fn rec_seq(line: &str) -> Option<u32> {
    fn int<T: std::str::FromStr>(v: &str) -> Option<T> {
        let digits = v.strip_prefix('-').unwrap_or(v);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        v.parse().ok()
    }
    /// The next field's value if it is `key`, leaving the field in place otherwise.
    fn take<'a>(
        fields: &mut std::iter::Peekable<std::str::Split<'a, char>>,
        key: &str,
    ) -> Option<&'a str> {
        let (k, v) = fields.peek()?.split_once('=')?;
        if k != key {
            return None;
        }
        fields.next();
        Some(v)
    }
    let mut fields = line.strip_prefix("REC ")?.split(' ').peekable();
    let seq: u32 = int(take(&mut fields, "seq")?)?;
    int::<u32>(take(&mut fields, "t_ms")?)?;
    if let Some(v) = take(&mut fields, "temp_mc") {
        int::<i32>(v)?;
    }
    int::<i32>(take(&mut fields, "volt_mv")?)?;
    if let Some(v) = take(&mut fields, "pressure_pa") {
        int::<i32>(v)?;
    }
    if let Some(v) = take(&mut fields, "light_clear") {
        int::<u16>(v)?;
    }
    fields.next().is_none().then_some(seq)
}

/// Saves stored readings and acknowledges them to the CubeSat (see the module docs).
pub(crate) struct RecLog<L: Write, A: Write> {
    log: L,
    ack: A,
    saved: HashSet<String>,
    sync: fn(&mut L) -> io::Result<()>,
    append_failed: bool,
}

impl RecLog<fs::File, fs::File> {
    fn open(log: &Path, device: &Path) -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(log)
            .with_context(|| format!("open {}", log.display()))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .with_context(|| format!("read {}", log.display()))?;
        let end = contents
            .iter()
            .rposition(|&byte| byte == b'\n')
            .map_or(0, |i| i + 1);
        if end < contents.len() {
            // The last write did not finish. Keep complete lines and let the board retry.
            file.set_len(end as u64).with_context(|| {
                format!("remove the incomplete final line in {}", log.display())
            })?;
        }
        file.sync_data()
            .with_context(|| format!("sync {}", log.display()))?;
        let saved = contents[..end]
            .split(|&byte| byte == b'\n')
            .filter_map(|line| std::str::from_utf8(line).ok())
            .map(|line| line.trim_end_matches('\r'))
            .filter(|line| rec_seq(line).is_some())
            .map(str::to_string)
            .collect();
        let ack = fs::OpenOptions::new()
            .write(true)
            .open(device)
            .with_context(|| format!("open {} for writing acknowledgments", device.display()))?;
        Ok(Self {
            log: file,
            ack,
            saved,
            sync: |f| f.sync_data(),
            append_failed: false,
        })
    }
}

impl<L: Write, A: Write> RecLog<L, A> {
    /// Save a new `REC` line, then acknowledge it. Other lines are ignored.
    pub(crate) fn handle(&mut self, line: &str) -> Result<()> {
        let Some(seq) = rec_seq(line) else {
            return Ok(());
        };
        if !self.saved.contains(line) {
            ensure!(
                !self.append_failed,
                "restart the bridge before saving more stored readings"
            );
            // A failure can leave a partial line. Do not append to it until reopening the log.
            self.append_failed = true;
            writeln!(self.log, "{line}").context("save the stored reading")?;
            self.log.flush().context("save the stored reading")?;
            (self.sync)(&mut self.log).context("save the stored reading")?;
            self.append_failed = false;
            self.saved.insert(line.to_string());
        }
        write!(self.ack, "RECACK {seq}\r\n").context("send RECACK")?;
        self.ack.flush().context("send RECACK")?;
        Ok(())
    }
}

fn write_or_warn(outbox: &mut Outbox, field: Field, value: i64, now: Instant) {
    if let Err(e) = outbox.write(field, value, now) {
        eprintln!("Could not write {}: {e:#}", field.file);
    }
}

const MAX_LINE: usize = 1024;

fn spawn_reader(path: PathBuf) -> Result<mpsc::Receiver<io::Result<String>>> {
    let reader: Box<dyn BufRead + Send> = if path == Path::new("-") {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let file = fs::File::open(&path).with_context(|| {
            format!("open {} (for a serial port, set its speed first with stty; see docs/ground-station.md)", path.display())
        })?;
        Box::new(BufReader::new(file))
    };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut buf = Vec::new();
        loop {
            buf.clear();
            // Read raw bytes: boot messages, reset noise or a wrong baud rate produce
            // invalid UTF-8, which is skipped as a bad line instead of stopping the bridge.
            let result = (&mut reader)
                .take(MAX_LINE as u64)
                .read_until(b'\n', &mut buf);
            let msg = match result {
                Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "end of input")),
                Ok(_) if buf.len() >= MAX_LINE && buf.last() != Some(&b'\n') => {
                    // An over-long line is dropped whole: its cut-off end could turn
                    // `pressure_pa=101325` into a valid-looking 10132. Skip the rest in
                    // bounded chunks so a stream with no newlines cannot use up memory.
                    match skip_line(&mut reader) {
                        Ok(()) => {
                            eprintln!("Skipped: a line longer than {MAX_LINE} bytes");
                            continue;
                        }
                        Err(e) => Err(e),
                    }
                }
                Ok(_) if buf.last() != Some(&b'\n') => {
                    // The input ended partway through a line. Its last value may be cut
                    // short, so it is not passed on: a stored reading in it is sent again.
                    eprintln!("Skipped: an unfinished last line");
                    Err(io::Error::new(io::ErrorKind::UnexpectedEof, "end of input"))
                }
                Ok(_) => Ok(String::from_utf8_lossy(&buf)
                    .trim_end_matches(['\r', '\n'])
                    .to_string()),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => Err(e),
            };
            let end = msg.is_err();
            if tx.send(msg).is_err() || end {
                return;
            }
        }
    });
    Ok(rx)
}

/// Read and throw away bytes up to and including the next newline, holding at most
/// `MAX_LINE` bytes at a time. Reaching the end of input also ends the line.
fn skip_line(reader: &mut impl BufRead) -> io::Result<()> {
    let mut chunk = Vec::with_capacity(MAX_LINE);
    loop {
        chunk.clear();
        match reader.take(MAX_LINE as u64).read_until(b'\n', &mut chunk) {
            Ok(0) => return Ok(()),
            Ok(_) if chunk.last() == Some(&b'\n') => return Ok(()),
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_each_stored_reading_before_acknowledging_it() {
        let mut r = RecLog {
            log: Vec::new(),
            ack: Vec::new(),
            saved: HashSet::new(),
            sync: |_| Ok(()),
            append_failed: false,
        };
        r.handle("CSC1 temp_mc=21500").unwrap();
        r.handle("REC seq=7 t_ms=700 temp_mc=21000 volt_mv=3300")
            .unwrap();
        r.handle("REC seq=8 t_ms=800 temp_mc=2").unwrap(); // cut off: not kept, not acked
        r.handle("REC seq=9 t_ms=900 temp_mc=21000 volt_mv=")
            .unwrap(); // empty value
        r.handle("REC seq=7 t_ms=700 temp_mc=21000 volt_mv=3300")
            .unwrap(); // sent again
        r.handle("REC seq=7 t_ms=650 temp_mc=19000 volt_mv=3310")
            .unwrap(); // a new reading after a restart
        assert_eq!(
            String::from_utf8(r.log).unwrap(),
            "REC seq=7 t_ms=700 temp_mc=21000 volt_mv=3300\n\
             REC seq=7 t_ms=650 temp_mc=19000 volt_mv=3310\n"
        );
        assert_eq!(
            String::from_utf8(r.ack).unwrap(),
            "RECACK 7\r\nRECACK 7\r\nRECACK 7\r\n"
        );
    }

    #[test]
    fn only_complete_stored_readings_are_acknowledged() {
        for good in [
            "REC seq=7 t_ms=700 volt_mv=3300",
            "REC seq=7 t_ms=700 temp_mc=-1500 volt_mv=3300 pressure_pa=101325 light_clear=65535",
            "REC seq=4294967295 t_ms=0 volt_mv=0 light_clear=0",
        ] {
            assert!(rec_seq(good).is_some(), "{good}");
        }
        for bad in [
            "REC seq=7 t_ms=700 volt_mv=",
            "REC seq=7 t_ms=700 temp_mc= volt_mv=3300",
            "REC seq=7 t_ms=700 temp_mc=21000",
            "REC seq= t_ms=700 volt_mv=3300",
            "REC seq=7 volt_mv=3300",
            "REC seq=7 t_ms=700 volt_mv=33-0",
            "REC seq=7 t_ms=700 volt_mv=3300 pressure_pa=",
            "REC seq=7 t_ms=700 volt_mv=3300 light_clear=70000",
            "REC seq=7 t_ms=700 volt_mv=3300 volt_mv=3300",
            "REC seq=7 t_ms=700 volt_mv=3300 extra=1",
            "REC seq=7 t_ms=700 volt_mv=3300 ",
            "REC seq=7 t_ms=700 pressure_pa=1 volt_mv=3300",
            "REC seq=-7 t_ms=700 volt_mv=3300",
            "REC seq=+7 t_ms=700 volt_mv=3300",
        ] {
            assert_eq!(rec_seq(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_truncated_last_line_is_not_passed_on() {
        let dir = std::env::temp_dir().join(format!("bridge-trunc-{}", std::process::id()));
        fs::write(
            &dir,
            "REC seq=1 t_ms=100 volt_mv=3300\r\nREC seq=2 t_ms=200 volt_mv=33",
        )
        .unwrap();
        let rx = spawn_reader(dir.clone()).unwrap();
        assert_eq!(
            rx.recv().unwrap().unwrap(),
            "REC seq=1 t_ms=100 volt_mv=3300"
        );
        assert!(
            rx.recv().unwrap().is_err(),
            "the cut-off line ends the input instead"
        );
        fs::remove_file(dir).unwrap();
    }

    #[test]
    fn stored_log_recovers_from_an_unfinished_record() {
        let dir = std::env::temp_dir().join(format!("bridge-rec-recovery-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("records.log");
        let ack = dir.join("ack");
        let saved = "REC seq=7 t_ms=700 volt_mv=3300";
        let retry = "REC seq=8 t_ms=800 volt_mv=3290";
        // The torn line looks valid on its own. Only the missing newline exposes it.
        fs::write(&log, format!("{saved}\nREC seq=8 t_ms=800 volt_mv=32")).unwrap();
        fs::write(&ack, "").unwrap();

        let mut receiver = RecLog::open(&log, &ack).unwrap();
        receiver.handle(saved).unwrap();
        receiver.handle(retry).unwrap();
        assert_eq!(
            fs::read_to_string(&log).unwrap(),
            format!("{saved}\n{retry}\n")
        );
        assert_eq!(
            fs::read_to_string(&ack).unwrap(),
            "RECACK 7\r\nRECACK 8\r\n"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn no_acknowledgment_when_saving_fails() {
        struct Full;
        impl Write for Full {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("disk full"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut r = RecLog {
            log: Full,
            ack: Vec::new(),
            saved: HashSet::new(),
            sync: |_| Ok(()),
            append_failed: false,
        };
        assert!(r.handle("REC seq=7 t_ms=700 volt_mv=3300").is_err());
        let error = r.handle("REC seq=8 t_ms=800 volt_mv=3300").unwrap_err();
        assert!(error.to_string().contains("restart the bridge"));
        assert!(r.ack.is_empty());
    }

    #[test]
    fn parses_valid_lines_and_skips_noise() {
        let (v, p) = parse_line("CSC1 temp_mc=21500 volt_mv=3912 light_lux=460 extra=1");
        assert_eq!(
            v.iter().map(|(f, v)| (f.file, *v)).collect::<Vec<_>>(),
            vec![
                ("temperature_mc", 21500),
                ("voltage_mv", 3912),
                ("light_lux", 460)
            ]
        );
        assert!(p.is_empty());
        assert!(parse_line("booting...").0.is_empty());
        assert!(parse_line("CSC2 temp_mc=1").0.is_empty());
    }

    #[test]
    fn drops_bad_fields_individually() {
        let (v, p) = parse_line("CSC1 temp_mc=999999 volt_mv=abc light_lux=12");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0.key, "light_lux");
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn parses_mappings() {
        let m = parse_mapping("pressure_pa=/sys/x/in_pressure_input*1000").unwrap();
        assert_eq!(
            (m.field.file, m.path.to_str().unwrap(), m.scale),
            ("pressure_pa", "/sys/x/in_pressure_input", 1000.0)
        );
        let m = parse_mapping("temperature_mc=/sys/class/thermal/thermal_zone0/temp").unwrap();
        assert_eq!((m.field.key, m.scale), ("temp_mc", 1.0));
        assert!(parse_mapping("humidity=/x").is_err());
        assert!(parse_mapping("temp_mc=/x*0").is_err());
    }

    #[test]
    fn writes_atomically_reads_scaled_and_expires() {
        let dir = std::env::temp_dir().join(format!("csc-bridge-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut out = Outbox::new(dir.clone(), Duration::from_millis(50)).unwrap();
        let start = Instant::now();
        out.write(FIELDS[0], 21500, start).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("temperature_mc")).unwrap(),
            "21500\n"
        );
        assert!(!dir.join(".temperature_mc.tmp").exists());

        let src = dir.join("kpa");
        fs::write(&src, "101.325\n").unwrap();
        let m = parse_mapping(&format!("pressure_pa={}*1000", src.display())).unwrap();
        assert_eq!(read_mapping(&m).unwrap(), 101325);

        assert!(out.expire(start + Duration::from_millis(10)).is_empty());
        assert_eq!(
            out.expire(start + Duration::from_millis(100)),
            vec!["temperature_mc"]
        );
        assert!(!dir.join("temperature_mc").exists());
        drop(out);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clears_leftovers_on_start_and_stop() {
        let dir = std::env::temp_dir().join(format!("csc-bridge-clean-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("light_lux"), "460\n").unwrap();
        fs::write(dir.join("notes.txt"), "keep\n").unwrap();
        let mut out = Outbox::new(dir.clone(), Duration::from_secs(5)).unwrap();
        assert!(
            !dir.join("light_lux").exists(),
            "old value must not look live"
        );
        out.write(FIELDS[1], 3900, Instant::now()).unwrap();
        drop(out);
        assert!(!dir.join("voltage_mv").exists());
        assert!(
            dir.join("notes.txt").exists(),
            "only sensor files are removed"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reader_survives_invalid_bytes_and_long_lines() {
        let dir = std::env::temp_dir().join(format!("csc-bridge-read-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("serial");
        let mut data = b"\xff\xfe boot noise\n".to_vec();
        data.extend(vec![b'x'; MAX_LINE * 2]);
        data.extend(b"\nCSC1 temp_mc=21500\r\n");
        fs::write(&path, data).unwrap();
        let rx = spawn_reader(path).unwrap();
        let lines: Vec<_> = rx.iter().collect();
        let ok: Vec<_> = lines.iter().filter_map(|l| l.as_ref().ok()).collect();
        assert_eq!(ok.last().unwrap().as_str(), "CSC1 temp_mc=21500");
        assert!(lines.last().unwrap().is_err(), "ends with end of input");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn never_parses_the_cut_off_start_of_a_long_line() {
        let dir = std::env::temp_dir().join(format!("csc-bridge-long-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("serial");
        // The first MAX_LINE bytes end in `pressure_pa=10132`, a valid-looking value.
        let pad = MAX_LINE - "CSC1 pad=".len() - " pressure_pa=10132".len();
        let mut data = format!("CSC1 pad={} pressure_pa=101325\n", "x".repeat(pad)).into_bytes();
        // Then several MAX_LINE chunks with no newline before the next good line.
        data.extend(vec![b'y'; MAX_LINE * 5]);
        data.extend(b"\nCSC1 pressure_pa=101325\n");
        fs::write(&path, data).unwrap();
        let ok: Vec<String> = spawn_reader(path)
            .unwrap()
            .iter()
            .filter_map(Result::ok)
            .collect();
        let values: Vec<i64> = ok
            .iter()
            .flat_map(|l| parse_line(l).0)
            .map(|(_, v)| v)
            .collect();
        assert_eq!(values, vec![101_325]);
        let _ = fs::remove_dir_all(&dir);
    }
}
