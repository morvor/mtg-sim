//! CR 701.54: the Ring tempts you.
//!
//! * Each time the Ring tempts a player, they choose a creature they control; it becomes
//!   their Ring-bearer until another creature becomes their Ring-bearer or another player
//!   gains control of it (CR 701.54a). `Player::ring_bearer` records it.
//! * Ring-bearer is a designation, not a copiable value (CR 701.54b).
//! * A player tempted by the Ring who doesn't have an emblem named The Ring gets one first
//!   (CR 701.54c). The Ring has "Your Ring-bearer is legendary and can't be blocked by
//!   creatures with greater power", and, as the Ring has tempted that player two, three,
//!   and four or more times (`Player::ring_level`), "Whenever your Ring-bearer attacks,
//!   draw a card, then discard a card", "Whenever your Ring-bearer becomes blocked by a
//!   creature, the blocking creature's controller sacrifices it at end of combat", and
//!   "Whenever your Ring-bearer deals combat damage to a player, each opponent loses 3
//!   life".
//! * The Ring tempts a player once they complete those actions, even if some or all of
//!   them were impossible (CR 701.54d): a [`RING_TEMPTS`] event is reported (with the
//!   chosen creature, if any), and a [`RING_BEARER_CHOSEN`] event if a creature was
//!   chosen.
//! * "Is your Ring-bearer" is true only for a creature on the battlefield under your
//!   control with the designation (CR 701.54e): the filter [`RING_BEARER`] ("your
//!   Ring-bearer", relative to the evaluating ability's controller).

