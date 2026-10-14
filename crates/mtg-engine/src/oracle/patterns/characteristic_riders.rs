//! Characteristic riders on permanents put onto the battlefield: "Return it to the
//! battlefield ... It's a Spirit Detective." (Otherworldly Escort), "It's a land named
//! Moon. It gains \"{T}: Add {C}.\"" (Princess Yue), "It has flying and is an Angel in
//! addition to its other types." (Valkyrie's Call), "The creature is a Vampire in
//! addition to its other types." (Drana), "They're black Zombies in addition to their
//! other colors and types and they gain decayed." (Ghouls' Night Out), "They are Food
//! artifacts with \"...\"" (Supper for Spiders), "It's an artifact." (Tom, Bert, and
//! William).
//!
//! The sentence describes what the permanent is as it enters: a continuous effect of the
//! resolving instruction that applies to it from the moment it's on the battlefield
//! (CR 611.2c, 611.3c, 614.1c). It's read as the "becomes" sentence it amounts to ("it
//! becomes a Spirit Detective", CR 205.1a: new subtypes replace the old ones of their
//! kind) and kept as the destination's modifications (`Destination::with_mods`).
//! A permanent that enters as an Aura this way is attached to an object its controller
//! chooses as it enters (CR 303.4f; see `attach::entry_attachment`), and stays where it is
//! if it can't legally enchant anything (CR 303.4g).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use super::statics::Subject;
use crate::types::CardType;

/// The subjects that name the object an earlier instruction put onto the battlefield.
const SUBJECTS: [(&str, &str); 8] = [
    ("it's ", "is "),
    ("it is ", "is "),
    ("it has ", "has "),
    ("the creature is ", "is "),
    ("that creature is ", "is "),
    ("they're ", "is "),
    ("they are ", "is "),
    ("the permanent is ", "is "),
];

/// The last instruction of `e`, if it puts objects onto the battlefield.
fn last_battlefield_move(e: &mut Effect) -> Option<&mut Destination> {
    match e {
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(to),
        Effect::Seq(v) => v.last_mut().and_then(last_battlefield_move),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_battlefield_move(then),
        Effect::May { effect, .. } => last_battlefield_move(effect),
        Effect::Reflexive { body } => last_battlefield_move(&mut body.effect),
        _ => None,
    }
}

/// "is a Spirit in addition to its other types and gains decayed" → "becomes a Spirit
/// in addition to its other types and gains decayed": the predicates of a "becomes"
/// sentence about one object.
fn as_becomes(pred: &str) -> String {
    let p = pred
        .replace("their other colors and types", "its other colors and types")
        .replace("their other types", "its other types")
        .replace(" and they gain ", " and gains ")
        .replace(" and they lose ", " and loses ")
        .replace(" and they have ", " and gains ")
        .replace(", and it loses ", " and loses ")
        .replace(" and it loses ", " and loses ")
        .replace(" and is ", " and becomes ")
        .replace(", is ", ", becomes ");
    if let Some(r) = p.strip_prefix("is ") {
        format!("becomes {r}")
    } else if let Some(r) = p.strip_prefix("has ") {
        format!("gains {r}")
    } else {
        p
    }
}

/// Plural type words of a group ("black Zombies", "Food artifacts", "4/4 creatures") as
/// one object's ("a black Zombies": type words read plurals as their singulars).
fn singular_types(p: &str) -> String {
    match p.strip_prefix("becomes ") {
        Some(r) if !r.starts_with("a ") && !r.starts_with("an ") => format!("becomes a {r}"),
        _ => p.to_string(),
    }
}

