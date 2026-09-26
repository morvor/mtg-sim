//! "When you next cast [a spell] this turn, [effect]" (Saga chapters, loyalty abilities,
//! mana abilities, spells): the effect creates a delayed triggered ability that triggers
//! only the next time its controller casts such a spell this turn (CR 603.7b: "next"
//! makes it trigger only once; "this turn" ends it at the cleanup step).
//!
//! * "... copy that spell [twice | an additional time]" (CR 707.10) and the following
//!   "You may choose new targets for the copy/copies." (CR 707.10c).
//! * Any other effect the trigger body parser understands for a cast trigger ("it gains
//!   haste until end of turn"), with "that spell"/"it" meaning the spell cast.

use super::{AbilityPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Splits "when you next cast [spell] this turn, [effect]" into the spell phrase and the
/// effect text.
fn next_cast_parts(l: &str) -> Option<(&str, &str)> {
    let r = l.strip_prefix("when you next cast ")?;
    let (spell, eff) = r.split_once(" this turn, ")?;
    if spell.contains(" or activate ") || spell.contains(", cast ") {
        return None;
    }
    Some((spell, eff))
}

fn p_next_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (spell, eff) = next_cast_parts(l)?;
    let (trigger, it, it_player) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever you cast {spell}"))?;
    let cast = |t: &TriggerCond| {
        matches!(
            t,
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                ..
            }
        )
    };
    let ok = match &trigger {
        TriggerCond::Where { trigger: t, .. } => cast(t),
        t => cast(t),
    };
    if !ok {
        return None;
    }
    let body = crate::oracle::effects::parse_trigger_body(eff, b.ctx, it, it_player)?;
    if body.modal.is_some() {
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
        body: Box::new(body),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "levels_classes_sagas: when you next cast", priority: 90, parse: p_next_cast } }

/// An instant or sorcery whose text begins "When you next cast ...": a spell ability that
/// creates the delayed trigger, not a triggered ability of the card.
fn spell_next_cast_line(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let text = block.trim();
    if !text.to_lowercase().starts_with("when you next cast ") {
        return None;
    }
    let body = crate::oracle::effects::parse_body(text, ctx)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility { body }),
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "levels_classes_sagas: spell when you next cast", priority: 90, parse: spell_next_cast_line } }

/// "that creature enters with an additional +1/+1 counter on it", "it enters with two
/// additional +1/+1 counters on it" in an ability that triggers on casting a creature
/// spell: a replacement effect for the permanent that spell becomes (CR 614.1c, 400.7a).
fn p_spell_enters_with_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let l = end(l);
    let r = l
        .strip_prefix("that creature enters with ")
        .or_else(|| l.strip_prefix("it enters with "))?;
    let (n, r) = match r.strip_prefix("an additional ") {
        Some(x) => (Value::c(1), x),
        None => {
            let (n, x) = parse_number(r)?;
            n.as_const()?;
            (n, x.trim_start().strip_prefix("additional ")?)
        }
    };
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    if !matches!(r.trim(), "counter on it" | "counters on it") {
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::In(Box::new(Sel::TriggerSpell))),
            action: ReplacementAction::EnterWithCounters(kind, n),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::Permanent,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "levels_classes_sagas: cast creature enters with counters", priority: 90, parse: p_spell_enters_with_counters } }

/// "copy that spell twice", "copy it twice", "copy that spell an additional time" in an
/// ability that triggers on casting a spell (CR 707.10).
fn p_copy_trigger_spell_times(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let l = end(l);
    let (l, new_targets) = match l
        .strip_suffix(" and you may choose new targets for the copy")
        .or_else(|| l.strip_suffix(" and you may choose new targets for the copies"))
    {
        Some(x) => (x, true),
        None => (l, false),
    };
    let r = l
        .strip_prefix("copy that spell")
        .or_else(|| l.strip_prefix("copy it"))?;
    let n = match r.trim() {
        // The plain "copy it" is the core pattern's, unless new targets follow.
        "" if new_targets => 1,
        "an additional time" | "once" => 1,
        "twice" => 2,
        "three times" => 3,
        _ => return None,
    };
    Some(Effect::CopySpell {
        what: Sel::TriggerSpell,
        count: Value::c(n),
        new_targets,
    })
}

inventory::submit! { EffectPattern { name: "levels_classes_sagas: copy that spell twice", priority: 90, parse: p_copy_trigger_spell_times } }

/// "You may choose new targets for the copy/copies." after "when you next cast ..., copy
/// that spell": the copy the delayed triggered ability makes (CR 707.10c).
fn f_next_cast_new_targets(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if !matches!(
        end(&l),
        "you may choose new targets for the copy" | "you may choose new targets for the copies"
    ) {
        return false;
    }
    // "If you do, when you next cast ...": the delayed trigger inside the conditional.
    let last = match last_effect_mut(prev) {
        Some(Effect::If {
            then,
            otherwise,
            ..
        }) if matches!(**otherwise, Effect::Noop) => last_effect_mut(then),
        other => other,
    };
    let Some(Effect::DelayedTrigger { trigger, body, .. }) = last else {
        return false;
    };
    if !matches!(trigger, TriggerCond::ThisTurn(_)) {
        return false;
    }
    match last_effect_mut(&mut body.effect) {
        Some(Effect::CopySpell { new_targets, .. }) => {
            *new_targets = true;
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "levels_classes_sagas: next cast copy new targets", priority: 90, apply: f_next_cast_new_targets } }

/// The last effect of a sequence (in execution order).
fn last_effect_mut(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_effect_mut),
        other => Some(other),
    }
}
