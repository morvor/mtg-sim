//! Edicts with choices (CR 701.21a: the player who sacrifices chooses what they
//! sacrifice; CR 101.4: several players choose in APNAP order, then the permanents are
//! sacrificed at the same time):
//!
//! - several kinds at once: "Target player sacrifices an artifact and a land of their
//!   choice.", "Each player sacrifices an artifact, a creature, an enchantment, a land, and
//!   a planeswalker of their choice." (each permanent chosen fills one of the kinds; all
//!   are sacrificed together);
//! - a qualifier after "of their choice": "Target opponent sacrifices a creature of their
//!   choice that attacked or blocked this turn.", "defending player sacrifices a creature
//!   of their choice that's blocking it", "each opponent sacrifices a creature of their
//!   choice that dealt combat damage to you this turn";
//! - optional for several players: "any opponent may sacrifice a creature of their
//!   choice. If a player does, ..." / "If no one does, ..." (each opponent in turn
//!   chooses whether to and which, knowing the earlier choices; then the chosen creatures
//!   are sacrificed together).

use super::{EffectPattern, FilterSuffixPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;

/// The permanents each player has chosen so far (one variable per kind).
const PICKED: [Var; 6] = [
    vars::USER + 4410,
    vars::USER + 4411,
    vars::USER + 4412,
    vars::USER + 4413,
    vars::USER + 4414,
    vars::USER + 4415,
];

/// The permanents sacrificed by an optional edict ("if a player does").
const OPTIONAL_SACRIFICED: Var = vars::USER + 4416;

/// "a creature", "two lands", "that many permanents": a count and a single object phrase.
/// Returns (count, filter, the phrase's text, the rest).
fn counted_object(r: &str) -> Option<(Value, Filter, &str, &str)> {
    let r = r.trim_start();
    let (n, r2) = if let Some(x) = r.strip_prefix("that many ") {
        (Value::EventAmount, x)
    } else {
        parse_number(r)?
    };
    let r2 = r2.trim_start();
    let (f, _, tail) = parse_object_phrase(r2)?;
    let text = r2[..r2.len() - tail.len()].trim_end();
    Some((n, f, text, tail))
}

/// "an artifact, a creature, an enchantment, a land, and a planeswalker", "an artifact and
/// a land": the kinds and counts, with the last phrase's text and the rest.
fn object_list(r: &str) -> Option<(Vec<(Value, Filter)>, &str, &str)> {
    let mut items = Vec::new();
    let mut r = r;
    loop {
        let (n, f, text, tail) = counted_object(r)?;
        // "an artifact, a creature, ...": the phrase parser may have taken the comma.
        let (text, comma) = match text.strip_suffix(',') {
            Some(t) => (t, true),
            None => (text, false),
        };
        items.push((n, f));
        let t = tail.trim_start();
        let next = t
            .strip_prefix(", and ")
            .or_else(|| t.strip_prefix("and "))
            .or_else(|| t.strip_prefix(", "))
            .or_else(|| comma.then_some(t));
        if let Some(x) = next {
            // "and" joining another counted object (not "... and draw a card").
            if counted_object(x).is_some() {
                r = x;
                continue;
            }
        }
        return Some((items, text, tail));
    }
}

/// A qualifier after "of their choice" that the object phrase parser reads after the noun,
/// or one about "it" ("that's blocking it").
fn qualified(text: &str, post: &str, b: &Builder) -> Option<Filter> {
    let post = post.trim();
    if post.is_empty() {
        return parse_object_phrase(text).map(|(f, _, _)| f);
    }
    if let Some(r) = post
        .strip_prefix("that's blocking it")
        .or_else(|| post.strip_prefix("blocking it"))
    {
        if !r.trim().is_empty() {
            return None;
        }
        let (f, _, tail) = parse_object_phrase(text)?;
        if !end(tail).is_empty() {
            return None;
        }
        let blocking = match super::pronoun_groups::singular_it(b) {
            Sel::This => Filter::BlockingSource,
            sel @ (Sel::TriggerObject | Sel::AttachedTo) => Filter::BlockingAnyOf(Box::new(sel)),
            _ => return None,
        };
        return Some(Filter::and(vec![f, blocking]));
    }
    let probe = format!("{text} {post}");
    let (f, _, tail) = parse_object_phrase(&probe)?;
    end(tail).is_empty().then_some(f)
}

/// The subject of an edict: the players, whether it's optional ("may"), and the rest
/// after the verb.
fn subject<'a>(l: &'a str, b: &mut Builder) -> Option<(PlayerRef, bool, String)> {
    if let Some(r) = l.strip_prefix("any opponent may sacrifice ") {
        return Some((PlayerRef::EachOpponent, true, r.to_string()));
    }
    let (who, rest) = player_ref(l, b)?;
    let rest = rest.trim_start();
    if let Some(r) = rest.strip_prefix("sacrifices ") {
        return Some((who, false, r.to_string()));
    }
    None
}

