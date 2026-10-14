//! Targets introduced inside a qualifier of another object phrase (CR 601.2c: targets are
//! chosen as the spell is cast, wherever the text names them):
//!
//! - inside a comparison: "Destroy all creatures with power greater than target
//!   creature's power" (Fell the Mighty; the target isn't one of the creatures compared
//!   with itself, and its power is used as the spell resolves);
//! - as the player of an "among" group: "Exile a creature with the greatest power among
//!   creatures target opponent controls" (Fall of the Impostor).
//!
//! The target is declared first, then the sentence is read again with "that creature" /
//! "that player" in its place.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// The comparison words a target can follow inside a qualifier.
const COMPARISONS: [&str; 7] = [
    "greater than or equal to ",
    "less than or equal to ",
    "equal to or greater than ",
    "equal to or less than ",
    "greater than ",
    "less than ",
    "equal to ",
];

fn target_in_comparison(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // Only one instance of "target" in the sentence, after the start.
    if l.matches("target ").count() != 1 || l.starts_with("target ") {
        return None;
    }
    let at = l.find("target ")?;
    let before = &l[..at];
    let after = &l[at..];
    if COMPARISONS.iter().any(|c| before.ends_with(c)) && before.contains(" with ") {
        // "target creature's power"
        let (phrase, rest) = after.split_once("'s ")?;
        let phrase_sp = format!("{phrase} ");
        let (spec, tail) = parse_target(&phrase_sp)?;
        if !tail.trim().is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
            return None;
        }
        let saved = (b.targets.len(), b.it.clone());
        let slot = b.add_target(spec, phrase);
        b.it = Sel::Target(slot);
        let text = format!("{before}that creature's {rest}");
        let e = parse_clause(&text, b);
        // The comparison must have used it.
        let ok = e.as_ref().is_some_and(|e| {
            serde_json::to_string(e).is_ok_and(|j| j.contains(&format!("{{\"Target\":{slot}}}")))
        });
        if !ok {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
        return e;
    }
    if before.ends_with(" among creatures ") || before.ends_with(" among permanents ") {
        // "among creatures target opponent controls"
        let (spec, rest) = parse_target(after)?;
        if !matches!(spec.what, TargetKind::Player(_)) || !rest.trim_start().starts_with("controls") {
            return None;
        }
        let consumed = &after[..after.len() - rest.len()];
        let saved = b.targets.len();
        let slot = b.add_target(spec, consumed.trim());
        let text = format!("{before}that player {}", rest.trim_start());
        let Some(e) = parse_clause(&text, b) else {
            b.targets.truncate(saved);
            return None;
        };
        // "that player controls" in the qualifier is the target player.
        let they = serde_json::to_string(&Filter::ControlledBy(
            super::filters_relational::THEY,
        ))
        .ok()?;
        let target = serde_json::to_string(&Filter::ControlledBy(PlayerRel::Target(slot))).ok()?;
        let j = serde_json::to_string(&e).ok()?;
        if !j.contains(&they) {
            b.targets.truncate(saved);
            return None;
        }
        return serde_json::from_str(&j.replace(&they, &target)).ok();
    }
    None
}

inventory::submit! { EffectPattern { name: "target inside a qualifier", priority: 6, parse: target_in_comparison } }

/// The object each "any number of them" choice is about.
const EACH_OF_THEM: Var = vars::USER + 9310;

