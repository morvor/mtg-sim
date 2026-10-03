//! "Creatures entering don't cause abilities to trigger." (Torpor Orb, Hushwing Gryff,
//! Tocatli Honor Guard) and "Artifacts and creatures entering don't cause abilities to
//! trigger." (Doorkeeper Thrull): while such a static ability functions, a permanent of
//! those types entering the battlefield triggers no triggered abilities — its own
//! enters abilities or any other ability that would trigger on it entering. Replacement
//! effects and "as [this] enters" abilities, which aren't triggered abilities, still apply.
//! The text is parsed in `oracle/patterns/r603_torpor.rs`.

use crate::events::Event;
use crate::game::Game;
use crate::object::Zone;
use crate::types::CardType;
use smol_str::SmolStr;

/// `StaticEffect::Custom` name prefix, followed by the card types (comma-separated) whose
/// entering doesn't cause abilities to trigger.
pub const ENTERING_DOESNT_TRIGGER: &str = "entering doesn't cause abilities to trigger:";

/// The static ability's name for these card types.
pub fn static_name(types: &[CardType]) -> SmolStr {
    let words: Vec<String> = types.iter().map(|t| t.word().to_string()).collect();
    SmolStr::new(format!("{ENTERING_DOESNT_TRIGGER}{}", words.join(",")))
}

/// Whether `ev` is a permanent entering the battlefield that, because of such a static
/// ability, doesn't cause abilities to trigger. The permanent is looked at as it exists on
/// the battlefield; a static ability of a permanent entering at the same time applies.
pub fn entering_triggers_nothing(g: &Game, ev: &Event) -> bool {
    let Event::ZoneChange {
        new,
        to: Zone::Battlefield,
        ..
    } = ev
    else {
        return false;
    };
    let o = g.obj(*new);
    g.statics.customs.iter().any(|(_, _, name)| {
        name.strip_prefix(ENTERING_DOESNT_TRIGGER)
            .is_some_and(|types| {
                types
                    .split(',')
                    .filter_map(CardType::from_word)
                    .any(|t| o.is(t))
            })
    })
}
