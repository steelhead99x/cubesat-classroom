//! Cleartext air frame for the 915 MHz desk exercise.
//!
//! ```text
//!  0  magic u16 = 0xC591
//!  2  version u8 = 1
//!  3  kind u8
//!  4  service u8          0 simulation, 1 part15, 2 part97
//!  5  flags u8            must be 0 in version 1
//!  6  session u16
//!  8  sequence u32
//! 12  index u8            0..k are payload shards, then parity shards
//! 13  k u8                payload shard count
//! 14  r u8                parity shard count
//! 15  shard_len u16
//! 17  object_len u32      payload bytes before zero padding
//! 21  id_len u8
//! 22  station id          ASCII, present on every shard when set
//! ..  shard bytes
//! ..  crc32               CRC-32/ISO-HDLC over everything before it
//! ```
//!
//! Multi-byte integers are big-endian so a hex dump is readable. The station ID
//! and the payload bytes are copied in the clear. A receiver can read a data
//! shard without undoing the parity.

use crate::fec;
use crate::schema::MAX_OBJECT_BYTES;

/// Largest air frame, matching a 255-byte FSK FIFO fill.
pub const MTU: usize = 255;
/// Bytes before the station ID.
const PREFIX: usize = 22;
const CRC_LEN: usize = 4;
const MAGIC: u16 = 0xC591;
const VERSION: u8 = 1;
/// Most payload shards one object may use. Keeps a low-rate image inside 16 KiB.
pub const MAX_DATA_SHARDS: usize = 48;
/// Most Reed-Solomon parity shards. Enough for the burst exercise, not a modem.
pub const MAX_PARITY_SHARDS: usize = 16;

/// What went wrong while building a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The station ID is missing or not a plausible call-sign token.
    StationId(String),
    /// The object does not fit in the air frame.
    Payload(String),
    /// Reed-Solomon could not build or restore shards.
    Fec(String),
    /// JSON or CBOR did not match the classroom schema.
    Codec(String),
    /// A number in a scenario or channel config is out of range.
    Config(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StationId(msg)
            | Self::Payload(msg)
            | Self::Fec(msg)
            | Self::Codec(msg)
            | Self::Config(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

/// A decoded frame failed the CRC, or the CRC passed and the fields are nonsense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFault {
    /// CRC-32 did not match. Count this as a corrupted reception.
    Crc,
    /// The CRC matched and the header is not a version-1 classroom frame.
    Malformed,
}

/// Which classroom service the frame was built for.
///
/// This is a label on a desk exercise. It does not authorize a transmission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    /// No service claimed. Station ID is optional.
    Simulation,
    /// Built as a Part 15 (unlicensed) exercise. Station ID is optional.
    /// Not an FCC certification.
    Part15,
    /// Built as a Part 97 (amateur) exercise. Station ID is required and the
    /// payload stays readable. Not permission to transmit.
    Part97,
}

impl Service {
    fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Simulation),
            1 => Some(Self::Part15),
            2 => Some(Self::Part97),
            _ => None,
        }
    }

    fn as_byte(self) -> u8 {
        match self {
            Self::Simulation => 0,
            Self::Part15 => 1,
            Self::Part97 => 2,
        }
    }

    /// Whether every frame must carry a station ID.
    pub const fn requires_station_id(self) -> bool {
        matches!(self, Self::Part97)
    }
}

/// A control operator's station ID, syntax-checked and stored in uppercase.
///
/// The check is not a license lookup. `N0CALL` is the usual documentation
/// placeholder and is not an assigned call sign. Do not put a student's name
/// or call sign here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StationId(String);

impl StationId {
    /// Uppercase `text` and accept it when it looks like a call-sign token.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let text = text.trim().to_ascii_uppercase();
        validate_station_id(&text)?;
        Ok(Self(text))
    }

    /// The uppercase token carried in the frame.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_station_id(text: &str) -> Result<(), Error> {
    if !(3..=16).contains(&text.len()) {
        return Err(Error::StationId(
            "station ID must be 3 to 16 characters (a call-sign token, not a name)".into(),
        ));
    }
    let mut slash = 0;
    let mut letter = false;
    let mut digit = false;
    for (i, c) in text.chars().enumerate() {
        match c {
            'A'..='Z' => letter = true,
            '0'..='9' => digit = true,
            '/' => {
                slash += 1;
                if i == 0 || i + 1 == text.len() {
                    return Err(Error::StationId(
                        "a slash in a station ID cannot be first or last".into(),
                    ));
                }
            }
            _ => {
                return Err(Error::StationId(
                    "station ID uses only letters, digits, and one slash".into(),
                ));
            }
        }
    }
    if slash > 1 || !letter || !digit {
        return Err(Error::StationId(
            "station ID needs a letter, a digit, and at most one slash".into(),
        ));
    }
    Ok(())
}

