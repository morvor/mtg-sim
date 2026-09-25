//! Scryfall "Oracle Cards" bulk data.
//!
//! See <https://scryfall.com/docs/api/bulk-data> and
//! <https://scryfall.com/docs/api/cards> for the field definitions. Only the fields
//! relevant to game play are deserialized.

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// One face of a multi-face card (split, flip, transform, MDFC, adventure, ...).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CardFace {
    pub name: String,
    #[serde(default)]
    pub mana_cost: String,
    #[serde(default)]
    pub type_line: Option<String>,
    #[serde(default)]
    pub oracle_text: Option<String>,
    #[serde(default)]
    pub colors: Option<Vec<String>>,
    #[serde(default)]
    pub color_indicator: Option<Vec<String>>,
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub toughness: Option<String>,
    #[serde(default)]
    pub loyalty: Option<String>,
    #[serde(default)]
    pub defense: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
}

/// A related card entry (tokens a card makes, meld pieces, combo pieces).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RelatedCard {
    pub id: String,
    pub component: String,
    pub name: String,
    #[serde(default)]
    pub type_line: String,
}

/// A card object from the Oracle Cards bulk file.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ScryfallCard {
    pub id: String,
    #[serde(default)]
    pub oracle_id: String,
    pub name: String,
    pub layout: String,
    #[serde(default)]
    pub mana_cost: Option<String>,
    #[serde(default)]
    pub cmc: Option<f64>,
    #[serde(default)]
    pub type_line: Option<String>,
    #[serde(default)]
    pub oracle_text: Option<String>,
    #[serde(default)]
    pub colors: Option<Vec<String>>,
    #[serde(default)]
    pub color_identity: Vec<String>,
    #[serde(default)]
    pub color_indicator: Option<Vec<String>>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub toughness: Option<String>,
    #[serde(default)]
    pub loyalty: Option<String>,
    #[serde(default)]
    pub defense: Option<String>,
    #[serde(default)]
    pub hand_modifier: Option<String>,
    #[serde(default)]
    pub life_modifier: Option<String>,
    #[serde(default)]
    pub produced_mana: Option<Vec<String>>,
    #[serde(default)]
    pub card_faces: Option<Vec<CardFace>>,
    #[serde(default)]
    pub all_parts: Option<Vec<RelatedCard>>,
    #[serde(default)]
    pub legalities: HashMap<String, String>,
    #[serde(default)]
    pub games: Vec<String>,
    #[serde(default)]
    pub set: String,
    #[serde(default)]
    pub set_type: String,
    #[serde(default)]
    pub rarity: String,
    #[serde(default)]
    pub reserved: bool,
    #[serde(default)]
    pub game_changer: Option<bool>,
    #[serde(default)]
    pub edhrec_rank: Option<u32>,
    #[serde(default)]
    pub attraction_lights: Option<Vec<u32>>,
}

impl ScryfallCard {
    /// True for layouts that represent real, playable Magic cards (as opposed to
    /// art cards, tokens, emblems, and oversized variant cards).
    pub fn is_playable_card(&self) -> bool {
        !matches!(
            self.layout.as_str(),
            "art_series" | "token" | "double_faced_token" | "emblem" | "front_card"
        )
    }

    /// True if the card is legal (or restricted) in at least one format.
    pub fn is_legal_somewhere(&self) -> bool {
        self.legalities
            .values()
            .any(|v| v == "legal" || v == "restricted")
    }

    /// Legality in the named format (`"standard"`, `"commander"`, ...).
    pub fn legality(&self, format: &str) -> Option<&str> {
        self.legalities.get(format).map(String::as_str)
    }

