//! The game start, grammar V (cards5-game-rules-dice):
//!
//! - "If ~ is in your opening hand and you're not the starting player, you may begin the
//!   game with ~ on the battlefield with a luck counter on it. If you do, exile a card from
//!   your hand." (Gemstone Caverns): the Leylines' opening-hand action (CR 103.6a) for a
//!   player who isn't the starting player (CR 103.1), with counters it begins the game
//!   with (CR 122.6) and an instruction after it (`opening_hand::begin_on_battlefield`).

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "a luck counter", "two charge counters".
fn counters_on_it(s: &str) -> Option<(crate::types::CounterKind, u32)> {
    let (n, rest) = crate::oracle::phrases::parse_number(s)?;
    let n = n.as_const().filter(|n| *n > 0)? as u32;
    let kind = rest
        .trim()
        .strip_suffix(" counters")
        .or_else(|| rest.trim().strip_suffix(" counter"))?;
    if kind.is_empty() || kind.contains(' ') && !kind.starts_with(['+', '-']) {
        return None;
    }
    Some((kind.into(), n))
}

fn begin_the_game_with_it(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let (first, then) = match lower.split_once(". if you do, ") {
        Some((a, b)) => (a, Some(b)),
        None => (end(&lower), None),
    };
    let r = first.strip_prefix("if ~ is in your opening hand")?;
    let (not_starting_player, r) = match r.strip_prefix(" and you're not the starting player") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let r = ["~", "it"]
        .iter()
        .find_map(|it| r.strip_prefix(&format!(", you may begin the game with {it} on the battlefield")))?;
    let counters = match r {
        "" => vec![],
        r => vec![counters_on_it(r.strip_prefix(" with ")?.strip_suffix(" on it")?)?],
    };
    // (The plain form is `r600_opening_hand`'s.)
    if !not_starting_player && counters.is_empty() && then.is_none() {
        return None;
    }
    let then = match then {
        Some(e) => {
            let mut b = Builder::new(ctx);
            let e = parse_effect_text(e, &mut b)?;
            if !b.targets.is_empty() {
                return None;
            }
            Some(Box::new(e))
        }
        None => None,
    };
    let mut s = StaticAbility::new(StaticEffect::OpeningHand {
        delayed: None,
        not_starting_player,
        counters,
        then,
    });
    s.zone = FunctionZone::Hand;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
}

inventory::submit! { AbilityPattern { name: "game start 5: begin the game with ~ on the battlefield (not the starting player, counters, if you do)", priority: 0, parse: begin_the_game_with_it } }

/// The cards revealed from outside the game, and the one an opponent chose.
const OUTSIDE_REVEALED: Var = vars::USER + 43301;
const OPPONENT_CHOSE: Var = vars::USER + 43302;
const DECIDER: Var = vars::USER + 43303;

/// "reveal exactly two cards you own with different names from outside the game"
/// (Extrapolate the Impossible): that many cards from outside the game (the sideboard in
/// a match, CR 400.11, 100.4a), chosen with different names (if the player can't, they
/// reveal none); "them" afterward.
fn reveal_exactly_from_outside(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal exactly ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let n = n.as_const().filter(|n| *n >= 2)?;
    let r = r
        .trim_start()
        .strip_suffix(" you own with different names from outside the game")?;
    if r != "cards" {
        return None;
    }
    b.it = Sel::Var(OUTSIDE_REVEALED);
    Some(Effect::seq(vec![
        Effect::Store {
            var: crate::kw::reveal_from_hand::REVEALED,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Outside),
                    Filter::OwnedBy(PlayerRel::You),
                    Filter::Together(TargetGroup::DifferentNames),
                ]),
                count: Value::c(n),
                up_to: false,
                store: None,
            },
        },
        // "Exactly two": fewer can't be revealed (none are).
        Effect::If {
            cond: Condition::Compare(
                Value::CountSel(Box::new(Sel::Var(crate::kw::reveal_from_hand::REVEALED))),
                Cmp::Lt,
                Value::c(n),
            ),
            then: Box::new(Effect::Store {
                var: crate::kw::reveal_from_hand::REVEALED,
                sel: Sel::None,
            }),
            otherwise: Box::new(Effect::Noop),
        },
        Effect::Store {
            var: OUTSIDE_REVEALED,
            sel: Sel::Var(crate::kw::reveal_from_hand::REVEALED),
        },
        Effect::Custom(crate::kw::reveal_from_hand::REVEAL_CHOSEN.into()),
    ]))
}

inventory::submit! { EffectPattern { name: "game start 5: reveal exactly N cards you own with different names from outside the game", priority: 100, parse: reveal_exactly_from_outside } }

/// "An opponent chooses one of them." after [`reveal_exactly_from_outside`]: the opponent
/// who chooses is chosen by you (CR 801.5a, `Game::deciding_opponent`); "that card" is
/// the chosen one.
fn opponent_chooses_one_of_them(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "an opponent chooses one of them"
        || !matches!(b.it, Sel::Var(v) if v == OUTSIDE_REVEALED)
    {
        return None;
    }
    b.it = Sel::Var(OPPONENT_CHOSE);
    Some(Effect::seq(vec![
        crate::kw::choice_grammar::deciding_opponent_effect(DECIDER)?,
        Effect::Store {
            var: OPPONENT_CHOSE,
            sel: Sel::Choose {
                chooser: PlayerRef::Var(DECIDER),
                filter: Filter::In(Box::new(Sel::Var(OUTSIDE_REVEALED))),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
    ]))
}

inventory::submit! { EffectPattern { name: "game start 5: an opponent chooses one of them (the revealed cards from outside the game)", priority: 100, parse: opponent_chooses_one_of_them } }

/// "You put that card into your hand." after [`opponent_chooses_one_of_them`].
fn you_put_that_card_into_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    if l != "put that card into your hand" || !matches!(b.it, Sel::Var(v) if v == OPPONENT_CHOSE)
    {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Var(OPPONENT_CHOSE),
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "game start 5: you put that card into your hand (the opponent's choice)", priority: 100, parse: you_put_that_card_into_hand } }