/// What an air object carries. Relay health is computed on the ground and is
/// not a kind on the air.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum AirKind {
    /// One CBOR telemetry snapshot.
    Telemetry = 0,
    /// One JPEG picture.
    Image = 1,
    /// One `CSA1` audio frame.
    Audio = 2,
    /// One independent JPEG video frame.
    Video = 3,
}

impl AirKind {
    fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Telemetry),
            1 => Some(Self::Image),
            2 => Some(Self::Audio),
            3 => Some(Self::Video),
            _ => None,
        }
    }

    /// Whether the fan-out may drop this kind before telemetry.
    pub const fn is_media(self) -> bool {
        !matches!(self, Self::Telemetry)
    }
}

/// How to turn one object into air frames.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AirConfig {
    /// Frame-rule label. See [`Service`].
    pub service: Service,
    /// Control-operator station ID. Required for [`Service::Part97`].
    pub station_id: Option<StationId>,
    /// Changes when the sender restarts so old sequence numbers cannot collide.
    pub session: u16,
    /// Reed-Solomon parity shards added to every object. Zero sends the payload only.
    pub parity: u8,
    /// Largest payload shard. Zero means "as large as the 255-byte frame allows".
    pub max_shard: u16,
    /// Groups of objects whose shards are interleaved on the wire. 1 sends each
    /// object whole, which keeps telemetry latency to one object.
    pub interleave_depth: u16,
}

impl Default for AirConfig {
    fn default() -> Self {
        Self {
            service: Service::Simulation,
            station_id: None,
            session: 1,
            parity: 1,
            max_shard: 0,
            interleave_depth: 1,
        }
    }
}

impl AirConfig {
    /// Reject a configuration that cannot build a legal frame.
    pub fn check(&self) -> Result<(), Error> {
        if self.service.requires_station_id() && self.station_id.is_none() {
            return Err(Error::StationId(
                "amateur mode (Part 97) needs the control operator's station ID on every frame. This program still does not transmit.".into(),
            ));
        }
        if self.parity as usize > MAX_PARITY_SHARDS {
            return Err(Error::Config(format!(
                "parity shards must be 0 to {MAX_PARITY_SHARDS}"
            )));
        }
        if self.interleave_depth == 0 {
            return Err(Error::Config(
                "interleave depth starts at 1 (send each object whole)".into(),
            ));
        }
        let room = max_shard_room(
            self.station_id
                .as_ref()
                .map(|id| id.as_str().len())
                .unwrap_or(0),
            self.parity,
        );
        if room < if self.parity > 0 { 2 } else { 1 } {
            return Err(Error::Payload(
                "station ID leaves no room for a payload shard".into(),
            ));
        }
        if self.max_shard > 0 && usize::from(self.max_shard) > room {
            return Err(Error::Config(format!(
                "max shard {} does not fit in a {MTU}-byte frame (room is {room})",
                self.max_shard
            )));
        }
        Ok(())
    }
}

fn max_shard_room(id_len: usize, parity: u8) -> usize {
    let raw = MTU.saturating_sub(PREFIX + id_len + CRC_LEN);
    if parity > 0 { raw & !1 } else { raw }
}

/// One decoded air frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AirFrame {
    /// Payload kind.
    pub kind: AirKind,
    /// Service label copied from the header.
    pub service: Service,
    /// Sender session.
    pub session: u16,
    /// Object sequence.
    pub sequence: u32,
    /// Shard index. Below `data_shards` this is payload; at or above it, parity.
    pub index: u8,
    /// Payload shard count.
    pub data_shards: u8,
    /// Parity shard count.
    pub parity_shards: u8,
    /// Unpadded object length.
    pub object_len: u32,
    /// Station ID, when the frame carried one.
    pub station_id: Option<StationId>,
    /// Shard bytes, including zero padding on the last payload shard.
    pub shard: Vec<u8>,
}

