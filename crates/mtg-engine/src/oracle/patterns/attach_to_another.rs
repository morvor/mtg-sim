//! Moving Auras to another permanent (CR 701.3): "Attach target Aura attached to a
//! permanent to another permanent with the same controller." (Simic Guildmage), "attach
//! all Auras enchanting target permanent to another permanent with the same controller"
//! (Glamer Spinners), "Attach target Aura attached to a creature to another creature."
//!
//! Only the Aura (or the permanent losing its Auras) is targeted. The permanent that gets
//! them is chosen as the ability resolves, among permanents other than the one they're
//! attached to that each of them could legally enchant (CR 303.4d, 701.3a) — "with the
//! same controller": controlled by the player who controls the permanent they're attached
//! to as the ability resolves. If there's no such permanent, the Auras don't move.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "another permanent [with the same controller]": the object filter of the permanent the
/// Auras move to, relative to `host` (where they are now).
fn destination(r: &str, host: &Sel, auras: &Sel) -> Option<Filter> {
    let r = r.strip_prefix("another ")?;
    // "another permanent of that type" (Enchantment Alteration): the type of the
    // permanent the Aura is attached to, which no filter here describes ("that type"
    // would read as a type chosen earlier). Not understood.
    if r.contains(" of that ") {
        return None;
    }
    let (r, same_controller) = match r.strip_suffix(" with the same controller") {
        Some(r) => (r, true),
        None => (r, false),
    };
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    let mut parts = vec![
        f,
        Filter::InZone(ZoneKind::Battlefield),
        Filter::not(Filter::In(Box::new(host.clone()))),
        Filter::CanBeAttachedBy(Box::new(auras.clone())),
    ];
    if same_controller {
        parts.push(Filter::ControlledByPlayer(Box::new(PlayerRef::ControllerOf(
            Box::new(host.clone()),
        ))));
    }
    Some(Filter::and(parts))
}

fn attach_to_another(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("attach ")?;
    let aura = Filter::Subtype("Aura".into());
    let (what, host, dest) = if let Some(r) = r.strip_prefix("target aura attached to a ") {
        // "target Aura attached to a permanent to another permanent ...".
        let (host_kind, dest) = r.split_once(" to ")?;
        let (hf, plural, rest) = parse_object_phrase(host_kind)?;
        if plural || !end(rest).is_empty() {
            return None;
        }
        let hosts = Sel::All(Filter::and(vec![hf, Filter::InZone(ZoneKind::Battlefield)]));
        let spec = TargetSpec::object(
            Filter::and(vec![aura, Filter::AttachedTo(Box::new(hosts))]),
            format!("target Aura attached to a {host_kind}"),
        );
        let slot = b.add_target(spec, &format!("target aura attached to a {host_kind}"));
        let what = Sel::Target(slot);
        let host = Sel::HostOf(Box::new(what.clone()));
        (what, host, dest)
    } else if let Some(r) = r.strip_prefix("all auras enchanting target ") {
        // "all Auras enchanting target permanent to another permanent ...".
        let (host_kind, dest) = r.split_once(" to ")?;
        let (hf, plural, rest) = parse_object_phrase(host_kind)?;
        if plural || !end(rest).is_empty() {
            return None;
        }
        let spec = TargetSpec::object(hf, format!("target {host_kind}"));
        let slot = b.add_target(spec, &format!("target {host_kind}"));
        let host = Sel::Target(slot);
        let what = Sel::All(Filter::and(vec![
            aura,
            Filter::AttachedTo(Box::new(host.clone())),
        ]));
        (what, host, dest)
    } else {
        return None;
    };
    let filter = destination(dest, &host, &what)?;
    Some(Effect::Attach {
        what,
        to: Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "attach Auras to another permanent [with the same controller]", priority: 90, parse: attach_to_another } }
