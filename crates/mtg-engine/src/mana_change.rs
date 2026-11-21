//! Changes to the mana a permanent is tapped for (CR 106.12b, 614.1a): the replacement
//! effects of [`crate::ability::ReplacementAction::ManaChange`]. "If a land is tapped for two or more
//! mana, it produces {C} instead of any other type and amount." (Damping Sphere: only when
//! the mana ability would produce at least that much, CR 106.12), "If a basic land you
//! control is tapped for mana, it produces mana of a color of your choice instead of any
//! other type." (Pulse of Llanowar: the amount is unchanged, and the player tapping it
//! chooses one color for that mana each time), "it produces one mana of a color of your
//! choice instead of any other type and amount" (Harvest Mage), "it produces colorless
//! mana instead of white mana" (Quarum Trench Gnomes: only its white mana changes).

use crate::ability::{ManaChange, ManaChangeTo};
use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::ManaType;
use crate::types::PlayerId;

const COLORS: [ManaType; 5] = [
    ManaType::W,
    ManaType::U,
    ManaType::B,
    ManaType::R,
    ManaType::G,
];

/// Whether the change applies to mana of this type.
fn affects(c: &ManaChange, t: ManaType) -> bool {
    c.only.as_ref().is_none_or(|o| o.contains(&t))
}

/// The types the mana changes to: one type, or each color (a choice).
pub fn targets(c: &ManaChange) -> Vec<ManaType> {
    match c.to {
        ManaChangeTo::Type(t) => vec![t],
        ManaChangeTo::ColorOfChoice => COLORS.to_vec(),
        ManaChangeTo::ChosenColor => vec![],
    }
}

/// Applies the change to the produced types, with `to` the type the changed mana becomes.
fn apply_with(c: &ManaChange, types: &[ManaType], to: ManaType) -> Vec<ManaType> {
    if (types.len() as u32) < c.min_amount || !types.iter().any(|t| affects(c, *t)) {
        return types.to_vec();
    }
    match c.amount {
        // "instead of any other type and amount": all of it is replaced.
        Some(n) => vec![to; n as usize],
        None => types
            .iter()
            .map(|t| if affects(c, *t) { to } else { *t })
            .collect(),
    }
}

/// The types produced, without asking for a choice: every possible type is listed for a
/// choice of color (for what a permanent "could produce", CR 106.7, and payment hints).
pub fn apply_any(c: &ManaChange, types: &[ManaType]) -> Vec<ManaType> {
    match c.to {
        ManaChangeTo::Type(t) => apply_with(c, types, t),
        ManaChangeTo::ChosenColor => types.to_vec(),
        ManaChangeTo::ColorOfChoice => {
            if apply_with(c, types, ManaType::W) == types {
                return types.to_vec();
            }
            COLORS.to_vec()
        }
    }
}

/// Applies the change as the permanent is tapped for mana: `p`, the player tapping it,
/// chooses the color if there's a choice (a payment's hint first).
pub fn apply(
    g: &mut Game,
    p: PlayerId,
    ctx: &Ctx,
    c: &ManaChange,
    types: &[ManaType],
) -> Vec<ManaType> {
    let to = match c.to {
        ManaChangeTo::Type(t) => t,
        ManaChangeTo::ChosenColor => return types.to_vec(),
        ManaChangeTo::ColorOfChoice => {
            if apply_with(c, types, ManaType::W) == types
                && apply_with(c, types, ManaType::U) == types
            {
                return types.to_vec();
            }
            g.choose_mana_color(p, ctx, &COLORS)
        }
    };
    apply_with(c, types, to)
}

/// The units of mana (each with the types it could be) after the change, for planning a
/// payment (see `mana_abilities::replaced_units`).
pub fn apply_units(c: &ManaChange, units: Vec<Vec<ManaType>>) -> Vec<Vec<ManaType>> {
    let affected = |u: &Vec<ManaType>| u.iter().any(|t| affects(c, *t));
    if (units.len() as u32) < c.min_amount || !units.iter().any(affected) {
        return units;
    }
    let to = targets(c);
    if to.is_empty() {
        return units;
    }
    match c.amount {
        Some(n) => vec![to; n as usize],
        None => units
            .into_iter()
            .map(|u| if affected(&u) { to.clone() } else { u })
            .collect(),
    }
}
