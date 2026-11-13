//! Control changes involving several players (CR 101.4, 608.2f, 701.12):
//!
//! ```text
//! each    := "each player gains control of " ("all" | "each") objects-of-their-seat
//!          | "each player gains control of a " object " of your choice controlled by " seat
//!          | "each player gains control of the creature they chose"
//! order   := "starting with you and proceeding in the chosen direction, each player
//!             chooses a " object " controlled by the next player in that direction"
//! scramble:= "for each nonland permanent, choose a player at random" ". then each player
//!             gains control of each permanent for which they were chosen"
//! cultural:= "choose any number of " objects " target player controls" ". choose the same
//!             number of " objects " another target player controls" ". those players
//!             exchange control of those creatures"
//! targets := "[you may] choose new targets for the spell" | "the spell's controller may
//!             choose new targets for it"
//! ```
//!
//! "Each player gains control of ..." is one instruction for all the players: what each
//! gains is determined (and chosen) for all of them before control of any of it changes,
//! then all of it changes at once (see `crate::simultaneous`).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether `f` refers to the player of an "each player" instruction.
fn mentions_iterated(f: &Filter) -> bool {
    serde_json::to_string(f).is_ok_and(|j| j.contains("\"Iterated\""))
}

/// The "controlled by [seat]" part of a filter.
fn seat_part(f: &Filter) -> Option<Filter> {
    match f {
        Filter::ControlledByPlayer(p) if matches!(**p, PlayerRef::Seat { .. }) => Some(f.clone()),
        Filter::And(v) => v.iter().find_map(seat_part),
        _ => None,
    }
}

fn on_battlefield(f: Filter) -> Filter {
    if f.zone().is_some() {
        f
    } else {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
    }
}

fn each_player_gains(what: Sel) -> Effect {
    Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::GainControl {
            what,
            who: PlayerRef::Iterated,
            duration: Duration::Permanent,
        }),
    }
}

/// Marks, in [`Builder::named`], what "the creature they chose" is.
const THEY_CHOSE: &str = "\u{1}control players: the creature they chose";

/// "each player gains control of all nonland permanents other than ~ controlled by the
/// next player in the chosen direction" (Aminatou), "... of a nonland permanent of your
/// choice controlled by the player to their right" (Inniaz), "... of the creature they
/// chose" (Order of Succession).
fn each_player_gains_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player gains control of ")?;
    if r == "the creature they chose" {
        let sel = b
            .named
            .iter()
            .rev()
            .find(|(n, _)| n == THEY_CHOSE)
            .map(|(_, s)| s.clone())?;
        return Some(each_player_gains(sel));
    }
    if let Some(p) = r.strip_prefix("all ").or_else(|| r.strip_prefix("each ")) {
        let (f, _, tail) = parse_object_phrase(p)?;
        if !end(tail).is_empty() || !mentions_iterated(&f) {
            return None;
        }
        return Some(each_player_gains(Sel::All(on_battlefield(f))));
    }
    let p = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (noun, seat) = p.split_once(" of your choice ")?;
    let phrase = format!("{noun} {seat}");
    let (f, plural, tail) = parse_object_phrase(&phrase)?;
    if plural || !end(tail).is_empty() || !mentions_iterated(&f) {
        return None;
    }
    Some(each_player_gains(Sel::Choose {
        chooser: PlayerRef::You,
        filter: on_battlefield(f),
        count: Value::c(1),
        up_to: false,
        store: None,
    }))
}

inventory::submit! { EffectPattern { name: "control players: each player gains control of [their objects]", priority: 95, parse: each_player_gains_control } }

/// The creatures chosen by [`proceeding_in_direction_choose`].
const DIRECTION_CHOSEN: Var = vars::USER + 9613;

