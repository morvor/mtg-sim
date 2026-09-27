//! CR 702.189 Firebending.
//!
//! * "Firebending N" means "Whenever this creature attacks, add N {R}. Until end of
//!   combat, you don't lose this mana as steps and phases end." (CR 702.189a). It isn't a
//!   mana ability: it uses the stack. The mana is flagged
//!   [`crate::mana::Mana::until_end_of_combat`]: it stays as combat's steps end and is
//!   lost as the combat phase ends. Each instance triggers separately.
//! * "Firebending X, where X is [value]" keeps the value in [`Keyword::x`], determined as
//!   the ability resolves.
//! * "Whenever [a player] firebends" triggers whenever a firebending ability they control
//!   resolves (CR 702.189b): the ability reports a [`FIREBENT_EVENT`] as it resolves.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaType;
use smol_str::SmolStr;

/// `Event::Custom` name: a player firebent (a firebending ability they control resolved).
pub const FIREBENT_EVENT: &str = "firebend";
/// `Effect::Custom`: adds the mana (the amount is in [`COUNT`]) and reports the event.
const FIREBEND: &str = "firebending:add red mana until end of combat";
/// The number variable holding how much mana to add.
const COUNT: Var = vars::USER + 189;

fn count(kw: &Keyword) -> Value {
    match (&kw.x, kw.n) {
        (Some(x), _) => x.clone(),
        (None, Some(n)) => Value::c(n.max(0)),
        (None, None) => Value::c(1),
    }
}

pub struct Firebending;

impl KeywordRules for Firebending {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Firebending]
    }

    fn x_determined_on_resolution(&self) -> bool {
        true
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::effect(Effect::Seq(vec![
                    Effect::StoreValue {
                        var: COUNT,
                        value: count(kw),
                    },
                    Effect::Custom(SmolStr::new(FIREBEND)),
                ])),
            )),
            KeywordKind::Firebending.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != FIREBEND {
            return false;
        }
        let n = ctx.nums.get(&COUNT).copied().unwrap_or(0).max(0);
        let p = ctx.controller;
        let before = g.player(p).mana_pool.mana.len();
        g.exec(
            &Effect::AddMana {
                who: PlayerRef::You,
                mana: ManaProduction::Amount(ManaType::R, Value::c(n as i32)),
                restriction: None,
            },
            ctx,
        );
        let pool = &mut g.players[p.idx()].mana_pool.mana;
        let from = before.min(pool.len());
        for m in &mut pool[from..] {
            m.until_end_of_combat = true;
        }
        g.log(|_| format!("{p} firebends {n}"));
        crate::kwa::emit(g, FIREBENT_EVENT, p, ctx.source, n as i32);
        true
    }
}

inventory::submit! { KeywordRegistration(&Firebending) }
