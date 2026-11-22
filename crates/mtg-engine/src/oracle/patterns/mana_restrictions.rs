//! Spending restrictions on mana (CR 106.6): "Spend this mana only to cast Dragon spells
//! or activate abilities of Dragons", "Spend this mana only to cast a multicolored spell",
//! "Spend this mana only to activate abilities of land sources".
//!
//! Grammar (lowercase, after "spend this mana only "):
//!
//! ```text
//! purposes := purpose ([","] ("or" | "and" | "and/or") ["to"] purpose)*
//! purpose  := ["to "] "cast " spells | ["to "] "activate " abilities
//! spells   := "spells" | "a spell" | [a|an] OBJECT-PHRASE-ending-in-spell(s) ["or" [a|an] ...]
//! abilities:= "abilities" | "an ability" | ("abilities of " | "an ability of " [a|an])
//!             OBJECT-PHRASE [" source" | " sources"]
//! ```
//!
//! A spell purpose is checked against the spell being cast (on the stack while its costs
//! are paid, CR 601.2a, 601.2h), an ability purpose against the source of the ability
//! being activated. Purposes this grammar doesn't understand (paying cumulative upkeep,
//! turning permanents face up, "spells from your graveyard") leave the text unsupported.
//!
//! The sentence can follow any mana effect ("Add two mana in any combination of colors.
//! Spend this mana only to cast Dragon spells."), so it's a follow-up to the previous
//! sentence: every AddMana in it gets the restriction.

use super::FollowupPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::mana::{FaceUpWay, ManaRestriction, SpendFilter, SpendPurpose};

/// "pay a mana cost to turn a manifested creature face up" (Qarsi Deceiver): one purpose.
const MANIFEST_FACE_UP: &str = "pay a mana cost to turn a manifested creature face up";
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Parses the purposes after "spend this mana only ".
pub fn parse_purposes(r: &str) -> Option<ManaRestriction> {
    // "pay a mana cost to turn a manifested creature face up" is one purpose (its "to
    // turn" doesn't start another).
    let r = end(r)
        .trim()
        .replace(MANIFEST_FACE_UP, "pay manifest-face-up-cost");
    let words: Vec<&str> = r.split(' ').filter(|w| !w.is_empty()).collect();
    // Split at each "cast"/"activate" verb that starts a purpose.
    let mut chunks: Vec<(&str, Vec<&str>)> = Vec::new();
    for (i, w) in words.iter().enumerate() {
        let starts = matches!(
            *w,
            "cast" | "activate" | "gain" | "pay" | "foretell" | "unlock" | "turn"
        )
            && (i == 0
                || matches!(words[i - 1], "to" | "or" | "and" | "and/or")
                || words[i - 1].ends_with(','));
        if starts {
            chunks.push((w, Vec::new()));
        } else if let Some((_, c)) = chunks.last_mut() {
            c.push(w);
        } else if *w != "to" {
            return None;
        }
    }
    if chunks.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for (verb, mut c) in chunks {
        // Drop the connector before the next purpose ("... spells or to", "... spells,").
        while matches!(c.last(), Some(&("or" | "and" | "and/or" | "to"))) {
            c.pop();
        }
        let joined = c.join(" ");
        let text = joined.trim_end_matches(',');
        out.push(match verb {
            "cast" => cast_purpose(text)?,
            // "to gain a Class level" (CR 716.2c).
            "gain" if text == "a class level" => ManaRestriction::ClassLevel,
            "gain" => return None,
            "pay" => pay_purpose(text)?,
            // "to foretell cards", "to foretell a card from your hand": the foretell special
            // action's cost (CR 702.143a; Karfell Harbinger's ruling).
            "foretell" if matches!(text, "cards" | "a card" | "a card from your hand") => {
                ManaRestriction::Purpose(SpendPurpose::Keyword(KeywordKind::Foretell))
            }
            "foretell" => return None,
            // "to unlock doors" (CR 709.5e).
            "unlock" if matches!(text, "doors" | "a door") => {
                ManaRestriction::Purpose(SpendPurpose::UnlockDoor)
            }
            "unlock" => return None,
            // "to turn permanents face up" (CR 116.2b): any way.
            "turn"
                if matches!(
                    text,
                    "permanents face up"
                        | "a permanent face up"
                        | "creatures face up"
                        | "a creature face up"
                ) =>
            {
                ManaRestriction::TurnFaceUp(None)
            }
            "turn" => return None,
            _ => activate_purpose(text)?,
        });
    }
    Some(if out.len() == 1 {
        out.pop().unwrap()
    } else {
        ManaRestriction::AnyOf(out)
    })
}

