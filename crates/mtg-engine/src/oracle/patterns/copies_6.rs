//! Copies of cards (round 5; CR 707.12):
//!
//! * "Whenever you reveal an instant or sorcery card this way, copy that card and you may
//!   cast the copy. That copy costs {2} less to cast." (God-Eternal Kefnet): a copy of the
//!   card the trigger names, wherever it is (here the hand it was drawn into), cast while
//!   the ability resolves (CR 608.2g), with its cost reduced (CR 601.2f). Likewise "you
//!   may cast it. It costs {4} less to cast this way." in the abilities a card exiled by
//!   Uvilda, Dean of Perfection gains.
//! * "{X}, {T}: Create a token that's a copy of a card you exiled with cards named ~. X is
//!   the mana value of that card." (Caller of the Untamed): X is defined by the card the
//!   effect chooses (CR 107.3c); it's announced first (CR 601.2b, 602.2b), so it must be
//!   the mana value of one of those cards, and the card chosen is one with that mana value
//!   (interpretation `x-defined-by-target`, as for a target).

use super::{AbilityPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "copy that card" / "copy it" in a triggered ability whose trigger names a card.
fn copy_trigger_card(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(end(l), "copy that card" | "copy it") || !b.in_trigger {
        return None;
    }
    if !matches!(b.it, Sel::TriggerObject) {
        return None;
    }
    // Only a card revealed as it was drawn ("Whenever you reveal ... this way").
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("whenever you reveal ") || !raw.contains(" as you draw it") {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::TriggerObject,
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "copies 6: copy that card (the card a reveal trigger names)", priority: 50, parse: copy_trigger_card } }

/// "That copy costs {2} less to cast." after "you may cast the copy".
fn that_copy_costs_less(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let (r, copy) = match l
        .strip_prefix("that copy costs {")
        .and_then(|r| r.strip_suffix("} less to cast"))
    {
        Some(r) => (r, true),
        None => match l
            .strip_prefix("it costs {")
            .and_then(|r| r.strip_suffix("} less to cast this way"))
        {
            Some(r) => (r, false),
            None => return false,
        },
    };
    let Ok(n) = r.parse::<u32>() else {
        return false;
    };
    fn last_cast(e: &mut Effect, copy: bool) -> Option<&mut Effect> {
        match e {
            Effect::CastCard {
                what: Sel::Var(v),
                free: false,
                ..
            } if *v == vars::CREATED => Some(e),
            Effect::CastCard { free: false, .. } if !copy => Some(e),
            Effect::Seq(v) => v.last_mut().and_then(|x| last_cast(x, copy)),
            Effect::May { effect, .. } => last_cast(effect, copy),
            Effect::If {
                then, otherwise, ..
            } if matches!(**otherwise, Effect::Noop) => last_cast(then, copy),
            _ => None,
        }
    }
    let Some(cast) = last_cast(prev, copy) else {
        return false;
    };
    let inner = std::mem::take(cast);
    *cast = Effect::WithPlayTerms {
        terms: PlayTerms {
            cost_reduction: n,
            ..Default::default()
        },
        effect: Box::new(inner),
    };
    true
}

inventory::submit! { FollowupPattern { name: "copies 6: that copy costs {N} less to cast, it costs {N} less to cast this way", priority: 50, apply: that_copy_costs_less } }

/// "{X}, [cost]: [effect choosing one card]. X is the mana value of that card."
fn x_is_mana_value_of_chosen(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (cost, _) = text.split_once(": ")?;
    if !cost.contains("{X}") || cost.contains('"') {
        return None;
    }
    let before = text.strip_suffix(". X is the mana value of that card.")?;
    if before.contains(" X ") || before.contains('"') {
        return None;
    }
    let mut abilities = crate::oracle::parse_ability(&format!("{before}."), ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Activated(act) = &mut a.kind else {
        return None;
    };
    if !act.body.targets.is_empty() {
        return None;
    }
    // The one card the effect chooses.
    let mut json = serde_json::to_value(&act.body.effect).ok()?;
    let mut found: Option<Filter> = None;
    fn walk(v: &mut serde_json::Value, found: &mut Option<Filter>, n: &mut usize) {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(c) = m.get_mut("Choose") {
                    *n += 1;
                    if let Some(f) = c.get_mut("filter") {
                        if let Ok(filter) = serde_json::from_value::<Filter>(f.clone()) {
                            let with = Filter::and(vec![
                                filter.clone(),
                                Filter::ManaValue(Cmp::Eq, Box::new(Value::X)),
                            ]);
                            *found = Some(filter);
                            if let Ok(w) = serde_json::to_value(&with) {
                                *f = w;
                            }
                        }
                    }
                    return;
                }
                for (_, x) in m.iter_mut() {
                    walk(x, found, n);
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, found, n)),
            _ => {}
        }
    }
    let mut n = 0;
    walk(&mut json, &mut found, &mut n);
    let found = found?;
    if n != 1 || found.zone().is_none() {
        return None;
    }
    act.body.effect = serde_json::from_value(json).ok()?;
    let only = Condition::SelNonEmpty(Sel::All(Filter::and(vec![
        found,
        Filter::ManaValue(Cmp::Eq, Box::new(Value::X)),
    ])));
    act.condition = Some(match act.condition.take() {
        Some(c) => Condition::And(vec![c, only]),
        None => only,
    });
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "copies 6: {X} defined as the mana value of the card the effect chooses", priority: 1095, parse: x_is_mana_value_of_chosen } }
