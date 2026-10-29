//! Choice grammar VI (cards5-choices-players): objects chosen by other players.
//!
//! - "You choose target creature an opponent controls, and that opponent chooses target
//!   creature." (Mogg Assassin): two targets, the second chosen by the controller of the
//!   first (CR 601.2c, 115.1); "the creature you chose" and "the creature your opponent
//!   chose" are the two targets afterward.
//! - "The owner of each creature card revealed this way with the lowest mana value puts
//!   it onto the battlefield." (Stronghold Gambit, after "Each player chooses a card in
//!   their hand. Then each player reveals their chosen card."): the revealed cards of
//!   that kind with the least mana value among them (tied or not) enter under their
//!   owners' control (CR 110.2a); none if no such card was revealed.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "you choose target [object], and that opponent chooses target [object]".
fn you_and_that_opponent_choose_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you choose target ")
        .or_else(|| l.strip_prefix("choose target "))?;
    let (first, second) = r.split_once(", and that opponent chooses target ")?;
    // The first target is another player's object: "that opponent" is its controller.
    if !first.ends_with(" an opponent controls") {
        return None;
    }
    let t1 = format!("target {first}");
    let (s1, tail) = parse_target(&t1)?;
    if !end(tail).is_empty() || !matches!(s1.what, TargetKind::Object(_)) {
        return None;
    }
    let t2 = format!("target {second}");
    let (mut s2, tail) = parse_target(&t2)?;
    if !end(tail).is_empty() || !matches!(s2.what, TargetKind::Object(_)) {
        return None;
    }
    let first_slot = b.add_target(s1, &t1);
    s2.chosen_by = Some(PlayerRef::ControllerOf(Box::new(Sel::Target(first_slot))));
    let second_slot = b.add_target(s2, &t2);
    let noun = |t: &str| t.split_whitespace().nth(1).unwrap_or("permanent").to_string();
    let (n1, n2) = (noun(&t1), noun(&t2));
    b.named
        .push((format!("the {n1} you chose"), Sel::Target(first_slot)));
    b.named.push((
        format!("the {n2} your opponent chose"),
        Sel::Target(second_slot),
    ));
    b.named.push((
        format!("the {n2} that opponent chose"),
        Sel::Target(second_slot),
    ));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "you choose target [object], and that opponent chooses target [object]", priority: 90, parse: you_and_that_opponent_choose_targets } }

/// "the owner of each [card] revealed this way [with the lowest/greatest mana value] puts
/// it onto the battlefield", after cards were chosen and revealed.
fn owners_put_revealed(l: &str, b: &mut Builder) -> Option<Effect> {
    use super::filters_relational::{stat_word, tested};
    let l = end(l);
    let r = l
        .strip_prefix("the owner of each ")?
        .strip_suffix(" puts it onto the battlefield")?;
    let (noun, quality) = r.split_once(" revealed this way")?;
    let revealed = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == "the chosen card")
        .map(|(_, s)| s.clone())?;
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !end(tail).is_empty() {
        return None;
    }
    let group = Sel::Matching(Box::new(revealed), f);
    let what = match quality.trim() {
        "" => group,
        q => {
            let q = q.strip_prefix("with the ")?;
            let (greatest, q) = if let Some(x) = q.strip_prefix("lowest ") {
                (false, x)
            } else {
                (true, q.strip_prefix("greatest ")?)
            };
            let (stat, rest) = stat_word(q)?;
            if !rest.trim().is_empty() {
                return None;
            }
            let ext = Value::Extreme(Box::new(tested(stat)), Box::new(group.clone()), greatest);
            let cmp = if greatest { Cmp::Ge } else { Cmp::Le };
            Sel::Matching(
                Box::new(group),
                Filter::ValueCmp(Box::new(tested(stat)), cmp, Box::new(ext)),
            )
        }
    };
    // Each card under its own owner's control.
    let mut to = Destination::battlefield();
    to.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
    Some(Effect::Move { what, to })
}

inventory::submit! { EffectPattern { name: "the owner of each [card] revealed this way [with the lowest mana value] puts it onto the battlefield", priority: 90, parse: owners_put_revealed } }

/// "Then each player reveals their chosen card." (Stronghold Gambit): each player reveals
/// the card they chose (read for each player as "reveal your chosen card").
fn reveal_their_chosen_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("reveal ")
        .or_else(|| l.strip_prefix("that player reveals "))?;
    let word = r
        .strip_prefix("your chosen ")
        .or_else(|| r.strip_prefix("their chosen "))?;
    if word.contains(' ') {
        return None;
    }
    let theirs = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| *n == format!("their chosen {word}"))
        .map(|(_, s)| s.clone())?;
    crate::kw::zones_dig_effects::reveal_effect(&theirs)
}

inventory::submit! { EffectPattern { name: "each player reveals their chosen card", priority: 90, parse: reveal_their_chosen_card } }
