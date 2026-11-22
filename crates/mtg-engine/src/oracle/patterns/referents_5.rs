//! Referent grammar VI (round 5):
//!
//! * "Activated abilities of the chosen permanent can't be activated." (Scheming Fence):
//!   the permanent chosen as ~ entered (CR 607.2d, see `choice_grammar_etb`) is the
//!   static ability's object. A new object after it changes zones is no longer the chosen
//!   one (CR 400.7), and with nothing chosen the ability does nothing.
//! * "Change the target of target spell with a single target if that target is you."
//!   (Reflecting Mirror), "Counter target spell if its target is you": the instruction
//!   happens only if, as the ability resolves, the spell's target is you (CR 115.9b).
//! * "{X}, {T}: ... target spell ... X is twice the mana value of that spell." (Reflecting
//!   Mirror): X is defined by the target (CR 107.3c). The value of X is announced before
//!   targets are chosen (CR 601.2b, 602.2b), so the target must be a spell with that mana
//!   value; an X that no such spell gives (an odd X here) can't be announced
//!   (interpretation `x-defined-by-target`).
//! * "X is the number of artifact and/or creature cards in an opponent's graveyard as you
//!   cast ~." (Spoils of War): X is defined by the text (CR 107.3c), as the spell is cast:
//!   the number for one opponent, the caster's choice (interpretation
//!   `x-defined-for-a-player`).

use super::{AbilityPattern, EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "Activated abilities of the chosen permanent can't be activated."
fn chosen_cant_activate(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("activated abilities of the chosen ")?;
    let noun = r.strip_suffix(" can't be activated")?;
    if !matches!(noun, "permanent" | "creature")
        || !ctx.is_permanent()
        || !super::choice_grammar_etb::card_chooses_object()
    {
        return None;
    }
    let r = Restriction::CantActivate {
        who: PlayerFilter::Any,
        sources: Filter::In(Box::new(Sel::LinkedNoted)),
        include_mana: true,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "referents 5: activated abilities of the chosen permanent can't be activated", priority: 40, parse: chosen_cant_activate } }

/// "[instruction about target spell] if that target is you".
fn if_that_target_is_you(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l
        .strip_suffix(" if that target is you")
        .or_else(|| l.strip_suffix(" if its target is you"))?;
    if !head.contains("target spell") {
        return None;
    }
    let before = b.targets.len();
    let e = parse_sentence(head, b)?;
    // The one new target: the spell.
    let slot = before as u8;
    let spell = b.targets.len() == before + 1
        && matches!(&b.targets[before].what, TargetKind::Object(f) | TargetKind::Spell(f) if is_spell_filter(f));
    if !spell {
        b.targets.truncate(before);
        return None;
    }
    Some(Effect::If {
        cond: Condition::SelMatches(
            Sel::Target(slot),
            Filter::StackTargets(Box::new(TargetsFilter::Targets {
                objects: None,
                players: Some(PlayerFilter::You),
            })),
        ),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

fn is_spell_filter(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(is_spell_filter),
        _ => false,
    }
}

inventory::submit! { EffectPattern { name: "referents 5: [instruction about target spell] if that target is you", priority: 100, parse: if_that_target_is_you } }

/// "{X}, {T}: [instruction about target spell]. X is twice the mana value of that spell."
fn x_defined_by_target(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (cost, _) = text.split_once(": ")?;
    if !cost.contains("{X}") || cost.contains('"') {
        return None;
    }
    let (before, def) = text.rsplit_once(". X is ")?;
    let def = end(def).to_lowercase();
    let (k, rest) = match def.strip_prefix("twice ") {
        Some(r) => (2, r),
        None => (1, def.as_str()),
    };
    let noun = rest.strip_prefix("the mana value of that ")?;
    if !matches!(noun, "spell" | "creature" | "card" | "permanent") || before.contains(" X ") {
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
    let [t] = act.body.targets.as_mut_slice() else {
        return None;
    };
    let (TargetKind::Object(f) | TargetKind::Spell(f)) = &mut t.what else {
        return None;
    };
    let x = if k == 1 {
        Value::X
    } else {
        Value::Div(Box::new(Value::X), k, false)
    };
    *f = Filter::and(vec![
        f.clone(),
        Filter::ManaValue(Cmp::Eq, Box::new(x.clone())),
    ]);
    if k > 1 {
        // Only an X that is k times a mana value.
        let only = Condition::Compare(
            Value::Mul(Box::new(x), Box::new(Value::c(k))),
            Cmp::Eq,
            Value::X,
        );
        act.condition = Some(match act.condition.take() {
            Some(c) => Condition::And(vec![c, only]),
            None => only,
        });
    }
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "referents 5: {X} in an activation cost defined by the target", priority: 1090, parse: x_defined_by_target } }

/// "X is the number of [cards] in an opponent's graveyard as you cast ~." (a spell's line).
fn x_is_in_an_opponents_graveyard(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("x is the number of ")?;
    let noun = r.strip_suffix(" in an opponent's graveyard as you cast ~")?;
    let (f, true, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() || f.zone().is_some() {
        return None;
    }
    let v = Value::Count(Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::Iterated),
    ]));
    let rule = CostChange::Rule(CostRule::XOneOf(PlayerFilter::Opponent, Box::new(v)));
    Some(vec![super::costs_casting_self::this_spell_cost_ability(rule, None, text)])
}

inventory::submit! { AbilityPattern { name: "referents 5: X is the number of [cards] in an opponent's graveyard as you cast ~", priority: 80, parse: x_is_in_an_opponents_graveyard } }

/// "it was unattached from a Ninja" (Shuriken's granted ability, "That creature's
/// controller gains control of ~ unless it was unattached from a Ninja."): what the cost
/// "Unattach ~" unattached it from, as that permanent last existed (CR 400.7j, 608.2h).
fn unattached_from(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("it was unattached from ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(r)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::SelMatches(
        Sel::Var(crate::zones::COST_UNATTACHED_FROM),
        f,
    ))
}

