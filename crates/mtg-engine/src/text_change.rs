//! Text-changing effects (CR 612): changing color words, basic land types, and creature
//! types in an object's rules text and type line (Sleight of Mind, Magical Hack,
//! Artificial Evolution), setting names, and exchanging or copying text boxes.
//!
//! The ability language stores text structurally, so "the text" of an ability is its
//! compiled form: a color word is a [`Filter::Color`], a [`Modification::SetColors`],
//! a protection keyword's filter, a token's color, and so on. A text change rewrites
//! exactly those places where the word is used as that kind of word (CR 612.2), never
//! names ([`Filter::Named`], the object's name).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::*;
use crate::object::Characteristics;
use crate::types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Which kinds of words a text-changing effect replaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextWords {
    /// Color words (Sleight of Mind).
    Color,
    /// Basic land types (Magical Hack).
    BasicLandType,
    /// Color words or basic land types (Mind Bend).
    ColorOrBasicLandType,
    /// Creature types (Artificial Evolution).
    CreatureType,
}

/// The kind of a word that text-changing effects can change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordKind {
    Color(Color),
    BasicLandType,
    CreatureType,
}

pub fn word_kind(w: &str) -> Option<WordKind> {
    if let Some(c) = Color::from_word(w) {
        return Some(WordKind::Color(c));
    }
    if is_basic_land_type(w) {
        return Some(WordKind::BasicLandType);
    }
    if is_creature_type(w) {
        return Some(WordKind::CreatureType);
    }
    None
}

fn same_kind(a: &str, b: &str) -> bool {
    matches!(
        (word_kind(a), word_kind(b)),
        (Some(WordKind::Color(_)), Some(WordKind::Color(_)))
            | (Some(WordKind::BasicLandType), Some(WordKind::BasicLandType))
            | (Some(WordKind::CreatureType), Some(WordKind::CreatureType))
    )
}

/// Replaces one color word, basic land type, or creature type with another in the
/// object's text: its abilities and its type line (CR 612.1–612.2). Names are never
/// changed. Replacing a word with a word of a different kind does nothing.
pub fn change_text(c: &mut Characteristics, from: &str, to: &str) {
    if from == to || !same_kind(from, to) {
        return;
    }
    // Type line: land types and creature types (colors aren't words in the type line).
    if !matches!(word_kind(from), Some(WordKind::Color(_))) {
        let to_s = SmolStr::new(to);
        let mut changed = false;
        for s in c.subtypes.iter_mut() {
            if s.as_str() == from {
                *s = to_s.clone();
                changed = true;
            }
        }
        if changed {
            let mut seen: Vec<SmolStr> = Vec::new();
            c.subtypes.retain(|s| {
                if seen.contains(s) {
                    false
                } else {
                    seen.push(s.clone());
                    true
                }
            });
        }
    }
    // Rules text.
    c.abilities = c
        .abilities
        .iter()
        .map(|a| change_ability_text(a, from, to))
        .collect();
    let text = replace_words(&c.rules_text, from, to);
    if text != *c.rules_text {
        c.rules_text = Arc::from(text.as_str());
    }
}

/// An ability with one word replaced (the same ability if the word doesn't appear).
/// Results are cached so the changed ability keeps a stable identity across
/// recomputations.
pub fn change_ability_text(a: &Ability, from: &str, to: &str) -> Ability {
    type Key = (u64, SmolStr, SmolStr);
    static CACHE: OnceLock<Mutex<HashMap<Key, Ability>>> = OnceLock::new();
    let key: Key = (a.uid, SmolStr::new(from), SmolStr::new(to));
    let cache = CACHE.get_or_init(Default::default);
    if let Some(x) = cache.lock().unwrap().get(&key) {
        return x.clone();
    }
    let result = rewrite_ability(a, from, to).unwrap_or_else(|| a.clone());
    // Keep the first result if another thread computed it meanwhile (a stable identity).
    cache.lock().unwrap().entry(key).or_insert(result).clone()
}

fn rewrite_ability(a: &Ability, from: &str, to: &str) -> Option<Ability> {
    let json = serde_json::to_value(&**a).ok()?;
    let mut w = Rewriter {
        from,
        to,
        from_color: Color::from_word(from),
        to_color: Color::from_word(to),
        changed: false,
    };
    let new = w.walk(json, None);
    if !w.changed {
        return None;
    }
    let new = renumber(new, from, to);
    let def: AbilityDef = serde_json::from_value(new).ok()?;
    Some(Arc::new(def))
}

struct Rewriter<'a> {
    from: &'a str,
    to: &'a str,
    from_color: Option<Color>,
    to_color: Option<Color>,
    changed: bool,
}

