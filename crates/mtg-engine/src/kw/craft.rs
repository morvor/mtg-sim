//! CR 702.167 Craft: "Craft with [materials] [cost]" means "[Cost], Exile this permanent,
//! Exile [materials] from among permanents you control and/or cards in your graveyard:
//! Return this card to the battlefield transformed under its owner's control. Activate
//! only as a sorcery." (CR 702.167a).
//!
//! * The keyword (parsed in `oracle/patterns/craft.rs`) keeps the materials' description in
//!   `Keyword::filter` and their number in `Keyword::n`: `n` objects, or at least `-n`
//!   objects when it's negative ("one or more", "four or more").
//! * A material described with only a card type or subtype ("artifact", "two creatures",
//!   "Island") is a permanent you control of that type or a card of that type in your
//!   graveyard (CR 702.167b); one described as a "card" is a card in your graveyard. Some
//!   may come from each (the source itself isn't one of them).
//! * The ability's effect returns the card the cost exiled (CR 400.7j). A card that isn't
//!   a double-faced card stays in exile (CR 712.14a), e.g. a copy of a card with craft.
//! * "The exiled cards used to craft it" (CR 702.167c) are the cards the cost exiled as
//!   materials, as long as they remain in exile and the permanent the ability returned
//!   remains on the battlefield ([`USED_TO_CRAFT`]). Tokens exiled as materials cease to
//!   exist, so they're never among them.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The cost part exiling the materials (`CostPart::Effect(Effect::Custom(..))`).
pub const MATERIALS: &str = "craft:exile the materials";
/// The effect of the craft ability.
pub const RETURN: &str = "craft:return this card to the battlefield transformed";
/// `Filter::Custom`: an exiled card used to craft the source (CR 702.167c).
pub const USED_TO_CRAFT: &str = "craft:exiled card used to craft it";
/// The link under which the materials are recorded: on the source as the craft ability's
/// cost is paid, then on the permanent it returns as.
pub const CRAFT_LINK: u16 = 0x7ff0;
/// The card the craft ability returns, while it's being moved.
const RETURNING: Var = vars::USER + 170;

pub struct Craft;

/// The materials of the craft keyword: (description, fewest, most).
fn materials(kw: &Keyword) -> Option<(Filter, u32, u32)> {
    let f = kw.filter.clone()?;
    let n = kw.n.unwrap_or(1);
    Some(if n < 0 {
        (f, n.unsigned_abs(), u32::MAX)
    } else {
        (f, n as u32, n as u32)
    })
}

/// Whether the description says "card(s)": only cards in the graveyard qualify.
fn cards_only(f: &Filter) -> bool {
    match f {
        Filter::Card => true,
        Filter::And(v) => v.iter().any(cards_only),
        _ => false,
    }
}

/// The objects `p` could exile as materials for the craft ability of `src` (CR 702.167b).
fn candidates(g: &Game, p: PlayerId, src: ObjectId, filter: &Filter, ctx: &Ctx) -> Vec<ObjectId> {
    let mut out = Vec::new();
    if !cards_only(filter) {
        out.extend(
            g.permanents()
                .filter(|o| o.controller == p && o.id != src && g.matches(o.id, filter, ctx))
                .map(|o| o.id),
        );
    }
    out.extend(
        g.player(p)
            .graveyard
            .iter()
            .copied()
            .filter(|c| *c != src && g.matches(*c, filter, ctx)),
    );
    out
}

/// The craft keyword of `src` (as it is, or as it last existed).
fn craft_keyword(g: &Game, src: ObjectId) -> Option<Keyword> {
    g.obj(src)
        .chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Craft)
        .cloned()
}

/// Whether `obj` became `src`'s next incarnations (`src` moved and became `obj`).
fn descends_from(g: &Game, mut obj: ObjectId, src: ObjectId) -> bool {
    while let Some(p) = g.obj(obj).prev {
        if p == src {
            return true;
        }
        obj = p;
    }
    false
}

