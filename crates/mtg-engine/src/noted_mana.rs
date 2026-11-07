//! Noted mana (CR 607.2e): "Note the type of mana spent to pay this activation cost." with
//! "Add one mana of ~'s last noted type." (Jeweled Amulet), and "note the type and amount
//! of mana spent to pay this activation cost" with "Add ~'s last noted type and amount of
//! mana." (Ice Cauldron).
//!
//! * [`crate::ability::Effect::NoteManaSpent`] notes, as the ability resolves, the mana spent to pay its
//!   cost (recorded on the ability on the stack as it was paid), for the abilities linked
//!   to it: the note is kept on the source object (a new object after a zone change has
//!   none, CR 400.7), and noting again replaces it ("the last noted"). Only mana spent on
//!   this ability counts (Ice Cauldron's ruling: it doesn't see the value of X). With one
//!   type to note and mana of several types spent (a cost increase), its controller
//!   chooses the type (interpretation `jeweled-amulet-several-types`).
//! * [`crate::ability::ManaProduction::Noted`] adds the noted mana: one mana of the noted type, or each
//!   noted mana. With nothing noted, the ability adds no mana (CR 607.5a; Jeweled
//!   Amulet's ruling: a charge counter put there some other way gives no mana).

use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::ManaType;

/// [`crate::ability::Effect::NoteManaSpent`].
pub fn note(g: &mut Game, amount: bool, ctx: &mut Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    let spent: Vec<ManaType> = ctx
        .stack_obj
        .and_then(|s| g.try_obj(s))
        .and_then(|o| o.stack.as_deref())
        .map(|si| si.cast.mana_spent.clone())
        .unwrap_or_default();
    if spent.is_empty() {
        // "X can be zero. This places a zero mana counter on the Cauldron." (nothing is
        // left to add later).
        if amount {
            set(g, src, ctx.link, vec![]);
        }
        return;
    }
    let noted = if amount {
        spent
    } else {
        let types: Vec<ManaType> = ManaType::ALL
            .iter()
            .copied()
            .filter(|t| spent.contains(t))
            .collect();
        let t = if types.len() > 1 {
            let names = types.iter().map(|t| format!("{t:?}")).collect();
            let i = g.ask_option(
                ctx.controller,
                Some(src),
                "Note which type of mana spent?",
                names,
            );
            types[i.min(types.len() - 1)]
        } else {
            types[0]
        };
        vec![t]
    };
    g.log(|g| format!("{} notes {noted:?}", g.describe(src)));
    set(g, src, ctx.link, noted);
}

fn set(g: &mut Game, src: crate::types::ObjectId, link: u16, mana: Vec<ManaType>) {
    g.objects[src.0 as usize]
        .linked_choices
        .entry(link)
        .or_default()
        .mana = mana;
}

/// [`crate::ability::ManaProduction::Noted`]: the mana the ability adds.
pub fn produced(g: &Game, amount: bool, ctx: &Ctx) -> Vec<ManaType> {
    let noted = g
        .linked_choice(ctx)
        .map(|c| c.mana.clone())
        .unwrap_or_default();
    if amount {
        noted
    } else {
        noted.into_iter().take(1).collect()
    }
}
