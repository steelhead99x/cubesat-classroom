//! Classroom telemetry, relay health, and the small audio frame.
//!
//! Telemetry is a full snapshot every time. A late viewer does not need an
//! earlier message. JSON is what a browser or school app should parse. CBOR is
//! the same snapshot on its own track for a native app. The air path uses a
//! smaller fixed body so a beta LoRa board can show it on a 128×64 OLED without
//! a CBOR library.
//!
//! Readers must ignore JSON fields they do not know. This program only writes
//! `desk-simulation`. A later sender may use another evidence label; showing
//! that label is not a claim that this repository flew.

use crate::assemble::Metrics;
use crate::frame::{Error, Service};

/// Schema name inside every telemetry object.
pub const TELEMETRY_SCHEMA: &str = "cubesat.telemetry.v1";
/// Schema name inside every relay-health object.
pub const HEALTH_SCHEMA: &str = "cubesat.relay-health.v1";
/// Evidence label this crate writes. It means a desk simulation, not a flight.
pub const EVIDENCE_DESK: &str = "desk-simulation";
/// `source` string for the built-in formula, matching the station simulator.
pub const SOURCE_SIMULATION: &str = "simulation";
/// `source` string when the numbers came from the sensor files.
pub const SOURCE_SENSORS: &str = "configured sensor inputs";
/// Lab-exercise label for a frequency inside 902–928 MHz. Not a channel assignment.
pub const FREQUENCY_HZ: u64 = 915_000_000;
/// Largest telemetry or media object the air path will frame.
pub const MAX_OBJECT_BYTES: usize = 16 * 1024;
/// Largest telemetry document a reader accepts.
pub const MAX_TELEMETRY_BYTES: usize = 4096;
/// How far ahead of the receiver clock a capture time may be.
pub const AGE_SKEW_MS: u64 = 2_000;
/// How old a capture time may be and still count as fresh.
pub const MAX_AGE_MS: u64 = 5_000;
/// Magic at the start of an audio frame.
pub const AUDIO_MAGIC: &[u8] = b"CSA1";
/// Fixed telemetry body carried inside one air frame for a LoRa board.
pub const BOARD_BODY_LEN: usize = 18;
/// Conservative single-packet budget. A frame with a station ID of up to 12
/// characters fits. This is a size target for the beta board, not a LoRaWAN rule.
pub const LORA_SMALL_PAYLOAD: usize = 51;
/// Size target that fits every station ID this crate accepts (up to 16 characters).
pub const LORA_BOARD_PAYLOAD: usize = 64;
/// Characters on one line of the beta OLED preview (8-pixel font on a 128-pixel row).
pub const OLED_COLUMNS: usize = 16;
/// Lines in the beta OLED preview. A 128×64 panel has room for more; four lines
/// also fit a 128×32 panel.
pub const OLED_ROWS: usize = 4;

/// Numbers read from the sensor files, already converted to display units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorReading {
    /// Degrees Celsius.
    pub temperature_c: f64,
    /// Bus voltage, volts.
    pub bus_voltage_v: f64,
    /// Illuminance, lux.
    pub light_lux: u32,
    /// Air pressure, pascals, when the file was readable.
    pub pressure_pa: Option<u32>,
}

/// One telemetry snapshot. Every field the contract requires is present.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Telemetry {
    /// Always [`TELEMETRY_SCHEMA`] for objects this crate writes.
    pub schema: String,
    /// Counts up by one for each snapshot the sender attempted.
    pub sequence: u64,
    /// Capture time, Unix milliseconds. Not a time of arrival.
    pub captured_unix_ms: u64,
    /// Where the numbers came from, in short words.
    pub source: String,
    /// What kind of evidence this is. This crate writes [`EVIDENCE_DESK`].
    pub evidence: String,
    /// Frame-rule label. See [`Service`].
    pub service: Service,
    /// Exercise frequency in hertz. See [`FREQUENCY_HZ`].
    pub frequency_hz: u64,
    /// Control-operator station ID, when the sender supplied one.
    pub station_id: Option<String>,
    /// Degrees Celsius.
    pub temperature_c: f64,
    /// Bus voltage, volts.
    pub bus_voltage_v: f64,
    /// Illuminance, lux.
    pub light_lux: u32,
    /// Air pressure, pascals. Absent when the sender has no pressure sensor.
    pub pressure_pa: Option<u32>,
}

