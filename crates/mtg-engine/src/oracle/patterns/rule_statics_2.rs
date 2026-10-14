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
