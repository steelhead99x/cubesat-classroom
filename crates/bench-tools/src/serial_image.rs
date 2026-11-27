//! Rebuild a photo sent by the XIAO bench CubeSat's `DOWNLINK` command.
//!
//! Save the Serial Monitor output to a text file. This finds the last
//! `IMG BEGIN ... IMG END` block, puts the hex chunks back in order, and checks
//! the byte count and CRC-32 that the CubeSat sent. Missing chunks are reported
//! by offset and filled with zeros, so a crew can see which part of the picture
//! was lost. Other lines (`CSC1`, `REC`, `# bench`) are ignored.
//!
//! The photo counts as intact only if received chunks cover every byte AND the
//! CRC-32 matches. A gap is a failure even when the CRC happens to match: the
//! zero fill can equal the lost bytes (for example a run of zeros in the JPEG).

use std::collections::HashMap;
use std::io::Write;

use crate::util::{Fail, fail};

/// The firmware refuses photos over 120000 bytes. Anything far larger is a
/// garbled header, and believing it would try to allocate gigabytes.
pub const MAX_PHOTO_BYTES: u64 = 1_000_000;

/// Fields from an `IMG BEGIN` line, such as `n`, `bytes` and `crc32`.
pub type Header = HashMap<String, String>;

/// `key=value` words from the rest of an `IMG BEGIN` line.
#[must_use]
pub fn parse_fields(text: &str) -> Header {
    text.split_whitespace()
        .filter_map(|part| part.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn begin_rest(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("IMG BEGIN")?;
    // `\b` after BEGIN: the next character must not continue a word.
    match rest.chars().next() {
        Some(ch) if ch.is_alphanumeric() || ch == '_' => None,
        _ => Some(rest),
    }
}

/// Every `IMG BEGIN` block in the log, in order, with its chunk lines. A block
/// whose `IMG END` never arrived is still returned.
#[must_use]
pub fn blocks(text: &str) -> Vec<(Header, Vec<String>)> {
    let mut out = Vec::new();
    let mut current: Option<(Header, Vec<String>)> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(rest) = begin_rest(line) {
            // A new BEGIN before END replaces the open block: the firmware
            // restarted the transfer.
            current = Some((parse_fields(rest), Vec::new()));
        } else if line == "IMG END" {
            if let Some(block) = current.take() {
                out.push(block);
            }
        } else if let Some((_, chunks)) = current.as_mut() {
            chunks.push(line.to_string());
        }
    }
    if let Some(block) = current {
        out.push(block);
    }
    out
}

/// Byte ranges `[start, end)` not covered by any received chunk.
#[must_use]
pub fn find_gaps(have: &[(usize, usize)], expected: usize) -> Vec<(usize, usize)> {
    let mut sorted = have.to_vec();
    sorted.sort_unstable();
    let mut gaps = Vec::new();
    let mut position = 0;
    for (start, end) in sorted {
        if start > position {
            gaps.push((position, start));
        }
        position = position.max(end);
    }
    if position < expected {
        gaps.push((position, expected));
    }
    gaps
}

fn chunk(line: &str) -> Option<(u64, &str)> {
    let rest = line.strip_prefix("IMG ")?;
    let (offset, data) = rest.split_once(' ')?;
    if offset.is_empty() || !offset.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if data.is_empty() || !data.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    // An offset too large for u64 is past any photo anyway.
    Some((offset.parse().unwrap_or(u64::MAX), data))
}

fn hex(data: &str) -> Vec<u8> {
    data.as_bytes()
        .chunks(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap_or("00");
            u8::from_str_radix(text, 16).unwrap_or(0)
        })
        .collect()
}

/// A rebuilt photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rebuilt {
    /// The photo bytes, zero where chunks were lost.
    pub image: Vec<u8>,
    /// Byte ranges no chunk covered.
    pub gaps: Vec<(usize, usize)>,
    /// The CRC-32 the CubeSat sent.
    pub crc_sent: u128,
    /// The CRC-32 of `image`.
    pub crc_got: u32,
}

fn parse_int(text: &str, radix: u32) -> Option<i128> {
    let t = text.trim();
    let t = if radix == 16 {
        t.strip_prefix("0x")
            .or_else(|| t.strip_prefix("0X"))
            .unwrap_or(t)
    } else {
        t
    };
    i128::from_str_radix(&t.replace('_', ""), radix).ok()
}