/// Phrases whose filters would be checked against something other than the spell's own
/// characteristics on the stack (where it was cast from, a choice made for the mana's
/// source, the last card exiled with it), or conditions this grammar doesn't track.
///
/// "Of the chosen type" is bound to the source's choice as the mana is produced
/// (`mana_abilities::bind_restriction`); "of that color" is the chosen color only after
/// mana of the chosen color (see [`that_color_ok`]).
fn unsupported_spell_words(s: &str) -> bool {
    s.contains('~')
        || s.contains("that type")
        || s.contains("starting deck")
        || s
            .split([' ', ',', '\''])
            .any(|w| {
                matches!(
                    w,
                    "from"
                        | "graveyard"
                        | "graveyards"
                        | "exile"
                        | "exiled"
                        | "hand"
                        | "library"
                        | "commander"
                        | "own"
                        | "owned"
                )
            })
}

/// "Spend this mana only to cast monocolored spells of that color" names the color chosen
/// for the source only when the mana is "of the chosen color" (Throne of Eldraine).
pub(crate) fn that_color_ok(text: &str, e: &Effect) -> bool {
    if !text.contains("that color") {
        return true;
    }
    matches!(
        e,
        Effect::AddMana {
            mana: ManaProduction::ChosenColor(_),
            ..
        }
    )
}

/// "to pay [costs]": "cumulative upkeep costs" (CR 702.24a), "a disturb cost" (casting
/// the spell with disturb, CR 702.146a), "a cost that contains {C}".
fn pay_purpose(s: &str) -> Option<ManaRestriction> {
    Some(match s {
        "cumulative upkeep costs" | "a cumulative upkeep cost" => {
            ManaRestriction::Purpose(SpendPurpose::Keyword(KeywordKind::CumulativeUpkeep))
        }
        "a disturb cost" | "disturb costs" => ManaRestriction::CastSpell(SpendFilter::new(
            Filter::and(vec![
                Filter::Spell,
                Filter::CastWithCost(crate::kw::disturb::DISTURB.into()),
            ]),
        )),
        "a cost that contains {c}" | "costs that contain {c}" => {
            ManaRestriction::CostWithColorless
        }
        // Turning a face-down permanent face up one way (CR 702.37e, 701.40b).
        "a morph cost" | "morph costs" => ManaRestriction::TurnFaceUp(Some(FaceUpWay::Morph)),
        "a disguise cost" | "disguise costs" => {
            ManaRestriction::TurnFaceUp(Some(FaceUpWay::Disguise))
        }
        "manifest-face-up-cost" => ManaRestriction::TurnFaceUp(Some(FaceUpWay::Manifest)),
        _ => return None,
    })
}

/// "equip abilities", "an equip ability", "power-up abilities": the activated abilities
/// of a keyword (CR 702.6a, 702.193a).
fn keyword_abilities(s: &str) -> Option<KeywordKind> {
    let w = s
        .strip_suffix(" abilities")
        .or_else(|| s.strip_prefix("an ").and_then(|r| r.strip_suffix(" ability")))
        .or_else(|| s.strip_prefix("a ").and_then(|r| r.strip_suffix(" ability")))?;
    KeywordKind::from_name(w)
}

/// "to cast [spells]".
fn cast_purpose(s: &str) -> Option<ManaRestriction> {
    if matches!(s, "spells" | "a spell") {
        return Some(ManaRestriction::SpellsOnly);
    }
    if unsupported_spell_words(s) {
        return None;
    }
    let f = spell_alternatives(s)?;
    Some(ManaRestriction::CastSpell(SpendFilter::new(f)))
}

/// "a Dragon spell or an Omen spell", "Dragon creature spells", "instant and sorcery
/// spells", "creature spells with mana value 4 or greater".
pub(crate) fn spell_alternatives(s: &str) -> Option<Filter> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    // "an Assassin spell or a spell that has freerunning" (Brotherhood Headquarters): two
    // alternatives, each with its own article.
    for sep in [" spell or a ", " spell or an "] {
        if let Some(i) = s.find(sep) {
            let first = &s[..i + " spell".len()];
            let second = &s[i + " spell or ".len()..];
            if let (Some(f), Some(g)) = (spell_alternatives(first), spell_alternatives(second.trim())) {
                return Some(Filter::Or(vec![f, g]));
            }
        }
    }
    if let Some(f) = crate::oracle::patterns::mana_grammar::spell_phrase(s) {
        return Some(f);
    }
    let Some((f, _, tail)) = parse_object_phrase(s) else {
        return crate::oracle::patterns::mana_grammar::subtype_spell(s);
    };
    // "Dragon creature spell": a second narrowing noun.
    let (f, tail) = match tail
        .trim_start()
        .strip_prefix("spells")
        .or_else(|| tail.trim_start().strip_prefix("spell"))
    {
        Some(t) if !names_spells(&f) => (Filter::and(vec![f, Filter::Spell]), t),
        _ => (f, tail),
    };
    if !names_spells(&f) {
        return None;
    }
    let tail = end(tail).trim();
    if tail.is_empty() {
        return Some(f);
    }
    let rest = tail.strip_prefix("or ")?;
    let g = spell_alternatives(rest)?;
    Some(Filter::Or(vec![f, g]))
}

