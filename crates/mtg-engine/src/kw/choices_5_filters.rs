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

/// `Effect::Custom` name: "choose a card type shared among two exiled cards used to craft
/// it" (Eye of Ojer Taq // Apex Observatory): a card type every exiled card used to craft
/// the source has (CR 702.167c), kept as the source's chosen card type (CR 607.2d); none
/// if they share none.
pub const CHOOSE_TYPE_SHARED_BY_CRAFT_MATERIALS: &str =
    "choices5:choose a card type shared among the exiled cards used to craft it";

/// `Condition::Custom` name: the player being iterated over is a defending player (CR
/// 506.2, 802.2: with several, each of them).
pub const IS_DEFENDING_PLAYER: &str = "choices5:is a defending player";

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
        if name == CHOOSE_TYPE_SHARED_BY_CRAFT_MATERIALS {
            choose_type_shared_by_materials(g, ctx);
            return true;
        }
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

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == IS_DEFENDING_PLAYER).then(|| {
            ctx.iter_player.is_some_and(|p| {
                g.combat
                    .as_ref()
                    .is_some_and(|c| c.defending_players.contains(&p))
            })
        })
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

/// [`CHOOSE_TYPE_SHARED_BY_CRAFT_MATERIALS`]. As the permanent enters, the cards its craft
/// cost exiled are those kept for it (`kw::craft::CRAFT_LINK`), still in exile.
fn choose_type_shared_by_materials(g: &mut Game, ctx: &Ctx) {
    use crate::types::CardType;
    let Some(src) = ctx.source else {
        return;
    };
    // The materials are kept for the crafted permanent once it's on the battlefield; as
    // it enters, they're those kept for the permanent the craft ability exiled (the object
    // it came from, CR 400.7).
    let mut at = Some(src);
    let mut kept = None;
    for _ in 0..4 {
        let Some(id) = at else { break };
        if let Some(v) = g.obj(id).linked.get(&crate::kw::craft::CRAFT_LINK) {
            kept = Some(v.clone());
            break;
        }
        at = g.obj(id).prev;
    }
    let materials: Vec<ObjectId> = kept
        .unwrap_or_default()
        .into_iter()
        .filter(|id| {
            g.is_live(*id)
                && g.obj(*id).zone == crate::object::Zone::Exile
                && g.obj(*id).kind == crate::object::ObjKind::Card
        })
        .collect();
    if materials.is_empty() {
        return;
    }
    let shared: Vec<CardType> = CardType::ALL
        .iter()
        .copied()
        .filter(|t| materials.iter().all(|m| g.obj(*m).chars.card_types.contains(*t)))
        .collect();
    if shared.is_empty() {
        return;
    }
    let opts: Vec<String> = shared.iter().map(|t| t.word().to_string()).collect();
    let i = g.ask_option(ctx.controller, Some(src), "Choose a card type", opts);
    let t = shared.get(i).copied().unwrap_or(shared[0]);
    g.objects[src.0 as usize].choices.card_type = Some(t);
    g.dirty = true;
}
