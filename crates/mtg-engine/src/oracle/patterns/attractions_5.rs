//! Attractions, grammar V (cards5-game-rules-dice):
//!
//! - "Exile target instant or sorcery card ... and choose an Attraction you control .... Put
//!   a midway counter on it. For as long as that Attraction is on the battlefield, whenever
//!   you visit it, copy the exiled card. You may cast the copy without paying its mana
//!   cost." (Myra the Magnificent): a triggered ability created by the effect (CR 603.7)
//!   for visits of the chosen Attraction (CR 701.52a), lasting while it's on the
//!   battlefield and the card stays exiled (the card's rulings).
//! - Drop Tower's and Ferris Wheel's durations ending on a die roll and the "phased out
//!   with this Attraction" history are in `dice_coins_5`.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// Where the created visit trigger keeps the exiled card and the chosen Attraction.
const MYRA_EXILED: Var = vars::USER + 43401;
const MYRA_ATTRACTION: Var = vars::USER + 43402;

/// The variable an earlier "choose an Attraction you control ..." stored its choice in.
fn chosen_attraction(e: &Effect) -> Option<Var> {
    match e {
        Effect::Seq(v) => v.iter().rev().find_map(chosen_attraction),
        Effect::Store {
            var,
            sel: Sel::Choose { filter, .. },
        } if serde_json::to_string(filter).is_ok_and(|j| j.contains("\"Attraction\"")) => {
            Some(*var)
        }
        _ => None,
    }
}

/// Puts `Store { var, sel: it }` right after the first exile of a target in `e`.
fn keep_exiled(e: &mut Effect, var: Var) -> bool {
    let Effect::Seq(v) = e else {
        return false;
    };
    let Some(i) = v.iter().position(|x| {
        matches!(
            x,
            Effect::Exile {
                what: Sel::Target(_),
                ..
            }
        )
    }) else {
        return false;
    };
    v.insert(
        i + 1,
        Effect::Store {
            var,
            sel: Sel::Var(vars::IT),
        },
    );
    true
}

/// The body of the created visit trigger: `text`, with "the exiled card" the card kept in
/// [`MYRA_EXILED`].
fn visit_body(text: &str, b: &Builder) -> Option<Body> {
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::TriggerObject;
    let effect = parse_effect_text(text, &mut sub)?;
    if !sub.targets.is_empty() {
        return None;
    }
    // "copy the exiled card": the card exiled by the creating ability.
    let json = serde_json::to_string(&effect).ok()?;
    let copy = "{\"CopyCard\":{\"what\":{\"Var\":0}";
    if !json.contains(copy) {
        return None;
    }
    let json = json.replace(copy, &format!("{{\"CopyCard\":{{\"what\":{{\"Var\":{MYRA_EXILED}}}"));
    Some(Body {
        targets: vec![],
        effect: serde_json::from_str(&json).ok()?,
        modal: None,
    })
}

/// The created trigger's text so far (see [`for_as_long_as_visit_it`]).
const COPY_SENTENCE: &str = "copy the exiled card.";

fn for_as_long_as_visit_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("for as long as that attraction is on the battlefield, whenever you visit it, ") else {
        return continue_visit_trigger(l, prev, b);
    };
    if format!("{r}.") != COPY_SENTENCE {
        return false;
    }
    let Some(attraction) = chosen_attraction(prev) else {
        return false;
    };
    let Some(body) = visit_body(COPY_SENTENCE, b) else {
        return false;
    };
    if !keep_exiled(prev, MYRA_EXILED) {
        return false;
    }
    let on_battlefield = |v: Var, z: ZoneKind| {
        Condition::SelMatches(Sel::Var(v), Filter::InZone(z))
    };
    let trigger = TriggerCond::ForAsLongAs(
        Box::new(TriggerCond::Where {
            trigger: Box::new(TriggerCond::Custom(crate::kw::visit::YOU_VISIT.into())),
            cond: Condition::SelMatches(
                Sel::TriggerObject,
                Filter::In(Box::new(Sel::Var(MYRA_ATTRACTION))),
            ),
        }),
        Condition::And(vec![
            on_battlefield(MYRA_ATTRACTION, ZoneKind::Battlefield),
            on_battlefield(MYRA_EXILED, ZoneKind::Exile),
        ]),
    );
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Store {
            var: MYRA_ATTRACTION,
            sel: Sel::Var(attraction),
        },
        Effect::DelayedTrigger {
            trigger,
            body: Box::new(body),
            once: false,
        },
    ]);
    true
}

