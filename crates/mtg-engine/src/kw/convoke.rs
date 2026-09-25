//! CR 702.51 Convoke: "For each colored mana in this spell's total cost, you may tap an
//! untapped creature of that color you control rather than pay that mana. For each
//! generic mana in this spell's total cost, you may tap an untapped creature you control
//! rather than pay that mana." (CR 702.51a). It isn't an additional or alternative cost:
//! it applies once the total cost is determined (CR 702.51b). A creature tapped this way
//! "convoked" the spell (CR 702.51c). Several instances are redundant (CR 702.51d): the
//! payment is offered once per spell.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, ManaSymbol};
use crate::object::Zone;
use crate::types::*;
use smallvec::SmallVec;
use std::collections::BTreeSet;

/// `Filter::Custom`: a creature that convoked the spell (or the permanent it became)
/// whose cast information the context refers to ("each creature that convoked it").
pub const CONVOKED_IT: &str = "convoke:convoked it";
/// `Value::Custom`: the number of creatures that convoked it.
pub const CONVOKED_COUNT: &str = "convoke:number of creatures that convoked it";

/// One mana of the total cost that tapping a creature could pay instead: a colored symbol
/// (payable by a creature sharing one of its colors) or one generic mana.
#[derive(Clone, Copy, Debug)]
struct Slot {
    colors: Option<ColorSet>,
    symbol: ManaSymbol,
}

/// Splits a mana cost into slots convoke could pay (colored symbols first, then one per
/// generic mana) and the symbols it can't pay ({C}, {S}, ...).
fn slots(m: &ManaCost) -> (Vec<Slot>, Vec<ManaSymbol>) {
    let mut colored = Vec::new();
    let mut generic = Vec::new();
    let mut other = Vec::new();
    for s in &m.symbols {
        match *s {
            ManaSymbol::Generic(n) => generic.extend(std::iter::repeat_n(
                Slot {
                    colors: None,
                    symbol: ManaSymbol::Generic(1),
                },
                n as usize,
            )),
            ManaSymbol::Colored(_)
            | ManaSymbol::Hybrid(..)
            | ManaSymbol::Phyrexian(_)
            | ManaSymbol::PhyrexianHybrid(..)
            | ManaSymbol::TwoHybrid(_)
            | ManaSymbol::ColorlessHybrid(_) => colored.push(Slot {
                colors: Some(s.colors()),
                symbol: *s,
            }),
            o => other.push(o),
        }
    }
    colored.extend(generic);
    (colored, other)
}

fn can_fill(slot: &Slot, colors: ColorSet) -> bool {
    slot.colors.is_none_or(|c| c.intersects(colors))
}

/// Kuhn's augmenting-path matching of creatures (by their colors) to slots. Returns, for
/// each creature, the slot it pays (if any).
fn matching(creatures: &[ColorSet], slots: &[Slot]) -> Vec<Option<usize>> {
    fn augment(
        k: usize,
        creatures: &[ColorSet],
        slots: &[Slot],
        seen: &mut [bool],
        slot_of: &mut [Option<usize>],
    ) -> bool {
        for j in 0..slots.len() {
            if seen[j] || !can_fill(&slots[j], creatures[k]) {
                continue;
            }
            seen[j] = true;
            if slot_of[j].is_none_or(|other| augment(other, creatures, slots, seen, slot_of)) {
                slot_of[j] = Some(k);
                return true;
            }
        }
        false
    }
    let mut slot_of: Vec<Option<usize>> = vec![None; slots.len()];
    for k in 0..creatures.len() {
        let mut seen = vec![false; slots.len()];
        augment(k, creatures, slots, &mut seen, &mut slot_of);
    }
    let mut out = vec![None; creatures.len()];
    for (j, k) in slot_of.iter().enumerate() {
        if let Some(k) = k {
            out[*k] = Some(j);
        }
    }
    out
}

/// The mana cost left after creatures with these colors pay what they can, and the slot
/// each of them pays.
fn remaining(m: &ManaCost, creatures: &[ColorSet]) -> (ManaCost, Vec<Option<usize>>) {
    let (sl, other) = slots(m);
    let assign = matching(creatures, &sl);
    let used: BTreeSet<usize> = assign.iter().flatten().copied().collect();
    let mut generic = 0u32;
    let mut symbols: SmallVec<[ManaSymbol; 6]> = SmallVec::new();
    for (j, s) in sl.iter().enumerate() {
        match (used.contains(&j), s.colors) {
            (true, _) => {}
            (false, None) => generic += 1,
            (false, Some(_)) => symbols.push(s.symbol),
        }
    }
    symbols.extend(other);
    if generic > 0 {
        symbols.insert(0, ManaSymbol::Generic(generic));
    }
    (ManaCost { symbols }, assign)
}

