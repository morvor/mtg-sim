//! Heirloom Epic: "{4}, {T}: Draw a card. For each mana in this ability's activation cost,
//! you may tap an untapped creature you control rather than pay that mana. Activate only as
//! a sorcery." Like convoke for an ability (CR 702.51 is for spells): each creature tapped
//! pays for one generic mana of the total cost; any untapped creature may be tapped, and at
//! most as many as the mana in the cost (rulings). They're tapped as the total cost is paid,
//! after mana abilities are activated (CR 601.2g–h): see `Game::paid_otherwise`.

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::ManaCost;
use crate::types::{Entity, ObjectId, PlayerId};
use std::collections::BTreeSet;

const TEXT: &str = "{4}, {T}: Draw a card. For each mana in this ability's activation cost, you may tap an untapped creature you control rather than pay that mana. Activate only as a sorcery.";

inventory::submit! { ManualAbility {
    card: "Heirloom Epic",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{4}, {T}: Draw a card. Activate only as a sorcery.")
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect()
    },
    reason: "creatures tapped rather than paying mana in an ability's activation cost: only this card",
} }

fn is_epic(a: &Ability) -> bool {
    a.text == TEXT
}

fn untapped_creatures(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| g.obj(*o).is_creature() && !g.obj(*o).tapped)
        .collect()
}

/// Takes `n` generic mana out of `cost`.
fn reduce(cost: &mut Cost, n: u32) {
    if let Some(m) = cost.mana.as_mut() {
        let g = m.generic_amount();
        let mut out = ManaCost::default();
        for s in &m.symbols {
            if !matches!(s, crate::mana::ManaSymbol::Generic(_)) {
                out.symbols.push(s.clone());
            }
        }
        if g > n {
            out.symbols
                .insert(0, crate::mana::ManaSymbol::Generic(g - n));
        }
        *m = out;
    }
}

fn has_mana_ability(g: &Game, id: ObjectId) -> bool {
    g.obj(id)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
}

/// The default choice (`default-cost-helpers`, as for convoke): no creatures if the mana
/// can be paid otherwise; else creatures (preferring ones without mana abilities, which
/// can't pay for it another way) until the rest can be paid.
fn default_choice(
    g: &Game,
    p: PlayerId,
    src: ObjectId,
    cost: &Cost,
    cands: &[ObjectId],
) -> Vec<ObjectId> {
    let chars = g.obj(src).chars.clone();
    let mut ctx = Ctx::new(Some(src), p);
    ctx.cost_of = Some(crate::rule_statics::payment::CostOf::Ability);
    let payable = |n: usize| {
        let mut c = cost.clone();
        reduce(&mut c, n as u32);
        g.can_pay_cost_optimistic_in(p, &c, Some(src), &chars, &ctx)
    };
    let mut order = cands.to_vec();
    order.sort_by_key(|c| has_mana_ability(g, *c));
    let generic = cost.mana.as_ref().map_or(0, |m| m.generic_amount()) as usize;
    let max = generic.min(order.len());
    let n = (0..=max).find(|n| payable(*n)).unwrap_or(max);
    order.truncate(n);
    order
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn activation_payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        _src: ObjectId,
        a: &Ability,
        cost: &mut Cost,
    ) {
        if is_epic(a) {
            reduce(cost, untapped_creatures(g, p).len() as u32);
        }
    }
    fn pay_activation_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        src: ObjectId,
        a: &Ability,
        cost: &mut Cost,
    ) {
        if !is_epic(a) {
            return;
        }
        let generic = cost.mana.as_ref().map_or(0, |m| m.generic_amount());
        let cands = untapped_creatures(g, p);
        if generic == 0 || cands.is_empty() {
            return;
        }
        let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
        let max = generic.min(cands.len() as u32);
        let chosen: Vec<ObjectId> = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(src),
                prompt: "Tap creatures rather than pay mana".into(),
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
            _ => default_choice(g, p, src, cost, &cands),
        };
        let tapped = chosen.into_iter().filter(|c| g.tap(*c)).count() as u32;
        reduce(cost, tapped);
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
