//! Shuffling objects into libraries (CR 701.24):
//!
//! - "[Player] shuffles their graveyard into their library", "each player shuffles their
//!   hand and graveyard into their library, then draws seven cards", "shuffle the cards
//!   from your hand into your library, then draw that many cards".
//! - "Target player shuffles up to three target cards from their graveyard into their
//!   library", "shuffle any number of target creature cards from your graveyard into your
//!   library".
//! - "[Object]'s owner shuffles it into their library", "the owner of target nonland
//!   permanent shuffles it into their library, then draws two cards".
//!
//! Each becomes [`Effect::ShuffleIntoLibrary`]: the objects move at the same time, and the
//! library is shuffled even if they couldn't be moved or there were none (CR 701.24c-d).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "control_exile: shuffle hand/graveyard into library", priority: 90, parse: p_shuffle_zones }
}
inventory::submit! {
    EffectPattern { name: "control_exile: shuffle target cards into library", priority: 90, parse: p_shuffle_targets }
}
inventory::submit! {
    EffectPattern { name: "control_exile: owner shuffles it into their library", priority: 90, parse: p_owner_shuffles }
}
inventory::submit! {
    EffectPattern { name: "control_exile: choose target", priority: 90, parse: p_choose_target }
}

/// "Choose target artifact or enchantment." as a sentence of its own: the target is chosen
/// as the spell is cast (CR 601.2c, 115.1) and is "it" for the sentences that follow
/// ("Its owner shuffles it into their library."). Choosing it does nothing by itself. A
/// chosen creature stays "that creature" even after "it" has come to mean another object
/// (see `Builder::chosen_creature`).
fn p_choose_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("choose ")?;
    let (spec, tail) = parse_target(r)?;
    let TargetKind::Object(f) = &spec.what else {
        return None;
    };
    if !end(tail).is_empty() || spec.min != 1 || !matches!(spec.max, Value::Const(1)) {
        return None;
    }
    let creature = is_creature(f) && f.zone().is_none_or(|z| z == ZoneKind::Battlefield);
    let slot = b.add_target(spec, r);
    if creature {
        b.chosen_creature = Some((slot, b.targets[slot as usize].text.clone()));
    }
    Some(Effect::Noop)
}

/// Whether a target filter names creatures ("target attacking or blocking creature").
fn is_creature(f: &Filter) -> bool {
    match f {
        Filter::Type(crate::types::CardType::Creature) => true,
        Filter::And(v) => v.iter().any(is_creature),
        Filter::Or(v) => !v.is_empty() && v.iter().all(is_creature),
        _ => false,
    }
}

/// The subject of a "shuffles" sentence: the player, the possessive that refers back to
/// them ("your" for you, "their" otherwise), and the rest after the verb.
fn subject<'a>(l: &'a str, b: &mut Builder) -> Option<(PlayerRef, &'static str, String)> {
    if let Some(r) = l.strip_prefix("shuffle ") {
        return Some((PlayerRef::You, "your", r.to_string()));
    }
    let (who, rest) = player_ref(l, b)?;
    let r = rest.trim().strip_prefix("shuffles ")?.to_string();
    if matches!(who, PlayerRef::You) {
        return None;
    }
    Some((who, "their", r))
}

/// ", then draws N cards" / ", then draw that many cards" after the shuffle.
fn then_draw(rest: &str, who: &PlayerRef, you: bool) -> Option<Option<Effect>> {
    let rest = end(rest);
    if rest.is_empty() {
        return Some(None);
    }
    let r = rest.strip_prefix(", then ")?;
    let r = if you {
        r.strip_prefix("draw ")?
    } else {
        r.strip_prefix("draws ")?
    };
    let (n, tail) = if let Some(t) = r.strip_prefix("that many cards") {
        (Value::Prev, t)
    } else {
        parse_card_count(r)?
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Some(Effect::Draw {
        who: who.clone(),
        n,
    }))
}

fn owned_in(zone: ZoneKind, owner: PlayerRel) -> Sel {
    Sel::All(Filter::and(vec![
        Filter::InZone(zone),
        Filter::OwnedBy(owner),
    ]))
}

/// The cards each player shuffled away, for "then draws that many cards".
const SHUFFLED: Var = vars::USER + 140;

/// The owners named by a subject that is a group of players ("each player", "each
/// opponent"), for moving all of their cards at the same time.
fn group_owners(who: &PlayerRef) -> Option<PlayerRel> {
    match who {
        PlayerRef::EachPlayer => Some(PlayerRel::Any),
        PlayerRef::EachOpponent => Some(PlayerRel::Opponent),
        PlayerRef::EachOtherPlayer => Some(PlayerRel::NotYou),
        _ => None,
    }
}

