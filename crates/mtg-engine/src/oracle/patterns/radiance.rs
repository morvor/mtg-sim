//! Radiance (an ability word, CR 207.2c): "target creature and each other creature that
//! shares a color with it" (Cleansing Beam, Incite Hysteria, Bathe in Light, Brightflame).
//!
//! Only the one creature is targeted; the others are whatever creatures share a color
//! with it as the spell resolves (CR 608.2c, 105.2): a creature shares a color with a
//! creature that is at least one of its colors, and a colorless creature shares a color
//! with nothing. If the target is illegal as the spell resolves, the spell doesn't
//! resolve at all (CR 608.2b), so no other creature is affected.
//!
//! The clause is parsed as if it named the target alone ("~ deals 2 damage to target
//! creature", "target creature gains ..."), then what the instruction affects is widened
//! to the target and the creatures sharing a color with it.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};

const GROUP: &str = "target creature and each other creature that shares a color with it";

/// The target in `slot` and each other creature that shares a color with it.
fn radiance_group(slot: u8) -> Sel {
    let target = Sel::Target(slot);
    Sel::Union(vec![
        target.clone(),
        Sel::All(Filter::and(vec![
            Filter::creature(),
            Filter::not(Filter::In(Box::new(target.clone()))),
            Filter::SharesColor(Box::new(target)),
        ])),
    ])
}

/// Makes the instructions of `e` that affect exactly the target in `slot` affect `group`
/// instead. Whether any did.
fn widen(e: &mut Effect, slot: u8, group: &Sel) -> bool {
    let is_slot = |s: &Sel| matches!(s, Sel::Target(x) if *x == slot);
    match e {
        Effect::DealDamage { to, .. } if is_slot(to) => {
            *to = group.clone();
            true
        }
        Effect::Modify { what, .. } if is_slot(what) => {
            *what = group.clone();
            true
        }
        Effect::Seq(v) => v
            .iter_mut()
            .fold(false, |acc, x| widen(x, slot, group) || acc),
        _ => false,
    }
}

fn radiance(l: &str, b: &mut Builder) -> Option<Effect> {
    let i = l.find(GROUP)?;
    let (head, tail) = (&l[..i], &l[i + GROUP.len()..]);
    // The group is plural: "... gain protection" names the target alone as "gains".
    let tail = match tail.strip_prefix(" gain ") {
        Some(r) => format!(" gains {r}"),
        None => tail.to_string(),
    };
    let single = format!("{head}target creature{tail}");
    let slot = b.targets.len();
    let mut e = parse_clause(&single, b)?;
    // Exactly the one target this phrase names.
    if b.targets.len() != slot + 1 {
        return None;
    }
    let slot = slot as u8;
    widen(&mut e, slot, &radiance_group(slot)).then_some(e)
}

inventory::submit! { EffectPattern { name: "radiance shares a color", priority: 50, parse: radiance } }
