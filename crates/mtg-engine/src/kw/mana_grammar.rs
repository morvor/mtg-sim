//! Engine rules behind the mana grammar (`oracle/patterns/mana_grammar.rs`):
//!
//! * [`EXCEPT_MANA_ABILITIES`]: "All lands lose all abilities except mana abilities."
//!   (Blood Sun; layer 6, CR 613.1f): the permanents keep only their mana abilities
//!   (CR 605.1a-b).
//! * [`COULD_PRODUCE`]: "a land that could produce {C}" (CR 106.7): a mana ability of the
//!   permanent could produce mana of that type.
//! * [`CHROMA_GRAVEYARD`]: "the number of black mana symbols in the mana costs of cards in
//!   your graveyard" (chroma, CR 207.2c): hybrid symbols count once for each of their
//!   colors (CR 107.4e).
//! * [`MANA_BURN`]: "A player losing unspent mana causes that player to lose that much
//!   life." (Yurlok of Scorch Thrash): applied as mana empties from a pool (CR 500.5) or is
//!   lost to an effect (see `mana_abilities::empty_pool`, `lose_unspent_mana`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Characteristics;
use crate::types::*;

/// `Modification::Custom` (layer 6): loses all abilities except mana abilities.
pub const EXCEPT_MANA_ABILITIES: &str = "lose all abilities except mana abilities";
/// `Filter::Custom` prefix, followed by a mana type letter: could produce that type.
pub const COULD_PRODUCE: &str = "could produce:";
/// `Value::Custom` prefix, followed by a color letter: that color's mana symbols in the
/// mana costs of cards in the controller's graveyard.
pub const CHROMA_GRAVEYARD: &str = "chroma:graveyard:";
/// `Value::Custom`: the number of unlocked doors among Rooms the controller controls
/// (CR 709.5).
pub const UNLOCKED_DOORS: &str = "rooms:unlocked doors you control";
/// `PlayerModification::Custom`: losing unspent mana loses that much life.
pub const MANA_BURN: &str = "mana burn";

pub struct ManaGrammarRules;

impl KeywordRules for ManaGrammarRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_modification(
        &self,
        _g: &Game,
        name: &str,
        chars: &mut Characteristics,
        _ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        if name != EXCEPT_MANA_ABILITIES {
            return false;
        }
        chars.abilities.retain(|a| match &a.kind {
            AbilityKind::Activated(act) => act.is_mana_ability,
            AbilityKind::Triggered(t) => t.is_mana_ability,
            _ => false,
        });
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let t = name.strip_prefix(COULD_PRODUCE)?;
        let t = crate::mana::ManaType::from_letter(t.chars().next()?)?;
        Some(
            g.try_obj(id).is_some()
                && crate::mana_abilities::could_produce(g, id).contains(&t),
        )
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == UNLOCKED_DOORS {
            let n: usize = g
                .battlefield
                .iter()
                .filter(|o| {
                    g.obj(**o).controller == ctx.controller && g.obj(**o).chars.has_subtype("Room")
                })
                .map(|o| crate::rooms::unlocked(g, *o).iter().filter(|u| **u).count())
                .sum();
            return Some(n as i64);
        }
        let c = name.strip_prefix(CHROMA_GRAVEYARD)?;
        let color = Color::from_letter(c.chars().next()?)?;
        let p = ctx.controller;
        let n: usize = g
            .player(p)
            .graveyard
            .iter()
            .filter_map(|o| g.obj(*o).chars.mana_cost.as_ref())
            .map(|m| m.symbols.iter().filter(|s| s.colors().contains(color)).count())
            .sum();
        Some(n as i64)
    }
}

inventory::submit! { KeywordRegistration(&ManaGrammarRules) }

/// Whether `p` loses life for losing unspent mana ([`MANA_BURN`]).
pub fn mana_burn(g: &Game, p: PlayerId) -> bool {
    g.player(p)
        .mods
        .iter()
        .any(|m| matches!(m, PlayerModification::Custom(n) if n == MANA_BURN))
}
