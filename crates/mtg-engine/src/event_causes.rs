//! Event attribution: who or what caused an event.
//!
//! * Counters (CR 122.6, 122.6a): the player who puts counters on a permanent or player
//!   is the controller of the spell or ability putting them, the player paying a cost
//!   that puts them, the controller of the source of damage that results in counters
//!   (wither, infect, toxic: CR 702.80a, 702.90b–c, 702.164c), the player a turn-based
//!   action has put them (a Saga's lore counter, CR 714.3c), or, for counters a permanent
//!   enters with, the player the effect names (tribute's chosen opponent, CR 702.104a) or
//!   else the permanent's controller. Only counters put by the effect of a spell or
//!   ability are put "by an effect" (CR 609.1): not counters put as a cost ("those
//!   counters are put on as a cost, not as an effect"), as the result of damage, or by a
//!   turn-based or special action ([`CounterPut`], [`CounterOrigin`]).
//! * Destroying (CR 701.8) and countering (CR 701.6): the spell or ability whose effect
//!   did it, and its controller ([`Cause`]). A permanent destroyed by a state-based action
//!   (lethal damage, CR 704.5g–h) has no cause. A replacement effect's modified event is
//!   caused by what caused the event it replaced (CR 614.6): when umbra armor destroys
//!   the Aura instead, the spell or ability that would have destroyed the permanent
//!   destroys the Aura (CR 702.89a).
//!
//! The trigger conditions about these ([`TriggerCond::CountersPutBy`],
//! [`TriggerCond::DestroyedBy`], [`TriggerCond::CounteredBy`]) are matched here; conditions
//! about this turn ("if a noncreature permanent under your control was destroyed this turn
//! by a spell or ability an opponent controlled") check them against the turn's events
//! (`Condition::AllTriggerConditionsThisTurn`).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{CounterOrigin, Event};
use crate::game::Game;
use crate::object::{EventInfo, ObjKind};
use crate::types::*;
use serde::{Deserialize, Serialize};

/// What caused an event: the spell or ability whose effect did it, and that spell's or
/// ability's controller. Both `None`: a game rule (a state-based action) did it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cause {
    pub obj: Option<ObjectId>,
    pub by: Option<PlayerId>,
}

impl Cause {
    /// A game rule, not a spell or ability (CR 704).
    pub const RULE: Cause = Cause {
        obj: None,
        by: None,
    };

    /// The spell or ability performing the effects of `ctx`: the resolving spell or
    /// ability (or the ability's source when there's no stack object) and its controller,
    /// even while another player performs part of it ([`Effect::AsPlayer`]); or the cause
    /// the context inherited from a replaced event.
    pub fn of(ctx: &Ctx) -> Cause {
        ctx.cause.unwrap_or(Cause {
            obj: ctx.stack_obj.or(ctx.source),
            by: Some(ctx.resolving_controller.unwrap_or(ctx.controller)),
        })
    }

    /// The spell or ability `source` (or the source of an ability), controlled by its
    /// controller.
    pub fn of_source(g: &Game, source: Option<ObjectId>) -> Cause {
        Cause {
            obj: source,
            by: source.map(|s| g.obj(s).controller),
        }
    }
}

/// Who puts counters, and how (CR 122.6, 122.6a).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CounterPut {
    /// The spell or ability putting them (or its source), if any.
    pub source: Option<ObjectId>,
    /// The player who puts them.
    pub by: Option<PlayerId>,
    pub origin: CounterOrigin,
}

impl CounterPut {
    /// Counters put by the player performing the effects of `ctx` (the controller of the
    /// resolving spell or ability, or the player it has perform them): by an effect, or
    /// as a cost while `ctx` is paying one.
    pub fn of(ctx: &Ctx) -> CounterPut {
        CounterPut {
            source: ctx.source,
            by: Some(ctx.controller),
            origin: if ctx.paying_cost {
                CounterOrigin::Cost
            } else {
                CounterOrigin::Effect
            },
        }
    }

