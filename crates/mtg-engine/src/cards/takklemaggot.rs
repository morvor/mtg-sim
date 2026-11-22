//! Takklemaggot: "When enchanted creature dies, that creature's controller chooses a
//! creature that ~ could enchant. If the player does, return ~ to the battlefield under
//! your control attached to that creature. If they don't, return ~ to the battlefield
//! under your control as a non-Aura enchantment. It loses \"enchant creature\" and gains
//! \"At the beginning of that player's upkeep, ~ deals 1 damage to that player.\""
//!
//! The trigger condition is the compiled one; the return is hand-written. Rulings: the
//! creature's controller chooses as the ability resolves (no target, so a creature with
//! shroud can be chosen, one with protection from black can't), and must choose one if
//! they can, whoever controls it; Takklemaggot returns only if it's still in the
//! graveyard, under the control of the player who controlled it (CR 303.4f: an Aura put
//! onto the battlefield this way doesn't target). With no creature to choose it returns
//! as a non-Aura enchantment (losing the subtype Aura and its enchant ability, CR 303.4i)
//! that deals 1 damage to that player at the beginning of each of their upkeeps.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{Entity, ObjectId, PlayerId};

const TEXT: &str = "When enchanted creature dies, that creature's controller chooses a creature that ~ could enchant. If the player does, return ~ to the battlefield under your control attached to that creature. If they don't, return ~ to the battlefield under your control as a non-Aura enchantment. It loses \"enchant creature\" and gains \"At the beginning of that player's upkeep, ~ deals 1 damage to that player.\"";
const RETURN: &str = "card:Takklemaggot:return it attached to a creature, or as a non-Aura enchantment";
const ITS_PLAYER: &str = "card:Takklemaggot:it's that player's upkeep";
/// Rows `[enchantment, player]`: "that player" of a returned non-Aura Takklemaggot.
const PLAYERS: &str = "card:Takklemaggot:that player";
const GAINED: &str = "At the beginning of that player's upkeep, ~ deals 1 damage to that player.";

const CARD: Var = vars::USER + 61;
const CHOSEN: Var = vars::USER + 62;

inventory::submit! { ManualAbility {
    card: "Takklemaggot",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When enchanted creature dies, you gain 1 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(RETURN.into())))
            .collect()
    },
    reason: "returns as a non-Aura enchantment aimed at the dead creature's controller: only this card",
} }

fn gained_trigger() -> Ability {
    AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility {
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::Any,
            },
            intervening_if: Some(Condition::Custom(ITS_PLAYER.into())),
            body: Body::simple(
                vec![],
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::Const(1),
                    to: Sel::Players(PlayerRef::ActivePlayer),
                },
            ),
            once_per_turn: false,
            is_mana_ability: false,
            zone: FunctionZone::Battlefield,
            cant_be_countered: false,
            do_once_per_turn: false,
            limit: None,
        }),
        GAINED,
    )
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != ITS_PLAYER {
            return None;
        }
        let Some(src) = ctx.source else {
            return Some(false);
        };
        let active = g.turn.active.0 as i64;
        Some(
            g.cards
                .get(PLAYERS)
                .iter()
                .any(|r| r[0] == src.0 as i64 && r[1] == active),
        )
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RETURN {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        // "that creature's controller", as it last existed.
        let Some(player): Option<PlayerId> = g.eval_player(
            &PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
            ctx,
        ) else {
            return true;
        };
        // Only if it's still in the graveyard it went to.
        let card = g.current(src);
        if !g.is_live(card) || !matches!(g.obj(card).zone, Zone::Graveyard(_)) {
            return true;
        }
        if g.dirty {
            g.recompute();
        }
        let candidates: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.is_creature())
            .map(|o| o.id)
            .filter(|c| crate::attach::can_attach(g, card, Entity::Object(*c)))
            .collect();
        let chosen = if candidates.is_empty() {
            None
        } else {
            g.ask_objects(
                player,
                Some(card),
                "Choose a creature for Takklemaggot to enchant",
                candidates.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .filter(|c| candidates.contains(c))
            .or(candidates.first().copied())
        };
        ctx.set_var(CARD, vec![Entity::Object(card)]);
        let mut to = Destination::zone(ZoneKind::Battlefield);
        to.controller = Some(PlayerRef::You);
        match chosen {
            Some(c) => {
                ctx.set_var(CHOSEN, vec![Entity::Object(c)]);
                to.attached_to = Some(Sel::Var(CHOSEN));
            }
            None => {
                to.with_mods = vec![Modification::RemoveSubtypes(vec!["Aura".into()])];
                if let Some(k) = g
                    .obj(card)
                    .chars
                    .keywords()
                    .find(|k| k.kind == KeywordKind::Enchant)
                    .cloned()
                {
                    to.with_mods.push(Modification::LoseKeyword(k));
                }
                to.with_mods.push(Modification::AddAbility(gained_trigger()));
            }
        }
        g.exec(
            &Effect::Move {
                what: Sel::Var(CARD),
                to,
            },
            ctx,
        );
        if chosen.is_none() {
            let now = g.current(card);
            if g.is_live(now) && g.obj(now).zone == Zone::Battlefield {
                g.cards
                    .push(PLAYERS, vec![now.0 as i64, player.0 as i64]);
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
