//! "As long as ~ isn't on the battlefield, it's a 1/1 Insect creature in addition to its
//! other types." (Grist, the Hunger Tide): an ability that states the zone it doesn't
//! function in functions everywhere else, even outside the game and before the game
//! begins (CR 113.6c). So the card is a creature in its owner's hand, library, graveyard,
//! exile, on the stack and in the command zone, and while the deck is built: it can be a
//! commander (CR 903.3, see `kw::partner::can_be_commander`).

use super::statics::{type_predicate_mods, Subject};
use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::*;

fn outside_the_battlefield(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let r = end(l).strip_prefix("as long as ~ isn't on the battlefield, ")?;
    let r = ["it's ", "~ is ", "it is "]
        .iter()
        .find_map(|p| r.strip_prefix(p))?;
    if !r.ends_with(" in addition to its other types") {
        return None;
    }
    let subj = Subject {
        filter: Filter::Source,
        it: Some(Sel::This),
        hint: CardType::Creature,
        lands: false,
        creatures: ctx.type_line.card_types.contains(CardType::Creature),
    };
    let mods = type_predicate_mods(r, &subj)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility {
            condition: None,
            effect: StaticEffect::Continuous {
                affected: Filter::Source,
                mods,
            },
            zone: FunctionZone::AnywhereExcept(ZoneKind::Battlefield),
            is_cda: false,
        }),
        text,
    )])
}

inventory::submit! {
    StaticPattern {
        name: "r113.6c: as long as ~ isn't on the battlefield, it's a [type]",
        priority: 50,
        parse: outside_the_battlefield,
    }
}
