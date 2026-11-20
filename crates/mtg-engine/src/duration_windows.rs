//! "For as long as" durations of the effects of an ability waiting on the stack (CR 611.2b):
//! whether the duration ended before the effect would first be applied, so the effect does
//! nothing.
//!
//! Such a duration is a period, not a condition looked at once (Seasinger ruling: "Whether
//! you control Seasinger is checked continually, starting when the ability is activated.").
//! The period starts as the ability becomes activated (CR 602.2b, 601.2i: after its costs
//! are paid) or triggers, or, for a copy of an ability, as the copy is put onto the stack
//! (CR 707.10); it ends the first moment its condition is false. Once ended, it doesn't
//! begin again before the ability resolves, even if the condition is true again by then: it
//! can begin again only during the resolution itself ("doesn't begin again during that
//! spell or ability's resolution", CR 611.2b). Register entries
//! `for-as-long-as-checked-continually-from-activation` (the main case),
//! `copied-ability-duration-starts-as-the-copy-is-created`,
//! `phasing-out-ends-a-waiting-for-as-long-as-duration`,
//! `duration-about-another-object-ends-while-the-ability-waits` and
//! `duration-the-resolution-sets-up-is-checked-as-the-effect-begins`.
//!
//! - Conditions about the ability's source — "for as long as you control ~", "for as long
//!   as ~ remains tapped", "for as long as ~ remains on the battlefield" — are read from this
//!   turn's events since the period started: the source changing control, becoming tapped
//!   or untapped, or phasing out or in (CR 702.26f: such durations "can no longer see it")
//!   ends it ([`duration_ended`]). Leaving the battlefield makes it a new object (CR 400.7),
//!   which the state at resolution already shows. "Until ~ leaves the battlefield" isn't
//!   such a duration: phasing out isn't leaving the battlefield (CR 702.26d), so only the
//!   source leaving ends it.
//! - Other conditions of a duration about objects or players — "that creature's power
//!   remains less than or equal to ~'s power", "for as long as that Aura is attached to
//!   it", "for as long as they're the monarch" — are checked each time characteristics are
//!   computed while the ability waits ([`watch`]): one that stops holding, or that holds
//!   only after not holding at first, has ended. Conditions an ability's own instructions
//!   set up — a counter it puts on the object, a tap — are checked as the effect begins, as
//!   is any condition that hasn't held at all while the ability waited (the resolution can
//!   set it up: "you may have an opponent gain control of it. If you do, it's goaded for as
//!   long as they control it").

use crate::ability::{vars, AbilityKind, Condition, Duration, Effect, Filter, TriggerCond};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::{StackKind, Zone};
use crate::types::{Entity, ObjectId, PlayerId};
use serde::{Deserialize, Serialize};

/// A moment of the game: how many of this turn's events (`Game::turn_events`, followed by
/// the events not processed yet) had happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moment {
    pub turn: u32,
    pub pos: u32,
}

/// The period of the "for as long as" durations of a spell or ability on the stack
/// (`StackInfo::window`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    /// When the period started, if not when the ability became activated or triggered (a
    /// copy put onto the stack, CR 707.10); see [`start`].
    pub from: Option<Moment>,
    /// When the spell or ability began to resolve.
    pub resolving: Option<Moment>,
    /// The check of conditions about other objects ([`watch`]).
    pub others: Watch,
}

impl Window {
    /// The window of a copy of a spell or ability put onto the stack at `at`.
    pub fn copy_at(at: Moment) -> Window {
        Window {
            from: Some(at),
            ..Window::default()
        }
    }
}

/// Where [`watch`] stands for an ability's conditions about other objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Watch {
    /// Not checked yet.
    #[default]
    Unseen,
    /// None to check.
    Off,
    /// It didn't hold when first checked, nor since: it can still begin during the
    /// resolution, which the ability's own instructions can set it up in (checked as the
    /// effect begins).
    NotYet,
    /// It has held ever since it was first checked.
    Holding,
    /// It was false at some point after holding, or it held after not holding at first
    /// (before the resolution, not during it): the duration ended or never started.
    Ended,
}

