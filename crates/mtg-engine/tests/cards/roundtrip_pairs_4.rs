//! roundtrip-pairs-4 (docs/agents/items/roundtrip-pairs-4.md): untriaged round-trip
//! mismatches from the long tail (the two-card clusters).

use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts the real Aura or Equipment `name` onto the battlefield under `p`'s control attached
/// to `to`.
fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(t.g.attach(id, Entity::Object(to)), "{name} couldn't be attached");
    t.g.recompute();
    id
}

#[test]
fn seven_dwarves_counts_creatures_named_seven_dwarves_not_ones_sharing_another_name() {
    cr!("201.2", "201.2a", "201.5");
    // "~ gets +1/+1 for each other creature named ~ you control." The creatures named
    // Seven Dwarves, not the ones with the same name as this one: with Spy Kit ("has all
    // names of nonlegendary creature cards") it has Grizzly Bears' name too, but Grizzly
    // Bears isn't named Seven Dwarves.
    let mut t = TestGame::new(2);
    let dwarves = t.battlefield(P0, "Seven Dwarves");
    t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Spy Kit", dwarves);
    assert_eq!(t.pt(dwarves), (3, 3), "Spy Kit's +1/+1 only");
    // Another Seven Dwarves counts.
    t.battlefield(P0, "Seven Dwarves");
    t.g.recompute();
    assert_eq!(t.pt(dwarves), (4, 4));
}

#[test]
fn lake_of_the_dead_sacrifices_a_swamp_without_asking() {
    cr!("614.1c", "614.12", "608.2c");
    ruling!(
        "Scorched Ruins",
        "If you don't sacrifice the lands, Scorched Ruins never enters"
    );
    // "If ~ would enter, sacrifice a Swamp instead. If you do, put ~ onto the battlefield.
    // If you don't, put it into its owner's graveyard." An instruction, not an offer
    // (Mox Diamond's "you may discard a land card instead" is one): with a Swamp, it's
    // sacrificed and Lake of the Dead enters whatever the player would answer.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let lake = t.hand(P0, "Lake of the Dead");
    t.answer_yes(P0, false);
    t.play_land(P0, lake).unwrap();
    t.settle();
    assert_eq!(t.named_on_battlefield("Lake of the Dead").len(), 1);
    assert!(t.in_graveyard(P0, "Swamp"));
    // Without a Swamp, it never enters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let lake = t.hand(P0, "Lake of the Dead");
    t.play_land(P0, lake).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Lake of the Dead"));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn lotus_vale_with_one_untapped_land_sacrifices_it_and_never_enters() {
    cr!("614.12", "609.3");
    interp!("enter-sacrifice-instead-partial");
    // "Sacrifice two untapped lands instead. If you do, ...": with only one, as much as
    // possible is done (it's sacrificed), but not all of it, so Lotus Vale goes to the
    // graveyard.
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 2);
    t.g.tap(forests[0]);
    let vale = t.hand(P0, "Lotus Vale");
    t.play_land(P0, vale).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Lotus Vale"));
    assert_eq!(t.named_on_battlefield("Forest"), vec![forests[0]]);
}

#[test]
fn crackdown_construct_ignores_abilities_of_cards_in_hand() {
    cr!("109.2", "602.1", "702.29a");
    // "Whenever you activate an ability of an artifact or creature that isn't a mana
    // ability, ~ gets +1/+1 until end of turn": of an artifact or creature permanent
    // (CR 109.2), not a creature card's cycling from a hand.
    let mut t = TestGame::new(2);
    let construct = t.battlefield(P0, "Crackdown Construct");
    t.lands(P0, "Forest", 3);
    t.library_top(P0, "Island");
    let tusker = t.hand(P0, "Krosan Tusker");
    t.activate(P0, tusker, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Krosan Tusker"), "cycled");
    assert_eq!(t.pt(construct), (2, 2));
    // A creature's ability on the battlefield.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.g.recompute();
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.pt(construct), (3, 3));
}
