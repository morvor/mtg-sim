//! Oracle patterns for drawing cards (CR 121): limits on draws (CR 121.2b, 121.3),
//! replacement effects referring to the number of cards drawn (CR 121.2a), and "if you
//! would draw a card while your library has no cards in it" (CR 121.6a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

fn replacement(event: ReplacementEvent, action: ReplacementAction) -> StaticEffect {
    StaticEffect::Replacement(ReplacementDef {
        event,
        action,
        self_replacement: false,
        optional: false,
    })
}

/// "Each player can't draw more than one card each turn", "Players can't draw cards".
fn draw_limits(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (who, n) = match l {
        "each player can't draw more than one card each turn" => (PlayerFilter::Any, 1),
        "each opponent can't draw more than one card each turn" => (PlayerFilter::Opponent, 1),
        "you can't draw more than one card each turn" => (PlayerFilter::You, 1),
        "players can't draw cards" => (PlayerFilter::Any, 0),
        "you can't draw cards" => (PlayerFilter::You, 0),
        "your opponents can't draw cards" => (PlayerFilter::Opponent, 0),
        _ => return None,
    };
    Some(vec![static_ability(
        StaticEffect::Restriction(Restriction::MaxDrawsPerTurn(who, n)),
        text,
    )])
}

/// "If an opponent would draw two or more cards, instead you and that player each draw a
/// card." (Alms Collector.)
fn multiple_draws(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (who, rest) =
        if let Some(r) = l.strip_prefix("if an opponent would draw two or more cards, ") {
            (PlayerFilter::Opponent, r)
        } else if let Some(r) = l.strip_prefix("if a player would draw two or more cards, ") {
            (PlayerFilter::Any, r)
        } else {
            return None;
        };
    let effect = match rest {
        "instead you and that player each draw a card" => Effect::seq(vec![
            Effect::Draw {
                who: PlayerRef::TriggerPlayer,
                n: Value::c(1),
            },
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(1),
            },
        ]),
        "that player draws a card instead" | "instead that player draws a card" => Effect::Draw {
            who: PlayerRef::TriggerPlayer,
            n: Value::c(1),
        },
        _ => return None,
    };
    Some(vec![static_ability(
        replacement(
            ReplacementEvent::DrawCards { who, min: 2 },
            ReplacementAction::Instead(Box::new(effect)),
        ),
        text,
    )])
}

/// "If you would draw a card while your library has no cards in it, you win the game
/// instead." (CR 121.6a: the replacement applies even though no card could be drawn.)
fn draw_from_empty_library(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "if you would draw a card while your library has no cards in it, you win the game instead"
    {
        return None;
    }
    let mut s = StaticAbility::new(replacement(
        ReplacementEvent::Draw(PlayerFilter::You),
        ReplacementAction::Instead(Box::new(Effect::WinGame {
            who: PlayerRef::You,
        })),
    ));
    s.condition = Some(Condition::Compare(
        Value::LibrarySize(PlayerRef::You),
        Cmp::Eq,
        Value::c(0),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "draw limits", priority: 0, parse: draw_limits } }
inventory::submit! { StaticPattern { name: "would draw two or more cards", priority: 0, parse: multiple_draws } }
inventory::submit! { StaticPattern { name: "draw from an empty library", priority: 0, parse: draw_from_empty_library } }
