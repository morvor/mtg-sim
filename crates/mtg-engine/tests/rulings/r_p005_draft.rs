//! Rulings batch P005 — Lore Seeker ("Reveal this card as you draft it. After you draft
//! this card, you may add a booster pack to the draft."): a card that functions during the
//! draft (CR 905.2), whose drafter supplies the booster pack added.

use crate::r_s01_common::supported;
use mtg_engine::card::{card, CardDef};
use mtg_engine::draft::{Draft, DraftStyle};
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::Arc;

fn pack(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| card(n)).collect()
}

#[test]
fn lore_seekers_drafter_provides_a_booster_pack_from_any_set() {
    cr!("905.2", "905.1a");
    ruling!(
        "Lore Seeker",
        "The booster pack can be from any Magic: The Gathering set, but you must provide it. Consult your co-conspirators for guidance."
    );
    supported("Lore Seeker");
    let mut d = Draft::new(
        vec![
            vec![pack(&["Lore Seeker", "Opt", "Shock"])],
            vec![pack(&["Grizzly Bears", "Hill Giant", "Memnite"])],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    // Only the player who drafted Lore Seeker adds a pack, and it's the one they provide:
    // nothing is generated for them (an empty "pack" isn't one).
    assert!(d.add_booster(P1, pack(&["Divination"])).is_err());
    assert!(d.add_booster(P0, vec![]).is_err());
    // Cards from sets of any era (Alpha, Kaladesh, Dominaria United, Bloomburrow) and
    // none of the draft's own packs.
    let added = pack(&[
        "Lightning Bolt",
        "Electrosiphon",
        "Sprouting Goblin",
        "Archmage's Newt",
    ]);
    d.add_booster(P0, added).unwrap();
    d.pick(P1, &[0]).unwrap();
    let in_front: Vec<&str> = d.pack(P0).iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        in_front,
        [
            "Lightning Bolt",
            "Electrosiphon",
            "Sprouting Goblin",
            "Archmage's Newt"
        ]
    );
    d.pick(P0, &[1]).unwrap();
    assert!(d.pool(P0).iter().any(|c| c.name == "Electrosiphon"));
}
