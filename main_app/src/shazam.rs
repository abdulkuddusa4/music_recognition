#![allow(dead_code,unused_variables,unused_imports)]

pub mod spectogram;
pub mod fingerprint;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use std::sync::Arc;
use cot::db::Database;
use cot::db::query;
use cot::db::Auto;

use crate::models::FingerPrint;
use crate::models::Song;

const TARGET_ZONE_SIZE: usize = 5;

#[derive(Debug, Clone)]
pub struct Match {
    pub song_id: i64,
    pub youtube_url: String,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Couple {
    pub anchor_time_ms: u32,
    pub song_id: i64,
}

#[derive(Debug)]
pub enum MatchError {
    SpectrogramError(String),
    DatabaseError(String),
    SongNotFound(u32),
}

impl std::fmt::Display for MatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchError::SpectrogramError(msg) => write!(f, "Spectrogram error: {}", msg),
            MatchError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MatchError::SongNotFound(id) => write!(f, "Song not found: {}", id),
        }
    }
}

impl std::error::Error for MatchError {}

// Trait for database operations - implement this for your specific database
pub trait DatabaseClient {
    fn get_couples(&self, addresses: &[u32]) -> Result<HashMap<u32, Vec<Couple>>, MatchError>;
    fn get_song_by_id(&self, song_id: u32) -> Result<Option<Song>, MatchError>;
}

async fn get_couples(
    db: &Arc<Database>,
    addresses: &[u32]
)->Result<HashMap<u32, Vec<Couple>>, MatchError>
{

    let mut results = HashMap::<u32, Vec<Couple>>::new();
    for address in addresses{
        let addr = *address;
        let mut cpls = Vec::<Couple>::new();
        for cpl in query!(FingerPrint, $address == addr).all(db).await.unwrap(){
            cpls.push(Couple{
                anchor_time_ms: cpl.anchor_time_ms,
                song_id: cpl.song_id
            });
        }
        results.insert(*address, cpls);
    }
    return Ok(results);
}

async fn get_song_by_id(
    db: &Arc<Database>,
    song_id: i64
)->Result<Option<Song>, MatchError>
{
    let mut songs = query!(Song, $id == Auto::from(song_id)).all(db).await.unwrap();

    if songs.len() > 0{
        return Ok(Some(songs.remove(0)));
    }
    return Ok(None);
}

/// Analyzes the audio sample to find matching songs in the database.
pub async fn find_matches(
    db_client: &Arc<Database>,
    audio_sample: &[f64],
    audio_duration: f64,
    sample_rate: usize,
) -> Result<(Vec<Match>, Duration), MatchError> {
    let start_time = Instant::now();

    let spectrogram = spectogram::spectrogram(audio_sample, sample_rate)
        .map_err(|e| MatchError::SpectrogramError(e.to_string()))?;

    let peaks = spectogram::extract_peaks(&spectrogram, audio_duration);
    let sample_fingerprint = fingerprint::fingerprint(peaks, generate_unique_id());

    let mut sample_fingerprint_map: HashMap<u32, u32> = HashMap::new();
    for (address, couple) in sample_fingerprint {
        sample_fingerprint_map.insert(address, couple.anchor_time_ms);
    }

    let (matches, _) = find_matches_fgp(&sample_fingerprint_map, db_client).await?;

    Ok((matches, start_time.elapsed()))
}

/// Uses the sample fingerprint to find matching songs in the database.
pub async fn find_matches_fgp(
    sample_fingerprint: &HashMap<u32, u32>,
    db_client: &Arc<Database>,
) -> Result<(Vec<Match>, Duration), MatchError> {
    let start_time = Instant::now();

    let addresses: Vec<u32> = sample_fingerprint.keys().copied().collect();

    let couples_map = get_couples(db_client, &addresses).await?;

    let mut matches: HashMap<i64, Vec<[u32; 2]>> = HashMap::new();
    let mut timestamps: HashMap<i64, u32> = HashMap::new();
    let mut target_zones: HashMap<i64, HashMap<u32, i32>> = HashMap::new();

    for (address, couples) in couples_map {
        for couple in couples {
            let song_id = couple.song_id;
            
            // Add to matches
            matches
                .entry(song_id)
                .or_insert_with(Vec::new)
                .push([sample_fingerprint[&address], couple.anchor_time_ms]);

            // Update timestamp
            timestamps
                .entry(song_id)
                .and_modify(|existing| {
                    if couple.anchor_time_ms < *existing {
                        *existing = couple.anchor_time_ms;
                    }
                })
                .or_insert(couple.anchor_time_ms);

            // Update target zones
            target_zones
                .entry(song_id)
                .or_insert_with(HashMap::new)
                .entry(couple.anchor_time_ms)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    // Optionally filter matches
    // matches = filter_matches(10, matches, &target_zones);

    let scores = analyze_relative_timing(&matches);

    let mut match_list = Vec::new();

    for (song_id, score) in scores {
        match get_song_by_id(db_client, song_id).await? {
            Some(song) => {
                let timestamp = timestamps.get(&song_id).copied().unwrap_or(0);
                match_list.push(Match {
                    song_id,
                    youtube_url: song.youtube_url,
                    score,
                });
            }
            None => {
                eprintln!("Song with ID {} doesn't exist", song_id);
                continue;
            }
        }
    }

    // Sort by score in descending order
    match_list.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok((match_list, start_time.elapsed()))
}

/// Filters out matches that don't have enough target zones to meet the specified threshold
fn filter_matches(
    threshold: usize,
    mut matches: HashMap<u32, Vec<[u32; 2]>>,
    target_zones: &HashMap<u32, HashMap<u32, i32>>,
) -> HashMap<u32, Vec<[u32; 2]>> {
    // Filter out non target zones.
    // When a target zone has less than `TARGET_ZONE_SIZE` anchor times,
    // it is not considered a target zone.
    let mut filtered_target_zones: HashMap<u32, HashMap<u32, i32>> = HashMap::new();

    for (song_id, anchor_times) in target_zones {
        let mut valid_zones = HashMap::new();
        for (anchor_time, count) in anchor_times {
            if *count >= TARGET_ZONE_SIZE as i32 {
                valid_zones.insert(*anchor_time, *count);
            }
        }
        if !valid_zones.is_empty() {
            filtered_target_zones.insert(*song_id, valid_zones);
        }
    }

    // Keep only matches that have enough target zones
    matches.retain(|song_id, _| {
        filtered_target_zones
            .get(song_id)
            .map(|zones| zones.len() >= threshold)
            .unwrap_or(false)
    });

    matches
}


fn analyze_relative_timing(matches: &HashMap<i64, Vec<[u32; 2]>>) -> HashMap<i64, f64> {
    let mut scores = HashMap::new();

    for (song_id, times) in matches {
        let mut count = 0;

        for i in 0..times.len() {
            for j in (i + 1)..times.len() {
                let sample_diff = (times[i][0] as i32 - times[j][0] as i32).abs() as f64;
                let db_diff = (times[i][1] as i32 - times[j][1] as i32).abs() as f64;

                // Allow some tolerance
                if (sample_diff - db_diff).abs() < 100.0 {
                    count += 1;
                }
            }
        }

        scores.insert(*song_id, count as f64);
    }

    scores
}

/// Generates a unique ID (placeholder - implement your own unique ID generation)
fn generate_unique_id() -> i64 {
    use std::time::SystemTime;
    
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

