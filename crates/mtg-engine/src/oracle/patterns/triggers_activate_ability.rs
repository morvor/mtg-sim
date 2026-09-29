//! Triggers on activating abilities (CR 602.2) that copy them (CR 707.10): "Whenever you
//! activate an ability that targets a creature or player, copy that ability. You may
//! choose new targets for the copy." (Ertha Jo, Frontier Mentor), "Whenever you activate
//! an ability that isn't a mana ability, copy it.", "Whenever you cast an instant or
//! sorcery spell that targets only ~ or activate an ability that targets only ~, copy that
//! spell or ability." (Bill Potts). A copy isn't cast or activated, so it doesn't trigger
//! these again.

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::CardType;

/// "you activate an ability that isn't a mana ability", "you activate an ability that
/// targets a creature or player" (CR 115.9b: some current target is one).
fn you_activate_an_ability(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let rest = end(r).strip_prefix("you activate an ability")?;
    let activated = TriggerCond::AbilityActivated {
        who: PlayerRel::You,
        source: Filter::Any,
        include_mana: false,
    };
    let trigger = match rest {
        " that isn't a mana ability" => activated,
        " that targets a creature or player" => TriggerCond::Where {
            trigger: Box::new(activated),
            cond: Condition::SelMatches(
                Sel::TriggerSpell,
                Filter::StackTargets(Box::new(TargetsFilter::Targets {
                    objects: Some(Filter::and(vec![
                        Filter::Permanent,
                        Filter::Type(CardType::Creature),
                    ])),
                    players: Some(PlayerFilter::Any),
                })),
            ),
        },
        _ => return None,
    };
    Some((trigger, Sel::TriggerSpell, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "you activate an ability [that ...]", priority: 100, parse: you_activate_an_ability } }

/// "Whenever you activate an ability, if it isn't a mana ability, ..." (Rings of
/// Brighthearth), "Whenever you activate an ability of an artifact, if it isn't a mana
/// ability, ..." (Kurkesh, Onakke Ancient), "Whenever an ability of equipped creature is
/// activated, if it isn't a mana ability, ..." (Illusionist's Bracers): the condition
/// read with the trigger (a mana ability never stops being one, so it can't trigger it).
fn activate_an_ability_if_not_mana(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r).strip_suffix(", if it isn't a mana ability")?;
    let (who, source) = if let Some(rest) = r.strip_prefix("you activate an ability") {
        let source = match rest {
            "" => Filter::Any,
            " of an artifact" => Filter::Type(CardType::Artifact),
            _ => return None,
        };
        (PlayerRel::You, source)
    } else if r == "an ability of equipped creature is activated" {
        // The creature it equips when the ability becomes activated, after its costs are
        // paid: one sacrificed to pay them isn't equipped by it any more (CR 602.2b, 601.2i).
        (
            PlayerRel::Any,
            Filter::In(Box::new(Sel::All(Filter::AttachedToSource))),
        )
    } else {
        return None;
    };
    Some((
        TriggerCond::AbilityActivated {
            who,
            source,
            include_mana: false,
        },
        Sel::TriggerSpell,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "an ability is activated, if it isn't a mana ability", priority: 100, parse: activate_an_ability_if_not_mana } }

/// "you cast an instant or sorcery spell that targets only ~ or activate an ability that
/// targets only ~" (CR 115.9c: ~ is the only object or player chosen as its targets).
fn cast_or_activate_targeting_only_source(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let spell = end(r)
        .strip_suffix(" or activate an ability that targets only ~")?
        .strip_prefix("you cast ")?
        .strip_suffix(" that targets only ~")?;
    let spell = spell
        .strip_prefix("a ")
        .or_else(|| spell.strip_prefix("an "))
        .unwrap_or(spell);
    // "an instant or sorcery spell", "a creature spell": a description of a spell.
    if !spell.ends_with(" spell") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(spell)?;
    if !end(tail).is_empty() {
        return None;
    }
    let only_source = Filter::StackTargets(Box::new(TargetsFilter::Only {
        objects: Some(Filter::Source),
        players: None,
    }));
    let trigger = TriggerCond::AnyOf(vec![
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::and(vec![f, only_source.clone()]),
        },
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::AbilityActivated {
                who: PlayerRel::You,
                source: Filter::Any,
                include_mana: false,
            }),
            cond: Condition::SelMatches(Sel::TriggerSpell, only_source),
        },
    ]);
    Some((trigger, Sel::TriggerSpell, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "you cast [spell] or activate an ability that targets only ~", priority: 100, parse: cast_or_activate_targeting_only_source } }

/// "copy that ability", "copy that spell or ability" in an ability that triggers on
/// activating one (or casting a spell): the ability activated or spell cast ("copy it" is
/// `copy_spells`).
fn copy_that_ability(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(end(l), "copy that ability" | "copy that spell or ability")
        || !matches!(b.it, Sel::TriggerSpell)
    {
        return None;
    }
    Some(Effect::CopySpell {
        what: Sel::TriggerSpell,
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy that ability (the one activated)", priority: 100, parse: copy_that_ability } }
