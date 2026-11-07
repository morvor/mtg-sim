//! Seats and directions (CR 101.4, 103.1; see `crate::seats`):
//!
//! ```text
//! seat      := "the player to " ("your" | "their") " " side
//!            | "the opponent to your " side
//!            | "the next player in " ("the chosen" | "that") " direction"
//! side      := "left" | "right"
//! choose    := "choose left or right"
//! attack    := "each player may attack only the nearest opponent in the [last] chosen
//!               direction and planeswalkers controlled by that opponent"
//! grenzo    := "starting with you, each player chooses " kinds " from among the
//!               permanents controlled by " seat
//! ```
//!
//! "The next player in the chosen direction" is relative to each player of an "each
//! player" instruction. The direction is the source's chosen word ("left" or "right"),
//! the latest choice being "the last chosen direction".

use super::{EffectPattern, FilterSuffixPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// A seat phrase at the start of `s`, with the rest.
pub fn seat_phrase(s: &str) -> Option<(PlayerRef, &str)> {
    let s = s.trim_start();
    let seat = |of: PlayerRef, side: SeatSide, opponent: bool| PlayerRef::Seat {
        of: Box::new(of),
        side,
        opponent,
    };
    for (p, of, opponent) in [
        ("the player to your ", PlayerRef::You, false),
        ("the player to their ", PlayerRef::Iterated, false),
        ("the opponent to your ", PlayerRef::You, true),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            for (w, side) in [("left", SeatSide::Left), ("right", SeatSide::Right)] {
                if let Some(rest) = r.strip_prefix(w) {
                    if rest.is_empty() || rest.starts_with([' ', ',', '.', '\'']) {
                        return Some((seat(of, side, opponent), rest));
                    }
                }
            }
            return None;
        }
    }
    for p in [
        "the next player in the chosen direction",
        "the next player in that direction",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with([' ', ',', '.']) {
                return Some((seat(PlayerRef::Iterated, SeatSide::Chosen, false), rest));
            }
        }
    }
    None
}

/// "controlled by [seat]" / "[seat] controls" after an object phrase.
fn controlled_by_seat<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    if let Some(r) = t.strip_prefix("controlled by ") {
        let (p, rest) = seat_phrase(r)?;
        return Some((Filter::ControlledByPlayer(Box::new(p)), rest));
    }
    let (p, rest) = seat_phrase(t)?;
    let rest = rest.strip_prefix(" controls")?;
    Some((Filter::ControlledByPlayer(Box::new(p)), rest))
}

inventory::submit! { FilterSuffixPattern { name: "seats: controlled by the player to their left", priority: 100, parse: controlled_by_seat } }

/// "choose left or right".
fn choose_direction(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "choose left or right").then(|| Effect::Choose {
        who: PlayerRef::You,
        kind: ChoiceKind::Word(crate::seats::DIRECTIONS.iter().map(|s| s.to_string()).collect()),
    })
}

inventory::submit! { EffectPattern { name: "seats: choose left or right", priority: 100, parse: choose_direction } }

/// "each player may attack only the nearest opponent in the [last] chosen direction and
/// planeswalkers controlled by that opponent".
fn attack_only_nearest(l: &str) -> Option<Restriction> {
    let r = end(l).strip_prefix("each player may attack only the nearest opponent ")?;
    let (side, r) = if let Some(r) = r
        .strip_prefix("in the chosen direction")
        .or_else(|| r.strip_prefix("in the last chosen direction"))
    {
        (SeatSide::Chosen, r)
    } else if let Some(r) = r.strip_prefix("to the left") {
        (SeatSide::Left, r)
    } else {
        (SeatSide::Right, r.strip_prefix("to the right")?)
    };
    (r == " and planeswalkers controlled by that opponent")
        .then_some(Restriction::AttackOnlyNearestOpponent { side })
}

fn attack_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = attack_only_nearest(l)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "seats: each player may attack only the nearest opponent in the chosen direction", priority: 100, parse: attack_static } }

/// "until your next turn, each player may attack only the nearest opponent in the last
/// chosen direction and planeswalkers controlled by that opponent".
fn attack_until(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until your next turn, ")?;
    Some(Effect::AddRestriction {
        restriction: attack_only_nearest(r)?,
        duration: Duration::UntilYourNextTurn,
    })
}

