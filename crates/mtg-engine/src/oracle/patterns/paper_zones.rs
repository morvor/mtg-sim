//! Zone phrases of older Oracle wordings:
//!
//! - the card's name for the card in its new zone: "When ~ is put into your graveyard from
//!   the battlefield, you may shuffle ~ into your library." (Guan Yu, Sainted Warrior: the
//!   trigger follows the card to the graveyard, CR 400.7, 603.10a; if it has left the
//!   graveyard, nothing is shuffled, its ruling);
//! - graveyard order as an object: "Put the top card of your graveyard on the bottom of
//!   your library." (Soldevi Digger; the engine keeps graveyard order, CR 404.2);
//! - "from it" after revealing a hand: "you may reveal your hand and put all land cards
//!   from it onto the battlefield. If you do, discard your hand." (Manabond: "if you do"
//!   is whether you revealed, and the cards still in your hand are discarded, its
//!   rulings).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "shuffle ~ into your library" in an ability whose "it" is the source: as "shuffle it".
fn shuffle_self(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("shuffle ~ into ")?;
    if !matches!(b.it, Sel::This) || !b.in_trigger {
        return None;
    }
    parse_clause(&format!("shuffle it into {r}"), b)
}

inventory::submit! { EffectPattern { name: "shuffle ~ into your library (the card in its new zone)", priority: 300, parse: shuffle_self } }

/// "[put/exile/return] the top card of your graveyard [destination]": the card on top of
/// your graveyard as the instruction is performed.
fn top_of_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    const PHRASE: &str = "the top card of your graveyard";
    let i = l.find(PHRASE)?;
    let head = &l[..i];
    if !matches!(head, "put " | "exile " | "return ") {
        return None;
    }
    let text = format!("{head}target card from your graveyard{}", &l[i + PHRASE.len()..]);
    let first = b.targets.len();
    let e = parse_clause(&text, b)?;
    if b.targets.len() != first + 1 {
        b.targets.truncate(first);
        return None;
    }
    b.targets.truncate(first);
    let top = Sel::TopOfGraveyard(PlayerRef::You);
    match e {
        Effect::Move {
            what: Sel::Target(s),
            to,
        } if s as usize == first => Some(Effect::Move { what: top, to }),
        Effect::Exile {
            what: Sel::Target(s),
            face_down,
            link,
        } if s as usize == first => Some(Effect::Exile {
            what: top,
            face_down,
            link,
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "the top card of your graveyard (object)", priority: 300, parse: top_of_graveyard } }

/// "reveal your hand and put [cards] from it [destination]": the cards from your hand.
fn reveal_hand_put_from_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("reveal your hand and put ")?;
    let (objs, dest) = r.split_once(" from it ")?;
    let reveal = Effect::RevealHand {
        who: PlayerRef::You,
    };
    let put = parse_clause(&format!("put {objs} from your hand {dest}"), b)?;
    if !matches!(put, Effect::Move { .. }) {
        return None;
    }
    Some(Effect::Seq(vec![reveal, put]))
}

inventory::submit! { EffectPattern { name: "reveal your hand and put [cards] from it", priority: 300, parse: reveal_hand_put_from_it } }

/// "If you do, [instruction]" after "you may reveal your hand and put [cards] from it
/// ...": whether you chose to reveal (not whether any card was put), so the instruction is
/// part of the optional one.
fn if_you_do_after_reveal(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you do, ") else {
        return false;
    };
    let Effect::May { effect, .. } = prev else {
        return false;
    };
    let Effect::Seq(v) = &mut **effect else {
        return false;
    };
    if !matches!(
        v.as_slice(),
        [Effect::RevealHand { who: PlayerRef::You }, Effect::Move { .. }]
    ) {
        return false;
    }
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    v.push(e);
    true
}

inventory::submit! { FollowupPattern { name: "if you do, after you may reveal your hand and put [cards] from it", priority: 50, apply: if_you_do_after_reveal } }