/// The modifications of a rider about the object `b.it` refers to.
fn rider_mods(l: &str, b: &mut Builder) -> Option<Vec<Modification>> {
    let l = end(l);
    let (subj, verb) = SUBJECTS.iter().find(|(s, _)| l.starts_with(s))?;
    let pred = format!("{verb}{}", &l[subj.len()..]);
    let mut sentence = as_becomes(&pred);
    if subj.starts_with("they") {
        sentence = singular_types(&sentence);
    }
    let saved = (b.targets.len(), b.it.clone());
    if !sentence.contains('"') {
        // The object put onto the battlefield (a creature card, or one the rider makes a
        // creature).
        let subj = Subject {
            filter: Filter::Any,
            it: Some(b.it.clone()),
            hint: CardType::Creature,
            lands: false,
            creatures: true,
        };
        let mut mods = Vec::new();
        for p in super::becomes_grammar::split_predicates(&sentence) {
            match super::becomes_grammar::predicate_mods(p, &subj, b) {
                Some(m) => mods.extend(m),
                None => {
                    b.targets.truncate(saved.0);
                    b.it = saved.1;
                    return None;
                }
            }
        }
        if b.targets.len() != saved.0 || mods.is_empty() {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
        return Some(mods);
    }
    if let Some(mods) = bundle_with_quotes(&sentence, b) {
        return Some(mods);
    }
    let e = crate::oracle::effects::parse_sentence(&format!("it {sentence}"), b);
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
    };
    let Some(Effect::Modify {
        mods,
        duration: Duration::Permanent,
        ..
    }) = e
    else {
        restore(b);
        return None;
    };
    if b.targets.len() != saved.0 || mods.is_empty() {
        restore(b);
        return None;
    }
    b.it = saved.1;
    Some(mods)
}

/// "becomes an Aura enchantment with enchant creature you control and \"[ability]\" and
/// loses all other abilities": the bundle's modifications, with its quoted abilities
/// granted to the object itself (layer 6 after the abilities it loses, CR 613.1f).
fn bundle_with_quotes(sentence: &str, b: &mut Builder) -> Option<Vec<Modification>> {
    let (masked, quotes) = super::statics::mask_quotes(sentence)?;
    let mut rest = masked.trim_end_matches(',').to_string();
    let mut lose = false;
    for tail in [", and loses all other abilities", " and loses all other abilities"] {
        if let Some(r) = rest.strip_suffix(tail) {
            rest = r.trim_end_matches(',').to_string();
            lose = true;
            break;
        }
    }
    let mut idx = Vec::new();
    for k in 0..quotes.len() {
        let q = format!("\"#{k}\"");
        let mut found = false;
        for sep in [", and ", " and ", ", "] {
            let pat = format!("{sep}{q}");
            if let Some(i) = rest.find(&pat) {
                rest.replace_range(i..i + pat.len(), "");
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
        idx.push(k);
    }
    if idx.is_empty() || rest.contains('"') {
        return None;
    }
    let subj = Subject {
        filter: Filter::Any,
        it: Some(Sel::This),
        hint: CardType::Enchantment,
        lands: false,
        creatures: true,
    };
    let saved = b.targets.len();
    let mut mods = Vec::new();
    if lose {
        mods.push(Modification::RemoveAllAbilities);
    }
    for p in super::becomes_grammar::split_predicates(&rest) {
        mods.extend(super::becomes_grammar::predicate_mods(p, &subj, b)?);
    }
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    let hint = if mods.iter().any(|m| matches!(m, Modification::SetTypes { types, .. } if types.contains(&CardType::Creature))) {
        CardType::Creature
    } else {
        CardType::Enchantment
    };
    let granted = super::grant_grammar::compile_quotes(&idx, &quotes, hint, true, b)?;
    mods.extend(granted.into_iter().map(Modification::AddAbility));
    Some(mods)
}

/// "[return it to the battlefield]. It's [characteristics]."
fn rider_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    {
        let Some(to) = last_battlefield_move(prev) else {
            return false;
        };
        if !to.with_mods.is_empty() {
            return false;
        }
    }
    let Some(mods) = rider_mods(l, b) else {
        return false;
    };
    let Some(to) = last_battlefield_move(prev) else {
        return false;
    };
    to.with_mods = mods;
    true
}

inventory::submit! { FollowupPattern { name: "characteristic riders: [put it onto the battlefield]. it's [characteristics]", priority: 1150, apply: rider_followup } }

/// A rider that adds to the one before it: "She's a land named Moon. She gains \"{T}: Add
/// {C}.\"" (Princess Yue): "it gains [abilities]" after a rider is part of what the
/// permanent is as it enters.
fn rider_gains_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.starts_with("it gains ") {
        return false;
    }
    {
        let Some(to) = last_battlefield_move(prev) else {
            return false;
        };
        if to.with_mods.is_empty() {
            return false;
        }
    }
    let saved = (b.targets.len(), b.it.clone());
    let e = crate::oracle::effects::parse_sentence(l, b);
    let Some(Effect::Modify {
        mods,
        duration: Duration::Permanent,
        ..
    }) = e
    else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    if b.targets.len() != saved.0 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    }
    let Some(to) = last_battlefield_move(prev) else {
        return false;
    };
    to.with_mods.extend(mods);
    true
}

