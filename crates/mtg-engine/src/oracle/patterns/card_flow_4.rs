//! Card flow (cards3-cast-zones): "You may exile ~ and four cards named ~ from your
//! graveyard. If you do, ..." (Dragon's Approach, Sphinx's Approach): the resolving spell
//! (CR 608.2: it's still on the stack) and that many cards with its name. The choice can
//! be made only if all of them can be exiled (a copy of the spell, which isn't a card, can
//! be exiled too and ceases to exist, CR 707.10a, 704.5e).
//!
//! "Exile target nonland permanent and the top card of your library." (Suspend
//! Aggression): both at once; "those cards" are both.
//!
//! "Look at target opponent's hand and exile those cards face down." (Apple of Eden, Isu
//! Relic): all the cards in that hand, which the player who exiled them has seen.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};

fn exile_self_and_named(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    // ("You may" is usually read first: then the choice is offered around this.)
    let (may, l) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let r = l.strip_prefix("exile ~ and ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    if n < 1 || !b.ctx.is_spell() {
        return None;
    }
    if r.trim() != "cards named ~ from your graveyard" {
        return None;
    }
    let named = Filter::and(vec![
        Filter::Card,
        Filter::Named(b.ctx.card_name.into()),
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    let exile = Effect::Exile {
        what: Sel::Union(vec![
            Sel::This,
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: named.clone(),
                count: Value::c(n),
                up_to: false,
                store: None,
            },
        ]),
        face_down: false,
        link: false,
    };
    // Only if there are that many such cards (and the spell is still on the stack): an
    // instruction that can't be done wasn't (CR 608.2c, "If you do").
    let possible = Condition::And(vec![
        Condition::Compare(
            Value::CountSel(Box::new(Sel::All(named))),
            Cmp::Ge,
            Value::c(n),
        ),
        Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Stack)),
    ]);
    Some(Effect::If {
        cond: possible,
        then: Box::new(if may {
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(exile),
            }
        } else {
            exile
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "card flow 4: you may exile ~ and N cards named ~ from your graveyard", priority: 90, parse: exile_self_and_named } }

fn exile_target_and_top_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("exile ")?;
    let target = r.strip_suffix(" and the top card of your library")?;
    if !target.starts_with("target ") {
        return None;
    }
    let n0 = b.targets.len();
    let (sel, rest) = crate::oracle::effects::object_ref(target, b)?;
    if !rest.trim().is_empty() || b.targets.len() != n0 + 1 || !matches!(sel, Sel::Target(_)) {
        b.targets.truncate(n0);
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what: Sel::Union(vec![sel, Sel::TopOfLibrary(PlayerRef::You, Value::c(1))]),
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "card flow 4: exile target [permanent] and the top card of your library", priority: 90, parse: exile_target_and_top_card } }

fn look_at_hand_and_exile_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("look at ")?;
    let who = r.strip_suffix("'s hand and exile those cards face down")?;
    if !matches!(who, "target opponent" | "target player") {
        return None;
    }
    crate::oracle::effects::parse_sentence(
        &format!("exile all cards from {who}'s hand face down"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "card flow 4: look at target opponent's hand and exile those cards face down", priority: 90, parse: look_at_hand_and_exile_face_down } }

/// Adds `extra` to what the move moves (`Move { what: All(...) }`).
fn narrow_move(e: &mut Effect, extra: Filter) -> bool {
    match e {
        Effect::Move {
            what: Sel::All(f), ..
        } => {
            *f = Filter::and(vec![f.clone(), extra]);
            true
        }
        _ => false,
    }
}

/// "When that token leaves the battlefield, return all cards exiled with ~ except ~ to the
/// battlefield tapped under their owners' control." (Mechtitan Core): the cards its
/// abilities exiled (CR 607.2a) other than the card that was ~ (it exiled itself to pay
/// the cost). "Exile it, then return all other permanent cards exiled with ~ to the
/// battlefield ..." (Mirror of Life Trapping): other than the card just exiled.
fn return_exiled_with_except(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (text, extra) = if let Some((head, tail)) = l.split_once(" exiled with ~ except ~ ") {
        if !head.starts_with("return all ") {
            return None;
        }
        (
            format!("{head} exiled with ~ {tail}"),
            Filter::not(Filter::Custom(
                crate::kw::hand_graveyard_actions::SOURCE_OR_NEXT.into(),
            )),
        )
    } else if let Some(r) = l.strip_prefix("return all other ") {
        if !r.contains(" exiled with ~ ") || matches!(b.it, Sel::This) {
            return None;
        }
        (
            format!("return all {r}"),
            Filter::not(Filter::In(Box::new(Sel::Var(vars::IT)))),
        )
    } else {
        return None;
    };
    let mut e = crate::oracle::effects::parse_clause(&text, b)?;
    narrow_move(&mut e, extra).then_some(e)
}

inventory::submit! { EffectPattern { name: "card flow 4: return all cards exiled with ~ except ~ / all other cards exiled with ~", priority: 90, parse: return_exiled_with_except } }

