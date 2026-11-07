//! Illusionary Mask: "{X}: You may choose a creature card in your hand whose mana cost could
//! be paid by some amount of, or all of, the mana you spent on {X}. If you do, you may cast
//! that card face down as a 2/2 creature spell without paying its mana cost. If the
//! creature that spell becomes as it resolves has not been turned face up and would
//! assign or deal damage, be dealt damage, or become tapped, instead it's turned face up
//! and assigns or deals damage, is dealt damage, or becomes tapped. Activate only as a
//! sorcery."
//!
//! The activation cost and timing are the compiled ones; the rest is hand-written:
//! * the choice looks at both the amount and the types of mana spent on {X} (ruling);
//! * the card is cast face down (CR 708.4) as a morph-style face-down spell (it can be
//!   turned face up for a morph cost only if it would have morph, ruling) for no cost,
//!   during the resolution (CR 608.2g); it can be responded to and countered;
//! * the permanent it becomes is turned face up as a replacement effect (CR 614.1a, no
//!   stack) when it would assign combat damage, deal or be dealt damage, or become tapped,
//!   then does so; this goes on after the Mask has left the battlefield (rulings). As it's
//!   turned face up its "enters" and "as enters" abilities don't apply (CR 708.8).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::{Mana, ManaType, SpendContext};
use crate::object::{CastMethod, FaceState, Zone};
use crate::types::{CardType, Entity, ObjectId};

const TEXT: &str = "{X}: You may choose a creature card in your hand whose mana cost could be paid by some amount of, or all of, the mana you spent on {X}. If you do, you may cast that card face down as a 2/2 creature spell without paying its mana cost. If the creature that spell becomes as it resolves has not been turned face up and would assign or deal damage, be dealt damage, or become tapped, instead it's turned face up and assigns or deals damage, is dealt damage, or becomes tapped. Activate only as a sorcery.";
const CAST: &str = "card:Illusionary Mask:cast a creature card face down";
/// Rows `[spell]`: the face-down spells cast with the ability.
const SPELLS: &str = "card:Illusionary Mask:face-down spells";

inventory::submit! { ManualAbility {
    card: "Illusionary Mask",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{X}: You gain X life. Activate only as a sorcery.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(CAST.into())))
            .collect()
    },
    reason: "casts a creature card face down from an ability, chosen by the mana spent on {X}, and turns it face up by replacement: only this card",
} }

/// Whether the mana cost of `card` could be paid by some or all of `spent` (with X as 0).
fn payable_with(g: &Game, card: ObjectId, spent: &[ManaType]) -> bool {
    let Some(cost) = g.obj(card).chars.mana_cost.clone() else {
        // No mana cost: it can't be paid (CR 118.6), though nothing needs paying.
        return false;
    };
    let pool: Vec<Mana> = spent.iter().map(|t| Mana::new(*t)).collect();
    crate::mana::find_payment(&pool, &cost.with_x(0), &SpendContext::default(), 0).is_some()
}

/// Whether `obj` is a face-down permanent that a spell cast with the ability became and
/// that hasn't been turned face up.
fn masked(g: &Game, obj: ObjectId) -> bool {
    let Some(o) = g.try_obj(obj) else {
        return false;
    };
    o.face_down
        && o.zone == Zone::Battlefield
        && o.prev.is_some_and(|p| g.cards.get(SPELLS).iter().any(|r| r[0] == p.0 as i64))
}

/// Turns `obj` face up instead, if it's such a permanent.
fn reveal_instead(g: &mut Game, obj: ObjectId) {
    if !masked(g, obj) {
        return;
    }
    let by = g.obj(obj).controller;
    g.log(|g| format!("{} is turned face up (Illusionary Mask)", g.describe(obj)));
    crate::facedown::turn_face_up_by(g, obj, by);
    if let Some(p) = g.obj(obj).prev {
        let p = p.0 as i64;
        g.cards.retain(SPELLS, |r| r[0] != p);
    }
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CAST {
            return false;
        }
        let p = ctx.controller;
        // The mana spent on {X} (the whole activation cost).
        let spent: Vec<ManaType> = ctx
            .stack_obj
            .and_then(|s| g.try_obj(s))
            .and_then(|o| o.stack.as_deref())
            .map(|si| si.cast.mana_spent.clone())
            .unwrap_or_default();
        let candidates: Vec<ObjectId> = g
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| g.obj(*c).chars.is(CardType::Creature) && payable_with(g, *c, &spent))
            .collect();
        if candidates.is_empty() {
            return true;
        }
        let Some(card) = g
            .ask_objects(
                p,
                ctx.source,
                "You may choose a creature card whose mana cost the mana spent on X could pay",
                candidates.clone(),
                0,
                1,
            )
            .first()
            .copied()
            .filter(|c| candidates.contains(c))
        else {
            return true;
        };
        if !g.ask_yes_no(p, Some(card), "Cast it face down as a 2/2 creature spell?", true) {
            return true;
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::FaceDown(KeywordKind::Morph);
        opt.alt_cost = Some(crate::ability::Cost::free());
        opt.any_time = true;
        if let Ok(spell) = g.cast_with_option(p, card, opt) {
            g.cards.push(SPELLS, vec![spell.0 as i64]);
        }
        true
    }

    fn before_tap(&self, g: &mut Game, obj: ObjectId) {
        reveal_instead(g, obj);
    }

    fn before_damage(&self, g: &mut Game, events: &[(ObjectId, Entity, u32)]) {
        for (src, to, _) in events {
            reveal_instead(g, *src);
            if let Entity::Object(o) = to {
                reveal_instead(g, *o);
            }
        }
    }

    fn before_combat_damage_assignment(&self, g: &mut Game, creatures: &[ObjectId]) {
        for c in creatures {
            // A creature with 0 or less power assigns no combat damage (CR 510.1a).
            if masked(g, *c) && g.obj(*c).power() > 0 {
                reveal_instead(g, *c);
            }
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