    /// Counters that the player `p` puts while performing the effects of `ctx` (a keyword
    /// action that player performs: "proliferate", "amass", "blight N", ...).
    pub fn by_player(p: PlayerId, ctx: &Ctx) -> CounterPut {
        CounterPut {
            by: Some(p),
            ..CounterPut::of(ctx)
        }
    }

    /// Counters put by the effect of the spell or ability `source` (or an ability of
    /// `source`): its controller puts them (CR 122.6a).
    pub fn by_source(g: &Game, source: Option<ObjectId>) -> CounterPut {
        CounterPut {
            source,
            by: source.map(|s| g.obj(s).controller),
            origin: CounterOrigin::Effect,
        }
    }

    /// Counters `p` puts to pay a cost of `source` (CR 118, 602.2b).
    pub fn cost(p: PlayerId, source: Option<ObjectId>) -> CounterPut {
        CounterPut {
            source,
            by: Some(p),
            origin: CounterOrigin::Cost,
        }
    }

    /// Counters that are the result of damage dealt by `source` (wither, infect, toxic):
    /// its controller puts them (CR 702.80a, 702.90b–c, 702.164c).
    pub fn damage(g: &Game, source: ObjectId) -> CounterPut {
        CounterPut {
            source: Some(source),
            by: Some(g.obj(source).controller),
            origin: CounterOrigin::Damage,
        }
    }

    /// Counters `p` puts as a turn-based or special action (CR 714.3c, 116.2f).
    pub fn rule(p: PlayerId) -> CounterPut {
        CounterPut {
            source: None,
            by: Some(p),
            origin: CounterOrigin::Rule,
        }
    }
}

/// What a permanent was as counters were put on it: its characteristics, controller and
/// whether it was face down. Conditions about this turn's events look at that, not at what
/// it is later ("a +1/+1 counter was placed on a permanent that you controlled as the
/// counter was placed. It doesn't matter whether you still control the permanent").
#[derive(Clone, Debug)]
pub struct AsPut {
    pub chars: crate::object::Characteristics,
    pub controller: PlayerId,
    pub face_down: bool,
}

impl AsPut {
    /// `id` as it is now, if it's a permanent.
    pub fn of(g: &Game, id: ObjectId) -> Option<std::sync::Arc<AsPut>> {
        let o = g.obj(id);
        (o.zone == crate::object::Zone::Battlefield).then(|| {
            std::sync::Arc::new(AsPut {
                chars: o.chars.clone(),
                controller: o.controller,
                face_down: o.face_down,
            })
        })
    }
}

/// Sees one permanent as it was ([`AsPut`]) and everything else as it is.
struct AsPutView<'a> {
    id: ObjectId,
    chars: &'a crate::object::Characteristics,
    controller: PlayerId,
    face_down: bool,
}

impl crate::eval::View for AsPutView<'_> {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a crate::object::Characteristics {
        if id == self.id {
            self.chars
        } else {
            &g.obj(id).chars
        }
    }
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId {
        if id == self.id {
            self.controller
        } else {
            g.obj(id).controller
        }
    }
    fn face_down(&self, g: &Game, id: ObjectId) -> bool {
        if id == self.id {
            self.face_down
        } else {
            g.obj(id).face_down
        }
    }
}

/// Whether `cond`, a condition about counters being put on a permanent, matches the
/// earlier event `ev` of this turn as it happened: the permanent is matched as it was as
/// they were put on it (as it entered, for counters it entered with), whatever it is now
/// ("Sigardian Paladin's first ability applies as long as you put a +1/+1 counter on a
/// permanent this turn and that permanent was a creature at the time you put the counter
/// on"). `None` if `cond` or `ev` isn't about that, or nothing was recorded.
pub fn happened_this_turn(g: &Game, cond: &TriggerCond, ctx: &Ctx, ev: &Event) -> Option<bool> {
    let Event::CountersAdded {
        target: Entity::Object(o),
        kind: k,
        n,
        by,
        as_put,
        ..
    } = ev
    else {
        return None;
    };
    let (filter, kind, who) = match cond {
        TriggerCond::CountersPut { filter, kind, .. } => (filter, kind, None),
        TriggerCond::CountersPutBy {
            who,
            on_objects: Some(filter),
            kind,
            ..
        } => (filter, kind, Some(*who)),
        _ => return None,
    };
    let view = match as_put {
        Some(a) => AsPutView {
            id: *o,
            chars: &a.chars,
            controller: a.controller,
            face_down: a.face_down,
        },
        None => {
            let e = g
                .history
                .permanents_entered
                .iter()
                .rev()
                .find(|e| e.id == *o)?;
            let (chars, face_down) = e.as_entered.as_ref()?;
            AsPutView {
                id: *o,
                chars,
                controller: e.controller,
                face_down: *face_down,
            }
        }
    };
    Some(
        *n > 0
            && kind.as_ref().is_none_or(|x| x == k)
            && who.is_none_or(|w| by.is_some_and(|p| g.player_rel_matches(w, p, ctx)))
            && g.matches_view(&view, *o, filter, ctx),
    )
}

