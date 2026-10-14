//! "Whenever you cast a spell with one or more targets, draw that many cards." (Voracious
//! Bibliophile): a cast trigger for spells that have targets (CR 115.1, 115.9a), counting
//! them. The value is `kw/spell_targets.rs`.

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::spell_targets::TRIGGERING_SPELL_TARGETS;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn cast_spell_with_targets_draw(t: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = t.to_lowercase();
    if end(lower.trim()) != "whenever you cast a spell with one or more targets, draw that many cards"
    {
        return None;
    }
    // One or more targets: not a spell with zero targets.
    let has_targets = Filter::not(Filter::StackTargets(Box::new(TargetsFilter::Count(0))));
    let body = Body {
        targets: vec![],
        effect: Effect::Draw {
            who: PlayerRef::You,
            n: Value::Custom(TRIGGERING_SPELL_TARGETS.into()),
        },
        modal: None,
    };
    let tr = TriggeredAbility::new(
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: has_targets,
        },
        body,
    );
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "whenever you cast a spell with one or more targets, draw that many cards", priority: 100, parse: cast_spell_with_targets_draw } }