/// "You may have target player reveal their hand. If you do, choose a nonland card from it
/// and exile that card." (Thought Prison): the choice, from the hand revealed, if it was
/// (see `card_flow_reveal_hand`).
fn f_if_you_do_choose_from_revealed_hand(
    l: &str,
    prev: &mut Effect,
    b: &mut crate::oracle::effects::Builder,
) -> bool {
    let Some(r) = end(l.trim()).strip_prefix("if you do, choose ") else {
        return false;
    };
    let Effect::May { effect, .. } = prev else {
        return false;
    };
    let mut inner = (**effect).clone();
    if !super::card_flow_reveal_hand::choose_from_it(&format!("you choose {r}"), &mut inner, b) {
        return false;
    }
    **effect = inner;
    true
}

inventory::submit! { super::FollowupPattern { name: "card flow 4: if you do, choose a card from the revealed hand", priority: 90, apply: f_if_you_do_choose_from_revealed_hand } }

/// "Return two cards exiled with ~ to the battlefield under their owners' control and put
/// the rest on the bottom of their owners' libraries." (Vault 13: Dweller's Journey): the
/// cards exiled with ~ (CR 607.2a) still in exile after the first part are the rest.
fn return_some_exiled_with_and_the_rest(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let head = l.strip_suffix(" and put the rest on the bottom of their owners' libraries")?;
    if !head.starts_with("return ") || !head.contains(" cards exiled with ~ ") {
        return None;
    }
    let first = crate::oracle::effects::parse_clause(head, b)?;
    if !matches!(first, Effect::Move { .. }) {
        return None;
    }
    let mut to = Destination::zone(ZoneKind::Library);
    to.position = LibraryPosition::Bottom;
    Some(Effect::seq(vec![
        first,
        Effect::Move {
            what: Sel::All(Filter::and(vec![
                Filter::In(Box::new(Sel::Linked)),
                Filter::InZone(ZoneKind::Exile),
            ])),
            to,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "card flow 4: return N cards exiled with ~ ... and put the rest on the bottom", priority: 90, parse: return_some_exiled_with_and_the_rest } }

/// The cards an earlier instruction exiled, for "the rest" after "Target opponent may
/// choose one of the exiled cards and put it onto the battlefield under their control."
const EXILED_SET: Var = vars::USER + 9480;

/// "Target opponent may choose one of the exiled cards and put it onto the battlefield
/// under their control." (Dubious Challenge): that player chooses (CR 608.2c), and the
/// chosen card is put onto the battlefield under their control.
fn f_opponent_chooses_one_of_the_exiled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some(r) = l.strip_prefix("target opponent may choose one of the exiled cards and put it ")
    else {
        return false;
    };
    if r != "onto the battlefield under their control" {
        return false;
    }
    // After exiling cards (perhaps chosen from among cards looked at, then shuffling).
    fn exiled_last(e: &Effect) -> bool {
        match e {
            Effect::Exile { .. } => true,
            Effect::DigStep(s) => {
                matches!(&**s, DigStep::Take { to, .. } if to.zone == ZoneKind::Exile)
            }
            Effect::Seq(v) => v
                .iter()
                .rev()
                .find(|x| !matches!(x, Effect::Shuffle { .. }))
                .is_some_and(exiled_last),
            _ => false,
        }
    }
    if !exiled_last(prev) {
        return false;
    }
    let Some((spec, rest)) = crate::oracle::phrases::parse_target("target opponent") else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let slot = b.add_target(spec, "target opponent");
    let chooser = PlayerRef::Target(slot);
    let exiled = Sel::All(Filter::and(vec![
        Filter::In(Box::new(Sel::Var(EXILED_SET))),
        Filter::InZone(ZoneKind::Exile),
    ]));
    let mut to = Destination::zone(ZoneKind::Battlefield);
    to.controller = Some(chooser.clone());
    let choose = Effect::Move {
        what: Sel::Choose {
            chooser,
            filter: Filter::and(vec![
                Filter::In(Box::new(Sel::Var(EXILED_SET))),
                Filter::InZone(ZoneKind::Exile),
            ]),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        to,
    };
    b.named.push((super::r406_exile_until::THE_REST.into(), exiled));
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::Store {
            var: EXILED_SET,
            sel: Sel::Var(vars::IT),
        },
        choose,
    ]);
    true
}

/// "Put the rest onto the battlefield under your control." after the above.
fn put_the_rest_onto_the_battlefield(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l.trim()) != "put the rest onto the battlefield under your control" {
        return None;
    }
    let (_, rest) = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == super::r406_exile_until::THE_REST)?;
    let rest = rest.clone();
    if !format!("{rest:?}").contains(&format!("Var({EXILED_SET})")) {
        return None;
    }
    Some(Effect::Move {
        what: rest,
        to: Destination::battlefield().under_your_control(),
    })
}

inventory::submit! { super::FollowupPattern { name: "card flow 4: target opponent may choose one of the exiled cards and put it onto the battlefield", priority: 90, apply: f_opponent_chooses_one_of_the_exiled } }
inventory::submit! { EffectPattern { name: "card flow 4: put the rest onto the battlefield under your control", priority: 90, parse: put_the_rest_onto_the_battlefield } }