inventory::submit! { super::ConditionPattern { name: "referents 5: it was unattached from a [permanent]", priority: 100, parse: unattached_from } }

/// "[effect] unless it was unattached from a Ninja": the condition is about what the
/// ability's unattach cost did (the trailing-condition grammar keeps away from pronouns).
fn unless_unattached_from(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (x, c) = l.rsplit_once(" unless ")?;
    if !c.starts_with("it was unattached from ") || x.contains(" unless ") || x.contains(" if ") {
        return None;
    }
    let cond = unattached_from(c)?;
    let e = crate::oracle::effects::parse_clause(x, b)?;
    Some(Effect::If {
        cond: Condition::Not(Box::new(cond)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "referents 5: [effect] unless it was unattached from a [permanent]", priority: 390, parse: unless_unattached_from } }

/// "that isn't the target of an ability from another creature named ~" (Goblin Artisans):
/// not a target of any ability on the stack whose source (as it last existed, CR 113.7a)
/// is such a creature (CR 115.9b).
fn target_of_an_ability_from<'a>(s: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let (negate, r) = if let Some(r) = s.strip_prefix("that isn't the target of an ability from ") {
        (true, r)
    } else {
        (false, s.strip_prefix("that's the target of an ability from ")?)
    };
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(r)? else {
        return None;
    };
    let abilities = Sel::All(Filter::and(vec![
        Filter::InZone(ZoneKind::Stack),
        Filter::AbilityFrom(Box::new(f)),
    ]));
    let targeted = Filter::TargetOf(Box::new(abilities));
    Some((
        if negate {
            Filter::not(targeted)
        } else {
            targeted
        },
        tail,
    ))
}

inventory::submit! { super::FilterSuffixPattern { name: "referents 5: that isn't the target of an ability from [objects]", priority: 100, parse: target_of_an_ability_from } }

/// "Whenever an Aura you control becomes attached to a nonland permanent an opponent
/// controls with mana value less than or equal to that Aura's mana value" (Eriette, the
/// Beguiler): the comparison is part of the trigger event, made as the Aura becomes
/// attached (not an intervening "if", CR 603.4).
fn attached_with_lesser_mana_value(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let head = end(l)
        .strip_suffix(" with mana value less than or equal to that aura's mana value")?;
    if !head.contains(" becomes attached to ") || !head.starts_with("an aura ") {
        return None;
    }
    let (t, it, player) = crate::oracle::triggers::parse_trigger_condition(&format!("whenever {head}"))?;
    if !matches!(t, TriggerCond::AttachChanged { attached: true, .. }) {
        return None;
    }
    let cond = Condition::Compare(
        Value::ManaValueOf(Box::new(Sel::TriggerOtherObject)),
        Cmp::Le,
        Value::ManaValueOf(Box::new(Sel::TriggerObject)),
    );
    Some((
        TriggerCond::Where {
            trigger: Box::new(t),
            cond,
        },
        it,
        player,
    ))
}

inventory::submit! { super::TriggerPattern { name: "referents 5: an Aura becomes attached to a permanent with lesser or equal mana value", priority: 100, parse: attached_with_lesser_mana_value } }

/// "gain control of that permanent for as long as that Aura is attached to it" in an
/// "Aura becomes attached to a permanent" trigger: the permanent, for as long as the Aura
/// stays attached to it (both fixed as the effect begins, `duration_refs`).
fn gain_control_while_aura_attached(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "gain control of that permanent for as long as that aura is attached to it"
        || !b.in_trigger
        || !crate::oracle::raw_text()
            .to_lowercase()
            .contains("aura you control becomes attached to ")
    {
        return None;
    }
    Some(Effect::GainControl {
        what: Sel::TriggerOtherObject,
        who: PlayerRef::You,
        duration: Duration::WhileCondition(Condition::SelMatches(
            Sel::TriggerObject,
            Filter::AttachedTo(Box::new(Sel::TriggerOtherObject)),
        )),
    })
}

inventory::submit! { EffectPattern { name: "referents 5: gain control of that permanent for as long as that Aura is attached to it", priority: 100, parse: gain_control_while_aura_attached } }