use super::*;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Event::Custom` name: the Ring tempted a player (the object is the creature chosen as
/// their Ring-bearer, if any; the amount is how many times the Ring has tempted them).
pub const RING_TEMPTS: &str = "the ring tempts you";
/// `Event::Custom` name: a player chose a creature as their Ring-bearer.
pub const RING_BEARER_CHOSEN: &str = "ring-bearer chosen";
/// `Filter::Custom` name: "your Ring-bearer".
pub const RING_BEARER: &str = "your ring-bearer";
/// `Value::Custom` name: how many times the Ring has tempted you.
pub const TIMES_TEMPTED: &str = "times the ring has tempted you";

/// Whether `id` is `p`'s Ring-bearer (CR 701.54e): on the battlefield, under their
/// control, with the designation.
pub fn is_ring_bearer(g: &Game, p: PlayerId, id: ObjectId) -> bool {
    g.player(p).ring_bearer == Some(id) && on_battlefield(g, id) && g.obj(id).controller == p
}

/// `p`'s Ring-bearer, if they have one.
pub fn ring_bearer(g: &Game, p: PlayerId) -> Option<ObjectId> {
    g.player(p)
        .ring_bearer
        .filter(|id| is_ring_bearer(g, p, *id))
}

/// `p`'s emblem named The Ring.
pub fn the_ring(g: &Game, p: PlayerId) -> Option<ObjectId> {
    g.command.iter().copied().find(|id| {
        let o = g.obj(*id);
        o.kind == crate::object::ObjKind::Emblem && o.owner == p && o.base.name == "The Ring"
    })
}

fn ring_bearer_filter() -> Filter {
    Filter::Custom(SmolStr::new(RING_BEARER))
}

/// "As long as the Ring has tempted you N or more times".
fn tempted_at_least(n: i32) -> Condition {
    Condition::Compare(
        Value::Custom(SmolStr::new(TIMES_TEMPTED)),
        Cmp::Ge,
        Value::c(n),
    )
}

fn emblem_trigger(trigger: TriggerCond, level: i32, effect: Effect, text: &str) -> Ability {
    let mut t = TriggeredAbility::new(
        TriggerCond::Where {
            trigger: Box::new(trigger),
            cond: tempted_at_least(level),
        },
        Body::effect(effect),
    );
    t.zone = FunctionZone::Command;
    AbilityDef::new(AbilityKind::Triggered(t), text)
}

/// The abilities of the emblem named The Ring (CR 701.54c). Those it has only as long as
/// the Ring has tempted its owner enough times trigger only then.
fn ring_abilities() -> Vec<Ability> {
    let mut legendary = StaticAbility::new(StaticEffect::Continuous {
        affected: ring_bearer_filter(),
        mods: vec![Modification::AddSupertypes(vec![Supertype::Legendary])],
    });
    legendary.zone = FunctionZone::Command;
    // The blocking creature (stored as the ability resolves) is sacrificed at end of
    // combat by its controller.
    const BLOCKER: Var = vars::USER + 1054;
    vec![
        AbilityDef::new(
            AbilityKind::Static(legendary),
            "Your Ring-bearer is legendary and can't be blocked by creatures with greater power.",
        ),
        emblem_trigger(
            TriggerCond::Attacks(ring_bearer_filter()),
            2,
            Effect::seq(vec![
                Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(1),
                },
                Effect::Discard {
                    who: PlayerRef::You,
                    n: Value::c(1),
                    random: false,
                    filter: Filter::Any,
                },
            ]),
            "Whenever your Ring-bearer attacks, draw a card, then discard a card.",
        ),
        emblem_trigger(
            TriggerCond::BlockedByCreature {
                attacker: ring_bearer_filter(),
                blocker: Filter::creature(),
            },
            3,
            Effect::seq(vec![
                Effect::Store {
                    var: BLOCKER,
                    sel: Sel::TriggerObject,
                },
                Effect::AtNext {
                    step: TriggerStep::EndOfCombat,
                    effect: Box::new(Effect::SacrificeObjects {
                        what: Sel::Var(BLOCKER),
                    }),
                },
            ]),
            "Whenever your Ring-bearer becomes blocked by a creature, the blocking creature's controller sacrifices it at end of combat.",
        ),
        emblem_trigger(
            TriggerCond::DealsDamage {
                source: ring_bearer_filter(),
                to: DamageRecipient::Player(PlayerRel::Any),
                combat_only: true,
            },
            4,
            Effect::LoseLife {
                who: PlayerRef::EachOpponent,
                n: Value::c(3),
            },
            "Whenever your Ring-bearer deals combat damage to a player, each opponent loses 3 life.",
        ),
    ]
}

/// The Ring tempts `p` (CR 701.54a, 701.54c, 701.54d). Returns the creature chosen as
/// their Ring-bearer.
pub fn tempt(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> Option<ObjectId> {
    if the_ring(g, p).is_none() {
        let id = crate::tokens::create_emblem(g, p, ring_abilities(), source);
        let o = &mut g.objects[id.0 as usize];
        o.base.name = SmolStr::new("The Ring");
        o.chars.name = SmolStr::new("The Ring");
        g.dirty = true;
    }
    g.players[p.idx()].ring_level += 1;
    g.recompute();
    let cands: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p && o.is_creature())
        .map(|o| o.id)
        .collect();
    let chosen = match cands.as_slice() {
        [] => None,
        [one] => Some(*one),
        _ => g
            .ask_objects(
                p,
                source,
                "The Ring tempts you: choose your Ring-bearer",
                cands.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .or(Some(cands[0])),
    };
    if let Some(c) = chosen {
        g.players[p.idx()].ring_bearer = Some(c);
        g.dirty = true;
        g.log(|g| format!("{p} chooses {} as their Ring-bearer", g.describe(c)));
        emit(g, RING_BEARER_CHOSEN, p, Some(c), 0);
    }
    let level = g.player(p).ring_level as i32;
    emit(g, RING_TEMPTS, p, chosen, level);
    chosen
}

pub struct TheRingTemptsYou;

impl KeywordActionRules for TheRingTemptsYou {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::TheRingTemptsYou]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        for p in g.eval_players(a.who, ctx) {
            tempt(g, p, ctx.source);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&TheRingTemptsYou) }

/// Ring-bearers in rules: the filter, the number of times tempted, the evasion ability,
/// and the end of the designation once another player gains control of the creature.
struct RingRules;

impl crate::kw::KeywordRules for RingRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == RING_BEARER).then(|| is_ring_bearer(g, ctx.controller, id))
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == TIMES_TEMPTED).then(|| g.player(ctx.controller).ring_level as i64)
    }

    /// "[Your Ring-bearer] can't be blocked by creatures with greater power."
    fn block_allowed(&self, g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
        let p = g.obj(attacker).controller;
        !(is_ring_bearer(g, p, attacker)
            && the_ring(g, p).is_some()
            && g.obj(blocker).power() > g.obj(attacker).power())
    }

    /// The designation ends when another player gains control of the creature (or it
    /// leaves the battlefield) (CR 701.54a).
    fn state_based_actions(&self, g: &mut Game) -> bool {
        for i in 0..g.players.len() {
            let p = g.players[i].id;
            if let Some(rb) = g.players[i].ring_bearer {
                if !is_ring_bearer(g, p, rb) {
                    g.players[i].ring_bearer = None;
                }
            }
        }
        false
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&RingRules) }
