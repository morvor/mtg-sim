//! Rule-modifying statics II:
//!
//! - "You can cast spells only during your turn and you can cast no more than two spells
//!   each turn." (Fires of Invention; CR 601.3, counting spells cast earlier in the turn);
//! - "As long as ~ is on the battlefield, [static ability]" (Haakon): a static ability of
//!   a permanent functions only on the battlefield anyway (CR 113.6);
//! - "Spells and abilities your opponents control can't cause you to discard cards or
//!   sacrifice permanents." (Tamiyo, Collector of Tales), "... can't cause their
//!   controller to search their library." (Ashiok, Dream Render): see
//!   `rule_statics::caused_actions`;
//! - "Creatures your opponents control can be the targets of spells and abilities as
//!   though they didn't have hexproof. Ward abilities of those creatures don't trigger."
//!   (Nowhere to Run; CR 702.11e, 609.4);
//! - "Until end of turn, your opponents and creatures your opponents control with hexproof
//!   can be the targets of spells and abilities you control as though they didn't have
//!   hexproof." (Detection Tower).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn stat(e: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(e)), text)
}

fn rule_statics(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l).trim();
    let r = |r: Restriction| stat(StaticEffect::Restriction(r), text);
    let custom = |n: &str| stat(StaticEffect::Custom(n.into()), text);
    match l {
        "you can cast spells only during your turn and you can cast no more than two spells each turn" => {
            return Some(vec![
                r(Restriction::CantCast {
                    who: PlayerFilter::And(vec![
                        PlayerFilter::You,
                        PlayerFilter::Not(Box::new(PlayerFilter::Active)),
                    ]),
                    what: Filter::Any,
                }),
                r(Restriction::MaxSpellsPerTurn(PlayerFilter::You, 2)),
            ]);
        }
        "spells and abilities your opponents control can't cause you to discard cards or sacrifice permanents" => {
            return Some(vec![
                custom(crate::rule_statics::caused_actions::CANT_CAUSE_YOU_TO_DISCARD),
                r(Restriction::CantCauseSacrifice {
                    what: Filter::Permanent,
                    by: SacrificeCauses::OpponentsSpellsAndAbilities,
                    exile: false,
                }),
            ]);
        }
        "spells and abilities your opponents control can't cause you to discard cards" => {
            return Some(vec![custom(
                crate::rule_statics::caused_actions::CANT_CAUSE_YOU_TO_DISCARD,
            )]);
        }
        "spells and abilities your opponents control can't cause their controller to search their library" => {
            return Some(vec![custom(
                crate::rule_statics::caused_actions::CANT_CAUSE_CONTROLLER_TO_SEARCH,
            )]);
        }
        "untap all permanents you control during each opponent's untap step" => {
            return Some(vec![custom(
                crate::rule_statics::opponents_steps::UNTAP_DURING_OPPONENTS_UNTAP_STEPS,
            )]);
        }
        "you draw a card during each opponent's draw step" => {
            return Some(vec![custom(
                crate::rule_statics::opponents_steps::DRAW_DURING_OPPONENTS_DRAW_STEPS,
            )]);
        }
        "creatures your opponents control can be the targets of spells and abilities as though they didn't have hexproof. ward abilities of those creatures don't trigger" => {
            return Some(vec![custom(
                crate::kw::hexproof::OPPONENT_CREATURES_TARGETABLE_BY_ANY,
            )]);
        }
        _ => {}
    }
    // "As long as ~ is on the battlefield, [static ability]".
    let rest = l.strip_prefix("as long as ~ is on the battlefield, ")?;
    let original = text.get(l.len() - rest.len()..).unwrap_or(rest);
    let v = crate::oracle::statics::parse_static(original, ctx)?;
    let all_battlefield = v.iter().all(|a| {
        matches!(&a.kind, AbilityKind::Static(s) if s.zone == FunctionZone::Battlefield)
    });
    all_battlefield.then(|| {
        v.into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), text))
            .collect()
    })
}

inventory::submit! { StaticPattern { name: "rule statics 2: casting limits, caused discards and searches, hexproof", priority: 95, parse: rule_statics } }

/// "Until end of turn, your opponents and creatures your opponents control with hexproof
/// can be the targets of spells and abilities you control as though they didn't have
/// hexproof." (Detection Tower).
fn targetable_until_eot(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l != "until end of turn, your opponents and creatures your opponents control with hexproof can be the targets of spells and abilities you control as though they didn't have hexproof" {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::Custom(
            crate::kw::hexproof::OPPONENTS_AND_CREATURES_AS_THOUGH_NO_HEXPROOF.into(),
        ),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "rule statics 2: until end of turn, [players and creatures] can be the targets ... as though they didn't have hexproof", priority: 95, parse: targetable_until_eot } }

