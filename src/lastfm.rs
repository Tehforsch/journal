use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Scrobble {
    pub track: String,
    pub artist: String,
    pub album: String,
    pub date: i64, // Unix timestamp in milliseconds
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LastFmData {
    pub username: String,
    pub scrobbles: Vec<Scrobble>,
}

#[derive(Debug, Clone)]
pub struct AlbumStats {
    pub name: String,
    pub artist: String,
    pub play_count: usize,
}

#[derive(Debug, Clone)]
pub struct TrackStats {
    pub name: String,
    pub artist: String,
    pub play_count: usize,
}

pub struct LastFmAnalyzer {
    data: LastFmData,
}

impl LastFmAnalyzer {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let data: LastFmData = serde_json::from_str(&content)?;
        Ok(LastFmAnalyzer { data })
    }

    pub fn get_scrobbles_for_date(&self, date_str: &str) -> Vec<&Scrobble> {
        // Parse the date string (assuming format like "2025-07-10")
        let target_date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => return Vec::new(),
        };

        self.data
            .scrobbles
            .iter()
            .filter(|scrobble| {
                let scrobble_date = DateTime::<Utc>::from_timestamp(scrobble.date / 1000, 0)
                    .map(|dt| dt.naive_utc().date())
                    .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
                scrobble_date == target_date
            })
            .collect()
    }

    pub fn get_top_albums_for_date(&self, date_str: &str, limit: usize) -> Vec<AlbumStats> {
        let scrobbles = self.get_scrobbles_for_date(date_str);
        let mut album_counts: HashMap<(String, String), usize> = HashMap::new();

        for scrobble in scrobbles {
            let key = (scrobble.album.clone(), scrobble.artist.clone());
            *album_counts.entry(key).or_insert(0) += 1;
        }

        let mut albums: Vec<AlbumStats> = album_counts
            .into_iter()
            .map(|((name, artist), play_count)| AlbumStats {
                name,
                artist,
                play_count,
            })
            .collect();

        albums.sort_by(|a, b| b.play_count.cmp(&a.play_count));
        albums.truncate(limit);
        albums
    }

    pub fn get_top_tracks_for_date(&self, date_str: &str, limit: usize) -> Vec<TrackStats> {
        let scrobbles = self.get_scrobbles_for_date(date_str);
        let mut track_counts: HashMap<(String, String), usize> = HashMap::new();

        for scrobble in scrobbles {
            let key = (scrobble.track.clone(), scrobble.artist.clone());
            *track_counts.entry(key).or_insert(0) += 1;
        }

        let mut tracks: Vec<TrackStats> = track_counts
            .into_iter()
            .map(|((name, artist), play_count)| TrackStats {
                name,
                artist,
                play_count,
            })
            .collect();

        tracks.sort_by(|a, b| b.play_count.cmp(&a.play_count));
        tracks.truncate(limit);
        tracks
    }

    pub fn get_total_scrobbles_for_date(&self, date_str: &str) -> usize {
        self.get_scrobbles_for_date(date_str).len()
    }
}
