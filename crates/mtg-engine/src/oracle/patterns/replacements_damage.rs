//! Replacement and prevention effects keyed on a damage event, by source and recipient
//! (CR 614.1a, 615.1a, 615.10, 701.10g):
//!
//! Static abilities:
//! - "If a [red] source [an opponent controls] would deal damage to you, prevent 1 of
//!   that damage." (Circle spheres, Heart-Shaped Herb, Urza's Armor) — a static
//!   prevention effect that prevents that much of each applicable damage event
//!   (CR 615.10).
//! - "If a source would deal damage to [a planeswalker you control | equipped creature |
//!   you or a permanent you control], prevent N of that damage.", "If a spell would deal
//!   damage to you or another permanent you control, prevent that damage."
//! - "If [a source you control | a creature you control | enchanted creature | ~] would
//!   deal [combat | noncombat] damage [to a permanent or player | to an opponent | to
//!   you], it deals double that damage [to that permanent or player] instead."
//!   (CR 701.10g)
//! - "If another red source you control would deal damage to a permanent or player, it
//!   deals that much damage plus 1 to that permanent or player instead."
//!
//! One-shot effects (until end of turn): the same with "this turn", e.g. "If a source you
//! control would deal damage this turn, it deals double that damage instead." (Insult),
//! "If a source you control would deal damage to an opponent this turn, it deals double
//! that damage to that player instead." (Goblin Goliath). A one-shot "prevent N of that
//! damage" would be a shield (CR 615.7) rather than a per-event effect, so it isn't
//! accepted here.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Strips `p` from the start of `s` if a word boundary follows.
fn word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let r = s.strip_prefix(p)?;
    match r.chars().next() {
        None | Some(' ' | ',') => Some(r.trim_start()),
        _ => None,
    }
}

/// "you control", "an opponent controls", ... after a source noun.
fn controller_suffix(s: &str) -> Option<Option<PlayerRel>> {
    Some(match s.trim() {
        "" => None,
        "you control" => Some(PlayerRel::You),
        "an opponent controls" | "your opponents control" | "controlled by an opponent" => {
            Some(PlayerRel::Opponent)
        }
        _ => return None,
    })
}

/// One adjective or noun qualifying "source": colors ("red"), card types ("artifact"),
/// subtypes ("Giant").
fn source_quality(w: &str) -> Option<Filter> {
    if let Some(f) = adjective(w) {
        return match f {
            Filter::Color(_) | Filter::Colorless | Filter::Multicolored | Filter::Monocolored => {
                Some(f)
            }
            Filter::Not(ref x) if matches!(**x, Filter::Color(_)) => Some(f),
            _ => None,
        };
    }
    match head_noun(w)? {
        f @ (Filter::Type(_) | Filter::Subtype(_)) => Some(f),
        _ => None,
    }
}

/// The damage source of "if [source] would deal damage": "~", "enchanted creature", "a
/// [red|black or red|Giant] source [you control]", "another red source you control",
/// "a creature [you control]", "an artifact", "a red instant or sorcery spell you
/// control". The whole phrase must be understood.
fn source_phrase(s: &str) -> Option<Filter> {
    let s = s.trim();
    match s {
        "~" | "this creature" => return Some(Filter::Source),
        "enchanted creature" | "equipped creature" => return Some(Filter::AttachedToSource),
        _ => {}
    }
    let (other, r) = if let Some(r) = word(s, "another") {
        (true, r)
    } else {
        let r = word(s, "a")
            .or_else(|| word(s, "an"))
            .or_else(|| word(s, "any"))?;
        (false, r)
    };
    let mut parts = Vec::new();
    if other {
        parts.push(Filter::Other);
    }
    if let Some(idx) = r.find("source") {
        // "[qualities] source [controller]"
        let before = r[..idx].trim();
        let after = &r[idx + "source".len()..];
        if !(after.is_empty() || after.starts_with(' ')) {
            return None;
        }
        if !before.is_empty() {
            let mut alts = Vec::new();
            for alt in before.split(" or ") {
                let mut all = Vec::new();
                for w in alt.split(' ') {
                    all.push(source_quality(w.trim())?);
                }
                alts.push(Filter::and(all));
            }
            parts.push(if alts.len() == 1 {
                alts.pop()?
            } else {
                Filter::Or(alts)
            });
        }
        if let Some(rel) = controller_suffix(after)? {
            parts.push(Filter::ControlledBy(rel));
        }
        return Some(Filter::and(parts));
    }
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    parts.push(f);
    Some(Filter::and(parts))
}

