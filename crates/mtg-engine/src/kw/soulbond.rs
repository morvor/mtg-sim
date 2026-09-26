//! CR 702.95 Soulbond.
//!
//! "Soulbond" represents two triggered abilities (CR 702.95a): "When this creature enters,
//! if you control both this creature and another creature and both are unpaired, you may
//! pair this creature with another unpaired creature you control for as long as both
//! remain creatures on the battlefield under your control" and "Whenever another creature
//! you control enters, if you control both that creature and this one and both are
//! unpaired, you may pair that creature with this creature for as long as both remain
//! creatures on the battlefield under your control."
//!
//! A pair is recorded in [`GameObject::paired_with`] on both creatures (CR 702.95b, a
//! creature is paired with at most one other, CR 702.95d). It's broken when either leaves
//! the battlefield (see `actions.rs`), or when control of either changes or either stops
//! being a creature ([`update_pairs`], called as characteristics are computed, CR 702.95e).
//!
//! Abilities of paired creatures ("As long as ~ is paired with another creature, each of
//! those creatures gets +1/+1") use the condition [`IS_PAIRED`] and the filter
//! [`THE_PAIR`] (see `oracle/patterns/k702_084_097.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: "if you control both this creature and another creature and both
/// are unpaired".
const THIS_AND_ANOTHER_UNPAIRED: &str = "soulbond:you control this and another creature, unpaired";
/// `Condition::Custom`: "if you control both that creature and this one and both are
/// unpaired" (that creature: the one that entered).
const THAT_AND_THIS_UNPAIRED: &str = "soulbond:you control that creature and this, unpaired";
/// `Effect::Custom`: "you may pair this creature with another unpaired creature you
/// control".
const PAIR_WITH_ANOTHER: &str = "soulbond:pair this with another unpaired creature you control";
/// `Effect::Custom`: "you may pair that creature with this creature".
const PAIR_THAT_WITH_THIS: &str = "soulbond:pair that creature with this";
/// `Condition::Custom`: "~ is paired with another creature".
pub const IS_PAIRED: &str = "soulbond:this is paired with another creature";
/// `Filter::Custom`: "each of those creatures" / "both creatures": the source and the
/// creature it's paired with, while it's paired.
pub const THE_PAIR: &str = "soulbond:this and the creature it's paired with";

/// The creature `id` is paired with, if it's paired (CR 702.95b).
pub fn partner(g: &Game, id: ObjectId) -> Option<ObjectId> {
    let o = g.obj(id);
    let p = o.paired_with?;
    let ok = g.is_live(id)
        && g.is_live(p)
        && o.zone == Zone::Battlefield
        && g.obj(p).zone == Zone::Battlefield
        && g.obj(p).paired_with == Some(id);
    ok.then_some(p)
}

/// Whether `id` is an unpaired creature on the battlefield controlled by `p`.
fn unpaired_creature_of(g: &Game, id: ObjectId, p: PlayerId) -> bool {
    let o = g.obj(id);
    g.is_live(id)
        && o.zone == Zone::Battlefield
        && o.is_creature()
        && o.controller == p
        && partner(g, id).is_none()
}

/// The other unpaired creatures `p` controls.
fn other_unpaired(g: &Game, this: ObjectId, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .map(|o| o.id)
        .filter(|id| *id != this && unpaired_creature_of(g, *id, p))
        .collect()
}

/// Pairs `a` and `b` for the soulbond ability controlled by `p`, unless either is no
/// longer a creature on the battlefield under `p`'s control or already paired
/// (CR 702.95c, 702.95d).
fn pair(g: &mut Game, a: ObjectId, b: ObjectId, p: PlayerId) {
    if a == b || !unpaired_creature_of(g, a, p) || !unpaired_creature_of(g, b, p) {
        return;
    }
    g.objects[a.0 as usize].paired_with = Some(b);
    g.objects[b.0 as usize].paired_with = Some(a);
    g.dirty = true;
    g.log(|g| format!("{} is paired with {}", g.describe(a), g.describe(b)));
}

