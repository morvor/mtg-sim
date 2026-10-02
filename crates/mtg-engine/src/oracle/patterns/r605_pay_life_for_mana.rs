//! "Until end of turn, any time you could activate a mana ability, you may pay N life. If
//! you do, add {C}." (Channel): until end of turn, the spell's controller may repeatedly
//! pay life for mana, an action an effect allows later (CR 116.2c) that doesn't use the
//! stack. The engine offers it whenever that player has priority, one of the times a
//! player could activate a mana ability (CR 605.3a), so its mana is added to the pool
//! before a spell is cast or an ability activated. Paying the life follows CR 119.4: no
//! more than the player's life total, so none at all once it's 0.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn pay_life_for_mana(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let lower = block.trim().to_lowercase();
    let rest =
        lower.strip_prefix("until end of turn, any time you could activate a mana ability, you may ")?;
    let (cost, then) = rest.split_once(". if you do, ")?;
    let (cost, loyalty) = crate::oracle::costs::parse_cost(cost)?;
    if loyalty
        || cost.mana.is_some()
        || !cost.parts.iter().all(|p| matches!(p, CostPart::PayLife(_)))
    {
        return None;
    }
    let mut b = Builder::new(ctx);
    let then = parse_effect_text(end(then), &mut b)?;
    if !matches!(then, Effect::AddMana { .. }) || !b.targets.is_empty() {
        return None;
    }
    let offer = Effect::OfferSpecialAction {
        def: Box::new(SpecialActionDef {
            who: PlayerFilter::You,
            cost,
            action: SpecialActionEffect::Effect(then),
        }),
        duration: Duration::EndOfTurn,
        repeatable: true,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body {
                targets: Vec::new(),
                effect: offer,
                modal: None,
            },
        }),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "pay life for mana any time you could activate a mana ability", priority: 0, parse: pay_life_for_mana } }
