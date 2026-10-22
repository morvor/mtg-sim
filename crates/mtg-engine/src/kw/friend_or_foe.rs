//! "For each player, choose friend or foe. Each friend [does something]. Each foe [does
//! something]." (the Battlebond spells: Pir's Whim, Virtus's Maneuver, ...): as the spell
//! resolves, its controller calls each player a friend or a foe — any player may be
//! either, even one who couldn't do what the spell will instruct. The designation matters
//! only to this spell. Then the friends act (in APNAP order) before the foes.
//!
//! The choice is `Effect::Custom(`[`CHOOSE`]`)`, which stores the friends and the foes,
//! in APNAP order, in the variables [`FRIENDS`] and [`FOES`] of the resolving spell; "each
//! friend"/"each foe" clauses refer to them (`oracle/patterns/friend_or_foe.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Var};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::Entity;

/// `Effect::Custom`: "for each player, choose friend or foe".
pub const CHOOSE: &str = "friend or foe:choose";
/// The players the controller called friends.
pub const FRIENDS: Var = vars::USER + 30000;
/// The players the controller called foes.
pub const FOES: Var = vars::USER + 30001;

pub struct FriendOrFoe;

impl KeywordRules for FriendOrFoe {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CHOOSE {
            return false;
        }
        choose(g, ctx);
        true
    }
}

inventory::submit! { KeywordRegistration(&FriendOrFoe) }

/// The controller calls each player (in APNAP order) a friend or a foe. By default,
/// themselves and their teammates are friends and the other players foes.
fn choose(g: &mut Game, ctx: &mut Ctx) {
    let me = ctx.controller;
    let mut friends = Vec::new();
    let mut foes = Vec::new();
    for p in g.apnap() {
        // Options: 0 = friend, 1 = foe.
        let friend = match g.ask(
            me,
            Decision::ChooseOption {
                source: ctx.source,
                prompt: format!("Choose friend or foe for {p}"),
                options: vec!["Friend".to_string(), "Foe".to_string()],
            },
        ) {
            Answer::Index(0) => true,
            Answer::Index(1) => false,
            _ => !g.are_opponents(me, p),
        };
        if friend {
            friends.push(Entity::Player(p));
        } else {
            foes.push(Entity::Player(p));
        }
    }
    ctx.set_var(FRIENDS, friends);
    ctx.set_var(FOES, foes);
}