/// Now, as a [`Moment`].
pub fn now(g: &Game) -> Moment {
    let pending = g
        .events
        .iter()
        .filter(|e| !matches!(e, Event::BatchBoundary))
        .count();
    Moment {
        turn: g.turn.number,
        pos: (g.turn_events.len() + pending) as u32,
    }
}

/// This turn's events (those not processed yet last) with their positions, from `from` on.
fn events_from(g: &Game, from: usize) -> impl Iterator<Item = (usize, &Event)> {
    g.turn_events
        .iter()
        .chain(
            g.events
                .iter()
                .filter(|e| !matches!(e, Event::BatchBoundary)),
        )
        .enumerate()
        .skip(from)
}

/// The ability `id` begins to resolve (see [`Window::resolving`]).
pub(crate) fn resolution_begins(g: &mut Game, id: ObjectId) {
    let at = now(g);
    if let Some(si) = g.objects[id.0 as usize].stack.as_mut() {
        si.window.resolving = Some(at);
    }
}

/// Where the period of the durations of the ability `ab` on the stack started, as a
/// position in this turn's events: as a copy was put onto the stack, as a triggered ability
/// triggered (after its trigger event and the events simultaneous with it,
/// `EventInfo::turn_events_seen`), or as the ability became activated or (a triggered
/// ability whose trigger time isn't recorded) was put onto the stack. A reflexive triggered
/// ability (CR 603.12) triggers during the resolution of the ability that created it, whose
/// trigger event it carries: its period starts as it's put onto the stack, right after that
/// resolution.
fn start(g: &Game, ab: ObjectId) -> Option<usize> {
    let si = g.obj(ab).stack.as_deref()?;
    if let Some(m) = si.window.from {
        return (m.turn == g.turn.number).then_some(m.pos as usize);
    }
    let reflexive = matches!(&si.kind, StackKind::Triggered { ability, .. }
        if matches!(&ability.kind, AbilityKind::Triggered(t)
            if matches!(&t.trigger, TriggerCond::Custom(n) if n.as_str() == "reflexive")));
    let triggered = si.event.as_ref().and_then(|e| e.turn_events_seen);
    if let Some(pos) = triggered.filter(|p| !reflexive && *p <= now(g).pos as usize) {
        return Some(pos);
    }
    events_from(g, 0).find_map(|(i, ev)| match ev {
        Event::AbilityActivated {
            ability: Some(a), ..
        }
        | Event::AbilityTriggeredOnStack { ability: a, .. }
            if *a == ab =>
        {
            Some(i + 1)
        }
        _ => None,
    })
}

/// What a duration requires of the source of the ability creating the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Needs {
    /// The ability's controller controls it.
    control: bool,
    /// It's tapped.
    tapped: bool,
    /// It's on the battlefield and phased in (a "for as long as" duration can't see a
    /// phased-out permanent, CR 702.26f).
    present: bool,
    /// It's on the battlefield, phased in or out ("until ~ leaves the battlefield":
    /// phasing out isn't leaving, CR 702.26d).
    remains: bool,
}

impl Needs {
    fn union(self, o: Needs) -> Needs {
        Needs {
            control: self.control || o.control,
            tapped: self.tapped || o.tapped,
            present: self.present || o.present,
            remains: self.remains || o.remains,
        }
    }
}

