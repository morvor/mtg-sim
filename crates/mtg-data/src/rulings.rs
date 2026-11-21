//! Scryfall bulk-data rulings.

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Ruling {
    pub oracle_id: String,
    /// `"wotc"` or `"scryfall"`.
    pub source: String,
    pub published_at: String,
    pub comment: String,
}

pub struct RulingsDatabase {
    rulings: Vec<Ruling>,
    by_oracle_id: HashMap<String, Vec<usize>>,
}

impl RulingsDatabase {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader: Box<dyn BufRead> = if path.extension().is_some_and(|e| e == "gz") {
            Box::new(BufReader::new(GzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        };
        let mut rulings = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim().trim_end_matches(',');
            if trimmed.is_empty() || trimmed == "[" || trimmed == "]" {
                continue;
            }
            let r: Ruling = serde_json::from_str(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            rulings.push(r);
        }
        let mut by_oracle_id: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, r) in rulings.iter().enumerate() {
            by_oracle_id.entry(r.oracle_id.clone()).or_default().push(i);
        }
        Ok(Self {
            rulings,
            by_oracle_id,
        })
    }

    pub fn len(&self) -> usize {
        self.rulings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rulings.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Ruling> {
        self.rulings.iter()
    }

    pub fn for_oracle_id(&self, oracle_id: &str) -> Vec<&Ruling> {
        self.by_oracle_id
            .get(oracle_id)
            .map(|v| v.iter().map(|&i| &self.rulings[i]).collect())
            .unwrap_or_default()
    }
}