/// "You may cast the copy without paying its mana cost." after the copy: part of the
/// created trigger.
fn continue_visit_trigger(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if l != "you may cast the copy without paying its mana cost" {
        return false;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(Effect::DelayedTrigger { trigger, body, .. }) = v.last_mut() else {
        return false;
    };
    let ours = matches!(trigger, TriggerCond::ForAsLongAs(t, _)
        if matches!(t.as_ref(), TriggerCond::Where { trigger, .. }
            if matches!(trigger.as_ref(), TriggerCond::Custom(n) if n == crate::kw::visit::YOU_VISIT)));
    if !ours || !matches!(body.effect, Effect::CopyCard { .. }) {
        return false;
    }
    let Some(new) = visit_body(&format!("{COPY_SENTENCE} {l}."), b) else {
        return false;
    };
    **body = new;
    true
}

inventory::submit! { FollowupPattern { name: "attractions 5: for as long as that Attraction is on the battlefield, whenever you visit it, ...", priority: 50, apply: for_as_long_as_visit_it } }

/// "Whenever you roll a die, you may reveal the top card of your library. Do this only
/// once each turn. Whenever you reveal a card with mana value less than the result this
/// way, you may exile it. If you do, you may play it this turn." (Priority Boarding): the
/// second triggered ability triggers on the first one's reveal (CR 603.2, 607.2), with
/// "the result" the roll that triggered the first. It's compiled into the first ability:
/// if this permanent is still on the battlefield as the card is revealed (its triggered
/// abilities trigger only then, CR 603.6) and the card's mana value is less than the
/// result, a triggered ability triggers then (CR 603.12's timing) and does the rest.
fn reveal_then_whenever_you_reveal(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let t = block.trim();
    let (head, rest) =
        t.split_once(" Whenever you reveal a card with mana value less than the result this way, ")?;
    let lower = head.to_lowercase();
    if !lower.starts_with("whenever you roll a die, you may reveal the top card of your library.") {
        return None;
    }
    let mut abilities = crate::oracle::parse_ability(head, ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let def = std::sync::Arc::make_mut(a);
    let AbilityKind::Triggered(tr) = &mut def.kind else {
        return None;
    };
    // The reveal is the ability's only instruction.
    let reveals = matches!(&tr.body.effect, Effect::May { effect, .. }
        if matches!(effect.as_ref(), Effect::Dig { reveal: true, n: Value::Const(1), take: Value::Const(0), .. }));
    if !reveals || !tr.body.targets.is_empty() {
        return None;
    }
    let b = Builder::new(ctx);
    let body = super::r600_triggers::reflexive_body_about(
        &rest.to_lowercase(),
        &b,
        Sel::Var(vars::DUG),
    )?;
    let revealed = Sel::Var(vars::DUG);
    let cond = Condition::And(vec![
        Condition::SelNonEmpty(revealed.clone()),
        Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Battlefield)),
        Condition::SelMatches(
            revealed,
            Filter::ManaValue(
                Cmp::Lt,
                Box::new(Value::Custom(crate::dice::THE_RESULT.into())),
            ),
        ),
    ]);
    let old = std::mem::take(&mut tr.body.effect);
    tr.body.effect = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(Effect::Reflexive {
                body: Box::new(body),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    def.text = block.to_string();
    Some(abilities)
}

inventory::submit! { super::AbilityPattern { name: "attractions 5: whenever you reveal a card with mana value less than the result this way", priority: 104, parse: reveal_then_whenever_you_reveal } }
