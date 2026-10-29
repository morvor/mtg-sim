//! "Return any number of Aura cards from your graveyard to the battlefield attached to
//! creatures you control" (Storm Herald): Aura cards put onto the battlefield by one
//! instruction, each attached to an object or player chosen for it among the ones the
//! instruction names (CR 303.4f), all at the same time. An Aura with none it could enchant
//! stays where it is (CR 303.4g). The text is parsed in `oracle/patterns/attach_moves.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Destination, Sel, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// The Aura cards to put onto the battlefield.
pub const AURAS: Var = vars::USER + 52030;
/// The objects or players they may be attached to.
pub const HOSTS: Var = vars::USER + 52031;
/// The host chosen for the Aura being prepared.
const HOST: Var = vars::USER + 52032;

/// `Effect::Custom`: the cards in [`AURAS`] are put onto the battlefield at the same time,
/// each attached to one of [`HOSTS`] it could enchant, chosen by the effect's controller;
/// "it" is the Auras that entered.
pub const PUT_AURAS_ONTO_CHOSEN_HOSTS: &str = "put the Aura cards onto the battlefield attached to chosen hosts";

pub struct AurasToChosenHosts;

impl KeywordRules for AurasToChosenHosts {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PUT_AURAS_ONTO_CHOSEN_HOSTS {
            return false;
        }
        let auras: Vec<ObjectId> = ctx
            .vars
            .get(&AURAS)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        let hosts: Vec<Entity> = ctx.vars.get(&HOSTS).cloned().unwrap_or_default();
        let mut to = Destination::battlefield();
        to.attached_to = Some(Sel::Var(HOST));
        let mut moves = Vec::new();
        let auras: Vec<ObjectId> = auras.into_iter().filter(|a| g.is_live(*a)).collect();
        for a in auras {
            let ok: Vec<Entity> = hosts
                .iter()
                .copied()
                .filter(|h| crate::attach::can_attach(g, a, *h))
                .collect();
            let Some(first) = ok.first().copied() else {
                continue;
            };
            let prompt = format!("Choose what {} will enchant", g.describe(a));
            let chosen = g.ask_entities(ctx.controller, ctx.source, &prompt, ok, 1, 1);
            ctx.set_var(HOST, vec![chosen.first().copied().unwrap_or(first)]);
            moves.extend(g.destination_moves(vec![a], &to, ctx));
        }
        let res: Vec<ObjectId> = if moves.is_empty() {
            vec![]
        } else {
            g.move_objects(moves).into_iter().flatten().collect()
        };
        let res: Vec<Entity> = res
            .into_iter()
            .filter(|o| g.obj(*o).zone == Zone::Battlefield)
            .map(Entity::Object)
            .collect();
        ctx.prev_happened = !res.is_empty();
        ctx.set_var(vars::IT, res);
        true
    }
}

inventory::submit! { KeywordRegistration(&AurasToChosenHosts) }
