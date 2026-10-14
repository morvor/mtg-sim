//! Villainous choices whose options are optional instructions ("or you may put ... onto
//! the battlefield", "or you may cast a spell ...") or name "that opponent" (CR 701.55),
//! and "Each opponent who doesn't control [something] loses the game." (CR 104.3e).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn option(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn dr_eggman_lets_you_put_a_robot_onto_the_battlefield() {
    cr!("701.55a");
    assert_supported(&["Dr. Eggman"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dr. Eggman");
    t.hand(P0, "Smuggler's Copter");
    option(&mut t, P1, 1);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Smuggler's Copter").len(), 1);
    // The other option: the opponent discards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dr. Eggman");
    t.hand(P1, "Grizzly Bears");
    option(&mut t, P1, 0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn missy_deals_damage_to_that_opponent_or_draws() {
    cr!("701.55a");
    assert_supported(&["Missy"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Missy");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    option(&mut t, P1, 0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn great_intelligences_plan_may_cast_a_spell_for_free() {
    cr!("701.55a");
    assert_supported(&["Great Intelligence's Plan"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.lands(P0, "Swamp", 1);
    let plan = t.hand(P0, "Great Intelligence's Plan");
    let bears = t.hand(P0, "Grizzly Bears");
    option(&mut t, P1, 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, plan).target(P1).go();
    t.resolve_all();

    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn nicol_bolas_ultimate_spares_opponents_with_a_legendary_creature() {
    cr!("104.3e", "606.6");
    // (Its +1 ability is another construct; the -8 compiles.)
    assert!(!card("Nicol Bolas, Dragon-God")
        .unsupported_text()
        .iter()
        .any(|t| t.contains("loses the game")));
    let mut t = TestGame::new(3);
    let bolas = t.battlefield(P0, "Nicol Bolas, Dragon-God");
    t.g.add_counters(Entity::Object(bolas), "loyalty", 4, None);
    t.battlefield(P2, "Isamaru, Hound of Konda");
    // The -8 is its last activated ability (after the -3).
    t.activate(P0, bolas, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert!(!t.has_lost(P2));
}

#[test]
fn sycorax_commander_discard_and_draw_one_fewer_or_damage() {
    cr!("701.55a", "107.1b");
    assert_supported(&["Sycorax Commander"]);
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    option(&mut t, P1, 0);
    t.enter(P0, "Sycorax Commander");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.graveyard_size(P1), 3);
    // The other option: damage equal to the cards in their hand.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    option(&mut t, P1, 1);
    t.enter(P0, "Sycorax Commander");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn this_is_how_it_ends_shuffles_another_creature_they_own() {
    cr!("701.55a");
    assert_supported(&["This Is How It Ends"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "This Is How It Ends");
    option(&mut t, P1, 1);
    let lib = t.library_size(P1);
    t.cast(P0, spell).target(a).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.library_size(P1), lib + 2);
    assert_eq!(t.life(P1), 20);
}
