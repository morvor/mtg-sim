//! Card definitions built from Scryfall oracle data, and the card database.

use crate::ability::Ability;
use crate::mana::{ManaCost, ManaType};
use crate::object::{Characteristics, FaceState};
use crate::oracle;
use crate::types::*;
use mtg_data::scryfall::ScryfallCard;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Card layouts (Scryfall `layout`, CR 709–722).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layout {
    Normal,
    Split,
    Flip,
    Transform,
    ModalDfc,
    Meld,
    Leveler,
    Class,
    Case,
    Saga,
    Adventure,
    Mutate,
    Prototype,
    Battle,
    Planar,
    Scheme,
    Vanguard,
    Token,
    DoubleFacedToken,
    Emblem,
    Augment,
    Host,
    ArtSeries,
    ReversibleCard,
    Prepare,
    Other,
}

impl Layout {
    pub fn from_scryfall(s: &str) -> Layout {
        match s {
            "normal" => Layout::Normal,
            "split" => Layout::Split,
            "flip" => Layout::Flip,
            "transform" => Layout::Transform,
            "modal_dfc" => Layout::ModalDfc,
            "meld" => Layout::Meld,
            "leveler" => Layout::Leveler,
            "class" => Layout::Class,
            "case" => Layout::Case,
            "saga" => Layout::Saga,
            "adventure" => Layout::Adventure,
            "mutate" => Layout::Mutate,
            "prototype" => Layout::Prototype,
            "battle" => Layout::Battle,
            "planar" => Layout::Planar,
            "scheme" => Layout::Scheme,
            "vanguard" => Layout::Vanguard,
            "token" => Layout::Token,
            "double_faced_token" => Layout::DoubleFacedToken,
            "emblem" => Layout::Emblem,
            "augment" => Layout::Augment,
            "host" => Layout::Host,
            "art_series" => Layout::ArtSeries,
            "reversible_card" => Layout::ReversibleCard,
            "prepare" => Layout::Prepare,
            _ => Layout::Other,
        }
    }

    /// Double-faced cards (CR 712).
    pub fn is_double_faced(self) -> bool {
        matches!(
            self,
            Layout::Transform
                | Layout::ModalDfc
                | Layout::Meld
                | Layout::DoubleFacedToken
                | Layout::Battle
        )
    }
}

/// The card a set of characteristics comes from ([`Characteristics::printed`]): its halves,
/// faces and alternative characteristics are part of the copiable values (CR 709.5b,
/// 715.2b, 720.2b, 722.2b).
#[derive(Clone)]
pub struct PrintedCard(pub Arc<CardDef>);

impl std::fmt::Debug for PrintedCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrintedCard({})", self.0.name)
    }
}

/// Layer 0: characteristics include the card they come from (see [`PrintedCard`]), except
/// a face-down object's (CR 708.2).
pub fn mark_printed(g: &mut crate::game::Game, live: &[ObjectId]) {
    for id in live {
        let o = &mut g.objects[id.0 as usize];
        if o.face_down {
            continue;
        }
        o.chars.printed = o.card.clone().map(PrintedCard);
    }
}

/// One face (or half) of a card.
#[derive(Clone, Debug)]
pub struct FaceDef {
    pub chars: Characteristics,
    /// Oracle text sentences the compiler could not understand.
    pub unsupported: Vec<String>,
    /// Printed P/T contained `*` (defined by a characteristic-defining ability).
    pub star_power: bool,
    pub star_toughness: bool,
}

