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
