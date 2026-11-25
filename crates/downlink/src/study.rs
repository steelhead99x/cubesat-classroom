//! Fixed desk-simulation runs. The numbers come from this code, not from a radio.

use std::collections::HashSet;
use std::time::Instant;

use serde::Serialize;

use crate::assemble::{Assembler, Heard};
use crate::channel::{Channel, ChannelConfig};
use crate::frame::{AirConfig, AirKind, Error, MTU, Service, encode_object, interleave};

/// One object to run through the virtual channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimObject {
    /// Air kind.
    pub kind: AirKind,
    /// Object sequence.
    pub sequence: u32,
    /// Clear payload.
    pub payload: Vec<u8>,
}

/// Counts from one [`simulate`] call.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SimReport {
    /// Objects returned by the assembler.
    pub recovered: u64,
    /// Objects that never came back. `recovered + missing` is the object count.
    pub missing: u64,
    /// Recovered objects that needed parity.
    pub fec_repairs: u64,
    /// Extra copies of a shard.
    pub duplicates: u64,
    /// Air frames before loss.
    pub frames_sent: u64,
    /// Air frames the receivers kept, counting each receiver.
    pub frames_heard: u64,
    /// Frames that failed the CRC. The loss model drops frames; it does not flip bits.
    pub crc_failures: u64,
}

/// Encode, optionally interleave, apply the channel, and reassemble.
pub fn simulate(
    air: &AirConfig,
    channel: &ChannelConfig,
    objects: &[SimObject],
) -> Result<SimReport, Error> {
    air.check()?;
    let mut groups = Vec::with_capacity(objects.len());
    for object in objects {
        groups.push(encode_object(
            air,
            object.kind,
            object.sequence,
            &object.payload,
        )?);
    }
    let wire = interleave(&groups, usize::from(air.interleave_depth));
    let frames_sent = wire.len() as u64;
    let heard = Channel::new(channel)?.run(&wire);
    let frames_heard = heard.len() as u64;
    let mut assembler = Assembler::new();
    for frame in heard {
        assembler.push(frame);
    }
    assembler.finish();
    let metrics = assembler.metrics();
    Ok(SimReport {
        recovered: metrics.recovered,
        missing: (objects.len() as u64).saturating_sub(metrics.recovered),
        fec_repairs: metrics.fec_repairs,
        duplicates: metrics.duplicates,
        frames_sent,
        frames_heard,
        crc_failures: metrics.crc_failures,
    })
}

/// The burst example: six consecutive drops, with and without interleaving.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BurstDemo {
    /// Objects in the example.
    pub objects: u32,
    /// Payload shards per object.
    pub data_shards: u8,
    /// Parity shards per object.
    pub parity_shards: u8,
    /// How many leading air frames were deleted.
    pub dropped_frames: u32,
    /// Objects recovered when each object's shards sit together.
    pub recovered_without_interleave: u32,
    /// Objects recovered when shard 0 of every object is sent, then shard 1, and so on.
    pub recovered_with_interleave: u32,
}

/// Three receivers, no loss: every object arrives once and the extra copies are counted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DedupDemo {
    /// Objects sent.
    pub objects: u32,
    /// Virtual receivers.
    pub relays: u8,
    /// Objects recovered. Equal to `objects` when nothing is lost.
    pub recovered: u32,
    /// Extra copies. Two extra receivers and one shard each is `2 * objects`.
    pub duplicates: u32,
}

/// One random-loss run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Scenario {
    /// Short name.
    pub name: String,
    /// What the row is comparing.
    pub summary: String,
    /// Objects offered to the channel.
    pub objects: u32,
    /// Virtual receivers.
    pub relays: u8,
    /// Loss probability in parts per thousand.
    pub loss_per_mille: u16,
    /// Burst length, including the first drop.
    pub burst: u16,
    /// Parity shards.
    pub parity: u8,
    /// Interleave depth.
    pub interleave_depth: u16,
    /// Objects recovered.
    pub recovered: u32,
    /// Objects lost.
    pub missing: u32,
    /// Objects rebuilt from parity.
    pub fec_repairs: u32,
    /// Extra copies.
    pub duplicates: u32,
    /// Air frames before loss.
    pub frames_sent: u32,
    /// Receiver copies kept.
    pub frames_heard: u32,
}