/// Put the chunks of one block back together.
///
/// # Errors
/// If the header has no byte count or CRC, claims no bytes, or claims more
/// than a CubeSat photo can be.
pub fn rebuild(header: &Header, chunks: &[String]) -> Result<Rebuilt, Fail> {
    let missing = || Fail("The IMG BEGIN line is missing bytes= or crc32=.".into());
    let expected = header
        .get("bytes")
        .and_then(|v| parse_int(v, 10))
        .ok_or_else(missing)?;
    let crc_sent = header
        .get("crc32")
        .and_then(|v| parse_int(v, 16))
        .ok_or_else(missing)?;
    if expected <= 0 {
        return fail("The IMG BEGIN line has no photo bytes. Send PHOTO again, then DOWNLINK.");
    }
    if expected > i128::from(MAX_PHOTO_BYTES) {
        return fail(format!(
            "The IMG BEGIN line claims {expected} bytes, more than the {MAX_PHOTO_BYTES} a CubeSat photo can be. \
             The line was probably garbled: send DOWNLINK again."
        ));
    }
    let expected = expected as usize;
    let mut image = vec![0u8; expected];
    let mut have = Vec::new();
    for line in chunks {
        let Some((offset, data)) = chunk(line) else {
            continue;
        };
        if data.len() % 2 == 1 {
            continue; // a cut-off line; treat it as lost
        }
        let piece = hex(data);
        let end = offset.saturating_add(piece.len() as u64);
        if piece.is_empty() || end > expected as u64 {
            continue;
        }
        let (start, end) = (offset as usize, end as usize);
        image[start..end].copy_from_slice(&piece);
        have.push((start, end));
    }
    let gaps = find_gaps(&have, expected);
    let crc_got = crc32fast::hash(&image);
    Ok(Rebuilt {
        image,
        gaps,
        crc_sent: crc_sent.max(0) as u128,
        crc_got,
    })
}

