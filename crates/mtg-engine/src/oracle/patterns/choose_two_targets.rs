//! "Choose target creature you control and target creature you don't control. Put a +1/+1
//! counter on the creature you control if the gift was promised. Then those creatures
//! fight each other." (Longstalk Brawl, Tail Swipe, Malamet Battle Glyph, Joust): the
//! targets are chosen as the spell is cast (CR 601.2c, 115.1) and choosing them does
//! nothing by itself. Later sentences name each one ("the creature you control", "the
//! chosen creature you control", "the creature an opponent controls") and both ("those
//! creatures", "the chosen creatures"), recorded in `Builder::named`.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

fn choose_two_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (first, second) = r.split_once(" and target ")?;
    let second = format!("target {second}");
    let mut specs = Vec::new();
    for text in [first, second.as_str()] {
        let (spec, tail) = parse_target(text)?;
        if !end(tail).is_empty()
            || spec.fixed_min() != Some(1)
            || !matches!(spec.max, Value::Const(1))
            || !matches!(spec.what, TargetKind::Object(_))
        {
            return None;
        }
        specs.push((spec, text.to_string()));
    }
    let mut slots = Vec::new();
    for (spec, text) in specs {
        let slot = b.add_target(spec, &text);
        let noun = text.strip_prefix("target ").unwrap_or(&text).to_string();
        b.named
            .push((format!("the chosen {noun}"), Sel::Target(slot)));
        b.named.push((format!("the {noun}"), Sel::Target(slot)));
        slots.push((slot, noun));
    }
    let both = Sel::Union(slots.iter().map(|(s, _)| Sel::Target(*s)).collect());
    let kind = |noun: &str| noun.split(' ').next().unwrap_or("").to_string();
    if kind(&slots[0].1) == kind(&slots[1].1) {
        let plural = format!("{}s", kind(&slots[0].1));
        for p in [format!("those {plural}"), format!("the chosen {plural}")] {
            b.named.push((p, both.clone()));
        }
    }
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choose target [A] and target [B]", priority: 80, parse: choose_two_targets } }

/// "those creatures fight each other" (CR 701.14a): the two objects named.
fn fight_each_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let subject = end(l).strip_suffix(" fight each other")?;
    let (sel, rest) = object_ref(subject, b)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let Sel::Union(v) = sel else {
        return None;
    };
    let [x, y] = v.as_slice() else {
        return None;
    };
    Some(Effect::Fight {
        a: x.clone(),
        b: y.clone(),
    })
}

inventory::submit! { EffectPattern { name: "[two objects] fight each other", priority: 80, parse: fight_each_other } }