impl Telemetry {
    /// The same formula as the station's synthetic sample, plus the live-feed fields.
    pub fn synthetic(
        sequence: u64,
        captured_unix_ms: u64,
        service: Service,
        station_id: Option<&str>,
    ) -> Self {
        Self {
            schema: TELEMETRY_SCHEMA.into(),
            sequence,
            captured_unix_ms,
            source: SOURCE_SIMULATION.into(),
            evidence: EVIDENCE_DESK.into(),
            service,
            frequency_hz: FREQUENCY_HZ,
            station_id: station_id.map(str::to_string),
            temperature_c: 18.0 + (sequence % 11) as f64 / 2.0,
            bus_voltage_v: (3900 + sequence % 61) as f64 / 1000.0,
            light_lux: if sequence % 30 < 20 { 460 } else { 0 },
            pressure_pa: None,
        }
    }

    /// A snapshot from sensor files. Missing pressure stays absent.
    pub fn from_sensors(
        sequence: u64,
        captured_unix_ms: u64,
        service: Service,
        station_id: Option<&str>,
        reading: SensorReading,
    ) -> Self {
        Self {
            schema: TELEMETRY_SCHEMA.into(),
            sequence,
            captured_unix_ms,
            source: SOURCE_SENSORS.into(),
            evidence: EVIDENCE_DESK.into(),
            service,
            frequency_hz: FREQUENCY_HZ,
            station_id: station_id.map(str::to_string),
            temperature_c: reading.temperature_c,
            bus_voltage_v: reading.bus_voltage_v,
            light_lux: reading.light_lux,
            pressure_pa: reading.pressure_pa,
        }
    }

    /// Milliseconds since capture. Errors when the snapshot is stale or too far in the future.
    pub fn age_ms(&self, now_unix_ms: u64) -> Result<u64, Error> {
        age_ms(self.captured_unix_ms, now_unix_ms, MAX_AGE_MS, AGE_SKEW_MS)
    }

    /// Encode as JSON, the form school apps should subscribe to.
    pub fn to_json(&self) -> Result<Vec<u8>, Error> {
        encode_json(self)
    }

    /// Encode as CBOR for the `telemetry.cbor` track. The air path uses [`Self::to_board_body`].
    pub fn to_cbor(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        ciborium::into_writer(self, &mut out).map_err(|err| Error::Codec(err.to_string()))?;
        if out.len() > MAX_TELEMETRY_BYTES {
            return Err(Error::Codec("telemetry CBOR is above 4 KiB".into()));
        }
        Ok(out)
    }

