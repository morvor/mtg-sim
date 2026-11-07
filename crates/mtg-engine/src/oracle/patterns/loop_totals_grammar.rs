//! Open-ended processes in which the players take turns (CR 608.2c, 101.4b), and what
//! each player did over all of them:
//!
//! ```text
//! pass    := "starting with you, each player may put a" card " from their hand onto the
//!             battlefield"
//!          | "starting with you, each player may pay any amount of life"
//!          | "starting with you, each player chooses one of the exiled cards and puts it
//!             onto the battlefield tapped under their control"   (after an exile)
//! repeat  := "repeat this process until no one puts a card onto the battlefield"
//!          | "repeat this process until no one pays life"
//!          | "repeat this process until all cards exiled this way have been chosen"
//! tokens  := "[then] each player creates " token " for each 1 life they paid this way"
//!          | "[then] each player creates a number of " tokens " equal to the amount of
//!             mana they paid this way"
//!          | "[then] each player creates " token " for each card they revealed this way"
//! mana    := "each player may pay any amount of mana"
//! ```
//!
//! A pass goes around the table once, starting with you; each player performs the
//! instruction as themselves (so a card they put onto the battlefield enters under their
//! control, CR 110.2a), knowing what the players before did. The process repeats after a
//! pass in which someone did it, and ends after a pass in which no one did (Eureka,
//! Plague of Vermin rulings: a player who declined may act on a later pass). A card that
//! can't legally enter (an Aura with nothing to enchant) can't be chosen, so an auction
//! ends when only such cards are left (Thieves' Auction rulings, CR 303.4g). Each
//! player's totals over all the passes are their own ([`crate::tallies`]).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::tallies::{LIFE_PAID, MANA_PAID, SOMEONE_DID};

/// One pass: each player in turn order, starting with you, performs `each` as
/// themselves.
fn pass(each: Effect) -> Effect {
    Effect::Seq(vec![
        Effect::StoreValue {
            var: SOMEONE_DID,
            value: Value::c(0),
        },
        Effect::InTurnOrder {
            first: TurnOrderStart::You,
            who: PlayerFilter::Any,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(each),
            }),
            direction: SeatSide::Left,
        },
    ])
}

/// Whether `e` is a [`pass`].
fn is_pass(e: &Effect) -> bool {
    matches!(e, Effect::Seq(v) if matches!(v.first(), Some(Effect::StoreValue { var: SOMEONE_DID, .. }))
        && matches!(v.get(1), Some(Effect::InTurnOrder { .. })))
}

/// `e`, noting that someone did it if it happened.
fn noted(e: Effect) -> Effect {
    Effect::Seq(vec![
        e,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::StoreValue {
                var: SOMEONE_DID,
                value: Value::c(1),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ])
}

/// "starting with you, each player may put a permanent card from their hand onto the
/// battlefield" (Eureka, Hypergenesis).
fn each_may_put_from_hand(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("starting with you, each player may put ")?;
    let r = r.strip_suffix(" from their hand onto the battlefield")?;
    let p = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(p)?;
    if plural || !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let put = Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                f,
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::You),
            ]),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        to: Destination::battlefield().under_your_control(),
    };
    Some(pass(noted(put)))
}

inventory::submit! { EffectPattern { name: "loop totals: starting with you, each player may put a card from their hand onto the battlefield", priority: 90, parse: each_may_put_from_hand } }

/// "starting with you, each player may pay any amount of life" (Plague of Vermin).
fn each_may_pay_life(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "starting with you, each player may pay any amount of life" {
        return None;
    }
    Some(pass(Effect::Custom(
        crate::kw::loop_totals::PAY_ANY_LIFE_TALLIED.into(),
    )))
}

inventory::submit! { EffectPattern { name: "loop totals: starting with you, each player may pay any amount of life", priority: 90, parse: each_may_pay_life } }

/// The cards an auction is about.
const AUCTIONED: Var = vars::USER + 9618;

/// "Starting with you, each player chooses one of the exiled cards and puts it onto the
/// battlefield tapped under their control." after "Exile all nontoken permanents."
/// (Thieves' Auction): each player must choose one of those cards still in exile that
/// can enter, if any.
fn each_chooses_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l)
        != "starting with you, each player chooses one of the exiled cards and puts it onto the battlefield tapped under their control"
    {
        return false;
    }
    let exiled_all = match prev {
        Effect::Exile { what: Sel::All(_), .. } => true,
        Effect::Seq(v) => matches!(v.last(), Some(Effect::Exile { what: Sel::All(_), .. })),
        _ => false,
    };
    if !exiled_all {
        return false;
    }
    let put = Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::In(Box::new(Sel::Var(AUCTIONED))),
                Filter::InZone(ZoneKind::Exile),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to: Destination::battlefield().tapped().under_your_control(),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Store {
            var: AUCTIONED,
            sel: Sel::Var(vars::IT),
        },
        pass(noted(put)),
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "loop totals: starting with you, each player chooses one of the exiled cards and puts it onto the battlefield", priority: 100, apply: each_chooses_exiled } }

