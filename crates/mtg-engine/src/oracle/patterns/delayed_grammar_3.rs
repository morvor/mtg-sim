//! Delayed triggered abilities, grammar III (cards3-triggers-durations).
//!
//! The cards a player exiled, returned later by that player: "Target player exiles all
//! cards from their hand face down. At the beginning of the end step of that player's next
//! turn, that player returns those cards to their hand." (Suppress; CR 603.7a, 603.7c: the
//! delayed ability refers to the cards the first instruction exiled).
//!
//! A card from the graveyard of the trigger object's owner: "Choose target creature. When
//! that creature dies this turn, return a creature card from its owner's graveyard to the
//! battlefield under the control of that creature's owner." (Reincarnation: the creature
//! card is chosen as the delayed ability resolves, by its controller; it may be the
//! creature that died).
//!
//! "At the beginning of the next end step, if the player hasn't played the card, they put
//! it into their graveyard." (Elkin Lair, after "that player exiles a card at random from
//! their hand. The player may play that card this turn."): read as "if you haven't played
//! the card, put it into its owner's graveyard" (`delayed_grammar_2.rs` and the
//! interpretation `delayed-if-not-played-means-still-exiled`: the card is still exiled),
//! the card's owner being the player who exiled it from their hand.
//!
//! "Return that card and the sacrificed card to the battlefield under your control at the
//! beginning of your next upkeep." (Rescue from the Underworld): "the sacrificed card" is
//! the card the sacrifice cost put into a public zone (CR 400.7j: a token sacrificed
//! returns nothing; one put into another public zone instead returns from there).
//!
//! "this turn" triggers about the chosen player: "Choose target opponent. Whenever a
//! creature you control deals combat damage to that player this turn, create a tapped
//! Treasure token." (Great Train Heist; CR 603.7b, 603.7c: the player the creating effect
//! chose).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether the effect ends with an exile instruction (possibly performed by another
/// player).
fn ends_with_exile(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile),
        Effect::AsPlayer { effect, .. } => ends_with_exile(effect),
        Effect::Exile { .. } => true,
        _ => false,
    }
}

/// Whether the effect's first instruction exiles a group of objects ("Exile all opponents'
/// graveyards."), whose cards the following sentences call "those cards" and "them".
fn starts_with_group_exile(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.first().is_some_and(starts_with_group_exile),
        Effect::Exile {
            what: Sel::All(_), ..
        } => true,
        _ => false,
    }
}

/// "Exile all opponents' graveyards. [...] At the beginning of the next end step, if any
/// of those cards remain exiled, return them to their owners' graveyards." (Mnemonic
/// Betrayal): "them" is the exiled cards (the exile instruction's result, CR 603.7c), even
/// after an instruction in between that doesn't name other objects.
fn delayed_return_of_group_exiled(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let s = end(s);
    if !starts_with_group_exile(prev)
        || !s.starts_with("at the beginning of ")
        || !s.contains(", if any of those cards remain exiled, ")
        || !crate::oracle::patterns::oracle_hardening_referents::is_no_referent(&b.it)
    {
        return false;
    }
    let saved = b.it.clone();
    b.it = Sel::Var(EXILED_GROUP);
    let e = crate::oracle::effects::parse_sentence(s, b);
    b.it = saved;
    let Some(e) = e else {
        return false;
    };
    // The exiled cards, noted as the exile instruction resolves.
    let Effect::Seq(v) = prev else {
        return false;
    };
    let note = Effect::Store {
        var: EXILED_GROUP,
        sel: Sel::Var(vars::IT),
    };
    v.insert(1, note);
    v.push(e);
    true
}

/// The cards a group exile exiled (see [`delayed_return_of_group_exiled`]).
const EXILED_GROUP: Var = vars::USER + 13_101;

inventory::submit! { FollowupPattern { name: "delayed grammar 3: return them (the group exiled) later", priority: 100, apply: delayed_return_of_group_exiled } }