/// The phrase's head noun is "spell(s)" (possibly narrowing types: "creature spell").
fn names_spells(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(names_spells),
        Filter::Or(v) => !v.is_empty() && v.iter().all(names_spells),
        _ => false,
    }
}

/// "to activate [abilities]".
fn activate_purpose(s: &str) -> Option<ManaRestriction> {
    if matches!(s, "abilities" | "an ability") {
        return Some(ManaRestriction::AbilitiesOnly);
    }
    if let Some(k) = keyword_abilities(s) {
        return Some(ManaRestriction::Purpose(SpendPurpose::Keyword(k)));
    }
    let r = s
        .strip_prefix("abilities of ")
        .or_else(|| s.strip_prefix("an ability of "))?;
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))
        .unwrap_or(r);
    // "a source of the chosen type", "a creature source of the chosen type": an object
    // of that type in any zone (Eclipsed Realms's ruling).
    if let Some(head) = r
        .strip_suffix(" of the chosen type")
        .and_then(|h| h.strip_suffix("source").or_else(|| h.strip_suffix("sources")))
    {
        let f = match head.trim() {
            "" => Filter::ChosenType,
            h => {
                let (f, _, tail) = parse_object_phrase(h)?;
                if !end(tail).trim().is_empty() || names_spells(&f) {
                    return None;
                }
                Filter::and(vec![f, Filter::ChosenType])
            }
        };
        return Some(ManaRestriction::ActivateAbilityOf(SpendFilter::new(f)));
    }
    // CR 109.2: "abilities of creatures" are abilities of creature permanents; "creature
    // sources" are creature objects in any zone.
    let (r, any_zone) = match r
        .strip_suffix(" sources")
        .or_else(|| r.strip_suffix(" source"))
    {
        Some(x) => (x, true),
        None => (r, false),
    };
    if unsupported_spell_words(r) || r.contains("card") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).trim().is_empty() || names_spells(&f) {
        return None;
    }
    let f = if any_zone {
        f
    } else {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
    };
    Some(ManaRestriction::ActivateAbilityOf(SpendFilter::new(f)))
}

/// Sets the restriction on every AddMana in `e`. False if there is none.
pub fn restrict(e: &mut Effect, r: &ManaRestriction) -> bool {
    match e {
        Effect::AddMana { restriction, .. } => {
            if restriction.is_some() {
                return false;
            }
            *restriction = Some(r.clone());
            true
        }
        Effect::Seq(v) => {
            let mut any = false;
            for x in v.iter_mut() {
                if contains_add_mana(x) {
                    if !restrict(x, r) {
                        return false;
                    }
                    any = true;
                }
            }
            any
        }
        Effect::ChooseOne { options, .. } => {
            let mut any = false;
            for (_, x) in options.iter_mut() {
                if !restrict(x, r) {
                    return false;
                }
                any = true;
            }
            any
        }
        Effect::If {
            then, otherwise, ..
        } => {
            let a = !contains_add_mana(then) || restrict(then, r);
            let b = !contains_add_mana(otherwise) || restrict(otherwise, r);
            a && b && (contains_add_mana(then) || contains_add_mana(otherwise))
        }
        Effect::May { effect, .. } => restrict(effect, r),
        Effect::PersistentMana(inner) | Effect::CombatMana(inner) => restrict(inner, r),
        Effect::AddManaWithSpentTrigger { add, .. } => restrict(add, r),
        _ => false,
    }
}

/// Whether `e` adds mana somewhere (other effects in a sequence keep no restriction).
pub fn contains_add_mana(e: &Effect) -> bool {
    match e {
        Effect::AddMana { .. } | Effect::AddManaWithSpentTrigger { .. } => true,
        Effect::Seq(v) => v.iter().any(contains_add_mana),
        Effect::ChooseOne { options, .. } => options.iter().any(|(_, x)| contains_add_mana(x)),
        Effect::If {
            then, otherwise, ..
        } => contains_add_mana(then) || contains_add_mana(otherwise),
        Effect::May { effect, .. } => contains_add_mana(effect),
        Effect::PersistentMana(inner) | Effect::CombatMana(inner) => contains_add_mana(inner),
        _ => false,
    }
}

/// "Spend this mana only to ..." after a sentence that adds mana.
fn f_spend_only(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !l.starts_with("spend this mana only ") && !l.starts_with("this mana can't be spent ") {
        return false;
    }
    let Some(restriction) = crate::oracle::patterns::r106_mana::parse_restriction(l) else {
        return false;
    };
    that_color_ok(l, prev) && restrict(prev, &restriction)
}

inventory::submit! { FollowupPattern { name: "mana: spend this mana only to ...", priority: 50, apply: f_spend_only } }
