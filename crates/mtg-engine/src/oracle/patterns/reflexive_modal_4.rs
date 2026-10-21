//! Modal headers with extra structure (CR 700.2):
//!
//! - A header joined with an instruction: "choose one and ~ gets +1/+1 until end of turn
//!   —" (Glorfindel, Dauntless Rescuer), "choose one that hasn't been chosen and you lose
//!   2 life —" (Zuko, Conflicted). The modes are chosen as the ability is put on the
//!   stack; as it resolves, the chosen mode is performed, then the instruction (CR
//!   608.2c). With no mode chosen, the ability is removed from the stack (CR 700.2b) and
//!   the instruction isn't performed.

use super::{EffectPattern, ModalHeader, ModalHeaderPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, parse_modal, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

/// The number of times the cost was paid, and whether the player stopped paying.
const PAID: Var = vars::USER + 41020;
const STOPPED: Var = vars::USER + 41021;

/// "you may pay {G} up to three times" (Tranquil Frillback): the player pays the cost
/// once more each time they choose to, at most that many times (CR 118.12); the number
/// of payments becomes X for the reflexive ability that follows ("choose up to that
/// many", CR 603.12a).
fn pay_up_to_n_times(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let (c, n) = r.split_once(" up to ")?;
    let n = n.strip_suffix(" times")?;
    let (n, rest) = parse_number(n)?;
    let n = n.as_const()?;
    if !rest.trim().is_empty() || n < 2 {
        return None;
    }
    let cost = crate::oracle::keywords::parse_keyword_cost(c)?;
    if cost.mana.is_none() || !cost.parts.is_empty() {
        return None;
    }
    let pay_once = Effect::If {
        cond: Condition::Compare(Value::Var(STOPPED), Cmp::Eq, Value::c(0)),
        then: Box::new(Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::StoreValue {
                var: PAID,
                value: Value::Sum(vec![Value::Var(PAID), Value::c(1)]),
            }),
            otherwise: Box::new(Effect::StoreValue {
                var: STOPPED,
                value: Value::c(1),
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    Some(Effect::Seq(vec![
        Effect::StoreValue {
            var: PAID,
            value: Value::c(0),
        },
        Effect::StoreValue {
            var: STOPPED,
            value: Value::c(0),
        },
        Effect::Repeat {
            times: Value::c(n),
            effect: Box::new(pay_once),
        },
        Effect::SetX {
            value: Value::Var(PAID),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "reflexive modal 4: you may pay [mana] up to N times", priority: 100, parse: pay_up_to_n_times } }

/// Adds "other than the source" to the choice of objects an exile makes.
fn exclude_source(e: &mut Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::Choose { filter, .. },
            ..
        } => {
            let f = std::mem::replace(filter, Filter::Any);
            *filter = Filter::and(vec![
                f,
                Filter::not(Filter::Custom(
                    crate::kw::hand_graveyard_actions::SOURCE_OR_NEXT.into(),
                )),
            ]);
            true
        }
        Effect::Seq(v) => v.iter_mut().any(exclude_source),
        _ => false,
    }
}

/// "exile another card from a graveyard" (Cemetery Desecrator), "you may exile another
/// creature card from your graveyard" (Resplendent Marshal): a card other than the
/// source, which may itself be in a graveyard after a "dies" trigger (CR 400.7: the card
/// it became).
fn exile_another_from_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("exile another ")?;
    let (_, zone) = r.rsplit_once(" from ")?;
    if !zone.ends_with("graveyard") {
        return None;
    }
    let article = if r.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    let mut e = crate::oracle::effects::parse_clause(&format!("exile {article} {r}"), b)?;
    exclude_source(&mut e).then_some(e)
}

inventory::submit! { EffectPattern { name: "reflexive modal 4: exile another [card] from a graveyard", priority: 100, parse: exile_another_from_graveyard } }

/// "collect evidence X" with no X defined (Incinerator of the Guilty: "you may collect
/// evidence X. When you do, ~ deals X damage ..."): X is chosen as it's performed.
fn collect_evidence_x(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "collect evidence x" || super::value_grammar::x_defined() {
        return None;
    }
    Some(Effect::Custom(
        crate::kw::choices_4::COLLECT_EVIDENCE_X.into(),
    ))
}

inventory::submit! { EffectPattern { name: "reflexive modal 4: collect evidence X (X chosen)", priority: 100, parse: collect_evidence_x } }

/// "When it deals excess damage this way, destroy target artifact or enchantment an
/// opponent controls with mana value less than or equal to that amount of excess damage."
/// (Unleash the Inferno): a reflexive triggered ability that triggers if the damage
/// instruction dealt excess damage (CR 120.10, 603.12); "that amount of excess damage" is
/// the excess (`vars::EXCESS`).
fn when_excess_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when it deals excess damage this way, ")?;
    let r = r.replace(
        "that amount of excess damage",
        "the excess damage dealt this way",
    );
    let e = crate::oracle::effects::parse_clause(&format!("when you do, {r}"), b)?;
    let Effect::If {
        cond: Condition::PrevHappened,
        then,
        otherwise,
    } = e
    else {
        return None;
    };
    if !matches!(*then, Effect::Reflexive { .. }) {
        return None;
    }
    Some(Effect::If {
        cond: Condition::Compare(Value::Var(vars::EXCESS), Cmp::Gt, Value::c(0)),
        then,
        otherwise,
    })
}

inventory::submit! { EffectPattern { name: "reflexive modal 4: when it deals excess damage this way", priority: 100, parse: when_excess_this_way } }

/// "It fights target creature an opponent controls. When excess damage is dealt to the
/// creature an opponent controls this way, destroy up to one target ..." (Rhino's
/// Rampage): a reflexive triggered ability that triggers if the second creature of the
/// fight was dealt excess damage (CR 120.10, 603.12).
fn when_excess_to_fought(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("when excess damage is dealt to ") else {
        return false;
    };
    let Some((who, rest)) = r.split_once(" this way, ") else {
        return false;
    };
    let Effect::Fight {
        b: Sel::Target(k), ..
    } = last_fight(prev)
    else {
        return false;
    };
    let Some(spec) = b.targets.get(*k as usize) else {
        return false;
    };
    // "the creature an opponent controls": the second fighter's target description.
    let Some(noun) = who.strip_prefix("the ") else {
        return false;
    };
    if spec.text.trim_start_matches("target ") != noun {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(&format!("when you do, {rest}"), b) else {
        return false;
    };
    let Effect::If {
        cond: Condition::PrevHappened,
        then,
        otherwise,
    } = e
    else {
        return false;
    };
    if !matches!(*then, Effect::Reflexive { .. }) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::Compare(
                Value::Var(crate::excess_damage::FIGHT_EXCESS_TO_SECOND),
                Cmp::Gt,
                Value::c(0),
            ),
            then,
            otherwise,
        },
    ]);
    true
}

/// The last fight of an effect.
fn last_fight(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_fight),
        _ => e,
    }
}

