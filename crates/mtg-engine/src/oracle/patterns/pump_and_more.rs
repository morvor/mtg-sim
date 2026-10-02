//! A pump joined to another instruction about the same object:
//! * "~ gets +1/+0 until end of turn and deals 1 damage to you." (Firedrinker Satyr): read
//!   as the two sentences "~ gets +1/+0 until end of turn. ~ deals 1 damage to you."
//! * "Until end of turn, target nonartifact creature gets +1/+0 and becomes an artifact in
//!   addition to its other types." (Thran Forge): one modification of one object.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_pt_mod, parse_sentence, Builder};
use crate::oracle::phrases::end;

fn pump_and_deals(s: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(s);
    let r = l.strip_prefix("~ gets ")?;
    let (pump, dmg) = r.split_once(" until end of turn and deals ")?;
    let first = parse_sentence(&format!("~ gets {pump} until end of turn"), b)?;
    let second = parse_sentence(&format!("~ deals {dmg}"), b)?;
    if !matches!(first, Effect::Modify { .. }) || !matches!(second, Effect::DealDamage { .. }) {
        return None;
    }
    Some(Effect::seq(vec![first, second]))
}

inventory::submit! { EffectPattern { name: "~ gets +N/+N until end of turn and deals N damage to ...", priority: 100, parse: pump_and_deals } }

fn pump_and_becomes(s: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(s);
    let r = l.strip_prefix("until end of turn, ")?;
    let (subject, rest) = r.split_once(" gets ")?;
    let (p, t, tail) = parse_pt_mod(rest)?;
    let becomes = tail.trim().strip_prefix("and becomes ")?;
    let mut e = parse_sentence(&format!("{subject} becomes {becomes} until end of turn"), b)?;
    let Effect::Modify { mods, duration, .. } = &mut e else {
        return None;
    };
    if !matches!(duration, Duration::EndOfTurn) {
        return None;
    }
    mods.insert(0, Modification::ModifyPT(p, t));
    Some(e)
}

inventory::submit! { EffectPattern { name: "until end of turn, [object] gets +N/+N and becomes ...", priority: 100, parse: pump_and_becomes } }
