//! CR 702.167 Craft: "Craft with [materials] [cost]" means "[Cost], Exile this permanent,
//! Exile [materials] from among permanents you control and/or cards in your graveyard:
//! Return this card to the battlefield transformed under its owner's control. Activate only
//! as a sorcery." (CR 702.167a).
//!
//! * The keyword keeps the materials' description in [`Keyword::text`] ("with artifact")
//!   and the mana cost in [`Keyword::cost`]; [`parse_materials`] reads the description.
//! * A material described only by a card type or subtype without the word "card" is a
//!   permanent of that type you control (other than the one being crafted) or a card of
//!   that type in your graveyard; described as a "card", only a card in your graveyard
//!   (CR 702.167b).
//! * The materials are exiled as part of the cost ([`EXILE_MATERIALS`], paid before the
//!   permanent itself is exiled); the cost can be paid only if they can be.
//! * The exiled cards used to craft a permanent are linked to the permanent the ability
//!   returns (its `linked` objects under [`CRAFT_LINK`]): its abilities refer to them as
//!   "the exiled cards used to craft it" ([`USED_TO_CRAFT`], CR 702.167c).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` prefix of the cost part "Exile [materials] from among permanents you
/// control and/or cards in your graveyard"; the materials' description follows.
pub const EXILE_MATERIALS: &str = "craft:exile materials:";
/// `Effect::Custom`: "Return this card to the battlefield transformed under its owner's
/// control."
pub const RETURN_TRANSFORMED: &str = "craft:return this card transformed";
/// `Filter::Custom`: an exiled card used to craft the source (CR 702.167c).
pub const USED_TO_CRAFT: &str = "craft:exiled card used to craft it";
/// "used to craft it": cards in exile that were exiled to craft the source (CR 702.167c).
pub fn used_to_craft() -> Filter {
    Filter::and(vec![
        Filter::InZone(ZoneKind::Exile),
        Filter::Custom(USED_TO_CRAFT.into()),
    ])
}

/// The link under which a crafted permanent (and, until it returns, the permanent being
/// crafted) keeps the exiled cards used to craft it.
pub const CRAFT_LINK: u16 = 0x3fa7;

/// What a craft ability's materials are.
#[derive(Clone, Debug)]
pub struct Materials {
    /// What each material is.
    pub filter: Filter,
    pub min: u32,
    /// `None`: any number ("one or more", "four or more").
    pub max: Option<u32>,
    /// "a Dinosaur, a Merfolk, a Pirate, and a Vampire": one distinct material for each.
    pub slots: Vec<Filter>,
    /// "two that share a card type".
    pub share_card_type: bool,
    /// Described as cards: only cards in the graveyard (CR 702.167b).
    pub cards_only: bool,
}

/// Parses a craft ability's materials: "artifact", "Cave", "two creatures", "one or more
/// creatures", "one or more", "six artifacts", "two that share a card type", "four or more
/// red instant and/or sorcery cards", "a Dinosaur, a Merfolk, a Pirate, and a Vampire".
pub fn parse_materials(s: &str) -> Option<Materials> {
    let s = end(s.trim()).to_lowercase();
    let s = s.as_str();
    let base = |filter: Filter, min: u32, max: Option<u32>| Materials {
        filter,
        min,
        max,
        slots: vec![],
        share_card_type: false,
        cards_only: false,
    };
    if s == "one or more" {
        return Some(base(Filter::Any, 1, None));
    }
    if let Some((n, rest)) = parse_number(s) {
        let n = n.as_const()?.max(0) as u32;
        if end(rest) == "that share a card type" {
            let mut m = base(Filter::Any, n, Some(n));
            m.share_card_type = true;
            return Some(m);
        }
        // "a Dinosaur, a Merfolk, a Pirate, and a Vampire".
        if s.contains(", ") {
            let mut slots = Vec::new();
            for part in s.split(", ") {
                let part = part.trim_start_matches("and ");
                let part = part
                    .strip_prefix("a ")
                    .or_else(|| part.strip_prefix("an "))?;
                slots.push(phrase(part)?.0);
            }
            let n = slots.len() as u32;
            let mut m = base(Filter::Any, n, Some(n));
            m.slots = slots;
            return Some(m);
        }
        if let Some(r) = rest.strip_prefix("or more ") {
            let (f, cards) = phrase(r)?;
            let mut m = base(f, n, None);
            m.cards_only = cards;
            return Some(m);
        }
        let (f, cards) = phrase(rest)?;
        let mut m = base(f, n, Some(n));
        m.cards_only = cards;
        return Some(m);
    }
    // A single material: "artifact", "Cave", "creature".
    let (f, cards) = phrase(s)?;
    let mut m = base(f, 1, Some(1));
    m.cards_only = cards;
    Some(m)
}