impl Rewriter<'_> {
    fn color_name(c: Color) -> &'static str {
        match c {
            Color::White => "White",
            Color::Blue => "Blue",
            Color::Black => "Black",
            Color::Red => "Red",
            Color::Green => "Green",
        }
    }

    fn remap_colors(&mut self, n: u64) -> u64 {
        let (Some(f), Some(t)) = (self.from_color, self.to_color) else {
            return n;
        };
        let mut set = ColorSet(n as u8);
        if set.contains(f) {
            set.remove(f);
            set.insert(t);
            self.changed = true;
        }
        set.0 as u64
    }

    fn walk(&mut self, v: J, key: Option<&str>) -> J {
        match v {
            J::Object(m) => {
                let is_token_spec = m.contains_key("subtypes") && m.contains_key("card_types");
                let mut out = serde_json::Map::new();
                for (k, v) in m {
                    let nv = match (k.as_str(), v) {
                        // Names are never changed (CR 612.2) ...
                        ("Named", v) | ("SameNameAs", v) | ("scryfall_name", v) => v,
                        // ... except a token's name defined by its creature types
                        // (CR 612.2a).
                        ("name", J::String(s)) if is_token_spec => {
                            J::String(self.replace_in_name(&s))
                        }
                        ("SetColors" | "AddColors" | "ExactColors" | "colors", J::Number(n)) => {
                            match n.as_u64() {
                                Some(x) => J::Number(self.remap_colors(x).into()),
                                None => J::Number(n),
                            }
                        }
                        ("uid", v) | ("link", v) => v,
                        (_, v) => self.walk(v, Some(k.as_str())),
                    };
                    out.insert(k, nv);
                }
                J::Object(out)
            }
            J::Array(a) => J::Array(a.into_iter().map(|x| self.walk(x, key)).collect()),
            J::String(s) => {
                // Display text: replace whole words.
                if matches!(key, Some("text")) {
                    let r = replace_words(&s, self.from, self.to);
                    return J::String(r);
                }
                if let (Some(f), Some(t)) = (self.from_color, self.to_color) {
                    if s == Self::color_name(f) {
                        self.changed = true;
                        return J::String(Self::color_name(t).to_string());
                    }
                    return J::String(s);
                }
                if s == self.from {
                    self.changed = true;
                    return J::String(self.to.to_string());
                }
                J::String(s)
            }
            other => other,
        }
    }

    fn replace_in_name(&mut self, s: &str) -> String {
        let r = replace_words(s, self.from, self.to);
        if r != s {
            self.changed = true;
        }
        r
    }
}

/// Gives every ability in a rewritten ability tree a new, stable uid.
fn renumber(v: J, from: &str, to: &str) -> J {
    match v {
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, v)| {
                    let nv = match (k.as_str(), &v) {
                        ("uid", J::Number(n)) => {
                            J::Number(derived_uid(n.as_u64().unwrap_or(0), from, to).into())
                        }
                        _ => renumber(v, from, to),
                    };
                    (k, nv)
                })
                .collect(),
        ),
        J::Array(a) => J::Array(a.into_iter().map(|x| renumber(x, from, to)).collect()),
        other => other,
    }
}

fn derived_uid(uid: u64, from: &str, to: &str) -> u64 {
    type Key = (u64, SmolStr, SmolStr);
    static UIDS: OnceLock<Mutex<HashMap<Key, u64>>> = OnceLock::new();
    let m = UIDS.get_or_init(Default::default);
    *m.lock()
        .unwrap()
        .entry((uid, SmolStr::new(from), SmolStr::new(to)))
        .or_insert_with(next_ability_uid)
}

/// Replaces whole-word occurrences of `from` (case-insensitively, keeping the case of
/// the first letter) in display text.
pub fn replace_words(text: &str, from: &str, to: &str) -> String {
    if from.is_empty() || text.is_empty() {
        return text.to_string();
    }
    let lower = text.to_lowercase();
    let lf = from.to_lowercase();
    if lower.len() != text.len() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while let Some(pos) = lower[i..].find(&lf) {
        let start = i + pos;
        let end = start + lf.len();
        let before = lower[..start].chars().next_back();
        let after = lower[end..].chars().next();
        let boundary = before.is_none_or(|c| !c.is_alphanumeric())
            && after.is_none_or(|c| !c.is_alphabetic() || lower[end..].starts_with("walk"));
        out.push_str(&text[i..start]);
        if boundary {
            let upper = text[start..]
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase());
            if upper {
                let mut cs = to.chars();
                if let Some(f) = cs.next() {
                    out.extend(f.to_uppercase());
                    out.push_str(cs.as_str());
                }
            } else {
                out.push_str(&to.to_lowercase());
            }
        } else {
            out.push_str(&text[start..end]);
        }
        i = end;
    }
    out.push_str(&text[i..]);
    out
}

/// True if `n` is the name of a nonlegendary creature card in the Oracle card reference,
/// including faces of multi-faced cards but not tokens (CR 612.7).
pub fn is_nonlegendary_creature_name(n: &str) -> bool {
    let Some(c) = mtg_data::cards().by_name(n) else {
        return false;
    };
    c.is_playable_card()
        && c.faces().iter().any(|f| {
            let ty = f.type_line.as_deref().unwrap_or("");
            let ty = ty.split('\u{2014}').next().unwrap_or("");
            f.name.eq_ignore_ascii_case(n)
                && ty.split_whitespace().any(|w| w == "Creature")
                && !ty.split_whitespace().any(|w| w == "Legendary")
        })
}

