//! Color text (CR 105): characteristic-defining color abilities ("~ is all colors",
//! "~ is colorless"), color-setting statics ("enchanted creature is red"), and
//! color-changing effects ("target creature becomes blue until end of turn", "target
//! spell or permanent becomes colorless", "... in addition to its other colors").

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// Parses a color list: "red", "red and green", "white, blue, and black", "colorless",
/// "all colors", "every color". Returns the set and the unparsed rest.
pub fn parse_color_list(s: &str) -> Option<(ColorSet, &str)> {
    let s = s.trim_start();
    for (p, cs) in [
        ("all colors", ColorSet::ALL),
        ("every color", ColorSet::ALL),
        ("colorless", ColorSet::NONE),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((cs, r));
        }
    }
    let mut set = ColorSet::NONE;
    let mut rest = s;
    loop {
        let (w, r) = split_word(rest);
        let w = w.trim_end_matches(',');
        let Some(c) = Color::from_word(w) else {
            break;
        };
        set.insert(c);
        rest = r;
        // Separators: ", ", "and ", ", and ".
        let t = rest.trim_start();
        if let Some(r2) = t.strip_prefix("and ") {
            rest = r2;
            continue;
        }
        let (next, _) = split_word(t);
        if Color::from_word(next.trim_end_matches(',')).is_some() && w != next {
            // "white, blue" — the comma was on the previous word.
            continue;
        }
        break;
    }
    if set.is_colorless() {
        return None;
    }
    Some((set, rest))
}

fn cda_colors(cs: ColorSet, text: &str) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::SetColors(cs)],
    });
    // CR 604.3 / 113.6a: characteristic-defining abilities function everywhere.
    s.zone = FunctionZone::Anywhere;
    s.is_cda = true;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// "~ is all colors." / "~ is colorless." / "~ is red and green." (CR 105.2, 604.3).
fn color_cda(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let l = lower.strip_prefix("~ is ")?;
    let l = l
        .strip_suffix(". this ability doesn't affect its color identity.")
        .or_else(|| l.strip_suffix("."))
        .unwrap_or(l);
    let (cs, rest) = parse_color_list(l)?;
    if !end(rest).is_empty() {
        return None;
    }
    Some(vec![cda_colors(cs, block)])
}

inventory::submit! { AbilityPattern { name: "r105 color cda", priority: 50, parse: color_cda } }

/// "enchanted creature is red", "equipped creature is black" (CR 105.3, 113.12).
fn attached_color_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l
        .strip_prefix("enchanted creature is ")
        .or_else(|| l.strip_prefix("enchanted permanent is "))
        .or_else(|| l.strip_prefix("equipped creature is "))?;
    let (cs, rest) = if let Some(r2) = r.strip_prefix("colorless") {
        (ColorSet::NONE, r2)
    } else {
        parse_color_list(r)?
    };
    let rest = end(rest);
    let m = if rest.is_empty() {
        Modification::SetColors(cs)
    } else if rest == "in addition to its other colors" {
        Modification::AddColors(cs)
    } else {
        return None;
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::AttachedToSource,
            mods: vec![m],
        })),
        text,
    )])
}

/// The core static parser stops at any "enchanted creature ..." line it can't parse, so
/// this is registered as a whole-ability pattern.
fn attached_color_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    attached_color_static(end(&lower), block, ctx)
}

inventory::submit! { AbilityPattern { name: "r105 attached color", priority: 50, parse: attached_color_block } }

/// Duration suffixes for color-changing effects.
fn color_duration(s: &str) -> (Duration, &str) {
    let t = s.trim();
    for (p, d) in [
        (" until end of turn", Duration::EndOfTurn),
        (" this turn", Duration::EndOfTurn),
        (" until your next turn", Duration::UntilYourNextTurn),
    ] {
        if let Some(r) = t.strip_suffix(p) {
            return (d, r);
        }
    }
    (Duration::Permanent, t)
}

/// The subject of a color-changing effect: a target phrase, "~", "it", or
/// "enchanted creature".
fn color_subject<'a>(s: &'a str, b: &mut Builder) -> Option<(Sel, &'a str)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("~ ") {
        return Some((Sel::This, r));
    }
    if let Some(r) = s.strip_prefix("it ") {
        return Some((b.it.clone(), r));
    }
    if let Some(r) = s.strip_prefix("enchanted creature ") {
        return Some((Sel::AttachedTo, r));
    }
    let (spec, rest) = parse_target(s)?;
    let text = s[..s.len() - rest.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some((Sel::Target(slot), rest))
}

/// "target creature becomes blue until end of turn", "target spell or permanent becomes
/// colorless", "target creature becomes black in addition to its other colors",
/// "target creature becomes the color or colors of your choice until end of turn"
/// (CR 105.3).
fn becomes_color(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = color_duration(l);
    let (what, rest) = color_subject(l, b)?;
    let r = rest.trim_start().strip_prefix("becomes ")?;
    let (mods, tail) = if let Some(t) = r.strip_prefix("colorless") {
        (vec![Modification::SetColors(ColorSet::NONE)], t)
    } else {
        let (cs, t) = parse_color_list(r)?;
        let t = t.trim_start();
        if let Some(t2) = t.strip_prefix("in addition to its other colors") {
            (vec![Modification::AddColors(cs)], t2)
        } else {
            (vec![Modification::SetColors(cs)], t)
        }
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "r105 becomes color", priority: 50, parse: becomes_color } }
