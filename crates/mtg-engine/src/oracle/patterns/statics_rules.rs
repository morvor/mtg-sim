//! Rule-modifying static abilities about spells and abilities: "creature spells you
//! control can't be countered" (CR 701.6), "activated abilities of artifacts can't be
//! activated" (CR 602.5, which includes mana abilities, CR 605.1a).

use super::statics::{filter_mentions, mentions_other_zones, union_nouns, whole_object_phrase};
use crate::ability::*;
use crate::oracle::patterns::StaticPattern;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn restriction(r: Restriction, text: &str) -> Vec<Ability> {
    vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )]
}

/// "Creature spells you control can't be countered", "Green spells you control can't be
/// countered", "Creature spells can't be countered".
fn spells_cant_be_countered(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let subject = end(l).strip_suffix(" can't be countered")?;
    let (f, plural) = whole_object_phrase(&union_nouns(subject))?;
    if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Spell)) {
        return None;
    }
    Some(restriction(Restriction::CantBeCountered(f), text))
}

/// "Activated abilities of artifacts can't be activated", "Activated abilities of
/// creatures your opponents control can't be activated", "... unless they're mana
/// abilities".
fn abilities_cant_be_activated(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l);
    let (rest, include_mana) =
        if let Some(r) = l.strip_suffix(" can't be activated unless they're mana abilities") {
            (r, false)
        } else {
            (l.strip_suffix(" can't be activated")?, true)
        };
    let group = rest.strip_prefix("activated abilities of ")?;
    let (f, plural) = whole_object_phrase(&union_nouns(group))?;
    if !plural || mentions_other_zones(&f) {
        return None;
    }
    Some(restriction(
        Restriction::CantActivate {
            who: PlayerFilter::Any,
            // "Artifacts" are artifact permanents (CR 109.2).
            sources: Filter::and(vec![f, Filter::Permanent]),
            include_mana,
        },
        text,
    ))
}

/// "Cards in graveyards can't be the targets of spells or abilities" (CR 115.4).
fn graveyard_cards_untargetable(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let subject = end(l).strip_suffix(" can't be the targets of spells or abilities")?;
    let (f, plural) = whole_object_phrase(&union_nouns(subject))?;
    if !plural || f.zone() != Some(ZoneKind::Graveyard) {
        return None;
    }
    Some(restriction(
        Restriction::CantBeTargeted {
            what: f,
            by: TargetRestriction::Any,
        },
        text,
    ))
}

inventory::submit! {
    StaticPattern {
        name: "statics: cards in graveyards can't be targeted",
        priority: 50,
        parse: graveyard_cards_untargetable,
    }
}

inventory::submit! {
    StaticPattern {
        name: "statics: spells can't be countered",
        priority: 50,
        parse: spells_cant_be_countered,
    }
}

inventory::submit! {
    StaticPattern {
        name: "statics: activated abilities of a group can't be activated",
        priority: 50,
        parse: abilities_cant_be_activated,
    }
}