/// "~ has each activated ability of the exiled cards used to craft it. You may activate
/// each of those abilities only once each turn." (The Enigma Jewel // Locus of
/// Enlightenment): activation instructions for the abilities a static ability gives
/// (CR 113.10a, 602.5c; see `gained_terms`).
fn static_with_gained_terms(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || block.contains('\n') || block.contains(':') {
        return None;
    }
    let lower = block.to_lowercase();
    let i = lower.find(". you may ")?;
    let (first, rider) = (&block[..i + 1], &lower[i + 2..]);
    let v = crate::oracle::statics::parse_static(first, ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let AbilityKind::Static(st) = &a.kind else {
        return None;
    };
    let mut st = st.clone();
    let StaticEffect::Continuous { mods, .. } = &mut st.effect else {
        return None;
    };
    let terms = super::gained_terms::last_terms(mods)?;
    if !super::gained_terms::add_rider(rider, terms) {
        return None;
    }
    Some(vec![AbilityDef::new(AbilityKind::Static(st), block)])
}

inventory::submit! { super::AbilityPattern { name: "rule statics 2: [static giving abilities]. you may activate each of those abilities only once each turn", priority: 50, parse: static_with_gained_terms } }

/// "If ~ is in a graveyard, effects from spells named Flame Burst count it as a card named
/// Flame Burst." (Pardic Firecat, Diligent Farmhand): see `counts_as_named`. It functions
/// in graveyards (CR 113.6b).
fn counts_as_named_static(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ~ is in a graveyard, effects from spells named ")?;
    let (a, b) = r.split_once(" count it as a card named ")?;
    if a != b {
        return None;
    }
    let (name, rest) = super::value_grammar::literal_card_name(a)?;
    if !rest.is_empty() || ctx.card_name.is_empty() {
        return None;
    }
    let mut st = StaticAbility::new(StaticEffect::Custom(
        format!("{}{name}", crate::counts_as_named::PREFIX).into(),
    ));
    st.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { StaticPattern { name: "rule statics 2: effects from spells named [name] count it as a card named [name]", priority: 95, parse: counts_as_named_static } }

/// "Creatures with flying can't attack or block, and their activated abilities with {T}
/// in their costs can't be activated." (Katabatic Winds): see `tap_abilities_prohibited`.
fn tap_abilities_cant_be_activated(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l);
    let (head, tail) = l.split_once(", and their activated abilities with {t} in their costs can't be activated")?;
    if !tail.is_empty() {
        return None;
    }
    let (subj, pred) = head.split_once(" can't ")?;
    let (f, plural, rest) = crate::oracle::phrases::parse_object_phrase(subj)?;
    if !plural || !end(rest).is_empty() {
        return None;
    }
    let restriction = match pred {
        "attack or block" => Restriction::CantAttackOrBlock(f.clone()),
        "attack" => Restriction::CantAttack(f.clone()),
        "block" => Restriction::CantBlock(f.clone()),
        _ => return None,
    };
    Some(vec![
        stat(StaticEffect::Restriction(restriction), text),
        stat(crate::kw::tap_abilities_prohibited::effect(&f)?, text),
    ])
}

inventory::submit! { StaticPattern { name: "rule statics 2: their activated abilities with {T} in their costs can't be activated", priority: 95, parse: tap_abilities_cant_be_activated } }

/// "You may cast ~ from exile [as long as (condition)]": granted to an exiled card (Monk
/// Class; Lukka, Coppercoat Outcast), a permission that functions while the card is in
/// exile (CR 113.6b, 604.6).
fn cast_self_from_exile(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may cast ~ from exile")?;
    let condition = if r.is_empty() {
        None
    } else {
        Some(crate::oracle::statics::parse_condition(r.strip_prefix(" as long as ")?, ctx)?)
    };
    let mut s = StaticAbility::new(StaticEffect::PlayPermission(PlayPermission {
        who: PlayerRel::You,
        zone: ZoneKind::Exile,
        top_only: false,
        what: Filter::Source,
        lands: false,
        spells: true,
        cost: None,
        flash: false,
        terms: Default::default(),
    }));
    s.zone = FunctionZone::Exile;
    s.condition = condition;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "rule statics 2: you may cast ~ from exile [as long as ...]", priority: 95, parse: cast_self_from_exile } }