/// The source's part of a condition: "for as long as ~ remains tapped" (and "you control
/// ~ and ~ remains tapped"), alone or among the conditions of an "and" (Old Man of the
/// Sea: "for as long as ~ remains tapped and that creature's power remains ...").
fn cond_needs(c: &Condition) -> Needs {
    use crate::untap_choice::{CONTROL_AND_SOURCE_REMAINS_TAPPED, SOURCE_REMAINS_TAPPED};
    match c {
        Condition::Custom(n) if n.as_str() == SOURCE_REMAINS_TAPPED => Needs {
            tapped: true,
            present: true,
            ..Needs::default()
        },
        Condition::Custom(n) if n.as_str() == CONTROL_AND_SOURCE_REMAINS_TAPPED => Needs {
            control: true,
            tapped: true,
            present: true,
            ..Needs::default()
        },
        Condition::And(v) => v
            .iter()
            .fold(Needs::default(), |n, c| n.union(cond_needs(c))),
        _ => Needs::default(),
    }
}

fn needs(d: &Duration) -> Needs {
    match d {
        Duration::WhileYouControlSource => Needs {
            control: true,
            present: true,
            ..Needs::default()
        },
        Duration::WhileSourceOnBattlefield => Needs {
            present: true,
            ..Needs::default()
        },
        Duration::UntilSourceLeavesBattlefield => Needs {
            remains: true,
            ..Needs::default()
        },
        Duration::WhileCondition(c) => cond_needs(c),
        _ => Needs::default(),
    }
}

/// The resolving ability (`ctx.stack_obj`) and its source (`ctx.source`), if `ctx` is the
/// resolution of an activated or triggered ability of that source.
fn ability_of(g: &Game, ctx: &Ctx) -> Option<(ObjectId, ObjectId)> {
    let (src, ab) = (ctx.source?, ctx.stack_obj?);
    match &g.obj(ab).stack.as_deref()?.kind {
        StackKind::Activated { source, .. } | StackKind::Triggered { source, .. }
            if *source == src =>
        {
            Some((src, ab))
        }
        _ => None,
    }
}

/// Whether the source `src` now is as `n` requires for `controller`.
fn holds_now(g: &Game, src: ObjectId, n: Needs, controller: PlayerId) -> bool {
    let o = g.obj(src);
    g.is_live(src)
        && o.zone == Zone::Battlefield
        && (!n.present || !o.phased_out)
        && (!n.control || o.controller == controller)
        && (!n.tapped || o.tapped)
}

/// Whether, since the period of the durations of the ability `ab` of `src` started and
/// before it began to resolve, `src` stopped being as `n` requires for `controller` — it
/// changed control, became untapped or tapped (it wasn't tapped just before), or phased out
/// or in — and it wasn't made so again during the resolution (CR 611.2b: the duration
/// "doesn't begin again during that spell or ability's resolution"). The state at the
/// moment the effect would begin is [`holds_now`]'s.
fn changed_while_waiting(
    g: &Game,
    src: ObjectId,
    ab: ObjectId,
    n: Needs,
    controller: PlayerId,
) -> bool {
    let Some(from) = start(g, ab) else {
        return false;
    };
    let resolving = g
        .obj(ab)
        .stack
        .as_deref()
        .and_then(|si| si.window.resolving)
        .filter(|m| m.turn == g.turn.number)
        .map_or(usize::MAX, |m| m.pos as usize);
    let (mut ended, mut again) = (Needs::default(), Needs::default());
    for (i, ev) in events_from(g, from) {
        let during = i >= resolving;
        match ev {
            Event::ControlChanged { obj, to, .. } if *obj == src && n.control => {
                if !during {
                    ended.control = true;
                } else if *to == controller {
                    again.control = true;
                }
            }
            Event::Untapped { obj } if *obj == src && n.tapped && !during => ended.tapped = true,
            Event::Tapped { obj, .. } if *obj == src && n.tapped => {
                if during {
                    again.tapped = true;
                } else {
                    ended.tapped = true;
                }
            }
            Event::PhasedOut { obj } if *obj == src && n.present && !during => ended.present = true,
            Event::PhasedIn { obj } if *obj == src && n.present => {
                if during {
                    again.present = true;
                } else {
                    ended.present = true;
                }
            }
            _ => {}
        }
    }
    (ended.control && !again.control)
        || (ended.tapped && !again.tapped)
        || (ended.present && !again.present)
}