/// CR 702.95e: breaks the pairs of creatures whose control changed (`control_changed`),
/// that stopped being creatures, or whose partner left the battlefield. Returns true if a
/// pair was broken.
pub fn update_pairs(g: &mut Game, control_changed: &[ObjectId]) -> bool {
    let paired: Vec<(ObjectId, ObjectId)> = g
        .battlefield
        .iter()
        .filter_map(|id| g.obj(*id).paired_with.map(|p| (*id, p)))
        .collect();
    let mut broke = false;
    for (a, b) in paired {
        let intact = partner(g, a) == Some(b)
            && !control_changed.contains(&a)
            && !control_changed.contains(&b)
            && g.obj(a).is_creature()
            && g.obj(b).is_creature()
            && g.obj(a).controller == g.obj(b).controller;
        if intact {
            continue;
        }
        g.objects[a.0 as usize].paired_with = None;
        if g.obj(b).paired_with == Some(a) {
            g.objects[b.0 as usize].paired_with = None;
        }
        broke = true;
    }
    broke
}

pub struct Soulbond;

impl KeywordRules for Soulbond {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Soulbond]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut enters = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::Custom(SmolStr::new(PAIR_WITH_ANOTHER))),
        );
        enters.intervening_if = Some(Condition::Custom(THIS_AND_ANOTHER_UNPAIRED.into()));
        let mut other = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::and(vec![
                Filter::creature(),
                Filter::Other,
                Filter::ControlledBy(PlayerRel::You),
            ])),
            Body::effect(Effect::Custom(SmolStr::new(PAIR_THAT_WITH_THIS))),
        );
        other.intervening_if = Some(Condition::Custom(THAT_AND_THIS_UNPAIRED.into()));
        Some(vec![
            AbilityDef::new(AbilityKind::Triggered(enters), KeywordKind::Soulbond.name()),
            AbilityDef::new(AbilityKind::Triggered(other), KeywordKind::Soulbond.name()),
        ])
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let this = ctx.source;
        let p = ctx.controller;
        match name {
            THIS_AND_ANOTHER_UNPAIRED => Some(this.is_some_and(|s| {
                unpaired_creature_of(g, s, p) && !other_unpaired(g, s, p).is_empty()
            })),
            THAT_AND_THIS_UNPAIRED => {
                let that = ctx.event.as_ref().and_then(|e| e.object);
                Some(match (this, that) {
                    (Some(s), Some(o)) => {
                        s != o && unpaired_creature_of(g, s, p) && unpaired_creature_of(g, o, p)
                    }
                    _ => false,
                })
            }
            IS_PAIRED => Some(this.is_some_and(|s| partner(g, s).is_some())),
            _ => None,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != THE_PAIR {
            return None;
        }
        let Some(s) = ctx.source else {
            return Some(false);
        };
        Some(match partner(g, s) {
            Some(p) => id == s || id == p,
            None => false,
        })
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let p = ctx.controller;
        match name {
            PAIR_WITH_ANOTHER => {
                let Some(this) = ctx.source else {
                    return true;
                };
                if !unpaired_creature_of(g, this, p) {
                    return true;
                }
                let cands = other_unpaired(g, this, p);
                let pick = g.ask_objects(
                    p,
                    Some(this),
                    "Soulbond: pair this creature with another unpaired creature you control?",
                    cands,
                    0,
                    1,
                );
                if let Some(other) = pick.first() {
                    pair(g, this, *other, p);
                }
                true
            }
            PAIR_THAT_WITH_THIS => {
                let (Some(this), Some(that)) =
                    (ctx.source, ctx.event.as_ref().and_then(|e| e.object))
                else {
                    return true;
                };
                if !unpaired_creature_of(g, this, p) || !unpaired_creature_of(g, that, p) {
                    return true;
                }
                let prompt = format!(
                    "Soulbond: pair {} with {}?",
                    g.describe(that),
                    g.describe(this)
                );
                if g.ask_yes_no(p, Some(this), &prompt, true) {
                    pair(g, that, this, p);
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&Soulbond) }
