//! Copying a spell (CR 707.10), the general grammar:
//!
//! ```text
//! copy SPELL [COUNT] [, except EXCEPTIONS] [and [you] may choose a new target for (the|that) copy]
//!      [if CONDITION]
//! SPELL := it | that spell | ~ | this spell     (the spell the ability is about)
//! COUNT := twice | N times | X times | for each COUNTED
//! ```
//!
//! "It"/"that spell" is the spell a cast trigger names ("Whenever you cast a historic
//! spell, copy it"), or a target spell an earlier instruction named. "~"/"this spell"/"it"
//! is the spell itself: in its own text as it resolves ("If this spell was cast from a
//! graveyard, you may copy this spell", Sevinne's Reclamation), or in its "When you cast
//! this spell" trigger, which functions on the stack (CR 113.6c, 603.6). A copy of a
//! permanent spell becomes a token (CR 608.3f).
//!
//! The exceptions ("except the copy isn't legendary", "except the copy is an artifact in
//! addition to its other types") become part of the copy's copiable values (CR 707.9b,
//! `Effect::CopySpellExcept`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether the text is about the spell itself while it's on the stack: the spell's own
/// effect, or its "When you cast this spell" trigger.
fn about_spell_itself(b: &Builder) -> bool {
    if !b.in_trigger {
        return b.ctx.is_spell();
    }
    if !matches!(b.it, Sel::This) {
        return false;
    }
    // An instant's or sorcery's only triggered abilities that do anything function on
    // the stack; a permanent's "When you cast this spell" does too (CR 113.6c).
    if b.ctx.is_spell() {
        return true;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let name = b.ctx.card_name.to_lowercase();
    raw.contains("when you cast this spell, ") || raw.contains(&format!("when you cast {name}, "))
}

/// The spell a word names ("it", "that spell", "~", "this spell").
fn spell_ref(w: &str, b: &Builder) -> Option<Sel> {
    match w {
        "~" | "this spell" => about_spell_itself(b).then_some(Sel::This),
        // "Counter target instant or sorcery spell ... you may copy the spell countered
        // this way" (Psychic Rebuttal): copied as it last existed on the stack (CR 707.10,
        // 608.2h).
        "it" | "that spell" | "the spell countered this way" => {
            if matches!(b.it, Sel::TriggerSpell) {
                return Some(Sel::TriggerSpell);
            }
            if let Sel::Target(n) = b.it {
                if matches!(
                    b.targets.get(n as usize).map(|t| &t.what),
                    Some(TargetKind::Spell(_))
                ) {
                    return Some(Sel::Target(n));
                }
                return None;
            }
            (w == "it" && b.in_trigger && about_spell_itself(b)).then_some(Sel::This)
        }
        _ => None,
    }
}

/// "twice", "three times", "X times", "for each ...": how many copies.
fn copy_count(s: &str, b: &Builder) -> Option<Value> {
    let s = s.trim();
    if s.is_empty() {
        return Some(Value::c(1));
    }
    if s == "twice" {
        return Some(Value::c(2));
    }
    if let Some(r) = s.strip_suffix(" times") {
        let (n, tail) = parse_number(r)?;
        if !tail.trim().is_empty() {
            return None;
        }
        // X is the spell's: only the spell itself has it (its cast trigger uses the
        // spell's value of X, CR 107.3m).
        if matches!(n, Value::X) && !about_spell_itself(b) {
            return None;
        }
        return Some(n);
    }
    let r = s.strip_prefix("for each ")?;
    super::statics::parse_for_each(r, None)
}

/// The exceptions of a spell copy ("the copy isn't legendary", "the copy is a 1/1 Spirit
/// in addition to its other types", "it isn't legendary if the spell is legendary").
fn spell_copy_exceptions(s: &str, b: &Builder) -> Option<Vec<Modification>> {
    let s = s.trim();
    // Removing a supertype the spell doesn't have does nothing, so "if the spell is
    // legendary" (Double Major) changes nothing.
    let s = s.strip_suffix(" if the spell is legendary").unwrap_or(s);
    let s = if let Some(r) = s.strip_prefix("the copy is ") {
        format!("it's {r}")
    } else if let Some(r) = s.strip_prefix("the copy isn't ") {
        format!("it isn't {r}")
    } else if let Some(r) = s.strip_prefix("the copy has ") {
        format!("it has {r}")
    } else {
        s.to_string()
    };
    let (masked, quotes) = super::statics::mask_quotes(&s)?;
    super::tokens_copies_copy::copy_exceptions(&masked, &quotes, b.ctx)
}

fn copy_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let n = b.targets.len();
    let e = copy_spell_inner(l, b);
    if e.is_none() {
        b.targets.truncate(n);
    }
    e
}

