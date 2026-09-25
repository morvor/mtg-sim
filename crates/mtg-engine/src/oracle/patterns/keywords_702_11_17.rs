//! Oracle patterns around the keywords of CR 702.11–702.17 (hexproof, indestructible,
//! intimidate, landwalk, lifelink, protection, reach) when they're granted to or taken
//! from players and objects by other abilities:
//!
//! * "You have protection from [quality]" / "you gain protection from everything until
//!   your next turn", "you gain hexproof until end of turn" (CR 702.11c, 702.16b–e, j–k);
//! * "[objects] lose hexproof and indestructible until end of turn" (CR 702.11e);
//! * "Instant and sorcery spells you control have lifelink" (CR 702.15d).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::keywords::{parse_keyword_line, protection_qualities, split_keyword_phrases};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

/// A trailing duration: "until end of turn", "until your next turn".
fn duration_suffix(s: &str) -> (Duration, &str) {
    for (suffix, d) in [
        (" until end of turn", Duration::EndOfTurn),
        (" this turn", Duration::EndOfTurn),
        (" until your next turn", Duration::UntilYourNextTurn),
        (" until the end of your next turn", Duration::UntilEndOfYourNextTurn),
    ] {
        if let Some(r) = s.strip_suffix(suffix) {
            return (d, r);
        }
    }
    (Duration::Permanent, s)
}

