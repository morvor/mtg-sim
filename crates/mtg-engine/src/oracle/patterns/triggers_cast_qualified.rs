//! Cast triggers qualified by more than the spell's characteristics (Pyromancer
//! Ascension):
//!
//! * "Whenever you cast an instant or sorcery spell that has the same name as a card in
//!   your graveyard": the spell's name is compared with the cards there as it's cast.
//! * "Whenever you cast an instant or sorcery spell while ~ has two or more quest counters
//!   on it": a condition that's part of the trigger event, checked when the spell is cast
//!   (not an intervening "if", CR 603.4).

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn cast_trigger(head: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = crate::oracle::triggers::parse_trigger_condition(head)?;
    matches!(t.0, TriggerCond::CastSpell { .. }).then_some(t)
}

fn cast_qualified(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    // "... that has the same name as a card in your graveyard".
    if let Some(head) = r.strip_suffix(" that has the same name as a card in your graveyard") {
        let (TriggerCond::CastSpell { who, filter }, it, p) = cast_trigger(head)? else {
            return None;
        };
        let (cards, _, tail) = parse_object_phrase("card in your graveyard")?;
        if !end(tail).is_empty() {
            return None;
        }
        let filter = Filter::and(vec![
            filter,
            Filter::SameNameAs(Box::new(Sel::All(cards))),
        ]);
        return Some((TriggerCond::CastSpell { who, filter }, it, p));
    }
    // "... while ~ has two or more quest counters on it".
    let (head, cond) = r.split_once(" while ")?;
    let (trigger, it, p) = cast_trigger(head)?;
    let cond = super::triggers::event_condition(cond)?;
    Some((
        TriggerCond::Where {
            trigger: Box::new(trigger),
            cond,
        },
        it,
        p,
    ))
}

inventory::submit! { TriggerPattern { name: "cast triggers: same name as a graveyard card, while", priority: 100, parse: cast_qualified } }
