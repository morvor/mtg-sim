//! Oracle text compiler: turns Scryfall oracle text into [`crate::ability`] structures.
//!
//! Pipeline:
//! 1. **Normalize** ([`normalize`]): strip reminder text, replace self-references
//!    (the card's name, "this creature", "this spell", ...) with `~`, normalize dashes.
//! 2. **Split** into abilities: one per line; bullet lines (`•`) attach to the preceding
//!    modal line ("Choose one —").
//! 3. **Classify and parse** each ability ([`parse_ability`]): keyword lines, activated
//!    abilities (`cost: effect`), triggered abilities (`When/Whenever/At ...`), static
//!    abilities, and spell text (instants/sorceries).
//!
//! Anything not understood is kept as [`AbilityKind::Unsupported`] and reported in
//! [`Compiled::unsupported`], so card support can be measured precisely.

pub mod costs;
pub mod effects;
pub mod keywords;
pub mod patterns;
pub mod phrases;
pub mod statics;
pub mod triggers;

use crate::ability::*;
use crate::card::Layout;
use crate::types::*;

/// Information about the card face being compiled.
pub struct CompileContext<'a> {
    pub card_name: &'a str,
    pub full_name: &'a str,
    pub type_line: &'a TypeLine,
    pub layout: Layout,
    pub face_index: usize,
    /// Scryfall's keyword list for the card (hints for keyword parsing).
    pub keywords: &'a [String],
    pub power: Option<&'a str>,
    pub toughness: Option<&'a str>,
}

impl CompileContext<'_> {
    pub fn is_spell(&self) -> bool {
        self.type_line.card_types.contains(CardType::Instant)
            || self.type_line.card_types.contains(CardType::Sorcery)
    }
    pub fn is_permanent(&self) -> bool {
        self.type_line.card_types.has_permanent_type()
    }
}

#[derive(Default)]
pub struct Compiled {
    pub abilities: Vec<Ability>,
    pub unsupported: Vec<String>,
}