/// The keywords of a list ("hexproof and indestructible"), as keyword instances.
fn keyword_list(s: &str) -> Option<Vec<crate::keywords::Keyword>> {
    let tl = crate::types::TypeLine::default();
    let ctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let mut out = Vec::new();
    for p in split_keyword_phrases(s) {
        for a in parse_keyword_line(&p, &ctx)? {
            match &a.kind {
                AbilityKind::Keyword(k) => out.push(k.clone()),
                _ => return None,
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// A player's protection or hexproof: "protection from everything", "hexproof".
fn player_mods(s: &str) -> Option<Vec<PlayerModification>> {
    if s == "hexproof" {
        // CR 702.11c
        return Some(vec![PlayerModification::Hexproof]);
    }
    let q = s.strip_prefix("protection from ")?;
    Some(
        protection_qualities(q)?
            .into_iter()
            .map(PlayerModification::ProtectionFrom)
            .collect(),
    )
}

/// "You have protection from everything", "You have protection from each of your
/// opponents", "You have protection from the chosen card name" (CR 702.16b–e).
fn you_have_protection(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("you have ")?;
    if !r.starts_with("protection from ") {
        return None;
    }
    Some(
        player_mods(r)?
            .into_iter()
            .map(|m| {
                static_ability(
                    StaticEffect::PlayerEffect {
                        affected: PlayerFilter::You,
                        effect: m,
                    },
                    text,
                )
            })
            .collect(),
    )
}

/// "You gain protection from everything until your next turn", "you gain hexproof until
/// end of turn".
fn you_gain_protection(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (duration, l) = duration_suffix(l);
    let r = l.strip_prefix("you gain ")?;
    let mods = player_mods(r)?;
    Some(Effect::seq(
        mods.into_iter()
            .map(|effect| Effect::AddPlayerEffect {
                who: PlayerRef::You,
                effect,
                duration: duration.clone(),
            })
            .collect(),
    ))
}

/// "[objects] lose [keywords] until end of turn", "that permanent loses indestructible
/// until end of turn", "until end of turn, ~ loses hexproof and gains first strike and
/// deathtouch". Losing hexproof also loses every "hexproof from" ability (CR 702.11e).
fn loses_keywords(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = match l.strip_prefix("until end of turn, ") {
        Some(r) => format!("{r} until end of turn"),
        None => l.to_string(),
    };
    let (subject, verb_gain, rest) = [(" loses ", "gains"), (" lose ", "gain")]
        .into_iter()
        .find_map(|(v, g)| l.split_once(v).map(|(s, r)| (s.to_string(), g, r.to_string())))?;
    let (duration, rest) = duration_suffix(&rest);
    // "loses hexproof and gains first strike": the lost part, then what's gained.
    let (lost, gained) = match rest
        .split_once(" and gains ")
        .or_else(|| rest.split_once(" and gain "))
    {
        Some((a, g)) => (a.to_string(), Some(g.to_string())),
        None => (rest.to_string(), None),
    };
    let lost = keyword_list(&lost)?;
    // Only whole keywords can be lost this way ("loses protection from red" would need
    // to single out one protection ability).
    if lost.iter().any(|k| k.filter.is_some() || k.cost.is_some() || k.n.is_some()) {
        return None;
    }
    // Parse "[subject] gains [something] [duration]" to resolve the subject (targets,
    // "creatures your opponents control", pronouns) the way granting effects do.
    let probe_keywords = match &gained {
        Some(g) => g.clone(),
        None => lost
            .iter()
            .map(|k| k.kind.name().to_lowercase())
            .collect::<Vec<_>>()
            .join(" and "),
    };
    let dur = match duration {
        Duration::EndOfTurn => " until end of turn",
        Duration::UntilYourNextTurn => " until your next turn",
        Duration::UntilEndOfYourNextTurn => " until the end of your next turn",
        _ => "",
    };
    let probe = format!("{subject} {verb_gain} {probe_keywords}{dur}");
    let Effect::Modify {
        what,
        mods,
        duration,
    } = parse_simple(&probe, b)?
    else {
        return None;
    };
    let mut out: Vec<Modification> = lost
        .iter()
        .map(|k| Modification::RemoveKeyword(k.kind))
        .collect();
    if gained.is_some() {
        out.extend(mods);
    }
    Some(Effect::Modify {
        what,
        mods: out,
        duration,
    })
}

/// "Instant and sorcery spells you control have lifelink", "Red instant and sorcery
/// spells you control have lifelink": a static ability affecting spells on the stack
/// (CR 702.15d: lifelink works whatever zone the source deals damage from).
fn spells_you_control_have(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, kws) = l.split_once(" spells you control have ")?;
    let mut types = Vec::new();
    let mut adjectives = Vec::new();
    let words: Vec<&str> = subject.split_whitespace().collect();
    for w in words {
        match w {
            "and" | "or" => {}
            _ => {
                let (f, _, tail) = parse_object_phrase(w)?;
                if !end(tail).is_empty() {
                    return None;
                }
                match f {
                    Filter::Type(_) => types.push(f),
                    other => adjectives.push(other),
                }
            }
        }
    }
    let mods: Vec<Modification> = keyword_list(kws)?
        .into_iter()
        .map(Modification::AddKeyword)
        .collect();
    let mut parts = adjectives;
    match types.len() {
        0 => {}
        1 => parts.push(types.pop().unwrap()),
        _ => parts.push(Filter::Or(types)),
    }
    parts.push(Filter::Spell);
    parts.push(Filter::ControlledBy(PlayerRel::You));
    Some(vec![static_ability(
        StaticEffect::Continuous {
            affected: Filter::and(parts),
            mods,
        },
        text,
    )])
}

/// "Creatures your opponents control with hexproof can be the targets of spells and
/// abilities you control as though they didn't have hexproof" (Glaring Spotlight), "Your
/// opponents and permanents your opponents control with hexproof can ..." (CR 702.11e).
fn as_though_no_hexproof(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let subject = l.strip_suffix(
        " with hexproof can be the targets of spells and abilities you control as though they didn't have hexproof",
    )?;
    let name = match subject {
        "creatures your opponents control" => {
            crate::kw::hexproof::OPPONENT_CREATURES_AS_THOUGH_NO_HEXPROOF
        }
        "your opponents and permanents your opponents control" => {
            crate::kw::hexproof::OPPONENTS_AND_PERMANENTS_AS_THOUGH_NO_HEXPROOF
        }
        _ => return None,
    };
    Some(vec![static_ability(StaticEffect::Custom(name.into()), text)])
}

inventory::submit! {
    StaticPattern { name: "you have protection from", priority: 100, parse: you_have_protection }
}
inventory::submit! {
    StaticPattern { name: "as though they didn't have hexproof", priority: 100, parse: as_though_no_hexproof }
}
inventory::submit! {
    StaticPattern { name: "spells you control have [keywords]", priority: 100, parse: spells_you_control_have }
}
inventory::submit! {
    EffectPattern { name: "you gain protection/hexproof", priority: 100, parse: you_gain_protection }
}
inventory::submit! {
    EffectPattern { name: "[objects] lose [keywords]", priority: 100, parse: loses_keywords }
}
