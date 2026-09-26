//! Scryfall oracle tags ("otags"): community-maintained functional tags such as
//! `removal`, `ramp`, `tutor-creature`. Useful for choosing representative cards when
//! writing tests.

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Tagging {
    pub oracle_id: String,
    #[serde(default)]
    pub weight: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OracleTag {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_ids: Vec<String>,
    #[serde(default)]
    pub child_ids: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub taggings: Vec<Tagging>,
}

pub struct TagDatabase {
    pub tags: Vec<OracleTag>,
    by_label: HashMap<String, usize>,
}

impl TagDatabase {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(GzDecoder::new(file));
        let mut tags = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim().trim_end_matches(',');
            if trimmed.is_empty() || trimmed == "[" || trimmed == "]" {
                continue;
            }
            let t: OracleTag = serde_json::from_str(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            tags.push(t);
        }
        let by_label = tags
            .iter()
            .enumerate()
            .map(|(i, t)| (t.label.clone(), i))
            .collect();
        Ok(Self { tags, by_label })
    }

    pub fn by_label(&self, label: &str) -> Option<&OracleTag> {
        self.by_label.get(label).map(|&i| &self.tags[i])
    }
}
