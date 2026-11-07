//! "You may cast an artifact spell from your hand or graveyard by paying life equal to its
//! mana value rather than paying its mana cost." (Anrakyr the Traveller): as the ability
//! resolves, its controller may cast one card of that kind from those zones (CR 608.2g),
//! ignoring timing permissions, for an alternative cost (CR 118.9) of life equal to the
//! mana value of the spell it would become (CR 601.3e; X is 0, CR 107.3b). That is the
//! only way to cast it this way: not for its mana cost, nor for another alternative cost
//! (CR 118.9a); additional costs such as kicker may be paid, and mandatory ones must be.
//! Life can be paid only by a player whose life total is at least that amount (CR 119.4).
//!
//! The effect is an `Effect::Custom` named by [`effect_name`]; the oracle phrase is parsed
//! in `oracle/patterns/r601_cast_paying_life.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Cost, CostPart, Value, ZoneKind};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::CastMethod;
use crate::types::{CardType, Entity, ObjectId};

const PREFIX: &str = "cast paying life equal to its mana value:";
/// The `CastMethod::Alternative` id of paying life equal to the spell's mana value.
pub const PAY_LIFE_METHOD: u64 = 0x11FE_0118_0009;

/// The `Effect::Custom` name of "you may cast a[n] [kind] spell from [zones] by paying
/// life equal to its mana value rather than paying its mana cost": `kind` is a card type
/// or `None` for any spell.
pub fn effect_name(zones: &[ZoneKind], kind: Option<CardType>) -> String {
    let zones: Vec<&str> = zones
        .iter()
        .map(|z| match z {
            ZoneKind::Graveyard => "graveyard",
            _ => "hand",
        })
        .collect();
    let kind = kind.map(|k| format!("{k:?}")).unwrap_or_default();
    format!("{PREFIX}{}:{kind}", zones.join(","))
}

fn parse_name(name: &str) -> Option<(Vec<ZoneKind>, Option<CardType>)> {
    let (zones, kind) = name.strip_prefix(PREFIX)?.split_once(':')?;
    let zones = zones
        .split(',')
        .map(|z| match z {
            "graveyard" => ZoneKind::Graveyard,
            _ => ZoneKind::Hand,
        })
        .collect();
    let kind = if kind.is_empty() {
        None
    } else {
        Some(CardType::from_word(kind)?)
    };
    Some((zones, kind))
}

pub struct CastPayingLife;

impl KeywordRules for CastPayingLife {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((zones, kind)) = parse_name(name) else {
            return false;
        };
        cast_paying_life(g, ctx, &zones, kind);
        true
    }
}

fn cast_paying_life(g: &mut Game, ctx: &mut Ctx, zones: &[ZoneKind], kind: Option<CardType>) {
    let p = ctx.controller;
    ctx.prev_happened = false;
    // CR 702.61a: no spell can be cast while a spell with split second is on the stack.
    if g.split_second_on_stack() {
        return;
    }
    g.recompute();
    let mut cards: Vec<ObjectId> = Vec::new();
    for z in zones {
        match z {
            ZoneKind::Graveyard => cards.extend(g.player(p).graveyard.iter().copied()),
            _ => cards.extend(g.player(p).hand.iter().copied()),
        }
    }
    let mut options = Vec::new();
    for card in cards {
        for (label, mut opt) in crate::casting::free_cast_options(g, p, card, |_| true) {
            let chars = g.option_characteristics(card, &opt);
            if kind.is_some_and(|k| !chars.card_types.contains(k)) {
                continue;
            }
            let mv = chars
                .mana_cost
                .as_ref()
                .map_or(0, |m| m.mana_value_with_x(0));
            if opt.method == CastMethod::Free {
                opt.method = CastMethod::Alternative(PAY_LIFE_METHOD);
            }
            opt.alt_cost = Some(Cost::free().with(CostPart::PayLife(Value::c(mv as i32))));
            options.push((card, label, opt));
        }
    }
    let mut choices: Vec<ObjectId> = Vec::new();
    for (card, _, _) in &options {
        if !choices.contains(card) {
            choices.push(*card);
        }
    }
    // Choosing none (the default) is declining: it's "you may".
    let prompt = "Cast a spell by paying life equal to its mana value (or none)";
    let Some(card) = g
        .ask_objects(p, ctx.source, prompt, choices, 0, 1)
        .into_iter()
        .next()
    else {
        return;
    };
    let mut ways: Vec<_> = options
        .into_iter()
        .filter(|(c, _, _)| *c == card)
        .map(|(_, label, opt)| (label, opt))
        .collect();
    if ways.is_empty() {
        return;
    }
    let i = if ways.len() > 1 {
        let labels = ways
            .iter()
            .map(|(label, _)| format!("Cast {label}"))
            .collect();
        g.ask_option(p, Some(card), "Choose which spell to cast", labels)
    } else {
        0
    };
    let (_, opt) = ways.swap_remove(i.min(ways.len() - 1));
    if let Ok(spell) = g.cast_with_option(p, card, opt) {
        ctx.prev_happened = true;
        // CR 400.7h: other parts of the effect can find the spell cast this way.
        ctx.set_var(vars::IT, vec![Entity::Object(spell)]);
    }
}

inventory::submit! { KeywordRegistration(&CastPayingLife) }
