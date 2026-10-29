//! Object qualities and instructions for cards5-choices-players' choice grammar:
//!
//! - "other spells and/or abilities" (Boltbender): a spell or ability on the stack other
//!   than the one resolving;
//! - "You choose the name of a nonland card revealed this way." (Alhammarret, High
//!   Arbiter, after "each opponent reveals their hand"): the controller chooses one of the
//!   names of those cards, kept as the source's chosen name (CR 607.2d); with no such
//!   card, no name is chosen (CR 607.5a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Sel;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom` name: the same object, still on the battlefield ("the chosen
/// permanents are still on the battlefield": not one that left, CR 400.7).
pub const STILL_ON_BATTLEFIELD: &str = "choices5:still on the battlefield";

/// `Filter::Custom` name: not the resolving spell or ability itself.
pub const NOT_RESOLVING: &str = "choices5:not the resolving spell or ability";

/// `Effect::Custom` name prefix: choose the name of one of the cards (a JSON `Sel`
/// follows) for the source.
pub const NAME_AMONG: &str = "choices5:choose the name of one of:";

/// The instruction choosing a name among `cards`.
pub fn name_among(cards: &Sel) -> Option<crate::ability::Effect> {
    let json = serde_json::to_string(cards).ok()?;
    Some(crate::ability::Effect::Custom(
        format!("{NAME_AMONG}{json}").into(),
    ))
}

/// The cards a [`NAME_AMONG`] instruction chooses a name among.
pub fn name_among_cards(name: &str) -> Option<Sel> {
    serde_json::from_str(name.strip_prefix(NAME_AMONG)?).ok()
}

pub struct Choices5Filters;

impl KeywordRules for Choices5Filters {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(cards) = name_among_cards(name) else {
            return false;
        };
        let Some(src) = ctx.source else {
            return true;
        };
        let mut names: Vec<String> = Vec::new();
        for id in g.resolve_objects(&cards, ctx) {
            let n = g.obj(id).chars.name.to_string();
            if !n.is_empty() && !names.contains(&n) {
                names.push(n);
            }
        }
        names.sort();
        if names.is_empty() {
            return true;
        }
        let i = g.ask_option(ctx.controller, Some(src), "Choose a card name", names.clone());
        let n = names.get(i).unwrap_or(&names[0]).clone();
        g.objects[src.0 as usize].choices.card_name = Some(n.into());
        g.dirty = true;
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        match name {
            NOT_RESOLVING => Some(ctx.stack_obj != Some(id)),
            STILL_ON_BATTLEFIELD => Some(
                g.is_live(id) && g.obj(id).zone == crate::object::Zone::Battlefield,
            ),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&Choices5Filters) }