/// Who or what the damage would be dealt to: (players, objects).
type Recipient = (Option<PlayerFilter>, Option<Filter>);

fn or_players(a: Option<PlayerFilter>, b: PlayerFilter) -> Option<PlayerFilter> {
    Some(match a {
        None => b,
        Some(PlayerFilter::Or(mut v)) => {
            v.push(b);
            PlayerFilter::Or(v)
        }
        Some(x) => PlayerFilter::Or(vec![x, b]),
    })
}

fn or_objects(a: Option<Filter>, b: Filter) -> Option<Filter> {
    Some(match a {
        None => b,
        Some(Filter::Or(mut v)) => {
            v.push(b);
            Filter::Or(v)
        }
        Some(x) => Filter::Or(vec![x, b]),
    })
}

/// "you", "an opponent", "a player", "enchanted player", "~", "equipped creature", "a
/// planeswalker you control", "another permanent you control", "a permanent or player",
/// and "or"/"and/or" lists of those: "you or a permanent you control", "an opponent or a
/// permanent an opponent controls".
fn recipient(s: &str) -> Option<Recipient> {
    let s = s.trim();
    if s == "a permanent or player" || s == "a player or permanent" {
        return Some((Some(PlayerFilter::Any), Some(Filter::Any)));
    }
    let mut out: Recipient = (None, None);
    let mut rest = s;
    loop {
        // The next item ends at " or " / " and/or " followed by a new noun phrase.
        let (item, next) = split_item(rest);
        match item {
            "you" => out.0 = or_players(out.0.take(), PlayerFilter::You),
            "an opponent" => out.0 = or_players(out.0.take(), PlayerFilter::Opponent),
            "a player" => out.0 = or_players(out.0.take(), PlayerFilter::Any),
            "enchanted player" => {
                out.0 = or_players(
                    out.0.take(),
                    PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(
                        Sel::AttachedTo,
                    )))),
                )
            }
            "~" | "this creature" => out.1 = or_objects(out.1.take(), Filter::Source),
            "equipped creature" | "enchanted creature" => {
                out.1 = or_objects(out.1.take(), Filter::AttachedToSource)
            }
            _ => {
                let f = if let Some(r) = word(item, "another") {
                    let (f, plural, tail) = parse_object_phrase(r)?;
                    if plural || !tail.trim().is_empty() {
                        return None;
                    }
                    Filter::and(vec![f, Filter::Other])
                } else {
                    let r = word(item, "a")
                        .or_else(|| word(item, "an"))
                        .or_else(|| word(item, "one or more"));
                    let r = r?;
                    let (f, plural, tail) = parse_object_phrase(r)?;
                    if !tail.trim().is_empty() || (plural != item.starts_with("one or more")) {
                        return None;
                    }
                    f
                };
                out.1 = or_objects(out.1.take(), f);
            }
        }
        match next {
            Some(n) => rest = n,
            None => break,
        }
    }
    Some(out)
}

/// Splits "X or a Y" / "X and/or Y" into ("X", Some("a Y")). Only splits where a new
/// recipient starts (so "a white or blue creature" stays whole).
fn split_item(s: &str) -> (&str, Option<&str>) {
    for sep in [" and/or ", " or "] {
        let mut from = 0;
        while let Some(i) = s[from..].find(sep) {
            let at = from + i;
            let next = &s[at + sep.len()..];
            let starts_new = ["a ", "an ", "another ", "one or more ", "you", "~"]
                .iter()
                .any(|p| next.starts_with(p))
                || next.starts_with("enchanted ")
                || next.starts_with("equipped ");
            if starts_new {
                return (&s[..at], Some(next));
            }
            from = at + sep.len();
        }
    }
    (s, None)
}

/// "it deals double that damage [to that permanent or player] instead" etc.: the
/// recipient restated at the end of a doubling or "plus N" replacement.
fn restated_recipient(s: &str) -> bool {
    matches!(
        s.trim(),
        "" | "to that permanent or player"
            | "to that player or permanent"
            | "to that player"
            | "to that permanent"
            | "to that creature"
            | "to you"
            | "to ~"
            | "to equipped creature"
            | "to enchanted creature"
    )
}