/// Split `payload` into air frames in shard-index order (not yet interleaved).
pub fn encode_object(
    config: &AirConfig,
    kind: AirKind,
    sequence: u32,
    payload: &[u8],
) -> Result<Vec<Vec<u8>>, Error> {
    config.check()?;
    if payload.is_empty() {
        return Err(Error::Payload("refusing to frame an empty object".into()));
    }
    if payload.len() > MAX_OBJECT_BYTES {
        return Err(Error::Payload(format!(
            "object is {} bytes; the air path caps an object at {MAX_OBJECT_BYTES}",
            payload.len()
        )));
    }
    let id = config.station_id.as_ref().map(|id| id.as_str().as_bytes());
    let id_len = id.map_or(0, <[u8]>::len);
    let room = max_shard_room(id_len, config.parity);
    let limit = if config.max_shard == 0 {
        room
    } else {
        usize::from(config.max_shard).min(room)
    };
    let (k, shard_len) = plan(payload.len(), limit, config.parity)?;
    let mut data = vec![vec![0_u8; shard_len]; k];
    for (i, byte) in payload.iter().enumerate() {
        data[i / shard_len][i % shard_len] = *byte;
    }
    let parity = if config.parity == 0 {
        Vec::new()
    } else {
        fec::parity(&data, usize::from(config.parity))?
    };
    let mut frames = Vec::with_capacity(k + parity.len());
    for (index, shard) in data.into_iter().chain(parity).enumerate() {
        frames.push(encode_frame(
            kind,
            config.service,
            config.session,
            sequence,
            u8::try_from(index).expect("shard index fits in u8"),
            u8::try_from(k).expect("shard count fits in u8"),
            config.parity,
            u32::try_from(payload.len()).expect("object length fits in u32"),
            id.unwrap_or(&[]),
            &shard,
        )?);
    }
    Ok(frames)
}