inventory::submit! { super::FollowupPattern { name: "reflexive modal 4: when excess damage is dealt to the second fighter this way", priority: 100, apply: when_excess_to_fought } }

/// The value grammar's atom "the excess damage dealt this way" (see
/// [`when_excess_this_way`]).
pub fn excess_atom(s: &str) -> Option<(Value, String)> {
    let r = s.strip_prefix("the excess damage dealt this way")?;
    (r.is_empty() || r.starts_with([' ', ',', '.']))
        .then(|| (Value::Var(vars::EXCESS), r.to_string()))
}

/// "choose up to that many" (after paying a cost several times): at most X modes.
fn up_to_that_many(h: &str, _ctx: &CompileContext) -> Option<ModalHeader> {
    (h == "choose up to that many").then(|| ModalHeader {
        min: Value::c(0),
        max: Value::X,
        allow_repeat: false,
        chooser: ModeChooser::Controller,
    })
}

inventory::submit! { ModalHeaderPattern { name: "reflexive modal 4: choose up to that many", priority: 100, parse: up_to_that_many } }

/// Triggered ability bodies with a modal reflexive ability on a result of the first
/// instruction (see [`paid_times_modal`], [`removed_counter_modal`]).
pub fn reflexive_modal_bodies(
    t: &str,
    ctx: &CompileContext,
    it: &Sel,
    it_player: &PlayerRef,
) -> Option<Body> {
    paid_times_modal(t, ctx, it, it_player).or_else(|| removed_counter_modal(t, ctx, it, it_player))
}

