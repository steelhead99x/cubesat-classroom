//! Desk simulation of a cleartext 915 MHz downlink and the schedule that feeds
//! classroom viewers.
//!
//! Nothing here transmits. A 902–928 MHz channel in the United States is used
//! either under FCC Part 15 (unlicensed, with emission limits) or under Part 97
//! (the 33 cm amateur band). `part97` frames stay readable and carry a station
//! ID because a licensed control operator has to follow 47 CFR 97.113(a)(4) and
//! 97.119. The CRC and Reed-Solomon parity detect and repair loss. They do not
//! hide the payload. Choosing a service in this crate is a frame rule for a
//! desk exercise, not a certification, a coordination, or permission to transmit.
//!
//! The air frame is sized for a 255-byte payload limit, the single-transmission
//! size called out for the SX1276 FSK bench in `docs/why-moq.md`. Live telemetry
//! uses an 18-byte body so a beta LoRa board can show it on a small OLED. The
//! ground gateway expands that body to JSON and CBOR for phones and school apps.
//! Optional image, audio, and video objects use the same frame and are the first
//! things dropped when the fan-out budget runs out. Those media objects are not
//! for the OLED board.

mod assemble;
mod channel;
mod fec;
mod frame;
mod schedule;
mod schema;
mod study;

pub use assemble::{Assembler, GroupKey, Heard, Metrics, Recovered, RelayView};
pub use channel::{Channel, ChannelConfig};
pub use frame::{
    AirConfig, AirFrame, AirKind, Error, FrameFault, MTU, Service, StationId, encode_object,
    interleave,
};
pub use schedule::{Budget, FeedTrack, LIVE_BROADCAST, Piece, Scheduler};
pub use schema::{
    AGE_SKEW_MS, AUDIO_MAGIC, AudioFrame, BOARD_BODY_LEN, EVIDENCE_DESK, FREQUENCY_HZ,
    HEALTH_SCHEMA, LORA_BOARD_PAYLOAD, LORA_SMALL_PAYLOAD, MAX_OBJECT_BYTES, OLED_COLUMNS,
    RelayHealth, SOURCE_SENSORS, SOURCE_SIMULATION, SensorReading, TELEMETRY_SCHEMA, Telemetry,
    age_ms,
};
pub use study::{Bench, Studies, bench, bench_text, studies, text_report};