fn copy_spell_inner(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("copy ")?;
    let (w, rest) = if let Some(x) = r.strip_prefix("this spell") {
        ("this spell", x)
    } else if let Some(x) = r.strip_prefix("that spell") {
        ("that spell", x)
    } else if let Some(x) = r.strip_prefix("the spell countered this way") {
        ("the spell countered this way", x)
    } else {
        let (w, x) = split_word(r);
        let w = w.trim_end_matches(',');
        let x = if r[w.len()..].starts_with(',') {
            &r[w.len()..]
        } else {
            x
        };
        (w, x)
    };
    let (what, rest) = if r.starts_with("target ") {
        // "copy target permanent spell you control three times" (Myojin of Cryptic
        // Dreams), "copy target creature spell you control, except ..." (Double Major).
        // "target instant or sorcery spell that targets you" (Mirror Sheen).
        let (spec, tail) = super::basic_effects_counter::stack_target(r).or_else(|| parse_target(r))?;
        if !matches!(spec.what, TargetKind::Spell(_)) {
            return None;
        }
        let text = r[..r.len() - tail.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        (Sel::Target(slot), tail)
    } else {
        (spell_ref(w, b)?, rest)
    };
    let mut rest = rest.trim().to_string();
    // "... and may choose a new target for the copy" (Chain of Acid, Sevinne's
    // Reclamation): one target, if the spell has one (CR 707.10c).
    let mut new_targets = false;
    for p in [
        " and may choose a new target for that copy",
        " and may choose a new target for the copy",
        " and you may choose a new target for the copy",
        " and you may choose new targets for the copy",
    ] {
        let padded = format!(" {rest}");
        if let Some(x) = padded.strip_suffix(p) {
            rest = x.trim().to_string();
            new_targets = true;
            break;
        }
    }
    // "copy it if you gained life this turn" (checked as the ability resolves).
    let mut cond = None;
    let padded = format!(" {rest}");
    if let Some(i) = padded.find(" if ") {
        if !padded[..i].contains(", except") {
            let c = crate::oracle::statics::parse_condition(&padded[i + 4..], b.ctx)?;
            cond = Some(c);
            rest = padded[..i].trim().to_string();
        }
    }
    let (count_s, except) = match rest.split_once(", except ") {
        Some((c, e)) => (c.to_string(), Some(e.to_string())),
        None => match rest.strip_prefix("except ") {
            Some(e) => (String::new(), Some(e.to_string())),
            None => (rest.clone(), None),
        },
    };
    let count = copy_count(count_s.trim_start_matches(',').trim(), b)?;
    // The plain forms are the older patterns' (`copy_spells`, `r707_*`) unless this adds
    // something.
    let mut e = match except {
        Some(x) => Effect::CopySpellExcept {
            what,
            count,
            new_targets,
            mods: spell_copy_exceptions(&x, b)?,
        },
        None => Effect::CopySpell {
            what,
            count,
            new_targets,
        },
    };
    if let Some(cond) = cond {
        e = Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        };
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "copy spell grammar: copy it [count] [except] [if]", priority: 101, parse: copy_spell } }

/// "copy each exiled card you own with a kick counter on it" (Zethi, Arcane
/// Blademaster): a copy of each of those cards, in the zone it's in (CR 707.12).
fn copy_each_card(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy each ")?;
    // "exiled card you own ...": a card in exile.
    let (r, exiled) = match r.strip_prefix("exiled ") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (f, _, tail) = parse_object_phrase(r)?;
    let f = if exiled {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Exile)])
    } else {
        f
    };
    if !end(tail).is_empty() || f.zone().is_none_or(|z| z == ZoneKind::Battlefield) {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::All(f),
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "copy spell grammar: copy each [card]", priority: 101, parse: copy_each_card } }
