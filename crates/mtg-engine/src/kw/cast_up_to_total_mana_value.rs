//! "You may cast up to two instant and/or sorcery spells with total mana value 6 or less
//! from your graveyard and/or hand without paying their mana costs." (Invoke Calamity): as
//! the spell resolves, its controller casts the spells one after the other (CR 608.2g),
//! each for the alternative cost of nothing (CR 118.9; X is 0, CR 107.3b), ignoring
//! timing permissions. The types and mana values are those of the spells as they're cast
//! (CR 601.3e): one half of a split card, the back face of a modal double-faced card
//! (CR 709.3a, 712.11c). The one cast second is on top of the stack and resolves first
//! (CR 405.2). "Those spells" are the spells cast (`vars::IT`).
//!
//! "You may cast any number of spells from among cards exiled with this artifact with
//! total mana value X or less without paying their mana costs." (Rod of Absorption): any
//! number of them (`u32::MAX`), from the cards linked to the source (CR 607), with a total
//! of the ability's X (see [`effect_name_with`]).
//!
//! "Copy those cards. You may cast any number of spells with total mana value 6 or less
//! from among the copies without paying their mana costs." (Uldaros Theorix), "Copy those
//! exiled cards. You may cast up to three of the copies without paying their mana costs."
//! (Baron Helmut Zemo): the copies the previous instruction made (`vars::CREATED`, CR
//! 707.12), capped by a total, a count or both (see [`copies_effect_name`]); each copy is
//! cast or not individually (CR 707.12a), and those not cast cease to exist (CR 704.5e).
//!
//! The effect is an `Effect::Custom` named by [`effect_name`]; the oracle phrase is parsed
//! in `oracle/patterns/r601_cast_up_to_total_mana_value.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, ZoneKind};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{CardType, Entity, ObjectId};

const PREFIX: &str = "cast spells with total mana value at most:";

/// The `Effect::Custom` name of "you may cast up to `n` [types] spells with total mana
/// value `total` or less from [zones] without paying their mana costs": `types` are the
/// card types any one of which each spell must have (none: any spell).
pub fn effect_name(n: u32, total: u32, zones: &[ZoneKind], types: &[CardType]) -> String {
    effect_name_with(n, Some(total), zones, types)
}

/// [`effect_name`] with the total the ability's X when `total` is `None`; `n` is
/// `u32::MAX` for "any number of spells"; `ZoneKind::Exile` names the cards exiled with
/// the source (linked to the ability, CR 607).
pub fn effect_name_with(
    n: u32,
    total: Option<u32>,
    zones: &[ZoneKind],
    types: &[CardType],
) -> String {
    let zones: Vec<&str> = zones
        .iter()
        .map(|z| match z {
            ZoneKind::Graveyard => "graveyard",
            ZoneKind::Exile => "linked",
            _ => "hand",
        })
        .collect();
    let types: Vec<String> = types.iter().map(|t| format!("{t:?}")).collect();
    let total = total.map_or("x".to_string(), |t| t.to_string());
    format!("{PREFIX}{n}:{total}:{}:{}", zones.join(","), types.join(","))
}

/// The source token of the copies the previous instruction made.
const COPIES: &str = "copies";

/// The `Effect::Custom` name of "you may cast up to `n` of the copies" (`total` `None`)
/// or "any number of spells with total mana value `total` or less from among the copies"
/// (`n` `u32::MAX`): the copies of cards the previous instruction made (`vars::CREATED`);
/// `types` as for [`effect_name`].
pub fn copies_effect_name(n: u32, total: Option<u32>, types: &[CardType]) -> String {
    let total = total.unwrap_or(u32::MAX);
    let types: Vec<String> = types.iter().map(|t| format!("{t:?}")).collect();
    format!("{PREFIX}{n}:{total}:{COPIES}:{}", types.join(","))
}

/// Where the spells are cast from.
#[derive(Clone, Copy, PartialEq)]
enum Source {
    Hand,
    Graveyard,
    /// The cards exiled with the source (CR 607).
    Linked,
    /// The copies the previous instruction made (`vars::CREATED`).
    Copies,
}

type Parsed = (u32, Option<u32>, Vec<Source>, Vec<CardType>);

