//! Rules about paying a cost that don't change the cost (see `payment_rules.rs`):
//!
//! - A spell's own rules ([`CostRule`]): "X can't be 0." (CR 107.3a) on a line of its own
//!   or after the spell's effect or a keyword with {X} ("Kicker {X}. X can't be 0."),
//!   "Spend only black mana on X." ("black and/or red", "colored", with "No more than one
//!   mana of each color may be spent this way."), and "You can't spend mana to cast this
//!   spell." An activated ability's "Spend only black mana on X." is stripped from its text
//!   by [`strip_x_spend`] (see `oracle/mod.rs`).
//! - "For each {B} in a cost, you may pay 2 life rather than pay that mana." (K'rrik, Son
//!   of Yawgmoth): a modification of how its controller pays costs.

use super::costs_casting_self::this_spell_cost_ability;
use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::*;

/// "spend only black mana on x", "spend only black and/or red mana on x", "spend only
/// colored mana on x", optionally followed by ". no more than one mana of each color may
/// be spent this way" (lowercase, no final period).
pub(crate) fn parse_x_spend(s: &str) -> Option<CostRule> {
    let s = end(s);
    let (s, distinct) = match s.strip_suffix(". no more than one mana of each color may be spent this way") {
        Some(r) => (r, true),
        None => (s, false),
    };
    let words = s.strip_prefix("spend only ")?.strip_suffix(" mana on x")?;
    let colors = if words == "colored" {
        ColorSet::ALL
    } else {
        let mut set = ColorSet::NONE;
        for w in words.split(" and/or ") {
            set.insert(Color::from_word(w)?);
        }
        set
    };
    Some(CostRule::XOnlyColors { colors, distinct })
}

/// The effect text of an activated ability without its "Spend only [colors] mana on X."
/// sentences, and the rule they state: in the main text, or after every mode of a modal
/// ability ("• You gain X life. Spend only white mana on X."). `None` if it has none, or
/// if only some modes say it (a restriction on choosing those modes, not expressed here).
pub(crate) fn strip_x_spend(eff: &str) -> Option<(String, CostRule)> {
    let lower = eff.to_lowercase();
    if !lower.contains("spend only ") || !lower.contains(" mana on x.") {
        return None;
    }
    let mut rule: Option<CostRule> = None;
    let (mut modes, mut said) = (0, 0);
    let mut lines = Vec::new();
    for line in eff.lines() {
        let is_mode = line.trim_start().starts_with('•');
        modes += is_mode as usize;
        let lower = line.to_lowercase();
        let t = lower.trim_end();
        let kept = match t.find("spend only ") {
            // A sentence of its own at the end of the line.
            Some(i)
                if (i == 0 || t[..i].ends_with(". ") || t[..i].ends_with(".) "))
                    && line.is_char_boundary(i) =>
            {
                let r = parse_x_spend(&t[i..])?;
                if rule.as_ref().is_some_and(|x| *x != r) {
                    return None;
                }
                rule = Some(r);
                said += is_mode as usize;
                line[..i].trim_end().to_string()
            }
            Some(_) => return None,
            None => line.to_string(),
        };
        if !kept.trim().is_empty() {
            lines.push(kept);
        }
    }
    if said > 0 && said != modes {
        return None;
    }
    Some((lines.join("\n"), rule?))
}

/// A line stating one of the spell's rules (an ability pattern: an instant's or sorcery's
/// lines aren't tried as static abilities).
fn static_rules(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.trim().to_lowercase();
    let l = end(&lower);
    let rule = match l {
        "you can't spend mana to cast ~" | "you can't spend mana to cast this spell" => {
            CostRule::NoMana
        }
        "x can't be 0" => CostRule::XAtLeast(1),
        _ => parse_x_spend(l)?,
    };
    Some(vec![this_spell_cost_ability(CostChange::Rule(rule), None, text)])
}

inventory::submit! { AbilityPattern { name: "payment rules: X can't be 0, spend only [color] mana on X, can't spend mana to cast ~", priority: 80, parse: static_rules } }

/// "[keyword with {X} or spell effect]. X can't be 0." (Thieving Skydiver: "Kicker {X}.
/// X can't be 0."; Mind Grind: "Each opponent reveals ... X can't be 0."): the ability,
/// and the spell's rule that X can't be 0 (CR 107.3a). An activated ability's "X can't be
/// 0" is a condition of its own (see `r107_x_cant_be_zero.rs`).
fn trailing_x_cant_be_zero(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let head = block.trim_end().strip_suffix(" X can't be 0.")?;
    if !head.ends_with('.') {
        return None;
    }
    let mut out = crate::oracle::parse_ability(head, ctx)?;
    // Only keyword abilities of the spell, or the spell's own effect.
    let spell_effect = ctx.is_spell()
        && out
            .iter()
            .all(|a| matches!(a.kind, AbilityKind::Spell(_)));
    let keywords = out.iter().all(|a| a.keyword().is_some());
    if out.is_empty() || !(spell_effect || keywords) {
        return None;
    }
    out.push(this_spell_cost_ability(
        CostChange::Rule(CostRule::XAtLeast(1)),
        None,
        "X can't be 0.",
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "[ability]. X can't be 0. (a spell's rule)", priority: 80, parse: trailing_x_cant_be_zero } }

/// "For each {B} in a cost, you may pay 2 life rather than pay that mana."
fn pay_life_for_mana(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rest = end(l).strip_prefix("for each {")?;
    let (sym, rest) = rest.split_once("} in a cost, you may pay ")?;
    let life = rest.strip_suffix(" life rather than pay that mana")?;
    let color = match sym.chars().collect::<Vec<_>>().as_slice() {
        [c] => Color::from_letter(*c)?,
        _ => return None,
    };
    let life: u32 = life.parse().ok()?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected: PlayerFilter::You,
            effect: PlayerModification::PayLifeForMana { color, life },
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "for each {C} in a cost, you may pay N life rather than pay that mana", priority: 60, parse: pay_life_for_mana } }