    /// Parse a JSON snapshot and require [`TELEMETRY_SCHEMA`].
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        bounded(bytes)?;
        let value: Self =
            serde_json::from_slice(bytes).map_err(|err| Error::Codec(err.to_string()))?;
        value.check_schema()
    }

    /// Parse a CBOR snapshot and require [`TELEMETRY_SCHEMA`].
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, Error> {
        bounded(bytes)?;
        let value: Self =
            ciborium::from_reader(bytes).map_err(|err| Error::Codec(err.to_string()))?;
        value.check_schema()
    }

    fn check_schema(self) -> Result<Self, Error> {
        if self.schema != TELEMETRY_SCHEMA {
            return Err(Error::Codec(format!(
                "expected schema {TELEMETRY_SCHEMA}, found {}",
                self.schema
            )));
        }
        Ok(self)
    }

    /// Fixed 18-byte body for one LoRa packet. Sub-second time is dropped, temperature
    /// is stored to 0.01 °C, and voltage is stored to 1 mV.
    pub fn to_board_body(&self) -> Result<[u8; BOARD_BODY_LEN], Error> {
        let temp = (self.temperature_c * 100.0).round();
        if !temp.is_finite() || !(-32_768.0..=32_767.0).contains(&temp) {
            return Err(Error::Payload(
                "temperature does not fit in a signed 0.01 °C field".into(),
            ));
        }
        let volts = (self.bus_voltage_v * 1000.0).round();
        if !volts.is_finite() || !(0.0..=65_535.0).contains(&volts) {
            return Err(Error::Payload(
                "bus voltage does not fit in a 0 to 65.535 V field".into(),
            ));
        }
        let captured_unix_s = u32::try_from(self.captured_unix_ms / 1000).map_err(|_| {
            Error::Payload("capture time does not fit in a 32-bit second count".into())
        })?;
        let mut flags = u8::from(self.pressure_pa.is_some());
        if self.source == SOURCE_SENSORS {
            flags |= 0b0000_0010;
        }
        let mut body = [0_u8; BOARD_BODY_LEN];
        body[1] = flags;
        body[2..6].copy_from_slice(&captured_unix_s.to_be_bytes());
        body[6..8].copy_from_slice(&(temp as i16).to_be_bytes());
        body[8..10].copy_from_slice(&(volts as u16).to_be_bytes());
        body[10..14].copy_from_slice(&self.light_lux.to_be_bytes());
        let pressure = self.pressure_pa.unwrap_or(0);
        body[14..18].copy_from_slice(&pressure.to_be_bytes());
        Ok(body)
    }

    /// Decode a board body. Sequence, service, and station ID come from the air header.
    ///
    /// The free-form source string is only a sensor-file bit on the air, and the
    /// evidence label is filled in as [`EVIDENCE_DESK`].
    pub fn from_board_body(
        body: &[u8],
        sequence: u64,
        service: Service,
        station_id: Option<&str>,
    ) -> Result<Self, Error> {
        if body.len() != BOARD_BODY_LEN || body[0] != 0 {
            return Err(Error::Payload(
                "board telemetry body must be 18 bytes and start with a zero".into(),
            ));
        }
        let flags = body[1];
        let captured_unix_s = u32::from_be_bytes(body[2..6].try_into().expect("four bytes"));
        let temp = i16::from_be_bytes(body[6..8].try_into().expect("two bytes"));
        let volts = u16::from_be_bytes(body[8..10].try_into().expect("two bytes"));
        let light_lux = u32::from_be_bytes(body[10..14].try_into().expect("four bytes"));
        let pressure = u32::from_be_bytes(body[14..18].try_into().expect("four bytes"));
        Ok(Self {
            schema: TELEMETRY_SCHEMA.into(),
            sequence,
            captured_unix_ms: u64::from(captured_unix_s) * 1000,
            source: if (flags & 0b0000_0010) == 0 {
                SOURCE_SIMULATION.into()
            } else {
                SOURCE_SENSORS.into()
            },
            evidence: EVIDENCE_DESK.into(),
            service,
            frequency_hz: FREQUENCY_HZ,
            station_id: station_id.map(str::to_string),
            temperature_c: f64::from(temp) / 100.0,
            bus_voltage_v: f64::from(volts) / 1000.0,
            light_lux,
            pressure_pa: ((flags & 0b0000_0001) == 1).then_some(pressure),
        })
    }

    /// Four short lines for a beta 128×64 OLED. Each line is at most [`OLED_COLUMNS`] characters.
    ///
    /// The keyboard on those boards is not on this downlink. A button may change
    /// what the board draws locally; it does not transmit.
    pub fn oled_lines(&self) -> [String; OLED_ROWS] {
        let id = self.station_id.as_deref().unwrap_or("----");
        let head = format!("{id} #{}", self.sequence);
        let reading = format!("{:.1}C {:.2}V", self.temperature_c, self.bus_voltage_v);
        let light = match self.pressure_pa {
            Some(pa) => format!("{}lx {}hPa", self.light_lux, pa / 100),
            None => format!("{} lx", self.light_lux),
        };
        let service = match self.service {
            Service::Simulation => "sim",
            Service::Part15 => "part15",
            Service::Part97 => "part97",
        };
        [
            fit_oled(&head),
            fit_oled(&reading),
            fit_oled(&light),
            fit_oled(&format!("{service} desk")),
        ]
    }
}

fn fit_oled(text: &str) -> String {
    text.chars().take(OLED_COLUMNS).collect()
}

/// How old `captured_unix_ms` is at `now_unix_ms`.
pub fn age_ms(
    captured_unix_ms: u64,
    now_unix_ms: u64,
    max_age_ms: u64,
    skew_ms: u64,
) -> Result<u64, Error> {
    match now_unix_ms.checked_sub(captured_unix_ms) {
        Some(age) if age <= max_age_ms => Ok(age),
        Some(age) => Err(Error::Codec(format!(
            "telemetry is stale ({age} ms); the freshness limit is {max_age_ms} ms"
        ))),
        None => {
            let ahead = captured_unix_ms - now_unix_ms;
            if ahead <= skew_ms {
                Ok(0)
            } else {
                Err(Error::Codec(format!(
                    "capture time is {ahead} ms ahead of the receiver clock"
                )))
            }
        }
    }
}

