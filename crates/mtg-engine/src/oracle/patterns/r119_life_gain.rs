//! Life gain replaced or used as a cost (CR 119.7, 614.1a):
//!
//! * "If an opponent would gain life, that player loses that much life instead." (Tainted
//!   Remedy, Plague Drone), "If you would gain life, draw that many cards instead." (Lich,
//!   Nefarious Lich): replacement effects that replace a life-gain event with another
//!   event. A player who can't gain life has no life-gain event to replace (CR 119.7;
//!   see `Game::gain_life`).
//! * "{1}: The next time you would draw a card this turn, you gain 5 life instead." (the
//!   Words cycle): a one-use replacement effect created by an ability (CR 614.1a, 615.7).
//! * "If you control a Forest, rather than pay this spell's mana cost, you may have an
//!   opponent gain 3 life." (Invigorate, Reverent Silence): an alternative cost that has
//!   other players gain life (CR 118.9); it can't be paid if a player who would gain the
//!   life can't gain life (CR 119.7; see `life_totals::cost_life_gain_possible`).

use super::costs_casting_self::{cost_condition, this_spell_cost_ability};
use super::{AbilityPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "If [an opponent | a player | you] would gain life, [that player loses that much life |
/// draw that many cards] instead."
fn gain_life_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ")?;
    let (who, r) = [
        ("an opponent would gain life, ", PlayerFilter::Opponent),
        ("a player would gain life, ", PlayerFilter::Any),
        ("you would gain life, ", PlayerFilter::You),
    ]
    .into_iter()
    .find_map(|(p, f)| r.strip_prefix(p).map(|r| (f, r)))?;
    // In a replacement effect, the event's player is the trigger player and "that much"
    // is the event's amount.
    let effect = match (r, &who) {
        ("that player loses that much life instead", PlayerFilter::Opponent | PlayerFilter::Any) => {
            Effect::LoseLife {
                who: PlayerRef::TriggerPlayer,
                n: Value::EventAmount,
            }
        }
        (
            "draw that many cards instead" | "you draw that many cards instead",
            PlayerFilter::You,
        ) => Effect::Draw {
            who: PlayerRef::You,
            n: Value::EventAmount,
        },
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::GainLife(who),
                action: ReplacementAction::Instead(Box::new(effect)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r119 if [player] would gain life, [other event] instead", priority: 0, parse: gain_life_instead } }

/// "The next time you would draw a card this turn, [effect] instead.": one use of a
/// replacement effect for the rest of the turn. The effect can't have targets (they'd be
/// chosen as the replacement applies, which this form doesn't express).
fn next_draw_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let inner = end(l)
        .strip_prefix("the next time you would draw a card this turn, ")?
        .strip_suffix(" instead")?;
    let saved = b.targets.len();
    let effect = parse_clause(inner, b);
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(effect?)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "r119 the next time you would draw a card this turn, [effect] instead", priority: 50, parse: next_draw_instead } }

/// "If [condition], rather than pay ~'s mana cost, you may have [an opponent | each other
/// player | each opponent] gain N life."
fn life_gain_alternative_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let l = end(&lower);
    let (head, rest) = l.split_once(", rather than pay ~'s mana cost, you may have ")?;
    let cond = cost_condition(head, ctx)?;
    let (who, amount) = rest.split_once(" gain ")?;
    let (n, tail) = parse_number(amount)?;
    if tail.trim() != "life" || matches!(n, Value::X) {
        return None;
    }
    let effect = match who {
        // The opponent is chosen as the cost is paid.
        "an opponent" => Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Opponent,
            },
            Effect::GainLife {
                who: PlayerRef::ChosenOpponent,
                n,
            },
        ]),
        "each other player" => Effect::GainLife {
            who: PlayerRef::EachOtherPlayer,
            n,
        },
        "each opponent" => Effect::GainLife {
            who: PlayerRef::EachOpponent,
            n,
        },
        _ => return None,
    };
    let cost = Cost::free().with(CostPart::Effect(Box::new(effect)));
    Some(vec![this_spell_cost_ability(
        CostChange::AlternativeCost(cost),
        Some(cond),
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "r119 rather than pay this spell's mana cost, have players gain life", priority: 80, parse: life_gain_alternative_cost } }