/// All of the built-in desk runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Studies {
    /// Always `desk-simulation`.
    pub evidence: String,
    /// Always false. This function does not transmit.
    pub transmits: bool,
    /// Air frame limit.
    pub mtu_bytes: usize,
    /// What a reader should not conclude.
    pub note: String,
    /// Hand-built burst.
    pub burst: BurstDemo,
    /// Hand-built duplicate suppression.
    pub dedup: DedupDemo,
    /// Seeded loss runs.
    pub scenarios: Vec<Scenario>,
}

/// A timed recovery run for the release binary. Not a radio measurement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Bench {
    /// Always `desk-simulation`.
    pub evidence: String,
    /// Objects in the timed run.
    pub objects: u32,
    /// Objects recovered.
    pub recovered: u32,
    /// Objects lost.
    pub missing: u32,
    /// Bytes of air frames before loss.
    pub air_bytes: u64,
    /// Wall time of encode, loss, and reassembly.
    pub elapsed_ms: u128,
    /// `objects / elapsed`, rounded down. Zero when the clock did not move.
    pub objects_per_second: u64,
}

/// Run the built-in studies.
pub fn studies() -> Studies {
    Studies {
        evidence: crate::EVIDENCE_DESK.into(),
        transmits: false,
        mtu_bytes: MTU,
        note: "Virtual receivers and invented RSSI. No radio, no certification, and no permission to transmit.".into(),
        burst: burst_demo(),
        dedup: dedup_demo(),
        scenarios: vec![
            scenario(&LossSpec {
                name: "ten-percent-no-parity",
                summary: "One receiver, 10% of frames dropped, no parity.",
                count: 400,
                relays: 1,
                loss_per_mille: 100,
                burst: 1,
                parity: 0,
                interleave_depth: 1,
                seed: 0x0915_0001,
            }),
            scenario(&LossSpec {
                name: "ten-percent-one-parity",
                summary: "Same draw settings with one Reed-Solomon parity shard.",
                count: 400,
                relays: 1,
                loss_per_mille: 100,
                burst: 1,
                parity: 1,
                interleave_depth: 1,
                seed: 0x0915_0001,
            }),
            scenario(&LossSpec {
                name: "forty-percent-one-receiver",
                summary: "One receiver, 40% of frames dropped, no parity.",
                count: 300,
                relays: 1,
                loss_per_mille: 400,
                burst: 1,
                parity: 0,
                interleave_depth: 1,
                seed: 0x0915_0002,
            }),
            scenario(&LossSpec {
                name: "forty-percent-three-receivers",
                summary: "Three receivers, each dropping 40% on its own, duplicates removed.",
                count: 300,
                relays: 3,
                loss_per_mille: 400,
                burst: 1,
                parity: 0,
                interleave_depth: 1,
                seed: 0x0915_0002,
            }),
        ],
    }
}

/// Plain-text form of [`studies`] for the command line.
pub fn text_report(studies: &Studies) -> String {
    let mut out = String::new();
    out.push_str("915 MHz downlink desk simulation. This program does not transmit.\n");
    out.push_str(&format!(
        "Evidence: {}. Air frame limit: {} bytes. {}\n",
        studies.evidence, studies.mtu_bytes, studies.note
    ));
    out.push_str(&format!(
        "Beta LoRa board: telemetry is one cleartext packet (18-byte body plus the air header).\n\
         A station ID of up to 12 characters stays within {small} bytes. The longest ID this program\n\
         accepts stays within {board} bytes. Phones still use the MoQ tracks. The board keyboard does not transmit.\n\n",
        small = crate::LORA_SMALL_PAYLOAD,
        board = crate::LORA_BOARD_PAYLOAD,
    ));
    let burst = &studies.burst;
    out.push_str(&format!(
        "Burst of {} frames across {} objects ({} data shards + {} parity):\n",
        burst.dropped_frames, burst.objects, burst.data_shards, burst.parity_shards
    ));
    out.push_str(&format!(
        "  without interleave: {} of {} objects recovered\n",
        burst.recovered_without_interleave, burst.objects
    ));
    out.push_str(&format!(
        "  with interleave:    {} of {} objects recovered\n\n",
        burst.recovered_with_interleave, burst.objects
    ));
    let dedup = &studies.dedup;
    out.push_str(&format!(
        "Duplicate suppression: {} receivers, no loss, {} objects, {} recovered, {} extra copies dropped.\n\n",
        dedup.relays, dedup.objects, dedup.recovered, dedup.duplicates
    ));
    out.push_str(
        "Seeded loss (each row is one virtual pass; RSSI values in the health track are invented):\n",
    );
    for row in &studies.scenarios {
        out.push_str(&format!(
            "  {:<32} {:>3} recovered, {:>3} missing, {:>3} repaired, {:>4} duplicates  ({})\n",
            row.name, row.recovered, row.missing, row.fec_repairs, row.duplicates, row.summary
        ));
    }
    out
}

