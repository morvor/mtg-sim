//! Timing permissions for the next spell ("The next sorcery spell you cast this turn can
//! be cast as though it had flash.") and restrictions on casting (CR 101.2, 601.3):
//! "Players can cast spells only during their own turns.", "Each player can't cast more
//! than one noncreature spell each turn.", "Each opponent who controls more creatures than
//! you can't cast creature spells.", "You can't play lands or cast spells from your
//! hand.", "Players can't cast spells from graveyards ...", "You can't cast ~ unless an
//! opponent lost life this turn."

use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn try_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> bool {
    let c = t.g.current(card);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, CastMethod::Normal).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

#[test]
fn quicken_lets_the_next_sorcery_be_cast_as_though_it_had_flash() {
    cr!("601.3b", "611.2f");
    ruling!(
        "Quicken",
        "As soon as you actually cast a sorcery, you lose this capability."
    );
    assert_supported("Quicken");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    let quicken = t.hand(P0, "Quicken");
    let spike = t.hand(P0, "Lava Spike");
    let spike2 = t.hand(P0, "Lava Spike");
    t.advance_to(P1, Step::Upkeep);
    // Not before Quicken.
    assert!(!try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert!(try_cast(&mut t, P0, quicken, &[]));
    assert!(try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
    // Used up by that sorcery.
    assert!(!try_cast(&mut t, P0, spike2, &[Entity::Player(P1)]));
}

#[test]
fn dosan_players_cast_spells_only_during_their_own_turns() {
    cr!("101.2", "601.3");
    ruling!(
        "Dosan the Falling Leaf",
        "only stops players from casting spells"
    );
    assert_supported("Dosan the Falling Leaf");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dosan the Falling Leaf");
    t.lands(P1, "Mountain", 2);
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let mine = t.hand(P0, "Lightning Bolt");
    // P0's turn: P1 can't cast an instant; P0 can.
    assert!(!try_cast(&mut t, P1, bolt, &[Entity::Player(P0)]));
    assert!(try_cast(&mut t, P0, mine, &[Entity::Player(P1)]));
    t.advance_to(P1, Step::Upkeep);
    assert!(try_cast(&mut t, P1, bolt, &[Entity::Player(P0)]));
}

#[test]
fn deafening_silence_one_noncreature_spell_each_turn() {
    cr!("601.3");
    ruling!(
        "Deafening Silence",
        "Players may cast any number of creature spells plus one noncreature spell each turn."
    );
    assert_supported("Deafening Silence");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Deafening Silence");
    t.lands(P0, "Mountain", 6);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    let goblin = t.hand(P0, "Raging Goblin");
    let goblin2 = t.hand(P0, "Raging Goblin");
    assert!(try_cast(&mut t, P0, goblin, &[]));
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert!(!try_cast(&mut t, P0, bolt2, &[Entity::Player(P1)]));
    assert!(try_cast(&mut t, P0, goblin2, &[]));
}

#[test]
fn ethersworn_canonist_one_nonartifact_spell_counting_earlier_ones() {
    cr!("601.3");
    ruling!(
        "Ethersworn Canonist",
        "takes into account spells that were cast earlier in the turn before Ethersworn Canonist entered the battlefield"
    );
    assert_supported("Ethersworn Canonist");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    let ornithopter = t.hand(P0, "Ornithopter");
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    t.battlefield(P1, "Ethersworn Canonist");
    assert!(!try_cast(&mut t, P0, bolt2, &[Entity::Player(P1)]));
    // Artifact spells: any number.
    assert!(try_cast(&mut t, P0, ornithopter, &[]));
}

#[test]
fn ward_of_bones_opponents_with_more_creatures_cant_cast_creature_spells() {
    cr!("601.3", "305.2");
    assert_supported("Ward of Bones");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ward of Bones");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    let forest = t.hand(P1, "Forest");
    t.advance_to(P1, Step::PrecombatMain);
    // One creature to none, three lands to none.
    assert!(!try_cast(&mut t, P1, bears, &[]));
    assert!(t.play_land(P1, forest).is_err());
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    assert!(try_cast(&mut t, P1, bears, &[]));
    t.play_land(P1, forest)
        .expect("as many lands as their controller");
}

#[test]
fn experimental_frenzy_cant_play_cards_from_your_hand() {
    cr!("601.3", "305.2");
    ruling!(
        "Experimental Frenzy",
        "You can’t cast spells or play lands from your hand"
    );
    assert_supported("Experimental Frenzy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Experimental Frenzy");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let mountain = t.hand(P0, "Mountain");
    let top = t.library_top(P0, "Lightning Bolt");
    assert!(!try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert!(t.play_land(P0, mountain).is_err());
    // From the top of the library.
    assert!(try_cast(&mut t, P0, top, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn ashes_of_the_abhorrent_no_spells_from_graveyards() {
    cr!("601.3");
    ruling!(
        "Ashes of the Abhorrent",
        "doesn’t stop players from playing land cards from the graveyard"
    );
    assert_supported("Ashes of the Abhorrent");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ashes of the Abhorrent");
    t.lands(P0, "Swamp", 4);
    let legion = t.graveyard(P0, "Their Number Is Legion");
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, legion, CastMethod::Normal).is_err());
    let forest = t.graveyard(P0, "Forest");
    t.battlefield(P0, "Zask, Skittering Swarmlord");
    t.play_land(P0, forest)
        .expect("a land from the graveyard is played, not cast");
}

#[test]
fn rakdos_cant_be_cast_unless_an_opponent_lost_life_this_turn() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Swamp", 2);
    let rakdos = t.hand(P0, "Rakdos, Lord of Riots");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!try_cast(&mut t, P0, rakdos, &[]));
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert!(try_cast(&mut t, P0, rakdos, &[]));
    assert_eq!(t.named_on_battlefield("Rakdos, Lord of Riots").len(), 1);
}
