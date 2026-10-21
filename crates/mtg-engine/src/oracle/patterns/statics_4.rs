//! Characteristic statics IV (cards3-combat-statics):
//!
//! * "Until your next turn, you and planeswalkers you control gain protection from that
//!   player." (Eon Frolicker): protection from a player (CR 702.16k) for the player and
//!   for permanents, the player bound as the effect begins.
//! * "Target creature gains landwalk of each of the land types of the sacrificed land
//!   until end of turn." (Excavator): one landwalk ability per land type (CR 702.14a),
//!   determined from the sacrificed land as it last existed (CR 608.2h).
//! * "Target creature gains snow landwalk of the chosen type until end of turn." (Barbarian
//!   Guides): snow [type]walk (CR 702.14c).
//! * "Each creature target opponent controls phases out. Until the end of your next turn,
//!   they can't phase in." (Teferi, Timeless Voyager): "they" are the permanents phased
//!   out this way.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

/// `Filter::Custom` marker in a landwalk keyword: "landwalk of each of the land types of
/// the sacrificed land", expanded into one landwalk per land type as the effect is created
/// (`Game::fix_mods`).
pub const LAND_TYPES_OF_SACRIFICED: &str = "statics4:land types of the sacrificed land";

fn keyword(kind: KeywordKind, filter: Option<Filter>, text: &str) -> Keyword {
    Keyword {
        kind,
        n: None,
        cost: None,
        costs: vec![],
        filter,
        text: Some(text.into()),
        x: None,
    }
}

/// A leading or trailing duration of a "gain" instruction.
fn split_duration(l: &str) -> (Duration, String) {
    for (p, d) in [
        ("until your next turn, ", Duration::UntilYourNextTurn),
        ("until end of turn, ", Duration::EndOfTurn),
        ("until the end of your next turn, ", Duration::UntilEndOfYourNextTurn),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            return (d, r.to_string());
        }
    }
    for (s, d) in [
        (" until your next turn", Duration::UntilYourNextTurn),
        (" until end of turn", Duration::EndOfTurn),
        (" until the end of your next turn", Duration::UntilEndOfYourNextTurn),
    ] {
        if let Some(r) = l.strip_suffix(s) {
            return (d, r.to_string());
        }
    }
    (Duration::Permanent, l.to_string())
}

/// "[you and] [permanents] gain protection from [player]".
fn p_protection_from_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = split_duration(end(l.trim()));
    if matches!(duration, Duration::Permanent) {
        return None;
    }
    let (subject, who) = l.split_once(" gain protection from ")?;
    let n = b.targets.len();
    let (p, rest) = player_ref(who, b)?;
    if !rest.trim().is_empty()
        || b.targets.len() != n
        || crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent(&p)
        || matches!(p, PlayerRef::You)
    {
        b.targets.truncate(n);
        return None;
    }
    let quality = Filter::ControlledByPlayer(Box::new(p));
    let (you, objects) = if subject == "you" {
        (true, None)
    } else if let Some(o) = subject.strip_prefix("you and ") {
        (true, Some(o))
    } else {
        (false, Some(subject))
    };
    let mut out = Vec::new();
    if you {
        out.push(Effect::AddPlayerEffect {
            who: PlayerRef::You,
            effect: PlayerModification::ProtectionFrom(quality.clone()),
            duration: duration.clone(),
        });
    }
    if let Some(o) = objects {
        let (f, plural, tail) = parse_object_phrase(o)?;
        if !plural || !tail.trim().is_empty() {
            return None;
        }
        out.push(Effect::Modify {
            what: Sel::All(f),
            mods: vec![Modification::AddKeyword(keyword(
                KeywordKind::Protection,
                Some(quality),
                "protection from that player",
            ))],
            duration,
        });
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "statics 4: you and [permanents] gain protection from that player", priority: 90, parse: p_protection_from_player } }

/// "[target] gains landwalk of each of the land types of the sacrificed land [duration]",
/// "[target] gains snow landwalk of the chosen type [duration]".
fn p_landwalk_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = split_duration(end(l.trim()));
    let (subject, kw) = l
        .split_once(" gains ")
        .or_else(|| l.split_once(" gain "))?;
    let k = match kw {
        "landwalk of each of the land types of the sacrificed land" => keyword(
            KeywordKind::Landwalk,
            Some(Filter::Custom(LAND_TYPES_OF_SACRIFICED.into())),
            kw,
        ),
        "snow landwalk of the chosen type" => keyword(
            KeywordKind::Landwalk,
            Some(Filter::and(vec![
                Filter::Supertype(crate::types::Supertype::Snow),
                Filter::ChosenType,
            ])),
            kw,
        ),
        _ => return None,
    };
    let n = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subject, b)?;
    if !rest.trim().is_empty() || matches!(what, Sel::None) {
        b.targets.truncate(n);
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddKeyword(k)],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "statics 4: gains landwalk of the sacrificed land's types / snow landwalk", priority: 90, parse: p_landwalk_of } }

/// The last instruction of `e`, if it phases permanents out.
fn phases_out(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(phases_out),
        Effect::PhaseOut { what } => matches!(what, Sel::All(_)),
        _ => false,
    }
}

/// "Until the end of your next turn, they can't phase in." after "Each creature target
/// opponent controls phases out.": the permanents phased out this way.
fn f_they_cant_phase_in(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    if !phases_out(prev) {
        return false;
    }
    let rewritten = ["they can't phase in", "those creatures can't phase in", "those permanents can't phase in"]
        .iter()
        .find(|p| l.contains(*p))
        .map(|p| l.replacen(p, "it can't phase in", 1));
    let Some(rewritten) = rewritten else {
        return false;
    };
    let saved = b.it.clone();
    b.it = Sel::Var(vars::PHASED_OUT);
    let n = b.targets.len();
    let e = crate::oracle::effects::parse_sentence(&rewritten, b);
    b.it = saved;
    let Some(e) = e.filter(|_| b.targets.len() == n) else {
        b.targets.truncate(n);
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "statics 4: they can't phase in (the permanents phased out)", priority: 60, apply: f_they_cant_phase_in } }