/// "starting with you and proceeding in the chosen direction, each player chooses a
/// creature controlled by the next player in that direction" (Order of Succession): one
/// at a time around the table (CR 101.4b); not targeted. "The creature they chose" is
/// then the chosen creature controlled by the player they chose from (no control changes
/// while they choose).
fn proceeding_in_direction_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix(
        "starting with you and proceeding in the chosen direction, each player chooses ",
    )?;
    let p = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(p)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let seat = seat_part(&f)?;
    let filter = on_battlefield(f);
    b.named.push((
        THEY_CHOSE.into(),
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(DIRECTION_CHOSEN))),
            seat,
        ])),
    ));
    Some(Effect::Seq(vec![
        Effect::Store {
            var: DIRECTION_CHOSEN,
            sel: Sel::Union(vec![]),
        },
        Effect::InTurnOrder {
            first: TurnOrderStart::You,
            who: PlayerFilter::Any,
            effect: Box::new(Effect::Store {
                var: DIRECTION_CHOSEN,
                sel: Sel::Union(vec![
                    Sel::Var(DIRECTION_CHOSEN),
                    Sel::Choose {
                        chooser: PlayerRef::Iterated,
                        filter,
                        count: Value::c(1),
                        up_to: false,
                        store: None,
                    },
                ]),
            }),
            direction: SeatSide::Chosen,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "control players: starting with you and proceeding in the chosen direction, each player chooses", priority: 95, parse: proceeding_in_direction_choose } }

/// The permanents of "for each nonland permanent, choose a player at random".
const SCRAMBLED: Var = vars::USER + 9614;
/// Each of them in turn.
const SCRAMBLED_EACH: Var = vars::USER + 9615;
/// The player chosen for it.
const SCRAMBLED_PLAYER: Var = vars::USER + 9616;

/// "for each nonland permanent, choose a player at random" (Scrambleverse): a player for
/// each permanent, any player (its controller included; Scrambleverse ruling).
fn for_each_choose_player_at_random(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let noun = r.strip_suffix(", choose a player at random")?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if plural || !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let random = crate::kw::choice_grammar::random_player_effect(SCRAMBLED_PLAYER, PlayerRef::EachPlayer)?;
    Some(Effect::Seq(vec![
        Effect::Store {
            var: SCRAMBLED,
            sel: Sel::All(on_battlefield(f)),
        },
        Effect::ForEach {
            sel: Sel::Var(SCRAMBLED),
            var: SCRAMBLED_EACH,
            effect: Box::new(random),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "control players: for each permanent, choose a player at random", priority: 95, parse: for_each_choose_player_at_random } }

/// "Then each player gains control of each permanent for which they were chosen." after
/// [`for_each_choose_player_at_random`]: all the players are chosen first, then all the
/// control changes happen at the same time (Scrambleverse rulings). "Those permanents"
/// afterwards are all of them.
fn gains_control_for_which_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player gains control of each permanent for which they were chosen" {
        return false;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(Effect::ForEach {
        var: SCRAMBLED_EACH,
        effect,
        ..
    }) = v.last_mut()
    else {
        return false;
    };
    let random = std::mem::take(effect.as_mut());
    **effect = Effect::Seq(vec![
        random,
        Effect::GainControl {
            what: Sel::Var(SCRAMBLED_EACH),
            who: PlayerRef::Var(SCRAMBLED_PLAYER),
            duration: Duration::Permanent,
        },
    ]);
    b.it = Sel::Var(SCRAMBLED);
    b.named
        .push(("those permanents".into(), Sel::Var(SCRAMBLED)));
    true
}

inventory::submit! { FollowupPattern { name: "control players: then each player gains control of each permanent for which they were chosen", priority: 100, apply: gains_control_for_which_chosen } }

/// The second group of "choose the same number of creatures another target player
/// controls".
const SAME_NUMBER: Var = vars::USER + 9617;
/// Marks, in [`Builder::named`], the two groups to exchange.
const EXCHANGE_GROUPS: &str = "\u{1}control players: exchange groups";

/// "Choose the same number of creatures another target player controls." after "Choose
/// any number of creatures target player controls." (Cultural Exchange): exactly as many
/// (if they can't be, the exchange can't happen, CR 701.12a).
fn choose_the_same_number(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("choose the same number of ") else {
        return false;
    };
    let Effect::Store {
        var: first_var,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: first_filter,
            ..
        },
    } = prev
    else {
        return false;
    };
    let first_var = *first_var;
    // "target player controls" in the first choice.
    let Some(first_slot) = target_controller_slot(first_filter) else {
        return false;
    };
    let Some((noun, who)) = r.split_once(" another target player controls") else {
        return false;
    };
    if !who.is_empty() {
        return false;
    }
    let Some((f, true, tail)) = parse_object_phrase(noun) else {
        return false;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return false;
    }
    // "another target player": a second player target, different from the first.
    let Some(mut spec) = b.targets.get(first_slot as usize).cloned() else {
        return false;
    };
    if !matches!(spec.what, TargetKind::Player(_)) {
        return false;
    }
    spec.distinct_from = vec![first_slot];
    let slot = b.add_target(spec, "another target player");
    let second = Effect::Store {
        var: SAME_NUMBER,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                f,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::ControlledBy(PlayerRel::Target(slot)),
            ]),
            count: Value::CountSel(Box::new(Sel::Var(first_var))),
            up_to: false,
            store: None,
        },
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, second]);
    b.named.push((
        EXCHANGE_GROUPS.into(),
        Sel::Union(vec![Sel::Var(first_var), Sel::Var(SAME_NUMBER)]),
    ));
    true
}

