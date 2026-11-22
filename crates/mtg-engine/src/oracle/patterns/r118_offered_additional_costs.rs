//! Optional additional costs a permanent offers for the spells its controller casts
//! (CR 118.8, 601.2b): "As an additional cost to cast green permanent spells, you may pay 2
//! life. Those spells cost {G} less to cast if you paid life this way. This effect reduces
//! only the amount of green mana you pay." (the Defilers).
//!
//! Compiled as two cost modifiers for those spells: the optional additional cost
//! (`CostChange::OptionalAdditionalCost`, announced as the spell is cast, see
//! `kw/offered_costs.rs`), and the reduction for the spells it was paid for (a filter
//! [`crate::kw::offered_costs::PAID_OFFERED_COST`] that looks at the costs paid for the
//! spell, relative to the permanent offering them), which applies after all increases
//! (CR 601.2f). The spells don't gain Phyrexian mana symbols (Defiler rulings).

use super::StaticPattern;
use crate::ability::*;
use crate::kw::offered_costs::{PAID_OFFERED_AMOUNT, PAID_OFFERED_COST};
use crate::mana::ManaCost;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::{CardType, Color};
use smol_str::SmolStr;

/// "[plural spell phrase]", e.g. "green permanent spells".
fn spells(s: &str) -> Option<Filter> {
    if s == "spells" {
        return Some(Filter::Any);
    }
    let (f, plural, tail) = parse_object_phrase(s)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    Some(f)
}

fn defiler_cost(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let r = l.strip_prefix("as an additional cost to cast ")?;
    let (spells_s, r) = r.split_once(", you may pay ")?;
    let filter = spells(spells_s)?;
    let (life, r) = parse_number(r)?;
    let Value::Const(life) = life else {
        return None;
    };
    let r = r
        .trim_start()
        .strip_prefix("life. those spells cost ")?;
    let (red, r) = r.split_once(" less to cast if you paid life this way. ")?;
    let mana = ManaCost::parse(&red.to_uppercase())?;
    // "This effect reduces only the amount of green mana you pay." (CR 118.7).
    let color = r
        .strip_prefix("this effect reduces only the amount of ")?
        .strip_suffix(" mana you pay")?;
    let only = Color::from_word(color)?;
    if mana.mana_value() != 1 || format!("{mana}") != format!("{{{}}}", only.letter()) {
        return None;
    }
    let name = SmolStr::new(format!("pay {life} life"));
    let cost = Cost::free().with(CostPart::PayLife(Value::c(life)));
    let optional = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(filter.clone()),
        who: PlayerRel::You,
        change: CostChange::OptionalAdditionalCost {
            name: name.clone(),
            cost,
        },
    }));
    let paid = Filter::Custom(SmolStr::new(format!("{PAID_OFFERED_COST}{name}")));
    let reduction = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(Filter::and(vec![filter, paid])),
        who: PlayerRel::You,
        change: CostChange::ReduceMana {
            mana,
            colored_only: true,
        },
    }));
    Some(vec![
        AbilityDef::new(AbilityKind::Static(optional), text),
        AbilityDef::new(AbilityKind::Static(reduction), text),
    ])
}

inventory::submit! { StaticPattern { name: "r118.8 as an additional cost to cast [spells], you may pay N life; those spells cost {C} less", priority: 100, parse: defiler_cost } }

/// "As an additional cost to cast creature spells, you may pay any amount of mana. If you
/// do, that creature enters with that many additional +1/+1 counters on it." (Chorus of
/// the Conclave): the amount is announced as the spell is cast (CR 601.2b; an optional
/// additional cost of {X}, see `kw/offered_costs.rs`), and a replacement effect of the
/// permanent offering it puts that many counters on the creature as it enters
/// (CR 614.1c, 122.6) — it works only while that permanent is on the battlefield (Chorus
/// ruling).
fn any_amount_of_mana(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let r = l.strip_prefix("as an additional cost to cast ")?;
    let (spells_s, r) = r.split_once(", you may pay any amount of mana. if you do, ")?;
    let filter = spells(spells_s)?;
    let (noun, r) = r.split_once(" enters with that many additional ")?;
    if noun != "that creature" || !filter_is_creature(&filter) {
        return None;
    }
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    if r.trim() != "counters on it" {
        return None;
    }
    let name = SmolStr::new("pay any amount of mana");
    let optional = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(filter.clone()),
        who: PlayerRel::You,
        change: CostChange::OptionalAdditionalCost {
            name: name.clone(),
            cost: Cost::mana(ManaCost::parse("{X}")?),
        },
    }));
    let paid = Filter::Custom(SmolStr::new(format!("{PAID_OFFERED_COST}{name}")));
    // The permanent the spell becomes, as it enters.
    let entering = crate::casting::as_spell_filter(&filter);
    let counters = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::and(vec![entering, paid])),
        action: ReplacementAction::EnterWithCounters(
            kind,
            Value::Custom(SmolStr::new(format!("{PAID_OFFERED_AMOUNT}{name}"))),
        ),
        self_replacement: false,
        optional: false,
    }));
    Some(vec![
        AbilityDef::new(AbilityKind::Static(optional), text),
        AbilityDef::new(AbilityKind::Static(counters), text),
    ])
}

/// Whether the spells a filter describes are creature spells.
fn filter_is_creature(f: &Filter) -> bool {
    match f {
        Filter::Type(CardType::Creature) => true,
        Filter::And(v) => v.iter().any(filter_is_creature),
        _ => false,
    }
}

inventory::submit! { StaticPattern { name: "r118.8 as an additional cost to cast creature spells, you may pay any amount of mana", priority: 100, parse: any_amount_of_mana } }