inventory::submit! { EffectPattern { name: "seats: until your next turn, each player may attack only the nearest opponent", priority: 100, parse: attack_until } }

/// The permanents chosen by [`starting_with_you_choose_kinds`].
pub const SEAT_CHOSEN: Var = vars::USER + 9612;

/// "Starting with you, each player chooses an artifact, a creature, and a land from among
/// the permanents controlled by the player to their left." (Grenzo's Rebuttal): each
/// player in turn order chooses one of each kind (a permanent of several kinds may be
/// chosen for any of them, and for more than one; Grenzo's Rebuttal ruling), knowing the
/// earlier choices (CR 101.4b). Later "each permanent chosen this way" is all of them.
fn starting_with_you_choose_kinds(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("starting with you, each player chooses ")?;
    let (kinds, from) = r.split_once(" from among the permanents controlled by ")?;
    let (seat, rest) = seat_phrase(from)?;
    if !rest.is_empty() {
        return None;
    }
    let kinds = kinds.replace(", and ", ", ");
    let mut filters = Vec::new();
    for k in kinds.split(", ") {
        let k = k
            .strip_prefix("an ")
            .or_else(|| k.strip_prefix("a "))?;
        let (f, plural, tail) = parse_object_phrase(k)?;
        if plural || !tail.trim().is_empty() || f.zone().is_some() {
            return None;
        }
        filters.push(f);
    }
    if filters.len() < 2 {
        return None;
    }
    let choices = filters
        .into_iter()
        .map(|f| Effect::Store {
            var: SEAT_CHOSEN,
            sel: Sel::Union(vec![
                Sel::Var(SEAT_CHOSEN),
                Sel::Choose {
                    chooser: PlayerRef::Iterated,
                    filter: Filter::and(vec![
                        f,
                        Filter::InZone(ZoneKind::Battlefield),
                        Filter::ControlledByPlayer(Box::new(seat.clone())),
                    ]),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
            ]),
        })
        .collect();
    for name in [
        "each permanent chosen this way",
        "the chosen permanents",
        "permanents chosen this way",
    ] {
        b.named.push((name.into(), Sel::Var(SEAT_CHOSEN)));
    }
    Some(Effect::Seq(vec![
        Effect::Store {
            var: SEAT_CHOSEN,
            sel: Sel::Union(vec![]),
        },
        Effect::InTurnOrder {
            first: TurnOrderStart::You,
            who: PlayerFilter::Any,
            effect: Box::new(Effect::Seq(choices)),
            direction: SeatSide::Left,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "seats: starting with you, each player chooses an artifact, a creature, and a land from among the permanents controlled by the player to their left", priority: 90, parse: starting_with_you_choose_kinds } }

/// The seat named in a target's filter ("target creature the opponent to your left
/// controls").
fn seat_in_targets(b: &Builder) -> Option<PlayerRef> {
    fn find(f: &Filter) -> Option<PlayerRef> {
        match f {
            Filter::ControlledByPlayer(p) if matches!(**p, PlayerRef::Seat { .. }) => {
                Some((**p).clone())
            }
            Filter::And(v) | Filter::Or(v) => v.iter().find_map(find),
            _ => None,
        }
    }
    b.targets.iter().rev().find_map(|t| match &t.what {
        TargetKind::Object(f) => find(f),
        _ => None,
    })
}

/// "Then that player may copy ~ and may choose new targets for the copy." after a
/// sentence naming "the opponent to your left" (Barroom Brawl): that player copies the
/// spell (CR 707.10, 707.10c); in the copy, "you" is that player.
fn that_seat_may_copy(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "that player may copy ~ and may choose new targets for the copy" {
        return false;
    }
    let Some(who) = seat_in_targets(b) else {
        return false;
    };
    let copy = Effect::AsPlayer {
        who,
        effect: Box::new(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::CopySpell {
                what: Sel::This,
                count: Value::c(1),
                new_targets: true,
            }),
        }),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, copy]);
    true
}

inventory::submit! { FollowupPattern { name: "seats: then that player may copy ~", priority: 100, apply: that_seat_may_copy } }
