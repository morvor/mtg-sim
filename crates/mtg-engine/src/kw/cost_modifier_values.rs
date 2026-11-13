//! Values cost-modifying effects count about the spell or ability whose cost they change
//! (CR 601.2f, 602.2b):
//!
//! * [`SPELL_TARGETS`]: "Spells you cast cost {1} less to cast for each target." (Hinata,
//!   Dawn-Crowned): the targets of the spell being cast, once chosen (CR 601.2c — before
//!   that, none);
//! * [`COLOR_SYMBOLS`]: "Activated abilities cost an additional "Sacrifice a Swamp" to
//!   activate for each black mana symbol in their activation costs." / "Spells cost an
//!   additional ... for each black mana symbol in their mana costs." (Drought): the mana
//!   symbols of that color in the activation cost of the ability being activated (the
//!   context's ability), or in the mana cost of the spell being cast. Hybrid and Phyrexian
//!   symbols of that color count (CR 107.4e–f, 202.2).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::types::*;

/// Context variables: the source and the ability uid of the activated ability whose cost
/// a cost-modifying effect is changing.
const ABILITY_SOURCE: Var = vars::USER + 4250;
const ABILITY_UID: Var = vars::USER + 4251;

/// Notes in the context of a cost-modifying effect the ability whose cost it changes.
pub fn note_ability(ctx: &mut Ctx, src: ObjectId, uid: u64) {
    ctx.vars.insert(ABILITY_SOURCE, vec![Entity::Object(src)]);
    ctx.nums.insert(ABILITY_UID, uid as i64);
}

/// `Value::Custom` name: the number of targets of the spell being cast.
pub const SPELL_TARGETS: &str = "cost: targets of the spell being cast";

/// `Value::Custom` name prefix, then a color letter: the mana symbols of that color in the
/// cost of the spell being cast or the ability being activated.
pub const COLOR_SYMBOLS: &str = "cost: mana symbols of color in the cost:";

fn symbols_of(m: &ManaCost, c: Color) -> i64 {
    m.symbols.iter().filter(|s| s.colors().contains(c)).count() as i64
}

/// The spell a cost change is about: the spell being cast (the event's spell for an effect
/// of another object, or the context's source for its own).
fn spell_of(g: &Game, ctx: &Ctx) -> Option<ObjectId> {
    ctx.event
        .as_ref()
        .and_then(|e| e.spell)
        .or(ctx.source.filter(|s| {
            matches!(
                g.obj(*s).stack.as_deref().map(|si| &si.kind),
                Some(crate::object::StackKind::Spell)
            )
        }))
}

pub struct CostModifierValues;

impl KeywordRules for CostModifierValues {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == SPELL_TARGETS {
            // The players and objects that are targets, each counted once (Hinata ruling:
            // "target creature" twice targeting the same creature counts one).
            let mut seen: Vec<Entity> = Vec::new();
            if let Some(si) = spell_of(g, ctx).and_then(|s| g.obj(s).stack.as_deref()) {
                for e in si.chosen.iter().flat_map(|cm| cm.targets.iter().flatten()) {
                    if !seen.contains(e) {
                        seen.push(*e);
                    }
                }
            }
            return Some(seen.len() as i64);
        }
        let c = name
            .strip_prefix(COLOR_SYMBOLS)?
            .chars()
            .next()
            .and_then(Color::from_letter)?;
        // The ability being activated: noted by the cost computation, or the context's own.
        let noted = match (ctx.vars.get(&ABILITY_SOURCE), ctx.nums.get(&ABILITY_UID)) {
            (Some(v), Some(uid)) => v.first().and_then(|e| e.object()).map(|s| (s, *uid as u64)),
            _ => None,
        };
        let ability = noted.or_else(|| {
            (ctx.ability_uid != 0)
                .then_some(ctx.source)
                .flatten()
                .map(|s| (s, ctx.ability_uid))
        });
        if let Some((src, uid)) = ability {
            {
                let act = g
                    .obj(src)
                    .chars
                    .abilities
                    .iter()
                    .find_map(|a| match &a.kind {
                        AbilityKind::Activated(act) if a.uid == uid => Some(act),
                        _ => None,
                    });
                if let Some(act) = act {
                    return Some(act.cost.mana.as_ref().map_or(0, |m| symbols_of(m, c)));
                }
            }
        }
        let spell = spell_of(g, ctx)?;
        Some(
            g.obj(spell)
                .chars
                .mana_cost
                .as_ref()
                .map_or(0, |m| symbols_of(m, c)),
        )
    }
}

inventory::submit! { KeywordRegistration(&CostModifierValues) }