/// An object phrase of a material, and whether it describes cards.
fn phrase(s: &str) -> Option<(Filter, bool)> {
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).is_empty() {
        return None;
    }
    let cards = s.split_whitespace().any(|w| matches!(w, "card" | "cards"));
    Some((f, cards))
}

/// The materials' description of a craft keyword ("with artifact" → "artifact").
fn materials_of(kw: &Keyword) -> Option<Materials> {
    let t = kw.text.as_deref()?;
    parse_materials(t.trim().strip_prefix("with ").unwrap_or(t))
}

/// The objects that could be exiled as materials for crafting `src`: permanents `p`
/// controls other than it and cards in `p`'s graveyard (CR 702.167a–b).
pub fn candidates(g: &Game, p: PlayerId, src: ObjectId, m: &Materials) -> Vec<ObjectId> {
    let ctx = Ctx::new(Some(src), p);
    let src_now = g.current(src);
    let mut out = Vec::new();
    if !m.cards_only {
        out.extend(
            g.permanents()
                .filter(|o| o.controller == p && o.id != src && o.id != src_now)
                .map(|o| o.id)
                .filter(|id| fits_any(g, *id, m, &ctx)),
        );
    }
    // Cards in the graveyard: not a token that's there until state-based actions.
    out.extend(
        g.player(p)
            .graveyard
            .iter()
            .copied()
            .filter(|id| !g.obj(*id).is_token() && fits_any(g, *id, m, &ctx)),
    );
    out
}

fn fits_any(g: &Game, id: ObjectId, m: &Materials, ctx: &Ctx) -> bool {
    if m.slots.is_empty() {
        g.matches(id, &m.filter, ctx)
    } else {
        m.slots.iter().any(|f| g.matches(id, f, ctx))
    }
}

/// Whether `chosen` is a legal set of materials.
fn valid(g: &Game, chosen: &[ObjectId], m: &Materials, ctx: &Ctx) -> bool {
    let n = chosen.len() as u32;
    if n < m.min || m.max.is_some_and(|max| n > max) {
        return false;
    }
    if !chosen.iter().all(|c| fits_any(g, *c, m, ctx)) {
        return false;
    }
    if m.share_card_type {
        let shared = chosen
            .iter()
            .map(|c| g.obj(*c).chars.card_types)
            .reduce(|a, b| CardTypeSet(a.0 & b.0));
        if shared.is_none_or(|s| s.is_empty()) {
            return false;
        }
    }
    if !m.slots.is_empty() {
        return assign_slots(g, chosen, &m.slots, ctx);
    }
    true
}

/// Whether each slot can be filled by a distinct chosen object.
fn assign_slots(g: &Game, chosen: &[ObjectId], slots: &[Filter], ctx: &Ctx) -> bool {
    fn go(g: &Game, chosen: &[ObjectId], slots: &[Filter], used: &mut Vec<bool>, ctx: &Ctx) -> bool {
        let Some((first, rest)) = slots.split_first() else {
            return true;
        };
        for i in 0..chosen.len() {
            if !used[i] && g.matches(chosen[i], first, ctx) {
                used[i] = true;
                if go(g, chosen, rest, used, ctx) {
                    return true;
                }
                used[i] = false;
            }
        }
        false
    }
    chosen.len() == slots.len() && go(g, chosen, slots, &mut vec![false; chosen.len()], ctx)
}

/// A legal set of materials among `cands`, if there is one.
fn default_choice(g: &Game, cands: &[ObjectId], m: &Materials, ctx: &Ctx) -> Option<Vec<ObjectId>> {
    if !m.slots.is_empty() {
        // A distinct object for each slot, by search.
        fn go(
            g: &Game,
            cands: &[ObjectId],
            slots: &[Filter],
            picked: &mut Vec<ObjectId>,
            ctx: &Ctx,
        ) -> bool {
            let Some((first, rest)) = slots.split_first() else {
                return true;
            };
            for c in cands {
                if !picked.contains(c) && g.matches(*c, first, ctx) {
                    picked.push(*c);
                    if go(g, cands, rest, picked, ctx) {
                        return true;
                    }
                    picked.pop();
                }
            }
            false
        }
        let mut picked = Vec::new();
        return go(g, cands, &m.slots, &mut picked, ctx).then_some(picked);
    }
    let n = m.min as usize;
    if m.share_card_type {
        for ty in CardType::ALL {
            let with: Vec<ObjectId> = cands
                .iter()
                .copied()
                .filter(|c| g.obj(*c).chars.card_types.contains(ty))
                .collect();
            if with.len() >= n {
                return Some(with.into_iter().take(n).collect());
            }
        }
        return None;
    }
    (cands.len() >= n).then(|| cands.iter().copied().take(n).collect())
}

