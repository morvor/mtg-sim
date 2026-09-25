//! Basic game vocabulary: identifiers, colors, card types, supertypes, subtypes
//! (CR 105, 205).

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashSet;
use std::fmt;
use std::sync::OnceLock;

/// A player in the game (CR 102). Index into `Game::players`.
#[derive(
    Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default, Serialize, Deserialize,
)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P{}", self.0)
    }
}

/// A game object (CR 109). Every zone change creates a new object with a new id
/// (CR 400.7); the old id keeps its last known information.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct ObjectId(pub u32);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Timestamps order continuous effects (CR 613.7).
pub type Timestamp = u64;

/// Either a player or an object — the things that can be targeted, dealt damage,
/// attacked, etc.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Entity {
    Player(PlayerId),
    Object(ObjectId),
}

impl Entity {
    pub fn object(self) -> Option<ObjectId> {
        match self {
            Entity::Object(o) => Some(o),
            _ => None,
        }
    }
    pub fn player(self) -> Option<PlayerId> {
        match self {
            Entity::Player(p) => Some(p),
            _ => None,
        }
    }
}

impl From<PlayerId> for Entity {
    fn from(p: PlayerId) -> Self {
        Entity::Player(p)
    }
}
impl From<ObjectId> for Entity {
    fn from(o: ObjectId) -> Self {
        Entity::Object(o)
    }
}

// ---------------------------------------------------------------------------
// Colors (CR 105)
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Color {
    pub const ALL: [Color; 5] = [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ];

    pub fn from_letter(c: char) -> Option<Color> {
        Some(match c.to_ascii_uppercase() {
            'W' => Color::White,
            'U' => Color::Blue,
            'B' => Color::Black,
            'R' => Color::Red,
            'G' => Color::Green,
            _ => return None,
        })
    }

    pub fn letter(self) -> char {
        match self {
            Color::White => 'W',
            Color::Blue => 'U',
            Color::Black => 'B',
            Color::Red => 'R',
            Color::Green => 'G',
        }
    }

    pub fn from_word(w: &str) -> Option<Color> {
        Some(match w.to_ascii_lowercase().as_str() {
            "white" => Color::White,
            "blue" => Color::Blue,
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            _ => return None,
        })
    }

    pub fn word(self) -> &'static str {
        match self {
            Color::White => "white",
            Color::Blue => "blue",
            Color::Black => "black",
            Color::Red => "red",
            Color::Green => "green",
        }
    }

    /// The basic land type associated with the color (CR 305.6).
    pub fn basic_land_type(self) -> &'static str {
        match self {
            Color::White => "Plains",
            Color::Blue => "Island",
            Color::Black => "Swamp",
            Color::Red => "Mountain",
            Color::Green => "Forest",
        }
    }

    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

/// A set of colors. Empty means colorless (CR 105.2c).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ColorSet(pub u8);

impl ColorSet {
    pub const NONE: ColorSet = ColorSet(0);
    pub const ALL: ColorSet = ColorSet(0b11111);

    pub fn single(c: Color) -> Self {
        ColorSet(c.bit())
    }
    pub fn contains(self, c: Color) -> bool {
        self.0 & c.bit() != 0
    }
    pub fn insert(&mut self, c: Color) {
        self.0 |= c.bit();
    }
    pub fn remove(&mut self, c: Color) {
        self.0 &= !c.bit();
    }
    pub fn union(self, o: ColorSet) -> ColorSet {
        ColorSet(self.0 | o.0)
    }
    pub fn intersects(self, o: ColorSet) -> bool {
        self.0 & o.0 != 0
    }
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }
    pub fn is_colorless(self) -> bool {
        self.0 == 0
    }
    pub fn is_multicolored(self) -> bool {
        self.count() >= 2
    }
    pub fn is_monocolored(self) -> bool {
        self.count() == 1
    }
    pub fn iter(self) -> impl Iterator<Item = Color> {
        Color::ALL.into_iter().filter(move |c| self.contains(*c))
    }
    /// The ten color pairs (CR 105.5): every set of exactly two of the five colors.
    pub fn color_pairs() -> [ColorSet; 10] {
        let mut out = [ColorSet::NONE; 10];
        let mut i = 0;
        for a in 0..5 {
            for b in a + 1..5 {
                let mut s = ColorSet::single(Color::ALL[a]);
                s.insert(Color::ALL[b]);
                out[i] = s;
                i += 1;
            }
        }
        out
    }
    /// Exactly two of the five colors (CR 105.5).
    pub fn is_color_pair(self) -> bool {
        self.count() == 2
    }
    /// Parses Scryfall color arrays like `["W","U"]`.
    pub fn from_letters<S: AsRef<str>>(letters: &[S]) -> Self {
        let mut s = ColorSet::NONE;
        for l in letters {
            if let Some(c) = l.as_ref().chars().next().and_then(Color::from_letter) {
                s.insert(c);
            }
        }
        s
    }
}

