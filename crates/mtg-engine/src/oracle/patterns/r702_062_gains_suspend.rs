//! "If it doesn't have suspend, it gains suspend." after an instruction that exiles a card
//! with time counters on it (Delay: "Counter target spell. If the spell is countered this
//! way, exile it with three time counters on it instead of putting it into its owner's
//! graveyard. If it doesn't have suspend, it gains suspend."): the card in exile gains
//! suspend, so it's suspended (CR 702.62b) and suspend's triggered abilities, which
//! function in exile, apply to it (CR 702.62a).
//!
//! The ability is granted to the object the card became in exile (CR 400.7j: other parts
//! of the effect find it there); it lasts for as long as that object exists — once the
//! card leaves exile (cast with suspend's last ability), it's a new object (CR 400.7),
//! and a creature spell cast that way has haste from suspend's last ability.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The objects of `what` that are in exile and don't have suspend gain suspend.
pub fn gains_suspend(what: Sel) -> Effect {
    let card = Sel::Var(CARD);
    Effect::ForEach {
        sel: what,
        var: CARD,
        effect: Box::new(Effect::If {
            cond: Condition::SelMatches(
                card.clone(),
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Exile),
                    Filter::Not(Box::new(Filter::HasKeyword(KeywordKind::Suspend))),
                ]),
            ),
            then: Box::new(Effect::Modify {
                what: card,
                mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Suspend))],
                duration: Duration::Permanent,
            }),
            otherwise: Box::new(Effect::Noop),
        }),
    }
}

/// Whether `e` is [`gains_suspend`]'s effect (for the renderer).
pub fn is_gains_suspend(e: &Effect) -> bool {
    matches!(e, Effect::ForEach { var, effect, .. } if *var == CARD
        && matches!(&**effect, Effect::If { then, .. }
            if matches!(&**then, Effect::Modify { mods, .. }
                if matches!(mods.as_slice(), [Modification::AddKeyword(k)] if k.kind == KeywordKind::Suspend))))
}

/// The variable iterating over the cards that may gain suspend.
const CARD: Var = vars::USER + 1762;

fn says_gains_suspend(l: &str) -> bool {
    matches!(
        end(l),
        "if it doesn't have suspend, it gains suspend"
            | "if that card doesn't have suspend, it gains suspend"
            | "then if the exiled card doesn't have suspend, it gains suspend"
            | "if the exiled card doesn't have suspend, it gains suspend"
    )
}

/// Whether the effect exiles a countered spell's card with time counters on it instead of
/// putting it into a graveyard (Delay).
fn counters_into_exile_with_time_counters(e: &Effect) -> bool {
    match e {
        Effect::SelfReplace {
            replacement:
                ReplacementDef {
                    action: ReplacementAction::MoveInstead(d),
                    ..
                },
            effect,
        } => {
            d.zone == ZoneKind::Exile
                && d.with_counters.iter().any(|(k, _)| k == crate::types::counters::TIME)
                && matches!(**effect, Effect::CounterSpell { .. })
        }
        Effect::Seq(v) => v.last().is_some_and(counters_into_exile_with_time_counters),
        _ => false,
    }
}

/// "Counter target spell. If the spell is countered this way, exile it with three time
/// counters on it instead .... If it doesn't have suspend, it gains suspend.": the
/// countered card, found in exile (`vars::IT`, CR 400.7j).
fn countered_card_gains_suspend(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !says_gains_suspend(l) || !counters_into_exile_with_time_counters(prev) {
        return false;
    }
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        gains_suspend(Sel::Var(vars::IT)),
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "k702.62 countered card exiled with time counters gains suspend", priority: 45, apply: countered_card_gains_suspend } }


/// "exile that card with three time counters on it instead of putting it into your
/// graveyard as it resolves" said of the spell a "whenever you cast" ability triggered on
/// (Gandalf of the Secret Fire): the spell gets a replacement effect of where it goes as
/// it resolves (see `kw/suspend_as_it_resolves.rs`).
fn exile_that_card_as_it_resolves(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l)
        .strip_prefix("exile that card with ")
        .or_else(|| end(l).strip_prefix("exile that spell with "))?
        .strip_suffix(" on it instead of putting it into your graveyard as it resolves")?;
    let (n, rest) = crate::oracle::phrases::parse_number(r)?;
    let Value::Const(n) = n else {
        return None;
    };
    let (kind, rest) = crate::oracle::costs::counter_kind(rest)?;
    if !matches!(rest.trim(), "counter" | "counters") || n < 1 {
        return None;
    }
    Some(Effect::Modify {
        what: Sel::TriggerSpell,
        mods: vec![crate::kw::suspend_as_it_resolves::marker(
            &kind, n as u32, false,
        )],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "exile that card with N time counters on it instead of putting it into your graveyard as it resolves", priority: 100, parse: exile_that_card_as_it_resolves } }

/// "... as it resolves. Then if the exiled card doesn't have suspend, it gains suspend."
fn then_exiled_card_gains_suspend(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !says_gains_suspend(l) {
        return false;
    }
    let Effect::Modify {
        what: Sel::TriggerSpell,
        mods,
        ..
    } = prev
    else {
        return false;
    };
    let [Modification::AddAbility(a)] = mods.as_slice() else {
        return false;
    };
    let Some((kind, n, false)) = crate::kw::suspend_as_it_resolves::parse_marker(&a.text) else {
        return false;
    };
    if kind != crate::types::counters::TIME {
        return false;
    }
    *mods = vec![crate::kw::suspend_as_it_resolves::marker(&kind, n, true)];
    true
}

inventory::submit! { FollowupPattern { name: "... as it resolves. Then if the exiled card doesn't have suspend, it gains suspend.", priority: 45, apply: then_exiled_card_gains_suspend } }
