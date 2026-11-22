//! Oracle patterns for the ante cards that ante an object from its current zone or change
//! owners (CR 407.3, 407.4):
//!
//! - "Ante ~. If you do, put all other cards you own from the ante into your graveyard,
//!   then draw a card." (Jeweled Bird)
//! - "The owner of target artifact may ante the top card of their library. If that player
//!   doesn't, exchange ownership of that artifact and ~. Put the artifact card into your
//!   graveyard and ~ from anywhere into that player's graveyard. This change in ownership
//!   is permanent." (Timmerian Fiends), and Tempest Efreet's "Exchange ownership of the
//!   revealed card and ~. Put the revealed card into your hand and ~ from anywhere into
//!   that player's graveyard."
//! - "Exile ~ and target nontoken permanent an opponent owns. That player may pay 10 life.
//!   If they do, put ~ into its owner's graveyard. Otherwise, that player owns ~ and you
//!   own the other exiled card." (Bronze Tablet)
//! - "Each player may ante the top card of their library. If a player does, that player's
//!   life total becomes 20." (Rebirth)
//!
//! The behaviors are `Effect::Custom`s implemented in `crate::ante`.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::ante::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

fn custom(name: &str) -> Effect {
    Effect::Custom(name.into())
}

/// Whether `e` contains the custom effect `name`.
fn contains_custom(e: &Effect, name: &str) -> bool {
    format!("{e:?}").contains(&format!("Custom({name:?})"))
}

/// "ante ~" (CR 407.4: from whichever zone it's in; only its owner can).
fn ante_this(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "ante ~").then(|| custom(ANTE_THIS))
}

inventory::submit! { EffectPattern { name: "r407 ante ~", priority: 100, parse: ante_this } }

/// "put all other cards you own from the ante into your graveyard".
fn other_ante_cards(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "put all other cards you own from the ante into your graveyard")
        .then(|| custom(OTHER_ANTE_CARDS_TO_GRAVEYARD))
}

inventory::submit! { EffectPattern { name: "r407 other ante cards to graveyard", priority: 100, parse: other_ante_cards } }

/// "the owner of target artifact may ante the top card of their library": that player is
/// "that player" of the following sentences, and the artifact "that artifact".
fn owner_of_target_may_ante(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the owner of ")?;
    let (spec, tail) = parse_target(r)?;
    if end(tail).trim() != "may ante the top card of their library"
        || !matches!(spec.what, TargetKind::Object(_))
    {
        return None;
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    let owner = PlayerRef::OwnerOf(Box::new(Sel::Target(slot)));
    b.it = Sel::Target(slot);
    b.it_player = owner.clone();
    Some(Effect::AsPlayer {
        who: owner,
        effect: Box::new(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(custom(ANTE_TOP)),
        }),
    })
}

inventory::submit! { EffectPattern { name: "r407 the owner of target object may ante", priority: 100, parse: owner_of_target_may_ante } }