thread_local! {
    static RAW_TEXT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// The raw (un-normalized) oracle text of the face being compiled on this thread.
/// Normalization merges the card's name and "this creature" into `~`; patterns that must
/// tell them apart (a quoted ability granted to another object, where "this creature" is
/// the object that gets it but the card's name is still the card) can look here.
pub fn raw_text() -> String {
    RAW_TEXT.with(|r| r.borrow().clone())
}

/// Compiles a face's oracle text.
pub fn compile(text: &str, ctx: &CompileContext) -> Compiled {
    RAW_TEXT.with(|r| *r.borrow_mut() = text.to_string());
    let mut out = Compiled::default();
    let norm = normalize(text, ctx);
    for block in crate::oracle_ext::group_blocks(split_abilities(&norm), ctx) {
        match parse_ability(&block, ctx) {
            Some(mut abilities) => out.abilities.append(&mut abilities),
            None => {
                out.unsupported.push(block.clone());
                out.abilities.push(AbilityDef::new(
                    AbilityKind::Unsupported(block.clone()),
                    block,
                ));
            }
        }
    }
    // Characteristic-defining abilities implied by the type line/P/T.
    if let Some(cda) = statics::star_pt_cda(&norm, ctx) {
        // Replace an unsupported CDA line if the star P/T was handled.
        out.abilities.push(cda);
    }
    out
}

/// Normalizes oracle text for parsing.
pub fn normalize(text: &str, ctx: &CompileContext) -> String {
    let mut s = strip_reminder(text);
    s = s
        .replace('\u{2212}', "-")
        .replace('\u{2014}', "—")
        .replace('\u{2019}', "'")
        .replace('\u{201C}', "\"")
        .replace('\u{201D}', "\"");
    // Self references.
    let mut names: Vec<String> = vec![ctx.card_name.to_string()];
    if ctx.full_name != ctx.card_name {
        names.push(ctx.full_name.to_string());
    }
    if ctx.type_line.supertypes.contains(Supertype::Legendary) {
        if let Some((short, _)) = ctx.card_name.split_once(',') {
            names.push(short.to_string());
        }
        if let Some((short, _)) = ctx.card_name.split_once(" of ") {
            if !short.contains(' ') {
                names.push(short.to_string());
            }
        }
    }
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for n in &names {
        if !n.is_empty() {
            s = s.replace(n.as_str(), "~");
        }
    }
    // Legendary cards are also called by the first word(s) of their name ("Whenever Edgar
    // attacks" on Edgar Markov, "Zur" for Zur the Enchanter, "Jedit Ojanen" for Jedit
    // Ojanen of Efrava): the longest such prefix first.
    if let Some(first) = short_first_name(ctx) {
        let words: Vec<&str> = ctx.card_name.split(' ').collect();
        for k in (2..words.len()).rev() {
            let prefix = words[..k].join(" ");
            if words[k - 1]
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
            {
                s = replace_word(&s, &prefix, "~");
            }
        }
        s = replace_word(&s, first, "~");
    }
    const SELF_REFS: [&str; 23] = [
        "this creature",
        "this artifact",
        "this enchantment",
        "this land",
        "this permanent",
        "this spell",
        "this card",
        "this Aura",
        "this Equipment",
        "this Vehicle",
        "this token",
        "this planeswalker",
        "this Saga",
        "this battle",
        "this Class",
        "this Case",
        "this Room",
        "this Fortification",
        "this Spacecraft",
        "this Siege",
        "this Mount",
        "this object",
        "this scheme",
    ];
    for r in SELF_REFS {
        s = replace_ci(&s, r, "~");
    }
    s
}

/// The first word of a legendary card's name when it can stand for the card: not a
/// subtype ("Ajani", "Sliver"), a title ("Captain", "General") or an article.
fn short_first_name<'a>(ctx: &CompileContext<'a>) -> Option<&'a str> {
    if !ctx.type_line.supertypes.contains(Supertype::Legendary) || ctx.card_name.contains(',') {
        return None;
    }
    let (first, _) = ctx.card_name.split_once(' ')?;
    let ok = first.chars().count() >= 3
        && first.chars().next().is_some_and(|c| c.is_uppercase())
        && first
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == '\'')
        && !first.ends_with("'s")
        && !matches!(
            first,
            "The"
                | "Captain"
                | "General"
                | "Lord"
                | "Lady"
                | "King"
                | "Queen"
                | "Space"
                | "Lander"
                | "Marit"
                | "Mitotic"
                | "Doctor"
                | "Professor"
                | "Sir"
        )
        && crate::types::subtype_kind(first).is_none();
    ok.then_some(first)
}