/// For the resolution of an ability of `ctx.source` (`ctx`), whether the part of duration
/// `d` about that source has ended, so the effect the ability is about to create with it
/// does nothing (CR 611.2b): the source isn't as the duration requires now, or it stopped
/// being so while the ability waited, even if it's so again (see the module docs). Other
/// spells and abilities, and durations not about the source, aren't affected.
pub fn duration_ended(g: &Game, d: &Duration, ctx: &Ctx) -> bool {
    let n = needs(d);
    if n == Needs::default() {
        return false;
    }
    let Some((src, ab)) = ability_of(g, ctx) else {
        return false;
    };
    !holds_now(g, src, n, ctx.controller) || changed_while_waiting(g, src, ab, n, ctx.controller)
}

/// The "for as long as ~ remains tapped" conditions (`untap_choice`) during the resolution
/// of an ability of the source: whether the source stopped being tapped (or under the
/// ability's controller's control, with `control`) while the ability waited.
pub fn tapped_source_changed(g: &Game, ctx: &Ctx, control: bool) -> bool {
    let Some((src, ab)) = ability_of(g, ctx) else {
        return false;
    };
    let n = Needs {
        control,
        tapped: true,
        present: true,
        ..Needs::default()
    };
    changed_while_waiting(g, src, ab, n, ctx.controller)
}

/// The duration of the continuous effect the instruction `e` creates, if it creates one.
fn duration_of(e: &Effect) -> Option<&Duration> {
    match e {
        Effect::Modify { duration, .. }
        | Effect::AddRestriction { duration, .. }
        | Effect::AddPlayerEffect { duration, .. }
        | Effect::AddReplacement { duration, .. }
        | Effect::GainControl { duration, .. }
        | Effect::OfferSpecialAction { duration, .. }
        | Effect::SpendAnyTypeMana { duration, .. }
        | Effect::BecomeCopy { duration, .. }
        | Effect::BecomeCopyExcept { duration, .. }
        | Effect::GrantPlayPermission { duration, .. }
        | Effect::PreventDamage { duration, .. }
        | Effect::PreventDividedDamage { duration, .. }
        | Effect::ChangeText { duration, .. } => Some(duration),
        _ => None,
    }
}

/// CR 611.2b, as the resolving spell or ability (`ctx`) performs the instruction `e`: the
/// "for as long as" duration of the continuous effect it would create has ended
/// ([`ended`]), so the instruction does nothing. (A play permission, a player effect, a
/// replacement effect, a control change, a modification ...: one check for all of them.)
pub fn effect_does_nothing(g: &Game, e: &Effect, ctx: &Ctx) -> bool {
    duration_of(e).is_some_and(|d| ended(g, d, ctx))
}

/// Whether the duration `d` of an effect the resolving ability (`ctx`) is about to create
/// ended while the ability waited: its part about the ability's source
/// ([`duration_ended`]), or a condition about another object [`watch`] saw become false.
pub fn ended(g: &Game, d: &Duration, ctx: &Ctx) -> bool {
    duration_ended(g, d, ctx) || others_ended(g, d, ctx)
}

fn others_ended(g: &Game, d: &Duration, ctx: &Ctx) -> bool {
    watched_part(d).is_some()
        && ability_of(g, ctx).is_some_and(|(_, ab)| {
            g.obj(ab)
                .stack
                .as_deref()
                .is_some_and(|si| si.window.others == Watch::Ended)
        })
}

/// A filter the ability's own instructions can make an object match before its effect
/// begins (it taps it, puts a counter on it): such a condition is checked as the effect
/// begins (Braided Net, Zygon Infiltrator, Xolatoyac; register
/// `duration-the-resolution-sets-up-is-checked-as-the-effect-begins`).
fn set_up_by_the_resolution(f: &Filter) -> bool {
    match f {
        Filter::Tapped | Filter::HasCounter(_) => true,
        Filter::And(v) => v.iter().any(set_up_by_the_resolution),
        _ => false,
    }
}

