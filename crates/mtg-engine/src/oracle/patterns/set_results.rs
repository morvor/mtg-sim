//! Results of optional and each-player actions on sets (cards5-costs-sets):
//!
//! - "you may remove any number of +1/+1 counters from ~. If you do, create that many 1/1
//!   ... tokens." (Tetravus): "If you do" is "if one or more counters were removed this
//!   way", and "that many" the number removed (CR 608.2c; removing none is not doing it).
//! - "you may exile any number of cards from your graveyard with four or more card types
//!   among them. If you do, put a permanent card from among them onto the battlefield ..."
//!   (Winter, Cynical Opportunist): a set quality in an instruction, as in a cost
//!   (`Filter::Together`), and "them" for the cards exiled.
//! - "Each player may exile their hand and ... you may play cards you own exiled this way
//!   ..." (Wheel of Potential): the cards all the players exiled, collected over the loop.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "If you do, [instruction with "that many"]" after an optional removal of any number of
/// counters.
fn if_you_do_that_many_removed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(clause) = end(l).strip_prefix("if you do, ") else {
        return false;
    };
    if !clause.contains(" that many ")
        || clause.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x")
    {
        return false;
    }
    // The removal is optional ("you may remove ...").
    let optional = match &*prev {
        Effect::May { effect, .. } => matches!(**effect, Effect::RemoveCountersUpTo { .. }),
        _ => false,
    };
    if !optional {
        return false;
    }
    let count = Value::Var(super::counter_grammar::REMOVED);
    let saved = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(&clause.replace(" that many ", " x "), b)
        .and_then(|e| super::r107_numbers::substitute_x(&e, &count))
    else {
        b.targets.truncate(saved);
        return false;
    };
    let Some(removal) = super::counter_grammar::last_removal(prev) else {
        return false;
    };
    let r = std::mem::take(removal);
    *removal = Effect::Seq(vec![
        r,
        Effect::StoreValue {
            var: super::counter_grammar::REMOVED,
            value: Value::Prev,
        },
    ]);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::StoreValue {
            var: super::counter_grammar::REMOVED,
            value: Value::c(0),
        },
        old,
        Effect::If {
            cond: Condition::Compare(count, Cmp::Ge, Value::c(1)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "set results: if you do, [that many] after removing any number of counters", priority: 49, apply: if_you_do_that_many_removed } }

#[cfg(test)]
mod tests {
    #[test]
    fn set_result_texts_compile() {
        for (t, ty) in [
            ("At the beginning of your end step, you may exile any number of cards from your graveyard with four or more card types among them. If you do, put a permanent card from among them onto the battlefield with a finality counter on it.", "Creature — Human"),
            ("You get {E}{E}{E} (three energy counters), then you may pay any amount of {E}.\nEach player may exile their hand and draw a number of cards equal to the amount of {E} paid this way. If seven or more {E} was paid this way, you may play cards you own exiled this way until the end of your next turn.", "Sorcery"),
            ("At the beginning of your upkeep, you may remove any number of +1/+1 counters from Probe. If you do, create that many 1/1 colorless Tetravite artifact creature tokens. They each have flying and \"This token can't be enchanted.\"", "Artifact Creature — Construct"),
        ] {
            let u = super::super::costs_6::compile_tests::unsupported(t, ty);
            if !u.is_empty() {
                eprintln!("{}", super::super::costs_6::compile_tests::dump(t, ty));
            }
            assert_eq!(u, Vec::<String>::new(), "{t}");
        }
    }
}

/// "exile any number of cards from your graveyard with four or more card types among
/// them" as an instruction (Winter, Cynical Opportunist): the cards chosen must together
/// have the relationship (`Filter::Together`, chosen as `target_groups::fit_together`
/// does; a choice without it exiles nothing), as in a cost (Nethergoyf's escape cost).
fn exile_set_with_relationship(l: &str, b: &mut Builder) -> Option<crate::ability::Effect> {
    let l = end(l);
    let r = l.strip_prefix("exile any number of ")?;
    let (i, j, grp) = crate::oracle::phrases::find_group_phrase(r)?;
    if j != r.len() || !matches!(grp, TargetGroup::CardTypesAmong(_) | TargetGroup::TotalAtLeast(..) | TargetGroup::ManaSymbolsAmong(..)) {
        return None;
    }
    let saved = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&format!("exile any number of {}", &r[..i]), b);
    let Some(Effect::Exile {
        what: Sel::Choose {
            chooser,
            filter,
            count,
            up_to: true,
            store,
        },
        face_down,
        link,
    }) = e
    else {
        b.targets.truncate(saved);
        return None;
    };
    if b.targets.len() != saved || !crate::relational::groups_of(&filter).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::Exile {
        what: Sel::Choose {
            chooser,
            filter: Filter::and(vec![filter, Filter::Together(grp)]),
            count,
            up_to: true,
            store,
        },
        face_down,
        link,
    })
}

inventory::submit! { super::EffectPattern { name: "set results: exile any number of [cards] with [a set quality]", priority: 60, parse: exile_set_with_relationship } }