/// Puts `reflexive` after the counter-removing option of a choice between instructions.
fn after_removal(e: &mut Effect, reflexive: &Effect) -> bool {
    match e {
        Effect::ChooseOne { options, .. } => {
            let mut found = false;
            for (_, o) in options.iter_mut() {
                if matches!(o, Effect::RemoveCounters { .. }) {
                    let removal = std::mem::take(o);
                    *o = Effect::seq(vec![removal, reflexive.clone()]);
                    found = true;
                }
            }
            found
        }
        Effect::RemoveCounters { .. } => {
            let removal = std::mem::take(e);
            *e = Effect::seq(vec![removal, reflexive.clone()]);
            true
        }
        Effect::Seq(v) => v.iter_mut().any(|x| after_removal(x, reflexive)),
        Effect::If {
            then, otherwise, ..
        } => {
            let a = after_removal(then, reflexive);
            let b = after_removal(otherwise, reflexive);
            a || b
        }
        _ => false,
    }
}

/// "put a charge counter on it or remove one from it. When you remove a counter this way,
/// choose one —" (Immard, the Stormcleaver): a modal reflexive ability that triggers only
/// if a counter was removed (CR 603.12).
pub fn removed_counter_modal(
    t: &str,
    ctx: &CompileContext,
    it: &Sel,
    it_player: &PlayerRef,
) -> Option<Body> {
    let (head, rest) = t.split_once('\n')?;
    let lower = head.to_lowercase();
    let p = ". when you remove a counter this way, choose ";
    let i = lower.find(p)?;
    let (first, header) = (&head[..i + 1], &head[i + p.len() - "choose ".len()..]);
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it.clone();
    b.it_player = it_player.clone();
    let mut effect = parse_effect_text(first, &mut b)?;
    let modal = parse_modal(&format!("{header}\n{rest}"), ctx, Some((it, it_player)))?;
    let reflexive = Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(Effect::Reflexive {
            body: Box::new(Body {
                targets: vec![],
                effect: Effect::Noop,
                modal: Some(modal),
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    if !after_removal(&mut effect, &reflexive) {
        return None;
    }
    Some(Body {
        targets: b.targets,
        effect,
        modal: None,
    })
}

/// "you may pay {G} up to three times. When you pay this cost one or more times, choose
/// up to that many —" (Tranquil Frillback), "you may pay {1} up to three times. When you
/// do, choose up to that many —" (Hawkeye, Master Marksman): a modal reflexive ability
/// that triggers once if the cost was paid at least once (CR 603.12a), with up to as
/// many modes as payments.
pub fn paid_times_modal(
    t: &str,
    ctx: &CompileContext,
    it: &Sel,
    it_player: &PlayerRef,
) -> Option<Body> {
    let (head, rest) = t.split_once('\n')?;
    let lower = head.to_lowercase();
    let (i, n) = [
        ". when you pay this cost one or more times, choose ",
        ". when you do, choose ",
    ]
    .iter()
    .find_map(|p| lower.find(p).map(|i| (i, p.len())))?;
    let (first, header) = (&head[..i + 1], &head[i + n - "choose ".len()..]);
    if !first.to_lowercase().contains(" times.") {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it.clone();
    b.it_player = it_player.clone();
    let effect = parse_effect_text(first, &mut b)?;
    // Only after paying up to N times (which sets X to the number of payments).
    if !format!("{effect:?}").contains(&format!("SetX {{ value: Var({PAID}) }}")) {
        return None;
    }
    let modal = parse_modal(&format!("{header}\n{rest}"), ctx, Some((it, it_player)))?;
    if !matches!(modal.max, Value::X) {
        return None;
    }
    let reflexive = Effect::If {
        cond: Condition::Compare(Value::X, Cmp::Ge, Value::c(1)),
        then: Box::new(Effect::Reflexive {
            body: Box::new(Body {
                targets: vec![],
                effect: Effect::Noop,
                modal: Some(modal),
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    Some(Body {
        targets: b.targets,
        effect: Effect::seq(vec![effect, reflexive]),
        modal: None,
    })
}

/// "[header] and [instruction] —\n• mode\n• mode": the modal body, with the instruction
/// as its effect (performed after the chosen modes, see `Game::exec_chosen`).
pub fn modal_with_effect(
    t: &str,
    ctx: &CompileContext,
    it: &Sel,
    it_player: &PlayerRef,
) -> Option<Body> {
    let (head, rest) = t.split_once('\n')?;
    let h = head.trim().trim_end_matches(['—', ':', ' ']);
    let lower = h.to_lowercase();
    if !lower.starts_with("choose ") {
        return None;
    }
    // The header is the words before the first " and " that leave a header.
    let mut from = 0;
    while let Some(i) = lower[from..].find(" and ") {
        let at = from + i;
        from = at + " and ".len();
        let header = &h[..at];
        let instruction = &h[at + " and ".len()..];
        let Some(modal) = parse_modal(&format!("{header}\n{rest}"), ctx, Some((it, it_player)))
        else {
            continue;
        };
        let mut b = Builder::new(ctx);
        b.in_trigger = true;
        b.it = it.clone();
        b.it_player = it_player.clone();
        let Some(effect) = parse_effect_text(instruction, &mut b) else {
            continue;
        };
        // Targets of the instruction would have to be chosen with the modes' (none here).
        if !b.targets.is_empty() || matches!(effect, Effect::Noop) {
            continue;
        }
        return Some(Body {
            targets: vec![],
            effect,
            modal: Some(modal),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn modal_headers_with_an_instruction() {
        for (ty, text) in [
            ("Legendary Creature", "Whenever you scry, choose one and ~ gets +1/+1 until end of turn.\n• ~ must be blocked this turn if able.\n• ~ can't be blocked by more than one creature each combat this turn."),
            ("Creature", "When ~ enters, you may pay {G} up to three times. When you pay this cost one or more times, choose up to that many —\n• Destroy target artifact or enchantment.\n• Exile target player's graveyard.\n• You gain 4 life."),
            ("Legendary Creature", "Whenever ~ becomes tapped, you may pay {1} up to three times. When you do, choose up to that many —\n• Net — Target creature can't block this turn.\n• Explosive — ~ deals 2 damage to target player.\n• Boomerang — Discard a card, then draw a card."),
            ("Creature", "When ~ enters or dies, exile another card from a graveyard. When you do, choose one —\n• Remove X counters from target permanent, where X is the mana value of the exiled card.\n• Target creature an opponent controls gets -X/-X until end of turn, where X is the mana value of the exiled card."),
            ("Creature", "When ~ enters or dies, you may exile another creature card from your graveyard. When you do, put a +1/+1 counter on each creature you control other than ~ that shares a creature type with the exiled card."),
            ("Creature", "Whenever ~ deals combat damage to a player, you may collect evidence X. When you do, ~ deals X damage to each creature and planeswalker that player controls."),
            ("Sorcery", "~ deals 7 damage to target creature or planeswalker. When it deals excess damage this way, destroy target artifact or enchantment an opponent controls with mana value less than or equal to that amount of excess damage."),
            ("Sorcery", "Target creature you control gets +1/+0 until end of turn. It fights target creature an opponent controls. When excess damage is dealt to the creature an opponent controls this way, destroy up to one target noncreature artifact with mana value 3 or less."),
            ("Legendary Creature", "Whenever ~ enters or attacks, put a charge counter on it or remove one from it. When you remove a counter this way, choose one —\n• ~ deals 4 damage to any target.\n• ~ gains lifelink and indestructible until end of turn."),
            ("Legendary Creature", "At the beginning of your first main phase, choose one that hasn't been chosen and you lose 2 life —\n• Draw a card.\n• Put a +1/+1 counter on ~.\n• Add {R}.\n• Exile ~, then return him to the battlefield under an opponent's control."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
