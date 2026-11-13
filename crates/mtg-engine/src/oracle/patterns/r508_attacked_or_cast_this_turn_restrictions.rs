//! Restrictions on players by what they did this turn (Angelic Arbiter):
//!
//! * "Each opponent who cast a spell this turn can't attack with creatures." — creatures
//!   whose controller is such an opponent can't be declared as attackers (CR 508.1c);
//! * "Each opponent who attacked with a creature this turn can't cast spells." — a player
//!   who declared one or more attackers this turn (CR 508.1) can't cast spells (CR 601.3);
//! * "Each opponent who attacked you or a planeswalker you control this turn can't cast
//!   spells." (Sandswirl Wanderglyph) — only attacks on you and your planeswalkers count
//!   (CR 508.6), not on battles you protect or on your teammates.
//!
//! Both look at the whole turn, including what happened before the ability began to apply
//! (Angelic Arbiter rulings), and stop applying as soon as the source leaves.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn restriction(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let opp = |f: PlayerFilter| PlayerFilter::And(vec![PlayerFilter::Opponent, f]);
    let r = match end(l).trim() {
        "each opponent who cast a spell this turn can't attack with creatures" => {
            Restriction::CantAttack(Filter::and(vec![
                Filter::Type(CardType::Creature),
                Filter::ControllerMatches(Box::new(opp(PlayerFilter::CastSpellThisTurn))),
            ]))
        }
        "each opponent who attacked with a creature this turn can't cast spells" => {
            Restriction::CantCast {
                who: opp(PlayerFilter::AttackedThisTurn),
                what: Filter::Any,
            }
        }
        "each opponent who attacked you or a planeswalker you control this turn can't cast spells" => {
            Restriction::CantCast {
                who: opp(PlayerFilter::AttackRelation {
                    attacking: true,
                    other: Box::new(PlayerFilter::You),
                    past: true,
                    planeswalkers: true,
                }),
                what: Filter::Any,
            }
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r508: each opponent who cast a spell / attacked this turn can't attack / cast", priority: 120, parse: restriction } }