fn parse_name(name: &str) -> Option<Parsed> {
    let mut parts = name.strip_prefix(PREFIX)?.split(':');
    let n = parts.next()?.parse().ok()?;
    let total = match parts.next()? {
        "x" => None,
        t => Some(t.parse().ok()?),
    };
    let zones = parts
        .next()?
        .split(',')
        .map(|z| match z {
            "graveyard" => Source::Graveyard,
            "linked" => Source::Linked,
            COPIES => Source::Copies,
            _ => Source::Hand,
        })
        .collect();
    let types = parts
        .next()?
        .split(',')
        .filter(|t| !t.is_empty())
        .map(CardType::from_word)
        .collect::<Option<Vec<_>>>()?;
    Some((n, total, zones, types))
}

pub struct CastUpToTotalManaValue;

impl KeywordRules for CastUpToTotalManaValue {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((n, total, zones, types)) = parse_name(name) else {
            return false;
        };
        let p = ctx.controller;
        let mut left = total.unwrap_or(ctx.x.max(0) as u32);
        // No total: only a count ("up to three of the copies").
        let capped = left != u32::MAX;
        // The copies, as the instruction begins (the spells cast don't change them).
        let copies: Vec<ObjectId> = ctx
            .vars
            .get(&vars::CREATED)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        let mut cast: Vec<Entity> = Vec::new();
        let mut used: Vec<ObjectId> = Vec::new();
        for _ in 0..n {
            // CR 702.61a: no spell can be cast while a spell with split second is on the
            // stack (including one cast just now).
            if g.split_second_on_stack() {
                break;
            }
            g.recompute();
            let mut cards: Vec<ObjectId> = Vec::new();
            for z in &zones {
                match z {
                    Source::Graveyard => cards.extend(g.player(p).graveyard.iter().copied()),
                    // The copies that still exist (CR 704.5e).
                    Source::Copies => cards.extend(copies.iter().copied().filter(|o| g.is_live(*o))),
                    // The cards exiled with the source that are still in exile.
                    Source::Linked => cards.extend(
                        g.eval_sel(&crate::ability::Sel::Linked, ctx)
                            .into_iter()
                            .filter_map(|e| e.object())
                            .filter(|o| g.is_live(*o) && g.obj(*o).zone == crate::object::Zone::Exile),
                    ),
                    _ => cards.extend(g.player(p).hand.iter().copied()),
                }
            }
            let mut options = Vec::new();
            for card in cards.into_iter().filter(|c| !used.contains(c)) {
                for (label, opt) in crate::casting::free_cast_options(g, p, card, |mv| mv <= left)
                {
                    let chars = g.option_characteristics(card, &opt);
                    if !types.is_empty() && !types.iter().any(|t| chars.card_types.contains(*t)) {
                        continue;
                    }
                    let mv = chars
                        .mana_cost
                        .as_ref()
                        .map_or(0, |m| m.mana_value_with_x(0));
                    options.push((card, label, opt, mv));
                }
            }
            let mut castable: Vec<ObjectId> = Vec::new();
            for (card, ..) in &options {
                if !castable.contains(card) {
                    castable.push(*card);
                }
            }
            // Choosing none (the default) is casting no more: it's "up to".
            let prompt = if capped {
                format!(
                    "Cast a spell with mana value {left} or less without paying its mana cost (or none)"
                )
            } else {
                "Cast a spell without paying its mana cost (or none)".to_string()
            };
            let Some(card) = g
                .ask_objects(p, ctx.source, &prompt, castable, 0, 1)
                .into_iter()
                .next()
            else {
                break;
            };
            let mut ways: Vec<_> = options
                .into_iter()
                .filter(|(c, ..)| *c == card)
                .map(|(_, label, opt, mv)| (label, opt, mv))
                .collect();
            if ways.is_empty() {
                break;
            }
            let i = if ways.len() > 1 {
                let labels = ways
                    .iter()
                    .map(|(label, ..)| format!("Cast {label}"))
                    .collect();
                g.ask_option(p, Some(card), "Choose which spell to cast", labels)
            } else {
                0
            };
            let (_, opt, mv) = ways.swap_remove(i.min(ways.len() - 1));
            used.push(card);
            if let Ok(spell) = g.cast_with_option(p, card, opt) {
                left = left.saturating_sub(mv);
                cast.push(Entity::Object(spell));
            }
        }
        ctx.prev_happened = !cast.is_empty();
        // CR 400.7h: other parts of the effect can find the spells cast this way.
        ctx.set_var(vars::IT, cast);
        true
    }
}

inventory::submit! { KeywordRegistration(&CastUpToTotalManaValue) }