inventory::submit! { FollowupPattern { name: "characteristic riders: ... it gains [abilities] (as it enters)", priority: 1150, apply: rider_gains_followup } }

/// "Return it to the battlefield and it's a 3/3 Robot artifact creature with flying."
/// (Brilliance Unleashed): the rider joined to the instruction in one sentence.
fn and_its_rider(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, rider) = l.split_once(" and it's ")?;
    if !head.contains(" to the battlefield") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = crate::oracle::effects::parse_clause(head, b)?;
    let Some(mods) = rider_mods(&format!("it's {rider}"), b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    let to = last_battlefield_move(&mut e)?;
    if !to.with_mods.is_empty() {
        return None;
    }
    to.with_mods = mods;
    Some(e)
}

inventory::submit! { EffectPattern { name: "characteristic riders: return it to the battlefield and it's [characteristics]", priority: 1150, parse: and_its_rider } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    #[test]
    fn rider_predicates() {
        let tl = TypeLine::default();
        let ctx = CompileContext {
            card_name: "X",
            full_name: "X",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let mut b = Builder::new(&ctx);
        let s = singular_types(&as_becomes(
            "is black zombies in addition to their other colors and types and they gain decayed",
        ));
        assert_eq!(s, "becomes a black zombies in addition to its other colors and types and gains decayed");
        let subj = Subject {
            filter: Filter::Any,
            it: Some(Sel::This),
            hint: CardType::Creature,
            lands: false,
            creatures: true,
        };
        let parts = super::super::becomes_grammar::split_predicates(&s);
        assert_eq!(parts.len(), 2, "{parts:?}");
        for p in parts {
            assert!(super::super::becomes_grammar::predicate_mods(p, &subj, &mut b).is_some(), "{p}");
        }
    }
}

/// "[subject] is a Dragon with base power and toughness 4/4, flying, and \"[ability]\""
/// (Goddric, Cloaked Reveler), "Enchanted creature loses all abilities and is a Citizen
/// with base power and toughness 1/1 and \"{T}: Add {C}\" named Humble Merchant."
/// (Honest Work): a static characteristic bundle whose quoted abilities the objects have
/// (layer 6, after the abilities they lose, CR 613.1f), with its name (layer 3), types
/// (layer 4) and base power and toughness (layer 7b).
fn static_bundle_with_quotes(
    l: &str,
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell() || !l.contains('"') {
        return None;
    }
    let (masked, quotes) = super::statics::mask_quotes(end(l))?;
    // The quoted abilities in the bundle: ", and \"#k\"", " and \"#k\"", ", \"#k\"".
    let mut rest = masked.clone();
    let mut idx = Vec::new();
    for k in 0..quotes.len() {
        let q = format!("\"#{k}\"");
        let mut found = false;
        for sep in [", and ", " and ", ", "] {
            let pat = format!("{sep}{q}");
            if let Some(i) = rest.find(&pat) {
                rest.replace_range(i..i + pat.len(), "");
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
        idx.push(k);
    }
    if rest.contains('"') {
        return None;
    }
    let (subj_text, pred) = rest.split_once(" is ")?;
    let (subj_text, lose) = match subj_text.strip_suffix(" loses all abilities and") {
        Some(s) => (s, true),
        None => (subj_text, false),
    };
    let subj = super::statics::parse_subject(subj_text, Some(&Sel::This), ctx)?;
    let mut b = Builder::new(ctx);
    let mut mods = Vec::new();
    if lose {
        mods.push(Modification::RemoveAllAbilities);
    }
    mods.extend(super::becomes_grammar::predicate_mods(
        &format!("becomes {pred}"),
        &subj,
        &mut b,
    )?);
    if !b.targets.is_empty() {
        return None;
    }
    let to_source = matches!(subj.filter, Filter::Source);
    let granted = super::grant_grammar::compile_quotes(&idx, &quotes, subj.hint, to_source, &b)?;
    mods.extend(granted.into_iter().map(Modification::AddAbility));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: subj.filter,
            mods,
        })),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "characteristic riders: [objects] is [types with base p/t, keywords and \"abilities\"] [named ...]", priority: 1090, parse: static_bundle_with_quotes } }