/// "You may have any number of them phase out." (Change of Plans): for each of the
/// objects "them" names, you may have it do so (CR 608.2d: the choice is made as the
/// spell resolves).
fn any_number_of_them(l: &str, b: &mut Builder) -> Option<Effect> {
    // ("You may" is read first: this is the rest, "have any number of them ...".)
    let r = end(l)
        .strip_prefix("you may have any number of ")
        .or_else(|| end(l).strip_prefix("have any number of "))?;
    // "them": the targets of a slot of several ("X target creatures").
    let several = b.targets.iter().rposition(|t| !matches!(t.max, Value::Const(1)));
    let (sel, rest) = match (r.strip_prefix("them "), several) {
        (Some(rest), Some(i)) if b.targets.len() == i + 1 => (Sel::Target(i as u8), rest.to_string()),
        _ => match super::pronoun_groups::plural_object_ref(r, b) {
            Some(Some(x)) => x,
            _ => return None,
        },
    };
    let rest = rest.trim_start();
    if rest.is_empty() {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::Var(EACH_OF_THEM);
    let e = parse_clause(&format!("you may have it {rest}"), b);
    b.it = saved.1;
    let e = e.filter(|_| b.targets.len() == saved.0)?;
    Some(Effect::ForEach {
        sel,
        var: EACH_OF_THEM,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "you may have any number of them [verb]", priority: 100, parse: any_number_of_them } }

/// "for each isolation counter on it, exile up to one target nonland permanent an
/// opponent controls until ~ leaves the battlefield" (Quarantine Field): up to one target
/// for each such counter, all chosen as the ability is put on the stack (CR 601.2c), and
/// the instruction acts on all of them.
fn up_to_one_target_for_each_counter(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (counter, rest) = r.split_once(" on it, ")?;
    let kind = counter.strip_suffix(" counter")?;
    if kind.contains(' ') || !rest.contains("up to one target ") || rest.matches("target ").count() != 1 {
        return None;
    }
    let it = b.it.clone();
    if !matches!(it, Sel::This) {
        return None;
    }
    let before = b.targets.len();
    let e = parse_clause(rest, b)?;
    if b.targets.len() != before + 1 || !matches!(b.targets[before].max, Value::Const(1)) {
        b.targets.truncate(before);
        return None;
    }
    b.targets[before].max = Value::CountersOn(Box::new(it), Some(kind.into()));
    Some(e)
}

inventory::submit! { EffectPattern { name: "for each [counter] on it, ... up to one target ...", priority: 60, parse: up_to_one_target_for_each_counter } }

/// `StaticEffect::Custom` prefix: "You can't choose [an object] as ~'s target as you cast
/// it" (Enthralling Hold, Dream Leash). The rest is the object filter, as JSON. Only the
/// choice of the Aura spell's target as it's cast is restricted (CR 601.2c); the target
/// isn't rechecked against it as the spell resolves (CR 608.2b), and an Aura put onto the
/// battlefield without being cast isn't affected (CR 303.4f).
pub const CAST_TARGET_FORBIDDEN: &str = "cast target forbidden:";

fn cant_choose_as_target_as_cast(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you can't choose ")?;
    let r = r.strip_suffix(" as ~'s target as you cast it")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural || !rest.trim().is_empty() {
        return None;
    }
    let json = serde_json::to_string(&f).ok()?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            format!("{CAST_TARGET_FORBIDDEN}{json}").into(),
        ))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "you can't choose [object] as ~'s target as you cast it", priority: 100, parse: cant_choose_as_target_as_cast } }

/// The objects a spell's text forbids choosing as its target as it's cast (see
/// [`CAST_TARGET_FORBIDDEN`]).
pub fn cast_target_forbidden(chars: &crate::object::Characteristics) -> Option<Filter> {
    let fs: Vec<Filter> = chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Custom(n) => n
                    .strip_prefix(CAST_TARGET_FORBIDDEN)
                    .and_then(|j| serde_json::from_str(j).ok()),
                _ => None,
            },
            _ => None,
        })
        .collect();
    match fs.len() {
        0 => None,
        1 => fs.into_iter().next(),
        _ => Some(Filter::Or(fs)),
    }
}

/// "Then that creature gets +1/+1 until end of turn and fights target creature you don't
/// control." (Ent's Fury): the subject of the first instruction also fights (CR 701.14).
fn gets_and_fights(l: &str, b: &mut Builder) -> Option<Effect> {
    let (a, c) = end(l).split_once(" and fights ")?;
    if !a.contains(" gets ") && !a.contains(" get ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let e1 = parse_clause(a, b)?;
    // The first instruction's subject is what "it" now refers to.
    if !matches!(b.it, Sel::Target(_)) {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    let Some(e2) = parse_clause(&format!("it fights {c}"), b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    Some(Effect::Seq(vec![e1, e2]))
}

inventory::submit! { EffectPattern { name: "[subject] gets ... and fights [object]", priority: 100, parse: gets_and_fights } }
