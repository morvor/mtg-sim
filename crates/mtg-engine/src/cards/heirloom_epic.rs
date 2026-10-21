//! Heirloom Epic: "{4}, {T}: Draw a card. For each mana in this ability's activation cost,
//! you may tap an untapped creature you control rather than pay that mana. Activate only as
//! a sorcery." Like convoke for an ability (CR 702.51 is for spells): each creature tapped
//! pays for one generic mana of the total cost; any untapped creature may be tapped, and at
//! most as many as the mana in the cost (rulings).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::ManaCost;
use crate::types::{ObjectId, PlayerId};

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
            out.symbols.insert(0, crate::mana::ManaSymbol::Generic(g - n));
        }
        *m = out;
    }
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
        let chosen = g.ask_objects(
            p,
            Some(src),
            "Tap creatures rather than pay mana",
            cands,
            0,
            generic,
        );
        let tapped = chosen.into_iter().filter(|c| g.tap(*c)).count() as u32;
        reduce(cost, tapped);
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