/// "If ~ is a Detective, it becomes a legendary creature named Mileva, the Stalwart, it
/// has base power and toughness 5/5, and it gains \"[ability]\"." (Tenth District Hero):
/// one characteristic-changing effect whose predicates repeat their subject, and whose
/// last predicate grants quoted abilities (layer 6, CR 613.1f).
fn becomes_it_has_it_gains(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" becomes ") || !(l.contains(", it has ") || l.contains(", and it gains ")) {
        return None;
    }
    let (masked, quotes) = super::statics::mask_quotes(l)?;
    let mut core = masked
        .replace(", it has ", ", has ")
        .replace(", and it gains ", ", and gains ");
    // Trailing quoted abilities: ", and gains \"#0\"" / " and \"#1\"".
    let mut idx = Vec::new();
    loop {
        let Some(k) = core
            .strip_suffix('"')
            .and_then(|r| r.rsplit_once("\"#"))
            .and_then(|(_, n)| n.parse::<usize>().ok())
        else {
            break;
        };
        let q = format!("\"#{k}\"");
        let head = core.strip_suffix(q.as_str())?;
        let head = head
            .strip_suffix(", and gains ")
            .or_else(|| head.strip_suffix(" and gains "))
            .or_else(|| head.strip_suffix(" and "))
            .or_else(|| head.strip_suffix(", "))?;
        idx.insert(0, k);
        core = head.to_string();
        if !core.ends_with('"') {
            break;
        }
    }
    if core.contains('"') {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = crate::oracle::effects::parse_sentence(&core, b)?;
    let (what, mods) = match &mut e {
        Effect::Modify { what, mods, .. } => (what.clone(), mods),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => match &mut **then {
            Effect::Modify { what, mods, .. } => (what.clone(), mods),
            _ => return None,
        },
        _ => return None,
    };
    if !idx.is_empty() {
        let to_source = matches!(what, Sel::This);
        let granted =
            super::grant_grammar::compile_quotes(&idx, &quotes, CardType::Creature, to_source, b);
        let Some(granted) = granted else {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        };
        mods.extend(granted.into_iter().map(Modification::AddAbility));
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "characteristic riders: [subject] becomes [...], it has [...], and it gains [...]", priority: 1090, parse: becomes_it_has_it_gains } }

/// "[object]'s base toughness becomes equal to 1 plus the number of creature cards in
/// your graveyard" (Wall of Tombstones), "... base power becomes equal to ...": a base
/// power or toughness set by a resolving effect (layer 7b, CR 613.4b), the value fixed as
/// the effect begins (CR 608.2h), lasting indefinitely.
fn base_stat_becomes(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (obj, stat, value) = [("'s base power becomes equal to ", true), ("'s base toughness becomes equal to ", false)]
        .iter()
        .find_map(|(p, power)| l.split_once(p).map(|(o, v)| (o, *power, v)))?;
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, rest) = crate::oracle::effects::object_ref(obj, b)?;
        if !rest.trim().is_empty() || matches!(what, Sel::None | Sel::All(_)) {
            return None;
        }
        let (v, rest) = crate::oracle::statics::parse_value_phrase(value, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        let m = if stat {
            Modification::SetPT(Some(v), None)
        } else {
            Modification::SetPT(None, Some(v))
        };
        Some(Effect::Modify {
            what,
            mods: vec![m],
            duration: Duration::Permanent,
        })
    })();
    if result.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    result
}

