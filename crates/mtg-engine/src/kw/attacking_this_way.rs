//! "If you gain control of a creature this way, tap it, and it's attacking that player."
//! (Midnight Crusader Shuttle): an effect states that the creatures are attacking a player
//! (CR 508.4). Each becomes an attacking creature attacking that player, without being
//! declared as an attacker and unaffected by requirements and restrictions on declaring
//! attackers (CR 508.4c); it doesn't if that player has left the game (CR 508.4b), or
//! if it isn't a creature its controller attacks with (CR 506.3).
//!
//! [`Effect::Custom`] `attacking this way:<objects var>:<player>` (see
//! [`attacking_effect`]): the objects in the variable that the ability's controller
//! controls each attack the player (a [`PlayerRef`], as JSON).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// [`Effect::Custom`] prefix.
pub const ATTACKING: &str = "attacking this way:";

/// "[the objects in `objects` you control] are attacking [`who`]".
pub fn attacking_effect(objects: Var, who: &PlayerRef) -> Option<Effect> {
    let who = serde_json::to_string(who).ok()?;
    Some(Effect::Custom(format!("{ATTACKING}{objects}:{who}").into()))
}

/// The variable and player of an [`attacking_effect`] name.
pub fn parse_name(name: &str) -> Option<(Var, PlayerRef)> {
    let (a, b) = name.strip_prefix(ATTACKING)?.split_once(':')?;
    Some((a.parse().ok()?, serde_json::from_str(b).ok()?))
}

pub struct AttackingThisWay;

impl KeywordRules for AttackingThisWay {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((objects, who)) = parse_name(name) else {
            return false;
        };
        let Some(p) = g.eval_player(&who, ctx) else {
            return true;
        };
        let objs: Vec<ObjectId> = ctx
            .vars
            .get(&objects)
            .into_iter()
            .flatten()
            .filter_map(|e| e.object())
            .filter(|o| g.is_live(*o) && g.obj(*o).controller == ctx.controller)
            .collect();
        for o in objs {
            crate::combat::make_attacking(g, o, Entity::Player(p));
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&AttackingThisWay) }