impl KeywordRules for Craft {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Craft]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        materials(kw)?;
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::ExileSelf);
        cost.parts.push(CostPart::Effect(Box::new(Effect::Custom(
            SmolStr::new(MATERIALS),
        ))));
        let mut act = ActivatedAbility::new(
            cost,
            Body::effect(Effect::Custom(SmolStr::new(RETURN))),
        );
        act.timing = ActivationTiming::Sorcery;
        let text = kw
            .text
            .clone()
            .unwrap_or_else(|| SmolStr::new(KeywordKind::Craft.name()));
        Some(vec![AbilityDef::new(AbilityKind::Activated(act), text)])
    }

    fn custom_effect_possible(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != MATERIALS {
            return None;
        }
        let Some(src) = ctx.source else {
            return Some(false);
        };
        let Some((filter, min, _)) = craft_keyword(g, src).as_ref().and_then(materials) else {
            return Some(false);
        };
        Some(candidates(g, ctx.controller, src, &filter, ctx).len() as u32 >= min)
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            MATERIALS => {
                exile_materials(g, ctx);
                true
            }
            RETURN => {
                return_crafted(g, ctx);
                true
            }
            _ => false,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != USED_TO_CRAFT {
            return None;
        }
        Some(used_to_craft(g, ctx.source, id))
    }
}

/// Pays the materials part of the cost: `p` exiles the chosen objects, which are recorded
/// on the source.
fn exile_materials(g: &mut Game, ctx: &Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    let p = ctx.controller;
    let Some((filter, min, max)) = craft_keyword(g, src).as_ref().and_then(materials) else {
        return;
    };
    let cands = candidates(g, p, src, &filter, ctx);
    // Checked before paying (`custom_effect_possible`).
    if (cands.len() as u32) < min {
        return;
    }
    let max = max.min(cands.len() as u32);
    let chosen = g.ask_objects(p, Some(src), "Choose materials to exile (craft)", cands, min, max);
    let mut exiled = Vec::new();
    for c in chosen {
        if let Some(new) = g.exile_object(c, Some(src)) {
            exiled.push(new);
        }
    }
    g.objects[src.0 as usize]
        .linked
        .insert(CRAFT_LINK, exiled);
}

/// The craft ability's effect: the card its cost exiled returns transformed under its
/// owner's control, and the materials become the cards used to craft that permanent.
fn return_crafted(g: &mut Game, ctx: &mut Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    // CR 400.7j: the card the cost exiled, if it's still there.
    let card = ctx
        .var_objects(crate::zones::COST_MOVED)
        .into_iter()
        .find(|o| descends_from(g, *o, src))
        .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Exile);
    let Some(card) = card else {
        return;
    };
    let materials = g
        .obj(src)
        .linked
        .get(&CRAFT_LINK)
        .cloned()
        .unwrap_or_default();
    let mut d = Destination::battlefield();
    d.transformed = true;
    d.controller = Some(PlayerRef::Player(g.obj(card).owner));
    let tmp = RETURNING;
    let mut c = ctx.clone();
    c.set_var(tmp, vec![Entity::Object(card)]);
    g.exec(
        &Effect::Move {
            what: Sel::Var(tmp),
            to: d,
        },
        &mut c,
    );
    let now = g.current(card);
    if now != card && g.obj(now).zone == Zone::Battlefield {
        g.objects[now.0 as usize]
            .linked
            .insert(CRAFT_LINK, materials);
        g.dirty = true;
    }
    ctx.set_var(vars::IT, vec![Entity::Object(now)]);
}

/// Whether `id` is an exiled card used to craft `permanent` (CR 702.167c).
pub fn used_to_craft(g: &Game, permanent: Option<ObjectId>, id: ObjectId) -> bool {
    let Some(src) = permanent else {
        return false;
    };
    let s = g.obj(src);
    if !g.is_live(src) || s.zone != Zone::Battlefield {
        return false;
    }
    let o = g.obj(id);
    s.linked.get(&CRAFT_LINK).is_some_and(|v| v.contains(&id))
        && g.is_live(id)
        && o.zone == Zone::Exile
        && o.kind == ObjKind::Card
}

/// "The exiled card(s) used to craft it": `Sel::All` of [`USED_TO_CRAFT`].
pub fn used_to_craft_sel() -> Sel {
    Sel::All(Filter::And(vec![
        Filter::InZone(ZoneKind::Exile),
        Filter::Custom(SmolStr::new(USED_TO_CRAFT)),
    ]))
}

inventory::submit! { KeywordRegistration(&Craft) }
