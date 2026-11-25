//! Reed-Solomon erasure coding over air shards.
//!
//! The implementation is the `reed-solomon-simd` crate (Leopard-RS). Parity
//! shards are redundant. A listener who received the payload shards can read
//! them without using the parity.

use crate::frame::Error;

/// Build `parity` recovery shards for `data`. Every shard must be the same even length.
pub(crate) fn parity(data: &[Vec<u8>], parity: usize) -> Result<Vec<Vec<u8>>, Error> {
    if parity == 0 {
        return Ok(Vec::new());
    }
    if data.is_empty() || !reed_solomon_simd::ReedSolomonEncoder::supports(data.len(), parity) {
        return Err(Error::Fec(format!(
            "Reed-Solomon does not support {} data shards and {parity} parity shards",
            data.len()
        )));
    }
    reed_solomon_simd::encode(data.len(), parity, data.iter())
        .map_err(|err| Error::Fec(err.to_string()))
}

/// Restore every payload shard. `None` slots are missing. Returns an error when
/// too many shards are missing for a unique restore.
pub(crate) fn restore(
    data: &[Option<Vec<u8>>],
    parity: &[Option<Vec<u8>>],
) -> Result<Vec<Vec<u8>>, Error> {
    let k = data.len();
    let r = parity.len();
    if data.iter().all(Option::is_some) {
        return Ok(data
            .iter()
            .map(|shard| shard.clone().expect("checked"))
            .collect());
    }
    let have = data.iter().filter(|shard| shard.is_some()).count()
        + parity.iter().filter(|shard| shard.is_some()).count();
    if r == 0 || have < k {
        return Err(Error::Fec("not enough shards to restore the object".into()));
    }
    let originals: Vec<(usize, Vec<u8>)> = data
        .iter()
        .enumerate()
        .filter_map(|(index, shard)| shard.clone().map(|bytes| (index, bytes)))
        .collect();
    let recovery: Vec<(usize, Vec<u8>)> = parity
        .iter()
        .enumerate()
        .filter_map(|(index, shard)| shard.clone().map(|bytes| (index, bytes)))
        .collect();
    let restored = reed_solomon_simd::decode(
        k,
        r,
        originals
            .iter()
            .map(|(index, bytes)| (*index, bytes.as_slice())),
        recovery
            .iter()
            .map(|(index, bytes)| (*index, bytes.as_slice())),
    )
    .map_err(|err| Error::Fec(err.to_string()))?;
    let mut out = Vec::with_capacity(k);
    for (index, shard) in data.iter().enumerate() {
        if let Some(shard) = shard {
            out.push(shard.clone());
        } else {
            let Some(restored) = restored.get(&index) else {
                return Err(Error::Fec(
                    "recovery did not return a missing payload shard".into(),
                ));
            };
            out.push(restored.clone());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_missing_shards_come_back() {
        let data: Vec<Vec<u8>> = (0..4).map(|n| vec![n; 8]).collect();
        let parity = parity(&data, 2).unwrap();
        // Shards 1 and 3 are gone. Two parity shards are enough to rebuild them.
        let have = vec![Some(data[0].clone()), None, Some(data[2].clone()), None];
        let extra = vec![Some(parity[0].clone()), Some(parity[1].clone())];
        let restored = restore(&have, &extra).unwrap();
        assert_eq!(restored, data);
    }

    #[test]
    fn three_erasures_with_two_parity_shards_fail() {
        let data: Vec<Vec<u8>> = (0..4).map(|n| vec![n; 8]).collect();
        let parity = parity(&data, 2).unwrap();
        let have = vec![None, None, None, Some(data[3].clone())];
        let extra = vec![Some(parity[0].clone()), Some(parity[1].clone())];
        assert!(restore(&have, &extra).is_err());
    }
}