/// The target slot of "[objects] target player controls" in a filter.
fn target_controller_slot(f: &Filter) -> Option<u8> {
    match f {
        Filter::ControlledBy(PlayerRel::Target(n)) => Some(*n),
        Filter::And(v) => v.iter().find_map(target_controller_slot),
        _ => None,
    }
}

inventory::submit! { FollowupPattern { name: "control players: choose the same number of [objects] another target player controls", priority: 100, apply: choose_the_same_number } }

/// "Those players exchange control of those creatures." after
/// [`choose_the_same_number`]: the two groups are exchanged (CR 701.12a, 701.12b).
fn those_players_exchange(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(
        end(l),
        "those players exchange control of those creatures"
            | "those players exchange control of those permanents"
    ) {
        return None;
    }
    let groups = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == EXCHANGE_GROUPS)
        .map(|(_, s)| s.clone())?;
    let Sel::Union(v) = groups else {
        return None;
    };
    let [a, bb] = v.as_slice() else {
        return None;
    };
    Some(Effect::ExchangeControl {
        a: a.clone(),
        b: bb.clone(),
    })
}

inventory::submit! { EffectPattern { name: "control players: those players exchange control of those creatures", priority: 95, parse: those_players_exchange } }

/// The target slot of a spell target ("target noncreature spell").
fn spell_target(b: &Builder) -> Option<u8> {
    b.targets
        .iter()
        .position(|t| matches!(t.what, TargetKind::Spell(_)))
        .map(|i| i as u8)
}

/// "you may choose new targets for the spell" after a sentence about the spell that
/// triggered the ability (Perplexing Chimera; CR 115.7d): the ability's controller
/// chooses.
fn choose_new_targets_for_the_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l);
    let (may, r) = match r.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    if !matches!(r, "choose new targets for the spell" | "choose new targets for that spell") {
        return None;
    }
    if !b.in_trigger || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let e = Effect::ChangeTargets {
        what: Sel::TriggerSpell,
        who: PlayerRef::You,
        how: TargetChange::ChooseNew,
        to: None,
    };
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

inventory::submit! { EffectPattern { name: "control players: you may choose new targets for the spell", priority: 95, parse: choose_new_targets_for_the_spell } }

/// "the spell's controller may choose new targets for it" (Sudden Substitution): whoever
/// controls the target spell then (CR 115.7d).
fn spells_controller_may_choose_new_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "the spell's controller may choose new targets for it" {
        return None;
    }
    let slot = spell_target(b)?;
    let spell = Sel::Target(slot);
    Some(Effect::AsPlayer {
        who: PlayerRef::ControllerOf(Box::new(spell.clone())),
        effect: Box::new(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::ChangeTargets {
                what: spell,
                who: PlayerRef::You,
                how: TargetChange::ChooseNew,
                to: None,
            }),
        }),
    })
}

inventory::submit! { EffectPattern { name: "control players: the spell's controller may choose new targets for it", priority: 95, parse: spells_controller_may_choose_new_targets } }
