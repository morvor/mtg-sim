//! Per-player outcomes of an instruction several players were given (CR 101.4, 118.12,
//! 608.2c, 608.2e):
//!
//! - after "each [players] may [instruction]" (the players decide in APNAP order, then
//!   those who accepted do it, see `player_subjects`): "Each player who doesn't loses 3
//!   life" (The Death of Gwen Stacy), "If they don't, they lose 3 life" (Professor Onyx),
//!   "For each opponent who doesn't, that player loses 2 life and you draw a card" (Braids,
//!   Arisen Nightmare), "~ deals 3 damage to each opponent who didn't" (Zoyowa
//!   Lava-Tongue), "If a player does, creatures that player controls can't attack you ..."
//!   (Orzhov Advokist), "For each opponent who does, you gain protection from that player
//!   ..." (Noble Heritage). "Does" is whether the player chose to (CR 118.12).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_sentence, Builder};
use crate::oracle::phrases::end;
use crate::scry_rules::{OPTED, OPT_IN};

/// The "each [players] may [instruction]" a sequence ends with.
struct OptIn {
    /// The players offered the choice.
    offered: PlayerFilter,
    /// The variable holding the players who accepted.
    accepted: Var,
    /// Where the "those who accepted do it" step is.
    at: usize,
}

fn offered_filter(p: &PlayerRef) -> Option<PlayerFilter> {
    Some(match p {
        PlayerRef::EachPlayer => PlayerFilter::Any,
        PlayerRef::EachOpponent => PlayerFilter::Opponent,
        PlayerRef::EachOtherPlayer => PlayerFilter::NotYou,
        PlayerRef::Each(f) => f.clone(),
        _ => return None,
    })
}

fn opt_in_of(v: &[Effect]) -> Option<OptIn> {
    let n = v.len();
    if n < 3 {
        return None;
    }
    let Effect::ForEachPlayer {
        who: PlayerRef::Var(accepted),
        ..
    } = &v[n - 1]
    else {
        return None;
    };
    let Effect::Store {
        var,
        sel: Sel::Var(OPTED),
    } = &v[n - 2]
    else {
        return None;
    };
    if var != accepted {
        return None;
    }
    let Effect::ForEachPlayer { who, effect } = &v[n - 3] else {
        return None;
    };
    if !format!("{effect:?}").contains(OPT_IN) {
        return None;
    }
    Some(OptIn {
        offered: offered_filter(who)?,
        accepted: *accepted,
        at: n - 1,
    })
}

/// The sequence an "each [players] may [instruction]" ends, at the end of `e` (also
/// inside "If you do, ...").
fn opt_in_seq(e: &mut Effect) -> Option<&mut Vec<Effect>> {
    if matches!(&*e, Effect::Seq(v) if opt_in_of(v).is_some()) {
        return match e {
            Effect::Seq(v) => Some(v),
            _ => None,
        };
    }
    match e {
        Effect::Seq(v) => v.last_mut().and_then(opt_in_seq),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => opt_in_seq(then),
        _ => None,
    }
}

/// Who the follow-up is about: the players who accepted (`true`) or those who didn't,
/// among the players a noun names ("player", "opponent").
#[derive(Debug)]
struct Who {
    accepted: bool,
    among: Option<PlayerFilter>,
}

fn noun_filter(noun: &str) -> Option<Option<PlayerFilter>> {
    Some(match noun {
        "player" => None,
        "opponent" => Some(PlayerFilter::Opponent),
        _ => return None,
    })
}

/// The follow-up's form: who it's about and the instruction, as a clause about "that
/// player" (the player it's performed for).
fn split(l: &str) -> Option<(Who, String)> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    // "if they don't, [clause]", "if a player does, [clause]".
    for (p, accepted) in [
        ("if they don't, ", false),
        ("if they do, ", true),
        ("if a player does, ", true),
        ("if a player doesn't, ", false),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            return Some((
                Who {
                    accepted,
                    among: None,
                },
                r.to_string(),
            ));
        }
    }
    // "for each opponent who doesn't, [clause]".
    if let Some(r) = l.strip_prefix("for each ") {
        let (noun, r) = r.split_once(" who ")?;
        let (did, rest) = r.split_once(", ")?;
        let accepted = match did {
            "does" | "did" => true,
            "doesn't" | "didn't" => false,
            _ => return None,
        };
        return Some((
            Who {
                accepted,
                among: noun_filter(noun)?,
            },
            rest.to_string(),
        ));
    }
    // "each player who doesn't loses 3 life": the predicate about that player.
    if let Some(r) = l.strip_prefix("each ") {
        let (noun, r) = r.split_once(" who ")?;
        let (did, pred) = r.split_once(' ')?;
        let accepted = match did {
            "does" => true,
            "doesn't" => false,
            _ => return None,
        };
        return Some((
            Who {
                accepted,
                among: noun_filter(noun)?,
            },
            format!("that player {pred}"),
        ));
    }
    // "~ deals 3 damage to each opponent who didn't".
    for (suffix, accepted) in [(" who didn't", false), (" who doesn't", false), (" who did", true)] {
        let Some(head) = l.strip_suffix(suffix) else {
            continue;
        };
        for noun in ["opponent", "player"] {
            if let Some(h) = head.strip_suffix(&format!(" each {noun}")) {
                return Some((
                    Who {
                        accepted,
                        among: noun_filter(noun)?,
                    },
                    format!("{h} that player"),
                ));
            }
        }
    }
    None
}