/// A card definition: everything printed on the card (all faces).
#[derive(Clone, Debug)]
pub struct CardDef {
    pub oracle_id: String,
    pub name: SmolStr,
    pub layout: Layout,
    pub faces: Vec<FaceDef>,
    pub color_identity: ColorSet,
    pub produced_mana: Vec<ManaType>,
    /// Names of related cards (meld partner, tokens it makes).
    pub related: Vec<(String, String)>,
    /// Formats in which the card is legal or restricted.
    pub legal_formats: Vec<String>,
    /// Formats in which the card is restricted to one copy (tournament rules, CR 100.6).
    pub restricted_formats: Vec<String>,
    /// Attraction lights, for attraction cards (CR 717).
    pub attraction_lights: Vec<u32>,
    /// An alternate name printed in the upper left corner of this printing, with the
    /// card's Oracle name in a secondary title bar (CR 201.6). The card has only its
    /// Oracle name for deck construction, game rules, and effects: this is never one of
    /// its names.
    pub alternate_name: Option<SmolStr>,
}

impl CardDef {
    pub fn front(&self) -> &FaceDef {
        &self.faces[0]
    }

    pub fn back(&self) -> Option<&FaceDef> {
        self.faces.get(1)
    }

    /// Whether the compiler understood every ability of every face.
    pub fn is_fully_supported(&self) -> bool {
        self.faces.iter().all(|f| f.unsupported.is_empty())
    }

    pub fn unsupported_text(&self) -> Vec<&str> {
        self.faces
            .iter()
            .flat_map(|f| f.unsupported.iter().map(String::as_str))
            .collect()
    }

    /// Characteristics for the given face state (CR 709.4 for split cards, 712.8 for
    /// double-faced cards, 715.4 for adventurers, 710 for flip cards).
    pub fn characteristics(&self, face: FaceState) -> Characteristics {
        match (self.layout, face) {
            (_, FaceState::Back) if self.faces.len() > 1 => self.faces[1].chars.clone(),
            (_, FaceState::Half(i)) if (i as usize) < self.faces.len() => {
                self.faces[i as usize].chars.clone()
            }
            (Layout::Flip, FaceState::Flipped) if self.faces.len() > 1 => {
                // CR 710.1b, 710.2: the alternative name, text box, type line, power and
                // toughness; CR 710.1c: its color and mana cost don't change.
                let front = &self.faces[0].chars;
                let mut c = self.faces[1].chars.clone();
                c.mana_cost = front.mana_cost.clone();
                c.colors = front.colors;
                c.color_indicator = front.color_indicator;
                c
            }
            (Layout::Split, FaceState::Front) | (Layout::Split, FaceState::Fused)
                if self.faces.len() > 1 =>
            {
                // CR 709.4: in every zone except the stack, a split card has the combined
                // characteristics of its halves (on the stack, a fused split spell too).
                combine_split(&self.faces)
            }
            _ => self.faces[0].chars.clone(),
        }
    }

    pub fn is_legal_in(&self, format: &str) -> bool {
        self.legal_formats.iter().any(|f| f == format)
    }

    /// Whether the card is restricted to one copy in a format (CR 100.6).
    pub fn is_restricted_in(&self, format: &str) -> bool {
        self.restricted_formats.iter().any(|f| f == format)
    }