/// Untapped creatures `p` controls, which could be tapped for convoke.
fn candidates(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| o.controller == p && o.is_creature() && !o.tapped)
        .map(|o| o.id)
        .collect()
}

fn colors_of(g: &Game, ids: &[ObjectId]) -> Vec<ColorSet> {
    ids.iter().map(|c| g.obj(*c).chars.colors).collect()
}

fn has_mana_ability(g: &Game, id: ObjectId) -> bool {
    g.obj(id)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
}

/// The default choice: no creatures if the mana can be paid otherwise; else creatures
/// (preferring ones without mana abilities) until the rest can be paid.
fn default_choice(
    g: &Game,
    p: PlayerId,
    spell: ObjectId,
    m: &ManaCost,
    cands: &[ObjectId],
) -> Vec<ObjectId> {
    let chars = g.obj(spell).chars.clone();
    let payable = |cost: &ManaCost| {
        g.can_pay_cost_optimistic(p, &Cost::mana(cost.clone()), Some(spell), &chars)
    };
    if payable(m) {
        return vec![];
    }
    let mut order = cands.to_vec();
    order.sort_by_key(|c| has_mana_ability(g, *c));
    let mut chosen: Vec<ObjectId> = Vec::new();
    for c in order {
        let mut trial = chosen.clone();
        trial.push(c);
        let (rest, assign) = remaining(m, &colors_of(g, &trial));
        // Only creatures that pay something are worth tapping.
        if assign.iter().any(|a| a.is_none()) {
            continue;
        }
        chosen = trial;
        if payable(&rest) {
            break;
        }
    }
    chosen
}

pub struct Convoke;

impl KeywordRules for Convoke {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Convoke]
    }

    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        let Some(m) = cost.mana.clone() else {
            return Ok(());
        };
        let (sl, _) = slots(&m);
        let cands = candidates(g, p);
        if sl.is_empty() || cands.is_empty() {
            return Ok(());
        }
        let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
        let max = sl.len().min(cands.len()) as u32;
        let chosen: Vec<ObjectId> = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Tap creatures to convoke".into(),
                candidates: entities.clone(),
                min: 0,
                max,
            },
        ) {
            Answer::Entities(v)
                if v.len() as u32 <= max
                    && v.iter().all(|e| entities.contains(e))
                    && v.iter().collect::<BTreeSet<_>>().len() == v.len() =>
            {
                v.iter().filter_map(|e| e.object()).collect()
            }
            _ => default_choice(g, p, spell, &m, &cands),
        };
        if chosen.is_empty() {
            return Ok(());
        }
        // Tap each chosen creature that pays part of the cost (CR 601.2h).
        let (_, assign) = remaining(&m, &colors_of(g, &chosen));
        let mut convoked = Vec::new();
        for (c, slot) in chosen.iter().zip(assign) {
            if slot.is_some() && g.obj(*c).zone == Zone::Battlefield && g.tap(*c) {
                convoked.push(*c);
            }
        }
        let (rest, _) = remaining(&m, &colors_of(g, &convoked));
        cost.mana = Some(rest);
        g.log(|g| {
            format!(
                "{p} taps {} creature(s) to convoke {}",
                convoked.len(),
                g.describe(spell)
            )
        });
        if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
            si.cast.convoked.extend(convoked);
        }
        Ok(())
    }

    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) {
        let Some(m) = cost.mana.clone() else {
            return;
        };
        let cands = candidates(g, p);
        if !cands.is_empty() {
            cost.mana = Some(remaining(&m, &colors_of(g, &cands)).0);
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == CONVOKED_IT).then(|| g.cast_info(ctx).is_some_and(|c| c.convoked.contains(&id)))
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == CONVOKED_COUNT)
            .then(|| g.cast_info(ctx).map_or(0, |c| c.convoked.len() as i64))
    }
}

inventory::submit! { KeywordRegistration(&Convoke) }