/// The conditions of duration `d` about other objects or players that [`watch`] checks:
/// not those about the source (custom conditions, see [`needs`]), nor those
/// [`set_up_by_the_resolution`].
fn watched_part(d: &Duration) -> Option<Condition> {
    let Duration::WhileCondition(c) = d else {
        return None;
    };
    let all = match c {
        Condition::And(v) => v.as_slice(),
        c => std::slice::from_ref(c),
    };
    let mut parts: Vec<Condition> = all
        .iter()
        .filter(|c| match c {
            Condition::Custom(_) => false,
            Condition::SelMatches(_, f) => !set_up_by_the_resolution(f),
            _ => true,
        })
        .cloned()
        .collect();
    match parts.len() {
        0 => None,
        1 => parts.pop(),
        _ => Some(Condition::And(parts)),
    }
}

/// The instructions of `e` that create continuous effects, through sequences and choices.
fn effects_with_durations<'a>(e: &'a Effect, out: &mut Vec<&'a Effect>) {
    match e {
        Effect::Seq(v) => v.iter().for_each(|x| effects_with_durations(x, out)),
        Effect::May { effect, .. } => effects_with_durations(effect, out),
        Effect::If {
            then, otherwise, ..
        }
        | Effect::PayOptional {
            then, otherwise, ..
        } => {
            effects_with_durations(then, out);
            effects_with_durations(otherwise, out);
        }
        e if duration_of(e).is_some() => out.push(e),
        _ => {}
    }
}

/// The objects the effect `e` would apply to, as the ability waits ("that creature" of a
/// condition about the affected objects).
fn affected(g: &Game, e: &Effect, ctx: &Ctx) -> Vec<Entity> {
    match e {
        Effect::Modify { what, .. }
        | Effect::GainControl { what, .. }
        | Effect::BecomeCopy { what, .. }
        | Effect::BecomeCopyExcept { what, .. } => g
            .eval_sel(what, ctx)
            .into_iter()
            .filter(|x| x.object().is_some_and(|o| g.is_live(o)))
            .collect(),
        _ => vec![],
    }
}

/// The body of the activated or triggered ability `id` on the stack.
fn ability_body(g: &Game, id: ObjectId) -> Option<&crate::ability::Body> {
    let si = g.obj(id).stack.as_deref()?;
    match &si.kind {
        StackKind::Activated { ability, .. } | StackKind::Triggered { ability, .. } => {
            match &ability.kind {
                AbilityKind::Activated(a) => Some(&a.body),
                AbilityKind::Triggered(t) => Some(&t.body),
                _ => None,
            }
        }
        StackKind::Spell => None,
    }
}

/// Whether any instruction of the ability `id` on the stack (in any mode) creates a
/// continuous effect whose duration has conditions [`watch`] checks.
fn watches_anything(g: &Game, id: ObjectId) -> bool {
    let Some(body) = ability_body(g, id) else {
        return false;
    };
    let mut found = Vec::new();
    effects_with_durations(&body.effect, &mut found);
    if let Some(modal) = &body.modal {
        for m in &modal.modes {
            effects_with_durations(&m.effect, &mut found);
        }
    }
    found
        .iter()
        .any(|e| duration_of(e).and_then(watched_part).is_some())
}

