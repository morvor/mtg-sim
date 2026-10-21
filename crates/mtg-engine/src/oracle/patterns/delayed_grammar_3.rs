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
//!
//! "Exile target instant or sorcery card from your graveyard. [...] Whenever a creature you
//! control deals combat damage to a player this turn, copy the exiled card. You may cast
//! the copy without paying its mana cost." (Surge to Victory; CR 603.7c, 707.12): the
//! card the first instruction exiled, captured as it's exiled.
//!
//! "When ~ dies, exile it and choose target creature an opponent controls. When that
//! creature leaves the battlefield, return ~ from exile to the battlefield under its
//! owner's control." (Lucius the Eternal; CR 603.7c, 400.7): "~ from exile" is the card
//! the dies ability exiled (the source's card, a new object), captured as the delayed
//! ability is created.

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

/// The card exiled by the creating spell's first instruction (see
/// [`copy_the_exiled_card_this_turn`]).
const EXILED_CARD: Var = vars::USER + 13_102;

/// Whether the ability's first instruction exiled its target card ("Exile target instant
/// or sorcery card from your graveyard."): its first target is a card in a graveyard.
fn first_target_is_a_graveyard_card(b: &Builder) -> bool {
    b.targets
        .first()
        .is_some_and(|t| t.text.starts_with("target ") && t.text.contains(" card from "))
}

fn copy_the_exiled_card_this_turn(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let s = end(s);
    let Some(r) = s.strip_prefix("whenever ") else {
        return false;
    };
    let Some((cond, eff)) = r.split_once(" this turn, ") else {
        return false;
    };
    if eff != "copy the exiled card" || !first_target_is_a_graveyard_card(b) {
        return false;
    }
    let Some((t, _, _)) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever {cond}"))
    else {
        return false;
    };
    // A copy of the card in exile (CR 707.12), recorded as "the copy".
    let body = Body {
        targets: vec![],
        effect: Effect::CopyCard {
            what: Sel::Var(EXILED_CARD),
            named: None,
        },
        modal: None,
    };
    // The exiled card: what the exile instruction moved ("it"; the instructions in
    // between don't change it).
    let note = Effect::Store {
        var: EXILED_CARD,
        sel: Sel::Var(vars::IT),
    };
    let delayed = Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(t)),
        body: Box::new(body),
        once: false,
    };
    let old = std::mem::take(prev);
    *prev = Effect::Seq(vec![old, note, delayed]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar 3: whenever ... this turn, copy the exiled card", priority: 40, apply: copy_the_exiled_card_this_turn } }

/// "You may cast the copy without paying its mana cost." after that trigger: part of its
/// effect.
fn cast_the_copy_after_this_turn_copy(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let s = end(s);
    if !s.starts_with("you may cast the copy") {
        return false;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(Effect::DelayedTrigger { body, once: false, .. }) = v.last_mut() else {
        return false;
    };
    let copies_exiled = serde_json::to_string(&body.effect)
        .is_ok_and(|j| j.contains(&format!("{{\"Var\":{EXILED_CARD}}}")));
    if !copies_exiled {
        return false;
    }
    let mut b2 = Builder::new(b.ctx);
    let Some(cast) = crate::oracle::effects::parse_sentence(s, &mut b2) else {
        return false;
    };
    if !b2.targets.is_empty() {
        return false;
    }
    let first = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![first, cast]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar 3: you may cast the copy (of the exiled card, this turn)", priority: 40, apply: cast_the_copy_after_this_turn_copy } }

/// The card "exile it" exiled (see [`return_self_from_exile_when_that_creature_leaves`]).
const EXILED_SELF: Var = vars::USER + 13_103;

fn return_self_from_exile_when_that_creature_leaves(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (cond, eff) = l.split_once(", ")?;
    if eff != "return ~ from exile to the battlefield under its owner's control"
        || !cond.starts_with("when that creature ")
    {
        return None;
    }
    // The ability exiled its source's card first ("When ~ dies, exile it and ...").
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("dies, exile it and ") {
        return None;
    }
    let mut e = super::delayed_grammar::referent_trigger(&format!("{cond}, draw a card"), b)?;
    fn patch(e: &mut Effect) -> bool {
        match e {
            Effect::Seq(v) => v.iter_mut().any(patch),
            Effect::DelayedTrigger { body, .. } => {
                let mut to = Destination::battlefield();
                to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(EXILED_SELF))));
                body.effect = Effect::Move {
                    what: Sel::Var(EXILED_SELF),
                    to,
                };
                true
            }
            _ => false,
        }
    }
    if !patch(&mut e) {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Store {
            var: EXILED_SELF,
            sel: Sel::Var(vars::IT),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "delayed grammar 3: when that creature leaves, return ~ from exile", priority: 50, parse: return_self_from_exile_when_that_creature_leaves } }