impl FromIterator<Color> for ColorSet {
    fn from_iter<I: IntoIterator<Item = Color>>(iter: I) -> Self {
        let mut s = ColorSet::NONE;
        for c in iter {
            s.insert(c);
        }
        s
    }
}

impl fmt::Debug for ColorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_colorless() {
            return write!(f, "Colorless");
        }
        for c in self.iter() {
            write!(f, "{}", c.letter())?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Card types and supertypes (CR 205.2, 205.4)
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum CardType {
    Artifact,
    Battle,
    Conspiracy,
    Creature,
    Dungeon,
    Enchantment,
    Instant,
    Kindred,
    Land,
    Phenomenon,
    Plane,
    Planeswalker,
    Scheme,
    Sorcery,
    Vanguard,
}

impl CardType {
    pub const ALL: [CardType; 15] = [
        CardType::Artifact,
        CardType::Battle,
        CardType::Conspiracy,
        CardType::Creature,
        CardType::Dungeon,
        CardType::Enchantment,
        CardType::Instant,
        CardType::Kindred,
        CardType::Land,
        CardType::Phenomenon,
        CardType::Plane,
        CardType::Planeswalker,
        CardType::Scheme,
        CardType::Sorcery,
        CardType::Vanguard,
    ];

    pub fn from_word(w: &str) -> Option<CardType> {
        Some(match w.to_ascii_lowercase().as_str() {
            "artifact" | "artifacts" => CardType::Artifact,
            "battle" | "battles" => CardType::Battle,
            "conspiracy" | "conspiracies" => CardType::Conspiracy,
            "creature" | "creatures" => CardType::Creature,
            "dungeon" | "dungeons" => CardType::Dungeon,
            "enchantment" | "enchantments" => CardType::Enchantment,
            "instant" | "instants" => CardType::Instant,
            // "Tribal" was renamed "Kindred" (CR 308); old printings may still say it.
            "kindred" | "tribal" => CardType::Kindred,
            "land" | "lands" => CardType::Land,
            "phenomenon" | "phenomena" => CardType::Phenomenon,
            "plane" | "planes" => CardType::Plane,
            "planeswalker" | "planeswalkers" => CardType::Planeswalker,
            "scheme" | "schemes" => CardType::Scheme,
            "sorcery" | "sorceries" => CardType::Sorcery,
            "vanguard" | "vanguards" => CardType::Vanguard,
            _ => return None,
        })
    }

    pub fn word(self) -> &'static str {
        match self {
            CardType::Artifact => "artifact",
            CardType::Battle => "battle",
            CardType::Conspiracy => "conspiracy",
            CardType::Creature => "creature",
            CardType::Dungeon => "dungeon",
            CardType::Enchantment => "enchantment",
            CardType::Instant => "instant",
            CardType::Kindred => "kindred",
            CardType::Land => "land",
            CardType::Phenomenon => "phenomenon",
            CardType::Plane => "plane",
            CardType::Planeswalker => "planeswalker",
            CardType::Scheme => "scheme",
            CardType::Sorcery => "sorcery",
            CardType::Vanguard => "vanguard",
        }
    }

    /// Permanent types (CR 110.4).
    pub fn is_permanent_type(self) -> bool {
        matches!(
            self,
            CardType::Artifact
                | CardType::Battle
                | CardType::Creature
                | CardType::Enchantment
                | CardType::Land
                | CardType::Planeswalker
        )
    }

    fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CardTypeSet(pub u16);

impl CardTypeSet {
    pub const NONE: CardTypeSet = CardTypeSet(0);
    pub fn single(t: CardType) -> Self {
        CardTypeSet(t.bit())
    }
    pub fn contains(self, t: CardType) -> bool {
        self.0 & t.bit() != 0
    }
    pub fn insert(&mut self, t: CardType) {
        self.0 |= t.bit();
    }
    pub fn remove(&mut self, t: CardType) {
        self.0 &= !t.bit();
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn union(self, o: CardTypeSet) -> Self {
        CardTypeSet(self.0 | o.0)
    }
    pub fn intersects(self, o: CardTypeSet) -> bool {
        self.0 & o.0 != 0
    }
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }
    pub fn iter(self) -> impl Iterator<Item = CardType> {
        CardType::ALL.into_iter().filter(move |t| self.contains(*t))
    }
    pub fn has_permanent_type(self) -> bool {
        self.iter().any(CardType::is_permanent_type)
    }
}

impl FromIterator<CardType> for CardTypeSet {
    fn from_iter<I: IntoIterator<Item = CardType>>(iter: I) -> Self {
        let mut s = CardTypeSet::NONE;
        for t in iter {
            s.insert(t);
        }
        s
    }
}

impl fmt::Debug for CardTypeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Supertype {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
}

impl Supertype {
    pub const ALL: [Supertype; 5] = [
        Supertype::Basic,
        Supertype::Legendary,
        Supertype::Ongoing,
        Supertype::Snow,
        Supertype::World,
    ];

    pub fn from_word(w: &str) -> Option<Supertype> {
        Some(match w.to_ascii_lowercase().as_str() {
            "basic" => Supertype::Basic,
            "legendary" => Supertype::Legendary,
            "ongoing" => Supertype::Ongoing,
            "snow" => Supertype::Snow,
            "world" => Supertype::World,
            _ => return None,
        })
    }
    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SupertypeSet(pub u8);

impl SupertypeSet {
    pub const NONE: SupertypeSet = SupertypeSet(0);
    pub fn contains(self, t: Supertype) -> bool {
        self.0 & t.bit() != 0
    }
    pub fn insert(&mut self, t: Supertype) {
        self.0 |= t.bit();
    }
    pub fn remove(&mut self, t: Supertype) {
        self.0 &= !t.bit();
    }
    pub fn iter(self) -> impl Iterator<Item = Supertype> {
        Supertype::ALL
            .into_iter()
            .filter(move |t| self.contains(*t))
    }
}

impl fmt::Debug for SupertypeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// A subtype such as "Goblin", "Aura", "Equipment", "Forest", "Jace", "Arcane".
pub type Subtype = SmolStr;

/// Which card type a subtype is correlated with (CR 205.3d).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SubtypeKind {
    Artifact,
    Enchantment,
    Land,
    Planeswalker,
    Spell,
    Creature,
    Plane,
    Battle,
    Dungeon,
}

/// Subtype lists from CR 205.3g–205.3q, parsed from the loaded Comprehensive Rules.
pub struct SubtypeLists {
    pub artifact: HashSet<String>,
    pub enchantment: HashSet<String>,
    pub land: HashSet<String>,
    pub planeswalker: HashSet<String>,
    pub spell: HashSet<String>,
    pub creature: Vec<String>,
    pub creature_set: HashSet<String>,
    pub plane: HashSet<String>,
    pub battle: HashSet<String>,
    pub basic_land: [&'static str; 5],
}

fn parse_type_list(rule_text: &str) -> Vec<String> {
    // "... The artifact types are Attraction (see rule 717), Blood, ..., and Vehicle."
    let Some(pos) = rule_text.find(" are ").or_else(|| rule_text.find(" is ")) else {
        return vec![];
    };
    // Use the *last* sentence that enumerates the list.
    let list_start = rule_text
        .match_indices(" types are ")
        .last()
        .map(|(i, m)| i + m.len())
        .or_else(|| {
            rule_text
                .match_indices(" type is ")
                .last()
                .map(|(i, m)| i + m.len())
        })
        .unwrap_or(pos + 5);
    let mut list = &rule_text[list_start..];
    if let Some(end) = list
        .find(". ")
        .or_else(|| list.find(".\n"))
        .or_else(|| list.rfind('.'))
    {
        list = &list[..end];
    }
    list.split(',')
        .map(|s| {
            let s = s.trim();
            let s = s.strip_prefix("and ").unwrap_or(s);
            let s = s.split(" (").next().unwrap_or(s);
            s.trim().replace('\u{2019}', "'")
        })
        .filter(|s| !s.is_empty() && s.chars().next().is_some_and(|c| c.is_uppercase()))
        .collect()
}

pub fn subtype_lists() -> &'static SubtypeLists {
    static LISTS: OnceLock<SubtypeLists> = OnceLock::new();
    LISTS.get_or_init(|| {
        let cr = mtg_data::comprehensive_rules();
        let get = |id: &str| {
            cr.get(id)
                .map(|r| parse_type_list(&r.text))
                .unwrap_or_default()
        };
        let set = |v: Vec<String>| v.into_iter().collect::<HashSet<_>>();
        let creature = {
            // 205.3m: "One creature type is two words long: Time Lord. All other creature types are one word long: Advisor, ..."
            let text = cr.get("205.3m").map(|r| r.text.clone()).unwrap_or_default();
            let mut v: Vec<String> = Vec::new();
            if let Some(i) = text.find("one word long:") {
                let list = &text[i + "one word long:".len()..];
                let list = list
                    .split(".\n")
                    .next()
                    .unwrap_or(list)
                    .trim_end_matches('.');
                for s in list.split(',') {
                    let s = s.trim();
                    let s = s.strip_prefix("and ").unwrap_or(s).trim();
                    if !s.is_empty() {
                        v.push(s.to_string());
                    }
                }
            }
            v.push("Time Lord".to_string());
            v
        };
        SubtypeLists {
            artifact: set(get("205.3g")),
            enchantment: set(get("205.3h")),
            land: set(get("205.3i")),
            planeswalker: set(get("205.3j")),
            spell: set(get("205.3k")),
            creature_set: creature.iter().cloned().collect(),
            creature,
            plane: set(get("205.3n")),
            battle: set(get("205.3q")),
            basic_land: ["Plains", "Island", "Swamp", "Mountain", "Forest"],
        }
    })
}

pub fn is_creature_type(s: &str) -> bool {
    subtype_lists().creature_set.contains(s)
}

pub fn is_basic_land_type(s: &str) -> bool {
    matches!(s, "Plains" | "Island" | "Swamp" | "Mountain" | "Forest")
}

/// Classifies a subtype by the card type it's correlated with.
pub fn subtype_kind(s: &str) -> Option<SubtypeKind> {
    let l = subtype_lists();
    if l.creature_set.contains(s) {
        Some(SubtypeKind::Creature)
    } else if l.land.contains(s) {
        Some(SubtypeKind::Land)
    } else if l.artifact.contains(s) {
        Some(SubtypeKind::Artifact)
    } else if l.enchantment.contains(s) {
        Some(SubtypeKind::Enchantment)
    } else if l.planeswalker.contains(s) {
        Some(SubtypeKind::Planeswalker)
    } else if l.spell.contains(s) {
        Some(SubtypeKind::Spell)
    } else if l.battle.contains(s) {
        Some(SubtypeKind::Battle)
    } else if l.plane.contains(s) {
        Some(SubtypeKind::Plane)
    } else {
        None
    }
}

/// A parsed type line ("Legendary Artifact Creature — Human Wizard").
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeLine {
    pub supertypes: SupertypeSet,
    pub card_types: CardTypeSet,
    pub subtypes: Vec<Subtype>,
}

impl TypeLine {
    pub fn parse(line: &str) -> TypeLine {
        let mut tl = TypeLine::default();
        let (left, right) = match line.split_once('—') {
            Some((l, r)) => (l, Some(r)),
            None => match line.split_once(" - ") {
                Some((l, r)) => (l, Some(r)),
                None => (line, None),
            },
        };
        for w in left.split_whitespace() {
            if let Some(s) = Supertype::from_word(w) {
                tl.supertypes.insert(s);
            } else if let Some(t) = CardType::from_word(w) {
                tl.card_types.insert(t);
            }
            // Unknown words (e.g. "Token", "Emblem", "Card") are ignored.
        }
        if let Some(r) = right {
            let r = r.trim();
            // Planes have multi-word subtypes ("Plane — Serra's Realm"): the whole
            // remainder is one subtype (CR 205.3n).
            if tl.card_types.contains(CardType::Plane) {
                if !r.is_empty() {
                    tl.subtypes.push(SmolStr::new(r));
                }
            } else {
                let words: Vec<&str> = r.split_whitespace().collect();
                let mut i = 0;
                while i < words.len() {
                    // Two-word creature type "Time Lord" (CR 205.3m).
                    if i + 1 < words.len() && words[i] == "Time" && words[i + 1] == "Lord" {
                        tl.subtypes.push(SmolStr::new("Time Lord"));
                        i += 2;
                        continue;
                    }
                    tl.subtypes.push(SmolStr::new(words[i]));
                    i += 1;
                }
            }
        }
        tl
    }
}

/// Counter kinds (CR 122). Stored by name; common ones have helpers.
pub type CounterKind = SmolStr;

pub mod counters {
    pub const PLUS1: &str = "+1/+1";
    pub const MINUS1: &str = "-1/-1";
    pub const LOYALTY: &str = "loyalty";
    pub const DEFENSE: &str = "defense";
    pub const POISON: &str = "poison";
    pub const ENERGY: &str = "energy";
    pub const EXPERIENCE: &str = "experience";
    pub const RAD: &str = "rad";
    pub const TICKET: &str = "ticket";
    pub const LORE: &str = "lore";
    pub const TIME: &str = "time";
    pub const FADE: &str = "fade";
    pub const AGE: &str = "age";
    pub const CHARGE: &str = "charge";
    pub const SHIELD: &str = "shield";
    pub const STUN: &str = "stun";
    pub const FINALITY: &str = "finality";
    pub const OIL: &str = "oil";
    pub const LEVEL: &str = "level";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_line_parsing() {
        let t = TypeLine::parse("Legendary Artifact Creature — Human Wizard");
        assert!(t.supertypes.contains(Supertype::Legendary));
        assert!(t.card_types.contains(CardType::Artifact));
        assert!(t.card_types.contains(CardType::Creature));
        assert_eq!(
            t.subtypes,
            vec![SmolStr::new("Human"), SmolStr::new("Wizard")]
        );
        let t = TypeLine::parse("Creature — Time Lord Doctor");
        assert_eq!(
            t.subtypes,
            vec![SmolStr::new("Time Lord"), SmolStr::new("Doctor")]
        );
        let t = TypeLine::parse("Plane — Serra's Realm");
        assert_eq!(t.subtypes, vec![SmolStr::new("Serra's Realm")]);
    }

    #[test]
    fn subtype_lists_parse() {
        let l = subtype_lists();
        assert!(l.creature_set.contains("Goblin"));
        assert!(l.creature_set.contains("Time Lord"));
        assert!(l.land.contains("Forest"));
        assert!(l.land.contains("Urza's"));
        assert!(l.artifact.contains("Equipment"));
        assert!(l.enchantment.contains("Aura"));
        assert!(l.spell.contains("Arcane"));
        assert!(l.planeswalker.contains("Jace"));
        assert!(l.battle.contains("Siege"));
        assert!(l.creature.len() > 250, "{}", l.creature.len());
    }
}