fn bounded(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_TELEMETRY_BYTES {
        return Err(Error::Codec("telemetry object is above 4 KiB".into()));
    }
    Ok(())
}

fn encode_json(value: &impl serde::Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|err| Error::Codec(err.to_string()))
}

/// Ground view of the virtual receivers. This is not sent on the air.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelayHealth {
    /// Always [`HEALTH_SCHEMA`] for objects this crate writes.
    pub schema: String,
    /// When the ground assembled this report, Unix milliseconds.
    pub captured_unix_ms: u64,
    /// Evidence label. This crate writes [`EVIDENCE_DESK`].
    pub evidence: String,
    /// Objects recovered since the assembler started.
    pub recovered: u64,
    /// Objects given up.
    pub lost: u64,
    /// Objects dropped because the unfinished-object map was full.
    pub abandoned: u64,
    /// Objects rebuilt with parity.
    pub fec_repairs: u64,
    /// Frames that failed the CRC.
    pub crc_failures: u64,
    /// Extra copies of a shard.
    pub duplicates: u64,
    /// One row per receiver that has been heard.
    pub relays: Vec<RelayRow>,
}

/// One receiver inside [`RelayHealth`].
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelayRow {
    /// `relay-N`.
    pub id: String,
    /// Latest RSSI sample, dBm. A desk-simulation number unless a later sender says otherwise.
    pub rssi_dbm: f64,
    /// Latest SNR sample, dB.
    pub snr_db: f64,
    /// Frames from this receiver.
    pub frames: u64,
    /// CRC failures from this receiver.
    pub crc_failures: u64,
    /// Duplicate shards from this receiver.
    pub duplicates: u64,
}

impl RelayHealth {
    /// Build a health snapshot from assembler totals.
    pub fn from_metrics(captured_unix_ms: u64, metrics: &Metrics) -> Self {
        Self {
            schema: HEALTH_SCHEMA.into(),
            captured_unix_ms,
            evidence: EVIDENCE_DESK.into(),
            recovered: metrics.recovered,
            lost: metrics.lost,
            abandoned: metrics.abandoned,
            fec_repairs: metrics.fec_repairs,
            crc_failures: metrics.crc_failures,
            duplicates: metrics.duplicates,
            relays: metrics
                .relays
                .iter()
                .map(|relay| RelayRow {
                    id: relay.id.clone(),
                    rssi_dbm: relay.rssi_dbm,
                    snr_db: relay.snr_db,
                    frames: relay.frames,
                    crc_failures: relay.crc_failures,
                    duplicates: relay.duplicates,
                })
                .collect(),
        }
    }

    /// JSON for the `health` track.
    pub fn to_json(&self) -> Result<Vec<u8>, Error> {
        encode_json(self)
    }

    /// Parse a health object and require [`HEALTH_SCHEMA`].
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_OBJECT_BYTES {
            return Err(Error::Codec("health object is too large".into()));
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|err| Error::Codec(err.to_string()))?;
        if value.schema != HEALTH_SCHEMA {
            return Err(Error::Codec(format!(
                "expected schema {HEALTH_SCHEMA}, found {}",
                value.schema
            )));
        }
        Ok(value)
    }
}

/// Mono PCM wrapped so a client can see the sample rate without a side channel.
///
/// The bytes are a desk-simulation tone when this crate generates them, not a
/// microphone. Clients that do not want sound should not subscribe to `audio`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioFrame {
    /// Object sequence.
    pub sequence: u32,
    /// Capture time, Unix milliseconds.
    pub captured_unix_ms: u64,
    /// Samples per second.
    pub sample_rate_hz: u32,
    /// Channel count. Version 1 is mono, so this is 1.
    pub channels: u8,
    /// Little-endian signed 16-bit samples.
    pub pcm_s16le: Vec<u8>,
}

impl AudioFrame {
    /// A short square wave. `samples` is the number of mono samples, not bytes.
    pub fn tone(sequence: u32, captured_unix_ms: u64, samples: usize) -> Self {
        let mut pcm_s16le = Vec::with_capacity(samples.saturating_mul(2));
        for n in 0..samples {
            let sample: i16 = if (n / 4).is_multiple_of(2) { 800 } else { -800 };
            pcm_s16le.extend(sample.to_le_bytes());
        }
        Self {
            sequence,
            captured_unix_ms,
            sample_rate_hz: 8_000,
            channels: 1,
            pcm_s16le,
        }
    }

