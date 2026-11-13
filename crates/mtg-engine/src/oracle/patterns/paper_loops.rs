//! Repeated processes of older Oracle wordings:
//!
//! - "Target opponent mills a card, then repeats this process until a creature card or X
//!   cards have been put into their graveyard this way, whichever comes first." (Helm of
//!   Obedience): see `kw/mill_until.rs`; "If one or more creature cards were put into that
//!   graveyard this way, sacrifice ~ and put one of them onto the battlefield under your
//!   control." reads the cards that process put there.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// The creature cards (of the stopping type) the process put into the graveyard.
const PUT_THIS_WAY: Var = vars::USER + 41_201;

/// "[target player] mills a card, then repeats this process until a [type] card or x cards
/// have been put into their graveyard this way, whichever comes first".
fn mill_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subj, r) = l.split_once(" mills a card, then repeats this process until a ")?;
    let (ty, r) = r.split_once(" card or x cards have been put into their graveyard this way")?;
    if !matches!(r, ", whichever comes first" | "") {
        return None;
    }
    let stop = crate::types::CardType::from_word(ty)?;
    if !subj.starts_with("target ") {
        return None;
    }
    let first = b.targets.len();
    let (who, rest) = player_ref(subj, b)?;
    let PlayerRef::Target(slot) = who else {
        b.targets.truncate(first);
        return None;
    };
    if !rest.trim().is_empty() {
        b.targets.truncate(first);
        return None;
    }
    Some(Effect::Custom(SmolStr::new(
        crate::kw::mill_until::effect_name(stop, slot),
    )))
}

inventory::submit! { EffectPattern { name: "mills a card, then repeats this process until a [type] card or X cards", priority: 300, parse: mill_until } }

/// The stopping type of a mill-until process the effect ends with.
fn ends_with_mill_until(e: &Effect) -> Option<crate::types::CardType> {
    match e {
        Effect::Seq(v) => v.last().and_then(ends_with_mill_until),
        Effect::Custom(n) => crate::kw::mill_until::parts(n).map(|(t, _)| t),
        _ => None,
    }
}

/// "If one or more creature cards were put into that graveyard this way, [effect]" after
/// the process: those cards (still in that graveyard) are "them".
fn if_put_into_that_graveyard(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if one or more ") else {
        return false;
    };
    let Some((ty, x)) = r.split_once(" cards were put into that graveyard this way, ") else {
        return false;
    };
    let Some(stop) = ends_with_mill_until(prev) else {
        return false;
    };
    if crate::types::CardType::from_word(ty) != Some(stop) {
        return false;
    }
    let saved = b.it.clone();
    b.it = Sel::Var(PUT_THIS_WAY);
    let parsed = parse_clause(x, b);
    b.it = saved;
    let Some(e) = parsed else {
        return false;
    };
    let them = Effect::Store {
        var: PUT_THIS_WAY,
        sel: Sel::All(Filter::and(vec![
            Filter::Type(stop),
            Filter::Card,
            Filter::In(Box::new(Sel::Var(vars::IT))),
            Filter::InZone(ZoneKind::Graveyard),
        ])),
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        old,
        them,
        Effect::If {
            cond: Condition::SelNonEmpty(Sel::Var(PUT_THIS_WAY)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if one or more [type] cards were put into that graveyard this way", priority: 60, apply: if_put_into_that_graveyard } }

/// The card chosen in the current pass of the process.
const LAST_CHOSEN: Var = vars::USER + 41_202;
/// Every card chosen in a pass so far.
const ALL_CHOSEN: Var = vars::USER + 41_203;

/// In a process's body, the choice "[player] chooses a card in your graveyard": excludes
/// the cards already chosen and records the chosen one as the last chosen card (only when
/// one was chosen: with nothing left to choose, that part is skipped).
fn exclude_chosen(body: &mut Effect) -> bool {
    let Effect::Seq(v) = body else {
        return false;
    };
    let Some(i) = v.iter().position(|e| {
        matches!(e, Effect::ForEachPlayer { effect, .. }
            if matches!(&**effect, Effect::Store { sel: Sel::Union(u), .. }
                if u.iter().any(|s| matches!(s, Sel::Choose { count: Value::Const(1), .. }))))
    }) else {
        return false;
    };
    let Effect::ForEachPlayer { effect, .. } = &mut v[i] else {
        return false;
    };
    let Effect::Store {
        var,
        sel: Sel::Union(u),
    } = &mut **effect
    else {
        return false;
    };
    let chosen = *var;
    for s in u.iter_mut() {
        if let Sel::Choose { filter, .. } = s {
            *filter = Filter::and(vec![
                filter.clone(),
                Filter::not(Filter::In(Box::new(Sel::Var(ALL_CHOSEN)))),
            ]);
        }
    }
    v.insert(
        i + 1,
        Effect::If {
            cond: Condition::SelNonEmpty(Sel::Var(chosen)),
            then: Box::new(Effect::Seq(vec![
                Effect::Store {
                    var: LAST_CHOSEN,
                    sel: Sel::Var(chosen),
                },
                Effect::Store {
                    var: ALL_CHOSEN,
                    sel: Sel::Union(vec![Sel::Var(ALL_CHOSEN), Sel::Var(chosen)]),
                },
            ])),
            otherwise: Box::new(Effect::Noop),
        },
    );
    true
}

/// "[Player] chooses a card in your graveyard. You may pay {G}. If you do, repeat this
/// process except that [player] can't choose a card already chosen for ~. Then put the
/// last chosen card into your hand." (Forgotten Lore): each pass's choice excludes the
/// cards chosen in earlier passes (only for this spell, its ruling); when you stop paying,
/// the card chosen last goes to your hand.
fn repeat_except_already_chosen(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    const EXCEPT: &str = " except that opponent can't choose a card already chosen for ~.";
    const LAST: &str = " Then put the last chosen card into your hand.";
    let t = block.trim();
    let body_text = t.strip_suffix(LAST)?;
    let i = body_text.find(EXCEPT)?;
    if !body_text.ends_with(EXCEPT) || !body_text[..i].ends_with("If you do, repeat this process") {
        return None;
    }
    let plain = format!("{}.", &body_text[..i]);
    let mut abilities = crate::oracle::parse_ability(&plain, ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Spell(s) = &mut a.kind else {
        return None;
    };
    let Effect::RepeatProcess { body } = &mut s.body.effect else {
        return None;
    };
    if !exclude_chosen(body) {
        return None;
    }
    let process = std::mem::take(&mut s.body.effect);
    s.body.effect = Effect::Seq(vec![
        process,
        Effect::Move {
            what: Sel::Var(LAST_CHOSEN),
            to: Destination::zone(ZoneKind::Hand),
        },
    ]);
    a.text = t.to_string();
    Some(abilities)
}

inventory::submit! { super::AbilityPattern { name: "repeat this process except that [player] can't choose a card already chosen", priority: 60, parse: repeat_except_already_chosen } }
