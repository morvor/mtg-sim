//! Shuffling objects into libraries (patterns in
//! `src/oracle/patterns/control_exile_shuffle.rs`, CR 701.24).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn shuffle_cards_compile() {
    assert_compiles(&[
        "Clear the Mind",
        "Echo of Eons",
        "Game Plan",
        "Struggle // Survive",
        "Whirlpool Rider",
        "Kozilek, Butcher of Truth",
        "Blitz Hellion",
        "Deglamer",
        "Oblation",
        "Renewing Touch",
        "Memory's Journey",
        "Dwell on the Past",
    ]);
}

#[test]
fn target_player_shuffles_their_graveyard_into_their_library() {
    cr!("701.24a", "701.24d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    for _ in 0..3 {
        t.graveyard(P1, "Grizzly Bears");
    }
    let mine = t.graveyard(P0, "Grizzly Bears");
    let lib1 = t.library_size(P1);
    let hand0 = t.hand_size(P0);
    let spell = t.hand(P0, "Clear the Mind");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.library_size(P1), lib1 + 3);
    // Only that player's graveyard; then "Draw a card."
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand0 + 1);

    // CR 701.24d: with an empty graveyard, the library is still shuffled.
    let trickster = t.battlefield(P0, "Cosi's Trickster");
    let spell = t.hand(P0, "Clear the Mind");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
}

#[test]
fn each_player_shuffles_hand_and_graveyard_then_draws_seven() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    for p in [P0, P1] {
        t.hand(p, "Grizzly Bears");
        t.graveyard(p, "Grizzly Bears");
        t.graveyard(p, "Wall of Wood");
    }
    let total = |t: &TestGame, p| t.library_size(p) + t.hand_size(p) + t.graveyard_size(p);
    let before = [total(&t, P0), total(&t, P1)];
    let spell = t.hand(P0, "Echo of Eons");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.graveyard_size(P1), 0);
    // Echo of Eons itself goes to the graveyard after resolving.
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(total(&t, P1), before[1]);
}

#[test]
fn exile_this_spell_as_it_resolves() {
    cr!("608.2n");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.graveyard(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Game Plan");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.in_exile("Game Plan"));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn shuffle_hand_into_library_then_draw_that_many() {
    cr!("701.24a", "608.2c");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    let lib = t.library_size(P0);
    t.enter(P0, "Whirlpool Rider");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn shuffle_target_cards_from_a_graveyard() {
    cr!("701.24a", "115.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Wall of Wood");
    let c = t.graveyard(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Memory's Journey");
    t.cast(P0, spell)
        .target(Entity::Player(P1))
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Library(P1));
    assert_eq!(t.zone(b), Zone::Library(P1));
    assert_eq!(t.zone(c), Zone::Graveyard(P1));
    // The cards must be in the target player's graveyard.
    let mine = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Memory's Journey");
    t.cast(P0, spell)
        .target(Entity::Player(P1))
        .targets(&[Entity::Object(mine)])
        .go();
    t.resolve_all();
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));

    // "Shuffle any number of target creature cards from your graveyard into your library."
    let bear = t.graveyard(P0, "Grizzly Bears");
    let land = t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 1);
    let touch = t.hand(P0, "Renewing Touch");
    t.cast(P0, touch).targets(&[Entity::Object(bear)]).go();
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Library(P0));
    assert_eq!(t.zone(land), Zone::Graveyard(P0));
}

#[test]
fn owner_shuffles_it_into_their_library() {
    cr!("701.24a", "400.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let hand1 = t.hand_size(P1);
    let spell = t.hand(P0, "Oblation");
    t.cast(P0, spell).target(bear).go();
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Library(P1));
    // "then draws two cards": the owner draws.
    assert_eq!(t.hand_size(P1), hand1 + 2);

    // "Choose target artifact or enchantment. Its owner shuffles it into their library."
    t.lands(P0, "Forest", 2);
    let relic = t.battlefield(P1, "Ornithopter");
    let deglamer = t.hand(P0, "Deglamer");
    t.cast(P0, deglamer).target(relic).go();
    t.resolve_all();
    assert_eq!(t.zone(relic), Zone::Library(P1));
}

#[test]
fn put_into_a_graveyard_from_anywhere_shuffles_the_graveyard() {
    cr!("603.6c", "701.24a");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let kozilek = t.hand(P0, "Kozilek, Butcher of Truth");
    let lib = t.library_size(P0);
    t.g.discard(P0, kozilek, None);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), lib + 2);
}

#[test]
fn owner_shuffles_it_at_the_beginning_of_the_end_step() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    let hellion = t.battlefield(P0, "Blitz Hellion");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(hellion), Zone::Library(P0));
}