/// "If you do, put a permanent card from among them onto the battlefield ..." right after
/// an instruction that exiled cards (Winter, Cynical Opportunist): "them" are the cards
/// exiled this way (read as "a permanent card exiled this way", `referents_values_4`).
fn from_among_them_exiled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let body = l.strip_prefix("if you do, ").unwrap_or(l);
    if !body.starts_with("put ") || body.matches(" from among them ").count() != 1 {
        return false;
    }
    fn is_exile(e: &Effect) -> bool {
        match e {
            Effect::Exile { .. } => true,
            Effect::May { effect, .. } => is_exile(effect),
            Effect::Seq(v) => v.last().is_some_and(is_exile),
            _ => false,
        }
    }
    if !is_exile(prev) {
        return false;
    }
    crate::oracle_ext::apply_followup_ext(&l.replacen(" from among them ", " exiled this way ", 1), prev, b)
}

inventory::submit! { FollowupPattern { name: "set results: put a [card] from among them (the cards exiled)", priority: 60, apply: from_among_them_exiled } }

/// The cards every player exiled from their hand in an "Each player [may] exile[s] their
/// hand" instruction, collected over the players (see [`each_players_exiled_cards`]).
pub const EACH_EXILED_HAND: Var = vars::USER + 30_873;

/// Whether `e` is the per-player part of "each player [may] exile[s] their hand ...": a
/// sequence starting with the exile of the iterated player's hand.
fn exiles_own_hand(e: &Effect) -> bool {
    matches!(e, Effect::Exile { what: Sel::All(f), .. }
        if matches!(f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::InZone(ZoneKind::Hand)))
            && v.iter().any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You)))))
}

/// Makes the loop over players of "each player [may] exile[s] their hand" collect the
/// cards every player exiled into [`EACH_EXILED_HAND`]. `false` if `prev` has no such loop.
fn collect_each_exiled_hand(prev: &mut Effect) -> bool {
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(i) = v.iter().rposition(|e| {
        matches!(e, Effect::ForEachPlayer { effect, .. }
            if matches!(&**effect, Effect::AsPlayer { effect, .. }
                if matches!(&**effect, Effect::Seq(s) if s.first().is_some_and(exiles_own_hand))))
    }) else {
        return false;
    };
    let Effect::ForEachPlayer { effect, .. } = &mut v[i] else {
        return false;
    };
    let Effect::AsPlayer { effect, .. } = &mut **effect else {
        return false;
    };
    let Effect::Seq(s) = &mut **effect else {
        return false;
    };
    s.insert(
        1,
        Effect::Store {
            var: EACH_EXILED_HAND,
            sel: Sel::Union(vec![Sel::Var(EACH_EXILED_HAND), Sel::Var(vars::IT)]),
        },
    );
    v.insert(
        i,
        Effect::Store {
            var: EACH_EXILED_HAND,
            sel: Sel::None,
        },
    );
    true
}

/// "[If ...,] you may play cards you own exiled this way until the end of your next turn."
/// after "Each player may exile their hand and ..." (Wheel of Potential): the cards all the
/// players exiled that way (each player's choice was made in turn order, its ruling), of
/// those the ones you own.
fn each_players_exiled_cards(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (phrase, own) = if l.contains("cards you own exiled this way") {
        ("cards you own exiled this way", true)
    } else if l.contains("cards exiled this way") {
        ("cards exiled this way", false)
    } else {
        return false;
    };
    if l.matches(phrase).count() != 1 || !l.contains("you may play ") && !l.contains("you may cast ") {
        return false;
    }
    let mut new_prev = prev.clone();
    if !collect_each_exiled_hand(&mut new_prev) {
        return false;
    }
    let all = Sel::Var(EACH_EXILED_HAND);
    let cards = if own {
        Sel::Matching(Box::new(all), Filter::OwnedBy(PlayerRel::You))
    } else {
        all
    };
    // Read as the permission for the cards a cost exiled ("You may play that card ...",
    // `permission_grammar`), with those cards replaced by these.
    let saved_it = std::mem::replace(&mut b.it, Sel::Var(crate::zones::COST_EXILED));
    let saved_targets = b.targets.len();
    let e = crate::oracle::effects::parse_sentence(&l.replacen(phrase, "them", 1), b);
    b.it = saved_it;
    let Some(e) = e else {
        b.targets.truncate(saved_targets);
        return false;
    };
    let (Ok(from), Ok(to), Ok(mut json)) = (
        serde_json::to_value(Sel::Var(crate::zones::COST_EXILED)),
        serde_json::to_value(&cards),
        serde_json::to_value(&e),
    ) else {
        return false;
    };
    if replace_json(&mut json, &from, &to) != 1 {
        b.targets.truncate(saved_targets);
        return false;
    }
    let Ok(e) = serde_json::from_value::<Effect>(json) else {
        return false;
    };
    let Effect::Seq(mut v) = new_prev else {
        return false;
    };
    v.push(e);
    *prev = Effect::Seq(v);
    true
}

/// Replaces each `from` in `v` with `to`; how many.
fn replace_json(v: &mut serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> usize {
    if v == from {
        *v = to.clone();
        return 1;
    }
    match v {
        serde_json::Value::Array(a) => a.iter_mut().map(|x| replace_json(x, from, to)).sum(),
        serde_json::Value::Object(m) => m.values_mut().map(|x| replace_json(x, from, to)).sum(),
        _ => 0,
    }
}

inventory::submit! { FollowupPattern { name: "set results: cards [you own] exiled this way after each player exiles their hand", priority: 60, apply: each_players_exiled_cards } }
