//! Putting cards onto the battlefield and what they are as they enter (CR 110.2, 611.2e):
//!
//! - "Put target creature card from a graveyard onto the battlefield under your control",
//!   "put it onto the battlefield tapped under its owner's control": the player who puts
//!   a permanent onto the battlefield controls it unless the effect says otherwise
//!   (CR 110.2a).
//! - After such a sentence (or a "return ... to the battlefield"): "That creature is a
//!   black Zombie in addition to its other colors and types", "It's a Phyrexian in
//!   addition to its other types", "They're Zombies in addition to their other types", and
//!   "It's an enchantment." (the Enduring cycle): characteristics the permanent has as it
//!   enters, from a continuous effect of the resolving ability (CR 611.2e).

use super::{ConditionPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::types::*;

inventory::submit! {
    EffectPattern { name: "control_exile: put onto the battlefield", priority: 90, parse: p_put_onto_battlefield }
}
inventory::submit! {
    FollowupPattern { name: "control_exile: it's [types] as it enters", priority: 60, apply: f_enters_as }
}
inventory::submit! {
    ConditionPattern { name: "control_exile: it was a creature", priority: 100, parse: c_it_was_a_creature }
}

/// "When ~ dies, if it was a creature, ...": the source as it last existed on the
/// battlefield (CR 603.10a, 603.4). "It" in a trigger condition is the source (see
/// `oracle::triggers::parse_triggered`), and a source that left the battlefield is
/// evaluated with its last known information.
fn c_it_was_a_creature(c: &str) -> Option<Condition> {
    (c == "it was a creature").then(|| Condition::SelMatches(Sel::This, Filter::creature()))
}

/// "onto the battlefield [tapped] [under your control / under its owner's control]".
fn onto_battlefield(s: &str, owned: &Sel) -> Option<Destination> {
    let mut r = s.trim().strip_prefix("onto the battlefield")?.trim_start();
    let mut d = Destination::battlefield().under_your_control();
    loop {
        if let Some(x) = r.strip_prefix("tapped") {
            d.tapped = true;
            r = x.trim_start();
        } else if let Some(x) = r.strip_prefix("under your control") {
            r = x.trim_start();
        } else if let Some(x) = r
            .strip_prefix("under its owner's control")
            .or_else(|| r.strip_prefix("under their owner's control"))
            .or_else(|| r.strip_prefix("under their owners' control"))
        {
            d.controller = Some(PlayerRef::OwnerOf(Box::new(owned.clone())));
            r = x.trim_start();
        } else {
            break;
        }
    }
    r.is_empty().then_some(d)
}

/// "put target creature card from a graveyard onto the battlefield under your control".
fn p_put_onto_battlefield(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("put ")?;
    let (what, tail) = object_ref(r, b)?;
    // Specific objects: targets or objects named earlier. (Choosing "a creature card"
    // from a zone is a different instruction.)
    if !matches!(
        what,
        Sel::Target(_) | Sel::This | Sel::TriggerObject | Sel::Var(_)
    ) {
        return None;
    }
    let to = onto_battlefield(&tail, &what)?;
    // "Then that player mills X cards" after taking a card from a graveyard: the owner
    // of that graveyard.
    if let Sel::Target(slot) = what {
        let from_graveyard = b.targets.get(slot as usize).is_some_and(
            |t| matches!(&t.what, TargetKind::Object(f) if f.zone() == Some(ZoneKind::Graveyard)),
        );
        if from_graveyard {
            b.it_player = PlayerRef::OwnerOf(Box::new(what.clone()));
        }
    }
    // "That creature ..." afterwards: the permanent it became (CR 400.7).
    b.it = Sel::Var(vars::IT);
    Some(Effect::Move { what, to })
}

/// The last effect of a sequence (in execution order; `Effect::seq` flattens nested
/// sequences).
fn last_effect_mut(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    }
}

/// Whether the objects selected are known to be creatures (a target or a trigger object
/// described as a creature).
fn selects_creatures(what: &Sel, b: &Builder) -> bool {
    fn has_creature(f: &Filter) -> bool {
        match f {
            Filter::Type(CardType::Creature) => true,
            Filter::And(v) => v.iter().any(has_creature),
            Filter::Or(v) => !v.is_empty() && v.iter().all(has_creature),
            _ => false,
        }
    }
    match what {
        Sel::Target(slot) => b
            .targets
            .get(*slot as usize)
            .is_some_and(|t| matches!(&t.what, TargetKind::Object(f) if has_creature(f))),
        _ => false,
    }
}

/// "it's a black Zombie in addition to its other colors and types", "that creature is a
/// Phyrexian in addition to its other types", "they're Zombies in addition to their other
/// types", "it's an enchantment".
fn f_enters_as(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(Effect::Move { what, to }) = last_effect_mut(prev) else {
        return false;
    };
    if to.zone != ZoneKind::Battlefield || !to.with_mods.is_empty() {
        return false;
    }
    let Some(r) = [
        "it's ",
        "it is ",
        "that creature is ",
        "they're ",
        "they are ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    // "It's an enchantment. (It's not a creature.)": its only card type (and it has no
    // creature types, CR 205.3d). Only for the card itself returning, when its printed
    // subtypes are all creature types (setting the types removes them all).
    if r == "an enchantment" {
        let own = matches!(what, Sel::This | Sel::TriggerObject)
            || (b.in_trigger && matches!(b.it, Sel::This | Sel::TriggerObject));
        let only_creature_types = b
            .ctx
            .type_line
            .subtypes
            .iter()
            .all(|s| subtype_kind(s) == Some(SubtypeKind::Creature));
        if !own || !only_creature_types {
            return false;
        }
        to.with_mods = vec![Modification::SetTypes {
            types: vec![CardType::Enchantment],
            subtypes: vec![],
        }];
        return true;
    }
    let (x, with_colors) = if let Some(x) = r
        .strip_suffix(" in addition to its other colors and types")
        .or_else(|| r.strip_suffix(" in addition to their other colors and types"))
    {
        (x, true)
    } else if let Some(x) = r
        .strip_suffix(" in addition to its other types")
        .or_else(|| r.strip_suffix(" in addition to their other types"))
    {
        (x, false)
    } else {
        return false;
    };
    let x = x
        .strip_prefix("a ")
        .or_else(|| x.strip_prefix("an "))
        .unwrap_or(x);
    let mut colors = ColorSet::NONE;
    let mut subtypes: Vec<Subtype> = Vec::new();
    for w in x.split(' ') {
        if let Some(c) = Color::from_word(w) {
            colors.insert(c);
        } else if let Some(st) = subtype_word(w) {
            // Only creature types, added to creatures (CR 205.3d).
            if subtype_kind(&st) != Some(SubtypeKind::Creature) {
                return false;
            }
            subtypes.push(st);
        } else {
            return false;
        }
    }
    if subtypes.is_empty() || (colors != ColorSet::NONE) != with_colors {
        return false;
    }
    let what = what.clone();
    if !selects_creatures(&what, b) {
        return false;
    }
    let mut mods = Vec::new();
    if with_colors {
        mods.push(Modification::AddColors(colors));
    }
    mods.push(Modification::AddSubtypes(subtypes));
    let Some(Effect::Move { to, .. }) = last_effect_mut(prev) else {
        return false;
    };
    to.with_mods = mods;
    true
}