/// "[player] shuffles their [hand / graveyard / hand and graveyard] into their library
/// [, then draws N cards]".
fn p_shuffle_zones(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, poss, r) = subject(l, b)?;
    let r = r.strip_prefix("the cards from ").unwrap_or(&r);
    let r = r.strip_prefix(poss)?.strip_prefix(' ')?;
    let (zones, r) = [
        (
            "hand and graveyard",
            vec![ZoneKind::Hand, ZoneKind::Graveyard],
        ),
        (
            "graveyard and hand",
            vec![ZoneKind::Hand, ZoneKind::Graveyard],
        ),
        ("graveyard", vec![ZoneKind::Graveyard]),
        ("hand", vec![ZoneKind::Hand]),
    ]
    .into_iter()
    .find_map(|(p, z)| r.strip_prefix(p).map(|x| (z, x)))?;
    let r = r.strip_prefix(" into ")?.strip_prefix(poss)?;
    let r = r.strip_prefix(" library")?;
    let draw = then_draw(r, &who, matches!(who, PlayerRef::You))?;
    let cards = |owner: PlayerRel| {
        if zones.len() == 1 {
            owned_in(zones[0], owner)
        } else {
            Sel::Union(zones.iter().map(|z| owned_in(*z, owner)).collect())
        }
    };
    let that_many = matches!(draw, Some(Effect::Draw { n: Value::Prev, .. }));
    if let Some(owners) = group_owners(&who) {
        // Each of those players' cards are put into their libraries at the same time,
        // each library is shuffled (CR 701.24c-d), and only then does anyone draw.
        let shuffle = Effect::ShuffleIntoLibrary {
            what: cards(owners),
            library: who.clone(),
        };
        let draw = if that_many {
            // Each player draws as many cards as they shuffled away.
            Effect::ForEachPlayer {
                who,
                effect: Box::new(Effect::Draw {
                    who: PlayerRef::Iterated,
                    n: Value::Count(Filter::and(vec![
                        Filter::InZone(ZoneKind::Library),
                        Filter::In(Box::new(Sel::Var(SHUFFLED))),
                        Filter::OwnedBy(PlayerRel::Iterated),
                    ])),
                }),
            }
        } else {
            draw.unwrap_or(Effect::Noop)
        };
        return Some(Effect::seq(vec![
            shuffle,
            Effect::Store {
                var: SHUFFLED,
                sel: Sel::Var(vars::IT),
            },
            draw,
        ]));
    }
    // A single player.
    let shuffle = Effect::ShuffleIntoLibrary {
        what: cards(PlayerRel::Iterated),
        library: PlayerRef::Iterated,
    };
    let body = match draw {
        // "Then draw that many cards": the number of cards shuffled away.
        Some(Effect::Draw { n: Value::Prev, .. }) => Effect::seq(vec![
            shuffle,
            Effect::Draw {
                who: PlayerRef::Iterated,
                n: Value::Prev,
            },
        ]),
        Some(draw) => {
            return Some(Effect::seq(vec![
                Effect::ForEachPlayer {
                    who,
                    effect: Box::new(shuffle),
                },
                draw,
            ]))
        }
        None => shuffle,
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(body),
    })
}

/// "target player shuffles up to three target cards from their graveyard into their
/// library", "shuffle any number of target creature cards from your graveyard into your
/// library".
fn p_shuffle_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, poss, r) = subject(l, b)?;
    let player_rel = match &who {
        PlayerRef::You => PlayerRel::You,
        PlayerRef::Target(n) => PlayerRel::Target(*n),
        _ => return None,
    };
    let (objs, rest) = r.split_once(&format!(" from {poss} graveyard into {poss} library"))?;
    if !end(rest).is_empty() {
        return None;
    }
    // The cards must be in that player's graveyard.
    let (mut spec, tail) = parse_target(objs)?;
    if !end(tail).is_empty() {
        return None;
    }
    let TargetKind::Object(f) = spec.what else {
        return None;
    };
    spec.what = TargetKind::Object(Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(player_rel),
    ]));
    let slot = b.add_target(spec, objs);
    Some(Effect::ShuffleIntoLibrary {
        what: Sel::Target(slot),
        library: who,
    })
}

/// "[object]'s owner shuffles it into their library", "its owner shuffles it into their
/// library", "the owner of [object] shuffles it into their library[, then draws N
/// cards]".
fn p_owner_shuffles(l: &str, b: &mut Builder) -> Option<Effect> {
    let (what, rest) = if let Some(r) = l.strip_prefix("the owner of ") {
        let (obj, rest) = r.split_once(" shuffles ")?;
        let (what, tail) = object_ref(obj, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (what, rest)
    } else if let Some(rest) = l.strip_prefix("its owner shuffles ") {
        (b.it.clone(), rest)
    } else {
        let (obj, rest) = l.split_once("'s owner shuffles ")?;
        let (what, tail) = object_ref(obj, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (what, rest)
    };
    if matches!(what, Sel::All(_) | Sel::Players(_) | Sel::None) {
        return None;
    }
    let rest = ["it into their library", "that card into their library"]
        .iter()
        .find_map(|p| rest.strip_prefix(p))?;
    let owner = PlayerRef::OwnerOf(Box::new(what.clone()));
    let draw = then_draw(rest, &owner, false)?;
    if matches!(draw, Some(Effect::Draw { n: Value::Prev, .. })) {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::ShuffleIntoLibrary {
            what,
            library: owner,
        },
        draw.unwrap_or(Effect::Noop),
    ]))
}