fn plan(payload_len: usize, max_shard: usize, parity: u8) -> Result<(usize, usize), Error> {
    if max_shard == 0 || (parity > 0 && max_shard < 2) {
        return Err(Error::Payload(
            "no room for a payload shard in a 255-byte frame".into(),
        ));
    }
    let mut k = payload_len.div_ceil(max_shard).max(1);
    loop {
        if k > MAX_DATA_SHARDS {
            return Err(Error::Payload(format!(
                "object needs more than {MAX_DATA_SHARDS} shards"
            )));
        }
        let mut shard = payload_len.div_ceil(k);
        if parity > 0 {
            if shard % 2 == 1 {
                shard += 1;
            }
            if shard < 2 {
                shard = 2;
            }
        }
        if shard <= max_shard {
            return Ok((k, shard));
        }
        k += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_frame(
    kind: AirKind,
    service: Service,
    session: u16,
    sequence: u32,
    index: u8,
    k: u8,
    r: u8,
    object_len: u32,
    station_id: &[u8],
    shard: &[u8],
) -> Result<Vec<u8>, Error> {
    if station_id.len() > 16 {
        return Err(Error::StationId(
            "station ID is longer than 16 bytes".into(),
        ));
    }
    let mut body = Vec::with_capacity(PREFIX + station_id.len() + shard.len());
    body.extend(MAGIC.to_be_bytes());
    body.push(VERSION);
    body.push(kind as u8);
    body.push(service.as_byte());
    body.push(0);
    body.extend(session.to_be_bytes());
    body.extend(sequence.to_be_bytes());
    body.push(index);
    body.push(k);
    body.push(r);
    body.extend(u16::try_from(shard.len()).unwrap_or(u16::MAX).to_be_bytes());
    body.extend(object_len.to_be_bytes());
    body.push(u8::try_from(station_id.len()).unwrap_or(u8::MAX));
    body.extend(station_id);
    body.extend(shard);
    if body.len() + CRC_LEN > MTU {
        return Err(Error::Payload(format!(
            "frame is {} bytes, above the {MTU}-byte air limit",
            body.len() + CRC_LEN
        )));
    }
    let crc = crc32fast::hash(&body);
    body.extend(crc.to_be_bytes());
    Ok(body)
}

/// Check the CRC and parse one air frame.
pub fn decode(frame: &[u8]) -> Result<AirFrame, FrameFault> {
    if frame.len() < PREFIX + CRC_LEN || frame.len() > MTU {
        return Err(FrameFault::Malformed);
    }
    let (body, crc_bytes) = frame.split_at(frame.len() - CRC_LEN);
    let mut crc_buf = [0_u8; 4];
    crc_buf.copy_from_slice(crc_bytes);
    if crc32fast::hash(body) != u32::from_be_bytes(crc_buf) {
        return Err(FrameFault::Crc);
    }
    if u16::from_be_bytes([body[0], body[1]]) != MAGIC || body[2] != VERSION || body[5] != 0 {
        return Err(FrameFault::Malformed);
    }
    let kind = AirKind::from_byte(body[3]).ok_or(FrameFault::Malformed)?;
    let service = Service::from_byte(body[4]).ok_or(FrameFault::Malformed)?;
    let session = u16::from_be_bytes([body[6], body[7]]);
    let sequence = u32::from_be_bytes([body[8], body[9], body[10], body[11]]);
    let index = body[12];
    let k = body[13];
    let r = body[14];
    let shard_len = usize::from(u16::from_be_bytes([body[15], body[16]]));
    let object_len = u32::from_be_bytes([body[17], body[18], body[19], body[20]]);
    let id_len = usize::from(body[21]);
    if k == 0
        || usize::from(k) > MAX_DATA_SHARDS
        || usize::from(r) > MAX_PARITY_SHARDS
        || usize::from(index) >= usize::from(k) + usize::from(r)
        || shard_len == 0
        || object_len == 0
        || u64::from(object_len) > u64::from(k) * shard_len as u64
        || (r > 0 && (shard_len < 2 || shard_len % 2 == 1))
        || id_len > 16
        || PREFIX + id_len + shard_len != body.len()
    {
        return Err(FrameFault::Malformed);
    }
    let id_bytes = &body[PREFIX..PREFIX + id_len];
    let station_id = if id_len == 0 {
        if service.requires_station_id() {
            return Err(FrameFault::Malformed);
        }
        None
    } else {
        let text = std::str::from_utf8(id_bytes).map_err(|_| FrameFault::Malformed)?;
        Some(StationId::parse(text).map_err(|_| FrameFault::Malformed)?)
    };
    if service.requires_station_id() && station_id.is_none() {
        return Err(FrameFault::Malformed);
    }
    let shard = body[PREFIX + id_len..].to_vec();
    Ok(AirFrame {
        kind,
        service,
        session,
        sequence,
        index,
        data_shards: k,
        parity_shards: r,
        object_len,
        station_id,
        shard,
    })
}

/// Reorder shards so a short burst does not land inside one object.
///
/// `depth` is how many objects share a block. Depth 1 keeps each object's
/// shards together. A larger depth spreads shard 0 of every object in the
/// block, then shard 1, and so on. The receiver does not need to know the
/// depth: it reassembles by session, kind, sequence, and shard index.
pub fn interleave(groups: &[Vec<Vec<u8>>], depth: usize) -> Vec<Vec<u8>> {
    let depth = depth.max(1);
    let mut out = Vec::new();
    for block in groups.chunks(depth) {
        let width = block.iter().map(Vec::len).max().unwrap_or(0);
        for shard in 0..width {
            for group in block {
                if let Some(frame) = group.get(shard) {
                    out.push(frame.clone());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fec;
    use crate::schema::Telemetry;

    fn part97() -> AirConfig {
        AirConfig {
            service: Service::Part97,
            station_id: Some(StationId::parse("n0call").unwrap()),
            parity: 1,
            ..AirConfig::default()
        }
    }

    #[test]
    fn part97_frame_is_readable_and_named() {
        let payload = Telemetry::synthetic(7, 1_700_000_000_000, Service::Part97, Some("N0CALL"))
            .to_cbor()
            .unwrap();
        let frames = encode_object(&part97(), AirKind::Telemetry, 7, &payload).unwrap();
        assert!(frames.iter().all(|frame| frame.len() <= MTU));
        let joined = frames.concat();
        assert!(joined.windows(6).any(|window| window == b"N0CALL"));
        assert!(joined.windows(13).any(|window| window == b"temperature_c"));
        let decoded = decode(&frames[0]).unwrap();
        assert_eq!(decoded.station_id.unwrap().as_str(), "N0CALL");
        assert_eq!(decoded.service, Service::Part97);
        assert_eq!(decoded.shard, frames_payload_shard(&frames[0]));
    }

    fn frames_payload_shard(frame: &[u8]) -> Vec<u8> {
        decode(frame).unwrap().shard
    }

    #[test]
    fn compact_telemetry_fits_the_beta_lora_board() {
        use crate::schema::{LORA_BOARD_PAYLOAD, LORA_SMALL_PAYLOAD, Telemetry};
        let sample = Telemetry::synthetic(7, 1_700_000_000_900, Service::Part97, Some("N0CALL"));
        let body = sample.to_board_body().unwrap();
        let short = AirConfig {
            service: Service::Part97,
            station_id: Some(StationId::parse("N0CALL").unwrap()),
            parity: 1,
            ..AirConfig::default()
        };
        let frames = encode_object(&short, AirKind::Telemetry, 7, &body).unwrap();
        assert_eq!(frames.len(), 2, "one data shard and one parity shard");
        assert!(frames[0].len() <= LORA_SMALL_PAYLOAD, "{}", frames[0].len());
        let decoded = decode(&frames[0]).unwrap();
        assert_eq!(decoded.index, 0);
        assert_eq!(&decoded.shard[..body.len()], &body);
        let long_id = "W1AW/PORTABLE";
        assert_eq!(long_id.len(), 13);
        let long = AirConfig {
            service: Service::Part97,
            station_id: Some(StationId::parse(long_id).unwrap()),
            parity: 0,
            ..AirConfig::default()
        };
        let wide = encode_object(&long, AirKind::Telemetry, 7, &body).unwrap();
        assert_eq!(wide.len(), 1);
        assert!(wide[0].len() <= LORA_BOARD_PAYLOAD, "{}", wide[0].len());
        assert!(
            wide[0]
                .windows(long_id.len())
                .any(|window| window == long_id.as_bytes())
        );
    }

    #[test]
    fn a_flipped_bit_fails_the_crc() {
        let mut frame = encode_object(
            &AirConfig::default(),
            AirKind::Telemetry,
            1,
            b"temperature_c=21.5",
        )
        .unwrap()
        .remove(0);
        frame[10] ^= 0x01;
        assert_eq!(decode(&frame), Err(FrameFault::Crc));
    }

    #[test]
    fn amateur_mode_refuses_a_missing_or_personal_id() {
        let bare = AirConfig {
            service: Service::Part97,
            ..AirConfig::default()
        };
        let err = encode_object(&bare, AirKind::Telemetry, 1, b"abc").unwrap_err();
        assert!(err.to_string().contains("station ID"));
        assert!(StationId::parse("SIM").is_err());
        assert!(StationId::parse("student").is_err());
        assert_eq!(StationId::parse("w1aw/p").unwrap().as_str(), "W1AW/P");
    }

    #[test]
    fn parity_repairs_a_missing_data_shard() {
        let config = AirConfig {
            parity: 2,
            max_shard: 8,
            ..AirConfig::default()
        };
        let payload = b"0123456789abcdef0123456789abcdef";
        let frames = encode_object(&config, AirKind::Image, 3, payload).unwrap();
        assert!(frames.len() > 4);
        let mut kept = Vec::new();
        for (index, frame) in frames.iter().enumerate() {
            if index == 0 || index == 2 {
                continue;
            }
            kept.push(decode(frame).unwrap());
        }
        let restored = restore_from(&kept, payload.len());
        assert_eq!(restored, payload);
    }

    fn restore_from(frames: &[AirFrame], object_len: usize) -> Vec<u8> {
        let k = usize::from(frames[0].data_shards);
        let r = usize::from(frames[0].parity_shards);
        let mut data = vec![None; k];
        let mut parity = vec![None; r];
        for frame in frames {
            if usize::from(frame.index) < k {
                data[usize::from(frame.index)] = Some(frame.shard.clone());
            } else {
                parity[usize::from(frame.index - frame.data_shards)] = Some(frame.shard.clone());
            }
        }
        let shards = fec::restore(&data, &parity).unwrap();
        let mut raw = shards.concat();
        raw.truncate(object_len);
        raw
    }
}