/// Time 10 000 small objects through a 10% loss channel with one parity shard.
pub fn bench() -> Bench {
    let objects = 10_000;
    let air = AirConfig {
        parity: 1,
        ..AirConfig::default()
    };
    let channel = ChannelConfig {
        relays: 1,
        loss_per_mille: 100,
        burst: 1,
        reorder: 0,
        seed: 0x0915_00B0,
    };
    let payload = vec![0x5A; 64];
    let batch: Vec<_> = (0..objects)
        .map(|sequence| SimObject {
            kind: AirKind::Telemetry,
            sequence,
            payload: payload.clone(),
        })
        .collect();
    let started = Instant::now();
    let report = simulate(&air, &channel, &batch).expect("bench configuration fits the frame");
    let elapsed = started.elapsed();
    let elapsed_ms = elapsed.as_millis();
    let objects_per_second = u128::from(objects)
        .saturating_mul(1000)
        .checked_div(elapsed_ms)
        .map_or(0, |rate| u64::try_from(rate).unwrap_or(u64::MAX));
    Bench {
        evidence: crate::EVIDENCE_DESK.into(),
        objects,
        recovered: u32::try_from(report.recovered).unwrap_or(u32::MAX),
        missing: u32::try_from(report.missing).unwrap_or(u32::MAX),
        air_bytes: report
            .frames_sent
            .saturating_mul(u64::try_from(MTU).unwrap_or(255)),
        elapsed_ms,
        objects_per_second,
    }
}

/// Plain-text form of [`bench`].
pub fn bench_text(bench: &Bench) -> String {
    format!(
        "Desk-simulation bench, not a radio measurement. Evidence: {}.\n\
         {} objects, {} recovered, {} missing, about {} air-frame bytes, {} ms, {} objects/s.\n\
         The byte count uses the 255-byte ceiling, not the shorter real frames.\n",
        bench.evidence,
        bench.objects,
        bench.recovered,
        bench.missing,
        bench.air_bytes,
        bench.elapsed_ms,
        bench.objects_per_second
    )
}

fn burst_demo() -> BurstDemo {
    let payload = vec![0xA5; 32];
    let plain = AirConfig {
        service: Service::Simulation,
        parity: 2,
        max_shard: 8,
        interleave_depth: 1,
        ..AirConfig::default()
    };
    let mut spread = plain.clone();
    spread.interleave_depth = 4;
    let drops: Vec<usize> = (0..6).collect();
    BurstDemo {
        objects: 4,
        data_shards: 4,
        parity_shards: 2,
        dropped_frames: 6,
        recovered_without_interleave: recover_with_drops(&plain, &payload, 4, &drops),
        recovered_with_interleave: recover_with_drops(&spread, &payload, 4, &drops),
    }
}

fn dedup_demo() -> DedupDemo {
    let air = AirConfig {
        parity: 0,
        ..AirConfig::default()
    };
    let report = simulate(
        &air,
        &ChannelConfig {
            relays: 3,
            loss_per_mille: 0,
            burst: 1,
            reorder: 0,
            seed: 1,
        },
        &objects(50, &[7_u8; 24]),
    )
    .expect("dedup scenario fits the frame");
    DedupDemo {
        objects: 50,
        relays: 3,
        recovered: u32::try_from(report.recovered).unwrap_or(u32::MAX),
        duplicates: u32::try_from(report.duplicates).unwrap_or(u32::MAX),
    }
}

struct LossSpec {
    name: &'static str,
    summary: &'static str,
    count: u32,
    relays: u8,
    loss_per_mille: u16,
    burst: u16,
    parity: u8,
    interleave_depth: u16,
    seed: u64,
}

