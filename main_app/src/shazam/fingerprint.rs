use std::collections::HashMap;
use crate::shazam::spectogram::Peak;

const MAX_FREQ_BITS: u32 = 9;
const MAX_DELTA_BITS: u32 = 14;
const TARGET_ZONE_SIZE: usize = 5;


/// Represents a couple containing anchor time and song ID
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct Couple {
//     pub anchor_time_ms: u32,
//     pub song_id: i64,
// }

use crate::shazam::Couple; //use this instead

/// Generates fingerprints from a list of peaks and stores them in a HashMap.
/// Each fingerprint consists of an address and a couple.
/// The address is a hash. The couple contains the anchor time and the song ID.
pub fn fingerprint(peaks: Vec<Peak>, song_id: i64) -> HashMap<u32, Couple> {
    let mut fingerprints = HashMap::new();

    for (i, anchor) in peaks.iter().enumerate() {
        let end = (i + 1 + TARGET_ZONE_SIZE).min(peaks.len());
        
        for j in (i + 1)..end {
            let target = &peaks[j];
            let address = create_address(anchor, target);
            let anchor_time_ms = (anchor.time * 1000.0) as u32;
            
            fingerprints.insert(
                address,
                Couple {
                    anchor_time_ms,
                    song_id,
                },
            );
        }
    }

    fingerprints
}

/// Creates a unique address for a pair of anchor and target points.
/// The address is a 32-bit integer where certain bits represent the frequency of
/// the anchor and target points, and other bits represent the time difference (delta time)
/// between them. This function combines these components into a single address (a hash).
fn create_address(anchor: &Peak, target: &Peak) -> u32 {
    let anchor_freq = anchor.freq.re as i32;
    let target_freq = target.freq.re as i32;
    let delta_ms = ((target.time - anchor.time) * 1000.0) as u32;

    // Combine the frequency of the anchor, target, and delta time into a 32-bit address
    let address = ((anchor_freq as u32) << 23) | ((target_freq as u32) << 14) | delta_ms;

    address
}