    /// Builds a card definition from a Scryfall card, compiling its oracle text.
    pub fn from_scryfall(c: &ScryfallCard) -> CardDef {
        let layout = Layout::from_scryfall(&c.layout);
        let faces_src = c.faces();
        let mut faces = Vec::with_capacity(faces_src.len());
        for (i, f) in faces_src.iter().enumerate() {
            let type_line = f
                .type_line
                .clone()
                .or_else(|| c.type_line.clone())
                .unwrap_or_default();
            let tl = TypeLine::parse(&type_line);
            let mana_cost = ManaCost::parse(&f.mana_cost);
            let colors = match (&f.colors, &c.colors) {
                (Some(fc), _) => ColorSet::from_letters(fc),
                // Single-faced: top-level colors. Multi-face without per-face colors: derive.
                (None, Some(cc)) if faces_src.len() == 1 => ColorSet::from_letters(cc),
                _ => mana_cost.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
            };
            let color_indicator = f
                .color_indicator
                .as_ref()
                .or(if faces_src.len() == 1 {
                    c.color_indicator.as_ref()
                } else {
                    None
                })
                .map(|ci| ColorSet::from_letters(ci));
            let (mut power, star_power) = parse_pt(f.power.as_deref());
            let (mut toughness, star_toughness) = parse_pt(f.toughness.as_deref());
            // CR 721.2b, 721.2c: a station card's power/toughness box belongs to its
            // highest station symbol; elsewhere than the battlefield it has no power or
            // toughness.
            if c.keywords.iter().any(|k| k.eq_ignore_ascii_case("station")) {
                power = None;
                toughness = None;
            }
            let (loyalty, _) = parse_pt(f.loyalty.as_deref());
            let (defense, _) = parse_pt(f.defense.as_deref());
            let hand_modifier = c
                .hand_modifier
                .as_deref()
                .and_then(|s| s.trim_start_matches('+').parse().ok());
            let life_modifier = c
                .life_modifier
                .as_deref()
                .and_then(|s| s.trim_start_matches('+').parse().ok());
            let text = f.oracle_text.clone().unwrap_or_default();
            let ctx = oracle::CompileContext {
                card_name: &f.name,
                full_name: &c.name,
                type_line: &tl,
                layout,
                face_index: i,
                keywords: &c.keywords,
                power: f.power.as_deref(),
                toughness: f.toughness.as_deref(),
            };
            let compiled = oracle::compile(&text, &ctx);
            faces.push(FaceDef {
                chars: Characteristics {
                    name: SmolStr::new(&f.name),
                    mana_cost,
                    color_indicator,
                    colors,
                    supertypes: tl.supertypes,
                    card_types: tl.card_types,
                    subtypes: tl.subtypes.into_iter().collect(),
                    abilities: compiled.abilities,
                    power,
                    toughness,
                    loyalty,
                    defense,
                    hand_modifier: if i == 0 { hand_modifier } else { None },
                    life_modifier: if i == 0 { life_modifier } else { None },
                    rules_text: Arc::from(text.as_str()),
                    all_creature_names: false,
                    interchangeable_names: Default::default(),
                    all_creature_types: false,
                    printed: None,
                },
                unsupported: compiled.unsupported,
                star_power,
                star_toughness,
            });
        }
        let produced_mana = c
            .produced_mana
            .as_ref()
            .map(|v| {
                v.iter()
                    .filter_map(|s| s.chars().next().and_then(ManaType::from_letter))
                    .collect()
            })
            .unwrap_or_default();
        CardDef {
            oracle_id: c.oracle_id.clone(),
            name: SmolStr::new(&c.name),
            layout,
            faces,
            color_identity: ColorSet::from_letters(&c.color_identity),
            produced_mana,
            related: c
                .all_parts
                .as_ref()
                .map(|v| {
                    v.iter()
                        .map(|p| (p.component.clone(), p.name.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            legal_formats: c
                .legalities
                .iter()
                .filter(|(_, v)| *v == "legal" || *v == "restricted")
                .map(|(k, _)| k.clone())
                .collect(),
            restricted_formats: c
                .legalities
                .iter()
                .filter(|(_, v)| *v == "restricted")
                .map(|(k, _)| k.clone())
                .collect(),
            attraction_lights: c.attraction_lights.clone().unwrap_or_default(),
            alternate_name: c.flavor_name.as_deref().map(SmolStr::new),
        }
    }

    /// A hand-built card for tests and custom content.
    pub fn custom(chars: Characteristics) -> CardDef {
        CardDef {
            oracle_id: String::new(),
            name: chars.name.clone(),
            layout: Layout::Normal,
            faces: vec![FaceDef {
                chars,
                unsupported: vec![],
                star_power: false,
                star_toughness: false,
            }],
            color_identity: ColorSet::NONE,
            produced_mana: vec![],
            related: vec![],
            legal_formats: vec![],
            restricted_formats: vec![],
            attraction_lights: vec![],
            alternate_name: None,
        }
    }
}

/// Parses a printed power/toughness/loyalty value. Returns (numeric part, has star).
pub fn parse_pt(s: Option<&str>) -> (Option<i32>, bool) {
    let Some(s) = s else { return (None, false) };
    let s = s.trim();
    if s == "∞" {
        return (Some(1_000_000), false);
    }
    let star = s.contains('*') || s.contains('?') || s.contains('X');
    let numeric: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    let n = if numeric.is_empty() || numeric == "-" {
        0
    } else {
        numeric
            .parse::<f64>()
            .map(|f| f.floor() as i32)
            .unwrap_or(0)
    };
    (Some(n), star)
}

fn combine_split(faces: &[FaceDef]) -> Characteristics {
    let mut out = faces[0].chars.clone();
    out.name = SmolStr::new(
        faces
            .iter()
            .map(|f| f.chars.name.as_str())
            .collect::<Vec<_>>()
            .join(" // "),
    );
    let mut cost = out.mana_cost.clone().unwrap_or_default();
    let mut abilities = out.abilities.clone();
    let mut text = String::from(&*out.rules_text);
    for f in &faces[1..] {
        if let Some(m) = &f.chars.mana_cost {
            cost.symbols.extend(m.symbols.iter().copied());
        }
        out.colors = out.colors.union(f.chars.colors);
        out.card_types = out.card_types.union(f.chars.card_types);
        for s in &f.chars.subtypes {
            if !out.subtypes.contains(s) {
                out.subtypes.push(s.clone());
            }
        }
        abilities.extend(f.chars.abilities.iter().cloned());
        text.push_str("\n//\n");
        text.push_str(&f.chars.rules_text);
    }
    out.mana_cost = Some(cost);
    out.abilities = abilities;
    out.rules_text = Arc::from(text.as_str());
    out
}

/// Cache of compiled card definitions, keyed by lowercase name.
pub struct CardDb {
    cache: Mutex<HashMap<String, Arc<CardDef>>>,
}

impl CardDb {
    pub fn global() -> &'static CardDb {
        static DB: OnceLock<CardDb> = OnceLock::new();
        DB.get_or_init(|| CardDb {
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Looks up (and compiles, on first use) a real card by name.
    pub fn get(&self, name: &str) -> Option<Arc<CardDef>> {
        let key = name.to_lowercase();
        if let Some(c) = self.cache.lock().unwrap().get(&key) {
            return Some(c.clone());
        }
        let sc = mtg_data::cards().by_name(name)?;
        let def = Arc::new(CardDef::from_scryfall(sc));
        self.cache.lock().unwrap().insert(key, def.clone());
        Some(def)
    }

    /// Looks up a token definition from Scryfall token cards.
    pub fn token(&self, name: &str) -> Option<Arc<CardDef>> {
        let key = format!("token:{}", name.to_lowercase());
        if let Some(c) = self.cache.lock().unwrap().get(&key) {
            return Some(c.clone());
        }
        let sc = mtg_data::cards().token_by_name(name)?;
        let def = Arc::new(CardDef::from_scryfall(sc));
        self.cache.lock().unwrap().insert(key, def.clone());
        Some(def)
    }

    /// Registers a custom card definition under its name.
    pub fn register(&self, def: CardDef) -> Arc<CardDef> {
        let def = Arc::new(def);
        self.cache
            .lock()
            .unwrap()
            .insert(def.name.to_lowercase(), def.clone());
        def
    }
}

/// Convenience: look up a card by name in the global database. Panics if unknown.
pub fn card(name: &str) -> Arc<CardDef> {
    CardDb::global()
        .get(name)
        .unwrap_or_else(|| panic!("unknown card: {name}"))
}

/// All abilities on a face, for inspection.
pub fn abilities_of(def: &CardDef, face: usize) -> &[Ability] {
    &def.faces[face].chars.abilities
}
