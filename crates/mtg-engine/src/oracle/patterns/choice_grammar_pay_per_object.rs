//! "[player] may choose any number of tapped blue creatures they control and pay {U} for
//! each creature chosen this way. If the player does, untap those creatures." (Thelon's
//! Curse, Magnetic Mountain, Dream Tides): the player chooses the objects, then may pay
//! the cost once for each of them (all of it or none, CR 118.12, 601.2h); "if the player
//! does" is whether they paid, and "those creatures" are the chosen ones.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

/// The objects chosen.
const CHOSEN: Var = vars::USER + 30527;

fn choose_and_pay_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subj, r) = l.split_once(" may choose any number of ")?;
    let (noun, r) = r.split_once(" and pay ")?;
    let (cost_s, each) = r.split_once(" for each ")?;
    if !each.ends_with(" chosen this way") {
        return None;
    }
    let targets = b.targets.len();
    let (who, rest) = player_ref(subj, b)?;
    if !rest.trim().is_empty() || b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    // "tapped blue creatures they control": theirs.
    let noun = noun
        .strip_suffix(" they control")
        .or_else(|| noun.strip_suffix(" you control"))?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let (cost, false) = crate::oracle::costs::parse_cost(cost_s)? else {
        return None;
    };
    let filter = Filter::and(vec![
        f,
        Filter::ControlledBy(PlayerRel::You),
        Filter::InZone(ZoneKind::Battlefield),
    ]);
    let body = Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: filter.clone(),
                count: Value::CountSel(Box::new(Sel::All(filter))),
                up_to: true,
                store: None,
            },
        },
        Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::free().with(CostPart::Repeated {
                cost: Box::new(cost),
                times: Value::CountSel(Box::new(Sel::Var(CHOSEN))),
            }),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(body),
    })
}

inventory::submit! { EffectPattern { name: "may choose any number of [objects] and pay [cost] for each", priority: 60, parse: choose_and_pay_for_each } }

/// "If the player does, untap those creatures." after the above: if they paid.
fn if_the_player_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l)
        .strip_prefix("if the player does, ")
        .or_else(|| end(l).strip_prefix("if that player does, "))
    else {
        return false;
    };
    let Effect::AsPlayer { effect, .. } = prev else {
        return false;
    };
    let Effect::Seq(v) = effect.as_mut() else {
        return false;
    };
    if !matches!(v.first(), Some(Effect::Store { var, .. }) if *var == CHOSEN) {
        return false;
    }
    let saved = std::mem::replace(&mut b.it, Sel::Var(CHOSEN));
    let targets = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&r.replace("those creatures", "them"), b);
    b.it = saved;
    let Some(e) = e else {
        b.targets.truncate(targets);
        return false;
    };
    v.push(Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "if the player does (paid for the chosen objects)", priority: 60, apply: if_the_player_does } }
