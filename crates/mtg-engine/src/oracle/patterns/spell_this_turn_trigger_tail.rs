//! An instant or sorcery whose last instruction creates a delayed triggered ability with a
//! multi-sentence effect: "Target creature you control deals damage equal to its power to
//! each other creature. Until end of turn, whenever a creature you control dies, exile the
//! top card of your library. You may play it until the end of your next turn." (Waltz of
//! Rage). Everything after "until end of turn, whenever [event]," to the end of the block
//! is the delayed ability's effect (CR 603.7a, 603.7b): its later sentences ("you may play
//! it") refer to what the delayed ability did, not to the spell.

use crate::ability::*;
use crate::oracle::patterns::AbilityPattern;
use crate::oracle::CompileContext;

inventory::submit! {
    AbilityPattern { name: "spell ending in an until-end-of-turn trigger with several sentences", priority: 100, parse: spell_then_this_turn_trigger }
}

fn spell_then_this_turn_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let t = block.trim();
    let (head, tail) = t.split_once(". Until end of turn, whenever ")?;
    let tail = tail.to_lowercase();
    let (cond, body_text) = tail.split_once(", ")?;
    // A single-sentence effect is read sentence by sentence by the standard parser.
    if !body_text.trim_end_matches('.').contains(". ") || cond.contains('"') {
        return None;
    }
    let [a] = &crate::oracle::parse_ability(&format!("{head}."), ctx)?[..] else {
        return None;
    };
    let AbilityKind::Spell(mut s) = a.kind.clone() else {
        return None;
    };
    let (trigger, it, it_player) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever {cond}"))?;
    let body = crate::oracle::effects::parse_trigger_body(body_text, ctx, it, it_player)?;
    // The delayed ability's effect can't refer to the spell itself.
    let json = serde_json::to_string(&body.effect).ok()?;
    if json.contains("\"This\"") {
        return None;
    }
    s.body.effect = Effect::seq(vec![
        s.body.effect,
        Effect::DelayedTrigger {
            trigger: TriggerCond::ThisTurn(Box::new(trigger)),
            body: Box::new(body),
            once: false,
        },
    ]);
    Some(vec![AbilityDef::with_link(AbilityKind::Spell(s), t, a.link)])
}