    /// The faces of this card; single-faced cards yield one synthetic face built from
    /// the top-level fields.
    pub fn faces(&self) -> Vec<CardFace> {
        match &self.card_faces {
            Some(f) if !f.is_empty() => f.clone(),
            _ => vec![CardFace {
                name: self.name.clone(),
                mana_cost: self.mana_cost.clone().unwrap_or_default(),
                type_line: self.type_line.clone(),
                oracle_text: self.oracle_text.clone(),
                colors: self.colors.clone(),
                color_indicator: self.color_indicator.clone(),
                power: self.power.clone(),
                toughness: self.toughness.clone(),
                loyalty: self.loyalty.clone(),
                defense: self.defense.clone(),
                layout: None,
            }],
        }
    }
}

/// All Oracle cards, indexed by Oracle ID and by (case-insensitive) name.
pub struct CardDatabase {
    cards: Vec<ScryfallCard>,
    by_oracle_id: HashMap<String, usize>,
    by_name: HashMap<String, Vec<usize>>,
}

impl CardDatabase {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader: Box<dyn BufRead> = if path.extension().is_some_and(|e| e == "gz") {
            Box::new(BufReader::with_capacity(1 << 20, GzDecoder::new(file)))
        } else {
            Box::new(BufReader::with_capacity(1 << 20, file))
        };
        let mut cards = Vec::with_capacity(40_000);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim().trim_end_matches(',');
            if trimmed.is_empty() || trimmed == "[" || trimmed == "]" {
                continue;
            }
            let card: ScryfallCard = serde_json::from_str(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            cards.push(card);
        }
        Ok(Self::from_cards(cards))
    }

    pub fn from_cards(cards: Vec<ScryfallCard>) -> Self {
        let mut by_oracle_id = HashMap::with_capacity(cards.len());
        let mut by_name: HashMap<String, Vec<usize>> = HashMap::with_capacity(cards.len() * 2);
        for (i, c) in cards.iter().enumerate() {
            if !c.oracle_id.is_empty() {
                by_oracle_id.entry(c.oracle_id.clone()).or_insert(i);
            }
            by_name.entry(c.name.to_lowercase()).or_default().push(i);
            if let Some(faces) = &c.card_faces {
                for f in faces {
                    let key = f.name.to_lowercase();
                    if key != c.name.to_lowercase() {
                        by_name.entry(key).or_default().push(i);
                    }
                }
            }
        }
        // Prefer real playable cards over tokens/art cards with the same name.
        for idxs in by_name.values_mut() {
            idxs.sort_by_key(|&i| {
                let c = &cards[i];
                (!c.is_playable_card(), i)
            });
        }
        Self {
            cards,
            by_oracle_id,
            by_name,
        }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScryfallCard> {
        self.cards.iter()
    }

    /// Looks up a card by exact (case-insensitive) name. Accepts full names of
    /// multi-face cards ("Fire // Ice") or any single face name ("Fire"). Real cards
    /// are preferred over tokens with the same name.
    pub fn by_name(&self, name: &str) -> Option<&ScryfallCard> {
        self.by_name
            .get(&name.to_lowercase())
            .and_then(|v| v.first())
            .map(|&i| &self.cards[i])
    }

    /// Looks up a token (layout `token` / `double_faced_token`) by name.
    pub fn token_by_name(&self, name: &str) -> Option<&ScryfallCard> {
        self.by_name.get(&name.to_lowercase()).and_then(|v| {
            v.iter()
                .map(|&i| &self.cards[i])
                .find(|c| c.layout == "token" || c.layout == "double_faced_token")
        })
    }

    /// All cards with the given name (a real card plus possibly tokens/art cards).
    pub fn all_by_name(&self, name: &str) -> Vec<&ScryfallCard> {
        self.by_name
            .get(&name.to_lowercase())
            .map(|v| v.iter().map(|&i| &self.cards[i]).collect())
            .unwrap_or_default()
    }

    pub fn by_oracle_id(&self, oracle_id: &str) -> Option<&ScryfallCard> {
        self.by_oracle_id.get(oracle_id).map(|&i| &self.cards[i])
    }
}