/// Gives an object the full text of a card (CR 612.6): the text representing its name,
/// mana cost, color indicator, type line, rules text, power, and toughness. Its color
/// follows its new mana cost and color indicator (CR 202.2).
pub fn take_full_text(c: &mut Characteristics, of: &Characteristics) {
    c.name = of.name.clone();
    c.all_creature_names = of.all_creature_names;
    c.mana_cost = of.mana_cost.clone();
    c.color_indicator = of.color_indicator;
    c.colors = of.colors;
    c.supertypes = of.supertypes;
    c.card_types = of.card_types;
    c.subtypes = of.subtypes.clone();
    c.abilities = of.abilities.clone();
    c.rules_text = of.rules_text.clone();
    c.power = of.power;
    c.toughness = of.toughness;
    c.loyalty = of.loyalty;
    c.defense = of.defense;
}

/// "Exchange the text boxes of [two objects]" (CR 612.5): each loses its rules text and
/// gets the rules text the other had as the effect began (abilities granted by other
/// effects aren't part of the rules text, CR 612.3). Returns the per-object modification
/// lists, or `None` if the modifications don't ask for an exchange of two objects.
pub fn exchange_mods(
    g: &Game,
    objs: &[crate::types::ObjectId],
    mods: &[Modification],
) -> Option<Vec<(crate::types::ObjectId, Vec<Modification>)>> {
    if !mods.iter().any(|m| matches!(m, Modification::ExchangeText)) || objs.len() != 2 {
        return None;
    }
    let text = |o: crate::types::ObjectId| {
        let c = &g.obj(o).copiable;
        Modification::SetText {
            abilities: c.abilities.clone(),
            text: SmolStr::new(&*c.rules_text),
        }
    };
    let rest: Vec<Modification> = mods
        .iter()
        .filter(|m| !matches!(m, Modification::ExchangeText))
        .cloned()
        .collect();
    let mut a = rest.clone();
    a.push(text(objs[1]));
    let mut b = rest;
    b.push(text(objs[0]));
    Some(vec![(objs[0], a), (objs[1], b)])
}

/// "Change the text of [objects] by replacing all instances of one [kind of word] with
/// another": the controller chooses the words as the effect resolves, then a layer 3
/// continuous effect is created (CR 612.1, 613.1c). `exclude_to` lists words the new word
/// can't be ("The new creature type can't be Wall").
pub fn exec_change_text(
    g: &mut Game,
    objs: Vec<crate::types::ObjectId>,
    words: TextWords,
    exclude_to: &[SmolStr],
    duration: &Duration,
    ctx: &Ctx,
) {
    let objs: Vec<_> = objs.into_iter().filter(|o| g.is_live(*o)).collect();
    if objs.is_empty() {
        return;
    }
    let mut options: Vec<String> = Vec::new();
    if matches!(words, TextWords::Color | TextWords::ColorOrBasicLandType) {
        options.extend(Color::ALL.iter().map(|c| c.word().to_string()));
    }
    if matches!(
        words,
        TextWords::BasicLandType | TextWords::ColorOrBasicLandType
    ) {
        options.extend(
            ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .map(|s| s.to_string()),
        );
    }
    if words == TextWords::CreatureType {
        // Offer the creature types that appear on the objects first.
        for o in &objs {
            for s in &g.obj(*o).chars.subtypes {
                if is_creature_type(s) && !options.contains(&s.to_string()) {
                    options.push(s.to_string());
                }
            }
        }
        for s in &subtype_lists().creature {
            if !options.iter().any(|o| o == s) {
                options.push(s.to_string());
            }
        }
    }
    if options.len() < 2 {
        return;
    }
    let p = ctx.controller;
    let from_i = g.ask_option(p, ctx.source, "Replace which word?", options.clone());
    let from = options[from_i].clone();
    let to_options: Vec<String> = options
        .iter()
        .filter(|w| **w != from && same_kind(w, &from) && !exclude_to.iter().any(|x| x == *w))
        .cloned()
        .collect();
    if to_options.is_empty() {
        return;
    }
    let to_i = g.ask_option(p, ctx.source, "With which word?", to_options.clone());
    let to = to_options[to_i].clone();
    let (from, to) = match (Color::from_word(&from), Color::from_word(&to)) {
        (Some(_), Some(_)) => (from.to_lowercase(), to.to_lowercase()),
        _ => (from, to),
    };
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: ctx.source,
        controller: ctx.controller,
        timestamp: ts,
        duration: duration.clone(),
        affected: Affected::Objects(objs),
        mods: vec![Modification::ChangeText {
            from: SmolStr::new(from),
            to: SmolStr::new(to),
        }],
        layer1: None,
        created_turn: g.turn.number,
    });
    g.dirty = true;
}
