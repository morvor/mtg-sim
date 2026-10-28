//! More game-rules grammar (round 3):
//!
//! - "protection from each color that's not in your commander's color identity"
//!   (Commander's Plate, CR 903.4, 903.4f);
//! - "you may activate ~'s loyalty ability two more times this turn" (Comet, Stellar Pup;
//!   CR 606.3);
//! - "[player] loses all [kind] counters" (Survivor's Med Kit; CR 122.1);
//! - "for each differently named dungeon you've completed" (Ellywick Tumblestrum;
//!   CR 309.7);
//! - "[group] have [keyword] and [keyword] and get +N/+N ..." (the keywords listed first);
//! - "each player on your team" (Azra Bladeseeker): you and your teammates, the players
//!   who aren't your opponents (CR 102.3).
//!
//! The behaviors are in `kw/game_rules_4.rs`.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::kw::game_rules_4::{
    more_loyalty_activations, COLOR_OUTSIDE_COMMANDER_IDENTITY, DIFFERENT_DUNGEONS_COMPLETED,
};
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

const OUTSIDE_IDENTITY: &str =
    " protection from each color that's not in your commander's color identity";

/// "Equipped creature gets +3/+3 and has protection from each color that's not in your
/// commander's color identity.": a protection ability from the objects that have a color
/// outside the commander's color identity of the protected permanent's controller (none
/// without a commander, CR 903.4f).
fn protection_outside_identity(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let head = end(l).strip_suffix(OUTSIDE_IDENTITY)?;
    let parsed =
        crate::oracle::statics::parse_static(&format!("{head} protection from creatures"), ctx)?;
    let mut out = Vec::new();
    for a in parsed {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::Continuous { affected, mods } = &s.effect else {
            return None;
        };
        let mut found = false;
        let mods = mods
            .iter()
            .map(|m| match m {
                Modification::AddKeyword(k)
                    if k.kind == crate::keywords::KeywordKind::Protection =>
                {
                    found = true;
                    let mut k = k.clone();
                    k.filter = Some(Filter::Custom(COLOR_OUTSIDE_COMMANDER_IDENTITY.into()));
                    k.text = Some(OUTSIDE_IDENTITY.trim().into());
                    Modification::AddKeyword(k)
                }
                m => m.clone(),
            })
            .collect();
        if !found {
            return None;
        }
        let mut s = s.clone();
        s.effect = StaticEffect::Continuous {
            affected: affected.clone(),
            mods,
        };
        out.push(AbilityDef::new(AbilityKind::Static(s), text));
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "game rules 4: protection from colors outside your commander's color identity", priority: 100, parse: protection_outside_identity } }

/// "Creatures you control have trample and haste and get +2/+2 for each ...": the same
/// as "Creatures you control get +2/+2 for each ... and have trample and haste".
fn have_keywords_and_get(l: &str, _text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (subject, rest) = l.split_once(" have ")?;
    let (keywords, pump) = rest.rsplit_once(" and get ")?;
    // Only a list of two or more keywords joined by "and" (one keyword is the core
    // grammar's "have X and get").
    if !keywords.contains(" and ") || keywords.contains(['"', ',']) {
        return None;
    }
    crate::oracle::statics::parse_static(&format!("{subject} get {pump} and have {keywords}"), ctx)
}

inventory::submit! { StaticPattern { name: "game rules 4: have [keywords] and get", priority: 100, parse: have_keywords_and_get } }

/// "you may activate ~'s loyalty ability two more times this turn" (CR 606.3): that many
/// more activations of the source's loyalty abilities this turn.
fn more_loyalty_times(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("you may activate ~'s loyalty ability ")
        .or_else(|| end(l).strip_prefix("you may activate ~'s loyalty abilities "))?;
    let r = r.strip_suffix(" more times this turn")?;
    let n = match r {
        "one" => 1,
        "two" => 2,
        "three" => 3,
        _ => return None,
    };
    Some(more_loyalty_activations(n))
}

inventory::submit! { EffectPattern { name: "game rules 4: activate ~'s loyalty ability N more times this turn", priority: 100, parse: more_loyalty_times } }

/// "Target player loses all rad counters." (CR 122.1): every counter of that kind on the
/// player is removed.
fn loses_all_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, rest) = l
        .split_once(" loses all ")
        .or_else(|| l.split_once(" lose all "))?;
    let r = rest.strip_suffix(" counters")?;
    let (kind, tail) = crate::oracle::costs::counter_kind(r)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let saved = b.targets.len();
    let Some((who, tail)) = player_ref(subject, b) else {
        b.targets.truncate(saved);
        return None;
    };
    if !tail.trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::RemoveCounters {
        what: Sel::Players(who.clone()),
        n: Value::PlayerCounters(who, kind.clone()),
        kind: Some(kind),
    })
}

