//! Mana added by mana abilities, as referents and triggers (CR 605.1b, 106.12):
//!
//! - "Whenever a land's ability causes you to add one or more mana of the chosen color,
//!   add an additional one mana of that color." (Caged Sun): `TriggerCond::Custom`
//!   [`trigger_name`] on `Event::ManaAbilityAdded` (an activated mana ability of a land
//!   resolved and added mana to that player's pool, with or without {T} in its cost). It's a
//!   triggered mana ability (CR 605.1b), resolving at once.
//! - "Add one mana of any color. ~ becomes that color until end of turn." (Foraging
//!   Wickermaw): the colors of the mana an `AddMana` instruction just added are kept for the
//!   rest of the resolution ([`record_added`]), and the `Modification::Custom`
//!   [`BECOMES_COLOR_ADDED`] becomes those colors as the effect is created.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::ManaType;
use crate::object::EventInfo;
use crate::types::*;

/// `TriggerCond::Custom` prefix: `mana ability added:<card type>:<you|any>`.
pub const TRIGGER: &str = "mana ability added:";

/// The trigger's name for an ability of a permanent of `card_type` adding mana to `you`
/// (the trigger's controller) or to any player.
pub fn trigger_name(card_type: CardType, you: bool) -> String {
    format!("{TRIGGER}{card_type:?}:{}", if you { "you" } else { "any" })
}

/// Whether a `TriggerCond::Custom` name is this trigger (a triggered mana ability's
/// trigger, CR 605.1b).
pub fn is_mana_ability_trigger(name: &str) -> bool {
    name.starts_with(TRIGGER)
}

/// `Modification::Custom`: the object becomes the color(s) of the mana just added.
pub const BECOMES_COLOR_ADDED: &str = "mana added:becomes that color";

/// The `Ctx::nums` key of the colors of the mana just added.
const ADDED_COLORS: crate::ability::Var = crate::ability::vars::USER + 14_300;

/// Records the colors of the mana an instruction added.
pub fn record_added(ctx: &mut Ctx, added: &[ManaType]) {
    let mut cs = ColorSet::NONE;
    for t in added {
        if let Some(c) = t.color() {
            cs.insert(c);
        }
    }
    ctx.nums.insert(ADDED_COLORS, cs.0 as i64);
}

/// The colors of the mana the last `AddMana` instruction added, if one did.
pub fn added_colors(ctx: &Ctx) -> Option<ColorSet> {
    ctx.nums
        .get(&ADDED_COLORS)
        .map(|n| ColorSet(*n as u8))
        .filter(|cs| !cs.is_colorless())
}

struct ManaAbilityAdded;

impl KeywordRules for ManaAbilityAdded {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        let r = name.strip_prefix(TRIGGER)?;
        let (ty, who) = r.split_once(':')?;
        let Event::ManaAbilityAdded { obj, player, mana } = ev else {
            return Some(vec![]);
        };
        let ty = CardType::ALL.iter().find(|t| format!("{t:?}") == ty)?;
        let matches =
            g.try_obj(*obj).is_some_and(|o| o.chars.is(*ty)) && (who == "any" || *player == ctl);
        Some(if matches {
            vec![EventInfo {
                object: Some(*obj),
                player: Some(*player),
                mana: mana.clone(),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&ManaAbilityAdded) }