/// "exchange ownership of the revealed card and ~" (the card a previous instruction
/// revealed) / "exchange ownership of that artifact and ~" (a target).
fn exchange_ownership(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange ownership of ")?;
    let what = r.strip_suffix(" and ~")?;
    match what {
        "the revealed card" => Some(custom(EXCHANGE_OWNERSHIP)),
        _ if what.starts_with("that ") => {
            let Sel::Target(slot) = b.it else {
                return None;
            };
            Some(Effect::Seq(vec![
                Effect::Store {
                    var: vars::IT,
                    sel: Sel::Target(slot),
                },
                custom(EXCHANGE_OWNERSHIP),
            ]))
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r407 exchange ownership", priority: 100, parse: exchange_ownership } }

/// "put the revealed card into your hand and ~ from anywhere into that player's
/// graveyard", "put the artifact card into your graveyard and ~ from anywhere into that
/// player's graveyard": the object whose ownership was exchanged, and the source.
fn put_exchanged(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put the ")?;
    let (_, r) = r.split_once(" card into your ")?;
    let (to, rest) = r.split_once(' ')?;
    if rest != "and ~ from anywhere into that player's graveyard" {
        return None;
    }
    match to {
        "hand" | "graveyard" => Some(custom(&format!("{PUT_EXCHANGED}{to}"))),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r407 put the exchanged cards", priority: 100, parse: put_exchanged } }

/// "This change in ownership is permanent." after an exchange of ownership (and the
/// instruction putting the exchanged cards into their new zones): it lasts
/// beyond the game (CR 407.3), which the game itself doesn't see.
fn permanent_change(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    end(&s.to_lowercase()) == "this change in ownership is permanent"
        && (contains_custom(prev, EXCHANGE_OWNERSHIP) || format!("{prev:?}").contains(PUT_EXCHANGED))
}

inventory::submit! { FollowupPattern { name: "r407 this change in ownership is permanent", priority: 100, apply: permanent_change } }

/// The target slot of "Exile ~ and target [permanent]" (`Exile { Union([This,
/// Target(slot)]) }`).
fn exiled_with_this(e: &Effect) -> Option<u8> {
    match e {
        Effect::Exile {
            what: Sel::Union(v),
            ..
        } => match v[..] {
            [Sel::This, Sel::Target(slot)] => Some(slot),
            _ => None,
        },
        _ => None,
    }
}

/// "That player may pay N life." after "Exile ~ and target nontoken permanent an
/// opponent owns.": that player is the exiled permanent's owner, who is "that player" of
/// the rest of the text.
fn that_player_may_pay_life(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    let Some(n) = end(&l)
        .strip_prefix("that player may pay ")
        .and_then(|r| r.strip_suffix(" life"))
        .and_then(|n| n.parse::<i32>().ok())
    else {
        return false;
    };
    let Some(slot) = exiled_with_this(prev) else {
        return false;
    };
    let owner = PlayerRef::OwnerOf(Box::new(Sel::Target(slot)));
    b.it_player = owner.clone();
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::Seq(vec![
        p,
        Effect::AsPlayer {
            who: owner,
            effect: Box::new(Effect::PayOptional {
                who: PlayerRef::You,
                cost: Cost {
                    mana: None,
                    parts: vec![CostPart::PayLife(Value::c(n))],
                },
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::Noop),
            }),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "r407 that player (the exiled card's owner) may pay life", priority: 100, apply: that_player_may_pay_life } }

/// "If they do, put ~ into its owner's graveyard." after "Exile ~ and target ... That
/// player may pay N life.": the exiled source.
fn exiled_this_to_graveyard(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    let Some(r) = end(&l).strip_prefix("if they do, ") else {
        return false;
    };
    if r != "put ~ into its owner's graveyard"
        || !matches!(b.it_player, PlayerRef::OwnerOf(_))
        || !matches!(prev, Effect::Seq(v) if v.first().and_then(exiled_with_this).is_some()
            && matches!(v.last(), Some(Effect::AsPlayer { effect, .. })
                if matches!(**effect, Effect::PayOptional { .. })))
    {
        return false;
    }
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        p,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(custom(EXILED_THIS_TO_GRAVEYARD)),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "r407 if they do, put the exiled ~ into its owner's graveyard", priority: 100, apply: exiled_this_to_graveyard } }

/// The last `If { cond: PrevHappened, otherwise: Noop }` of `e`.
fn last_if_did(e: &mut Effect) -> Option<&mut Box<Effect>> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_if_did),
        Effect::If {
            cond: Condition::PrevHappened,
            otherwise,
            ..
        } if matches!(**otherwise, Effect::Noop) => Some(otherwise),
        _ => None,
    }
}

/// "Otherwise, that player owns ~ and you own the other exiled card."
fn otherwise_owners(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if end(&l) != "otherwise, that player owns ~ and you own the other exiled card"
        || !contains_custom(prev, EXILED_THIS_TO_GRAVEYARD)
    {
        return false;
    }
    let Some(otherwise) = last_if_did(prev) else {
        return false;
    };
    **otherwise = custom(GIVE_THIS_FOR_IT);
    true
}

inventory::submit! { FollowupPattern { name: "r407 otherwise, that player owns ~ and you own the other card", priority: 100, apply: otherwise_owners } }

/// "If a player does, that player's life total becomes N." after "Each player may ante
/// the top card of their library": for each player who anted.
fn if_a_player_does(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    let Some(r) = end(&l).strip_prefix("if a player does, that player's ") else {
        return false;
    };
    let Effect::ForEachPlayer { effect, .. } = prev else {
        return false;
    };
    if !matches!(**effect, Effect::May { .. }) || !contains_custom(effect, ANTE_TOP) {
        return false;
    }
    let Some(then) = parse_clause(&format!("your {r}"), b) else {
        return false;
    };
    let may = std::mem::replace(&mut **effect, Effect::Noop);
    **effect = Effect::seq(vec![
        may,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(then),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "r407 if a player does (each player may ante)", priority: 100, apply: if_a_player_does } }