/// The follow-up of an "each [players] may [instruction]".
fn after_each_may(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(v) = opt_in_seq(prev) else {
        return false;
    };
    let Some(opt) = opt_in_of(v) else {
        return false;
    };
    let Some((who, clause)) = split(end(l)) else {
        return false;
    };
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let body = parse_sentence(&clause, b).or_else(|| parse_clause(&clause, b));
    b.it_player = saved;
    let Some(body) = body else {
        return false;
    };
    let body = players_objects_later(body);
    // Only about that player: no other player is "that player".
    let text = format!("{body:?}");
    if text.contains("TriggerPlayer") {
        return false;
    }
    // Who did it: a player who accepted but couldn't do it didn't (Zoyowa Lava-Tongue
    // ruling: an opponent with no cards in hand can't choose to discard), when the
    // instruction records that; otherwise who accepted (CR 118.12).
    let recorded = already_recorded(v);
    let did = record_done(v, opt.at);
    let at = opt.at + usize::from(did.is_some() && !recorded);
    let did_var = did.unwrap_or(opt.accepted);
    let mut filter = vec![opt.offered];
    filter.extend(who.among);
    let did_it = PlayerFilter::Ref(Box::new(PlayerRef::Var(did_var)));
    if who.accepted {
        // After doing it, in the same "each player" instruction.
        let Effect::ForEachPlayer { effect, .. } = &mut v[at] else {
            return false;
        };
        filter.push(did_it);
        let old = std::mem::take(&mut **effect);
        **effect = Effect::seq(vec![
            old,
            Effect::If {
                cond: Condition::PlayerMatches(PlayerRef::Iterated, PlayerFilter::And(filter)),
                then: Box::new(body),
                otherwise: Box::new(Effect::Noop),
            },
        ]);
        return true;
    }
    filter.push(PlayerFilter::Not(Box::new(did_it)));
    v.push(Effect::ForEachPlayer {
        who: PlayerRef::Each(PlayerFilter::And(filter)),
        effect: Box::new(body),
    });
    true
}

/// "If a player does, creatures that player controls can't attack you ... until your
/// next turn": the creatures that player controls at any time during the effect, not
/// only those they control now (Orzhov Advokist ruling).
fn players_objects_later(e: Effect) -> Effect {
    match e {
        Effect::AddRestriction {
            mut restriction,
            duration,
        } => {
            if let Restriction::CantAttackPlayer { attackers, .. } = &mut restriction {
                if let Filter::In(inner) = attackers {
                    if let Sel::All(f) = &**inner {
                        if format!("{f:?}").contains("ControlledByPlayer(Iterated)") {
                            *attackers = f.clone();
                        }
                    }
                }
            }
            Effect::AddRestriction {
                restriction,
                duration,
            }
        }
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(players_objects_later).collect()),
        e => e,
    }
}

/// The players who did what they accepted to do ("each opponent who didn't").
pub const DID: Var = vars::USER + 9650;

/// Whether the instruction a player performs records whether they did it.
fn records_done(e: &Effect) -> bool {
    match e {
        Effect::Discard { .. }
        | Effect::Sacrifice { .. }
        | Effect::SacrificeObjects { .. }
        | Effect::AddCounters { .. }
        | Effect::Exile { .. }
        | Effect::Move { .. }
        | Effect::PayOptional { .. } => true,
        Effect::ChooseOne { options, .. } => options.iter().all(|(_, e)| records_done(e)),
        Effect::AsPlayer { effect, .. } => records_done(effect),
        _ => false,
    }
}

fn already_recorded(v: &[Effect]) -> bool {
    v.iter()
        .any(|e| matches!(e, Effect::Store { var, sel: Sel::None } if *var == DID))
}

/// Records, in [`DID`], the players who did the accepted instruction at `v[at]`, if it
/// records whether it was done; returns [`DID`] then.
fn record_done(v: &mut Vec<Effect>, at: usize) -> Option<Var> {
    if already_recorded(v) {
        return Some(DID);
    }
    let Effect::ForEachPlayer { effect, .. } = &mut v[at] else {
        return None;
    };
    if !records_done(effect) {
        return None;
    }
    let old = std::mem::take(&mut **effect);
    **effect = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::Store {
                var: DID,
                sel: Sel::Union(vec![Sel::Var(DID), Sel::Players(PlayerRef::Iterated)]),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    v.insert(
        at,
        Effect::Store {
            var: DID,
            sel: Sel::None,
        },
    );
    Some(DID)
}

inventory::submit! { FollowupPattern { name: "followups: each player who doesn't (after each player may)", priority: 140, apply: after_each_may } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn after_each_player_may() {
        for (ty, text) in [
            ("Sorcery", "Each player may discard a card. Each player who doesn't loses 3 life."),
            ("Sorcery", "Each opponent may discard a card. If they don't, they lose 3 life."),
            ("Sorcery", "Each opponent may sacrifice a creature. For each opponent who doesn't, that player loses 2 life and you draw a card."),
            ("Sorcery", "Each opponent may discard a card. ~ deals 3 damage to each opponent who didn't."),
            ("Sorcery", "Each player may sacrifice a creature. If a player does, that player draws a card."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
