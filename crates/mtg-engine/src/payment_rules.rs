//! Rules about announcing and paying a cost that don't change the cost (CR 118.7: "what a
//! player actually needs to do to pay a cost may be changed"):
//!
//! - What the text of a spell or activated ability says about its own cost
//!   ([`CostChange::Rule`]): "X can't be 0." on a spell (CR 107.3a: X is announced within
//!   what the text allows; an activated ability's "X can't be 0" is a condition on the
//!   announced X, see `oracle/patterns/r107_x_cant_be_zero.rs`), "Spend only black mana
//!   on X." (the X in the cost is still generic mana, so cost reductions can reduce it and
//!   life can't pay it — Drain Life, Helm of Awakening and K'rrik rulings), and "You can't
//!   spend mana to cast this spell." (only other ways of paying its total cost, such as
//!   convoke and delve, can pay it, CR 601.2h).
//! - A player modification of how they pay mana symbols ("For each {B} in a cost, you may
//!   pay 2 life rather than pay that mana.", [`PlayerModification::PayLifeForMana`]): it
//!   applies to every cost the player pays (spells, abilities, special actions, costs paid
//!   as a spell or ability resolves) and changes only how it's paid (CR 118.3b, 119.4).
//!   A symbol that can be paid in several ways is paid that way if the player chooses its
//!   half of that color ({B/R}, {2/B}); generic mana never can be.

use crate::ability::*;
use crate::game::Game;
use crate::mana::XSpend;
use crate::object::Characteristics;
use crate::types::*;

/// The rules the text of a spell (its static abilities that function on the stack) makes
/// about its own cost.
pub fn spell_rules(chars: &Characteristics) -> Vec<CostRule> {
    chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::CostModifier(CostModifier {
                    applies_to: CostTarget::ThisSpell,
                    change: CostChange::Rule(r),
                    ..
                }) => Some(r.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The rules an activated ability's text makes about its own cost.
pub fn ability_rules(act: &ActivatedAbility) -> Vec<CostRule> {
    act.own_cost_changes
        .iter()
        .filter_map(|c| match &c.change {
            CostChange::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect()
}

/// The least value that may be announced for X ("X can't be 0", CR 107.3a).
pub fn x_minimum(rules: &[CostRule]) -> i64 {
    rules
        .iter()
        .filter_map(|r| match r {
            CostRule::XAtLeast(n) => Some(*n as i64),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// "Spend only [colors] mana on X" for `amount` generic mana of the cost that X still
/// represents.
pub fn x_spend(rules: &[CostRule], amount: u32) -> Option<XSpend> {
    rules.iter().find_map(|r| match r {
        CostRule::XOnlyColors { colors, distinct } => Some(XSpend {
            amount,
            colors: *colors,
            distinct: *distinct,
        }),
        _ => None,
    })
}

/// "You can't spend mana to cast this spell."
pub fn no_mana(rules: &[CostRule]) -> bool {
    rules.contains(&CostRule::NoMana)
}

/// How much of a cost's generic mana X still represents after the generic mana it had
/// before reductions (`before`, X included) became `after`: a cost reduction, or a way of
/// paying generic mana other than with mana (convoke, delve), may apply to the X part
/// (Drain Life: "Cost reducers can be used to reduce the X part of the mana cost"), and the
/// player applies it there first: that leaves the most freedom in what mana pays the rest.
pub fn x_left(x: u32, before: u32, after: u32) -> u32 {
    x.saturating_sub(before.saturating_sub(after)).min(after)
}

/// The colors whose mana symbols `p` may pay with life instead of mana, and the life each
/// costs ([`PlayerModification::PayLifeForMana`]): the cheapest for each color.
pub fn life_for_mana(g: &Game, p: PlayerId) -> Vec<(Color, u32)> {
    let mut out: Vec<(Color, u32)> = Vec::new();
    for m in &g.player(p).mods {
        if let PlayerModification::PayLifeForMana { color, life } = m {
            match out.iter_mut().find(|(c, _)| c == color) {
                Some((_, n)) => *n = (*n).min(*life),
                None => out.push((*color, *life)),
            }
        }
    }
    out
}

/// Records the mana spent on the X of the spell `spell`, limited to mana of particular
/// colors ("the amount of {B} spent on X").
pub fn record_x_mana(g: &mut Game, spell: ObjectId, mana: &[crate::mana::ManaType]) {
    let o = &mut g.objects[spell.0 as usize];
    if o.zone != crate::object::Zone::Stack {
        return;
    }
    if let Some(si) = o.stack.as_mut() {
        si.cast.mana_spent_on_x.extend_from_slice(mana);
    }
}

/// `Value::Custom` prefix: "the amount of {B} spent on X" (`"mana spent on X: B"`), the
/// mana of that type spent on the resolving spell's X ([`record_x_mana`]).
pub const MANA_SPENT_ON_X: &str = "mana spent on X: ";

/// `Value::Custom` name for "the amount of [type] mana spent on X".
pub fn mana_spent_on_x_value(t: crate::mana::ManaType) -> Value {
    Value::Custom(format!("{MANA_SPENT_ON_X}{t:?}").into())
}

struct ManaSpentOnX;

impl crate::kw::KeywordRules for ManaSpentOnX {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<i64> {
        let t = name.strip_prefix(MANA_SPENT_ON_X)?;
        let t = crate::mana::ManaType::ALL
            .into_iter()
            .find(|x| format!("{x:?}") == t)?;
        Some(g.cast_info(ctx).map_or(0, |c| {
            c.mana_spent_on_x.iter().filter(|m| **m == t).count() as i64
        }))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ManaSpentOnX) }