/// Rebuild the last photo in `log` and write it to `output`.
///
/// Returns `Ok(true)` when the photo arrived intact, `Ok(false)` when it is
/// incomplete or damaged (the file is still written).
///
/// # Errors
/// If there is no photo in the log, the header is unusable, or the output
/// cannot be written.
pub fn run(log: &str, output: &str, out: &mut impl Write) -> Result<bool, Fail> {
    let all = blocks(log);
    let Some((header, chunks)) = all.last() else {
        return fail(
            "No IMG BEGIN line found. Send PHOTO, then DOWNLINK, and save all of the output.",
        );
    };
    let rebuilt = rebuild(header, chunks)?;
    std::fs::write(output, &rebuilt.image).map_err(|e| Fail(format!("{output}: {e}")))?;
    let n = header.get("n").map_or("?", String::as_str);
    writeln!(
        out,
        "photo {n}: {} bytes written to {output}",
        rebuilt.image.len()
    )?;
    if !rebuilt.gaps.is_empty() {
        let lost: usize = rebuilt.gaps.iter().map(|(s, e)| e - s).sum();
        writeln!(
            out,
            "missing {lost} bytes in {} gap(s), filled with zeros:",
            rebuilt.gaps.len()
        )?;
        for (start, end) in &rebuilt.gaps {
            writeln!(out, "  bytes {start} to {}", end - 1)?;
        }
    }
    let got = rebuilt.crc_got;
    let crc_ok = u128::from(got) == rebuilt.crc_sent;
    if crc_ok && rebuilt.gaps.is_empty() {
        writeln!(
            out,
            "CRC-32 {got:08X} matches and every byte arrived: the photo arrived intact"
        )?;
        return Ok(true);
    }
    if crc_ok {
        writeln!(
            out,
            "CRC-32 {got:08X} matches only because the zero fill equals the lost bytes."
        )?;
        writeln!(
            out,
            "The transfer is incomplete: send DOWNLINK again to fill the gaps."
        )?;
    } else {
        writeln!(
            out,
            "CRC-32 {got:08X} does not match the {:08X} the CubeSat sent",
            rebuilt.crc_sent
        )?;
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    const CHUNK: usize = 48;

    fn fixture() -> Vec<u8> {
        // A small JPEG-shaped fixture with a 48-byte run of zeros at offset 96.
        let mut data = vec![0xff, 0xd8, 0xff, 0xe0];
        data.extend(0u8..92);
        data.extend(std::iter::repeat_n(0u8, CHUNK));
        for _ in 0..2 {
            data.extend(200u8..=255);
        }
        data.extend([0xff, 0xd9]);
        data
    }

    fn capture(data: &[u8], drop: &[usize]) -> String {
        let crc = crc32fast::hash(data);
        let mut text = format!(
            "CSC1 seq=1 noise\nIMG BEGIN n=1 bytes={} crc32={crc:08X}\n",
            data.len()
        );
        for off in (0..data.len()).step_by(CHUNK) {
            if drop.contains(&off) {
                continue;
            }
            let end = (off + CHUNK).min(data.len());
            let hex = data[off..end].iter().fold(String::new(), |mut s, b| {
                let _ = write!(s, "{b:02X}");
                s
            });
            let _ = writeln!(text, "IMG {off} {hex}");
        }
        text.push_str("IMG END\n");
        text
    }

    fn go(text: &str) -> (Result<bool, Fail>, Vec<u8>) {
        let dir = std::env::temp_dir().join(format!(
            "serial-image-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("photo.jpg");
        let result = run(text, out.to_str().unwrap(), &mut Vec::new());
        let image = std::fs::read(&out).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        (result, image)
    }

    #[test]
    fn complete_transfer_is_intact() {
        let (result, image) = go(&capture(&fixture(), &[]));
        assert_eq!(result, Ok(true));
        assert_eq!(image, fixture());
    }

    #[test]
    fn missing_zero_chunk_is_incomplete_even_though_crc_matches() {
        // The lost chunk is all zeros, so the zero fill restores the CRC
        // exactly. That must still be reported as an incomplete transfer.
        let data = fixture();
        let text = capture(&data, &[96]);
        let (header, chunks) = blocks(&text).pop().unwrap();
        let rebuilt = rebuild(&header, &chunks).unwrap();
        assert_eq!(
            rebuilt.crc_sent,
            u128::from(rebuilt.crc_got),
            "fixture must reproduce the CRC coincidence"
        );
        assert_eq!(rebuilt.gaps, vec![(96, 144)]);
        assert_eq!(go(&text).0, Ok(false));
    }

    #[test]
    fn missing_data_chunk_fails() {
        assert_eq!(go(&capture(&fixture(), &[48])).0, Ok(false));
    }

    #[test]
    fn missing_last_chunk_fails() {
        let last = (fixture().len() - 1) / CHUNK * CHUNK;
        assert_eq!(go(&capture(&fixture(), &[last])).0, Ok(false));
    }

    #[test]
    fn readings_between_image_lines_are_ignored() {
        // The firmware keeps sampling during a downlink, so CSC1 lines can sit
        // between IMG lines.
        let mut mixed = String::new();
        for line in capture(&fixture(), &[]).lines() {
            mixed.push_str(line);
            mixed.push('\n');
            if line.starts_with("IMG ") && !line.starts_with("IMG BEGIN") && line != "IMG END" {
                mixed.push_str("CSC1 seq=9 temp_mc=21500 volt_mv=3900 light_lux=400\n");
            }
        }
        let (result, image) = go(&mixed);
        assert_eq!(result, Ok(true));
        assert_eq!(image, fixture());
    }

    #[test]
    fn empty_photo_is_not_intact() {
        let (result, _) = go("IMG BEGIN n=1 bytes=0 crc32=0\nIMG END\n");
        assert!(result.unwrap_err().0.contains("no photo bytes"));
    }

    #[test]
    fn huge_byte_count_is_refused_without_allocating() {
        let (result, _) = go("IMG BEGIN n=1 bytes=99999999999999 crc32=0\nIMG END\n");
        assert!(result.unwrap_err().0.contains("garbled"));
    }

    #[test]
    fn no_block_is_reported() {
        let (result, _) = go("CSC1 seq=1\n");
        assert!(result.unwrap_err().0.contains("No IMG BEGIN"));
    }

    #[test]
    fn gap_finder() {
        assert_eq!(
            find_gaps(&[(0, 10), (20, 30)], 40),
            vec![(10, 20), (30, 40)]
        );
        assert_eq!(find_gaps(&[(0, 10), (5, 40)], 40), vec![]);
        assert_eq!(find_gaps(&[], 0), vec![]);
    }
}
