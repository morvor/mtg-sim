//! Oracle patterns for Rooms (CR 709.5): "When you unlock this door, [effect]" on a half
//! (CR 709.5h), "whenever you fully unlock a Room" (CR 709.5i), and effects that lock or
//! unlock a door (CR 709.5f, 709.5g).

use super::{AbilityPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::card::Layout;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_target};
use crate::oracle::CompileContext;
use crate::rooms::{
    FULLY_UNLOCKED, LOCK_EFFECT, LOCK_OR_UNLOCK_EFFECT, UNLOCK_EFFECT, UNLOCK_THIS_DOOR,
};
use smol_str::SmolStr;
use std::sync::Arc;

/// "When you unlock this door, [effect]": the door is the half this text is on.
fn unlock_this_door(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let rest = text.strip_prefix("When you unlock this door, ")?;
    if ctx.layout != Layout::Split {
        return None;
    }
    let inner = format!(
        "When you unlock door {} of this Room, {rest}",
        ctx.face_index
    );
    let mut out = crate::oracle::parse_ability(&inner, ctx)?;
    for a in out.iter_mut() {
        Arc::make_mut(a).text = text.to_string();
    }
    Some(out)
}

fn room_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    if let Some(n) = r
        .strip_prefix("you unlock door ")
        .and_then(|s| s.strip_suffix(" of this room"))
    {
        let n: usize = n.parse().ok()?;
        return Some((
            TriggerCond::Custom(format!("{UNLOCK_THIS_DOOR}{n}").into()),
            Sel::This,
            PlayerRef::You,
        ));
    }
    let who = match r {
        "you fully unlock a room" => PlayerRel::You,
        "a player fully unlocks a room" => PlayerRel::Any,
        _ => return None,
    };
    Some((
        TriggerCond::PlayerAction {
            name: SmolStr::new(FULLY_UNLOCKED),
            who,
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

/// "Lock or unlock a door of target Room you control", "unlock a locked door of up to one
/// target Room you control", "lock an unlocked door of ..." (CR 709.5f, 709.5g; a door is
/// a half of a Room, CR 709.5j).
fn lock_unlock_door(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (name, rest) = if let Some(r) = l.strip_prefix("lock or unlock a door of ") {
        (LOCK_OR_UNLOCK_EFFECT, r)
    } else if let Some(r) = l.strip_prefix("unlock a locked door of ") {
        (UNLOCK_EFFECT, r)
    } else if let Some(r) = l.strip_prefix("lock an unlocked door of ") {
        (LOCK_EFFECT, r)
    } else {
        return None;
    };
    let (spec, tail) = parse_target(rest)?;
    if !end(tail).is_empty() {
        return None;
    }
    let slot = b.add_target(spec, rest);
    Some(Effect::ForEach {
        sel: Sel::Target(slot),
        var: vars::AFFECTED,
        effect: Box::new(Effect::Custom(name.into())),
    })
}

inventory::submit! { AbilityPattern { name: "when you unlock this door", priority: 0, parse: unlock_this_door } }
inventory::submit! { TriggerPattern { name: "room unlock triggers", priority: 0, parse: room_triggers } }
inventory::submit! { EffectPattern { name: "lock or unlock a door", priority: 0, parse: lock_unlock_door } }
