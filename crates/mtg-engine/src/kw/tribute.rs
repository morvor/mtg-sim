//! CR 702.104 Tribute: "Tribute N" means "As this creature enters, choose an opponent.
//! That player may put an additional N +1/+1 counters on it as it enters." (CR 702.104a).
//! It's a replacement effect: the choices are made as the creature enters, too late to
//! respond to the creature spell, and the counters are on it as it enters.
//!
//! "If tribute wasn't paid" (CR 702.104b, [`TRIBUTE_NOT_PAID`]) is true if the chosen
//! opponent didn't have it enter with those counters. What happened is recorded on the
//! entering object (linked choices under [`TRIBUTE_LINK`]) and carried onto the permanent,
//! so an ability checking it can still tell after the permanent left the battlefield.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The key (in `GameObject::linked_choices`) of what happened with an object's tribute:
/// the chosen opponent, and whether tribute was paid ([`PAID`]).
pub const TRIBUTE_LINK: u16 = 0x7ffc;
/// `Condition::Custom`: "if tribute wasn't paid".
pub const TRIBUTE_NOT_PAID: &str = "tribute:tribute wasn't paid";
/// `Effect::Custom` prefix: the tribute replacement effect, followed by N.
const TRIBUTE: &str = "tribute:";
/// The recorded text of paid tribute.
const PAID: &str = "tribute paid";
const NOT_PAID: &str = "tribute not paid";

pub struct Tribute;

impl KeywordRules for Tribute {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Tribute]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Custom(SmolStr::new(
                format!("{TRIBUTE}{n}"),
            )))),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            format!("Tribute {n}"),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(n) = name.strip_prefix(TRIBUTE) else {
            return false;
        };
        let n: u32 = n.parse().unwrap_or(0);
        if let Some(this) = ctx.source {
            tribute(g, ctx, this, n);
        }
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == TRIBUTE_NOT_PAID).then(|| {
            ctx.source.is_none_or(|s| {
                g.obj(s)
                    .linked_choices
                    .get(&TRIBUTE_LINK)
                    .and_then(|c| c.text.as_deref())
                    != Some(PAID)
            })
        })
    }
}

/// As `this` enters under the context's controller: they choose an opponent, who may have
/// it enter with `n` additional +1/+1 counters.
fn tribute(g: &mut Game, ctx: &mut Ctx, this: ObjectId, n: u32) {
    let p = ctx.controller;
    let opponents: Vec<Entity> = g
        .opponents(p)
        .into_iter()
        .filter(|q| g.player(*q).in_game())
        .map(Entity::Player)
        .collect();
    let chosen = g
        .ask_entities(
            p,
            Some(this),
            "Choose an opponent (tribute)",
            opponents,
            1,
            1,
        )
        .first()
        .and_then(|e| e.player());
    let name = g.obj(this).chars.name.clone();
    let paid = chosen.is_some_and(|q| {
        g.ask_yes_no(
            q,
            Some(this),
            &format!("Pay tribute: put {n} +1/+1 counters on {name}?"),
            false,
        )
    });
    if paid {
        if let Some(em) = ctx.entering.as_mut() {
            em.counters.push((SmolStr::new(counters::PLUS1), n));
        }
    }
    let rec = g.objects[this.0 as usize]
        .linked_choices
        .entry(TRIBUTE_LINK)
        .or_default();
    rec.player = chosen;
    rec.text = Some(SmolStr::new(if paid { PAID } else { NOT_PAID }));
    g.log(|_| {
        format!(
            "{name}: tribute {}",
            if paid { "paid" } else { "not paid" }
        )
    });
}

inventory::submit! { KeywordRegistration(&Tribute) }