/// Matches the trigger conditions about who put counters or what destroyed or countered
/// something. `None` if `cond` isn't one of them.
pub fn trigger_matches(
    g: &Game,
    cond: &TriggerCond,
    ctx: &Ctx,
    ev: &Event,
) -> Option<Vec<EventInfo>> {
    Some(match (cond, ev) {
        (
            TriggerCond::CountersPutBy {
                who,
                on_objects,
                on_players,
                kind,
                each,
            },
            Event::CountersAdded {
                target,
                kind: k,
                n,
                by: Some(by),
                ..
            },
        ) => {
            let on = match target {
                Entity::Object(o) => on_objects.as_ref().is_some_and(|f| g.matches(*o, f, ctx)),
                Entity::Player(p) => on_players
                    .as_ref()
                    .is_some_and(|f| g.player_filter_matches(f, *p, ctx)),
            };
            if *n == 0
                || !on
                || kind.as_ref().is_some_and(|x| x != k)
                || !g.player_rel_matches(*who, *by, ctx)
            {
                return Some(vec![]);
            }
            // "Whenever you put a [kind] counter on …" triggers for each counter.
            let (times, amount) = if *each { (*n, 1) } else { (1, *n as i32) };
            (0..times)
                .map(|_| EventInfo {
                    object: target.object(),
                    player: Some(*by),
                    amount,
                    ..Default::default()
                })
                .collect()
        }
        (
            TriggerCond::DestroyedBy { filter, by: rel },
            Event::Destroyed {
                obj, by: Some(by), ..
            },
        ) => {
            if g.player_rel_matches(*rel, *by, ctx) && g.matches(*obj, filter, ctx) {
                vec![EventInfo {
                    object: Some(g.current(*obj)),
                    lki: Some(*obj),
                    player: Some(*by),
                    ..Default::default()
                }]
            } else {
                vec![]
            }
        }
        (
            TriggerCond::CounteredBy { filter, by: rel },
            Event::Countered {
                what, by: Some(by), ..
            },
        ) => {
            if g.obj(*what).kind != ObjKind::StackAbility
                && g.player_rel_matches(*rel, *by, ctx)
                && g.matches(*what, filter, ctx)
            {
                vec![EventInfo {
                    object: Some(g.current(*what)),
                    lki: Some(*what),
                    spell: Some(*what),
                    player: Some(*by),
                    ..Default::default()
                }]
            } else {
                vec![]
            }
        }
        (
            TriggerCond::CountersPutBy { .. }
            | TriggerCond::DestroyedBy { .. }
            | TriggerCond::CounteredBy { .. },
            _,
        ) => vec![],
        _ => return None,
    })
}

/// Whether the trigger condition looks back in time for the event (CR 603.10a, 603.10e):
/// a permanent destroyed, or a spell countered, still has its abilities ("If a spell or
/// ability an opponent controls destroys Karmic Justice, Karmic Justice's ability will
/// trigger").
pub fn looks_back(cond: &TriggerCond, ev: &Event) -> bool {
    matches!(
        (cond, ev),
        (TriggerCond::DestroyedBy { .. }, Event::Destroyed { .. })
            | (TriggerCond::CounteredBy { .. }, Event::Countered { .. })
    )
}