/// Replaces whole-word, case-sensitive occurrences of `word` ("Edgar" but not
/// "Edgarian"; "Edgar's" is fine).
fn replace_word(s: &str, word: &str, rep: &str) -> String {
    let is_word = |c: char| c.is_alphanumeric() || c == '-';
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = s[i..].find(word) {
        let start = i + pos;
        let end = start + word.len();
        let before = s[..start].chars().next_back();
        let after = s[end..].chars().next();
        // Part of a longer proper name ("a token named Tuktuk the Returned").
        let rest = &s[end..];
        let rest = rest
            .strip_prefix(" the ")
            .or_else(|| rest.strip_prefix(" of "))
            .or_else(|| rest.strip_prefix(' '))
            .unwrap_or("");
        let longer_name = rest.chars().next().is_some_and(|c| c.is_uppercase());
        out.push_str(&s[i..start]);
        if before.is_some_and(|c| is_word(c) || c == '\'')
            || after.is_some_and(is_word)
            || longer_name
        {
            out.push_str(word);
        } else {
            out.push_str(rep);
        }
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

fn replace_ci(s: &str, pat: &str, rep: &str) -> String {
    let lower = s.to_lowercase();
    let lp = pat.to_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = lower[i..].find(&lp) {
        let start = i + pos;
        let end = start + lp.len();
        // Word boundary after.
        let next = lower[end..].chars().next();
        if next.is_some_and(|c| c.is_alphanumeric()) {
            out.push_str(&s[i..end]);
            i = end;
            continue;
        }
        out.push_str(&s[i..start]);
        out.push_str(rep);
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

/// Removes parenthesized reminder text.
pub fn strip_reminder(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    // Clean up spaces left behind.
    out.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Splits normalized text into ability blocks; bullets join their modal header.
pub fn split_abilities(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if l.starts_with('•') && !out.is_empty() {
            let last = out.last_mut().unwrap();
            last.push('\n');
            last.push_str(l);
        } else {
            out.push(l.to_string());
        }
    }
    out
}

/// Parses one ability block. Returns None if not understood.
pub fn parse_ability(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    // Ability words (CR 207.2c) have no rules meaning: "Landfall — Whenever ...".
    let text = strip_ability_word(text);
    // Pluggable whole-ability patterns (level up, class levels, sagas, ...).
    if let Some(v) = crate::oracle_ext::parse_ability_ext(text, ctx) {
        return Some(v);
    }
    // Keyword lines.
    if let Some(kws) = keywords::parse_keyword_line(text, ctx) {
        return Some(kws);
    }
    // Activated abilities: "cost: effect".
    if let Some((cost_s, eff_s)) = split_cost(text) {
        if let Some(a) = parse_activated(cost_s, eff_s, text, ctx) {
            return Some(vec![a]);
        }
        return None;
    }
    // Triggered abilities.
    let lower = text.to_lowercase();
    if lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at ") {
        return triggers::parse_triggered(text, ctx).map(|a| vec![a]);
    }
    // Spell abilities for instants and sorceries.
    if ctx.is_spell() {
        if let Some(a) = statics::parse_spell_static(text, ctx) {
            return Some(vec![a]);
        }
        let body = effects::parse_body(text, ctx)?;
        return Some(vec![AbilityDef::new(
            AbilityKind::Spell(SpellAbility { body }),
            text,
        )]);
    }
    // Static abilities.
    statics::parse_static(text, ctx).map(|v| v.into_iter().collect())
}

/// Strips a leading ability word ("Landfall — ", "Threshold — ").
pub fn strip_ability_word(text: &str) -> &str {
    if let Some((head, rest)) = text.split_once(" — ") {
        let words = head.split_whitespace().count();
        let looks_like_word = words <= 4
            && !head.contains(':')
            && !head.to_lowercase().starts_with("choose")
            && head.chars().next().is_some_and(|c| c.is_uppercase())
            && !head.contains('{')
            // "Companion — [condition]" is a keyword, not an ability word (CR 702.139a);
            // so is "Forecast — [activated ability]" (CR 702.57a).
            && head != "Companion"
            && head != "Forecast";
        if looks_like_word {
            return rest;
        }
    }
    text
}

/// Splits "cost: effect" at the first top-level colon (not inside quotes).
pub fn split_cost(text: &str) -> Option<(&str, &str)> {
    let mut in_quote = false;
    for (i, ch) in text.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            ':' if !in_quote => {
                let (c, e) = (&text[..i], &text[i + 1..]);
                // Heuristic: costs are short and don't start with trigger words.
                let cl = c.to_lowercase();
                if cl.starts_with("when") || cl.starts_with("at ") || c.len() > 120 {
                    return None;
                }
                return Some((c.trim(), e.trim()));
            }
            _ => {}
        }
    }
    None
}

fn parse_activated(cost_s: &str, eff_s: &str, full: &str, ctx: &CompileContext) -> Option<Ability> {
    let (cost, loyalty) = costs::parse_cost(cost_s)?;
    // Activation restrictions at the end of the effect text.
    let (eff_text, timing, max_per_turn, any_player) = costs::split_activation_restrictions(eff_s);
    // CR 400.7j: "the exiled card" is the card the cost exiled.
    let body = match crate::zones::cost_exiled_text(&cost, eff_text) {
        Some(text) => effects::parse_body_with_it(&text, ctx, Sel::Var(crate::zones::COST_EXILED))?,
        None => effects::parse_body(eff_text, ctx)?,
    };
    // CR 605.1a: no target, could add mana, not a loyalty ability, and neither its cost
    // nor its effect moves a card to or from a library.
    let is_mana = effects::is_mana_effect(&body.effect)
        && body.targets.is_empty()
        && !loyalty
        && !touches_library(&cost, &body.effect);
    let mut act = ActivatedAbility::new(cost, body);
    act.timing = timing;
    act.max_per_turn = max_per_turn;
    act.is_loyalty = loyalty;
    act.is_mana_ability = is_mana;
    act.any_player = any_player;
    act.zone = activated_zone(cost_s, eff_text);
    Some(AbilityDef::new(AbilityKind::Activated(act), full))
}

/// `s` without its quoted parts: an ability granted in quotes ("creature cards in your
/// graveyard gain \"You may cast this card from your graveyard\"") says nothing about
/// where the ability granting it functions.
pub(crate) fn without_quotes(s: &str) -> String {
    s.split('"').step_by(2).collect::<Vec<_>>().join("\"\"")
}

/// Where an activated ability functions: one whose cost can be paid only from the hand
/// ("Exile this card from your hand", "Discard this card") functions from the hand
/// (CR 113.6j); one whose cost or effect moves the object out of a zone ("Return this card
/// from your graveyard to the battlefield") functions only in that zone (CR 113.6m).
fn activated_zone(cost: &str, effect: &str) -> FunctionZone {
    let (c, e) = (cost.to_lowercase(), without_quotes(&effect.to_lowercase()));
    let moves_self_from = |s: &str, zone: &str| {
        ["~", "this card", "this creature"]
            .iter()
            .any(|me| s.contains(&format!("{me} from your {zone}")))
    };
    if moves_self_from(&c, "hand") || c.contains("discard ~") || c.contains("discard this card") {
        FunctionZone::Hand
    } else if moves_self_from(&c, "graveyard") || moves_self_from(&e, "graveyard") {
        FunctionZone::Graveyard
    } else {
        FunctionZone::Battlefield
    }
}

/// Whether a cost or effect moves cards to or from a library (drawing, milling, searching,
/// surveilling, putting cards into a library, ...).
fn touches_library(cost: &Cost, effect: &Effect) -> bool {
    let text = format!(
        "{} {}",
        serde_json::to_string(cost).unwrap_or_default(),
        serde_json::to_string(effect).unwrap_or_default()
    );
    ["Library", "Draw", "Mill", "Search", "Surveil", "Explore"]
        .iter()
        .any(|w| text.contains(w))
}

/// Shifts `Sel::Target(i)` references by `offset` (used when combining spell bodies).
pub fn offset_targets(e: &Effect, offset: u8) -> Effect {
    if offset == 0 {
        return e.clone();
    }
    let json = serde_json::to_value(e).unwrap();
    let shifted = shift_json(json, offset);
    serde_json::from_value(shifted).unwrap_or_else(|_| e.clone())
}

fn shift_json(v: serde_json::Value, offset: u8) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        J::Object(mut m) => {
            if m.len() == 1 {
                if let Some(J::Number(n)) = m.get("Target") {
                    let k = n.as_u64().unwrap_or(0) + offset as u64;
                    m.insert("Target".into(), J::Number(k.into()));
                    return J::Object(m);
                }
            }
            J::Object(
                m.into_iter()
                    .map(|(k, v)| (k, shift_json(v, offset)))
                    .collect(),
            )
        }
        J::Array(a) => J::Array(a.into_iter().map(|x| shift_json(x, offset)).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_reminder_text() {
        assert_eq!(
            strip_reminder("Flying (This creature can't be blocked.)"),
            "Flying"
        );
    }

    #[test]
    fn splits_cost() {
        assert_eq!(split_cost("{T}: Add {G}."), Some(("{T}", "Add {G}.")));
        assert_eq!(split_cost("Whenever you gain life, draw: nope"), None);
    }
}
