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

use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};

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
    let mut effect = parse_effect_text(text, &mut sub)?;
    if !sub.targets.is_empty() {
        return None;
    }
    // "copy the exiled card": the card exiled by the creating ability.
    let first = match &mut effect {
        Effect::Seq(v) => v.first_mut()?,
        e => e,
    };
    let Effect::CopyCard { what, named: None } = first else {
        return None;
    };
    *what = Sel::Var(MYRA_EXILED);
    Some(Body {
        targets: vec![],
        effect,
        modal: None,
    })
}

/// "[...choose an Attraction you control ... Put a midway counter on it.] For as long as
/// that Attraction is on the battlefield, whenever you visit it, [effect]" (Myra the
/// Magnificent): the ability, with the created trigger added to its effect.
fn for_as_long_as_visit_it(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let t = block.trim();
    let (head, text) =
        t.split_once(" For as long as that Attraction is on the battlefield, whenever you visit it, ")?;
    let mut abilities = crate::oracle::parse_ability(head, ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let def = std::sync::Arc::make_mut(a);
    let body = match &mut def.kind {
        AbilityKind::Activated(x) => &mut x.body,
        AbilityKind::Spell(x) => &mut x.body,
        AbilityKind::Triggered(x) => &mut x.body,
        _ => return None,
    };
    if body.modal.is_some() {
        return None;
    }
    let attraction = chosen_attraction(&body.effect)?;
    let b = Builder::new(ctx);
    let visit = visit_body(&text.to_lowercase(), &b)?;
    if !keep_exiled(&mut body.effect, MYRA_EXILED) {
        return None;
    }
    let on = |v: Var, z: ZoneKind| Condition::SelMatches(Sel::Var(v), Filter::InZone(z));
    let trigger = TriggerCond::ForAsLongAs(
        Box::new(TriggerCond::Where {
            trigger: Box::new(TriggerCond::Custom(crate::kw::visit::YOU_VISIT.into())),
            cond: Condition::SelMatches(
                Sel::TriggerObject,
                Filter::In(Box::new(Sel::Var(MYRA_ATTRACTION))),
            ),
        }),
        // (The card's rulings: while the Attraction is on the battlefield and the card
        // remains exiled.)
        Condition::And(vec![
            on(MYRA_ATTRACTION, ZoneKind::Battlefield),
            on(MYRA_EXILED, ZoneKind::Exile),
        ]),
    );
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![
        old,
        Effect::Store {
            var: MYRA_ATTRACTION,
            sel: Sel::Var(attraction),
        },
        Effect::DelayedTrigger {
            trigger,
            body: Box::new(visit),
            once: false,
        },
    ]);
    def.text = block.to_string();
    Some(abilities)
}

inventory::submit! { super::AbilityPattern { name: "attractions 5: for as long as that Attraction is on the battlefield, whenever you visit it, ...", priority: 104, parse: for_as_long_as_visit_it } }

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