/// Whether `p` could exile materials to craft `src`.
pub fn materials_available(g: &Game, p: PlayerId, src: ObjectId, m: &Materials) -> bool {
    let ctx = Ctx::new(Some(src), p);
    let cands = candidates(g, p, src, m);
    default_choice(g, &cands, m, &ctx).is_some()
}

pub struct Craft;

impl KeywordRules for Craft {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Craft]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = kw.text.as_deref()?;
        materials_of(kw)?;
        let desc = text.trim().strip_prefix("with ").unwrap_or(text);
        let mut cost = kw.cost.clone().unwrap_or_default();
        // The materials first, while the permanent is still on the battlefield; then the
        // permanent itself.
        cost.parts.push(CostPart::Effect(Box::new(Effect::Custom(SmolStr::new(
            format!("{EXILE_MATERIALS}{desc}"),
        )))));
        cost.parts.push(CostPart::ExileSelf);
        let mut act = ActivatedAbility::new(
            cost,
            Body::effect(Effect::Custom(RETURN_TRANSFORMED.into())),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Craft.name(),
        )])
    }

    /// The cost can be paid only if there are materials to exile (checked as the ability
    /// is activated and again as the cost is paid, after any mana abilities).
    fn custom_effect_possible(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let desc = name.strip_prefix(EXILE_MATERIALS)?;
        Some(match (parse_materials(desc), ctx.source) {
            (Some(m), Some(src)) => materials_available(g, ctx.controller, src, &m),
            _ => false,
        })
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(desc) = name.strip_prefix(EXILE_MATERIALS) {
            let (Some(m), Some(src)) = (parse_materials(desc), ctx.source) else {
                return true;
            };
            let p = ctx.controller;
            let cands = candidates(g, p, src, &m);
            let Some(default) = default_choice(g, &cands, &m, ctx) else {
                return true;
            };
            let max = m.max.unwrap_or(cands.len() as u32);
            let chosen = g.ask_objects(
                p,
                Some(src),
                &format!("Craft: exile materials ({desc})"),
                cands,
                m.min,
                max,
            );
            let chosen = if valid(g, &chosen, &m, ctx) {
                chosen
            } else {
                default
            };
            let moves: Vec<MoveEv> = chosen
                .iter()
                .map(|o| MoveEv {
                    obj: *o,
                    to: Zone::Exile,
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Cost,
                    by: Some(p),
                    etb: EtbInfo::default(),
                    source: Some(src),
                })
                .collect();
            let exiled: Vec<ObjectId> = g
                .move_objects(moves)
                .into_iter()
                .flatten()
                .filter(|o| g.obj(*o).zone == Zone::Exile)
                .collect();
            g.objects[src.0 as usize].linked.insert(CRAFT_LINK, exiled);
            return true;
        }
        if name != RETURN_TRANSFORMED {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let card = g.current(src);
        let o = g.obj(card);
        // Only the card exiled by the cost, still in exile; only a double-faced card can
        // be put onto the battlefield transformed (CR 712.14a).
        if o.zone != Zone::Exile || o.card.as_ref().and_then(|d| d.back()).is_none() {
            return true;
        }
        let owner = o.owner;
        let materials: Vec<ObjectId> = g
            .obj(src)
            .linked
            .get(&CRAFT_LINK)
            .cloned()
            .unwrap_or_default();
        let new = g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(ctx.controller),
            etb: EtbInfo {
                controller: Some(owner),
                transformed: true,
                ..Default::default()
            },
            source: ctx.source,
        });
        if let Some(new) = new.filter(|n| g.obj(*n).zone == Zone::Battlefield) {
            // CR 702.167c: the cards exiled to pay the cost are the exiled cards used to
            // craft it.
            g.objects[new.0 as usize].linked.insert(CRAFT_LINK, materials);
            g.dirty = true;
        }
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != USED_TO_CRAFT {
            return None;
        }
        let Some(src) = ctx.source else {
            return Some(false);
        };
        let o = g.obj(id);
        Some(
            o.zone == Zone::Exile
                && g.obj(src)
                    .linked
                    .get(&CRAFT_LINK)
                    .is_some_and(|v| v.iter().any(|m| g.current(*m) == id)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Craft) }