fn those_exiled_cards_returned_later(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_exile(prev) {
        return false;
    }
    let s = end(s);
    let Some((delay, instr)) = s.split_once(", ") else {
        return false;
    };
    if !delay.starts_with("at the beginning of ")
        || !matches!(
            instr,
            "that player returns those cards to their hand" | "they return those cards to their hand"
        )
    {
        return false;
    }
    // "Those cards": what the exile instruction exiled (its "it", CR 603.7c).
    let saved = b.it.clone();
    b.it = Sel::Var(vars::IT);
    let e = super::delayed_grammar::delayed_instruction(
        &format!("{delay}, return those cards to their owner's hand"),
        b,
    );
    b.it = saved;
    let Some(e) = e else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar 3: that player returns those (exiled) cards to their hand later", priority: 100, apply: those_exiled_cards_returned_later } }

/// Whether the instruction is the one [`return_card_from_its_owners_graveyard`] reads.
pub(crate) fn reads_from_its_owners_graveyard(eff: &str) -> bool {
    end(eff).starts_with("return a creature card from its owner's graveyard to the battlefield")
}

fn return_card_from_its_owners_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("return a creature card from its owner's graveyard to the battlefield")?;
    if !b.in_trigger || !matches!(b.it, Sel::TriggerObject | Sel::TriggerLki) {
        return None;
    }
    // "its owner" / "that creature's owner": the trigger object's.
    let owner = PlayerRef::OwnerOf(Box::new(Sel::TriggerLki));
    let controller = match r {
        " under the control of that creature's owner" | " under its owner's control" => {
            owner.clone()
        }
        " under your control" => PlayerRef::You,
        _ => return None,
    };
    let mut to = Destination::battlefield();
    to.controller = Some(controller);
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::creature(),
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedByPlayer(Box::new(owner)),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to,
    })
}

inventory::submit! { EffectPattern { name: "delayed grammar 3: return a creature card from its owner's graveyard", priority: 100, parse: return_card_from_its_owners_graveyard } }

fn if_the_player_hasnt_played_the_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (delay, rest) = l.split_once(", if the player hasn't played the card, ")?;
    if !delay.starts_with("at the beginning of ") {
        return None;
    }
    let instr = match rest {
        "they put it into their graveyard" | "that player puts it into their graveyard" => {
            "put it into its owner's graveyard"
        }
        _ => return None,
    };
    crate::oracle::effects::parse_sentence(
        &format!("{delay}, if you haven't played the card, {instr}"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "delayed grammar 3: if the player hasn't played the card, they put it into their graveyard", priority: 100, parse: if_the_player_hasnt_played_the_card } }

/// Adds the sacrificed card to what the (delayed) return instruction moves.
fn add_sacrificed_card(e: &mut Effect) -> bool {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().any(add_sacrificed_card),
        Effect::DelayedTrigger { body, .. } => add_sacrificed_card(&mut body.effect),
        Effect::Move { what, .. } => {
            let old = std::mem::replace(what, Sel::None);
            *what = Sel::Union(vec![old, Sel::Var(crate::zones::COST_MOVED)]);
            true
        }
        _ => false,
    }
}

fn and_the_sacrificed_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, tail) = l.split_once(" and the sacrificed card ")?;
    if !head.starts_with("return ") {
        return None;
    }
    let mut e = crate::oracle::effects::parse_sentence(&format!("{head} {tail}"), b)?;
    add_sacrificed_card(&mut e).then_some(e)
}

inventory::submit! { EffectPattern { name: "delayed grammar 3: return [object] and the sacrificed card", priority: 100, parse: and_the_sacrificed_card } }

fn this_turn_damage_to_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("whenever ")?;
    let (cond, eff) = r.split_once(" this turn, ")?;
    let subj = cond.strip_suffix(" deals combat damage to that player")?;
    let PlayerRef::Target(_) = b.it_player else {
        return None;
    };
    let chosen = b.it_player.clone();
    let (t, it, _) = crate::oracle::triggers::parse_trigger_condition(&format!(
        "whenever {subj} deals combat damage to a player"
    ))?;
    if !matches!(t, TriggerCond::DealsDamage { .. }) {
        return None;
    }
    let trigger = TriggerCond::Where {
        trigger: Box::new(t),
        cond: Condition::PlayerMatches(
            PlayerRef::TriggerPlayer,
            PlayerFilter::Ref(Box::new(chosen)),
        ),
    };
    let body = crate::oracle::effects::parse_trigger_body(eff, b.ctx, it, PlayerRef::TriggerPlayer)?;
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
        body: Box::new(body),
        once: false,
    })
}

inventory::submit! { EffectPattern { name: "delayed grammar 3: whenever ... deals combat damage to that player this turn", priority: 100, parse: this_turn_damage_to_that_player } }