/// The replacement: "prevent that damage", "prevent N of that damage" (static only),
/// "it deals double that damage [to ...] instead", "it deals that much damage plus N [to
/// ...] instead".
fn damage_action(s: &str, is_static: bool) -> Option<ReplacementAction> {
    let s = end(s.trim());
    if s == "prevent that damage" {
        return Some(ReplacementAction::Prevent);
    }
    if let Some(r) = s.strip_prefix("prevent ") {
        let (n, r) = parse_number(r)?;
        if !matches!(n, Value::Const(_)) || r.trim() != "of that damage" || !is_static {
            return None;
        }
        return Some(ReplacementAction::PreventAmount(n));
    }
    let r = s
        .strip_prefix("it deals ")
        .or_else(|| s.strip_prefix("that source deals "))?;
    let r = r.strip_suffix(" instead")?;
    if let Some(x) = r
        .strip_prefix("double that damage")
        .map(|x| (2, x))
        .or_else(|| r.strip_prefix("triple that damage").map(|x| (3, x)))
    {
        let (k, tail) = x;
        return restated_recipient(tail).then_some(ReplacementAction::Multiply(k));
    }
    let r = r.strip_prefix("that much damage plus ")?;
    let (n, tail) = parse_number(r)?;
    if !matches!(n, Value::Const(_)) || !restated_recipient(tail) {
        return None;
    }
    Some(ReplacementAction::Add(n))
}

/// Replaces "another" (not the source) with "not the object the source is attached to".
fn other_than_attached(f: Filter) -> Filter {
    match f {
        Filter::Other => Filter::not(Filter::AttachedToSource),
        Filter::And(v) => Filter::And(v.into_iter().map(other_than_attached).collect()),
        f => f,
    }
}

/// "if [source] would deal [combat|noncombat] damage [to recipient][ this turn], [action]"
/// → (definition, whether "this turn" was said).
fn damage_replacement(l: &str, is_static: bool) -> Option<(ReplacementDef, bool)> {
    let r = end(l).strip_prefix("if ")?;
    let (src, r) = r.split_once(" would deal ")?;
    let source = source_phrase(src)?;
    let (cond, act) = r.split_once(", ")?;
    let (kind, cond) = if let Some(x) = cond.strip_prefix("combat damage") {
        (Some(true), x)
    } else if let Some(x) = cond.strip_prefix("noncombat damage") {
        (Some(false), x)
    } else {
        (None, cond.strip_prefix("damage")?)
    };
    let (this_turn, cond) = match cond.strip_suffix(" this turn") {
        Some(x) => (true, x),
        None => (false, cond),
    };
    let cond = cond.trim();
    let to = if cond.is_empty() {
        (Some(PlayerFilter::Any), Some(Filter::Any))
    } else {
        recipient(cond.strip_prefix("to ")?)?
    };
    // "If another creature would deal combat damage to equipped creature": "another"
    // contrasts with the equipped creature, not with the Equipment.
    let source = if matches!(to, (None, Some(Filter::AttachedToSource))) {
        other_than_attached(source)
    } else {
        source
    };
    let action = damage_action(act, is_static)?;
    let event = match kind {
        Some(false) => ReplacementEvent::NoncombatDamage {
            source,
            to_players: to.0,
            to_objects: to.1,
        },
        combat => ReplacementEvent::Damage {
            source,
            to_players: to.0,
            to_objects: to.1,
            combat_only: combat == Some(true),
        },
    };
    Some((
        ReplacementDef {
            event,
            action,
            self_replacement: false,
            optional: false,
        },
        this_turn,
    ))
}

fn s_damage_replacement(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (def, this_turn) = damage_replacement(l, true)?;
    if this_turn {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements: if a source would deal damage, prevent/double", priority: 70, parse: s_damage_replacement } }

fn p_damage_replacement(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (def, this_turn) = damage_replacement(l, false)?;
    if !this_turn {
        return None;
    }
    Some(Effect::AddReplacement {
        def,
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "replacements: if a source would deal damage this turn", priority: 70, parse: p_damage_replacement } }