inventory::submit! { EffectPattern { name: "characteristic riders: [object]'s base toughness becomes equal to [value]", priority: 1090, parse: base_stat_becomes } }

/// "~'s base power is equal to the number of cards in your hand." granted in quotes (Ms.
/// Marvel, Kamala Khan): a granted ability that sets power isn't characteristic-defining
/// (CR 604.3a), so it's an ordinary static ability setting the base power (layer 7b).
fn base_stat_is_equal(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (power, value) = if let Some(v) = l.strip_prefix("~'s base power is equal to ") {
        (true, v)
    } else {
        (false, l.strip_prefix("~'s base toughness is equal to ")?)
    };
    let mut b = Builder::new(ctx);
    b.it = Sel::This;
    let (v, rest) = crate::oracle::statics::parse_value_phrase(value, &mut b)?;
    if !rest.trim().is_empty() || !b.targets.is_empty() {
        return None;
    }
    let m = if power {
        Modification::SetPT(Some(v), None)
    } else {
        Modification::SetPT(None, Some(v))
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![m],
        })),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "characteristic riders: ~'s base power is equal to [value]", priority: 1090, parse: base_stat_is_equal } }

/// One mode written as a characteristic bundle: "3/1 Human Warrior with trample and
/// haste", "2/2 white Fox with vigilance": its power and toughness, the words before
/// "with" (colors and types) and the abilities after it.
fn bundle_parts(mode: &str) -> Option<(&str, &str, &str)> {
    let mode = mode.trim().trim_start_matches('•').trim().trim_end_matches('.');
    let (pt, rest) = mode.split_once(' ')?;
    let (p, t) = pt.split_once('/')?;
    if p.parse::<i32>().is_err() || t.parse::<i32>().is_err() {
        return None;
    }
    let (words, abilities) = rest.split_once(" with ")?;
    Some((pt, words, abilities))
}

/// "choose one at random. Create a red and white creature token with those
/// characteristics. / • 3/1 Human Warrior with trample and haste. / ..." (Outlaws'
/// Merriment, Genku), "Choose one. Until end of turn, target creature you control has that
/// base power and toughness, becomes that creature type, and gains that ability. / • 1/3
/// Turtle with hexproof. / ..." (Wild Shape): modes listed as characteristic bundles
/// (CR 700.2). Each mode is the instruction with its bundle's characteristics, and the
/// ability is parsed as the modal ability it amounts to.
fn bundle_modes(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let mut lines = block.lines();
    let header = lines.next()?;
    let modes: Vec<&str> = lines.collect();
    if modes.len() < 2 || !modes.iter().all(|m| m.trim_start().starts_with('•')) {
        return None;
    }
    let lower = header.to_lowercase();
    let c = lower.find("choose ")?;
    let dot = c + lower[c..].find(". ")?;
    let (choose, instr) = (&header[..dot], header[dot + 2..].trim());
    let instr_l = instr.to_lowercase();
    let instr_l = end(&instr_l);
    let mut new_modes = Vec::new();
    for m in &modes {
        let (pt, words, abilities) = bundle_parts(m)?;
        let mode = if let Some(colors) = instr_l
            .strip_prefix("create a ")
            .and_then(|r| r.strip_suffix("creature token with those characteristics"))
        {
            // The header's colors (original case) and the mode's words.
            let colors = &instr["create a ".len().."create a ".len() + colors.len()];
            format!("Create a {pt} {colors}{words} creature token with {abilities}.")
        } else {
            let (lead, subj) = match instr_l.strip_prefix("until end of turn, ") {
                Some(r) => ("Until end of turn, ", r),
                None => ("", instr_l),
            };
            let subj = subj.strip_suffix(
                " has that base power and toughness, becomes that creature type, and gains that ability",
            )?;
            let subj = &instr[instr.len() - instr_l.len() - (instr.len() - instr.trim_end_matches('.').len())..];
            let subj = &subj[lead.len()..lead.len() + instr_l.len() - lead.len() - " has that base power and toughness, becomes that creature type, and gains that ability".len()];
            // The creature types only: a mode's words are its creature types.
            if words.split(' ').any(|w| crate::types::Color::from_word(w).is_some()) {
                return None;
            }
            format!("{lead}{subj} has base power and toughness {pt}, becomes a {words}, and gains {abilities}.")
        };
        new_modes.push(format!("• {mode}"));
    }
    let rebuilt = format!("{choose} —\n{}", new_modes.join("\n"));
    if rebuilt.to_lowercase().contains("those characteristics") {
        return None;
    }
    let v = crate::oracle::parse_ability(&rebuilt, ctx)?;
    Some(
        v.into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), block, a.link))
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "characteristic riders: modes listed as characteristic bundles", priority: 60, parse: bundle_modes } }

