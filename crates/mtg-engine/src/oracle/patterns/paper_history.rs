//! History conditions and values of old (paper) card wordings (see `kw::paper_history`):
//!
//! * "if it wasn't the first land you played this turn" (Fastbond's land-play trigger):
//!   you played another land earlier this turn (CR 305.2; its rulings: only lands played,
//!   not lands put onto the battlefield);
//! * "if ~ started the turn untapped" (Rasputin Dreamweaver);
//! * "if ~ regenerated this turn", "for each time it regenerated this turn" (Spiny
//!   Starfish);
//! * "each opponent and planeswalker it has dealt damage to this game" (The Fallen);
//! * "if it wasn't sacrificed" (Urza's Miter), about the trigger's object.

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::kw::paper_history as h;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn custom(name: &str) -> Condition {
    Condition::Custom(name.into())
}

fn history_condition(c: &str) -> Option<Condition> {
    Some(match end(c.trim()) {
        "it wasn't the first land you played this turn" => Condition::SelMatches(
            Sel::TriggerObject,
            Filter::Custom(h::NOT_FIRST_LAND_PLAYED.into()),
        ),
        "~ started the turn untapped" => custom(h::STARTED_TURN_UNTAPPED),
        "~ regenerated this turn" => Condition::Compare(
            Value::Custom(h::REGENERATIONS_THIS_TURN.into()),
            Cmp::Ge,
            Value::c(1),
        ),
        "it wasn't sacrificed" => Condition::SelMatches(
            Sel::TriggerObject,
            Filter::Custom(h::NOT_SACRIFICED.into()),
        ),
        _ => return None,
    })
}

inventory::submit! { ConditionPattern { name: "paper history: first land played, started the turn untapped, regenerated this turn, wasn't sacrificed", priority: 100, parse: history_condition } }

/// "[instruction] for each time it regenerated this turn" (Spiny Starfish: "create a 0/1
/// blue Starfish creature token for each time it regenerated this turn"): the amount,
/// counted as the effect happens (its rulings).
fn for_each_time_regenerated(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (head, who) = l
        .strip_suffix(" regenerated this turn")?
        .rsplit_once(" for each time ")?;
    if !(who == "~" || (who == "it" && matches!(b.it, Sel::This | Sel::None))) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::effects::parse_clause(head, b).and_then(|e| {
        super::damage_removal_foreach::multiply(
            e,
            Value::Custom(h::REGENERATIONS_THIS_TURN.into()),
        )
    });
    if e.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    e
}

inventory::submit! { EffectPattern { name: "paper history: [instruction] for each time it regenerated this turn", priority: 100, parse: for_each_time_regenerated } }

/// "~ deals N damage to each opponent and planeswalker it has dealt damage to this game".
fn damage_to_each_damaged(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("~ deals ")?;
    let (amount, r) = r.split_once(" damage to ")?;
    if r != "each opponent and planeswalker it has dealt damage to this game" {
        return None;
    }
    let (n, rest) = crate::oracle::phrases::parse_number(amount)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let _ = b;
    let damaged = || Condition::Custom(h::DAMAGED_BY_SOURCE.into());
    Some(Effect::DealDamage {
        source: Sel::This,
        amount: n,
        to: Sel::Union(vec![
            Sel::Players(PlayerRef::Each(PlayerFilter::And(vec![
                PlayerFilter::Opponent,
                PlayerFilter::History(Box::new(damaged())),
            ]))),
            Sel::All(Filter::and(vec![
                Filter::Type(crate::types::CardType::Planeswalker),
                Filter::Custom(h::DAMAGED_BY_SOURCE.into()),
            ])),
        ]),
    })
}

inventory::submit! { EffectPattern { name: "paper history: each opponent and planeswalker it has dealt damage to this game", priority: 100, parse: damage_to_each_damaged } }
