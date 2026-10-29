//! Curse of Leeches // Leeching Lurker: "As this permanent transforms into Curse of
//! Leeches, attach it to a player."
//!
//! Applied while the permanent transforms (CR 712.20), through the compiled "As ~
//! transforms into ~, ..." replacement; the hand-written part is the attaching. Its
//! controller chooses a player it can be attached to (CR 701.3a: an Aura can't be
//! attached to a player it couldn't enchant; the choice of CR 303.4f), without targeting
//! (ruling: a player with hexproof can be chosen). If there's no such player it stays
//! unattached and is put into its owner's graveyard as a state-based action (CR 704.5m).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const ATTACH: &str = "card:Curse of Leeches // Leeching Lurker:attach it to a player";
const TEXT: &str = "As ~ transforms into ~, attach it to a player.";

inventory::submit! { ManualAbility {
    card: "Curse of Leeches // Leeching Lurker",
    face: 0,
    text: TEXT,
    build: |_| vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Transforms,
            action: ReplacementAction::AsEnters(Box::new(Effect::Custom(ATTACH.into()))),
            self_replacement: true,
            optional: false,
        }))),
        TEXT,
    )],
    reason: "attached to a chosen player as it transforms into an Aura: the only such card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != ATTACH {
            return false;
        }
        let Some(me) = ctx.source.filter(|s| g.is_live(*s)) else {
            return true;
        };
        let cands: Vec<Entity> = g
            .players_in_game()
            .into_iter()
            // CR 801.5a: a player within the chooser's range of influence.
            .filter(|p| crate::multiplayer::range::player_in_range(g, ctx.controller, *p))
            .map(Entity::Player)
            .filter(|e| crate::attach::can_attach(g, me, *e))
            .collect();
        let pick = g.ask_entities(ctx.controller, Some(me), "Attach it to a player", cands, 1, 1);
        ctx.prev_happened = match pick.first() {
            Some(p) => g.attach(me, *p),
            None => false,
        };
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