inventory::submit! { EffectPattern { name: "game rules 4: player loses all counters of a kind", priority: 100, parse: loses_all_counters } }

/// "for each differently named dungeon you've completed" / "dungeon you've completed"
/// (CR 309.7), hooked into the "for each" counts of `statics::parse_for_each`.
pub fn for_each(s: &str) -> Option<Value> {
    match s {
        "differently named dungeon you've completed" => {
            Some(Value::Custom(DIFFERENT_DUNGEONS_COMPLETED.into()))
        }
        "dungeon you've completed" => Some(Value::Custom(
            crate::kw::dungeon_completion::DUNGEONS_COMPLETED.into(),
        )),
        _ => None,
    }
}

/// The first "for each player" of an effect.
fn first_each_player(e: &mut Effect) -> Option<&mut PlayerRef> {
    match e {
        Effect::ForEachPlayer { who, .. } if matches!(who, PlayerRef::EachPlayer) => Some(who),
        Effect::Seq(v) => v.iter_mut().find_map(first_each_player),
        _ => None,
    }
}

/// "each player on your team may discard a card, then ...": you and your teammates, the
/// players who aren't your opponents (CR 102.3, 808.2, 810.2).
fn each_player_on_your_team(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, rest) = l.split_once("each player on your team ")?;
    if rest.contains("on your team") {
        return None;
    }
    let mut e = parse_clause(&format!("{head}each player {rest}"), b)?;
    let who = first_each_player(&mut e)?;
    *who = PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::OpponentOf(
        Box::new(PlayerRef::You),
    ))));
    Some(e)
}

inventory::submit! { EffectPattern { name: "game rules 4: each player on your team", priority: 100, parse: each_player_on_your_team } }

/// "Search your library, graveyard, and/or outside the game for an instant or sorcery card
/// you own" (Invasion of Arcavios): every card in your own library, graveyard, hand, and
/// outside the game is a card you own (CR 108.3, 400.11b), so "you own" adds nothing.
fn search_for_cards_you_own(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, rest) = l.split_once(" for ")?;
    if !head.starts_with("search your ") || head.contains("'s ") {
        return None;
    }
    let (spec, tail) = rest.split_once(',').unwrap_or((rest, ""));
    let spec2 = spec
        .replace(" cards you own", " cards")
        .replace(" card you own", " card");
    if spec2 == spec {
        return None;
    }
    let sep = if tail.is_empty() && !rest.contains(',') {
        ""
    } else {
        ","
    };
    parse_clause(&format!("{head} for {spec2}{sep}{tail}"), b)
}

inventory::submit! { EffectPattern { name: "game rules 4: search your zones for a card you own", priority: 100, parse: search_for_cards_you_own } }

/// Cards you own outside the game (CR 400.11b).
fn outside_cards(quality: Filter) -> Filter {
    Filter::and(vec![
        quality,
        Filter::Card,
        Filter::InZone(ZoneKind::Outside),
        Filter::OwnedBy(PlayerRel::You),
    ])
}

/// "You may play a card you own from outside the game this turn." (Wish): one card you
/// own outside the game may be played this turn, chosen as it's played (its rulings),
/// following the normal timing rules and paying its costs.
fn may_play_from_outside(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let (spells_only, r) = if let Some(r) = l.strip_prefix("play ") {
        (false, r)
    } else {
        (true, l.strip_prefix("cast ")?)
    };
    let r = r.strip_suffix(" you own from outside the game this turn")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let f = if r == "card" {
        Filter::Card
    } else {
        let (f, false, tail) = parse_object_phrase(r)? else {
            return None;
        };
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    Some(Effect::WithPlayTerms {
        terms: PlayTerms {
            spells_only,
            limit: Some(1),
            ..Default::default()
        },
        effect: Box::new(Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::All(outside_cards(f)),
            duration: Duration::EndOfTurn,
            free: false,
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules 4: you may play a card you own from outside the game this turn", priority: 100, parse: may_play_from_outside } }

/// "Cast any number of Eldrazi spells from among cards you own outside the game without
/// paying their mana costs." (Spawnsire of Ulamog): cast as the ability resolves, one at a
/// time (CR 608.2g), from outside the game (CR 400.11b).
fn cast_any_number_from_outside(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("cast any number of ")?.strip_suffix(
        " from among cards you own outside the game without paying their mana costs",
    )?;
    let (before, after) = r.split_once("spells")?;
    let desc = format!("{before}cards{after}");
    let (quality, _, tail) = parse_object_phrase(desc.trim())?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                outside_cards(quality),
                Filter::not(Filter::Type(crate::types::CardType::Land)),
            ]),
            count: Value::c(99),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    })
}

inventory::submit! { EffectPattern { name: "game rules 4: cast any number of spells from among cards you own outside the game", priority: 100, parse: cast_any_number_from_outside } }