/// "Repeat this process until no one puts a card onto the battlefield." / "... until no
/// one pays life." / "... until all cards exiled this way have been chosen." after a
/// pass: the pass is performed again after each pass in which someone did it.
fn repeat_until_no_one(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "repeat this process until no one puts a card onto the battlefield"
            | "repeat this process until no one pays life"
            | "repeat this process until all cards exiled this way have been chosen"
    ) {
        return false;
    }
    // The pass: the whole of what came before, or its last two instructions.
    let repeat = |body: Effect| Effect::RepeatProcess {
        body: Box::new(Effect::Seq(vec![
            body,
            Effect::If {
                cond: Condition::Compare(Value::Var(SOMEONE_DID), Cmp::Gt, Value::c(0)),
                then: Box::new(Effect::RepeatThisProcess),
                otherwise: Box::new(Effect::Noop),
            },
        ])),
    };
    if is_pass(prev) {
        let body = std::mem::take(prev);
        *prev = repeat(body);
        return true;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let n = v.len();
    if n < 3 {
        return false;
    }
    let tail = Effect::Seq(v[n - 2..].to_vec());
    if !is_pass(&tail) {
        return false;
    }
    v.truncate(n - 2);
    v.push(repeat(tail));
    true
}

inventory::submit! { FollowupPattern { name: "loop totals: repeat this process until no one does it", priority: 100, apply: repeat_until_no_one } }

/// "each player may pay any amount of mana" (Liege of the Hollows): in APNAP order
/// (Liege of the Hollows ruling), each player's amount their own.
fn each_may_pay_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "each player may pay any amount of mana" {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::Custom(crate::kw::join_forces::PAY_ANY_AMOUNT.into())),
        }),
    })
}

inventory::submit! { EffectPattern { name: "loop totals: each player may pay any amount of mana", priority: 90, parse: each_may_pay_mana } }

/// "each player creates [tokens] for each 1 life they paid this way", "... a number of
/// [tokens] equal to the amount of mana they paid this way", "... for each card they
/// revealed this way": each player as many as they themselves paid (or revealed), all
/// in one instruction.
fn each_creates_per_own_total(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (head, count) = if let Some(h) = l.strip_suffix(" for each 1 life they paid this way") {
        (h.to_string(), Value::PlayerTally {
            var: LIFE_PAID,
            who: Box::new(PlayerRef::Iterated),
        })
    } else if let Some(h) = l.strip_suffix(" for each card they revealed this way") {
        if !super::hand_graveyard_grammar::acted(b) {
            return None;
        }
        (h.to_string(), Value::PlayerTally {
            var: crate::tallies::CARDS_ACTED_ON,
            who: Box::new(PlayerRef::Iterated),
        })
    } else {
        let h = l.strip_suffix(" equal to the amount of mana they paid this way")?;
        let h = h.replacen("creates a number of ", "creates a ", 1);
        let h = h.replacen(" tokens", " token", 1);
        (h, Value::PlayerTally {
            var: MANA_PAID,
            who: Box::new(PlayerRef::Iterated),
        })
    };
    if !head.starts_with("each player creates a") {
        return None;
    }
    let e = parse_clause(&head, b)?;
    let token = match e {
        Effect::ForEachPlayer { who: PlayerRef::EachPlayer, effect } => match *effect {
            t @ Effect::CreateToken {
                controller: PlayerRef::Iterated,
                ..
            } => t,
            _ => return None,
        },
        Effect::CreateToken {
            spec,
            count,
            controller: PlayerRef::EachPlayer,
            tapped,
            attacking,
        } => Effect::CreateToken {
            spec,
            count,
            controller: PlayerRef::Iterated,
            tapped,
            attacking,
        },
        _ => return None,
    };
    let Effect::CreateToken {
        spec,
        count: Value::Const(1),
        controller,
        tapped,
        attacking,
    } = token
    else {
        return None;
    };
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::CreateToken {
            spec,
            count,
            controller,
            tapped,
            attacking,
        }),
    })
}

inventory::submit! { EffectPattern { name: "loop totals: each player creates tokens for each life they paid this way", priority: 90, parse: each_creates_per_own_total } }
