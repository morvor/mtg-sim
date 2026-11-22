//! Pygmy Hippo: "Whenever ~ attacks and isn't blocked, you may have defending player
//! activate a mana ability of each land they control and lose all unspent mana. If you do,
//! ~ assigns no combat damage this turn and at the beginning of your next main phase this
//! turn, you add an amount of {C} equal to the amount of mana that player lost this way."

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::ManaType;
use crate::types::CardType;

const DRAIN: &str = "card:Pygmy Hippo:drain the defending player's lands";
const TEXT: &str = "Whenever ~ attacks and isn't blocked, you may have defending player activate a mana ability of each land they control and lose all unspent mana. If you do, ~ assigns no combat damage this turn and at the beginning of your next main phase this turn, you add an amount of {C} equal to the amount of mana that player lost this way.";

inventory::submit! { ManualAbility {
    card: "Pygmy Hippo",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever ~ attacks and isn't blocked, ~ assigns no combat damage this turn.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(DRAIN.into())))
            .collect()
    },
    reason: "drains the defending player's lands and gives you the mana lost at your next main phase: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != DRAIN {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let Some(dp) = g.combat.as_ref().and_then(|c| c.defending_player_of(g, src)) else {
            return true;
        };
        if !g.ask_yes_no(ctx.controller, Some(src), "Drain the defending player's lands?", false) {
            return true;
        }
        g.exec(
            &Effect::ActivateManaAbilities {
                who: PlayerRef::Player(dp),
                filter: Filter::Type(CardType::Land),
            },
            ctx,
        );
        let lost = g.player(dp).mana_pool.total() as i32;
        g.exec(
            &Effect::LoseUnspentMana {
                who: PlayerRef::Player(dp),
                to: None,
            },
            ctx,
        );
        g.exec(
            &Effect::AddRestriction {
                restriction: Restriction::AssignsNoCombatDamage(Filter::In(Box::new(Sel::This))),
                duration: Duration::EndOfTurn,
            },
            ctx,
        );
        if lost > 0 {
            let main = |step| TriggerCond::BeginningOf {
                step,
                whose: PlayerRel::You,
            };
            g.exec(
                &Effect::DelayedTrigger {
                    trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::AnyOf(vec![
                        main(TriggerStep::PrecombatMain),
                        main(TriggerStep::PostcombatMain),
                    ]))),
                    body: Box::new(Body::effect(Effect::AddMana {
                        who: PlayerRef::You,
                        mana: ManaProduction::Amount(ManaType::C, Value::Const(lost)),
                        restriction: None,
                    })),
                    once: true,
                },
                ctx,
            );
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
