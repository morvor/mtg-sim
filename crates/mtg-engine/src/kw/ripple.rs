//! CR 702.60 Ripple. "Ripple N" means "When you cast this spell, you may reveal the top N
//! cards of your library, or, if there are fewer than N cards in your library, you may
//! reveal all the cards in your library. If you reveal cards from your library this way,
//! you may cast any of those cards with the same name as this spell without paying their
//! mana costs, then put all revealed cards not cast this way on the bottom of your
//! library in any order." (CR 702.60a). A triggered ability that functions on the stack;
//! each instance triggers separately (CR 702.60b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{CastMethod, Zone};
use smol_str::SmolStr;

/// Prefix of the custom effect of a ripple ability; the number of cards follows it.
const RIPPLE: &str = "ripple:";

pub struct Ripple;

impl KeywordRules for Ripple {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Ripple]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Body::effect(Effect::Custom(SmolStr::new(format!("{RIPPLE}{n}")))),
        );
        t.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Ripple.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(n) = name.strip_prefix(RIPPLE).and_then(|n| n.parse::<u32>().ok()) else {
            return false;
        };
        ripple(g, n, ctx);
        true
    }
}

fn ripple(g: &mut Game, n: u32, ctx: &mut Ctx) {
    let p = ctx.controller;
    // "this spell": as it last existed on the stack if it has left it.
    let Some(spell) = ctx.source else {
        return;
    };
    let spell_chars = g.obj(spell).chars.clone();
    let revealed = crate::library::top_cards(g, p, n);
    if revealed.is_empty()
        || !g.ask_yes_no(
            p,
            Some(spell),
            &format!("Reveal the top {n} cards of your library (ripple)?"),
            true,
        )
    {
        return;
    }
    let names: Vec<String> = revealed
        .iter()
        .map(|c| g.obj(*c).chars.name.to_string())
        .collect();
    g.log(|_| format!("{p} reveals {}", names.join(", ")));
    for card in revealed.clone() {
        let same_name = g.obj(card).chars.shares_name_with(&spell_chars);
        if !same_name || g.obj(card).zone != Zone::Library(p) {
            continue;
        }
        let prompt = format!(
            "Cast {} without paying its mana cost?",
            g.obj(card).chars.name
        );
        if g.ask_yes_no(p, Some(card), &prompt, true) {
            let _ = crate::casting::cast_during_resolution(g, p, card, CastMethod::Free);
        }
    }
    let rest: Vec<_> = revealed
        .into_iter()
        .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Library(p))
        .collect();
    crate::library::place_rest(g, p, rest, &Destination::library_bottom(), ctx);
}

inventory::submit! { KeywordRegistration(&Ripple) }