/// "You may cast up to one target instant card and/or up to one target sorcery card from
/// your graveyard each with mana value X or less without paying their mana costs." (Finale
/// of Promise): the target groups read as the zone-move grammar reads them ("exile up to
/// one target ... and/or up to one target ..."); the cards are cast one at a time while the
/// effect resolves, in the order their controller chooses (CR 608.2g; its ruling).
fn cast_target_groups(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // ("You may" is read around this: `Effect::May`.)
    let (optional, r) = match l.strip_prefix("you may cast ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("cast ")?),
    };
    let head = r.strip_suffix(" without paying their mana costs")?;
    if !head.contains(" and/or ") || !head.contains("target ") {
        return None;
    }
    let before = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&format!("exile {head}"), b);
    let what = match e {
        Some(Effect::Exile { what, .. })
            if matches!(&what, Sel::Union(v) if v.len() > 1 && v.iter().all(|x| matches!(x, Sel::Target(_)))) =>
        {
            what
        }
        _ => {
            b.targets.truncate(before);
            return None;
        }
    };
    // "... from your graveyard each with mana value X or less": the zone and the quality
    // are every group's (the target grammar gives them to the last one).
    if head.contains(" each with ") {
        let parts = |f: &Filter| -> Vec<Filter> {
            let v = match f {
                Filter::And(v) => v.clone(),
                other => vec![other.clone()],
            };
            v.into_iter()
                .filter(|x| {
                    matches!(
                        x,
                        Filter::InZone(_) | Filter::OwnedBy(_) | Filter::ManaValue(..)
                    )
                })
                .collect()
        };
        let Some(TargetKind::Object(last)) = b.targets.last().map(|t| t.what.clone()) else {
            b.targets.truncate(before);
            return None;
        };
        let shared = parts(&last);
        let n = b.targets.len();
        for t in &mut b.targets[before..n - 1] {
            if let TargetKind::Object(f) = &mut t.what {
                let have = parts(f);
                let mut v = vec![f.clone()];
                for x in &shared {
                    let same = |y: &Filter| std::mem::discriminant(y) == std::mem::discriminant(x);
                    if !have.iter().any(same) {
                        v.push(x.clone());
                    }
                }
                *f = Filter::and(v);
            }
        }
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what,
        free: true,
        optional,
    })
}

inventory::submit! { EffectPattern { name: "referents 5: you may cast [target groups] without paying their mana costs", priority: 100, parse: cast_target_groups } }

/// Whether the effect ends by casting cards.
fn ends_casting(e: &Effect) -> bool {
    match e {
        Effect::CastCard { .. } => true,
        Effect::May { effect, .. } => ends_casting(effect),
        Effect::Seq(v) => v
            .iter()
            .rev()
            .find(|x| !matches!(x, Effect::AddReplacement { .. }))
            .is_some_and(ends_casting),
        _ => false,
    }
}

/// "If X is 10 or more, copy each of those spells twice." / "Copy that spell twice." after
/// casting cards (Finale of Promise): the spells cast (CR 400.7h, 707.10).
fn copy_spells_cast(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_casting(prev) {
        return false;
    }
    let l = end(l);
    let (cond, r) = match l.strip_prefix("if ").and_then(|r| r.split_once(", ")) {
        Some((c, r)) => match crate::oracle::statics::parse_condition(c, b.ctx) {
            Some(c) => (Some(c), r),
            None => return false,
        },
        None => (None, l),
    };
    let Some(r) = r
        .strip_prefix("copy each of those spells ")
        .or_else(|| r.strip_prefix("copy that spell "))
    else {
        return false;
    };
    let count = match r {
        "twice" => 2,
        "three times" => 3,
        _ => return false,
    };
    let copy = Effect::CopySpell {
        what: Sel::Matching(
            Box::new(Sel::Var(vars::IT)),
            Filter::InZone(ZoneKind::Stack),
        ),
        count: Value::c(count),
        new_targets: false,
    };
    let e = match cond {
        Some(c) => Effect::If {
            cond: c,
            then: Box::new(copy),
            otherwise: Box::new(Effect::Noop),
        },
        None => copy,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "referents 5: copy each of those spells twice (after casting them)", priority: 50, apply: copy_spells_cast } }

/// "You may choose new targets for the copies." after one instruction copying a spell
/// more than once (CR 707.10c).
fn new_targets_for_the_copies(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "you may choose new targets for the copies" {
        return false;
    }
    fn last_copy(e: &mut Effect) -> Option<&mut Effect> {
        match e {
            Effect::CopySpell { count, .. } if count.as_const().is_none_or(|n| n > 1) => Some(e),
            Effect::Seq(v) => v.last_mut().and_then(last_copy),
            Effect::If {
                then, otherwise, ..
            } if matches!(**otherwise, Effect::Noop) => last_copy(then),
            _ => None,
        }
    }
    let Some(Effect::CopySpell { new_targets, .. }) = last_copy(prev) else {
        return false;
    };
    if *new_targets {
        return false;
    }
    *new_targets = true;
    true
}

inventory::submit! { FollowupPattern { name: "referents 5: you may choose new targets for the copies (one copy instruction)", priority: 95, apply: new_targets_for_the_copies } }
