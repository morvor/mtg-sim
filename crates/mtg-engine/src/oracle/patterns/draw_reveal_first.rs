//! "Reveal the first card you draw each turn. Whenever you reveal a creature card this way,
//! draw a card." (Primitive Etchings, Rowen, Keranos, God of Storms, Inquisitor
//! Eisenhorn): each "Whenever you reveal a [card] this way" is a triggered ability that
//! triggers when you draw your first card of the turn (CR 121.1; "on each of your turns":
//! only during your turns) if it's that kind of card (the card is revealed as it's drawn,
//! CR 701.20a). "You may reveal the first card you draw each turn as you draw it": the
//! card is revealed only if you choose to as you draw it, and the triggered abilities
//! trigger only then.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

inventory::submit! { AbilityPattern { name: "reveal the first card you draw each turn; whenever you reveal a [card] this way", priority: 50, parse: reveal_first_draw } }

/// The static ability and the triggered abilities written after it in its paragraph are
/// linked (CR 603.11, 607.2h).
const REVEAL_FIRST_DRAW_LINK: u16 = 0x3fa9;

fn reveal_first_draw(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(block).to_lowercase();
    let (rest, your_turns, may) = if let Some(r) =
        l.strip_prefix("reveal the first card you draw each turn. ")
    {
        (r, false, false)
    } else if let Some(r) = l.strip_prefix("reveal the first card you draw on each of your turns. ")
    {
        (r, true, false)
    } else if let Some(r) =
        l.strip_prefix("you may reveal the first card you draw each turn as you draw it. ")
    {
        (r, false, true)
    } else {
        return None;
    };
    let rest = rest.strip_prefix("whenever you reveal ")?;
    let mut out = Vec::new();
    // The reveal itself: a static ability (`kw/reveal_first_draw.rs`). "You may reveal"
    // is chosen as the card is drawn, and the triggered abilities trigger only if it was
    // revealed.
    let name = if may {
        crate::kw::reveal_first_draw::MAY_EACH_TURN
    } else if your_turns {
        crate::kw::reveal_first_draw::YOUR_TURNS
    } else {
        crate::kw::reveal_first_draw::EACH_TURN
    };
    out.push(AbilityDef::with_link(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name.into()))),
        block,
        REVEAL_FIRST_DRAW_LINK,
    ));
    for part in rest.split(". whenever you reveal ") {
        let (desc, effect) = part.split_once(" this way, ")?;
        let desc = desc
            .strip_prefix("a ")
            .or_else(|| desc.strip_prefix("an "))?;
        let (filter, _, tail) = parse_object_phrase(desc)?;
        if !tail.trim().is_empty() || !desc.ends_with(" card") {
            return None;
        }
        let mut cond = vec![
            // The first card drawn this turn (`Event::Drew`'s count).
            Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(1)),
            Condition::SelMatches(Sel::TriggerObject, filter),
        ];
        if your_turns {
            cond.push(Condition::PlayerMatches(
                PlayerRef::You,
                PlayerFilter::Active,
            ));
        }
        if may {
            // Revealed only if you chose to (as it was drawn).
            cond.push(Condition::Custom(
                crate::kw::reveal_first_draw::REVEALED_THIS_WAY.into(),
            ));
        }
        let body = crate::oracle::effects::parse_trigger_body(
            effect,
            ctx,
            Sel::TriggerObject,
            PlayerRef::You,
        )?;
        let trigger = TriggerCond::Where {
            trigger: Box::new(TriggerCond::Draws {
                who: PlayerRel::You,
            }),
            cond: Condition::And(cond),
        };
        let tr = TriggeredAbility::new(trigger, body);
        out.push(AbilityDef::with_link(
            AbilityKind::Triggered(tr),
            block,
            REVEAL_FIRST_DRAW_LINK,
        ));
    }
    Some(out)
}
