//! More cost modifiers (CR 601.2f, 602.2b, 118.9):
//!
//! * additional non-mana costs: "Activated abilities of nontoken Rebels cost an additional
//!   "Sacrifice a land" to activate." (Brutal Suppression), "Activated abilities cost an
//!   additional "Sacrifice a Swamp" to activate for each black mana symbol in their
//!   activation costs." and "Spells cost an additional "Sacrifice a Swamp" to cast for each
//!   black mana symbol in their mana costs." (Drought; see `kw/cost_modifier_values.rs`);
//! * a cap on a reduction: "~ costs {1} less to cast for each creature type among creatures
//!   you control. This effect can't reduce the amount of mana ~ costs by more than {5}."
//!   (Valiant Changeling);
//! * an alternative to a keyword's cost on other permanents: "You may pay {0} rather than
//!   pay the echo cost for permanents you control." (Thick-Skinned Goblin; see
//!   `kw/echo.rs`);
//! * a quality of a targeted creature in a cost condition: "if it targets a creature whose
//!   controller has eight or more cards in their graveyard" (Lullmage's Domination).

use super::{AbilityPattern, FilterSuffixPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::Color;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

/// "[subject] cost(s) an additional "[cost]" to activate|cast[ for each [color] mana symbol
/// in their activation|mana costs]".
fn additional_quoted_cost(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (subject, rest) = l
        .split_once(" cost an additional \"")
        .or_else(|| l.split_once(" costs an additional \""))?;
    let (cost_s, rest) = rest.split_once('"')?;
    let (cost, false) = crate::oracle::costs::parse_cost(cost_s)? else {
        return None;
    };
    if cost.mana.is_some() {
        return None;
    }
    let (activate, rest) = if let Some(r) = rest.strip_prefix(" to activate") {
        (true, r)
    } else {
        (false, rest.strip_prefix(" to cast")?)
    };
    let applies_to = match (activate, subject) {
        (true, "activated abilities") => CostTarget::Abilities(Filter::Any),
        (true, s) => {
            let group = s.strip_prefix("activated abilities of ")?;
            CostTarget::Abilities(super::activated_ability_costs::sources(group)?)
        }
        (false, "spells") => CostTarget::Spells(Filter::Any),
        _ => return None,
    };
    let change = match rest.trim() {
        "" => CostChange::AdditionalCost(cost),
        r => {
            let each = r.strip_prefix("for each ")?;
            let (color, tail) = each.split_once(" mana symbol in their ")?;
            let c = Color::from_word(color)?;
            let ok = match tail {
                "activation costs" => activate,
                "mana costs" => !activate,
                _ => false,
            };
            if !ok {
                return None;
            }
            CostChange::AdditionalCost(Cost::free().with(CostPart::Repeated {
                cost: Box::new(cost),
                times: Value::Custom(
                    format!(
                        "{}{}",
                        crate::kw::cost_modifier_values::COLOR_SYMBOLS,
                        c.letter()
                    )
                    .into(),
                ),
            }))
        }
    };
    Some(vec![static_ability(
        StaticEffect::CostModifier(CostModifier {
            applies_to,
            who: PlayerRel::Any,
            change,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "cost modifiers: cost an additional \"[cost]\" to activate/cast", priority: 100, parse: additional_quoted_cost } }

/// "[~ costs {1} less to cast for each ...]. This effect can't reduce the amount of mana ~
/// costs by more than {5}.": the reduction is at most that much (Valiant Changeling
/// ruling: the cap applies to this effect only).
fn reduction_cap(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let (head, cap) = t.split_once(". This effect can't reduce the amount of mana ")?;
    let cap = cap
        .strip_prefix("~ costs by more than {")
        .or_else(|| cap.strip_prefix("this spell costs by more than {"))?
        .strip_suffix("}.")?;
    let cap: i32 = cap.parse().ok()?;
    let mut abilities = crate::oracle::parse_ability(&format!("{head}."), ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Static(s) = &mut a.kind else {
        return None;
    };
    let StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        change: CostChange::ReduceGeneric(v),
        ..
    }) = &mut s.effect
    else {
        return None;
    };
    *v = Value::Min(Box::new(v.clone()), Box::new(Value::c(cap)));
    a.text = t.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "cost modifiers: this effect can't reduce the cost by more than {N}", priority: 80, parse: reduction_cap } }

/// "You may pay {0} rather than pay the echo cost for permanents you control."
fn echo_alternative(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "you may pay {0} rather than pay the echo cost for permanents you control" {
        return None;
    }
    Some(vec![static_ability(
        StaticEffect::Custom(crate::kw::echo::ECHO_FOR_ZERO.into()),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "cost modifiers: pay {0} rather than the echo cost", priority: 100, parse: echo_alternative } }

/// "whose controller has eight or more cards in their graveyard".
fn controller_graveyard<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("whose controller has ")?;
    let (cmp, n, rest) = super::statics_conditions::amount_cmp(r)?;
    let rest = rest
        .trim_start()
        .strip_prefix("cards in their graveyard")?;
    Some((
        Filter::ControllerMatches(Box::new(PlayerFilter::GraveyardSize(cmp, Box::new(n)))),
        rest,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "cost modifiers: whose controller has N or more cards in their graveyard", priority: 100, parse: controller_graveyard } }
