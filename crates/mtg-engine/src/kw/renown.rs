//! CR 702.112 Renown.
//!
//! * "Renown N" means "When this creature deals combat damage to a player, if it isn't
//!   renowned, put N +1/+1 counters on it and it becomes renowned." (CR 702.112a): an
//!   intervening "if" clause (CR 603.4), checked as it triggers and as it resolves.
//! * Renowned is a designation of permanents (`GameObject::renowned`, CR 702.112b): only a
//!   permanent can become renowned, and it stays renowned until it leaves the battlefield
//!   (a new object isn't renowned, CR 400.7). It's neither an ability nor a copiable
//!   value, so losing abilities doesn't end it and copies don't have it; the filter
//!   [`RENOWNED`] checks the designation itself.
//! * With several instances, each triggers; the first to resolve makes it renowned and
//!   the others do nothing (CR 702.112c).
//! * Becoming renowned reports a [`RENOWNED`] event (`Event::Custom`), so abilities that
//!   trigger "when [this] becomes renowned" can see it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom` name ("[this] is renowned") and `Event::Custom` name (a permanent
/// became renowned).
pub const RENOWNED: &str = "renowned";

/// `Effect::Custom`: the source of the resolving ability becomes renowned.
const BECOME_RENOWNED: &str = "renown:it becomes renowned";

/// Whether `id` is a renowned permanent.
pub fn is_renowned(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.renowned && o.zone == Zone::Battlefield
}

/// Makes the permanent `id` renowned. Returns false if it isn't a permanent (only
/// permanents can become renowned, CR 702.112b) or already was renowned.
pub fn become_renowned(g: &mut Game, id: ObjectId) -> bool {
    if !g.is_live(id) || g.obj(id).zone != Zone::Battlefield || g.obj(id).renowned {
        return false;
    }
    g.objects[id.0 as usize].renowned = true;
    g.dirty = true;
    let p = g.obj(id).controller;
    g.log(|g| format!("{} becomes renowned", g.describe(id)));
    g.emit(crate::events::Event::Custom {
        name: SmolStr::new(RENOWNED),
        player: Some(p),
        obj: Some(id),
        amount: 0,
    });
    true
}

pub struct Renown;

impl KeywordRules for Renown {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Renown]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        let mut t = TriggeredAbility::new(
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Player(PlayerRel::Any),
                combat_only: true,
            },
            Body::effect(Effect::Seq(vec![
                Effect::AddCounters {
                    what: Sel::This,
                    kind: counters::PLUS1.into(),
                    n: Value::c(n),
                },
                Effect::Custom(BECOME_RENOWNED.into()),
            ])),
        );
        t.intervening_if = Some(Condition::Not(Box::new(Condition::SelMatches(
            Sel::This,
            Filter::Custom(RENOWNED.into()),
        ))));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            format!("Renown {n}"),
        )])
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == RENOWNED).then(|| is_renowned(g, id))
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != BECOME_RENOWNED {
            return false;
        }
        ctx.prev_happened = ctx.source.is_some_and(|s| become_renowned(g, s));
        true
    }
}

inventory::submit! { KeywordRegistration(&Renown) }
