//! Hand disruption where another player chooses (CR 701.9b): "Target opponent reveals
//! their hand. You choose a nonland card from it. That player discards that card.", "You
//! choose a card from it with mana value 3 or less.", "You choose a nonland card from it
//! and exile that card."

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;

/// The cards chosen from the revealed hand.
const CHOSEN: Var = vars::USER + 55;

inventory::submit! {
    EffectPattern { name: "card_flow: [player] reveals their hand", priority: 90, parse: reveals_hand }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: you choose a [card] from it", priority: 90, apply: choose_from_it }
}
inventory::submit! {
    EffectPattern { name: "card_flow: that player discards that card", priority: 90, parse: discards_chosen }
}

/// "target opponent reveals their hand", "target player reveals their hand".
fn reveals_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, rest) = player_ref(end(l), b)?;
    if rest.trim() != "reveals their hand" {
        return None;
    }
    // Only a single player whose hand "it" can name.
    rel_of(&who)?;
    b.it_player = who.clone();
    Some(Effect::RevealHand { who })
}

fn rel_of(p: &PlayerRef) -> Option<PlayerRel> {
    Some(match p {
        PlayerRef::Target(s) => PlayerRel::Target(*s),
        PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
        PlayerRef::DefendingPlayer => PlayerRel::Defending,
        _ => return None,
    })
}

fn revealed_hand_owner(e: &Effect) -> Option<PlayerRef> {
    match e {
        Effect::RevealHand { who } => Some(who.clone()),
        Effect::Seq(v) => v.last().and_then(revealed_hand_owner),
        _ => None,
    }
}

/// "you choose a nonland card from it", "you may choose a creature card from it", "you
/// choose a card from it with mana value 3 or greater", "... from it and exile that card".
fn choose_from_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (up_to, r) = if let Some(r) = l.strip_prefix("you may choose ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("you choose ") {
        (false, r)
    } else {
        return false;
    };
    let Some((n, r)) = parse_number(r) else {
        return false;
    };
    if n.as_const().is_none() {
        return false;
    }
    let Some((desc, after)) = r.split_once(" from it") else {
        return false;
    };
    let (suffix, exile) = match after.strip_suffix(" and exile that card") {
        Some(s) => (s, true),
        None => (after, false),
    };
    // "from it with mana value 3 or less": the description continues after "from it".
    let desc = format!("{}{}", desc.trim(), suffix);
    let Some(filter) = card_filter(&desc, b) else {
        return false;
    };
    let Some(owner) = revealed_hand_owner(prev) else {
        return false;
    };
    let Some(rel) = rel_of(&owner) else {
        return false;
    };
    let choose = Effect::Store {
        var: CHOSEN,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                filter,
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(rel),
            ]),
            count: n,
            up_to,
            store: None,
        },
    };
    let mut v = vec![std::mem::take(prev), choose];
    if exile {
        v.push(Effect::Exile {
            what: Sel::Var(CHOSEN),
            face_down: false,
            link: false,
        });
    }
    *prev = Effect::seq(v);
    b.it = Sel::Var(CHOSEN);
    b.it_player = owner;
    true
}

/// "that player discards that card" / "... those cards" after a card was chosen from
/// their revealed hand: the chosen cards are discarded (CR 701.9b).
fn discards_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::Var(CHOSEN)) {
        return None;
    }
    let (who, rest) = player_ref(end(l), b)?;
    if !matches!(rest.trim(), "discards that card" | "discards those cards") {
        return None;
    }
    Some(Effect::Discard {
        who,
        n: Value::CountSel(Box::new(Sel::Var(CHOSEN))),
        random: false,
        filter: Filter::In(Box::new(Sel::Var(CHOSEN))),
    })
}