fn scenario(spec: &LossSpec) -> Scenario {
    let air = AirConfig {
        parity: spec.parity,
        interleave_depth: spec.interleave_depth,
        ..AirConfig::default()
    };
    let report = simulate(
        &air,
        &ChannelConfig {
            relays: spec.relays,
            loss_per_mille: spec.loss_per_mille,
            burst: spec.burst,
            reorder: 0,
            seed: spec.seed,
        },
        &objects(spec.count, &[0x11; 80]),
    )
    .expect("loss scenario fits the frame");
    Scenario {
        name: spec.name.into(),
        summary: spec.summary.into(),
        objects: spec.count,
        relays: spec.relays,
        loss_per_mille: spec.loss_per_mille,
        burst: spec.burst,
        parity: spec.parity,
        interleave_depth: spec.interleave_depth,
        recovered: u32::try_from(report.recovered).unwrap_or(u32::MAX),
        missing: u32::try_from(report.missing).unwrap_or(u32::MAX),
        fec_repairs: u32::try_from(report.fec_repairs).unwrap_or(u32::MAX),
        duplicates: u32::try_from(report.duplicates).unwrap_or(u32::MAX),
        frames_sent: u32::try_from(report.frames_sent).unwrap_or(u32::MAX),
        frames_heard: u32::try_from(report.frames_heard).unwrap_or(u32::MAX),
    }
}

fn objects(count: u32, payload: &[u8]) -> Vec<SimObject> {
    (0..count)
        .map(|sequence| SimObject {
            kind: AirKind::Telemetry,
            sequence,
            payload: payload.to_vec(),
        })
        .collect()
}

fn recover_with_drops(air: &AirConfig, payload: &[u8], count: u32, drops: &[usize]) -> u32 {
    let groups: Vec<_> = (0..count)
        .map(|sequence| {
            encode_object(air, AirKind::Image, sequence, payload).expect("burst scenario fits")
        })
        .collect();
    let wire = interleave(&groups, usize::from(air.interleave_depth));
    let skip: HashSet<usize> = drops.iter().copied().collect();
    let mut assembler = Assembler::new();
    for (index, frame) in wire.into_iter().enumerate() {
        if skip.contains(&index) {
            continue;
        }
        assembler.push(Heard {
            relay: 0,
            rssi_dbm: -92,
            snr_db_x10: 80,
            frame,
        });
    }
    u32::try_from(assembler.metrics().recovered).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interleave_saves_the_object_a_burst_erases() {
        let studies = studies();
        assert!(!studies.transmits);
        assert_eq!(studies.evidence, "desk-simulation");
        assert_eq!(studies.burst.recovered_without_interleave, 3);
        assert_eq!(studies.burst.recovered_with_interleave, 4);
        assert_eq!(studies.dedup.recovered, studies.dedup.objects);
        assert_eq!(studies.dedup.duplicates, studies.dedup.objects * 2);
    }

    #[test]
    fn parity_and_extra_receivers_lose_fewer_objects() {
        let studies = studies();
        let rows: std::collections::BTreeMap<_, _> = studies
            .scenarios
            .iter()
            .map(|row| (row.name.as_str(), row))
            .collect();
        let plain = rows["ten-percent-no-parity"];
        let coded = rows["ten-percent-one-parity"];
        let one = rows["forty-percent-one-receiver"];
        let three = rows["forty-percent-three-receivers"];
        // Locked to the seeded virtual channel. A crate upgrade that changes
        // the random sequence should update these counts in the same change.
        assert_eq!((plain.recovered, plain.missing), (364, 36));
        assert_eq!(
            (coded.recovered, coded.missing, coded.fec_repairs),
            (395, 5, 34)
        );
        assert_eq!((one.recovered, one.missing), (182, 118));
        assert_eq!(
            (three.recovered, three.missing, three.duplicates),
            (285, 15, 245)
        );
        for row in studies.scenarios {
            assert_eq!(row.recovered + row.missing, row.objects);
        }
    }

    #[test]
    fn reordering_does_not_lose_a_clean_object() {
        let air = AirConfig {
            parity: 1,
            ..AirConfig::default()
        };
        let report = simulate(
            &air,
            &ChannelConfig {
                relays: 2,
                loss_per_mille: 0,
                burst: 1,
                reorder: 5,
                seed: 9,
            },
            &objects(30, b"reorder-me-please"),
        )
        .unwrap();
        assert_eq!(report.recovered, 30);
        assert_eq!(report.missing, 0);
        assert!(report.duplicates > 0);
    }
}