/// The instructions of the chosen modes of the ability `id` on the stack that create
/// continuous effects whose durations have conditions [`watch`] checks, each with the
/// targets of its mode.
fn watched_effects(g: &Game, id: ObjectId) -> Vec<(&Effect, Condition, &[Vec<Entity>])> {
    let (Some(si), Some(body)) = (g.obj(id).stack.as_deref(), ability_body(g, id)) else {
        return vec![];
    };
    let mut out = Vec::new();
    for cm in &si.chosen {
        let effect = match (cm.mode, &body.modal) {
            (Some(m), Some(modal)) => match modal.modes.get(m) {
                Some(mode) => &mode.effect,
                None => continue,
            },
            _ => &body.effect,
        };
        let mut found = Vec::new();
        effects_with_durations(effect, &mut found);
        for e in found {
            if let Some(c) = duration_of(e).and_then(watched_part) {
                out.push((e, c, cm.targets.as_slice()));
            }
        }
    }
    out
}

/// Whether the ability `id` is all on the stack (its targets chosen and costs paid): it
/// became activated or was put onto the stack, or it's a copy.
fn on_the_stack(g: &Game, id: ObjectId) -> bool {
    g.obj(id)
        .stack
        .as_deref()
        .is_some_and(|si| si.window.from.is_some())
        || events_from(g, 0).any(|(_, ev)| match ev {
            Event::AbilityActivated {
                ability: Some(a), ..
            }
            | Event::AbilityTriggeredOnStack { ability: a, .. } => *a == id,
            _ => false,
        })
}

/// Whether the conditions of `effects` (see [`watched_effects`]) hold now for the ability
/// `id`.
fn watched_hold(g: &Game, id: ObjectId, effects: &[(&Effect, Condition, &[Vec<Entity>])]) -> bool {
    let base = g.stack_ctx(id);
    effects.iter().all(|(e, c, targets)| {
        let mut ctx = base.clone();
        if !targets.is_empty() {
            ctx.targets = targets.to_vec();
        }
        let objs = affected(g, e, &ctx);
        ctx.set_var(vars::AFFECTED, objs);
        g.eval_cond(c, &ctx)
    })
}

/// CR 611.2b: checks, while abilities wait on the stack, the conditions about other objects
/// of their effects' durations ([`watched_part`]): one that held and then didn't has ended,
/// even if it holds again by the time the ability resolves (Old Man of the Sea ruling: "The
/// same is true if the targeted creature's power becomes greater than Old Man of the Sea's
/// power before the ability resolves."). One that didn't hold when first checked never
/// started: if it holds while the ability waits, it didn't begin during the resolution, so
/// it has ended too (Rootwater Matriarch ruling: "Can target an unenchanted creature but has
/// no effect on that creature."); one that holds only from the resolution on is checked as
/// the effect begins (the resolution can set it up). Called as characteristics are computed
/// (`Game::recompute`); the resolving ability isn't checked (CR 611.2b lets its duration
/// begin again during the resolution).
pub(crate) fn watch(g: &mut Game) {
    for i in 0..g.stack.len() {
        let id = g.stack[i];
        if g.resolving == Some(id) {
            continue;
        }
        let Some(state) = g.obj(id).stack.as_deref().map(|si| si.window.others) else {
            continue;
        };
        if matches!(state, Watch::Off | Watch::Ended) {
            continue;
        }
        // Its modes and targets are chosen as it's put on the stack: it's first checked
        // once that's done.
        let next = if state == Watch::Unseen && !watches_anything(g, id) {
            Watch::Off
        } else if state == Watch::Unseen && !on_the_stack(g, id) {
            continue;
        } else {
            let effects = watched_effects(g, id);
            if effects.is_empty() {
                Watch::Off
            } else {
                match (state, watched_hold(g, id, &effects)) {
                    (Watch::Unseen, true) => Watch::Holding,
                    (Watch::Unseen, false) => Watch::NotYet,
                    // It stopped holding, or it began to hold while the ability waited,
                    // not during its resolution (CR 611.2b).
                    (Watch::Holding, false) | (Watch::NotYet, true) => Watch::Ended,
                    (s, _) => s,
                }
            }
        };
        if next != state {
            if let Some(si) = g.objects[id.0 as usize].stack.as_mut() {
                si.window.others = next;
            }
        }
    }
}
