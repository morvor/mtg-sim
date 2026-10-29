//! Player verbs and filters of older Oracle wordings:
//!
//! - a player as the subject of "destroy", choosing what to destroy among what they
//!   control: "You destroy four lands you control, then target opponent destroys four
//!   lands they control." (Burning of Xinye: chosen as the spell resolves; a regeneration
//!   shield applies, unlike a sacrifice, and a permanent with indestructible can't be
//!   chosen, its rulings);
//! - the player who didn't pay as the subject of a follow-up: "Counter target spell
//!   unless its controller pays {X}. If that player doesn't, they tap all lands with mana
//!   abilities they control and lose all unspent mana." (Power Sink);
//! - "you both own and control" / "you own and control" as a target qualifier (Obelisk of
//!   Undoing).

use super::{FilterSuffixPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;

/// The relation for "[objects] they control" after a player subject.
fn rel_of(p: &PlayerRef) -> Option<PlayerRel> {
    match p {
        PlayerRef::You => Some(PlayerRel::You),
        PlayerRef::Target(n) => Some(PlayerRel::Target(*n)),
        _ => None,
    }
}

/// "you destroy four lands you control", "target opponent destroys four lands they
/// control": that player chooses that many of them as the instruction is performed.
fn player_destroys(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r, poss) = if let Some(r) = l.strip_prefix("you destroy ") {
        (PlayerRef::You, r.to_string(), "you")
    } else {
        let saved = b.targets.len();
        let (who, rest) = player_ref(l, b)?;
        let Some(r) = rest.trim_start().strip_prefix("destroys ") else {
            b.targets.truncate(saved);
            return None;
        };
        if matches!(who, PlayerRef::You) {
            b.targets.truncate(saved);
            return None;
        }
        (who, r.trim_end().to_string(), "they")
    };
    let rel = rel_of(&who)?;
    let r = r.strip_suffix(&format!(" {poss} control"))?;
    let (n, objs) = parse_number(r)?;
    let Value::Const(1..) = n else {
        return None;
    };
    let (f, plural, tail) = parse_object_phrase(objs.trim_start())?;
    if !end(tail).is_empty() || (plural != !matches!(n, Value::Const(1))) {
        return None;
    }
    Some(Effect::Destroy {
        what: Sel::Choose {
            chooser: who,
            filter: Filter::and(vec![
                f,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::ControlledBy(rel),
                // A player can't choose to destroy a permanent with indestructible
                // (Burning of Xinye's ruling; interpretation `destroy-choice-indestructible`).
                Filter::not(Filter::HasKeyword(crate::keywords::KeywordKind::Indestructible)),
            ]),
            count: n,
            up_to: false,
            store: None,
        },
        no_regen: false,
    })
}

inventory::submit! { super::EffectPattern { name: "[player] destroys N [permanents] they control", priority: 300, parse: player_destroys } }

/// Whether the last instruction of an effect is "[instruction] unless [player] pays
/// [cost]", which records whether that player paid.
fn ends_with_unless_paid(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_with_unless_paid),
        Effect::PayOptional { who, then, .. } => {
            matches!(**then, Effect::Noop) && !matches!(who, PlayerRef::You)
        }
        _ => false,
    }
}

/// "If that player doesn't, they [instructions]" after "... unless [player] pays [cost]":
/// what that player does if they didn't pay. "They [verb] ... they control" is the
/// player's own instruction, read as "[verb] ... you control" performed by them.
fn if_that_player_doesnt_they(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("if that player doesn't, they ")
        .or_else(|| l.strip_prefix("if they don't, they "))
    else {
        return false;
    };
    if r.starts_with("may ") || !ends_with_unless_paid(prev) {
        return false;
    }
    let who = b.it_player.clone();
    if matches!(who, PlayerRef::You)
        || super::oracle_hardening_referents::is_no_player_referent(&who)
    {
        return false;
    }
    // The plural verb form is the imperative; "they"/"their" are that player.
    let words: Vec<&str> = r.split(' ').collect();
    if words.iter().any(|w| matches!(*w, "you" | "your" | "them")) {
        return false;
    }
    let you: Vec<&str> = words
        .iter()
        .map(|w| match *w {
            "they" => "you",
            "their" => "your",
            w => w,
        })
        .collect();
    let text = you.join(" ");
    let saved = (b.it.clone(), b.it_player.clone());
    b.it_player = PlayerRef::You;
    let parsed = parse_clause(&text, b);
    (b.it, b.it_player) = saved;
    let Some(e) = parsed else {
        return false;
    };
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        p,
        Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(Effect::AsPlayer {
                who,
                effect: Box::new(e),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if that player doesn't, they [instructions]", priority: 880, apply: if_that_player_doesnt_they } }

/// "you both own and control", "you own and control": owned and controlled by you.
fn own_and_control<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t
        .strip_prefix("you both own and control")
        .or_else(|| t.strip_prefix("you own and control"))?;
    if !(r.is_empty() || r.starts_with(' ') || r.starts_with(',')) {
        return None;
    }
    Some((
        Filter::and(vec![
            Filter::OwnedBy(PlayerRel::You),
            Filter::ControlledBy(PlayerRel::You),
        ]),
        r,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "you both own and control", priority: 100, parse: own_and_control } }

/// "lose all unspent mana" (you): the mana empties from your mana pool (CR 106.4). "That
/// player loses all unspent mana" is `r106_mana`'s.
fn lose_unspent_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    matches!(end(l), "lose all unspent mana" | "you lose all unspent mana").then_some(
        Effect::LoseUnspentMana {
            who: PlayerRef::You,
            to: None,
        },
    )
}

inventory::submit! { super::EffectPattern { name: "lose all unspent mana", priority: 300, parse: lose_unspent_mana } }
