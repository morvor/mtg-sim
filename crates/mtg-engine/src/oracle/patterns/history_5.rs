//! History grammar (round 5):
//!
//! * "At the beginning of each end step, if a creature dealt damage by ~ this turn died,
//!   put that card onto the battlefield under your control." (Krovikan Vampire): the
//!   condition is about the creatures that died this turn, judged as they last existed
//!   (CR 603.10a, 700.4); "that card" is each such card still in a graveyard as the
//!   ability resolves, including one that died in response (its rulings). A card that
//!   left the graveyard is a new object the ability doesn't find (CR 400.7).
//! * "Sacrifice it when you lose control of ~." after an instruction that put cards onto
//!   the battlefield: a delayed triggered ability (CR 603.7a, 603.10d) created as they're
//!   put there, sacrificing those you still control (CR 701.21a); see
//!   `triggers_5::sacrifice_when_you_lose_control` for the form after a delayed return.

use super::{ConditionPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "a creature dealt damage by ~ this turn died": the filter of the creatures.
fn died_creature(c: &str) -> Option<Filter> {
    let r = c
        .strip_prefix("a ")
        .or_else(|| c.strip_prefix("an "))?
        .strip_suffix(" died")?;
    let (head, by) = r.split_once(" dealt damage by ")?;
    let source = match by.strip_suffix(" this turn")? {
        "~" => Sel::This,
        _ => return None,
    };
    let (f, false, tail) = parse_object_phrase(head)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Filter::and(vec![
        f,
        Filter::DealtDamageThisTurnBy(Box::new(source)),
    ]))
}

/// The creatures that died this turn matching `f` (as they last existed), as the cards
/// they became in a graveyard.
fn died_cards(f: Filter) -> Sel {
    Sel::Matching(
        Box::new(Sel::ThisTurn(Box::new(TriggerCond::Dies(f)))),
        Filter::InZone(ZoneKind::Graveyard),
    )
}

fn died_condition(c: &str) -> Option<Condition> {
    let f = died_creature(end(c))?;
    Some(Condition::Compare(
        Value::CountSel(Box::new(Sel::ThisTurn(Box::new(TriggerCond::Dies(f))))),
        Cmp::Ge,
        Value::c(1),
    ))
}

inventory::submit! { ConditionPattern { name: "history 5: a creature dealt damage by ~ this turn died", priority: 100, parse: died_condition } }

/// "that card" in a card's text with the condition "if a creature dealt damage by ~ this
/// turn died": the cards that died.
pub(crate) fn died_that_card() -> Option<Sel> {
    died_condition_filter().map(died_cards)
}

/// The filter of "if a creature dealt damage by ~ this turn died" in the card's text.
fn died_condition_filter() -> Option<Filter> {
    let raw = crate::oracle::raw_text().to_lowercase();
    let i = raw.find(", if a")?;
    let rest = &raw[i + ", if ".len()..];
    let (cond, _) = rest.split_once(", ")?;
    let name = crate::oracle::card_name().to_lowercase();
    let cond = if name.is_empty() {
        cond.to_string()
    } else {
        cond.replace(&name, "~")
    };
    let cond = cond.replace("this creature", "~");
    died_creature(&cond)
}

/// The permanents the instruction before put onto the battlefield, and the source.
const RETURNED: Var = vars::USER + 14_550;
const RETURNED_BY: Var = vars::USER + 14_551;

/// Whether the last instruction of `e` moves objects onto the battlefield.
fn last_moves_onto_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(last_moves_onto_battlefield),
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield,
        _ => false,
    }
}

/// "Sacrifice it when you lose control of ~." after "put that card onto the battlefield
/// under your control".
fn sacrifice_it_when_you_lose_control(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "sacrifice it when you lose control of ~" | "sacrifice them when you lose control of ~"
    ) || !last_moves_onto_battlefield(prev)
    {
        return false;
    }
    let mine = Sel::All(Filter::and(vec![
        Filter::In(Box::new(Sel::Var(RETURNED))),
        Filter::ControlledBy(PlayerRel::You),
    ]));
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Store {
            var: RETURNED,
            sel: Sel::Var(vars::IT),
        },
        Effect::Store {
            var: RETURNED_BY,
            sel: Sel::This,
        },
        Effect::DelayedTrigger {
            trigger: TriggerCond::LoseControl(Filter::In(Box::new(Sel::Var(RETURNED_BY)))),
            body: Box::new(Body::effect(Effect::SacrificeObjects { what: mine })),
            once: true,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "history 5: sacrifice it when you lose control of ~", priority: 40, apply: sacrifice_it_when_you_lose_control } }
