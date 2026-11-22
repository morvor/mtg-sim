//! Rulings batch P019 — converge ("for each color of mana spent to cast this spell"):
//! Prismatic Ending and Sweep the Skies, where a larger X lets more colors be spent (CR
//! 107.3, 601.2f); and Obelisk of Urd's "As this artifact enters, choose a creature type."
//! (CR 614.12, 205.3m).

use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Adds one mana of each of `types` to `p`'s mana pool.
fn float(t: &mut TestGame, p: PlayerId, types: &[ManaType]) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, 1);
    }
}

use ManaType::{B, G, R, U, W};

/// P0 casts Prismatic Ending ("Converge — Exile target nonland permanent if its mana value
/// is less than or equal to the number of colors of mana spent to cast this spell.") with
/// X = `x`, paying `mana`, targeting `target`; it resolves.
fn prismatic_ending(t: &mut TestGame, x: i64, mana: &[ManaType], target: ObjectId) {
    float(t, P0, mana);
    let card = t.hand(P0, "Prismatic Ending");
    t.cast(P0, card).x(x).target(Entity::Object(target)).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
}

#[test]
fn prismatic_ending_with_a_larger_x_can_spend_more_colors() {
    cr!("107.3", "601.2f");
    ruling!(
        "Prismatic Ending",
        "You can choose any value for X. Choosing a higher value for X lets you pay more mana and thus spend more colors of mana to cast it. For example, if you choose 0 for the value of X and pay {W} to cast Prismatic Ending, you can exile a nonland permanent with mana value 0 or 1. If you choose 4 for the value of X and pay {W}{U}{B}{R}{G}, you can exile a nonland permanent with mana value 5 or less."
    );
    supported("Prismatic Ending");
    // X = 0, {W}: Llanowar Elves (1) is exiled, Grizzly Bears (2) isn't.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    prismatic_ending(&mut t, 0, &[W], elves);
    assert!(t.in_exile("Llanowar Elves"));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    prismatic_ending(&mut t, 0, &[W], bears);
    assert!(t.on_battlefield(bears));
    // X = 4, {W}{U}{B}{R}{G}: Craw Wurm (6) isn't exiled; a five-mana-value creature is.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    prismatic_ending(&mut t, 4, &[W, U, B, R, G], wurm);
    assert!(t.on_battlefield(wurm));
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    prismatic_ending(&mut t, 4, &[W, U, B, R, G], angel);
    assert!(t.in_exile("Serra Angel"));
}

#[test]
fn prismatic_endings_mana_value_check_isnt_a_targeting_restriction() {
    cr!("115.1", "608.2b", "608.2c");
    ruling!(
        "Prismatic Ending",
        "The mana value is not a targeting condition. It is checked only as the spell resolves."
    );
    // Craw Wurm (6) may be targeted with X = 0: the spell resolves and does nothing.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    float(&mut t, P0, &[W]);
    let card = t.hand(P0, "Prismatic Ending");
    let from = t.asked().len();
    t.cast(P0, card).x(0).target(Entity::Object(wurm)).go();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&Entity::Object(wurm)));
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert!(t.in_graveyard(P0, "Prismatic Ending"));
}

#[test]
fn sweep_the_skies_with_a_larger_x_can_spend_more_colors() {
    cr!("107.3", "601.2f");
    ruling!(
        "Sweep the Skies",
        "You can choose any value for X. Choosing a higher value for X lets you pay more mana and thus spend more colors of mana to cast it. For example, if you choose 0 for the value of X and pay {U}{U} to cast Sweep the Skies, you'll create one Thopter. If you choose 4 for the value of X and pay {W}{U}{U}{B}{R}{G}, you'll create five Thopters."
    );
    supported("Sweep the Skies");
    // "Converge — Create a 1/1 colorless Thopter artifact creature token with flying for
    // each color of mana spent to cast this spell."
    for (x, mana, thopters) in [(0, vec![U, U], 1), (4, vec![W, U, U, B, R, G], 5)] {
        let mut t = TestGame::new(2);
        float(&mut t, P0, &mana);
        let card = t.hand(P0, "Sweep the Skies");
        t.cast(P0, card).x(x).go();
        t.resolve_all();
        assert_eq!(
            crate::r_s01_common::with_subtype(&t, P0, "Thopter").len(),
            thopters,
            "X = {x}"
        );
    }
}

#[test]
fn obelisk_of_urds_creature_type_is_chosen_as_it_enters() {
    cr!("614.12", "614.1c", "205.3m");
    ruling!(
        "Obelisk of Urd",
        "The choice of creature type is made as Obelisk of Urd enters the battlefield. Players can't respond to this choice. The bonus starts applying immediately."
    );
    ruling!(
        "Obelisk of Urd",
        "You must choose an existing creature type."
    );
    supported("Obelisk of Urd");
    // "Convoke. As this artifact enters, choose a creature type. Creatures you control of
    // the chosen type get +2/+2." Bear chosen; the Bears are 4/4 as soon as it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 6);
    let card = t.hand(P0, "Obelisk of Urd");
    crate::r_p125_common::choose_creature_type(&mut t, P0, "Bear");
    let from = t.asked().len();
    t.cast(P0, card).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bears), (4, 4));
    // The options were exactly the creature types.
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    let creature: Vec<String> = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .cloned()
        .collect();
    assert_eq!(options[0], creature);
}