/// "If you do, it gains equip {3} and \"Equipped creature gets +3/+3\" and loses all other
/// abilities." (The Irencrag): the object has only the abilities it gains (layer 6: it
/// loses its abilities, then gets those, in the order the effect lists, CR 613.1f).
fn gains_and_loses_all_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let head = end(l).strip_suffix(" and loses all other abilities")?;
    if !head.contains(" gains ") {
        return None;
    }
    let mut e = crate::oracle::effects::parse_clause(head, b)?;
    let Effect::Modify { mods, .. } = &mut e else {
        return None;
    };
    if mods.iter().any(|m| {
        !matches!(
            m,
            Modification::AddKeyword(_) | Modification::AddAbility(_)
        )
    }) {
        return None;
    }
    mods.insert(0, Modification::RemoveAllAbilities);
    Some(e)
}

inventory::submit! { EffectPattern { name: "characteristic riders: [subject] gains [abilities] and loses all other abilities", priority: 1090, parse: gains_and_loses_all_other } }

/// "If X is 6 or more, those permanents are 4/4 creatures in addition to their other
/// types." (Dance of the Manse): a rider that applies only if its condition holds; the
/// permanents enter that way then (CR 611.3c).
fn conditional_rider(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((cond, rider)) = l.strip_prefix("if ").and_then(|r| r.split_once(", ")) else {
        return false;
    };
    let rider = rider
        .replacen("those permanents are ", "they're ", 1)
        .replacen("those creatures are ", "they're ", 1);
    {
        let Some(to) = last_battlefield_move(prev) else {
            return false;
        };
        if !to.with_mods.is_empty() {
            return false;
        }
    }
    let Some(c) = super::conditions_referents::parse_condition_with(cond, b)
        .or_else(|| crate::oracle::statics::parse_condition(cond, b.ctx))
    else {
        return false;
    };
    let Some(mods) = rider_mods(&rider, b) else {
        return false;
    };
    // The instruction with the rider when the condition holds, without it otherwise.
    let Some(i) = last_move_index(prev) else {
        return false;
    };
    let plain = take_at(prev, &i);
    let mut with = plain.clone();
    if let Some(to) = last_battlefield_move(&mut with) {
        to.with_mods = mods;
    }
    put_at(
        prev,
        &i,
        Effect::If {
            cond: c,
            then: Box::new(with),
            otherwise: Box::new(plain),
        },
    );
    true
}

/// The path to the last battlefield move of `e` (indices into nested sequences).
fn last_move_index(e: &Effect) -> Option<Vec<usize>> {
    match e {
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(vec![]),
        Effect::Seq(v) => {
            let n = v.len().checked_sub(1)?;
            let mut p = last_move_index(&v[n])?;
            p.insert(0, n);
            Some(p)
        }
        _ => None,
    }
}

fn take_at(e: &mut Effect, path: &[usize]) -> Effect {
    match path.split_first() {
        None => e.clone(),
        Some((i, rest)) => match e {
            Effect::Seq(v) => take_at(&mut v[*i], rest),
            _ => e.clone(),
        },
    }
}

fn put_at(e: &mut Effect, path: &[usize], new: Effect) {
    match path.split_first() {
        None => *e = new,
        Some((i, rest)) => {
            if let Effect::Seq(v) = e {
                put_at(&mut v[*i], rest, new);
            }
        }
    }
}

inventory::submit! { FollowupPattern { name: "characteristic riders: if [condition], those permanents are [characteristics]", priority: 1150, apply: conditional_rider } }
