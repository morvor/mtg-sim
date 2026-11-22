//! CR 702.44 Sunburst: "If this object is entering as a creature, ignoring any
//! type-changing effects that would affect it, it enters with a +1/+1 counter on it for
//! each color of mana spent to cast it. Otherwise, it enters with a charge counter on it
//! for each color of mana spent to cast it." (CR 702.44a). It adds counters only as the
//! object enters from the stack as a resolving spell that had colored mana spent on its
//! costs (CR 702.44b); it can also set the number for another ability ("Modular—
//! Sunburst", CR 702.44c, see `modular.rs`); each instance works separately
//! (CR 702.44d).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// "As this enters": sunburst's counters (+1/+1 or charge counters).
pub const SUNBURST: &str = "sunburst:enters with counters";
/// "As this enters": +1/+1 counters for each color spent, whatever its types (the number
/// of "Modular—Sunburst", CR 702.44c).
pub const SUNBURST_PLUS1: &str = "sunburst:enters with +1/+1 counters";

/// The number of colors of mana spent to cast the entering object, if it's entering from
/// the stack as a resolving spell that was cast (CR 702.44b).
fn colors_spent(ctx: &Ctx) -> u32 {
    let Some(ci) = ctx.cast.as_ref().filter(|c| c.was_cast) else {
        return 0;
    };
    let mut s = ColorSet::NONE;
    for m in &ci.mana_spent {
        if let Some(c) = m.color() {
            s.insert(c);
        }
    }
    s.count()
}

pub struct Sunburst;

impl KeywordRules for Sunburst {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Sunburst]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Custom(SmolStr::new(
                SUNBURST,
            )))),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(AbilityKind::Static(s), "Sunburst")])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let always_plus1 = match name {
            SUNBURST => false,
            SUNBURST_PLUS1 => true,
            _ => return false,
        };
        let n = colors_spent(ctx);
        if n == 0 {
            return true;
        }
        // "Entering as a creature, ignoring any type-changing effects": its copiable
        // values decide (CR 702.44a).
        let creature = always_plus1
            || ctx
                .source
                .is_some_and(|s| g.obj(s).copiable.is(CardType::Creature));
        let kind = if creature {
            counters::PLUS1
        } else {
            counters::CHARGE
        };
        if let Some(em) = ctx.entering.as_mut() {
            em.counters.push((SmolStr::new(kind), n));
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Sunburst) }
