//! "Creatures entering don't cause abilities to trigger." (Torpor Orb, Hushwing Gryff,
//! Tocatli Honor Guard), "Artifacts and creatures entering don't cause abilities to
//! trigger." (Doorkeeper Thrull), "Creatures entering or dying don't cause abilities to
//! trigger." (Hushbringer) and "Permanents entering don't cause abilities of permanents
//! your opponents control to trigger." (Elesh Norn, Mother of Machines): while such a
//! static ability functions, a permanent of those types entering the battlefield (or
//! dying) triggers no triggered abilities (of the permanents the static names) — its own
//! enters or dies abilities or any other ability that would trigger on that event.
//! Replacement effects and "as [this] enters" abilities, which aren't triggered
//! abilities, still apply. The text is parsed in `oracle/patterns/r603_torpor.rs`.
//!
//! The permanent is looked at as it exists on the battlefield: after entering, or
//! immediately before dying. A permanent entering at the same time as the static's
//! source is affected (its statics apply once it's on the battlefield); so is one dying
//! at the same time (the statics that applied immediately before the event, CR 603.10a).
//! An ability that doesn't look back ("whenever a creature card is put into a graveyard
//! from anywhere") sees the statics after the event: it still triggers when the static's
//! source left at the same time, and the dying card's own such ability triggers.

use crate::events::Event;
use crate::game::Game;
use crate::object::Zone;
use crate::types::{CardType, ObjectId, PlayerId};
use smol_str::SmolStr;

/// `StaticEffect::Custom` name prefix, followed by `events;types;scope`: the events
/// (`enter` and/or `die`, comma-separated), the card types (comma-separated; empty for
/// any permanent) and whose abilities don't trigger (`all`, or `opponents`: abilities of
/// permanents the static's controller's opponents control).
pub const DOESNT_TRIGGER: &str = "doesn't cause abilities to trigger:";

/// Which events of such a permanent don't cause abilities to trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TorporEvent {
    Enter,
    Die,
}

/// The static ability's name.
pub fn static_name(events: &[TorporEvent], types: &[CardType], opponents_only: bool) -> SmolStr {
    let ev: Vec<&str> = events
        .iter()
        .map(|e| match e {
            TorporEvent::Enter => "enter",
            TorporEvent::Die => "die",
        })
        .collect();
    let words: Vec<String> = types.iter().map(|t| t.word().to_string()).collect();
    let scope = if opponents_only { "opponents" } else { "all" };
    SmolStr::new(format!(
        "{DOESNT_TRIGGER}{};{};{scope}",
        ev.join(","),
        words.join(",")
    ))
}

struct Torpor<'a> {
    events: &'a str,
    types: &'a str,
    opponents_only: bool,
}

fn parse(name: &str) -> Option<Torpor<'_>> {
    let r = name.strip_prefix(DOESNT_TRIGGER)?;
    let mut it = r.split(';');
    let events = it.next()?;
    let types = it.next()?;
    let scope = it.next()?;
    Some(Torpor {
        events,
        types,
        opponents_only: scope == "opponents",
    })
}

/// The event's kind and the permanent as it exists on the battlefield (entering: the
/// new object; dying: the last known information of the old one).
fn event_object(ev: &Event) -> Option<(&'static str, ObjectId)> {
    match ev {
        Event::ZoneChange {
            new,
            to: Zone::Battlefield,
            ..
        } => Some(("enter", *new)),
        Event::ZoneChange {
            old,
            from: Zone::Battlefield,
            to: Zone::Graveyard(_),
            ..
        } => Some(("die", *old)),
        _ => None,
    }
}

fn applies(
    g: &Game,
    t: &Torpor,
    kind: &str,
    obj: ObjectId,
    static_ctl: PlayerId,
    src: Option<(Option<ObjectId>, PlayerId)>,
) -> bool {
    if !t.events.split(',').any(|e| e == kind) {
        return false;
    }
    let o = g.obj(obj);
    // "Creatures dying": a creature put into a graveyard from the battlefield
    // (CR 700.4); "permanents entering": any permanent.
    let typed = t.types.is_empty()
        || t
            .types
            .split(',')
            .filter_map(CardType::from_word)
            .any(|ty| o.is(ty));
    if !typed {
        return false;
    }
    // "Abilities of permanents your opponents control": not delayed triggered abilities
    // or abilities of cards in other zones.
    if t.opponents_only {
        src.is_some_and(|(s, c)| {
            g.are_opponents(static_ctl, c)
                && s.is_some_and(|s| g.obj(s).zone == Zone::Battlefield)
        })
    } else {
        true
    }
}

/// The suppressing statics that apply to `ev`: those functioning now, or, for an ability
/// that looks back at a permanent leaving the battlefield, those that functioned
/// immediately before it.
fn statics<'a>(
    g: &'a Game,
    ev: &'a Event,
    look_back: bool,
) -> Vec<&'a (ObjectId, PlayerId, SmolStr)> {
    match ev {
        Event::ZoneChange {
            from: Zone::Battlefield,
            lookback: Some(lb),
            ..
        } if look_back => lb.trigger_suppressions.iter().collect(),
        _ => g.statics.customs.iter().collect(),
    }
}

/// Whether `ev` is a permanent entering that causes no ability at all to trigger because
/// of such a static ability.
pub fn entering_triggers_nothing(g: &Game, ev: &Event) -> bool {
    let Some((kind @ "enter", obj)) = event_object(ev) else {
        return false;
    };
    g.statics.customs.iter().any(|(_, c, name)| {
        parse(name).is_some_and(|t| applies(g, &t, kind, obj, *c, None))
    })
}

/// Whether `ev` doesn't cause the triggered ability of `src` (controlled by `src_ctl`;
/// no source: a delayed triggered ability without one)
/// to trigger, the ability looking back in time (`look_back`, CR 603.10a) or not.
pub fn suppressed_for(
    g: &Game,
    ev: &Event,
    src: Option<ObjectId>,
    src_ctl: PlayerId,
    look_back: bool,
) -> bool {
    let Some((kind, obj)) = event_object(ev) else {
        return false;
    };
    // The dying card's own "put into a graveyard from anywhere" ability (it doesn't look
    // back) still triggers.
    if kind == "die" && !look_back {
        if let Event::ZoneChange { new, .. } = ev {
            if Some(*new) == src {
                return false;
            }
        }
    }
    statics(g, ev, look_back).into_iter().any(|(_, c, name)| {
        parse(name).is_some_and(|t| applies(g, &t, kind, obj, *c, Some((src, src_ctl))))
    })
}

/// The suppressing statics functioning now, for the look-back snapshot of an event.
pub fn snapshot(g: &Game) -> Vec<(ObjectId, PlayerId, SmolStr)> {
    g.statics
        .customs
        .iter()
        .filter(|(_, _, n)| n.starts_with(DOESNT_TRIGGER))
        .cloned()
        .collect()
}