/// Builds the choices and the simultaneous sacrifice for each of `who`: each player
/// chooses a permanent for each kind in turn (one permanent can't fill two kinds, and
/// the player must fill as many kinds as they can), then the chosen permanents are
/// sacrificed together.
fn edict_effect(who: PlayerRef, items: Vec<(Value, Filter)>, optional: bool) -> Option<Effect> {
    // "two lands and a creature": a slot for each permanent.
    let mut slots: Vec<(Value, Filter)> = Vec::new();
    if items.len() == 1 {
        slots = items;
    } else {
        for (n, f) in items {
            let k = n.as_const().filter(|k| (1..=3).contains(k))?;
            for _ in 0..k {
                slots.push((Value::c(1), f.clone()));
            }
        }
    }
    if slots.len() > PICKED.len() {
        return None;
    }
    let kinds: Vec<Filter> = slots.iter().map(|(_, f)| f.clone()).collect();
    let mut body = Vec::new();
    let mut so_far: Option<Var> = None;
    for (i, (n, f)) in slots.into_iter().enumerate() {
        let mut parts = vec![
            f,
            Filter::InZone(ZoneKind::Battlefield),
            Filter::ControlledBy(PlayerRel::Iterated),
        ];
        if let Some(prev) = so_far {
            parts.push(Filter::not(Filter::In(Box::new(Sel::Var(prev)))));
        }
        if kinds.len() > 1 {
            let spec = crate::kw::choice_grammar::OneOfEach {
                kinds: kinds.clone(),
                slot: i,
                picked: so_far,
            };
            let json = serde_json::to_string(&spec).ok()?;
            parts.push(Filter::Custom(
                format!("{}{json}", crate::kw::choice_grammar::ONE_OF_EACH).into(),
            ));
        }
        let choose = Sel::Choose {
            chooser: PlayerRef::Iterated,
            filter: Filter::and(parts),
            count: n,
            up_to: optional,
            store: None,
        };
        let sel = match so_far {
            Some(prev) => Sel::Union(vec![Sel::Var(prev), choose]),
            None => choose,
        };
        body.push(Effect::Store {
            var: PICKED[i],
            sel,
        });
        so_far = Some(PICKED[i]);
    }
    let chosen = Sel::Var(so_far?);
    body.push(Effect::SacrificeObjects { what: chosen });
    if optional {
        // Gathered from every player, for "if a player does".
        body.push(Effect::Store {
            var: OPTIONAL_SACRIFICED,
            sel: Sel::Union(vec![
                Sel::Var(OPTIONAL_SACRIFICED),
                Sel::Var(vars::SACRIFICED),
            ]),
        });
    }
    let each = Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::seq(body)),
    };
    Some(if optional {
        Effect::seq(vec![
            Effect::Store {
                var: OPTIONAL_SACRIFICED,
                sel: Sel::Union(vec![]),
            },
            each,
        ])
    } else {
        each
    })
}

/// "[players] sacrifice(s) [objects] of their choice [qualifier]".
fn edict_of_their_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let k = l.find(" of their choice")?;
    let (pre, post) = (&l[..k], &l[k + " of their choice".len()..]);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let Some((who, optional, rest)) = subject(pre, b) else {
        restore(b);
        return None;
    };
    let parsed = object_list(&rest).and_then(|(mut items, text, tail)| {
        if !end(tail).is_empty() {
            return None;
        }
        // A plain single edict is the core's (`damage_removal::p_edict`).
        if items.len() == 1 && post.trim().is_empty() && !optional {
            return None;
        }
        let last = qualified(text, post, b)?;
        items.last_mut()?.1 = last;
        Some(items)
    });
    let Some(items) = parsed else {
        restore(b);
        return None;
    };
    // A single permanent one player sacrificed is "it" / "that creature" afterward.
    if items.len() == 1
        && !optional
        && items[0].0.as_const() == Some(1)
        && !matches!(
            who,
            PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
        )
    {
        b.it = Sel::Var(vars::SACRIFICED);
    }
    edict_effect(who, items, optional)
}

inventory::submit! { EffectPattern { name: "choice grammar: edict of their choice (kinds, qualifiers, optional)", priority: 49, parse: edict_of_their_choice } }

/// Whether the effect is an optional edict for several players (see [`edict_effect`]).
fn is_optional_edict(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => matches!(
            v.as_slice(),
            [Effect::Store { var, .. }, Effect::ForEachPlayer { .. }] if *var == OPTIONAL_SACRIFICED
        ),
        _ => false,
    }
}

/// "If a player does, [effect]." / "If no one does, [effect]." after an optional edict for
/// several players: whether any of them sacrificed a permanent this way.
fn if_a_player_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !is_optional_edict(prev) {
        return false;
    }
    let l = end(l);
    let (r, negate) = if let Some(r) = l.strip_prefix("if a player does, ") {
        (r, false)
    } else if let Some(r) = l
        .strip_prefix("if no one does, ")
        .or_else(|| l.strip_prefix("if no player does, "))
    {
        (r, true)
    } else {
        return false;
    };
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    let any = Condition::Compare(
        Value::CountSel(Box::new(Sel::Var(OPTIONAL_SACRIFICED))),
        Cmp::Ge,
        Value::c(1),
    );
    let cond = if negate {
        Condition::Not(Box::new(any))
    } else {
        any
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: if a player does (optional edict)", priority: 55, apply: if_a_player_does } }

/// "that attacked or blocked this turn" (CR 508.1, 509.1).
fn attacked_or_blocked<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that attacked or blocked this turn")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((
        Filter::Or(vec![
            Filter::AttackedThisTurn,
            Filter::Custom(crate::kw::basic_effects::BLOCKED_THIS_TURN.into()),
        ]),
        r,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "choice grammar: that attacked or blocked this turn", priority: 60, parse: attacked_or_blocked } }

/// "that dealt combat damage to you this turn" (CR 510.2).
fn dealt_combat_damage_to_you<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that dealt combat damage to you this turn")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((
        Filter::Custom(crate::kw::choice_grammar::DEALT_COMBAT_DAMAGE_TO_YOU.into()),
        r,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "choice grammar: that dealt combat damage to you this turn", priority: 60, parse: dealt_combat_damage_to_you } }
