//! Reflexive triggered abilities (CR 603.12) whose event is a result of an earlier
//! instruction, and sentences that continue a reflexive ability:
//!
//! - "When you discard one or more nonland cards this way, tap up to that many target
//!   creatures and put a stun counter on each of them." (Seasoned Cryomancer), "When you
//!   discard one or more cards this way, ~ deals twice that much damage to target
//!   creature." (Borborygmos and Fblthp): it triggers once if the instruction discarded
//!   such cards; "that many" is how many (CR 603.12a).
//! - "When a creature is put onto the battlefield this way, it deals damage equal to its
//!   power to target creature an opponent controls." (Vivien's Invocation): "it" is that
//!   creature.
//! - "Goad that creature." after "When you do, target creature that player controls gets
//!   +2/+2 ..." (Kitt Kanto, Mayhem Diva): part of the reflexive ability, about its target.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, parse_sentence, Builder};
use crate::oracle::phrases::end;

/// The reflexive ability `e` ends with, after "When you do".
fn last_reflexive(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_reflexive),
        Effect::If {
            cond: Condition::PrevHappened,
            then,
            otherwise,
        } if matches!(**otherwise, Effect::Noop) => match &mut **then {
            Effect::Reflexive { body } => Some(body),
            _ => None,
        },
        _ => None,
    }
}

/// "Goad that creature.", "Untap that creature." after a reflexive ability with one
/// object target.
fn reflexive_target_again(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if l.contains("target") || l.starts_with("that creature ") || l.starts_with("it ") {
        return false;
    }
    let Some(at) = l.find("that creature") else {
        return false;
    };
    let rest = &l[at + "that creature".len()..];
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return false;
    }
    let Some(body) = last_reflexive(prev) else {
        return false;
    };
    if body.targets.len() != 1 || body.modal.is_some() {
        return false;
    }
    if !matches!(body.targets[0].what, TargetKind::Object(_)) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Target(0);
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    sub.targets = body.targets.clone();
    let text = format!("{}it{}", &l[..at], rest);
    let Some(e) = parse_sentence(&text, &mut sub) else {
        return false;
    };
    if sub.targets.len() != 1 || !format!("{e:?}").contains("Target(0)") {
        return false;
    }
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "followup reflexive: a later instruction about its target", priority: 0, apply: reflexive_target_again } }

/// The cards the last instruction of `e` discarded, if it's a discard by you.
fn discards(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v
            .iter()
            .rev()
            .find(|x| !matches!(x, Effect::Store { .. } | Effect::StoreValue { .. } | Effect::Custom(_)))
            .is_some_and(discards),
        Effect::May { effect, .. } => discards(effect),
        Effect::Discard {
            who: PlayerRef::You,
            ..
        } => true,
        _ => false,
    }
}

/// The reflexive ability's text with "that many" / "that much" as X.
fn that_many_as_x(r: &str) -> String {
    r.replace("twice that much damage", "twice x damage")
        .replace("that much damage", "x damage")
        .replace("up to that many ", "up to x ")
        .replace("that many", "x")
}

/// "When you discard one or more [cards] this way, [effect]".
fn when_you_discard_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("when you discard one or more ") else {
        return false;
    };
    let Some((what, effect)) = r.split_once(" this way, ") else {
        return false;
    };
    if !discards(prev) {
        return false;
    }
    let q = match what {
        "cards" => Filter::Any,
        w => {
            let Some(q) = super::card_flow_search::card_filter(w, b) else {
                return false;
            };
            q
        }
    };
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    let Some(body) = parse_effect_text(&that_many_as_x(effect), &mut sub) else {
        return false;
    };
    let n = Value::CountSel(Box::new(Sel::Matching(
        Box::new(Sel::Var(crate::discard_rules::DISCARDED)),
        q,
    )));
    let reflexive = Effect::If {
        cond: Condition::Compare(n.clone(), Cmp::Ge, Value::c(1)),
        then: Box::new(Effect::seq(vec![
            Effect::SetX { value: n },
            Effect::Reflexive {
                body: Box::new(Body {
                    targets: sub.targets,
                    effect: body,
                    modal: None,
                }),
            },
        ])),
        otherwise: Box::new(Effect::Noop),
    };
    // Nothing discarded yet in this resolution counts: the record starts empty.
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: crate::discard_rules::DISCARDED,
            sel: Sel::None,
        },
        old,
        reflexive,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followup reflexive: when you discard one or more cards this way", priority: 0, apply: when_you_discard_this_way } }

/// "When a creature is put onto the battlefield this way, it deals damage ..." after an
/// instruction that may have put one there.
fn when_put_onto_battlefield(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(effect) = l.strip_prefix("when a creature is put onto the battlefield this way, ")
    else {
        return false;
    };
    if !format!("{prev:?}").contains("zone: Battlefield") {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    let Some(body) = parse_effect_text(effect, &mut sub) else {
        return false;
    };
    let entered = Sel::Matching(
        Box::new(Sel::Var(vars::IT)),
        Filter::and(vec![
            Filter::Type(crate::types::CardType::Creature),
            Filter::InZone(ZoneKind::Battlefield),
        ]),
    );
    let reflexive = Effect::If {
        cond: Condition::SelNonEmpty(entered),
        then: Box::new(Effect::Reflexive {
            body: Box::new(Body {
                targets: sub.targets,
                effect: body,
                modal: None,
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, reflexive]);
    true
}

inventory::submit! { FollowupPattern { name: "followup reflexive: when a creature is put onto the battlefield this way", priority: 0, apply: when_put_onto_battlefield } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn reflexive_results() {
        for (ty, text) in [
            ("Creature", "When ~ enters, draw two cards, then discard two cards. When you discard one or more nonland cards this way, tap up to that many target creatures and put a stun counter on each of them."),
            ("Creature", "At the beginning of combat on each player's turn, you may tap two untapped creatures you control. When you do, target creature that player controls gets +2/+2 and gains trample until end of turn. Goad that creature."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