    /// `CSA1`, then big-endian sequence, time, and rate, then one channel byte, then PCM.
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        if self.channels != 1 || !self.pcm_s16le.len().is_multiple_of(2) || self.sample_rate_hz == 0
        {
            return Err(Error::Payload(
                "audio frames are mono PCM with an even number of bytes".into(),
            ));
        }
        let mut out = Vec::with_capacity(21 + self.pcm_s16le.len());
        out.extend(AUDIO_MAGIC);
        out.extend(self.sequence.to_be_bytes());
        out.extend(self.captured_unix_ms.to_be_bytes());
        out.extend(self.sample_rate_hz.to_be_bytes());
        out.push(self.channels);
        out.extend(&self.pcm_s16le);
        if out.len() > MAX_OBJECT_BYTES {
            return Err(Error::Payload("audio frame is above 16 KiB".into()));
        }
        Ok(out)
    }

    /// Parse a `CSA1` frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 21 || !bytes.starts_with(AUDIO_MAGIC) {
            return Err(Error::Payload(
                "audio frame is missing the CSA1 header".into(),
            ));
        }
        let sequence = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let captured_unix_ms = u64::from_be_bytes(bytes[8..16].try_into().expect("eight bytes"));
        let sample_rate_hz = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let channels = bytes[20];
        let pcm_s16le = bytes[21..].to_vec();
        let frame = Self {
            sequence,
            captured_unix_ms,
            sample_rate_hz,
            channels,
            pcm_s16le,
        };
        // Re-check the rules so a truncated or stereo blob cannot pass.
        frame.encode()?;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_and_cbor_round_trip_the_same_snapshot() {
        let sample = Telemetry::synthetic(7, 1_700_000_000_123, Service::Part15, None);
        let json = sample.to_json().unwrap();
        let text = std::str::from_utf8(&json).unwrap();
        assert!(text.contains("\"schema\":\"cubesat.telemetry.v1\""));
        assert!(text.contains("\"evidence\":\"desk-simulation\""));
        assert!(text.contains("\"service\":\"part15\""));
        assert!(text.contains("\"temperature_c\":21.5"));
        assert!(text.contains("temperature_c"));
        let back = Telemetry::from_json(&json).unwrap();
        assert_eq!(back, sample);
        let cbor = sample.to_cbor().unwrap();
        assert!(cbor.windows(13).any(|window| window == b"temperature_c"));
        assert_eq!(Telemetry::from_cbor(&cbor).unwrap(), sample);
        assert!(cbor.len() < MAX_TELEMETRY_BYTES);
    }

    #[test]
    fn stale_and_future_snapshots_are_rejected() {
        let sample = Telemetry::synthetic(1, 5_000, Service::Simulation, None);
        assert!(sample.age_ms(5_000).is_ok());
        assert!(sample.age_ms(10_000).is_ok());
        assert!(sample.age_ms(10_001).is_err());
        assert!(sample.age_ms(3_000).is_ok());
        assert!(sample.age_ms(2_999).is_err());
    }

    #[test]
    fn board_body_fits_a_lora_payload_and_an_oled() {
        let sample = Telemetry::synthetic(7, 1_700_000_000_900, Service::Part97, Some("N0CALL"));
        let body = sample.to_board_body().unwrap();
        assert_eq!(body.len(), BOARD_BODY_LEN);
        let back = Telemetry::from_board_body(&body, 7, Service::Part97, Some("N0CALL")).unwrap();
        assert_eq!(back.temperature_c, 21.5);
        assert_eq!(back.bus_voltage_v, 3.907);
        assert_eq!(back.light_lux, 460);
        assert_eq!(back.captured_unix_ms, 1_700_000_000_000);
        assert!(back.pressure_pa.is_none());
        let lines = back.oled_lines();
        assert!(
            lines
                .iter()
                .all(|line| line.chars().count() <= OLED_COLUMNS)
        );
        assert_eq!(lines[0], "N0CALL #7");
        assert!(lines[1].contains("21.5C"));
        assert!(lines[1].contains("3.91V") || lines[1].contains("3.90V"));
        assert!(lines[3].contains("part97"));
        assert!(lines[3].contains("desk"));
    }

    #[test]
    fn audio_tone_round_trips() {
        let frame = AudioFrame::tone(3, 50, 16);
        let bytes = frame.encode().unwrap();
        assert!(bytes.starts_with(AUDIO_MAGIC));
        assert_eq!(AudioFrame::decode(&bytes).unwrap(), frame);
    }
}
