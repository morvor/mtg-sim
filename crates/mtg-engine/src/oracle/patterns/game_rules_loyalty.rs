//! Loyalty-ability rules (CR 606) in Oracle text: who can't activate loyalty abilities
//! ("Players can't activate planeswalkers' loyalty abilities.", "Your opponents can't
//! cast spells or activate planeswalkers' loyalty abilities this turn.", "Enchanted player
//! can't activate abilities that aren't mana abilities or loyalty abilities."), a
//! permission to activate them twice this turn (CR 606.3), and "target loyalty ability
//! of a planeswalker" (read by the target phrase parser).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::kw::game_rules as gr;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

const LOYALTY: &str = "activate planeswalkers' loyalty abilities";

fn custom_static(name: &str, text: &str) -> Vec<Ability> {
    let s = StaticAbility::new(StaticEffect::Restriction(Restriction::Custom(name.into())));
    vec![AbilityDef::new(AbilityKind::Static(s), text)]
}

/// "Players can't activate planeswalkers' loyalty abilities.", "Your opponents can't
/// activate ...", "Enchanted player can't activate abilities that aren't mana abilities
/// or loyalty abilities.".
fn cant_activate_loyalty(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let name = match end(l) {
        "players can't activate planeswalkers' loyalty abilities"
        | "each player can't activate planeswalkers' loyalty abilities" => gr::NO_LOYALTY_EACH,
        "your opponents can't activate planeswalkers' loyalty abilities"
        | "each opponent can't activate planeswalkers' loyalty abilities" => {
            gr::NO_LOYALTY_OPPONENTS
        }
        "enchanted player can't activate abilities that aren't mana abilities or loyalty abilities" => {
            gr::ENCHANTED_ONLY_MANA_OR_LOYALTY
        }
        _ => return None,
    };
    Some(custom_static(name, text))
}

inventory::submit! { StaticPattern { name: "game rules: can't activate loyalty abilities", priority: 100, parse: cant_activate_loyalty } }

/// "Your opponents can't cast spells or activate planeswalkers' loyalty abilities this
/// turn.", "Players can't activate planeswalkers' loyalty abilities this turn.".
fn cant_activate_loyalty_this_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" this turn")?;
    let (who, r) = if let Some(r) = r.strip_prefix("your opponents can't ") {
        (PlayerFilter::Opponent, r)
    } else if let Some(r) = r.strip_prefix("players can't ") {
        (PlayerFilter::Any, r)
    } else {
        return None;
    };
    let (cast, rest) = match r.strip_prefix("cast spells or ") {
        Some(rest) => (true, rest),
        None => (false, r),
    };
    if rest != LOYALTY {
        return None;
    }
    let name = match who {
        PlayerFilter::Opponent => gr::NO_LOYALTY_OPPONENTS,
        _ => gr::NO_LOYALTY_EACH,
    };
    let mut v = Vec::new();
    if cast {
        v.push(Effect::AddRestriction {
            restriction: Restriction::CantCast {
                who,
                what: Filter::Any,
            },
            duration: Duration::EndOfTurn,
        });
    }
    v.push(Effect::AddRestriction {
        restriction: Restriction::Custom(name.into()),
        duration: Duration::EndOfTurn,
    });
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "game rules: can't activate loyalty abilities this turn", priority: 100, parse: cant_activate_loyalty_this_turn } }

/// "You may activate the loyalty abilities of planeswalkers you control twice this turn
/// rather than only once." (Urza Assembles the Titans): the permission of Oath of Teferi
/// until end of turn (CR 606.3).
fn twice_this_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("you may activate the loyalty abilities of ")?
        .strip_suffix(" twice this turn rather than only once")?;
    let sources = super::activated_ability_costs::sources(r)?;
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::ActivationPermission(ActivationPermission {
            scope: AbilityScope::new(sources, AbilityClass::Loyalty),
            instant_timing: false,
            loyalty_per_turn: Some(2),
            as_though_haste: false,
        }),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "game rules: activate loyalty abilities twice this turn", priority: 100, parse: twice_this_turn } }
