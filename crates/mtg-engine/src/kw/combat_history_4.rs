//! Combat history and combat-scoped qualities (cards3-combat-statics):
//!
//! * "each creature it blocked this combat" (Cathedral Membrane): the attacking creatures
//!   the source blocked, judged as it last existed if it has left the battlefield
//!   (CR 509.1, 608.2h).
//! * "unless defending player controls three or more creatures that share a creature type"
//!   (Graxiplon).
//! * "[creatures] that's attacking a planeswalker you control" in an attack tax (Onakke
//!   Oathkeeper): the creature being declared attacks a planeswalker (CR 508.1d, 508.1h).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: an attacking creature the source blocked this combat.
pub const SOURCE_BLOCKED_THIS_COMBAT: &str = "combat4:creature ~ blocked this combat";
/// `Condition::Custom` prefix, followed by the number: the defending player controls that
/// many creatures that share a creature type.
pub const DEFENDING_SHARE_A_TYPE: &str =
    "combat4:defending player controls N creatures that share a creature type:";
/// `Filter::Custom`: in an attack tax, the creature is being declared attacking a
/// planeswalker (the attack target is [`ATTACK_TARGET`]).
pub const ATTACKING_A_PLANESWALKER: &str = "combat4:being declared attacking a planeswalker";
/// `Effect::Custom` prefix, followed by "[player slot]:[creatures slot]": the target
/// attacking creatures are now attacking the target player (CR 508.7).
pub const NOW_ATTACKING: &str = "combat4:now attacking:";
/// While the costs to attack or block are paid: the creatures being declared as
/// attackers or blockers (CR 508.1h, 509.1d).
pub const BEING_DECLARED: Var = vars::USER + 13375;
/// `Filter::Custom`: not one of the creatures being declared as attackers or blockers
/// this combat (nor one declared earlier this combat).
pub const NOT_DECLARED: &str = "combat4:not declared as an attacking or blocking creature";
/// `TriggerCond::Custom`: "you attack enchanted opponent or a planeswalker they control".
pub const YOU_ATTACK_ENCHANTED: &str = "combat4:you attack enchanted player or their planeswalker";
/// `TriggerCond::Custom`: "[enchanted opponent] attack(s) you or a planeswalker you
/// control".
pub const ENCHANTED_ATTACKS_YOU: &str = "combat4:enchanted player attacks you or your planeswalker";
/// The attacking creature an "each creature you control attacking a player gets ... for
/// each creature that player controls" instruction is being performed for.
pub const EACH_ATTACKER: Var = vars::USER + 13376;
/// `Filter::Custom`: controlled by the player [`EACH_ATTACKER`] is attacking.
pub const CONTROLLED_BY_ITS_DEFENDER: &str = "combat4:controlled by the player that creature attacks";
/// While an attack tax is determined: what the creature would attack.
pub const ATTACK_TARGET: Var = vars::USER + 13373;

/// The object `id` and the objects it was before (CR 400.7) and has become since.
fn incarnations(g: &Game, id: ObjectId) -> Vec<ObjectId> {
    let mut v = vec![id];
    let mut cur = id;
    while let Some(p) = g.obj(cur).prev {
        if v.contains(&p) {
            break;
        }
        v.push(p);
        cur = p;
    }
    let now = g.current(id);
    if !v.contains(&now) {
        v.push(now);
    }
    v
}

pub struct CombatHistory4;

impl KeywordRules for CombatHistory4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if let Some(n) = name.strip_prefix(DEFENDING_SHARE_A_TYPE) {
            let n: usize = n.parse().ok()?;
            let Some(p) = g.defending_player_for(ctx) else {
                return Some(false);
            };
            return Some(
                crate::oracle::patterns::creatures_share_a_type::player_has_n_sharing(g, p, n),
            );
        }
        None
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(r) = name.strip_prefix(NOW_ATTACKING) else {
            return false;
        };
        let Some((p, c)) = r
            .split_once(':')
            .and_then(|(p, c)| Some((p.parse::<u8>().ok()?, c.parse::<u8>().ok()?)))
        else {
            return true;
        };
        let Some(player) = g.resolve_sel(&Sel::Target(p), ctx).first().and_then(|e| e.player())
        else {
            return true;
        };
        for a in g.resolve_objects(&Sel::Target(c), ctx) {
            crate::combat::reselect_attack_target(g, a, Entity::Player(player));
        }
        true
    }

    /// "When you attack enchanted opponent or a planeswalker they control or when they
    /// attack you or a planeswalker you control" (Tenuous Truce): attackers are declared
    /// (CR 508.3a) attacking that player or a planeswalker of theirs.
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &crate::events::Event,
    ) -> Option<Vec<crate::object::EventInfo>> {
        let you_attack = match name {
            YOU_ATTACK_ENCHANTED => true,
            ENCHANTED_ATTACKS_YOU => false,
            _ => return None,
        };
        let crate::events::Event::AttackersDeclared {
            player, attackers, ..
        } = ev
        else {
            return Some(vec![]);
        };
        let Some(Entity::Player(enchanted)) = g.obj(src).attached_to else {
            return Some(vec![]);
        };
        let (attacker, defender) = if you_attack {
            (ctl, enchanted)
        } else {
            (enchanted, ctl)
        };
        if *player != attacker {
            return Some(vec![]);
        }
        let hit = attackers.iter().any(|(_, t)| match t {
            Entity::Player(p) => *p == defender,
            Entity::Object(o) => {
                g.obj(*o).is(CardType::Planeswalker) && g.obj(*o).controller == defender
            }
        });
        Some(if hit {
            vec![crate::object::EventInfo {
                player: Some(defender),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        match name {
            SOURCE_BLOCKED_THIS_COMBAT => {
                let Some(c) = g.combat.as_ref() else {
                    return Some(false);
                };
                let Some(src) = ctx.source else {
                    return Some(false);
                };
                let ids = incarnations(g, src);
                Some(ids.iter().any(|b| {
                    (g.is_live(*b) && c.blockers_of(id).contains(b))
                        || g.left_combat_pair(*b, *b, id)
                }))
            }
            NOT_DECLARED => {
                let now = ctx
                    .vars
                    .get(&BEING_DECLARED)
                    .is_some_and(|v| v.contains(&Entity::Object(id)));
                let before = g.combat.as_ref().is_some_and(|c| {
                    c.declared_attackers.iter().any(|(a, _)| *a == id)
                        || c.declared_blockers.contains(&id)
                });
                Some(!now && !before)
            }
            CONTROLLED_BY_ITS_DEFENDER => {
                let attacker = ctx
                    .vars
                    .get(&EACH_ATTACKER)
                    .and_then(|v| v.first())
                    .and_then(|e| e.object());
                let defender = attacker.and_then(|a| {
                    g.combat
                        .as_ref()
                        .and_then(|c| c.attack_target(a))
                        .and_then(|t| t.player())
                });
                Some(defender.is_some_and(|p| g.obj(id).controller == p))
            }
            ATTACKING_A_PLANESWALKER => Some(
                ctx.vars
                    .get(&ATTACK_TARGET)
                    .and_then(|v| v.first())
                    .and_then(|e| e.object())
                    .is_some_and(|o| g.obj(o).is(CardType::Planeswalker)),
            ),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&CombatHistory4) }
